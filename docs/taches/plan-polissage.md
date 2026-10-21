# Plan de la semaine de polissage des quatre clones

Rédigé le 2026-10-21 (branche `plan-polissage`, base `origin/main`). **Décision de William des 2026-10-19 à
2026-10-21** : pause du développement de mécaniques ; la semaine qui vient (≈ 5 jours, 4 à 5 workers, ≈ 20
jours-agent) sert à polir `zombies`, `throne`, `gungeon` et `isaac`. Elle s'intercale **entre M3 et M4**
(`docs/taches.md` §8). L'inventaire factuel qui fonde les tableaux est
`docs/taches/rapports/plan-polissage.inventaire.md` (noté « inventaire » ci-dessous, avec sa section).

Convention : chaque fait cite sa source ; `[INFERENCE]` = déduit, non observé. Aucun chiffre n'est inventé : une
cellule sans mesure dit « non mesuré ».

## 1. Objectif

> « 4 jeux équilibrés, fun, fluides et complets ; pas raffinés au niveau d'un jeu release, mais quelque chose de
> propre » — pour que des amis y jouent ensemble en ligne, et pour servir de démo de vente de l'engine (William,
> 2026-10-19/21). Ensuite viendra 1837 (hors périmètre).

**Définition de « fini » par axe** (critères observables, un par clone sauf mention) :

| Axe | Fini quand… (vérifiable) |
|---|---|
| 1. Lancer en ligne sans terminal | Un ami qui n'a ni Rust ni Docker télécharge **une archive**, double-clique, choisit « Héberger » ou « Rejoindre » + un nom de salon, et la partie démarre ; testé à 2 (puis 4) sur **deux machines réelles** pour **chaque clone** (rapport de session, traces identiques via `ALACOD_STATE_TRACE`, 0 `DESYNC`). Le signaling par défaut du binaire est joignable depuis internet (pas de `docker compose` à lancer). |
| 2. Contrôles | Chaque action utile d'un clone a une touche clavier **et** un bouton de manette, listés dans un écran d'aide du jeu ; William a coché l'essai manuel E4 (`dette-d69-e4-personnage.md` §8) clavier/souris et manette, 1 et 2 joueurs. |
| 3. Lisibilité et visuels | Sur une capture et une vidéo de chaque clone : on distingue **son joueur** de ses coéquipiers (couleur par handle), un ennemi qui tire d'un ennemi qui marche (D66/D67), un ramassable d'un décor ; aucun sprite d'un autre jeu (« zombie » dans un clone qui n'en a pas) sur un ennemi principal ; William valide la page de revue vidéo. Pas d'attente d'un jeu release. |
| 4. Équilibrage | Par clone, un **tableau de campagne chiffré** (bot `explorateur`, 1 et 2 bots, ≥ 100 graines, avec la mesure de la mort par cause) publié dans un rapport, et un essai de William de 30 min en coop qui dit « ni trop dur ni trop facile » ; aucun soft-lock de bot non expliqué. La fourchette n'est pas une boucle automatique (William sceptique) : la campagne **informe**, William **tranche**. |
| 5. Son et feedback | Chaque clone : un coup de feu = un échantillon court (D32 fermée), un son de début/fin de vague ou de salle, un son de dégâts reçus/mort audibles et à un volume non pénible (S7) ; réglage de volume global ; licences dans `assets.yaml`. Pas de musique exigée. |
| 6. Menus et écrans | Écran de démarrage → partie → pause/inventaire → fin de partie → retour au menu, **sans argument de ligne de commande**, tout en français ; déconnexion et `DESYNC` expliqués au joueur avec une action (retour au menu). |
| 7. Performance | Sur le Mac de William (au calme, pas le VPS) : chaque clone tient 60 Hz **sans à-coup** à 2 joueurs et ≥ 40 fps simulées à 4 (`min_sim_fps` de `tests/budgets.ron`), et le pire rollback (`rollback_ms_worst`) reste sous 2 ticks (33 ms) sur les scénarios jouables ; mesures publiées. |

## 2. État mesuré

Verdicts : ✅ fait et prouvé · ⚠️ partiel ou non vérifié sur la cible · ❌ manquant. Source entre parenthèses.

### Axe 1 — Lancer une partie en ligne sans terminal

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Lancement local | ✅ `make zombies` (inventaire §1) | ✅ `make throne` | ✅ `make gungeon`, démarre au hub (`docs/jouer.md:24-27`) | ✅ `make isaac` (`m3-salles-murs-isaac.md:210`) |
| En ligne documenté | ⚠️ `jouer-a-deux.md:39-43` (recette de 2026-10-04, dernier état « deux machines distinctes **non vérifié** » dans ce document ; vérifié depuis par `m2-p2p-deux-machines.md`, inventaire §1, `etat-engine.md:125`) | ❌ pas de cible `throne_matchbox` trouvée (inventaire §1) | ✅ procédure `jouer.md:34-60` ; run complète 2/4 en ligne (`m2-p2p-run-complete.md`) | ⚠️ essais à 2 et 4 sur **une** machine ; deux machines pas vérifiées sur le contenu actuel (`m3-salles-murs-isaac.md:164-177`) |
| Signaling public par défaut | ❌ `wss://allumette.estran.studio` (`Makefile:22`) : « ne marche pas » en ligne de commande, `--allumette` exige un salon créé par le site (`docs/jouer.md:36-62`) | ❌ idem | ❌ idem | ❌ idem ; repli = `docker compose -f docker-compose.ci.yaml up -d signaling` sur une machine |
| Traversée NAT | ⚠️ STUN Google en dur, pas de TURN (`jouer-a-deux.md:5-6`) ; NAT résidentiel sans Tailscale **non vérifié** (ibid.) | ⚠️ idem | ⚠️ idem | ⚠️ idem |
| Distribution binaire | ❌ aucune archive Mac/Windows : `.github/workflows/release.yaml` ne construit que le WASM puis Cloudflare Pages (runner auto-hébergé) | ❌ | ❌ | ❌ |
| Build web | ⚠️ rapporté pour zombies (`dette-d69-e4-personnage.md:78`) ; contrat : `/play?id=zombies\|throne`, boutons online **désactivés** (`contrat-lancement-web.md:6-14,41-45`) | ⚠️ idem | ❌ hors du catalogue web (contrat : zombies et throne seulement) | ❌ idem |
| Lobby | ⚠️ `ui/lobby.rs` : liste des joueurs et état de connexion ; nombre de joueurs et salon viennent des arguments, aucun choix en jeu (inventaire §6) | idem | idem | idem |
| Pairs en trop / attente | ⚠️ D116 (attente non bornée) : ouverte en tant que cas non couvert (`dettes.md:126`) | idem | idem | idem |

### Axe 2 — Contrôles

| Point | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| Clavier/souris | ⚠️ table `docs/jouer.md:71-94` ; essai E4 de William **non fait** (`dette-d69-e4-personnage.md` §8-9) | ⚠️ idem (même `get_input_map`, `control.rs:58-108`) | ⚠️ idem | ⚠️ idem |
| Manette | ❌ **on ne peut pas tirer** : `PointerClick` (le tir) = clic gauche seul, aucune gâchette ni bouton (`control.rs:102`) ; la visée est le stick gauche (`input_devices.rs:43-50`) alors que le déplacement est au D-pad ; **aucune liaison** pour Sprint, Dash, SwitchWeapon/Mode, DropWeapon, UseActive, Blank, Bomb, ChoiceA/B/C, Modifier (inventaire §2, `control.rs:71-88`) ; jamais essayée | ❌ idem | ❌ idem | ❌ idem (actif, bombe, choix) |
| Remappage | ❌ non trouvé (inventaire §2) | ❌ | ❌ | ❌ |
| Écran d'aide des touches | ❌ non trouvé [INFERENCE : aucune source ne le cite hors `docs/jouer.md`] ; F7 = légende des silhouettes seulement | ❌ | ⚠️ `legend` dans `hud.ron` (3 occurrences) | ❌ |

### Axe 3 — Lisibilité et visuels

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
| Démarrage | ❌ ligne de commande (inventaire §1) | ❌ | ❌ (hub oui, mais après le lancement) | ❌ |
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

**Lecture** : le moteur est solide (0 desync sur 420 graines isaac ; deux machines x86-64/arm64 identiques, D73 fausse
alerte, `etat-engine.md:105`) ; ce qui manque est **autour** : atteindre une partie sans terminal, savoir qui est qui à
quatre, des armes réglées, des sons, des mesures sur un vrai poste.

## 3. Tâches

**B** = bloque « jouer entre amis ». **P** = polissage. Estimations en jours-agent. « Preuve attendue » = ce qui ferme la
tâche. Fiches des B : `docs/taches/polissage-*.md`. Les tâches P n'ont pas de fiche : elles sont écrites si William les
retient (décision 12).

### 3.1 Bloquantes (B) — total **20 j**

| # | B/P | Tâche (fiche) | Clone(s) | Ce que ça change pour le joueur | Est. | Dépend de | Preuve attendue |
|---|---|---|---|---|---|---|---|
| B1 | B | Signaling joignable par défaut + salon par nom (`polissage-reseau-amis.md`) | 4 | On se retrouve entre amis par un nom de salon, sans lancer Docker ni connaître d'IP | 2 | décisions 1, 11 | partie à 2 sur deux machines réelles via le serveur par défaut, traces identiques ; D116 refermée |
| B2 | B | Archives Mac (et Windows) téléchargeables et jouables (`polissage-distribution.md`) | 4 | Un ami télécharge un zip, double-clique, joue : plus de Rust ni de `make` | 2 | décisions 2, 9, 10 | archive construite en CI, lancée sur une machine **sans** toolchain, un clone par archive |
| B3 | B | Écran de démarrage et textes en français (`polissage-demarrage.md`) | 4 | Menu : solo / héberger / rejoindre + salon, nombre de joueurs, nom ; messages réseau, DESYNC, fin de partie en français | 3 | B1 (interface réseau), décisions 3, 8 | de l'archive B2 au premier niveau sans argument CLI, pour chaque clone ; capture de chaque écran |
| B4 | B | Manette jouable (tir, visée, toutes les actions) et écran d'aide des touches (`polissage-manette.md`) | 4 | Aujourd'hui **on ne peut pas tirer à la manette** ; toute action utile a un bouton ; on lit les touches dans le jeu | 3 | rien | essai E4 coché par William (clavier + manette, 1 et 2 joueurs) ; scénarios inchangés |
| B5 | B | Couleur par joueur et lisibilité coop (`polissage-lisibilite-coop.md`) | zombies, throne, isaac (gungeon : contrôle) | À 2–4 joueurs, on sait quel héros est le sien ; halo des ramassables ; légende F7 | 2 | rien | captures/vidéos 2 et 4 joueurs par clone, `hud.ron` avec `player_colors` et `legend` ; traces intactes |
| B6 | B | Zombies : armes, son de vague et volumes (`polissage-zombies-equilibrage-son.md`) | zombies | Fusil à pompe, pistolet et mitraillette utilisables ; on entend les vagues ; les tirs ne fatiguent pas | 2 | rien (campagne b2) | tableau de campagne `alacod-sim` avant/après ; traces rebénies avec preuve §5 ; écoute de William |
| B7 | B | Throne jouable : D66/D67 + campagne chiffrée (`polissage-throne-jouable.md`) | throne | Une tourelle ressemble à une tourelle et son tir se lit ; on sait si le jeu est trop dur | 2 | rien | vidéo des tourelles/boss, campagne 1 et 2 bots publiée ; D66/D67 fermées |
| B8 | B | Perf sur le Mac de William : mesure, plafonds, D70 (`polissage-perf-machine.md`) | 4 | Pas d'à-coup à 2 et surtout à 4 joueurs (`isaac_quad` : 10 fps sur le VPS) sur un portable | 2 | B5 (scènes à 4) conseillé | `make bench` strict au calme sur Mac, tableau par clone ; trace identique si le champ de flux est touché |
| B9 | B | Session d'amis : validation en ligne de bout en bout (`polissage-session-amis.md`) | 4 | La promesse tenue : 4 jeux, 2 puis 4 joueurs, déconnexion et retour au menu propres | 2 | B1 à B3 | rapport de session, un tableau par clone, 0 DESYNC, notes de William |
| | | **Total B** | | | **20** | | |

### 3.2 Polissage (P), classé valeur / coût — total **26,5 j** (≈ 1,3 semaine de plus)

| # | B/P | Tâche | Clone(s) | Ce que ça change pour le joueur | Est. | Dépend de | Preuve attendue |
|---|---|---|---|---|---|---|---|
| P1 | P | Gungeon : équilibrage fin des armes et du boss (`taches.md` §7 ligne 11 ; D64) | gungeon | Salles `combat` moins mortelles à 1 joueur (27 % de morts aujourd'hui), boss lisible | 2 | B9 (essai humain) | campagne 200 graines avant/après ; traces rebénies avec preuve §5 |
| P2 | P | Isaac : équilibrage, D93 (armure) et décision Q14 | isaac | Les parties ne se gagnent plus « par l'armure » ; difficulté choisie | 2 | décision 6 | campagne ; D93 fermée |
| P3 | P | Sons : coup unique pour throne/testbed (D32), événements de salle/étage/boss, volume global | 4 | Le jeu « sonne » ; réglage du volume | 3 | décisions 5, 7 | licences dans `assets.yaml`, écoute de William |
| P4 | P | Bots : D119 (piques), D120, D121, D124 | isaac (D119 touche aussi `crates/bots`) | Campagnes fiables pour toute la suite | 2,5 | rien | campagne sans soft-lock (graines 63, 178) |
| P5 | P | Sprites propres des personnages/ennemis/boss isaac (D123) : sol, ombres, pointeur de visée | isaac | Isaac ne ressemble plus à gungeon | 3 | décision 5 | captures avant/après |
| P6 | P | Écran de mutation `bevy_hui` et minicarte (D71, D72) | throne | Choix de mutation lisible, repère dans la carte | 1,5 | rien | `dettes-ui-d71-d72.md` |
| P7 | P | Caméra par salle d'isaac et mur fissuré vus avec rendu (D125) | isaac | Pas de glissement ni de mur secret invisible | 1 | rendu sur Mac (b0/b1/b3) | captures, D125 fermée |
| P8 | P | Remappage des touches | 4 | Chacun règle ses touches | 2 | B4 | écran de réglage, fichier de config |
| P9 | P | Perf : champ de flux incrémental (D70, D122), gungeon D118 | 4 | Marge de manœuvre sur vieux portables | 3 | B8 | traces identiques (preuve §5), `rollback_ms_worst` en baisse |
| P10 | P | Build web et ligue en ligne web pour zombies/throne | zombies, throne | Jouer depuis un lien | 4 | contrat Allumette (`contrat-lancement-web.md`), B1 | `/play?id=…` avec online activé |
| P11 | P | Musique d'ambiance courte par clone | 4 | Ambiance | 1,5 | décisions 5, 7 | licences, volume |
| P12 | P | Revue de lisibilité de gungeon (D66/D67 en vidéo, légende, `player_colors` déjà là) | gungeon | Cohérence avec isaac | 1 | rien | page de revue |
| P13 | P | Hub minimal de retour au menu pour zombies/throne/isaac | 3 | `[INFERENCE]` une salle d'attente avant la partie | — | **non retenue** : mécanique, voir § 6 | — |
| | | **Total P (hors P13)** | | | **26,5** | | |

Ordre de valeur/coût : P1, P2, P7, P6, P12, P4, P3, P5, P8, P9, P11, P10.

## 4. Semaine

Cinq voies. Les voies **Mac** (b0, b1, b3) ont un affichage et Docker ; les voies **VPS** (b2, b6) n'ont ni l'un ni
l'autre : **aucune capture ni vidéo** n'y est produite (D94, `bevy_winit` sans `DISPLAY`) ; elles font contenu, bots,
campagnes, p2p headless, hébergement du signaling. Total : 20 j de B sur 25 j de capacité : 5 j de marge pour les
reprises et les P classés par valeur.

| Jour | b0 (Mac) | b1 (Mac) | b3 (Mac) | b2 (VPS) | b6 (VPS) |
|---|---|---|---|---|---|
| 1 | B1 : signaling par défaut | B2 : binaires et workflow | B5 : couleurs et légendes (captures) | campagnes zombies et throne (entrées de B6/B7), préparation de B1 côté VPS (hébergement si décision 1 = auto-héberger) | B6 : armes de zombies, volumes, son de vague |
| 2 | B1 fin (D116) | B2 : un clone par archive | B5 fin | B7 (campagne 1/2 bots throne) | B6 fin, traces rebénies |
| 3 | B3 : menu de démarrage | B4 : liaisons manette (tir en premier) | B8 : mesures `make bench` au calme sur Mac, dont `isaac_quad`/`isaac_duo` (sans budget aujourd'hui) | B9 : préparation, p2p headless à 4 sur l'archive | B7 : D66/D67 (données, vidéo par b3 ensuite) |
| 4 | B3 : textes FR et écrans réseau | B4 : tir à la manette, bouton par action, essai E4 avec William | B8 : plafonds ; D70 si nécessaire | B9 : stress réseau dégradé (`p2p-stress.sh`) | B7 fin, P1/P4 si de la marge |
| 5 | B3 fin ; **B9 deux machines** avec William | B4 fin : écran d'aide, scénarios inchangés ; page de revue | vidéos de recette (4 clones) | rapport B9, `make check`, ligne de journal | P1/P2 (campagnes) |

**Ordre de merge** (du moins au plus conflictuel) : B6 et B7 (contenu d'un seul clone, traces d'un seul jeu) → B5
(`hud.ron` de trois jeux) → B2 (`Makefile`, `.github/workflows`) → B4 (`control.rs`, `input_devices.rs`) → B1 (`jjrs/`) →
B3 (`ui/lobby.rs`, `core.rs`, `game_over.rs` : dépend de B1 et touche aux textes de B4) → B8 (`tests/budgets.ron`) → B9.

**Conflits prévisibles** : `docs/taches.md` et `docs/taches/dettes.md` (journal et dettes : chaque worker ajoute sa ligne
à la fin de son lot, résoudre par rebase) ; `tests/budgets.ron` (B6, B7, B8) ; `crates/game/src/ui/lobby.rs` (B1 pour la
logique, B3 pour l'écran : B1 livre l'API avant que B3 ne commence) ; `Makefile` (B2 et B1 pour `MATCHBOX_URL`) ;
traces `tests/scenarios/*.trace` (B6 et B7 : jeux différents, pas de conflit ; B8 ne doit en bouger aucune).

**Ce qui attend William** :

- Jour 0 : les décisions 1 à 4 (§ 5), sinon B1 et B2 démarrent sur les défauts.
- Jour 4 : essai manuel E4 (clavier/souris, manette, 1 et 2 joueurs) et écoute des sons de zombies (B6).
- Jour 5 : session en ligne à deux machines réelles et à quatre (B9) ; revue de la page de vidéos des quatre clones.
- Fin de semaine : 30 min en coop par clone (jugement d'équilibrage), notes transformées en dettes **D126** et suivantes.

## 5. Décisions attendues de William

Avec l'option par défaut si William ne répond pas.

1. **Signaling public** : réparer `allumette.estran.studio` (service d'Allumette, avec JWT) ou **héberger** un
   `matchbox_server` nu sur le VPS ? Défaut : héberger `matchbox_server` sur le VPS et en faire la valeur par défaut du
   binaire (la « réparation par défaut » avait été refusée faute de décision explicite ; jouer entre amis l'exige).
2. **Plateformes livrées aux amis** : Mac arm64 seul, Mac + Windows x86-64, ou web ? Défaut : **Mac arm64 + Windows
   x86-64** (le web reste P10 : les boutons online y sont désactivés, `contrat-lancement-web.md`).
3. **Nombre de joueurs visé** : 2 ou 4 ? Défaut : **2 garanti sur les quatre clones, 4 « supporté »** (isaac et zombies
   déjà essayés à 4 ; throne et gungeon à confirmer en B9).
4. **Clone en premier** : si le temps manque, lequel on garantit d'abord ? Défaut : zombies, puis isaac, gungeon,
   throne [INFERENCE : zombies a les plaintes de William en attente, isaac a la campagne la plus récente].
5. **Direction artistique** : garder les sprites CC0 générés, ou acheter/commander ? Défaut : **garder** cette semaine,
   aucun achat, décision pour M4 ; les sprites d'isaac (P5) attendent cette décision.
6. **Q14** : la couverture des mécaniques remplace-t-elle la fourchette de victoires ? Défaut : **oui** (conforme au
   scepticisme sur l'équilibrage automatique) ; William juge à l'essai.
7. **Sons et musique** : autorise-t-on de nouveaux fichiers (licences CC0 / CC BY à citer) et de la musique ? Défaut :
   sons CC0 oui, **pas de musique** cette semaine.
8. **Langue du jeu** : tout en français ? Défaut : **oui** (textes de menus, réseau, fin de partie).
9. **Mac non signé** : accepter le contournement Gatekeeper (clic droit → Ouvrir) pour des amis, sans compte développeur
   Apple ? Défaut : **oui**, notice dans l'archive.
10. **Hébergement des archives** : GitHub Release du dépôt (privé ?) ou page Cloudflare ? Défaut : **GitHub Release**,
    lien partagé aux amis.
11. **TURN** : héberger un serveur TURN (STUN seul aujourd'hui, `jouer-a-deux.md:5-6`) pour les amis derrière un NAT
    symétrique ? Défaut : **non** ; repli documenté = Tailscale.
12. **Les P** : lesquels engager si une voie finit tôt ? Défaut : dans l'ordre de § 3.2 (P1, P2, P7, P6…).
13. **Essai humain** : William joue 30 min en coop par clone en fin de semaine et écrit ses notes dans
    `docs/taches/rapports/plan-polissage-notes.md` ? Défaut : oui (c'est la preuve de l'axe 4).

## 6. Hors périmètre

Dit explicitement, pour qu'aucune mécanique ne passe pour du polissage :

- **Aucune mécanique nouvelle** : pas d'objet, d'ennemi, d'arme, de mode ni de hub dans zombies, throne ou isaac
  (P13 non retenue).
- **Suite de D69** (E8 et au-delà, frontière simulation/présentation) : pas cette semaine.
- **1837** et M4 : après la semaine.
- **Équilibrage automatique** par fourchette de victoires dans la boucle de l'engine : refusé par William.
- **Remappage, musique, build web** : P, jamais B.
- **Modifier la simulation sans preuve §5** : tout réglage de gameplay (B6, B7, P1, P2) passe par `BLESS` justifié ; les
  tâches présentation (B3, B4, B5) ne doivent faire bouger aucune trace.
- **Dettes de tooling sans effet sur le joueur** (D110, D111, D112, D114, D115, D117) : inchangées.

## 7. Règles de la semaine

- Déterminisme et checklist GGRS de `CLAUDE.md` : toute fiche qui touche la simulation (B6, B7, B8) respecte `Fixed`,
  `order_iter!`, `despawn_rollback`, traces au format `ggrs{f=…}`.
- Présentation dérivée de l'état, jamais d'un événement de simulation (`CLAUDE.md`, « Événements dans la simulation »).
- Numéros de dettes : prochain libre **D126** ; réservés cette semaine : **D126 à D130** à l'orchestrateur du plan (les
  workers en ouvrent à partir de **D131** après avoir vérifié `dettes.md`).
- Chaque lot finit par `LIVRÉ <branche>` dans le journal (`docs/taches.md` §10) et un rapport dans
  `docs/taches/rapports/`.
