# Passation R-1 « Rendu façon sprite » (Mac, bascule de modèle, sam 7 nov)

Jalon : S1 dim 15. Branche engine `wquintal/r-1-rendu-facon-sprite` (tête : voir le commit de cette note ; code à `45b0133a`, `origin/main` avec M-1 fusionné) ;
branche 1837 `r-1-rendu` @ `db66451` (poussée, rev engine `45b0133a`). Rapport déjà écrit : `docs/taches/rapports/r-1-rendu-facon-sprite.md` (à compléter par
les dernières lignes de `make check_rapide` une fois vert) ; § 111, D198–D199, ligne `LIVRÉ` du journal : faits. Spec : `~/orca/runs/alpha/r-1-rendu-facon-sprite.md`.

## Fait et vérifié
- Le WIP compile (natif, Metal) ; pantin du testbed affiché (`play_scenario --capture`), Frère et fantassin dans 1837 (capture hors écran par le `play_scenario` de l'engine :
  `ALACOD_LABELS=0 ALACOD_MUTE=1 ALACOD_GAME_NAME=1837 ALACOD_GAME_DIR=<1837> target/debug/play_scenario <1837>/docs/preuves/r-1/<scénario>.ron --capture <dossier> --every 5`).
- Captures : `~/orca/runs/alpha/logs/r-1-captures/` (`frere-cote-a-cote.png` à côté du banc `/labo/armes/` pris dans Chrome, `frere-planche.png` 3 directions × 6 poses,
  `tri-y.png`, `scene-fantassin.png`, `pantin-planche.png`, `web-chrome-pantin*.png`) ; les trois premières aussi dans `1837/docs/preuves/r-1/`.
- Traces : `model_pantin` neuve (`BLESS` de ce seul scénario) ; `two_players_shooting`, `four_players_idle`, `dash_aim_change` inchangées.
- Essai wasm dans Chrome (une session muette) : build de `zombies` dont le joueur était, le temps de l'essai, le pantin (changement jeté) : rendu correct en WebGL2
  (`ANGLE Metal Renderer: Apple M4 Max`), aucune erreur de shader. `make check_forbidden` vert (HashMap → BTreeMap dans `model3d`), `cargo fmt` propre,
  `cargo test -p content models` vert, `alacod lint` de 1837 sans erreur.
- Statuts envoyés : pantin (bf013dfb), Frère dans 1837 (5feb9ab0).

## Pas vert : `make check_rapide WASM=1` sur `45b0133a`
Journal `~/orca/runs/alpha/logs/r-1-rendu-facon-sprite-check-rapide-45b0133a.log` : format, test_tools, check_forbidden, check_rollback_registration,
check_sim_boundary, lint passent ; **`test_crates` échoue sur un test** (le seul, 349 autres de `game --lib` verts) :
`model3d::pose::tests::gaits_take_the_nearest_baked_direction_or_its_opposite_backwards`, `pose.rs:340` : `variant(&m, "fire", 0.3, false)` rend `fire-walk` au lieu de
`fire`. Cause : `AnimeManifest::directional(base)` (`crates/content/src/models.rs`) prend tout clip `base-*` qui a une direction, donc `fire-walk` (autre famille) passe
pour une variante de `fire`. Correctif à faire : une variante de `base` est `base` ou `base-<suffixe>` dont le suffixe n'est pas lui-même la base d'une autre famille
dirigée (p. ex. exclure `name` si un autre clip dirigé `base-<x>` est préfixe de `name` et que `base-<x>` n'est pas une direction, ou plus simplement exiger que le
suffixe soit un mot de direction connu : `gauche|droite|avant-*|arriere*|l|r|fl|fr|bl|br|b` selon le manifeste). Impact en jeu : `variant` n'est appelé qu'avec des bases
« en marchant » (`tir-marche`, `marche`…), d'où un défaut invisible dans les captures ; `knows()` du lint l'utilise aussi. `check_wasm` n'a pas tourné (après l'échec).

## Prochaine étape exacte
1. Corriger `directional` (content) + garder le test ; `cargo test -p content models` et
   `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 cargo test -p game --profile headless --no-default-features --lib model3d`.
2. Pousser, puis `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make check_rapide WASM=1 > ~/orca/runs/alpha/logs/r-1-rendu-facon-sprite-check-rapide-<sha>.log 2>&1`
   (`target_wasm` est déjà chaud : ~10 Go). Vert → status « E prêt <sha> » + journal ; dernières lignes dans le rapport ; supprimer cette note dans ce commit.
3. Réépingler le rev de `1837/r-1-rendu` sur la tête engine finale (sept lignes `Cargo.toml`, `cargo check` pour le `Cargo.lock`), pousser.
4. Essai de **1837** dans Chrome (M-1 est sur `main` : `alacod build --web`) : branche locale **non poussée** `r-1-web-essai` dans `~/orca/workspaces/1837/r-1-rendu`
   (= `r-1-rendu` + `origin/m-1-web`, conflits pris de notre côté pour `Cargo.toml`/`Cargo.lock`/`jouer.sh`). Commande :
   `cd ~/orca/workspaces/alacod/r-1-rendu-facon-sprite && CARGO_TARGET_DIR=$PWD/target_wasm cargo run -p content --bin alacod -- build --web ~/orca/workspaces/1837/r-1-rendu --serve`
   (après `git -C ~/orca/workspaces/1837/r-1-rendu switch r-1-web-essai`), une capture dans Chrome muet (`--mute-audio`, profil `/tmp/r1/chrome-profile`), puis revenir sur `r-1-rendu`.
   `start_map` de 1837 est maintenant `maps/salle.ldtk` (le Frère et 4 fantassins).
5. `worker_done` (rapport, SHA engine et 1837).

## Pièges
- Un seul cargo à la fois ; le build de 1837 natif utilise le target de l'engine (`CARGO_TARGET_DIR=<engine>/target`) et le patch `/tmp/r1/patch-1837.toml`
  (`cargo --config /tmp/r1/patch-1837.toml …` : engine en chemin local) ; sans patch, le rev git poussé. Disque ~43 Go : `scripts/target-sweep.sh target` entre deux builds.
- Écran du Mac éteint la nuit : la fenêtre native rend noir (`alacod remote screenshot` noir) ; utiliser la capture hors écran.
- Tri en y : ne pas « unifier » les `z` entre couches (essayé : le décor de 1837 et les joueurs sont sur la même couche, z 3 ; le vrai défaut était l'image du modèle à
  `z` 0 contre 1,0 pour les planches importées → champ `offset_z`, 1,0 dans 1837). Testbed : `y_sort: false` (décision du 7 nov).
- Bouche du canon = présentation (décalage du dessin), aucune trace ne bouge. Épaulé invisible en 1837 (D199). `scripts/` Python d'images jetables dans `/tmp/r1/`.
- Ne pas utiliser `sed`/`python` pour éditer des fichiers du dépôt.

## Retours de revue ouverts
Aucun.
