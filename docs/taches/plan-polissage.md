# Plan de la semaine de polissage des quatre clones — révision du 2026-10-21

Rédigé le 2026-10-21 (branche `plan-polissage`), **révisé le même jour après revue de William** : les jeux se jouent
**seulement dans le navigateur, via le site** (`website/`) ; pas d'écran de démarrage dans le jeu ; **la manette est
cruciale** ; quatre axes manquaient (game feel, visuel complet, effets sonores complets, map design). Décision
d'origine (2026-10-19/21) : pause des mécaniques, polissage des quatre clones, entre M3 et M4 (`docs/taches.md` §8.0).
Inventaire factuel : `docs/taches/rapports/plan-polissage.inventaire.md` (« inventaire », sections 1 à 13).

Convention : chaque fait cite sa source ; `[INFERENCE]` = déduit, non observé. Aucun chiffre inventé : « non mesuré » ou
« non trouvé » dit ce qui manque, et **la tâche correspondante commence alors par un audit chiffré**. Les estimations
(jours-agent) sont toutes des `[INFERENCE]` de planification.

## 1. Objectif

> « 4 jeux équilibrés, fun, fluides et complets ; pas raffinés au niveau d'un jeu release, mais quelque chose de
> propre » — pour que des amis y jouent ensemble en ligne, et pour servir de démo de vente de l'engine (William,
> 2026-10-19/21). Ensuite viendra 1837 (hors périmètre).

**Définition de « fini » par axe** (un critère observable ; « par clone » = vrai pour chacun des quatre) :

| Axe | Fini quand… (vérifiable) |
|---|---|
| 1. Web et en ligne | **Depuis le site, un ami choisit un clone et solo ou multi (salon), et joue dans son navigateur, à la manette.** Les quatre clones sont dans `releases.json` (`solo` et `online` vrais), la partie à 2 s'établit entre **deux réseaux distincts** (preuve ouverte aujourd'hui, `contrat-lancement-web.md:97-100`), 0 `DESYNC`, retour au salon propre ; le signaling est en `wss://` joignable d'un navigateur. |
| 2. Contrôles | Par clone, **toutes les actions utiles et tous les menus** (pause, inventaire, fin de partie, choix de mutation) se jouent à la manette **dans le navigateur** (tir compris) ; schéma validé par William ; un écran d'aide liste les boutons ; William a fait l'essai E4 (`dette-d69-e4-personnage.md` §8). |
| 3. Lisibilité | Sur capture/vidéo de chaque clone à 2 et 4 joueurs : son héros se distingue (couleur par handle), un ennemi qui tire se distingue d'un ennemi qui marche (D66/D67), un ramassable d'un décor. |
| 4. Équilibrage | Par clone, un tableau de campagne chiffré (bot `explorateur`, 1 et 2 bots, ≥ 100 graines, morts par cause) et un essai de William de 30 min en coop : « ni trop dur ni trop facile ». La campagne informe, William tranche (pas de fourchette automatique). |
| 5. Son et feedback (réglages) | Pas de tir soutenu (D32), tirs non pénibles (S7), un **réglage de volume global** dans le jeu, licences à jour ; le son démarre au premier clic dans le navigateur. |
| 6. Menus et écrans | Pause → fin de partie → retour au salon du site, **tout en français**, déconnexion et `DESYNC` expliqués avec une action ; **aucun menu de démarrage dans le jeu** (le site connecte). |
| 7. Performance | Dans le navigateur (Chrome, Mac de William) : FPS d'affichage et FPS de simulation **mesurés** par clone à 2 joueurs, aucune partie qui tombe sous la vitesse de simulation à 60 Hz [seuil à fixer par la mesure] ; en natif, `make bench` strict vert au calme. |
| 8. Game feel | Pour chaque clone, une vidéo de 60 s de prise en main où mouvement, dash, tir et coups reçus ont un retour net (accélération/friction réglées en données, recul, flash, secousse, hit-stop, caméra stable), le rendu est **fluide à toute fréquence d'écran** (interpolation d'affichage), et chaque personnage a ses animations idle/marche/attaque/mort. William dit « agréable » sur course, dash et tir. |
| 9. Visuel complet | Un tableau « classe d'entité × clone » (joueur, chaque famille d'ennemi, boss, projectile, ramassable, porte, mur, sol, prop, portail, HUD) où **chaque case est « propre au jeu »** (plus de tuile d'un autre jeu), animée pour les personnages ; sol et murs propres par clone (`wall_art` aussi pour zombies et throne). Pas de niveau release : cohérent et lisible. |
| 10. Effets sonores complets | Un tableau « événement × clone » où chaque événement a un son : tir par arme, impact, dégâts reçus, mort par famille, boss, porte, achat, ramassage, vague/salle/étage, explosion/bombe, menus. Aucune case vide. |
| 11. Map design | Par clone, un audit de cartes (nombre de salles/gabarits, spawners par salle, couvertures, couloirs problématiques) et les défauts relevés corrigés : variété, rythme (pas de salle vide ni de couloir mort), spawners dans toutes les salles de combat ; William valide à la vidéo. |

## 2. État mesuré

Verdicts : ✅ fait et prouvé · ⚠️ partiel ou non vérifié sur la cible · ❌ manquant. Source entre parenthèses.

### Axe 1 — Web et en ligne

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Dans le catalogue du site | ✅ `releases.json` (build `rtc-fix-20261009`, `solo` et `online` vrais, `website/static/releases.json:3-16`) ; entrée dans `catalog.ts` | ✅ idem (`catalog.ts`, `image: ''`) | ❌ absent de `releases.json` et de `catalog.ts` ; `scripts/build-web-games.mjs:6-8` refuse le jeu | ❌ idem |
| Compile en wasm | ✅ build du 2026-10-09 (`contrat-lancement-web.md`) | ✅ idem | ⚠️ non tenté : même `Cargo.toml` (`webgl2`, `getrandom` js à confirmer) mais **jamais compilé en wasm** [INFERENCE : aucune trace dans les rapports] | ⚠️ idem |
| Online navigateur | ⚠️ `BrowserChannel` (`jjrs/browser.rs:44-70`), `HostedSession`/`ReturnToHostLobby` (`run_state.rs:338-350`) ; **preuve deux Chromium sur le même hôte** (2026-10-09) ; deux réseaux **non prouvé** ; `online: true` publié alors que le contrat dit « boutons online désactivés » tant que la recette n'est pas faite (`contrat-lancement-web.md:39-45,78-81`) : incohérence à trancher | ⚠️ idem | ❌ | ❌ |
| Signaling | ⚠️ le site passe une URL `wss://` exacte + ticket (`contrat-lancement-web.md:20-32`) ; le flux HTTP Allumette est natif seulement (`jjrs/mod.rs:9-12`) ; `allumette.estran.studio` « ne marche pas » en ligne de commande native (`docs/jouer.md:36-62`) mais le parcours **navigateur** a tourné en production le 2026-10-09 ; état actuel non vérifié | idem | idem | idem |
| Capacité du salon | ⚠️ le site ne propose que 2 joueurs au départ (`contrat-lancement-web.md`, § intégration : « Le site ne propose initialement que deux joueurs ») | idem | idem | idem (4 joueurs prouvés en natif seulement, `m3-salles-murs-isaac.md:164-169`) |
| Gungeon : hub sur le web | — | — | ⚠️ `hub: Some(...)` (`game.ron:47-57`) ; `ALACOD_SKIP_HUB` est une variable d'environnement : équivalent web non trouvé | — |
| FPS en wasm | ❌ aucune mesure (inventaire §13) | ❌ | ❌ | ❌ |
| Threads wasm | ⚠️ aucun `atomics`/`wasm-bindgen-rayon` trouvé (inventaire §13) : la simulation tourne sur le thread principal [INFERENCE] | idem | idem | idem |
| Audio navigateur | ⚠️ plugin Kira ; déblocage au premier clic **non prouvé** (inventaire §13) | idem | idem | idem |
| Limite de taille | ⚠️ le script avertit au-delà de 25 MiB par fichier (limite Cloudflare Pages, `build-web-games.mjs`) ; tailles des quatre assets non mesurées | idem | idem (198 RON, sprites) | idem (349 RON) |
| Déploiement | ⚠️ `release.yaml` : WASM → Cloudflare Pages sur tag `v*`, runner auto-hébergé ; construit **zombies et throne** seulement (`Makefile:379-380`) | idem | ❌ | ❌ |

### Axe 2 — Contrôles

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Clavier/souris | ⚠️ `docs/jouer.md:71-94` ; essai E4 de William **non fait** (`dette-d69-e4-personnage.md` §8-9) | ⚠️ idem | ⚠️ idem | ⚠️ idem |
| Manette : tir | ❌ **aucune gâchette ni bouton pour le tir** : `PointerClick` = clic gauche seul (`control.rs:102`) ; la visée est le stick **gauche** (`control.rs:107`, `input_devices.rs:43-50`) alors que le déplacement est au D-pad | ❌ idem | ❌ idem | ❌ idem |
| Manette : actions | ❌ **aucune liaison** pour Sprint, Dash, SwitchWeapon, SwitchWeaponMode, DropWeapon, UseActive, Blank, Bomb, ChoiceA/B/C, Modifier (`control.rs:71-88`, inventaire §2) ; présentes : D-pad (déplacement), Nord interaction, Ouest recharge, Est mêlée, D-pad G/D `ChoicePrev/Next`, Sud `ChoiceConfirm` | ❌ idem | ❌ idem (actif, blank) | ❌ idem (actif, bombe) |
| Manette : menus | ⚠️ pause, inventaire : Start, Select, D-pad, Sud, Est (`ui/screens/input.rs:53-60`) ; fin de partie et mutation : Sud/D-pad seulement pour la mutation ; **fin de partie non vérifiée à la manette** [INFERENCE : aucune liaison trouvée dans `game_over.rs`] | idem + choix de mutation | idem | idem |
| Manette dans le navigateur | ⚠️ `bevy_gilrs`/`gilrs` est dans `Cargo.lock:1775-1802` et dans les features par défaut de `bevy` 0.19.1 (`Cargo.toml` de bevy, liste `default`), que `crates/game/Cargo.toml:8` (`bevy = "0.19.1"`, défauts actifs) fait unifier même si `games/*/Cargo.toml:8-10` les désactivent [INFERENCE : à confirmer par `cargo tree -e features --target wasm32-unknown-unknown`] ; **activation wasm et API Gamepad du navigateur non prouvées** (inventaire §13) | ❌ | ❌ | ❌ |
| Remappage | ❌ non trouvé | ❌ | ❌ | ❌ |
| Écran d'aide | ❌ non trouvé | ❌ | ⚠️ `legend` de `hud.ron` (silhouettes) | ❌ |

### Axe 3 — Lisibilité (couleur par joueur, légende, télégraphes)

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Statut des sprites | ⚠️ manifestes historiques, pas de ligne `statut: placeholder` (inventaire §3) | ❌ placeholders copiés du testbed (`games/throne/assets/game.ron:1-4`) | ⚠️ silhouettes CC0 générées sur placeholders (`games/gungeon/assets/assets.yaml:1-9`) | ⚠️ copie de gungeon ; personnages, ennemis et boss = tuiles Kenney (`m3-salles-murs-isaac.md:34-35`) |
| `wall_art` (hud.ron) | ❌ non trouvé | ❌ non trouvé | ✅ présent (`m2-salles-murs-gungeon.md`) | ✅ présent (121 fichiers générés, `m3-salles-murs-isaac.md:10-14`) |
| Couleur par joueur (`player_colors`) | ❌ `hud.ron` : 0 occurrence (grep du 2026-10-21) | ❌ 0 | ✅ 1 | ❌ 0 (D123) |
| Légende F7 / halo ramassables | ❌ 0 | ❌ 0 | ✅ 3 | ❌ 0 (D123) |
| Dettes visuelles | — | D66 (ennemis immobiles, orientation figée) et D67 (télégraphe pris pour une zone) **ouvertes** (`dettes.md:74-75`) ; fiche `lisibilite-throne-d66-d67.md` | D125 sans objet ici | D123 (sol uni, pas de pointeur de visée) et D125 (cadrage de caméra, mur fissuré non vus avec rendu) **ouvertes** (`dettes.md:133,135`) |
| Revue vidéo de William | ⚠️ `session-test-manuelle.md` : bloc à jouer, non fait (inventaire §8) | ⚠️ idem | ⚠️ captures M2 revues en partie | ⚠️ `m3-salles-murs-isaac.md` §7 : captures à regarder (inventaire §8) |

### Axe 4 — Équilibrage

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Campagne chiffrée | ❌ aucune campagne `alacod-sim` chiffrée trouvée (inventaire §4) ; contenu : 4 armes, 6 configs, 1 boss, 6 salles (inventaire §4) | ❌ idem ; 13 armes, 14 personnages, 1 boss | ✅ 200 graines `etages_deux` : 27 % de morts à 1 bot, 3,5 % à 2 bots, 0 desync ; 4 soft-locks à 1 bot (`m2-t14d-gabarits.md:91`) ; 21 armes, 20 objets, 1 boss, 15 gabarits | ✅ après salles ×2 : 138 victoires / 60 pertes à 1 bot, 183 / 14 à 2, 18 / 2 à 4 ; 0 desync sur 420 graines (`m3-salles-murs-isaac.md:114-124`) ; 100 objets, 9 boss, 23 gabarits |
| Plaintes de William en attente | ❌ S4 (fusil à pompe « super fort », pistolet inutile, mitraillette imprécise), S7 tirs trop forts, S6 pas de son de vague (`m0-revue-suite-contenu.md:47-58`) | ⚠️ D66/D67 (lisibilité plus qu'équilibre) | ⚠️ D64 accepté à 27 % pour sortir de M2 ; « équilibrage fin des armes et du boss » (`taches.md` §7, ligne 11) | ⚠️ D93 : quatre scénarios gagnent par l'armure (`dettes.md:103`) ; Q14 non tranchée (`m3-t18-integration-sortie.md:119-120`) |
| Soft-locks de bot | — | — | ⚠️ 4 résiduels à 1 bot | ⚠️ D121 (tas de sous), D124 (colosse à 2 bots), D119 (piques ignorées : première cause de chute à 2 et 4 bots, 56 des 210 chutes de d2, `m3-t18-integration-sortie.md` §2), D120 (échange d'actifs en boucle) |
| Essai humain en coop | ❌ non fait | ❌ non fait | ❌ non fait | ❌ non fait (`m3-t18-integration-sortie.md:235-259`) |

### Axe 5 — Son et feedback

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Fichiers son | 11 | 10 | 11 | 11 (inventaire §5) |
| Coup de feu | ⚠️ `shot_pistol/shotgun/smg` déclarés dans `feedback.ron` ; D32 (échantillon de 17,7 s coupé à 250 ms) concernait `machine-gun.ogg` du testbed et de throne (`dettes.md:40`) [INFERENCE : à réécouter, zombies a des échantillons dédiés] | ⚠️ D32 ouverte | ⚠️ lot A/B déclarés (`assets.yaml:68-98`) | ⚠️ mêmes entrées |
| Volumes par son | ✅ `volumes` dans `feedback.ron` (−12 dB sur la plupart, `games/zombies/assets/ui/feedback.ron:38`) : S7 **à rejuger à l'oreille** | ⚠️ non lu | ⚠️ non lu | ⚠️ non lu |
| Son de vague/salle | ❌ S6 en file | ❌ | ❌ | ❌ [INFERENCE : aucune source] |
| Musique | ❌ aucune (inventaire §5) | ❌ | ❌ | ❌ |
| Réglage de volume global dans le jeu | ❌ non trouvé [INFERENCE] | ❌ | ❌ | ❌ |

### Axe 6 — Menus et écrans

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Démarrage | ✅ **décision de William : pas d'écran dans le jeu**, le site choisit solo/multi (paramètres d'URL, salon) | idem | idem | idem |
| Pause / inventaire | ✅ `ui/screens` (Échap/Start, I/Select) (inventaire §6) | ✅ | ✅ | ✅ |
| Fin de partie | ✅ `ui/game_over.rs` « Rejouer (R) » / « Lobby » | ✅ | ✅ | ✅ |
| Hub | ❌ (mode `Waves`, `game.ron:40-53`) | ❌ (`Floors`, `game.ron:29-36`) | ✅ `hub: Some(...)` | ❌ (`game.ron:38-43`) |
| Déconnexion / desync | ⚠️ écran « DESYNC » + lobby « new attempt (2/2) » ; textes mélangés FR/EN (`lobby.rs:124-142`, `game_over.rs:120-125`, `jouer.md:64-69`) | idem | idem | idem |
| Mutation / minicarte | — | ⚠️ D71, D72 ouvertes (`dettes-ui-d71-d72.md`) | — | — |

### Axe 7 — Performance

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Budget `tests/budgets.ron` | 40 fps par défaut ; `four_players_*` 33 fps (`budgets.ron:14-16,36-50`) | 23–29 fps selon scénario | `bench_floor` 18 fps / 74 ms ; D118 : 45,9 → 34,2 fps, pire rollback 25,0 → 36,9 ms après agrandissement | `bench_floor_isaac` 26 fps / 40 ms ; D122 : ≈ 28 fps contre ≈ 53 sous charge comparable ; **`isaac_duo` et `isaac_quad` sans plancher ni plafond** |
| Dernière mesure disponible (headless VPS, `latest.json` du commit `0d2c1b61`, 2026-10-20, inventaire §7) | ✅ `four_players_idle` 70,5 fps / 13,5 ms, `four_players_shooting` 71,3 fps / 13,2 ms ; `bench_horde` 42,0 fps / 37,8 ms | ✅ `throne_quad` 44,5 fps / 29,0 ms, `throne_three_floors` 53,3 fps / 23,9 ms | ⚠️ `gungeon_full_run_four` 27,2 fps / 31,7 ms (plancher 15) ; `bench_floor` 33,2 fps / 37,2 ms (plancher 18) | ❌ `bench_floor_isaac` 22,7 fps **sous son plancher de 26** (D122) ; `isaac_duo` 20,5 fps / 34,4 ms ; **`isaac_quad` 10,1 fps / pire rollback 50,4 ms (3 ticks)** ; tout le reste sous budget. Poste partagé, pas le Mac : non mesuré sur la cible |
| Dettes | D70 : champ de flux 9,1 ms/frame à 4 joueurs qui bougent | — | D118 | D122 (champ de flux ×4 cases), D97 (larmes empilées : 56 ms au pire rollback, 4 joueurs × 10 objets) |


### Axe 8 — Game feel

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Mouvement | ✅ accélération/décélération en fixed-point à 60 Hz (`movement.rs:147-176`) ; William : course et dash « marchent assez bien » (2026-10-09, `revue-m0.md:31-33`) ; **aucune donnée `max_speed`/dash dans ses configs** (inventaire §9 : « non trouvé ») [INFERENCE : valeurs par défaut du code] | ⚠️ `max_speed` 0–150, dash 0–64 px / 0–8 frames / cooldown 0–24 / i-frames 6 (un RON) | ⚠️ mêmes valeurs que throne | ⚠️ `max_speed` 0–120, **`dash_distance` 0 partout** : pas de dash [INFERENCE : voulu pour isaac, à confirmer par William] |
| Recul, bousculade | ⚠️ knockback amorti à 0,85/frame et `Shove` du dash (`movement.rs:179-205`) ; **aucun recul de tir** | idem | idem | idem |
| Flash, secousse, hit-stop, chiffres, télégraphe | ✅ en données, `ui/feedback.ron` : flash 4 frames, secousse 8/12, hit-stop 2, chiffres (`feedback.rs:1-13`) | ✅ mêmes valeurs + `by_weapon` | ✅ mêmes valeurs + `pattern_telegraph`, `blank`, `boss_end` | ✅ idem gungeon |
| Réglage propre au clone | ❌ **mêmes valeurs dans les quatre jeux** : aucun réglage de feel propre à un clone | ❌ | ❌ | ❌ |
| Flash de bouche, traînée de balle, douilles, particules d'impact | ❌ non trouvés (`muzzle|trail|impact|casing` : aucun résultat dans `crates/game/src`) | ❌ | ❌ | ❌ |
| Disparition à la mort | ⚠️ `Death` et visuels dérivés ; effet de mort propre non identifié (`feedback.rs:288-295`) | ⚠️ idem | ⚠️ idem | ⚠️ idem |
| Caméra | ⚠️ `lerp_speed 12`, marge 0,05, zoom min 0,6 (`camera.ron:1-13`) | ⚠️ idem + fondu et recentrage d'étage | ⚠️ zoom min 0,5, fondu | ⚠️ `room_frame: true` ; D125 : jamais vu avec rendu |
| Fluidité du rendu | ⚠️ `FixedTransform3D` recopié vers `Transform` après chaque frame GGRS, **sans interpolation** (`core_sim.rs:193-198`) : saccades possibles si l'écran n'est pas à 60 Hz [INFERENCE] ; non mesuré | idem | idem | idem |
| Animations | ⚠️ 16 fichiers d'animation (`sprites/**/*.ron`) ; **décompte idle/marche/attaque/mort par entité non trouvé** | ⚠️ 14 | ⚠️ 91 | ⚠️ 157 |

### Axe 9 — Visuel complet

| Classe d'entité | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Joueur, ennemis, boss | ⚠️ registre du jeu, statut propre/emprunté non trouvé (inventaire §10) | ❌ empruntés au testbed (`game.ron:1-4`) | ⚠️ silhouettes générées CC0 sur placeholders (`assets.yaml:1-12`) | ❌ tuiles Kenney de `sprites/gungeon/` (`assets.yaml:1-6`, `m3-salles-murs-isaac.md:34-35`) |
| Projectiles, objets, props | ⚠️ statut non trouvé | ⚠️ statut non trouvé | ⚠️ générés/provisoires, détail par classe non trouvé | ⚠️ objets générés ; projectiles/props : détail non trouvé |
| Portes, murs (`wall_art`) | ❌ `wall_art` absent | ❌ absent | ✅ présent | ✅ présent (121 fichiers générés) |
| Sol et tilesets | ❌ SunnyLand/Dungeon partagés (inventaire §10) | ❌ idem | ❌ idem (hors `wall_art`) | ⚠️ sol d'une couleur unie `#7B6A5F`, D123 |
| Portail, HUD | ⚠️ sprite de portail propre non trouvé | ⚠️ idem | ✅ hub et `wall_art` | ⚠️ portail propre non trouvé |
| Animations idle/marche/attaque/mort | audit à faire (comptes d'animation par entité non trouvés) | audit à faire | audit à faire | audit à faire |
| Comptage propre/emprunté/absent par classe | ❌ **non trouvé pour les quatre** : les manifestes déclarent sources et licences, pas un statut par entité (inventaire §10) → **le premier livrable de chaque tâche visuelle est ce tableau chiffré** | | | |

### Axe 10 — Effets sonores complets

| Événement | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Tir, recharge | ⚠️ `shot_pistol/shotgun/smg`, `reload` : **trois familles**, pas un son par arme (`audio_config` peut remplacer, `feedback.ron:18-21`) | ⚠️ idem (13 armes) | ⚠️ idem (21 armes) | ⚠️ idem |
| Impact, dégâts reçus, mêlée, dash, porte | ✅ clés `hit`, `hurt`, `melee`, `dash`, `door` | ✅ | ✅ | ✅ |
| Mort | ⚠️ une seule clé `death` pour joueur et ennemis (`feedback.rs:203-212`) : pas de son par famille | ⚠️ idem | ⚠️ idem | ⚠️ idem |
| Blank | — | — | ✅ `blank` | ✅ `blank` |
| Vague, salle, étage | ❌ aucun son (S6 en file pour zombies) | ❌ | ❌ | ❌ |
| Boss | ❌ aucun son dédié | ❌ | ❌ | ❌ |
| Achat, ramassage | ❌ | ❌ | ❌ | ❌ |
| Explosion, bombe | ❌ (inventaire §11) | ❌ | ❌ | ❌ |
| Menus (navigation, confirmation) | ❌ | ❌ | ❌ | ❌ |
| Fichiers | 11 (mêmes noms communs que les trois autres : `hit.ogg`, `death.ogg`, `shot_*.ogg`, `reload.ogg`, `dash.ogg`, `door.ogg`) ; **identiques entre jeux ? MD5 non comparé** (inventaire §11) | 10 | 11 | 11 |
| Volume | ⚠️ par clé en dB (`volumes`, −12 sur la plupart) ; **pas de volume global** | idem | idem | idem |

### Axe 11 — Map design

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Cartes | 2 LDtk, 25 niveaux : `avant_poste` (9, `Depart` ; les cinq spawners étaient **tous** dans `Depart`, `revue-m0.md:38-48`) et `la_fonderie` (16 sections, boucles) | cavernes : 5 `caves`, `floors/run.ron` ; **nombre de salles par étage non trouvé** | 20 niveaux : 15 gabarits `salles`, 3 `etage_1`, hub, armes | 24 niveaux : 23 gabarits `salles` de 480×288 à 960×576, armes |
| Grammaires | 1 `floor_grammars`, 6 `rooms` | 1 `floors/run.ron` | 7 `floor_grammars`, 9 `rooms` | 6 `floor_grammars`, 9 `rooms` |
| Variété, couvertures | ⚠️ spawners dans d'autres salles que `Depart` : constat « les 5 `ZombieSpawn` sont tous dans `Depart` » ; ajouter spawners et fenêtres aux autres salles (`m0-revue-suite-contenu.md:60-65`, en file) ; couloirs problématiques signalés, **décompte non trouvé** | ❌ audit à faire | ⚠️ salles de combat devenues vides après agrandissement (D64, D118) ; tables/barils/rochers : **nombre par gabarit non trouvé** | ⚠️ props (rochers, tables, barils) : **comptes par étage non trouvés** ; D123 (sol), D125 (cadrage de salle, mur fissuré non vus avec rendu) |
| Rythme / spawners | ❌ audit à faire | ❌ audit à faire | ⚠️ 27 % de morts à 1 bot, dont la plupart dans les salles `combat` (D64) | ⚠️ D121/D124 : soft-locks aux sous et au `colosse` ; piques = première cause de chute à 2 et 4 bots (D119) |

**Lecture** : le moteur est solide (0 desync sur 420 graines isaac ; x86-64/arm64 identiques, D73 fausse alerte,
`etat-engine.md:105`), la sim est déterministe et les effets de base (flash, shake, hit-stop) existent. Ce qui manque est
**autour** : la manette (on ne peut pas tirer), le web pour gungeon et isaac, le son hors tir/dégâts, des visuels et
des cartes propres à chaque jeu, un feel réglé par jeu. Et trois **inconnues à mesurer d'abord** : la manette en wasm,
les FPS en wasm, les tailles de build.

## 3. Tâches

**B** = bloque « jouer entre amis, dans le navigateur, à la manette, sur des jeux propres ». **P** = polissage classé
valeur/coût. Estimation en jours-agent. **Capacité : 4 voies × 5 jours = 20 j par semaine** (b0, b1, b3 sur Mac ; **une
seule** voie VPS). Le B total **dépasse largement 20 j** (≈ 40 j, § 3.3) : il est découpé en **semaine 1** et
**semaine 2**. Les fiches (`docs/taches/polissage-*.md`) existent pour chaque B de la semaine 1 ; celles de la semaine 2
sont écrites à la fin de la semaine 1 avec ce qu'elle aura appris (audits).

**Choix de découpe** : la semaine 1 met en place ce qui vaut **pour les quatre clones à la fois** (web + online, manette,
textes français, mesure wasm, un premier lot de game feel et de sons **dans le moteur**, en données) et rend **un clone
complet** : `zombies` (le plus petit contenu : 4 armes, 6 configs, 1 boss, 6 salles, inventaire §4 ; déjà dans le
catalogue ; plaintes de William en file S4/S6/S7 et spawners). Semaine 2 : `gungeon`, puis `isaac`, puis `throne`
(décision 4 à confirmer). Faire « quatre clones à moitié » est écarté : un clone terminé sert de démo.

### 3.1 Semaine 1 — B, total **19,5 j**

| # | B/P | Tâche (fiche) | Clone(s) | Ce que ça change pour le joueur | Est. | Dépend de | Preuve attendue |
|---|---|---|---|---|---|---|---|
| S1-1 | B | **Manette** jouable dans le navigateur, menus compris, écran d'aide (`polissage-manette.md`) — **priorité 1** | 4 | On peut tirer, viser, dasher, utiliser un actif, naviguer pause/fin de partie/mutation à la manette dans Chrome | 3 | schéma validé par William (décision 2), `bevy_gilrs` wasm | partie de zombies et d'une autre jouée entièrement à la manette dans le navigateur ; scénarios et traces inchangés ; essai E4 |
| S1-2 | B | **Web : les 4 clones dans le catalogue du site** (`polissage-web-4-clones.md`) | 4 | gungeon et isaac jouables dans le navigateur comme zombies et throne | 3 | rien | `build-web-games.mjs` construit 4 jeux ; `releases.json` et `catalog.ts` à 4 entrées ; chaque clone atteint la première salle sous Chrome ; tailles et temps de chargement mesurés |
| S1-3 | B | **Online navigateur** : signaling `wss://` joignable, deux réseaux, `online` activé honnêtement, D116 (`polissage-reseau-amis.md`) | 4 | On se retrouve par le site (salon) et on joue à 2 | 2,5 | décision 1 (signaling) | partie à 2 entre deux réseaux distincts, traces identiques ; `online: true` seulement là où la recette est faite |
| S1-4 | B | **Textes en français** : lobby, réseau, DESYNC, fin de partie, pause (`polissage-textes-fr.md`) | 4 | Plus de « Connecting to peer… » à côté de « Rejouer » | 1 | rien | grep des textes anglais listés = 0 ; captures ; aucune trace ne bouge |
| S1-5 | B | **Perf wasm** : FPS d'affichage et de simulation par clone dans le navigateur (`polissage-perf-web.md`) | 4 | Savoir si la partie tourne à 60 Hz dans un onglet ; seuils pour la suite | 1 | S1-2 | tableau FPS par clone (Chrome, Mac de William), tailles de build |
| S1-6 | B | **Game feel, lot commun** : interpolation d'affichage, réglages `movement`/`dash`/`feedback.ron`/`camera.ron` **par clone**, flash de bouche, recul de tir, effet de mort (`polissage-feel.md`) | 4 (réglé sur zombies) | Mouvement et tirs nets, fluides à 144 Hz | 2 | rien | vidéo de prise en main de zombies ; l'interpolation ne touche pas `GgrsSchedule` (traces inchangées) ; réglages en données |
| S1-7 | B | **Sons : événements manquants dans le moteur** (vague, salle, étage, boss, achat, ramassage, explosion, menus, mort par famille, volume global, déblocage audio navigateur) (`polissage-sons-evenements.md`) | 4 | Chaque événement peut avoir un son par clone, réglable | 1,5 | rien | clés `sounds` et lint ; sons dérivés de l'état (jamais d'un événement de simulation) ; traces inchangées |
| S1-8 | B | **Zombies complet : visuel** (audit d'abord ; sol/murs/`wall_art`, joueur et ennemis animés, armes ; couleur par joueur et légende) (`polissage-zombies-visuel.md`) | zombies | Zombies a son propre visage et on s'y retrouve à 4 | 2 | décision 5 (DA) | tableau classe × état avant/après ; captures et vidéo ; traces intactes |
| S1-9 | B | **Zombies complet : sons** (tir par arme, vague S6, volumes S7, mort, achat, ramassage…) (`polissage-zombies-sons.md`) | zombies | Le jeu « sonne » | 1 | S1-7 | tableau événement × son sans case vide ; écoute de William |
| S1-10 | B | **Zombies complet : map et équilibrage** (spawners dans toutes les salles, S4 armes, campagne chiffrée) (`polissage-zombies-map-equilibrage.md`) | zombies | Les vagues viennent de partout, les trois armes ont un rôle | 1,5 | rien | audit de cartes, campagne avant/après, traces rebénies avec preuve §5 |
| S1-11 | B | **Session d'amis** : zombies à 2 puis 4 sur le site, réseau dégradé, déconnexion, restart (`polissage-session-amis.md`) | zombies (puis les autres) | La promesse tenue sur un clone complet | 1 | S1-1 à S1-4 | rapport tableau « fait / non vérifié », 0 DESYNC |
| | | **Total semaine 1** | | | **19,5** | | |

### 3.2 Semaine 2 — B, total **20 j**

| # | B/P | Tâche | Clone(s) | Ce que ça change pour le joueur | Est. | Dépend de | Preuve attendue |
|---|---|---|---|---|---|---|---|
| S2-1 | B | Gungeon complet : visuel (silhouettes générées → sprites propres et animés, 11 personnages, boss, sol) | gungeon | Gungeon a son identité | 3 | S1-8 (méthode), décision 5 | tableau classe × état, vidéo |
| S2-2 | B | Gungeon complet : sons (21 armes → familles, boss, hub, achat, coffre, salles) | gungeon | idem | 1 | S1-7 | tableau événement × son |
| S2-3 | B | Gungeon complet : map et équilibrage (D64 : 27 % de morts à 1 bot, salles vides après agrandissement, armes et boss) | gungeon | Moins mortel, mieux rythmé | 2,5 | rien | campagne 200 graines avant/après, preuve §5 |
| S2-4 | B | Isaac complet : visuel (personnages/ennemis/boss : fin des tuiles Kenney, sol, pointeur de visée ; D123) | isaac | Isaac ne ressemble plus à gungeon | 3,5 | S1-8, décision 5 | tableau classe × état, vidéo |
| S2-5 | B | Isaac complet : sons (bombe, objets, boss ×9, clés, sous) | isaac | idem | 1 | S1-7 | tableau |
| S2-6 | B | Isaac complet : map et équilibrage (D93 armure, Q14, props, D125 cadrage de salle vu avec rendu) | isaac | idem | 3,5 | décision 6 | campagne, captures |
| S2-7 | B | Throne jouable : D66/D67 (tourelle lisible), campagne chiffrée (`polissage-throne-jouable.md`) | throne | On sait ce qu'est une tourelle | 2 | rien | D66/D67 fermées, campagne publiée |
| S2-8 | B | Lisibilité coop : couleur par joueur, légende, halo pour throne et isaac (`polissage-lisibilite-coop.md`) | throne, isaac | À 4 on se reconnaît | 1,5 | S1-8 | captures à 4 |
| S2-9 | B | Perf : budgets du navigateur et du natif (`isaac_quad` 10,1 fps / 50 ms au pire rollback sur le VPS, `bench_floor_isaac` sous son plancher), D70 | 4 | Pas d'à-coup à 4 | 2 | S1-5 | mesures navigateur et Mac ; traces identiques si le champ de flux est touché |
| | | **Total semaine 2** | | | **20** | | |

### 3.3 Totaux et ce qui n'entre pas

**B : 39,5 j** (19,5 + 20) : deux semaines pleines à quatre voies. Rester dans ce total demande de **déclasser en P** :
**throne complet** (visuel 3 j, sons 1 j, map 2 j = 6 j) ; throne n'a, en semaine 2, que la lisibilité et la campagne. Si William
veut quatre clones complets, il faut une troisième semaine (+ 6 j) ou deux voies de plus. Toutes les estimations sont des
`[INFERENCE]` : les audits (visuel, animations, cartes, tailles wasm) peuvent les changer ; chaque tâche commence par
son audit chiffré et **redit son estimation à la fin du premier jour**.

### 3.4 Polissage (P), classé valeur / coût — total **21,5 j**

| # | B/P | Tâche | Clone(s) | Ce que ça change pour le joueur | Est. | Dépend de | Preuve attendue |
|---|---|---|---|---|---|---|---|
| P1 | P | Throne complet : visuel, sons, map (cavernes, nombre de salles par étage à auditer) | throne | Throne a son identité | 6 | S1-8, S1-7 | tableaux classe × état |
| P2 | P | Bots : D119 (piques), D120, D121, D124 | isaac, `crates/bots` | Campagnes fiables | 2,5 | rien | campagne sans soft-lock (graines 63, 178) |
| P3 | P | Écran de mutation `bevy_hui` et minicarte (D71, D72), à la manette | throne | Choix lisible | 1,5 | S1-1 | `dettes-ui-d71-d72.md` |
| P4 | P | Caméra par salle d'isaac et mur fissuré vus avec rendu (D125) | isaac | Pas de glissement | 1 | rendu Mac | captures |
| P5 | P | Remappage des touches et boutons | 4 | Chacun règle sa manette | 2 | S1-1 | écran de réglage, fichier de config |
| P6 | P | Salon à 4 joueurs sur le site (le site propose 2) | 4 | Jouer à 4 | 2 | S1-3 | partie à 4 entre réseaux distincts |
| P7 | P | Perf : champ de flux incrémental (D70, D122, D118) | 4 | Marge | 3 | S2-9 | traces identiques (preuve §5) |
| P8 | P | Feel, lot 2 : traînées de balles, douilles, avance de caméra | 4 | Plus de punch | 2 | S1-6 | vidéo |
| P9 | P | Musique d'ambiance courte par clone (licences) | 4 | Ambiance | 1,5 | décision 7 | licences |
| | | **Total P** | | | **21,5** | | |

Ordre valeur/coût : P2, P4, P3, P6, P5, P1, P8, P9, P7. (P1 est le plus gros et le plus cher mais « 4 clones complets » le
demande ; William tranche, décision 4.)

## 4. Semaine 1 : calendrier à quatre voies

Quatre voies : **b0, b1, b3 sur Mac** (affichage, navigateur, Docker, manette physique) et **une seule voie VPS, b2**
(ni affichage ni navigateur : **aucune capture ni vidéo** — D94 ; contenu, bots, campagnes, p2p headless, hébergement du
signaling). La semaine 2 reprend la même grille avec le tableau § 3.2 (b0 : S2-4 isaac visuel ; b1 : S2-1 et S2-2 gungeon ;
b3 : S2-8, S2-9 et S2-5 ; b2 : S2-3, S2-6, S2-7 [INFERENCE : campagnes et contenu sur le VPS, vidéos de recette sur Mac]).

| Jour | b0 (Mac) | b1 (Mac) | b3 (Mac) | b2 (VPS) |
|---|---|---|---|---|
| 1 | S1-1 : **proposition de schéma manette à William** (matin), vérification `bevy_gilrs` en wasm, liaisons | S1-2 : script 4 jeux, compilation wasm de gungeon et isaac | S1-6 : interpolation d'affichage, réglages par clone | S1-3 : état réel de `allumette.estran.studio` ; serveur `wss://` (décision 1) |
| 2 | S1-1 : tir, visée, actions | S1-2 : tailles, chargement, `catalog.ts` | S1-6 : flash de bouche, recul, mort | S1-3 : D116, test headless à deux |
| 3 | S1-1 : menus, mutation, écran d'aide ; **essai E4 avec William** | S1-2 : `releases.json`, `release.yaml`, gungeon hub web | S1-8 : audit visuel de zombies, tilesets | S1-7 : clés de sons, volume global |
| 4 | S1-4 : textes français | S1-3 : recette navigateur entre **deux réseaux** | S1-8 : sprites animés, couleurs de joueur | S1-7 fin (déblocage audio côté Mac) ; S1-10 : audit des cartes, spawners |
| 5 | S1-11 : session d'amis (avec William) | S1-5 : FPS wasm par clone | S1-9 : sons de zombies | S1-10 : campagne avant/après, traces rebénies |

**Ordre de merge** (du moins au plus conflictuel) : S1-7 → S1-9 → S1-10 → S1-8 → S1-6 → S1-2 → S1-3 → S1-1 → S1-4 → S1-5
→ S1-11.

**Conflits prévisibles** : `website/static/releases.json`, `website/src/lib/game/catalog.ts`, `Makefile`,
`.github/workflows/release.yaml` (S1-2 et S1-3) ; `crates/game/src/ui/screens/input.rs`, `character/control.rs`,
`input_devices.rs` (S1-1) et `ui/game_over.rs`, `ui/lobby.rs` (S1-4 et S1-1) ; `crates/game/src/feedback.rs` et
`crates/content/src/feedback.rs` (S1-6 et S1-7) ; `core_sim.rs` (interpolation, S1-6) ; `games/zombies/assets/ui/hud.ron`
et `feedback.ron` (S1-6, S1-8, S1-9) ; `tests/scenarios/*.trace` de zombies (S1-10 seul peut les bouger ; les autres
tâches n'en bougent aucune) ; `docs/taches.md` et `dettes.md` (une ligne de journal par lot, rebase).

**Ce qui attend William** : jour 0 : décisions 1 à 5 de § 5 (sans réponse, défauts) ; jour 1 : **validation du schéma
manette** (S1-1 écrit le code après) ; jour 3 : essai E4 à la manette dans le navigateur ; jour 4 : un **second réseau**
(le Mac de recette passait par un tunnel qui échouait même sans engine : `contrat-lancement-web.md`, preuve sur deux réseaux
ouverte) ; jour 5 : session d'amis, écoute des sons de zombies, revue de la vidéo de zombies ; fin de semaine 2 : 30 min en
coop par clone.

## 5. Décisions attendues de William

Décisions **déjà prises** (2026-10-21, retirées de la liste) : jeux **web seulement** via le site (plus d'archives
Mac/Windows, plus de question de plateforme, de signature Gatekeeper ni d'hébergement d'archives) ; **pas d'écran de
démarrage dans le jeu** ; **manette prioritaire**. Restent, avec l'option par défaut :

1. **Signaling** : héberger `matchbox_server` nu derrière TLS sur le VPS, ou **réparer/relancer Allumette** ? Défaut :
   Allumette, parce que le site passe déjà par lui (invités, salons, TURN : parcours prouvé sur deux Chromium le 2026-10-09,
   `contrat-lancement-web.md`) ; un `matchbox_server` nu obligerait à réécrire le flux du site et n'apporte pas de TURN
   [INFERENCE]. Dans les deux cas : `wss://`, joignable d'un navigateur ; version du protocole Matchbox à vérifier
   (client 0.15 d'un fork, serveur Allumette 0.13.0, `contrat-lancement-web.md`).
2. **Schéma manette** (proposition à valider, table dans `polissage-manette.md`) : stick gauche déplacement, stick droit
   visée, gâchette droite tir, etc. Défaut : la proposition de la fiche.
3. **Perf web acceptable** : seuil à fixer après la mesure de S1-5. Défaut de travail : la simulation tient 60 Hz à
   2 joueurs dans Chrome sur le Mac de William ; sinon, dégrader (lumière désactivée pour le web, déjà le défaut de la
   feature `lighting`, `crates/game/Cargo.toml`) avant de réduire le contenu.
4. **Quel(s) clone(s) rendre complets en premier** : défaut **zombies (semaine 1), puis gungeon, puis isaac (semaine 2),
   throne en P1** (+ 6 j). Alternative : isaac avant gungeon.
5. **Direction artistique** : garder la génération CC0 de la voie V6 (`plan-assets.md`) pour les sprites « propres au jeu »,
   ou acheter/commander ? Défaut : générer cette semaine, décider un achat avant S2-1 si le rendu de zombies déçoit.
6. **Q14** : la couverture des mécaniques remplace-t-elle la fourchette de victoires ? Défaut : oui.
7. **Sons et musique** : sons CC0/CC BY nouveaux autorisés (licence dans `assets.yaml`) ; musique en P9. Défaut : sons oui,
   musique non.
8. **Gungeon sur le web** : commence-t-on au hub (`hub: Some(...)`) ou directement en run ? Défaut : au hub, comme natif.
9. **`online` dans `releases.json`** : le fichier annonce `online: true` pour zombies et throne alors que le contrat dit
   « désactivé tant que la recette n'est pas faite » : repasser à `false` jusqu'à S1-3 ? Défaut : oui.
10. **Isaac sans dash** (`dash_distance` 0 partout, inventaire §9) : voulu ? Défaut : oui, isaac n'a pas de dash.
11. **Essai humain** : 30 min en coop par clone à la fin de chaque semaine, notes dans
    `docs/taches/rapports/plan-polissage-notes.md`. Défaut : oui.
12. **Les P** : lesquels engager si une voie finit tôt. Défaut : l'ordre de § 3.4.

## 6. Hors périmètre

- **Archives natives Mac/Windows** et workflow `dist` : supprimés (décision du 2026-10-21).
- **Écran de démarrage, menu solo/héberger/rejoindre dans le jeu** : le site connecte (décision du 2026-10-21).
- **Mécaniques nouvelles** (objet, ennemi, arme, mode, hub pour un clone qui n'en a pas) ; **suite de D69** (E8+) ; **1837**
  et M4 ; **équilibrage automatique** par fourchette de victoires.
- **Remappage, salon à 4 sur le site, musique, feel lot 2, perf profonde** : P, jamais B.
- **Dettes d'outillage sans effet sur le joueur** (D110, D111, D112, D114, D115, D117).

## 7. Règles de la semaine

- Déterminisme et checklist GGRS de `CLAUDE.md` : l'**interpolation d'affichage** (S1-6) vit en `PostUpdate`/présentation,
  jamais dans `GgrsSchedule` et jamais dans le hash ; les sons et effets sont **dérivés de l'état** (`Update`), jamais d'un
  événement émis par la simulation ; tout réglage de gameplay (S1-10, S2-3, S2-6) passe par `BLESS` avec preuve §5.
- Présentation seule (S1-1, S1-4, S1-6, S1-7, S1-8, S1-9) : `make test_scenarios` sans `BLESS`, aucune trace ne bouge.
- Numéros de dettes : prochain libre **D126** ; **D126 à D130 réservés** à ce plan (audits : manette wasm, interpolation,
  tailles wasm, sons manquants, cartes) ; les workers ouvrent à partir de **D131** après avoir relu `dettes.md`.
- Chaque lot finit par `LIVRÉ <branche>` dans le journal (`docs/taches.md` §10) et un rapport dans
  `docs/taches/rapports/`.
