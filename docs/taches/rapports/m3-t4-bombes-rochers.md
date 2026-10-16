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

## Vérifié (sur la fusion avec `main` `af46f192`, commit de fusion `fde0036f`)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 412 scénarios verts ; **aucune trace existante modifiée**
  (seules les 11 traces neuves de la branche, une par scénario ci-dessus).
  Les scénarios jouent en synctest (rollback et resimulation à chaque frame) : bombes, rochers, mur secret et
  piques passent sans mismatch.
- `make test_crates` : 937 tests, 69 suites vertes. (Un test unitaire de `world::explosion` de ma branche, à
  l'arithmétique fausse, avait échoué au premier passage : corrigé, `make test_crates` rejoué vert ; ce changement ne touche que du code de test, la suite de scénarios n'a pas été rejouée après lui.)
- `make lint` : cinq jeux « aucune erreur ». `make gen GAME=zombies|testbed|throne|gungeon|isaac` : sans diff.
- `cargo fmt --all -- --check`, `make check_sim_boundary` (11 crates, 5 en dette D69), `make test_tools`,
  `./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh` : OK.
- `fuzz_inputs` : `ALACOD_FUZZ=0:8` (zombies, 3600 frames) sans divergence ; `ALACOD_FUZZ=0:8 ALACOD_FUZZ_BOMBS=1`
  (personnage à huit bombes dans `rock_flow`, huit explosions par partie) sans divergence.

## Non fait, non vérifié

- **p2p à deux clients non rejoué** (pas de Docker dans cette session) : l'input `Bomb` existait déjà (bit 18, T0c),
  mais la pose, l'explosion et la suppression d'entités rollback sont nouvelles : à rejouer par l'orchestrateur
  (`make test_multiplayer`, ou `scripts/check-p2p-traces.py` sur un scénario à bombes).
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
