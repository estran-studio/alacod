# X-5 — Création de 1837 et kit de la cour (VPS)

Base `origin/main` `7c707277`, branche `wquintal/x-5-depot-1837`. Fiche : `docs/taches/plan-1837-cli.md` (lot X-5, § 3 « Décisions de William », § 5, § 6), spec du run
`/home/debian/orca/runs/x-5/spec-x5.md`. Convention : `docs/conventions.md` § 107. **Simulation intouchée** : aucun type rollback, aucun système `GgrsSchedule`, aucune trace
de clone ne bouge ; seule la trace de `sprite_import_fantassin` disparaît avec le revert. Aucun `BLESS`. `make check` : lancé par le coordinateur sur la tête poussée.

## Partie E (engine)

| Point | Issue | Preuve |
|---|---|---|
| E1 retrait de l'exception du testbed | fait | `git revert` de `a81523fe`, `7b5d9d20`, `a878966c` (trois commits `Revert …`) : huit fichiers supprimés (personnage, feuille, animation, PNG, scénario, trace, entrée `sprites.ron` et `assets.yaml`). `docs/conventions.md` §91 (« Preuve sur un vrai manifeste ») et `plan-1837-cli.md` § 5 (exception levée) corrigés ; `x-4b-import-sprite.md` inchangé. `alacod lint games/testbed` : « aucune erreur (40 personnages, 25 armes…) ». |
| E2 `alacod new 1837` | fait | `new_game` accepte un nom qui commence par un chiffre ; `cargo_package_name` : `2048` → paquet et binaire `jeu-2048` (préfixe `jeu-` si le nom commence par un chiffre, sinon inchangé) ; `templates/starter/Cargo.toml` : `name = "{{package}}"`, `{{name}}` reste le nom du jeu (dossier, `game.ron`, titre de fenêtre, messages). Test `crates/content/tests/cli.rs::new_accepts_a_numeric_game_name_with_a_valid_cargo_package` (nom neutre `2048` ; `cargo metadata --no-deps` accepte le manifeste ; lint vert). `cargo test -p content --test cli` : 12 passés, 2 ignorés. `make test_alacod_cli` : 2 passés (`new_game_opens_its_sandbox_without_any_added_code`, `test_command_plays_generated_scenarios…`), 2164 s. |
| E3 import de décor | fait (la partie J l'a exigé) | `alacod import-sprite decor-mur-rondins.sprite.json` refusait : « clip « static » : fps = 0 : nombre fini > 0 ». Deux compléments génériques : `fps = 0` + une colonne = clip fixe ; `--direction <DIR>` garde une seule ligne (décor figé dans une orientation, `mirror: false`). Tests synthétiques neutres `crates/content/tests/sprite_import.rs` : `static_clip_with_zero_fps_imports_as_one_image`, `zero_fps_with_several_columns_or_a_negative_fps_fails_without_artifacts`, `direction_keeps_only_that_row_of_every_clip` ; 25 passés. |
| E4 docs et dettes | fait | `docs/conventions.md` § 107 (+ renvoi au § de `alacod new`) ; `templates/starter/CLAUDE.md` (nom du paquet) ; dettes D186–D189 dans `docs/taches/dettes.md`. |
| E5 vérifications | fait | `cargo test -p content --test cli`, `--test sprite_import`, `--lib sprite_import` verts ; `make test_alacod_cli` vert ; `make test_tools` : 50 tests OK ; `alacod lint games/testbed` : 0 erreur. Pas de `make check` (coordinateur). |

## Partie J (dépôt 1837)

Voir la fin de ce fichier, remplie après la partie J.
