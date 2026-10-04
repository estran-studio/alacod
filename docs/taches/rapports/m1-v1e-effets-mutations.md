# Rapport — m1-v1e-effets-mutations : effets v1 et mutations (T1.10, C1 v1 et C4 v1)

Fiche : [m1-v1e-effets-mutations](../m1-v1e-effets-mutations.md). Branche
`m1-v1e-effets-mutations` depuis `4f52d2b` (LIVRÉ T1.14) ; worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : b1.

## État en cours

- **Fait** : étapes 1 à 3 (effets, progression et mutations, drop d'arme par niveau), sept
  scénarios, attentes `Gauge`/`Level`/`Mutations`, lint et fixtures, §27, `CLAUDE.md`.
- **Amendements** (en plus de ceux de départ) : système dans `DeathManagement` (accepté) ;
  progression **opt-in** (`entry.progression`, `Scenario::progression`) et plusieurs fichiers
  `progression/` (le testbed en a deux, `base` et `armes`) ; attentes en `handle` (comme
  `Currency`) ; `Mutations.count` ; `OnLevelUp` à la frame qui suit le choix.
- **Prochaine étape** : suite complète (scénarios sans bless, crates, lint, fmt, scripts, gen,
  exemples), purge, merge `origin/main`, LIVRÉ.
