# Inventaire factuel — quatre clones jouables

Comptages de fichiers réalisés par parcours récursif de `games/<jeu>/assets` (extension `.ron`) et des fichiers présents dans `assets/sounds`. « Non trouvé » signifie qu'aucune occurrence n'a été trouvée dans les chemins indiqués.

## 1. Lancer une partie en ligne sans terminal

| | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Lancement local | `make zombies` (cible Makefile:325-328) | `make throne` (Makefile:332-333) | `make gungeon` ; démarre au hub (docs/jouer.md:24-27) | `make isaac` (rapport m3-salles-murs-isaac.md:210, 250) |
| Local à 2 | `--players localhost localhost` (docs/taches/rapports/dette-d69-e4-personnage.md:92) | même option (même table d'entrées) | commande explicite `--players localhost localhost`; les deux joueurs reçoivent les mêmes touches/manette (docs/jouer.md:28) | commande explicite `--players localhost localhost` (rapport m3-salles-murs-isaac.md:210) |
| En ligne documenté | `--matchbox ... --number-player 2 --players localhost remote` (docs/jouer-a-deux.md:39-43) | même client commun, cible `throne_matchbox` non trouvée dans Makefile | procédure détaillée, deux machines, `--cid`/`--name` distincts (docs/jouer.md:34-60) | même procédure que gungeon avec le binaire isaac (rapport m3-salles-murs-isaac.md:249-251) |
| Limite déclarée | `--number-player` est paramétrable; aucune valeur propre trouvée dans `game.ron` | idem | essais documentés à 2; Makefile `NUMBER_PLAYER ?= 2` (Makefile:8-10) | essais documentés à 2 et 4 (rapport m3-salles-murs-isaac.md:164-169) |
| Manifeste / hub | `entry.mode: Waves`, pas de hub (game.ron:40-53) | `mode: Floors`, pas de `hub` (game.ron:29-36) | `hub: Some((map: "maps/hub.ldtk"))` (game.ron:47-57); dossier `assets/hub/` présent | pas de hub dans le manifeste (game.ron:38-43) |

Signaling: le défaut Makefile est `wss://allumette.estran.studio` (`Makefile:18-23`). Le rapport de lancement indique que ce serveur public échoue en ligne de commande (« socket dropped ») et que `--allumette` demande un salon créé par le site; la procédure qui fonctionne utilise `docker compose -f docker-compose.ci.yaml up -d signaling`, port 3536 (`docs/jouer.md:36-62`). L'ancien `allumette.bascanada.org` ne résout plus selon `docs/jouer-a-deux.md:7-11` et `docs/etat-engine.md:125-128`. Le rapport M3 note que les essais deux machines actuels d'Isaac ne sont pas vérifiés sur le contenu actuel (`m3-salles-murs-isaac.md:166-177`).

Web/distribution: le Makefile contient `TARGET=web`, `build_wasm_apps` et `build_web_games` (Makefile:74-80, 379-380, 468-471). Le contrat web décrit `/play?id=zombies|throne`, un catalogue `website/static/releases.json`, et précise que les boutons online restent désactivés jusqu'à validation du contrat (`docs/contrat-lancement-web.md:6-14,41-45`). La présence d'un build WASM a été rapportée pour zombies (`dette-d69-e4-personnage.md:78`); pour les quatre clones, une publication distribuable effective n'est pas trouvée. `APP_VERSION` est injecté par le Makefile/build.rs (`Makefile:33-40`).

## 2. Contrôles

| Action | Clavier/souris | Manette | Source ligne |
|---|---|---|---|
| `MoveUp` | W | D-pad haut | `control.rs:62,91` |
| `MoveDown` | S | D-pad bas | `control.rs:64,92` |
| `MoveLeft` | A | D-pad gauche | `control.rs:66,93` |
| `MoveRight` | D ou flèche droite | D-pad droit | `control.rs:68-69,94` |
| `Interaction` | H | bouton Nord | `control.rs:70,95` |
| `Sprint` | Shift gauche | aucune | `control.rs:86` |
| `Dash` | C | aucune | `control.rs:87` |
| `SwitchWeapon` | Tab | aucune | `control.rs:71` |
| `SwitchWeaponMode` | Z | aucune | `control.rs:72` |
| `Reload` | R | bouton Ouest | `control.rs:73,96` |
| `MeleeAttack` | F | bouton Est | `control.rs:74,97` |
| `DropWeapon` | L | aucune | `control.rs:75` |
| `UseActive` | Espace | aucune | `control.rs:76` |
| `Blank` | Q | aucune | `control.rs:77` |
| `Bomb` | E | aucune | `control.rs:78` |
| `ChoiceA` | 1 | aucune | `control.rs:79` |
| `ChoiceB` | 2 | aucune | `control.rs:80` |
| `ChoiceC` | 3 | aucune | `control.rs:81` |
| `ChoicePrev` | flèche gauche | D-pad gauche | `control.rs:82,98` |
| `ChoiceNext` | flèche droite | D-pad droit | `control.rs:83,99` |
| `ChoiceConfirm` | Entrée | bouton Sud | `control.rs:84,100` |
| `Modifier` | Ctrl gauche | aucune | `control.rs:88` |
| `Pan` (visée) | position de souris (pas un bouton) | stick gauche (`GamepadStick::LEFT`) | `control.rs:107` et `input_devices.rs:43-50` |
| `PointerClick` (tir) | clic gauche | aucune; aucune gâchette n'est associée | `control.rs:102` |
| `SwitchLockMode` | P | aucune | `control.rs:104` |
| `SwitchToUnlockMode` | O | aucune | `control.rs:105` |
| `PointerPosition` | position de souris | aucune | `control.rs:43,102`; lecture du curseur `input_devices.rs:43-50` |
| `SwitchTargetPlayer` | aucune liaison dans `get_input_map` | aucune | `control.rs:46` |
| `MoveCameraUp` | flèche haut | aucune | `control.rs:63` |
| `MoveCameraDown` | flèche bas | aucune | `control.rs:65` |
| `MoveCameraLeft` | flèche gauche | aucune | `control.rs:67` |
| `MoveCameraRight` | flèche droite | aucune | `control.rs:85` |
| `DebugForceCrash` | F12 | aucune | `control.rs:61` |

Le code associe la visée au **stick gauche** (`GamepadStick::LEFT`, `control.rs:107`). Il n'associe aucune gâchette (`GamepadAxis` ou bouton de gâchette) au tir: le tir passe par `PointerClick`, exclusivement `MouseButton::Left` (`control.rs:102`). La table manette de `docs/jouer.md` est donc une description partielle de la table de code, et non une preuve d'une liaison de gâchette (`docs/jouer.md:96-97`).

| Élément | Fait et source |
|---|---|
| Actions codées | `PlayerAction` expose déplacement, interaction, sprint, dash, changement d'arme/mode, reload, mêlée, lâcher, actif, blank, bombe, choix de mutation, modifier, pointeur et caméra (`crates/game/src/character/control.rs:5-56`). |
| Remappage | Non trouvé dans les données de jeu; la table est construite par `get_input_map` (`control.rs:58-108`) et attachée aux joueurs locaux par `input_devices.rs:23-26`. |
| Essai manuel William | §8 demande clavier/souris, manette branchée/débranchée, premier appui, mutation, lâcher d'arme, lumière, deux joueurs; clavier et manette sont explicitement « non essayés » (§8 et §9, `dette-d69-e4-personnage.md:83-99`). |

## 3. Lisibilité et visuels

| | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| RON dans assets (comptage) | 39 | 57 | 198 | 349 |
| Entrées non-commentaires dans `assets.yaml` (comptage de lignes) | 201 | 158 | 165 | 202 |
| `wall_art` dans `ui/hud.ron` | non trouvé | non trouvé | présent | présent |
| Statut des sprites | Manifestes et dossiers historiques; non trouvé de ligne `statut: placeholder` indiquant une copie pour zombies | `game.ron` dit explicitement sprites/interface/caméra copiés du testbed, placeholders (`games/throne/assets/game.ron:1-4`) | aucun asset propre au départ; placeholders copiés de throne/testbed/zombies, puis silhouettes générées CC0 (`games/gungeon/assets/assets.yaml:1-9`; `sprites/sprites.ron:9-12`) | tous les fichiers copiés de gungeon; placeholders (`games/isaac/assets/assets.yaml:1-6`; `game.ron:1-5`); personnages/ennemis/boss restent les tuiles Kenney de `sprites/gungeon/` (`m3-salles-murs-isaac.md:34-35`) |
| Visuels propres importés | `plan-assets.md` couvre le registre et le remplacement de sprites partagés; aucun import propre supplémentaire trouvé | non trouvé | lot C génère le langage visuel provisoire; les originaux sont remplacés | génération de 121 fichiers de murs/portes/objets; les personnages/ennemis/boss restent partagés avec gungeon (`m3-salles-murs-isaac.md:10-14,34-39`) |

Licences: les registres indiquent CC BY 4.0, CC0 Ansimuz, licence Pixel_Poem à valider, et sons CC BY 3.0/CC0 (`games/gungeon/assets/assets.yaml:11-66,68-98`; même registre repris pour Isaac, `games/isaac/assets/assets.yaml:25-63`). Le plan D32/D6 prévoit inventaire, licence, statut placeholder et remplacement des sprites partagés (`docs/taches/plan-assets.md:15-20`).

Dettes visuelles demandées: D66 (« ennemis immobiles/orientation figée ») et D67 (« télégraphe de salve ») sont lignes ouvertes dans `docs/taches/dettes.md:74-75`; D123 (limites visuelles Isaac) et D125 (cadrage/caméra et mur fissuré non vus avec rendu) sont ouvertes (`dettes.md:133-135`, état détaillé `m3-salles-murs-isaac.md:179-182`).

## 4. Équilibrage et volume de contenu

Campagnes publiées:

- **Gungeon**: campagne `etages_deux`, 200 graines: le rapport final de gabarits donne 27 % de morts à 1 bot, 3,5 % à 2 bots, 0 désynchronisation; 4 soft-locks résiduels à 1 bot dans le rapport (`docs/taches/rapports/m2-t14d-gabarits.md:91` et `docs/taches/dettes.md:72`).
- **Isaac**: T18, 200 graines à 1 et 2 bots, 20 à 4: 24/175/1 soft-lock à 1 bot, 95/105/0 à 2, 15/5/0 à 4 (`m3-t18-integration-sortie.md:89-101`). La campagne après salles ×2 donne 138 victoires/60 pertes à 1 bot, 183/14 à 2, 18/2 à 4; 0 desync sur 420 graines (`m3-salles-murs-isaac.md:114-124`).
- **Zombies**: aucune campagne `alacod-sim` chiffrée trouvée dans `docs/taches/rapports/*.md`; le rapport du relais contient seulement une commande prévue (`docs/captures/le-relais/README.md:255-258`).
- **Throne**: aucune campagne `alacod-sim` chiffrée trouvée dans `docs/taches/rapports/*.md`.

Dettes d'équilibrage demandées: D64 est ouverte (salles gungeon trop mortelles; historique 200/200 puis 27 %/3,5 %), D93 est ouverte/réduite (scénarios Isaac gagnant par armure), D119 est ouverte (bot ignore piques), D121 est ouverte (soft-lock devant tas de sous), D124 est ouverte (soft-lock devant colosse); sources `docs/taches/dettes.md:72,103,129-134` et `m3-t18-integration-sortie.md:205-230`. Q14 reste non tranchée (`m3-t18-integration-sortie.md:119-120`).

Comptage effectué avec cette commande Python (sans compilation): `python3 - <<'PY' ...` parcourant `games/<jeu>/assets`, comptant les clés de niveau 1 (`^    "..."`) dans les RON d'armes, les fichiers RON dans `characters/`, `objets/` et `rooms/`, les fichiers de configuration de personnages zombies, et `len(json.load(open("maps/salles.ldtk"))["levels"])` pour les gabarits LDtk. Un boss est compté lorsqu'un RON de personnage/configuration contient le mot `boss` ou `phases`.

| Contenu (entrées, pas nombre de fichiers) | zombies | throne | gungeon | isaac |
|---|---:|---:|---:|---:|
| Armes (clés dans `weapons.ron`) | 4 | 13 | 21 | 3 |
| Armes de mêlée (clés dans `melee*.ron`) | non trouvé | 4 | 1 | 2 |
| Ennemis/personnages (configs RON) | 6 | 14 | 11 | 24 |
| Objets (`objets/*.ron`) | non trouvé | non trouvé | 20 | 100 |
| Boss (configs contenant `boss` ou `phases`) | 1 (`boss_zombie_config.ron`) | 1 (`roi_rat.ron`) | 1 (`gatling.ron`) | 9 |
| Gabarits de salles (`maps/salles.ldtk`, nombre de `levels`) | non trouvé | non trouvé | 15 | 23 |

Sources structurelles: configs zombies déclarées `game.ron:11-20`; armes/personnages throne `game.ron:8-16`; armes/personnages/objets/salles gungeon `game.ron:9-23,31-45`; armes/personnages/objets/salles isaac `game.ron:9-36`. Les 23 gabarits Isaac sont également décrits dans `m3-salles-murs-isaac.md:46-52`.

## 5. Son et feedback

| | zombies | throne | gungeon | isaac |
|---|---:|---:|---:|---:|
| Fichiers dans `assets/sounds/` | 11 | 10 | 11 | 11 |
| Entrées audio dans `assets.yaml` | registre partagé; sons lot A/B déclarés | registre partagé; sons lot A/B déclarés | entrées lot A/B, non-placeholder (`assets.yaml:68-98`) | mêmes entrées et statuts (`assets.yaml:65-95`) |
| Musique | aucune occurrence `music` dans `games/zombies/assets` | aucune occurrence trouvée | aucune occurrence trouvée | aucune occurrence trouvée |

`FeedbackKind`/feedback joue les sons de tir et reload (`feedback.rs:907-950`), puis `hit`, `hurt`, `melee`, `death`, `dash`, `blank`, `door` (`feedback.rs:1172-1351`). `ALACOD_MUTE=1` coupe tous les sons de feedback (`feedback.rs:1172-1177`; `docs/jouer.md:101-105`). D32 est enregistrée comme tir soutenu de 17,7 s à remplacer par un coup unique; le plan-assets prévoit le lot A (trois familles d'armes) et le lot B d'événements (`docs/taches/dettes.md:40`; `docs/taches/plan-assets.md:31-34`). Les restes M0 zombies S4 (équilibre des armes), S6 (son de vague) et S7 (tirs trop forts) sont gardés en file (`docs/taches/m0-revue-suite-contenu.md:47-58`).

## 6. Menus et écrans

- **Lobby**: l'écran local affiche le résumé de la partie quittée et attend Entrée ou « Nouvelle partie » (`crates/game/src/ui/lobby.rs:40-142`). Le lobby réseau affiche la liste des joueurs et un état de connexion (`lobby.rs:144-233`), mais aucun choix de personnage ou de nombre de joueurs n'est défini dans ce module; ces valeurs viennent des arguments (`GggrsSessionConfiguration`).
- **Pause/inventaire**: Échap/Start ouvre la pause, I/Select l'inventaire; flèches/D-pad naviguent, Entrée/A confirme, Retour/B revient. La simulation continue écran ouvert (`crates/game/src/ui/screens/mod.rs:1-10`).
- **Fin de partie**: résumé dérivé de `Run`; boutons « Rejouer » et « Lobby », touche R pour relancer (`crates/game/src/ui/game_over.rs:36-41,80-125`).
- **Hub**: seul gungeon a `hub: Some(...)` dans son manifeste (`games/gungeon/assets/game.ron:29-30,47-57`); le dossier `games/gungeon/assets/hub/` est présent. Non trouvé pour zombies, throne ou isaac.
- **Déconnexion/desync**: le code documente l'écran « DESYNC » quand la partie commune est perdue (`docs/jouer.md:68-69`); le lobby montre « Connecting to peer... », « new attempt (2/2) », « back to the lobby » (`docs/jouer.md:64-66`; `docs/conventions.md:4590-4597`).
- **Langues affichées relevées**: mélange français/anglais: « Nouvelle partie », « Lobby », « Rejouer », « DESYNC », « Connecting to peer... », « back to the lobby », « Waiting for players », « Frame » (`lobby.rs:124-142`; `game_over.rs:120-125`; `docs/jouer.md:64-69`; `docs/conventions.md:4590-4597`).

## 7. Performance

Budgets: défaut 40 fps simulées (`tests/budgets.ron:14-16`); gungeon `bench_floor` 18 fps/74 ms, runs 21/50 et 15/60, throne 23–29 fps selon scénario, Isaac `bench_floor_isaac` 26 fps/40 ms (`tests/budgets.ron:17-24,68-80`). Budgets spécifiques zombies: `four_players_idle` et `four_players_shooting` 33 fps (`tests/budgets.ron:36-50`).

Extraction Python de `/home/debian/orca/workers/b6/target/metrics/latest.json` (date mtime: **2026-10-20 10:56:43**, répertoire métrique le plus récent: `target/metrics/0d2c1b61/metrics.json`, donc commit **0d2c1b61**). Le champ du pire rollback est `rollback_ms_worst`. « Oui » signifie `sim_fps >= min_sim_fps` et, lorsqu'il existe, `rollback_ms_worst <= max_rollback_ms`; « n/a » signifie qu'aucun budget de ce scénario n'est défini dans `tests/budgets.ron`.

| Scénario | sim_fps | rollback_ms_worst (ms) | plancher / plafond | sous budget ? |
|---|---:|---:|---:|---|
| `bench_bullets` | 59,232 | 16,851 | 41 / 24 | Oui |
| `bench_bullets_room` | 45,686 | 23,981 | 31 / 36 | Oui |
| `bench_cave` | 75,984 | 14,116 | 44 / 24 | Oui |
| `bench_familiars` | 87,272 | 13,023 | 42 / 27 | Oui |
| `bench_floor` | 33,190 | 37,193 | 18 / 74 | Oui |
| `bench_floor_isaac` | 22,702 | 35,310 | 26 / 40 | **Non** (fps) |
| `bench_horde` | 42,044 | 37,817 | 26 / 61 | Oui |
| `bench_tears` | 31,815 | 45,933 | 22 / 65 | Oui |
| `bench_tears_trio` | 74,892 | 16,927 | 48 / 27 | Oui |
| `four_players_idle` | 70,531 | 13,542 | 33 / — | Oui |
| `four_players_shooting` | 71,277 | 13,248 | 33 / — | Oui |
| `gungeon_full_run` | 39,073 | 24,472 | 21 / 50 | Oui |
| `gungeon_full_run_four` | 27,234 | 31,700 | 15 / 60 | Oui |
| `isaac_duo` | 20,495 | 34,426 | non trouvé / non trouvé | n/a |
| `isaac_full_run` | 36,293 | 27,526 | non trouvé / non trouvé | n/a |
| `isaac_quad` | 10,121 | 50,352 | non trouvé / non trouvé | n/a |
| `throne_quad` | 44,517 | 29,024 | 23 / — | Oui |
| `throne_three_floors` | 53,275 | 23,887 | 27 / — | Oui |

Scénarios demandés mais absents de `latest.json`: aucun parmi les noms recherchés ci-dessus. Le fichier `/home/debian/orca/workers/b2/target/metrics/latest.json` reste absent.

Mesures et dettes: D70 mesure 9,1 ms/frame pour reconstruire le champ de flux avec quatre joueurs et quatre zombies (`docs/taches/dettes.md:80`); D118 mesure gungeon 45,9→34,2 fps et pire rollback 25,0→36,9 ms après agrandissement (`dettes.md:128`); D122 mesure Isaac 1 500→2 502 µs de champ de flux et environ 28 fps contre 53 sous charge comparable (`dettes.md:132`; `m3-salles-murs-isaac.md:78-79`). Le rapport gungeon décrit une machine Mac Apple Silicon pour les commandes de jeu et des campagnes, mais aucune mesure « vraie machine » de performance dédiée n'est publiée dans le §5 (`docs/taches/rapports/m2-salles-murs-gungeon.md:1-5`).

## 8. Revue humaine : ce qui attend William

- `m3-t18-integration-sortie.md` §6: produire les vidéos, faire `review_videos`, jouer trente minutes en coop, juger lisibilité des larmes/cœurs/familiers/champions et le tir ami de `nova`, puis transformer chaque note en attente/dette/tâche (`m3-t18-integration-sortie.md:235-259`).
- `dette-d69-e4-personnage.md` §8: tester réellement clavier/souris et manette, premier appui, mutation, lâcher d'arme, lumière et deux joueurs; la section §9 dit ces périphériques non essayés (`dette-d69-e4-personnage.md:83-99`).
- `m3-salles-murs-isaac.md` §7: regarder les captures des salles, murs, bombe/mur secret, rochers, piques, boutique, champions, caméra et coop (`m3-salles-murs-isaac.md:179-210`).
- `docs/taches/session-test-manuelle.md`: enregistrer OBS et transcription, jouer les blocs zombies/throne, noter déplacement/tir/lisibilité/économie/difficulté/envie de rejouer, puis déposer transcriptions, scénarios et logs (`session-test-manuelle.md:1-26,66-119,121-135`).

## Résumé

1. Les quatre jeux ont une cible locale `make <jeu>`; seul gungeon documente complètement le hub et la procédure en ligne.
2. Le signaling public `allumette.estran.studio` est décrit comme échouant en ligne de commande; le signaling local Docker est la procédure documentée.
3. La limite de deux joueurs est la valeur Makefile par défaut; Isaac a aussi des preuves à quatre clients sur une machine.
4. Aucun remappage de contrôles n'est trouvé; plusieurs actions n'ont pas de bouton manette documenté.
5. Gungeon et Isaac déclarent explicitement des placeholders copiés d'autres jeux; Isaac garde les personnages/ennemis Kenney de gungeon.
6. Les visuels Isaac ont reçu murs/portes/objets générés, mais D123 et D125 restent ouvertes.
7. Gungeon a une campagne publiée à 27 % de morts à un bot et 3,5 % à deux bots.
8. Isaac publie 420 graines sans désync; D119, D121, D124 et Q14 restent ouverts/non tranchés.
9. Zombies et throne n'ont pas de campagne `alacod-sim` chiffrée trouvée dans les rapports.
10. Les quatre jeux ont environ 10–11 fichiers son; aucune musique n'est trouvée dans leurs assets.
11. D32 reste la dette du son de tir soutenu; S4/S6/S7 zombies sont encore en file.
12. Pause/inventaire, fin de partie et retour lobby sont communs; le hub déclaré est celui de gungeon.
13. Les textes affichés mélangent français et anglais.
14. Le budget de référence est 40 fps; les budgets lourds gungeon/Isaac sont réduits par D118/D122 et D70 concerne le champ de flux.
15. Les revues humaines, essais périphériques et sessions coop de William sont encore explicitement demandés.
