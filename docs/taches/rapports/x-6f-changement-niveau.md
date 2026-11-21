# X-6f — Changement de niveau par une porte

Jalon servi : **D2 dim 29 nov 2026** (`plan-1837-cli.md` § 6, ligne X-6f). Branche `wquintal/x-6f-changement-niveau`, partie de `94038bb9` (contenu de `ca3d7e28` + docs).

## (a) Relevé de l'existant (avant tout code, état `94038bb9`)

| Sujet | fichier:ligne | Ce que c'est |
|---|---|---|
| Passage de niveau | `crates/map_ldtk/src/game/floors.rs:473-677` (`floor_transition_system`) | garde `portal_open` :505 ; joueur debout qui franchit : :518-526 (`crosses_portal`, `run/src/floors.rs:65`) ; détruit tout non-joueur `despawn_rollback` :535-543 ; recrée le niveau du slot suivant :548-659 ; **place tous les joueurs** (par handle) sur les `PlayerSpawn` :588-619 ; `floor_state.index += 1`, `portal_open = false` :669-671 |
| Ouverture du portail | `floors.rs:360-391` (`floor_portal_open_system`) ; `run/src/floors.rs:59` (`portal_should_open`) | niveau vidé (aucun `Enemy`) ; victoire au dernier niveau si `victory_at_end` |
| Ordre des systèmes | `floors.rs:237-250` | `debug_clear_enemies → floor_portal_open → floor_transition`, `Run`, `run_if(FloorPlan)` |
| État du mode | `crates/run/src/floors.rs:24` (`FloorState`) ; `floors.rs:78` (`FloorPlan`, hors rollback) ; `:179-233` (`compute_floor_plan`) | `FloorPlan` lue de `content::registry::FloorsEntry` (`registry.rs:971`, `FloorsFileSchema:1190`, insertion :3873) |
| Slots / assemblé par graine | `floors.rs:88-111` (`slot_of_level` : chaque `floor:<id>` est son slot) ; conventions § 40 « Étapes 2 » | graine = graine de run + slot × `0x3C6E_F372` |
| Porte de carte | `crates/map/src/generation/entity/door.rs:15` (`DoorConfig`, `lock`) ; `map_ldtk/src/game/entity/door.rs:58` (lecture des champs LDtk) ; `map_ldtk/src/game/plugin.rs:432-488` (entité rollback : `Wall` + `DoorComponent` + `Collider` + `Interactable{Door}` + `DoorShape`) | ouverte = `Collider`, `CollisionLayer`, `Interactable` retirés |
| La serrure traverse la génération | `map_ldtk/src/generation/from.rs:116-150,308` ; `map/src/generation/imp/basic.rs:463-620` (jumelle :595) ; `map_ldtk/src/generation/to.rs:564` | un champ de porte d'auteur qui doit survivre à `map_generation` suit ce chemin |
| Ouverture avec clé | `crates/game/src/interaction.rs:305` (`handle_door_interaction`) | clé d'équipe cherchée avant le prix ; `consume` retire ; jumelle ouverte avec |
| Clé d'équipe | `crates/run/src/keys.rs:21` (`TeamKeys`) ; `crates/game/src/keys.rs:105` (`key_pickup_system`) ; `items/src/loot.rs:684` (`KeyPickup`) | `BTreeMap`, neutre ; remise à zéro à chaque run |
| Prompt de porte | `crates/game/src/ui/hud.rs:1624` (`door_prompt`) | « Ouvrir » / « Fermé — clé » |
| Attentes | `combat/src/weapons/expectations.rs:284` (`FloorIndex`) ; `MapDoor`, `TeamKeys`, `KeyPiles` (§ 117) | |

Constat : le passage de niveau **place déjà toute l'équipe** (joueurs debout, à terre, feu follet, tous gardés) ; seul le **déclencheur** (portail « niveau vidé ») est à rendre remplaçable par une porte.

## Ce qui a été construit (générique, noms neutres ; détail et contrats : `docs/conventions.md` § 128)

- Champ de porte LDtk **`exit_next_level`** (booléen) : porte de carte marquée **sortie vers le niveau suivant**. Il traverse la génération comme la serrure de X-6a1 (`EntityLocations::door_exits` → `get_doors` → `DoorConfig::exit` → `to.rs`), copié sur la jumelle, **coût 0** (sans quoi la porte libre demandait 750 + profondeur × 250).
- Champ de séquence **`exit_by_door`** (`floors/<id>.ron`, défaut faux) : le portail « niveau vidé » ne s'ouvre pas aux niveaux qui ont un suivant ; le dernier garde la règle d'avant. **Le défaut ne change rien** (traces existantes identiques, point d).
- **`floor_transition_system`** étendu (pas de système neuf) : une porte de sortie **ouverte**, franchie par le premier joueur **debout** (`GgrsNetId`), déclenche le passage du § 17 sans attendre le niveau vide ; tous les joueurs sont replacés (un coéquipier à terre ou en feu follet garde son état). Clé d'équipe consommée ou non : donnée de serrure (§ 117).
- **Lint** : porte de sortie sur le dernier niveau ; niveau (sauf le dernier) sans porte de sortie. **Prompt** « Entrer » (présentation). Aucun type rollback neuf, aucune écriture hors de `GgrsSchedule`, `Fixed` seulement.

## Preuves

### (a) Relevé avant code
Voir le tableau en tête (fichier:ligne, état `94038bb9`).

### (b) Scénarios neutres du testbed (`tests/scenarios/`, traces neuves ; `sortie_porte_*`)

| Scénario (jeu court `rapide` : solo, sans_cle, niveau_vide, coop) | Issue | Ce qu'il prouve |
|---|---|---|
| `sortie_porte_solo` | **fait**, vert | clé ramassée f115 → porte ouverte f266 (clé consommée) → franchie f269 → `FloorIndex(1)` ; joueur sur le départ du niveau 1 (≈ (434, 696)) ; santé 55, `machine_gun` et 26 balles gardées |
| `sortie_porte_sans_cle` | **fait**, vert | témoin sans clé : porte `Locked`, `key_refused`, `FloorIndex(0)` |
| `sortie_porte_niveau_vide` | **fait**, vert | témoin niveau vidé sans franchir : aucun ennemi, aucun passage, aucun portail |
| `sortie_porte_graine` | **fait**, vert | niveau suivant = étage assemblé `floor:etage_sortie` |
| `sortie_porte_coop` | **fait**, vert | un franchit, les deux arrivent (le second sur son départ (424, 664)) |
| `sortie_porte_coop_a_terre` | **fait**, vert | coéquipier à terre : arrive à terre |
| `sortie_porte_coop_feu_follet` | **fait**, vert | coéquipier en feu follet : arrive en feu follet |
| test `la_relance_apres_la_porte_revient_au_niveau_zero_et_rejoue_a_l_identique` (`crates/scenario/tests/floors.rs`) | **fait**, vert (`scenario/floors code=0 passed=3`) | relance = niveau 0, clé remise à zéro, même passage à la même frame, même trace |

Chacun commence par `// À regarder : …`. Tests unitaires : `run::floors::franchissement_d_une_porte_de_sortie`, `map_ldtk::game::floors::le_portail_cede_la_place_a_la_porte_sauf_au_dernier_niveau`, `map::…::la_sortie_traverse_la_generation_gratuite_et_se_partage_avec_la_jumelle`, `game::ui::hud` (texte « Entrer »), `content` (`x_6f_exit_door_fixtures`, 4 tests).
Piège rencontré : un scénario à `map_seed` autre que le défaut réassemble les deux salles de `porte_sortie.ldtk` ailleurs (BasicMapGeneration) ; les scénarios gardent donc la graine par défaut.

### (c) Capture du vrai jeu
**Fait.** Binaire natif **release** du testbed (`cargo build --release -p testbed`, LTO, 24 min), fenêtre sur ce Mac, piloté par `alacod remote` (`spawn porteur_cle`, `input`, `step 1` + `screenshot` à chaque frame, script `/tmp/x6f-drive.py`, non commité ; `games/testbed/assets/game.ron` modifié le temps de la capture pour `mode: Floors, floors: Some("porte_sortie")`, **rétabli, non commité**). Frames jouées : porteur tué (f33), clé ramassée (HUD « Clés : cle_test »), porte (« [H] Ouvrir »), ouverture (f82), « [H] Entrer », franchissement (f87), fondu, arrivée au niveau 1 en (424, 696), santé 100, `machine_gun` 26 | 240.
Fichiers : `docs/taches/rapports/preuves/x-6f/x-6f-porte-sortie-testbed-release.mp4` (215 images à 10 i/s = 21,5 s, 58 Ko) et trois images (`1-cle-porte-fermee-ouvrir.png`, `2-porte-ouverte-entrer.png`, `3-niveau-suivant.png`). Pas de vidéo de scénario (`make videos`) : non faite.

### (d) Familles touchées, avant / après
**Après** : `make test_scenarios SCENARIO=<79 noms>` sur la tête (`7ffc5454`), sans `BLESS` : **79 scénarios, 0 trace différente, 0 attente en échec** (`test result: ok`, 181 s).

| Famille | Scénarios | Après (tête) |
|---|---|---|
| `floors` / étages / portail (`portal_next_floor`, `*_floor*`, `gungeon_floor_seed_*`, `gungeon_two_floors`, `throne_floor_1`, `throne_three_floors`, `blank_floor_refill`, `loot_floor_table`, `difficulty_scales`, `curse_bigger_floor`, `clock_floor_reset`, `fenetre_etage`, `secret_wall_floors`, `bot_*floor*`…) | 30 | traces inchangées |
| `generated/*` à étages (gungeon `room_*`, `boss_gatling` ; isaac `boss_*`, `room_combat` ; testbed `room_*`) | 13 | traces inchangées |
| `key_*` | 8 | traces inchangées |
| `door_*` (dont `door_state_*`, `door_key_kept`) | 4 | traces inchangées |
| `rooms_*` | 4 | traces inchangées |
| `downed_*` | 3 | traces inchangées |
| `wisp_*` | 10 | traces inchangées |
| `sortie_porte_*` (neufs) | 7 | traces neuves écrites (`BLESS=1` sur ces seuls nouveaux fichiers) |

**Avant (`ca3d7e28`) : non rejoué sur ce commit.** Les traces de référence commitées sont celles de `ca3d7e28` (`git diff ca3d7e28 --name-status -- '*.trace'` : sept ajouts, **aucune modification** de trace existante) ; la tête les rejoue à l'identique. Rejouer la famille sur `ca3d7e28` demandait un second build complet du workspace (≈ 35 min de CPU et plus de 10 Go de cible, disque tombé à 29–35 Go libres, plancher 30) : écarté. À faire sur le VPS si le relecteur l'exige.
**`make schedule_order_diff BASE=ca3d7e28` : non lancé** (même coût : build de la base). Argument : le diff n'ajoute **aucun système** à un schedule (`floor_transition_system` change de paramètres, pas d'ordre), **aucun type rollback** et aucun enregistrement ; `make check_rollback_registration` et `check_sim_boundary` sont verts dans le contrôle rapide (journal). Non vérifié par l'outil dédié.

### (e) Contrôle rapide découpé
Vert sur `7ffc5454` (le commit suivant ne touche que la documentation et les preuves) : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_integration` code 0 (187 s), puis `test_crates` découpé code 0 (840 s) :
`workspace --lib --bins code=0 passed=1169 failed=0`, `binaires joués : 59 / 58 tests d'intégration + lib/bins/doc ; failed : 0, codes non nuls : 0`, `content/x_6f_exit_door_fixtures passed=4`, `scenario/floors passed=3`, `test_crates découpé : code 0, 840 s ; total 1027 s`.
Journal complet : `~/orca/runs/alacod/logs/x-6f-check-rapide-7ffc5454.log`.
**Écart au brief** : le script `docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh` appelle la cible `check_animation`, supprimée par T-3b (remplacée par `check_integration`, `Makefile:101`) : la première tentative a échoué (`No rule to make target`) ; j'ai joué une **copie** du script avec `check_integration` (non commitée ; à corriger dans les specs du coordinateur).

## Ce qui reste / limites

- (d) « avant » sur `ca3d7e28` et `schedule_order_diff` : non joués (raison ci-dessus) ; la suite complète tourne la nuit sur le VPS après le merge (non jouée ici).
- Dettes **D224** (franchissement par rectangle élargi, pas de transition visuelle, invite non testable en scénario) et **D225** (sens unique, niveau recréé neuf, arrivée sur les `PlayerSpawn`, deux déclencheurs coexistent sans `exit_by_door`) : `docs/taches/dettes.md`.
- Hors périmètre, non fait : carte dessinée posée à la position LDtk (`layout: Drawn`), image propre d'une porte (déjà dans `propositions.md`).
- Le niveau d'arrivée de `sortie_porte_graine` est un étage minimal (départ + boss, gabarits du testbed) ; la salle centrale à N sorties et l'intérieur par graine sont **X-6g**.
