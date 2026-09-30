use bevy::prelude::*;

#[cfg(feature = "lighting")]
use bevy_light_2d::light::PointLight2d;

use combat::inventory::AmmoReserves;
#[cfg(feature = "harmonium")]
use harmonium_bevy::components::AiDriver;
use leafwing_input_manager::prelude::ActionState;
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
    control::{get_input_map, PlayerAction},
    input::CursorPosition,
    LocalPlayer, Player,
};
use bevy_fixed::fixed_math::{self, FixedVec3};

const PLAYER_COLORS: &[LinearRgba] = &[
    LinearRgba::RED,
    LinearRgba::BLUE,
    LinearRgba::GREEN,
    LinearRgba::BLACK,
];

pub fn create_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    weapons_asset: &Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: &Res<Assets<MeleeWeaponsConfig>>,
    character_asset: &Res<Assets<CharacterConfig>>,
    collision_settings: &Res<CollisionSettings>,

    position: FixedVec3,

    local: bool,
    handle: usize,
    name: String,
    pubkey: String,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
) {
    let player_name = name;
    let player_pubkey = pubkey;

    // Use "player" as the character config name (defined in global_asset.rs)
    let config_name = "player".to_string();

    let entity = create_character(
        commands,
        global_assets,
        character_asset,
        config_name.clone(),
        Some(if handle == 0 { "1" } else { "2" }.into()),
        (LinearRgba::GREEN).into(),
        position,
        CollisionLayer(collision_settings.player_layer),
        id_factory,
        // Pas de stats additionnelles pour un joueur (T1.2) : les cinq stats d'ennemi
        // n'ont pas de sens ici, voir la doc du paramètre.
        &[],
    );

    // `Team` est un composant statique, non enregistré en rollback (voir sa doc dans
    // `sim_core::team`) : ne pas l'ajouter à `RollbackTraceApp` sans blesser les traces.
    commands.entity(entity).insert(Team::Players);

    if local {
        commands.entity(entity).insert((
            LocalPlayer {},
            ActionState::<PlayerAction>::default(),
            get_input_map(),
        ));

        #[cfg(feature = "harmonium")]
        commands.entity(entity).insert(AiDriver {
            ai_influence: 1.0,
            detection_radius: 300.0,
            ..default()
        });
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
    let character_config = global_assets
        .character_configs
        .get(&config_name)
        .and_then(|handle| character_asset.get(handle));
    if let (Some(character_config), Some(weapons_config)) =
        (character_config, weapons_asset.get(&global_assets.weapons))
    {
        for (i, weapon_id) in character_config.starting_weapons.iter().enumerate() {
            let Some(weapon_asset) = weapons_config.0.get(weapon_id) else {
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
    if let Some(melee_weapons_config) = melee_weapons_asset.get(&global_assets.melee_weapons) {
        if let Some(bare_hands) = melee_weapons_config.0.get("bare_hands") {
            spawn_melee_weapon_for_character(commands, entity, bare_hands.clone(), id_factory);
        }
    }

    // Monnaie et perks (T2.3, chantier C5 v1) : posés une fois, à la création, comme le
    // reste de l'équipement de départ ci-dessus. `starting_currency` vient de
    // `CharacterConfig` (défaut 500, voir sa doc) ; `Perks` démarre toujours vide (aucun
    // perk n'est jamais acheté avant la première frame simulée).
    let starting_currency = character_config.map_or(500, |c| c.starting_currency);
    let currency = run::currency::Currency::new(starting_currency);
    let perks = run::perks::Perks::new();

    #[cfg(feature = "lighting")]
    {
        commands.entity(entity).insert((
            inventory,
            ammo_reserves,
            CursorPosition::default(),
            super::input::InteractionInput::default(),
            crate::interaction::Interactor,
            Player {
                handle,
                color: PLAYER_COLORS[handle].into(),
                name: player_name.clone(),
                pubkey: player_pubkey.clone(),
            },
            PointLight2d {
                radius: 200.,
                cast_shadows: false,
                falloff: 4.,
                ..default()
            },
            currency,
            perks,
        ));
    }

    #[cfg(not(feature = "lighting"))]
    {
        commands.entity(entity).insert((
            inventory,
            ammo_reserves,
            CursorPosition::default(),
            super::input::InteractionInput::default(),
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
    }
}
