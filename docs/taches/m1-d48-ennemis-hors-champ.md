# m1-d48-ennemis-hors-champ — un ennemi n'est jamais hors de son propre champ de flux (D48) ; `alacod-sim --log` (D49)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-d48-ennemis-hors-champ` créée depuis `origin/main` à jour). Engine (V1c navigation) +
outillage, 1 j. Compilation : `CARGO_BUILD_JOBS=2`, **après le feu vert** d'orch (une
compilation lourde à la fois sur la machine).

## Contexte

Les rejeux des soft-locks des 200 graines `throne` (b1, m1-v3-bots-softlocks) montrent deux
ennemis **hors de leur propre champ de flux**, donc inatteignables par les bots et immobiles :

- graine 43, `roi_rat` (corps 20 × 20 × scale 1,4 = 28 px, `AgentSize::Large`) en case (57, 4)
  de `niveau_3` : la case est bloquée pour la clé `Large` (une voisine en 8-voisinage est un
  obstacle, `is_blocked_for`, `navigation.rs`), sur `main` comme avec les correctifs de bots ;
- graine 162, `brute` (collider 20 × 20, offset y −6, scale 1) en (9, 4)–(9, 5) de son étage,
  cachée à 41 px des bots, même symptôme.

Ce qui existe : D41 (m1-navigation-profils-tailles, m1-d41-spawns-degages) a introduit
`NavKey { profil, Small|Large }` (`SMALL_AGENT_MAX = 20` : au-delà, toute case voisine d'un
obstacle est bloquée) et `CaveConfig.spawn_clearance` dérivé par le registre du plus grand corps
de `characters` (`spawn_clearance_for` : 1 jusqu'à 24 px, 2 jusqu'à 40 px…), appliqué par
`points_of_interest(…, enemy_clearance)` / `world::is_open_within`. Les deux règles ne disent
pas la même chose : « dégagement » compte des cases libres autour du **point**, « Large »
exige que la case **et ses 8 voisines** soient libres — un corps de 21 à 24 px est `Large` avec
un dégagement de 1. Et un ennemi peut arriver hors champ autrement (poussé par une charge, un
recul, une porte, un terrain creusé puis… non : creuser ouvre), ce que la tâche doit établir.

## Décisions (fixées ici)

1. **Diagnostic d'abord**, avec les deux graines : rejouer (binaire `alacod-sim-f80b82b` de
   `alacod_tasks/m1-200-throne/`, `--save-scenario`, puis scénario avec `EnemyState`/`CellState`
   ou la trace détaillée **un rejeu à la fois**) et dire à quelle frame l'ennemi est entré dans
   une case bloquée pour sa clé : **à l'apparition** (point d'apparition mal dégagé) ou **après**
   (déplacement). Les deux cas se corrigent différemment ; ne corrige que ce qui est prouvé.
2. **Si c'est l'apparition** : une seule règle de dégagement, celle de la navigation. Un point
   d'apparition d'ennemi n'est retenu que si la case est **libre pour la clé de navigation du
   personnage** (`is_blocked_for(pos, NavKey::of(character))`) ; `spawn_clearance_for` reste pour
   les corps > 24 px. Lint : une caverne dont aucun point ne convient au plus grand corps de ses
   `characters` est refusée (fixture). Test unitaire : sur les trois cavernes de `throne` et
   1 000 graines, **chaque** point d'apparition d'ennemi est une case libre pour la clé de
   chaque personnage de la caverne (test dans `world` ou `game`, où vivent déjà ceux de D41).
3. **Si c'est un déplacement** : le système de déplacement des ennemis (`move_enemies`) ne doit
   jamais laisser un agent `Large` entrer dans une case bloquée pour sa clé (clamp du pas, comme
   pour les murs) ; test unitaire sur un pas qui traverserait une voisine d'obstacle.
4. **Traces** : si la correction change les points d'apparition, des traces `throne` bougent
   (toutes celles où un ennemi apparaissait sur un point désormais refusé) : liste exacte avec la
   preuve du §10 (premier `EnemyState`/position qui diffère, pourquoi). `zombies` et `testbed`
   ne doivent pas bouger (cartes LDtk sans caverne, ou caverne `petite` sans ennemi) ; si
   `explode_wall`/`bench_cave` bougent, preuve. Rien de béni par l'agent.
5. **D49, `alacod-sim --log`** : `RUST_LOG` ne produit rien (aucun subscriber). Installer le
   `tracing_subscriber` (ou `LogPlugin` headless, ce que fait déjà le runner de scénarios) quand
   `--log` est passé, sortie sur stderr, format sans timestamp (comparable) ; sans `--log`, rien
   ne change (sorties et JSON identiques). Doc dans l'en-tête du binaire et conventions §24.
6. **Hors périmètre** : les bots, le contenu `throne`, les gabarits de salles LDtk, D4.

## Critères d'acceptation

1. Diagnostic écrit (frame d'entrée hors champ, cause) pour les graines 43 et 162 ; après
   correction, les deux ennemis sont atteignables (rejeu : l'ennemi est dans une case libre pour
   sa clé à toutes les frames ; les bots de `main` finissent ou meurent mais pas de soft-lock sur
   ces deux graines, `alacod-sim` construit depuis ta branche).
2. Test 1 000 graines × 3 cavernes (décision 2) ou test de pas (décision 3) verts ; fixture de
   lint si décision 2.
3. Suite complète : traces qui bougent listées avec preuve ; suite des crates, `make lint` des
   trois jeux, `make fmt`, scripts, `make gen` des trois jeux sans modification, exemples.
4. `alacod-sim --log` montre les journaux ; sans `--log`, 20 graines JSON identiques à avant.
5. Rapport `docs/taches/rapports/m1-d48-ennemis-hors-champ.md` (README §7), point d'état,
   purge du target après chaque suite.

## Livrer

Merger `origin/main` juste avant de livrer. `git push -u origin m1-d48-ennemis-hors-champ`, puis
`SendMessage` à `orch` : `LIVRÉ m1-d48-ennemis-hors-champ <sha> : <cause, correction, traces qui
bougent>`. Ne merge pas, ne bénis pas.
