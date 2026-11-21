# X-6g — corrections de revue (consigne du coordinateur, 21 nov 2026)

Tu reprends le lot **X-6g** (intérieur par graine) sur sa branche `wquintal/x-6g-interieur-graine` @ `3975cb5e`, dans le worktree `~/orca/workspaces/alacod/x-6g-interieur-graine`.
Brief d'origine (règles de la machine, communication, contrôle rapide découpé) : `docs/taches/specs-coordinateur/x-6g-interieur-graine.md` — il s'applique tel quel. Français partout.
Verdict du relecteur : **merge après corrections**, `docs/taches/specs-coordinateur/revue-x-6g-verdict.md` sur `origin/main` (copie : `~/orca/runs/alacod/revue-x-6g-verdict.md`). Lis-le.

## À corriger (rien d'autre)
1. **P2.1** : lint `exact` + type de remplissage aussi requis, en `Graph` (`crates/world/src/floor_grammar.rs` `problems()`, près de l. 298), comme le promet `docs/conventions.md` § 129 ; test.
2. **P2.2** : lint `apart` avec `start_exits: Some(n)` où n < 2 ; test.
3. **P3.2** : lint `apart` sans `exact` (le contrôle ne regarde que la première salle de chaque type) — ou contrôler toutes les paires ; choisis le plus simple, documente au § 129.
4. **P3.1** : le test de relance doit aussi prouver que `entry.default_seed` du manifeste du testbed est lu (sans `map_seed` explicite, ou assertion sur le manifeste) ; en build rendu, dire qu'il est sauté.
5. **P3.5** : aligner commentaire et attentes de `tests/scenarios/interieur_arrivee.ron` (commentaire seulement si les attentes sont justes ; trace inchangée).
6. **P3.4** : commentaire « à réécrire avec D227 » sur le test de `imp/basic.rs`.
7. Fusionne `origin/main` dans ta branche (`git merge origin/main`, pas de rebase) : conflit `propositions.md` = garder les deux côtés.
Les P3.3 et P3.6 vont dans le rapport, section « reste ».

## Preuve
Tests nouveaux rouges avant / verts après (dans le rapport) ; aucune trace modifiée ; contrôle rapide **découpé** vert sur la tête poussée
(`LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, puis `scripts/target-sweep.sh target`), journal copié en
`~/orca/runs/alacod/logs/x-6g-check-rapide-<sha>.log`. Rapport complété (section « Corrections de revue »). Disque : ~35 Go, plancher 30 : `df` avant le contrôle.
Fin : `status` « X-6g prêt <sha> » puis **un seul** `worker_done`.
