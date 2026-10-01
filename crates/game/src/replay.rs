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
    BoxInput, InputSegment, ScriptedInputs, INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON,
    INPUT_FORCE_CRASH, INPUT_INTERACTION, INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER,
    INPUT_RELOAD, INPUT_RIGHT, INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE, INPUT_UP,
};
use bevy_fixed::fixed_math::Fixed;
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
    /// Force `PowerUpsConfig::drop_chance` pour ce scénario (T2.5), pour prouver le chemin
    /// « drop à la mort » sans dépendre du tirage réel du jeu (scénario
    /// `powerup_drop_on_kill`) — voir `scenario::runner::apply_powerup_drop_chance_override`.
    /// `None` (défaut) : `drop_chance` de `items/powerups.ron` du jeu, inchangé pour tous
    /// les scénarios existants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub powerup_drop_chance_override: Option<Fixed>,
}

/// Placement scripté d'un power-up (T2.5) : fait apparaître le power-up `id` (clé de
/// `items/powerups.ron`) à la position `(x, y)` du monde, à la frame `at_frame` exacte.
/// Même idée que `wave_overrides`/`weapon_overrides` (réglage de scénario, pas un vrai
/// champ de contenu) mais appliquée pendant la simulation (`GgrsSchedule`, pas `Update`) :
/// un placement doit apparaître à une frame précise et rester rollback-safe, comme un
/// spawn d'ennemi de vague — voir `scenario::runner::apply_scenario_powerup_placements`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerUpPlacement {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveOverride {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
}

/// Un modificateur de scénario, posé sur un joueur après sa création
/// (`scenario::runner::apply_player_overrides`). Format RON :
/// `(stat: MoveSpeed, op: Mul, value: "0.5")`. `until` absent = permanent (dure toute la
/// partie, largement suffisant pour un scénario de test).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl BotProfile {
    /// Nom RON en minuscules (`#[serde(rename_all = "lowercase")]`), utilisé aussi par la CLI
    /// `alacod-sim` (`--profiles fonceur,prudent,...`) pour ne pas dupliquer le mapping.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Immobile => "immobile",
            Self::Fonceur => "fonceur",
            Self::Prudent => "prudent",
        }
    }

    /// Analyse inverse de [`Self::name`] ; `None` si le nom n'est pas un profil connu.
    pub fn parse_name(name: &str) -> Option<Self> {
        match name {
            "immobile" => Some(Self::Immobile),
            "fonceur" => Some(Self::Fonceur),
            "prudent" => Some(Self::Prudent),
            _ => None,
        }
    }
}

/// Input maintenu sur les frames `from..to`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Catégorie d'entités pour `EntityCount`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityKind {
    /// Joueurs vivants.
    Player,
    /// Ennemis vivants.
    Enemy,
    /// Balles en vol.
    Bullet,
    /// Toutes les entités marquées `Rollback`.
    Rollback,
}

/// Vérification faite quand la simulation atteint `at_frame`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expectation {
    PlayerAlive {
        handle: usize,
        at_frame: u32,
    },
    PlayerDead {
        handle: usize,
        at_frame: u32,
    },
    WaveAtLeast {
        wave: u32,
        at_frame: u32,
    },
    KillsAtLeast {
        kills: u32,
        at_frame: u32,
    },
    /// Fenêtres détruites (obstacles cassables qui ne bloquent plus).
    WindowsBrokenAtLeast {
        windows: u32,
        at_frame: u32,
    },
    /// Arme active du joueur (et son mode de tir, si précisé).
    ActiveWeapon {
        handle: usize,
        weapon: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
        at_frame: u32,
    },
    /// Munitions exactes dans le chargeur de l'arme active.
    Ammo {
        handle: usize,
        ammo: u32,
        at_frame: u32,
    },
    /// Munitions exactes dans la réserve du joueur pour un type de munition donné (T2.2,
    /// chantier B7, `combat::inventory::AmmoReserves`). Contrairement à `Ammo` (chargeur de
    /// l'arme active), vérifie la réserve d'un type précis, partagée entre toutes les armes
    /// qui le déclarent — voir le scénario `ammo_shared_reserve`.
    AmmoReserve {
        handle: usize,
        ammo_type: AmmoType,
        amount: u32,
        at_frame: u32,
    },
    /// Nombre d'armes tombées au sol (T2.2, chantier B7, `weapons::WeaponPickup` sans
    /// `price`), toutes entités confondues (pas par joueur : lâcher/ramasser n'a pas de
    /// propriétaire une fois l'arme au sol). Les armes murales (`price: Some(..)`, posées par
    /// la carte, T2.3) ne comptent pas. Bornes `[min, max]` inclusives, `None` = pas de
    /// borne — voir le scénario `drop_pickup_swap`.
    WeaponPickups {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
        at_frame: u32,
    },
    /// Nombre de power-ups au sol (T2.5, `game::powerups::PowerUpPickup`, même forme que
    /// [`Self::WeaponPickups`]). Bornes `[min, max]` inclusives, `None` = pas de borne —
    /// voir le scénario `powerup_drop_on_kill` (preuve du drop déterministe à la mort).
    PowerUpPickups {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
        at_frame: u32,
    },
    /// Portes ouvertes (sans collider).
    DoorsOpenAtLeast {
        doors: u32,
        at_frame: u32,
    },
    /// Santé exacte d'une fenêtre (par son GgrsNetId).
    WindowHealth {
        window: usize,
        health: u8,
        at_frame: u32,
    },
    /// Toutes les balles en vol sont dans la zone (ex. elles ne traversent pas un mur).
    BulletsInside {
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        at_frame: u32,
    },
    /// Position du joueur, à `tolerance` unités près sur chaque axe.
    PlayerPosition {
        handle: usize,
        x: f32,
        y: f32,
        tolerance: f32,
        at_frame: u32,
    },
    /// Santé du joueur `handle` dans l'intervalle `[min, max]` (bornes inclusives, `None` = pas de borne).
    /// Les `f32` sont convertis en `Fixed` pour la comparaison.
    Health {
        handle: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f32>,
        at_frame: u32,
    },
    /// Santé de l'entité rollback `net_id` dans l'intervalle `[min, max]` (bornes inclusives, `None` = pas de borne).
    /// Les `f32` sont convertis en `Fixed` pour la comparaison.
    EntityHealth {
        net_id: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f32>,
        at_frame: u32,
    },
    /// Nombre de coups reçus par l'entité `net_id`, dans `[min, max]` (`max` optionnel,
    /// T2.10 : borne haute d'un `test:` de définition d'arme). T2.9, testbed : la cible
    /// `target`, composant `HitCount` posé par `CharacterConfig::counts_hits`. L'entité doit
    /// exister et porter `HitCount` à `at_frame`, sinon l'attente échoue (comme
    /// `EntityHealth`).
    EntityHits {
        net_id: usize,
        min: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
        at_frame: u32,
    },
    /// La santé du joueur `handle` ne diminue à aucune frame entre `from_frame` et `to_frame` inclus.
    /// C'est une attente **continue** : le runner relève la santé à chaque frame de l'intervalle.
    /// Une baisse produit une failure qui dit la frame et les deux valeurs.
    NoDamageBetween {
        handle: usize,
        from_frame: u32,
        to_frame: u32,
    },
    /// Compte d'entités vivantes du type `kind` dans l'intervalle `[min, max]` (bornes inclusives, `None` = pas de borne).
    EntityCount {
        kind: EntityKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
        at_frame: u32,
    },
    /// Un `GameEvent` de ce `kind` (et dont le label contient la sous-chaîne, si donnée) est survenu
    /// à une frame ≤ `by_frame`. Les `kind` possibles : "wave", "kill", "player", "hit", "reload",
    /// "weapon", "move", "melee", "death", "window", "door", "downed", "revived", "defeat",
    /// "drop", "pickup" (T2.2, chantier B7).
    Event {
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_contains: Option<String>,
        by_frame: u32,
    },
    /// Le joueur `handle` est à terre (`combat::downed::Downed`) à la frame exacte
    /// `at_frame` — vérification ponctuelle, comme `PlayerAlive`/`PlayerDead` (T1.3,
    /// chantier B6).
    PlayerDowned {
        handle: usize,
        at_frame: u32,
    },
    /// Le joueur `handle` a été réanimé (`combat::downed::Downed` retiré par une
    /// réanimation complète, pas par un saignement mortel) à une frame ≤ `by_frame` —
    /// vérification cumulative sur l'historique des événements (`GameEvents`, kind
    /// `"revived"`), comme `Event` (T1.3, chantier B6).
    PlayerRevived {
        handle: usize,
        by_frame: u32,
    },
    /// Tous les joueurs sont à terre ou morts (`combat::downed::RunOutcome::defeat_at_frame`)
    /// à une frame ≤ `by_frame` (T1.3, chantier B6).
    Defeat {
        by_frame: u32,
    },
    /// Solde de monnaie du joueur `handle` dans `[min, max]` (bornes inclusives, `None` =
    /// pas de borne), T2.3 chantier C5 v1 — même forme que [`Self::EntityCount`].
    Currency {
        handle: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
        at_frame: u32,
    },
    /// Valeur résolue (base + modificateurs actifs, `stats::StatReader`) d'une stat du
    /// joueur `handle` — T2.3 chantier C5 v1, scénario `buy_perk` (`Stat MaxHealth 200`
    /// après l'achat de Juggernog). Le `f32` est converti en `Fixed` pour la comparaison,
    /// comme [`Self::Health`].
    Stat {
        handle: usize,
        stat: StatId,
        value: f32,
        at_frame: u32,
    },
}

impl Expectation {
    pub fn at_frame(&self) -> u32 {
        match self {
            Self::PlayerAlive { at_frame, .. }
            | Self::PlayerDead { at_frame, .. }
            | Self::WaveAtLeast { at_frame, .. }
            | Self::KillsAtLeast { at_frame, .. }
            | Self::WindowsBrokenAtLeast { at_frame, .. }
            | Self::ActiveWeapon { at_frame, .. }
            | Self::Ammo { at_frame, .. }
            | Self::AmmoReserve { at_frame, .. }
            | Self::WeaponPickups { at_frame, .. }
            | Self::PowerUpPickups { at_frame, .. }
            | Self::DoorsOpenAtLeast { at_frame, .. }
            | Self::WindowHealth { at_frame, .. }
            | Self::BulletsInside { at_frame, .. }
            | Self::PlayerPosition { at_frame, .. }
            | Self::Health { at_frame, .. }
            | Self::EntityHealth { at_frame, .. }
            | Self::EntityHits { at_frame, .. }
            | Self::EntityCount { at_frame, .. }
            | Self::PlayerDowned { at_frame, .. }
            | Self::Currency { at_frame, .. }
            | Self::Stat { at_frame, .. }
            | Self::Event {
                by_frame: at_frame, ..
            }
            | Self::PlayerRevived {
                by_frame: at_frame, ..
            }
            | Self::Defeat {
                by_frame: at_frame, ..
            } => *at_frame,
            Self::NoDamageBetween { to_frame, .. } => *to_frame,
        }
    }
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

const ALL_BUTTONS: [Button; 15] = [
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
];

fn button_bit(button: Button) -> u16 {
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
        Button::Fire | Button::SwitchWeapon => 0,
    }
}
