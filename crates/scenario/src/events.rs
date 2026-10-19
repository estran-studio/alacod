//! Moments clés d'une partie (vague, kills, morts, fenêtres, portes, armes, power-ups, fin
//! de partie), détectés en comparant l'état du jeu d'une frame à l'autre. Affichés sous les
//! vidéos de la page de revue pour aller directement aux instants importants.
//!
//! M2 : `item_pickup`, `blank`, `room`, `boss_phase`, `purchase` (monnaie, dont l'achat en
//! boutique de raison `shop`), `chest` (coffre ouvert), `arena` (arène tenue), `portal`, `floor`,
//! `unlock` (déblocages du profil lus par la partie), `hub` (run entrée depuis le hub) et, M3-T9,
//! `curse` (curse tirée à l'entrée d'un étage).
//!
//! Hors simulation : lit l'état après chaque update, ne modifie rien.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use combat::downed::Downed;
use combat::inventory::AmmoReserves;
use game::{
    character::{health::Health, player::Player},
    collider::Collider,
    powerups::{PowerUpPickedUp, PowerUpPickup},
    waves::{WavePhase, WaveState},
    weapons::{WeaponInventory, WeaponModesState, WeaponPickup, WeaponState},
};
use map::game::entity::map::{door::DoorComponent, window::WindowHealth};
use run::currency::CurrencyEvent;
use run::{Run, RunEnd, RunStep, RunSummary};
use serde::Serialize;
use sim_core::frame_events::FrameEvents;
use utils::{frame::FrameCount, net_id::GgrsNetId};

/// Un moment clé : sa frame, sa catégorie (pour le style) et son libellé.
#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub frame: u32,
    pub kind: &'static str,
    pub label: String,
}

#[derive(Resource, Default)]
pub struct GameEvents {
    pub events: Vec<GameEvent>,
    /// Résumé de fin de partie (T2.4, chantier F1, `run::run::Run::summary`) : copié ici
    /// dès qu'il apparaît (`Run.step` devient `Ended` et `finalize_run_summary_system` le
    /// calcule), pour que les outils qui consomment déjà `GameEvents` (page de revue, CI de
    /// nuit) n'aient pas besoin de lire `Run` séparément. `None` tant que la partie n'est
    /// pas terminée.
    pub summary: Option<RunSummary>,
    previous: Option<Snapshot>,
}

#[derive(Default, Clone, PartialEq)]
struct PlayerSnapshot {
    reloading: bool,
    weapon_index: usize,
    hit: bool,
    /// Arme active : nom, mode, munitions du chargeur et réserve du type de munition
    weapon: String,
    mode: String,
    ammo: u32,
    /// Réserve du type de munition de l'arme active (T2.2, chantier B7,
    /// `combat::inventory::AmmoReserves`) : remplace l'ancien `mag_quantity` (chargeurs de
    /// réserve propres à l'arme), désormais partagé par type entre les armes d'un joueur.
    reserve: u32,
    dashing: bool,
    sprinting: bool,
    melee: bool,
    position: (i32, i32),
    /// À terre (T1.3, chantier B6) : `combat::downed::Downed` présent sur ce joueur.
    downed: bool,
    /// T1.10 : niveau de progression et mutations prises.
    level: u32,
    mutations: Vec<String>,
}

#[derive(Default, Clone)]
struct Snapshot {
    wave: u32,
    /// T1.8 : niveau du mode `Floors` (`FloorState::index`) et portail ouvert.
    floor: u32,
    portal_open: bool,
    phase: Option<WavePhase>,
    kills: u32,
    players: BTreeMap<usize, PlayerSnapshot>,
    windows: BTreeMap<usize, u8>,
    closed_doors: BTreeSet<usize>,
    open_doors: BTreeSet<usize>,
    /// Armes au sol (T2.2, chantier B7) : `GgrsNetId` -> id de l'arme
    /// (`WeaponPickup::weapon_id`). Un id qui apparaît = lâcher (`drop`) ; un id qui
    /// disparaît = ramassage (`pickup`). D21 : sans les armes murales (`price: Some`,
    /// jamais consommées) : elles apparaissent avec la carte et n'ont jamais été lâchées
    /// (leur achat est déjà le moment clé `purchase`).
    weapon_pickups: BTreeMap<usize, String>,
    /// D21 : power-ups au sol (`game::powerups::PowerUpPickup`), `GgrsNetId` -> id.
    powerup_pickups: BTreeMap<usize, String>,
    /// T1.9 : événements d'horloge déjà déclenchés (`run::Clock::fired`) ; un id nouveau =
    /// moment clé `clock`.
    clock_fired: std::collections::BTreeSet<String>,
    /// T1.5 : personnages à variante (`game::character::variant::Variant`), `GgrsNetId` ->
    /// nom ; une entrée nouvelle = moment clé `variant_spawn`.
    variants: BTreeMap<usize, String>,
    /// Issue de la partie (`run::run::Run::step` : `RunStep::Ended { outcome, .. }`), `None`
    /// tant qu'elle n'est pas finie (T2.4, chantier F1 ; D21 : victoire et abandon en plus
    /// de la défaite).
    outcome: Option<RunEnd>,
}

/// D21 : moment clé de fin de partie, à la frame où `Run.step` devient `Ended`. La défaite
/// garde sa catégorie et son libellé d'avant D21 : l'attente `Defeat` (`runner.rs`) les lit.
fn outcome_event(before: Option<RunEnd>, now: Option<RunEnd>) -> Option<(&'static str, String)> {
    if before.is_some() {
        return None;
    }
    match now? {
        RunEnd::Defeat => Some((
            "defeat",
            "défaite : tous les joueurs sont à terre ou morts".to_string(),
        )),
        RunEnd::Victory => Some((
            "victory",
            "victoire : condition du mode atteinte".to_string(),
        )),
        RunEnd::Abandon => Some(("abandon", "partie abandonnée (retour au lobby)".to_string())),
    }
}

/// D21 : power-ups au sol entre deux frames. `picked` : les ramassages de la frame
/// (`FrameEvents<PowerUpPickedUp>`), `(id du power-up, handle du joueur s'il est connu)`.
/// Un power-up qui apparaît = `powerup` ; un ramassage = `powerup_pickup` ; un power-up
/// disparu sans ramassage du même id cette frame = expiré (`powerup`).
fn powerup_events(
    before: &BTreeMap<usize, String>,
    now: &BTreeMap<usize, String>,
    picked: &[(String, Option<usize>)],
) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (net_id, id) in now {
        if !before.contains_key(net_id) {
            out.push(("powerup", format!("power-up {id} au sol ({net_id})")));
        }
    }
    for (id, handle) in picked {
        let by = handle.map_or_else(|| "un joueur".to_string(), |h| format!("joueur {h}"));
        out.push(("powerup_pickup", format!("power-up {id} ramassé par {by}")));
    }
    for (net_id, id) in before {
        if !now.contains_key(net_id) && !picked.iter().any(|(picked_id, _)| picked_id == id) {
            out.push(("powerup", format!("power-up {id} expiré ({net_id})")));
        }
    }
    out
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameEvents>()
            .init_resource::<HubFlow>()
            .add_systems(
                Last,
                (
                    hub_events.before(detect_events),
                    detect_events,
                    feedback_events.after(detect_events),
                    room_events.after(detect_events),
                    explosion_events.after(detect_events),
                    item_events.after(detect_events),
                    blank_events.after(detect_events),
                    boss_events.after(detect_events),
                    chest_events.after(detect_events),
                    unlock_events.after(detect_events).after(hub_events),
                    lock_events.after(detect_events),
                    curse_events.after(detect_events),
                ),
            );
    }
}

/// Moment clé `curse` (M3-T9, `docs/conventions.md` §74) : la curse d'un étage tirée à son entrée
/// (`run::RunCurses`). Un étage sans curse n'est pas un moment clé. Une relance (état remis à zéro)
/// ou un rollback qui défait le tirage le fait oublier : il est annoncé de nouveau s'il est retiré.
fn curse_events(
    frame: Res<FrameCount>,
    curses: Option<Res<run::RunCurses>>,
    mut announced: Local<BTreeMap<u32, String>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(curses) = curses else {
        return;
    };
    announced.retain(|floor, id| curses.floors.get(floor) == Some(id));
    for (floor, id) in &curses.floors {
        if announced.insert(*floor, id.clone()).as_ref() != Some(id) {
            events.events.push(GameEvent {
                frame: frame.frame,
                kind: "curse",
                label: format!("étage {} : curse « {id} »", floor + 1),
            });
        }
    }
}

/// Moments clés `feedback` (T1.17, `docs/conventions.md` §31) : relus de
/// `game::feedback::FeedbackLog` (journal de présentation, rempli aussi en headless). Seuls
/// les indices rares deviennent des moments clés : télégraphe, hit stop, secousse (le flash
/// et le chiffre accompagnent chaque coup, déjà visibles par les moments `hit`). Libellé :
/// « télégraphe 34 (30 frames, rayon 48) », « hit stop 35 (4 frames) », « secousse 26 (8
/// frames) ». M2-T19 : forme du pattern, blank, fin de boss.
fn feedback_events(log: Option<Res<game::feedback::FeedbackLog>>, mut events: ResMut<GameEvents>) {
    use game::feedback::FeedbackKind;
    let Some(log) = log else {
        return;
    };
    for cue in &log.frame_cues {
        let label = match cue.kind {
            FeedbackKind::Telegraph => format!(
                "{} {} ({} frames, rayon {})",
                cue.kind.label(),
                cue.net_id,
                cue.value,
                cue.extra
            ),
            FeedbackKind::HitStop | FeedbackKind::Shake => {
                format!("{} {} ({} frames)", cue.kind.label(), cue.net_id, cue.value)
            }
            // M2-T19 (§55) : « télégraphe fan 31 (5 rayons) », « blank 0 (rayon 160) »,
            // « fin de boss 40 ».
            FeedbackKind::PatternTelegraph => match &cue.pattern {
                Some(shape) => format!(
                    "{} {}",
                    cue.kind.label(),
                    game::feedback::pattern_label(cue.net_id, shape)
                ),
                None => continue,
            },
            FeedbackKind::Blank => {
                format!("{} {} (rayon {})", cue.kind.label(), cue.value, cue.extra)
            }
            FeedbackKind::BossEnd => format!("{} {}", cue.kind.label(), cue.net_id),
            // M3-T17 (§82) : « larme 12 (guidée, poison) », « familier 31 (suiveur, porteur 0, rang
            // 0) », « champion 14 (explosif) », « mèche 31 (20 frames) », « anneau d'explosion
            // (rayon 70) », « rocher cassé (12,3) », « mur secret ouvert (salle) » ; suivi Q10 : « transformation
            // 8 (champignons) ».
            FeedbackKind::Tear
            | FeedbackKind::Familiar
            | FeedbackKind::Champion
            | FeedbackKind::BombFuse
            | FeedbackKind::Explosion
            | FeedbackKind::RockBroken
            | FeedbackKind::SecretWall
            | FeedbackKind::Transformation => match game::feedback_v3::cue_label(cue) {
                Some(label) => label,
                None => continue,
            },
            FeedbackKind::Flash | FeedbackKind::Number => continue,
        };
        events.events.push(GameEvent {
            frame: cue.frame,
            kind: "feedback",
            label,
        });
    }
}

/// Moments clés `item_pickup` (M2-T0b) : un objet ramassé (`items::ItemPicked`, borné à sa
/// frame d'émission comme `CurrencyEvent`).
fn item_events(
    picked: Option<Res<FrameEvents<items::ItemPicked>>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(picked) = picked else {
        return;
    };
    for pick in picked.iter() {
        events.events.push(GameEvent {
            frame: pick.frame,
            kind: "item_pickup",
            label: format!("joueur {} ramasse {}", pick.player_handle, pick.item_id),
        });
    }
}

/// Moments clés `blank` (M2-T1) : un blank utilisé (`items::BlankUsed`).
fn blank_events(used: Option<Res<FrameEvents<items::BlankUsed>>>, mut events: ResMut<GameEvents>) {
    let Some(used) = used else {
        return;
    };
    for blank in used.iter() {
        events.events.push(GameEvent {
            frame: blank.frame,
            kind: "blank",
            label: format!(
                "joueur {} utilise un blank ({} balles effacées)",
                blank.player_handle, blank.cleared
            ),
        });
    }
}

/// Moments clés `room` (M2-E1, `docs/conventions.md` §35) : un changement d'état de salle
/// (`world::RoomChanged`, borné à la frame d'émission comme `CurrencyEvent`) : « salle combat
/// verrouillée », « salle combat nettoyée ».
///
/// M2-full-run : une salle dont le type porte une arène (`RoomKindTable::arena`) qui passe de
/// verrouillée à nettoyée donne en plus le moment `arena` « arène tenue » : l'horloge est allée
/// à son terme avec un joueur en vie (si tous meurent, la salle ne se rouvre pas, voir §45).
fn room_events(
    changes: Option<Res<FrameEvents<world::RoomChanged>>>,
    kinds: Option<Res<world::RoomKindTable>>,
    mut locked_at: Local<BTreeMap<String, u32>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(changes) = changes else {
        return;
    };
    for change in changes.iter() {
        let what = match change.to {
            world::RoomState::Dormant => "endormie",
            world::RoomState::Locked => "verrouillée",
            world::RoomState::Cleared => "nettoyée",
        };
        events.events.push(GameEvent {
            frame: change.frame,
            kind: "room",
            label: format!("salle {} {what}", change.kind),
        });
        let is_arena = kinds
            .as_ref()
            .is_some_and(|kinds| kinds.arena(&change.kind).is_some());
        if !is_arena {
            continue;
        }
        match change.to {
            world::RoomState::Locked => {
                locked_at.insert(change.room.clone(), change.frame);
            }
            world::RoomState::Cleared => {
                let held = locked_at
                    .remove(&change.room)
                    .map_or(0, |since| change.frame.saturating_sub(since));
                events.events.push(GameEvent {
                    frame: change.frame,
                    kind: "arena",
                    label: format!("arène {} tenue ({held} frames)", change.kind),
                });
            }
            world::RoomState::Dormant => {}
        }
    }
}

/// Moments clés `chest` (M2-full-run) : un coffre ouvert (`items::ChestOpened`, borné à sa frame
/// d'émission). La clé d'un coffre verrouillé est consommée dans la même frame (attente
/// `ItemCount`/`Consumable` pour le prouver), le libellé donne l'objet tiré.
fn chest_events(
    opened: Option<Res<FrameEvents<items::ChestOpened>>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(opened) = opened else {
        return;
    };
    for chest in opened.iter() {
        events.events.push(GameEvent {
            frame: chest.frame,
            kind: "chest",
            label: format!(
                "joueur {} ouvre le coffre {} : {}",
                chest.player_handle, chest.chest_key, chest.item_id
            ),
        });
    }
}

/// Où en est la session par rapport au hub (M2-T13, §57), pour les moments clés : `Idle` sans hub
/// ou une fois la run commencée, `InHub` pendant le hub, `Leaving` entre la confirmation de la
/// porte et la première frame de la run.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum HubFlow {
    #[default]
    Idle,
    InHub,
    Leaving,
}

/// Moment clé `hub` (m2-full-run, M2-T13) : la run démarre depuis le hub. Les frames repartent de
/// 0 à la run : les moments clés du hub (rien d'utile : pas d'ennemi) sont écartés à la relance,
/// et « entrée en run depuis le hub » est daté de la première frame de la run, avec la frame du hub
/// où le joueur a franchi la porte (`HubState::exit`) dans le libellé.
fn hub_events(
    frame: Res<FrameCount>,
    session: Option<Res<game::hub::HubSession>>,
    hub: Option<Res<game::hub::HubState>>,
    app_state: Option<Res<State<game::session::AppState>>>,
    mut flow: ResMut<HubFlow>,
    mut exit: Local<Option<game::hub::HubExit>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(session) = session else {
        return;
    };
    if session.in_hub {
        *flow = HubFlow::InHub;
        if let Some(decided) = hub.as_ref().and_then(|hub| hub.exit) {
            *exit = Some(decided);
        }
        return;
    }
    match *flow {
        HubFlow::Idle => {}
        HubFlow::InHub => {
            *flow = HubFlow::Leaving;
            events.events.clear();
            events.previous = None;
        }
        HubFlow::Leaving => {
            let running = app_state
                .is_some_and(|state| *state.get() == game::session::AppState::InGame)
                && frame.frame >= 1;
            if !running {
                events.events.clear();
                events.previous = None;
                return;
            }
            *flow = HubFlow::Idle;
            let label = match exit.take() {
                Some(decided) => format!(
                    "entrée en run depuis le hub (joueur {} franchit la porte, f{} du hub)",
                    decided.handle, decided.frame
                ),
                None => "entrée en run depuis le hub".to_string(),
            };
            events.events.push(GameEvent {
                frame: frame.frame,
                kind: "hub",
                label,
            });
        }
    }
}

/// Moment clé `unlock` (M2-full-run, M2-T12) : les déblocages lus du profil par la partie
/// (`items::RunUnlocks`, figés au lancement), signalés quand l'ensemble change et n'est pas vide.
/// Au hub, rien : les déblocages comptent à la run (`HubFlow`).
fn unlock_events(
    frame: Res<FrameCount>,
    flow: Res<HubFlow>,
    unlocks: Option<Res<items::RunUnlocks>>,
    mut announced: Local<std::collections::BTreeSet<String>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(unlocks) = unlocks else {
        return;
    };
    if *flow != HubFlow::Idle || unlocks.0.is_empty() || unlocks.0 == *announced {
        return;
    }
    announced.clone_from(&unlocks.0);
    events.events.push(GameEvent {
        frame: frame.frame,
        kind: "unlock",
        label: format!(
            "déblocages du profil dans la partie : {}",
            unlocks.0.iter().cloned().collect::<Vec<_>>().join(", ")
        ),
    });
}

/// Moments clés `explosion` (M3-T0c, `docs/conventions.md` §62) : une explosion (`world::Exploded`).
/// Déclaré ici, émis par M3-T4 : aucun scénario existant n'en produit.
fn explosion_events(
    exploded: Option<Res<FrameEvents<world::Exploded>>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(exploded) = exploded else {
        return;
    };
    for explosion in exploded.iter() {
        events.events.push(GameEvent {
            frame: explosion.frame,
            kind: "explosion",
            label: format!(
                "explosion en ({}, {}) rayon {}",
                explosion.x, explosion.y, explosion.radius
            ),
        });
    }
}

/// Moments clés `door_lock` et `bomb` (M3-T14, `docs/conventions.md` §79) : une salle entre dans
/// `DoorLocks` (mur secret ouvert par une explosion, ou porte à clé ouverte : le libellé dit laquelle)
/// et une bombe est posée. Lus dans l'état (`DoorLocks`, `Bomb`) : un rollback qui défait l'ouverture la
/// fait oublier (`retain`), elle est annoncée de nouveau si elle revient.
#[allow(clippy::too_many_arguments)]
fn lock_events(
    frame: Res<FrameCount>,
    locks: Option<Res<world::DoorLocks>>,
    table: Option<Res<world::RoomKindTable>>,
    floor: Option<Res<run::FloorState>>,
    rooms: map_ldtk::game::rooms::CurrentRooms,
    bombs: Query<(&GgrsNetId, &world::Bomb)>,
    shelves: Query<(&GgrsNetId, &items::Shelf)>,
    mut opened: Local<BTreeSet<(u32, String)>>,
    mut placed: Local<BTreeSet<usize>>,
    mut sold: Local<BTreeSet<usize>>,
    mut events: ResMut<GameEvents>,
) {
    let index = floor.as_ref().map_or(0, |floor| floor.index);
    if let (Some(locks), Some(table)) = (locks, table) {
        opened.retain(|(at, room)| *at != index || locks.is_open(room));
        for (level, _, kind) in rooms.iter() {
            if locks.is_open(&level.0) && opened.insert((index, level.0.clone())) {
                let what = match table.door_lock(&kind.0) {
                    Some(world::DoorLock::Secret) => "mur secret ouvert",
                    Some(world::DoorLock::Key(_)) => "porte à clé ouverte",
                    None => "porte ouverte",
                };
                events.events.push(GameEvent {
                    frame: frame.frame,
                    kind: "door_lock",
                    label: format!("{what} : salle {} (niveau {index})", kind.0),
                });
            }
        }
    }
    let alive: BTreeSet<usize> = bombs.iter().map(|(id, _)| id.0).collect();
    placed.retain(|id| alive.contains(id));
    for (id, bomb) in &bombs {
        if placed.insert(id.0) {
            events.events.push(GameEvent {
                frame: frame.frame,
                kind: "bomb",
                label: format!("bombe posée par {}", bomb.owner.0),
            });
        }
    }
    let on_sale: BTreeSet<usize> = shelves
        .iter()
        .filter(|(_, shelf)| shelf.sold)
        .map(|(id, _)| id.0)
        .collect();
    sold.retain(|id| on_sale.contains(id));
    for (id, shelf) in &shelves {
        if shelf.sold && sold.insert(id.0) {
            events.events.push(GameEvent {
                frame: frame.frame,
                kind: "shelf",
                label: format!(
                    "présentoir {} vendu : {} ({})",
                    shelf.key,
                    shelf.item_id,
                    if shelf.cost.is_some() {
                        "diable"
                    } else {
                        "douilles"
                    }
                ),
            });
        }
    }
}

/// Moments clés `boss_phase` (M2-T0c) : un boss change de phase (`behaviors::BossPhaseChanged`,
/// borné à sa frame d'émission comme `CurrencyEvent`).
fn boss_events(
    changes: Option<Res<FrameEvents<behaviors::BossPhaseChanged>>>,
    mut events: ResMut<GameEvents>,
) {
    let Some(changes) = changes else {
        return;
    };
    for change in changes.iter() {
        events.events.push(GameEvent {
            frame: change.frame,
            kind: "boss_phase",
            label: format!(
                "boss {} : phase {} → {}",
                change.net_id, change.from, change.to
            ),
        });
    }
}

fn phase_label(phase: WavePhase) -> &'static str {
    match phase {
        WavePhase::NotStarted => "pas commencée",
        WavePhase::GracePeriod => "préparation",
        WavePhase::Spawning => "arrivée des zombies",
        WavePhase::InProgress => "tous les zombies sont là",
        WavePhase::WaveComplete => "vague terminée",
    }
}

#[allow(clippy::too_many_arguments)]
fn detect_events(
    frame: Res<FrameCount>,
    mut events: ResMut<GameEvents>,
    wave: Option<Res<WaveState>>,
    run: Option<Res<Run>>,
    // (étage, horloge) groupés : une fonction système accepte au plus 16 paramètres.
    floor_and_clock: (Option<Res<run::FloorState>>, Option<Res<run::Clock>>),
    players: Query<(
        &Player,
        &Health,
        Option<&WeaponInventory>,
        Option<&AmmoReserves>,
        Option<&game::character::dash::DashState>,
        Option<&game::character::movement::SprintState>,
        Option<&game::weapons::melee::MeleeAttackState>,
        &bevy_fixed::fixed_math::FixedTransform3D,
        Has<Downed>,
        Option<&game::progression::Level>,
        Option<&game::progression::Mutations>,
    )>,
    weapons: Query<(&WeaponState, &WeaponModesState)>,
    windows: Query<(&GgrsNetId, &WindowHealth)>,
    // Porte ouverte = sans collider (une porte non interactive reste fermée)
    doors: Query<(&GgrsNetId, Has<Collider>), With<DoorComponent>>,
    // T2.2, chantier B7 : armes tombées au sol.
    weapon_pickups: Query<(&GgrsNetId, &WeaponPickup)>,
    // D21 : power-ups au sol et leurs ramassages (même lecture que `currency_events`).
    powerup_pickups: Query<(&GgrsNetId, &PowerUpPickup)>,
    powerup_picked: Option<Res<FrameEvents<PowerUpPickedUp>>>,
    player_ids: Query<(&GgrsNetId, &Player)>,
    // T2.3, chantier C5 v1 : monnaie (points, achats, refus). Lu directement (pas de
    // diffing par snapshot comme le reste de cette fonction) : `CurrencyEvent` est déjà un
    // événement borné à sa frame d'émission (`FrameEvents`, vidé au `FrameStart` suivant),
    // encore valide ici (`Last` tourne après la dernière frame GGRS simulée cet `Update`).
    currency_events: Res<FrameEvents<CurrencyEvent>>,
    // T1.6 : terrain creusé dans la frame (file neutre, absente hors jeu complet).
    terrain_events: Option<Res<FrameEvents<world::TerrainDestroyed>>>,
    // T1.5 : variantes (moment clé `variant_spawn`).
    variants: Query<(&GgrsNetId, &game::character::variant::Variant)>,
) {
    let (floor_state, clock) = floor_and_clock;
    let mut now = Snapshot::default();
    if let Some(wave) = &wave {
        now.wave = wave.current_wave;
        now.phase = Some(wave.phase);
        now.kills = wave.total_enemies_killed;
    }
    if let Some(floor_state) = &floor_state {
        now.floor = floor_state.index;
        now.portal_open = floor_state.portal_open;
    }
    now.outcome = run.as_deref().and_then(|run| match run.step {
        RunStep::Ended { outcome, .. } => Some(outcome),
        _ => None,
    });
    // T2.4, chantier F1 : copie le résumé dans `GameEvents` dès qu'il apparaît (voir la
    // doc du champ) ; ne l'efface jamais (un rollback qui défait `Run.step` avant la
    // confirmation de frame n'a pas de sens ici, `events` lui-même n'est jamais rejoué).
    if events.summary.is_none() {
        events.summary = run.as_deref().and_then(|run| run.summary);
    }
    for (
        player,
        health,
        inventory,
        ammo_reserves,
        dash,
        sprint,
        melee,
        transform,
        downed,
        level,
        mutations,
    ) in &players
    {
        let mut snapshot = PlayerSnapshot {
            reloading: inventory.is_some_and(|i| i.reloading_ending_frame.is_some()),
            weapon_index: inventory.map_or(0, |i| i.active_weapon_index),
            hit: health.current < health.max,
            dashing: dash.is_some_and(|d| d.is_dashing),
            sprinting: sprint.is_some_and(|s| s.is_sprinting),
            melee: melee.is_some_and(|m| m.is_attacking),
            position: (
                transform.translation.x.to_num::<i32>(),
                transform.translation.y.to_num::<i32>(),
            ),
            downed,
            level: level.map_or(0, |l| l.0),
            mutations: mutations.map(|m| m.0.clone()).unwrap_or_default(),
            ..Default::default()
        };
        if let Some((entity, weapon)) = inventory.and_then(|i| i.weapons.get(i.active_weapon_index))
        {
            snapshot.weapon = weapon.config.name.clone();
            snapshot.reserve = ammo_reserves.map_or(0, |r| r.get(&weapon.config.ammo_type));
            if let Ok((state, modes)) = weapons.get(*entity) {
                snapshot.mode = state.active_mode.clone();
                if let Some(mode) = modes.modes.get(&state.active_mode) {
                    snapshot.ammo = mode.mag_ammo;
                }
            }
        }
        now.players.insert(player.handle, snapshot);
    }
    for (id, health) in &windows {
        now.windows.insert(id.0, health.current);
    }
    for (id, closed) in &doors {
        if closed {
            now.closed_doors.insert(id.0);
        } else {
            now.open_doors.insert(id.0);
        }
    }
    for (id, pickup) in &weapon_pickups {
        if pickup.price.is_none() {
            now.weapon_pickups.insert(id.0, pickup.weapon_id.clone());
        }
    }
    for (id, pickup) in &powerup_pickups {
        now.powerup_pickups.insert(id.0, pickup.id.clone());
    }
    for (id, variant) in &variants {
        now.variants.insert(id.0, variant.0.clone());
    }
    if let Some(clock) = &clock {
        now.clock_fired = clock.fired.clone();
    }
    // Ramassages de cette frame, avec le handle du ramasseur (`picked_up_by` est son
    // `GgrsNetId`) : lu avant `events.previous.replace`, comme `currency_events` plus bas.
    let picked: Vec<(String, Option<usize>)> = powerup_picked
        .as_deref()
        .map(|picked| {
            picked
                .iter()
                .map(|event| {
                    let handle = player_ids
                        .iter()
                        .find(|(net_id, _)| **net_id == event.picked_up_by)
                        .map(|(_, player)| player.handle);
                    (event.id.clone(), handle)
                })
                .collect()
        })
        .unwrap_or_default();

    // Première frame vue : référence, rien à signaler (sauf joueurs présents)
    let Some(before) = events.previous.replace(now.clone()) else {
        return;
    };
    let f = frame.frame;
    let mut push = |kind: &'static str, label: String| {
        events.events.push(GameEvent {
            frame: f,
            kind,
            label,
        });
    };

    if now.wave != before.wave || now.phase != before.phase {
        if let Some(phase) = now.phase {
            push(
                "wave",
                format!("vague {} : {}", now.wave, phase_label(phase)),
            );
        }
    }
    // T1.8 : portail ouvert, passage au niveau suivant (mode `Floors`).
    if now.portal_open && !before.portal_open {
        push("portal", format!("portail ouvert (niveau {})", now.floor));
    }
    if now.floor > before.floor {
        push("floor", format!("niveau {}", now.floor));
    }
    if now.kills > before.kills {
        push("kill", format!("zombie tué ({} au total)", now.kills));
    }
    for (handle, player) in &now.players {
        match before.players.get(handle) {
            None => push("player", format!("joueur {handle} apparaît")),
            Some(previous) => {
                if player.hit && !previous.hit {
                    push("hit", format!("joueur {handle} touché"));
                }
                if player.reloading && !previous.reloading {
                    push(
                        "reload",
                        format!(
                            "joueur {handle} recharge ({} {})",
                            player.weapon, player.mode
                        ),
                    );
                }
                if !player.reloading && previous.reloading {
                    push(
                        "reload",
                        format!(
                            "joueur {handle} a rechargé : {} ({} en réserve)",
                            player.ammo, player.reserve
                        ),
                    );
                }
                if player.weapon_index != previous.weapon_index {
                    push(
                        "weapon",
                        format!(
                            "joueur {handle} prend {} ({}, {} balles)",
                            player.weapon, player.mode, player.ammo
                        ),
                    );
                } else if player.mode != previous.mode {
                    push(
                        "weapon",
                        format!(
                            "joueur {handle} passe en mode {} ({} balles)",
                            player.mode, player.ammo
                        ),
                    );
                }
                if player.dashing && !previous.dashing {
                    push(
                        "move",
                        format!("joueur {handle} dash depuis {:?}", previous.position),
                    );
                }
                if !player.dashing && previous.dashing {
                    push(
                        "move",
                        format!("joueur {handle} fin du dash en {:?}", player.position),
                    );
                }
                if player.sprinting != previous.sprinting {
                    let what = if player.sprinting {
                        "sprinte"
                    } else {
                        "arrête de sprinter"
                    };
                    push("move", format!("joueur {handle} {what}"));
                }
                if player.melee && !previous.melee {
                    push("melee", format!("joueur {handle} attaque au corps à corps"));
                }
                // À terre (T1.3, chantier B6). `revived` seulement quand le joueur est
                // toujours là (`now.players` le contient) : `Downed` retiré par une mort
                // de saignement, sans joueur qui reste, tombe dans la boucle `death`
                // ci-dessous, pas ici.
                if player.downed && !previous.downed {
                    push("downed", format!("joueur {handle} tombe à terre"));
                }
                if !player.downed && previous.downed {
                    push("revived", format!("joueur {handle} réanimé"));
                }
                // T1.10 : progression
                if player.level > previous.level {
                    push(
                        "levelup",
                        format!("joueur {handle} passe niveau {}", player.level),
                    );
                }
                if player.mutations.len() > previous.mutations.len() {
                    push(
                        "mutation",
                        format!(
                            "joueur {handle} prend la mutation {}",
                            player.mutations.last().map_or("?", String::as_str)
                        ),
                    );
                }
                if player.weapon_index == previous.weapon_index
                    && player.ammo == 0
                    && previous.ammo > 0
                {
                    push(
                        "weapon",
                        format!("joueur {handle} : chargeur vide ({})", player.weapon),
                    );
                }
            }
        }
    }
    for handle in before
        .players
        .keys()
        .filter(|h| !now.players.contains_key(h))
    {
        push("death", format!("joueur {handle} meurt"));
    }
    for (id, health) in &now.windows {
        let previous = before.windows.get(id).copied().unwrap_or(*health);
        if *health == 0 && previous > 0 {
            push("window", format!("fenêtre {id} cassée"));
        } else if *health > previous {
            push("window", format!("fenêtre {id} réparée ({health})"));
        }
    }
    for id in now.open_doors.intersection(&before.closed_doors) {
        push("door", format!("porte {id} ouverte"));
    }
    // T2.2, chantier B7 : une arme qui apparaît au sol = lâcher ; une qui disparaît = ramassage.
    for (id, weapon_id) in &now.weapon_pickups {
        if !before.weapon_pickups.contains_key(id) {
            push("drop", format!("arme {weapon_id} tombée au sol ({id})"));
        }
    }
    for (id, weapon_id) in &before.weapon_pickups {
        if !now.weapon_pickups.contains_key(id) {
            push("pickup", format!("arme {weapon_id} ramassée ({id})"));
        }
    }
    for (kind, label) in powerup_events(&before.powerup_pickups, &now.powerup_pickups, &picked) {
        push(kind, label);
    }
    // T1.9 : événements d'horloge déclenchés cette frame.
    for id in now.clock_fired.difference(&before.clock_fired) {
        push("clock", format!("horloge : {id}"));
    }
    // T1.5 : un personnage à variante qui apparaît. M3-T18 : `variant_death`, une variante qui
    // disparaît sans changement d'étage (champion tué : le changement d'étage retire aussi les
    // ennemis de l'étage quitté, ce n'est pas une mort).
    for (id, name) in &now.variants {
        if !before.variants.contains_key(id) {
            push("variant_spawn", format!("variante {name} ({id})"));
        }
    }
    if now.floor == before.floor {
        for (id, name) in &before.variants {
            if !now.variants.contains_key(id) {
                push("variant_death", format!("variante {name} tuée ({id})"));
            }
        }
    }
    if let Some((kind, label)) = outcome_event(before.outcome, now.outcome) {
        push(kind, label);
    }
    // T2.3, chantier C5 v1 : monnaie. `delta > 0` = points gagnés, `< 0` = achat réussi,
    // `== 0` = achat refusé (solde insuffisant, voir `run::currency::CurrencyEvent`).
    for event in currency_events.iter() {
        match event.delta.cmp(&0) {
            std::cmp::Ordering::Greater => push(
                "points",
                format!(
                    "joueur {} gagne {} points ({})",
                    event.handle, event.delta, event.reason
                ),
            ),
            std::cmp::Ordering::Less => push(
                "purchase",
                format!(
                    "joueur {} paie {} ({})",
                    event.handle, -event.delta, event.reason
                ),
            ),
            std::cmp::Ordering::Equal => push(
                "purchase_refused",
                format!("joueur {} : achat refusé ({})", event.handle, event.reason),
            ),
        }
    }
    for event in terrain_events.iter().flat_map(|events| events.iter()) {
        push(
            "terrain",
            format!("terrain creusé : {} case(s)", event.cells.len()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fin_de_partie_une_seule_fois_par_issue() {
        assert_eq!(outcome_event(None, None), None);
        let (kind, label) = outcome_event(None, Some(RunEnd::Defeat)).unwrap();
        // Libellé et catégorie d'avant D21 : l'attente `Defeat` les lit.
        assert_eq!(kind, "defeat");
        assert_eq!(label, "défaite : tous les joueurs sont à terre ou morts");
        assert_eq!(
            outcome_event(None, Some(RunEnd::Victory)).unwrap().0,
            "victory"
        );
        assert_eq!(
            outcome_event(None, Some(RunEnd::Abandon)).unwrap().0,
            "abandon"
        );
        // Déjà finie à la frame précédente : rien de nouveau.
        assert_eq!(
            outcome_event(Some(RunEnd::Victory), Some(RunEnd::Victory)),
            None
        );
    }

    #[test]
    fn power_ups_apparition_ramassage_expiration() {
        let before: BTreeMap<usize, String> =
            [(7, "nuke".to_string()), (8, "max_ammo".to_string())].into();
        let now: BTreeMap<usize, String> = [(9, "insta_kill".to_string())].into();
        // `nuke` ramassé par le joueur 1, `max_ammo` disparu sans ramassage : expiré.
        let events = powerup_events(&before, &now, &[("nuke".to_string(), Some(1))]);
        assert_eq!(
            events,
            vec![
                ("powerup", "power-up insta_kill au sol (9)".to_string()),
                (
                    "powerup_pickup",
                    "power-up nuke ramassé par joueur 1".to_string()
                ),
                ("powerup", "power-up max_ammo expiré (8)".to_string()),
            ]
        );
        assert!(powerup_events(&now, &now, &[]).is_empty());
    }
}
