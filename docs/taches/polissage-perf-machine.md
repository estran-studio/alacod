# polissage-perf-machine — mesure sur le Mac de William, plafonds, D70 (B8)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 7),
`CLAUDE.md` (« Coût du rollback »), `tests/budgets.ron`, `docs/taches/dettes.md` (D70, D84, D97, D118, D122),
`docs/taches/rapports/m3-salles-murs-isaac.md` (§ perf). Branche `polissage-perf-machine` depuis `origin/main`.
Estimation : 2 j. Clones : les quatre ; priorité isaac. Machine : **Mac au calme** (peu de charge, le dire) ; pas de VPS.

## Contexte (mesuré, `latest.json` du commit `0d2c1b61`, headless VPS partagé, inventaire §7)

`isaac_quad` 10,1 fps simulées / pire rollback 50,4 ms (sans plancher ni plafond) ; `isaac_duo` 20,5 fps / 34,4 ms (sans
budget) ; `bench_floor_isaac` 22,7 fps, sous son plancher de 26 (D122) ; `gungeon_full_run_four` 27,2 fps / 31,7 ms ;
tout le reste sous budget. Aucune mesure sur la machine de jeu.

## À faire

1. `make bench` (strict) sur le Mac au calme, deux passes entrelacées avec un commit de référence si possible (D84) :
   tableau par scénario, par clone ; les chiffres du VPS ne remplacent pas les siens.
2. Ajouter des planchers `min_sim_fps` et plafonds `max_rollback_ms` à `isaac_duo`, `isaac_quad`, `isaac_full_run`
   (mesure Mac, marge de bruit dite).
3. Profil `ALACOD_SYSTEM_PROFILE=1` sur `isaac_quad` : où passe la frame (champ de flux D70/D122, `Projectiles` D97).
4. Si `isaac_quad` dépasse 2 ticks (33 ms) de pire rollback sur le Mac : première optimisation **à trace identique**
   (champ de flux reconstruit seulement quand la case d'un joueur change *ou* un blocage change : déjà le cas ; viser le
   coût par reconstruction, p. ex. recouvrement partiel). Preuve §5 : `schedule_order_diff` vide, toutes traces identiques.
   Sinon, ne rien optimiser : inscrire en P9.
5. Jouabilité : une partie réelle d'isaac à 4 joueurs locaux (affichage) : noter les à-coups visibles.

## Acceptation

- Tableau Mac par clone dans `docs/taches/rapports/polissage-perf-machine.md`, machine et charge nommées.
- Budgets ajoutés pour les scénarios isaac listés ; `make bench` strict vert sur le Mac.
- Toutes les traces inchangées.
