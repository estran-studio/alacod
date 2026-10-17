# Rapport m3-t9-curses (b3)

Conventions : `docs/conventions.md` §74. Branche `m3-t9-curses` depuis `origin/main` `d4162f18` (M3-T0e mergée), puis fusionnée
avec `origin/main` `25c0e49b` (m2-p2p-run-complete) : fusion sans conflit. Décision de William **Q7** appliquée : les cinq
curses (`Darkness` HideMinimap, `Unknown` HideHealth, `Blind` HideItemNames, `Labyrinth` FloorRooms ×2, `Modifier(stat)`),
rien d'autre. Machine locale avec Docker (OrbStack) : la mention « session cloud, pas de Docker » de la fiche ne s'applique pas.

## 1. Fait

| Point de la fiche | Livré |
|---|---|
| 1. Tirage à l'entrée de chaque étage | `game::curses::curse_system` (`GgrsSchedule`, `FrameCounter`, après `Run`) : à l'entrée dans un étage — y compris le premier, frame 0 — `run::draw_curse` (règle pure de T0e, RNG local `curse_seed(graine, étage)`, aucun flux de `RngStreams`) écrit `RunCurses`, trace `ggrs{f=… curse floor=… id=<id\|none>}`, moment clé `curse`. **Écart** : pas lu sur `FloorEntered` (émis seulement avec horloges ou difficulté, jamais pour l'étage 0) : le système détecte l'entrée lui-même (`FloorState::index` ≠ `RunCurses::entered`), à la même frame. |
| 2. `Modifier` | posé sur chaque joueur à l'entrée (`sim_core::modifier`, source `curse:<id>`), tout `curse:*` retiré à l'entrée de l'étage suivant ; un joueur apparu plus tard le reçoit à la frame suivante ; rien n'est écrit tant que le compte est bon. |
| 2. `FloorRooms` | **à la génération** (toutes les cartes d'une séquence sont assemblées au chargement) : `with_curse_floor_rooms` dans `setup_generated_map` applique `FloorGrammar::scaled_rooms(×m)` à la grammaire de l'emplacement, grille et arbre. Le tirage ne dépend que de (graine, étage) : le planificateur retrouve la curse que `curse_system` écrira. « Du même étage » (pas « de l'étage suivant »), dit au §74. |
| 2. `HideMinimap`, `HideHealth`, `HideItemNames` | présentation dérivée de `RunCurses` à chaque `Update` : `CurseHides`, lu par `hud_values` (minicarte vide, santé absente : texte et barre) et par le prompt de `ui::hud` (nom `???` pour `Weapon`, `Shelf`) ; nouvelle source HUD `curses` (id de la curse de l'étage), montrée par `isaac/ui/hud.ron`. |
| 3. Scénarios | `curse_drawn_seeded` (+ `_other`, + test `curses` : même graine même trace, autre graine autre tirage, graine sans curse), `curse_modifier`, `curse_bigger_floor`, `curse_hides_minimap` (attente `HudText`), plus `curse_hides_health` et `curse_blind` (témoin). |
| Attentes | `RoomCount(min, max)` ; `HudText(empty: true)` (texte vide : `contains: ""` ne prouvait rien). Listées dans `CLAUDE.md`. |
| Lint | `reach::stat_uses` compte le `Modifier` d'une curse (`UnreadStat` si la stat n'est lue par aucun système). |

Autres décisions et corrections :
- `RunCurses` devient `{ floors, entered }` (T0e : tuple `BTreeMap`) : sans mémoire de l'étage traité, « tiré, pas de curse » ne
  se distingue pas de « pas encore tiré ». Neutre à l'état par défaut ; **plus neutre dès qu'une séquence a un champ `curses`**.
  Conséquence : `curse_pool_unset` (T0e) est **supprimé** (.ron et .trace) : il affirmait « rien n'exécute le tirage » sur la
  séquence `maudit`, ce qui n'est plus vrai (règle « un test qui épingle l'état d'avant la livraison se supprime »).
- Données `isaac` : dossier `curses/` (les cinq de Q7, copies du testbed), `floors/sous_sol_maudit.ron` (un étage, chance 1 : la
  graine choisit la curse). **La séquence par défaut d'`isaac` n'est pas touchée** : la fiche attribue la déclaration de
  `curses` dans la séquence par défaut à M3-T11d. Donc aucune trace d'`isaac` ne bouge (seul `isaac/ui/hud.ron` gagne un widget,
  présentation).
- Testbed : `floors/maudit_maze.ron` (pool `maze` seul) ; `floors/maudit.ron` (commentaire : exécuté).
- `scaled_rooms` ramène la grammaire à une grammaire satisfiable (au moins départ + boss + requises, au plus la capacité de la
  grille) : une curse ne peut pas rendre un étage impossible à planifier dans les limites éprouvées (×2).

## 2. Vérifié (sur la fusion avec `origin/main` `25c0e49b`, HEAD `23589f63` avant le commit LIVRÉ)

- `make test_scenarios_par SHARDS=3` sans BLESS : **507 scénarios**, 3 tranches ok, 158 s (sur la branche avant fusion : 507,
  197 s). Aucune `.trace` existante modifiée (`git status` propre après), 7 traces nouvelles, 1 supprimée.
- `make test_crates` : **1 063 passés, 0 échec**, 14 ignorés (T0e : 1 044 ; +1 venu de `main`) ; nouveaux : `world` (`scaled_rooms`), `run::curse`
  (`entered`, produit de `FloorRooms`), `game::curses` (4), `ui::hud` (noms cachés), `map` (1 000 graines doublées sur la grille),
  `map_ldtk` (2 × 200 graines ×2 en grille `isaac` et en arbre `gungeon`), `scenario::curses` (3).
- `make lint` ×5 : zombies, testbed, throne, gungeon, isaac « aucune erreur ».
- `make gen GAME=…` ×5 : code 0, `git status` propre après (aucun diff), sur la fusion.
- `cargo fmt --all -- --check` ; `check-forbidden.sh` (38 occurrences auditées) ; `check-rollback-registration.sh` ;
  `make check_sim_boundary` (11 crates, 5 en dette D69) ; `make test_tools` (19 tests) : OK.
- Fuzz `ALACOD_FUZZ=0:8` (sur la fusion) : 1 test passé (29 s) ; sur la branche avant fusion, en plus, sur les scénarios à curse
  (`ALACOD_FUZZ_SCENARIO=curse_modifier` et `curse_bigger_floor`, 600 frames, inputs aléatoires, synctest) : passés.
- **Mutations** (appliquées puis annulées) : sans retrait des modificateurs, `curse_modifier` échoue (`MoveSpeed 127,5 au lieu de
  150` à f250 et f299) ; sans `FloorRooms`, `curse_bigger_floor` échoue (8 salles au lieu de 16-24) ; sans le masquage de la santé,
  `curse_hides_health` échoue (`HUD health = « 6/6 »`).
- **p2p-stress** (Docker OrbStack, `MATCHBOX_PORT=3576`, `COMPOSE_PROJECT_NAME=b3`), montages **temporaires non commités**
  (`entry` de `game.ron` changée puis restaurée par `git checkout`) :
  - testbed, `floors: "maudit_maze"`, graine 3, 3 000 frames : **5 720 frames resimulées** (client 0), 0 `DesyncDetected`,
    codes 0/0, traces identiques (2 999 lignes) ; `ggrs{f=0 curse floor=0 id=maze}` aux deux clients ;
  - testbed même montage, 2 400 frames, `ALACOD_CLEAR_ENEMIES_AT=1500`, inputs sans déplacement sauf `Right`+`Down` f1600-1612
    (`/tmp/inputs-portal.ron`) : **6 597 frames resimulées**, 0 desync, traces identiques (2 399 lignes) ; curse `maze` à f0
    (modificateur posé), passage d'étage à f1606 (`curse floor=1 id=none`, modificateur retiré) ;
  - isaac, `floors: "sous_sol_maudit"`, graine 2 (labyrinth), 3 000 frames : **4 608 frames resimulées**, 0 desync, traces
    identiques (2 999 lignes) ; chaque client assemble `salles (8, 12) → (16, 24)`.
  - **sur la fusion** avec `main` `25c0e49b` (nouveau `p2p-stress.sh`, `EXPECT_FLOORS=1`), montage testbed ci-dessus, 2 400 frames :
    **5 990 frames resimulées** (client 0 ; client 1 : 1), 0 desync, **1 transition d'étage** relevée au journal des deux
    clients (résimulée à f1606), traces contiguës et identiques (2 399 lignes) ; `curse floor=0 id=maze` à f0, `curse floor=1
    id=none` à f1606. Les trois mesures précédentes ont été faites sur la branche avant la fusion ; isaac n'a pas été refait
    sur la fusion (le script seul a changé, pas le code d'`isaac`).

## 3. Non fait / incertain / non vérifié

- `HideItemNames` : le prompt est calculé dans `ui::hud` (hors `HudSnapshot`) : aucun scénario ne l'observe, seul le test
  unitaire `blind_curse_hides_the_name_of_pickups_and_keeps_the_rest` ; `curse_blind` ne prouve que « santé et minicarte
  intactes ». Aucune capture d'écran : le rendu (`--features render`) n'a pas été lancé.
- Les écrans de pause et d'inventaire (§56) ne lisent pas `RunCurses` : l'inventaire montre les objets **tenus** (pas de nom du sol
  à cacher) ; ils n'affichent ni santé ni minicarte.
- Étages au-delà de la séquence (boucle sur le dernier niveau) : pas de `FloorRooms` de leur propre tirage (monde déjà assemblé).
  Sans effet sur `isaac` (`victory_at_end`).
- Un `FloorRooms` que la grille ne tient pas (×100 sur 7 × 7) échoue explicitement au chargement (`Exhausted`), comme toute
  grammaire impossible ; seuls les ×2 sont prouvés (grille `isaac` et arbre `gungeon`). Pas de lint là-dessus (dépend de la grammaire).
- Le hub (`HubSession`) n'est pas couvert : une séquence à curses jouée depuis le hub n'a pas été essayée.
- Relecture indépendante : non faite par moi (attendue de l'orchestrateur). Points à regarder :
  1. tirage par détection d'entrée plutôt que par `FloorEntered` (§74, « Tirage ») ;
  2. `RunCurses` non neutre dès qu'un champ `curses` existe (`entered`), suppression de `curse_pool_unset` ;
  3. `FloorRooms` lu à la génération, étage = emplacement (§74, « Étage et emplacement »), `scaled_rooms` borné ;
  4. portée de `HideItemNames` (`Weapon` et `Shelf` : armes et objets à ramasser ou acheter, pas la machine à perk) ;
  5. graine du tirage = `RunSeed` = `MapGenerationConfig.seed as u32`, lue à la génération par la même expression ;
  6. valeur exacte `127,5009155` de `curse_modifier` (150 × 0,85 en `Fixed`, pas 127,5).

## 4. Traces bougées

**Aucune trace existante.** Les 500 `.trace` d'avant (hors `curse_pool_unset`) sont identiques à `main` : 507 scénarios verts sans
BLESS. Ajoutées : `curse_drawn_seeded`, `curse_drawn_seeded_other`, `curse_modifier`, `curse_hides_minimap`, `curse_hides_health`,
`curse_blind`, `curse_bigger_floor` (nouveaux scénarios, aucun bless d'une trace existante). Supprimée : `curse_pool_unset` (T0e),
dont la trace aurait bougé (`RunCurses.entered` s'écrit dès la frame 0 d'une séquence à curses) et dont le propos est caduc.
**Pas de preuve §5 (dump comparé)** : rien n'a été re-béni ; la preuve de neutralité est la suite verte sans bless sur 500 traces
existantes (dont `grid_*`, `isaac_*`, `gungeon_*`, qui n'ont pas de champ `curses`) et le test
`a_sequence_without_curses_draws_nothing_and_leaves_the_state_neutral` (`RunCurses` jamais écrite, vérifié à chaque frame).

## 5. Dettes laissées, questions ouvertes

- M3-T11d : déclarer `curses` dans la séquence par défaut d'`isaac` (ses traces bougeront alors : preuve §5 à faire à ce moment) et
  nommer la graine qui tire une curse à l'étage 1 ou 2 (`sous_sol_maudit` joue déjà les cinq par graine : 1 darkness, 7 unknown,
  5 blind, 2 labyrinth, 4 maze).
- M3-T16 (HUD v3) : lire `CurseHides::item_names` aussi pour les objets sans prompt (`InteractionType::Item` n'a pas de prompt
  avant le HUD de M2/V4).
- Journal `docs/taches.md` §10 : à ajouter sur `main` après le merge (règle du README §6) ; proposition :
  `| 2026-10-16 | M3-T9 (curses d'étage en jeu : tirage à l'entrée par graine+étage, Modifier curse:<id>, FloorRooms ×2 à la génération grille et arbre, HUD masqué minimap/santé/noms ; attentes RoomCount et HudText empty ; §74) | m3-t9-curses | b3 | mergée ; 507 scénarios sans trace bougée sur la fusion avec main 25c0e49b (7 nouveaux, curse_pool_unset retiré), test_crates 1 063, lint/gen ×5, fuzz (+2 scénarios à curse), 3 mutations tuées, p2p-stress testbed 5 720 puis 6 597 (passage d'étage) puis 5 990 sur la fusion (EXPECT_FLOORS=1) et isaac 4 608 (labyrinth ×2) frames resimulées ; non fait : curses dans la séquence par défaut d'isaac (T11d), item_names hors prompt (T16) |`

## 6. Atteignabilité

`isaac/floors/sous_sol_maudit` joue les cinq curses (une par graine : scénarios `curse_hides_minimap` graine 1, `curse_hides_health`
7, `curse_blind` 5, `curse_bigger_floor` 2) ; le testbed joue `maudit` (graines 3 et 2, deux étages) et `maudit_maze` (graine 3).
La partie ordinaire d'`isaac` ne rencontre aucune curse avant M3-T11d (dette nommée ci-dessus).
