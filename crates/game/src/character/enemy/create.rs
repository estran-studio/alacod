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
        spawn_weapon_for_player, WeaponInventory, WeaponsConfig,
    },
};

use super::{
    ai::{
        pathing::{EnemyPath, PathfindingConfig, WallSlideTracker},
        state::{EnemyAiConfig, EnemyBehaviors, EnemyTarget, MonsterState},
    },
    Enemy,
};

/// Quatre des cinq stats d'ennemi (T1.2, chantier B2) posées à la création, avant les
/// surcharges `CharacterConfig::stats` du RON de ce type d'ennemi : la valeur de base vient
/// de `PathfindingConfig::default()`, la même constante partagée que lisait jusqu'ici
/// `move_enemies` (`docs/conventions.md` §9). Un type d'ennemi qui ne déclare rien dans
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
    weapons_asset: &Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: &Res<Assets<MeleeWeaponsConfig>>,
    characters_asset: &Res<Assets<CharacterConfig>>,

    global_assets: &Res<GlobalAsset>,
    collision_settings: &Res<CollisionSettings>,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
    team: Team,
    // Santé max résolue (F5, chantier m0-v11) : voir `create_character` (paramètre
    // `health_max`).
    health_max: fixed_math::Fixed,
    // Variantes (T1.5) : graine de run (la graine de carte : `RngStreams::run_seed` en jeu,
    // `MapGenerationConfig::seed` au chargement de la carte, même valeur) et variante imposée (champ LDtk `variant`
    // d'un `CharacterSpawn`, `None` ailleurs). Sans table `variants`, ignorés.
    run_seed: u32,
    forced_variant: Option<&str>,
    // M3-T7 : joueurs et étage de l'apparition (identifiants de `chance` en expression).
    variant_context: crate::character::variant::VariantContext,
) -> Entity {
    let character_config = global_assets
        .character_configs
        .get(&enemy_type_name)
        .and_then(|handle| characters_asset.get(handle));
    let ai_ron = character_config.and_then(|config| config.ai.as_ref());
    let mut ai_config = ai_ron
        .map(EnemyAiConfig::from)
        .unwrap_or_else(EnemyAiConfig::zombie);
    // T1.4 : règles de comportement (liste du RON, sinon liste par défaut dérivée de la
    // config : exactement le comportement d'avant T1.4).
    let mut behaviors = EnemyBehaviors::from_config(ai_ron, &ai_config);
    // M2-T0c : règles de chaque phase d'un boss (la phase courante est `BossState`, rollback).
    let boss = character_config.and_then(|config| config.boss.clone());
    if let Some(boss) = &boss {
        behaviors.phase_rules = boss.phases.iter().map(|p| p.behaviors.clone()).collect();
    }

    // Variantes (T1.5, `character::variant`) : seulement pour un personnage qui déclare une
    // table. Le `GgrsNetId` est alloué ici (même valeur que dans `create_character`, voir son
    // paramètre `net_id`) parce que le tirage en dépend et que la variante choisit le skin
    // avant la création. Sans table : chemin d'origine, inchangé (traces).
    let mut stat_defaults: Vec<(StatId, fixed_math::Fixed)> = enemy_stat_defaults().to_vec();
    let mut net_id = None;
    let mut variant_net_id = 0;
    let mut chosen: Option<(String, &crate::character::variant::VariantDef)> = None;
    let mut spawn_health = health_max;
    if let Some(variants) = character_config.and_then(|config| config.variants.as_ref()) {
        let id = id_factory.next(enemy_type_name.clone());
        if let Some(name) = crate::character::variant::draw_variant(
            variants,
            forced_variant,
            run_seed,
            id.0 as u64,
            variant_context,
        ) {
            let def = &variants.table[&name];
            spawn_health =
                crate::character::variant::variant_health(health_max, &def.modifiers(&name), 0);
            chosen = Some((name, def));
        }
        // Base `MaxHealth` = santé F5, pour que `sync_health_from_stats` applique le
        // modificateur de la variante (un ennemi n'a pas cette stat de base sinon).
        stat_defaults.push((StatId::MaxHealth, health_max));
        variant_net_id = id.0;
        net_id = Some(id);
    }
    let skin = chosen.as_ref().and_then(|(_, def)| def.skin.clone());
    // M3-T7 (§72) : les `behaviors` d'une variante remplacent ceux du personnage à la création ;
    // `Shoot` se compile vers `ai_config.ranged` comme pour une liste de personnage (`From`).
    if let Some(rules) = chosen.as_ref().and_then(|(_, def)| def.behaviors.as_ref()) {
        ai_config.ranged = rules.iter().find_map(|rule| match rule {
            behaviors::Behavior::Shoot {
                weapon,
                pattern,
                range,
                cooldown_frames,
            } => Some(super::ai::state::RangedAttack {
                weapon: weapon.clone(),
                pattern: pattern.clone(),
                range: *range,
                cooldown_frames: *cooldown_frames,
            }),
            _ => None,
        });
        behaviors.rules = rules.clone();
    }

    let entity = create_character(
        commands,
        global_assets,
        characters_asset,
        enemy_type_name,
        skin,
        (LinearRgba::RED).into(),
        position,
        CollisionLayer(collision_settings.enemy_layer),
        id_factory,
        &stat_defaults,
        spawn_health,
        net_id,
    );
    if let Some((name, def)) = &chosen {
        let mut tags = character_config
            .map(|config| config.tags.clone())
            .unwrap_or_default();
        for tag in def.tags.iter() {
            tags.insert(tag.clone());
        }
        commands.entity(entity).insert((
            sim_core::modifier::Modifiers(def.modifiers(name)),
            tags,
            crate::character::variant::Variant(name.clone()),
        ));
        info!("ggrs{{variant net_id={} name={}}}", variant_net_id, name);
    }

    let mut inventory = WeaponInventory::default();

    // Arme de mêlée de la règle `Melee(arme)` (T1.4 : une donnée ; avant, `zombie_claws`
    // codé en dur), repli `bare_hands` si l'arme manque au jeu (le lint la refuse). Sans règle
    // `Melee` (T2.9 : `dummy`/`target`/`follower`/`ally`/`civilian`, `attack_range: "0"`),
    // aucune arme : `enemy_melee_attack_system` (`crates/game/src/weapons/melee.rs`) attaque
    // dès qu'un joueur entre dans la portée de l'ARME équipée, indépendamment de l'IA. La
    // liste par défaut contient `Melee("zombie_claws")` exactement quand `attack_range > 0` :
    // inchangé pour le contenu existant.
    if let Some(melee_weapon) = behaviors.melee_weapon() {
        if let Some(melee_weapons_config) = melee_weapons_asset.get(&global_assets.melee_weapons) {
            if let Some(weapon) = melee_weapons_config
                .0
                .get(melee_weapon)
                .or_else(|| melee_weapons_config.0.get("bare_hands"))
            {
                spawn_melee_weapon_for_character(commands, entity, weapon.clone(), id_factory);
            }
        }
    }

    // Tir à distance (T1.2) : l'arme de `ranged.weapon` est équipée (active) dans
    // l'inventaire ; sa table `projectiles` fournit les projectiles de l'émetteur. Sans
    // `ranged` (tout le contenu d'avant T1.2), rien ne change : inventaire vide, aucun
    // `RangedAttackState`.
    // M2-T7 : armes des règles `Shoot` des phases d'un boss (tir par phase), équipées après
    // celle de `ranged` (inactives), une fois chacune. Sans règle `Shoot` de phase : rien.
    let mut shoot_weapons: Vec<String> = ai_config
        .ranged
        .iter()
        .map(|ranged| ranged.weapon.clone())
        .collect();
    if let Some(boss) = &boss {
        for rule in boss.phase_shoots() {
            if let behaviors::Behavior::Shoot { weapon, .. } = rule {
                if !shoot_weapons.contains(weapon) {
                    shoot_weapons.push(weapon.clone());
                }
            }
        }
    }
    let ranged = !shoot_weapons.is_empty();
    for (index, weapon_name) in shoot_weapons.iter().enumerate() {
        match weapons_asset
            .get(&global_assets.weapons)
            .and_then(|config| config.0.get(weapon_name))
        {
            Some(weapon) => {
                spawn_weapon_for_player(
                    commands,
                    index == 0,
                    entity,
                    weapon.clone(),
                    &mut inventory,
                    id_factory,
                    None,
                );
            }
            None => warn!(
                "ennemi : arme de tir « {} » inconnue (voir `alacod lint`), pas de tir à distance",
                weapon_name
            ),
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
    if ranged {
        commands
            .entity(entity)
            .insert(super::ai::state::RangedAttackState::default());
    }
    // T1.4 : état des behaviors nouveaux, seulement s'ils sont listés (checksum neutre) ;
    // les règles elles-mêmes sont statiques, hors rollback (comme `Team`).
    if behaviors.needs_runtime() {
        commands
            .entity(entity)
            .insert(super::ai::state::BehaviorRuntime::default());
    }
    commands.entity(entity).insert(behaviors);
    if let Some(boss) = boss {
        commands
            .entity(entity)
            .insert((crate::boss::BossPlan(boss), behaviors::BossState::default()));
    }

    #[cfg(feature = "harmonium")]
    commands
        .entity(entity)
        .insert(HarmoniumTag::new(&["danger", "combat", "monster"], 1.0));

    entity
}
