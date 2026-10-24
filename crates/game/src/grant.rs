//! Donner une arme, un objet ou de l'argent à un joueur (bac à sable, `docs/taches/plan-bac-a-sable.md` D4/D5).
//!
//! Deux appelants, **les mêmes fonctions** :
//! - la **pose initiale** d'un profil (`map_ldtk::game::profile::apply_player_profile`, qui *remplace* l'équipement) ;
//! - le **don à chaud** [`give`] (`alacod/give`, qui *ajoute* à un joueur en cours de partie).
//!
//! [`grant_item`] et [`equip_weapon`] sont ces fonctions partagées : l'ordre des opérations (et donc des
//! `GgrsNetId` alloués) est celui de l'ancien `apply_player_overrides`, les traces de scénario en dépendent.
//!
//! # Hors `GgrsSchedule`
//!
//! [`give`] écrit dans des composants rollback (`WeaponInventory`, `AmmoReserves`, `Inventory`, `Modifiers`,
//! `Currency`) depuis le monde, entre deux frames, et crée des armes avec le `GgrsNetIdFactory` : comme
//! `alacod/set_modifier`, c'est sûr **seulement** en session locale à `check_distance == 0`, où rien n'est
//! sauvegardé ni resimulé (garde `session_hot_edit_allowed` de `remote.rs`). Un don n'est **pas** enregistré
//! (`InputRecorder`) : un scénario enregistré ne le rejoue pas.

use bevy::prelude::*;
use combat::inventory::AmmoReserves;
use items::{ActiveSlot, Inventory, ItemKind, ItemTable};
use run::currency::Currency;
use serde::Serialize;
use sim_core::modifier::Modifiers;
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{config::CharacterConfigKey, config::CharacterConfigs, player::Player},
    weapons::{
        default_mode_ammo_contribution,
        melee::{spawn_melee_weapon_for_character, MeleeWeapon},
        spawn_weapon_for_player, WeaponInventory, WeaponRegistry,
    },
};

/// Pose l'objet `id` comme un ramassage (passif : inventaire et modificateurs `item:<id>` ; actif :
/// emplacement actif **chargé**, qui remplace l'actif tenu ; consommable : un exemplaire de plus).
/// `Err` : objet inconnu de `table`.
pub fn grant_item(
    table: &ItemTable,
    id: &str,
    inventory: &mut Inventory,
    modifiers: &mut Modifiers,
) -> Result<ItemKind, String> {
    let def = table
        .0
        .get(id)
        .ok_or_else(|| format!("objet inconnu « {id} »"))?;
    match def.kind {
        ItemKind::Passive => {
            inventory.passives.push(id.to_string());
            for modifier in &def.modifiers {
                modifiers.push(modifier.to_modifier(id));
            }
        }
        ItemKind::Active(charge) => {
            let mut slot = ActiveSlot::new(id.to_string());
            slot.charge = charge.required();
            inventory.active = Some(slot);
        }
        ItemKind::Consumable => {
            *inventory.consumables.entry(id.to_string()).or_insert(0) += 1;
        }
    }
    Ok(def.kind)
}

/// Ce que [`equip_weapon`] a créé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    /// Arme à distance : entrée de `WeaponInventory`, active, réserve de munitions créditée.
    Ranged,
    /// Arme de mêlée : enfant `MeleeWeapon` du joueur (le joueur n'en tient qu'une).
    Melee,
}

/// Crée l'arme `weapon_id` du registre (à distance, sinon de mêlée) sur `player`. À distance : la réserve
/// de son type de munition est créditée de la contribution du mode par défaut, puis l'arme est ajoutée à
/// `inventory` et devient **active**. Ne retire aucune arme : c'est à l'appelant (la pose d'un profil les
/// retire toutes, [`give`] ne remplace que ce qu'il doit).
pub fn equip_weapon(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    registry: &WeaponRegistry,
    player: Entity,
    inventory: &mut WeaponInventory,
    ammo_reserves: &mut AmmoReserves,
    weapon_id: &str,
) -> Result<WeaponKind, String> {
    if let Some(asset) = registry.weapons.get(weapon_id) {
        let (ammo_type, amount) = default_mode_ammo_contribution(asset);
        ammo_reserves.add(ammo_type, amount);
        spawn_weapon_for_player(
            commands,
            true,
            player,
            asset.clone(),
            inventory,
            id_factory,
            None,
        );
        Ok(WeaponKind::Ranged)
    } else if let Some(asset) = registry.melee.get(weapon_id) {
        spawn_melee_weapon_for_character(commands, player, asset.clone(), id_factory);
        Ok(WeaponKind::Melee)
    } else {
        Err(format!(
            "arme inconnue « {weapon_id} » (ni dans weapons.ron ni dans melee_weapons.ron)"
        ))
    }
}

/// Ce qu'on donne : tous les champs sont facultatifs, plusieurs à la fois.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gift {
    /// Id d'une arme à distance ou de mêlée.
    pub weapon: Option<String>,
    /// Id d'un objet (passif, actif ou consommable).
    pub item: Option<String>,
    /// Montant **ajouté** au solde (négatif : retiré, jamais sous zéro).
    pub currency: Option<i64>,
    /// Solde **imposé**.
    pub set_currency: Option<u32>,
}

impl Gift {
    pub fn is_empty(&self) -> bool {
        self.weapon.is_none()
            && self.item.is_none()
            && self.currency.is_none()
            && self.set_currency.is_none()
    }
}

/// Résultat d'un don (renvoyé tel quel par `alacod/give`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GiveReport {
    pub handle: usize,
    /// `Ranged` ou `Melee` si une arme a été donnée.
    pub weapon: Option<String>,
    /// Arme à distance déjà tenue : seule sa réserve a été complétée, l'arme est redevenue active.
    pub weapon_already_held: bool,
    /// Arme à distance retirée faute d'emplacement (`weapon_slots` du personnage), ou arme de mêlée remplacée.
    pub weapon_replaced: Option<String>,
    pub item: Option<String>,
    /// `passive`, `active` ou `consumable`.
    pub item_kind: Option<&'static str>,
    /// Solde final, si l'argent a bougé.
    pub currency: Option<u32>,
}

/// Donne `gift` au joueur `handle`, **à chaud** (voir le module). Valide tout avant d'écrire : `Err` ne
/// touche à rien.
///
/// Arme à distance : si elle est déjà tenue, sa réserve est complétée et elle redevient active ; sinon elle
/// s'ajoute, et si le personnage n'a plus d'emplacement (`weapon_slots`), l'arme active est retirée (sans
/// tomber au sol : ce n'est pas un échange). Arme de mêlée : remplace celle que le joueur tient.
pub fn give(world: &mut World, handle: usize, gift: &Gift) -> Result<GiveReport, String> {
    if gift.is_empty() {
        return Err("rien à donner : préciser weapon, item, currency ou set_currency".into());
    }
    let entity = {
        let mut query = world.query::<(Entity, &Player)>();
        query
            .iter(world)
            .find(|(_, player)| player.handle == handle)
            .map(|(entity, _)| entity)
            .ok_or_else(|| format!("aucun joueur de handle {handle}"))?
    };

    // Validation (rien n'est écrit tant qu'elle n'est pas passée).
    if let Some(id) = &gift.weapon {
        let registry = world.resource::<WeaponRegistry>();
        if !registry.weapons.contains_key(id) && !registry.melee.contains_key(id) {
            return Err(format!(
                "arme inconnue « {id} » (ni dans weapons.ron ni dans melee_weapons.ron)"
            ));
        }
    }
    if let Some(id) = &gift.item {
        let table = world
            .get_resource::<ItemTable>()
            .ok_or("table des objets absente (la carte n'est pas encore chargée)")?;
        if !table.0.contains_key(id) {
            return Err(format!("objet inconnu « {id} »"));
        }
    }
    if (gift.currency.is_some() || gift.set_currency.is_some())
        && world.get::<Currency>(entity).is_none()
    {
        return Err(format!("le joueur {handle} n'a pas de solde"));
    }

    let mut report = GiveReport {
        handle,
        ..Default::default()
    };

    if let Some(amount) = gift.set_currency {
        world.get_mut::<Currency>(entity).unwrap().0 = amount;
    }
    if let Some(delta) = gift.currency {
        let mut wallet = world.get_mut::<Currency>(entity).unwrap();
        let magnitude = u32::try_from(delta.unsigned_abs()).unwrap_or(u32::MAX);
        if delta >= 0 {
            wallet.earn(magnitude);
        } else {
            wallet.0 = wallet.0.saturating_sub(magnitude);
        }
    }
    if gift.currency.is_some() || gift.set_currency.is_some() {
        report.currency = world.get::<Currency>(entity).map(|wallet| wallet.0);
    }

    if let Some(id) = &gift.item {
        let kind = world.resource_scope(|world, table: Mut<ItemTable>| {
            let mut inventory = world.get::<Inventory>(entity).cloned().unwrap_or_default();
            let mut modifiers = world.get::<Modifiers>(entity).cloned().unwrap_or_default();
            let kind = grant_item(&table, id, &mut inventory, &mut modifiers)?;
            world.entity_mut(entity).insert((inventory, modifiers));
            Ok::<_, String>(kind)
        })?;
        report.item = Some(id.clone());
        report.item_kind = Some(match kind {
            ItemKind::Passive => "passive",
            ItemKind::Active(_) => "active",
            ItemKind::Consumable => "consumable",
        });
    }

    if let Some(id) = &gift.weapon {
        give_weapon(world, entity, id, &mut report)?;
    }
    Ok(report)
}

fn give_weapon(
    world: &mut World,
    entity: Entity,
    weapon_id: &str,
    report: &mut GiveReport,
) -> Result<(), String> {
    let slots = world
        .get::<CharacterConfigKey>(entity)
        .and_then(|key| world.resource::<CharacterConfigs>().get(&key.0))
        .map_or(2, |config| config.weapon_slots) as usize;
    let mut inventory = world
        .get::<WeaponInventory>(entity)
        .cloned()
        .unwrap_or_default();
    let mut ammo_reserves = world
        .get::<AmmoReserves>(entity)
        .cloned()
        .unwrap_or_default();
    // Armes de mêlée tenues (enfants du joueur), lues avant d'emprunter le monde pour les `Commands`.
    let held_melee: Vec<(Entity, String)> = world
        .get::<Children>(entity)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    world
                        .get::<MeleeWeapon>(child)
                        .map(|melee| (child, melee.config.name.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    world.resource_scope(|world, registry: Mut<WeaponRegistry>| {
        let mut state =
            bevy::ecs::system::SystemState::<(Commands, ResMut<GgrsNetIdFactory>)>::new(world);
        let (mut commands, mut id_factory) = state
            .get_mut(world)
            .map_err(|error| format!("ressources de la simulation indisponibles : {error}"))?;
        if let Some(asset) = registry.weapons.get(weapon_id) {
            if let Some(index) = inventory
                .weapons
                .iter()
                .position(|(_, held)| held.config.name == weapon_id)
            {
                // Déjà tenue : réserve complétée, arme réactivée.
                let (ammo_type, amount) = default_mode_ammo_contribution(asset);
                let current = ammo_reserves.get(&ammo_type);
                if amount > current {
                    ammo_reserves.add(ammo_type, amount - current);
                }
                inventory.active_weapon_index = index;
                report.weapon_already_held = true;
            } else {
                if !inventory.weapons.is_empty() && inventory.weapons.len() >= slots {
                    let index = inventory
                        .active_weapon_index
                        .min(inventory.weapons.len() - 1);
                    let (old, weapon) = inventory.weapons.remove(index);
                    commands.entity(old).despawn();
                    report.weapon_replaced = Some(weapon.config.name.clone());
                }
                equip_weapon(
                    &mut commands,
                    &mut id_factory,
                    &registry,
                    entity,
                    &mut inventory,
                    &mut ammo_reserves,
                    weapon_id,
                )?;
            }
            commands.entity(entity).insert((inventory, ammo_reserves));
            report.weapon = Some("Ranged".into());
        } else {
            // Mêlée : le joueur n'en tient qu'une (`bare_hands` par défaut), la nouvelle remplace l'ancienne.
            for (child, name) in &held_melee {
                commands.entity(*child).despawn();
                report.weapon_replaced = Some(name.clone());
            }
            equip_weapon(
                &mut commands,
                &mut id_factory,
                &registry,
                entity,
                &mut inventory,
                &mut ammo_reserves,
                weapon_id,
            )?;
            report.weapon = Some("Melee".into());
        }
        state.apply(world);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::stats::StatId;

    fn table() -> ItemTable {
        let def = |text: &str| ron::from_str(text).unwrap_or_else(|e| panic!("{e}"));
        ItemTable(
            [
                (
                    "bottes".to_string(),
                    def("(name: \"Bottes\", kind: Passive, modifiers: [(stat: MoveSpeed, op: Mul, value: \"1.1\")])"),
                ),
                ("cle".to_string(), def("(name: \"Clé\", kind: Consumable)")),
            ]
            .into(),
        )
    }

    fn world_with_player(handle: usize) -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(table());
        let player = world
            .spawn((
                Player {
                    handle,
                    ..Default::default()
                },
                Currency::new(100),
                Modifiers::default(),
            ))
            .id();
        (world, player)
    }

    #[test]
    fn l_argent_s_ajoute_se_retire_sans_passer_sous_zero_ou_s_impose() {
        let (mut world, player) = world_with_player(0);
        let report = give(
            &mut world,
            0,
            &Gift {
                currency: Some(50),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.currency, Some(150));
        give(
            &mut world,
            0,
            &Gift {
                currency: Some(-1000),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(world.get::<Currency>(player), Some(&Currency::new(0)));
        let report = give(
            &mut world,
            0,
            &Gift {
                set_currency: Some(500),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.currency, Some(500));
    }

    #[test]
    fn un_objet_se_pose_comme_un_ramassage_et_s_ajoute_a_l_inventaire_tenu() {
        let (mut world, player) = world_with_player(0);
        let gift = |id: &str| Gift {
            item: Some(id.into()),
            ..Default::default()
        };
        let report = give(&mut world, 0, &gift("bottes")).unwrap();
        assert_eq!(report.item_kind, Some("passive"));
        give(&mut world, 0, &gift("cle")).unwrap();
        give(&mut world, 0, &gift("cle")).unwrap();
        let inventory = world.get::<Inventory>(player).unwrap();
        assert_eq!(inventory.passives, vec!["bottes".to_string()]);
        assert_eq!(inventory.consumables["cle"], 2);
        // Le modificateur `item:bottes` de l'objet, et un second don ajoute au lieu de remplacer.
        let modifiers = world.get::<Modifiers>(player).unwrap();
        assert_eq!(
            modifiers
                .iter()
                .filter(|m| m.stat == StatId::MoveSpeed)
                .count(),
            1
        );
        give(&mut world, 0, &gift("bottes")).unwrap();
        let inventory = world.get::<Inventory>(player).unwrap();
        assert_eq!(inventory.passives.len(), 2);
        let modifiers = world.get::<Modifiers>(player).unwrap();
        assert_eq!(
            modifiers
                .iter()
                .filter(|m| m.stat == StatId::MoveSpeed)
                .count(),
            2
        );
    }

    #[test]
    fn un_don_refuse_ne_touche_a_rien() {
        let (mut world, player) = world_with_player(0);
        let refused =
            |world: &mut World, handle: usize, gift: Gift| give(world, handle, &gift).unwrap_err();
        let both = Gift {
            item: Some("fantome".into()),
            currency: Some(10),
            ..Default::default()
        };
        assert!(refused(&mut world, 0, both).contains("objet inconnu « fantome »"));
        assert!(refused(
            &mut world,
            3,
            Gift {
                currency: Some(10),
                ..Default::default()
            }
        )
        .contains("aucun joueur de handle 3"));
        assert!(refused(&mut world, 0, Gift::default()).contains("rien à donner"));
        assert_eq!(
            world.get::<Currency>(player),
            Some(&Currency::new(100)),
            "l'argent du même don n'est pas posé"
        );
        assert!(world.get::<Inventory>(player).is_none());
    }
}
