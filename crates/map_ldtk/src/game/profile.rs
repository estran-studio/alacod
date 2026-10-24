//! Mise en place d'un **profil** de départ : ce que `game::replay::{Scenario, PlayerScript}` décrit en
//! dehors des inputs (armes, objets, argent, modificateurs, mutations, dash, bombe, visée du joueur ;
//! placements de personnages, de power-ups, d'objets, de coffres et de présentoirs).
//!
//! Un seul code d'application pour tous les appelants : le runner de scénarios (`scenario::runner`) et,
//! hors runner, tout jeu qui veut poser un profil (bac à sable, `docs/taches/plan-bac-a-sable.md` D4).
//! Les types restent ceux de `game::replay` ; ce module n'en définit pas un second format.
//!
//! Deux étages :
//! - **fonctions** : [`apply_player_profile`] (un joueur déjà créé), [`spawn_character_placement`] ;
//! - **plugin** : [`ProfilePlugin`] pose les ressources et les systèmes, dans l'ordre exact historique du
//!   runner (voir ci-dessous).
//!
//! # Ordre des systèmes (preuve : `make schedule_order_diff`, vide)
//!
//! `GgrsSchedule`, dans cet ordre de déclaration :
//! 1. [`apply_scenario_powerup_placements`] : `RollbackSystemSet::Effects`, avant
//!    `powerup_pickup_detect_system` ;
//! 2. [`apply_scenario_item_placements`] : `Effects`, après (1), avant `item_consumable_pickup_system` ;
//! 3. [`apply_scenario_chest_placements`] : `Effects`, après (2), avant `item_consumable_pickup_system` ;
//! 4. [`apply_scenario_shelf_placements`] : `Effects`, après (3), avant `item_consumable_pickup_system` et
//!    avant `commands::apply_game_commands_pickups` (les net ids des commandes d'édition suivent ceux du
//!    profil) ;
//! 5. [`apply_scenario_character_placements`] : `RollbackSystemSet::EnemySpawning`, après les spawns de
//!    vague et de spawner, avant `commands::apply_game_commands_spawn`, seulement s'il y a un placement.
//!
//! `Update` : [`apply_player_overrides`], une seule fois, quand tous les joueurs du profil existent.
//!
//! Les noms des systèmes sont ceux d'avant le déplacement : `schedule_order_diff` normalise le chemin de
//! module mais pas le nom court, et il doit rester vide.
//!
//! Un placement s'applique à `frame == at_frame` : rejoué à l'identique après un rollback (le net id est
//! alloué à cette frame, `GgrsNetIdFactory` est rollback).

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::GgrsSchedule;
use combat::damage::Defenses;
use combat::inventory::AmmoReserves;
use game::{
    character::player::Player,
    effects_runtime::{EffectState, Effects},
    progression::{Mutations, ProgressionTable},
    replay::{
        CharacterPlacement, ChestPlacement, ItemPlacement, ModifierSpec, PlayerScript,
        PowerUpPlacement, Scenario, ShelfPlacement,
    },
    system_set::RollbackSystemSet,
    weapons::{
        melee::{self, MeleeWeapon},
        spawn_weapon_for_player, Weapon, WeaponInventory, WeaponRegistry,
    },
};
use run::currency::Currency;
use sim_core::{
    modifier::{Modifier, ModifierSource, Modifiers},
    tag::Tags,
};
use utils::{frame::FrameCount, net_id::GgrsNetIdFactory};

use super::local::LevelSpawnAssets;

// --- Données --------------------------------------------------------------------------------

/// Réglages d'un joueur : les champs de [`PlayerScript`] qui ne sont pas des inputs. Tags, immunités,
/// modificateurs de stats, choix d'arme, argent, mutations, dash, objets, bombe, visée.
#[derive(Debug, Clone, Default)]
pub struct PlayerProfile {
    pub tags: Vec<String>,
    pub immune_to: Vec<String>,
    pub modifiers: Vec<ModifierSpec>,
    /// Voir `PlayerScript::weapon` : le joueur n'a que cette arme.
    pub weapon: Option<String>,
    /// T2.3, chantier C5 v1 : voir `PlayerScript::currency`.
    pub currency: Option<u32>,
    /// T1.10 : voir `PlayerScript::mutations`.
    pub mutations: Vec<String>,
    /// M2-T2 : voir `PlayerScript::dash`.
    pub dash: Option<game::character::movement::DashConfig>,
    /// M2-T17 : voir `PlayerScript::items`.
    pub items: Vec<String>,
    /// M3-T4 : voir `PlayerScript::bomb`.
    pub bomb: Option<world::BombDef>,
    /// Suivi Q17 : voir `PlayerScript::aim`.
    pub aim: Option<sim_core::aim::AimMode>,
}

impl PlayerProfile {
    /// Les réglages de `script` (sans ses inputs ni son bot).
    pub fn from_script(script: &PlayerScript) -> Self {
        Self {
            tags: script.tags.clone(),
            immune_to: script.immune_to.clone(),
            modifiers: script.modifiers.clone(),
            weapon: script.weapon.clone(),
            currency: script.currency,
            mutations: script.mutations.clone(),
            dash: script.dash.clone(),
            items: script.items.clone(),
            bomb: script.bomb,
            aim: script.aim,
        }
    }

    /// `true` si rien n'est à poser.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
            && self.immune_to.is_empty()
            && self.modifiers.is_empty()
            && self.weapon.is_none()
            && self.currency.is_none()
            && self.mutations.is_empty()
            && self.dash.is_none()
            && self.items.is_empty()
            && self.bomb.is_none()
            && self.aim.is_none()
    }
}

/// Profil complet d'une partie : un [`PlayerProfile`] par joueur (indexé par handle) et les placements.
#[derive(Debug, Clone, Default)]
pub struct Profile {
    pub players: Vec<PlayerProfile>,
    pub characters: Vec<CharacterPlacement>,
    pub powerups: Vec<PowerUpPlacement>,
    pub items: Vec<ItemPlacement>,
    pub chests: Vec<ChestPlacement>,
    pub shelves: Vec<ShelfPlacement>,
}

impl Profile {
    /// Le profil d'un scénario : `players[].{tags, …}`, `characters`, `powerups`, `items`, `chests`,
    /// `shop.shelves`.
    pub fn from_scenario(scenario: &Scenario) -> Self {
        Self {
            players: scenario
                .players
                .iter()
                .map(PlayerProfile::from_script)
                .collect(),
            characters: scenario.characters.clone(),
            powerups: scenario.powerups.clone(),
            items: scenario.items.clone(),
            chests: scenario.chests.clone(),
            shelves: scenario.shop.shelves.clone(),
        }
    }
}

/// Placements de personnages (T1.13, `Scenario::characters`, `docs/conventions.md` §28).
#[derive(Resource)]
pub struct ScenarioCharacterPlacements(pub Vec<CharacterPlacement>);

/// Placements de power-ups (T2.5, `Scenario::powerups`).
#[derive(Resource)]
pub struct ScenarioPowerUpPlacements(pub Vec<PowerUpPlacement>);

/// Placements d'objets (M2-T0b, `Scenario::items`).
#[derive(Resource)]
pub struct ScenarioItemPlacements(pub Vec<ItemPlacement>);

/// Placements de coffres (M2-T5, `Scenario::chests`).
#[derive(Resource)]
pub struct ScenarioChestPlacements(pub Vec<ChestPlacement>);

/// Placements de présentoirs (M2-T6, `Scenario::shop`).
#[derive(Resource)]
pub struct ScenarioShelfPlacements(pub Vec<ShelfPlacement>);

/// Réglages des joueurs, indexés par handle ([`apply_player_overrides`]).
#[derive(Resource)]
pub struct PlayerOverrides(pub Vec<PlayerProfile>);

/// Placements refusés (personnage ou variante inconnus), par index : à rapporter en échec en fin
/// de partie. Hors rollback ; un placement rejoué par un rollback n'ajoute pas de doublon.
#[derive(Resource, Default)]
pub struct PlacementErrors(pub BTreeMap<usize, String>);

/// Index (dans `Scenario::characters`) du placement qui a créé ce personnage, pour
/// `EntityRef::Placed`. Composant d'observation, **hors rollback** (non enregistré, donc hors
/// checksum et hors trace) : reposé à l'identique quand un rollback rejoue le placement.
#[derive(Component)]
pub struct ScriptedPlacement(pub usize);

// --- Plugin ---------------------------------------------------------------------------------

/// Pose un [`Profile`] : ressources, systèmes de placement dans `GgrsSchedule` (ordre du module) et
/// [`apply_player_overrides`] dans `Update`. À ajouter après `LdtkLocalGamePlugin` (qui enregistre
/// `commands::apply_game_commands_*`, ordonnés par rapport aux systèmes d'ici).
pub struct ProfilePlugin(pub Profile);

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        let profile = &self.0;
        app.insert_resource(ScenarioPowerUpPlacements(profile.powerups.clone()))
            .add_systems(
                GgrsSchedule,
                apply_scenario_powerup_placements
                    .before(game::powerups::powerup_pickup_detect_system)
                    .in_set(RollbackSystemSet::Effects),
            )
            // M2-T0b : placements scriptés d'objets
            .insert_resource(ScenarioItemPlacements(profile.items.clone()))
            .add_systems(
                GgrsSchedule,
                apply_scenario_item_placements
                    .after(apply_scenario_powerup_placements)
                    .before(game::items::item_consumable_pickup_system)
                    .in_set(RollbackSystemSet::Effects),
            )
            // M2-T5 : placements scriptés de coffres
            .insert_resource(ScenarioChestPlacements(profile.chests.clone()))
            .add_systems(
                GgrsSchedule,
                apply_scenario_chest_placements
                    .after(apply_scenario_item_placements)
                    .before(game::items::item_consumable_pickup_system)
                    .in_set(RollbackSystemSet::Effects),
            )
            // M2-T6 : présentoirs scriptés
            .insert_resource(ScenarioShelfPlacements(profile.shelves.clone()))
            .add_systems(
                GgrsSchedule,
                apply_scenario_shelf_placements
                    .after(apply_scenario_chest_placements)
                    .before(game::items::item_consumable_pickup_system)
                    // Commandes d'édition (K5-2) : leurs net ids viennent après ceux des placements du profil.
                    .before(super::commands::apply_game_commands_pickups)
                    .in_set(RollbackSystemSet::Effects),
            )
            // T1.13 : placements scriptés de personnages.
            .insert_resource(ScenarioCharacterPlacements(profile.characters.clone()))
            .init_resource::<PlacementErrors>()
            .add_systems(
                GgrsSchedule,
                apply_scenario_character_placements
                    .run_if(|placements: Res<ScenarioCharacterPlacements>| !placements.0.is_empty())
                    // Après les spawns du jeu de la même frame (ordre des net ids déterministe).
                    .after(game::waves::systems::wave_spawning_system)
                    .after(game::character::enemy::spawning::enemy_spawn_from_spawners_system)
                    .before(super::commands::apply_game_commands_spawn)
                    .in_set(RollbackSystemSet::EnemySpawning),
            )
            .insert_resource(PlayerOverrides(profile.players.clone()))
            .add_systems(Update, apply_player_overrides);
    }
}

// --- Placements (GgrsSchedule) ---------------------------------------------------------------

/// Pose un personnage du profil, par le chemin des `CharacterSpawn` de carte
/// (`local::spawn_character`) : `Err` (message de `PlacementErrors`) si le personnage ou sa variante
/// sont inconnus. `index` est le rang du placement (message d'erreur, `ScriptedPlacement`).
pub fn spawn_character_placement(
    commands: &mut Commands,
    assets: &LevelSpawnAssets,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    floor: u32,
    index: usize,
    placement: &CharacterPlacement,
) -> Result<Entity, String> {
    let Some(config) = assets.character_configs.get(&placement.character) else {
        return Err(format!(
            "placement {index} : personnage inconnu « {} »",
            placement.character
        ));
    };
    if let Some(variant) = &placement.variant {
        let known = config
            .variants
            .as_ref()
            .is_some_and(|variants| variants.table.contains_key(variant));
        if !known {
            return Err(format!(
                "placement {index} : variante « {variant} » inconnue de « {} »",
                placement.character
            ));
        }
    }
    let position = fixed_math::FixedVec3::new(placement.x, placement.y, fixed_math::FIXED_ZERO);
    super::local::spawn_character(
        commands,
        assets,
        id_factory,
        &placement.character,
        position,
        placement.team,
        placement.variant.as_deref(),
        assets.difficulty.current(),
        assets.difficulty.variant_context_at(floor),
    )
    .ok_or_else(|| {
        format!(
            "placement {index} : personnage « {} » non créé",
            placement.character
        )
    })
}

/// Placements de personnages (T1.13, `Scenario::characters`, `docs/conventions.md`
/// §28) : à `frame == at_frame`, dans l'ordre de déclaration, crée le personnage par le chemin
/// des `CharacterSpawn` de carte (`local::spawn_character`). Dans
/// `GgrsSchedule`, `RollbackSystemSet::EnemySpawning`, comme un spawn de vague : rejoué à
/// l'identique après un rollback (le net id est alloué à cette frame). Sans placement, le
/// système ne tourne pas (condition d'exécution).
pub fn apply_scenario_character_placements(
    frame: Res<FrameCount>,
    placements: Res<ScenarioCharacterPlacements>,
    assets: LevelSpawnAssets,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut errors: ResMut<PlacementErrors>,
    // M3-T7 : étage courant (contexte de `chance` des variantes)
    floor: Option<Res<run::FloorState>>,
) {
    let floor = floor.as_deref().map_or(0, |floor| floor.index);
    for (index, placement) in placements
        .0
        .iter()
        .enumerate()
        .filter(|(_, p)| p.at_frame == frame.frame)
    {
        match spawn_character_placement(
            &mut commands,
            &assets,
            &mut id_factory,
            floor,
            index,
            placement,
        ) {
            Ok(entity) => {
                commands.entity(entity).insert(ScriptedPlacement(index));
            }
            Err(message) => {
                errors.0.insert(index, message);
            }
        }
    }
}

/// Placements de power-ups (T2.5, `Scenario::powerups`) : fait apparaître un
/// power-up à une position et une frame exactes, sans dépendre d'une carte LDtk ni du
/// tirage RNG `loot` (voir la doc de `game::replay::PowerUpPlacement`). Contrairement aux
/// réglages de joueur ([`apply_player_overrides`], `Update`, une seule application avant la
/// première frame), celui-ci tourne dans `GgrsSchedule` : un placement doit apparaître à
/// une frame précise **pendant** la simulation, comme un spawn d'ennemi de vague — la
/// condition `frame.frame == at_frame` est naturellement rollback-safe (même raisonnement
/// que `waves::systems::wave_spawning_system` : re-simuler la même frame retrouve le même
/// `GgrsNetIdFactory`, donc le même id), pas besoin d'un `Local<bool>` par placement.
///
/// `RollbackSystemSet::Effects`, `.before(game::powerups::powerup_pickup_detect_system)` :
/// un power-up placé exactement sur un joueur déjà présent est ramassable la **même**
/// frame où il apparaît (contrairement à un drop à la mort, posé dans
/// `DeathManagement`, un set plus tard — un frame de latence avant que
/// `Effects` ne le revoie, sans conséquence observable).
pub fn apply_scenario_powerup_placements(
    frame: Res<FrameCount>,
    placements: Res<ScenarioPowerUpPlacements>,
    table: Res<game::powerups::PowerUpsTable>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let Some(config) = &table.0 else {
        return;
    };
    for placement in &placements.0 {
        if frame.frame != placement.at_frame {
            continue;
        }
        let Some(definition) = config.powerups.get(&placement.id) else {
            continue;
        };
        let position = fixed_math::FixedVec3::new(placement.x, placement.y, fixed_math::FIXED_ZERO);
        let expires_at_frame = frame.frame.saturating_add(definition.lifetime_frames);
        game::powerups::spawn_powerup_pickup(
            &mut commands,
            placement.id.clone(),
            position,
            expires_at_frame,
            &mut id_factory,
        );
    }
}

/// Placements d'objets (M2-T0b, `Scenario::items`) : même principe que
/// [`apply_scenario_powerup_placements`], dans `RollbackSystemSet::Effects` avant le ramassage.
pub fn apply_scenario_item_placements(
    frame: Res<FrameCount>,
    placements: Res<ScenarioItemPlacements>,
    table: Res<items::ItemTable>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    for placement in &placements.0 {
        if frame.frame != placement.at_frame {
            continue;
        }
        let position = fixed_math::FixedVec3::new(placement.x, placement.y, fixed_math::FIXED_ZERO);
        if game::items::spawn_item_pickup(
            &mut commands,
            &table,
            placement.id.clone(),
            position,
            &mut id_factory,
        )
        .is_none()
        {
            warn!("placement d'objet : « {} » inconnu", placement.id);
        }
    }
}

/// Placements de coffres (M2-T5, `Scenario::chests`).
#[allow(clippy::too_many_arguments)]
pub fn apply_scenario_chest_placements(
    frame: Res<FrameCount>,
    placements: Res<ScenarioChestPlacements>,
    tables: Res<items::LootTables>,
    floors: Res<game::loot::FloorLootTables>,
    item_table: Res<items::ItemTable>,
    unlocks: Res<items::UnlockTable>,
    active: Res<items::RunUnlocks>,
    run_seed: Option<Res<bevy_fixed::rng::RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut pools: ResMut<items::PoolState>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    let (run_seed, floor) = game::loot::seed_context(&run_seed, &floor);
    let ctx = game::loot::SlotContext {
        frame: frame.frame,
        tables: &tables,
        items: &item_table,
        unlocks: &unlocks,
        active: &active,
        run_seed,
        floor,
    };
    for placement in &placements.0 {
        if frame.frame != placement.at_frame {
            continue;
        }
        let position = fixed_math::FixedVec3::new(placement.x, placement.y, fixed_math::FIXED_ZERO);
        let chest = items::Chest {
            key: placement.key.clone(),
            table: placement.table.clone(),
            rarity: placement.rarity,
            locked: placement.locked,
            opened: false,
        };
        // M3-T8 (§73) : comme un coffre de salle, l'objet d'un pool qui s'épuise est assigné à la
        // pose (idempotent : une seconde pose de la même clé donne le même objet).
        let table_id = game::loot::chest_table_id(&chest, &floors, floor).to_string();
        game::loot::assign_slot(&ctx, &mut pools, &table_id, chest.rarity, false, &chest.key);
        game::loot::spawn_chest(&mut commands, &mut id_factory, chest, position);
    }
}

/// Placements de présentoirs (M2-T6, `Scenario::shop`).
pub fn apply_scenario_shelf_placements(
    frame: Res<FrameCount>,
    placements: Res<ScenarioShelfPlacements>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    for placement in &placements.0 {
        if frame.frame != placement.at_frame {
            continue;
        }
        let position = fixed_math::FixedVec3::new(placement.x, placement.y, fixed_math::FIXED_ZERO);
        game::shop::spawn_shelf(
            &mut commands,
            &mut id_factory,
            items::Shelf {
                key: placement.key.clone(),
                item_id: placement.item.clone(),
                price: placement.price,
                sold: false,
                retry_after: 0,
                cost: placement.cost.clone(),
            },
            position,
        );
    }
}

// --- Réglages d'un joueur ---------------------------------------------------------------------

/// Tables de contenu dont dépend [`apply_player_profile`]. `progression` et `items` valent `None`
/// tant que leur table n'est pas posée (chargement) : un réglage qui en a besoin est alors refusé.
pub struct ProfileTables<'a> {
    pub progression: Option<&'a ProgressionTable>,
    pub items: Option<&'a items::ItemTable>,
    pub weapons: &'a WeaponRegistry,
}

/// Le joueur sur lequel un [`PlayerProfile`] est posé.
pub struct PlayerTarget<'a> {
    pub entity: Entity,
    pub handle: usize,
    /// Enfants du joueur (ses armes y sont rattachées).
    pub children: &'a Children,
    pub effects: Option<&'a Effects>,
    pub modifiers: Option<&'a Modifiers>,
}

/// Pose `profile` sur un joueur déjà créé : composants `Tags`/`Defenses`/`Modifiers`/`Inventory`/
/// `WeaponInventory`/`AmmoReserves`/`Currency`/`Effects`/`Mutations`, dash, bombe, visée.
/// `is_weapon(child)` dit si un enfant du joueur est une arme (à distance ou de mêlée) : `weapon`
/// retire toutes celles que `create_player` a posées avant d'en créer une seule.
///
/// `Tags`/`Defenses` sont hors rollback (voir leur doc) : les poser ici, avant la première
/// frame simulée, suffit — ils ne sont jamais mutés ensuite. `Modifiers`/`WeaponInventory`
/// sont en rollback, mais poser leur valeur initiale avant la première frame simulée est
/// tout aussi correct : elle est identique sur tous les clients (même profil), donc fait
/// partie de l'état de départ comme les autres — `weapon` n'ajoute donc rien de nouveau au
/// checksum GGRS, ce n'est qu'un choix d'inventaire à la création (voir la doc de
/// `PlayerScript::weapon`).
///
/// `weapon` : le joueur apparaît avec **cette seule arme** (à distance ou de mêlée, id du
/// registre), à la place de la totalité de son équipement par défaut — exclusivité complète
/// (les armes à distance par défaut du personnage *et* son arme de mêlée par défaut,
/// `bare_hands`, sont retirées, même si `weapon` désigne une arme à distance). Décision
/// (rapport T2.10) : plus simple et plus sûr pour un scénario généré isolé par arme (aucun
/// risque qu'un coup de mêlée ou un tir parasite d'une arme de départ non retirée fausse
/// `EntityHits`) que de ne retirer que la catégorie choisie. Cherche `weapon_id` d'abord
/// dans `weapons.ron` (arme à distance), puis dans `melee_weapons.ron` (arme de mêlée).
///
/// `Err` (rien n'est posé) : arme, objet ou mutation inconnus du contenu, ou table de mutations ou
/// d'objets absente alors que le profil en demande une.
pub fn apply_player_profile(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    tables: &ProfileTables,
    target: &PlayerTarget,
    profile: &PlayerProfile,
    is_weapon: impl Fn(Entity) -> bool,
) -> Result<(), String> {
    let PlayerTarget {
        entity,
        handle,
        children,
        effects,
        modifiers: existing_modifiers,
    } = *target;

    // Validation d'abord : un `Err` ne laisse aucune commande en file.
    let item_defs = if profile.items.is_empty() {
        None
    } else {
        let table = tables.items.ok_or_else(|| {
            format!("PlayerScript::items : table des objets absente (joueur {handle})")
        })?;
        for id in &profile.items {
            if !table.0.contains_key(id) {
                return Err(format!(
                    "PlayerScript::items : objet inconnu « {id} » pour le joueur {handle}"
                ));
            }
        }
        Some(table)
    };
    if let Some(weapon_id) = &profile.weapon {
        if !tables.weapons.weapons.contains_key(weapon_id)
            && !tables.weapons.melee.contains_key(weapon_id)
        {
            return Err(format!(
                "PlayerScript::weapon : arme inconnue « {weapon_id} » pour le joueur {handle} (ni dans weapons.ron ni dans melee_weapons.ron)"
            ));
        }
    }
    let mutation_table = if profile.mutations.is_empty() {
        None
    } else {
        let table = tables.progression.ok_or_else(|| {
            format!("PlayerScript::mutations : table des mutations absente (joueur {handle})")
        })?;
        for id in &profile.mutations {
            if !table.mutations.contains_key(id) {
                return Err(format!(
                    "PlayerScript::mutations : mutation inconnue « {id} » pour le joueur {handle}"
                ));
            }
        }
        Some(table)
    };

    if let Some(dash) = &profile.dash {
        commands.entity(entity).insert((
            game::character::dash::DashOverride(dash.clone()),
            combat::actors::DashExtra::with_charges(dash.charges.unwrap_or(0)),
        ));
    }
    if let Some(def) = profile.bomb {
        commands.entity(entity).insert(world::BombCarrier::new(def));
    }
    // Suivi Q17 : `Free` retire la visée du personnage (scénarios de contenu, voir
    // `PlayerScript::aim`), `FourWay` la pose.
    match profile.aim {
        Some(sim_core::aim::AimMode::Free) => {
            commands.entity(entity).remove::<sim_core::aim::AimMode>();
        }
        Some(aim) => {
            commands.entity(entity).insert(aim);
        }
        None => {}
    }
    if !profile.tags.is_empty() {
        commands
            .entity(entity)
            .insert(Tags::parse(profile.tags.clone()));
    }
    if !profile.immune_to.is_empty() {
        commands.entity(entity).insert(Defenses {
            immune_to: Tags::parse(profile.immune_to.clone()),
            resistances: Default::default(),
        });
    }
    let mut built = existing_modifiers.cloned().unwrap_or_default();
    let mut modifiers_changed = false;
    if !profile.modifiers.is_empty() {
        built = Modifiers::default();
        modifiers_changed = true;
        for (i, spec) in profile.modifiers.iter().enumerate() {
            built.push(Modifier {
                stat: spec.stat.clone(),
                op: spec.op,
                value: spec.value,
                // Source stable par joueur/index : suffit pour un scénario (jamais
                // retiré par `remove_by_source`), et distingue les modificateurs entre
                // eux si un futur scénario en pose plusieurs sur le même joueur.
                source: ModifierSource::Named(format!("scenario_player_{handle}_{i}")),
                until: spec.until,
            });
        }
    }
    // M2-T17 (§54) : objets tenus dès le départ, posés comme un ramassage (passif : inventaire
    // et modificateurs `item:<id>` ; actif : emplacement actif **chargé**).
    if let Some(table) = item_defs {
        let mut inventory = items::Inventory::default();
        for id in &profile.items {
            let def = &table.0[id];
            match def.kind {
                items::ItemKind::Passive => {
                    inventory.passives.push(id.clone());
                    for modifier in &def.modifiers {
                        built.push(modifier.to_modifier(id));
                    }
                }
                items::ItemKind::Active(charge) => {
                    let mut slot = items::ActiveSlot::new(id.clone());
                    slot.charge = charge.required();
                    inventory.active = Some(slot);
                }
                items::ItemKind::Consumable => {
                    *inventory.consumables.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }
        modifiers_changed = true;
        commands.entity(entity).insert(inventory);
    }
    if modifiers_changed {
        commands.entity(entity).insert(built);
    }
    if let Some(weapon_id) = &profile.weapon {
        // Exclusivité complète (voir la doc de la fonction) : retire toute arme (à
        // distance ou de mêlée) déjà posée par `create_player` avant d'en spawner une
        // seule.
        for child in children.iter() {
            if is_weapon(child) {
                commands.entity(child).despawn();
            }
        }
        let mut inventory = WeaponInventory::default();
        // Réserve de munitions (T2.2) recalculée pour cette seule arme — pas la somme
        // des armes de départ du personnage (`create_player`), qui n'existent plus dans
        // l'inventaire une fois l'exclusivité appliquée ci-dessus. Vide si `weapon_id`
        // est une arme de mêlée (pas de munitions).
        let mut ammo_reserves = AmmoReserves::new();
        if let Some(weapon_asset) = tables.weapons.weapons.get(weapon_id) {
            let (ammo_type, amount) = game::weapons::default_mode_ammo_contribution(weapon_asset);
            ammo_reserves.add(ammo_type, amount);
            spawn_weapon_for_player(
                commands,
                true,
                entity,
                weapon_asset.clone(),
                &mut inventory,
                id_factory,
                None,
            );
        } else if let Some(melee_asset) = tables.weapons.melee.get(weapon_id) {
            melee::spawn_melee_weapon_for_character(
                commands,
                entity,
                melee_asset.clone(),
                id_factory,
            );
        }
        // Remplace l'inventaire par défaut (potentiellement plusieurs armes à distance)
        // par celui-ci (une seule arme, ou aucune si `weapon_id` est une arme de mêlée).
        commands.entity(entity).insert(inventory);
        commands.entity(entity).insert(ammo_reserves);
    }
    // T2.3, chantier C5 v1 (scénarios d'achat `buy_door`/`buy_wall_weapon`/`buy_perk`) :
    // remplace le `Currency` posé par `create_player`
    // (`CharacterConfig::starting_currency`), avant la première frame simulée — même
    // raisonnement que `weapon` ci-dessus (voir sa doc).
    if let Some(amount) = profile.currency {
        commands.entity(entity).insert(Currency::new(amount));
    }
    // T1.10 : mutations imposées, prises dans l'ordre (effets ajoutés à ceux du joueur).
    if let Some(table) = mutation_table {
        let mut all = effects.cloned().unwrap_or_default();
        for id in &profile.mutations {
            all.0.extend(table.mutations[id].iter().cloned());
        }
        commands.entity(entity).insert((
            all,
            EffectState::default(),
            Mutations(profile.mutations.clone()),
        ));
    }
    Ok(())
}

/// Pose les réglages de [`PlayerOverrides`] sur les joueurs une fois créés. Contrairement à
/// `apply_weapon_overrides` du runner (qui doit s'appliquer *avant* la création des joueurs, qui
/// copient la config de leurs armes), celui-ci s'applique *après* : il attend que les
/// entités `Player` existent (et que les tables de mutations et d'objets soient posées si un
/// réglage en a besoin), puis applique [`apply_player_profile`] une seule fois (`Local<bool>`).
///
/// Panique sur un contenu inconnu (arme, objet, mutation) : faute de contenu plutôt que silencieuse,
/// même style que `apply_weapon_overrides`.
#[allow(clippy::too_many_arguments)]
pub fn apply_player_overrides(
    overrides: Res<PlayerOverrides>,
    mut commands: Commands,
    players: Query<(
        Entity,
        &Player,
        &Children,
        Option<&Effects>,
        Option<&Modifiers>,
    )>,
    progression: Option<Res<ProgressionTable>>,
    item_table: Option<Res<items::ItemTable>>,
    ranged_children: Query<(), With<Weapon>>,
    melee_children: Query<(), With<MeleeWeapon>>,
    weapon_registry: Res<WeaponRegistry>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    if overrides.0.iter().all(PlayerProfile::is_empty) {
        *applied = true;
        return;
    }
    if players.iter().count() < overrides.0.len() {
        return; // les joueurs ne sont pas encore tous créés
    }
    if progression.is_none() && overrides.0.iter().any(|o| !o.mutations.is_empty()) {
        return; // table des mutations pas encore résolue (`OnEnter(GameLoading)`)
    }
    if item_table.is_none() && overrides.0.iter().any(|o| !o.items.is_empty()) {
        return; // table des objets posée au chargement de la carte
    }
    let tables = ProfileTables {
        progression: progression.as_deref(),
        items: item_table.as_deref(),
        weapons: &weapon_registry,
    };
    for (entity, player, children, effects, existing_modifiers) in players.iter() {
        let Some(profile) = overrides.0.get(player.handle) else {
            continue;
        };
        let target = PlayerTarget {
            entity,
            handle: player.handle,
            children,
            effects,
            modifiers: existing_modifiers,
        };
        apply_player_profile(
            &mut commands,
            &mut id_factory,
            &tables,
            &target,
            profile,
            |child| ranged_children.get(child).is_ok() || melee_children.get(child).is_ok(),
        )
        .unwrap_or_else(|message| panic!("{message}"));
    }
    *applied = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Un joueur (un enfant, pour `Children`) et la pose de `profile` dessus, sans registre ni table.
    fn apply(world: &mut World, entity: Entity, profile: PlayerProfile) -> Result<(), String> {
        world.init_resource::<GgrsNetIdFactory>();
        world.init_resource::<WeaponRegistry>();
        world
            .run_system_once(
                move |mut commands: Commands,
                      mut ids: ResMut<GgrsNetIdFactory>,
                      registry: Res<WeaponRegistry>,
                      players: Query<&Children>| {
                    let tables = ProfileTables {
                        progression: None,
                        items: None,
                        weapons: &registry,
                    };
                    let target = PlayerTarget {
                        entity,
                        handle: 3,
                        children: players.get(entity).unwrap(),
                        effects: None,
                        modifiers: None,
                    };
                    apply_player_profile(
                        &mut commands,
                        &mut ids,
                        &tables,
                        &target,
                        &profile,
                        |_| false,
                    )
                },
            )
            .unwrap()
    }

    fn player(world: &mut World) -> Entity {
        world.spawn(()).with_children(|p| {
            p.spawn(());
        });
        world
            .query::<(Entity, &Children)>()
            .iter(world)
            .next()
            .unwrap()
            .0
    }

    #[test]
    fn un_profil_pose_argent_tags_et_visee() {
        let mut world = World::new();
        let entity = player(&mut world);
        let profile = PlayerProfile {
            currency: Some(500),
            tags: vec!["lourd".into()],
            aim: Some(sim_core::aim::AimMode::FourWay),
            ..Default::default()
        };
        apply(&mut world, entity, profile).unwrap();
        assert_eq!(world.get::<Currency>(entity), Some(&Currency::new(500)));
        assert_eq!(world.get::<Tags>(entity).map(Tags::len), Some(1));
        assert_eq!(
            world.get::<sim_core::aim::AimMode>(entity),
            Some(&sim_core::aim::AimMode::FourWay)
        );
    }

    /// Un contenu inconnu est une erreur nommant le joueur, et **rien** n'est mis en file : l'argent,
    /// pourtant valide, n'est pas posé (un appelant qui ne panique pas, ex. une commande vocale,
    /// n'obtient pas un joueur à moitié équipé).
    #[test]
    fn un_contenu_inconnu_est_refuse_sans_rien_poser() {
        for (profile, attendu) in [
            (
                PlayerProfile {
                    weapon: Some("fantome".into()),
                    ..Default::default()
                },
                "arme inconnue « fantome » pour le joueur 3",
            ),
            (
                PlayerProfile {
                    items: vec!["fantome".into()],
                    ..Default::default()
                },
                "table des objets absente",
            ),
            (
                PlayerProfile {
                    mutations: vec!["fantome".into()],
                    ..Default::default()
                },
                "table des mutations absente",
            ),
        ] {
            let mut world = World::new();
            let entity = player(&mut world);
            let profile = PlayerProfile {
                currency: Some(500),
                ..profile
            };
            let error = apply(&mut world, entity, profile).unwrap_err();
            assert!(error.contains(attendu), "{error}");
            assert_eq!(world.get::<Currency>(entity), None);
        }
    }
}
