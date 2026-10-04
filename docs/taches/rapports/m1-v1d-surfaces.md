# Rapport — m1-v1d-surfaces : surfaces v1 (T1.7, chantier E4 v1)

Fiche : [m1-v1d-surfaces](../m1-v1d-surfaces.md). Branche `m1-v1d-surfaces` depuis `53e3acb`
(LIVRÉ T1.6) ; worktree `alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : b1.

## État en cours

- **Fait** : branche créée, fiche lue.
- **En cours** : exploration (mouvement joueur/ennemi, stats, chargement des couches LDtk).
- **Prochaines étapes** : `SurfaceGrid` (world), kind `Surface` + lint, lecture couche
  `Surfaces` (chargement + Floors), système Input, `CellState.surface`, carte
  `testbed/surfaces.ldtk`, 4 scénarios, §26, vérifs, rapport, LIVRÉ.
- **Idée pour `surface_enemy`** : `NoDamageBetween` + `Event(hit)` (pas d'attente
  `EnemyDistance`).
