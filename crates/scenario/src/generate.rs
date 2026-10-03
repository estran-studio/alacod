//! Générateur de scénarios v0 (T2.10, `docs/taches.md`) : pour chaque arme (à distance ou de
//! corps à corps) du registre d'un jeu, construit en mémoire un [`Scenario`] à partir d'un
//! gabarit ([`Template::WeaponOnTarget`], le seul aujourd'hui) et le rend disponible à l'appelant
//! (CLI `alacod-gen` ou test) pour écriture/lecture sur disque.
//!
//! Les scénarios générés tournent tous dans le **testbed** (`games/testbed/assets/testbed/arena.ldtk`),
//! quel que soit le jeu dont on énumère le contenu (`generate_game(games/zombies)` lit le
//! registre de `zombies` mais joue dans `testbed`, exactement comme
//! `tests/scenarios/testbed_target_hits.ron`) : c'est le seul endroit où `target` (la cible
//! qui compte les coups, `HitCount`) existe. `games/testbed` partage aujourd'hui le même
//! `weapons.ron`/`melee_weapons.ron` que `games/zombies` (fichiers identiques, voir le
//! rapport de la tâche) : un id d'arme de `games/zombies` se résout donc aussi dans le
//! registre de `testbed`.
//!
//! Le `net_id` de `target` n'est **pas** une constante : il dépend du nombre d'entités déjà
//! créées avant lui (joueur, ses armes...), qui varie avec l'arme choisie (une seule arme,
//! au lieu des trois par défaut). [`discover_target_net_id`] le retrouve en simulant
//! brièvement le scénario et en cherchant l'entité qui porte `HitCount` — robuste à tout
//! changement futur de la numérotation, plutôt qu'un calcul arithmétique fragile.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use game::character::health::HitCount;
use game::replay::{Button, Expectation, Invariants, PlayerScript, Scenario, Segment};
use game::weapons::melee::{MeleeWeaponAsset, MeleeWeaponsConfig};
use game::weapons::{WeaponAsset, WeaponTest, WeaponsConfig};
use utils::net_id::GgrsNetId;

/// Gabarit utilisé pour construire un scénario généré. Un seul aujourd'hui (T2.10 v0) ;
/// d'autres (perks, ennemis...) s'ajouteront à cet enum sans toucher au reste du générateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Le joueur apparaît dans l'arène du testbed avec une seule arme (`PlayerScript::weapon`),
    /// vise/marche vers `target` et l'attaque pendant `frames` images.
    WeaponOnTarget,
}

/// Catégorie d'une arme générée (détermine le registre d'où elle vient et la façon de
/// l'utiliser dans le gabarit : viser et tirer, ou marcher et frapper).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Ranged,
    Melee,
}

impl WeaponKind {
    fn label(self) -> &'static str {
        match self {
            WeaponKind::Ranged => "arme à distance",
            WeaponKind::Melee => "arme de corps à corps",
        }
    }
}

/// Un scénario généré, prêt à être écrit (`tests/scenarios/generated/<jeu>/<file_name>`) ou
/// joué directement (`scenario::run`).
#[derive(Debug, Clone)]
pub struct GeneratedScenario {
    pub weapon_id: String,
    pub kind: WeaponKind,
    /// Nom de fichier seul (pas de dossier) : `weapon_<id>.ron`.
    pub file_name: String,
    pub scenario: Scenario,
}

/// Jeu et carte où tournent tous les scénarios générés (voir la doc du module).
const TEMPLATE_GAME: &str = "testbed";
const TEMPLATE_MAP: &str = "testbed/arena.ldtk";
/// Même graine que `Scenario::default` (`game::replay::default_map_seed`, privée) : construit
/// ici directement (pas de RON, pas de `#[serde(default)]`), donc explicite.
const TEMPLATE_MAP_SEED: i32 = 123456;

/// Frame de départ des inputs (délai de chargement de la map, comme les scénarios
/// manuscrits, ex. `testbed_target_hits.ron` : `from: 10`).
const INPUT_START_FRAME: u32 = 10;

/// Nombre de frames par défaut d'un scénario généré sans `test:` (remplacé par
/// `WeaponTest::frames` quand présent). Plus long pour la mêlée : le joueur doit d'abord
/// marcher jusqu'à `target` (voir [`WALK_Y_FRAMES`]) avant de pouvoir frapper.
pub const RANGED_DEFAULT_FRAMES: u32 = 200;
pub const MELEE_DEFAULT_FRAMES: u32 = 400;

/// Visée (`Segment::pan`, vecteur joueur -> pointeur en unités monde) du spawn du joueur 0
/// vers `target` dans `testbed/arena.ldtk` : delta LDtk (+128, -48) sur un seul niveau (pas
/// d'assemblage procédural entre salles), axe Y inversé en monde (voir `testbed_target_hits.ron`,
/// qui vise déjà `target` avec `pan: (128, 48)` depuis le même spawn). Sert aussi de direction
/// de marche pour la mêlée (§ ci-dessous).
const AIM_AT_TARGET: (i16, i16) = (128, 48);

/// Script de marche du gabarit de mêlée : le joueur avance à l'horizontale (`Right`, vers
/// +128 unités), s'arrête (aucun bouton, [`WALK_PAUSE_FRAMES`]), avance à la verticale (`Up`,
/// vers +48 unités), s'arrête à nouveau, puis attaque — un chemin en L qui l'amène à moins de
/// 10 unités de `target`, largement dans la portée de n'importe quelle arme de corps à corps
/// (`club`, la plus courte des deux avec `test:`, a une portée de 40).
///
/// Les deux arrêts sont nécessaires, pas seulement esthétiques : `apply_friction`
/// (`crates/game/src/character/player/input.rs`) ne freine que quand **aucun** bouton de
/// déplacement n'est tenu, pas seulement celui de l'axe qu'on relâche. Sans le premier arrêt,
/// la vitesse horizontale accumulée pendant `Right` continue de pousser le joueur pendant tout
/// `Up` (les deux boutons comptent comme « en mouvement ») ; sans le second, la vitesse
/// verticale continue de le pousser pendant toute la tenue de l'attaque, l'éloignant de
/// `target` au lieu de l'y laisser. `WALK_PAUSE_FRAMES` (~20 frames) laisse la friction
/// (`friction: "10.0"`) dissiper la vitesse avant de changer d'axe ou d'attaquer.
///
/// Durées calées empiriquement sur `movement.acceleration`/`max_speed` du joueur
/// (`150.0`/`150.0`, testbed/zombies) : position finale mesurée à moins de 10 unités de
/// `target` (voir le rapport de la tâche T2.10), confirmée par les coups comptés
/// (`alacod-gen --play`, `club`/`sword`). Un changement de ces constantes de mouvement
/// (`player_config.ron`) invaliderait ce calage.
const WALK_X_FRAMES: u32 = 72;
const WALK_PAUSE_FRAMES: u32 = 20;
const WALK_Y_FRAMES: u32 = 45;

/// Erreurs de génération : contenu invalide, fichier d'arme illisible, ou `target`
/// introuvable (voir la doc du module).
#[derive(Debug)]
pub enum GenerateError {
    Manifest(content::ManifestError),
    Content(Vec<content::LintError>),
    WeaponFile { path: PathBuf, message: String },
    MissingTarget { weapon_id: String },
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateError::Manifest(e) => write!(f, "{e}"),
            GenerateError::Content(errors) => {
                writeln!(f, "{} erreur(s) de contenu :", errors.len())?;
                for e in errors {
                    writeln!(f, "  {e}")?;
                }
                Ok(())
            }
            GenerateError::WeaponFile { path, message } => {
                write!(f, "{} : {message}", path.display())
            }
            GenerateError::MissingTarget { weapon_id } => write!(
                f,
                "arme « {weapon_id} » : impossible de trouver l'entité « target » (HitCount) en jouant {TEMPLATE_MAP} ; `games/testbed/assets/characters/target.ron` existe-t-il et déclare-t-il `counts_hits: true` ?"
            ),
        }
    }
}

impl std::error::Error for GenerateError {}

/// Construit en mémoire un scénario généré pour chaque arme à distance et chaque arme de
/// corps à corps du registre de `game_dir` (ex. `games/zombies`), triées par id (`BTreeMap`
/// du registre) : ranged puis melee, chacune dans l'ordre du registre — déterministe, même
/// résultat à chaque appel. N'écrit rien sur disque (voir `bin/alacod-gen.rs`).
pub fn generate_game(game_dir: &Path) -> Result<Vec<GeneratedScenario>, GenerateError> {
    let (registry, _manifest, errors) =
        content::load_and_lint(game_dir).map_err(GenerateError::Manifest)?;
    if !errors.is_empty() {
        return Err(GenerateError::Content(errors));
    }

    let assets_dir = content::GameManifest::assets_dir(game_dir);
    let mut weapons_cache: BTreeMap<PathBuf, WeaponsConfig> = BTreeMap::new();
    let mut melee_cache: BTreeMap<PathBuf, MeleeWeaponsConfig> = BTreeMap::new();

    // (id, kind, test) dans l'ordre déterministe voulu : toutes les armes à distance
    // (ordre du registre), puis toutes les armes de corps à corps (ordre du registre).
    let mut drafts: Vec<(String, WeaponKind, Option<WeaponTest>)> = Vec::new();

    for (id, entry) in &registry.weapons {
        let asset = read_weapon_asset(&assets_dir, &entry.file, id.as_str(), &mut weapons_cache)?
            .ok_or_else(|| GenerateError::WeaponFile {
            path: entry.file.clone(),
            message: format!("arme « {id} » absente du fichier relu"),
        })?;
        drafts.push((
            id.as_str().to_string(),
            WeaponKind::Ranged,
            asset.config.test,
        ));
    }
    for (id, entry) in &registry.melee_weapons {
        let asset = read_melee_asset(&assets_dir, &entry.file, id.as_str(), &mut melee_cache)?
            .ok_or_else(|| GenerateError::WeaponFile {
                path: entry.file.clone(),
                message: format!("arme de corps à corps « {id} » absente du fichier relu"),
            })?;
        drafts.push((
            id.as_str().to_string(),
            WeaponKind::Melee,
            asset.config.test,
        ));
    }

    let mut out = Vec::with_capacity(drafts.len());
    for (id, kind, test) in drafts {
        // Sonde (test: None, net_id 0 non utilisé) : seule l'arme équipée influence le
        // nombre d'entités créées avant `target`, jamais `test`/`frames` (voir la doc du
        // module et de `discover_target_net_id`).
        let probe = build_scenario(&id, kind, None, 0);
        let target_net_id =
            discover_target_net_id(&probe).ok_or_else(|| GenerateError::MissingTarget {
                weapon_id: id.clone(),
            })?;
        let scenario = build_scenario(&id, kind, test.as_ref(), target_net_id);
        out.push(GeneratedScenario {
            file_name: format!("weapon_{id}.ron"),
            weapon_id: id,
            kind,
            scenario,
        });
    }

    Ok(out)
}

/// Construit le [`Scenario`] du gabarit [`Template::WeaponOnTarget`] pour une arme donnée.
/// Pure (aucune I/O, aucune simulation) : `target_net_id` est fourni par l'appelant
/// (`generate_game` l'obtient via [`discover_target_net_id`] ; un test peut en fournir un
/// fabriqué). `test: None` -> scénario "invariants seulement" (pas d'attente `EntityHits`),
/// mais qui doit quand même atteindre sa dernière frame (vérifié par le runner).
pub fn build_scenario(
    weapon_id: &str,
    kind: WeaponKind,
    test: Option<&WeaponTest>,
    target_net_id: usize,
) -> Scenario {
    let default_frames = match kind {
        WeaponKind::Ranged => RANGED_DEFAULT_FRAMES,
        WeaponKind::Melee => MELEE_DEFAULT_FRAMES,
    };
    let frames = test.map(|t| t.frames).unwrap_or(default_frames);

    let inputs = match kind {
        WeaponKind::Ranged => fire_pulses(INPUT_START_FRAME, frames),
        WeaponKind::Melee => {
            let x_end = INPUT_START_FRAME + WALK_X_FRAMES;
            let y_start = x_end + WALK_PAUSE_FRAMES;
            let y_end = y_start + WALK_Y_FRAMES;
            // Deuxième pause (même durée) : sans elle, la vitesse verticale accumulée
            // pendant `Up` continuerait à déplacer le joueur (par inertie, la friction ne
            // s'applique que quand aucun bouton de déplacement n'est tenu) tout au long de
            // la tenue de `Melee`, l'éloignant de `target` au lieu de l'y laisser (voir le
            // rapport de la tâche T2.10 pour la trace qui a révélé ce dépassement).
            let attack_start = y_end + WALK_PAUSE_FRAMES;
            assert!(
                frames > attack_start,
                "gabarit WeaponOnTarget (mêlée) « {weapon_id} » : test.frames ({frames}) trop petit pour le script de marche ({attack_start} frames)"
            );
            vec![
                Segment {
                    from: INPUT_START_FRAME,
                    to: x_end,
                    buttons: vec![Button::Right],
                    pan: AIM_AT_TARGET,
                },
                Segment {
                    from: y_start,
                    to: y_end,
                    buttons: vec![Button::Up],
                    pan: AIM_AT_TARGET,
                },
                Segment {
                    from: attack_start,
                    to: frames,
                    buttons: vec![Button::Melee],
                    pan: AIM_AT_TARGET,
                },
            ]
        }
    };

    let mut expect = Vec::new();
    if let Some(t) = test {
        expect.push(Expectation::EntityHits {
            net_id: target_net_id,
            min: t.min_hits,
            max: t.max_hits,
            at_frame: frames,
        });
        expect.extend(t.expect.iter().cloned());
    }

    Scenario {
        game: TEMPLATE_GAME.to_string(),
        map: TEMPLATE_MAP.to_string(),
        map_seed: TEMPLATE_MAP_SEED,
        frames,
        players: vec![PlayerScript {
            inputs,
            bot: None,
            tags: vec![],
            immune_to: vec![],
            modifiers: vec![],
            weapon: Some(weapon_id.to_string()),
            currency: None,
        }],
        expect,
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Invariants::default(),
        // Isoler les essais d'armes des drops : ils ont leur propre scénario de preuve.
        powerups: vec![],
        powerup_drop_chance_override: Some(bevy_fixed::fixed_math::FIXED_ZERO),
        floors: None,
    }
}

/// Segments `Fire` en alternance un frame appuyé / un frame relâché, de `from` à `to`
/// (visée [`AIM_AT_TARGET`] constante). Une pression continue ne ferait tirer qu'une seule
/// fois les modes `Manual`/`Shotgun` (`game::weapons::weapon_rollback_system` : `can_fire`
/// exige `!weapon_state.is_firing`, qui ne redevient `false` qu'à un frame où `input.fire`
/// est relâché) — seul `Automatic` tolère une pression continue. Relâcher un frame sur deux
/// (bien plus rapide que la cadence de tir de n'importe quelle arme du jeu) redéclenche
/// `Manual`/`Shotgun`/`Burst` à leur propre cadence sans jamais la limiter, et ne change rien
/// pour `Automatic` (qui ne regarde pas `is_firing`).
fn fire_pulses(from: u32, to: u32) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut frame = from;
    while frame + 1 < to {
        segments.push(Segment {
            from: frame,
            to: frame + 1,
            buttons: vec![Button::Fire],
            pan: AIM_AT_TARGET,
        });
        frame += 2;
    }
    segments
}

/// Retrouve le `net_id` de `target` en simulant brièvement `scenario` (une frame suffit :
/// `target` est créé au chargement de la map, avant la première frame simulée — voir
/// `crates/scenario/tests/scenarios.rs::net_ids`, qui inspecte déjà `GgrsNetId` à la frame 1
/// par défaut) et en cherchant l'entité qui porte `HitCount` (posé uniquement par
/// `CharacterConfig::counts_hits`, seulement vrai pour `target` dans `games/testbed`
/// aujourd'hui). `None` si aucune entité n'en porte un (contenu testbed cassé/absent).
pub fn discover_target_net_id(scenario: &Scenario) -> Option<usize> {
    let mut app = crate::runner::run_until(scenario, 1);
    let world = app.world_mut();
    world
        .query::<(&GgrsNetId, &HitCount)>()
        .iter(world)
        .next()
        .map(|(net_id, _)| net_id.0)
}

/// Lit et parse `assets_dir/file` comme `WeaponsConfig` (une seule fois par fichier, mis en
/// cache : plusieurs armes de `games/zombies` partagent `weapons.ron`), puis retourne l'entrée
/// `id`.
fn read_weapon_asset(
    assets_dir: &Path,
    file: &Path,
    id: &str,
    cache: &mut BTreeMap<PathBuf, WeaponsConfig>,
) -> Result<Option<WeaponAsset>, GenerateError> {
    if !cache.contains_key(file) {
        let parsed = parse_ron_file::<WeaponsConfig>(assets_dir, file)?;
        cache.insert(file.to_path_buf(), parsed);
    }
    Ok(cache.get(file).and_then(|config| config.0.get(id)).cloned())
}

/// Voir [`read_weapon_asset`] (même logique, pour `melee_weapons.ron`).
fn read_melee_asset(
    assets_dir: &Path,
    file: &Path,
    id: &str,
    cache: &mut BTreeMap<PathBuf, MeleeWeaponsConfig>,
) -> Result<Option<MeleeWeaponAsset>, GenerateError> {
    if !cache.contains_key(file) {
        let parsed = parse_ron_file::<MeleeWeaponsConfig>(assets_dir, file)?;
        cache.insert(file.to_path_buf(), parsed);
    }
    Ok(cache.get(file).and_then(|config| config.0.get(id)).cloned())
}

fn parse_ron_file<T: serde::de::DeserializeOwned>(
    assets_dir: &Path,
    file: &Path,
) -> Result<T, GenerateError> {
    let abs = assets_dir.join(file);
    let text = fs::read_to_string(&abs).map_err(|e| GenerateError::WeaponFile {
        path: file.to_path_buf(),
        message: format!("lecture impossible ({}) : {e}", abs.display()),
    })?;
    ron::from_str(&text).map_err(|e| GenerateError::WeaponFile {
        path: file.to_path_buf(),
        message: format!("erreur RON : {e}"),
    })
}

/// En-tête écrit en RON au-dessus de chaque scénario généré (voir `bin/alacod-gen.rs`).
pub fn header_comment(weapon_id: &str, kind: WeaponKind) -> String {
    format!(
        "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
         // Gabarit WeaponOnTarget (T2.10) : le joueur apparaît dans l'arène du testbed avec\n\
         // uniquement « {weapon_id} » ({}) et {} `target`.\n",
        kind.label(),
        match kind {
            WeaponKind::Ranged => "vise et tire sur",
            WeaponKind::Melee => "marche jusqu'à et frappe",
        }
    )
}
