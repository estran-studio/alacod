//! Attentes des gabarits d'armes et des scénarios : métadonnées hors simulation et checksum.
use run::RunEnd;
use serde::{Deserialize, Serialize};
use sim_core::{ammo::AmmoType, stats::StatId, team::Team};

/// Entité visée par une attente (T1.1, `Expectation::HitsAtLeast`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityRef {
    /// Entité rollback par son `GgrsNetId`.
    NetId(usize),
    /// L'entité qui compte les coups (`HitCount`) de plus petit `GgrsNetId` : `target` dans
    /// l'arène du testbed. Sert au `test.expect` d'une arme, où le `GgrsNetId` de `target`
    /// dépend de l'arme (voir `scenario::generate`).
    Target,
    /// Le personnage créé par le placement scripté d'index `n` de `Scenario::characters`
    /// (T1.13, `docs/conventions.md` §28) : sert au `test:` d'un personnage, dont le
    /// `GgrsNetId` dépend du gabarit (voir `scenario::generate`). Absent avant `at_frame`.
    Placed(usize),
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

/// Étape de `Run` attendue (T2.4, chantier F1, `Expectation::RunState`) : miroir minimal de
/// `run::run::RunStep` pour le format RON des scénarios — pas de `since_frame` (`Playing`)
/// ni `at_frame` (`Ended`, déjà le rôle du champ `at_frame` de l'attente elle-même).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStepExpectation {
    Playing,
    Ended(RunEnd),
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
    /// "drop", "pickup" (T2.2, chantier B7), "portal", "floor" (T1.8, mode `Floors`),
    /// "terrain" (T1.6, terrain de caverne creusé), "levelup", "mutation" (T1.10), "feedback"
    /// (T1.17 : "télégraphe", "hit stop", "secousse" dans le libellé).
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
    /// Tous les joueurs sont à terre ou morts (`run::run::Run::step`, `RunStep::Ended {
    /// outcome: RunEnd::Defeat, .. }`) à une frame ≤ `by_frame` (T1.3, chantier B6 ; T2.4,
    /// chantier F1 : ne lit plus `combat::downed::RunOutcome`, disparue).
    Defeat {
        by_frame: u32,
    },
    /// Étape de `Run` (T2.4, chantier F1) à la frame exacte `at_frame` — vérification
    /// ponctuelle (comme `PlayerAlive`), pas cumulative sur l'historique : une fois posée,
    /// `RunStep::Ended` ne change plus (voir `run::run::Run`), `at_frame` n'a donc qu'à être
    /// une frame ≥ celle où l'issue a été constatée.
    RunState {
        step: RunStepExpectation,
        at_frame: u32,
    },
    /// Résumé de fin de partie (T2.4, chantier F1, `run::run::RunSummary`) à `at_frame` :
    /// bornes inférieures plutôt que valeurs exactes (un joueur immobile ne tue jamais
    /// personne — voir le scénario `run_lose_summary` — `0` est une borne toujours vraie,
    /// pas une absence de vérification : elle documente que le résumé est cohérent avec
    /// « aucun kill »). Échoue si `Run.summary` est encore `None` à `at_frame` (la partie
    /// n'est pas terminée, ou `finalize_run_summary_system` n'a pas encore tourné — une
    /// seule frame de retard sur `RunState`, voir sa doc dans `game::run_state`).
    RunSummary {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        wave_reached_min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kills_min: Option<u32>,
        /// Niveau atteint minimal (mode `Floors`, T1.8 : `RunSummary::floor_reached`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        floor_reached_min: Option<u32>,
        at_frame: u32,
    },
    /// Index du niveau courant (mode `Floors`, T1.8 : `run::FloorState::index`, `0` = premier
    /// niveau) à la frame exacte `at_frame` — vérification ponctuelle, comme `PlayerAlive`.
    /// Hors mode `Floors`, l'index vaut toujours `0`.
    FloorIndex {
        index: u32,
        at_frame: u32,
    },
    /// Événement d'horloge `id` déjà déclenché (`fired: true`) ou pas encore (`false`) dans
    /// sa portée courante, à la frame exacte `at_frame` (T1.9, `run::Clock::fired`,
    /// `docs/conventions.md` §23). Un événement répété a l'id `"<id>#<n>"`.
    Clock {
        id: String,
        fired: bool,
        at_frame: u32,
    },
    /// État de la case `(x, y)` à la frame exacte `at_frame`, en cases de grille monde (16
    /// unités, **+y vers le haut**). `kind` (T1.6, `docs/conventions.md` §21) : nature du terrain
    /// d'une caverne (`world::CellGrid`, échoue hors de la grille — donc toujours hors caverne).
    /// `surface` (T1.7, §26) : nom de la surface de la case (`world::SurfaceGrid`, `"aucune"`
    /// pour une case sans surface). Au moins un des deux ; `kind: Rock` reste valide (RON des
    /// scénarios en `implicit_some`).
    CellState {
        x: i32,
        y: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<world::CellKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        surface: Option<String>,
        at_frame: u32,
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
        /// T1.5 : la stat d'une entité (ex. un ennemi à variante, `EnemyMoveSpeed`) plutôt que
        /// du joueur `handle` (alors ignoré). Absent : le joueur, comme avant.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<EntityRef>,
    },
    /// Nombre **exact** de projectiles vivants (`weapons::Bullet`) à `at_frame` (T1.1,
    /// chantier B5 v1). `projectile` : seulement les projectiles composables de cet id
    /// (`projectile::Projectile::id` : l'arme qui a tiré, ou l'entrée de sa table
    /// `projectiles` pour un projectile né d'un `on_expire`) ; `team` : seulement ceux de
    /// cette équipe de tireur. Sans filtre : toutes les balles.
    BulletCount {
        count: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projectile: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        team: Option<Team>,
        at_frame: u32,
    },
    /// Le compteur de coups (`HitCount`) de `entity` vaut au moins `hits` à `at_frame` (T1.1).
    /// Échoue si l'entité n'existe pas ou ne compte pas ses coups (comme `EntityHits`).
    HitsAtLeast {
        entity: EntityRef,
        hits: u32,
        at_frame: u32,
    },
    /// Le behavior retenu par l'ennemi `entity` à `at_frame` est `behavior` (nom de variante :
    /// `Chase`, `Melee`, `Shoot`, `KeepDistance`, `Strafe`, `Charge`, `Flee`, `Wander` ; T1.4,
    /// `docs/conventions.md` §22). Échoue si l'entité n'est pas un ennemi ou n'a aucune règle
    /// retenue.
    EnemyState {
        entity: EntityRef,
        behavior: String,
        at_frame: u32,
    },
    /// Distance de l'ennemi `entity` à `target` dans `[min, max]` (bornes inclusives, `None` =
    /// pas de borne ; `f32` convertis en `Fixed`) à `at_frame` (T1.4).
    EnemyDistance {
        entity: EntityRef,
        target: DistanceTarget,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f32>,
        at_frame: u32,
    },
    /// Variante du personnage `entity` à `at_frame` (T1.5, `docs/conventions.md` §25) :
    /// `Some("rapide")`, ou `None` = aucune variante (pas de composant `Variant`). Échoue si
    /// l'entité n'existe pas.
    EnemyVariant {
        entity: EntityRef,
        variant: Option<String>,
        at_frame: u32,
    },
    /// Diagnostic de navigation (T1.4) : l'ennemi `entity` arrive à portée de mêlée
    /// (`EnemyAiConfig::attack_range`) d'un joueur au plus tard à la frame `frames`. Attente
    /// **continue** : relevée à chaque frame jusqu'à `frames`.
    EnemyContactBefore {
        entity: EntityRef,
        frames: u32,
    },
    /// Diagnostic de navigation (T1.4) : le collider de l'ennemi `entity` ne chevauche aucun
    /// `Wall` à aucune frame de `from` à `to` inclus. Attente **continue** (comme
    /// `NoDamageBetween`) : la première frame fautive est rapportée.
    EnemyNeverInWall {
        entity: EntityRef,
        from: u32,
        to: u32,
    },
    /// Jauge `id` (`game::effects_runtime::Gauges`, ex. les rads de la progression) du joueur
    /// `handle` dans `[min, max]` (T1.10, `docs/conventions.md` §27). Échoue si le joueur n'a
    /// pas cette jauge.
    Gauge {
        handle: usize,
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f32>,
        at_frame: u32,
    },
    /// L'entité `entity` porte le statut `status` (id du kind `Status`, `combat::status::
    /// Statuses`) à `at_frame` (T1.3, `docs/conventions.md` §19) ; `present: false` vérifie
    /// au contraire qu'elle ne le porte pas (expiré). Échoue si l'entité n'existe pas.
    HasStatus {
        entity: EntityRef,
        status: String,
        #[serde(default = "default_true")]
        present: bool,
        at_frame: u32,
    },
    /// Nombre **exact** de piles du statut `status` sur `entity` à `at_frame` (T1.3 ; 0 si le
    /// statut n'est pas porté).
    StatusStacks {
        entity: EntityRef,
        status: String,
        stacks: u32,
        at_frame: u32,
    },
    /// Niveau **exact** (`game::progression::Level`) du joueur `handle` (T1.10). Échoue sans
    /// progression active.
    Level {
        handle: usize,
        level: u32,
        at_frame: u32,
    },
    /// Les mutations prises par le joueur `handle` (`game::progression::Mutations`) contiennent
    /// toutes celles de `contains` et, si `count` est donné, sont exactement `count` (T1.10 ;
    /// `count: 0` : aucune mutation prise, joueur sans `Mutations` compris).
    Mutations {
        handle: usize,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        contains: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u32>,
        at_frame: u32,
    },
    /// Phase courante du boss `entity` (`behaviors::BossState::phase`, M2-T0c,
    /// `docs/conventions.md` §37) à la frame exacte `at_frame` ; échoue sans `BossState`.
    BossPhase {
        entity: EntityRef,
        phase: u32,
        at_frame: u32,
    },
    /// Le profil de méta-progression du joueur local `handle` (`meta::Profiles`, scénario
    /// `profile: true`, M2-T0c, §38) possède le déblocage `unlock` et/ou au moins `min` de la
    /// monnaie `id` (`currency: Some((id, min))`) ; évaluée après la fin de run.
    ProfileHas {
        handle: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unlock: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        currency: Option<(String, u64)>,
        at_frame: u32,
    },
    /// Le texte de la source `source` du HUD (`game::ui::hud_model::HudSnapshot`, joueur
    /// affiché, sans préfixe) contient `contains` à la frame exacte `at_frame` (T1.18,
    /// `docs/conventions.md` §32). Présentation : lu hors simulation, hors trace.
    HudText {
        source: String,
        contains: String,
        at_frame: u32,
    },
    /// Le joueur `handle` tient l'objet `id` (passif ou actif, `items::Inventory`) à la frame
    /// exacte `at_frame` (M2-T0b, `docs/conventions.md` §36).
    HasItem {
        handle: usize,
        id: String,
        at_frame: u32,
    },
    /// Charge exacte de l'objet actif tenu par le joueur `handle` ; échoue sans actif.
    ItemCharge {
        handle: usize,
        charge: u32,
        at_frame: u32,
    },
    /// Nombre exact de consommables `id` du joueur `handle` (0 si l'inventaire est absent).
    Consumable {
        handle: usize,
        id: String,
        count: u32,
        at_frame: u32,
    },
    /// État de la `nth` salle de type `kind` (`room_kind`, salles triées par position x puis y,
    /// `world::RoomStates`, M2-E1, `docs/conventions.md` §35) à la frame exacte `at_frame` ;
    /// échoue si la carte n'a pas assez de salles de ce type. `doors_closed` (optionnel) : le
    /// nombre de portes fermées (avec collider) de **toute** la carte.
    RoomState {
        kind: String,
        #[serde(default)]
        nth: usize,
        state: world::RoomState,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        doors_closed: Option<u32>,
        at_frame: u32,
    },
}

/// Cible d'une distance (`EnemyDistance`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DistanceTarget {
    /// Le joueur de handle GGRS `0`, `1`, ...
    Player(usize),
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
            | Self::BulletCount { at_frame, .. }
            | Self::HitsAtLeast { at_frame, .. }
            | Self::EnemyState { at_frame, .. }
            | Self::EnemyDistance { at_frame, .. }
            | Self::EnemyVariant { at_frame, .. }
            | Self::Clock { at_frame, .. }
            | Self::RunState { at_frame, .. }
            | Self::RunSummary { at_frame, .. }
            | Self::FloorIndex { at_frame, .. }
            | Self::CellState { at_frame, .. }
            | Self::Gauge { at_frame, .. }
            | Self::HasStatus { at_frame, .. }
            | Self::StatusStacks { at_frame, .. }
            | Self::Level { at_frame, .. }
            | Self::Mutations { at_frame, .. }
            | Self::HudText { at_frame, .. }
            | Self::RoomState { at_frame, .. }
            | Self::HasItem { at_frame, .. }
            | Self::ItemCharge { at_frame, .. }
            | Self::Consumable { at_frame, .. }
            | Self::BossPhase { at_frame, .. }
            | Self::ProfileHas { at_frame, .. }
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
            Self::EnemyContactBefore { frames, .. } => *frames,
            Self::EnemyNeverInWall { to, .. } => *to,
        }
    }
}

fn default_true() -> bool {
    true
}
