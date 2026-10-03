//! Source d'inputs `InputSource::Bot` (T2.11) : [`BotAssignments`] (profil par joueur local),
//! le système [`read_bot_inputs`] (schedule `ReadInputs`, à côté de `read_local_inputs`) et
//! [`BotsPlugin`] qui les enregistre.
//!
//! ## RNG et rollback (à lire avant de toucher ce fichier)
//!
//! `read_bot_inputs` tourne dans `ReadInputs`, **pas** dans `GgrsSchedule` : bevy_ggrs
//! l'exécute une seule fois par frame réelle (`run_synctest`/`run_p2p`,
//! `crates/bevy_ggrs-0.22.0/src/schedule_systems.rs`), jamais rejoué pendant une resimulation
//! de rollback (seul `AdvanceWorld`/`GgrsSchedule` l'est). Les inputs qu'il décide sont ensuite
//! échangés comme n'importe quel input GGRS : une fois décidés, ils sont fixes, resimulés tels
//! quels.
//!
//! Le flux RNG `"bots"` vit dans [`RngStreams`], une ressource **rollback** (checksum + trace,
//! `crates/game/src/core.rs`). Comme ce système n'est pas rejoué, un rollback qui restaure un
//! snapshot antérieur restaure aussi l'état du flux `"bots"` à ce qu'il était à ce moment-là :
//! l'avancement fait par cette frame (avant le rollback) est perdu, et le flux peut « dériver »
//! par rapport à une exécution sans rollback (certains tirages sont répétés ou sautés). Ça ne
//! cause **pas** de desync : rien dans `GgrsSchedule` ne lit ce flux, et l'input décidé pour
//! chaque frame reste fixé une fois choisi, resimulé identique à lui-même. En synctest à
//! processus unique (scénarios, `alacod-sim`), le schéma de rollback d'une frame donnée est
//! déterministe d'une exécution à l'autre (`check_distance` fixe, pas de réseau) : la trace
//! reste reproductible d'un run à l'autre du même scénario, malgré cette dérive.
//!
//! **Le flux n'est créé dans `RngStreams` que s'il est réellement consommé** (voir
//! [`decide_with_lazy_rng`]) : aucun des trois profils v0 n'en tire (`crate::decide`), donc
//! `RngStreams` (checksummée) reste identique à ce qu'elle serait sans bots tant que seuls ces
//! profils sont utilisés. C'est nécessaire, pas juste une optimisation : un run de bots
//! enregistré (`--save-scenario`) se rejoue en `Scripted` (`crates/scenario/tests/bots.rs`), un
//! mode qui ne touche jamais `BotAssignments` ni le flux "bots" ; si le run original avait créé
//! l'entrée "bots" (même sans jamais tirer dedans), sa présence à elle seule aurait changé la
//! trace, et le replay aurait divergé dès la première frame sans qu'aucun gameplay n'ait changé.
//! Le jour où un profil consommera vraiment ce flux (« aléatoire seedé », plan §9.7), cette
//! garantie de rejouabilité en `Scripted` ne tiendra plus pour ce profil-là : la dérive documentée
//! ci-dessus deviendra observable dans la trace, pas seulement en théorie.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bevy_fixed::rng::{fnv1a, RngStreams, RollbackRng};
use bevy_ggrs::{LocalInputs, LocalPlayers, ReadInputs, Rollback};
use game::character::enemy::Enemy;
use game::character::health::Health;
use game::character::player::input::{read_local_inputs, BoxInput};
use game::character::player::jjrs::PeerConfig;
use game::character::player::Player;
use game::recording::record_local_inputs;
use game::replay::BotProfile;
use game::waves::WaveState;
use game::weapons::{WeaponInventory, WeaponModesState, WeaponState};
use map::game::entity::map::window::WindowHealth;
use sim_core::kinds::{KindDecl, KindRegistry};
use utils::net_id::GgrsNetId;
use utils::order_iter;

use crate::decide::decide;
use crate::view::{nearest_by_net_id, BotView, EnemyView, WindowView};

/// Profil de bot par joueur local (handle GGRS). Ressource **ordinaire**, pas rollback, pas
/// dans le checksum GGRS (assignée une fois avant la partie par le runner de scénario ou
/// `alacod-sim`, jamais modifiée en jeu) : les traces des scénarios existants (aucun bot) ne
/// changent pas. `BTreeMap` pour un ordre stable si elle est un jour itérée.
#[derive(Resource, Debug, Clone, Default)]
pub struct BotAssignments(pub BTreeMap<usize, BotProfile>);

/// Enregistre [`BotAssignments`] (vide par défaut ; un appelant l'assigne ensuite avec
/// `insert_resource`) et ajoute [`read_bot_inputs`] au schedule `ReadInputs`. Peut être ajouté
/// sans risque à n'importe quelle App (`BotAssignments` vide ⇒ `read_bot_inputs` ne fait rien) :
/// c'est ce que fait `crates/scenario::runner::build_app` pour tous les scénarios.
pub struct BotsPlugin;

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BotAssignments>();

        // Vocabulaire (docs/conventions.md §4, étape 2) : les noms de profils, pour un futur
        // lint/diagnostic qui voudrait les valider (aucun aujourd'hui : les scénarios ne sont
        // pas lus par `content::lint`, seul `game.ron` l'est).
        app.register_kinds([
            KindDecl::new("bot", BotProfile::Immobile.name()),
            KindDecl::new("bot", BotProfile::Fonceur.name()),
            KindDecl::new("bot", BotProfile::Prudent.name()),
        ]);

        app.add_systems(
            ReadInputs,
            read_bot_inputs
                .after(read_local_inputs)
                .before(record_local_inputs),
        );
    }
}

/// Construit la [`BotView`] de chaque joueur local présent dans [`BotAssignments`] (requêtes
/// triées par `GgrsNetId`, CLAUDE.md règle 2/4) et appelle [`decide`], en remplaçant son entrée
/// dans `LocalInputs` (déjà posée par `read_local_inputs`, qui tourne juste avant). Un joueur
/// local absent de `BotAssignments` garde l'input que `read_local_inputs` lui a donné (Devices,
/// Scripted, Neutral...) : les deux sources cohabitent dans un même scénario.
#[allow(clippy::too_many_arguments)]
pub fn read_bot_inputs(
    local_players: Res<LocalPlayers>,
    assignments: Option<Res<BotAssignments>>,
    local_inputs: Option<ResMut<LocalInputs<PeerConfig>>>,
    mut rng_streams: ResMut<RngStreams>,
    wave: Option<Res<WaveState>>,
    floor_state: Option<Res<run::FloorState>>,
    players: Query<
        (
            &GgrsNetId,
            &Player,
            &FixedTransform3D,
            &Health,
            &WeaponInventory,
        ),
        With<Rollback>,
    >,
    weapons: Query<(&WeaponState, &WeaponModesState)>,
    enemies: Query<(&GgrsNetId, &FixedTransform3D), (With<Enemy>, With<Rollback>)>,
    windows: Query<(&GgrsNetId, &FixedTransform3D, &WindowHealth), With<Rollback>>,
) {
    let Some(assignments) = assignments else {
        return;
    };
    if assignments.0.is_empty() {
        return;
    }
    // `read_local_inputs` tourne juste avant (voir `BotsPlugin`) et pose toujours cette
    // ressource ; `Option` défensif seulement (même style que `record_local_inputs`).
    let Some(mut local_inputs) = local_inputs else {
        return;
    };

    let wave_number = wave.map_or(0, |w| w.current_wave);
    let portal = floor_state
        .filter(|state| state.portal_open)
        .and_then(|state| state.anchor_vec());
    let enemies_sorted = order_iter!(enemies);
    let windows_sorted = order_iter!(windows);

    // `_net_id` : nécessaire en première position pour `order_iter!` (tri déterministe des
    // joueurs avant de consommer le flux RNG "bots"), pas utilisé ensuite (même convention que
    // `move_characters`, `crates/game/src/character/player/input.rs`).
    for (_net_id, player, transform, health, inventory) in order_iter!(players) {
        if !local_players.0.contains(&player.handle) {
            continue;
        }
        let Some(&profile) = assignments.0.get(&player.handle) else {
            continue;
        };

        let position = transform.translation.truncate();

        let nearest_enemy =
            nearest_by_net_id(enemies_sorted.iter().map(|(id, enemy_transform)| {
                let enemy_position = enemy_transform.translation.truncate();
                let distance = position.distance(&enemy_position);
                (
                    id.0,
                    distance,
                    EnemyView {
                        position: enemy_position,
                        distance,
                    },
                )
            }));

        let nearest_window = nearest_by_net_id(windows_sorted.iter().map(
            |(id, window_transform, window_health)| {
                let window_position = window_transform.translation.truncate();
                let distance = position.distance(&window_position);
                (
                    id.0,
                    distance,
                    WindowView {
                        position: window_position,
                        health: window_health.current,
                        distance,
                        repairable: window_health.current < window_health.max,
                    },
                )
            },
        ));

        let ammo = inventory
            .weapons
            .get(inventory.active_weapon_index)
            .and_then(|(entity, _)| weapons.get(*entity).ok())
            .and_then(|(state, modes)| modes.modes.get(&state.active_mode).map(|m| m.mag_ammo))
            .unwrap_or(0);

        let view = BotView {
            position,
            health: health.current,
            health_max: health.max,
            ammo,
            wave: wave_number,
            nearest_enemy,
            nearest_window,
            portal,
        };

        let input = decide_with_lazy_rng(&mut rng_streams, profile, &view);

        local_inputs.0.insert(player.handle, input);
    }
}

/// Nom du flux RNG des bots dans [`RngStreams`] (voir la doc du module).
const RNG_STREAM: &str = "bots";

/// Appelle [`decide`] sans créer l'entrée `"bots"` dans [`RngStreams`] (ressource checksummée,
/// CLAUDE.md règle 6) si elle n'a pas réellement servi : lit sa valeur courante sans la créer
/// (`RngStreams::get`, reconstruit la même graine initiale que `RngStreams::get_mut` si le flux
/// n'existe pas encore), et n'écrit dans `RngStreams` (créant l'entrée si besoin) que si `decide`
/// a changé cette valeur. Voir la doc du module pour pourquoi c'est nécessaire, pas cosmétique.
fn decide_with_lazy_rng(
    rng_streams: &mut RngStreams,
    profile: BotProfile,
    view: &BotView,
) -> BoxInput {
    let before = rng_streams.get(RNG_STREAM).copied().unwrap_or_else(|| {
        let seed = fnv1a(RNG_STREAM.as_bytes()) as u32 ^ rng_streams.run_seed;
        RollbackRng::new(seed)
    });
    let mut stream = before;
    let input = decide(profile, view, &mut stream);
    if stream != before {
        *rng_streams.get_mut(RNG_STREAM) = stream;
    }
    input
}
