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
use game::replay::{
    Button, CharacterPlacement, EntityRef, Expectation, Invariants, PlayerScript, Scenario,
    Segment, WaveOverride,
};
use game::weapons::melee::{MeleeWeaponAsset, MeleeWeaponsConfig};
use game::weapons::{WeaponAsset, WeaponTest, WeaponsConfig};
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
}

impl Template {
    fn file_suffix(self) -> &'static str {
        match self {
            Template::WeaponOnTarget => "",
            Template::EnemyVsStillPlayer => "still",
            Template::EnemyVsMovingPlayer => "moving",
        }
    }
}

/// Sujet d'un scénario généré : une arme (gabarit [`Template::WeaponOnTarget`]) ou un
/// personnage (gabarits ennemis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedKind {
    Weapon(WeaponKind),
    Character(Template),
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
    /// `enemy_<id>_moving.ron`.
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
    for (id, entry) in &registry.characters {
        if entry.test.is_none() {
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
    }

    Ok(out)
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
        Template::WeaponOnTarget => vec![],
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
        powerup_drop_chance_override: Some(fixed_math::FIXED_ZERO),
        floors: None,
        clocks: None,
        difficulty: None,
        characters: vec![CharacterPlacement {
            character: character_id.to_string(),
            x: probe.player_x + Fixed::from_num(ENEMY_OFFSET_X),
            y: probe.player_y,
            at_frame: ENEMY_AT_FRAME,
            variant: None,
            team: None,
        }],
        mode: arena.mode,
    }
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
        }],
        expect,
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Invariants::default(),
        // Isoler les essais d'armes des drops : ils ont leur propre scénario de preuve.
        powerups: vec![],
        powerup_drop_chance_override: Some(bevy_fixed::fixed_math::FIXED_ZERO),
        floors: None,
        clocks: None,
        difficulty: None,
        characters: vec![],
        mode: arena.mode,
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
