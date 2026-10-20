# M4-T0c : contrats du monde (bombes, rochers, piques, portes verrouillées, grille)

Branche `m4-t0c-contrats-monde` (depuis `origin/main` 94ab9854, puis fusions de main). Conventions :
`docs/conventions.md` §62. Questions Q1 à Q18 : propositions par défaut (décision de William). Q8 :
salles de 15 × 9 cases, une seule taille. Q9 (nom de la couche) : `Rocks`. **Contrats seuls** : aucune
exécution, aucune trace existante ne bouge.

## Ce qui est livré

| Brique | Où | Contrat |
|---|---|---|
| `Action::Explode { radius, damage, destroys }` | `effects::actions`, `describe` | fin d'enum, hashs d'avant intacts ; lint rayon > 0, dégâts ≥ 0, `Unsupported` dans un effet (T4), refusée en power-up |
| Règle d'explosion | `world::explosion` (`explosion_plan`, `within_radius`) | cibles à `< radius`, ordre `GgrsNetId`, `destroys` = `destroy_terrain` ; dans `world` (frontière SIM, comme `destroy_terrain`) |
| Bombe | `INPUT_BOMB` (bit 18), `Button::Bomb`, touche E, `CharacterConfig.bomb`, `world::{BombDef, Bomb}` | `Bomb` rollback neutre ; `bomb_requested` faux sans `bomb` ; lint `fuse_frames > 0`, rayon > 0, dégâts ≥ 0 |
| Rochers de salle | couche IntGrid `Rocks` (1 rocher, 2 rocher à butin), `world::Rock`, `map_ldtk::game::collider::spawn_level_rocks` | **une entité par rocher** (écart à la proposition, voir ci-dessous) |
| Piques | `SurfaceDef.hazard: Option<HazardDef { damage, period }>` | champ de fin de structure, `is_due`, lint `damage > 0`, `period > 0` |
| Portes verrouillées | `RoomKindDef.door_lock: Option<DoorLock { Key(n), Secret }>`, `world::DoorLocks` (neutre) | lint `Key(0)` ; types réservés `secrete`, `tresor`, `diable` dans le testbed |
| Grille | `FloorGrammar.layout: Option<FloorLayout { Graph, Grid { cell, max } }>` | lint de grammaire (taille, ouverture centrée par côté, capacité) ; `plan_floor` renvoie `FloorError::Unsupported` |
| Attentes / événements | `DoorLocked(kind, nth, locked, at_frame)`, `EntityCount(kind: Rock)`, moment clé `explosion` (`world::Exploded`) | `BombsHeld` non ajoutée : `Consumable(handle, "bomb", n)` suffit |
| Carte et scénario | `testbed/rocks_static.ldtk` (`scripts/construire-cartes-t0c.py`), `rocks_static` | joueur arrêté par 3 rochers à x ≈ 294 (rocher à 304), 4 rochers comptés à f5 et f219 |

## Écart : un rocher est une entité, pas une case de `CellGrid`

La fiche proposait de remplir `CellGrid` (`CellKind::Rock`) pour réutiliser `destroy_terrain`. La
lecture du code l'exclut sur un étage assemblé : `CellGrid` est dense, ancrée à l'origine et non
négative ; `rebuild_cave_walls_system` détruit **tous** les `Wall` rollback pour les recréer depuis la
grille et `FlowFieldCache::reload_walls` remplace les cases murées par celles de la grille. Sur des salles
placées n'importe où, avec leurs propres murs IntGrid, la première explosion effacerait les murs des
salles. Une entité `Rock` (collider de la couche des murs + `Obstacle::Wall`) est vue par le champ de flux
sans code de plus, se détruit par `despawn_rollback`, et une carte sans la couche ne crée aucune entité
ni aucun `GgrsNetId` (traces inchangées). `world::explosion_plan` garde le volet `CellGrid` pour les
cavernes ; T4 ajoutera les rochers-entités à ses candidats.

## Preuves

- `make test_scenarios_par SHARDS=4` vert **sans BLESS** (seule `rocks_static.trace` est nouvelle) ;
  `recording_replays_identically` vert (le bouton `Bomb` est un nom de plus dans le format).
- `make test_crates`, `make lint`, `cargo fmt --check`, `make check_sim_boundary`, `make gen GAME=gungeon`
  (sans diff) verts. Fixtures de lint : `explode_radius_zero`, `door_lock_key_zero`,
  `hazard_period_zero`, `bomb_fuse_zero`.
- `fuzz_inputs` (`ALACOD_FUZZ=0:4`, 3600 frames) sans divergence ; le bit 18 n'a aucun effet (aucun système ne le lit, aucun personnage n'a `bomb`).
- Contrôle de `rocks_static` : départ à x = 168, arrêt à 293,8 de f100 à f215 (rocher de 304 à 320,
  demi-collider du joueur 10) ; l'invariant `joueur_hors_mur` tient.

## Limites

- p2p non rejoué ici : l'input change d'usage (un bit de plus), à rejouer par l'orchestrateur.
- `cargo clippy` n'est pas installé sur le nightly de la machine : non vérifié.
- Rien n'est rencontré en partie ordinaire (atteignabilité : T4 et T5 exécutent, T11d place).
