//! M3-T12 (`docs/conventions.md` §77) : lint des kinds M3 et atteignabilité des mécaniques.
//!
//! Chaque règle part de la fixture `mechanics_ok` (un jeu qui porte les douze mécaniques déclarées, sans
//! erreur ni avertissement de mécanique) et lui retire ou lui change un élément, dans un dossier
//! temporaire : une règle = une mutation = un avertissement (ou une erreur) attendu, le reste de la base
//! reste muet.

use content::{load_and_lint, LintError, LintErrorKind, LintWarning, LintWarningKind};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

/// Une modification de la copie temporaire de la fixture (chemins relatifs à `assets/`).
enum Edit<'a> {
    Write(&'a str, &'a str),
    Remove(&'a str),
    Replace(&'a str, &'a str, &'a str),
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

struct Run {
    dir: PathBuf,
    errors: Vec<LintError>,
    warnings: Vec<LintWarning>,
}

impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn run(test: &str, edits: &[Edit<'_>]) -> Run {
    let dir = std::env::temp_dir().join(format!("alacod-m3t12-{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&fixture_dir("mechanics_ok"), &dir);
    let assets = dir.join("assets");
    for edit in edits {
        match edit {
            Edit::Write(rel, content) => fs::write(assets.join(rel), content).unwrap(),
            Edit::Remove(rel) => fs::remove_file(assets.join(rel)).unwrap(),
            Edit::Replace(rel, from, to) => {
                let text = fs::read_to_string(assets.join(rel)).unwrap();
                assert!(text.contains(from), "{rel} : « {from} » absent");
                fs::write(assets.join(rel), text.replace(from, to)).unwrap();
            }
        }
    }
    let (registry, manifest, errors) = load_and_lint(&dir).unwrap();
    let warnings = content::warnings(&registry, &manifest);
    Run {
        dir,
        errors,
        warnings,
    }
}

impl Run {
    fn mechanic_warnings(&self) -> Vec<&LintWarning> {
        self.warnings
            .iter()
            .filter(|w| w.kind == LintWarningKind::MechanicUnmet)
            .collect()
    }

    /// Un avertissement de mécanique contient `needle`.
    fn assert_warns(&self, needle: &str) {
        assert!(
            self.mechanic_warnings()
                .iter()
                .any(|w| w.message.contains(needle)),
            "attendu un avertissement contenant {needle:?}, obtenu :\n{:#?}",
            self.warnings
        );
    }

    fn assert_silent_about(&self, needle: &str) {
        assert!(
            self.warnings.iter().all(|w| !w.message.contains(needle)),
            "avertissement inattendu contenant {needle:?} :\n{:#?}",
            self.warnings
        );
    }

    fn assert_error(&self, kind: LintErrorKind, needle: &str) {
        assert!(
            self.errors
                .iter()
                .any(|e| e.kind == kind && e.message.contains(needle)),
            "attendu une erreur {kind:?} contenant {needle:?}, obtenu :\n{:#?}",
            self.errors
        );
    }
}

/// La base ne produit ni erreur, ni avertissement de mécanique, ni avertissement d'atteignabilité : elle
/// est la preuve que chaque avertissement des tests suivants vient de sa mutation.
#[test]
fn base_is_clean() {
    let base = run("base", &[]);
    assert!(base.errors.is_empty(), "{:#?}", base.errors);
    assert!(base.warnings.is_empty(), "{:#?}", base.warnings);
}

/// Sans `mechanics`, un contenu de mécanique absent n'est pas signalé (isaac avant T11 : seules les
/// règles « le contenu est là, rien ne le porte » parlent).
#[test]
fn undeclared_mechanics_are_not_checked() {
    let run = run(
        "undeclared",
        &[
            Edit::Replace(
                "game.ron",
                "mechanics: [Tears, Effects, Familiars, Bombs, SecretRoom, Keys, Hearts, ExhaustPools, DevilMarket, Champions, Curses, Achievements],",
                "",
            ),
            Edit::Write("loot/treasure.ron", "(entries: [(pick: Item(\"key\"), weight: 1)])"),
        ],
    );
    run.assert_silent_about("de la partie ordinaire");
}

// ---- règles toujours actives --------------------------------------------------------------------

#[test]
fn pool_that_nothing_draws() {
    let run = run(
        "pool_orphan",
        &[Edit::Write(
            "loot/orphelin.ron",
            "(entries: [(pick: Item(\"coeur\"), weight: 1)])",
        )],
    );
    run.assert_warns("pool « orphelin » : aucune salle");
    run.assert_silent_about("pool « treasure »");
}

#[test]
fn pool_of_a_default_chest_is_drawn() {
    // Un coffre sans `table` tire `commun` quand la séquence n'a pas de `loot` (et pas un pool d'étage
    // d'une autre séquence).
    let run = run(
        "pool_default_chest",
        &[
            Edit::Write(
                "rooms/diable.ron",
                "(chest: Some((table: None, locked: false, rarity: Common)))",
            ),
            Edit::Write(
                "loot/commun.ron",
                "(entries: [(pick: Item(\"coeur\"), weight: 1)])",
            ),
        ],
    );
    run.assert_silent_about("pool « commun »");
}

#[test]
fn familiar_whose_items_have_no_source() {
    let run = run(
        "familiar_no_source",
        &[Edit::Write(
            "loot/treasure.ron",
            "(entries: [(pick: Item(\"larme\"), weight: 1), (pick: Item(\"lentille\"), weight: 1), (pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1)], exhaust: true)",
        )],
    );
    run.assert_warns(
        "familier « orbital » : aucun objet atteignable ne le porte (objet(s) : orbital",
    );
    // l'avertissement d'objet nomme la mécanique
    assert!(
        run.warnings
            .iter()
            .any(|w| w.message.contains("objet « orbital » : aucune source")
                && w.message.contains("mécanique : familiers")),
        "{:#?}",
        run.warnings
    );
    run.assert_silent_about("familier « satellite »");
}

#[test]
fn synergy_with_an_item_that_has_no_source() {
    let run = run(
        "synergy_no_source",
        &[
            Edit::Write(
                "objets/isole.ron",
                "(name: \"Isolé\", kind: Passive, rarity: Common, modifiers: [(stat: MoveSpeed, op: Pct, value: \"0.1\")])",
            ),
            Edit::Write(
                "synergies/bouclier_larme.ron",
                "(name: \"x\", requires: [\"larme\", \"isole\"], modifiers: [(stat: MoveSpeed, op: Pct, value: \"0.1\")])",
            ),
        ],
    );
    run.assert_warns("synergie « bouclier_larme » : objet(s) requis sans source (isole)");
}

#[test]
fn special_room_that_carries_nothing() {
    let run = run(
        "diable_empty",
        &[Edit::Write("rooms/diable.ron", "(locks: false)")],
    );
    run.assert_warns("type de salle « diable » posé par la grammaire « etage » mais sans coffre");
    assert!(
        run.mechanic_warnings()
            .iter()
            .any(|w| w.message.contains("marché du diable (salle post-boss)")),
        "{:#?}",
        run.warnings
    );
    // le pool du diable n'est plus tiré par personne
    run.assert_warns("pool « devil » : aucune salle");
}

#[test]
fn secret_wall_without_a_bomb() {
    let run = run(
        "secret_no_bomb",
        &[Edit::Write(
            "characters/player.ron",
            "(movement: (max_speed: \"60.0\"), asset_name_ref: \"player\", base_health: (max: \"6.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {}), \"2\": (layers: {})})",
        )],
    );
    run.assert_warns("type de salle « secrete » : mur secret atteint mais aucune bombe");
    // même message de mécanique côté déclaration
    run.assert_warns("aucun personnage jouable n'a `bomb`");
}

#[test]
fn curse_that_no_sequence_draws() {
    let run = run(
        "curse_orphan",
        &[Edit::Write(
            "curses/labyrinthe.ron",
            "(weight: 1, effects: [HideHealth])",
        )],
    );
    run.assert_warns(
        "curse « labyrinthe » : aucune séquence `Floors` ne la met dans `curses.pool`",
    );
    run.assert_silent_about("curse « darkness »");
}

#[test]
fn variant_with_a_zero_chance() {
    let run = run(
        "variant_chance_zero",
        &[Edit::Replace(
            "characters/grunt.ron",
            "chance: \"0.2\"",
            "chance: \"0.0\"",
        )],
    );
    run.assert_warns("personnage « grunt » : `variants.chance` nulle");
}

#[test]
fn achievement_with_an_impossible_condition() {
    let run = run(
        "achievement_floor",
        &[Edit::Write(
            "achievements/deux_etages.ron",
            "(when: [FloorAtLeast(5)], unlocks: [\"debloque\"])",
        )],
    );
    run.assert_warns(
        "accomplissement « deux_etages » : condition inaccomplissable, FloorAtLeast(5)",
    );
    let run = self::run(
        "achievement_item",
        &[Edit::Write(
            "achievements/deux_etages.ron",
            "(when: [ItemHeld(\"fantome\")], unlocks: [\"debloque\"])",
        )],
    );
    run.assert_warns("ItemHeld(« fantome »)");
}

#[test]
fn unlock_that_nothing_grants() {
    let run = run(
        "unlock_orphan",
        &[Edit::Remove("achievements/deux_etages.ron")],
    );
    run.assert_warns("déblocage « debloque » : aucun accomplissement ni service `Merchant`");
}

// ---- mécaniques déclarées -----------------------------------------------------------------------

const TREASURE_NO_TEARS: &str = "(entries: [(pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1), (pick: Item(\"orbital\"), weight: 1)], exhaust: true)";

#[test]
fn tears_missing_from_the_treasure_pool() {
    let run = run(
        "tears_treasure",
        &[Edit::Write("loot/treasure.ron", TREASURE_NO_TEARS)],
    );
    run.assert_warns("mécanique « larmes » : aucun objet de larmes (`projectile`, `ShotSpeed`, `ShotSize`) dans le pool trésor");
    run.assert_silent_about(
        "mécanique « larmes » : aucun objet de larmes dans le pool d'une boutique",
    );
}

#[test]
fn tears_missing_from_the_shop_pool() {
    let run = run(
        "tears_shop",
        &[Edit::Write(
            "loot/shop.ron",
            "(entries: [(pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1), (pick: Item(\"key\"), weight: 1)])",
        )],
    );
    run.assert_warns("mécanique « larmes » : aucun objet de larmes dans le pool d'une boutique");
}

#[test]
fn tears_that_cannot_be_held_twice() {
    let run = run(
        "tears_exclude_held",
        &[
            Edit::Replace(
                "loot/treasure.ron",
                "exhaust: true",
                "exhaust: true, exclude_held: true",
            ),
            Edit::Replace(
                "loot/shop.ron",
                "(pick: Item(\"key\"), weight: 1),\n    ],",
                "(pick: Item(\"key\"), weight: 1),\n    ],\n    exclude_held: true,",
            ),
        ],
    );
    run.assert_warns("objet de larmes « larme » : deux exemplaires impossibles");
}

#[test]
fn effects_missing_from_the_boss_pool_and_no_transformation() {
    let run = run(
        "effects",
        &[
            Edit::Write(
                "loot/boss_pool.ron",
                "(entries: [(pick: Item(\"coeur\"), weight: 1)])",
            ),
            Edit::Remove("synergies/bouclier_larme.ron"),
        ],
    );
    run.assert_warns(
        "mécanique « effets et transformations » : aucun objet à effets dans le pool boss",
    );
    run.assert_warns("aucune synergie (transformation)");
}

#[test]
fn familiars_missing_from_the_treasure_and_shop_pools() {
    let run = run(
        "familiars",
        &[
            Edit::Write(
                "loot/treasure.ron",
                "(entries: [(pick: Item(\"larme\"), weight: 1), (pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1)], exhaust: true)",
            ),
            Edit::Write(
                "loot/shop.ron",
                "(entries: [(pick: Item(\"larme\"), weight: 1), (pick: Item(\"effet\"), weight: 1), (pick: Item(\"key\"), weight: 1)])",
            ),
        ],
    );
    run.assert_warns(
        "1 objet(s) à `familiar` dans le pool trésor de la partie ordinaire, 2 attendus",
    );
    run.assert_warns("aucun objet à `familiar` dans le pool d'une boutique");
}

#[test]
fn bombs_rocks_and_spikes() {
    let run = run(
        "bombs",
        &[
            Edit::Write(
                "loot/salle.ron",
                "(entries: [(pick: Item(\"key\"), weight: 2), (pick: Item(\"coeur\"), weight: 2), (pick: Currency(1), weight: 4)], reward: Some((chance: \"0.33\", chest_chance: \"0.0\")))",
            ),
            Edit::Replace("maps/salles.ldtk", "\"Rocks\"", "\"Pierres\""),
            Edit::Replace("maps/salles.ldtk", "\"Surfaces\"", "\"Sols\""),
        ],
    );
    run.assert_warns("aucune bombe de départ");
    run.assert_warns("aucun rocher (couche `Rocks`");
    run.assert_warns("aucune pique (surface à `hazard`");
    run.assert_silent_about("aucun personnage jouable n'a `bomb`");
}

#[test]
fn secret_room_not_required_or_not_walled() {
    let run = run(
        "secret_required",
        &[Edit::Replace(
            "floor_grammars/etage.ron",
            "\"secrete\": 1, ",
            "",
        )],
    );
    run.assert_warns(
        "grammaire « etage » : `required` ne demande pas la salle secrète « secrete »",
    );
    // l'avertissement M2 « aucune grammaire ne peut le tirer » nomme la mécanique que la salle porte
    assert!(
        run.warnings
            .iter()
            .any(|w| w.kind == LintWarningKind::Unreachable
                && w.message
                    .contains("type de salle « secrete » : aucune grammaire")
                && w.message
                    .contains("mécanique : salle secrète (murée, ouverte à la bombe)")),
        "{:#?}",
        run.warnings
    );
    let run = self::run(
        "secret_not_walled",
        &[Edit::Write(
            "rooms/secrete.ron",
            "(chest: Some((table: Some(\"treasure\"), locked: false, rarity: Common)))",
        )],
    );
    run.assert_warns("le type « secrete » n'a pas `door_lock: Secret`");
    // la même incohérence est une erreur de lint
    run.assert_error(
        LintErrorKind::OutOfRange,
        "la salle secrète « secrete » (required) a un type de salle sans `door_lock: Secret`",
    );
}

#[test]
fn keys_in_rewards_shops_and_locked_treasure() {
    let run = run(
        "keys",
        &[
            Edit::Write(
                "loot/salle.ron",
                "(entries: [(pick: Item(\"bomb\"), weight: 2), (pick: Item(\"coeur\"), weight: 2), (pick: Currency(1), weight: 4)], reward: Some((chance: \"0.33\", chest_chance: \"0.0\")))",
            ),
            Edit::Write(
                "loot/shop.ron",
                "(entries: [(pick: Item(\"larme\"), weight: 1), (pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1)])",
            ),
            Edit::Write("rooms/tresor.ron", "(chest: Some((table: Some(\"treasure\"), locked: false, rarity: Common)))"),
        ],
    );
    run.assert_warns("aucune clé (« key ») dans le pool de récompense de combat");
    run.assert_warns("aucune clé dans le pool d'une boutique");
    run.assert_warns("aucune salle exigée par la grammaire n'a `door_lock: Key`");
}

#[test]
fn hearts_coins_and_shop() {
    let run = run(
        "hearts",
        &[
            Edit::Write(
                "loot/salle.ron",
                "(entries: [(pick: Item(\"key\"), weight: 2), (pick: Item(\"bomb\"), weight: 2)], reward: Some((chance: \"0.33\", chest_chance: \"0.0\")))",
            ),
            Edit::Replace("floor_grammars/etage.ron", ", \"boutique\": 1", ""),
        ],
    );
    run.assert_warns("aucun tas de sous (`Currency`)");
    run.assert_warns("aucun cœur (objet à `Heal` ou `GainSoul`)");
    run.assert_warns("aucune boutique exigée par la grammaire");
}

#[test]
fn exhausting_pool_drawn_once() {
    let run = run(
        "exhaust_none",
        &[Edit::Replace(
            "loot/treasure.ron",
            "exhaust: true",
            "exhaust: false",
        )],
    );
    run.assert_warns("aucun pool n'a `exhaust: true`");
    // un seul étage et la salle secrète sur un autre pool : le pool n'est tiré qu'une fois
    let run = self::run(
        "exhaust_once",
        &[
            Edit::Write("floors/sequence.ron", "(levels: [\"floor:etage\"], victory_at_end: true, consumables_per_floor: {\"key\": 1}, curses: Some((chance: \"0.5\", pool: [\"darkness\"])))"),
            Edit::Write("rooms/secrete.ron", "(door_lock: Some(Secret), chest: Some((table: Some(\"devil\"), locked: false, rarity: Common)))"),
        ],
    );
    run.assert_warns("« treasure » tiré 1 fois");
}

#[test]
fn devil_market_without_a_post_boss_room() {
    let run = run(
        "devil",
        &[Edit::Replace(
            "floor_grammars/etage.ron",
            ", post_boss: Some((kind: \"diable\", chance: \"0.5\"))",
            "",
        )],
    );
    run.assert_warns(
        "aucune grammaire de la partie ordinaire n'a de salle post-boss à chance non nulle",
    );
}

#[test]
fn champions_on_no_reachable_enemy() {
    let run = run(
        "champions",
        &[Edit::Write(
            "characters/grunt.ron",
            "(movement: (max_speed: \"60.0\"), asset_name_ref: \"grunt\", base_health: (max: \"10.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {})}, ai: Some((behaviors: Some([Chase(profile: \"Ground\")]))))",
        )],
    );
    run.assert_warns("aucun ennemi des salles de la partie ordinaire ne porte `variants`");
}

/// Les curses ne comptent que dans la séquence ordinaire (`entry.floors`) : une autre séquence qui en tire
/// ne sauve pas la partie ordinaire, mais sauve la curse de l'avertissement « jamais tirée ».
#[test]
fn curses_only_count_in_the_ordinary_sequence() {
    let run = run(
        "curses",
        &[
            Edit::Write(
                "floors/sequence.ron",
                "(levels: [\"floor:etage\", \"floor:etage\"], victory_at_end: true, consumables_per_floor: {\"key\": 1})",
            ),
            Edit::Write(
                "floors/essai.ron",
                "(levels: [\"floor:etage\"], victory_at_end: true, curses: Some((chance: \"1.0\", pool: [\"darkness\"])))",
            ),
        ],
    );
    run.assert_warns("la séquence ordinaire n'a pas de `curses`");
    run.assert_silent_about("curse « darkness »");
}

#[test]
fn achievements_without_a_visible_unlock() {
    let run = run(
        "achievements",
        &[Edit::Write(
            "unlocks/debloque.ron",
            "(name: \"x\", grants: [Item(\"relique\")])",
        )],
    );
    run.assert_warns(
        "aucun accomplissement dont le déblocage donne un objet ou une entrée à un pool tiré",
    );
}

// ---- erreurs de lint ----------------------------------------------------------------------------

#[test]
fn exhaust_pool_of_coins_only_is_an_error() {
    let run = run(
        "exhaust_currency",
        &[Edit::Write(
            "loot/treasure.ron",
            "(entries: [(pick: Currency(1), weight: 1)], exhaust: true)",
        )],
    );
    run.assert_error(
        LintErrorKind::OutOfRange,
        "pool de butin « treasure » : exhaust: true mais toutes les entrées sont des Currency",
    );
}

#[test]
fn rocks_and_surfaces_layers_are_checked() {
    let run = run(
        "layers",
        &[
            Edit::Replace("maps/salles.ldtk", "[0, 1, 0, 2, 0,", "[0, 1, 0, 3, 0,"),
            Edit::Replace(
                "maps/salles.ldtk",
                "[0, 0, 0, 0, 0, 0, 3, 0, 0, 0,",
                "[0, 0, 0, 0, 0, 0, 9, 0, 0, 0,",
            ),
        ],
    );
    run.assert_error(
        LintErrorKind::OutOfRange,
        "couche Rocks : valeur(s) 3 : seules 1 (rocher) et 2",
    );
    run.assert_error(
        LintErrorKind::BrokenReference,
        "couche Surfaces : valeur(s) 9 sans surface",
    );
}

/// `alacod lint --json` rend les avertissements de mécanique avec leur kind.
#[test]
fn lint_json_reports_mechanic_warnings() {
    let dir = std::env::temp_dir().join(format!("alacod-m3t12-json-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&fixture_dir("mechanics_ok"), &dir);
    fs::write(dir.join("assets/loot/treasure.ron"), TREASURE_NO_TEARS).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_alacod"))
        .args(["lint", "--json"])
        .arg(&dir)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let warnings = report["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["kind"] == "MechanicUnmet"
            && w["message"]
                .as_str()
                .unwrap()
                .contains("mécanique « larmes »")),
        "{report}"
    );
}

// ---- corrections de la relecture ------------------------------------------------------------------

/// Une boutique à `floor_tables` ne tire jamais `table` (`RoomShopDef::table_for_floor`).
#[test]
fn shop_table_is_not_drawn_when_floor_tables_exist() {
    let run = run(
        "shop_floor_tables",
        &[Edit::Write(
            "rooms/boutique.ron",
            "(shop: Some((table: \"shop\", shelves: 2, floor_tables: [\"devil\"])))",
        )],
    );
    run.assert_warns("pool « shop » : aucune salle");
    run.assert_silent_about("pool « devil »");
}

/// `salle` n'est tiré en récompense que si le pool de l'étage n'a pas de `reward`.
#[test]
fn reward_pool_of_the_floor_replaces_the_default() {
    let run = run(
        "reward_floor",
        &[
            Edit::Write(
                "floors/sequence.ron",
                "(levels: [\"floor:etage\", \"floor:etage\"], victory_at_end: true, consumables_per_floor: {\"key\": 1}, loot: [\"recompense\"], curses: Some((chance: \"0.5\", pool: [\"darkness\"])))",
            ),
            Edit::Write(
                "loot/recompense.ron",
                "(entries: [(pick: Item(\"coeur\"), weight: 1)], reward: Some((chance: \"0.5\", chest_chance: \"0.0\")))",
            ),
        ],
    );
    run.assert_warns("pool « salle » : aucune salle");
    run.assert_silent_about("pool « recompense »");
}

/// Un coffre de récompense (`chest_chance > 0`) tire `commun` (aucun `loot` d'étage).
#[test]
fn reward_chest_draws_the_default_chest_pool() {
    let run = run(
        "reward_chest",
        &[
            Edit::Replace(
                "loot/salle.ron",
                "chest_chance: \"0.0\"",
                "chest_chance: \"0.5\"",
            ),
            Edit::Write(
                "loot/commun.ron",
                "(entries: [(pick: Item(\"coeur\"), weight: 1)])",
            ),
        ],
    );
    run.assert_silent_about("pool « commun »");
}

/// Un objet à `exhaust` dans un seul pool ne sort pas deux fois : l'empilement n'est pas rencontré.
#[test]
fn tears_in_a_single_exhausting_pool_cannot_stack() {
    let run = run(
        "tears_exhaust_only",
        &[Edit::Write(
            "loot/shop.ron",
            "(entries: [(pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1), (pick: Item(\"key\"), weight: 1)])",
        )],
    );
    run.assert_warns("objet de larmes « larme » : deux exemplaires impossibles");
}

/// La bombe d'un personnage de test ne compte pas (D63) ; une explosion enveloppée compte.
#[test]
fn test_only_characters_give_no_bomb_and_wrapped_explosions_count() {
    let no_player_bomb = "(movement: (max_speed: \"60.0\"), asset_name_ref: \"player\", base_health: (max: \"6.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {}), \"2\": (layers: {})})";
    let run = run(
        "bomb_test_only",
        &[
            Edit::Write("characters/player.ron", no_player_bomb),
            Edit::Write(
                "characters/essai.ron",
                "(test_only: true, movement: (max_speed: \"60.0\"), asset_name_ref: \"essai\", base_health: (max: \"6.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {})}, bomb: Some((fuse_frames: 30, radius: \"48.0\", damage: \"10.0\")))",
            ),
        ],
    );
    run.assert_warns("mur secret atteint mais aucune bombe");
    let run = self::run(
        "bomb_wrapped_explosion",
        &[
            Edit::Write("characters/player.ron", no_player_bomb),
            Edit::Write(
                "objets/mine.ron",
                "(name: \"Mine\", kind: Passive, rarity: Common, effects: [(on: OnPickup, do: [Scaled(action: Explode(radius: \"48.0\", damage: \"1.0\", destroys: true), scale: Some(PerCopy))])])",
            ),
            Edit::Replace("loot/treasure.ron", "(pick: Item(\"orbital\"), weight: 1),", "(pick: Item(\"orbital\"), weight: 1), (pick: Item(\"mine\"), weight: 1),"),
        ],
    );
    run.assert_silent_about("mur secret atteint mais aucune bombe");
}

/// Un familier de test n'est pas signalé (D63).
#[test]
fn test_only_familiar_is_not_warned() {
    let run = run(
        "familiar_test_only",
        &[
            Edit::Write(
                "loot/treasure.ron",
                "(entries: [(pick: Item(\"larme\"), weight: 1), (pick: Item(\"lentille\"), weight: 1), (pick: Item(\"effet\"), weight: 1), (pick: Item(\"satellite\"), weight: 1)], exhaust: true)",
            ),
            Edit::Write(
                "characters/orbital.ron",
                "(test_only: true, movement: (max_speed: \"60.0\"), asset_name_ref: \"orbital\", base_health: (max: \"20.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {})}, team: Some(Players), ai: Some((behaviors: Some([Orbit(radius: \"40.0\", frames_per_turn: 120)]))))",
            ),
        ],
    );
    run.assert_silent_about("familier « orbital »");
}

/// Un accomplissement n'est « visible » que si ses conditions peuvent être remplies.
#[test]
fn achievement_with_an_impossible_condition_is_not_visible() {
    let run = run(
        "achievement_not_visible",
        &[Edit::Write(
            "achievements/deux_etages.ron",
            "(when: [FloorAtLeast(5)], unlocks: [\"debloque\"])",
        )],
    );
    run.assert_warns("aucun accomplissement dont le déblocage donne un objet");
}

/// `stat_uses` lit les modificateurs de curse et les actions de phase de boss.
#[test]
fn unread_stats_are_found_in_curses_and_boss_phases() {
    let run = run(
        "unread_stats",
        &[
            Edit::Write(
                "curses/superstition.ron",
                "(weight: 1, effects: [Modifier(Custom(\"superstition\"), Add, \"1\")])",
            ),
            Edit::Write(
                "characters/chef.ron",
                "(movement: (max_speed: \"0.0\"), asset_name_ref: \"chef\", collider: (shape: Rectangle(width: \"20.\", height: \"20.\"), offset: (x: \"0.0\", y: \"0.0\", z: \"0.0\")), scale: \"1.0\", base_health: (max: \"100.0\"), starting_skin: \"1\", skins: {\"1\": (layers: {})}, ai: Some((aggro_range: Some(\"100.0\"), attack_range: Some(\"0.0\"), stationary: Some(true))), boss: Some((phases: [(on_enter: [Modifier(stat: Custom(\"rage\"), op: Add, value: \"1\")], timeline: Some((events: [(at: 0, do: [Heal(\"1\")])], repeat: Some(60))))])))",
            ),
        ],
    );
    run.assert_error(LintErrorKind::UnreadStat, "curse « superstition »");
    run.assert_error(LintErrorKind::UnreadStat, "phase 0, on_enter");
}
