# m1-dettes-doc-lot-2 — suites de la relecture des conventions (commentaires, README, nightly)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-dettes-doc-lot-2` créée depuis la tête livrée de `m1-relecture-conventions` `e75ec2f`, même
target). Petite tâche (½ j). Aucune trace, aucun changement de comportement de simulation.

## Contexte

Le rapport `docs/taches/rapports/m1-relecture-conventions.md` laisse trois écarts à décider,
six commentaires de code périmés et deux renvois faux dans `docs/taches/README.md`. Décisions
prises par l'orchestrateur ci-dessous ; cette tâche les applique.

## Décisions (fixées ici)

1. **Écart 1 (§9, `variant_health` appelle `resolve` directement)** : l'exception est
   **documentée pour de bon** dans §9 (une phrase : pourquoi au spawn, avant que `Stats` existe
   pour l'entité) et dans un commentaire `//` au point d'appel de
   `crates/game/src/character/variant.rs`. Pas de refonte.
2. **Écart 2 (« seules les tâches V1 changent les traces »)** : la règle devient « une tâche de
   toute voie peut bénir des traces, **avec la preuve du §10**, bénies par l'orchestrateur ; V1
   est la voie qui les change le plus souvent ». À corriger dans « Notes essentielles » de
   `conventions.md` ; `docs/taches.md` §1 règle 3 sera corrigée par l'orchestrateur (ne pas y
   toucher).
3. **Écart 3 (§6, MP4 absents de l'artefact du nightly)** : **copier les vidéos**.
   `scripts/nightly.sh` copie `target/videos/<commit>/*.mp4` (et `*.vues.mp4` s'il y en a) dans
   `nightly/<commit>/videos/` après `make videos`, sans échouer s'il n'y a pas de vidéo ; vérifier
   dans `.github/workflows/nightly.yaml` que `nightly/<commit>/` entier part dans l'artefact (sinon
   ajouter le chemin). La ligne « Vidéos MP4 » du §6 reste et précise le chemin. Vérification :
   `bash -n scripts/nightly.sh` et une lecture ; pas d'exécution du nightly.
4. **Commentaires de code périmés** (rapport, liste « hors numéros ») : les six sont corrigés
   (`health/mod.rs` six émetteurs ; `run_state.rs` restart livré D14 → §33 ; `run/src/run.rs`
   `Floors` livré §17, `Campaign` à venir ; `sim_core/src/kinds.rs` chemin `crates/combat` ;
   `games/throne/assets/items/powerups.ron` en-tête aux vrais poids et `drop_chance` ; `Makefile`
   commentaire de la cible `gen` qui dit ce que la recette fait). Commentaires et en-têtes
   seulement : aucune valeur RON ne change (le lint et les traces ne doivent pas bouger).
5. **`docs/taches/README.md`** : les deux « §8 » (l. 82 et 145) visent l'actuel §10 : corriger.
6. **Hors périmètre** : `CLAUDE.md` (diff appliqué par l'orchestrateur au merge), la section
   « Système IA (En Refonte) » et les autres points de fond laissés à William, `docs/taches.md`,
   tout changement de code autre que des commentaires.

## Critères d'acceptation

1. `git diff` ne touche que : commentaires Rust (`//`, `///`, `//!`), commentaires RON et en-tête
   de `powerups.ron`, `Makefile` (commentaire), `scripts/nightly.sh`, `.github/workflows/nightly.yaml`
   (si nécessaire), `docs/conventions.md` (§6, §9, Notes essentielles), `docs/taches/README.md`,
   le rapport.
2. `bash -n scripts/nightly.sh` ; `cargo check -p game -p run -p sim_core --profile headless`
   (CARGO_BUILD_JOBS=2, **après le feu vert d'orch** : la machine est prise par les 200 graines puis
   par une vérification jusqu'en début d'après-midi) ; `make lint GAME=throne` seulement si
   `powerups.ron` a été touché (même règle de feu vert).
3. Rapport `docs/taches/rapports/m1-dettes-doc-lot-2.md` : liste des fichiers, le diff de
   `nightly.sh`, et la confirmation que `cargo check` passe (ou « non lancé, pas de feu vert » si
   c'est le cas au moment de livrer, auquel cas orch le lance).

## Livrer

Merger `origin/main` juste avant de livrer. `git push -u origin m1-dettes-doc-lot-2`, puis
`SendMessage` à `orch` : `LIVRÉ m1-dettes-doc-lot-2 <sha> : <une ligne>`. Ne merge pas, ne bénis pas.
