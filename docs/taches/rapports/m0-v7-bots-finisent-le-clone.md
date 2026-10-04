# Rapport — m0-v7 : les bots finissent le clone

**SHA du code validé sur 20 graines : `95630213fda21eab8a06918f32bb876b34a8b456`** (état
fusionné avec `origin/main` `23a43fc`).
Fiche : [m0-v7-bots-finisent-le-clone](../m0-v7-bots-finisent-le-clone.md).
Branche : `m0-v7-phase2-bots-finisent-le-clone` ; agents : Codex (correctifs bots, tir,
aggro, récupération, secours de spawn), puis Claude Code (graines 16/17, merge de main,
ré-étalonnage des scénarios, preuve des traces, validation) ; date : 2026-10-03.

## Phase 2 — Correctif

Les profils `chasseur` et `acheteur` suivent des chemins physiques, tirent en marchant,
changent d'arme et rechargent. L'acheteur utilise les interactions ordinaires pour les
munitions, Juggernog et une porte abordable quand le combat est inaccessible ; les deux
profils peuvent réanimer. Les décisions v0 et leurs traces restent inchangées.
`alacod-sim` choisit quatre acheteurs par défaut, tout en conservant `--profiles` et le
`--map` de m0-v8. L'option `--progress` observe la progression toutes les 1000 frames
hors simulation, sans RNG ni état de décision supplémentaire.

### Diagnostic et choix du correctif

Le diagnostic de phase 1 est conservé ci-dessous avec ses huit dumps permanents.
Les compteurs de vague sont cohérents dans ces cas : les derniers zombies sont vivants,
physiquement bloqués aux fenêtres/murs, et les bots v0 cherchent en ligne droite ou
épuisent leur mitrailleuse en gardant des munitions de shotgun. Le correctif donne aux
nouveaux profils des postes de tir accessibles et récupère les waypoints invalides.

La navigation réutilise `FlowField` et `GridPos` avec un Dijkstra multi-source,
`BTreeMap`/`BTreeSet`, calculs `Fixed` et cibles triées par `GgrsNetId`. La grille de 8 px,
le corps et son offset permettent les ouvertures physiques dont le centre tombe entre
les tuiles de 16 px. Les fenêtres restent infranchissables pour le joueur mais permettent
le tir ; murs et portes fermées bloquent aussi la visibilité. Les diagonales ne coupent
pas les coins. Le champ se limite à la composante accessible du joueur ; les approches
de portes, fenêtres et zones d'aggro sont partagées et conservées tant que leurs clés
restent identiques. Sans poste de tir, une approche dans le rayon d'aggro réveille le
zombie distant pour qu'il casse sa fenêtre. L’approche d’aggro ne concerne que les zombies Idle : un zombie déjà en Chasing
ne doit pas retenir le bot dans son rayon sans poste de tir. Sans approche, réparer rapporte les points
nécessaires à la première porte ; les choix de portes ordonnent prix, distance à l'ennemi
et net_id.

Ces caches sont dérivés dans `ReadInputs`, avec toutes leurs dépendances dans leurs
clés, et non dans `GgrsSchedule`. Ils ne sont donc pas des ressources rollback : les
inputs décidés sont capturés et leur rejeu scripté donne la même trace. Le registre
rollback n'est pas étendu pour un état de décision caché.

Une sonde intermédiaire a révélé un problème de collision du joueur : lorsque XY était
bloqué, vérifier X et Y séparément depuis la position initiale pouvait autoriser leur
combinaison dans un coin de mur. Le test de régression échoue avant le correctif et
passe ensuite ; Y est désormais vérifié après le glissement X. La graine 1 passe ainsi
de 494 violations `joueur_hors_mur` dans la sonde intermédiaire à zéro dans la victoire
à f7069. Les traces v0 existantes ne rencontrent pas ce cas et restent identiques.

Une première validation (`3cf851e`) a aussi plafonné sur la graine 4 en vague 3 : sept zombies
aux fenêtres/murs, quatre bots immobiles, pistolets à 4–5 balles, zéro nouveau kill
entre f19400 et f20000. Le mode Manual exige un relâchement entre deux pressions ;
maintenir Fire ne redéclenchait pas le tir après le changement d'arme. Les profils v1
lisent maintenant `WeaponState.is_firing` pour relâcher Manual, Shotgun et Burst,
sans compteur caché ; Automatic garde la pression continue. Le test des quatre modes
et le rejeu scripté couvrent cette correction. Dump intermédiaire permanent :
[graine 4 avant correction du tir](m0-v7-bots-finisent-le-clone.phase2-stall.json).

Les graines 11/12 ont ensuite exposé une autre attente (`791f620`) : aucun poste de
tir accessible, zombie déjà Chasing mais approche d'aggro à distance nulle, ce qui
supprimait portes et réparations. La graine 11 conservait un zombie et quatre bots
à 100 de santé, avec des mitrailleuses pleines ; deux avaient 790/760 points pour une
porte à 750. La graine 12 conservait deux zombies et des soldes 580–720, nécessitant
encore des réparations pour sa porte à 750. L'approche d'aggro est désormais réservée
aux Idle ; les zombies éveillés inaccessibles ne bloquent plus ces interactions. Si
une autre surface possède le prompt près du but, le bot se rapproche encore ou essaie
une autre interaction. Le test `hunter_doors` rejoue les graines 2, 11 et 12 en synctest et
vérifie l'entrée en vague 2 avant f3500, avec quatre survivants et zéro failure.

Une validation suivante (`c8cb176`) a plafonné sur la graine 2 : le dernier zombie
Chasing visait un point presque confondu avec sa position, hors de sa case suivante.
L'investigation vers le point accessible le plus proche l'a débloqué, mais la graine
12 a révélé un autre zombie avec son waypoint dans le mur 222. Les bots avaient ouvert
deux portes et manquaient de points pour la suivante ; tous les compteurs de vague
restaient cohérents. Ce cas nécessite aussi une correction du guidage zombie.

Après 600 frames sans kill ni spawn, en Spawning ou InProgress, un zombie de vague
avec un waypoint sorti de sa case ou dans un mur vise un point physiquement libre
dans la prochaine case. Une diagonale peut passer par une étape cardinale pour aligner
le corps dans une ouverture. Chaque segment est vérifié à intervalles d'un pixel avec
le collider et son offset ; les fenêtres intactes restent bloquantes. Le classement
(étape, distance au waypoint, x, y) est déterministe. La vitesse et les collisions
ordinaires restent appliquées. Le délai lit les compteurs existants de WaveState,
sans nouveau registre rollback. Trois tests couvrent délai, point de frontière et
alignement au coin de fenêtre, avec refus d'un trajet totalement bloqué.

La nouvelle signature signalée sur avant_poste était distincte : phase `Spawning`,
ennemis encore à générer, aucun spawner à distance euclidienne 150–600. Après 600 frames
sans spawn (mesurées depuis le dernier spawn ou le début de phase), le système choisit
le spawner le plus proche d'un joueur, égalités départagées par net_id. Les spawners
normaux gardent la priorité ; nombre simultané, lots et intervalle restent appliqués.
Le drapeau `WaveState.spawn_fallback`, rollback et tracé avec cette ressource, maintient
ensuite la cadence normale dans cette vague et est remis à faux dans
`prepare_next_wave`. Le hash conserve la contribution historique lorsque le drapeau
est faux et ajoute une contribution lorsqu’il est vrai : le secours reste vérifié par
checksum sans décaler toutes les traces existantes par un nouveau type enregistré.
Une nouvelle vague doit atteindre son propre délai avant un nouveau secours.

### Graines 16/17 : monde non aligné sur la grille (reprise par Claude Code)

Avec le guidage de récupération (`c8cb176`/`318affc`), 18/20 graines passaient ; 16 et 17
plafonnaient en vague 1 (dumps permanents :
[graine 16](m0-v7-bots-finisent-le-clone.phase2-stall-16.json),
[graine 17](m0-v7-bots-finisent-le-clone.phase2-stall-17.json)). Cause prouvée :
`get_spawning_room` (`crates/map/src/generation/imp/basic.rs`) posait la salle de spawn à
une position pixel tirée au hasard, et toutes les autres salles en dérivent par multiples
de la taille de tuile : le monde entier héritait du même résidu (graine 16 : toutes les
salles ≡ (14, 2) mod 16). Les rectangles de collision fusionnés débordaient alors sur une
colonne et une rangée de cellules de plus que les cellules IntGrid enregistrées par la nav :
367 cellules bloquées physiquement mais libres pour le flow field dans la grille de la
graine 16 (`p` dans le dump), 0 cellule dans l'autre sens. Exemple mesuré : le mur 16×32
en (694, 370), rectangle `{x: 8, y: 11, w: 1, h: 2}` de Level_2 dans la salle (558, 258),
couvre physiquement les cellules (42..43, 22..24) alors que la nav n'enregistre que (43, 22)
et (43, 23). Le flow field faisait passer les zombies dans ces cellules et des lamelles de
2 px accrochaient le coin de leur corps de 20 px.

Premier correctif écarté : ajouter les cellules des colliders de mur à `wall_cells` dans
`rebuild_blocked_cells`. La nav ne promettait plus de passage physique, mais la graine 16
plafonnait encore à 4000 frames (3 zombies sans chemin, enfermés dans des poches).

Correctif retenu (`8785220`) : la position tirée est arrondie au multiple de tuile le plus
proche, bornes ramenées aux multiples contenus dans l'intervalle (`snap_to_tile_grid`). Le
flux RNG est inchangé, seul le placement bouge. Graines 16/17 seules à 20 000 frames :
victoire vague 5 à f8060 et f7909, 0 desync. Le snap translate de 0 à 8 px par axe le
monde de **toute** carte : toutes les cartes LDtk, arènes du testbed et avant_poste compris,
passent par le callback du loader (`crates/map_ldtk/src/loader/mod.rs`) → `map_generation`
→ `get_spawning_room`. D'où le changement des 82 traces et le ré-étalonnage ci-dessous.

### Merge de main

`origin/main` intégré deux fois : `d877a26` (`2a748bc`), puis `23a43fc` (`9563021`).
Conflits textuels gardant les deux côtés : `BotView` (`hunter` + `portal`), doc
d'`alacod-sim` (`--profiles`/`--progress` + `--floors`), `select_valid_spawners`
(`ResolvedWaveConfig` + `allow_nearest`), `conventions.md` (§16-18 de main, bots de
validation en §24 — §19 est réservé aux statuts de c3, renuméroté par l'orchestrateur), dépendance `run` en double dans `crates/bots/Cargo.toml`. Conflits
sémantiques avec F5 (m0-v11) : les bots v1 lisent prix, recharge et perks dans
`ResolvedBalance` comme les handlers d'interaction ; `spawn_stall` force la plage vide dans
`ResolvedBalance.waves`, déclaré `ambiguous_with_all` (`4769ad2`) ; champ `floors` dans les
`Scenario` des tests v1.

### Attentes de scénarios ré-étalonnées (`07c2862`)

Un seul commit, sans code ni trace. L'intention de chaque attente est conservée ; les
valeurs exactes sont re-mesurées avec `ALACOD_EVENTS=1`. Cause commune : translation du
monde de 0 à 8 px par axe.

| Scénario | Avant → après | Mesuré |
|---|---|---|
| `downed_all_lose` | `PlayerDowned(1)@1200` → `@1400` ; `PlayerDead(0)`, `Defeat@1600` → `@1700` ; `frames` 1600 → 1700 | à terre f1296, mort et défaite f1570 (avant f1174/f1505) |
| `run_lose_summary` | idem + `RunState(Ended(Defeat))`, `RunSummary@1600` → `@1700` ; `frames` 1600 → 1700 | idem |
| `two_players_idle` | `PlayerDowned(1)`, `PlayerAlive(0)@1200` → `@1400` | joueur 1 à terre f1296, joueur 0 mort f1570 (après la fin, 1500) |
| `shoot_around` | `PlayerDead(0)@1200` → `@1300` | mort f1241 (avant f906) |
| `movement_melee` | positions (612.4, 695), (556.4, 695), (555, 695) → (613.4, 696), (557.4, 696), (556, 696) | +1/+1 : translation exacte |
| `clone_quad` | soldes f4372 des bots 1440/1570/1290 → 1570/1720/1080 | victoire vague 5 conservée (f4323) |
| `clone_duo` | visée des deux joueurs ré-enregistrée à partir de f1000 ; soldes 3630@2400, 3790@3700, 3800/1440@4668 → 3620, 3710, 3790/1450 | victoire à l'entrée en vague 5 à f4668, 13 kills, deux joueurs debout |

`clone_duo` était bloqué avec les pans figés de T3.1 : les tirs ratent les nouveaux trajets,
pas de victoire en 9336 frames (2 × 4668), joueur 1 mort f6880. Règle d'écriture, à
rejouer à la prochaine translation : à partir de f1000, toutes les 15 frames, chaque joueur
tire 2 frames vers le zombie vivant le plus proche s'il est à 400 px au plus (pan = delta
monde arrondi, +y vers le haut). Les segments produits sont figés dans le `.ron`. Mise en
scène f0-f986 inchangée (tir ami f40-42, réanimation, achats, Max Ammo, Nuke). L'écrivain
temporaire est un test `#[ignore]` sur `run_with_options`, avec un système `ReadInputs`
entre `read_local_inputs` et `record_local_inputs` ; il n'est pas commité.

### Preuve des traces

`preuve-snap` = `23a43fc` + `8785220` + `07c2862` (cherry-picks `976e68b`, `09ce982`),
comparée à la tête `9563021`. Sur chaque côté :

```bash
make test_scenarios BLESS=1        # 3 réussis, 0 assertion en échec
cp --parents $(find tests/scenarios -name '*.trace') ../<côté>-traces/
git checkout -- tests/scenarios    # aucune trace commitée
diff -r ../preuve-snap-traces ../tete-traces
```

Résultat : **77/82 identiques, 5 différentes**, toutes tardivement. Cause prouvée : le
guidage de récupération des zombies (`navigation_recovery_due`,
`crates/game/src/character/enemy/ai/pathing.rs`, `318affc`) s'arme après 600 frames sans
kill ni spawn. Dans le monde translaté, des zombies de ces scénarios sans kill ont un point
de steering bloqué ; avant le snap, ce cas n'arrivait sur aucune trace. Contre-épreuve : sur
la tête, seuil temporairement porté de `>= 600` à `>= u32::MAX`, puis
`ALACOD_BLESS=1 ALACOD_SCENARIO=<nom>` pour ces 5 scénarios. Les 5 traces deviennent
identiques octet pour octet à `preuve-snap`. Fichier restauré, worktree propre.

`sha256sum` des 82 traces de la tête (triées, chemins relatifs) : `../tete-traces.sha256`,
lui-même de sha256 `a55280d9dcf245d37a8f3071d982443810db53b3b5dd5add76e2d81ae45b9e2b`.
Les traces sont bénies par l'orchestrateur, qui recoupe avec ce fichier. La référence p2p
(sha256 `55ec099d…`) change aussi, c'est attendu : l'orchestrateur la rejoue au merge.

| Scénario | Tête vs preuve-snap | Cause |
|---|---|---|
| `ammo_burst` | identique | — |
| `ammo_shared_reserve` | identique | — |
| `avant_poste_demo` | identique | — |
| `bench_bullets` | identique | — |
| `bench_horde` | écart à partir de la ligne 663 | récupération du guidage zombie (seuil 600) |
| `bots_four_mixed` | identique | — |
| `bots_two_fonceurs` | identique | — |
| `bullets_walls` | identique | — |
| `buy_door` | identique | — |
| `buy_perk` | identique | — |
| `buy_wall_weapon` | identique | — |
| `clone_duo` | identique | — |
| `clone_quad` | identique | — |
| `clone_solo` | identique | — |
| `dash_wall` | identique | — |
| `door_open` | identique | — |
| `downed_all_lose` | écart à partir de la ligne 1142 | récupération du guidage zombie (seuil 600) |
| `downed_bleedout` | identique | — |
| `downed_revive` | identique | — |
| `drop_pickup_swap` | identique | — |
| `equilibrage_joueurs_duo` | identique | — |
| `equilibrage_joueurs_quad` | identique | — |
| `four_players_idle` | identique | — |
| `four_players_shooting` | identique | — |
| `friendly_fire_cursed` | identique | — |
| `friendly_fire_never` | identique | — |
| `generated/testbed/weapon_axe` | identique | — |
| `generated/testbed/weapon_bare_hands` | identique | — |
| `generated/testbed/weapon_club` | identique | — |
| `generated/testbed/weapon_grenade` | identique | — |
| `generated/testbed/weapon_knife` | identique | — |
| `generated/testbed/weapon_machine_gun` | identique | — |
| `generated/testbed/weapon_pistol` | identique | — |
| `generated/testbed/weapon_proj_bounce` | identique | — |
| `generated/testbed/weapon_proj_gravity` | identique | — |
| `generated/testbed/weapon_proj_homing` | identique | — |
| `generated/testbed/weapon_proj_lifetime` | identique | — |
| `generated/testbed/weapon_proj_pierce` | identique | — |
| `generated/testbed/weapon_proj_size` | identique | — |
| `generated/testbed/weapon_rifle` | identique | — |
| `generated/testbed/weapon_shotgun` | identique | — |
| `generated/testbed/weapon_sword` | identique | — |
| `generated/testbed/weapon_zombie_claws` | identique | — |
| `generated/zombies/weapon_axe` | identique | — |
| `generated/zombies/weapon_bare_hands` | identique | — |
| `generated/zombies/weapon_club` | identique | — |
| `generated/zombies/weapon_knife` | identique | — |
| `generated/zombies/weapon_machine_gun` | identique | — |
| `generated/zombies/weapon_pistol` | identique | — |
| `generated/zombies/weapon_rifle` | identique | — |
| `generated/zombies/weapon_shotgun` | identique | — |
| `generated/zombies/weapon_sword` | identique | — |
| `generated/zombies/weapon_zombie_claws` | identique | — |
| `idle` | écart à partir de la ligne 1199 | récupération du guidage zombie (seuil 600) |
| `immune_tag` | identique | — |
| `movement_melee` | identique | — |
| `points_on_kill` | identique | — |
| `portal_next_floor` | identique | — |
| `powerup_carpenter` | identique | — |
| `powerup_double_points` | identique | — |
| `powerup_drop_on_kill` | identique | — |
| `powerup_insta_kill` | identique | — |
| `powerup_max_ammo` | identique | — |
| `powerup_nuke` | identique | — |
| `remote_first_fight` | identique | — |
| `run_lose_summary` | écart à partir de la ligne 1142 | récupération du guidage zombie (seuil 600) |
| `shoot_around` | identique | — |
| `shop_tour` | identique | — |
| `stat_move_speed` | identique | — |
| `testbed_ally_safe` | identique | — |
| `testbed_arena_idle` | identique | — |
| `testbed_civilian_blocks` | identique | — |
| `testbed_corridor_idle` | identique | — |
| `testbed_dummy_shoot` | identique | — |
| `testbed_follower` | identique | — |
| `testbed_target_hits` | identique | — |
| `testbed_two_rooms_door_idle` | identique | — |
| `testbed_window` | identique | — |
| `two_players_idle` | écart à partir de la ligne 1142 | récupération du guidage zombie (seuil 600) |
| `two_players_shooting` | identique | — |
| `weapons_workout` | identique | — |
| `window_repair` | identique | — |

## Validation de phase 2

Commande par graine `n`, après `source ../env.sh` et `export CARGO_BUILD_JOBS=4` :

```bash
cargo run -q -p scenario --profile headless --bin alacod-sim -- \
  --game zombies --bots 4 --map exemples/test_map.ldtk \
  --seeds "${n}..${n}" --until-wave 5 --max-frames 20000 --progress \
  --json "../validation-seeds/${n}.json"
```

Le binaire est compilé une fois, copié sous son SHA dans `../validation-bin/<sha>/`, puis
les graines indépendantes tournent par lots de quatre (`../validate-seeds.py`, qui ajoute
`--save-scenario ../recordings-final`). Cette copie ne change pas pendant la validation.
La carte est explicite : le `start_map` du manifeste est désormais avant_poste. Le JSON
permanent ([validation](m0-v7-bots-finisent-le-clone.phase2-validation.json)) et le
tableau ci-dessous concernent test_map, comme le digest M0. Les FPS sont observés sous
charge concurrente (quatre simulations et d'autres tâches) et ne constituent pas un bench
au calme.

**Validation finale sur l'état fusionné `9563021` : 20/20 victoires (vague 5), 0 mort,
0 desync, 0 plafond, 0 failure.** La même validation sur `8785220` (avant le merge de
main) avait donné exactement les mêmes vague, frames, morts et kills pour les 20 graines.

| Graine | Vague atteinte | Frames | Morts / 4 | Kills | Desync | Plafond | sim fps |
|---:|---:|---:|---:|---:|---|---|---:|
| 1 | 5 | 7425 | 0 | 52 | non | non | 13.7 |
| 2 | 5 | 7426 | 0 | 53 | non | non | 12.2 |
| 3 | 5 | 7114 | 0 | 53 | non | non | 18.4 |
| 4 | 5 | 8507 | 0 | 49 | non | non | 12.8 |
| 5 | 5 | 6953 | 0 | 51 | non | non | 19.3 |
| 6 | 5 | 7751 | 0 | 52 | non | non | 17.6 |
| 7 | 5 | 7997 | 0 | 53 | non | non | 12.9 |
| 8 | 5 | 7250 | 0 | 50 | non | non | 19.0 |
| 9 | 5 | 7690 | 0 | 52 | non | non | 39.6 |
| 10 | 5 | 6877 | 0 | 51 | non | non | 14.8 |
| 11 | 5 | 7293 | 0 | 52 | non | non | 20.1 |
| 12 | 5 | 8624 | 0 | 52 | non | non | 11.2 |
| 13 | 5 | 7720 | 0 | 52 | non | non | 17.5 |
| 14 | 5 | 7352 | 0 | 50 | non | non | 12.3 |
| 15 | 5 | 6887 | 0 | 50 | non | non | 18.6 |
| 16 | 5 | 8060 | 0 | 52 | non | non | 21.7 |
| 17 | 5 | 7909 | 0 | 51 | non | non | 14.0 |
| 18 | 5 | 8030 | 0 | 53 | non | non | 17.5 |
| 19 | 5 | 7582 | 0 | 51 | non | non | 22.7 |
| 20 | 5 | 8034 | 0 | 54 | non | non | 15.0 |

## Vérifié (phase 2)

Toutes les commandes de compilation sont précédées de `source ../env.sh` et de
`export CARGO_BUILD_JOBS=4`, dans le worktree de la tâche, sur la tête `9563021`.

- Suite de scénarios : `make test_scenarios BLESS=1` (traces copiées puis restaurées)
  sur la tête et sur `preuve-snap`, 82 scénarios, 3 tests réussis, 0 échec, 7 ignorés,
  aucune attente en échec. Sans bless, seules les traces diffèrent (voir Preuve).
- `cargo test -q --profile headless -p scenario --test hunter_doors` (graines 2, 11, 12
  en synctest, vague 2 avant f3500, quatre survivants) : 1 réussi, 268 s ;
  `--test spawn_stall` : 1 réussi.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content
  -p map_ldtk -p sim_core -p stats -p bots -p effects -p behaviors -p map --no-fail-fast` :
  381 réussis, 1 échec, 8 ignorés (39 blocs). L'échec est le test `scenarios`, uniquement
  sur les 82 traces non bénies ; aucune attente en échec.
- `make lint` : `zombies` et `testbed` sans erreur.
- `cargo fmt --all -- --check` : sortie vide, code 0 (`f2f7616` retire une ligne vide
  finale de `pathing.rs`).
- `./scripts/check-forbidden.sh` : code 0, quatre occurrences préexistantes, aucune nouvelle.
- `./scripts/check-rollback-registration.sh` : OK. Aucun type rollback ajouté ou retiré
  par la branche (diff contre la merge-base) : le piège de parité des types vides ne
  s'applique pas.
- `make gen GAME=zombies` : aucun fichier généré modifié, 10 armes avec attentes `ok` ;
  code 2 parce que les 10 traces générées diffèrent (incluses dans les 82 à bénir).

## Non fait / non vérifié (phase 2)

- Critère des 200 graines et re-validation avant_poste : réservés à l'orchestrateur après
  merge.
- Aucune trace bénie ni commitée ; les 82 sont à bénir par l'orchestrateur (sha256 ci-dessus).
- Pas de revue visuelle ni de test p2p dans cette reprise.
- Aucun merge dans `main`, aucune modification de `docs/taches.md`.

## Dettes / questions ouvertes (phase 2)

- D20 marquée « fait dans m0-v7-bots-finisent-le-clone » (`2fb3390`).
- Les scénarios scriptés à visée figée (`clone_duo` et ses pans) restent sensibles à toute
  translation du monde : la règle d'écriture ci-dessus permet de les ré-enregistrer.
- Le guidage de récupération change le comportement des zombies dans les scénarios sans
  kill de plus de 600 frames (5 traces) : c'est voulu, mais à garder en tête quand une
  trace d'`idle` bouge.

# Phase 1 — rapport d'origine (conservé tel quel)

**SHA de la tête avant commit du rapport : `e1aaf18fddc5339ed0c860aed11c0ac7ac829d03`.**
Fiche : [m0-v7-bots-finisent-le-clone](../m0-v7-bots-finisent-le-clone.md).
Branche : `m0-v7-bots-finisent-le-clone` ; agent : Codex ; date : 2026-10-02.

Livraison de **phase 1 uniquement**.
Les huit rejeux longs représentent 160 000 frames en 2 722,30 s (45 min 22,3 s),
0 failure, 0 desync. Le code du diagnostic initial est `3b5c6bb` (graines
4/11/12/13/16), enrichi par `e63c72b` (17/18/19). Les outils de sondage sont
complétés par `05ec58a`. `main` à `05fd7f9` est intégré par `554e99f`,
sans conflit ; les vérifications et sondes suivantes se font sur cet état fusionné.
La mise à jour documentaire de `main` à `b0e3f99` est ensuite intégrée par
`e1aaf18` : seul `docs/taches.md` diffère de la tête de code testée `554e99f`.
Le commit de livraison qui suit ne contient que le rapport, les dumps et D20.

## Fait

Dump permanent `softlock` du JSON `alacod-sim` quand le plafond est atteint sans
objectif ni mort de tous les joueurs. Le runner prend deux instantanés hors
simulation : fin et 600 frames auparavant. Ils contiennent la phase et les
compteurs de vague, les ennemis (positions, santé, cible, état, coût et prochaine
case du flow field), les joueurs encore présents (à terre signalé, munitions et
solde), les portes fermées (coût, interaction), les fenêtres et la grille ASCII
navigation/physique. Les observations restent factuelles : aucune cause arbitraire
n'est attribuée à une porte. Aucun système ni état rollback ajouté.

Profils v0 et fichiers `.trace` inchangés. Aucun bless.

## Diagnostic

Dumps complets : [huit rejeux longs](m0-v7-bots-finisent-le-clone.diagnostic.json)
et [huit sondes à f5000](m0-v7-bots-finisent-le-clone.probes.json).
Les enregistrements RON restent dans `../recordings/seed_n.ron` du worktree.

Commande pour chaque graine `n` dans `4 11 12 13 16 17 18 19` :

```bash
source ../env.sh
export CARGO_BUILD_JOBS=4
cargo run -q -p scenario --profile headless --bin alacod-sim -- \
  --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile \
  --seeds "${n}..${n}" --until-wave 5 --max-frames 20000 \
  --save-scenario ../recordings --json "../diagnostic/seed_${n}.json"
```

Sonde du scénario enregistré, avec les chemins absolus car Cargo lance les tests
depuis `crates/scenario` (premier essai avec chemins relatifs : fichiers introuvables,
relance corrigée ci-dessous). Couples `(n, id)` : `(4, 301)`, `(11, 792)`, `(12, 297)`,
`(13, 670)`, `(16, 320)`, `(17, 403)`, `(18, 247)`, `(19, 493)`.

```bash
ALACOD_PROBE_FILE="$PWD/../recordings/seed_${n}.ron" \
ALACOD_PROBE_JSON="$PWD/../diagnostic/probe_${n}.json" \
ALACOD_PROBE="${id}:5000" \
cargo test -q -p scenario --profile headless --test scenarios nav_probe -- --ignored --nocapture
```

### Cas observés

- Graine 4 : six zombies, cinq vers `(465, -735)` à la fenêtre 66 `(484, -730)`,
  un vers `(726, -479)` à la fenêtre 69 `(692, -474)` ; toutes deux cassées.
  Les six ont un chemin GroundBreaker (coûts 220/880). Entre f19400 et f20000,
  déplacement net maximal 1,28 px ; joueurs 2 et 3 strictement aux mêmes positions.
  Le prudent a un chargeur et une réserve Balle vides, mais 48 Plomb inutilisés.
  L'immobile a encore 30 + 240 Balle et ne tire jamais. Les deux fonceurs sont morts.
  La sonde f5000 montre les six points physiquement bloqués : `(488, -736)` par
  le mur 135 (`x = 476..492`, `y = −826..−746`), et `(712, −479)` par
  le mur 153 (`x = 684..716`, `y = −506..−490`).
- Graine 11 : un zombie vers `(-246, -527)` à la fenêtre 50 `(-265, -524)`,
  cassée, chemin de coût 644. Déplacement net 1,13 px, dernier kill f3236.
  Les fonceurs restent vers `(-491, -236)` et le prudent vers `(-427, -172)`,
  contre les murs en direction du zombie distant. Leurs entrées sauvegardées à
  f19999 sont `Down, Right`, sans réparation ; la fenêtre la plus proche est pleine.
  Quatre bots en vie avec des munitions.
  Les trois chasseurs ont le solde pour la porte 6 à 750, qui reste fermée.
  À f5000, le point `(−264, −528)` chevauche le mur 83, bornes
  `x = −273..−257`, `y = −572..−540`.
- Graine 12 : six zombies vers `(-504, 240)` à la fenêtre 60 `(-486, 247)`,
  cassée, tous avec un chemin de coût 214. Déplacement net maximal 1,32 px,
  dernier kill f698. Prudent sans Balle (48 Plomb inutilisés), immobile encore
  approvisionné, deux fonceurs morts. Positions des survivants inchangées à f19400/f20000.
  À f5000, les six points `(−488, 241)` sont bloqués par le mur 84,
  `x = −494..−478`, `y = 151..231`.

- Graine 13 : un zombie `(304, 288)` à la fenêtre 54 `(285, 282)`, cassée,
  chemin de coût 672. Même position à f19400/f20000, dernier kill f3211.
  Fonceurs `(59, 570)` et prudent `(123, 634)` contre les murs, avec des munitions ;
  quatre joueurs vivants. Les trois chasseurs gardent `Down, Right` jusqu’au
  plafond, sans réparation (entrées sauvegardées). À f5000, le zombie vise
  `(304, 288)` depuis `(304.02643, 287.99298)` : vitesse principale et knockback
  nulles, aucun mur chevauché au point visé. La prochaine case est `(18, 18)`,
  mais le point visé tombe dans `(19, 18)` ; il ne fait pas progresser jusqu’à
  cette prochaine case. Le même zombie reste exactement à cette position à
  f19400/f20000.
- Graine 16 : deux zombies vers `(712–713, 400)` à la fenêtre 46 `(694, 402)`,
  cassée, chemins de coût 608. Déplacements nets 0,54 et 1,08 px, dernier kill f954.
  Les trois combattants ont un chargeur et une réserve Balle vides (48 Plomb chacun
  encore disponibles). Les fonceurs bougent encore (déplacements nets 25–27 px)
  mais n'obtiennent ni kill ni contact avec les zombies ; prudent et immobile sont
  aux mêmes positions. Quatre joueurs vivants. À f5000, les deux points
  `(696, 401)` chevauchent le mur 101 (`x = 686..702`, `y = 354..386`).

- Graine 17 : deux zombies `(−680, 678)` au-dessus du mur 143, bornes
  `x = −704..−560`, `y = 645..661`, qui sépare les fonceurs `(−678, 641)`.
  Tous deux ont un chemin de coût 60 ; le point visé `(−680, 673)` ferait
  chevaucher leur corps avec ce mur (même résultat à f19400 et f20000).
  Déplacement net 0,85 px, dernier kill f1067. Trois combattants sans Balle,
  avec 48 Plomb inutilisés chacun ; quatre joueurs vivants.

- Graine 18 : deux zombies vers `(−323, −496)` à la fenêtre 50 `(−341, −489)`,
  cassée. Chemins de coûts 440/444, mais point visé `(−344, −495)` bloqué par
  le mur 175 : `x = −349..−333`, `y = −537..−505` ; le corps au point visé
  descend à −511, soit 6 px dans le mur. Même blocage aux deux instantanés.
  Déplacements nets 1,13 et 1,77 px, dernier kill f1173. Trois combattants sans
  Balle, 48 Plomb chacun encore disponibles, quatre joueurs vivants.

- Graine 19 : un zombie `(−83, 933)` au-dessus du mur 149, bornes
  `x = −154..−10`, `y = 901..917`, qui sépare les fonceurs vers `(−70..−77, 897)`.
  Chemin de coût 64 ; point visé `(−72, 929)` bloqué par ce mur (corps descendant
  à 913, 4 px dans le mur). Dernier kill f2533. Le zombie et les fonceurs se
  déplacent le long du mur (déplacements nets 21 et 31–43 px), sans résoudre
  le combat. Trois combattants sans Balle, 48 Plomb chacun inutilisés, quatre
  joueurs vivants (joueur 1 à 97,5 de santé).

À f5000, 20 des 21 zombies restants visent un point qui chevaucherait un mur ;
le cas 13 vise un point presque confondu avec sa position, hors de la prochaine case.
Les compteurs observés sont cohérents : phase `InProgress`, aucun ennemi à générer,
les ennemis de vague restants expliquent l'attente. Aucun cas observé de vague qui
resterait bloquée avec zéro zombie. Les fenêtres ont des ouvertures IntGrid dans
les gabarits LDtk. Des points de steering physiquement bloqués sont démontrés pour 17/18/19 ;
les sondes f5000 démontrent aussi ces chevauchements pour 4/11/12/16.
Le mauvais trajet des bots et la mitrailleuse
épuisée sont des faits distincts. Deux instantanés ne prouvent pas une immobilité
à chaque frame intermédiaire : les déplacements ci-dessus sont des déplacements nets.

La construction du [point de steering](../../../crates/game/src/character/enemy/ai/navigation.rs)
additionne les poussées cardinales et diagonales sans vérifier la collision du
point final ni sa présence dans la case suivante. Les dumps montrent deux
signatures de cette limite : chevauchement physique pour sept graines, point
presque confondu avec la position du zombie et situé hors de la case suivante
pour 13. Le repli vers la case d’après ne s’applique que lorsque la direction
calculée est exactement nulle. La sonde 13 constate une vitesse finale nulle ;
le détail de l’évitement à chaque frame n’est pas instrumenté ici.

## Tableau des 20 graines de validation

À cette livraison de phase 1, seules les huit graines bloquées sont rejouées avec
le line-up v0. **Le nouveau line-up n’existe pas encore : les 20 validations de
phase 2 sont toutes non faites.** Les douze autres graines v0 ne sont pas rejouées ici.
Les huit caps ont les mêmes vague/frames/morts/kills/desync que le JSON du digest.
Six caps ont quatre bots vivants (les tableaux du digest donnent six, malgré
le chiffre cinq de sa prose et de la fiche).

| Graine | Vague v0 rejouée ici | Frames | Morts / 4 | Kills | sim fps | Validation line-up final (phase 2) |
|---:|---:|---:|---:|---:|---:|---|
| 1 | Non rejouée | — | — | — | — | Non jouée |
| 2 | Non rejouée | — | — | — | — | Non jouée |
| 3 | Non rejouée | — | — | — | — | Non jouée |
| 4 | 1 | 20000 | 2 | 0 | 55.5 | Non jouée |
| 5 | Non rejouée | — | — | — | — | Non jouée |
| 6 | Non rejouée | — | — | — | — | Non jouée |
| 7 | Non rejouée | — | — | — | — | Non jouée |
| 8 | Non rejouée | — | — | — | — | Non jouée |
| 9 | Non rejouée | — | — | — | — | Non jouée |
| 10 | Non rejouée | — | — | — | — | Non jouée |
| 11 | 2 | 20000 | 0 | 18 | 60.2 | Non jouée |
| 12 | 1 | 20000 | 2 | 1 | 60.1 | Non jouée |
| 13 | 2 | 20000 | 0 | 19 | 60.8 | Non jouée |
| 14 | Non rejouée | — | — | — | — | Non jouée |
| 15 | Non rejouée | — | — | — | — | Non jouée |
| 16 | 1 | 20000 | 0 | 5 | 56.4 | Non jouée |
| 17 | 1 | 20000 | 0 | 6 | 61.7 | Non jouée |
| 18 | 1 | 20000 | 0 | 6 | 62.4 | Non jouée |
| 19 | 2 | 20000 | 0 | 16 | 54.6 | Non jouée |
| 20 | Non rejouée | — | — | — | — | Non jouée |

## Vérifié

Toutes les commandes de compilation ci-dessous sont précédées de `source ../env.sh`
et de `export CARGO_BUILD_JOBS=4`, dans le worktree de la tâche.

- `make test_scenarios` : 61 scénarios verts, traces comparées, 2 tests réussis,
  0 échec et 7 ignorés ; 416,44 s. `clone_quad` finit à f4372 en vague 5,
  13 kills et quatre survivants, avec la trace v0 inchangée.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content
  -p map_ldtk -p sim_core -p stats -p bots -p effects -p behaviors --no-fail-fast` :
  290 réussis, 0 échec, 8 ignorés (sommes des 34 blocs de résultats).
- `make lint` : `zombies` et `testbed` sans erreur.
- `cargo fmt --all -- --check` : sortie vide, code 0.
- `./scripts/check-forbidden.sh` : code 0, quatre occurrences préexistantes.
- `./scripts/check-rollback-registration.sh` : OK, aucun appel direct interdit.
- `make gen GAME=zombies` : code 0, dix scénarios d’armes comparés à leurs traces,
  aucun fichier généré modifié.
- `ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py` :
  deux codes 0, 61 scénarios verts, 2 tests réussis / 0 échec / 7 ignorés,
  318,99 s ; `bench_bullets` 102,3 fps ≥ 70, `bench_horde` 65,6 fps ≥ 38.
  Aucun autre `cargo`, `rustc`, rejeu ou `ffmpeg` détecté au départ ; les contrôles
  de processus pendant le bench ne montrent que ce rejeu comme charge lourde.
- `cargo test -q --profile headless -p scenario --test softlock` : 2 réussis,
  0 échec, 0 ignoré. Plafond artificiel à f601, état précédent à f1, objectif atteint
  exactement au plafond à f1 sans dump, sérialisation JSON, trace exactement égale
  au même run sans collecte (deux exécutions de 601 frames).
- Huit sondes explicites : 1 test réussi et 0 échec par graine, 40 000 frames
  en 841,89 s. Huit dumps JSON produits, tous exactement à f5000 ; 21 zombies
  restants, dont 20 avec un point visé physiquement bloqué.
- Huit graines rejouées : métriques exactement égales au digest
  pour vague/frames/morts/kills/desync, 0 failure, 0 desync.

## Non fait / non vérifié

- Phase 2 : pathfinding, nouveaux profils, achats et validation 20 graines.
- Critère 200 graines réservé à l'orchestrateur après merge.
- Aucune revue visuelle ni test p2p : aucun changement de simulation ou de session
  dans cette phase (diagnostic hors rollback).
- Aucun merge dans `main`, aucune modification manuelle de `docs/taches.md`.

## Dettes / questions ouvertes

D20 reste ouverte. Ne pas marquer « fait dans m0-v7 » avant la phase 2.
Le passage physique, la progression du steering, les chemins des bots et la
gestion de leur armement devront être traités en conservant les profils v0.
Attente du retour de l’orchestrateur avant de commencer la phase 2.
