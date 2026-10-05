# M1 `throne` — digest de fin de vague 2 (brouillon)

**Brouillon** écrit par m1-integration-scenarios (agent local b0), sur la branche
`m1-integration-scenarios` (base `main` après T1.17). Chiffres mesurés dans cette session sur la
machine de William, sauf mention « journal » (mesures de l'orchestrateur, `docs/taches.md` §10).
La ligne « 200 graines » et la revue humaine restent à remplir par l'orchestrateur et William.

## En bref

- Le clone joue la run de `throne` : trois cavernes générées par graine (enfin trois cavernes
  **différentes**, voir plus bas), portail à l'étage vidé, boss `roi_rat` au troisième étage,
  rads, niveaux et mutations, horloge d'étage, butin par munition. `throne_duo`
  (= `throne_three_floors`) et `throne_quad` finissent la run sur la graine 123456 ; `throne_solo`
  fixe l'état mesuré (le bot seul meurt au troisième étage). Les trois sont joués en synctest.
- **Correction d'un résultat antérieur** : jusqu'à cette vague, les trois étages de la run
  chargeaient tous `niveau_1` (défaut d'engine : un seul asset pour toutes les cavernes).
  **Les anciens 20/20 portaient sur trois fois `niveau_1`.** Corrigé (« une caverne = un
  asset »), avec un second bug caché (terrain et murs de deux cartes différentes aux étages 2
  et 3).
- Sur les vraies cavernes, les bots finissent **3/20 à deux et 11/20 à quatre** (0/20 seul),
  **0 desync** : ils ne vont pas chercher un ennemi qui ne vient pas à eux. Correctifs de bots en
  cours (b1, m1-v3-bots-portail).
- Ce qui reste pour sortir de M1 : bots (200 graines), revue humaine, fermeture des notes.

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
| Écran de mutation | T1.16 | choix à trois cartes, input scriptable (journal) |
| HUD throne | T1.18 | en cours chez b1 à la rédaction (journal) |
| Dettes | m1-dettes-lot-1, m1-d39 | D31, D33, D35, D37, D39 fermées ; D40 à moitié |
| Scénarios du clone, boss, une caverne = un asset | m1-integration-scenarios | `throne_solo`, `throne_duo`, `throne_quad`, boss `roi_rat` ; chaque caverne chargée depuis son propre asset (`cave://`) |

## Les chiffres

### Tests

Voir le rapport de m1-integration-scenarios (§6) : suite des crates, `scenarios`, lint des trois
jeux, `make gen` ; les traces `throne` changent toutes (étages 2 et 3 enfin joués, réserves de
départ) et sont à rebénir par l'orchestrateur avec la preuve du rapport.

### `alacod-sim` : 20 graines, jusqu'au troisième étage

```
cargo run -q -p scenario --profile headless --bin alacod-sim -- --game throne --bots N \
  --profiles prudent,… --floors run --seeds 1..20 --until-floor 3 --max-frames 15000
```

Joué sur la branche m1-integration-scenarios (état final : correctif cave://, boss 20 × 20,
butin relevé, réserves ×2, tourelle en dernier). « Fini » = troisième étage passé ; SL =
soft-lock détecté (1 200 frames sans kill ni étage). Résultats complets :
[`m1-fin-de-vague-2.sim.json`](m1-fin-de-vague-2.sim.json).

| Réglage | 1 bot | 2 bots | 4 bots |
|---|---|---|---|
| avant le correctif (journal : **trois fois `niveau_1`**) | — | 20/20 | — |
| correctif + boss 20 × 20 + butin relevé | 0/20 (7 SL, 13 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + réserves de départ ×2 | 0/20 (10 SL, 10 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + tourelle en dernier (**final**) | 0/20 (6 SL, 14 morts) | **3/20** (16 SL, 1 défaite) | **11/20** (9 SL) |

**Desync : 0** dans toutes les séries. Les soft-locks laissent presque toujours 1 ou 2 ennemis
sur les points les plus éloignés du départ (tourelle fixe, brute lente, tireurs) que le bot
`prudent` ne va pas chercher : limite des bots, pas des données (les réserves doublées ne
changent rien graine par graine). En solo, deux soft-locks au portail ouvert de l'étage 0
(graines 3 et 16) : défaut de bots aussi. sim fps moyen : 65 (1 bot), 52 (2), 40 (4).

### 200 graines

**À remplir par l'orchestrateur.**

## Vidéos

Rendues sur `146d09c` (`make videos SCENARIO=throne_solo,throne_three_floors,throne_quad`, puis
`make views SCENARIO=throne_quad`, 960 × 540, une image toutes les deux frames). Les vidéos de
moins de 5 Mo sont copiées dans `docs/digests/videos/` avec leurs moments clés :

| Vidéo | Durée | Taille | Source |
|---|---:|---:|---|
| [`throne_solo.mp4`](videos/throne_solo.mp4) | 61,7 s | 2,8 Mo | `target/videos/146d09c/throne_solo.mp4` |
| [`throne_three_floors.mp4`](videos/throne_three_floors.mp4) (duo) | 86,7 s | 4,3 Mo | `target/videos/146d09c/throne_three_floors.mp4` |
| [`throne_quad.mp4`](videos/throne_quad.mp4) | 56,7 s | 3,3 Mo | `target/videos/146d09c/throne_quad.mp4` |
| `montage.mp4` | — | 6,9 Mo | **lien seulement** (> 5 Mo) : `target/videos/146d09c/montage.mp4` |
| `throne_quad.vues.mp4` (quatre vues `--follow`) | 56,7 s | 14,3 Mo | **lien seulement** : `target/videos/146d09c/throne_quad.vues.mp4`, `make views SCENARIO=throne_quad` |

À voir en priorité : le troisième étage du quatuor (le roi des rats tire en couronne depuis le
coin le plus éloigné, mort vers f2558, nouveau boss au rechargement f3312) ; la mort du bot seul
dans la troisième caverne (f3640, cercles de télégraphe et chiffres de dégâts de T1.17) ; les
trois cavernes de tailles différentes du duo (48 × 32, 56 × 40, 64 × 44), qui étaient la même
avant le correctif.

## Ce qui manque pour M1

Critères de sortie du plan (`docs/plan-engine.md` §9.8) :

| Critère | État |
|---|---|
| Lint et tests verts sur `main` | **oui** jusqu'à `963f2d2` (journal) ; cette branche : lint des trois jeux vert, traces throne à rebénir (voir rapport m1-integration-scenarios) |
| Tous les scénarios du clone en synctest à 2 et à 4 | **oui** : `throne_three_floors` (2) et `throne_quad` (4) verts en synctest, plus `throne_solo` (1) |
| Les bots finissent le clone sur 200 graines sans softlock ni desync | **à remplir par l'orchestrateur** |
| Bench dans les budgets | **non vérifié** : bench strict au calme non fait depuis T1.6 (journal) |
| Vidéos publiées | **en partie** : générées, copiées dans le dépôt ; pas de publication automatique |
| Doc des conventions à jour | à jour pour chaque tâche livrée (§29 boss, §31 feedback…), sans relecture d'ensemble |
| Notes du jalon précédent fermées | **à confirmer par l’orchestrateur** (revue de M0, T3.3) ; la revue de M1 reste à faire |

**Bots** (le critère qui bloque) : 3/20 à deux et 11/20 à quatre sur les vraies cavernes, 0 desync.
Correctifs en cours chez b1 (portail du bot seul, cible fixe hors de vue, rapprochement d'un
ennemi immobile).

Autres manques :
- Phases de boss (D4) : reportées à M2 ; `roi_rat` n'a qu'une liste de behaviors.
- Navigation des grands agents (D41) : le boss a un corps de 20 px comme les autres.
- Diagnostic de soft-lock faux pour les ennemis `Ground` (D42).
- `grunt` du testbed sans attaque (reste de D40, décision de William).
- Restart p2p (reporté de M0, T1.x à porter).
- HUD throne (T1.18) : en cours à la rédaction.
