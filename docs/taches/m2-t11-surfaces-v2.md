# m2-t11-surfaces-v2 — surfaces v2 : fosses, tables renversables, barils explosifs (M2, V1d, chantier E4 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §26 (surfaces v1 : grille,
table, `intgrid_value`, modificateurs), §21 (cavernes, `DestroyTerrain`, `CellState`), §35 (salles),
§16 (projectiles composables, `on_expire`), et `CLAUDE.md` « Obstacles génériques » (`Obstacle`,
`ObstacleType::Pit`, `NavProfile::Flying`). Branche `m2-t11-surfaces-v2` depuis `main`.
Estimation : 2 j. Section de conventions réservée : **§46**.

## À faire

1. **Fosses** : surface `pit` (donnée : `fall_damage`, `respawn`) : un joueur qui y entre (hors dash
   en cours si la donnée l'autorise, `NavProfile::Flying` jamais) perd `fall_damage` PV et revient à
   la dernière case sûre (mémorisée dans un composant rollback **neutre**, mise à jour à chaque
   frame avant tout retour anticipé). Les ennemies `Ground` ne la traversent pas (flow field : case
   bloquée pour `Ground`/`GroundBreaker`, ouverte pour `Flying`).
2. **Tables renversables** : entité LDtk `Table` (obstacle `Barricade`) : Interaction à côté →
   renversée dans la direction opposée au joueur, bloque alors les balles (couverture) mais plus le
   passage dans son axe ; détruite après `n` balles (donnée). État rollback neutre, `GgrsNetId` trié
   par clé de contenu (règle « Numérotation des entités »).
3. **Barils explosifs** : entité `Barrel` (PV, rayon, dégâts, en donnée) : à 0 PV, explosion (même
   mécanique de dégâts de zone que les projectiles `on_expire`), qui peut déclencher un baril voisin
   à la frame suivante (chaîne déterministe, ordre `GgrsNetId`), `despawn_rollback()`.
4. **Lint** : fosse sans `fall_damage`, baril à rayon ≤ 0, table à 0 balle → refusés ; fixtures.
5. **Contenu** : `testbed` (une salle d'essai avec fosse, tables, barils) ; `gungeon` : barils et
   tables dans deux gabarits de salles (`games/gungeon/assets/rooms`), si la grammaire reste valide
   sur 1 000 graines.
6. **Scénarios** : `pit_fall` (`Health`, `PlayerPosition` au bord), `pit_dash` (traversée en dash
   si autorisée), `pit_enemy_path` (un zombie contourne, `EnemyNeverInWall`), `table_flip_cover`
   (`HitsAtLeast` 0 derrière la table), `barrel_chain` (deux barils, `EntityCount`, dégâts au joueur
   proche).

## Règles

Aucune trace existante ne doit bouger hors des cartes modifiées (nouveaux types en variante
**neutre**, variantes d'enum en fin) ; les traces `gungeon` qui bougent : preuve §5
(`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`), cause dans le commit, avant tout bless. Pas
de `f32`, `order_iter!`, `FrameEvents` (jamais `Message`). `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Rendu des tables/barils au-delà d'un sprite (V4), surfaces animées, eau profonde.

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ
toutes les deux heures et tue les processus d'arrière-plan : lancer la suite au premier plan) et finir par un commit
`LIVRÉ m2-t11-surfaces-v2` qui contient le rapport. p2p : `make test_multiplayer` si docker et un
affichage sont disponibles, sinon le dire au rapport.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ; rapport
`docs/taches/rapports/m2-t11-surfaces-v2.md`, conventions §46.
