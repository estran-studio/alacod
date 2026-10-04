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
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D, FixedVec2};
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
use crate::view::{
    nearest_by_net_id, projectile_views, BotView, EnemyView, ProjectileView, WindowView,
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
        ]);

        app.init_resource::<crate::navigation::BotNavigation>();
        app.init_resource::<DirectNavigation>();
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
        ),
        With<Rollback>,
    >,
    // T1.14 : projectiles (toute balle) et compteur d'esquives hors rollback
    bullets: Query<(&GgrsNetId, &FixedTransform3D, &Bullet, Option<&Collider>), With<Rollback>>,
    mut stats: ResMut<BotStats>,
    weapons: Query<(&WeaponState, &WeaponModesState)>,
    enemies: Query<(&GgrsNetId, &FixedTransform3D), (With<Enemy>, With<Rollback>)>,
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
        ),
        With<Rollback>,
    >,
    mut nav: ResMut<DirectNavigation>,
    run: Option<Res<run::Run>>,
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
    // Navigation seulement en mode `Floors` : il faut y trouver chaque ennemi puis le portail.
    // En vagues, les ennemis viennent aux joueurs (un zombie dehors est « caché » derrière les
    // murs jusqu'à sa fenêtre) : `prudent`/`fonceur` y gardent leur comportement de T1.14.
    let floors_mode = run
        .as_deref()
        .is_some_and(|run| matches!(run.mode, run::RunMode::Floors { .. }));
    let enemy_points: Vec<(usize, FixedVec2)> = enemies_sorted
        .iter()
        .map(|(id, t)| (id.0, t.translation.truncate()))
        .collect();

    // `_net_id` : nécessaire en première position pour `order_iter!` (tri déterministe des
    // joueurs avant de consommer le flux RNG "bots"), pas utilisé ensuite (même convention que
    // `move_characters`, `crates/game/src/character/player/input.rs`).
    for (_net_id, player, transform, health, inventory, collider, reserves, velocity) in
        order_iter!(players)
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
        };

        let mut view = view;
        if floors_mode && matches!(profile, BotProfile::Prudent | BotProfile::Fonceur) {
            if let Some(collider) = collider {
                let rects = geometry_rects.get_or_insert_with(|| {
                    order_iter!(geometry)
                        .into_iter()
                        .filter(|(_, _, _, wall, window, door)| {
                            wall.is_some() || window.is_some() || door.is_some()
                        })
                        .map(|(_, t, c, wall, _, door)| {
                            (
                                Rect::collider(t.translation.truncate(), c),
                                wall.is_some() || door.is_some(),
                            )
                        })
                        .collect()
                });
                let body = AgentBody::from_collider(collider);
                let (visible, route) =
                    navigate(&mut nav.0, rects, &body, &enemy_points, &view, profile);
                view.enemy_visible = visible;
                view.route = route;
            }
        }

        if profile == BotProfile::Prudent && crate::dodge::dodge(&view).is_some() {
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
            || (profile == BotProfile::Prudent
                && enemy.distance > crate::decide::PRUDENT_MAX_DISTANCE);
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
    if position.distance(&portal) < crate::decide::PORTAL_BRAKE_DISTANCE {
        return (false, None);
    }
    nav.update_from(geometry, body, &[], position);
    let route = nav
        .approach(
            position,
            Rect {
                min: portal,
                max: portal,
            },
            PORTAL_REACH,
        )
        .map(|(direction, _)| dead_zone(direction));
    (false, route)
}

/// Composante annulée sous laquelle une direction de route ne presse pas son axe.
const ROUTE_DEAD_ZONE: Fixed = Fixed::from_bits(2 << 16);

fn dead_zone(direction: FixedVec2) -> FixedVec2 {
    let keep = |c: Fixed| if c.abs() < ROUTE_DEAD_ZONE { Fixed::ZERO } else { c };
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
