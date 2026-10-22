# polissage-perf-budgets — budgets de perf, natif et navigateur (S2-9)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 7), `CLAUDE.md` (« Coût du rollback »),
`tests/budgets.ron`, `docs/taches/polissage-perf-web.md` (S1-5 : mesures navigateur), dettes D70, D84, D97, D118, D122.
Branche `polissage-perf-budgets` depuis `origin/main`, après S1-5. Estimation : 2 j. Voie b1 (Mac, au calme).

## Contexte (inventaire §7, `latest.json` commit `0d2c1b61`, headless VPS partagé)

`isaac_quad` 10,1 fps sim / 50,4 ms au pire rollback ; `isaac_duo` 20,5 fps ; `bench_floor_isaac` 22,7 fps sous son plancher de
26 ; `isaac_duo`, `isaac_quad`, `isaac_full_run` sans plancher ni plafond ; zombies, throne, gungeon sous budget ; aucune mesure sur le Mac.

## À faire

1. `make bench` strict sur le Mac au calme (charge notée), tableau par clone ; ajouter planchers et plafonds aux scénarios isaac listés.
2. `ALACOD_SYSTEM_PROFILE=1` sur `isaac_quad` : champ de flux (D70/D122), `Projectiles` (D97).
3. Si le pire rollback d'`isaac_quad` dépasse 2 ticks (33 ms) sur le Mac : une optimisation **à trace identique** (preuve §5 :
   `schedule_order_diff` vide, toutes traces identiques) ; sinon P7.
4. Reporter les seuils navigateur de S1-5 dans le rapport.

## Acceptation

- Tableau Mac par clone, machine et charge nommées ; budgets ajoutés ; `make bench` strict vert au calme ; traces inchangées.
- Rapport `docs/taches/rapports/polissage-perf-budgets.md`.
