# dettes-outillage-d15-d34-d60-d63 — mesures et outillage (lot de petites dettes)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches/dettes.md` (lignes D15, D34, D60,
D63), `docs/conventions.md` §5, §10 (preuve d'une trace) et §53 (atteignabilité). Branche
`dettes-outillage-d15-d34-d60-d63` depuis `origin/main`. Estimation : 1,5 j. Un commit par dette.

## À faire

1. **D15, grille de collision mesurée** : `bench_bullets` et `bench_horde` avec et sans la grille
   spatiale de `crates/combat` (un interrupteur de test, ou le commit d'avant T2.1 rejoué dans un
   worktree). Ratio au rapport et dans la ligne D15. Aucun changement de code livré si l'interrupteur
   n'est pas nécessaire ensuite.
2. **D34, CORDIC près de 2π** : mesurer l'écart de `fixed_math` sur ]−4π, 4π] (pas fin), et lister
   les appels de la simulation qui reçoivent des angles hors de ]−π, π]. Si l'écart touche du contenu
   réel (un tir, un pattern), replier dans `direction_of` avec la preuve §5 et la cause dans le
   commit. Sinon, un test qui borne l'écart et la dette fermée par la mesure.
3. **D60, `make compare_video` et scénarios générés** : `scripts/scenario-video` cherche aussi dans
   `tests/scenarios/generated/*/` (nom ambigu entre deux jeux : erreur qui liste les chemins, et
   `jeu/nom` accepté) ; message d'erreur explicite quand le GPU (Vulkan) manque. `DRY_RUN=1` prouve la
   résolution sans GPU.
4. **D63, contenu de test** : un marqueur `test_only: true` (optionnel, défaut `false`) sur les
   personnages et les armes ; l'analyse d'atteignabilité de M2-T15 (`content::reach`) ignore le
   contenu marqué. Marquer `cible`, `mannequin` (throne), `cible` (gungeon). Pour `zombie_scaled`
   (zombies) et l'arme `arsenal` (throne, gungeon) : dire au rapport à quoi ils servent (grep des
   scénarios et du contenu) et proposer marqueur ou placement, sans placer de contenu. Fixture de lint.
5. **Question ouverte de M2-T17** : un `UseActive` scripté en `from: 10, to: 11` n'agit qu'à la frame
   16 (6 frames), alors que `Fire` répond sans délai observé. Trouver la cause (délai d'input GGRS,
   anti-rebond de l'actif, ordre des systèmes) et l'écrire au rapport ; corriger seulement si c'est un
   défaut, avec preuve.

## Règles

Aucune trace ne bouge, sauf D34 si le repli est justifié (preuve §5). Pas de `f32` dans la
simulation. `cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud : ignorer les consignes `orca` du README (§1) ; pas
de GPU ni de Docker (D60 se prouve par `DRY_RUN=1` ; p2p non requis sauf si D34 change la
simulation, alors à rejouer par l'orchestrateur). Pousser la branche régulièrement (commits WIP
acceptés). Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le
commit `LIVRÉ dettes-outillage-d15-d34-d60-d63`, avec le rapport
`docs/taches/rapports/dettes-outillage-d15-d34-d60-d63.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4).
