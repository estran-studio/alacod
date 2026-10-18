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

`make videos SCENARIO=gungeon_full_run,gungeon_full_run_four,gungeon_classes` rendu sur le commit livré (dossier `target/videos/<sha du commit LIVRÉ>/` : `gungeon_full_run.mp4` 132 s,
`gungeon_full_run_four.mp4`, `gungeon_classes.mp4`, `montage.mp4`) ; copié dans `~/orca/revue-m2/target/videos/` à côté de `3163a83e` (gardé pour comparaison), page régénérée par
`python3 scripts/scenario-review.py` (`scripts/scenario-review.py` et `.html` de ce worktree copiés dans `~/orca/revue-m2/scripts/`). Le serveur déjà lancé (port 8766, 0.0.0.0) sert le dossier :
`http://127.0.0.1:8766/` → 200, la page liste le commit livré et `3163a83e` ; une requête `Range` sur le `.mp4` répond 206. Depuis le réseau local : `http://192.168.8.112:8766/`.
**Comparaison avant/après** : `make compare_video SCENARIO=gungeon_full_run BASE=f1d5a093` a fonctionné (rendu des deux côtés par `play_scenario` compilé dans un `target` à part, 29 min) :
`target/videos/compare/gungeon_full_run_f1d5a093_vs_<sha>.mp4` (42 Mo, copié dans `revue-m2/target/videos/compare/`) ; à gauche `f1d5a093` (salles minuscules, sans murs), à droite la branche.
Les vidéos ne sont pas versionnées (24 Mo chacune : `target/` est ignoré).

## 5. Jouer (testé par moi sur ce Mac, M-series, binaire `dev` optimisé)

Compilation (une fois, ~20 min à froid ; le binaire lit `games/gungeon` par le chemin capturé à la compilation, ne pas le déplacer) :
`cd alacod && APP_VERSION=dev cargo build -p gungeon --features native` → `target/debug/gungeon` (profil `dev` optimisé, comme `make gungeon`).

| Cas | Commande | Vérifié |
|---|---|---|
| **Solo** (démarre au hub : marchand, porte = entrée en run) | `make gungeon` (= `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost`) | lancé : fenêtre créée, chargement, partie locale démarrée, aucune erreur (le contenu a passé `alacod lint`) ; **je n'ai pas pu capturer l'écran** (`screencapture` : pas d'accès à l'affichage depuis ce processus) : rendu jugé par `play_scenario` et les vidéos |
| **Directement dans une run** (sans hub, graine fixée) | `ALACOD_SKIP_HUB=1 make gungeon GARGS="--seed 10"` | lancé : « graine de carte imposée --seed 10 », mode `Floors` `etages_deux`, 2 spawns joueurs |
| **Deux joueurs en local** | ~~`make gungeon GARGS="--players localhost localhost"`~~ **FAUX** (corrigé, voir « Suivi de revue » §3) : `--players` s'**ajoute** (clap) et donne **trois** joueurs. Commande juste : `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost localhost` | voir « Suivi de revue » §3 : deux héros vérifiés. **Limite** : les deux héros reçoivent **les mêmes touches et la même manette** (`get_input_map()` est identique pour chaque joueur, pas d'attribution clavier / manette par joueur) : utile pour regarder, pas pour jouer à deux ; jouer à deux = en ligne, ou deux instances |
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
- Consommation d'une clé par un coffre : plus attestée par `gungeon_full_run*` (voir §2) ; **attestée depuis par `gungeon_key_chest`** (voir « Suivi de revue »).
- `bounds` de grammaire : 3 000 px pour `etage` ; si William agrandit encore les salles ou le nombre de salles, le relever (le planificateur échoue explicitement, jamais en boucle).
- Un seul niveau de `min_zoom` (0,5) pour tous les écrans ; `max_zoom_out: 3.0` inchangé.
- Deux joueurs locaux sur un même clavier : non géré (voir §5).

## Suivi de revue (`m2-salles-murs-suivi`, 2026-10-18, b0)

Revue « OK avec réserves » de `m2-salles-murs-gungeon` ; branche `m2-salles-murs-suivi` depuis `origin/main f96fa463`. Ni `crates/bots/` ni la logique de détection de
`runner.rs` ne sont touchés (b3 réconcilie le bot avec `m3-t14-bot-isaac`).

### 1. Coffre à clé : plus attesté par une run de bots, attesté par un scénario dédié

- **Scénario `gungeon_key_chest`** (`game: "gungeon"`, `etage_1`, 120 frames) : coffre `locked: true` posé au pied du joueur (f2), clé posée f5 et ramassée au contact
  (`Consumable(key)` = 1 à f8), `Interaction` f10 : coffre ouvert (`Event(chest)` : « joueur 0 ouvre le coffre c : bastion », f15), **clé consommée** (`Consumable(key)` = 0 à f20),
  `Interaction` f40 : l'objet sorti est ramassé (`ItemCount(bastion)` 0 à f20 → 1 à f100). Trace bénie (nouveau scénario). Les deux scénarios de bots gardent `Consumable(key)` = 1
  (la limite de détour du bot écarte la salle de récompense : **hypothèse de la revue, non démontrée sur la graine 10** ; les attentes restent vraies sans toucher au bot).
- **Graines où l'explorateur ouvre un coffre avec sa clé** (`alacod-sim --game gungeon --bots 1 --profiles explorateur --floors etages_deux --until-floor 5 --shopping`, graines 1 à 33,
  32 parties terminées) : **19 graines sur 32** voient un coffre `locked` ouvert (1, 2, 3, 4, 7, 8, 9, 11, 15, 19, 20, 22, 23, 25, 27, 29, 30, 32, 33). La **graine 7** l'ouvre tôt : `trophee`
  à la frame 2192 de la run (joueur 0, une seule partie de 1 bot sans hub) ; aussi graine 1 (f6533, `grenadier`), graine 2 (f8821, `chercheur`), graine 11 (`bastion`). Mesure faite avec une
  ligne de journal temporaire `locked=<bool>` ajoutée à `chest_open` (`game/src/loot.rs`, **retirée**). Les parties sont plus longues que `gungeon_full_run` : je n'ai pas créé de scénario
  de bot sur la graine 7 (la décision de détour est dans le bot que b3 réconcilie).
- Le critère de sortie M2 « coffre à clé ouvert, clé consommée » est donc attesté par `gungeon_key_chest`. **Reste à faire dans le bot, après T14** : exempter du détour la salle de récompense
  dont le coffre est ouvrable (clé en main), pour que la graine 10 le montre aussi ; l'attente `Consumable(key)` 1 → 0 de `gungeon_full_run` ne redevient vraie qu'à ce moment-là
  (non rétablie : elle serait fausse aujourd'hui).

### 2. `docs/conventions.md` §57 et commentaires de `gungeon_full_run_four`

Tableau « Run complète de M2 » réécrit avec les frames actuelles (victoire f7200, 7 900 frames ; `NetId` de boss 175 et 607), ligne du coffre à clé renvoyée à `gungeon_key_chest`,
note de la phase 2 mise à jour. `gungeon_full_run_four.ron` : en-tête (l. 1 à 7, frames d'avant les gabarits agrandis dites comme telles), commentaire du coffre à clé (l. 86), arrivée
du second étage f3424 → f4012. `gungeon_full_run.ron` : commentaire de la clé renvoyé à `gungeon_key_chest`. Aucune attente changée : traces identiques.

### 3. « Deux joueurs locaux » : la commande était fausse

`make gungeon` ajoute `--players localhost` à la fin de la ligne (Makefile l. 309) et `--players` s'**ajoute** (clap) : `GARGS="--players localhost localhost"` donne **trois** joueurs
(constat de la revue, déduit du Makefile ; je n'ai pas rejoué cette mauvaise commande). **Commande juste, essayée sur ce Mac** :

```
APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost localhost
```

Lancée (ALACOD_REMOTE=1 en plus, pour lire l'état) : « Map is loaded with 4 player spawns », session locale, `scripts/alacod-remote brief` : **deux héros** (`joueur 0` en (40 ; −136), `joueur 1` en (56 ; −136),
pistolet, 100/100 PV). **Mêmes touches ?** Oui, d'après le code : chaque joueur local reçoit `LocalPlayer`, un `ActionState` et **le même** `get_input_map()` (`game/src/character/player/create.rs:89`) ;
je n'ai pas pu presser de touche réelle depuis ce processus (pas d'accès au clavier), donc non vérifié par une frappe. Deux héros sur un clavier bougent donc ensemble.
Section « Jouer » déplacée dans **`docs/jouer.md`** (solo, graine fixée, deux joueurs locaux, en ligne à deux avec signaling `docker compose -f docker-compose.ci.yaml up -d signaling` et `--matchbox ws://<ip>:3536`,
touches, F6/F7, `ALACOD_SKIP_HUB=1 --seed`), renvois depuis `README.md` et `docs/etat-engine.md` (critère 7). Les commandes solo, graine fixée et en ligne à deux ont été essayées au rapport initial (§5) ;
la commande à deux joueurs locaux l'a été ici. Manette : non testée.

### 4. Preuve §5 sur des traces jouées par un bot

Méthode : `ALACOD_DUMP_TRACE` (trace complète de chaque frame) + `scripts/trace-diff.py`, même binaire de test. Le bouton `steady` (vrai pour gungeon, `runner.rs:439`) est basculé
par une **modification temporaire** de cette ligne (variable `ALACOD_PROOF_NO_STEADY`, **retirée**) ; les assets d'avant sont `games/gungeon` de `f1d5a093` pour `salles.ldtk`, `floor_grammars`, `camera.ron`,
`hud.ron`, `assets.yaml`, les trois atlas modifiés (dossier `/tmp/gungeon_old`, `ALACOD_GAME_NAME=gungeon ALACOD_GAME_DIR=…`). Trois variantes : **A** = assets d'avant, `steady` faux ;
**B** = assets d'après, `steady` faux ; **C** = assets d'après, `steady` vrai (le code livré).

| Scénario (bot `explorateur`) | A → B (assets seuls) | B → C (`steady` seul) | types tracés |
|---|---|---|---|
| `bot_explorer_floor` (grammaire `etages_court_deux`, graine 1, 900 frames) | **frame 0** : positions des `character_spawn`, `FlowFieldCache` (géométrie des gabarits) | **frame 11** | 86 dans A, B et C, **mêmes noms** |
| `gungeon_boss_bot` (étage squelette `etage_1`, 600 frames) | **identique** (599 frames) : `etage_1.ldtk` n'a pas changé, ce scénario ne lit aucun gabarit de `salles.ldtk` | **frame 11** | 86 dans A, B et C, mêmes noms |

Lecture de B → C : les frames 0 à 10 sont identiques composant par composant ; à la frame 11, **seuls** le `FacingDirection`, la `Velocity` et le `FixedTransform3D` du joueur diffèrent
(B : `UpRight`, vitesse en diagonale ; C : `Up`, `Velocity.main` = (56,08 ; 150)), puis la divergence se propage. Aucun ennemi, aucune ressource ne diffère avant : la première frame divergente est celle
du **premier input différent** du bot (l'input est la seule entrée de la simulation ; la vitesse est son effet immédiat). Je n'ai pas relevé l'input brut lui-même (le dump ne le contient pas) : l'argument
est par élimination. Conclusion : **les changements de trace des scénarios de bots ont deux causes séparées et mesurées** : la géométrie des assets (frame 0, quand le scénario lit un gabarit) et les décisions
`steady` (premier input différent, frame 11 dans les deux scénarios) ; aucun type tracé n'est ajouté ni retiré.

### 5. P3

- **`FacingDirection::from_fixed_vector`** : la réduction ÷4 (norme² > 30 000) est remplacée par `atan2_would_overflow`, qui rejoue en `checked_mul`/`checked_add` les produits du polynôme d'octant
  de `fixed_trigonometry::atan2` (`x·y`, carrés, `x² + 0,28125·y²`, mêmes arrondis) et ne réduit **que si le calcul déborderait vraiment**. Toute entrée qui ne paniquait pas passe donc inchangée (avant :
  les vecteurs de longueur 173 à 181 étaient réduits sans nécessité). Tests `sim_core::animation_state` : (1) la prédiction égale la panique réelle (`catch_unwind`) sur une grille de 81 × 81 × 6 vecteurs autour des bornes ;
  (2) bornes de direction : sur un axe (`atan2` ne multiplie rien : aucune réduction, même à 400), diagonale à 300, cas de la graine 54. Cas d'axe trouvé par le test (la première version réduisait à tort un axe à 182).
  **Aucune trace ne bouge** (suite complète, ci-dessous).
- **`wall_art`** : `draw_walls` ne tient plus le marqueur `WallArtDone` (supprimé) : un niveau déjà dessiné se reconnaît à son enfant `WallArtRoot`. Lint : `wall_art.layer` doit être une couche **IntGrid**
  d'au moins une carte du jeu (`MapLevelInfo::int_grid_layers`, `HudWallArt` remplace le quadruplet) ; fixture `hud_wall_art_layer_unknown`, test `hud_wall_art_fixture_reports_unknown_layer`. Présentation
  seule : non joué avec rendu dans cette passe (aucune capture refaite), la logique est une substitution de condition.

### 6. Vérification (résultats réels)

- `make test_scenarios_par SHARDS=3` (sans BLESS) : **654 scénarios, trois tranches « ok », 301 s** ; aucune trace existante modifiée (`git status`), un seul fichier nouveau : `gungeon_key_chest.trace`.
- `make lint` ×5 (zombies, testbed, throne, gungeon, isaac) : **0 erreur** chacun. `cargo fmt --all` appliqué.
- `make test_crates` : **exit 0, 1 247 tests passés, 0 échec, 14 ignorés** (dont les deux tests de `sim_core::animation_state` et celui du lint `wall_art.layer`).
