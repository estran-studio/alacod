# Rapport — m1-v1d : terrain destructible et cavernes (T1.0b + T1.6)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport) ; base `origin/main`
`9ee2be3` (merges de main : `92e4754`, `d9a1ef3` avec T1.2, puis `9ee2be3`, documentation
seulement — les vérifications ci-dessous portent sur le code de `387ed0f`, inchangé depuis).
Fiche : [m1-v1d-terrain-destructible](../m1-v1d-terrain-destructible.md). Branche
`m1-v1d-terrain-destructible` (depuis `d3d25f3`, tête livrée de m0-v7 phase 2) ; worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ;
dates : 2026-10-03/04.

## État en cours

- **Fait** : tout le périmètre de la fiche (voir « Fait » ci-dessous).
- **Chiffres** : 87 traces existantes identiques sans bless ; 4 nouvelles traces à bénir ;
  86 destructions dans `bench_cave`, 62,7 fps simulés sous charge.
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **Crate `world`** : `CellKind { Floor, Wall, Rock }`, `CellGrid` (cases de 16, origine
  (0, 0), +y vers le haut, Debug compact en rangées), marqueur `Destructible` ;
  `cave::generate` (automate cellulaire sur `RollbackRng`, bordure `Wall`, plus grande
  composante 4-connexe gardée, ratio de sol `[min_floor_ratio, 0.9]` par essais successifs du
  même RNG) ; `points_of_interest` (spawns sur cases **dégagées**, voir Diagnostic) ;
  `destroy_terrain` ; `WorldPlugin` : `CellGrid` et files `DestroyTerrainRequest` /
  `TerrainDestroyed` rollback, **checksum neutre** (vide = 0).
- **`RollbackSystemSet::World`** entre `Projectiles` et `CollisionDamage` (avant `EnemyAI`).
- **sim_core** : `add_frame_events_neutral` (file vide = 0 au checksum).
- **Contenu** : kind `Cave` (`caves/<id>.ron` + `gabarit.ldtk`), désignation unique
  `cave:<id>` (start_map, `Scenario.map`, `--map`, `levels` Floors), lint (plages, gabarit,
  personnages, `cave:` inconnu). Champ optionnel `characters` (un `CharacterSpawn` par
  `ZombieSpawn`). Testbed : cavernes `petite` et `bench`, séquence Floors `caverne`, armes
  `grenade_creuse` (`on_expire`) et `foreuse` (`on_hit`).
- **Chemin LDtk** : `MapGenerationMode::Cave(CaveConfig)` ; le callback du loader réécrit le
  niveau unique du gabarit (`map_ldtk::generation::cave`) ; `cave:<id>` résolu au chargement
  (`resolve_map_config`, `CaveSlots`) ; `MapGenerationConfig` garde la désignation (rejeu des
  enregistrements). `CellGrid` remplie au `LdtkMapLoadingEvent` et au passage de niveau Floors.
- **Destruction** : `effects::Action::DestroyTerrain { radius }` (dernière variante) ;
  `ExpireAction::DestroyTerrain` (Hash manuel, voir Diagnostic) ; `ProjectileWallHit` (file
  neutre, émis dans `register_wall()` pour un projectile à `on_hit`) ; les deux posent des
  `DestroyTerrainRequest`. Après une destruction : murs `despawn_rollback` puis recréés depuis
  la grille (`cave_wall_<frame>_<i>`), `FlowFieldCache::reload_walls`.
- **Attente `CellState(x, y, kind, at_frame)`** ; moment clé `terrain` et métrique
  `terrain_destroyed` des scénarios.
- **Présentation** : un carré par case solide lu dans `CellGrid` (rendu seulement).
- **Docs** : `docs/conventions.md` §21 ; liste des attentes de `CLAUDE.md`.

## Diagnostic et décisions

- **Neutralité des traces.** Toute la nouvelle mécanique est inerte hors caverne : ressource et
  files neutres, événement mur émis seulement pour un projectile à `on_hit`, variantes ajoutées
  en dernier. Piège trouvé par `make gen GAME=testbed` : `#[derive(Hash)]` d'un enum à **une
  seule** variante n'écrit pas le discriminant ; ajouter `ExpireAction::DestroyTerrain`
  déplaçait le checksum de l'arme `grenade` (config hachée dès f0, trace `weapon_grenade`
  « différente » ligne 1). `ExpireAction` a désormais un `Hash` manuel où `Spawn` garde son
  hash historique.
- **Rollback de la navigation.** Une première version gardait « la dernière grille appliquée »
  dans une ressource hors rollback : le synctest l'a refusée (désync à la frame de la
  destruction). `FlowFieldCache` est rollback ; le rechargement se fait uniquement quand la
  frame a creusé, et le rejeu le refait (test `destruction_creuse_et_reconstruit_les_murs_en_synctest`).
- **Gabarit LDtk.** Les tilesets d'un `.ldtk` sont relatifs au fichier : copié dans `caves/`,
  le gabarit pointait vers `caves/atlas/...` inexistant et le niveau ne se chargeait jamais,
  sans erreur. Chemins réécrits en `../testbed/atlas/...` (documenté au §21).
- **Points d'intérêt.** Les `ZombieSpawn` « les plus éloignés » tombaient au fond d'impasses :
  le corps (20 × 20, décalé vers le bas) chevauchait la roche et chaque déplacement était
  refusé (followers immobiles). Spawns choisis parmi les cases dont les 8 voisines sont du sol.
- **Flow field incrémental** : non fait, `bench_cave` tient son budget (voir Vérifié).
- **Rejeté** : un collider par case (des milliers d'entités rollback au checksum).

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=4`, profil headless.

- `cargo test -p world` : 8 réussis, dont **1 000 graines** (sol connexe, bordure `Wall`, ratio
  dans `[0.3, 0.9]`, spawns sur cases dégagées), déterminisme (même graine = même grille,
  100 graines distinctes), rayon strict et `Wall` intact.
- `cargo test -p scenario --test cave` : 7 réussis — caverne chargée par le chemin LDtk
  (grille = grille générée, murs, joueur sur le sol), `CellState` (succès et échecs),
  destruction en **synctest** sans désync, caverne en `Floors` (floor_a → cave → floor_b),
  followers de `cave:bench` qui naviguent, `foreuse_creuse_au_contact` (on_hit),
  `bench_cave_detruit_au_moins_50`.
- `make test_scenarios` (sans bless, tête `ec551e6`) : **87 traces existantes identiques** ;
  échecs uniquement « pas de trace de référence » pour `explode_wall`, `bench_cave`,
  `weapon_grenade_creuse` ; aucune attente en échec. Revérifié sur la tête finale par le
  passage `scenarios` des tests de crates (ci-dessous), `weapon_foreuse` compris.
- `explode_wall` : `CellState` Rock → Floor sur (26, 15) et (27, 15), bordure (47, 15) intacte.
- `bench_cave` : **86 destructions**, **62,7 fps simulés** sous charge (200 graines de
  l'orchestrateur en parallèle, load 18-21) — le bench strict au calme reste à faire ;
  plancher 40 dans `tests/budgets.ron`.
- `make gen GAME=zombies` : code 0, aucun fichier modifié. `make gen GAME=testbed` : toutes
  les armes existantes `ok`, `grenade_creuse` et `foreuse` « absente » (nouvelles).
- `make lint` : `zombies` et `testbed` sans erreur. `cargo fmt --all -- --check` : vide.
- `./scripts/check-forbidden.sh` : 4 occurrences préexistantes, aucune nouvelle.
  `./scripts/check-rollback-registration.sh` : OK.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk
  -p map -p sim_core -p stats -p bots -p effects -p behaviors -p world -p utils --no-fail-fast`
  (tête `387ed0f`) : **412 réussis, 1 échec, 9 ignorés** (45 blocs). L'échec est le test
  `scenarios`, uniquement « pas de trace de référence » pour les 4 nouveaux scénarios : les
  87 traces existantes y sont identiques sur le code final.

## Non fait / non vérifié

- Bench strict au calme (`ALACOD_BENCH_STRICT=1`) : la machine est chargée toute la nuit.
- Rendu des cavernes non vérifié à l'écran (pas d'affichage ici) ; code compilé seulement en
  headless (`RENDER_ENABLED` faux).
- Pas de test p2p.
- Traces nouvelles non bénies (`explode_wall`, `bench_cave`, `weapon_grenade_creuse`,
  `weapon_foreuse`) : à l'orchestrateur.

## Dettes / questions ouvertes

- Flow field incrémental (E3) : inutile tant que `bench_cave` tient son budget.
- Une caverne du testbed sans `characters` n'a pas d'ennemi : son portail `Floors` s'ouvre
  aussitôt (et en boucle au dernier niveau).
- `on_hit: [DestroyTerrain]` sur un personnage touché ne creuse pas (seulement sur un mur).
