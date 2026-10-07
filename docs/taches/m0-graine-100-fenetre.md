# m0-graine-100-fenetre — zombie figé devant une fenêtre intacte (test_map, graine 100)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche `m0-graine-100-fenetre`
créée depuis `origin/main`). Diagnostic puis correctif, 1 j. Compilation et dumps après le **feu vert**
d'orch (mémoire partagée), `CARGO_BUILD_JOBS=2`, un dump à la fois.

## Contexte

Critère M0 rejoué sur `exemples/test_map.ldtk` (b0, 4 `acheteur`, binaire `alacod-sim-f80b82b` dans
`alacod_tasks/m1-200-throne/`) : 164/165 graines atteignent la vague 5 ; **la graine 100** finit en
soft-lock à la vague 1 (5 kills sur 6, aucun kill après f790, plafond 20 000). Relevé D42 de b0 :

- le dernier zombie (`zombie_full`, 50 PV) est en case (26, 25), **dehors, collé à une fenêtre
  intacte**, dans son champ de flux (path_cost 484), état `Chasing`, case suivante (25, 25), pas bloqué
  par un mur ; **immobile au millième de f19 400 à f20 000, sans frapper la fenêtre** ;
- les quatre acheteurs sont dans une salle fermée en (13, 43), à 364 px, sans le voir, réserves pleines,
  immobiles eux aussi.

JSON et log : `alacod_tasks/m0-200-test-map/` (b0). Le binaire `f80b82b` précède D48/D51 ; rejouer
d'abord avec lui (`--seeds 100..100 --map exemples/test_map.ldtk --bots 4 --until-wave 5
--max-frames 20000 --save-scenario`), puis avec un binaire de `main` pour savoir si c'est toujours vrai.

## Décisions (fixées ici)

1. **Diagnostic d'abord**, deux questions séparées, chacune avec la frame et la preuve :
   - **moteur** : pourquoi le zombie ne frappe-t-il pas la fenêtre ? (règle `enemy_target_selection` :
     fenêtre « sur leur chemin, case actuelle ou 3 suivantes, et à portée », `enemy_attack_system`,
     cooldown) ; dater l'instant où il s'arrête et l'état de ses cibles ;
   - **bots** : pourquoi quatre acheteurs restent-ils immobiles dans une salle sans ennemi visible ?
     (règle d'acheteur ; m1-v3-bots-softlocks n'a traité que `prudent`/`fonceur` en `Floors`).
2. **Correctifs** : moteur dans `crates/game` (IA des fenêtres) si c'est là, bots dans `crates/bots` ;
   les deux peuvent être livrés ensemble, mais chacun avec sa preuve. Si un des deux n'est pas fautif,
   le dire et ne rien changer de ce côté.
3. **Traces** : `zombies` est le clone M0 : toute trace `clone_*`/`equilibrage_*`/`bots_four_mixed` qui
   bouge exige la preuve du §10 et une justification (c'est un comportement de M0 qui change) ;
   `clone_quad` doit garder ses attentes M0. Un scénario figé `test_map_fenetre_graine_100` (depuis
   `--save-scenario`, réduit) qui prouve la sortie du soft-lock.
4. **Mesure** : la graine 100 seule, puis 20 graines `test_map` (1..20) avant/après à 4 acheteurs, une sim
   à la fois.

## Livrer

Suite complète, crates, lint ×3, fmt, scripts, gen ×3, exemples ; rapport
`docs/taches/rapports/m0-graine-100-fenetre.md`. Merger `origin/main` juste avant ; `git push -u origin
m0-graine-100-fenetre` ; `SendMessage` à `orch [f1df5b]` : `LIVRÉ m0-graine-100-fenetre <sha> : <cause
moteur / bots, correctif, traces qui bougent>`.
