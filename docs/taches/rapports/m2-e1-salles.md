# Rapport m2-e1-salles — salles typées et verrouillées (M2-E1, voie B)

Branche `m2-e1-salles` (base `7f9db06`), b1. Point de départ : prototype `m1-proto-etage-salles`
(repris proprement). Convention : `docs/conventions.md` §35.

## 1. Fait

- **Contrats** (`crates/world/src/rooms.rs`, sans rendu) : `RoomKind` (composant statique du
  niveau), `RoomKindDef`/`RoomKindTable` (types de salles du jeu), `RoomState`
  {Dormant, Locked, Cleared}, `RoomStates` (rollback **neutre**), `RoomDormant` (marqueur
  rollback **neutre**), `RoomChanged` (`FrameEvents` neutre), `entrance_point`, `inside_box`
  (purs, testés). Enregistrés dans `WorldPlugin`.
- **Contenu** : kind de dossier `Room` (`rooms/<id>.ron`, `(locks: bool)`), `Registry::rooms`,
  `MapEntry::room_kinds` (lu du champ de niveau `room_kind` des `.ldtk`), lint
  `lint_room_kinds` (`UnknownKind`), fixture `room_kind_unknown`. Testbed : `depart`, `combat`
  (`locks: true`), `recompense`.
- **Simulation** (`crates/map_ldtk/src/game/rooms.rs`) : `room_lock_system` (Run, après
  `floor_transition_system`) et `room_dormancy_system` (EnemySpawning). Verrouillage quand un joueur
  debout est à > 24 px des bords et qu'un ennemi vit dans la salle ; réouverture à salle vide ;
  jamais de porte refermée sur un occupant (tout est différé tant qu'un joueur/ennemi chevauche une
  porte) ; coop : les joueurs hors de la salle sont téléportés à l'entrée (40 px dans la salle,
  porte la plus proche du déclencheur, écart de 14 px). Ennemis dormants : `Without<RoomDormant>`
  sur `enemy_target_selection`, `behavior_select_system`, `update_enemy_targets`, `move_enemies`,
  `enemy_attack_system`.
- **Scénarios** : attente `RoomState(kind, nth, state, doors_closed)`, moment clé `room`,
  sonde `map_probe` étendue (salles, ennemis). Cartes `salles.ldtk` (dummy) et
  `salles_poursuivants.ldtk` (follower) générées par `m2-e1-salles.make_salles.py`. Scénarios
  `rooms_lock_solo` (verrou f85, nettoyée f479, 4 portes), `rooms_dormant` (ennemis immobiles à
  192 px puis réveil), `rooms_lock_coop` (joueur 1 téléporté en (-776, -576)), `rooms_door_occupied`
  (verrouillage différé tant que le joueur 1 est dans l'embrasure).

## 2. Vérifié

- `make test_scenarios` complet **sans BLESS** : vert (EXIT 0), aucun `.trace` existant modifié
  (`git status` : aucun `.trace` en M) ; seules les 4 traces nouvelles sont ajoutées. Les budgets fps
  sont tous en avertissement (machine partagée avec b0, ~15-25 fps) : sans valeur.
- `cargo test` de `content` + `world` : 84 + 82 + 18 réussis, 0 échec (dont la fixture).
- `make lint` sans erreur ; `check-rollback-registration` OK ; `check-forbidden` : 4 avertissements
  préexistants (HashSet ×3, rand ×1), rien de nouveau.
- Preuve §5 (dump main vs branche) **non faite** : aucune trace n'a bougé, donc pas de bless d'un
  scénario existant.

## 3. Non fait / incertain

- `fuzz_inputs` (`ALACOD_FUZZ=0:8`) non lancé.
- p2p à deux clients, bench au calme, vidéos : non vérifiés ici.
- Rendu des portes (sprites ouvert/fermé) non regardé en vidéo ; la logique ne dépend que du
  collider.

## 4. Dettes / questions

- Portes d'une salle repérées par la position (marge 24 px), pas par un lien explicite : à
  remplacer par un lien porte → salle dans E2.
- `Floors` ne remet pas `RoomStates` à zéro au changement d'étage.
- `room_lock_deferred` jamais levé si un ennemi dormant est posé dans une embrasure (non traité).
- Adaptation des défauts de l'assembleur (choix « aléatoire », chevauchement) : E2.

## 5. État final des tests

- `cargo test` de run, combat, game, content, map_ldtk, sim_core, stats, bots, effects, world :
  513 réussis, 0 échec, 1 ignoré.
- `cargo test` de `scenario` (lib + expectations, cave, cave_floors, determinism, floors, generated,
  hunter_doors, placement, run, softlock, spawn_stall, bots ; `scenarios` déjà couvert par
  `make test_scenarios`) : 79 réussis, 0 échec. `fuzz_inputs` (`ALACOD_FUZZ`) non lancé.
- `cargo fmt --all -- --check` : rien à signaler.
- Incident machine : disque plein (167 Go/197 après nettoyage) pendant une première tentative ; j'ai
  supprimé les anciennes générations de binaires de test de MON target (21 Go). Les autres
  worktrees (`m1-bots-apres-movement-feel` 56 Go, `m1-d55-coin-de-mur` 35 Go) pèsent lourd.
