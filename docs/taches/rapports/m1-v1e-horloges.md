# Rapport m1-v1e-horloges — horloges d'étage et de run, difficulté (T1.9, F2)

**Branche** `m1-v1e-horloges`, partie de la tête livrée de T1.5 `148a990` (contient T1.4 et
T1.5, pas encore dans main), même worktree et même target. Fiche :
`docs/taches/m1-v1e-horloges.md`.

## État en cours

- **Fait** : branche créée, target purgé (15 Go, /home 67 Go libres), fiche lue, décisions
  confirmées à orch avec amendements (file `FrameEvents` neutres ; mise à jour de `Clock` et
  `FloorEntered` seulement si horloges/difficulté activées → aucune trace existante ne change ;
  `deux_niveaux` pour `clock_floor_reset` ; dégâts ennemis × difficulté dans le résolveur unique ;
  santé × difficulté chez les appelants de `spawn_enemy` ; `DifficultyConfig` hors rollback ;
  `ClocksOverride`/`DifficultyOverride` ; séquence `deux_cibles` sur 2 cartes nouvelles).
- **En cours** : implémentation (`run::Clock`, kinds `Clock`/`Difficulty`, `clock_system`).
- **Prochaines étapes** : lint + fixtures, attente `Clock` + événement `clock`, contenu testbed,
  3 scénarios, §23 + phrase §18, D29, suite, rapport, LIVRÉ.
