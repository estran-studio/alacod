# Rapport — m1-v1d-surfaces : surfaces v1 (T1.7, chantier E4 v1)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport) ; base `origin/main`
`9ee2be3` + T1.6 (`53e3acb`, livrée, pas encore poussée sur main au moment de la livraison).
Fiche : [m1-v1d-surfaces](../m1-v1d-surfaces.md). Branche `m1-v1d-surfaces` ; worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date :
2026-10-04 (nuit).

## État en cours

- **Fait** : tout le périmètre de la fiche (voir « Fait »).
- **Chiffres** : 87 traces existantes identiques sans bless ; 4 nouvelles traces T1.7 à bénir
  (plus les 4 de T1.6) ; vitesses mesurées 1,25 / 2,0 / 2,5 px/frame.
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **world** : `SurfaceGrid` creuse (`BTreeMap<(i32, i32), SurfaceId>`, cases de grille monde,
  sans origine imposée), rollback, **checksum neutre** ; `SurfaceTable` (hors rollback) ;
  `surface_modifiers` : facteurs abstraits `move_speed`/`acceleration` → `MoveSpeed` +
  `Acceleration` (joueur), `EnemyMoveSpeed` (ennemi au sol), rien pour un volant ; facteur 1 =
  aucun modificateur.
- **content** : kind `Surface` (`surfaces/<id>.ron` : `intgrid_value`, `tags`, `move_speed`,
  `acceleration` optionnelle), lint (`intgrid_value` > 0 et unique, facteurs > 0, tags non
  vides) et fixtures `surface_duplicate_value`, `surface_factor_non_positive`.
- **map_ldtk** : couche IntGrid optionnelle `Surfaces` lue avec les murs, au chargement et au
  passage de niveau `Floors` (`surface_grid_of_levels`) ; `SurfaceTable` posée depuis le
  registre au chargement.
- **game** : `surface_modifiers_system` dans `Input`, avant `apply_inputs` : case sous les pieds
  = centre du collider + offset ; modificateurs de source `Named("surface")` comparés aux
  voulus, remplacés seulement s'ils diffèrent, sans `until` ; condition d'exécution
  `surfaces_active` (grille non vide ou qui vient de changer) : coût nul sans surface.
- **Attente** : `CellState` avec `kind` optionnel et champ `surface` (`"aucune"` = sans surface).
- **Testbed** : surfaces `eau` (×0,5), `sable` (×0,8), `glace` (accélération ×0,2) ; cartes
  `testbed/surfaces.ldtk` (couloir A à bandes, couloir B nu) et `testbed/surfaces_enemy.ldtk`
  (deux couloirs, un breacher chacun, eau dans A).
- **Scénarios** : `surface_walk`, `surface_none`, `surface_ice`, `surface_enemy`.
- **Docs** : `docs/conventions.md` §26 (+ renvoi depuis §21), champ `surface` dans `CLAUDE.md`.

## Décisions (fiche, amendements validés par l'orchestrateur)

- `SurfaceId` = la valeur IntGrid elle-même (pas de numérotation séparée).
- Changement de surface détecté sans état nouveau : comparaison des modificateurs présents et
  voulus à chaque frame.
- `CellState.kind` devient `Option` (RON en `implicit_some` : `kind: Rock` reste valide) : sans
  ça, `CellState` échouerait sur une carte LDtk à surfaces (`CellGrid` vide).
- `surface_enemy` : `NoDamageBetween` + `Event(hit)` (pas d'attente `EnemyDistance` avant
  T1.4).

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=3`, profil headless.

- **Scénarios** (`make test_scenarios`, sans bless, tête `c3a1692`) : 87 traces existantes
  identiques ; échecs uniquement « pas de trace de référence » pour les 8 nouveaux scénarios
  (4 de T1.6 non encore bénis, 4 de T1.7). Aucune attente en échec.
- `surface_walk` : sol 2,5 px/frame (f90 x -454.3) ; eau 125 px en 100 frames (f110 -425.6 →
  f210 -300.6, **rapport 0,5**) ; sable 80 px en 40 frames (f250 -235.2 → f290 -155.2,
  **rapport 0,8**) ; `CellState` des bandes (origine de carte x = -624). `surface_none` : même
  input dans le couloir nu, 2,5 px/frame constants (f290 x 45.6).
- `surface_ice` : demi-tour à f315 ; sur le sol, le joueur 1 s'arrête vers f375 (x ≈ 194) ; sur
  la glace, le joueur 0 glisse encore de 26 px entre f360 et f375 et ne s'arrête que vers f425.
- `surface_enemy` : breacher du couloir nu au contact à **f338**, celui qui traverse l'eau à
  **f543** (+205 frames, 240 unités à mi-vitesse).
- Tests unitaires : `world` (grille creuse à origine quelconque, traduction des stats, neutre à
  vide), `game::character::surface` (remplacement exact, rien si inchangé, volant ignoré,
  pieds = centre du collider + offset), fixtures de lint ; lecture de la couche LDtk à origine
  non nulle vérifiée par les `CellState` de `surface_walk`.
- **Benchs sous charge** (200 graines de l'orchestrateur et une autre tâche, load 9-17) :
  `bench_cave` 108,5 fps ; `bench_bullets` 75,3 ; `bench_horde` 24,1 dans la suite. A/B
  `bench_horde` : 21,3 / 21,2 fps avec le système, 23,8 / 22,0 sans, 17,0 / 19,4 avec la
  condition d'exécution (coût nul) : les écarts suivent la charge. Le plancher de
  `bench_horde` (38) n'est atteint dans aucune variante cette nuit ; mesure stricte au calme à
  faire.
- CRATES_PLACEHOLDER

## Non fait / non vérifié

- Bench strict au calme (`ALACOD_BENCH_STRICT=1`).
- Rendu : la couche `Surfaces` des cartes générées n'a pas de tuiles (rien à l'écran) ; pas de
  vérification visuelle.
- Pas de test p2p. Pas d'`EnemyDistance` (T1.4 pas encore dans main).
- Traces nouvelles non bénies (à l'orchestrateur).

## Dettes / questions ouvertes

- v2 : dash et friction par surface, coût de flow field par surface, dangers, surfaces de
  caverne (`CaveConfig.surfaces`).
- `bench_horde` hors plancher sous charge, comme toute la nuit : à remesurer au calme.
