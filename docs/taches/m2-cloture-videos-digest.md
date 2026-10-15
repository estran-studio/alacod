# m2-cloture-videos-digest — vidéos et digest de sortie de M2

Lire d'abord `docs/taches/README.md`, `docs/taches/m1-cloture-videos-digest.md` (le modèle),
`docs/taches/rapports/revue-sortie-m2.md`, `docs/taches/session-test-manuelle.md`. Branche
`m2-cloture-videos-digest` depuis `origin/main`, **en dernier** (après `m2-full-run`). Estimation : 1 j.

## À faire

1. Vidéos (`make videos`, `make views`) de `gungeon_full_run`, `gungeon_boss`, `gungeon_squad`,
   `shop_tour`, `dash_jump_bullets`, `blank_*` ; copier celles de moins de 5 Mo dans
   `docs/digests/videos/` avec leurs `events.json`.
2. Digest `docs/digests/m2-fin-de-jalon.md` : chiffres de la campagne (graines, outcomes, journal),
   tableau « ce que le clone sait faire », critères de sortie un par un (reprendre le tableau du
   rapport de revue), « Pour la revue humaine » : 5 à 8 points à regarder en jouant
   (`cargo run -p gungeon`).
3. Notes de revue : créer `tests/review-notes/<commit>.md` pour le carnet de William.

## Session cloud

Rendu sans GPU : Mesa lavapipe si disponible, sinon laisser les vidéos à l'orchestrateur et livrer le
digest avec les commandes exactes. Finir par `LIVRÉ m2-cloture-videos-digest`.
