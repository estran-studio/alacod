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
    BoxInput, InputSegment, ScriptedInputs, INPUT_BLANK, INPUT_CHOICE_A, INPUT_CHOICE_B,
    INPUT_CHOICE_C, INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON, INPUT_FORCE_CRASH,
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
    /// `scenario::runner::apply_scenario_powerup_placements`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub powerups: Vec<PowerUpPlacement>,
    /// Placements scriptés d'objets (M2-T0b, `docs/conventions.md` §36) : même principe que
    /// `powerups`, pour les objets de `items/*.ron` ([`ItemPlacement`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemPlacement>,
    /// Écrit les profils de méta-progression à la fin de la run, dans un dossier temporaire
    /// (M2-T0c, `docs/conventions.md` §38) ; faux par défaut : un scénario n'écrit rien.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub profile: bool,
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
    /// `scenario::runner::apply_scenario_character_placements`). Vide (défaut) : rien.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<CharacterPlacement>,
    /// Mode de run imposé (suite de T1.13, `docs/conventions.md` §28) : l'emporte sur
    /// `entry.mode` du manifeste (le runner le remplace avant de l'insérer). `Floors` exige
    /// `floors`. `None` (défaut) : `entry.mode`, comme avant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<content::EntryMode>,
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
/// spawn d'ennemi de vague — voir `scenario::runner::apply_scenario_powerup_placements`.
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
    /// Posé en composant `Tags` une fois le joueur créé (`scenario::runner::apply_player_overrides`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Tags de dégât contre lesquels ce joueur est immunisé (T1.1, scénario `immune_tag`).
    /// Remplace le `Defenses` posé par `CharacterConfig` (les joueurs n'en ont pas par
    /// défaut, donc pas de perte en pratique). Vide par défaut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub immune_to: Vec<String>,
    /// Modificateurs de stats posés sur ce joueur une fois créé (T1.2, chantier B2,
    /// scénario `stat_move_speed`), comme `tags`/`immune_to`
    /// (`scenario::runner::apply_player_overrides`). Vide par défaut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierSpec>,
    /// Arme unique de ce joueur (T2.10, générateur de scénarios), id du registre (arme à
    /// distance de `weapons.ron` ou de mêlée de `melee_weapons.ron`) : le joueur apparaît
    /// avec **cette seule arme** au lieu de ses `starting_weapons` (et sans l'arme de mêlée
    /// par défaut, `bare_hands`, si l'id choisi est une arme à distance — exclusivité
    /// complète, voir `scenario::runner::apply_player_overrides`). `None` (défaut) :
    /// comportement inchangé, le joueur reçoit les armes de son `CharacterConfig` comme
    /// avant ce champ (tous les scénarios existants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    /// Solde de départ (T2.3, chantier C5 v1, scénarios d'achat : `buy_door`,
    /// `buy_wall_weapon`, `buy_perk`) : remplace `starting_currency` du personnage, posé
    /// comme `weapon` ci-dessus (`scenario::runner::apply_player_overrides`), avant la
    /// première frame simulée. `None` (défaut) : comportement inchangé, le joueur démarre
    /// avec `CharacterConfig::starting_currency` (tous les scénarios existants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<u32>,
    /// Mutations imposées (T1.10) : ids du kind `Mutation`, prises dans l'ordre avant la
    /// première frame simulée (effets ajoutés à `Effects`, `Mutations` posé), comme
    /// `modifiers` ci-dessus (`scenario::runner::apply_player_overrides`). Vide (défaut) :
    /// aucune.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<String>,
}

/// Un modificateur de scénario, posé sur un joueur après sa création
/// (`scenario::runner::apply_player_overrides`). Format RON :
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

const ALL_BUTTONS: [Button; 20] = [
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
        Button::Fire | Button::SwitchWeapon => 0,
    }
}
