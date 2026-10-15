# m4-t15-bench-larmes — bench de larmes et d'étage en grille (M4, V3, K3)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Coût du rollback »), `docs/etat-engine.md` (mesures du
2026-10-15, D70), `docs/taches.md` §8.1 (critère 6), `docs/conventions.md` §66, §71, et `tests/budgets.ron`.
Branche `m4-t15-bench-larmes` depuis `origin/main`, **après le merge de M4-T1 et M4-T6**. Estimation : 2 j.
Section de conventions réservée : **§80**.

## À faire

1. **`bench_tears`** (testbed) : quatre joueurs, dix objets de larmes chacun (division, guidage, rebond,
   perce), deux familiers chacun, horde de 30 ennemis ; au moins 400 projectiles vivants au pic.
2. **`bench_floor_isaac`** (après T5 et T11d) : étage en grille de 12 à 15 salles rempli, bot explorateur.
3. Mesures : `advance_ms_avg`, `save_ms_avg`, `load_ms_avg`, `rollback_ms_worst`, `snapshot_bytes_max`,
   `snapshot_heaviest` ; profil par set (`ALACOD_SYSTEM_PROFILE=1`) ; planchers et `max_rollback_ms` dans
   `tests/budgets.ron` à la moitié de la mesure (règle existante), machine nommée.
4. Si `rollback_ms_worst` dépasse 16,7 ms × la marge retenue : profiler (`Split` en chaîne, `Homing` sur la
   grille de collision, familiers) et ouvrir une dette chiffrée ; ne pas relever un seuil pour passer.

## Règles

Mesure seulement, sauf optimisation qui garde les traces identiques (preuve : suite sans `BLESS`).

## Vérification

`make bench`, `make perf_review` contre une référence copiée avant ; rapport
`docs/taches/rapports/m4-t15-bench-larmes.md`, conventions §80.

## Session cloud

Mesure indicative au cloud (machine non calme) ; la mesure de référence se fait sur `orca` ou le Mac au
calme : le dire au rapport. Pousser la branche régulièrement ; commit final `LIVRÉ m4-t15-bench-larmes`.
