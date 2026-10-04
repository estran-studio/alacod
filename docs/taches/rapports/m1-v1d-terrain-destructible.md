# Rapport — m1-v1d : terrain destructible et cavernes (T1.0b + T1.6)

Fiche : [m1-v1d-terrain-destructible](../m1-v1d-terrain-destructible.md). Branche
`m1-v1d-terrain-destructible` (depuis `d3d25f3`, tête livrée de m0-v7 phase 2), worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1).

## État en cours

- **Fait** : crate `world` (CellGrid, cave::generate, points d'intérêt sur cases dégagées,
  destroy_terrain ; 1000 graines testées), set `World`, kind `Cave` + lint, `cave:<id>` par le
  chemin LDtk (gabarit + `MapGenerationMode::Cave`), CellGrid et files neutres au checksum,
  destruction → murs recréés + `FlowFieldCache::reload_walls` (synctest sans désync),
  `Action::DestroyTerrain` (lint), attente `CellState`, caverne en `Floors`, `cave:bench`
  (six follower qui naviguent), rendu minimal, §21.
- **Chiffres** : 82/82 traces identiques à `d3d25f3` en BLESS (avant DestroyTerrain, à
  refaire) ; tests `world` 8, `map_ldtk` lib 4, `scenario --test cave` 5, tous verts.
- **En attente** : annonce « T1.2 mergée » d'orch avant de toucher `projectile.rs`.
- **Prochaines étapes** : `ProjectileWallHit` + `ExpireAction::DestroyTerrain` + branchement
  `on_hit`/`on_expire` → `DestroyTerrainRequest` ; scénarios `explode_wall` et `bench_cave`
  (≥ 50 destructions, budget) ; merge `origin/main` (§24 bots, §21 cavernes) ; suite complète
  (traces inchangées), crates, lint, fmt, scripts, gen ; rapport final ; push ; LIVRÉ.
