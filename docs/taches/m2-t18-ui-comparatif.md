# m2-t18-ui-comparatif — test comparatif d'UI, puis HUD gungeon (M2-T18, V4, première partie)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« HUD », « Simulation et présentation »),
`docs/plan-engine.md` §7 (choix de l'UI, recommandation 1 à 4), `docs/conventions.md` §15 (HUD) et
§32 (HUD throne), `crates/game/src/ui/` (`hud.rs`, `hud_model.rs`, `mutation_screen.rs`,
`game_over.rs`), `games/gungeon/assets/ui/`. Branche `m2-t18-ui-comparatif` depuis `origin/main`.
Estimation : 2 j. Section de conventions réservée : **§56**.

## Contexte

Le plan (§7, point 3) prévoit au début de M2 un **test comparatif** pour les écrans (le HUD reste la
couche maison `ui/hud.ron`) : le même écran fait deux fois, avec `bevy_hui` (écrans décrits en
fichiers, rechargés à chaud) et avec la couche maison étendue. William choisit sur pièces. La partie
HUD gungeon (blanks, clés, objets, actif et sa charge, minicarte) est la **seconde** partie de M2-T18,
faite après sa décision, sur une autre branche.

## À faire

1. **Vérifier `bevy_hui`** : version compatible avec Bevy 0.19.1 (celle du dépôt). Si aucune ne
   l'est, le dire au rapport en une ligne, ne pas forker, et comparer la couche maison à la seconde
   option la plus crédible du §7 (ou s'arrêter là, au choix de l'agent, justifié).
2. **Deux écrans, faits deux fois**, derrière une feature ou un réglage, sans toucher au HUD :
   - le **menu de pause** : reprendre, recommencer, lobby, réglages vides ; navigation au clavier et
     à la **manette** (focus visible) ;
   - l'**inventaire gungeon** : armes, objets passifs, actif et sa charge, consommables (blanks,
     clés), lu dans l'état de la simulation (`Inventory`, `WeaponInventory`…), en présentation
     seulement.
3. **Critères au rapport**, un tableau par option : compile en wasm (`cargo check` cible wasm32),
   rechargement à chaud (oui/non, mesuré), manette, lignes de code et de données par écran, temps
   d'écriture, dépendances ajoutées, ce qu'il faudra jeter quand les fichiers `.bsn` de Bevy
   arriveront. Recommandation argumentée en dix lignes, sans décider à la place de William.
4. **Preuve sans écran** : tests des fonctions pures (modèle de l'inventaire, ordre du focus), et un
   scénario par écran qui ouvre l'écran (input de pause) et vérifie une attente `HudText` ou un
   modèle équivalent. Aucune trace ne bouge (présentation seule ; l'input de pause ne doit pas entrer
   dans la simulation, ou alors comme les autres boutons, documenté).
5. **Captures** : dire au rapport les commandes et frames pour les captures des deux options
   (`make videos SCENARIO=…` ou `play_scenario --capture`) ; l'orchestrateur les produit sur le Mac
   (GPU) pour William.

## Règles

Rien dans `GgrsSchedule` ; les crates de simulation n'importent rien de rendu. Aucune trace ne doit
bouger. Pas de dépendance ajoutée aux crates de simulation. `cargo fmt --all -- --check`.

## Hors périmètre

HUD gungeon (seconde partie, après la décision), refonte du HUD existant, lobby, menus principaux.

## Session cloud

Tâche prévue pour une session Claude Code cloud : ignorer les consignes `orca` du README (§1) ; pas
de GPU ni de Docker (p2p non requis : présentation seule). Pousser la branche régulièrement (commits
WIP acceptés). Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le
commit `LIVRÉ m2-t18-ui-comparatif`, avec le rapport `docs/taches/rapports/m2-t18-ui-comparatif.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`).
