# Rapport m2-salles-murs-gungeon (murs dessinés, salles à l'échelle, campagne de contrôle, revue vidéo, « Jouer »)

Revue humaine de M2 par William (2026-10-18) : dans les vidéos `gungeon_full_run` et `gungeon_full_run_four`,
(1) **chaque salle est minuscule**, (2) **les murs n'ont pas de sprite**. Conventions : §83 « Murs dessinés » (fin du
paragraphe) et §40 « Échelle des gabarits ». Base : `origin/main f1d5a093` ; fusionné deux fois (`4cd1b245`, puis `7ee084e8`).

## 1. Murs dessinés (présentation seule, aucune trace ne bouge)

**Cause.** Les gabarits LDtk de `gungeon` n'ont aucune tuile de mur (`autoLayerTiles` vide : la couche IntGrid `Walls` est
écrite par script, puis l'assembleur de salles la modifie : ouvertures murées, cases de sol). Des règles d'auto-couche LDtk
n'auraient donc rien à habiller. **Choix : auto-tuile en présentation** (`map_ldtk::wall_art`), dérivée des cases de la carte
*assemblée* ; correcte pour toute carte générée, aucun fichier de carte retouché.

**Mécanisme** (générique, activé par donnée) : `ui/hud.ron` gagne un champ optionnel
`wall_art: Some((layer: "Walls", atlas: "sprites/lisible/atlas_murs.png", hide_layers: ["LevelConnection"]))`
(`game::ui::hud::WallArt`) ; sans lui, ou sans rendu (headless), rien ne change : zombies, throne, isaac, testbed identiques.
`wall_art::draw_walls` (`LdtkRoguePlugin`, `Update`) pose, à l'apparition d'un niveau, un sprite par case de mur sous une racine
`WallArtRoot` enfant du niveau (visibilité héritée : l'étage quitté disparaît). Tuile choisie par `wall_tile` (pure, testée) :
**face de mur** (briques) quand la voisine du sud est un sol, **dessus** (liseré clair, contour sombre sur chaque bord qui touche le
sol, angles rentrants) ailleurs : 36 tuiles de 16 px. `hide_layers` cache `LevelConnection` (LDtk la peignait en noir par-dessus
le sol). Lint : fixture `hud_wall_art_missing`. **Sprites** : `scripts/generer-sprites-gungeon.py` (CC0, même script et même style
que §83) écrit `atlas_murs.png` ; les **portes** (`door_v`/`door_h`) reçoivent des jambages de pierre aux deux bouts (couleurs de
la face), le **mur fissuré** (`secret_wall`) reprend les briques du mur : tous se raccordent. `assets.yaml` et le test
`test_sprites_gungeon.py` mis à jour.

**Preuve.** Captures avant/après (barre rouge = `f1d5a093`, verte = branche), `gungeon_full_run` f42, f1890, f2825, f3439 :
`docs/taches/rapports/m2-salles-murs-gungeon/murs/avant_apres_f*.png`. Suite `make test_scenarios_par SHARDS=3` après l'étape 1
seule : 642 scénarios, **aucune trace bougée** (`git status` propre).

## 2. Salles à l'échelle

### Mesure d'avant (gabarits `salles.ldtk`, `etage_1.ldtk`, `hub.ldtk`)

| | cases (16 px) | pixels | sol intérieur | largeur de sol en héros (32 px) / colliders (20 px) | traversée à 150 px/s |
|---|---|---|---|---|---|
| quinze gabarits de `salles.ldtk` | 8×8 | 128×128 | 6×6 = 36 cases | 3 / **4,8** | 0,64 s |
| `etage_1.ldtk` (squelette T0d, trois salles) | 8×8 | 128×128 | 6×6 | idem | idem |
| `hub.ldtk` | 16×12 | 256×192 | 14×10 | 7 / 11 | 1,5 s |

Écran : 960×540, `camera.ron` `min_zoom: 0.6` → 576×324 unités visibles (36×20 cases) ; une salle de 128 px occupait
**22 % × 40 %** de l'image : quatre salles et demie dans la largeur. Enter the Gungeon : une salle de combat y fait plusieurs
écrans de personnage de large (ordre de grandeur de la référence ; **non mesuré sur le jeu**, la page de Dodge Roll ne publie pas
de dimensions : les seules sources consultées, la page PCGamingWiki et l'exemple Edgar, ne donnent pas de taille de salle).

### Échelle retenue (en données : `make_gabarits_grands.py`, déterministe, idempotent depuis la version 8×8)

| type | gabarits | cases | surface ×ancienne | intérieur | écran (zoom 0,5 = 480×270) |
|---|---|---|---|---|---|
| départ | `Depart` | 14×12 | 2,6 | 12×10 | 47 % × 71 % |
| combat, escouade | `CombatEW/NS/T/Croix`, `EscouadeEW` | 16×12 | 3,0 | 14×10 (11 colliders) | 53 % × 71 % |
| récompense, secret | `Recompense`, `Secret` | 12×10 | 1,9 | 10×8 | 40 % × 59 % |
| boutique | `Boutique` | 14×10 | 2,2 | 12×8 | 47 % × 59 % |
| coffre | `Coffre` | 14×12 | 2,6 | 12×10 | 47 % × 71 % |
| arène | `ArenaEW/Croix/NS` | 20×14 | 4,4 | 18×12 (14 colliders) | 67 % × 83 % |
| boss | `BossW/N` | 22×16 | 5,5 | 20×14 (16 colliders) | 73 % × 95 % |

Justification : combat à ×3 (la fiche attendait 2 à 3), boss ×5,5 ; à `min_zoom: 0.5` une salle de combat occupe un peu plus de la
moitié de la largeur et 71 % de la hauteur : on la lit d'un coup d'œil, portes comprises. Récompense, secret et boutique à ×2
(trois présentoirs à 32 px d'écart et un coffre n'ont pas besoin de plus). Arène (750 frames à tenir) et boss (patterns, trois
phases) ont besoin d'espace pour esquiver. **Le contenu ne change pas** : mêmes ennemis, mêmes props, mêmes nombres, placés
proportionnellement ; le script vérifie l'absence de recouvrement et de prop sur le passage de deux cases devant une ouverture. Les salles
sont donc plus vides : le nombre d'ennemis par salle est l'équilibrage de William (D64), pas cette tâche. Captures :
`echelle/avant_f42_salle_8x8.png`, `echelle/apres_f300_combat.png`, `apres_f600_coffre.png`, `apres_f1300_portes.png`.

### Ce que cela a demandé dans le moteur (§40)

- **Portes alignées, tailles différentes.** L'appariement des ouvertures (`Connection::are_matching`) exigeait la même distance au coin :
  impossible avec des portes au milieu de côtés de longueurs différentes. `populate_level_connections_with(.., offset_free)`, activé pour une
  grammaire `Graph` (`from.rs`) seulement. `Room::get_connecting_room_position`, jamais exercée avec des tailles différentes, avait deux défauts
  (taille de la voisine au lieu de la courante à l'est et au sud ; signe du décalage à contresens pour le sud et l'est) : réécrite, test
  `la_voisine_se_pose_ouverture_contre_ouverture`. Test de génération : toute ouverture restée libre a, en face, une ouverture libre
  (`etage`, `grand`, `court`, **2 000 graines**).
- **Étendue du monde** : `FloorGrammar::bounds` (donnée, lint) ; `etage` 3000, `grand` 4000, `court_secret` 2000.
- **Caméra** : `min_zoom` 0,6 → 0,5.

### Traces rebénies (19 gungeon, toutes avec cause)

Cause unique : **la géométrie des gabarits** (positions des entités posées par les salles, `SurfaceGrid`, `FlowFieldCache`, compteur d'ids : dès la
frame 0), plus, pour les scénarios dont le `.trace` avait été rebéni avant la seconde fusion, le tirage de butin D95/D96 de main.
**Preuve §5 (contrôlée)** : même binaire de test, `minimap_shop_visit`, dump `ALACOD_DUMP_TRACE` avec les assets d'avant (`salles.ldtk`, `floor_grammars`,
`camera.ron` de `f1d5a093`, par `ALACOD_GAME_DIR`) contre ceux d'après ; `scripts/trace-diff.py` : première frame qui diffère = **0** ; ce qui diffère =
positions des `character_spawn`/props, `SurfaceGrid`, `FlowFieldCache`, `GgrsNetIdFactory` (210 → 215) ; **les 90 types de composants et ressources
tracés sont les mêmes** (aucun type ajouté ni retiré). **Autres jeux identiques** : après correctifs, la suite complète (649 scénarios) ne change aucune
trace hors gungeon (`git status` propre).

Scénarios : `bench_floor` (145,3 fps, plancher 40), `bot_explorer_floor`, `bot_explorer_two`, `generated/gungeon/room_{arene,coffre,combat,depart,escouade,secret}`,
`gungeon_boss_bot`, `gungeon_floor_seed_{1,23,123}`, `gungeon_floor_start`, `gungeon_full_run`, `gungeon_full_run_four`, `gungeon_two_floors`,
`minimap_shop_visit`, `shop_seeded`. **Attentes recalées, aucune retirée** :
- scripts des quatre scénarios `etages_court*` re-créés par enregistrement d'un bot (`alacod-sim --save-scenario`) : combat verrouillé f79, nettoyé f259, boss verrouillé f339, victoire f572 ;
  `gungeon_two_floors` : `BossPhase` `NetId(143)` → `NetId(135)`, f1350 → f1370 ; `gungeon_floor_start` : l'étendard est ramassé par une `Interaction` ajoutée au segment initial.
- `gungeon_squad` : marche f10→f85 (verrouillage f78), trois `EnemyDistance` de f200 à f240 (mêmes bornes, l'escouade arrive plus tard dans la salle agrandie).
- `gungeon_boss_bot` : tout décalé d'environ 60 frames (boss verrouillé f319, victoire f534). `bot_explorer_floor` 760 → 900 frames, `bot_explorer_two` 720 → 880.
- `shop_seeded` : trois `Shelf` (autre salle de boutique, autre clé de tirage) : `arbalete` 15, `gilet` 15, `laser` 50.
- `gungeon_full_run` (victoire f7200, 7900 frames) : 26 attentes re-datées ; `NetId` de boss 205 → 175 et 704 → 607 ; `nth` des salles d'arène et de boss inversés entre les deux étages ;
  les coffres donnent `bastion` puis `rancune` (plus d'arme) ; la boutique de l'étage 1 vend `trefle`. `gungeon_full_run_four` (victoire f6844, 7000 frames) : 16 attentes re-datées, `NetId` 211 → 181 et 788 → 663.
- **Affaiblissement dit, forcé** : dans les deux runs de bots, **aucun coffre à clé n'est plus ouvert** (le bot ouvre un coffre libre) : les attentes `Consumable(key)` valent maintenant 1 jusqu'à la fin au lieu de 0 après
  l'ouverture. La consommation d'une clé par un coffre n'est donc plus attestée par ces deux scénarios (dette : un scénario dédié ou une autre graine).

## 3. Campagne de contrôle (`alacod-sim`, `etages_deux`, `--shopping`, 200 graines à 1 bot et 200 à 2 bots)

**Résultat final : 0 desync, 0 soft-lock, 0 panique, 400 victoires sur 400.** (JSON : `m2-salles-murs-gungeon/campagne/b{1,2}_*.json`, script `run.sh` ; victoires et morts à titre
d'information, aucun objectif d'équilibrage.)

| bots | graines | victoires | morts | soft-locks | desyncs | frames par partie (médiane, min, max) | dégâts subis (moyenne) |
|---|---|---|---|---|---|---|---|
| 1 | 1–200 | 200 | 0 | 0 | 0 | 10 713 ; 6 658 ; 15 577 | 173 |
| 2 | 1–200 | 200 | 0 | 0 | 0 | 8 906 ; 5 729 ; 12 057 | 164 |

Les salles plus vides (même nombre d'ennemis dans plus d'espace) ont supprimé les morts du bot (avant : D64 « 200 morts sur 200 » à 1 bot) : à dire à William pour l'équilibrage.

**Première mesure, sur le code de départ** (graines 1–24 à 1 bot, 6 soft-locks, tous à 100 % de santé) puis les trois campagnes successives (10, 2 puis 1 soft-lock) ont trouvé **quatre causes**, toutes corrigées :

1. **Le détecteur de soft-lock ne passait pas à l'échelle** (`FLOORS_SOFTLOCK_FRAMES` = 1 200 frames sans ennemi en moins ni dégât). Un étage mesure maintenant plus de 2 000 px : la marche d'une salle de combat à la suivante dépasse la fenêtre sans un seul coup.
   `FloorsProgress` compte aussi **les cases de 32 px jamais visitées** (un bot qui explore encore n'est pas bloqué ; un bot qui tourne en rond l'est : les cases visitées cessent de croître). Test unitaire.
2. **Arme sans munitions** (graines 3, 4, 12, 18…) : acheter ou ramasser un `fusil_precision` (munition `calibre`) remplace l'unique emplacement d'arme, sans réserve du type : le bot reste sans tir. L'`explorateur` n'achète plus un présentoir d'arme sans réserve de son type, ne ramasse plus une telle arme au sol, et à sec va chercher n'importe quelle arme ayant une réserve (`arms::wants_ground_weapon`, tests).
3. **D82, cycle limite devant une ouverture de 32 px** (graines 51, 182, 148, 76, 118, 158, 89, 179) : décalage d'input de 5 frames + élan + corps de 20 px dans 32 px : le bot déborde de ±8 px, rebrousse et déborde de l'autre côté sans fin (sondé : `x` ±7,7, `y` ±12, période 20 frames, `Up` pressé 3 frames sur 20).
   Correctifs : `explorer::lead` (corrige l'écart transverse de `vitesse × 5 frames`, pas l'axe du trajet), appliqué aussi à l'approche du portail, puis **`Patience`** : un bot resté 100 frames dans 40 px de son ancre change de manière (15 frames sans touche pour éteindre l'élan, puis cap droit sans corriger un écart transverse de moins de 6 px, séquence répétée). Essais notés : gain d'anticipation 8 frames au lieu de 5 : pire (5 graines bloquées de plus), d'où la patience.
4. **Panique `fixed … overflow` dans `move_enemies` (graine 54)** : `FacingDirection::from_fixed_vector` (`sim_core`) fait déborder `atan2` pour une vitesse de plus de ~181 (un champion rapide : (−181,6 ; −11,7)). **C'est un défaut du moteur** (un joueur aurait vu le jeu s'arrêter). Correctif : au-delà de 173 de long (norme² 30 000) le vecteur est réduit d'un facteur 4 avant `atan2` ; **rien ne change en deçà** (traces identiques, suite verte).

**Les décisions de bot nouvelles sont derrière `ExplorerOptions::steady`** (vrai pour `game: "gungeon"` dans le runner et `alacod-sim`, faux ailleurs) : avant ce garde-fou, les traces de dix scénarios d'explorateur d'isaac, testbed et throne bougeaient ; avec lui, aucune.

## 4. Vidéos et serveur de revue

(à compléter après le rendu : voir §6)

## 5. Jouer (testé par moi sur ce Mac, M-series, binaire `dev` optimisé)

Compilation (une fois, ~20 min à froid ; le binaire lit `games/gungeon` par le chemin capturé à la compilation, ne pas le déplacer) :
`cd alacod && APP_VERSION=dev cargo build -p gungeon --features native` → `target/debug/gungeon` (profil `dev` optimisé, comme `make gungeon`).

| Cas | Commande | Vérifié |
|---|---|---|
| **Solo** (démarre au hub : marchand, porte = entrée en run) | `make gungeon` (= `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost`) | lancé : fenêtre créée, chargement, partie locale démarrée, aucune erreur (le contenu a passé `alacod lint`) ; **je n'ai pas pu capturer l'écran** (`screencapture` : pas d'accès à l'affichage depuis ce processus) : rendu jugé par `play_scenario` et les vidéos |
| **Directement dans une run** (sans hub, graine fixée) | `ALACOD_SKIP_HUB=1 make gungeon GARGS="--seed 10"` | lancé : « graine de carte imposée --seed 10 », mode `Floors` `etages_deux`, 2 spawns joueurs |
| **Deux joueurs en local** | `make gungeon GARGS="--players localhost localhost"` (le `--players localhost` du Makefile est remplacé) | lancé : deux joueurs locaux, session locale. **Limite** : les deux héros reçoivent **les mêmes touches et la même manette** (`get_input_map()` est identique pour chaque joueur, pas d'attribution clavier / manette par joueur) : utile pour regarder, pas pour jouer à deux ; jouer à deux = en ligne, ou deux instances |
| **Manette** | rien à faire : D-pad = déplacement, stick gauche = visée (`Pan`), Nord = interagir, Ouest = recharger, Est = mêlée, Sud/D-pad = écrans | **non testé** (pas de manette ici) |
| **En ligne à deux (matchbox local)** | machine A : `docker compose -f docker-compose.ci.yaml up -d signaling` (`MATCHBOX_PORT=3570` pour un autre port ; Docker ou OrbStack), puis sur chaque machine : `APP_VERSION=dev target/debug/gungeon --number-player 2 --matchbox ws://<ip de A>:3536 --lobby salon --players localhost remote --cid alice --name alice` (**le même `--players localhost remote` des deux côtés**, `--cid` et `--name` différents) | **testé** : deux instances sur ce Mac, signaling `docker compose` sur 3570 : pairs connectés, « All 2 players are connected », synchronisation GGRS, 2 500 frames chacune sans desync |
| **En ligne, serveur public** | `wss://allumette.estran.studio` (le défaut du Makefile) | **ne marche pas depuis la ligne de commande** : `--matchbox` : « socket dropped » ; `--allumette https://allumette.estran.studio` exige un **UUID de lobby existant** (« HTTP 400 / 404 Lobby not found ») : les lobbies se créent par le site, pas par le client natif. Utiliser le `docker compose` local |
| **p2p avec rollbacks réels** | `scripts/p2p-gungeon-inputs.sh 2 10 5000 /tmp/run.ron` puis `ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" EXPECT_FLOORS=1 scripts/p2p-stress.sh gungeon /tmp/run.ron 4800` | **testé** (§7) |

**Touches** (clavier + souris ; codes de touche physiques, donc sur un clavier AZERTY **W A S D** sont les touches Z Q S D) : déplacement **W A S D** ; viser à la souris, **clic gauche** tire ; **H** interagir (ouvrir, acheter, ramasser, relever) ; **R** recharger ; **Tab** changer d'arme ;
**Z** changer de mode de tir ; **F** mêlée ; **L** lâcher l'arme ; **Espace** objet actif ; **Q** blank (efface les balles proches) ; **E** bombe ; **Maj gauche** sprint ; **C** esquive (dash) ; **Échap** pause ; **I** inventaire ;
**1 / 2 / 3** choix de mutation ; **F6** étiquettes (nom, prix, état) ; **F7** légende des silhouettes et couleurs ; **F3** champ de flux, **F4** profil, **F5** états des ennemis (débogage) ; `ALACOD_LABELS=1` allume les étiquettes au départ ;
`ALACOD_MUTE=1` coupe le son. Reprendre la même partie : `--seed <n>` ; sauter le hub : `ALACOD_SKIP_HUB=1`.

## 6. Reste à faire / limites (honnêtes)

- Rendu des murs et des salles jugé sur captures et vidéos **par moi** ; William les revoit (critère de sortie). Sol toujours uni (`#696A79`), pas d'ombre portée des murs.
- Les salles agrandies sont **plus vides** (même contenu) ; la densité d'ennemis est une décision d'équilibrage à prendre (le bot ne meurt plus : 0 mort sur 400 parties).
- Consommation d'une clé par un coffre : plus attestée par `gungeon_full_run*` (voir §2).
- `bounds` de grammaire : 3 000 px pour `etage` ; si William agrandit encore les salles ou le nombre de salles, le relever (le planificateur échoue explicitement, jamais en boucle).
- Un seul niveau de `min_zoom` (0,5) pour tous les écrans ; `max_zoom_out: 3.0` inchangé.
- Deux joueurs locaux sur un même clavier : non géré (voir §5).
