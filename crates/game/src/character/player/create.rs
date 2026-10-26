use bevy::prelude::*;

use combat::inventory::AmmoReserves;
use sim_core::team::Team;
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{config::CharacterConfigs, create::create_character},
    collider::{CollisionLayer, CollisionSettings},
    weapons::{
        melee::spawn_melee_weapon_for_character, spawn_weapon_for_player, WeaponInventory,
        WeaponRegistry,
    },
};

use super::{input::CursorPosition, LocalPlayer, Player};
use bevy_fixed::fixed_math::{self, FixedVec3};

const PLAYER_COLORS: &[LinearRgba] = &[
    LinearRgba::RED,
    LinearRgba::BLUE,
    LinearRgba::GREEN,
    LinearRgba::BLACK,
];

/// `character` : id du personnage (registre `CharacterConfigs`) ; le joueur reçoit sa configuration, ses
/// armes de départ et son solde. `health_max` doit être celui de ce même personnage
/// (`ResolvedBalance::health_max_by_character`).
pub fn create_player(
    commands: &mut Commands,
    character: &str,
    weapon_registry: &WeaponRegistry,
    character_configs: &CharacterConfigs,
    collision_settings: &Res<CollisionSettings>,

    position: FixedVec3,

    local: bool,
    handle: usize,
    name: String,
    pubkey: String,

    // Santé max résolue (F5, chantier m0-v11) : voir `create_character` (paramètre
    // `health_max`).
    health_max: fixed_math::Fixed,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
) {
    let player_name = name;
    let player_pubkey = pubkey;

    // `character` : un `CharacterId` du registre de contenu
    let config_name = character.to_string();

    let entity = create_character(
        commands,
        character_configs,
        config_name.clone(),
        Some(if handle == 0 { "1" } else { "2" }.into()),
        (LinearRgba::GREEN).into(),
        position,
        CollisionLayer(collision_settings.player_layer),
        id_factory,
        // Pas de stats additionnelles pour un joueur (T1.2) : les cinq stats d'ennemi
        // n'ont pas de sens ici, voir la doc du paramètre.
        &[],
        health_max,
        None,
    );

    // `Team` est un composant statique, non enregistré en rollback (voir sa doc dans
    // `sim_core::team`) : ne pas l'ajouter à `RollbackTraceApp` sans blesser les traces.
    commands.entity(entity).insert(Team::Players);

    // `ActionState`, `InputMap` (et `AiDriver` sous `harmonium`) sont posés par l'application sur le
    // joueur local (`character::input_devices::attach_local_player_input`, `Added<LocalPlayer>`).
    if local {
        commands.entity(entity).insert(LocalPlayer {});
    }

    let mut inventory = WeaponInventory::default();
    // Réserves de munitions par type (T2.2, chantier B7), créditées ci-dessous en même
    // temps que chaque arme de départ ; posées sur le joueur avec le reste de son
    // équipement (voir plus bas).
    let mut ammo_reserves = AmmoReserves::new();

    // Armes de départ déclarées par `characters/*.ron` (`starting_weapons`, T1.5) : avant,
    // tous les joueurs recevaient toutes les entrées de `weapons.ron`, triées par nom
    // (l'ordre déterminait l'arme active : la première du tri). `player_config.ron`
    // déclare `starting_weapons: ["machine_gun", "pistol", "shotgun"]`, cet ordre exact,
    // pour que la simulation ne change pas (même arme active, mêmes GgrsNetId).
    let character_config = character_configs.get(&config_name);
    if let Some(character_config) = character_config {
        for (i, weapon_id) in character_config.starting_weapons.iter().enumerate() {
            let Some(weapon_asset) = weapon_registry.weapons.get(weapon_id) else {
                // Ne devrait pas arriver : `alacod lint` refuse une référence cassée au
                // démarrage. On ignore plutôt que de paniquer si le contenu a changé sous
                // nos pieds (rechargement à chaud hors partie).
                warn!("starting_weapons : arme inconnue « {weapon_id} » pour « {config_name} »");
                continue;
            };

            // Réserve initiale (T2.2), additionnée par type de munition
            // (`AmmoReserves::add` : deux armes qui partagent un type s'additionnent, voir
            // le scénario `ammo_shared_reserve`).
            let (ammo_type, amount) = crate::weapons::default_mode_ammo_contribution(weapon_asset);
            ammo_reserves.add(ammo_type, amount);

            spawn_weapon_for_player(
                commands,
                i == 0,
                entity,
                weapon_asset.clone(),
                &mut inventory,
                id_factory,
                // Chargeur plein à la création (pas de restauration, T2.2) : comportement
                // inchangé, voir la doc du paramètre.
                None,
            );
        }
    }

    // Add a default melee weapon (bare hands) to all players
    if let Some(bare_hands) = weapon_registry.melee.get("bare_hands") {
        spawn_melee_weapon_for_character(commands, entity, bare_hands.clone(), id_factory);
    }

    // Monnaie et perks (T2.3, chantier C5 v1) : posés une fois, à la création, comme le
    // reste de l'équipement de départ ci-dessus. `starting_currency` vient de
    // `CharacterConfig` (défaut 500, voir sa doc) ; `Perks` démarre toujours vide (aucun
    // perk n'est jamais acheté avant la première frame simulée).
    let starting_currency = character_config.map_or(500, |c| c.starting_currency);
    let currency = run::currency::Currency::new(starting_currency);
    let perks = run::perks::Perks::new();

    // La lumière du joueur (`PointLight2d`, feature `lighting`) est posée par la présentation
    // (`light::attach_player_light`, `Added<Player>`).
    commands.entity(entity).insert((
        inventory,
        ammo_reserves,
        CursorPosition::default(),
        crate::interaction::InteractionInput::default(),
        crate::interaction::Interactor,
        currency,
        perks,
        Player {
            handle,
            color: PLAYER_COLORS[handle].into(),
            name: player_name,
            pubkey: player_pubkey,
        },
    ));

    // Consommables de départ (M2-T1) : un inventaire seulement si le personnage en déclare.
    if let Some(config) = character_config.filter(|c| !c.starting_consumables.is_empty()) {
        commands.entity(entity).insert(items::Inventory {
            consumables: config.starting_consumables.clone(),
            ..default()
        });
    }

    // Bombes (M3-T4, §69) : seulement un personnage qui déclare `bomb` (aucun contenu existant
    // n'en a) ; le stock est un consommable `bomb` (`starting_consumables`).
    if let Some(def) = character_config.and_then(|c| c.bomb) {
        commands.entity(entity).insert(world::BombCarrier::new(def));
    }

    // Visée à quatre directions (suivi Q17) : le composant n'existe que pour `FourWay` ; un
    // personnage libre (tout le contenu d'avant) ne porte rien.
    if let Some(aim) = character_config
        .map(|c| c.aim)
        .filter(|aim| *aim != sim_core::aim::AimMode::Free)
    {
        commands.entity(entity).insert(aim);
    }
}
