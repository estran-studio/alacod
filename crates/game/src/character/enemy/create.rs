use bevy::prelude::*;
use bevy_fixed::fixed_math;
#[cfg(feature = "harmonium")]
use harmonium_bevy::components::HarmoniumTag;
use sim_core::stats::StatId;
use sim_core::team::Team;
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{config::CharacterConfig, create::create_character},
    collider::{CollisionLayer, CollisionSettings},
    global_asset::GlobalAsset,
    weapons::{
        melee::{spawn_melee_weapon_for_character, MeleeWeaponsConfig},
        WeaponInventory, WeaponsConfig,
    },
};

use super::{
    ai::{
        pathing::{EnemyPath, PathfindingConfig, WallSlideTracker},
        state::{EnemyAiConfig, EnemyTarget, MonsterState},
    },
    Enemy,
};

/// Quatre des cinq stats d'ennemi (T1.2, chantier B2) posées à la création, avant les
/// surcharges `CharacterConfig::stats` du RON de ce type d'ennemi : la valeur de base vient
/// de `PathfindingConfig::default()`, la même constante partagée que lisait jusqu'ici
/// `move_enemies` (`docs/conventions.md` §7). Un type d'ennemi qui ne déclare rien dans
/// `stats:` se comporte donc exactement comme avant ce chantier.
///
/// La cinquième, `EnemyMoveSpeed`, n'est pas ici : sa valeur de base est
/// `movement.max_speed`, propre à chaque `CharacterConfig` (comme `MoveSpeed`), pas la
/// constante partagée de `PathfindingConfig` — `create_character` la pose directement.
fn enemy_stat_defaults() -> [(StatId, fixed_math::Fixed); 4] {
    let defaults = PathfindingConfig::default();
    [
        (
            StatId::SeparationDistance,
            defaults.enemy_separation_distance,
        ),
        (StatId::SeparationForce, defaults.enemy_separation_force),
        (StatId::SlowDownDistance, defaults.slow_down_distance),
        (
            StatId::OptimalAttackDistance,
            defaults.optimal_attack_distance,
        ),
    ]
}

/// Spawns an enemy entity and returns it.
///
/// Returns the spawned Entity so callers can attach additional components
/// (e.g., WaveEnemy for wave tracking).
///
/// `team` : équipe posée sur l'entité (T2.9, testbed : `dummy`/`ally`/`civilian`/... ne sont
/// pas tous `Team::Enemies`). Le seul appelant historique (`enemy_spawn_from_spawners_system`,
/// vagues zombies) passe `Team::Enemies` explicitement, comportement inchangé.
///
/// L'IA (portées, obstacles, immobilité) vient de `CharacterConfig::ai` du personnage
/// `enemy_type_name` s'il en déclare une (T2.9), sinon `EnemyAiConfig::zombie()` comme avant
/// T2.9 : aucun personnage zombie existant ne déclare `ai`, ce repli garde leur comportement
/// exact.
pub fn spawn_enemy(
    enemy_type_name: String,
    position: fixed_math::FixedVec3,
    commands: &mut Commands,
    _weapons_asset: &Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: &Res<Assets<MeleeWeaponsConfig>>,
    characters_asset: &Res<Assets<CharacterConfig>>,

    global_assets: &Res<GlobalAsset>,
    collision_settings: &Res<CollisionSettings>,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
    team: Team,
) -> Entity {
    let ai_config = global_assets
        .character_configs
        .get(&enemy_type_name)
        .and_then(|handle| characters_asset.get(handle))
        .and_then(|config| config.ai.as_ref())
        .map(EnemyAiConfig::from)
        .unwrap_or_else(EnemyAiConfig::zombie);

    let entity = create_character(
        commands,
        global_assets,
        characters_asset,
        enemy_type_name,
        None,
        (LinearRgba::RED).into(),
        position,
        CollisionLayer(collision_settings.enemy_layer),
        id_factory,
        &enemy_stat_defaults(),
    );

    let inventory = WeaponInventory::default();

    // Give the enemy a melee weapon (zombie claws, fallback to bare hands) — sauf si son
    // `attack_range` est nul (T2.9, testbed : `dummy`/`target`/`follower`/`ally`/`civilian`
    // ne doivent jamais attaquer). `enemy_melee_attack_system`
    // (`crates/game/src/weapons/melee.rs`) déclenche une attaque dès qu'un joueur entre dans
    // la portée de l'ARME elle-même, indépendamment d'`EnemyAiConfig` : sans cette garde, un
    // ennemi à `attack_range: "0"` continuerait de griffer via la griffe équipée. Aucun
    // personnage zombie existant n'a un `attack_range` nul (`EnemyAiConfig::zombie()` = 40,
    // voir aussi `EnemyAiConfig::default()`), donc inchangé pour le contenu existant.
    if ai_config.attack_range > fixed_math::FIXED_ZERO {
        if let Some(melee_weapons_config) = melee_weapons_asset.get(&global_assets.melee_weapons) {
            if let Some(weapon) = melee_weapons_config
                .0
                .get("zombie_claws")
                .or_else(|| melee_weapons_config.0.get("bare_hands"))
            {
                spawn_melee_weapon_for_character(commands, entity, weapon.clone(), id_factory);
            }
        }
    }

    commands.entity(entity).insert((
        inventory,
        EnemyPath::default(),
        WallSlideTracker::default(),
        Enemy::default(),
        // AI components for flow field navigation and combat
        ai_config,
        EnemyTarget::default(),
        MonsterState::default(),
        // `Team` est un composant statique, non enregistré en rollback (voir sa doc dans
        // `sim_core::team`) : ne pas l'ajouter à `RollbackTraceApp` sans blesser les traces.
        team,
    ));

    #[cfg(feature = "harmonium")]
    commands
        .entity(entity)
        .insert(HarmoniumTag::new(&["danger", "combat", "monster"], 1.0));

    entity
}
