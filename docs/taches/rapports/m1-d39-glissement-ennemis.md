# Rapport m1-d39-glissement-ennemis — les ennemis glissent comme les joueurs (D39)

**Branche** `m1-d39-glissement-ennemis`, partie de la tête livrée de m1-dettes-lot-1 `ac42ebd`
(contient T1.9, T1.13 et le lot de dettes). Fiche : `docs/taches/m1-d39-glissement-ennemis.md`
(main local d'orch). Base `origin/main` : `4393db6`, mergé avant la vérification (sans conflit) : la branche = main + D39.

## État en cours

- **Fait** : correctif, tests, preuve, critère M0 (§1 à §4). Livré.
- **Target** purgé après la suite (target repartit de zéro après la purge générale d'orch ;
  12G, /home 85G libres). Dumps supprimés après résumé ; binaires de référence dans
  `alacod_tasks/<tâche>/d39/`.
- **Reste à l'orchestrateur** : bless de `enemy_kiter_still` (preuve §2), fermer D39, bench, p2p,
  merge.

## 1. Fait

- **Fonction partagée** `combat::collider::slide_axes(start, dx, dy, blocked) -> (pos,
  moved_x, moved_y)` (réexportée par `game::collider`) : X seul, puis Y **depuis la position X
  obtenue** (depuis le départ si X est bloqué). Utilisée par :
  - `move_enemies` (`crates/game/src/character/enemy/ai/pathing.rs`) à la place du bloc « Try X
    only / Try Y only (independent of X) » : c'est le correctif.
  - `move_characters` (`crates/game/src/character/player/input.rs`) à la place de son bloc
    équivalent (qui testait déjà Y depuis le X obtenu) : **même résultat bit pour bit** (même
    arithmétique ; `saturating_add` au lieu de `+`, identique hors débordement). L'« opening
    assist » des joueurs reste à part.
  - Seul chemin de déplacement des ennemis : `Charge` (ruée), `KeepDistance`/`Flee` (recul),
    `Strafe`, `Wander` produisent une direction (`BehaviorMotion`) appliquée par
    `move_enemies`. Les échappements (glissement prolongé, blocage complet) testent déjà des
    positions complètes : inchangés.
- **Tests unitaires** (`combat`, `slide_tests`) : coin de mur de `enemy_kiter_still` (X et Y
  libres séparément, diagonale bloquée → plus aucune pénétration, X seul appliqué) ; X bloqué →
  repli Y depuis le départ ; rien ne bloque → même résultat que deux axes indépendants.
- **`enemy_kiter_still`** : le `test:` du kiter revient aux attentes par défaut (`EnemyNeverInWall`
  de f2 à **f600**, plus la borne f540) ; scénario régénéré (`to: 540` → `to: 600`).
- `docs/conventions.md` §24 : une phrase (« les ennemis glissent comme les joueurs »).

## 2. Preuve §10

Méthode (accord d'orch) : la branche est mergée avec `origin/main` `4393db6` ; la référence est
**main lui-même** — ses traces bénies (dans le dépôt) et ses binaires (`scenarios_main`,
`alacod-sim_main`, construits par orch sur `4393db6`, copiés dans `d39/`, checkout de main gelé
pendant les runs). Toute trace différente sur la branche est donc l'effet de D39 seul.

- **Suite de la branche** : 138 scénarios joués, **une seule trace différente :
  `enemy_kiter_still`** ; les **137 autres identiques** à main (tous les scénarios zombies,
  testbed, throne, générés compris : aucun autre ennemi ne touchait un coin de mur dans un
  scénario existant).
- **`enemy_kiter_still`** (dumps `scenarios_main` contre branche, `trace-diff.py` + comparaison
  par entité sur toutes les frames) : première différence **f543**, kiter (NetId 36) :
  `FixedTransform3D` et `WallSlideTracker` — c'est le coin diagnostiqué. Main : (582,86, 655,88)
  dans le coin du mur (616, 632), puis rebond à (581,69, 656,51) ; branche : glisse le long du
  mur, y constant 656,29 (582,86 → 583,76 → 584,65 → 585,53). Seules autres différences : ses
  trois flèches (NetId 66, 67, 68) tirées à f596 depuis une autre position. **Hors de ces quatre
  entités, les 599 frames sont identiques.** `EnemyNeverInWall` de f2 à f600 : vert.

## 3. Critère M0 : 20 graines zombies

`alacod-sim --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile --seeds 1..20
--until-wave 5 --max-frames 20000`, base (`ac42ebd`) contre branche :

| graine | vague (main) | frames | morts | kills | vague (branche) | frames | morts | kills |
|---|---|---|---|---|---|---|---|---|
| 1 | 5 | 7975 | 2/4 | 51 | 5 | 7975 | 2/4 | 51 |
| 2 | 5 | 8706 | 3/4 | 52 | 5 | 8706 | 3/4 | 52 |
| 3 | 5 | 9384 | 3/4 | 52 | 5 | 9384 | 3/4 | 52 |
| 4 | 5 | 8478 | 3/4 | 51 | 5 | 8478 | 3/4 | 51 |
| 5 | 4 | 9776 | 4/4 | 47 | 4 | 9776 | 4/4 | 47 |
| 6 | 5 | 8111 | 3/4 | 51 | 5 | 8111 | 3/4 | 51 |
| 7 | 3 | 20000 | 3/4 | 32 | 3 | 20000 | 3/4 | 32 |
| 8 | 5 | 8836 | 2/4 | 49 | 5 | 8836 | 2/4 | 49 |
| 9 | 5 | 9381 | 2/4 | 54 | 5 | 9381 | 2/4 | 54 |
| 10 | 2 | 20000 | 0/4 | 18 | 2 | 20000 | 0/4 | 18 |
| 11 | 5 | 9744 | 2/4 | 53 | 5 | 9744 | 2/4 | 53 |
| 12 | 5 | 8312 | 3/4 | 52 | 5 | 8312 | 3/4 | 52 |
| 13 | 5 | 9360 | 3/4 | 54 | 5 | 9360 | 3/4 | 54 |
| 14 | 5 | 8550 | 3/4 | 52 | 5 | 8550 | 3/4 | 52 |
| 15 | 4 | 20000 | 3/4 | 50 | 4 | 20000 | 3/4 | 50 |
| 16 | 5 | 9350 | 3/4 | 51 | 5 | 9350 | 3/4 | 51 |
| 17 | 5 | 8809 | 3/4 | 55 | 5 | 8809 | 3/4 | 55 |
| 18 | 5 | 9578 | 3/4 | 54 | 5 | 9578 | 3/4 | 54 |
| 19 | 5 | 8179 | 2/4 | 51 | 5 | 8179 | 2/4 | 51 |
| 20 | 5 | 8513 | 3/4 | 50 | 5 | 8513 | 3/4 | 50 |

**Identique graine par graine** (vague, frames, morts, kills) ; **0 desync, 0 softlock** des deux
côtés. Vague 5 atteinte sur les mêmes 16 graines ; graine 5 : défaite en vague 4 ; graines 7, 10,
15 : plafond de 20 000 frames en vague 3, 2, 4 — **identique sur main** (préexistant, sans lien
avec D39 ; même résultat déjà à `ac42ebd`).

## 4. Vérifié

- **Tests des crates** (`scenario run combat game content map_ldtk map sim_core stats bots
  effects utils behaviors world`, `--include-ignored`) : 508 verts hors `scenarios` (§2), dont
  `slide_tests` 2/2. Seul autre échec : le doctest `rust,ignore` de `game::waves`, préexistant.
- **`make lint`** : zombies, testbed, throne sans erreur.
- **`make gen`** : zombies 16/16 et testbed 32/32 attentes `ok` ; seule trace « différente » :
  `enemy_kiter_still` (§2) ; aucun fichier généré modifié par la génération (le seul changement,
  `to: 540` → `to: 600`, est commité avec le correctif).
- **`make gen GAME=throne`** : **échoue, déjà sur main** — `games/throne/assets/game.ron` ne
  déclare pas encore `generate_template`, les gabarits jouent donc dans le testbed et l'arme
  `arsenal` y est inconnue (panique `PlayerScript::weapon`). Sans lien avec D39 (b1/throne).
- **`fmt`** propre ; **`check_forbidden`** 4 occurrences (identique) ;
  **`check_rollback_registration`** OK ; **exemples racine** compilés.

## 5. Écarts, non fait / incertain

- Référence de la preuve : main `4393db6` au lieu de ma base `ac42ebd` (accord d'orch, main
  contenant T1.10, throne et le pathfinding).
- Fonction partagée placée dans `combat::collider` (pas `sim_core`) : c'est là que vivent
  `Collider`/`is_colliding`, dépendance commune de `game`.
- `make gen GAME=throne` en échec, préexistant (§4).
- Bench strict et p2p : non faits (orch).
