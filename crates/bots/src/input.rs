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

use crate::arms::{self, WeaponChoices, WeaponView};
use crate::stuck::EnemyMoves;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
use bevy_fixed::rng::{fnv1a, RngStreams, RollbackRng};
use bevy_ggrs::{LocalInputs, LocalPlayers, ReadInputs, Rollback};
use game::character::config::{CharacterConfig, CharacterConfigHandles};
use game::character::enemy::Enemy;
use game::character::health::Health;
use game::character::player::input::{read_local_inputs, BoxInput};
use game::character::player::jjrs::PeerConfig;
use game::character::player::Player;
use game::global_asset::GlobalAsset;
use game::recording::record_local_inputs;
use game::replay::BotProfile;
use game::waves::WaveState;
use game::weapons::{WeaponInventory, WeaponModesState, WeaponPickup, WeaponState, WeaponsConfig};
use map::game::entity::map::window::WindowHealth;
use sim_core::kinds::{KindDecl, KindRegistry};
use utils::net_id::GgrsNetId;
use utils::order_iter;

use crate::decide::decide;
use crate::view::{
    nearest_by_net_id, projectile_views, BotView, EnemyView, ProjectileView, ReviveView, WindowView,
};
use combat::collider::{Collider, ColliderShape};
use game::character::enemy::ai::navigation::AgentBody;
use game::collider::{Wall, Window};
use map::game::entity::map::door::DoorComponent;

use crate::navigation::{DirectNavigation, Rect};

/// Rayon d'arrivée au portail pour la navigation (le portail se franchit à 24).
const PORTAL_REACH: Fixed = Fixed::from_bits(16 << 16);
use combat::weapons::Bullet;
use sim_core::team::Team;

/// Profil de bot par joueur local (handle GGRS). Ressource **ordinaire**, pas rollback, pas
/// dans le checksum GGRS (assignée une fois avant la partie par le runner de scénario ou
/// `alacod-sim`, jamais modifiée en jeu) : les traces des scénarios existants (aucun bot) ne
/// changent pas. `BTreeMap` pour un ordre stable si elle est un jour itérée.
#[derive(Resource, Debug, Clone, Default)]
pub struct BotAssignments(pub BTreeMap<usize, BotProfile>);

/// Compteurs des bots (T1.14), **hors rollback et hors simulation** : incrémentés dans
/// `ReadInputs` (une fois par frame décidée), lus par `alacod-sim` en fin de partie.
#[derive(Resource, Debug, Clone, Default)]
pub struct BotStats {
    /// Frames où l'esquive a remplacé le déplacement d'au moins un bot `prudent`.
    pub dodges: u32,
}

/// Enregistre [`BotAssignments`] (vide par défaut ; un appelant l'assigne ensuite avec
/// `insert_resource`) et ajoute [`read_bot_inputs`] au schedule `ReadInputs`. Peut être ajouté
/// sans risque à n'importe quelle App (`BotAssignments` vide ⇒ `read_bot_inputs` ne fait rien) :
/// c'est ce que fait `crates/scenario::runner::build_app` pour tous les scénarios.
pub struct BotsPlugin;

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BotAssignments>();
        app.init_resource::<BotStats>();

        // Vocabulaire (docs/conventions.md §4, étape 2) : les noms de profils, pour un futur
        // lint/diagnostic qui voudrait les valider (aucun aujourd'hui : les scénarios ne sont
        // pas lus par `content::lint`, seul `game.ron` l'est).
        app.register_kinds([
            KindDecl::new("bot", BotProfile::Immobile.name()),
            KindDecl::new("bot", BotProfile::Fonceur.name()),
            KindDecl::new("bot", BotProfile::Prudent.name()),
            KindDecl::new("bot", BotProfile::Chasseur.name()),
            KindDecl::new("bot", BotProfile::Acheteur.name()),
            KindDecl::new("bot", BotProfile::Explorateur.name()),
        ]);

        app.init_resource::<crate::navigation::BotNavigation>();
        app.init_resource::<DirectNavigation>();
        app.init_resource::<WeaponChoices>();
        app.init_resource::<EnemyMoves>();
        app.init_resource::<crate::explorer::ExplorerOptions>();
        app.init_resource::<crate::explorer::ExplorerVisits>();
        app.init_resource::<crate::explorer::RockNavigation>();
        app.init_resource::<crate::explorer::ExplorerJitter>();
        app.add_systems(
            OnEnter(game::session::AppState::InGame),
            |mut choices: ResMut<WeaponChoices>,
             mut moves: ResMut<EnemyMoves>,
             mut jitter: ResMut<crate::explorer::ExplorerJitter>| {
                choices.0.clear();
                moves.0.clear();
                // Les identifiants de buts et les frames repartent de zéro avec la partie : les buts écartés d'une run
                // ne valent pas pour la suivante.
                *jitter = Default::default();
            },
        );
        app.add_systems(
            ReadInputs,
            crate::hunter::read_hunter_inputs
                .after(read_bot_inputs)
                .before(record_local_inputs),
        );
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
            Option<&Collider>,
            Option<&combat::inventory::AmmoReserves>,
            Option<&combat::actors::Velocity>,
            Option<&combat::downed::Downed>,
            Option<&CharacterConfigHandles>,
            // Suivi Q17 : visée à quatre directions (absent = libre).
            Option<&sim_core::aim::AimMode>,
        ),
        With<Rollback>,
    >,
    // T1.14 : projectiles (toute balle) et compteur d'esquives hors rollback
    bullets: Query<(&GgrsNetId, &FixedTransform3D, &Bullet, Option<&Collider>), With<Rollback>>,
    mut stats: ResMut<BotStats>,
    weapons: Query<(&WeaponState, &WeaponModesState)>,
    enemies: Query<
        (
            &GgrsNetId,
            &FixedTransform3D,
            Option<&sim_core::stats::Stats>,
            Option<&combat::actors::Velocity>,
            Option<&Collider>,
        ),
        (With<Enemy>, With<Rollback>),
    >,
    windows: Query<(&GgrsNetId, &FixedTransform3D, &WindowHealth), With<Rollback>>,
    // Navigation (suite T1.14) : murs, fenêtres et portes, comme `chasseur`
    geometry: Query<
        (
            &GgrsNetId,
            &FixedTransform3D,
            &Collider,
            Option<&Wall>,
            Option<&Window>,
            Option<&DoorComponent>,
            Option<&world::Rock>,
        ),
        With<Rollback>,
    >,
    mut nav: ResMut<DirectNavigation>,
    run: Option<Res<run::Run>>,
    // m1-v3-bots-reanimation : surfaces d'interaction (la réanimation et la sélection du jeu),
    // saignement des joueurs à terre, frame courante.
    // m1-v3-bots-softlocks : butin au sol (ramassé au passage).
    // m1-v3-bots-armes : armes au sol, leur config, gabarit des personnages (emplacements),
    // mémoire des choix d'arme.
    (
        interactables,
        frame,
        pickups,
        ground_weapons,
        global_asset,
        weapon_assets,
        character_configs,
        mut weapon_choices,
        mut enemy_moves,
        mut explorer,
        mut rock_nav,
        mut jitter,
    ): (
        Query<
            (
                &GgrsNetId,
                &FixedTransform3D,
                &sim_core::interaction::Interactable,
                Option<&Collider>,
                Option<&combat::downed::Downed>,
            ),
            With<Rollback>,
        >,
        Res<utils::frame::FrameCount>,
        Query<
            (&GgrsNetId, &FixedTransform3D),
            (With<game::powerups::PowerUpPickup>, With<Rollback>),
        >,
        Query<(&GgrsNetId, &FixedTransform3D, &WeaponPickup), With<Rollback>>,
        Option<Res<GlobalAsset>>,
        Res<Assets<WeaponsConfig>>,
        Res<Assets<CharacterConfig>>,
        ResMut<WeaponChoices>,
        // m1-bots-apres-movement-feel : déplacement réel des ennemis (coincés)
        ResMut<EnemyMoves>,
        // M2-T16 : salles, objets, inventaire et blank (profil `explorateur`)
        crate::explorer::ExplorerWorld,
        // M3-T14 : navigation sans les rochers (ce qu'une bombe ouvrirait)
        ResMut<crate::explorer::RockNavigation>,
        // M3-T14 : anti-jitter (seconde mémoire de l'explorateur, hors rollback)
        ResMut<crate::explorer::ExplorerJitter>,
    ),
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
    let bullets_sorted = order_iter!(bullets);
    let mut dodged = false;
    let mut geometry_rects: Option<Vec<(Rect, bool)>> = None;
    // Géométrie sans les rochers (navigation « si on les cassait »), à la demande.
    let mut free_geometry: Option<Vec<(Rect, bool)>> = None;
    // Navigation seulement en mode `Floors` : il faut y trouver chaque ennemi puis le portail.
    // En vagues, les ennemis viennent aux joueurs (un zombie dehors est « caché » derrière les
    // murs jusqu'à sa fenêtre) : `prudent`/`fonceur` y gardent leur comportement de T1.14.
    // m1-v3-bots-softlocks : butin au sol, dans l'ordre `GgrsNetId`.
    let loot_points: Vec<(usize, FixedVec2)> = order_iter!(pickups)
        .into_iter()
        .map(|(id, t)| (id.0, t.translation.truncate()))
        .collect();
    // m1-v3-bots-armes : armes au sol (sans prix : lâchées, pas murales), mode par défaut de leur
    // config, dans l'ordre `GgrsNetId`.
    let weapons_config = global_asset
        .as_deref()
        .and_then(|global| weapon_assets.get(&global.weapons));
    let ground: Vec<(
        usize,
        FixedVec2,
        String,
        WeaponView,
        sim_core::ammo::AmmoType,
    )> = order_iter!(ground_weapons)
        .into_iter()
        .filter(|(_, _, pickup)| pickup.price.is_none())
        .filter_map(|(id, t, pickup)| {
            let asset = weapons_config?.0.get(&pickup.weapon_id)?;
            let mode = asset
                .config
                .firing_modes
                .get(&asset.config.default_firing_mode)?;
            Some((
                id.0,
                t.translation.truncate(),
                pickup.weapon_id.clone(),
                WeaponView::from_config(mode, true),
                asset.config.ammo_type.clone(),
            ))
        })
        .collect();
    let floors_mode = run
        .as_deref()
        .is_some_and(|run| matches!(run.mode, run::RunMode::Floors { .. }));
    let enemy_points: Vec<(usize, FixedVec2)> = enemies_sorted
        .iter()
        .map(|(id, t, _, _, _)| (id.0, t.translation.truncate()))
        .collect();
    // M2-T16 : l'étage vu par les `explorateur` (mode `Floors`) ; leurs cibles sont les ennemis
    // éveillés (un ennemi endormi est une salle à visiter, pas une cible).
    let explorer_positions: Vec<FixedVec2> = order_iter!(players)
        .into_iter()
        .filter(|(_, player, ..)| {
            local_players.0.contains(&player.handle)
                && assignments.0.get(&player.handle) == Some(&BotProfile::Explorateur)
        })
        .map(|(_, _, transform, ..)| transform.translation.truncate())
        .collect();
    let explore_frame = (floors_mode && !explorer_positions.is_empty())
        .then(|| explorer.frame(&explorer_positions));
    // M3-T14 : les explorateurs qui peuvent poser une bombe (handle, position) : à un même mur, le plus petit
    // handle pose seul (`GoalContext::peers`).
    let bombers: Vec<(usize, FixedVec2)> = if explore_frame.is_some() {
        let policy = explorer.options().policy;
        order_iter!(players)
            .into_iter()
            .filter(|(_, player, _, _, _, _, _, _, downed, _, _)| {
                local_players.0.contains(&player.handle)
                    && downed.is_none()
                    && assignments.0.get(&player.handle) == Some(&BotProfile::Explorateur)
                    && explorer.holder(player.handle).can_bomb(&policy)
            })
            .map(|(_, player, transform, ..)| (player.handle, transform.translation.truncate()))
            .collect()
    } else {
        Vec::new()
    };
    let awake_points: Vec<(usize, FixedVec2)> = enemy_points
        .iter()
        .filter(|(id, _)| {
            explore_frame
                .as_ref()
                .is_none_or(|frame| !frame.dormant.contains(id))
        })
        .copied()
        .collect();
    let explorer_options = explorer.options();
    if explorer_options.policy.hysteresis() {
        jitter.enter_floor(explorer.floor_index());
    }
    // Ennemis immobiles (tourelle : `MoveSpeed` de base nulle ; ennemi coincé : vitesse
    // nulle, ou sur place depuis `STUCK_FRAMES` malgré une vitesse voulue, m1-bots-apres-
    // movement-feel), voir `BotView::enemy_still`.
    enemy_moves.observe(frame.frame, &enemy_points);
    let still_points: Vec<FixedVec2> = enemies_sorted
        .iter()
        .filter(|(id, _, stats, velocity, _)| {
            stats
                .and_then(|s| s.get(&sim_core::stats::StatId::MoveSpeed))
                .is_some_and(|speed| speed <= Fixed::ZERO)
                || velocity.is_some_and(|v| v.main.length() < Fixed::ONE)
                || enemy_moves.stuck(id.0, frame.frame)
        })
        .map(|(_, t, _, _, _)| t.translation.truncate())
        .collect();

    // `_net_id` : nécessaire en première position pour `order_iter!` (tri déterministe des
    // joueurs avant de consommer le flux RNG "bots"), pas utilisé ensuite (même convention que
    // `move_characters`, `crates/game/src/character/player/input.rs`).
    // Réanimation (m1-v3-bots-reanimation) : toutes les surfaces (id, rectangle, portée) et,
    // parmi elles, les joueurs à terre dont le saignement est **urgent** (moins de
    // `REVIVE_URGENT_FRAMES` avant la mort), dans l'ordre `GgrsNetId`.
    // `explorateur` (bot-explorateur-v2, D65) : relever est son premier but tant que le
    // coéquipier peut l'être, urgent ou non (`surfaces_any`) ; les autres profils attendent
    // l'urgence (`surfaces_urgent`).
    let (surfaces_urgent, surfaces_any): (Vec<_>, Vec<_>) = order_iter!(interactables)
        .into_iter()
        .map(|(id, t, interactable, c, downed)| {
            let p = t.translation.truncate();
            let urgent = downed.is_some_and(|d| revive_urgent(d.bleedout_at_frame, frame.frame));
            let rect = c.map_or(Rect { min: p, max: p }, |c| Rect::collider(p, c));
            let revive =
                interactable.interaction_type == sim_core::interaction::InteractionType::Revive;
            (
                (id.0, rect, interactable.interaction_range, revive && urgent),
                (id.0, rect, interactable.interaction_range, revive),
            )
        })
        .unzip();

    for (
        _net_id,
        player,
        transform,
        health,
        inventory,
        collider,
        reserves,
        velocity,
        downed,
        config_handles,
        aim_mode,
    ) in order_iter!(players)
    {
        if !local_players.0.contains(&player.handle) {
            continue;
        }
        let Some(&profile) = assignments.0.get(&player.handle) else {
            continue;
        };

        if matches!(profile, BotProfile::Chasseur | BotProfile::Acheteur) {
            continue;
        }

        // D68 : à terre, l'explorateur attend la réanimation. Continuer le pilotage de
        // prudent (repli ou portail) déplace sa surface hors de portée de l'équipier.
        if profile == BotProfile::Explorateur && downed.is_some() {
            local_inputs.0.insert(player.handle, BoxInput::default());
            continue;
        }

        let position = transform.translation.truncate();
        let surfaces = if profile == BotProfile::Explorateur {
            &surfaces_any
        } else {
            &surfaces_urgent
        };
        // M2-T16 : un `explorateur` ne voit que les ennemis éveillés (voir `awake_points`).
        let exploring = profile == BotProfile::Explorateur;
        let points_for: &[(usize, FixedVec2)] = if exploring {
            &awake_points
        } else {
            &enemy_points
        };

        let nearest_enemy = nearest_by_net_id(
            enemies_sorted
                .iter()
                .filter(|(id, ..)| !exploring || points_for.iter().any(|(a, _)| *a == id.0))
                .map(|(id, enemy_transform, _, _, _)| {
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
                }),
        );

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
        // T1.14 : (chargeur, rechargeable, détente prête) par arme, même calcul que `hunter`
        let ammunition: Vec<(u32, bool, bool)> = inventory
            .weapons
            .iter()
            .map(|(entity, weapon)| {
                weapons
                    .get(*entity)
                    .ok()
                    .and_then(|(state, modes)| {
                        let mode = modes.modes.get(&state.active_mode)?;
                        let config = weapon.config.firing_modes.get(&state.active_mode)?;
                        let reserve = reserves.map_or(0, |r| r.get(&weapon.config.ammo_type));
                        Some((
                            mode.mag_ammo,
                            mode.can_reload(&config.mag, reserve),
                            crate::hunter::fire_trigger_ready(config.firing_mode, state.is_firing),
                        ))
                    })
                    .unwrap_or((0, false, true))
            })
            .collect();
        let (active_ammo, reloadable, trigger_ready) = ammunition
            .get(inventory.active_weapon_index)
            .copied()
            .unwrap_or((0, false, true));
        let usable = active_ammo > 0 || reloadable;
        // m1-v3-bots-softlocks : plus aucune réserve pour aucune arme (chargeurs seulement).
        let dry = inventory
            .weapons
            .iter()
            .all(|(_, weapon)| reserves.map_or(0, |r| r.get(&weapon.config.ammo_type)) == 0);
        let switch_weapon = !usable && ammunition.iter().any(|(a, r, _)| *a > 0 || *r);

        let view = BotView {
            position,
            health: health.current,
            health_max: health.max,
            ammo,
            wave: wave_number,
            nearest_enemy,
            nearest_window,
            hunter: None,
            portal,
            projectiles: projectile_views(bullets_sorted.iter().map(
                |(id, bullet_transform, bullet, bullet_collider)| {
                    let bullet_position = bullet_transform.translation.truncate();
                    (
                        id.0,
                        bullet.source_team != Team::Players,
                        ProjectileView {
                            position: bullet_position,
                            velocity: bullet.velocity,
                            size: bullet_collider.map_or(Fixed::from_num(5), collider_radius),
                            distance: position.distance(&bullet_position),
                        },
                    )
                },
            )),
            body_radius: collider.map_or(Fixed::from_num(10), collider_radius),
            reload: active_ammo == 0 && reloadable,
            switch_weapon,
            trigger_ready,
            velocity: velocity.map_or(FixedVec2::ZERO, |v| v.main),
            // Hors navigation (autre mode que `Floors`) : comportement de T1.14, ennemi supposé
            // visible, aucune route.
            enemy_visible: true,
            route: None,
            enemy_still: false,
            enemy_shootable: true,
            revive: None,
            loot: None,
            loot_interact: false,
            fire_range: None,
            orbit_reach: None,
            engage_reach: None,
            engage_strafe: 0,
            hold_revive: explorer_options.policy.combat_aware,
            explore: None,
            aim: aim_mode.copied().unwrap_or_default(),
        };

        let mut view = view;
        // Anti-blocage de combat (`Oscillation::Hysteresis`) : avant `navigate`, qui calcule alors le chemin.
        if exploring && downed.is_none() && explorer_options.policy.hysteresis() {
            if let Some((reach, strafe)) = jitter.engage(
                player.handle,
                frame.frame,
                explorer.enemy_progress(),
                view.nearest_enemy.is_some(),
            ) {
                view.engage_reach = Some(reach);
                view.engage_strafe = strafe;
            }
        }
        if floors_mode && directed(profile) {
            if let Some(collider) = collider {
                let rects = geometry_rects.get_or_insert_with(|| {
                    order_iter!(geometry)
                        .into_iter()
                        .filter(|(_, _, _, wall, window, door, rock)| {
                            wall.is_some() || window.is_some() || door.is_some() || rock.is_some()
                        })
                        .map(|(_, t, c, wall, _, door, rock)| {
                            (
                                Rect::collider(t.translation.truncate(), c),
                                wall.is_some() || door.is_some() || rock.is_some(),
                            )
                        })
                        .collect()
                });
                let body = AgentBody::from_collider(collider);
                view.enemy_still = prudent_like(profile)
                    && view
                        .nearest_enemy
                        .is_some_and(|enemy| still_points.contains(&enemy.position));
                let (visible, route) =
                    navigate(&mut nav.0, rects, &body, points_for, &view, profile);
                view.enemy_visible = visible;
                view.enemy_shootable = visible
                    || view.nearest_enemy.is_some_and(|enemy| {
                        crate::navigation::walls_clear_with(
                            rects,
                            position,
                            enemy.position,
                            Fixed::ZERO,
                        )
                    });
                view.route = route;
            }
        }

        // m1-v3-bots-reanimation : relever un coéquipier à terre en l'absence de menace
        // immédiate (même règle que `chasseur`/`acheteur`), en `Floors` comme en vagues.
        if directed(profile) && downed.is_none() && surfaces.iter().any(|(.., revive)| *revive) {
            let threat = view
                .nearest_enemy
                .is_some_and(|enemy| enemy.distance <= REVIVE_SAFE_DISTANCE && view.enemy_visible);
            if let (false, Some(collider)) = (threat, collider) {
                let rects = geometry_rects.get_or_insert_with(|| {
                    order_iter!(geometry)
                        .into_iter()
                        .filter(|(_, _, _, wall, window, door, rock)| {
                            wall.is_some() || window.is_some() || door.is_some() || rock.is_some()
                        })
                        .map(|(_, t, c, wall, _, door, rock)| {
                            (
                                Rect::collider(t.translation.truncate(), c),
                                wall.is_some() || door.is_some() || rock.is_some(),
                            )
                        })
                        .collect()
                });
                view.revive = revive_step(
                    &mut nav.0,
                    rects,
                    &AgentBody::from_collider(collider),
                    points_for,
                    position,
                    &surfaces,
                    profile == BotProfile::Explorateur,
                    profile == BotProfile::Explorateur
                        && explorer_options.policy.combat_aware
                        && jitter.snap_hold(player.handle, frame.frame),
                );
            }
        }

        if floors_mode && directed(profile) && downed.is_none() {
            // m1-v3-bots-armes : armes portées vues par leur config (mode courant), portée de
            // l'arme en main, choix d'arme (`arms`, hystérésis par `WeaponChoices`).
            let arms_views: Vec<WeaponView> = inventory
                .weapons
                .iter()
                .zip(&ammunition)
                .map(|((entity, weapon), (ammo, reloadable, _))| {
                    weapons
                        .get(*entity)
                        .ok()
                        .and_then(|(state, _)| weapon.config.firing_modes.get(&state.active_mode))
                        .map_or(NO_WEAPON, |mode| {
                            WeaponView::from_config(mode, *ammo > 0 || *reloadable)
                        })
                })
                .collect();
            let active = inventory.active_weapon_index;
            view.fire_range = Some(arms_views.get(active).map_or(Fixed::ZERO, |w| w.range));
            // Seulement face à un ennemi : `decide_prudent` ne lit l'orbite que là (la composition alloue).
            view.orbit_reach = (explorer_options.policy.combat_aware
                && view.nearest_enemy.is_some())
            .then(|| explorer.orbit_reach(player.handle))
            .flatten();
            if let Some(enemy) = view.nearest_enemy {
                let radius = enemies_sorted
                    .iter()
                    .find(|(_, t, ..)| t.translation.truncate() == enemy.position)
                    .and_then(|(.., c)| c.map(collider_radius))
                    .unwrap_or(arms::TARGET_RADIUS);
                if weapon_choices.step(
                    player.handle,
                    frame.frame,
                    &arms_views,
                    active,
                    enemy.distance,
                    radius,
                ) {
                    view.switch_weapon = true;
                }
            }

            // m1-v3-bots-softlocks + m1-v3-bots-armes : ramasser, sans menace immédiate (même
            // règle que la réanimation) : un power-up à moins de `LOOT_RADIUS` (n'importe où à
            // sec), une arme au sol à moins de `LOOT_RADIUS` si elle fait mieux que l'arme en
            // main (qui tomberait au sol, emplacements pleins) ou si un emplacement est libre.
            let threat = view
                .nearest_enemy
                .is_some_and(|enemy| enemy.distance <= REVIVE_SAFE_DISTANCE && view.enemy_visible);
            // À sec (aucune réserve pour aucune arme), la menace ne retient plus : l'arme sans réserve
            // (un `fusil_precision` sorti d'un coffre, chargeur vidé) ne tirera plus jamais et
            // l'arme de départ lâchée au sol est le seul moyen de blesser l'ennemi (graines 3, 4, 18).
            // Règles de ramassage propres à l'`explorateur` : les autres profils (throne, isaac, zombies)
            // gardent les leurs, leurs traces de scénario ne bougent pas.
            let explorer_rules =
                profile == BotProfile::Explorateur && explorer_options.policy.weapon_reserve;
            if let (None, false, Some(collider), Some(rects)) = (
                view.revive,
                threat && !(dry && explorer_rules),
                collider,
                geometry_rects.as_ref(),
            ) {
                let near = |p: &FixedVec2| position.distance(p) <= LOOT_RADIUS;
                let powerups: Vec<(usize, FixedVec2)> = loot_points
                    .iter()
                    .filter(|(_, p)| dry || near(p))
                    .copied()
                    .collect();
                let slots = config_handles
                    .and_then(|handles| character_configs.get(&handles.config))
                    .map_or(0, |config| config.weapon_slots as usize);
                let free_slot = inventory.weapons.len() < slots;
                let wanted: Vec<usize> = ground
                    .iter()
                    .filter(|(_, p, name, w, ammo_type)| {
                        // m2-salles-murs-gungeon : une arme dont le type de munition n'a aucune réserve n'a
                        // que son chargeur (ramasser une arme au sol n'ajoute pas de réserve) : à sec, le
                        // bot aurait lâché son arme de départ pour rien (un seul emplacement) et resterait
                        // sans tir, 600 frames, jusqu'au soft-lock.
                        let owned = inventory
                            .weapons
                            .iter()
                            .any(|(_, owned)| &owned.config.name == name);
                        let worth = arms::worth_picking(w, arms_views.get(active), free_slot);
                        if explorer_rules {
                            arms::wants_ground_weapon(
                                near(p),
                                dry,
                                reserves.map_or(0, |r| r.get(ammo_type)),
                                owned,
                                worth,
                            )
                        } else {
                            near(p) && !owned && worth
                        }
                    })
                    .map(|(id, ..)| *id)
                    .collect();
                if !powerups.is_empty() || !wanted.is_empty() {
                    if let Some((direction, interact)) = pickup_step(
                        &mut nav.0,
                        rects,
                        &AgentBody::from_collider(collider),
                        points_for,
                        position,
                        &powerups,
                        &wanted,
                        &surfaces,
                    ) {
                        view.loot = Some(direction);
                        view.loot_interact = interact;
                    }
                }
            }
        }

        // M2-T16 : exploration de l'étage et blank (`explorateur`, mode `Floors`). M3-T14 : fuite
        // d'une bombe posée, portes à clé, bombes (mur secret, rochers).
        if exploring && downed.is_none() {
            if let (Some(frame_data), Some(collider), Some(rects)) =
                (explore_frame.as_ref(), collider, geometry_rects.as_ref())
            {
                let mut holder = explorer.holder(player.handle);
                holder.bomb_inset = collider.offset.truncate().length();
                // Pas de présentoir d'arme sans réserve de son munition (voir
                // `ExploreFrame::without_unusable_weapons`).
                let unfiltered = frame_data;
                let filtered = if explorer_options.policy.weapon_reserve {
                    frame_data.without_unusable_weapons(|weapon| {
                        weapons_config
                            .and_then(|config| config.0.get(weapon))
                            .is_none_or(|asset| {
                                reserves.is_some_and(|r| r.get(&asset.config.ammo_type) > 0)
                            })
                    })
                } else {
                    None
                };
                let frame_data = filtered.as_ref().unwrap_or(unfiltered);
                let mut explore = crate::explorer::ExploreView {
                    blank: crate::explorer::should_blank(&view, &holder),
                    ..Default::default()
                };
                let body = AgentBody::from_collider(collider);
                if !frame_data.bombs.is_empty()
                    && (explorer_options.policy.combat_aware || holder.bomb.is_some())
                {
                    nav.0.update_from(rects, &body, points_for, position);
                    explore.flee =
                        crate::explorer::flee_step(&mut nav.0, &frame_data.bombs, position);
                }
                if view.nearest_enemy.is_some() || explore.flee.is_some() {
                    // Combat ou fuite : l'attente sur place n'est pas un balancement d'exploration.
                    jitter.sway.remove(&player.handle);
                }
                if view.nearest_enemy.is_none() && explore.flee.is_none() {
                    nav.0.update_from(rects, &body, points_for, position);
                    let ctx = crate::explorer::GoalContext {
                        holder,
                        portal_open: view.portal.is_some(),
                        boss_direct: explorer_options.boss_direct,
                        policy: explorer_options.policy,
                        peers: bombers
                            .iter()
                            .filter(|(handle, _)| *handle < player.handle)
                            .map(|(_, at)| *at)
                            .collect(),
                    };
                    // Navigation sans les rochers : seulement s'il y en a et qu'une bombe peut
                    // les casser (`explore_step` ne s'en sert que sans autre but).
                    let with_rocks = !frame_data.rocks.is_empty()
                        && ctx.policy.bombs.rocks
                        && ctx.holder.bomb.is_some();
                    if with_rocks {
                        // Géométrie sans les rochers : construite une fois par frame (comme `geometry_rects`).
                        let free_rects = free_geometry.get_or_insert_with(|| {
                            order_iter!(geometry)
                                .into_iter()
                                .filter(|(_, _, _, wall, window, door, _)| {
                                    wall.is_some() || window.is_some() || door.is_some()
                                })
                                .map(|(_, t, c, wall, _, door, _)| {
                                    (
                                        Rect::collider(t.translation.truncate(), c),
                                        wall.is_some() || door.is_some(),
                                    )
                                })
                                .collect()
                        });
                        rock_nav
                            .0
                            .update_from(free_rects, &body, points_for, position);
                    }
                    let mut step = crate::explorer::explore_step(
                        &mut nav.0,
                        with_rocks.then_some(&mut rock_nav.0),
                        frame_data,
                        &ctx,
                        position,
                        portal,
                        &surfaces,
                        explorer_options
                            .policy
                            .hysteresis()
                            .then(|| jitter.focus(player.handle))
                            .flatten(),
                        &if explorer_options.policy.hysteresis() {
                            jitter.banned(player.handle)
                        } else {
                            Default::default()
                        },
                    );
                    if explorer_options.policy.hysteresis() {
                        // Seul un pas réellement exécuté compte : réanimation, butin ou esquive le remplacent
                        // (`decide_explorer`), et un pas nul n'est pas une poursuite.
                        let executed = view.revive.is_none()
                            && view.loot.is_none()
                            && crate::dodge::dodge(&view).is_none();
                        match step.as_ref() {
                            Some(step) if executed && step.direction != FixedVec2::ZERO => {
                                jitter.pursue(player.handle, frame.frame, position, step);
                            }
                            _ => {
                                jitter.pursuits.remove(&player.handle);
                            }
                        }
                        match step.as_mut() {
                            Some(step) => jitter.steady(
                                player.handle,
                                frame.frame,
                                position,
                                step,
                                |from, to| nav.0.clear(from, to),
                            ),
                            None => {
                                jitter.sway.remove(&player.handle);
                            }
                        }
                    }
                    if *BOT_DEBUG {
                        eprintln!(
                            "BOTDBG f={} handle={} pos=({},{}) step={:?} portal={:?} rooms={} items={} secret_doors={} bombs={}",
                            frame.frame,
                            player.handle,
                            position.x,
                            position.y,
                            step,
                            portal,
                            frame_data.rooms.len(),
                            frame_data.items.len(),
                            frame_data.secret_doors.len(),
                            ctx.holder.bombs
                        );
                    }
                    if let Some(step) = step {
                        explore.direction = Some(step.direction);
                        explore.interact = step.interact;
                        explore.bomb = step.bomb;
                    }
                }
                explore.lead = explorer_options.policy.lead;
                if explorer_options.policy.patient() {
                    explore.patience = jitter.patience(
                        player.handle,
                        frame.frame,
                        position,
                        explore.direction.is_some() && !explore.interact,
                    );
                }
                view.explore = Some(explore);
            }
        }
        if exploring && downed.is_some() {
            jitter.sway.remove(&player.handle);
            jitter.stalls.remove(&player.handle);
        }

        if prudent_like(profile) && crate::dodge::dodge(&view).is_some() {
            dodged = true;
        }
        let input = decide_with_lazy_rng(&mut rng_streams, profile, &view);

        local_inputs.0.insert(player.handle, input);
    }
    if dodged {
        stats.dodges += 1;
    }
}

/// Ligne de vue vers l'ennemi le plus proche et direction du pas suivant (voir
/// [`BotView::route`]) : vers l'ennemi le plus proche par le chemin (postes de tir du champ, puis
/// le point accessible le plus proche s'il n'y en a pas), ou vers le portail sans ennemi.
/// Dérivé hors rollback des colliders de la frame, comme pour `chasseur`.
///
/// Le champ n'est calculé que si le profil s'en sert (ennemi caché ; `prudent` : ennemi au-delà
/// de sa bande ; portail au-delà de la zone de freinage). Une composante de moins de
/// [`ROUTE_DEAD_ZONE`] est annulée : les boutons ne gardent que le signe, un écart d'un pixel
/// ferait un pas en diagonale contre un coin de mur.
fn navigate(
    nav: &mut crate::navigation::BotNavigation,
    geometry: &[(Rect, bool)],
    body: &AgentBody,
    enemies: &[(usize, FixedVec2)],
    view: &BotView,
    profile: BotProfile,
) -> (bool, Option<FixedVec2>) {
    let position = view.position;
    if let Some(enemy) = view.nearest_enemy {
        let visible = crate::navigation::walls_clear(geometry, position, enemy.position);
        let needed = !visible
            || view.engage_reach.is_some()
            || (view.enemy_still && enemy.distance > crate::decide::STILL_TARGET_DISTANCE)
            || (prudent_like(profile) && enemy.distance > crate::decide::PRUDENT_MAX_DISTANCE);
        if !needed {
            return (visible, None);
        }
        nav.update_from(geometry, body, enemies, position);
        let route = nav.chase(position).or_else(|| {
            let mut candidates = enemies.to_vec();
            candidates.sort_by_key(|(id, p)| (position.distance(p), *id));
            candidates.into_iter().find_map(|(_, p)| {
                nav.investigate(position, p)
                    .filter(|direction| direction.length_squared() > Fixed::from_num(4))
            })
        });
        return (visible, route.map(dead_zone));
    }
    let Some(portal) = view.portal else {
        return (false, None);
    };
    nav.update_from(geometry, body, &[], position);
    // m1-v3-bots-softlocks (graines 53, 81) : sous la zone de freinage, ligne droite vers le
    // portail, sauf si le corps y touche un obstacle (ancre contre la roche) : le chemin, sinon
    // le bot pousse dans la roche (graine 81 : la petite composante qui le dégagerait tombe sous
    // la zone morte du pilotage ; graine 53 : coincé dans un coin, ligne des centres libre).
    if position.distance(&portal) < crate::decide::PORTAL_BRAKE_DISTANCE
        && nav.body_clear(position, portal)
    {
        return (false, None);
    }
    // Aucune case libre pour le corps à moins de `PORTAL_REACH` de l'ancre (ancre contre la
    // roche, graines 53, 81) : la case accessible la plus proche de l'ancre, au lieu de la ligne
    // droite dans la roche (m1-v3-bots-softlocks).
    let route = nav
        .approach(
            position,
            Rect {
                min: portal,
                max: portal,
            },
            PORTAL_REACH,
        )
        .map(|(direction, _)| direction)
        .or_else(|| nav.investigate(position, portal))
        .map(dead_zone);
    (false, route)
}

/// Profils qui passent par la vue de `prudent`/`fonceur` en mode `Floors` (navigation,
/// réanimation, armes et ramassage) : `explorateur` (M2-T16) en plus.
fn directed(profile: BotProfile) -> bool {
    matches!(
        profile,
        BotProfile::Prudent | BotProfile::Fonceur | BotProfile::Explorateur
    )
}

/// `explorateur` (M2-T16) se bat comme `prudent` (bande de distance, ennemi immobile, esquive).
fn prudent_like(profile: BotProfile) -> bool {
    matches!(profile, BotProfile::Prudent | BotProfile::Explorateur)
}

/// Aucun ennemi visible à moins de cette distance : un bot `prudent`/`fonceur` peut relever un
/// coéquipier à terre (même seuil que `chasseur`/`acheteur`, `crate::hunter`).
pub const REVIVE_SAFE_DISTANCE: Fixed = Fixed::from_bits(150 << 16);

/// `prudent`/`fonceur` ne relèvent un coéquipier que si son saignement est urgent (moins de 600
/// frames, 10 s, avant la mort) : ils continuent sinon de se battre, et un joueur humain ou un
/// `chasseur` a le temps de le relever (le scénario de référence `clone_quad` : le joueur scripté
/// relève à f355 ; une réanimation plus tôt par un bot changeait toute la partie).
pub const REVIVE_URGENT_FRAMES: u32 = 600;

/// Le saignement qui finit à `bleedout_at_frame` est urgent à la frame `frame`.
pub fn revive_urgent(bleedout_at_frame: u32, frame: u32) -> bool {
    bleedout_at_frame.saturating_sub(frame) <= REVIVE_URGENT_FRAMES
}

/// Pas vers le joueur à terre le plus proche (rectangle de son collider, puis `GgrsNetId`) que
/// le chemin atteint ; à portée, Interaction tenue seulement si le jeu sélectionnerait bien
/// cette surface (la plus proche à portée), sinon on s'approche encore — comme `chasseur`.
fn revive_step(
    nav: &mut crate::navigation::BotNavigation,
    geometry: &[(Rect, bool)],
    body: &AgentBody,
    enemies: &[(usize, FixedVec2)],
    position: FixedVec2,
    surfaces: &[(usize, Rect, Fixed, bool)],
    settle: bool,
    snap: bool,
) -> Option<ReviveView> {
    nav.update_from(geometry, body, enemies, position);
    let mut targets: Vec<_> = surfaces.iter().filter(|(.., revive)| *revive).collect();
    targets.sort_by_key(|(id, rect, ..)| (rect.distance(position), *id));
    targets.into_iter().find_map(|(id, rect, reach, _)| {
        // `snap` (`ExplorerPolicy::combat_aware`) : déjà à portée de la surface que le jeu sélectionnerait : Interaction
        // tenue sans bouger. Sinon le pas d'approche, qui vise un point à portée − 8, fait osciller le bot autour de la
        // portée et la réanimation (maintenue plusieurs frames) ne s'achève jamais (graine 103 d'`isaac` à deux bots).
        if snap {
            let selected = surfaces
                .iter()
                .filter_map(|(other, r, range, _)| {
                    // Pas de marge sur la portée : une marge laisse le bot osciller à la limite (graines 1, 168, 180, 187 à
                    // deux bots) ; la tenue est bornée par `ExplorerJitter::snap_hold`.
                    let d = r.distance(position);
                    (d <= *range).then_some((d, *other))
                })
                .min();
            if selected.is_some_and(|(_, selected)| selected == *id) {
                return Some(ReviveView {
                    direction: FixedVec2::ZERO,
                    interact: true,
                });
            }
        }
        surface_step_with(nav, position, surfaces, *id, *rect, *reach, settle).map(
            |(direction, interact)| ReviveView {
                direction,
                interact,
            },
        )
    })
}

/// Pas vers une surface d'interaction `id` (rectangle `rect`, portée `reach`) : approche à
/// portée − 8 par le chemin ; à portée, Interaction seulement si le jeu sélectionnerait bien
/// cette surface (la plus proche à portée), sinon approche encore. Réanimation
/// (m1-v3-bots-reanimation) et arme au sol (m1-v3-bots-armes).
pub(crate) fn surface_step(
    nav: &mut crate::navigation::BotNavigation,
    position: FixedVec2,
    surfaces: &[(usize, Rect, Fixed, bool)],
    id: usize,
    rect: Rect,
    reach: Fixed,
) -> Option<(FixedVec2, bool)> {
    surface_step_with(nav, position, surfaces, id, rect, reach, false)
}

/// [`surface_step`] pour l'`explorateur` (bot-explorateur-v2, D65) : `settle` = un pas sous la
/// zone morte (`dead_zone`) compte comme arrivé. Sinon le bot reste immobile à quelques pixels de
/// la portée, sans jamais presser Interaction (graine 47 à 2 bots). Les autres profils gardent
/// leur comportement (leurs traces de scénario ne bougent pas).
pub(crate) fn surface_step_with(
    nav: &mut crate::navigation::BotNavigation,
    position: FixedVec2,
    surfaces: &[(usize, Rect, Fixed, bool)],
    id: usize,
    rect: Rect,
    reach: Fixed,
    settle: bool,
) -> Option<(FixedVec2, bool)> {
    let reach = (reach - Fixed::from_num(8)).max(Fixed::ZERO);
    let (mut direction, _) = nav.approach(position, rect, reach)?;
    if settle && dead_zone(direction) == FixedVec2::ZERO {
        direction = FixedVec2::ZERO;
    }
    let mut interact = false;
    if direction == FixedVec2::ZERO {
        let selected = surfaces
            .iter()
            .filter_map(|(other, r, range, _)| {
                let d = r.distance(position);
                (d <= *range).then_some((d, *other))
            })
            .min();
        interact = selected.is_some_and(|(_, selected)| selected == id);
        if !interact {
            let closer = (rect.distance(position) - Fixed::from_num(8)).max(Fixed::ZERO);
            direction = nav.approach(position, rect, closer)?.0;
        }
    }
    Some((dead_zone(direction), interact))
}

/// Portée visée pour ramasser un butin (sous le `pickup_range` de 30 px de `throne`).
const LOOT_REACH: Fixed = Fixed::from_bits(16 << 16);

/// m1-v3-bots-armes : rayon de détour pour ramasser (power-up, arme au sol).
const LOOT_RADIUS: Fixed = Fixed::from_bits(96 << 16);

/// Arme sans mode lisible : inutilisable.
const NO_WEAPON: WeaponView = WeaponView {
    usable: false,
    damage: Fixed::ZERO,
    projectiles: 0,
    rate: Fixed::ZERO,
    range: Fixed::ZERO,
    spread: Fixed::ZERO,
};

/// m1-v3-bots-softlocks + m1-v3-bots-armes : pas suivant vers le butin accessible le plus proche
/// (distance, puis `GgrsNetId`) : un power-up (ramassé au passage, `LOOT_REACH`) ou une arme au sol
/// (`wanted`, surface d'interaction : Interaction tenue à portée, voir [`surface_step`]).
#[allow(clippy::too_many_arguments)]
fn pickup_step(
    nav: &mut crate::navigation::BotNavigation,
    geometry: &[(Rect, bool)],
    body: &AgentBody,
    enemies: &[(usize, FixedVec2)],
    position: FixedVec2,
    powerups: &[(usize, FixedVec2)],
    wanted: &[usize],
    surfaces: &[(usize, Rect, Fixed, bool)],
) -> Option<(FixedVec2, bool)> {
    nav.update_from(geometry, body, enemies, position);
    let mut targets: Vec<(Fixed, usize, Option<(Rect, Fixed)>)> = powerups
        .iter()
        .map(|(id, p)| (position.distance(p), *id, None))
        .chain(
            surfaces
                .iter()
                .filter(|(id, ..)| wanted.contains(id))
                .map(|(id, rect, reach, _)| (rect.distance(position), *id, Some((*rect, *reach)))),
        )
        .collect();
    targets.sort_by_key(|(d, id, _)| (*d, *id));
    targets
        .into_iter()
        .find_map(|(_, id, surface)| match surface {
            None => {
                let p = powerups.iter().find(|(pid, _)| *pid == id)?.1;
                nav.approach(position, Rect { min: p, max: p }, LOOT_REACH)
                    .map(|(direction, _)| (dead_zone(direction), false))
            }
            Some((rect, reach)) => surface_step(nav, position, surfaces, id, rect, reach),
        })
}

/// Composante annulée sous laquelle une direction de route ne presse pas son axe.
const ROUTE_DEAD_ZONE: Fixed = Fixed::from_bits(2 << 16);

pub(crate) fn dead_zone(direction: FixedVec2) -> FixedVec2 {
    let keep = |c: Fixed| {
        if c.abs() < ROUTE_DEAD_ZONE {
            Fixed::ZERO
        } else {
            c
        }
    };
    FixedVec2::new(keep(direction.x), keep(direction.y))
}

/// Rayon d'un collider : cercle → rayon, rectangle → demi-diagonale.
fn collider_radius(collider: &Collider) -> Fixed {
    match &collider.shape {
        ColliderShape::Circle { radius } => *radius,
        ColliderShape::Rectangle { width, height } => {
            FixedVec2::new(*width / Fixed::from_num(2), *height / Fixed::from_num(2)).length()
        }
    }
}

/// `ALACOD_BOT_DEBUG=1` : l'`explorateur` écrit son pas d'exploration à chaque frame sur stderr (diagnostic des
/// soft-locks, hors simulation).
static BOT_DEBUG: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var_os("ALACOD_BOT_DEBUG").is_some());

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
