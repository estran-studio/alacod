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

## 9. Game feel (mouvement, physique, fluidité)

Le pas de simulation est fixe à 60 Hz (`FIXED_TIMESTEP = 1/60`, `crates/game/src/character/movement.rs:7`). Le déplacement des joueurs n'est donc pas une vitesse instantanée : `run_velocity` approche la vitesse cible par axe, avec accélération et décélération (`movement.rs:147-176`). La décélération absente de la donnée reprend l'accélération (`movement.rs:131-135`). Les ennemis lisent surtout `max_speed`, l'IA décidant le reste (`movement.rs:9-11`).

| Donnée extraite des configs de personnages | zombies | throne | gungeon | isaac |
|---|---:|---:|---:|---:|
| `max_speed` min–max (tous les RON, tests inclus) | non trouvé | 0–150 | 0–150 | 0–120 |
| `dash_distance` min–max | non trouvé | 0–64 | 0–64 | 0 |
| `dash_duration_frames` min–max | non trouvé | 0–8 | 0–8 | 0 |
| `dash_cooldown_frames` min–max | non trouvé | 0–24 | 0–24 | 0 |
| `dash_iframes` trouvé | non trouvé | 6 (un RON) | 6 (un RON) | non trouvé |

Le dash porte distance, durée, cooldown, i-frames, charges et bousculade (`movement.rs:27-43,68-90`); le recul est une `Velocity.knockback` amortie à 0,85 par frame (`movement.rs:179-205`). Les recherches `knockback|recoil|shove` trouvent cette implémentation et le `Shove` de dash; aucun autre recul de tir documenté. Le feedback commun implémente flash, secousse, hit-stop, télégraphe et chiffres (`feedback.rs:1-13,218-220`; `feedback_log.rs:47-58`); la simulation continue pendant le gel de rendu (`games/*/assets/ui/feedback.ron:1-4`).

| Clés de `ui/feedback.ron` | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Effets | `hit_flash`, `shake`, `hit_stop_frames`, `damage_numbers`, `telegraph` | mêmes clés + `by_weapon` | mêmes clés + `pattern_telegraph`, `blank`, `boss_end`, `by_weapon` | mêmes clés + `pattern_telegraph`, `blank`, `boss_end`, `by_weapon` |
| Réglages communs | flash 4 frames, secousse 8/12, hit-stop 2, chiffres activés | idem | idem | idem |

Les quatre caméras utilisent `lerp_speed: 12`, marge 0,05, zoom par défaut 1, suivi du joueur local en ligne et `use_edge_detection: true` (`games/*/assets/camera.ron:1-13`). Le zoom minimal est 0,6 pour zombies/throne/Isaac et 0,5 pour gungeon; throne/gungeon/Isaac ajoutent fondu et recentrage d'étage, et Isaac ajoute `room_frame: true` (`camera.ron:5-19`). Après chaque frame GGRS, `FixedTransform3D` est recopié vers `Transform` en `PostUpdate`; aucune interpolation entre deux états fixes n'est trouvée (`crates/game/src/core_sim.rs:193-198`).

Les registres sprites contiennent des fichiers d'animation (zombies 16 RON, throne 14, gungeon 91, Isaac 157; comptage des fichiers `assets/sprites/**/*.ron`). Les champs d'animation et `FacingDirection` sont gérés par le système de personnages (`crates/game/src/character/visuals.rs:183-200,331-380`). Un décompte fiable « idle/marche/attaque/mort » par entité et le nombre de sprites fixes n'est pas trouvé dans les registres. Les recherches `muzzle|trail|impact|casing` dans `crates/game/src` ne trouvent pas de système de traînée, flash de bouche ou douilles; les morts sont suivies par `Death` et les visuels dérivés, sans système de disparition sèche identifié (`feedback.rs:288-295`; `character/visuals.rs:331-380`).

William a écrit que le movement-feel (course et dash) « marche assez bien », validé le 2026-10-09 (`docs/digests/revue-m0.md:31-33,45-48`; historique `docs/taches/rapports/fix-revue-m0.md:27-30`). Les points ouverts associés sont l'équilibrage du shotgun/pistolet/mitraillette, les sons de vagues et le son des tirs (`revue-m0.md:33-35`; `m0-revue-suite-contenu.md:47-58`), ainsi qu'une meilleure carte avec des spawners dans les autres salles (`revue-m0.md:38-48`).

## 10. Visuel complet

Les catégories de rendu communes sont les personnages joueurs, ennemis, boss, projectiles, objets/ramassables, portes, murs, sol, props, portail et HUD. Le registre d'apparence des personnages est partagé par le moteur (`crates/game/src/character/visuals.rs:183-200`); les chemins et statuts d'assets sont ceux des manifestes:

| Classe d'entité | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Joueurs, ennemis, boss | registre du jeu; statut de copie non trouvé | empruntés au testbed/placeholders (`games/throne/assets/game.ron:1-4`) | placeholders historiques puis silhouettes générées CC0 (`games/gungeon/assets/assets.yaml:1-12`; `sprites/sprites.ron:9-12`) | joueurs/ennemis/boss encore Kenney de `sprites/gungeon/` (`games/isaac/assets/assets.yaml:1-6`; `docs/taches/rapports/m3-salles-murs-isaac.md:34-35`) |
| Projectiles, objets, portes, murs, sol, props | registres du jeu; statut propre/emprunté non trouvé | registres du jeu; statut propre/emprunté non trouvé | générés/provisoires pour plusieurs objets et murs; détail par classe non trouvé | murs/portes/objets générés; détail des projectiles/props non trouvé (`m3-salles-murs-isaac.md:10-14,34-39`) |
| Portail et HUD | présents dans le moteur; sprite propre non trouvé | présents dans le moteur; sprite propre non trouvé | hub et `wall_art` présents (`game.ron:29-30,47-57`; `ui/hud.ron`) | `wall_art` présent (`ui/hud.ron`); sprite de portail propre non trouvé |

Tilesets: les fichiers LDtk de chaque jeu référencent `SunnyLand_by_Ansimuz-extended.png`, `SunnyLand-player.png` et `Dungeon_Tileset.png` (zombies `avant_poste.ldtk`/`la_fonderie.ldtk`, throne via `assets/caves/gabarit.ldtk`, gungeon `maps/*.ldtk`, Isaac `maps/*.ldtk`). Un tileset exclusivement propre à un clone n'est donc pas trouvé. `wall_art` est absent de zombies/throne et présent dans gungeon/Isaac (inventaire §3, lignes 69-72).

Un comptage final exhaustif propres/empruntés/absents par classe n'est pas trouvé: les manifestes déclarent des sources et licences, mais pas un champ de statut pour chaque entité. Les seuls totaux publiés sont les RON d'assets (39/57/198/349, inventaire §3:69) et les constats de copie ci-dessus.

## 11. Effets sonores complets

Les systèmes de feedback jouent explicitement les sons de tir, rechargement, coup porté, dégâts reçus, mêlée, mort, dash, blank et porte (`crates/game/src/feedback.rs:203-212`). Les clés réellement configurées sont:

| Clés `sounds` | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Tir/recharge | `shot_pistol`, `shot_shotgun`, `shot_smg`, `reload` | mêmes | mêmes | mêmes |
| Combat | `hit`, `melee`, `hurt`, `death`, `dash`, `door` | mêmes | mêmes + `blank` | mêmes + `blank` |
| Spécifique | non trouvé | non trouvé | `blank` | `blank` |

Les événements « tir par arme » sont regroupés par famille de mode et les armes peuvent remplacer tir/reload via `audio_config` (`games/*/assets/ui/feedback.ron:18-21`); il n'existe pas de clé dédiée par arme dans ces fichiers. Pour impact, dégâts reçus, mort du joueur/ennemi et attaque de mêlée, les clés correspondantes existent. Aucun son dédié n'est trouvé pour vague/salle/étage, achat, ramassage, boss ou explosion; bombe et menu ne sont pas des clés `sounds` dans ces quatre RON. La mort d'ennemi utilise la même clé `death` que les autres morts (`feedback.rs:203-212`).

Chaque jeu contient 10–11 fichiers dans `assets/sounds/` (zombies 11, throne 10, gungeon 11, Isaac 11; inventaire §5:107). Les chemins des lots A/B et licences sont déclarés dans `assets/assets.yaml`; les registres gungeon et Isaac indiquent notamment CC BY 3.0/CC0 (`games/gungeon/assets/assets.yaml:11-98`; `games/isaac/assets/assets.yaml:25-95`). Les noms communs (`hit.ogg`, `death.ogg`, `shot_pistol.ogg`, `shot_shotgun.ogg`, `shot_smg.ogg`, `reload.ogg`, `dash.ogg`, `door.ogg`) sont présents dans les quatre répertoires; une comparaison MD5 complète n'est pas trouvée dans l'inventaire. Le volume est réglable par clé en décibels (`volumes`, plage documentée -60 à +12) et les quatre clones utilisent principalement -12 dB (`games/*/assets/ui/feedback.ron:35-38`).

## 12. Map design

| Donnée des cartes | zombies | throne | gungeon | isaac |
|---|---:|---:|---:|---:|
| Fichiers LDtk / niveaux | 2 / 25 (16 `la_fonderie`, 9 `avant_poste`) | gabarit de caverne présent; nombre de niveaux LDtk non trouvé | 4 / 20 (3 `etage_1`, 15 `salles`, 1 armes, 1 hub) | 2 / 24 (23 `salles`, 1 armes) |
| Gabarits/grammaires | 1 `floor_grammars`, 6 `rooms` | 5 `caves`, 1 `floors/run.ron` | 7 `floor_grammars`, 10 `floors`, 9 `rooms` | 6 `floor_grammars`, 7 `floors`, 9 `rooms` |
| Tailles relevées | 416×304; 512×512/1024×512/512×1024 | non trouvé | 192×160 à 352×256 pour `salles`; hub 256×192 | 480×288, 480×576, 960×288, 960×576 |

Zombies assemblait l'avant-poste par modules et graine; les sources indiquent 5–8 pièces, portes, obstacles et spawners (`docs/digests/revue-m0.md:297-315`). La carte « La Fonderie » est présente dans `games/zombies/assets/maps/la_fonderie.ldtk` et sa grammaire dans `assets/floor_grammars/la_fonderie.ron` (grep `fonderie` dans `games`/`docs`). Le défaut documenté est que les cinq spawners d'`avant_poste` étaient dans `Depart`, les autres salles sans menace (`docs/digests/revue-m0.md:38-48`; `m0-revue-suite-contenu.md:60-65`); des couloirs problématiques sont signalés dans les rapports de génération, mais un décompte de couloirs n'est pas trouvé.

Throne assemble des étages à partir des cavernes (`assets/caves/*.ron`, `floors/run.ron`); un nombre fixe de salles par étage et une variété par type ne sont pas trouvés. Le rapport de revue vise explicitement les cavernes (`docs/taches/dettes-d66-vue-directe.md:13-16`), sans inventaire de salles équivalent.

Gungeon utilise des grammaires de plancher et des salles typées (`combat`, `arene`, `secret`, `recompense`, `depart`, `boss`, `boutique`, `escouade`, `assets/rooms/*.ron`). Le rapport documente des salles de combat et des agrandissements; les salles devenues vides après agrandissement sont D64/D118 (`docs/taches/rapports/bot-explorateur-v2.md:77-80`; `docs/taches/dettes.md:72,128`). Les props de couverture (tables/barils/rochers) sont des entités de monde, mais leur nombre par gabarit n'est pas trouvé.

Isaac assemble les étages par grammaires `sous_sol*` et salles typées (`combat`, `transit`, `depart`, `tresor`, `secrete`, `diable`, `boss`, `boss_profond`). Les rapports donnent 23 gabarits et une unité de salle de 480×288 (`docs/taches/rapports/m3-salles-murs-isaac.md:46-52,78-79`). Les défauts cartographiques ouverts sont D123/D125 (limites visuelles, cadrage/caméra et mur fissuré) (`docs/taches/dettes.md:133-135`; `m3-salles-murs-isaac.md:179-182`). Les rochers, tables et barils sont couverts par les props du monde; les comptes par étage ne sont pas trouvés.

## 13. Contraintes web et manette

La cible web publiée par le Makefile construit seulement zombies et throne (`Makefile:379-380`); le script refuse tout autre jeu (`scripts/build-web-games.mjs:6-8`) et compile `-p <jeu> --target wasm32-unknown-unknown --no-default-features --features render` (`build-web-games.mjs:28-36`). Le catalogue commité contient zombies et throne, avec `solo: true` et `online: true` (`website/static/releases.json:3-16`), mais le contrat précise que les boutons online restent désactivés tant que la recette complète n'est pas autorisée (`docs/contrat-lancement-web.md:39-45,78-81`). `/play?id=zombies|throne` transmet les paramètres publics de build et jeu; solo n'envoie ni JWT, salon ni ICE (`contrat-lancement-web.md:6-18`).

Dans le navigateur, `BrowserChannel` encapsule le canal WebRTC Matchbox et GGRS (`crates/game/src/jjrs/browser.rs:44-70`). `HostedSession` et `ReturnToHostLobby` organisent le retour au salon par crochet JavaScript (`crates/game/src/run_state.rs:338-350,507-513`). Le flux HTTP Allumette challenge/login/lobby/ICE est natif seulement (`crates/game/src/jjrs/mod.rs:9-12`; `crates/game/src/jjrs/allumette.rs:1-10`); le navigateur reçoit donc une URL WSS/ticket par l'intégration site, et le contrat exige une URL WSS exacte (`docs/contrat-lancement-web.md:20-32`). Une preuve TURN/WebRTC dans deux Chromium existe, mais la preuve sur deux réseaux et le catalogue online publié restent ouverts (`contrat-lancement-web.md:83-100`).

La dépendance `leafwing-input-manager` est en version 0.21.0 (`crates/game/Cargo.toml:22`; `Cargo.lock:7479-7482`). `bevy_gilrs`/`gilrs` apparaît dans le lockfile (`Cargo.lock:1775-1802,6256-6270`), mais une activation WASM explicite et une liaison de manette navigateur ne sont pas trouvées; les contrôles codés utilisent notamment clavier/souris et `GamepadStick::LEFT` (`crates/game/src/character/control.rs:58-108`). Aucun `wasm-bindgen-rayon`/`atomics` n'est trouvé dans les manifestes ou le script web. Aucune mesure FPS WASM n'est trouvée dans `docs/taches/rapports/*.md`. L'audio web utilise le plugin Kira du moteur (`crates/game/src/audio/`), mais une preuve spécifique d'autoplay/déblocage au premier clic n'est pas trouvée.

## Résumé 2

1. Le mouvement joueur est accéléré/freiné en fixed-point à 60 Hz; le rendu recopie l'état fixe sans interpolation trouvée.
2. Dash, i-frames, knockback amorti, flash, secousse, hit-stop et chiffres de dégâts existent; traînées de balle, douilles et flash de bouche non trouvés.
3. Les quatre caméras ont le même lerp; Isaac ajoute le cadrage de salle.
4. Les registres d'animation existent, mais aucun décompte fiable par type d'animation n'est publié.
5. Les registres de sprites documentent surtout des placeholders/copies; les statuts propres par classe ne sont pas exhaustifs.
6. Les quatre jeux partagent les tilesets SunnyLand/Dungeon référencés par LDtk.
7. Les sons couvrent tir, reload, hit, hurt, melee, death, dash et door; blank s'ajoute à gungeon/Isaac.
8. Aucun son dédié de vague, achat, ramassage, boss ou explosion n'est trouvé dans les feedback RON.
9. Les volumes par clé sont réglables en dB; aucune comparaison MD5 publiée n'est trouvée.
10. Zombies possède l'avant-poste et La Fonderie; ses spawners concentrés dans une salle sont documentés.
11. Throne assemble des cavernes, mais le nombre de salles par étage n'est pas trouvé.
12. Gungeon et Isaac utilisent grammaires, floors et salles typées; leurs dettes de cartes sont documentées.
13. Le build WASM officiel cible zombies et throne; le script refuse gungeon et Isaac.
14. Le catalogue commité annonce solo/online pour ces deux jeux, avec boutons online contractuellement désactivés en attente de recette.
15. La manette navigateur, les FPS WASM, les threads wasm et l'audio autoplay n'ont pas de preuve complète trouvée.
