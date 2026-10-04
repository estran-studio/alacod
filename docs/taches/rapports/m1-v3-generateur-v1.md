# Rapport m1-v3-generateur-v1 — générateur v1 et placement scripté de personnage (T1.13, V3)

**Branche** `m1-v3-generateur-v1`, partie de la tête livrée de T1.9 `e0c9fd6` (même worktree,
même target). Fiche : `docs/taches/m1-v3-generateur-v1.md`.

## État en cours

- **Fait** : branche créée, target purgé (13 Go, /home 73 Go libres), fiche lue, décisions
  confirmées à orch avec trois amendements (placement via le chemin des `CharacterSpawn` de carte
  dans `EnemySpawning` ; ennemi placé relativement au spawn du joueur trouvé par sonde ; ennemis
  zombies joués dans `zombies`/`exemples/test_map.ldtk` sans vague).
- **En cours** : `Scenario.characters` et système de placement.
- **Prochaines étapes** : `CharacterTest` + lint, gabarits, contenu `test:`, génération, §28,
  suite, rapport, LIVRÉ.
