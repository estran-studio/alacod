//! Générateur de scénarios (v0 T2.10, v1 T1.13, `docs/conventions.md` §28) : pour chaque arme
//! (à distance ou de corps à corps) du registre d'un jeu, construit en mémoire un [`Scenario`] à
//! partir du gabarit [`Template::WeaponOnTarget`] ; pour chaque personnage qui déclare `test:`,
//! un scénario par gabarit actif ([`Template::EnemyVsStillPlayer`],
//! [`Template::EnemyVsMovingPlayer`]). Le tout est rendu à l'appelant (CLI `alacod-gen` ou test)
//! pour écriture/lecture sur disque.
//!
//! **Gabarits ennemis** (T1.13) : ils jouent dans le jeu énuméré lui-même (le registre du
//! testbed n'a pas les personnages de `zombies`), sur une petite carte ([`enemy_arena_map`]) ;
//! l'ennemi est **placé par le scénario** (`Scenario::characters`, à [`ENEMY_OFFSET_X`] unités à
//! droite du spawn du joueur, retrouvé par une sonde : [`discover_enemy_probe`]) et désigné dans
//! les attentes par `EntityRef::Placed(0)`. Un jeu en mode `Waves` reçoit une période de grâce
//! plus longue que le scénario : aucune vague ne s'ajoute à l'ennemi testé.
//!
//! **Gabarits M2-T17** (`docs/conventions.md` §54) : un scénario par objet qui déclare `test:`
//! ([`Template::ItemHeld`]), par type de salle ([`Template::RoomClear`]) et par boss
//! ([`Template::BossFight`]) ; les attentes par défaut en sont dérivées du contenu
//! ([`item_default_expectations`], [`room_default_expectations`], [`boss_default_expectations`]).
//!
//! Gabarit d'arme :
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

use bevy_fixed::fixed_math::{self, Fixed};
use game::character::config::{CharacterConfig, CharacterTest};
use game::character::health::HitCount;
use game::character::player::Player;
use game::gen_test::{
    BossFightTest, ChampionTest, FamiliarTest, ItemFile, ItemTest, RoomFile, RoomTest, TearsTarget,
    TearsTest,
};
use game::replay::{
    Button, CharacterPlacement, EntityRef, Expectation, Invariants, PlayerScript, Scenario,
    Segment, WaveOverride,
};
use game::weapons::melee::{MeleeWeaponAsset, MeleeWeaponsConfig};
use game::weapons::{WeaponAsset, WeaponTest, WeaponsConfig};
use sim_core::modifier::resolve;
use sim_core::stats::Stats;
use utils::net_id::GgrsNetId;

/// Gabarit utilisé pour construire un scénario généré.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Le joueur apparaît dans l'arène du testbed avec une seule arme (`PlayerScript::weapon`),
    /// vise/marche vers `target` et l'attaque pendant `frames` images.
    WeaponOnTarget,
    /// T1.13 : le joueur, avec ses armes de départ, ne fait rien ; l'ennemi testé apparaît à
    /// [`ENEMY_OFFSET_X`] unités à sa droite à la frame [`ENEMY_AT_FRAME`].
    EnemyVsStillPlayer,
    /// T1.13 : même placement ; le joueur marche en carré ([`SQUARE_SIDE_FRAMES`] frames par
    /// côté, en boucle), sans tirer.
    EnemyVsMovingPlayer,
    /// M2-T17 : le joueur commence avec l'objet (`PlayerScript::items`), sans input (hors
    /// `UseActive` d'un actif), dans l'arène du jeu.
    ItemHeld,
    /// M2-T17 : un étage (ou une carte) qui contient une salle de ce type, un bot `explorateur`.
    RoomClear,
    /// M2-T17 : la salle du boss, un bot `fonceur`.
    BossFight,
    /// M3-T13 (§78) : le joueur tient l'objet de larmes et tire sur la cible de l'arène du jeu.
    TearsOnTarget,
    /// M3-T13 : le joueur tient l'objet familier (`FamiliarCount`, coups du familier).
    FamiliarHeld,
    /// M3-T13 : l'ennemi apparaît avec sa variante imposée ; effets de mort attendus.
    ChampionForced,
}

impl Template {
    fn file_suffix(self) -> &'static str {
        match self {
            Template::WeaponOnTarget => "",
            Template::EnemyVsStillPlayer => "still",
            Template::EnemyVsMovingPlayer => "moving",
            Template::ItemHeld
            | Template::RoomClear
            | Template::BossFight
            | Template::TearsOnTarget
            | Template::FamiliarHeld
            | Template::ChampionForced => "",
        }
    }
}

/// Sujet d'un scénario généré : une arme (gabarit [`Template::WeaponOnTarget`]) ou un
/// personnage (gabarits ennemis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedKind {
    Weapon(WeaponKind),
    Character(Template),
    /// M2-T17 : objet ([`Template::ItemHeld`]).
    Item,
    /// M2-T17 : type de salle ([`Template::RoomClear`]).
    Room,
    /// M2-T17 : boss ([`Template::BossFight`]).
    Boss,
    /// M3-T13 : objet de larmes ([`Template::TearsOnTarget`]).
    Tears,
    /// M3-T13 : objet familier ([`Template::FamiliarHeld`]).
    Familiar,
    /// M3-T13 : champion imposé ([`Template::ChampionForced`]).
    Champion,
    /// M3-T13 : témoin d'un objet de larmes ou familier : le même tir sans l'objet.
    Control,
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
    /// Id de l'arme ou du personnage.
    pub subject_id: String,
    pub kind: GeneratedKind,
    /// Nom de fichier seul (pas de dossier) : `weapon_<id>.ron`, `enemy_<id>_still.ron`,
    /// `enemy_<id>_moving.ron`, `item_<id>.ron`, `room_<id>.ron`, `boss_<id>.ron`.
    pub file_name: String,
    pub scenario: Scenario,
}

/// Jeu et carte où tournent tous les scénarios générés (voir la doc du module).
const TEMPLATE_GAME: &str = "testbed";
const TEMPLATE_MAP: &str = "testbed/arena.ldtk";
/// Même graine que `Scenario::default` (`game::replay::default_map_seed`, privée) : construit
/// ici directement (pas de RON, pas de `#[serde(default)]`), donc explicite.
const TEMPLATE_MAP_SEED: i32 = 123456;

/// T1.13 : frame d'apparition de l'ennemi d'un gabarit ennemi.
pub const ENEMY_AT_FRAME: u32 = 1;
/// T1.13 : distance (unités monde, vers +x) entre le spawn du joueur et l'ennemi placé.
pub const ENEMY_OFFSET_X: i32 = 200;
/// T1.13 : durée d'un côté du carré du gabarit joueur mobile.
pub const SQUARE_SIDE_FRAMES: u32 = 90;

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
/// Calage sur la course du joueur (`player_config.ron` : `max_speed` 150, `acceleration` et
/// `deceleration` 3000, voir `character::movement::run_velocity`) : 3 frames pour atteindre
/// 2,5 px par frame, 3 frames pour s'arrêter une fois la touche relâchée, chaque axe pour son
/// compte. Tenir une touche `n` frames déplace donc de 2,5 × `n` px (accélération et freinage
/// se compensent) : 51 frames pour 128, 19 pour 48. Les pauses ne sont plus nécessaires au
/// calage (un axe relâché freine même si l'autre est tenu) ; elles laissent le joueur
/// s'arrêter (3 frames, plus une marge) avant l'axe suivant puis avant l'attaque, pour un
/// chemin lisible sur la vidéo. Un changement de ces constantes de mouvement invaliderait ce
/// calage (vérifié par les coups comptés, `alacod-gen --play`, `club`/`sword`).
const WALK_X_FRAMES: u32 = 51;
const WALK_PAUSE_FRAMES: u32 = 5;
const WALK_Y_FRAMES: u32 = 19;

/// Erreurs de génération : contenu invalide, fichier d'arme illisible, ou `target`
/// introuvable (voir la doc du module).
#[derive(Debug)]
pub enum GenerateError {
    Manifest(content::ManifestError),
    Content(Vec<content::LintError>),
    WeaponFile {
        path: PathBuf,
        message: String,
    },
    MissingTarget {
        weapon_id: String,
    },
    /// T1.13 : la sonde d'un gabarit ennemi n'a pas trouvé le joueur 0.
    MissingPlayer {
        character_id: String,
    },
    /// M2-T17 : la sonde d'un gabarit d'objet n'a pas trouvé la stat modifiée par l'objet.
    MissingStat {
        item_id: String,
        stat: String,
    },
    /// M3-T13 : `test.champions` désigne une variante absente de la table du personnage.
    UnknownVariant {
        character_id: String,
        variant: String,
    },
    /// M3-T13 : `test.familiar` sur un objet sans `familiar:`.
    NotAFamiliar {
        item_id: String,
    },
    /// M3-T13 : la sonde d'un champion forcé n'a pas pu mesurer ce qu'il faut attendre.
    ChampionProbe {
        character_id: String,
        variant: String,
        message: String,
    },
    /// M3-T13 : une cible de `tears:` n'a pas été retrouvée dans la sonde.
    TearsTarget {
        item_id: String,
        index: usize,
    },
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
            GenerateError::MissingPlayer { character_id } => write!(
                f,
                "personnage « {character_id} » : sonde du gabarit ennemi sans joueur 0 (carte sans spawn de joueur ?)"
            ),
            GenerateError::UnknownVariant {
                character_id,
                variant,
            } => write!(
                f,
                "personnage « {character_id} » : test.champions : variante « {variant} » absente de `variants`"
            ),
            GenerateError::ChampionProbe {
                character_id,
                variant,
                message,
            } => write!(
                f,
                "personnage « {character_id} » variante « {variant} » : {message}"
            ),
            GenerateError::TearsTarget { item_id, index } => write!(
                f,
                "objet « {item_id} » : test.tears : la cible {index} n'a pas de compteur de coups dans la sonde (`counts_hits: true` ?)"
            ),
            GenerateError::NotAFamiliar { item_id } => write!(
                f,
                "objet « {item_id} » : test.familiar sur un objet sans `familiar:`"
            ),
            GenerateError::MissingStat { item_id, stat } => write!(
                f,
                "objet « {item_id} » : la stat {stat} n'existe pas chez le joueur de la sonde (modificateur sans effet : pas d'attente Stat possible)"
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
    let (registry, manifest, errors) =
        content::load_and_lint(game_dir).map_err(GenerateError::Manifest)?;
    if !errors.is_empty() {
        return Err(GenerateError::Content(errors));
    }

    let assets_dir = content::GameManifest::assets_dir(game_dir);
    // D26 : dossiers déclarés `generate: false` (copies d'un autre jeu) : aucune définition qui
    // en vient ne reçoit de scénario généré.
    let not_generated: Vec<PathBuf> = manifest
        .content_folders
        .iter()
        .filter(|folder| !folder.generate)
        .map(|folder| PathBuf::from(&folder.path))
        .collect();
    let generated = |file: &Path| !not_generated.iter().any(|folder| file.starts_with(folder));
    let mut weapons_cache: BTreeMap<PathBuf, WeaponsConfig> = BTreeMap::new();
    let mut melee_cache: BTreeMap<PathBuf, MeleeWeaponsConfig> = BTreeMap::new();

    // (id, kind, test) dans l'ordre déterministe voulu : toutes les armes à distance
    // (ordre du registre), puis toutes les armes de corps à corps (ordre du registre).
    let mut drafts: Vec<(String, WeaponKind, Option<WeaponTest>)> = Vec::new();

    for (id, entry) in &registry.weapons {
        if !generated(&entry.file) {
            continue;
        }
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
        if !generated(&entry.file) {
            continue;
        }
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

    // T1.13 : `generate_template` du manifeste (jeu et carte propres), sinon le testbed.
    let weapon_arena = match &manifest.generate_template {
        Some(template) => WeaponArena {
            game: manifest.name.clone(),
            map: template.map.clone(),
            mode: Some(content::EntryMode::Sandbox),
        },
        None => WeaponArena::testbed(),
    };

    let mut out = Vec::with_capacity(drafts.len());
    for (id, kind, test) in drafts {
        // Sonde (test: None, net_id 0 non utilisé) : seule l'arme équipée influence le
        // nombre d'entités créées avant `target`, jamais `test`/`frames` (voir la doc du
        // module et de `discover_target_net_id`).
        let probe = build_weapon_scenario(&weapon_arena, &id, kind, None, 0);
        let target_net_id =
            discover_target_net_id(&probe).ok_or_else(|| GenerateError::MissingTarget {
                weapon_id: id.clone(),
            })?;
        let scenario =
            build_weapon_scenario(&weapon_arena, &id, kind, test.as_ref(), target_net_id);
        out.push(GeneratedScenario {
            file_name: format!("weapon_{id}.ron"),
            subject_id: id,
            kind: GeneratedKind::Weapon(kind),
            scenario,
        });
    }

    // T1.13 : personnages qui déclarent `test:` (opt-in), dans l'ordre du registre ; pour
    // chacun, gabarit immobile puis mobile.
    let arena = EnemyArena {
        game: manifest.name.clone(),
        map: match &manifest.generate_template {
            Some(template) => template.map.clone(),
            None => enemy_arena_map(&manifest.name, &manifest.entry.start_map),
        },
        // Même résolution que `game::jjrs` : `Waves` déclaré, ou mode absent et un dossier
        // `Wave` présent ; un `generate_template` impose `Sandbox`.
        waves: manifest.generate_template.is_none()
            && match manifest.entry.mode {
                Some(mode) => mode == content::manifest::EntryMode::Waves,
                None => !registry.waves.is_empty(),
            },
        mode: manifest
            .generate_template
            .as_ref()
            .map(|_| content::EntryMode::Sandbox),
    };
    let mut champions_out: Vec<GeneratedScenario> = Vec::new();
    for (id, entry) in &registry.characters {
        if entry.test.is_none() || !generated(&entry.file) {
            continue;
        }
        let config: CharacterConfig = parse_ron_file(&assets_dir, &entry.file)?;
        let Some(test) = config.test else {
            continue;
        };
        let id = id.as_str().to_string();
        let probe_scenario = build_enemy_scenario(
            &arena,
            &id,
            Template::EnemyVsStillPlayer,
            &test,
            &EnemyProbe::default(),
        );
        let probe =
            discover_enemy_probe(&probe_scenario).ok_or_else(|| GenerateError::MissingPlayer {
                character_id: id.clone(),
            })?;
        for (template, active) in [
            (Template::EnemyVsStillPlayer, test.still),
            (Template::EnemyVsMovingPlayer, test.moving),
        ] {
            if !active {
                continue;
            }
            out.push(GeneratedScenario {
                file_name: format!("enemy_{id}_{}.ron", template.file_suffix()),
                subject_id: id.clone(),
                kind: GeneratedKind::Character(template),
                scenario: build_enemy_scenario(&arena, &id, template, &test, &probe),
            });
        }
        // M3-T13 : gabarit ChampionForced, une entrée de `test.champions` par variante (ajoutés
        // après tous les autres scénarios : l'ordre des sujets existants ne change pas).
        for champion in &test.champions {
            let Some(def) = config
                .variants
                .as_ref()
                .and_then(|variants| variants.table.get(&champion.variant))
            else {
                return Err(GenerateError::UnknownVariant {
                    character_id: id.clone(),
                    variant: champion.variant.clone(),
                });
            };
            let loot = def.loot.is_some();
            let (shot, death_expect) = if champion.kill {
                let death = probe_champion_death(&arena, &id, test.frames, champion, &probe, loot)
                    .map_err(|message| GenerateError::ChampionProbe {
                        character_id: id.clone(),
                        variant: champion.variant.clone(),
                        message,
                    })?;
                let expect = champion_death_expectations(&def.on_death, loot, &death, test.frames)
                    .map_err(|message| GenerateError::ChampionProbe {
                        character_id: id.clone(),
                        variant: champion.variant.clone(),
                        message,
                    })?;
                (
                    ChampionShot::UntilDeath {
                        death_frame: death.frame,
                        walk: loot,
                    },
                    expect,
                )
            } else {
                (ChampionShot::None, vec![])
            };
            champions_out.push(GeneratedScenario {
                file_name: champion_file_name(&id, &champion.variant),
                subject_id: id.clone(),
                kind: GeneratedKind::Champion,
                scenario: build_champion_scenario(
                    &arena,
                    &id,
                    test.frames,
                    champion,
                    &probe,
                    shot,
                    &death_expect,
                ),
            });
        }
        // M2-T17 : gabarit BossFight (`test.boss`, personnage qui a `boss:`).
        if let (Some(fight), Some(boss)) = (&test.boss, &config.boss) {
            let scene = SceneContext::new(&manifest, &arena, &registry);
            out.push(GeneratedScenario {
                file_name: format!("boss_{id}.ron"),
                subject_id: id.clone(),
                kind: GeneratedKind::Boss,
                scenario: build_boss_scenario(&scene, test.frames, fight, boss.phases.len()),
            });
        }
    }

    // M2-T17 : objets qui déclarent `test:` (gabarit ItemHeld), dans l'ordre du registre.
    for (id, entry) in &registry.items {
        if entry.test.is_none() || !generated(&entry.file) {
            continue;
        }
        let file: ItemFile = parse_ron_file_implicit_some(&assets_dir, &entry.file)?;
        let Some(test) = file.test else {
            continue;
        };
        let base = probe_player_stats(&build_item_scenario(&arena, id, &entry.def, &test, &[]))
            .ok_or_else(|| GenerateError::MissingPlayer {
                character_id: id.clone(),
            })?;
        let stats = expected_item_stats(id, &entry.def, &base)?;
        out.push(GeneratedScenario {
            file_name: format!("item_{id}.ron"),
            subject_id: id.clone(),
            kind: GeneratedKind::Item,
            scenario: build_item_scenario(&arena, id, &entry.def, &test, &stats),
        });
    }

    // M2-T17 : types de salle qui déclarent `test:` (gabarit RoomClear).
    for (id, entry) in &registry.rooms {
        if entry.test.is_none() || !generated(&entry.file) {
            continue;
        }
        let file: RoomFile = parse_ron_file_implicit_some(&assets_dir, &entry.file)?;
        let Some(test) = file.test else {
            continue;
        };
        let scene = SceneContext::new(&manifest, &arena, &registry);
        out.push(GeneratedScenario {
            file_name: format!("room_{id}.ron"),
            subject_id: id.clone(),
            kind: GeneratedKind::Room,
            scenario: build_room_scenario(&scene, id, &entry.def, &test),
        });
    }

    // M3-T13 : gabarits d'objet de larmes et d'objet familier (`test.tears`, `test.familiar`),
    // chacun avec son témoin sans objet. Ajoutés après tous les autres sujets.
    for (id, entry) in &registry.items {
        if entry.test.is_none() || !generated(&entry.file) {
            continue;
        }
        let file: ItemFile = parse_ron_file_implicit_some(&assets_dir, &entry.file)?;
        let Some(test) = file.test else {
            continue;
        };
        if let Some(tears) = &test.tears {
            let probe = build_tears_scenario(
                &weapon_arena,
                id,
                &entry.def,
                &test,
                tears,
                &TearsRun::probe(),
            );
            let anchor = discover_target(&probe).ok_or_else(|| GenerateError::MissingTarget {
                weapon_id: id.clone(),
            })?;
            let base = probe_player_stats(&probe).ok_or_else(|| GenerateError::MissingPlayer {
                character_id: id.clone(),
            })?;
            let stats = expected_item_stats(id, &entry.def, &base)?;
            let positions = tears_positions(anchor, &tears.targets);
            let placed = build_tears_scenario(
                &weapon_arena,
                id,
                &entry.def,
                &test,
                tears,
                &TearsRun {
                    stats: &[],
                    anchor: Some(anchor),
                    net_ids: &[],
                    control: false,
                },
            );
            let net_ids = discover_hit_net_ids(&placed, &positions).map_err(|index| {
                GenerateError::TearsTarget {
                    item_id: id.clone(),
                    index,
                }
            })?;
            for control in [false, true] {
                let run = TearsRun {
                    stats: &stats,
                    anchor: Some(anchor),
                    net_ids: &net_ids,
                    control,
                };
                out.push(GeneratedScenario {
                    file_name: if control {
                        tears_control_file_name(id)
                    } else {
                        tears_file_name(id)
                    },
                    subject_id: id.clone(),
                    kind: if control {
                        GeneratedKind::Control
                    } else {
                        GeneratedKind::Tears
                    },
                    scenario: build_tears_scenario(
                        &weapon_arena,
                        id,
                        &entry.def,
                        &test,
                        tears,
                        &run,
                    ),
                });
            }
        }
        if let Some(familiar) = &test.familiar {
            let per_copy = entry
                .def
                .familiar
                .as_ref()
                .map(|f| f.per_copy)
                .ok_or_else(|| GenerateError::NotAFamiliar {
                    item_id: id.clone(),
                })?;
            let probe =
                build_familiar_scenario(&weapon_arena, id, &test, familiar, per_copy, None, false);
            let target = discover_target(&probe).ok_or_else(|| GenerateError::MissingTarget {
                weapon_id: id.clone(),
            })?;
            let mut variants = vec![false];
            if familiar.min_hits > 0 {
                variants.push(true);
            }
            for control in variants {
                out.push(GeneratedScenario {
                    file_name: if control {
                        familiar_control_file_name(id)
                    } else {
                        familiar_file_name(id)
                    },
                    subject_id: id.clone(),
                    kind: if control {
                        GeneratedKind::Control
                    } else {
                        GeneratedKind::Familiar
                    },
                    scenario: build_familiar_scenario(
                        &weapon_arena,
                        id,
                        &test,
                        familiar,
                        per_copy,
                        Some(target.net_id),
                        control,
                    ),
                });
            }
        }
    }
    out.extend(champions_out);

    Ok(out)
}

/// Nom de fichier du gabarit [`Template::TearsOnTarget`] (stable : une fonction de l'id seul).
pub fn tears_file_name(item_id: &str) -> String {
    format!("tears_{item_id}.ron")
}

/// Nom du témoin de [`tears_file_name`] : le même tir sans l'objet.
pub fn tears_control_file_name(item_id: &str) -> String {
    format!("tears_{item_id}_sans_objet.ron")
}

/// Nom de fichier du gabarit [`Template::FamiliarHeld`].
pub fn familiar_file_name(item_id: &str) -> String {
    format!("familiar_{item_id}.ron")
}

/// Nom du témoin de [`familiar_file_name`].
pub fn familiar_control_file_name(item_id: &str) -> String {
    format!("familiar_{item_id}_sans_objet.ron")
}

/// Nom de fichier du gabarit [`Template::ChampionForced`].
pub fn champion_file_name(character_id: &str, variant: &str) -> String {
    format!("champion_{character_id}_{variant}.ron")
}

/// Cible de l'arène d'un gabarit à cible : net id et position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetInfo {
    pub net_id: usize,
    pub x: Fixed,
    pub y: Fixed,
}

/// Ce qui varie entre les scénarios d'un même `tears:` : stats attendues, ancre (cible de l'arène),
/// net ids des cibles (index 0 = arène, puis `targets`) et témoin (sans objet) ou non.
#[derive(Debug, Clone, Copy)]
pub struct TearsRun<'a> {
    pub stats: &'a [(sim_core::stats::StatId, f32)],
    pub anchor: Option<TargetInfo>,
    pub net_ids: &'a [usize],
    pub control: bool,
}

impl TearsRun<'_> {
    /// Sonde : aucune ancre, aucun net id, avec l'objet.
    pub fn probe() -> TearsRun<'static> {
        TearsRun {
            stats: &[],
            anchor: None,
            net_ids: &[],
            control: false,
        }
    }
}

/// Positions des cibles d'un `tears:` (index 0 : la cible de l'arène).
pub fn tears_positions(anchor: TargetInfo, targets: &[TearsTarget]) -> Vec<(Fixed, Fixed)> {
    std::iter::once((anchor.x, anchor.y))
        .chain(targets.iter().map(|t| {
            (
                anchor.x + Fixed::from_num(t.dx),
                anchor.y + Fixed::from_num(t.dy),
            )
        }))
        .collect()
}

/// Scénarios du gabarit [`Template::TearsOnTarget`] (pur), avec l'objet ou, pour `run.control`, le
/// même tir **sans** l'objet : `HasItem`, une `Stat` par modificateur de l'objet, les `hits`
/// (au moins `with_item` coups ; au plus `without_item` pour le témoin) puis `expect` (ou
/// `control`). Le joueur tire par impulsions ([`fire_pulses`]) avec `tears.weapon`.
pub fn build_tears_scenario(
    arena: &WeaponArena,
    id: &str,
    def: &items::ItemDef,
    test: &ItemTest,
    tears: &TearsTest,
    run: &TearsRun,
) -> Scenario {
    let frames = tears.frames.unwrap_or(test.frames);
    let net_id = |target: usize| run.net_ids.get(target).copied().unwrap_or(0);
    let mut expect = if run.control {
        vec![]
    } else {
        item_default_expectations(id, def, run.stats)
    };
    for hits in &tears.hits {
        expect.push(Expectation::EntityHits {
            net_id: net_id(hits.target),
            min: if run.control { 0 } else { hits.with_item },
            max: run.control.then_some(hits.without_item),
            at_frame: frames,
        });
    }
    expect.extend(
        if run.control {
            &tears.control
        } else {
            &tears.expect
        }
        .iter()
        .cloned(),
    );
    let characters = tears
        .targets
        .iter()
        .map(|target| {
            let anchor = run
                .anchor
                .map_or((Fixed::ZERO, Fixed::ZERO), |t| (t.x, t.y));
            CharacterPlacement {
                character: target.character.clone(),
                x: anchor.0 + Fixed::from_num(target.dx),
                y: anchor.1 + Fixed::from_num(target.dy),
                at_frame: ENEMY_AT_FRAME,
                variant: None,
                team: None,
            }
        })
        .collect();
    let mut scenario = base_scenario(
        &arena.game,
        &arena.map,
        TEMPLATE_MAP_SEED,
        frames,
        PlayerScript {
            inputs: fire_pulses_aimed(
                INPUT_START_FRAME,
                frames,
                tears.aim.unwrap_or(AIM_AT_TARGET),
            ),
            weapon: tears.weapon.clone(),
            items: if run.control {
                vec![]
            } else {
                vec![id.to_string()]
            },
            ..PlayerScript::default()
        },
        expect,
    );
    scenario.characters = characters;
    scenario.mode = arena.mode;
    scenario
}

/// Scénario du gabarit [`Template::FamiliarHeld`] (pur) : `FamiliarCount` = `per_copy` ; si
/// `familiar.min_hits > 0`, le joueur tire sur la cible de l'arène et elle reçoit au moins ce
/// nombre de coups. `control` : le témoin, même tir sans l'objet, au plus `min_hits - 1` coups
/// et aucun familier.
pub fn build_familiar_scenario(
    arena: &WeaponArena,
    id: &str,
    test: &ItemTest,
    familiar: &FamiliarTest,
    per_copy: u32,
    target_net_id: Option<usize>,
    control: bool,
) -> Scenario {
    let frames = familiar.frames.unwrap_or(test.frames);
    let mut expect = vec![];
    if !control {
        expect.push(Expectation::HasItem {
            handle: 0,
            id: id.to_string(),
            at_frame: 1,
        });
        expect.push(Expectation::FamiliarCount {
            handle: 0,
            count: per_copy,
            at_frame: FAMILIAR_CHECK_FRAME,
        });
    }
    let shooting = familiar.min_hits > 0;
    if shooting {
        expect.push(Expectation::EntityHits {
            net_id: target_net_id.unwrap_or(0),
            min: if control { 0 } else { familiar.min_hits },
            max: control.then(|| familiar.min_hits - 1),
            at_frame: frames,
        });
    }
    if control {
        expect.push(Expectation::FamiliarCount {
            handle: 0,
            count: 0,
            at_frame: FAMILIAR_CHECK_FRAME,
        });
    } else {
        // Le familier existe encore à la fin (aucune destruction à la fin de la partie).
        expect.push(Expectation::FamiliarCount {
            handle: 0,
            count: per_copy,
            at_frame: frames.saturating_sub(1),
        });
        expect.extend(familiar.expect.iter().cloned());
    }
    let mut scenario = base_scenario(
        &arena.game,
        &arena.map,
        TEMPLATE_MAP_SEED,
        frames,
        PlayerScript {
            inputs: if shooting {
                fire_pulses(INPUT_START_FRAME, frames)
            } else {
                vec![]
            },
            weapon: familiar.weapon.clone(),
            items: if control {
                vec![]
            } else {
                vec![id.to_string()]
            },
            ..PlayerScript::default()
        },
        expect,
    );
    scenario.mode = arena.mode;
    scenario
}

/// Frame où `FamiliarCount` est vérifié après l'apparition des familiers.
pub const FAMILIAR_CHECK_FRAME: u32 = 5;

/// Ce que la sonde mesure autour de la mort d'un champion tué par le joueur.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChampionDeath {
    /// Première frame sans porteur de `Variant` (le champion est mort à cette frame).
    pub frame: u32,
    /// Ennemis vivants à la frame qui précède la mort.
    pub enemies_before: u32,
    /// Balles de l'équipe ennemie à la frame qui précède la mort, et à la frame qui la suit.
    pub enemy_bullets_before: u32,
    pub enemy_bullets_after: u32,
    /// Objets que le joueur tient à la dernière frame et ne tenait pas au départ (butin ramassé).
    pub items: Vec<String>,
}

/// Marche du joueur vers le butin d'un champion tué (frames), puis appuis sur Interaction.
const LOOT_WALK_FRAMES: u32 = 80;
const LOOT_INTERACTION_PRESSES: u32 = 3;

/// Comment le joueur du gabarit [`Template::ChampionForced`] joue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChampionShot {
    /// Immobile, sans tirer.
    None,
    /// Tire jusqu'à la dernière frame (sonde de la mort).
    ToEnd,
    /// Tire jusqu'à la frame de mort puis, pour le butin, marche vers +x jusqu'à la fin.
    UntilDeath { death_frame: u32, walk: bool },
}

/// Effets de mort attendus d'une variante tuée par le joueur (pur, `death` vient de
/// [`probe_champion_death`]) : toujours la **mort** (ennemis vivants avant, puis `-1` et les
/// personnages nés après) ; `Explode` → moment clé `explosion` ; `SpawnCharacters` → `count`
/// ennemis de plus ; `SpawnPattern` → les balles de l'équipe ennemie mesurées ; `loot` → `HasItem`
/// de chaque objet ramassé. `Err` : aucun effet de mort traduisible, ou un effet qui n'a laissé
/// aucune trace mesurable (rien à attendre : on refuse plutôt que de l'ignorer).
pub fn champion_death_expectations(
    on_death: &[effects::Action],
    loot: bool,
    death: &ChampionDeath,
    frames: u32,
) -> Result<Vec<Expectation>, String> {
    use effects::Action;
    let spawned: u32 = on_death
        .iter()
        .map(|action| match action {
            Action::SpawnCharacters { count, .. } => *count,
            _ => 0,
        })
        .sum();
    let exploding = on_death
        .iter()
        .any(|action| matches!(action, Action::Explode { .. }));
    let pattern = on_death
        .iter()
        .any(|action| matches!(action, Action::SpawnPattern { .. }));
    if !exploding && spawned == 0 && !pattern && !loot {
        return Err(
            "kill: true sans effet de mort traduisible (Explode, SpawnCharacters, SpawnPattern, loot)"
                .to_string(),
        );
    }
    let enemies = |count: u32, at_frame: u32| Expectation::EntityCount {
        kind: game::replay::EntityKind::Enemy,
        min: Some(count),
        max: Some(count),
        at_frame,
    };
    let mut expect = vec![
        // Vivant avant, mort après : `death.frame` est la première frame sans porteur de variante.
        enemies(death.enemies_before, death.frame - 1),
        enemies(death.enemies_before - 1 + spawned, death.frame + 1),
    ];
    if exploding {
        expect.push(Expectation::Event {
            kind: "explosion".to_string(),
            label_contains: None,
            by_frame: frames,
        });
    }
    if pattern {
        if death.enemy_bullets_after <= death.enemy_bullets_before {
            return Err("SpawnPattern : aucune balle ennemie après la mort".to_string());
        }
        for (count, at_frame) in [
            (death.enemy_bullets_before, death.frame - 1),
            (death.enemy_bullets_after, death.frame + 1),
        ] {
            expect.push(Expectation::BulletCount {
                count,
                projectile: None,
                team: Some(sim_core::team::Team::Enemies),
                at_frame,
            });
        }
    }
    if loot {
        if death.items.is_empty() {
            return Err(
                "loot : le joueur n'a ramassé aucun objet (`frames` trop court, ou butin d'arme)"
                    .to_string(),
            );
        }
        expect.extend(death.items.iter().map(|item| Expectation::HasItem {
            handle: 0,
            id: item.clone(),
            at_frame: frames - 1,
        }));
    }
    Ok(expect)
}

/// Sonde de la mort d'un champion (simulation) : joue le scénario où le joueur tire jusqu'au bout,
/// relève la première frame sans `Variant`, les ennemis et balles ennemies autour de cette frame ;
/// pour un butin, rejoue avec la marche vers +x et relève les objets ramassés.
pub fn probe_champion_death(
    arena: &EnemyArena,
    character_id: &str,
    frames: u32,
    champion: &ChampionTest,
    probe: &EnemyProbe,
    loot: bool,
) -> Result<ChampionDeath, String> {
    #[derive(Default, Clone, Copy)]
    struct Snapshot {
        variants: u32,
        enemies: u32,
        enemy_bullets: u32,
    }
    fn snapshots(scenario: &Scenario, frames: u32) -> BTreeMap<u32, Snapshot> {
        let mut by_frame = BTreeMap::new();
        crate::runner::run_visiting(scenario, frames, |world, frame| {
            by_frame.insert(
                frame,
                Snapshot {
                    variants: world
                        .query::<&game::character::variant::Variant>()
                        .iter(world)
                        .count() as u32,
                    enemies: world
                        .query_filtered::<(), bevy::prelude::With<game::character::enemy::Enemy>>()
                        .iter(world)
                        .count() as u32,
                    enemy_bullets: world
                        .query::<&game::weapons::Bullet>()
                        .iter(world)
                        .filter(|bullet| bullet.source_team == sim_core::team::Team::Enemies)
                        .count() as u32,
                },
            );
        });
        by_frame
    }
    let fire = build_champion_scenario(
        arena,
        character_id,
        frames,
        champion,
        probe,
        ChampionShot::ToEnd,
        &[],
    );
    let by_frame = snapshots(&fire, frames);
    let frame = by_frame
        .iter()
        .skip_while(|(_, snap)| snap.variants == 0)
        .find(|(_, snap)| snap.variants == 0)
        .map(|(frame, _)| *frame)
        .ok_or_else(|| {
            format!("le champion n'est pas mort en {frames} frames (arme ? frames ?)")
        })?;
    let before = by_frame[&(frame - 1)];
    let after = by_frame
        .get(&(frame + 1))
        .ok_or("la mort est à la dernière frame")?;
    let mut items = Vec::new();
    if loot {
        let walk = build_champion_scenario(
            arena,
            character_id,
            frames,
            champion,
            probe,
            ChampionShot::UntilDeath {
                death_frame: frame,
                walk: true,
            },
            &[],
        );
        let start = walk.players[0].items.clone();
        let mut held = start.clone();
        crate::runner::run_visiting(&walk, frames - 1, |world, _| {
            if let Some(inventory) = crate::runner::probe_player_inventory(world, 0) {
                held = inventory.passives.clone();
                held.extend(inventory.active.map(|active| active.item));
            }
        });
        items = held
            .into_iter()
            .filter(|item| !start.contains(item))
            .collect();
    }
    Ok(ChampionDeath {
        frame,
        enemies_before: before.enemies,
        enemy_bullets_before: before.enemy_bullets,
        enemy_bullets_after: after.enemy_bullets,
        items,
    })
}

/// Scénario du gabarit [`Template::ChampionForced`] (pur) : le gabarit ennemi immobile, la variante
/// imposée, `EnemyVariant` à la frame qui suit l'apparition ; `shot` : le jeu du joueur (tir dans
/// l'axe, +x) ; `death_expect` : les attentes de [`champion_death_expectations`].
pub fn build_champion_scenario(
    arena: &EnemyArena,
    character_id: &str,
    frames: u32,
    champion: &ChampionTest,
    probe: &EnemyProbe,
    shot: ChampionShot,
    death_expect: &[Expectation],
) -> Scenario {
    let test = CharacterTest {
        frames,
        still: true,
        moving: false,
        expect_still: vec![Expectation::PlayerAlive {
            handle: 0,
            at_frame: frames,
        }],
        expect_moving: vec![],
        boss: None,
        champions: vec![],
    };
    let mut scenario = build_enemy_scenario(
        arena,
        character_id,
        Template::EnemyVsStillPlayer,
        &test,
        probe,
    );
    scenario.characters[0].variant = Some(champion.variant.clone());
    let mut expect = vec![Expectation::EnemyVariant {
        entity: EntityRef::Placed(0),
        variant: Some(champion.variant.clone()),
        at_frame: ENEMY_AT_FRAME + 1,
    }];
    let kill_aim = (127, 0);
    match shot {
        ChampionShot::None => {}
        ChampionShot::ToEnd => {
            scenario.players[0].weapon = champion.weapon.clone();
            scenario.players[0].inputs = fire_pulses_aimed(INPUT_START_FRAME, frames, kill_aim);
        }
        ChampionShot::UntilDeath { death_frame, walk } => {
            scenario.players[0].weapon = champion.weapon.clone();
            let mut inputs = fire_pulses_aimed(INPUT_START_FRAME, death_frame + 1, kill_aim);
            if walk {
                // Marche vers le butin (2,5 unités par frame : 80 frames couvrent les 200 unités
                // qui séparent le joueur du champion, moins la portée de ramassage), puis appuie
                // sur Interaction à trois reprises.
                let arrival = (death_frame + 1 + LOOT_WALK_FRAMES).min(frames);
                inputs.push(Segment {
                    from: death_frame + 1,
                    to: arrival,
                    buttons: vec![Button::Right],
                    pan: kill_aim,
                });
                for press in 0..LOOT_INTERACTION_PRESSES {
                    let from = arrival + press * 4;
                    if from < frames {
                        inputs.push(Segment {
                            from,
                            to: from + 1,
                            buttons: vec![Button::Interaction],
                            pan: kill_aim,
                        });
                    }
                }
            }
            scenario.players[0].inputs = inputs;
        }
    }
    expect.extend(death_expect.iter().cloned());
    expect.extend(champion.expect.iter().cloned());
    scenario.expect = expect;
    scenario
}

/// Jeu et carte d'un gabarit ennemi (T1.13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyArena {
    pub game: String,
    pub map: String,
    /// Mode `Waves` : période de grâce au-delà de la durée du scénario (aucune vague).
    pub waves: bool,
    /// Mode imposé, comme [`WeaponArena::mode`].
    pub mode: Option<content::EntryMode>,
}

/// Carte des gabarits ennemis d'un jeu : l'arène du testbed, la petite carte d'exemple de
/// `zombies`, sinon la carte de départ du manifeste.
pub fn enemy_arena_map(game: &str, start_map: &str) -> String {
    match game {
        "testbed" => TEMPLATE_MAP.to_string(),
        "zombies" => "exemples/test_map.ldtk".to_string(),
        _ => start_map.to_string(),
    }
}

/// Ce qu'une sonde apprend de la carte d'un gabarit ennemi : position du spawn du joueur.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnemyProbe {
    pub player_x: Fixed,
    pub player_y: Fixed,
}

/// Attentes par défaut d'un gabarit ennemi (T1.13, `docs/conventions.md` §28) : immobile →
/// le joueur a été touché au moins une fois (moment clé `hit`, cumulatif : la santé du joueur
/// se régénère, une santé à la dernière frame ne prouverait rien), mobile → le joueur est
/// vivant à la dernière frame ; dans les deux cas, l'ennemi ne chevauche jamais un mur.
pub fn enemy_default_expectations(template: Template, frames: u32) -> Vec<Expectation> {
    let in_wall = Expectation::EnemyNeverInWall {
        entity: EntityRef::Placed(0),
        from: ENEMY_AT_FRAME + 1,
        to: frames,
    };
    match template {
        Template::EnemyVsStillPlayer => vec![
            Expectation::Event {
                kind: "hit".to_string(),
                label_contains: Some("joueur 0 touché".to_string()),
                by_frame: frames,
            },
            in_wall,
        ],
        Template::EnemyVsMovingPlayer => vec![
            Expectation::PlayerAlive {
                handle: 0,
                at_frame: frames,
            },
            in_wall,
        ],
        Template::WeaponOnTarget
        | Template::ItemHeld
        | Template::RoomClear
        | Template::BossFight
        | Template::TearsOnTarget
        | Template::FamiliarHeld
        | Template::ChampionForced => vec![],
    }
}

/// Construit le scénario d'un gabarit ennemi (pur : la sonde est fournie par l'appelant).
/// Attentes : celles du `test:` pour ce gabarit, sinon [`enemy_default_expectations`].
pub fn build_enemy_scenario(
    arena: &EnemyArena,
    character_id: &str,
    template: Template,
    test: &CharacterTest,
    probe: &EnemyProbe,
) -> Scenario {
    let frames = test.frames;
    let explicit = match template {
        Template::EnemyVsMovingPlayer => &test.expect_moving,
        _ => &test.expect_still,
    };
    let expect = if explicit.is_empty() {
        enemy_default_expectations(template, frames)
    } else {
        explicit.clone()
    };
    let inputs = match template {
        Template::EnemyVsMovingPlayer => square_walk(INPUT_START_FRAME, frames),
        _ => vec![],
    };
    Scenario {
        game: arena.game.clone(),
        map: arena.map.clone(),
        map_seed: TEMPLATE_MAP_SEED,
        frames,
        players: vec![PlayerScript {
            inputs,
            bot: None,
            tags: vec![],
            immune_to: vec![],
            modifiers: vec![],
            weapon: None,
            currency: None,
            mutations: vec![],
            dash: None,
            items: vec![],
            bomb: None,
        }],
        expect,
        weapon_overrides: vec![],
        wave_overrides: arena.waves.then_some(WaveOverride {
            max_wave: None,
            min_wave_delay_frames: None,
            base_enemies: None,
            enemies_per_wave: None,
            grace_period_frames: Some(frames + 1),
            max_concurrent_enemies: None,
            spawn_batch_size: None,
            spawn_interval_frames: None,
        }),
        invariants: Invariants::default(),
        powerups: vec![],
        items: vec![],
        chests: vec![],
        room_reward_chance_override: None,
        shop: Default::default(),
        profile: false,
        profile_setup: None,
        hub: None,
        shopping: false,
        explorer: None,
        powerup_drop_chance_override: Some(fixed_math::FIXED_ZERO),
        floors: None,
        clocks: None,
        difficulty: None,
        progression: None,
        characters: vec![CharacterPlacement {
            character: character_id.to_string(),
            x: probe.player_x + Fixed::from_num(ENEMY_OFFSET_X),
            y: probe.player_y,
            at_frame: ENEMY_AT_FRAME,
            variant: None,
            team: None,
        }],
        mode: arena.mode,
        ui: Vec::new(),
    }
}

/// Contexte des gabarits M2-T17 (objet, salle, boss), tiré du manifeste et du registre du jeu
/// énuméré : ces gabarits jouent dans le jeu lui-même, jamais dans le testbed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneContext {
    pub game: String,
    /// `entry.start_map` : carte d'un gabarit sans `floors` ni `map` (la partie du manifeste).
    pub start_map: String,
    /// `entry.mode` vaut `Floors` : un gabarit à `map` impose alors `Sandbox` (sinon la séquence
    /// du manifeste remplacerait la carte).
    pub floors_mode: bool,
    /// D59 : quand le manifeste désigne sa séquence par défaut (`entry.floors`), la séquence du
    /// dossier `Floors` dont l'unique niveau est `start_map` : un gabarit sans `floors` ni `map`
    /// joue alors cette carte (l'étage squelette), pas la séquence par défaut du jeu.
    pub start_floors: Option<String>,
    /// Jeu en mode `Waves` : période de grâce plus longue que le scénario ([`EnemyArena::waves`]).
    pub waves: bool,
    /// Le jeu déclare un type de salle `boss` (le boss y est enfermé, §45) : la salle se rouvre à
    /// sa mort.
    pub has_boss_room: bool,
}

impl SceneContext {
    fn new(
        manifest: &content::GameManifest,
        arena: &EnemyArena,
        registry: &content::Registry,
    ) -> Self {
        Self {
            game: manifest.name.clone(),
            start_map: manifest.entry.start_map.clone(),
            floors_mode: manifest.entry.mode == Some(content::EntryMode::Floors),
            start_floors: manifest
                .entry
                .floors
                .as_ref()
                .and_then(|_| {
                    registry.floors.values().find(|entry| {
                        entry.levels.len() == 1 && entry.levels[0] == manifest.entry.start_map
                    })
                })
                .map(|entry| entry.id.to_string()),
            waves: arena.waves,
            has_boss_room: registry.rooms.contains_key("boss"),
        }
    }
}

/// Frame de l'appui sur `UseActive` du gabarit [`Template::ItemHeld`] (un seul appui d'une frame).
pub const ITEM_USE_FRAME: u32 = 10;
/// Frame où `ItemCharge` est vérifiée avant l'usage.
const ITEM_CHARGE_CHECK_FRAME: u32 = 5;
/// Frame où `ItemCharge` est vérifiée après l'usage : l'appui scripté à `ITEM_USE_FRAME` n'agit qu'à la
/// frame 16 (mesuré : charge encore pleine à f15, retombée à f16 ; délai d'entrée de la session).
const ITEM_AFTER_USE_FRAME: u32 = ITEM_USE_FRAME + 6;

/// Stats modifiées par un objet passif et leur valeur attendue à la frame 1 : `(stat, valeur)`,
/// une entrée par stat (dans l'ordre des modificateurs). `base` est lu chez le joueur de la
/// sonde **sans** l'objet : l'attendu est calculé par `sim_core::modifier::resolve`, pas relu
/// dans la partie qui tient l'objet.
pub fn expected_item_stats(
    id: &str,
    def: &items::ItemDef,
    base: &Stats,
) -> Result<Vec<(sim_core::stats::StatId, f32)>, GenerateError> {
    let mut out: Vec<(sim_core::stats::StatId, f32)> = Vec::new();
    if def.kind != items::ItemKind::Passive {
        return Ok(out);
    }
    for modifier in &def.modifiers {
        if out.iter().any(|(stat, _)| *stat == modifier.stat) {
            continue;
        }
        let base_value = base
            .get(&modifier.stat)
            .or_else(|| sim_core::stats::default_base(&modifier.stat))
            .ok_or_else(|| GenerateError::MissingStat {
                item_id: id.to_string(),
                stat: format!("{:?}", modifier.stat),
            })?;
        let modifiers: Vec<_> = def
            .modifiers
            .iter()
            .filter(|m| m.stat == modifier.stat)
            .map(|m| m.to_modifier(id))
            .collect();
        let value = resolve(base_value, modifiers.iter(), 0);
        out.push((modifier.stat.clone(), value.to_num::<f32>()));
    }
    Ok(out)
}

/// Attentes par défaut du gabarit [`Template::ItemHeld`] : l'objet est tenu à la frame 1, chaque
/// stat modifiée a sa valeur résolue ; un actif est chargé, puis sa charge retombe après l'usage
/// (sauf `Frames`, qui se recharge tout de suite, et un actif à `cost`, dont l'usage peut être
/// refusé).
pub fn item_default_expectations(
    id: &str,
    def: &items::ItemDef,
    stats: &[(sim_core::stats::StatId, f32)],
) -> Vec<Expectation> {
    let mut expect = vec![Expectation::HasItem {
        handle: 0,
        id: id.to_string(),
        at_frame: 1,
    }];
    for (stat, value) in stats {
        expect.push(Expectation::Stat {
            handle: 0,
            stat: stat.clone(),
            value: *value,
            at_frame: 1,
            entity: None,
        });
    }
    if let items::ItemKind::Active(charge) = def.kind {
        expect.push(Expectation::ItemCharge {
            handle: 0,
            charge: charge.required(),
            at_frame: ITEM_CHARGE_CHECK_FRAME,
        });
        let drops = !matches!(charge, items::ActiveCharge::Frames(_)) && def.cost.is_none();
        if drops {
            expect.push(Expectation::ItemCharge {
                handle: 0,
                charge: 0,
                at_frame: ITEM_AFTER_USE_FRAME,
            });
        }
    }
    expect
}

/// Scénario du gabarit [`Template::ItemHeld`] (pur) : `stats` vient de [`expected_item_stats`].
pub fn build_item_scenario(
    arena: &EnemyArena,
    id: &str,
    def: &items::ItemDef,
    test: &ItemTest,
    stats: &[(sim_core::stats::StatId, f32)],
) -> Scenario {
    let frames = test.frames;
    let active = matches!(def.kind, items::ItemKind::Active(_));
    let inputs = if active {
        assert!(
            frames > ITEM_AFTER_USE_FRAME,
            "gabarit ItemHeld « {id} » : test.frames ({frames}) trop petit pour l'usage de l'actif ({ITEM_AFTER_USE_FRAME} frames)"
        );
        vec![Segment {
            from: ITEM_USE_FRAME,
            to: ITEM_USE_FRAME + 1,
            buttons: vec![Button::UseActive],
            pan: (0, 0),
        }]
    } else {
        vec![]
    };
    let mut expect = item_default_expectations(id, def, stats);
    expect.extend(test.expect.iter().cloned());
    let mut scenario = base_scenario(
        &arena.game,
        &arena.map,
        TEMPLATE_MAP_SEED,
        frames,
        PlayerScript {
            inputs,
            items: vec![id.to_string()],
            ..PlayerScript::default()
        },
        expect,
    );
    scenario.wave_overrides = arena.waves.then(|| quiet_waves(frames));
    scenario.mode = arena.mode;
    scenario
}

/// Attentes par défaut du gabarit [`Template::RoomClear`] : une salle qui verrouille passe
/// verrouillée puis nettoyée avant la dernière frame (moments clés `room`, cumulatifs : un état
/// ponctuel dépendrait de la frame exacte du bot) ; dans tous les cas le joueur vit.
pub fn room_default_expectations(kind: &str, locks: bool, frames: u32) -> Vec<Expectation> {
    let mut expect = Vec::new();
    if locks {
        for what in ["verrouillée", "nettoyée"] {
            expect.push(Expectation::Event {
                kind: "room".to_string(),
                label_contains: Some(format!("salle {kind} {what}")),
                by_frame: frames,
            });
        }
    }
    expect.push(Expectation::PlayerAlive {
        handle: 0,
        at_frame: frames.saturating_sub(1),
    });
    expect
}

/// Scénario du gabarit [`Template::RoomClear`] (pur) : un bot `explorateur`, aucun input.
pub fn build_room_scenario(
    scene: &SceneContext,
    id: &str,
    def: &world::RoomKindDef,
    test: &RoomTest,
) -> Scenario {
    let locks = def.locks || def.arena.is_some();
    let mut expect = room_default_expectations(id, locks, test.frames);
    expect.extend(test.expect.iter().cloned());
    scene_scenario(
        scene,
        test.frames,
        test.floors.as_deref(),
        test.map.as_deref(),
        test.seed,
        bot_player(game::replay::BotProfile::Explorateur),
        expect,
    )
}

/// Attentes par défaut du gabarit [`Template::BossFight`] : chaque phase atteinte (moment clé
/// `boss_phase`, cumulatif : l'attente `BossPhase` échoue une fois le boss mort), puis la salle du
/// boss nettoyée quand le jeu a un type de salle `boss`.
pub fn boss_default_expectations(
    phases: usize,
    has_boss_room: bool,
    frames: u32,
) -> Vec<Expectation> {
    let mut expect: Vec<Expectation> = (1..phases)
        .map(|phase| Expectation::Event {
            kind: "boss_phase".to_string(),
            label_contains: Some(format!("phase {} → {phase}", phase - 1)),
            by_frame: frames,
        })
        .collect();
    if has_boss_room {
        expect.push(Expectation::Event {
            kind: "room".to_string(),
            label_contains: Some("salle boss nettoyée".to_string()),
            by_frame: frames,
        });
    }
    expect
}

/// Scénario du gabarit [`Template::BossFight`] (pur) : un bot `fonceur`, aucun input.
pub fn build_boss_scenario(
    scene: &SceneContext,
    frames: u32,
    fight: &BossFightTest,
    phases: usize,
) -> Scenario {
    let mut expect = boss_default_expectations(phases, scene.has_boss_room, frames);
    expect.extend(fight.expect.iter().cloned());
    scene_scenario(
        scene,
        frames,
        fight.floors.as_deref(),
        fight.map.as_deref(),
        fight.seed,
        bot_player(game::replay::BotProfile::Fonceur),
        expect,
    )
}

fn bot_player(profile: game::replay::BotProfile) -> PlayerScript {
    PlayerScript {
        bot: Some(profile),
        ..PlayerScript::default()
    }
}

/// Scène d'un gabarit de salle ou de boss : séquence `Floors` (`floors`), carte seule (`map`,
/// `Sandbox` imposé dans un jeu en `Floors`) ou la partie du manifeste.
fn scene_scenario(
    scene: &SceneContext,
    frames: u32,
    floors: Option<&str>,
    map: Option<&str>,
    seed: i32,
    player: PlayerScript,
    expect: Vec<Expectation>,
) -> Scenario {
    let map_path = map.unwrap_or(&scene.start_map);
    let mut scenario = base_scenario(&scene.game, map_path, seed, frames, player, expect);
    scenario.floors = floors.map(str::to_string);
    if floors.is_none() && map.is_none() {
        scenario.floors = scene.start_floors.clone();
    }
    if map.is_some() && scene.floors_mode {
        scenario.mode = Some(content::EntryMode::Sandbox);
    }
    scenario.wave_overrides = scene.waves.then(|| quiet_waves(frames));
    scenario
}

/// Aucune vague pendant le scénario (période de grâce plus longue que lui).
fn quiet_waves(frames: u32) -> WaveOverride {
    WaveOverride {
        max_wave: None,
        min_wave_delay_frames: None,
        base_enemies: None,
        enemies_per_wave: None,
        grace_period_frames: Some(frames + 1),
        max_concurrent_enemies: None,
        spawn_batch_size: None,
        spawn_interval_frames: None,
    }
}

/// Squelette commun des gabarits M2-T17 : un joueur, pas d'override, pas de drop.
fn base_scenario(
    game: &str,
    map: &str,
    seed: i32,
    frames: u32,
    player: PlayerScript,
    expect: Vec<Expectation>,
) -> Scenario {
    Scenario {
        game: game.to_string(),
        map: map.to_string(),
        map_seed: seed,
        frames,
        players: vec![player],
        expect,
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Invariants::default(),
        powerups: vec![],
        items: vec![],
        profile: false,
        profile_setup: None,
        hub: None,
        shopping: false,
        explorer: None,
        powerup_drop_chance_override: Some(fixed_math::FIXED_ZERO),
        floors: None,
        clocks: None,
        difficulty: None,
        progression: None,
        characters: vec![],
        mode: None,
        chests: vec![],
        room_reward_chance_override: None,
        ui: Vec::new(),
        shop: Default::default(),
    }
}

/// Sonde d'un gabarit d'objet : simule une frame **sans** l'objet et rend les `Stats` du joueur 0.
pub fn probe_player_stats(scenario: &Scenario) -> Option<Stats> {
    let mut probe = scenario.clone();
    for player in &mut probe.players {
        player.items.clear();
    }
    let mut app = crate::runner::run_until(&probe, 1);
    let world = app.world_mut();
    world
        .query::<(&Player, &Stats)>()
        .iter(world)
        .find(|(player, _)| player.handle == 0)
        .map(|(_, stats)| stats.clone())
}

/// Le joueur marche en carré (droite, bas, gauche, haut), [`SQUARE_SIDE_FRAMES`] frames par
/// côté, de `from` à `to`, sans tirer ni viser.
fn square_walk(from: u32, to: u32) -> Vec<Segment> {
    const SIDES: [Button; 4] = [Button::Right, Button::Down, Button::Left, Button::Up];
    let mut segments = Vec::new();
    let mut frame = from;
    let mut side = 0;
    while frame < to {
        let end = (frame + SQUARE_SIDE_FRAMES).min(to);
        segments.push(Segment {
            from: frame,
            to: end,
            buttons: vec![SIDES[side % SIDES.len()]],
            pan: (0, 0),
        });
        frame = end;
        side += 1;
    }
    segments
}

/// Sonde d'un gabarit ennemi : simule [`ENEMY_AT_FRAME`] frames (le joueur n'a pas encore
/// bougé) et lit la position du joueur 0.
pub fn discover_enemy_probe(scenario: &Scenario) -> Option<EnemyProbe> {
    let mut app = crate::runner::run_until(scenario, ENEMY_AT_FRAME);
    let world = app.world_mut();
    world
        .query::<(&Player, &fixed_math::FixedTransform3D)>()
        .iter(world)
        .find(|(player, _)| player.handle == 0)
        .map(|(_, transform)| EnemyProbe {
            player_x: transform.translation.x,
            player_y: transform.translation.y,
        })
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
    build_weapon_scenario(
        &WeaponArena::testbed(),
        weapon_id,
        kind,
        test,
        target_net_id,
    )
}

/// Jeu et carte du gabarit d'arme : le testbed (`arena.ldtk`), ou ceux d'un jeu qui déclare
/// `generate_template` (T1.13). Les inputs du gabarit (visée [`AIM_AT_TARGET`], marche de la
/// mêlée) sont réglés sur l'arène du testbed : la carte d'un `generate_template` place la cible
/// au même décalage du spawn du joueur (+128, −48 en coordonnées LDtk).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponArena {
    pub game: String,
    pub map: String,
    /// Mode imposé (`Scenario::mode`) : `Sandbox` pour un `generate_template` (la carte du
    /// gabarit est jouée telle quelle, même si `entry.mode` vaut `Floors`) ; `None` pour le
    /// testbed (fichiers générés inchangés).
    pub mode: Option<content::EntryMode>,
}

impl WeaponArena {
    pub fn testbed() -> Self {
        Self {
            game: TEMPLATE_GAME.to_string(),
            map: TEMPLATE_MAP.to_string(),
            mode: None,
        }
    }
}

/// [`build_scenario`] dans une arène donnée.
pub fn build_weapon_scenario(
    arena: &WeaponArena,
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
            // Deuxième pause (même durée) : le joueur est arrêté quand il commence à frapper
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
        game: arena.game.clone(),
        map: arena.map.clone(),
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
            mutations: vec![],
            dash: None,
            items: vec![],
            bomb: None,
        }],
        expect,
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Invariants::default(),
        // Isoler les essais d'armes des drops : ils ont leur propre scénario de preuve.
        powerups: vec![],
        items: vec![],
        chests: vec![],
        room_reward_chance_override: None,
        shop: Default::default(),
        profile: false,
        profile_setup: None,
        hub: None,
        shopping: false,
        explorer: None,
        powerup_drop_chance_override: Some(bevy_fixed::fixed_math::FIXED_ZERO),
        floors: None,
        progression: None,
        clocks: None,
        difficulty: None,
        characters: vec![],
        mode: arena.mode,
        ui: Vec::new(),
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
    fire_pulses_aimed(from, to, AIM_AT_TARGET)
}

/// [`fire_pulses`] avec une visée choisie.
fn fire_pulses_aimed(from: u32, to: u32, pan: (i16, i16)) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut frame = from;
    while frame + 1 < to {
        segments.push(Segment {
            from: frame,
            to: frame + 1,
            buttons: vec![Button::Fire],
            pan,
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

/// Cible de l'arène (net id et position) : comme [`discover_target_net_id`], avec le transform.
pub fn discover_target(scenario: &Scenario) -> Option<TargetInfo> {
    let mut app = crate::runner::run_until(scenario, 1);
    let world = app.world_mut();
    world
        .query::<(
            &GgrsNetId,
            &HitCount,
            &bevy_fixed::fixed_math::FixedTransform3D,
        )>()
        .iter(world)
        .next()
        .map(|(net_id, _, transform)| TargetInfo {
            net_id: net_id.0,
            x: transform.translation.x,
            y: transform.translation.y,
        })
}

/// Net ids des cibles (`HitCount`) d'un gabarit de larmes, retrouvés par leur position
/// (`positions[0]` : la cible de l'arène, puis les cibles posées) après leur apparition. `Err(i)` :
/// aucune entité à compteur de coups à moins d'une unité de `positions[i]`.
pub fn discover_hit_net_ids(
    scenario: &Scenario,
    positions: &[(Fixed, Fixed)],
) -> Result<Vec<usize>, usize> {
    let mut app = crate::runner::run_until(scenario, ENEMY_AT_FRAME + 2);
    let world = app.world_mut();
    let found: Vec<(usize, Fixed, Fixed)> = world
        .query::<(
            &GgrsNetId,
            &HitCount,
            &bevy_fixed::fixed_math::FixedTransform3D,
        )>()
        .iter(world)
        .map(|(net_id, _, transform)| (net_id.0, transform.translation.x, transform.translation.y))
        .collect();
    positions
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            found
                .iter()
                .filter(|(_, fx, fy)| {
                    (*fx - *x).abs() < Fixed::ONE && (*fy - *y).abs() < Fixed::ONE
                })
                .map(|(net_id, _, _)| *net_id)
                .min()
                .ok_or(index)
        })
        .collect()
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

/// [`parse_ron_file`] avec `IMPLICIT_SOME`, comme le registre (`content`) lit les objets et les
/// types de salle : `test: (frames: 60)` sans `Some`.
fn parse_ron_file_implicit_some<T: serde::de::DeserializeOwned>(
    assets_dir: &Path,
    file: &Path,
) -> Result<T, GenerateError> {
    let abs = assets_dir.join(file);
    let text = fs::read_to_string(&abs).map_err(|e| GenerateError::WeaponFile {
        path: file.to_path_buf(),
        message: format!("lecture impossible ({}) : {e}", abs.display()),
    })?;
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&text)
        .map_err(|e| GenerateError::WeaponFile {
            path: file.to_path_buf(),
            message: format!("erreur RON : {e}"),
        })
}

/// En-tête écrit en RON au-dessus de chaque scénario généré (voir `bin/alacod-gen.rs`).
pub fn header_comment(subject_id: &str, kind: GeneratedKind) -> String {
    match kind {
        GeneratedKind::Weapon(kind) => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit WeaponOnTarget (T2.10) : le joueur apparaît dans l'arène du testbed avec\n\
             // uniquement « {subject_id} » ({}) et {} `target`.\n",
            kind.label(),
            match kind {
                WeaponKind::Ranged => "vise et tire sur",
                WeaponKind::Melee => "marche jusqu'à et frappe",
            }
        ),
        GeneratedKind::Item => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit ItemHeld (M2-T17, §54) : le joueur commence avec « {subject_id} »\n\
             // (`PlayerScript::items`) ; un actif est utilisé à la frame {ITEM_USE_FRAME}.\n"
        ),
        GeneratedKind::Tears => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit TearsOnTarget (M3-T13, §78) : le joueur tient « {subject_id} » et tire sur la\n\
             // cible de l'arène ; les coups reçus sont comptés.\n"
        ),
        GeneratedKind::Familiar => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit FamiliarHeld (M3-T13, §78) : le joueur tient « {subject_id} » ; ses familiers\n\
             // apparaissent (`FamiliarCount`) et, s'il y a lieu, tirent avec lui.\n"
        ),
        GeneratedKind::Control => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Témoin de « {subject_id} » (M3-T13, §78) : le même tir, sans l'objet ; il prouve que\n\
             // les attentes de la partie avec l'objet ne passent pas sans lui.\n"
        ),
        GeneratedKind::Champion => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit ChampionForced (M3-T13, §78) : « {subject_id} » apparaît à {ENEMY_OFFSET_X} unités\n\
             // du joueur avec sa variante imposée ; effets de mort attendus si le joueur le tire.\n"
        ),
        GeneratedKind::Room => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit RoomClear (M2-T17, §54) : un bot `explorateur` joue un étage qui contient\n\
             // une salle « {subject_id} » ; aucun input scripté.\n"
        ),
        GeneratedKind::Boss => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit BossFight (M2-T17, §54) : un bot `fonceur` affronte « {subject_id} » ;\n\
             // aucun input scripté.\n"
        ),
        GeneratedKind::Character(template) => format!(
            "// Généré par `alacod-gen`, ne pas éditer à la main (crates/scenario/src/generate.rs).\n\
             // Gabarit {template:?} (T1.13) : « {subject_id} » est placé à {ENEMY_OFFSET_X} unités à\n\
             // droite du joueur (frame {ENEMY_AT_FRAME}) ; le joueur {}.\n",
            match template {
                Template::EnemyVsMovingPlayer => "marche en carré sans tirer",
                _ => "ne fait rien",
            }
        ),
    }
}
