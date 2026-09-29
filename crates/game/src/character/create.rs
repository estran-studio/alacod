use animation::AnimationStateBundle;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_kira_audio::prelude::*;
use combat::damage::Defenses;
use sim_core::modifier::Modifiers;
use sim_core::stats::{StatId, Stats};
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{config::CharacterConfigHandles, movement::Velocity},
    collider::{Collider, CollisionLayer},
    global_asset::GlobalAsset,
    weapons::melee::MeleeAttackState,
};

use bevy_ggrs::Rollback;

use super::{
    config::CharacterConfig,
    dash::DashState,
    health::{Health, HealthRegen},
    movement::SprintState,
    visuals::CharacterAppearance,
    Character,
};

pub fn create_character(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    character_asset: &Res<Assets<CharacterConfig>>,

    config_name: String,

    skin: Option<String>,
    color_health_bar: Color,
    translation: fixed_math::FixedVec3,

    collision_layer: CollisionLayer,
    id_factory: &mut ResMut<GgrsNetIdFactory>,

    // Stats de base additionnelles, posées avant les surcharges `CharacterConfig::stats`
    // du RON (qui gagnent toujours) : sert à `enemy::create::spawn_enemy` pour les cinq
    // stats d'ennemi (T1.2, `docs/conventions.md` §7) qui n'ont pas de champ dédié dans
    // `CharacterConfig` (elles viennent de `PathfindingConfig::default()`, pas du RON du
    // personnage). Vide pour un joueur (`player::create::create_player`).
    extra_stat_defaults: &[(StatId, fixed_math::Fixed)],
) -> Entity {
    let handle = global_assets.character_configs.get(&config_name).unwrap();
    let config = character_asset.get(handle).unwrap();

    let player_config_handle = global_assets
        .character_configs
        .get(&config.asset_name_ref)
        .unwrap()
        .clone();

    // BTreeMap (pas HashMap) : `AnimationStateBundle` place ces layers dans `ActiveLayers`,
    // un composant rollback dont l'ordre d'itération doit être stable entre clients.
    let starting_layers: std::collections::BTreeMap<String, String> = config
        .skins
        .get(skin.as_deref().unwrap_or(&config.starting_skin))
        .unwrap()
        .layers
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let transform_fixed = fixed_math::FixedTransform3D::new(
        translation,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::splat(config.scale),
    );

    // Entité logique uniquement : les sprites et la barre de vie sont ajoutés par
    // la présentation à partir de CharacterAppearance (voir character::visuals).
    let entity = commands
        .spawn((
            transform_fixed.to_bevy_transform(),
            Visibility::default(),
            AnimationStateBundle::new(starting_layers),
            CharacterAppearance {
                config_name: config_name.clone(),
                skin: skin.clone(),
                health_bar_color: color_health_bar,
            },
        ))
        .id();

    // Apply character scale to collider dimensions
    let mut collider: Collider = (&config.collider).into();
    match &mut collider.shape {
        crate::collider::ColliderShape::Circle { radius } => {
            *radius = radius.saturating_mul(config.scale);
        }
        crate::collider::ColliderShape::Rectangle { width, height } => {
            *width = width.saturating_mul(config.scale);
            *height = height.saturating_mul(config.scale);
        }
    }
    // Also scale the collider offset
    collider.offset.x = collider.offset.x.saturating_mul(config.scale);
    collider.offset.y = collider.offset.y.saturating_mul(config.scale);
    collider.offset.z = collider.offset.z.saturating_mul(config.scale);

    let health: Health = config.base_health.clone().into();

    // Stats de base (T1.2, chantier B2, `docs/conventions.md` §7) : dérivées des champs
    // existants de `CharacterConfig`, puis `extra_stat_defaults` (ennemis), puis
    // `config.stats` du RON en dernier — RON gagne toujours. Aucun de ces défauts ne
    // change la simulation d'un personnage qui ne déclare pas `stats:` : c'est exactement
    // la valeur que lisait jusqu'ici le code branché sur `CharacterConfig`/`PathfindingConfig`.
    let mut stats = Stats::new();
    stats.set(StatId::MoveSpeed, config.movement.max_speed);
    // Même valeur de base que `MoveSpeed`, lue par `move_enemies` : une stat distincte pour
    // ne pas coupler réglage joueur et ennemi (voir la doc de `StatId::EnemyMoveSpeed`).
    // Posée pour tout personnage, y compris les joueurs (jamais lue pour eux : inoffensif).
    stats.set(StatId::EnemyMoveSpeed, config.movement.max_speed);
    stats.set(StatId::Acceleration, config.movement.acceleration);
    stats.set(StatId::SprintMultiplier, config.movement.sprint_multiplier);
    stats.set(StatId::MaxHealth, config.base_health.max);
    if let Some(regen_rate) = config.base_health.regen_rate {
        stats.set(StatId::HealthRegen, regen_rate);
    }
    // Multiplicateurs à 1 par défaut : un `Mul`/`Pct` posé dessus par un futur chantier
    // (perk, statut) s'applique tel quel ; sans lui, `resolve()` renvoie exactement 1 et le
    // produit avec la valeur de configuration (dégâts, portée, cadence...) ne change rien
    // (voir la contrainte « traces identiques » du rapport de tâche).
    stats.set(StatId::FireRate, fixed_math::FIXED_ONE);
    stats.set(StatId::ReloadSpeed, fixed_math::FIXED_ONE);
    stats.set(StatId::Damage, fixed_math::FIXED_ONE);
    stats.set(StatId::Range, fixed_math::FIXED_ONE);
    for (id, value) in extra_stat_defaults {
        stats.set(id.clone(), *value);
    }
    for (id, value) in &config.stats {
        stats.set(id.clone(), *value);
    }

    // Add gameplay components to the visual entity
    commands.entity(entity).insert((
        transform_fixed,
        SpatialAudioEmitter { instances: vec![] },
        Velocity {
            main: fixed_math::FixedVec2::ZERO,
            knockback: fixed_math::FixedVec2::ZERO,
        },
        SprintState::default(),
        DashState::default(),
        MeleeAttackState::default(),
        collider,
        health,
        collision_layer,
        Character,
        CharacterConfigHandles {
            config: player_config_handle.clone(),
        },
        // `Tags`/`Defenses` (T1.1, chantier B1) : composants statiques, non enregistrés en
        // rollback (même justification que `Team`, voir sa doc dans `sim_core::team`) —
        // jamais mutés en T1.1, une entité rollback détruite passe toujours par
        // `despawn_rollback()`, donc ils survivent intacts à une résurrection après
        // rollback.
        config.tags.clone(),
        Defenses {
            immune_to: config.immune_to.clone(),
            resistances: config.resistances.clone(),
        },
    ));
    // Second `insert` : un tuple `Bundle` est limité en arité (15 éléments avec bevy_ecs
    // 0.19) ; le premier est déjà plein.
    commands.entity(entity).insert((
        // `Stats`/`Modifiers` (T1.2, chantier B2) : à l'inverse de `Tags`/`Defenses`,
        // enregistrés en rollback par `stats::StatsPlugin` (`Modifiers` change avec le
        // temps — expiration, futurs statuts/perks — donc entre dans le `Checksum` GGRS).
        stats,
        Modifiers::default(),
        id_factory.next(config_name),
    ));

    // Add HealthRegen component if configured
    if let (Some(regen_rate), Some(regen_delay_frames)) = (
        config.base_health.regen_rate,
        config.base_health.regen_delay_frames,
    ) {
        commands.entity(entity).insert(HealthRegen {
            last_damage_frame: 0,
            regen_rate,
            regen_delay_frames,
        });
    }

    commands.entity(entity).insert(Rollback);

    entity
}
