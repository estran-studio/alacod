# Rapport — polissage-perf-web (S1-5, voie b0)

Fiche : `docs/taches/polissage-perf-web.md`. Branche `polissage-perf-web` depuis `origin/polissage-integration` (`a7a7a49b`). Section de
conventions : **§ 97**. Machine : MacBook Pro (Apple M4 Max, Liquid Retina XDR, macOS 26), Chrome for Testing 150.0.7871.24 (navigateur
headless géré par omp, `--headless=new --mute-audio`, rendu WebGL par ANGLE/Metal), **aucun son** (§ 1.4).

## 0. Verdict

- Ni zombies ni throne ne passent sous 60 Hz de simulation dans Chrome : **60,0 fps de simulation (p5 59,7 à 60,0)**, **0 rollback**,
  une frame simulée coûte **0,5 à 1,0 ms en moyenne** (pire 3 à 21 ms) pour 16,7 ms de budget, le fil principal du navigateur
  (simulation, ECS, appels WebGL) est occupé **18 à 27 %** du temps. Le FPS d'affichage mesuré est **60** : c'est la cadence de la
  composition de ce Chrome, pas la capacité du jeu (§ 2.2).
- Aucune dégradation n'est nécessaire, et la seule prévue par la fiche (lumière coupée pour le web) **est déjà en place** : les builds
  web n'activent pas la feature `lighting` (`bevy_light_2d` absent du graphe wasm, § 4). Aucun contenu réduit, aucune trace bougée.
- Le vrai point faible du web est la **taille** : `wasm_bg.wasm` fait **48,2 Mo (zombies) et 41,4 Mo (throne)**, soit 7,7 / 7,6 Mo en
  brotli et 11,8 / 11,5 Mo en gzip, au-dessus de la limite de 25 Mio de Cloudflare Pages (avertissement du script de build, § 3).
- **Non mesuré** : deux onglets en ligne (WebRTC impossible sur ce Mac, § 2.3), 120 Hz (écran ProMotion de William : à lire à l'E4 avec
  `?perf=1`, § 6), charge pire cas (§ 2.4), Firefox, Safari, autre machine.

## 1. Le compteur et la méthode

### 1.1 Compteur (`crates/game/src/fps_counter.rs`, conventions § 97)

`?perf=1` (surcouche + console) ou `?perf=console` dans l'URL du chargeur ; `ALACOD_FPS=1|console` en natif ; **F8** montre/cache la
surcouche. Une ligne par seconde dans la console : `perf fps_display=… fps_sim=… steps=… rollbacks=… advance_avg_ms=… advance_max_ms=…
frame_max_ms=… frame=… entities=…` ; en wasm, le dernier échantillon est aussi dans l'attribut `data-perf-sample` du canvas. Hors
`GgrsSchedule` (systèmes dans `AdvanceWorld`/`LoadWorld`, comme `utils::snapshot_profile`, et dans `Last`), rien en rollback, rien dans
un hash. **Sans demande, le plugin n'est pas installé** (coût nul). Une exception de `scripts/determinism-allowlist.json`
(`Instant::now()` du chrono d'une frame simulée, justifiée). Test unitaire : `fps_counter::tests` (2).

`loader.js` pose `data-perf` depuis `?perf=` de l'URL du chargeur **ou de la page hôte** (`window.parent.location.search`, même origine) :
`/play?game=zombies&perf=1` marche sans toucher au Svelte (voie b2).

### 1.2 Outils de mesure (committés, ce ne sont pas des pages du site)

- `website/static/perf-duo.html` : lance un build sans l'hôte ; `&mode=local` = deux joueurs locaux dans une page ; sans `mode=local`, en
  ligne par `matchbox_server 127.0.0.1:3536`, un onglet par joueur (`&role=0|1&lobby=…`).
- `scripts/perf-web/gamepad-bot.js` : manette standard simulée qui bouge (cap aléatoire graine fixe), tire par rafales de 2,5 s toutes les
  12 s (la réserve de la mitrailleuse ne dure sinon que 30 s), recharge, esquive, appuie sur A (relance après la mort). Le bot meurt
  toutes les 30 à 60 s : la colonne « relances » compte les parties perdues et relancées en 3 minutes (le jeu atteint la vague 2 à 4 ;
  les secondes de relance sont écartées du résumé).
- `scripts/perf-web/measure-helpers.js` (cellule JS d'`eval` d'omp) et `scripts/perf-web/table.mjs` (tableau Markdown depuis les JSON
  de `docs/taches/rapports/polissage-perf-web/`).

### 1.3 Protocole

Builds **release** (`WEB_PROFILE=release`, profil Cargo `opt-level = 'z'`, LTO, `codegen-units = 1`, puis `wasm-opt --strip-debug -O2` : ce
que le site publie), servis par `python3 -m http.server` sur 127.0.0.1:8778. Par série : cache du navigateur désactivé (chargement à
froid), fenêtre 1280×633 px CSS (écran Retina, facteur 2 : 2560×1266 px rendus), partie de **3 minutes** après la première frame de jeu, un échantillon par seconde (≈ 177 par série), deux
premières secondes écartées. Colonnes : médiane et p5 (le 5 % le plus bas) ; « fil principal occupé » = Δ`TaskDuration` / Δtemps du
CDP (`Performance.getMetrics`) sur les 3 minutes. Deux builds :
- `perf-a` : **l'intégration** `a7a7a49b` + le compteur (sans l'interpolation de S1-6, pas encore fusionnée) ;
- `perf-b` : un **build jetable, non poussé** = `perf-a` + fusion de `origin/polissage-feel` (S1-6, `d70be914`) ; interpolation
  d'affichage coupée par `interpolation: (enabled: false …)` dans le `camera.ron` du build, même binaire.

### 1.4 Son

Consigne de William (2026-10-29) : aucun run sonore. Les mesures tournent dans le Chrome headless d'omp, dont le processus principal porte
`--mute-audio` ; `measure-helpers.js` **refuse** de mesurer sinon (`assertMuted`) et suspend tout `AudioContext` de la page. Le bot n'envoie
que des événements manette (aucun geste utilisateur, donc aucun contexte audio actif). Avant la consigne, mes essais p2p avec des Chrome lancés à la main (headed, sans `--mute-audio`, ≈ 15 s chacun, même absence de geste) : signalé au coordinateur.

## 2. Résultats

### 2.1 Tableau clone × joueurs

Chargement = de la navigation à la première seconde en jeu (`frame > 0`), cache désactivé, **sur 127.0.0.1** (aucune latence ni bande
passante réseau) ; l'échantillon de la page se publie une fois par seconde (±1 s). Tous les chiffres d'affichage valent pour la
fenêtre de ce Chrome (voir § 2.2).

| clone | joueurs | build | affichage méd. / p5 (fps) | sim méd. / p5 (fps) | pas sim moy. / max (ms) | fil principal occupé | entités | chargement | série |
|---|---|---|---|---|---|---|---|---|---|
| zombies | 1 | perf-a | 60,0 / 59,7 | 60,0 / 59,7 | 0,96 / 8,9 | 24,9 % | 324–337 | 1,9 s | `perf-a-zombies1_b` |
| zombies | 2 locaux | perf-a | 60,0 / 59,7 | 60,0 / 59,7 | 1,00 / 7,8 | 26,6 % | 317–340 | 2,0 s | `perf-a-zombies2` |
| zombies | 2 en ligne (2 onglets) | — | **non mesuré** (§ 2.3) | | | | | | |
| throne | 1 | perf-a | 60,0 / 59,8 | 60,0 / 59,9 | 0,56 / 5,2 | 19,3 % | 78–93 | 1,9 s | `perf-a-throne1` |
| throne | 2 locaux | perf-a | 60,0 / 60,0 | 60,0 / 60,0 | 0,58 / 3,2 | 20,6 % | 83–104 | 2,0 s | `perf-a-throne2` |
| throne | 2 en ligne (2 onglets) | — | **non mesuré** (§ 2.3) | | | | | | |

Tableau complet généré (`node scripts/perf-web/table.mjs`), avec charge du Mac et alimentation de chaque série :

| série | affichage méd. / p5 (fps) | sim méd. / p5 (fps) | pas sim moy. / max (ms) | frame max (ms) | entités | relances | fil principal occupé (%) | chargement (s) | charge Mac (1 min) | alimentation |
|---|---|---|---|---|---|---|---|---|---|---|
| perf-a-throne1 | 60.0 / 59.8 | 60.0 / 59.9 | 0.56 / 5.2 | 30 | 78–93 | 10 | 19.3 | 1.9 | 28.66→19.02 | AC Power→AC Power |
| perf-a-throne2 | 60.0 / 60.0 | 60.0 / 60.0 | 0.58 / 3.2 | 31 | 83–104 | 6 | 20.6 | 2.0 | 19.44→21.29 | AC Power→AC Power |
| perf-a-zombies1 | 59.7 / 29.9 | 60.0 / 59.7 | 0.92 / 21.0 | 63 | 291–303 | 5 | 20.7 | 2.0 | 23.74→28.66 | Battery Power→AC Power |
| perf-a-zombies1_b | 60.0 / 59.7 | 60.0 / 59.7 | 0.96 / 8.9 | 53 | 324–337 | 4 | 24.9 | 1.9 | 20.22→15.46 | AC Power→AC Power |
| perf-a-zombies2 | 60.0 / 59.7 | 60.0 / 59.7 | 1.00 / 7.8 | 30 | 317–340 | 2 | 26.6 | 2.0 | 19.02→19.44 | AC Power→AC Power |
| perf-b-throne1_interp_off | 60.0 / 60.0 | 60.0 / 60.0 | 0.54 / 2.8 | 3791 | 74–92 | 10 | 18.4 | 1.9 | 13.29→12.33 | AC Power→AC Power |
| perf-b-throne1_interp_on | 60.0 / 60.0 | 60.0 / 60.0 | 0.53 / 1.9 | 24 | 76–95 | 9 | 18.1 | 1.8 | 51.63→13.29 | AC Power→AC Power |
| perf-b-zombies1_interp_off | 60.0 / 59.7 | 60.0 / 59.7 | 0.89 / 7.9 | 35 | 282–294 | 5 | 24.1 | 2.0 | 16.95→51.63 | AC Power→AC Power |
| perf-b-zombies1_interp_on | 60.0 / 59.7 | 60.0 / 59.7 | 0.94 / 10.9 | 49 | 331–343 | 4 | 25 | 1.9 | 26.71→16.95 | AC Power→AC Power |

Lecture :
- `perf-a-zombies1` est la première série de zombies, faite pendant que le Mac repassait de la batterie au secteur : p5 d'affichage à 29,9
  (une seconde sur dix à 30 fps). Gardée pour mémoire ; `perf-a-zombies1_b` est la série propre, utilisée dans le tableau.
- Les séries de la nuit (non gardées) montrent à quel point **le FPS d'affichage de ce Chrome dépend de l'état du Mac** : 85 fps, puis
  28–30 fps constants quand la batterie est passée sous 15 % (mode économie), à build et charge de jeu identiques ; la simulation, elle,
  est restée à 60 fps. Aucun chiffre d'affichage de cette période n'est repris.
- « 2 locaux » = **deux joueurs dans la même page** (session synctest, une manette qui pilote les deux : D159), pas de WebRTC. C'est le
  coût de simulation et de rendu de deux joueurs, sans le coût réseau ni les rollbacks que causerait la prédiction.
- Charge du Mac pendant les séries : moyenne sur 1 min de 13 à 52 (14 cœurs : d'autres voies compilaient et jouaient des scénarios), 0 à 2 processus
  `cargo/rustc/scenarios` pendant les séries elles-mêmes. **Ce n'est pas un Mac au calme.** Les séries se ressemblent malgré cela (19 à
  27 % de fil principal), un hoquet isolé reste visible : `perf-b-throne1_interp_off` a une frame de 3,8 s (le système, pas le jeu : le pas de sim
  moyen est identique).

### 2.2 Ce que « FPS d'affichage » veut dire ici

Le compteur compte les passages de `Last` : la cadence de `requestAnimationFrame`, bornée par la composition de Chrome (60 Hz dans ces
séries). Un fil principal occupé à 20-27 % montre que le jeu pourrait aller plus vite si l'affichage le permettait. Profil CPU de 5 s pris
pendant une série à 28 fps (batterie) : fil principal **86 % au repos**, 1,2 % `__wbg_info` (journal), le reste dans des fonctions wasm
sans nom (`wasm-opt --strip-debug`) : le plafond n'était pas du CPU.

### 2.3 Pourquoi pas de p2p à deux onglets

`perf-duo.html` en ligne (deux onglets, `matchbox_server` local) se connecte au signalement mais l'ICE échoue : Chrome ne propose
que `100.64.0.1` (interface `utun1`, VPN en route par défaut), et un paquet UDP du Mac à lui-même par cette interface est perdu
(mesuré : `socket` Python lié à `100.64.0.1` : rien ne revient ; sur `en0` : ça revient). Essayés sans effet : aucun serveur ICE, mDNS
désactivé, `--force-webrtc-ip-handling-policy=default_public_and_private_interfaces`, `--allow-loopback-in-peer-connection`. Il faudra
un Mac sans ce VPN, ou deux machines (S1-3, `polissage-reseau-amis`). Protocole prêt : deux onglets
`…/perf-duo.html?game=zombies&build=<id>&role=0|1&lobby=x&perf=1` (le STUN public du jeu rallonge l'attente ICE de ≈ 30 s :
`&stun=1` pour le garder).

### 2.4 Charge pire cas : non mesurée

Le bot ne dépasse pas ≈ 340 entités rollback (zombies) ni ≈ 105 (throne). Pour mémoire, côté natif (`target/metrics/latest.json`, Mac)
le scénario le plus lourd en balles, `bench_tears` (502 entités dont 432 balles), coûte 0,32 ms en moyenne et 4,4 ms au pire par frame
simulée. Je n'ai pas de rapport wasm/natif mesuré sur une même charge : pas d'extrapolation chiffrée. À mesurer si un clone doit un jour
porter autant d'entités.

### 2.5 Avec et sans l'interpolation de S1-6 (build jetable `perf-b`)

| clone | interpolation | affichage méd. / p5 | pas sim moy. / max (ms) | fil principal occupé | série |
|---|---|---|---|---|---|
| zombies | oui | 60,0 / 59,7 | 0,94 / 10,9 | 25,0 % | `perf-b-zombies1_interp_on` |
| zombies | non | 60,0 / 59,7 | 0,89 / 7,9 | 24,1 % | `perf-b-zombies1_interp_off` |
| throne | oui | 60,0 / 60,0 | 0,53 / 1,9 | 18,1 % | `perf-b-throne1_interp_on` |
| throne | non | 60,0 / 60,0 | 0,54 / 2,8 | 18,4 % | `perf-b-throne1_interp_off` |

**Coût de l'interpolation : dans le bruit** (écart de fil principal ≤ 1 point, dans les deux sens). Son effet visible n'existe qu'au-dessus de
60 Hz d'affichage, que ce Chrome n'atteint pas ici : à juger à l'E4 sur l'écran de William (§ 6).

## 3. Taille et chargement

| clone | `wasm_bg.wasm` brut | gzip -9 | brotli -11 | `assets/` | chargement à froid (127.0.0.1) |
|---|---|---|---|---|---|
| zombies | 48 149 686 o (`perf-a`) ; 48 227 433 o (`perf-b`, avec S1-6) | 11,76 Mo | 7,67 Mo | 8,0 Mio | 1,9 à 2,0 s |
| throne | 41 310 222 o (`perf-a`) ; 41 388 444 o (`perf-b`) | 11,53 Mo | 7,58 Mo | 0,9 Mio | 1,9 à 2,0 s |

- Les deux `wasm_bg.wasm` dépassent **25 Mio** (le script de build l'avertit : « Pages 25 MiB limit exceeded; use R2 or an optimized
  build »). Le `Makefile` (`build_*_web`) utilisait `wasm-opt -Oz` ; `build-web-games.mjs` fait `-O2`. **`-Oz` n'a pas été essayé** : c'est
  la première piste de taille (et une mesure de FPS à refaire ensuite, `-Oz` ralentit en général).
- Chargement mesuré **sur le disque local** : compilation et initialisation du module, sans transfert réseau. À titre d'arithmétique
  seulement (pas une mesure) : 7,7 Mo en brotli sur une ligne de 50 Mbit/s = 1,2 s de plus.
- Une première navigation sur ce build, la toute première fois, a pris 19 s (Mac chargé à 35, cache vide, pas de code cache du moteur
  wasm de Chrome) : une seule observation, non reproduite avec le cache désactivé (≈ 2 s), donc non retenue.
- Mémoire : non mesurée.

## 4. Un clone sous 60 Hz ? Profil et dégradation

Aucun clone n'est sous 60 Hz de simulation (§ 0). La dégradation « lumière coupée pour le web » (feature `lighting`) **est déjà la
configuration publiée** : `scripts/build-web-games.mjs` construit `--no-default-features --features render`, et
`cargo tree -e features --target wasm32-unknown-unknown -p zombies --no-default-features --features render -i bevy_light_2d` répond
« did not match any packages » (la crate n'est pas dans le graphe). Aucune autre dégradation n'a été appliquée ; **aucun contenu n'a été
réduit** (décision 3 : à William).

## 5. Issue par point de la fiche

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Compteur FPS d'affichage et de simulation, présentation seule, URL / console | **fait** | `fps_counter.rs`, § 97, 2 tests ; sortie console § 1.1 ; capture `docs/taches/rapports/polissage-perf-web/surcouche-zombies.png` |
| 2 | Mesures par clone, 1 puis 2 joueurs, 3 minutes, taille et chargement | **fait pour 1 joueur et 2 locaux** ; **2 onglets en ligne non mesuré** (§ 2.3) | § 2.1, § 3 ; `docs/taches/rapports/polissage-perf-web/*.json` |
| 3 | Sous 60 Hz : profil, dégradation à trace identique | **sans objet** (aucun clone sous 60 Hz) ; lumière déjà coupée (§ 4) | § 2.2, § 4 |
| — | Avec/sans l'interpolation de S1-6 | **fait** sur un build jetable (§ 2.5) | `perf-b-*.json` |
| — | E4 de vendredi 30 | **préparé** ; jugement de William **non vérifié** (§ 6) | § 6 |

## 6. E4 de vendredi 30 (S1-1 et S1-6) : préparation

Serveur statique `http://127.0.0.1:8777/` (PID 22332, dossier `website/static` de ce worktree) :
- S1-1, avant S1-6 : `loader.html?game=zombies&build=e4-manette` ; throne : `…game=throne&build=e4-manette`.
- S1-1 + S1-6, après : `loader.html?game=zombies&build=e4-feel` ; throne : `…game=throne&build=e4-feel`.
- **En plus, pour lire le FPS sur l'écran de William** (build release du jetable `perf-b`, interpolation active, compteur dedans) :
  `loader.html?game=zombies&build=perf-b&perf=1` et `…game=throne&build=perf-b&perf=1` (F8 cache la surcouche). C'est la mesure à 120 Hz
  qui manque (§ 2.2) et la seule où l'interpolation se voit.
Protocole : `docs/taches/rapports/polissage-manette.md` § 7 (manette branchée **avant** d'ouvrir la page, premier appui pour l'exposer,
RT, sticks, pause/« Touches », fin de partie, débrancher/rebrancher, mutation de throne) puis comparaison e4-manette contre e4-feel : course,
dash, tir (« agréable » ou non), la rigidité du stick (question 10 : déplacement analogique ou non). **Aucun son sans que William l'ait
demandé : il décide du volume de son poste.**

### Notes de l'E4

À remplir pendant l'essai ; copiées dans `docs/taches/rapports/plan-polissage-notes.md` et les sections « E4 » de `polissage-manette.md` et
`polissage-feel.md`.

*Non encore fait.*

## 7. Vérifications

`CARGO_TARGET_DIR` partagé `.target-mac`, `env -u CI`, `ALACOD_MUTE=1`, tête de branche au moment des vérifications : voir le journal.
- `make format check_forbidden check_rollback_registration check_sim_boundary lint` : **RC 0** (« OK: aucun nouveau motif interdit ; 52
  occurrences auditées », « OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69 », les quatre jeux sans erreur de lint).
  Premier passage : `check_forbidden` refusait `Instant::now()` dans `fps_counter.rs` : exception exacte et justifiée ajoutée à
  `scripts/determinism-allowlist.json`.
- `make test_crates` : **RC 0**, 93 blocs `test result: ok`, aucun échec (5 min 12).
- `make test_scenarios_par` **sans `BLESS`** : **RC 0**, **810 scénarios**, quatre tranches `ok` (`suite en 4 tranches : 634 s`) : **aucune trace
  ne bouge** (le compteur n'est pas installé en headless).
- `make schedule_order_diff BASE=origin/polissage-integration` : « OK : ordre des systèmes et des types rollback identiques (4550 lignes de
  schedules, 645 types) ».
- `make test_tools` : **3 échecs** `test_claude_say` (le test lie `/usr/bin/python3` dans un `PATH` vide : § 5.6 de `polissage-manette.md`),
  hors S1-5 ; `make check` entier n'est donc pas vert sur ce Mac, ses autres cibles le sont.
- Natif : `ALACOD_FPS=1` non lancé (aucun run de jeu avec rendu sur le Mac sans que William l'ait demandé).

## 8. Reste

- E4 avec William (vendredi 30) et ses notes (≈ 0,25 j avec les corrections éventuelles).
- Mesure à deux onglets en ligne sur un Mac sans VPN ou sur deux machines (S1-3).
- Mesure à 120 Hz dans le Chrome de William (`perf-b`, § 6).
- Piste de taille : `wasm-opt -Oz` (puis refaire la série de FPS).
- Charge pire cas dans le navigateur (§ 2.4).
