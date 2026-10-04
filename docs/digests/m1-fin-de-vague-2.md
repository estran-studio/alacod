# M1 `throne` — digest de fin de vague 2 (brouillon)

**Brouillon** écrit par m1-integration-scenarios (agent local b0), sur la branche
`m1-integration-scenarios` (base `main` après T1.17). Chiffres mesurés dans cette session sur la
machine de William, sauf mention « journal » (mesures de l'orchestrateur, `docs/taches.md` §10).
La ligne « 200 graines » et la revue humaine restent à remplir par l'orchestrateur et William.

## En bref

- Le clone joue la run complète de `throne` : trois cavernes générées par graine, portail à
  l'étage vidé, boss `roi_rat` au troisième étage, rads, niveaux et mutations, horloge d'étage,
  butin par munition — à 1, 2 et 4 bots (`throne_solo`, `throne_duo` = `throne_three_floors`,
  `throne_quad`), joués en synctest.
- MESURES À VENIR : passages d'étage, 20 graines, vidéos.
- Ce qui reste pour sortir de M1 : 200 graines, revue humaine, fermeture des notes (voir « Ce qui
  manque »).

## Ce que le clone sait faire

Une ligne par chantier du plan (`docs/taches.md` §6) livré pendant M1, avec sa tâche (journal).

| Chantier | Tâche(s) | Ce que ça donne |
|---|---|---|
| Contrats combat et IA | T1.0a | armes déplacées dans `combat`, enums squelettes (`ProjectileModifier`, `Pattern`, `StatusDef`, `Behavior`…), crate `behaviors` |
| Contrats monde et run, cavernes | T1.0b, T1.6 | `world::CellGrid` (rollback), générateur de cavernes par automate cellulaire et graine, terrain destructible, flow field |
| Contenu `throne` | T1.0c, T1.11, m1-throne-gen-et-d40 | `games/throne` : un pilote, douze armes sur cinq munitions, dix ennemis, mutations, trois cavernes, butin par munition (D40) |
| B5 v1 projectiles composables | T1.1 | `Bounce`, `Pierce`, `Size`, `Lifetime`, `Homing`, `Gravity`, `on_hit`, `on_expire` ; explosions comme projectile à durée nulle |
| B5 v1 patterns et tir ennemi | T1.2 | `Emitter` rollback, `Aimed`/`Spread`/`Ring`/`Sequence`/`Telegraph`, attaque `Shoot(pattern)` |
| B3 statuts | T1.3 | `Burn`, `Slow`, `Stun`, `Freeze`, empilement, visuel dérivé |
| D1 behaviors composables | T1.4 | `Chase`, `KeepDistance`, `Strafe`, `Charge`, `Shoot`, `Melee`, `Flee`, `Wander` par priorité |
| D2 variantes et élites | T1.5 | `variants` tirées par flux, tags (`champion`, `rapide`) |
| E4 v1 surfaces | T1.7 | tags de cellules et modificateurs de vitesse |
| F1 mode `Floors` | T1.8 | étages dans le `GgrsSchedule`, portail, boucle infinie au dernier niveau |
| F2 horloges et difficulté | T1.9 | `Clock` d'étage et de run, difficulté par étage et par temps |
| C1 v1 / C4 v1 effets, jauges, mutations | T1.10 | `Effect { on, if, do }`, rads → niveau → mutation |
| Lint et attentes de M1 | T1.12, T1.15 | audit du lint des nouveaux kinds, attentes `BulletCount` … `CellState` |
| Générateur v1 | T1.13, m1-throne-gen-et-d40 | gabarits par ennemi (immobile, mobile), placement scripté ; `make gen GAME=throne` vert (37 scénarios) |
| Bots v1 | T1.14, m1-v3-bots-pathfinding | `prudent` esquive et navigue par le champ, `alacod-sim --until-floor` |
| I2 feedback v1 | T1.17 | hit stop, secousse, flash, télégraphe au sol, chiffres ; `FeedbackLog` (preuve sans écran) |
| Écran de mutation, HUD | T1.16, T1.18 | VOIR JOURNAL (en cours de merge à la rédaction) |
| Dettes | m1-dettes-lot-1, m1-d39 | D31, D33, D35, D37, D39 fermées ; D40 à moitié |
| Scénarios du clone, boss | m1-integration-scenarios | `throne_solo`, `throne_duo`, `throne_quad`, boss `roi_rat` |

## Les chiffres

### Tests

À REMPLIR (suite de la branche).

### `alacod-sim` : 20 graines, jusqu'au troisième étage

À REMPLIR.

| Graine | Étage | Frames | Morts | Fin | sim fps | Desync |
|---:|---:|---:|---:|---|---:|---|

### 200 graines

**À remplir par l'orchestrateur.**

## Vidéos

À REMPLIR.

## Ce qui manque pour M1

Critères de sortie du plan (`docs/plan-engine.md` §9.8) :

| Critère | État |
|---|---|
| Lint et tests verts sur `main` | À REMPLIR |
| Tous les scénarios du clone en synctest à 2 et à 4 | À REMPLIR |
| Les bots finissent le clone sur 200 graines sans softlock ni desync | **à remplir par l'orchestrateur** |
| Bench dans les budgets | À REMPLIR (journal : bench strict au calme non fait depuis T1.6) |
| Vidéos publiées | À REMPLIR |
| Doc des conventions à jour | À REMPLIR |
| Notes du jalon précédent fermées | À REMPLIR |

Autres manques : À REMPLIR (phases de boss D4 → M2, `grunt` sans attaque, restart p2p…).
