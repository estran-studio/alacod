# Rapport m1-v1e — mode `Floors` (T1.8) + attente `FloorIndex`

**Fiche** : `docs/taches/m1-v1e-mode-floors.md`. **Branche** : `m1-v1e-mode-floors`, session
cloud c2. **Tête du code livré** : `07c8b9c` (merge d'`origin/main` `b6a58a0` fait juste avant
la livraison ; le commit de ce rapport vient par-dessus). Clone froid, compilation complète
depuis zéro, `CARGO_BUILD_JOBS=4`. **Aucune trace existante modifiée, aucun bless.** Pas de
bench ni de p2p (cloud). **Push refusé par le proxy git de la session (403)** : branche livrée
en bundle git (voir la fin).

## 1. Fait

### Décision d'architecture : le passage de niveau a lieu *dans* `GgrsSchedule`

Deux voies possibles : (a) repasser par `GameLoading` comme la relance (`RunRequest::Restart`),
(b) charger d'avance toutes les cartes et créer le niveau suivant dans la simulation. (a) recrée
la session GGRS (frame GGRS remise à 0, impossible à synchroniser en p2p — c'est la dette du
restart p2p) ; (b) est déterministe et rollback-safe par construction. **Choix : (b).**

- **Chargement** (`map_ldtk::game::floors`) : `FloorPlan` (séquence, hors rollback) est posé à
  l'entrée de `GameLoading` par `compute_floor_plan`, avec la même résolution de mode que
  `jjrs` ; `loader::setup_generated_map` charge alors **un monde LDtk par carte distincte**
  (`FloorWorld(emplacement)`, config de génération figée par chargement :
  `load_map_snapshot`). Les mondes se superposent à l'origine ; seul celui du niveau courant est
  visible (présentation). Le chargement attend que tous les mondes aient leurs niveaux
  (`FloorWorldsReady`) puis ne crée les entités rollback que du **premier** niveau ; le
  registre des entités de carte garde celles de tous les niveaux (`slot`), trié.
- **Passage** (`floor_transition_system`, `RollbackSystemSet::Run`, après
  `finalize_run_summary_system`) : à la frame où un joueur debout franchit le portail ouvert,
  `despawn_rollback` de **toute entité rollback qui n'appartient pas à un joueur** (armes =
  enfants du joueur, gardées), `FlowFieldCache` remis à zéro, création du niveau suivant avec
  **les mêmes fonctions et le même ordre qu'au chargement** (entités de carte du registre, murs
  triés par iid, personnages, armes murales, machines à perk) — `GgrsNetIdFactory` n'est
  jamais remise à zéro, donc la numérotation continue, triée par contenu ; joueurs placés sur
  les `PlayerSpawn` du niveau (même handle, repli sur le plus petit index). Les données lues
  (mondes LDtk déjà chargés) sont immuables et identiques sur tous les clients ; un rollback
  qui remonte avant le passage ressuscite l'ancien niveau.
- **Portail** (`floor_portal_open_system`) : s'ouvre quand il n'y a plus aucune entité `Enemy`
  (`EntityCount(enemy) == 0`), au **barycentre des `PlayerSpawn`** du niveau (centre de la
  salle de départ, pas d'entité LDtk dédiée), rayon 24.
- **Fin de séquence : boucle infinie au dernier niveau** (`run::floors::level_for_floor`),
  pas de victoire en `Floors` ; défaite universelle, résumé par les mêmes chemins que `Waves`.
- **Refactorisation pour réutiliser la création** : `local.rs` (`spawn_level_characters`,
  `spawn_level_weapon_locations`, `spawn_level_soda_locations`, `LevelSpawnAssets`),
  `plugin.rs` (`spawn_map_items`), `collider.rs` (`spawn_level_walls`) ; plus aucun
  `projects.single()` (`load_levels_if_not_present`, `add_room_component_to_ldtk_level`,
  murs : par monde). Hors `Floors` : carte unique = `FloorWorld(0)`, chemin inchangé.

### État rollback sans déplacer les traces des autres modes

- `run::FloorState` (ressource : `index`, `anchor`, `portal_open`, `enemies_placed`),
  enregistrée par une nouvelle méthode **`rollback_and_trace_resource_neutral`**
  (`crates/utils/src/rollback.rs`) : rollback + checksum + trace, mais la valeur par défaut
  contribue `0` au checksum GGRS (XOR de `bevy_ggrs::ChecksumPlugin`). Hors `Floors`, la
  ressource reste à sa valeur par défaut : checksum agrégé inchangé, donc **aucune des 62 traces
  existantes ne bouge** (vérifié, §2). En `Floors`, toute valeur non défaut est hachée
  normalement. Test unitaire `utils::rollback::tests`, et le test de couverture
  `game::rollback` passe à sept méthodes.
- `RunMode::Floors { config }` ajouté **en dernière variante** (hash dérivé = discriminant : les
  autres modes gardent le leur).
- `RunSummary::floor_reached` avec un `Hash` écrit à la main : mêmes champs, même ordre que
  l'ancien dérivé, `floor_reached` haché seulement s'il est non nul (résumés des autres modes :
  hash identique au bit près).

### Contenu, lint, scénarios, outils

- Kind `Floors` (`floors/<id>.ron`, `levels: [...]`), `EntryMode::Floors` (première séquence),
  lint : `levels` non vide (`OutOfRange`, fixture `floors_empty`), niveau = carte chargée
  (`BrokenReference`, `floors_unknown_map`), `entry.mode: Floors` sans dossier `Floors`
  (`BrokenReference`, `entry_mode_floors_without_floors`).
- Scénario : champ `floors: Some("<séquence>")` (`game::run_state::FloorsOverride`, hors
  rollback ; reporté par l'enregistreur). Attentes `FloorIndex(index, at_frame)` et
  `RunSummary(.., floor_reached_min)`. Moments clés `portal`/`floor` (vidéos, `Event`).
- Testbed (mode toujours `Sandbox`) : cartes `floor_a` (1 follower), `floor_b` (breacher +
  follower), `floor_c` (breacher + 2 followers), copies d'`arena` avec iids neufs ; séquences
  `deux_niveaux`, `trois_niveaux`.
- `portal_next_floor` : un joueur scripté tue le follower (portail f62), marche au portail
  (niveau 1 à f230), reste immobile ; le breacher le met à terre (défaite f2543) ; attentes
  `Event(portal)`, `FloorIndex` 0 puis 1, `EntityCount(enemy) == 2` au niveau 1,
  `RunState(Ended(Defeat))`, `RunSummary(kills_min: 1, floor_reached_min: 1)`.
- `alacod-sim --floors <séquence>` ; `floor` dans le JSON et la sortie (`Metrics::final_floor`).
  Pas de `--until-floor` (T1.14).
- Bots : `BotView::portal` ; `fonceur`/`prudent` sans ennemi marchent (ligne droite) vers le
  portail ouvert. Hors `Floors` il n'y a jamais de portail : décisions inchangées (traces
  `bots_*`/`clone_*` identiques).
- Écran de fin : « niveau N » en `Floors`.
- `docs/conventions.md` **§17 « Mode Floors »** (section distincte, rien d'autre touché) ;
  `CLAUDE.md` : `FloorIndex` dans la liste des attentes.

### Bug trouvé et corrigé en route : grille de collision des murs périmée

`combat::collision_grid::maybe_rebuild_wall_grid` ne reconstruisait la grille des murs que si
leur **nombre** changeait. Les niveaux de même géométrie (même nombre de murs) gardaient la
grille de l'ancien niveau, dont les entités n'existent plus : les joueurs traversaient les
murs (vu par `alacod-sim` : invariant `joueur_hors_mur`). Correctif : signature (nombre, somme
des `GgrsNetId`). Ressource dérivée hors rollback ; **les 62 traces existantes restent
identiques** avec ce correctif (§2). Ce défaut touchait aussi, en théorie, un rollback qui
remplace des murs à nombre constant.

## 2. Vérifié (commandes et chiffres réels, dans le cloud)

- `make test_scenarios` (état final avant formatage, puis le code n'a bougé que par `make fmt`
  et le merge docs) : **63 scénarios verts** = les 62 existants **à traces inchangées** + 
  `portal_next_floor` ; `git status tests/scenarios` : seuls `portal_next_floor.{ron,trace}`
  (nouveaux). Joué deux fois (avant et après le correctif de grille) : identique. Avertissements
  de budget fps sur toute la suite (ex. `idle` 50,9 < 80, `bench_horde` 31,2 < 38) : machine
  cloud, bench hors périmètre.
- Tests des crates (`-p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core
  -p stats -p bots -p effects -p utils`) : **323 réussis / 0 échec / 8 ignorés** (run complet
  avant le correctif de grille) ; sur l'état final, en deux lots faute de disque (voir §3) :
  hors `scenario` 275/0/1, `scenario` sans le binaire `scenarios` 45/0/0, le binaire
  `scenarios` étant couvert par `make test_scenarios` ci-dessus. Nouveaux tests : `run::floors`
  (4), `run::modes` (1), `game::run_state` (2), `utils::rollback` (1), `map_ldtk::game::floors`
  (1), `bots::decide` (1), `content` lint (3 fixtures + table), `crates/scenario/tests/floors.rs`
  (2 : passage de niveau et continuité des net ids — ids du nouveau niveau contigus et après le
  dernier distribué, ancien niveau entièrement détruit, joueurs : même id/monnaie/armes, placés
  sur leur point de départ, murs et cases murées rechargés ; deux parties → même trace).
- `make lint` : `games/zombies : aucune erreur`, `games/testbed : aucune erreur (7
  personnages, …, 7 cartes)`.
- `cargo fmt --all -- --check` : rien (après `make fmt`, qui n'a touché que mes fichiers).
- `./scripts/check-forbidden.sh` : 4 avertissements préexistants (3 `HashSet`, 1 `rand::`),
  aucun nouveau. `./scripts/check-rollback-registration.sh` : `OK`.
- `make gen GAME=zombies` : exit 0, `git status` identique avant/après (aucune modification).
- **`alacod-sim`** (critère 2) : `--game testbed --bots 4 --profiles
  fonceur,fonceur,prudent,immobile --seeds 1..5 --until-wave 99 --max-frames 4000 --floors
  trois_niveaux` (après le correctif de grille) :

  | graine | niveau atteint (index) | morts | desync |
  |---|---|---|---|
  | 1 | 13 | 0/4 | non |
  | 2 | 13 | 0/4 | non |
  | 3 | 10 | 0/4 | non |
  | 4 | 15 | 0/4 | non |
  | 5 | 8 | 0/4 | non |

  Les bots finissent **les trois niveaux sur les cinq graines** (index ≥ 3), puis bouclent sur
  `floor_c`. ~45 fps, ~85 s mural par graine.
  **Réserve honnête** : l'invariant `joueur_hors_mur` signale encore des chevauchements
  ponctuels (graines 2 à 5 : 2 ou 3 frames sur 4000 ; graine 1 : 113 frames). Diagnostiqués sur
  les graines 1 et 2 : en plein niveau (pas à la frame de passage), deux bots collés (centres à
  ~9 px, colliders de 20) se poussent et l'un dépasse de quelques pixels dans un mur. Non
  reproduit en `Sandbox` sur le testbed (même commande sans `--floors`, 2000 frames : 0
  violation), où les bots ne convergent pas tous vers un même point. Je ne l'attribue pas au
  passage de niveau, mais je ne l'ai pas prouvé préexistant sur `main` (cas non jouable sur
  `main`) : **à regarder** (séparation joueur-joueur contre les murs, `move_characters`).

## 3. Non fait / non vérifié

- **p2p à deux clients et bench** : non faisables dans le cloud, à rejouer par l'orchestrateur.
  Le passage de niveau crée des entités dans `GgrsSchedule` à partir de données LDtk chargées
  hors simulation : le synctest le couvre (aucun mismatch sur `portal_next_floor` ni sur les
  25 parties `alacod-sim`), mais **le p2p réel n'a pas été joué**.
- **Preuve `trace-diff`** : non requise (aucune trace existante ne change). Pas de dump
  `main` vs branche fait.
- **Trace de `portal_next_floor`** : fichier **nouveau**, écrit par le mécanisme d'écriture de
  référence restreint à ce seul scénario (`ALACOD_SCENARIO=portal_next_floor`) — aucune trace
  existante réécrite. À valider/bénir par l'orchestrateur s'il préfère la produire lui-même.
- Vidéo / rendu : non vérifiés (pas de rendu dans le cloud) — visibilité des mondes, carré du
  portail, barres de vie des fenêtres recréées sont du code de présentation **jamais exécuté
  avec rendu ici** (il tourne en headless, sans effet visible).
- Tests des crates sur l'état final : faits en deux lots (disque de session plein deux fois :
  binaires de test ~1 Go chacun, j'ai supprimé des artefacts de `target/` entre les lots).
- Le zombies n'a pas de séquence `Floors` (le testbed seul porte les cartes de test).

## 4. Dettes, questions ouvertes

- **Mondes superposés** : les `RoomBounds`/`LevelId` des niveaux non courants existent aussi,
  et les niveaux générés d'une même graine peuvent partager l'iid de niveau (observé :
  `29fc2629…` pour `floor_a` et `floor_b`). Sans conséquence pour les entités du testbed
  (filtrage par monde partout dans le chargement), mais **les spawners de vagues (`ZombieSpawn`)
  ne sont pas pris en charge en `Floors`** (`enemy_spawn_from_spawners_system` lit toutes les
  salles). Un décalage spatial par monde ou un filtre par monde courant serait nécessaire.
- `EntityCount(enemy)` compte toute entité `Enemy` (alliés/civils du testbed compris) : ne pas
  en placer dans un niveau `Floors` (documenté §17).
- Bots : ligne droite vers le portail, sans pathfinding (salles ouvertes du testbed) ; la
  complétion de niveau « réelle » reste T1.14.
- Kills du résumé `Floors` = ennemis placés − vivants (les ennemis ne viennent pas des vagues) ;
  `Metrics::kills` d'`alacod-sim` reste celui des vagues (0 en `Floors`).
- Chevauchements `joueur_hors_mur` entre bots collés (§2) à investiguer.
- Restart p2p : toujours non supporté (D14), inchangé.

## 5. Livraison

Le proxy git de la session refuse le push vers `estran-studio/alacod` (403, consigne de
l'orchestrateur relayée par William) : la branche est livrée en **bundle git**
(`/tmp/m1-v1e-mode-floors.bundle`, plage `origin/main..m1-v1e-mode-floors`, copie jointe à la
réponse finale). Import : `git fetch /chemin/m1-v1e-mode-floors.bundle
m1-v1e-mode-floors:m1-v1e-mode-floors`.
