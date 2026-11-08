//! Clé portée par un ennemi, lâchée à sa mort, ramassée vers l'état d'équipe (X-6a1,
//! `docs/conventions.md` §117).
//!
//! - **Porter** : `CharacterConfig::carries_key` (id de clé). Aucune règle de jeu ici : l'ennemi la
//!   porte tant qu'il vit, la lâche en mourant ; ce qu'on en fait (retour au poste, porte atteinte
//!   avant la clé…) est une affaire de contenu.
//! - **Lâcher** : [`key_drop_on_death_system`] (`DeathManagement`) pose une `items::KeyPickup` à la position
//!   du mort, `GgrsNetId` `key_<id du mort>` : les morts sont vus triés par `GgrsNetId`
//!   (`order_iter!`), donc la numérotation ne dépend ni du temps ni des `Entity`. `With<Death>`, pas
//!   `Added<Death>` (voir `shop::shell_drop_on_death_system`). Aucun tirage aléatoire.
//! - **Ramasser** : [`key_pickup_system`] (`Effects`), au contact : le joueur de plus petit `GgrsNetId` à portée
//!   la verse à `run::TeamKeys` (état d'**équipe** : l'autre joueur en profite aussi) et l'entité est
//!   détruite par `despawn_rollback()`. Comme les douilles, un joueur à terre ramasse aussi.
//! - **Ouvrir** : `interaction::handle_door_interaction` lit `TeamKeys` quand la porte porte une
//!   serrure (`DoorConfig::lock`).
//!
//! Enregistrements rollback (`TeamKeys`, `KeyPickup`, `FrameEvents<KeyEvent>`, tous en variante neutre : rien
//! à contribuer au checksum tant qu'aucune clé n'existe) : `core_sim`, avant les cinq de clôture.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use items::KeyPickup;
use run::{KeyEvent, TeamKeys};
use sim_core::frame_events::FrameEvents;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::character::config::{CharacterConfigKey, CharacterConfigs};
use crate::character::health::Death;
use crate::character::player::Player;
use crate::system_set::RollbackSystemSet;

/// Portée de ramassage d'une clé (unités monde) : celle des douilles.
const KEY_PICKUP_RANGE: i32 = 20;

/// Lâche la clé portée par chaque ennemi mort cette frame (`CharacterConfig::carries_key`).
#[allow(clippy::type_complexity)]
pub fn key_drop_on_death_system(
    frame: Res<FrameCount>,
    configs: Res<CharacterConfigs>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut commands: Commands,
    dying: Query<
        (
            &GgrsNetId,
            &CharacterConfigKey,
            &fixed_math::FixedTransform3D,
        ),
        (With<Rollback>, With<Death>),
    >,
) {
    for (net_id, config_key, transform) in order_iter!(dying) {
        let Some(key) = configs
            .0
            .get(&config_key.0)
            .and_then(|config| config.carries_key.as_ref())
        else {
            continue;
        };
        info!(
            "ggrs{{f={} key_drop key={} carrier_net_id={}}}",
            frame.frame, key, net_id.0
        );
        spawn_key_pickup(
            &mut commands,
            &mut id_factory,
            key.clone(),
            transform.translation,
            net_id.0,
        );
    }
}

/// Fait apparaître une clé au sol ; `origin` (id du porteur) donne la clé de contenu du `GgrsNetId`.
pub fn spawn_key_pickup(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    key: String,
    position: fixed_math::FixedVec3,
    origin: usize,
) -> Entity {
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("key_{origin}"));
    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            KeyPickup { key },
            g_id,
            Rollback,
        ))
        .id()
}

/// Ramassage au contact : le joueur de plus petit `GgrsNetId` à portée verse la clé à l'équipe.
#[allow(clippy::type_complexity)]
pub fn key_pickup_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut team_keys: ResMut<TeamKeys>,
    mut key_events: ResMut<FrameEvents<KeyEvent>>,
    pickups: Query<
        (
            &GgrsNetId,
            Entity,
            &KeyPickup,
            &fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
    players: Query<
        (&GgrsNetId, &Player, &fixed_math::FixedTransform3D),
        (With<Rollback>, Without<KeyPickup>),
    >,
) {
    if pickups.is_empty() {
        return;
    }
    let range = Fixed::from_num(KEY_PICKUP_RANGE);
    let mut order: Vec<(usize, usize, FixedVec2)> = players
        .iter()
        .map(|(id, player, t)| (id.0, player.handle, t.translation.truncate()))
        .collect();
    order.sort_by_key(|(id, ..)| *id);
    for (net_id, entity, pickup, transform) in order_iter!(pickups) {
        let position = transform.translation.truncate();
        let Some((player_id, handle, _)) = order
            .iter()
            .find(|(_, _, at)| position.distance(at) <= range)
        else {
            continue;
        };
        team_keys.add(&pickup.key);
        key_events.send(KeyEvent {
            handle: *handle,
            key: pickup.key.clone(),
            delta: 1,
            reason: "pickup".to_string(),
        });
        info!(
            "ggrs{{f={} key_pickup net_id={} key={} by={}}}",
            frame.frame, net_id.0, pickup.key, player_id
        );
        commands.entity(entity).despawn_rollback();
    }
}

pub struct KeysPlugin;

impl Plugin for KeysPlugin {
    fn build(&self, app: &mut App) {
        // La ressource existe toujours (valeur par défaut = aucune clé) ; son enregistrement rollback
        // (neutre) est dans `core_sim`, après tous les plugins.
        app.init_resource::<TeamKeys>();
        app.add_systems(
            GgrsSchedule,
            key_drop_on_death_system
                // Après les autres drops : ils consomment tous `GgrsNetIdFactory` ; un porteur de clé
                // qui meurt n'existe que dans du contenu nouveau, les ids existants ne changent pas.
                .after(crate::shop::shell_drop_on_death_system)
                .after(crate::champions::champion_death_system)
                .before(crate::character::health::rollback_apply_death)
                .in_set(RollbackSystemSet::DeathManagement),
        );
        app.add_systems(
            GgrsSchedule,
            key_pickup_system
                .after(crate::shop::shell_pickup_system)
                .in_set(RollbackSystemSet::Effects),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_fixed::fixed_math::{FixedTransform3D, FixedVec3};

    fn at(x: i32, y: i32) -> FixedTransform3D {
        FixedTransform3D::new(
            FixedVec3::new(Fixed::from_num(x), Fixed::from_num(y), Fixed::ZERO),
            fixed_math::FixedMat3::IDENTITY,
            FixedVec3::ONE,
        )
    }

    fn world() -> World {
        let mut world = World::new();
        world.init_resource::<bevy_ggrs::RollbackOrdered>();
        world.init_resource::<TeamKeys>();
        world.init_resource::<FrameEvents<KeyEvent>>();
        world.init_resource::<FrameCount>();
        world.init_resource::<GgrsNetIdFactory>();
        world
    }

    fn player(world: &mut World, net_id: usize, handle: usize, at: FixedTransform3D) {
        world.spawn((
            Rollback,
            GgrsNetId(net_id, "player".to_string()),
            Player {
                handle,
                ..Default::default()
            },
            at,
        ));
    }

    fn key(world: &mut World, at: FixedTransform3D) {
        world.spawn((
            Rollback,
            GgrsNetId(100, "key".to_string()),
            KeyPickup {
                key: "cle".to_string(),
            },
            at,
        ));
    }

    /// Deux joueurs à portée de la même clé (le plus grand `GgrsNetId` créé en premier) : un seul
    /// la ramasse, celui de plus petit `GgrsNetId`, une seule fois (aucun ordre d'`Entity` ne compte).
    #[test]
    fn un_seul_joueur_ramasse_la_cle_le_plus_petit_net_id() {
        let mut world = world();
        player(&mut world, 20, 1, at(0, 0));
        player(&mut world, 10, 0, at(0, 8));
        key(&mut world, at(0, 0));
        world.run_system_once(key_pickup_system).unwrap();
        assert_eq!(world.resource::<TeamKeys>().count("cle"), 1);
        let events: Vec<_> = world.resource::<FrameEvents<KeyEvent>>().iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!((events[0].handle, events[0].delta), (0, 1));
    }

    /// Une clé hors de portée de tous les joueurs reste au sol, l'équipe n'a rien.
    #[test]
    fn une_cle_hors_de_portee_reste_au_sol() {
        let mut world = world();
        player(&mut world, 10, 0, at(0, 0));
        key(&mut world, at(0, 60));
        world.run_system_once(key_pickup_system).unwrap();
        assert_eq!(world.resource::<TeamKeys>().count("cle"), 0);
        assert!(world.resource::<FrameEvents<KeyEvent>>().is_empty());
    }
}
