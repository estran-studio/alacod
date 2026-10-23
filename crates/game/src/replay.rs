//! Format RON des scénarios et replays (`tests/scenarios/*.ron`, enregistrements).
//!
//! ```ron
//! Scenario(
//!     frames: 600,
//!     players: [
//!         (inputs: [
//!             (from: 0, to: 120, buttons: [Right, Fire], pan: (100, 0)),
//!         ]),
//!     ],
//!     expect: [
//!         PlayerAlive(handle: 0, at_frame: 300),
//!         KillsAtLeast(kills: 1, at_frame: 600),
//!     ],
//! )
//! ```
//!
//! Les frames d'un segment sont celles où l'input est *lu* : GGRS l'applique après
//! le délai d'input de la session.
//!
//! Joué par `crates/scenario` ; écrit par l'enregistrement (`crate::recording`).

use crate::character::player::input::{
    BoxInput, InputSegment, ScriptedInputs, INPUT_BLANK, INPUT_BOMB, INPUT_CHOICE_A,
    INPUT_CHOICE_B, INPUT_CHOICE_C, INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON, INPUT_FORCE_CRASH,
    INPUT_INTERACTION, INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE, INPUT_UP, INPUT_USE_ACTIVE,
};
use bevy_fixed::fixed_math::Fixed;
pub use combat::weapons::expectations::{
    DistanceTarget, EntityKind, EntityRef, Expectation, RunStepExpectation,
};
use serde::{Deserialize, Serialize};
use sim_core::ammo::AmmoType;
use sim_core::damage::FriendlyFire;
use sim_core::modifier::ModifierOp;
use sim_core::stats::StatId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Jeu auquel ce scénario appartient (dossier `games/<game>/`).
    #[serde(default = "default_game")]
    pub game: String,
    /// Map LDtk, relative au dossier des assets du jeu.
    #[serde(default = "default_map")]
    pub map: String,
    #[serde(default = "default_map_seed")]
    pub map_seed: i32,
    /// Nombre de frames simulées.
    pub frames: u32,
    /// Un script par joueur ; le joueur `i` a le handle GGRS `i`.
    pub players: Vec<PlayerScript>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect: Vec<Expectation>,
    /// Modifications de la config des armes pour ce scénario (ex. moins de chargeurs pour
    /// tester leur épuisement en quelques secondes).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weapon_overrides: Vec<WeaponOverride>,
    /// Modification de la config des vagues pour ce scénario (T2.1, chantier B4b, bench
    /// `bench_horde` : le plus d'ennemis possible dès la première vague). Même idée que
    /// `weapon_overrides` mais appliquée à `waves::config::WaveConfig` ; voir
    /// `scenario::runner::apply_wave_overrides`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wave_overrides: Option<WaveOverride>,
    /// Invariants vérifiés à chaque frame par le runner ; tous actifs par défaut.
    #[serde(default, skip_serializing_if = "Invariants::tous_actifs")]
    pub invariants: Invariants,
    /// Placements scriptés de power-ups (T2.5, chantier C1 v0) : fait apparaître un
    /// power-up à une position et une frame exactes, sans dépendre d'une carte LDtk ni du
    /// tirage RNG `loot` — voir [`PowerUpPlacement`] et
    /// `map_ldtk::game::profile::apply_scenario_powerup_placements`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub powerups: Vec<PowerUpPlacement>,
    /// Placements scriptés d'objets (M2-T0b, `docs/conventions.md` §36) : même principe que
    /// `powerups`, pour les objets de `items/*.ron` ([`ItemPlacement`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemPlacement>,
    /// Placements scriptés de coffres (M2-T5, `docs/conventions.md` §48) : même principe que
    /// `items`, pour les coffres ([`ChestPlacement`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chests: Vec<ChestPlacement>,
    /// Force `RoomRewardDef::chance` de tous les pools pour ce scénario (M2-T5), pour prouver la
    /// récompense de salle sans dépendre du tirage. `None` : la chance du pool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_reward_chance_override: Option<Fixed>,
    /// Boutique et monnaie au sol (M2-T6, `docs/conventions.md` §50) : présentoirs et douilles
    /// posés par le scénario.
    #[serde(default, skip_serializing_if = "ShopSetup::is_empty")]
    pub shop: ShopSetup,
    /// Écrit les profils de méta-progression à la fin de la run, dans un dossier temporaire
    /// (M2-T0c, `docs/conventions.md` §38) ; faux par défaut : un scénario n'écrit rien.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub profile: bool,
    /// Profils imposés avant la première frame et achats hors simulation (M2-T12, §51), dans le
    /// dossier temporaire de `profile: true`. `None` : les profils se constituent seuls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_setup: Option<ProfileSetup>,
    /// Démarrer au hub (M2-T13, §57). `None` : la partie du scénario, comme avant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hub: Option<HubSetup>,
    /// Les bots `explorateur` ouvrent les coffres et achètent les présentoirs qu'ils peuvent payer
    /// (D62, `bots::explorer::ExplorerOptions::shopping`, l'option `--shopping` d'`alacod-sim`).
    /// Faux par défaut : aucune trace de bot existante ne bouge.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shopping: bool,
    /// M3-T14 : politique de l'`explorateur` (§79). `None` : celle de `Default` (+ `shopping` ci-dessus).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explorer: Option<ExplorerPolicy>,
    /// Force `PowerUpsConfig::drop_chance` pour ce scénario (T2.5), pour prouver le chemin
    /// « drop à la mort » sans dépendre du tirage réel du jeu (scénario
    /// `powerup_drop_on_kill`) — voir `scenario::runner::apply_powerup_drop_chance_override`.
    /// `None` (défaut) : `drop_chance` de `items/powerups.ron` du jeu, inchangé pour tous
    /// les scénarios existants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub powerup_drop_chance_override: Option<Fixed>,
    /// Mode `Floors` imposé (T1.8) : id d'une séquence de niveaux du dossier `Floors` du jeu
    /// (`content::registry::FloorsConfigId`). La partie joue alors cette séquence quel que
    /// soit `entry.mode` (`game::run_state::FloorsOverride`), et `map` est ignorée (le premier
    /// niveau de la séquence la remplace). `None` (défaut) : mode du manifeste.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floors: Option<String>,
    /// Progression imposée (T1.10) : id d'un fichier du kind `Progression` du jeu, prioritaire
    /// sur `entry.progression` du manifeste (`game::progression::ProgressionOverride`).
    /// `None` (défaut) : celle du manifeste (aucune pour le testbed et zombies).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progression: Option<String>,
    /// Horloges actives (T1.9, ids du kind `Clock`, `game::clock::ClocksOverride`). `None`
    /// (défaut) : `entry.clocks` du manifeste, sinon aucune.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clocks: Option<Vec<String>>,
    /// Difficulté active (T1.9, kind `Difficulty`, `game::clock::DifficultyOverride`).
    /// `None` (défaut) : `entry.difficulty` du manifeste, sinon non.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<bool>,
    /// Placements scriptés de personnages (T1.13, voir [`CharacterPlacement`] et
    /// `map_ldtk::game::profile::apply_scenario_character_placements`). Vide (défaut) : rien.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<CharacterPlacement>,
    /// Mode de run imposé (suite de T1.13, `docs/conventions.md` §28) : l'emporte sur
    /// `entry.mode` du manifeste (le runner le remplace avant de l'insérer). `Floors` exige
    /// `floors`. `None` (défaut) : `entry.mode`, comme avant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<content::EntryMode>,
    /// Script d'actions d'UI (M2-T18, `docs/conventions.md` §56) : ouvre le menu de pause ou
    /// l'inventaire et navigue, à des frames exactes. Présentation seule : lu hors simulation
    /// (`game::ui::screens::model::ScreenScript`), jamais dans l'input rollback, aucune trace ne bouge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui: Vec<crate::ui::screens::model::UiStep>,
}

/// Placement scripté d'un personnage (T1.13, `docs/conventions.md` §28) : fait apparaître
/// `character` (id du kind `Character`) à la position `(x, y)` du monde, à la frame `at_frame`
/// exacte, par le même chemin qu'un `CharacterSpawn` de carte (`map_ldtk::game::local::
/// spawn_character` : net id alloué à cette frame, santé × difficulté, variante imposée par
/// `variant` ou tirée). `team` : sinon `CharacterConfig.team`, sinon `Enemies`. Les placements
/// d'une même frame sont appliqués dans l'ordre de déclaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterPlacement {
    pub character: String,
    pub x: Fixed,
    pub y: Fixed,
    pub at_frame: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<sim_core::team::Team>,
}

/// Placement scripté d'un power-up (T2.5) : fait apparaître le power-up `id` (clé de
/// `items/powerups.ron`) à la position `(x, y)` du monde, à la frame `at_frame` exacte.
/// Même idée que `wave_overrides`/`weapon_overrides` (réglage de scénario, pas un vrai
/// champ de contenu) mais appliquée pendant la simulation (`GgrsSchedule`, pas `Update`) :
/// un placement doit apparaître à une frame précise et rester rollback-safe, comme un
/// spawn d'ennemi de vague — voir `map_ldtk::game::profile::apply_scenario_powerup_placements`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerUpPlacement {
    pub id: String,
    pub x: Fixed,
    pub y: Fixed,
    pub at_frame: u32,
}

/// Placement scripté d'un objet (M2-T0b) : fait apparaître l'objet `id` (fichier de `items/`) à
/// la position `(x, y)` du monde, à la frame `at_frame` exacte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemPlacement {
    pub id: String,
    pub x: Fixed,
    pub y: Fixed,
    pub at_frame: u32,
}

/// Profil imposé à un joueur d'un scénario (M2-T12) : son état avant la run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSeed {
    pub handle: usize,
    #[serde(default)]
    pub unlocks: Vec<String>,
    /// Essence possédée.
    #[serde(default)]
    pub essence: u64,
    #[serde(default)]
    pub stats: std::collections::BTreeMap<String, u64>,
}

/// Achat d'un déblocage par le profil du joueur `handle`, avant la run (`purchase_unlock`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePurchase {
    pub handle: usize,
    pub unlock: String,
}

/// Le scénario démarre **au hub** (M2-T13, `docs/conventions.md` §57) : `HubSession` posée avec la
/// carte du hub du manifeste (`entry.hub`) et `Scenario::map` pour la run. `stay: true` empêche le
/// lancement de la run une fois la sortie confirmée (la preuve de l'état `HubExit` reste dans la
/// session ; la relance a son propre test).
///
/// `stay: false` (m2-full-run, §57) : le runner suit la relance. Le scénario se joue alors en deux
/// phases : le hub (frames du hub, attentes `HubExit` et `HubVisit` seulement), puis la run, dont
/// les frames repartent de 0 : `frames`, les autres attentes, la trace et les moments clés sont
/// ceux de la run. `inputs` (un script par joueur, indice = handle) pilote le hub à la place de
/// `players[i].inputs`/`bot` ; vide, le hub est joué avec les `players` du scénario (comme avec
/// `stay: true`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HubSetup {
    #[serde(default)]
    pub stay: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Vec<Segment>>,
}

/// Profils d'un scénario `profile: true` (M2-T12) : états de départ puis achats, dans cet ordre.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfileSetup {
    #[serde(default)]
    pub profiles: Vec<ProfileSeed>,
    #[serde(default)]
    pub purchases: Vec<ProfilePurchase>,
}

/// Placement scripté d'un coffre (M2-T5) : `key` est sa clé de contenu (graine du tirage),
/// `table` son pool (`None` : celui de l'étage), `rarity` la rareté minimale de l'objet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChestPlacement {
    pub key: String,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub rarity: items::Rarity,
    #[serde(default)]
    pub locked: bool,
    pub x: Fixed,
    pub y: Fixed,
    pub at_frame: u32,
}

/// Réglages de boutique d'un scénario (M2-T6).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShopSetup {
    /// Présentoirs posés à une position et une frame exactes.
    #[serde(default)]
    pub shelves: Vec<ShelfPlacement>,
    /// Force `EconomyConfig::shells_drop` pour ce scénario (preuve du drop sans toucher
    /// l'économie du jeu). `None` : celle du jeu.
    #[serde(default)]
    pub shells_drop_override: Option<(u32, u32)>,
}

impl ShopSetup {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Placement scripté d'un présentoir : `key` est sa clé de contenu, `item` l'objet, `price` en
/// `shells`. `cost` (M3-T8, §73) : prix de vie maximale ou de consommable (`Price`), exécuté à la
/// place des douilles (`price` vaut alors 0) ; absent : le prix en douilles, comme avant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfPlacement {
    pub key: String,
    pub item: String,
    pub price: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<items::Price>,
    pub x: Fixed,
    pub y: Fixed,
    pub at_frame: u32,
}

/// Invariants de la simulation vérifiés par le runner à chaque frame (plan §9.4,
/// `crates/scenario/src/invariants.rs`). Un scénario en désactive un ainsi :
/// `invariants: (joueur_hors_mur: false)`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Invariants {
    /// Toute entité rollback avec une santé : `0 ≤ current ≤ max`.
    #[serde(default = "vrai")]
    pub sante_bornee: bool,
    /// Deux entités rollback n'ont jamais le même `GgrsNetId`.
    #[serde(default = "vrai")]
    pub net_ids_uniques: bool,
    /// Aucun joueur ne chevauche un collider de mur.
    #[serde(default = "vrai")]
    pub joueur_hors_mur: bool,
}

fn vrai() -> bool {
    true
}

impl Default for Invariants {
    fn default() -> Self {
        Self {
            sante_bornee: true,
            net_ids_uniques: true,
            joueur_hors_mur: true,
        }
    }
}

impl Invariants {
    pub fn tous_actifs(&self) -> bool {
        self.sante_bornee && self.net_ids_uniques && self.joueur_hors_mur
    }
}

/// Remplace des valeurs de chargeur d'une arme, pour tous ses modes ou un seul.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponOverride {
    pub weapon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_size: Option<u32>,
    /// Chargeurs de réserve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_limit: Option<u32>,
    /// Cadence de tir (coups/minute, voir `weapons::FiringModeConfig::firing_rate`), pour ce
    /// mode ou tous (T2.1, chantier B4b, bench `bench_bullets` : « cadence maximale »).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firing_rate: Option<Fixed>,
    /// Politique de tir ami de l'arme, pour ce scénario seulement (T1.1, chantier B1 :
    /// scénarios `friendly_fire_cursed`/`immune_tag`). S'applique à l'arme entière (pas
    /// `mode`, qui ne sélectionne que le sous-champ chargeur) : `WeaponConfig::friendly_fire`
    /// n'est pas par mode de tir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub friendly_fire: Option<FriendlyFire>,
    /// Type de munition de l'arme, pour ce scénario seulement (T2.2, chantier B7 :
    /// scénario `ammo_shared_reserve`, deux armes forcées sur le même type pour prouver la
    /// réserve partagée). S'applique à l'arme entière, comme `friendly_fire`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ammo_type: Option<AmmoType>,
}

/// Remplace des valeurs de `waves::config::WaveConfig` pour un scénario (T2.1, bench
/// `bench_horde`). Tous les champs sont optionnels ; absents, la valeur du RON du jeu
/// (`games/<jeu>/assets/**/waves.ron` ou équivalent) reste inchangée. `base_enemies`,
/// `enemies_per_wave` et `grace_period_frames` couvrent la demande de la tâche (plus
/// d'ennemis, plus tôt) ; `max_concurrent_enemies` s'y ajoute parce que sans lui la config
/// par défaut (20) plafonne les ennemis *vivants en même temps* bien en dessous de
/// `base_enemies` — la file d'attente grossirait sans jamais stresser la grille comme voulu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveOverride {
    /// Vague qui déclenche la victoire du mode Waves (T3.1, scénarios du clone).
    /// Absent : conserve la condition de fin définie par le jeu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_wave: Option<u32>,
    /// Délai entre vagues, pour rejouer une partie courte sans modifier les assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_wave_delay_frames: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_enemies: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemies_per_wave: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_period_frames: Option<u32>,
    /// Voir la doc du type : nécessaire pour que `base_enemies` se traduise en ennemis
    /// réellement présents à la fois, pas seulement en file d'attente.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concurrent_enemies: Option<u32>,
    /// Combien d'ennemis apparaissent par salve (`waves::config::WaveConfig::spawn_batch_size`).
    /// Ajouté avec `max_concurrent_enemies` pour la même raison : la config par défaut d'un
    /// jeu peut faire apparaître les ennemis un par un toutes les N frames, bien trop lentement
    /// pour atteindre `base_enemies` vivants dans la durée d'un bench.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_batch_size: Option<u32>,
    /// Délai entre deux salves (`waves::config::WaveConfig::spawn_interval_frames`). Voir
    /// `spawn_batch_size`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_interval_frames: Option<u32>,
}

fn default_game() -> String {
    "zombies".into()
}

fn default_map() -> String {
    "exemples/test_map.ldtk".into()
}

fn default_map_seed() -> i32 {
    123456
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlayerScript {
    #[serde(default)]
    pub inputs: Vec<Segment>,
    /// Profil de bot (T2.11) : ce joueur est piloté par `crates/bots::decide()` au lieu de
    /// `inputs`. Exclusif avec `inputs` non vide (un joueur est scripté OU piloté par un bot,
    /// jamais les deux ; le runner panique si les deux sont présents). RON : `(bot: fonceur)`.
    /// `skip_serializing_if` (même style que `WeaponOverride` plus bas) : un enregistrement
    /// (`InputRecorder::to_scenario`, toujours `bot: None`) ne l'écrit pas, un scénario rejoué
    /// reste un scénario `Scripted` ordinaire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotProfile>,
    /// Tags du personnage (T1.1, chantier B1), ex. `["cursed"]` (scénario
    /// `friendly_fire_cursed`). Vide par défaut : n'affecte pas les scénarios existants.
    /// Posé en composant `Tags` une fois le joueur créé (`map_ldtk::game::profile::apply_player_overrides`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Tags de dégât contre lesquels ce joueur est immunisé (T1.1, scénario `immune_tag`).
    /// Remplace le `Defenses` posé par `CharacterConfig` (les joueurs n'en ont pas par
    /// défaut, donc pas de perte en pratique). Vide par défaut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub immune_to: Vec<String>,
    /// Modificateurs de stats posés sur ce joueur une fois créé (T1.2, chantier B2,
    /// scénario `stat_move_speed`), comme `tags`/`immune_to`
    /// (`map_ldtk::game::profile::apply_player_overrides`). Vide par défaut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierSpec>,
    /// Arme unique de ce joueur (T2.10, générateur de scénarios), id du registre (arme à
    /// distance de `weapons.ron` ou de mêlée de `melee_weapons.ron`) : le joueur apparaît
    /// avec **cette seule arme** au lieu de ses `starting_weapons` (et sans l'arme de mêlée
    /// par défaut, `bare_hands`, si l'id choisi est une arme à distance — exclusivité
    /// complète, voir `map_ldtk::game::profile::apply_player_overrides`). `None` (défaut) :
    /// comportement inchangé, le joueur reçoit les armes de son `CharacterConfig` comme
    /// avant ce champ (tous les scénarios existants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    /// Solde de départ (T2.3, chantier C5 v1, scénarios d'achat : `buy_door`,
    /// `buy_wall_weapon`, `buy_perk`) : remplace `starting_currency` du personnage, posé
    /// comme `weapon` ci-dessus (`map_ldtk::game::profile::apply_player_overrides`), avant la
    /// première frame simulée. `None` (défaut) : comportement inchangé, le joueur démarre
    /// avec `CharacterConfig::starting_currency` (tous les scénarios existants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<u32>,
    /// Mutations imposées (T1.10) : ids du kind `Mutation`, prises dans l'ordre avant la
    /// première frame simulée (effets ajoutés à `Effects`, `Mutations` posé), comme
    /// `modifiers` ci-dessus (`map_ldtk::game::profile::apply_player_overrides`). Vide (défaut) :
    /// aucune.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<String>,
    /// Dash imposé (M2-T2, §43, scénarios `dash_shove`, `dash_in_place`) : remplace le `dash` du
    /// personnage (`map_ldtk::game::profile::apply_player_overrides`). Absent (défaut) : rien.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<crate::character::movement::DashConfig>,
    /// Objets tenus dès la première frame (M2-T17, §54, gabarit `ItemHeld`) : ids du kind `Item`,
    /// posés comme un ramassage (passif : inventaire et modificateurs ; actif : emplacement actif,
    /// **chargé**). Vide (défaut) : rien, aucune trace existante ne bouge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Bombe imposée (M3-T4, §69, scénarios `bomb_*`) : le joueur porte cette `BombDef` comme si son
    /// personnage la déclarait ; le stock se donne par `items: ["bomb", ...]` (un consommable par
    /// bombe). Absent (défaut) : rien, aucune trace existante ne bouge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bomb: Option<world::BombDef>,
    /// Visée imposée (suivi Q17, scénarios `aim_four_way*`) : le joueur vise comme si son personnage
    /// déclarait `aim:` (`Free` : comme s'il n'en déclarait aucun, même si son personnage a `FourWay`).
    /// Absent (défaut) : rien, aucune trace existante ne bouge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aim: Option<sim_core::aim::AimMode>,
}

/// Un modificateur de scénario, posé sur un joueur après sa création
/// (`map_ldtk::game::profile::apply_player_overrides`). Format RON :
/// `(stat: MoveSpeed, op: Mul, value: "0.5")`. `until` absent = permanent (dure toute la
/// partie, largement suffisant pour un scénario de test).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifierSpec {
    pub stat: StatId,
    pub op: ModifierOp,
    pub value: Fixed,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<u32>,
}

/// Politique de l'`explorateur` (M3-T14, `docs/conventions.md` §79) : ce qu'il ouvre, achète et fait
/// sauter, en données. Défaut = comportement d'avant M3-T14 (aucun achat, ordre du plus proche) ; les
/// décisions de bombe et de porte à clé sont **inertes** sans porteur de bombe, sans salle secrète ni
/// porte à clé, donc les parties des autres jeux ne changent pas. `games/<jeu>/assets/bot.ron`
/// (facultatif) la fixe pour les campagnes d'`alacod-sim` ; `Scenario::explorer` pour un scénario.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ExplorerPolicy {
    /// D62 : ouvre les coffres et achète les présentoirs payables (comme `Scenario::shopping`).
    #[serde(default)]
    pub shopping: bool,
    /// Ordre et conditions d'achat en douilles (présentoirs).
    #[serde(default)]
    pub shop: ShopOrder,
    /// Limites de détour (coût de chemin : 10 par case de 8 px en ligne droite) : un but facultatif plus coûteux n'est pas
    /// un but (voir [`Detour`]). Défaut : aucune limite.
    #[serde(default)]
    pub detour: Detour,
    /// Anticipe l'élan transverse (`bots::explorer::lead`, D82) : une touche n'agit qu'après le délai d'input (5 frames)
    /// et le personnage garde son élan ; sans cette correction le bot déborde de ±8 px devant une ouverture de 32 px et
    /// tourne en cycle limite (graines 51 et 182 de `etages_deux`).
    #[serde(default)]
    pub lead: bool,
    /// Armes selon leur réserve : ne pas acheter ni ramasser une arme dont le type de munition n'a aucune réserve (elle
    /// remplacerait l'arme de départ et le bot resterait sans tir), ramasser l'arme de départ lâchée quand plus aucune arme
    /// portée n'a de réserve (« à sec ») même sous la menace.
    #[serde(default)]
    pub weapon_reserve: bool,
    /// Marché du diable : présentoirs payés en vie maximale (`Shelf::cost`).
    #[serde(default)]
    pub devil: DevilPolicy,
    /// Bombes : mur secret, rochers.
    #[serde(default)]
    pub bombs: BombPolicy,
    /// Anti-oscillation (une seule stratégie par jeu, voir [`Oscillation`]). Défaut : aucune, les autres jeux ne changent pas.
    #[serde(default)]
    pub oscillation: Oscillation,
    /// Lecture du combat : fuit toute bombe allumée (la sienne ou celle d'un champion explosif), approche d'un ennemi quand
    /// ses larmes tournent autour de lui (`Orbit`). Faux par défaut : les autres jeux ne changent pas.
    #[serde(default)]
    pub combat_aware: bool,
}

/// Stratégie anti-oscillation de l'`explorateur` : **un seul mécanisme** (`bots::explorer::ExplorerJitter`, mémoire hors
/// rollback du bot), deux manières de s'en servir selon la cause du balancement, l'une excluant l'autre. Les deux ont été
/// essayées l'une pour l'autre : chacune bouge des traces de l'autre jeu (gungeon : `gungeon_full_run*`, isaac :
/// `bot_isaac_floor`, `bot_isaac_two`), voir `docs/conventions.md` §79.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Oscillation {
    /// Aucune (défaut : les parties d'avant ne changent pas).
    #[default]
    Off,
    /// Hystérésis des buts (M3-T14) : quand le bot veut avancer mais tourne sur place (douze unités pendant quarante frames :
    /// deux buts ou deux routes de coût presque égal, l'input arrive avec son délai), il s'attache à son but du moment et
    /// tient son prochain point de passage (soixante frames au plus) ; blocages de combat, poursuites sans fin et réanimation
    /// à portée suivent le même régime.
    Hysteresis,
    /// Patience (m2-salles-murs-gungeon, D82) : le pilotage touche par touche peut tourner en cycle limite devant une ouverture
    /// de 32 px quel que soit le gain de correction ; un bot resté 100 frames à moins de 40 de son ancre sans ennemi ni butin
    /// s'arrête (15 frames) puis file droit (45 frames), en boucle.
    Patience,
}

impl ExplorerPolicy {
    /// Stratégie [`Oscillation::Hysteresis`] active.
    pub fn hysteresis(&self) -> bool {
        self.oscillation == Oscillation::Hysteresis
    }

    /// Stratégie [`Oscillation::Patience`] active.
    pub fn patient(&self) -> bool {
        self.oscillation == Oscillation::Patience
    }

    /// La part de la politique que jouent les scénarios **générés** (`alacod-gen`, gabarits de salle et de boss) : le
    /// déplacement, le détour des objets, le ramassage des armes, la lecture du combat ; ni achat, ni sous, ni diable, ni
    /// bombe (un gabarit teste une salle, pas l'économie ni les explosifs : les scénarios d'avant ne les jouaient pas).
    pub fn scene(&self) -> Self {
        Self {
            shopping: false,
            shop: ShopOrder::default(),
            detour: Detour {
                shells: None,
                ..self.detour
            },
            devil: DevilPolicy::default(),
            bombs: BombPolicy {
                secret: false,
                rocks: false,
                ..BombPolicy::default()
            },
            ..*self
        }
    }
}

/// Limites de détour de l'`explorateur` (coût de chemin : 10 par case orthogonale de 8 px). Une **seule** règle, trois
/// bornes selon le but : un but facultatif plus coûteux que sa borne n'est pas un but (il le redevient quand le bot passe
/// à côté). `None` : pas de borne.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Detour {
    /// Tas de sous au sol (`ShellPickup`, M3-T14, `isaac` : 400, une quarantaine de cases) : ils ne valent pas un détour
    /// d'un bout à l'autre de l'étage. `None` : seulement ceux qu'il croise.
    #[serde(default)]
    pub shells: Option<u32>,
    /// Objets, coffres, présentoirs et salles sans verrou (récompense, boutique) tant qu'il reste une salle de combat
    /// (m2-salles-murs-gungeon, `gungeon` : 600, soixante cases) : dans des salles agrandies un étage mesure plus de
    /// 2 000 px et le soft-lock `Floors` tombe à 1 200 frames sans dégât, à courir d'un objet lointain à l'autre.
    #[serde(default)]
    pub optional: Option<u32>,
    /// Même borne quand plus aucune salle de combat ne reste (le boss et d'éventuels détours ; `gungeon` : 200) :
    /// `None` reprend `optional`.
    #[serde(default)]
    pub last_stretch: Option<u32>,
}

/// Ordre des achats en douilles de l'`explorateur`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum ShopOrder {
    /// Le présentoir le plus proche par le chemin que le plus riche des joueurs peut payer (D62).
    #[default]
    Nearest,
    /// M3-T14 : chaque joueur n'achète que ce qu'il paie ; un cœur (consommable `refuse_when_full`)
    /// d'abord quand sa vie est **au plus** `low_health` × sa vie maximale (et qu'il n'est pas plein),
    /// sinon le moins cher, puis le plus proche.
    HeartsThenCheapest { low_health: Fixed },
}

/// Décision de l'`explorateur` devant un présentoir payé en vie maximale (le diable).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum DevilPolicy {
    /// Jamais.
    #[default]
    Never,
    /// Achète tant que sa vie courante est **strictement** au-dessus de `n` (en points de vie) et que
    /// le prix ne la tuerait pas ; chaque achat baisse la vie : le suivant demande de la vie encore.
    IfHealthAbove(Fixed),
}

/// Bombes de l'`explorateur` (sans effet pour un joueur sans `BombCarrier` ni bombe en stock).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BombPolicy {
    /// Pose une bombe contre le mur d'une salle secrète probable.
    #[serde(default = "vrai")]
    pub secret: bool,
    /// Une salle secrète n'est « probable » que si elle touche au moins ce nombre de salles
    /// (règle de la grille, M3-T5 : une secrète voisine de plusieurs salles).
    #[serde(default = "default_secret_neighbours")]
    pub secret_min_neighbours: u32,
    /// Pose une bombe devant un rocher qui coupe le chemin d'un but.
    #[serde(default = "vrai")]
    pub rocks: bool,
    /// Bombes gardées en réserve : il n'en pose pas en dessous (défaut 0).
    #[serde(default)]
    pub reserve: u32,
}

fn default_secret_neighbours() -> u32 {
    2
}

impl Default for BombPolicy {
    fn default() -> Self {
        Self {
            secret: true,
            secret_min_neighbours: default_secret_neighbours(),
            rocks: true,
            reserve: 0,
        }
    }
}

/// Profil de bot (T2.11, `crates/bots`). Défini ici plutôt que dans `crates/bots` : le format
/// de scénario (`PlayerScript`) vit dans `game`, qui ne doit pas dépendre de `bots` (`bots`
/// dépend de `game`, jamais l'inverse) ; `crates/bots` ré-exporte ce type sous `bots::BotProfile`
/// et y ajoute le comportement (`decide`). Noms RON en minuscules (`immobile`, `fonceur`,
/// `prudent`), voir `Self::parse_name`/`Self::name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotProfile {
    /// Aucun input : reste immobile, ne tire pas, ne répare pas.
    Immobile,
    /// Va vers l'ennemi le plus proche, tire à portée, recharge à vide ; répare la fenêtre la
    /// plus proche quand aucun ennemi n'est à portée.
    Fonceur,
    /// Garde ses distances (recule si un ennemi est trop près, avance sinon), tire, recharge.
    Prudent,
    /// Chasse avec navigation physique, change d’arme et réanime.
    Chasseur,
    /// Chasseur qui ouvre les portes et achète armes, munitions et Juggernog.
    Acheteur,
    /// M2-T16 : joue un étage de salles (mode `Floors`) seul : va à la salle à visiter la plus
    /// proche par le chemin, s'y bat comme `prudent` (esquive, blank), ramasse les objets au
    /// sol, finit par le boss puis prend le portail (`docs/conventions.md` §47).
    Explorateur,
}

impl BotProfile {
    /// Nom RON en minuscules (`#[serde(rename_all = "lowercase")]`), utilisé aussi par la CLI
    /// `alacod-sim` (`--profiles fonceur,prudent,...`) pour ne pas dupliquer le mapping.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Immobile => "immobile",
            Self::Fonceur => "fonceur",
            Self::Prudent => "prudent",
            Self::Chasseur => "chasseur",
            Self::Acheteur => "acheteur",
            Self::Explorateur => "explorateur",
        }
    }

    /// Analyse inverse de [`Self::name`] ; `None` si le nom n'est pas un profil connu.
    pub fn parse_name(name: &str) -> Option<Self> {
        match name {
            "immobile" => Some(Self::Immobile),
            "fonceur" => Some(Self::Fonceur),
            "prudent" => Some(Self::Prudent),
            "chasseur" => Some(Self::Chasseur),
            "acheteur" => Some(Self::Acheteur),
            "explorateur" => Some(Self::Explorateur),
            _ => None,
        }
    }
}

/// Input maintenu sur les frames `from..to`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub from: u32,
    pub to: u32,
    #[serde(default)]
    pub buttons: Vec<Button>,
    /// Visée : vecteur du joueur vers le pointeur, en unités monde.
    #[serde(default)]
    pub pan: (i16, i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Fire,
    SwitchWeapon,
    SwitchWeaponMode,
    Reload,
    Sprint,
    Dash,
    Modifier,
    Interaction,
    Melee,
    /// Lâche l'arme active au sol (T2.2, chantier B7). Voir `INPUT_DROP_WEAPON`.
    DropWeapon,
    /// Touche de debug qui provoque un crash volontaire
    ForceCrash,
    /// Choix de mutation A/B/C (T1.10). Voir `INPUT_CHOICE_A`.
    ChoiceA,
    ChoiceB,
    ChoiceC,
    /// Utilise l'objet actif tenu (M2-T0b). Voir `INPUT_USE_ACTIVE`.
    UseActive,
    /// Blank (M2-T0b, contrat : compteur et input). Voir `INPUT_BLANK`.
    Blank,
    /// Bombe (M3-T0c, contrat : bit d'input sans effet sans `bomb`). Voir `INPUT_BOMB`.
    Bomb,
}

impl Scenario {
    /// Lecture avec l'extension RON `implicit_some` : un champ optionnel s'écrit
    /// `mode: "default"` plutôt que `mode: Some("default")`.
    pub fn from_ron(source: &str) -> Result<Self, ron::error::SpannedError> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(source)
    }

    pub fn to_ron(&self) -> String {
        let config = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .depth_limit(4)
            .indentor("    ".to_string())
            .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        ron::ser::to_string_pretty(self, config).expect("sérialisation du scénario")
    }

    pub fn scripted_inputs(&self) -> ScriptedInputs {
        ScriptedInputs {
            players: self
                .players
                .iter()
                .map(|player| {
                    player
                        .inputs
                        .iter()
                        .map(Segment::to_input_segment)
                        .collect()
                })
                .collect(),
        }
    }

    /// Pistes du hub (`HubSetup::inputs`), dans le même format que [`Self::scripted_inputs`].
    pub fn hub_scripted_inputs(&self) -> ScriptedInputs {
        ScriptedInputs {
            players: self
                .hub
                .iter()
                .flat_map(|hub| hub.inputs.iter())
                .map(|segments| segments.iter().map(Segment::to_input_segment).collect())
                .collect(),
        }
    }
}

impl Segment {
    /// Segment qui rejoue `input` sur les frames `from..to`.
    pub fn from_input(from: u32, to: u32, input: &BoxInput) -> Self {
        let mut buttons: Vec<Button> = ALL_BUTTONS
            .iter()
            .copied()
            .filter(|b| button_bit(*b) != 0 && input.buttons & button_bit(*b) != 0)
            .collect();
        if input.fire {
            buttons.push(Button::Fire);
        }
        if input.switch_weapon {
            buttons.push(Button::SwitchWeapon);
        }
        Self {
            from,
            to,
            buttons,
            pan: (input.pan_x, input.pan_y),
        }
    }

    fn to_input_segment(&self) -> InputSegment {
        InputSegment {
            from: self.from,
            to: self.to,
            input: box_input(&self.buttons, self.pan),
        }
    }
}

/// Input GGRS correspondant à des boutons enfoncés et une visée.
pub fn box_input(buttons: &[Button], pan: (i16, i16)) -> BoxInput {
    let mut input = BoxInput {
        pan_x: pan.0,
        pan_y: pan.1,
        ..Default::default()
    };
    for button in buttons {
        match button {
            Button::Fire => input.fire = true,
            Button::SwitchWeapon => input.switch_weapon = true,
            other => input.buttons |= button_bit(*other),
        }
    }
    input
}

const ALL_BUTTONS: [Button; 21] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::Fire,
    Button::SwitchWeapon,
    Button::SwitchWeaponMode,
    Button::Reload,
    Button::Sprint,
    Button::Dash,
    Button::Modifier,
    Button::Interaction,
    Button::Melee,
    Button::DropWeapon,
    Button::ForceCrash,
    Button::ChoiceA,
    Button::ChoiceB,
    Button::ChoiceC,
    Button::UseActive,
    Button::Blank,
    Button::Bomb,
];

fn button_bit(button: Button) -> u32 {
    match button {
        Button::Up => INPUT_UP,
        Button::Down => INPUT_DOWN,
        Button::Left => INPUT_LEFT,
        Button::Right => INPUT_RIGHT,
        Button::Reload => INPUT_RELOAD,
        Button::SwitchWeaponMode => INPUT_SWITCH_WEAPON_MODE,
        Button::Sprint => INPUT_SPRINT,
        Button::Dash => INPUT_DASH,
        Button::Modifier => INPUT_MODIFIER,
        Button::Interaction => INPUT_INTERACTION,
        Button::Melee => INPUT_MELEE_ATTACK,
        Button::DropWeapon => INPUT_DROP_WEAPON,
        Button::ForceCrash => INPUT_FORCE_CRASH,
        Button::ChoiceA => INPUT_CHOICE_A,
        Button::ChoiceB => INPUT_CHOICE_B,
        Button::ChoiceC => INPUT_CHOICE_C,
        Button::UseActive => INPUT_USE_ACTIVE,
        Button::Blank => INPUT_BLANK,
        Button::Bomb => INPUT_BOMB,
        Button::Fire | Button::SwitchWeapon => 0,
    }
}
