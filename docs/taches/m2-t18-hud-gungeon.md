# m2-t18-hud-gungeon — écrans en `bevy_hui`, puis HUD gungeon (M2-T18, seconde partie)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« HUD », « Simulation et présentation »), le rapport
`docs/taches/rapports/m2-t18-ui-comparatif.md` (première partie), `docs/conventions.md` §15, §32 et
§56, `crates/game/src/ui/` (`screens/`, `hud.rs`, `hud_model.rs`), `games/gungeon/assets/ui/`. Branche
`m2-t18-hud-gungeon` depuis `origin/main`. Estimation : 3 j. Compléter §56.

## Décision de William (2026-10-15)

**`bevy_hui` pour les écrans** (menu de pause, inventaire, et les écrans à venir), contre la
recommandation de la première partie, qui penchait pour la couche maison. Le HUD reste la couche
maison `ui/hud.ron` (plan §7, point 2) : il n'est pas concerné par cette décision.

## À faire

1. **Écrans : ne garder que `bevy_hui`.**
   - Feature `ui_hui` active par défaut dans le jeu natif et wasm ; le réglage `ALACOD_SCREENS` disparaît.
   - Retirer l'option maison (`screens/custom.rs`, `{pause,inventory}_screen.ron`). Garder le modèle
     commun (`model.rs`, `mod.rs`) et ses tests.
   - Réduire la colle par écran de `hui.rs` (aide générique pour les listes et le focus), sans
     forker `bevy_hui`. La correction de la panique (inventaire modifié pendant qu'il est ouvert) reste
     couverte par un test.
2. **Écrans existants en `bevy_hui`** : écran de fin (`game_over.rs`) et écran de mutation de throne
   (`mutation_screen.rs`), si le modèle commun s'y prête. Sinon, le dire au rapport et laisser une dette.
3. **HUD gungeon** (`games/gungeon/assets/ui/hud.ron`, couche maison) : blanks, clés, shells, objets
   passifs (icônes), actif et sa charge (barre), minicarte des salles (visitées, verrouillées, boss),
   en présentation seule.
   - Nouvelles sources dans `content::ui::HUD_SOURCES`, calculées par `hud_model` (pur, en headless).
   - Attentes `HudText` dans des scénarios gungeon existants.
   - La minicarte se dérive de `RoomStates` : aucun état nouveau dans la simulation.
4. Lint des `.html` d'écrans : au moins une vérification que les fichiers référencés existent et se lisent,
   avec une fixture.

## Règles

Présentation seule : rien dans `GgrsSchedule`, aucune trace ne bouge. Les crates de simulation
n'importent rien de rendu. `cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
GPU ni de Docker :
- les captures se font en rendu logiciel, comme dans la première partie ;
- dire au rapport les frames à regarder.

Pousser la branche régulièrement (commits WIP acceptés). Voie rapide du README §6 : fusionner
`origin/main` et rejouer la suite juste avant le commit `LIVRÉ m2-t18-hud-gungeon`, avec le rapport
`docs/taches/rapports/m2-t18-hud-gungeon.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
scripts, `cargo check` wasm32 du jeu gungeon).
