# X-1 « Points d'extension » — arrêt (2026-10-27, décision de William : gelé faute de besoin daté)

- **Branches poussées, non fusionnées** : `x-1-points-extension` @ `33ad1221` (worker b6, base `25af5b5f`) ; `integration-1837` @ `12543bc1`.
  `integration-1837` = `33ad1221` fusionné sur `main` `f2e39634` : conflits de conventions et de journal résolus, section X-1 renumérotée §91 (§89 = X-4a, §90 = SB-6) et décision « deux fantassins » au plan.
- **Ce que la branche fournit** :
  - `entry::run(GameSpec)` : un jeu y déclare ses plugins ; le même binaire sert la partie, `--sandbox`, les bots, `<jeu> lint` et `<jeu> scenarios gen|play`.
  - `game::extension::GamePlugin` : `build` (simulation, avant les cinq types rollback de clôture), `assemble` (partie seule) et `extension()`.
  - `Custom` sur `Behavior`, `Action` et `Condition`, avec des paramètres bruts ; section `plugin` de `game.ron` (kinds, ids, règles de lint) ; `Registry::plugin_content` ; fixture `plugin_game`.
- **Tests lus sur `12543bc1`** (VPS, headless, sans BLESS) :
  - `make test_scenarios` : `test scenarios ... ok`, 5 réussis, 0 échec, 810 lignes de scénario, 11 515 s ; aucun fichier modifié ensuite (`git status` vide).
  - `alacod_lint_knows_the_plugin_vocabulary_from_game_ron` : 1 réussi. `plugin_game_runs_its_scenario_with_its_own_binary` (e2e ignoré : `new`, plugin, `lint` du binaire, `scenarios play --bless`) : 1 réussi en 2 375 s.
  - `check-forbidden`, `check-rollback-registration`, `cargo fmt --check` : OK.
- **Rapport du worker sur `33ad1221`** (non rejoué ici) : `schedule_order_diff` vide (4 550 lignes, 645 types), tests par paquet sans échec, build des cinq jeux.
- **Non vérifié** : `schedule_order_diff` sur `12543bc1`, chemin wasm de `entry` (`GameSpec::embedded` non compilé), bench, p2p.
- **Dettes** : D133 fermée (cible séparée pour la base de `schedule_order_diff`). D139 ouverte : les règles de lint Rust d'un plugin ne tournent que dans `<jeu> lint`, pas dans `alacod lint`.
  Elle est décrite dans le rapport et au §91, sans ligne dans `dettes.md`. D140 à D143 n'ont pas été prises.
- **Limites connues** : un seul jeu de plugins par processus (état global installé par `entry::run`). Un lecteur d'`Action::Custom` doit se placer `.after(effect_status_system)`.
- **Reprise** (quand un jeu en a besoin, par exemple 1837 avec du Rust propre) : partir de `integration-1837` (`12543bc1`) et rebaser sur `main`. Rejouer `make test_scenarios`, `make schedule_order_diff` et `make test_alacod_cli`.
  Puis ajouter la ligne D139 à `dettes.md`. Aucun tag avant la preuve d'un jeu externe compilé sur le tag.
