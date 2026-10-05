# m1-v3-bots-softlocks — les soft-locks des bots sur `throne` (200 graines, critère §9.8)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche
`m1-v3-bots-softlocks` créée depuis la tête livrée de `m1-v3-bots-reanimation` `2c1cb21`, même
target). Voie V3 bots, 2 j. Calcul et compilation **après le feu vert** d'orch (machine prise
par les 200 graines puis par une vérification groupée jusqu'en début d'après-midi),
`CARGO_BUILD_JOBS=2`.

## Contexte

Critère de sortie de M1 (`docs/plan-engine.md` §9.8) : « les bots finissent le clone sur 200
graines **sans soft-lock** ni desync ». Sur `main` `61ac539` (bots-portail inclus, réanimation
pas encore mergée), 200 graines `throne` à 2 bots `prudent` : 0 desync, mais des **soft-locks**
(9 sur les 159 premières graines, `fin soft-lock` dans les logs : 1 200 frames sans kill ni étage,
`FloorsProgress`), presque toujours à l'étage 1 ou 2, et des défaites (hors critère, analysées
par b0 dans m1-analyse-200-throne : **ne pas** y travailler ici). Un soft-lock = les bots ne
trouvent plus l'ennemi restant (ou le portail) : c'est un défaut de bots, sauf preuve du
contraire (caverne fermée, ennemi inatteignable : à documenter comme dette, pas à corriger ici).

## Données

`/home/wq/Project/bascanada/alacod_tasks/m1-200-throne/` : `throne-2-*.json` / `.log` (puis
`throne-4-*`), `alacod-sim-61ac539` (binaire exact). Pour chaque soft-lock, le relevé D42 (ennemi
restant : personnage, case, PV, joueur le plus proche, distance, ligne de vue, « chemins non
calculés ») et `players_end` (D36). Orch dit « données prêtes » quand les JSON sont là.

## Décisions (fixées ici)

1. **D'abord la liste** : chaque soft-lock des 200 graines (2 bots, puis 4) dans une table :
   graine, étage, frame, ennemi(s) restant(s) et leur état D42, où sont les bots, ce qu'ils
   font (rejouer la graine avec le binaire fourni et `--save-scenario` pour les attentes), cause
   en un mot. Les causes se regroupent en deux ou trois familles au plus ; les corriger dans
   l'ordre de l'effectif.
2. **Correctifs dans `crates/bots` seulement** (`decide.rs`, `view.rs`, `input.rs`, navigation
   T1.14/m1-v3-bots-pathfinding) : par exemple aller chercher un ennemi immobile hors de vue par
   le champ de flux, abandonner une cible inatteignable pour une autre, ne pas rester bloqué
   derrière un coin de roche (marge de tir 4 px), prendre le portail quand `EntityCount(enemy)
   == 0`. Aucun changement de contenu, d'engine ni de règles de jeu ; si un soft-lock vient de
   l'engine (ennemi dans la roche, portail inaccessible, champ non calculé), le noter en dette
   proposée dans le rapport avec la graine, sans le corriger.
3. **Preuve par inputs** (conventions §24, §10) pour chaque trace qui bouge : première frame
   d'input différent, pourquoi, et l'effet. `clone_quad` et les scénarios M0 (`bots_four_mixed`,
   `equilibrage_*`) ne doivent pas bouger ; si un scénario throne à bots bouge, preuve.
4. **Mesure avant/après** sur les graines en soft-lock (toutes) et sur 20 graines témoin
   (1..20, 2 bots et 4 bots) avec un binaire construit depuis ta branche : table dans le rapport
   (étage 3, défaites, soft-locks, desync). Le but : **0 soft-lock** sur les graines listées
   sans augmenter les défaites. Les 200 graines complètes seront rejouées par l'orchestrateur
   après merge.
5. **Un scénario figé** par famille de cause (au plus 2) dans `games/throne/assets/scenarios/`,
   `throne_softlock_<cause>.ron`, de préférence construit depuis `--save-scenario` d'une graine
   et réduit (`max_frames`), avec une attente qui prouve la sortie du soft-lock après correctif
   (`FloorIndex`, `EntityCount(enemy) == 0`, `Event`…). Traces nouvelles : bless orchestrateur.
6. **Hors périmètre** : les défaites (b0), le contenu `throne`, la difficulté, D4 boss, 4 bots
   au-delà de la mesure témoin.

## Critères d'acceptation

1. Tests unitaires `crates/bots` verts, un par règle ajoutée.
2. Suite complète : tout vert ; traces qui bougent listées avec preuve par inputs ; zombies et
   testbed intacts.
3. Mesure avant/après (décision 4) dans le rapport ; 0 soft-lock sur les graines listées.
4. `make lint` des trois jeux, `make fmt`, `make gen GAME=throne` sans modification hors les
   nouveaux scénarios.
5. Rapport `docs/taches/rapports/m1-v3-bots-softlocks.md` (README §7), point d'état, purge du
   target après chaque suite.

## Livrer

Merger `origin/main` juste avant de livrer (conflits : garder les deux). `git push -u origin
m1-v3-bots-softlocks`, puis `SendMessage` à `orch` : `LIVRÉ m1-v3-bots-softlocks <sha> :
<soft-locks avant → après, traces qui bougent>`. Ne merge pas, ne bénis pas.
