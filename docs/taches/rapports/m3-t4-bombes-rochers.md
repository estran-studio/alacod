# M3-T4 : bombes, rochers, piques, mur secret

Branche `m3-t4-bombes-rochers` (depuis `origin/main` 803bf788, puis fusion de `main` `af46f192` : M3-T6 familiers).
Conventions : `docs/conventions.md` §69. Fiche : `docs/taches/m3-t4-bombes-rochers.md`.

## Ce qui est livré

| Point de la fiche | Où | Comment |
|---|---|---|
| 1. Bombe posée par `INPUT_BOMB` | `game::bombs::bomb_place_system`, `world::{Bomb, BombCarrier}` | front montant (`held` réécrit à chaque frame), consommable `bomb`, aux pieds du poseur ; mèche en frames ; `BombDef.friendly_fire` (défaut `Always`, Q4) |
| 1. `Explode` exécuté, crédit au poseur | `bomb_fuse_system`, `explosion_system`, `effects_runtime` (action `Explode`) | une `ExplosionRequest` par explosion, triée par `GgrsNetId` de l'objet ; dégâts `Physical` tag `explosion`, source = le poseur ; `Exploded` (moment clé `explosion`) |
| 1. Respect de l'invulnérabilité | `explosion_system` | `Health::is_invulnerable_at` filtre les cibles (familiers de M3-T6 : `u32::MAX`) ; test unitaire |
| 2. Baril sur `Explode` | `explosion_system` (l'ancien `barrel_explosion_system` est supprimé) | un baril mort émet sa demande dans le même système ; **`barrel_chain`, `bench_floor` et toutes les traces à barils inchangées** |
| 3. Rochers cassés, flux à jour | `explosion_terrain_system` | `despawn_rollback()` des `Rock` du rayon (ordre `(y, x)`), le champ de flux relit les `Obstacle` ; caverne : `DestroyTerrainRequest` (`bomb_digs_cave`) |
| 3. Rocher à butin | idem | `RollbackRng` local, `loot_seed(graine, étage, "rock:<salle>:<x>:<y>")` |
| 4. Mur secret | `map_ldtk::game::rooms::secret_wall_system` | portes (et jumelles) d'une salle `door_lock: Secret` closes sans `Interactable` ; une explosion `destroys` qui touche le bord d'un collider les ouvre toutes, `DoorLocks` |
| 5. Piques | `game::bombs::hazard_system`, `world::{HazardDef::applies, HazardState}` | `True`, tag `spikes`, une fois par `period` ; `dash_crosses` (défaut faux) |
| 6. Scénarios | `tests/scenarios/` | voir la table de §69 : `bomb_place_explode`, `bomb_hurts_owner` (témoin), `bomb_breaks_rock`, `rock_loot`, `bomb_chain_barrel`, `bomb_digs_cave`, `secret_wall_bomb`, `secret_wall_closed` (témoin), `spikes_damage`, `rock_flow_field`, `rock_flow_field_blocked` (témoin) ; 11 traces neuves, bénies une à une |

Autres livrables : `PlayerScript::bomb` (scénarios), `fuzz_inputs` avec `ALACOD_FUZZ_BOMBS=1`, cartes
`spikes`/`rock_flow`/`salles_secret` (`scripts/construire-cartes-t4.py`), objet `bomb`, surface `piques`, lint
(`Explode` n'est plus refusée dans un effet), fixture `explode_effect_ok`, tests unitaires (`world::explosion`,
`world::surface`, `game::bombs`).

## Décisions et écarts

- **Demandes transitoires** : `ExplosionRequest` et `Exploded` sont `add_frame_events_transient` (T0c avait déclaré
  `Exploded` neutre) : un baril d'une partie existante en émet, une file neutre non vide aurait déplacé les traces.
- **Deux étages** (`World` avant `apply_destroy_terrain_system`, puis `CollisionDamage` à la place de l'ancien système
  des barils) : le terrain de caverne se refait dans la frame comme pour `DestroyTerrain` ; les `DamageEvent` gardent
  leur emplacement dans la frame (mêmes traces).
- **Rayon strict** (`<`, `world::within_radius`) au lieu de l'inclusif du baril d'avant : aucune scène existante ne
  place un personnage à exactement `radius` (suite sans bless).
- **`Explode` d'un effet = bombe à mèche d'une frame** posée par commande différée (le `GgrsNetIdFactory` et la position
  du porteur se lisent à l'application) : même chemin que la bombe d'un joueur, explosion à la frame suivante comme un
  baril. `OnUse` d'un objet ne l'exécute pas (D80).
- **Mur secret** : les portes d'une salle secrète ne sont plus ouvertes au premier tick ; seule `Interactable` est
  retirée. Une explosion de **baril** n'ouvre pas le mur (pas de `destroys`).
- **Jeux livrés** : aucun ne reçoit de bombes (`isaac` déplacerait ses traces : D81).
- `HazardDef.dash_crosses` et `BombDef.friendly_fire` : champs de fin de structure, absents du contenu d'avant.

## Revue indépendante : corrections

1. **Nettoyer une salle voisine n'ouvre plus le mur secret** : `room_lock_system` ignore les portes secrètes (la
   porte d'une salle `Secret` et sa jumelle) dans ses deux branches `Locked → Cleared` (`secret_door`). Scénario
   `secret_wall_after_clear` (carte `salles_secret_combat.ldtk`) : la salle de combat voisine est nettoyée (f489), ses
   portes s'ouvrent, la porte secrète reste fermée et le joueur s'arrête contre elle. Contrôle : sans l'exclusion le
   joueur traverse (x -586).
2. **Mur secret en `Floors`** : `floor_transition_system` remet `DoorLocks` à zéro (les portes renaissent fermées ;
   l'état était par `LevelId`, répété quand un étage réutilise la carte). Scénario `secret_wall_floors` (séquence
   `deux_secrets`, deux étages identiques) : fermée, ouverte par une bombe, de nouveau fermée à l'étage 1 (f306),
   rouverte par la seconde bombe. Contrôle : sans la remise à zéro, `DoorLocked` reste faux à l'étage 1 et le joueur
   ne passe plus.
3. **Salle du butin d'un rocher** : le calcul passe dans `map_ldtk::game::rooms::rock_loot_system` (seul à voir
   `CurrentRooms`, `game` ne dépend pas de `map_ldtk`) ; `explosion_terrain_system` émet un `RockBroken` transitoire.
   Test unitaire `game::bombs::room_of` : deux mondes superposés, seule la salle de l'étage courant est retenue.
4. **`Explode` d'un effet testé** : scénario `explode_effect` (personnage `kamikaze`, `Tick(40)` une charge) : une
   explosion (f47), joueur 100 → 70 (source `Enemies`), porteur 50 → 20 (aucune exemption), `dummy` hors rayon intact,
   moment clé `explosion` ; test unitaire `game::effects_runtime` : bombe à mèche d'une frame, équipe et crédit du
   porteur, `destroys`, position.

Traces bénies (nouvelles, une à une) : `secret_wall_after_clear`, `secret_wall_floors`, `explode_effect`
(scénarios neufs). Aucune trace existante n'a bougé, `rock_loot` comprise (le butin déplacé dans un autre système donne
les mêmes `GgrsNetId` et la même suite).

## Vérifié (sur la fusion avec `main` `af46f192`, après la revue ; commit de fusion `fde0036f`)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 415 scénarios verts ; **aucune trace existante modifiée**
  (seules les 14 traces neuves de la branche : les 11 scénarios ci-dessus et les trois de la revue).
  Les scénarios jouent en synctest (rollback et resimulation à chaque frame) : bombes, rochers, mur secret et
  piques passent sans mismatch.
- `make test_crates` : 939 tests, vertes.
- `make lint` : cinq jeux « aucune erreur ». `make gen GAME=zombies|testbed|throne|gungeon|isaac` : sans diff.
- `cargo fmt --all -- --check`, `make check_sim_boundary` (11 crates, 5 en dette D69), `make test_tools`,
  `./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh` : OK.
- `fuzz_inputs` : `ALACOD_FUZZ=0:8` (zombies, 3600 frames) sans divergence ; `ALACOD_FUZZ=0:8 ALACOD_FUZZ_BOMBS=1`
  (personnage à huit bombes dans `rock_flow`, huit explosions par partie) sans divergence.

## p2p à deux clients (prédiction réelle)

Rejoué après la revue (`rooms.rs` et `bombs.rs` ont changé : butin déplacé, portes secrètes), même recette, même montage : voir les chiffres « après revue » en fin de section.

Montage **temporaire non commité** (restauré par `git checkout`) : `start_map` de `games/testbed/assets/game.ron` sur une
carte `p2p_t4.ldtk` (`salles_secret.ldtk` + couche `Rocks` : trois rochers dont un à butin devant les joueurs, salle
secrète derrière) et `bomb: Some((fuse_frames: 20, radius: "40.0", damage: "10.0"))`, `starting_consumables: {"bomb": 8}`
dans le `player_config.ron` du testbed. Binaire `cargo build -p testbed --profile headless --no-default-features`.
Script d'inputs (`/tmp/p2p_t4_script.ron`, deux joueurs, 600 frames) : marche vers les rochers, `Bomb` aux frames 25-26,
90-91, 130, 200-201, 300, 420 (rochers, puis mur secret, puis en jeu libre), tir et visée qui changent toutes les 6
frames, direction qui change : chaque changement est une prédiction ratée chez le pair.

Commande (script bash, `MATCHBOX_PORT=3556 COMPOSE_PROJECT_NAME=b1`, `docker compose -f docker-compose.ci.yaml up -d signaling`),
un client lancé 8 s après l'autre :
`RUST_LOG=info ALACOD_HEADLESS=1 ALACOD_INPUT_DELAY=0 ALACOD_INPUT=script:/tmp/p2p_t4_script.ron
ALACOD_STATE_TRACE=/tmp/p2p-$i.trace ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x ../target/headless/testbed --matchbox
ws://127.0.0.1:3556 --lobby test-$$ --number-player 2 --players localhost remote --cid client_$i --name client_$i`.

- Les deux clients sortent avec le code 0 ; `cmp /tmp/p2p-0.trace /tmp/p2p-1.trace` : **identiques**, 599 lignes chacune.
- **Frames resimulées** (comptées aux `ggrs{f=N …}` en double, le compteur final du log est faux) : client 0, 212 lignes
  ggrs en double sur 107 frames distinctes ; client 1, 207 lignes sur 107 frames (frames 0 à 558 et 0 à 595) : la
  prédiction a bien raté et resimulé. Parmi elles, des poses de bombe et des explosions : frames 91, 130, 150, 220, 300,
  420 (client 0) et 130, 150, 220, 300, 420 (client 1) apparaissent deux fois dans le journal.
- **Événements** (`bomb_place`, `explosion`, `rock_broken`, `rock_loot`, `secret_wall_open`, dédoublonnés) : 44 lignes
  distinctes de chaque côté, **ensembles identiques**, aux mêmes frames. Rocher cassé : f45, trois rochers
  (`net_id` 5 à 7, dont le rocher à butin, butin `key` tiré par `rock:<salle>:7:-7`) par l'explosion de la bombe du
  joueur posée à f25 (explosion f45 puis f46) ; mur secret : `secret_wall_open` à f110 (bombe posée à f90). Les douze
  explosions vues ont lieu aux frames 45, 46, 110, 111, 150 (×2), 220, 221, 320 (×2), 440 (×2) ; les poses résimulées
  donnent des lignes en double mais aucun événement n'existe d'un seul côté.
- **Après revue** : traces identiques (599 lignes), codes de sortie 0 ; client 0 : 224 lignes ggrs en double sur 110 frames, client 1 : 124 sur 62 ; mêmes 44 lignes d'événements distinctes des deux côtés (aucun écart).
- Écarts : aucun. Limite : la prédiction ne rate que sur les inputs du pair distant ; le rocher à butin et le mur
  secret ont chacun été cassés une seule fois (par les bombes des frames 25 et 90) sous prédiction.

## Non fait, non vérifié

- Pas de visuel de bombe ni d'explosion (T17), pas de bombes du bot (T14), pas de placement du mur secret par la
  grammaire (T5), pas de porte à clés (`Key(n)`, T8).
- Aucun bench ni campagne (machine partagée).

## Dettes, questions

D80 (`Explode` en `OnUse`), D81 (aucun jeu livré ne donne de bombes) : `docs/taches/dettes.md`. Questions pour William
(§69) : Q4 (tir ami de la bombe : proposition `Always`), pool de butin propre aux rochers, dégâts des piques en
demi-cœurs (Q2), mur secret ouvert par un baril.

## Atteignabilité

Scénarios testbed seulement (voir §69). `isaac` : D81 ; rochers et piques dans les gabarits `combat` : M3-T11d ; mur
secret : M3-T5.
