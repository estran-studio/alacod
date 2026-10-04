# Rapport — m1-v1a-statuts : statuts Burn/Slow/Stun/Freeze (T1.3, voie V1a)

Branche `m1-v1a-statuts` depuis `65a4d25` (LIVRÉ m1-v3-bots-pathfinding) ; reprise de zéro, en
local (la session cloud c3 n'avait jamais livré). Fiche : [m1-v1a-statuts](../m1-v1a-statuts.md),
amendée (décisions envoyées à l'orchestrateur et acceptées). Worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date : 2026-10-04.

## État en cours

- **Écrit, pas encore compilé** : la compilation a été tuée par Claude Code (pression mémoire) ;
  William fait supprimer tous les targets ; aucune compilation avant le « feu vert » de
  l'orchestrateur. Code : logique pure et tests (`combat::status`), `Statuses` en checksum
  neutre, `Action::ApplyStatus`, pose par `on_hit`, ticks de `Burn` en `DamageEvent`, `Slow` en
  modificateurs, `Stun`/`Freeze` (joueurs : inputs ignorés dans la simulation ; ennemis : règle,
  déplacement, attaque, émetteur), `status_motion_system`, kind `Status` (registre, lint,
  fixtures), `StatusLibrary`, attentes `HasStatus`/`StatusStacks`, contenu testbed (4 statuts,
  4 armes `status_*`), teinte dérivée, §19, `CLAUDE.md`.
- **Reste** : compiler, tests, calibrer les `test:` des 4 armes (frames estimées), deux
  scénarios écrits (stun/freeze sur un ennemi qui avance, slow mesuré), tests unitaires des
  attentes, suite complète, preuve des traces, rapport, LIVRÉ.
