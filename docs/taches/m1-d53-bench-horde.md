# m1-d53-bench-horde — `bench_horde` a perdu la moitié de sa vitesse pendant M1 (D53)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche `m1-d53-bench-horde`
créée depuis `origin/main`). Performance, 1 j. **Priorité sur m1-cloture-videos-digest** (le bench est un
critère de sortie de M1). Compilations après le **feu vert** d'orch, `CARGO_BUILD_JOBS=2`.

## Constat (orch, 2026-10-06, `main` `a8b813e`)

`ALACOD_BENCH_STRICT=1 SCENARIO=bench_horde make test_scenarios`, machine calme (aucune sim, charge 2 sur
12 cœurs) : **35,6 / 36,2 fps** (900 frames, 272 entités, 61 ennemis, 0 balle) — **sous le plancher de 38**
de `tests/budgets.ron`. `bench_bullets` (99 à 117) et `bench_cave` (161 à 167) sont dans leur budget.
Historique (journal `docs/taches.md` §10) : **66,7 à 81,5 fps au calme pendant M0** ; 42,3 sous charge le
2026-10-04 (D39) ; aucun bench strict au calme depuis T1.6. La perte (÷ 2) est donc arrivée pendant M1 et
touche ce que fait une horde de zombies sans balles : déplacement, navigation, behaviors, séparation.

Candidats, à vérifier et non à présumer : T1.4 (zombies réécrits en behaviors RON), T1.6 (`CellGrid`,
flow field incrémental), D41/D38 (`NavKey` Small/Large, champs multiples), D48 (`world::nav` appelé par
closure dans `is_blocked_for`), m1-v1c variantes/statuts (composants posés sur chaque ennemi).

## Décisions (fixées ici)

1. **Bissection** sur `git log --first-parent main` entre le dernier bench calme connu (fin de M0, journal)
   et `a8b813e` : à chaque étape, construire le binaire des scénarios du commit **dans le même target**
   (worktree temporaire, `CARGO_TARGET_DIR` partagé, incrémental) et jouer `bench_horde` deux fois ; noter
   fps et charge. Les commits de doc seule se sautent. Résultat : le ou les commits où le fps chute.
2. **Profil** du coupable : `perf record`/`cargo flamegraph` (ou compteurs de temps par système si
   `perf` n'est pas utilisable), sur `bench_horde` au commit coupable et à son parent : quel système a grossi.
3. **Correctif** si c'est un coût évitable (recalcul inutile, allocation par frame, clé de cache
   instable…), **sans changer la simulation** : traces identiques exigées (`make test_scenarios` sans
   bless), sinon preuve du §10 et accord d'orch avant. Si le coût est inhérent (plus de travail voulu),
   le dire avec les chiffres et proposer le nouveau plancher : ne pas le baisser soi-même.
4. Mesure finale au calme : `bench_horde`, `bench_bullets`, `bench_cave`, `bots_four_mixed`, deux passages
   chacun, avant/après, dans le rapport.

## Livrer

Suite complète sans bless, crates, lint ×3, fmt, scripts, exemples ; rapport
`docs/taches/rapports/m1-d53-bench-horde.md` (bissection, profil, correctif, mesures). Merger
`origin/main` juste avant ; `git push -u origin m1-d53-bench-horde` ; `SendMessage` à `orch [f1df5b]` :
`LIVRÉ m1-d53-bench-horde <sha> : <commit coupable, cause, fps avant → après>`.
