# dette-d66-vue-directe — D66 partie (2) : vue directe des ennemis (b4)

Branche `dette-d66-vue-directe`, depuis `origin/main` 0a719425, fusionnée avec `origin/main` avant livraison.

## 1. Reproduction

Constat qui change l'hypothèse de départ : `move_enemies` suit le champ de flux vers le joueur le plus
proche **quel que soit l'état** (`Idle` ou `Chasing`). Un ennemi à `Chase` seul approche donc de toute la
carte, `aggro_range` ou non (chien posé à 730 unités : il marche en `Idle`). `aggro_range` ne décide que de
la **cible** : attaques, tirs, règles à cible. Les ennemis immobiles hors `aggro_range` sont ceux dont le
comportement dépend d'une cible : `Wander` (`rodeur`, `Sight(300)`), tireurs.

- `throne_vue_directe` (nouveau, caverne d'essai `cave:essai_vue` 96×44 cases sans ennemi, graine 123456) :
  joueur immobile en (760, 344), `rodeur` posé en (1464, 536), à 730 unités, segment sans mur
  (`sight_probe`). Attente `EnemyDistance(max 45, f1000)`.
  **Rouge sur main/sans donnée** : « distance 700,4 > max 45 » (le rodeur erre) ; **vert** avec `sight_range`.
- `throne_vue_coupee` (témoin) : rodeur en (1448, 72), à 740 unités, mur sur le segment : reste à plus de 600
  unités pendant 1200 frames (vert avant et après : la vue ne traverse pas les murs).
- `throne_rodeur_vue_graine_4` : la **vraie carte** (`cave:niveau_1`, graine 4). Attente `EnemyDistance(NetId 75,
  max 45, f450)`. Rouge sans donnée (« distance 382 > 45 »), vert avec.

## 2. Cas « coincé dans un mur » trouvé

`nav_stats` sur `cave:niveau_1/2/3`, graines 1 à 8, 1500 frames, un joueur immobile. Cas net :
**`cave:niveau_1`, graine 4, `rodeur` #75**, posé en (72, 454), joueur en (376, 248) à 367 unités, vue
dégagée : il erre dans sa niche du bord ouest (x 58–108, y 425–476), **sprite de 32 px enfoncé de 12 px
dans la roche pendant 795 frames** (à f1282, (108, 476)), jamais au contact, `Idle`. Même profil sur
niveau_1 graines 2, 3, 7 (439, 264, 698 frames dans un mur, max 12 px). Cause : `Sight(300)` ne portait pas
jusqu'à lui, donc pas de cible, donc `Wander`. Niveaux 2 et 3 : au plus 171 frames, 5–12 px, sans
ennemi figé ; la tourelle (stationnaire, aggro 400) est le cas diagnostiqué avant (D66 partie 1).
Le collider (20 px) ne chevauche pas : seul le sprite de 32 px déborde ; c'est visuel (`EnemyNeverInWall` vert).

## 3. Règle retenue et pourquoi

`ai.sight_range` (donnée, chaîne `Fixed`, lint `>= 0`, défaut = `aggro_range`) : un joueur au-delà de
`aggro_range` mais à moins de `sight_range` est ciblé s'il est en **ligne de vue dégagée**
(`FlowFieldCache::line_of_sight` : segment contre `wall_cells`, parcours exact des cases en `FixedWide`,
sans flottant ; deux cases murées de part et d'autre d'un coin exact coupent aussi ; extrémités exclues ;
fenêtres laissent voir ; tie-break par `net_id` inchangé). Un ennemi déjà `Chasing` vers un joueur le
garde tant qu'il est à moins de `sight_range` (sinon le champ de flux qui contourne un mur lui ferait perdre
sa cible). Mémoire = `EnemyTarget`, déjà rollback : aucun état nouveau.

Pourquoi pas `behaviors::Sight`/`Hearing` : `Sight(r)` est un rayon seul qui se compile vers `aggro_range`
(il n'a pas de ligne de vue) ; `Hearing` ne vaut que pour les ennemis à `BehaviorRuntime` (le chien et le rat
n'en ont pas) et ne réagit qu'au tir. Changer le sens de `Sight` aurait bougé `drifter` (testbed). Un champ
séparé laisse tout le contenu existant à l'identique. `throne/rodeur` : `sight_range: "900.0"` (seul changement
de contenu ; les autres mêlées de throne approchaient déjà seuls).

`EnemyAiConfig` : champ statique, exclu du `Hash`, `Debug` manuel qui ne l'écrit que s'il est posé (le dump
§5 d'un ennemi sans vue directe est identique).

## 4. Traces

Aucune trace hors throne ne bouge. Sept traces throne bougent (le rodeur de `cave:niveau_1` fonce au lieu
d'errer) : `throne_solo`, `throne_progression`, `throne_floor_1`, `throne_mutation_choice`, `throne_quad`,
`throne_three_floors`, `throne_duo_defaite`. Les repères des `.ron` sont recalés sur les frames mesurées
(`throne_solo` : niveau 1 f506, étage 1 f1131, étage 2 f3399 ; `throne_duo_defaite` : mort f579 au lieu de
f732). Trois tests de `crates/scenario/tests/expectations.rs` recalés (HUD niveau 1 à f515, divergence
du report de choix f505) ; `throne_recorded_pillard_leaves_the_wall_corner` retire `sight_range` des fiches
(entrées enregistrées = monde historique).

Preuve §5 (`throne_duo_defaite`, dumps `.full`, `scripts/trace-diff.py`) :
- main 0a719425 vs branche **sans** `sight_range` (rodeur.ron de main) : « identique (999 frames comparées) » ;
- branche sans vs avec : première frame qui diffère **0**, uniquement `GgrsNetId(90, "rodeur")`
  (`EnemyTarget` None → Some(joueur 79), `MonsterState` Idle → Chasing, `BehaviorRuntime` règle 2 → 1,
  `Velocity`, `FixedTransform3D`, `FacingDirection`) et `RngStreams` (le flux `behaviors` n'est plus tiré
  puisque `Wander` n'est plus retenu). Cause unique : le rodeur voit le joueur dès f0.
Nouveaux scénarios (3) et leurs traces ajoutés, caverne `essai_vue` (`transit: true`, jamais jouée en run).

## 5. Vérifications

Sur la fusion avec `origin/main` 64ec961c (suite verte sur la fusion) :
- `make test_scenarios_par SHARDS=3` : 500 scénarios fusionnés, 3 tranches ok (344 s) ;
- `make test_crates` : 1014 passés, 0 échec (dont `line_of_sight` ×2 dans `navigation.rs`, `expectations` 50/50) ;
- `make lint` ×5 : zombies, testbed, throne, gungeon, isaac « aucune erreur » ;
- `make gen` ×5 : tous sortie 0, aucune trace de scénario généré bougée ;
- `cargo fmt --all -- --check`, `check_sim_boundary` (11 crates ECS seul, 5 en dette D69), `check-forbidden.sh`,
  `check-rollback-registration.sh`, `make test_tools` (18 tests) : sortie 0 ;
- fuzz `ALACOD_FUZZ=0:8` : ok ;
- p2p `scripts/p2p-stress.sh throne` 3000 frames (jeu `throne`, carte `cave:niveau_1` avec rodeur) : client 0 = 4890 frames
  resimulées, client 1 = 16, DesyncDetected 0, traces identiques (2999 lignes comparées sur 2999), « p2p-stress : OK ».

## 6. Non fait / limites

- William doit confirmer en jeu que « coincé » ne reste pas d'un autre cas : le sprite 32 px qui longe un mur
  (5–12 px visuels) subsiste sur d'autres ennemis (`nav_stats`), sans effet sur le collider.
- Le fuzz couvre zombies/testbed, pas la vue directe de throne : couverte par scénarios en synctest et par le p2p throne.
