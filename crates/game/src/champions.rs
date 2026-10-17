//! Champions et élites v2 (M3-T7, chantier D2 v2, `docs/conventions.md` §72) : l'exécution des
//! champs de `VariantDef` de §63 — `on_death` (ici), `behaviors` (à la création, `enemy::create`),
//! `loot` (ici), `tint` (présentation seule : aucune lecture dans la simulation) ; `chance` en
//! expression est résolue à l'apparition (`variant::draw_variant`, `VariantContext`).
//!
//! **`on_death`** ([`champion_death_system`], `RollbackSystemSet::DeathManagement`, après la pose des
//! `Death` de la frame et avant le despawn, comme `boss_defeated_system`) : pour chaque porteur de
//! [`Variant`] qui vient de mourir, par `GgrsNetId` croissant :
//! - `SpawnCharacters { id, count }` : `count` personnages `id` de l'équipe du mort, à des **décalages
//!   fixes** autour de sa position (un anneau, aucun RNG), `GgrsNetId` alloués dans l'ordre (mort
//!   puis rang `k`, clé de contenu `champion:<net_id>:<k>` dans les logs). Ils n'ont **jamais** de
//!   variante (pas de division en cascade) ;
//! - `Explode { radius, damage, destroys }` : le mécanisme est celui de M3-T4 (§69), sans second chemin :
//!   une **bombe à mèche d'une frame** (`world::Bomb`, comme l'`Action::Explode` des effets) est posée
//!   à la position du mort, de **son équipe**, créditée au dernier joueur qui l'a touché (sinon au
//!   mort), `destroys` transmis ; `game::bombs::bomb_fuse_system` puis `explosion_system` l'exécutent à
//!   la frame suivante (une `ExplosionRequest`, un `DamageEvent` par cible, un `Exploded`, exclusions
//!   `Death` et invulnérables de T4 : un familier immunisé n'est jamais touché) ;
//! - `SpawnPattern { pattern, weapon }` : un émetteur « fantôme » ([`PatternGhost`]) posé à la
//!   position du mort, qui joue le pattern puis est détruit (`despawn_rollback`) quand il a fini ;
//! - `loot` : un tirage local (`items::loot_seed(graine, étage, "champion:<net_id>")`, jamais
//!   `RngStreams`), l'objet (ou l'arme) posé au sol à côté du mort.
//!
//! Un personnage sans variante, ou dont la variante n'a aucun de ces champs, ne passe par rien de
//! tout cela : aucun composant, aucune ressource non vide, traces inchangées.

use std::collections::BTreeSet;

use behaviors::familiar::orbit_position;
use bevy::prelude::*;
use bevy_fixed::{
    fixed_math::{self, Fixed, FixedVec2},
    rng::{RngStreams, RollbackRng, RunSeed},
};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use combat::emitter::Emitter;
use effects::Action;
use items::{
    draw_loot_with, loot_seed, unlocked_table, ItemTable, LootTables, RunUnlocks, UnlockTable,
};
use sim_core::{damage::FriendlyFire, frame_events::FrameEvents, tag::Tags, team::Team};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::{
    character::{
        config::{CharacterConfig, CharacterConfigHandles},
        enemy::create::spawn_enemy,
        health::{rollback_apply_death, rollback_check_defeat, Death, HitBy},
        player::Player,
        variant::Variant,
    },
    collider::CollisionSettings,
    global_asset::GlobalAsset,
    rollback::RollbackTraceApp,
    system_set::RollbackSystemSet,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};

/// Rayon de l'anneau où apparaissent les personnages d'une division (unités).
pub const SPLIT_RING_RADIUS: i32 = 14;
/// Écart (unités, vers +x) entre le mort et l'objet qu'il lâche.
const DROP_OFFSET: i32 = 16;
/// Variante imposée aux personnages d'une division : absente de toute table, donc « aucune ».
pub const NO_VARIANT: &str = "-";

/// Émetteur sans personnage (`SpawnPattern` à la mort) : détruit une fois son `Emitter` terminé.
/// Rollback, checksum neutre.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PatternGhost;

/// Ce que les `on_death` d'un champion demandent, classé (fonction pure : testable sans ECS).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DeathPlan {
    /// `(rayon, dégâts, destroys)`, dans l'ordre déclaré.
    pub explosions: Vec<(Fixed, Fixed, bool)>,
    /// `(personnage, nombre)`, dans l'ordre déclaré.
    pub spawns: Vec<(String, u32)>,
    /// `(pattern, arme)`, dans l'ordre déclaré.
    pub patterns: Vec<(String, String)>,
    /// Actions refusées par le lint (jamais jouées, listées pour un `warn!`).
    pub ignored: Vec<String>,
}

/// Classe les actions `on_death` d'une variante.
pub fn death_plan(actions: &[Action]) -> DeathPlan {
    let mut plan = DeathPlan::default();
    for action in actions {
        match action {
            Action::Explode {
                radius,
                damage,
                destroys,
            } => plan.explosions.push((*radius, *damage, *destroys)),
            Action::SpawnCharacters { id, count } => plan.spawns.push((id.clone(), *count)),
            Action::SpawnPattern { pattern, weapon } => {
                plan.patterns.push((pattern.clone(), weapon.clone()))
            }
            other => plan.ignored.push(format!("{other:?}")),
        }
    }
    plan
}

/// Décalages (relatifs au mort) des `count` personnages d'une division : un anneau de
/// [`SPLIT_RING_RADIUS`] unités, régulièrement réparti, le rang 0 vers +x. Pure, `Fixed`.
pub fn split_offsets(count: u32) -> Vec<FixedVec2> {
    let origin = FixedVec2::ZERO;
    (0..count)
        .map(|rank| {
            orbit_position(
                origin,
                Fixed::from_num(SPLIT_RING_RADIUS),
                1,
                rank,
                count,
                0,
            )
        })
        .collect()
}

/// Clé de contenu de l'élément `rank` de la division d'un champion (`champion:<id>:<rang>`) ; le
/// butin utilise `champion:<id>` (`items::loot_seed`).
pub fn champion_key(net_id: usize, rank: Option<u32>) -> String {
    match rank {
        Some(rank) => format!("champion:{net_id}:{rank}"),
        None => format!("champion:{net_id}"),
    }
}

/// Un mort qui n'a rien à exécuter : variante sans `on_death` ni `loot` (la quasi-totalité des cas).
fn has_death_effects(def: &crate::character::variant::VariantDef) -> bool {
    !def.on_death.is_empty() || def.loot.is_some()
}

/// Paramètres d'apparition d'un personnage en cours de partie (division).
#[derive(bevy::ecs::system::SystemParam)]
pub struct DivisionAssets<'w> {
    global_assets: Res<'w, GlobalAsset>,
    collision_settings: Res<'w, CollisionSettings>,
    weapons_asset: Res<'w, Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<'w, Assets<MeleeWeaponsConfig>>,
    characters_asset: Res<'w, Assets<CharacterConfig>>,
    balance: Res<'w, crate::balance::ResolvedBalance>,
    difficulty: crate::clock::DifficultyReader<'w>,
}

/// Butin : tables, objets, déblocages, graine, étage.
#[derive(bevy::ecs::system::SystemParam)]
pub struct LootAssets<'w> {
    tables: Res<'w, LootTables>,
    items: Res<'w, ItemTable>,
    unlocks: Res<'w, UnlockTable>,
    active: Res<'w, RunUnlocks>,
    run_seed: Option<Res<'w, RunSeed>>,
    floor: Option<Res<'w, run::FloorState>>,
}

/// `on_death` et butin des champions qui meurent cette frame (voir la doc du module).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn champion_death_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    dying: Query<
        (
            &GgrsNetId,
            &Death,
            &Variant,
            &CharacterConfigHandles,
            &fixed_math::FixedTransform3D,
            &Team,
        ),
        With<Rollback>,
    >,
    players: Query<(&GgrsNetId, &Player), With<Rollback>>,
    configs: Res<Assets<CharacterConfig>>,
    division: DivisionAssets,
    loot: LootAssets,
    rng_streams: Res<RngStreams>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    effect_assets: crate::effects_runtime::EffectAssets,
) {
    if dying.is_empty() {
        return;
    }
    let player_ids: std::collections::BTreeMap<usize, GgrsNetId> = order_iter!(players)
        .into_iter()
        .map(|(net_id, player)| (player.handle, net_id.clone()))
        .collect();
    for (net_id, death, variant, handles, transform, team) in order_iter!(dying) {
        let Some(def) = configs
            .get(&handles.config)
            .and_then(|config| config.variants.as_ref())
            .and_then(|variants| variants.table.get(&variant.0))
            .filter(|def| has_death_effects(def))
        else {
            continue;
        };
        let center = transform.translation.truncate();
        let plan = death_plan(&def.on_death);
        for ignored in &plan.ignored {
            warn!(
                "champion {} : on_death : action {} non exécutée (voir `alacod lint`)",
                net_id, ignored
            );
        }
        info!(
            "ggrs{{f={} champion_death net_id={} variant={}}}",
            frame.frame, net_id.0, variant.0
        );

        // Division : personnages de l'équipe du mort, sans variante, décalages fixes.
        for (character, count) in &plan.spawns {
            let Some(health_max) = division.balance.health_max_by_character.get(character) else {
                warn!(
                    "champion {} : personnage « {character} » sans santé résolue (voir `alacod lint`)",
                    net_id
                );
                continue;
            };
            let health_max = crate::clock::scale(*health_max, division.difficulty.current());
            for (rank, offset) in split_offsets(*count).into_iter().enumerate() {
                let position = fixed_math::FixedVec3::new(
                    center.x.saturating_add(offset.x),
                    center.y.saturating_add(offset.y),
                    transform.translation.z,
                );
                spawn_enemy(
                    character.clone(),
                    position,
                    &mut commands,
                    &division.weapons_asset,
                    &division.melee_weapons_asset,
                    &division.characters_asset,
                    &division.global_assets,
                    &division.collision_settings,
                    &mut id_factory,
                    *team,
                    health_max,
                    rng_streams.run_seed,
                    Some(NO_VARIANT),
                    division
                        .difficulty
                        .variant_context_at(loot.floor.as_deref().map_or(0, |floor| floor.index)),
                );
                info!(
                    "ggrs{{f={} champion_split key={} character={}}}",
                    frame.frame,
                    champion_key(net_id.0, Some(rank as u32)),
                    character
                );
            }
        }

        // Explosions : une bombe à mèche d'une frame par action (mécanisme de M3-T4).
        if !plan.explosions.is_empty() {
            let source = death
                .last_hit_by
                .iter()
                .flatten()
                .find_map(|hit| match hit {
                    HitBy::Player(handle) => player_ids.get(handle).cloned(),
                    HitBy::Entity(_) => None,
                })
                .unwrap_or_else(|| net_id.clone());
            for (radius, damage, destroys) in &plan.explosions {
                let at = fixed_math::FixedTransform3D::new(
                    fixed_math::FixedVec3::new(center.x, center.y, fixed_math::FIXED_ZERO),
                    fixed_math::FixedMat3::IDENTITY,
                    fixed_math::FixedVec3::ONE,
                );
                commands.spawn((
                    world::Bomb {
                        owner: source.clone(),
                        fuse_until: frame.frame.saturating_add(1),
                        radius: *radius,
                        damage: *damage,
                        friendly_fire: FriendlyFire::Always,
                        team: *team,
                        destroys: *destroys,
                    },
                    at.to_bevy_transform(),
                    at,
                    id_factory.next(format!("explode_{}", net_id.0)),
                    Rollback,
                ));
            }
        }

        // Patterns : un émetteur fantôme par action.
        for (pattern, weapon) in &plan.patterns {
            let ghost_id = id_factory.next(format!("pattern_{pattern}"));
            let ghost = commands
                .spawn((
                    {
                        let t = fixed_math::FixedTransform3D::new(
                            transform.translation,
                            fixed_math::FixedMat3::IDENTITY,
                            fixed_math::FixedVec3::ONE,
                        );
                        (t.to_bevy_transform(), t)
                    },
                    ghost_id.clone(),
                    *team,
                    PatternGhost,
                    Rollback,
                ))
                .id();
            if !crate::effects_runtime::spawn_pattern(
                pattern,
                weapon,
                0,
                &ghost_id,
                frame.frame,
                (ghost, false),
                &mut commands,
                &effect_assets,
            ) {
                commands.entity(ghost).despawn_rollback();
            }
        }

        // Butin : tirage local, objet ou arme au sol.
        if let Some(table_id) = &def.loot {
            drop_loot(
                &mut commands,
                &mut id_factory,
                &loot,
                &division.global_assets,
                &division.weapons_asset,
                table_id,
                net_id,
                fixed_math::FixedVec3::new(
                    center.x.saturating_add(Fixed::from_num(DROP_OFFSET)),
                    center.y,
                    transform.translation.z,
                ),
                frame.frame,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn drop_loot(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    loot: &LootAssets,
    global_assets: &GlobalAsset,
    weapons: &Assets<WeaponsConfig>,
    table_id: &str,
    net_id: &GgrsNetId,
    position: fixed_math::FixedVec3,
    frame: u32,
) {
    let Some(table) = loot.tables.0.get(table_id) else {
        warn!(
            "champion {} : pool de butin « {table_id} » inconnu (voir `alacod lint`)",
            net_id
        );
        return;
    };
    let (run_seed, floor) = crate::loot::seed_context(&loot.run_seed, &loot.floor);
    let key = champion_key(net_id.0, None);
    let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &key));
    let table = unlocked_table(table_id, table, &loot.unlocks, &loot.active);
    let hidden: BTreeSet<&str> = loot.unlocks.hidden_items(&loot.active);
    let Some(item_id) = draw_loot_with(
        &table,
        &loot.items,
        None,
        items::Rarity::Common,
        false,
        &hidden,
        &mut rng,
    ) else {
        warn!("champion {} : le pool « {table_id} » ne donne rien", net_id);
        return;
    };
    if let Some(amount) = items::loot_currency(&item_id) {
        // M3-T3 (§68) : un tas de sous au sol.
        crate::shop::spawn_shell_pickup(commands, id_factory, amount, position, net_id.0);
        info!(
            "ggrs{{f={} champion_loot key={} item={}}}",
            frame, key, item_id
        );
        return;
    }
    match items::loot_weapon(&item_id) {
        Some(weapon_id) => {
            if !crate::loot::spawn_loot_weapon(
                commands,
                global_assets,
                weapons,
                weapon_id,
                position,
                id_factory,
            ) {
                warn!("champion {} : arme « {weapon_id} » inconnue", net_id);
            }
        }
        None => {
            crate::items::spawn_item_pickup(
                commands,
                &loot.items,
                item_id.clone(),
                position,
                id_factory,
            );
        }
    }
    info!(
        "ggrs{{f={} champion_loot key={} item={}}}",
        frame, key, item_id
    );
}

/// Détruit les émetteurs fantômes dont la séquence est finie.
pub fn pattern_ghost_cleanup_system(
    mut commands: Commands,
    ghosts: Query<(&GgrsNetId, Entity), (With<PatternGhost>, Without<Emitter>, With<Rollback>)>,
) {
    for (_, entity) in order_iter!(ghosts) {
        commands.entity(entity).despawn_rollback();
    }
}

pub struct ChampionsPlugin;

impl Plugin for ChampionsPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_and_trace_neutral::<PatternGhost>()
            .add_systems(
                GgrsSchedule,
                (
                    pattern_ghost_cleanup_system.before(rollback_check_defeat),
                    champion_death_system
                        .after(rollback_check_defeat)
                        .after(pattern_ghost_cleanup_system)
                        .after(crate::shop::shell_drop_on_death_system)
                        .before(rollback_apply_death),
                )
                    .in_set(RollbackSystemSet::DeathManagement),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    #[test]
    fn plan_classe_les_actions_dans_l_ordre() {
        let plan = death_plan(&[
            Action::Explode {
                radius: fx(40.0),
                damage: fx(10.0),
                destroys: false,
            },
            Action::SpawnCharacters {
                id: "petit".into(),
                count: 2,
            },
            Action::SpawnPattern {
                pattern: "couronne".into(),
                weapon: "canon".into(),
            },
            Action::Explode {
                radius: fx(8.0),
                damage: fx(0.0),
                destroys: true,
            },
            Action::Heal(fx(5.0)),
        ]);
        assert_eq!(
            plan.explosions,
            vec![(fx(40.0), fx(10.0), false), (fx(8.0), fx(0.0), true)]
        );
        assert_eq!(plan.spawns, vec![("petit".to_string(), 2)]);
        assert_eq!(
            plan.patterns,
            vec![("couronne".to_string(), "canon".to_string())]
        );
        assert_eq!(plan.ignored.len(), 1);
        assert_eq!(death_plan(&[]), DeathPlan::default());
    }

    #[test]
    fn anneau_de_division_regulier_et_deterministe() {
        assert!(split_offsets(0).is_empty());
        let one = split_offsets(1);
        assert!((one[0].x - fx(14.0)).abs() < fx(0.05) && one[0].y.abs() < fx(0.05));
        let four = split_offsets(4);
        assert_eq!(four.len(), 4);
        for offset in &four {
            assert!((offset.length() - fx(14.0)).abs() < fx(0.1), "{offset:?}");
        }
        // Rang 0 vers +x, rang 1 à un quart de tour (+y), rang 2 opposé au rang 0.
        assert!((four[0].x - fx(14.0)).abs() < fx(0.05));
        assert!((four[1].y - fx(14.0)).abs() < fx(0.05) && four[1].x.abs() < fx(0.05));
        assert!((four[2].x + fx(14.0)).abs() < fx(0.05));
        assert_eq!(split_offsets(3), split_offsets(3));
    }

    #[test]
    fn cles_de_contenu() {
        assert_eq!(champion_key(41, None), "champion:41");
        assert_eq!(champion_key(41, Some(1)), "champion:41:1");
        // La graine de butin dépend de la clé : deux champions, deux tirages distincts.
        assert_ne!(
            loot_seed(7, 0, &champion_key(41, None)),
            loot_seed(7, 0, &champion_key(42, None))
        );
    }

    #[test]
    fn aucune_variante_a_executer_sans_on_death_ni_butin() {
        let mut def = crate::character::variant::VariantDef {
            weight: 1,
            modifiers: vec![],
            tags: Tags::default(),
            skin: None,
            on_death: vec![],
            behaviors: None,
            loot: None,
            tint: Some("#ff0000".into()),
        };
        assert!(
            !has_death_effects(&def),
            "la teinte seule ne passe par rien"
        );
        def.loot = Some("commun".into());
        assert!(has_death_effects(&def));
        def.loot = None;
        def.on_death = vec![Action::GainShield(1)];
        assert!(has_death_effects(&def));
    }
}
