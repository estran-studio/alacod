# X-6a1 — Portes de carte à états et clé portée par un ennemi

Jalon servi : **S3, dimanche 29 novembre 2026** (`plan-1837-cli.md` §6, ligne X-6a1, lancée le 8 novembre en avance ; sa preuve écrite fait foi). Elle sert l'extérieur du prologue d'un jeu externe
(un ennemi porte la clé d'une porte, la porte reste close sans elle) ; le **contrat exact de la clé reste à fermer par le design** (D210) : l'engine livre des mécaniques génériques et des défauts simples.
Branche : `wquintal/x-6a1-portes-cle`, partie d'`origin/main` `58126c8a`. Tête de code poussée et vérifiée : **`bec5cf42`** (le commit suivant ne touche que ce rapport et la ligne `LIVRÉ`). Aucune note de passation (lot neuf).

## Relevé de l'existant (status de dix lignes, avant tout code, envoyé au coordinateur)

`door.rs:30-69` (`DoorConfig` lue des champs LDtk), `from.rs:265` + `basic.rs:463-595` (la génération ne garde que la **position** des portes : un verrou d'auteur devait traverser cette chaîne),
`plugin.rs:432-488` (une porte = `Wall` + `DoorComponent` + collider + `Interactable { Door }`), `interaction.rs:292-419` (`handle_door_interaction`, point d'extension), `rooms.rs:1256` (`key_door_system`, portes de salle d'`isaac`, **inchangées**),
`navigation.rs:889-1014` (cases bloquées = portes avec collider), `shop.rs:261-390` (patron du butin au sol), `config.rs:62-249` (`CharacterConfig`), `hud.rs:1637-1689` + `ui.rs:22` + `hud_model.rs:41` (prompt, sources), `core_sim.rs:103` (enregistrements neutres avant la clôture).

## Périmètre et résultat

| Point du lot | Issue | Preuve |
|---|---|---|
| Porte de carte à états **fermée / ouverte / verrouillée par clé**, id de clé en donnée, ouverture par interaction | Fait | `DoorConfig::lock: Option<DoorLockConfig { key, consume }>` (champs LDtk `lock_key` / `lock_consume`) ; états déduits des composants (`Collider` + serrure = verrouillée, sans collider = ouverte) ; scénarios `door_state_closed_open` (`Closed` → `Open` avec la jumelle, 1000 → 250) et `door_state_locked_no_key` (`Locked` de f20 à f250, aucun débit) |
| Clé consommée ou non selon la donnée | Fait | `lock_consume` (absent : consommée) : `key_carrier_coop` (`key_door`, `TeamKeys` 1 → 0) et `door_key_kept` (`key_door_open`, `TeamKeys` reste 1) |
| Prompt « Ouvrir » / « Fermé — clé » | Fait | `door_prompt` (pure) : test `invite_d_une_porte_a_serrure_selon_les_cles_de_l_equipe` (sans clé, autre clé, bonne clé, porte à prix) |
| Porte verrouillée bloquante comme une fermée, **champ de flux compris** | Fait | elle garde son collider ; `crates/scenario/tests/map_door_keys.rs` lit `FlowFieldCache::wall_cells` : cases murées sur toute la partie sans clé, rendues à l'ouverture |
| Clé portée par un ennemi (champ de personnage), lâchée à sa mort en objet ramassable | Fait | `CharacterConfig::carries_key` ; `game::keys::key_drop_on_death_system` → `items::KeyPickup`, `GgrsNetId` `key_<id du mort>` ; `KeyPiles` 1 à la mort du porteur dans `key_carrier_coop`, `key_single_pickup`, `door_key_kept` |
| Ramassée par l'un ou l'autre joueur vers un **état partagé d'équipe** (ressource rollback, `BTreeMap`) | Fait | `run::TeamKeys(BTreeMap<String, u32>)` ; `key_carrier_coop` : le **joueur 2** (handle 1) la ramasse, le **joueur 1** (handle 0) ouvre avec |
| **Un seul** ramasse quand deux joueurs touchent la même frame | Fait | `key_single_pickup` (`TeamKeys` 1, jamais 2, `key_pickup` du joueur 0 seulement) + test `un_seul_joueur_ramasse_la_cle_le_plus_petit_net_id` (plus petit `GgrsNetId`, indépendant de l'ordre des `Entity`) |
| Source HUD des clés tenues | Fait | `team_keys` (`HUD_SOURCES`, `PLAYER_SOURCES`) ; `HudText(source: "team_keys")` vide / `cle_test` dans `key_carrier_coop` et `door_key_kept` ; widget dans le HUD du testbed |
| GGRS : types rollback neufs en variante **neutre**, `GgrsNetId` déterministes, aucune trace existante ne bouge | Fait | `KeyPickup` (`rollback_and_trace_neutral`), puis `TeamKeys` (`rollback_and_trace_resource_neutral`), puis `FrameEvents<KeyEvent>` (`add_frame_events_neutral`), **dans cet ordre**, dans `core_sim` après `RangedCycle` et **avant les cinq enregistrements de clôture** ; `check_rollback_registration` vert ; suite complète rejouée sans `BLESS` : **867 scénarios, 0 trace existante modifiée** (4 tranches, 447 s) |
| `make schedule_order_diff` | Fait, diff **jugé** | voir ci-dessous (ajouts seulement, aucun conflit) |
| Scénarios **neutres** du testbed, **traces neuves seulement** | Fait | 5 traces neuves : `door_state_closed_open`, `door_state_locked_no_key`, `key_carrier_coop`, `key_single_pickup`, `door_key_kept` (`ALACOD_BLESS=1` limité à ces cinq noms) |
| Lint et `describe` des nouveaux champs | Fait | `carries_key` dans le schéma de personnage (`alacod describe Character` : règle « non vide »), erreur `OutOfRange` pour `Some("")` ; avertissement `UnobtainableKey` (`lock_key` sans porteur) ; fixtures `door_lock_unobtainable`, `door_lock_carried`, `carries_key_empty` (`crates/content/tests/x_6a1_door_lock_fixtures.rs`) ; testbed sans avertissement de ce genre |
| `make check_rapide` vert | Fait | voir « Contrôle rapide » |
| § 117, D210–D212, rapport, LIVRÉ | Fait | `docs/conventions.md` §117, `docs/taches/dettes.md`, ce rapport, ligne `LIVRÉ` de `docs/taches.md` |
| Portes de zombies et d'isaac **inchangées** | Fait | `door_open`, `buy_door`, `key_door_*`, `grid_treasure_key`, `bot_isaac_key`… dans les 867 scénarios sans `BLESS` ; `DoorConfig::lock` vaut `None` partout ailleurs, `handle_door_interaction` ne lit `TeamKeys` que pour une serrure |

Hors lot (non fait, comme demandé) : génération de l'intérieur, passage d'un niveau LDtk à l'autre (X-6). Rien de 1837 dans l'engine : identifiants `cle_test`, `porteur_cle`.

## Décisions

- **Étendre l'existant** : `handle_door_interaction` lit la serrure de la porte (pas de second système) ; la porte verrouillée **est** la porte fermée plus une donnée, donc le champ de flux, l'interaction, le collider et l'ouverture de la jumelle sont ceux d'avant.
- **La serrure traverse la génération** : toutes les cartes passent par `map_generation`, qui recalcule la `DoorConfig` de la position seule. `EntityLocations::door_locks` porte la valeur d'auteur, `get_doors` la remet, la **copie sur la jumelle** (un seul battant porte le champ dans la carte) et met le coût à 0 (le prix est recalculé de la profondeur, pas une valeur d'auteur). Conséquence assumée (D211) : pas de « clé **et** argent ».
- **État d'équipe, pas inventaire** : `TeamKeys` est une ressource ; l'inventaire `key` par joueur des salles d'`isaac` reste séparé et intact.
- **Défauts simples** (D210) : la clé tombe à la position de la mort, reste au sol, au contact, par n'importe quel joueur (à terre compris, comme les douilles), consommée par défaut. Le choix de donnée livré est `lock_consume`.
- **Ramassage au contact** (comme les douilles) plutôt que par interaction : pas de second bouton, tie-break par `GgrsNetId`, déjà éprouvé.
- **Pas de sprite pour la clé** (D212) : `world_sprites::attach_world_sprites` est au plafond de Bevy (15 composants de requête) ; le HUD et le prompt disent l'état, la présentation reste à R-1 ou au jeu.

## `make schedule_order_diff` (base `58126c8a`, sorties gardées sous `/tmp/x6a1-sched`)

Sortie 2 (le script refuse tout écart), **jugé un par un** : `rollback.txt` des cinq jeux = **trois ajouts** insérés entre `RangedCycle` et les cinq de clôture, dans l'ordre réel `KeyPickup`, `TeamKeys`, `FrameEvents<KeyEvent>` (les cinq de clôture, `GgrsNetIdFactory` à `RngStreams`, passent de 131-135 à 134-138 sans changer d'ordre relatif ; aucun autre type déplacé). `GgrsSchedule` : 225 → 229 entrées = **quatre ajouts**
(`key_drop_on_death_system`, `key_pickup_system`, `clear_frame_events<KeyEvent>`, un `apply_deferred`) ; les seules paires existantes retournées sont **(`shelf_interaction_system`, `table_flip_system`), (`hub_station_system`, `table_flip_system`),
(`key_door_system`, `table_flip_system`), (`blank_system`, `shell_pickup_system`)** et les `clear_frame_events<…>` entre eux : des systèmes sans donnée commune (`ambiguity_detection: Error` ne les aurait pas laissés non ordonnés sinon, et `schedule_order` les construit tous) que le tri topologique de
bevy a permutés en insérant les nouveaux ; `SaveWorld` / `LoadWorld` : mêmes ajouts et permutations de plugins de snapshot indépendants (le XOR du checksum est commutatif). Preuve de comportement : les 867 traces identiques.

## Contrôle rapide

Journal complet : `~/orca/runs/alpha/logs/x-6a1-portes-cle-check-rapide-bec5cf42.log`, **vert**, code de sortie 0, sur la tête poussée `bec5cf42` : `cargo fmt --check`, `test_tools`, `check_forbidden`, `check_rollback_registration`,
`check_sim_boundary`, `alacod lint` des cinq jeux (zombies, testbed, throne, gungeon, isaac : aucune erreur), `test_crates` (97 blocs `test result`, **1 640 tests passés, 0 échec**). Dernières lignes : `test result: ok. 0 passed; …` (derniers doctests),
`✓ Contrôle rapide passé (suite de scénarios : la nuit)`, `EXIT 0`. Sweep joué ensuite (`scripts/target-sweep.sh target`, pas de `target_wasm`).

Un premier passage sur `d2f476d1` avait échoué sur **mes deux tests** de `game::keys` (`RollbackOrdered` absente du monde d'essai : un `Rollback` posé dans un `World` nu l'exige, comme dans `powerups.rs`) : corrigé, les tests n'avaient pas été lancés avant ce passage.

Suite complète de scénarios (`make test_scenarios_par SHARDS=4`, sans `BLESS`) jouée sur la tête de simulation `1d401d7f` : **867 scénarios verts, 0 trace existante modifiée** (447 s). Les commits suivants ne touchent que `content`
(avertissement de lint, fixtures), un module de tests et les docs : aucun code de simulation. Les cinq traces neuves ont été écrites par `ALACOD_BLESS=1 ALACOD_SCENARIO=<les cinq noms>` (aucune autre trace réécrite : `git status` ne montre que les cinq `.trace` neufs).

## Suivi de revue (même branche, après la fusion d'`origin/main` `16573cd5`, X-6a2)

Fusion : conflits de fin de fichier seulement (`docs/conventions.md` §117 / §118, `docs/taches.md`, `docs/taches/dettes.md` D210–D212 / D213, `CLAUDE.md`), **les deux côtés gardés**.

- **P2, porte à `lock_key` ouverte sans clé sur une carte à salles typées** : confirmé. `room_lock_system` n'excluait que `sealed_door` (mur secret, salle à clé d'`isaac`) ; à la première frame hors `Waves` il
  retirait `Collider` / `CollisionLayer` / `Interactable` des portes fermées voisines d'une salle typée, et la salle vidée (`Locked → Cleared`, arène tenue) rouvrait les siennes. Correctif (`map_ldtk::game::rooms`) :
  `DoorConfig::lock.is_some()` fait de la porte une porte **scellée** (`map_locked` dans `sealed_door`, et branche explicite « rien à poser » à l'ouverture d'office) : elle garde son collider **et** son `Interactable { Door }`,
  seul `handle_door_interaction` l'ouvre, et ouverte elle n'est jamais refermée par une salle voisine. Scénario neutre **`key_door_rooms`** (carte `testbed/portes_cle_salles.ldtk`, graine 1, générée par script depuis
  `salles_cle_combat.ldtk` : départ, combat avec un seul ennemi `porteur_cle`, salle finale ; porte combat ↔ finale à `lock_key`) : `Locked` aux frames 1, 2, 100 (salle verrouillée), 160 (salle `Cleared`) et 415, puis `Open` avec la clé
  consommée (`key_door`). **Sans le correctif il échoue** (sortie obtenue : porte 2 et 3 `Open` aux frames 1, 2, 160 et 415, clé jamais consommée : `TeamKeys` 1 au lieu de 0 à f500). Aucune trace existante ne bouge (suite complète rejouée).
- **P3, deux interactions de la même frame consomment deux clés** : confirmé. `handle_door_interaction` voit la porte fermée tant que les commandes différées ne sont pas appliquées. Correctif : `BTreeSet` des `GgrsNetId` de portes
  ouvertes dans la frame (**jumelle comprise**), événement ignoré s'il y figure (patron `handled` de `shelf_interaction_system`). Scénario **`key_door_same_frame`** (deux joueurs, deux porteurs tués, `TeamKeys` 2, Interaction **à la même
  frame**, f150, les deux événements `key_door` visent la porte) : `TeamKeys` 1 après l'ouverture. **Sans le correctif il échoue** (journal : deux `key_door` à f156, `TeamKeys` 0). Un premier jet du scénario (Interaction pressée
  pendant la marche) ne prouvait rien (un joueur arrivait une frame avant l'autre) : les joueurs sont maintenant butés contre la porte avant d'appuyer. Le correctif vaut aussi pour le prix d'une porte payante (un seul débit).
- **Non traité (D211)** : `secret_wall_system` et `key_door_system` retirent encore le collider de toutes les portes jumelles proches de leur salle, serrure comprise.
- **Doc** : ordre réel des enregistrements **`KeyPickup`, puis `TeamKeys`, puis `FrameEvents<KeyEvent>`**, entre `RangedCycle` et les cinq de clôture (ici et § 117 ; la phrase « en fin de liste » est retirée) ; § 117 (portes scellées sur carte typée, une
  ouverture par porte et par frame, deux scénarios) ; D211.
- **Traces** : deux traces neuves (`key_door_rooms`, `key_door_same_frame`), aucun `BLESS` d'une trace existante.
- **Contrôle rapide du suivi** : `make check_rapide` **vert** sur la tête fusionnée et corrigée **`f1dbe740`** (fusion d'`origin/main` `16573cd5` + P2 + P3), journal `~/orca/runs/alpha/logs/x-6a1-portes-cle-check-rapide-f1dbe740.log`, `EXIT 0`, 97 blocs `test result`, **1 642 tests passés, 0 échec** ; sweep joué ensuite.
- **Suite complète de scénarios non rejouée après P2 / P3** (consigne du studio, REGLES § 6 : la suite tourne la nuit sur le VPS, jamais sur le Mac) : une tentative locale a été arrêtée à mi-course (aucun résultat retenu). Preuve locale : `key_door_rooms` et `key_door_same_frame` verts avec le correctif et **rouges sans** ; la suite complète de `1d401d7f` (867 scénarios, 0 trace existante modifiée) précède P2 / P3. **La nuit dira si une trace existante bouge** (salles d'`isaac`, `gungeon`, `throne` pour P2 ; toute porte pour P3, y compris le prix des portes payantes de deux joueurs la même frame).

## Protocole de la partie du soir (dix lignes au plus)

1. `ALACOD_MUTE=1 make play_scenario SCENARIO=key_carrier_coop` : deux joueurs (rendu), le porteur tombe, le joueur 2 ramasse (le HUD du testbed affiche « Clés : cle_test »), le joueur 1 ouvre la porte ; `SCENARIO=door_state_locked_no_key` pour la porte qui refuse.
2. `make videos SCENARIO=key_carrier_coop,door_key_kept` pour les vidéos (`// À regarder :` dans chaque `.ron`).
3. À regarder : la clé au sol n'a **pas de sprite** (D212) ; seuls le HUD et le prompt (`F6` n'étiquette pas la clé) disent l'état.

## Ce qui reste

- **D210** : le contrat exact de la clé (retour au poste, porte atteinte avant la clé, clé tombée hors de portée, ramassage réservé aux joueurs debout) à fermer par le design, puis posé en données.
- **D211** : état initial « ouvert » et prix cumulé en donnée ; lint d'atteignabilité du porteur (aujourd'hui seulement « un personnage porte cette clé »).
- **D212** : sprite de la clé au sol et visuel de la porte verrouillée (R-1 / le jeu).
- X-6 (génération de l'intérieur, passage de niveau), après la définition de William.
- 1837 : déclarer `carries_key` sur son garde et `lock_key` / `lock_consume` sur la porte de sa carte (dans son dépôt, jamais dans l'engine) ; ajouter les deux champs aux définitions d'entités de son projet LDtk.
