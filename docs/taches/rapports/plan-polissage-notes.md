# Notes d'essai du plan de polissage (William)

Fichier d'accueil des notes de William pour les essais humains du plan (`docs/taches/plan-polissage.md`). Une section par essai ;
les autres voies y ajoutent la leur (conflit de fusion prévisible : additionner les sections).

## E4 manette — essai physique, William, 2026-10-28 (S1-1, voie b0)

Build local `e4-manette` (zombies), Chrome, manette physique, page `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette`.

- **Verdict** : « ça marche assez bien ».
- **Note (a)** : le contrôle est « un peu trop rigide » → **à régler dans S1-6** (feel, réglages par clone : seuils de stick, accélération,
  inertie), **pas dans S1-1** : aucun seuil (stick 0,4 par axe, visée 0,3, gâchette 0,5/0,35) ni accélération n'a été touché depuis.
  Entrée pour S1-6 : déplacement au stick = huit directions à vitesse constante (`BoxInput` n'a pas de déplacement analogique), ce qui
  explique en partie la rigidité.
- **Note (b)** : une UI des boutons (icônes) quand une manette est branchée, dans les menus et l'aide → pas maintenant ;
  consignée dans `propositions.md` (racine du dépôt).
- Non couvert par cet essai de 2 min (à voir à l'E4 de vendredi 30) : mutation de throne, fin de partie et retour au salon,
  débranchement/rebranchement, L3 pour le sprint, pause qui ne fige pas la partie (D158).

## S1-9 — écoute des sons de zombies (protocoles n° 1 et n° 2 : `docs/taches/rapports/polissage-zombies-sons.md` § 4)

À remplir par William, une ligne par son : « trop fort / ni trop fort ni muet / trop faible ». Pour les sons de l'étape 2, dire aussi si le **repli public** (Kenney, sans écoute de ma part) est acceptable ou à remplacer.

| Son | Verdict |
|---|---|
| tir mitrailleuse | |
| tir pistolet | |
| tir fusil à pompe | |
| tir fusil | |
| recharge | |
| coup porté (hit) | |
| mort d'un zombie | |
| dégâts reçus (hurt) | |
| début / fin de vague (`wave_start`, `wave_end`) : surcharge Sonniss, puis repli Kenney | |
| achat (`purchase`), ramassage (`pickup`) : surcharge, puis repli | |
| fenêtre cassée / réparée (`window_*`, Kenney) | |
| mort par famille de zombie (`death_zombie_*` ; repli public = `death`) | |
| mort du joueur (`death_player`) : surcharge, puis repli | |
| boss : apparition, mort (`boss_spawn`, `boss_death`) : surcharge, puis repli | |
| menus : naviguer, confirmer, retour (`menu_*`) : surcharge, puis repli | |

## S2-11 — écoute des sons de throne (protocole : `docs/taches/rapports/polissage-throne-sons.md` § 4)

À remplir par William, une ligne par famille : « trop fort / ni trop fort ni muet / trop faible » ; dire si le **repli public** est acceptable ou à remplacer (tout est choisi sans écoute de ma part).

| Son | Verdict |
|---|---|
| armes à balles (revolver, fusil à pompe) | |
| lanceurs (lance-grenades, mortier) : tirs de fusil plus graves | |
| énergie (laser, plasma) : tirs de .22 changés de hauteur | |
| lames (lance-lames, disque) : impacts Kenney | |
| recharges (revolver, lanceur, énergie, lames) | |
| morts par famille (rongeurs, bêtes, humains, machines) : surcharge, puis repli | |
| étage / portail, ramassage | |

## E4 du 30 — essai physique avec William, vendredi 2026-10-30 (S1-1 + S1-6 + audio navigateur S1-7, voie b0)

Préparé le 2026-10-29 par la voie b0. **Verdicts de William : non encore donnés** (l'essai n'a pas eu lieu au moment de cette écriture ;
chaque ligne ci-dessous est à remplir mot pour mot pendant ou juste après l'essai, jamais reconstituée). Serveur déjà en route, ne pas le
relancer : `http://127.0.0.1:8777/` (PID 22332, sert `website/static` du worktree b0). Durée prévue : 30 min. Manette branchée **avant** d'ouvrir la page ;
Chrome normal (son de ton poste : volume à ta main, **rien n'est lancé par la voie**).

| Quoi | URL (zombies ; throne : `game=throne`) |
|---|---|
| avant S1-6 (e4-manette) | `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette` |
| après S1-6 (e4-feel, interpolation, flash) | `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-feel` |
| avec les sons de S1-7 et S1-9 (e4-sons, repli public Kenney) | `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-sons` |
| FPS réel de l'écran (120 Hz ?) | `http://127.0.0.1:8777/loader.html?game=zombies&build=perf-b&perf=1` (F8 cache la surcouche) |

### Protocole en dix lignes (détail : `polissage-manette.md` § 7, `polissage-perf-web.md` § 6)

1. `e4-manette` : stick gauche (course), stick droit (visée gardée au repos), RT (tir progressif ?), LB (dash), L3 (sprint), pause Start → « Touches » → retour.
2. Même partie sur `e4-feel` : course, dash, tir **agréables ou non**, et la rigidité du stick par rapport à `e4-manette`.
3. Fin de partie (se faire tuer) : A rejoue, ou B puis A depuis l'écran d'attente ; débrancher puis rebrancher la manette (appui sur un bouton).
4. Throne (`game=throne`, mêmes builds) : course, dash, tir ; 3 kills puis fin de salle : **mutation** (← / → surligne, A valide).
5. 120 Hz : `perf-b&perf=1`, lire le FPS d'affichage et de simulation, dire si l'écran est bien à 120 Hz.
6. **Audio navigateur (S1-7)**, build `e4-sons`, **avant tout clic** : ouvrir la page, ne rien cliquer ; la console (F12) peut dire « AudioContext was not allowed to start » (normal).
7. Premier clic ou première touche : le son doit démarrer ; tirer, recharger, tuer un zombie, casser une fenêtre. Noter : son au premier tir après le clic (oui/non).
8. Échap → Réglages : ← baisse le volume (un cran s'entend au tir suivant), 0 % coupe tout, → remonte à 50 % : son revenu (oui/non).
9. Écoute des sons de S1-9 sur ce même build (repli public) : remplir le tableau « S1-9 » plus haut, une ligne par son, et dire si le repli Kenney est acceptable.
10. Si rien ne sonne après le clic : ne rien corriger sur place ; la reprise de l'`AudioContext` sur `pointerdown` / `keydown` (`website/static/loader.js`) est le correctif attendu (non écrit, ≈ 0,25 j).

### Notes de William (mot pour mot quand c'est un verdict)

| Point | Verdict / note |
|---|---|
| (1) e4-manette : course, visée, RT, L3, pause et « Touches » | |
| (2) e4-feel : course, dash, tir « agréable » ? | |
| (3) fin de partie, débrancher / rebrancher | |
| (4) throne : course, dash, tir, mutation à la manette | |
| (5) 120 Hz : FPS d'affichage / de simulation lus (`perf-b&perf=1`) | |
| (6–8) audio navigateur : son après le premier clic ; 0 % puis 50 % | |
| (9) repli public Kenney acceptable ? (tableau S1-9 rempli ci-dessus) | |
| (10) « encore rigide ? » (stick, e4-feel contre e4-manette) | |

Point (10) : si William répond oui, **rien n'est codé**. Estimation de l'entrée analogique : ≈ 1 j (`BoxInput` n'a pas de déplacement
analogique : huit directions à vitesse constante, `polissage-manette.md` § 9 ; tâche à part, dite au coordinateur).

Build `e4-sons` (zombies et throne, `website/static/builds/e4-sons/`, ignoré par git, tête `polissage-e4` sur `origin/polissage-integration` `4cea5046`,
repli public : aucune surcharge privée copiée, `ALACOD_WEB_PRIVATE_ASSETS` absent) : `CARGO_TARGET_DIR=target_wasm WEB_BUILD_ID=e4-sons WEB_OPTIMIZE=0
WEB_ENABLE_SOLO=1 WASM_BINDGEN=/private/tmp/wbg/wasm-bindgen-0.2.129-aarch64-apple-darwin/wasm-bindgen node scripts/build-web-games.mjs zombies throne`, puis
`git checkout website/static/releases.json` ; servi par le serveur du 8777 (`build.json` et `wasm_bg.wasm` répondent 200 : 250 Mo zombies, 243 Mo throne, build de développement).
Aucun son lancé par la voie (`ALACOD_MUTE=1`, pas de navigateur ouvert) : le son se vérifie seulement chez William.
