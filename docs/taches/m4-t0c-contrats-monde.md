# m4-t0c-contrats-monde — étage en grille, bombes, rochers, piques, portes fermées (M4, vague 0)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Numérotation des entités », champ de flux),
`docs/taches.md` §8.1, `docs/conventions.md` §1 (LDtk), §4, §21 (terrain destructible : `CellGrid`,
`destroy_terrain`), §26 et §46 (surfaces, fosses, barils), §35 (salles), §40 (grammaire d'étage), §48
(coffres et clés), et le code `crates/world/src/` (`grid.rs`, `destroy.rs`, `surface.rs`, `props.rs`,
`rooms.rs`, `floor_grammar.rs`), `crates/map/src/generation/floor.rs` (`plan_floor`, `validate_plan`),
`crates/map/src/generation/context.rs` (connexions), `crates/map_ldtk/src/generation/to.rs`
(`close_unused_openings`), `crates/game/src/props.rs` (`barrel_explosion_system`),
`crates/combat/src/actors.rs` (bits d'input). Branche `m4-t0c-contrats-monde` depuis `origin/main`.
Estimation : 3 j. Section de conventions réservée : **§62**.

## But

Poser les contrats dont dépendent M4-T4 (bombes, rochers, secret), M4-T5 (grammaire en grille) et
M4-T7 (champions qui explosent), avec un chemin minimal dans le testbed. **Aucune trace existante ne
bouge** (l'élargissement de l'usage de l'input non plus).

## Contexte (état de `main`)

- Les rochers destructibles n'existent que dans les cavernes (`CellGrid`, `CellKind::Rock`, non vide sur
  une carte `cave://` seulement) ; une salle LDtk n'a que des murs (IntGrid `Walls`) et des surfaces.
- La seule explosion est celle du baril (`barrel_explosion_system` : dégâts dans un rayon, en chaîne, ordre
  `GgrsNetId`) ; elle ne casse aucun terrain. `DestroyTerrain` (projectiles) creuse la roche des cavernes.
- La grammaire d'étage pousse un **arbre** de gabarits par connexions alignées ; pas de grille, pas de
  boucle, pas de salle secrète. Tous les gabarits gungeon font 8×8 cases, mais rien ne l'impose.
- Les portes de `zombies` coûtent des points (`DoorConfig.cost`) ; une clé n'ouvre qu'un coffre.
- Bits d'input 0 à 17 pris (`INPUT_BLANK` = bit 17) ; `BoxInput.buttons` est un `u32`.

## À faire

1. **Action `Explode`** (`effects::Action`, fin d'enum) : `Explode { radius, damage, destroys: bool }`.
   Règle pure dans `world` ou `combat` (au choix, justifier) qui liste les cibles (personnages à PV dans le
   rayon, centre à centre, ordre `GgrsNetId`) et, si `destroys`, les cases à casser. Le baril **ne bascule
   pas** sur cette action dans cette tâche (sa trace ne doit pas bouger) : il la réutilisera en T4 avec
   preuve. Lint : rayon > 0, dégâts ≥ 0.
2. **Bombe** : bit `INPUT_BOMB` (18), bouton `Bomb` dans le format des scénarios (`replay.rs`), touche E
   (présentation, Q17 pour la manette). Composant rollback neutre `Bomb { owner: GgrsNetId, fuse_until:
   u32, radius, damage }` ; définition en données dans le personnage (`bomb: Some((fuse_frames, radius,
   damage))`, absent = pas de bombe). Contrat : types, bouton, lecture de l'input **sans effet** tant que le
   personnage n'a pas `bomb` (aucun contenu existant ne l'a). Exécution en T4.
3. **Rochers sur les salles** : couche IntGrid optionnelle `Rocks` dans un gabarit LDtk (valeur 1 = rocher,
   2 = rocher à butin : Q9 pour le nom). Proposition technique par défaut : remplir le `CellGrid` existant
   (`CellKind::Rock`) pour une carte assemblée qui a au moins une case `Rocks`, pour réutiliser
   `destroy_terrain`, la reconstruction des murs et la mise à jour du champ de flux des cavernes ; une carte
   sans `Rocks` garde un `CellGrid` vide (trace inchangée). Si la lecture du code montre qu'une entité par
   rocher est plus simple (collider et `Obstacle`), le dire et le justifier au rapport. Contrat : lecture de
   la couche, `CellGrid` rempli, scénario `rocks_static` (testbed : un joueur ne traverse pas un rocher,
   `joueur_hors_mur` tient, attente `CellState`).
4. **Piques** : champ optionnel `hazard: Option<HazardDef { damage, period }>` de `SurfaceDef` (fin de
   structure, hors hash quand absent) ; dégâts `True` ou typés (Q2 pour les demi-cœurs) au joueur dont les
   pieds sont sur la case, une fois par `period`. Contrat : forme RON, lint, règle pure ; exécution en T4.
5. **Portes fermées** : `world::DoorLock { Key(u32), Secret }` porté par un type de salle
   (`RoomKindDef.door_lock: Option<DoorLock>`, hors hash quand absent). `Key(n)` : la porte de la salle reste
   fermée jusqu'à ce qu'un joueur la touche par Interaction avec `n` clés (consommable `KEY_ITEM` de
   `game::loot`, consommées à l'ouverture) ; `Secret` : l'ouverture est un mur plein jusqu'à une explosion (`destroys`) qui le touche.
   Contrat : types, lint (type de salle inconnu, `Key(0)`), état rollback neutre de la porte ouverte ou non ;
   exécution en T4 (secret) et T5 (placement).
6. **Disposition en grille** : champ optionnel `layout: Option<FloorLayout>` de `FloorGrammar`,
   `FloorLayout { Graph (défaut, le planificateur actuel), Grid { cell: (u32, u32), max: (u32, u32) } }`.
   Contrat : types, lint (tous les gabarits de la grammaire font exactement `cell` cases, ou un multiple si
   Q8 retient les grandes salles ; chaque gabarit a une ouverture centrée sur chaque côté), et un
   `plan_floor` qui **refuse** `Grid` proprement (`FloorError::Unsupported`) tant que T5 n'est pas livrée.
   Taille de case : donnée, **Q8**.
7. **Types de salle réservés** (contenu testbed seulement, aucun dans `gungeon`) : `secrete`, `tresor`,
   `diable` comme fichiers `rooms/*.ron` d'exemple pour le lint ; leur sémantique arrive en T4, T5, T8.
8. **Attentes et événements** : `DoorLocked(room, bool, at_frame)`, `BombsHeld(handle, n)` si
   `Consumable(handle, "bomb", n)` ne suffit pas (le préférer) ; moment clé `explosion` (déclaré, émis en T4).
   Mettre à jour la liste de `CLAUDE.md` (bloc « Attentes », ligne M4).

## Règles

- Aucune trace existante ne bouge ; preuve : `make test_scenarios` vert sans `BLESS`, et
  `recording_replays_identically` vert (le bouton `Bomb` est un nom de plus dans le format).
- `fuzz_inputs` (`ALACOD_FUZZ=0:8`) : le bit 18 n'a aucun effet sans `bomb` ; le dire au rapport.
- Ordre de création des entités rollback (rochers, portes) trié par clé de contenu, `.after(MapNetIdAssignment)`.
- Pas de `f32`, `order_iter!`, `BTreeMap`, `FrameEvents`, `despawn_rollback()`.
- `world` reste dans la frontière `SIM` (`make check_sim_boundary`).

## Hors périmètre

Pose et explosion des bombes, rochers cassés, piques en jeu, mur secret ouvert (M4-T4) ; planificateur en
grille, placement de la salle secrète, du trésor et de la salle post-boss (M4-T5) ; HUD et visuels (T16,
T17).

## Vérification

README §4 complet, plus `fuzz_inputs`. Rapport `docs/taches/rapports/m4-t0c-contrats-monde.md`,
conventions §62, `CLAUDE.md` (bits d'input : ajouter le bit 18 à la phrase « bits 16/17 »).

## Atteignabilité

Contrats : rien n'est rencontré en partie ordinaire. T4 et T5 l'exécutent, T11d place rochers, piques et
salles dans les gabarits et la grammaire de `isaac`.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : p2p rejoué par l'orchestrateur (l'input change d'usage : à rejouer), le dire au rapport. Pousser la
branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le
commit `LIVRÉ m4-t0c-contrats-monde`, qui contient le rapport.
