# R-1 — Rendu 3D « façon sprite » de ce qui bouge (Mac de William)

Jalon servi : **S1, dim 15** (plan `docs/taches/plan-1837-cli.md` § 6, ligne R-1 : « une bouche de canon par direction », devenue un point du modèle ; la planche du
Frère jugée par William dans son rendu définitif). Branche engine `wquintal/r-1-rendu-facon-sprite` (base `d04cea3e`, WIP du VPS `93f5bf7f` jamais compilé, repris
sur le Mac le 7 nov) ; branche 1837 `r-1-rendu`. Convention : `docs/conventions.md` § 111 ; dettes D198–D199. **Présentation pure** : aucune trace existante ne bouge,
rien de nouveau dans le graphe des crates `SIM` : `content` (SIM) reçoit seulement la donnée du kind `Model` (types `serde`, aucune dépendance nouvelle, lue par la seule
présentation, comme `Ui` et `Camera`) ; le rendu est dans `game` (PENDING D69, `PresentationPlugin`) et un ordre de système dans `map_ldtk`. Seule trace neuve : `model_pantin`.

## Engine (générique, aucun nom de 1837)

| Point | Issue | Preuve |
|---|---|---|
| 1. Chargement `anime.glb` + `anime.json` | fait | kind `Model` (`crates/content/src/models.rs`, `models/models.ron`), manifeste lu par le registre (embarqué en web), lint (fichiers, plages, clips, objets, repères, armes), schéma ; `cargo test -p content models` : 4 tests verts. `alacod lint` de 1837 : « aucune erreur ». |
| 2. Animation et mélange des gestes | fait (D198, D199) | `model3d/pose.rs` (pur, testé) + `animate.rs` : esquive, coup, tir, épaulé/en joue, rechargement, allures ; variantes de direction de la marche, vitesse des pieds (`moments.vitesse`), fondus 0,1 s / 0,05 s. Le banc n'a pas de masque haut/bas : marcher en tirant = clips cuits par direction. Planche ci-dessous. |
| 3. Matériau, contour, ombre | fait | `facon_sprite.wgsl` (sRGB × rampe, lumière dans le repère de la vue, normales lissées par position), `model_pose.wgsl` (contour d'un pixel de sprite, teinte × `outline` sur les valeurs sRGB), `model_shadow.wgsl` (maillage projeté au sol). Valeurs dans le RON du jeu (1837 : 0,66 / 1,12 / 0,75 / 91). |
| 4. Caméra ortho 45°, calage | fait | un banc par personnage (Camera3d orthographique à `elevation_deg`, image ×2) ; 1 m de profondeur = `sin e` m de largeur (45° ≈ 64/91). Comparaison au banc : même pose, même direction (image ci-dessous). |
| 5. Tri en y | fait | `placement::y_sort` (opt-in `y_sort`), ordonné après `lift_characters_to_entity_layer` ; `offset_z` de l'image d'un modèle (1,0 comme `alacod import-sprite`). Deux défauts trouvés en capture et corrigés : le fantassin posé par scénario passait sous les tuiles (tri qui écrasait la remontée de couche), le décor couvrait le Frère (image à `z` 0 contre 1,0 pour les planches). Test `le_pied_le_plus_bas_passe_devant` ; preuve `tri_y` de 1837. |
| 6. Arme à la main, balles depuis la bouche | fait | objet du manifeste montré du côté de la prise du clip, objets portés (clé à la ceinture) ; **bouche = présentation** (`offset_model_bullets` décale le dessin de la balle : écart latéral résorbé en 8 frames, hauteur du canon gardée) : le repère vient du modèle animé en `f32` du rendu, il ne peut pas entrer dans la simulation ; aucune trace ne bouge. |
| 7. Présentation pure | fait | `PresentationPlugin` seulement ; traces : `two_players_shooting`, `four_players_idle`, `dash_aim_change` inchangées (`make test_scenarios SCENARIO=…`, 5 tests OK) ; `make check_sim_boundary` dans le contrôle rapide. |
| 8. Natif et wasm32 dans Chrome | fait | natif (Metal, M4 Max) ; **essai Chrome** (une session, muette) : build wasm de `zombies` dont le joueur était, le temps de l'essai, dessiné par le pantin (changement jeté, non commité) : le pantin s'affiche, ombré, contour et ombre, en **WebGL2** (`ANGLE Metal Renderer: Apple M4 Max`), aucune erreur de shader dans la console. `make check_rapide WASM=1` vert (bas de page). |
| 9. Testbed et 1837 | fait | pantin synthétique (`scripts/gen-pantin.py`, 40 clips) et scénario `model_pantin` (trace neuve, `BLESS` de ce seul scénario) ; testbed `y_sort: false` (décision du 7 nov). 1837 : le Frère et le fantassin (partie 1837 ci-dessous). |
| 10. § 111, D198–D199 | fait | `docs/conventions.md` § 111 ; `docs/taches/dettes.md` D198 (mort, coup reçu, victoire non joués ; tenue en salle pleine non mesurée), D199 (épauler sans phase dans la simulation). D157 se ferme pour ce qui est câblé (esquive, mêlée, tir, en joue, rechargement, allures). |

## 1837 (branche `r-1-rendu`)

- `assets/models/models.ron` (charte du banc, U = 24, 45°, `y_sort: true`, `offset_z: 1.0`), modèles copiés de `1867_lore/assets/anime/{frere-chasseur,fantassin-de-ligne}/`
  (identiques octet pour octet à `site/labo/armes/*.glb|json` du banc ; commits `103068e`, `cfba72a`), licences dans `assets/assets.yaml`, `game.ron` déclare `Model`.
  Le Frère (`player`) : fusil (`fusil_chasse` → `fusil-chasse`, bouche `bouche`), hache au coup (`bare_hands` → objet `hache`), gigues à l'esquive. Le fantassin : Brown Bess,
  clé à la ceinture (objet porté).
- `rev` de l'engine épinglé sur la tête de la branche R-1 (à réépingler sur `main` par le coordinateur 1837 après le merge de l'engine).
- Preuves rejouables : `docs/preuves/r-1/` (`rendu_poses.ron`, `comparaison_banc.ron`, `tri_y.ron`, `README.md` : commande de capture hors écran par le
  `play_scenario` de l'engine, `ALACOD_GAME_NAME=1837 ALACOD_GAME_DIR=…`).

## Captures

Dans `~/orca/runs/alpha/logs/r-1-captures/` (Mac) :

- `frere-cote-a-cote.png` : le Frère dans 1837 (échelle du jeu, ×9 au plus proche) à côté du banc `/labo/armes/` (Chrome, rendu « façon sprite », zoom 4), même pose
  (fusil au repos), même direction (corps vers le bas à droite) ;
- `frere-planche.png` : planche de poses, trois directions de visée (droite, bas, haut-droite) × repos, tir en marchant (variante de direction : on marche de côté
  en visant), rechargement en marchant, tir à l'arrêt, rechargement, coup de hache ;
- `tri-y.png` : le Frère devant puis derrière la lanterne ; `scene-fantassin.png` : le Frère et le fantassin dans la cour ;
- `pantin-planche.png` : le pantin du testbed ; `web-chrome-pantin.png` / `-zoom.png` : l'essai wasm dans Chrome ; `banc-frere.png` : la capture du banc.

Constats : contour sombre d'un pixel, ombre portée en haut à droite (soleil du banc), arme dans la main, hache au coup seulement, pas de saut de direction. L'épaulé ne
se voit pas en 1837 (D199 : le fusil tire à l'appui et recharge aussitôt). Écran du Mac éteint pendant la nuit : la fenêtre native rendait noir à la capture
(`alacod remote screenshot`) ; les preuves sont donc des captures **hors écran** (même rendu, image cible), le jeu natif est à voir par William.

## Protocole pour la partie du soir (dix lignes)

1. Après les merges (engine sur `main`, `r-1-rendu` sur `main` de 1837 avec le `rev` réépinglé) : `cd ~/Project/bascanada/1837 && ./jouer.sh`.
2. Le Frère paraît en 3D : bonnet rouge, fusil en main, contour sombre d'un pixel, ombre en haut à droite.
3. Viser à la souris tout autour : le corps tourne sans crans (visée libre), la balle part de la bouche du canon.
4. Marcher dans les huit directions, en visant ailleurs : les jambes suivent la marche, les pieds ne glissent pas.
5. Tirer en marchant (clip « tir en marchant ») ; puis le rechargement (fusil à un coup : aussitôt après le tir).
6. Mêlée : la hache paraît dans la main pendant le coup, le fusil disparaît.
7. Esquive : un pas de gigue (huit gigues à tour de rôle).
8. Passer derrière la lanterne et la palissade : le décor passe devant (tri en y) ; le fantassin : Brown Bess et clé à la ceinture.
9. Connus : pas d'épaulé (D199), pas d'animation de mort ni de coup reçu (D198).
10. À noter pour le retour : direction, pieds, arme dans la main, contour, ombre, fluidité (deux corps animés ; `ALACOD_FPS=1` pour le compteur).

## Ce qui reste

- D198 (mort, coup reçu, victoire ; mesure en salle pleine, Steam Deck), D199 (épaulé dans la simulation, P-1).
- L'essai wasm de **1837** lui-même attend `alacod build --web` (M-1, pas encore mergé) : rebaser sur `main` après son merge et refaire l'essai dans Chrome.
- La partie du soir de William (natif) fait foi pour le jeu en mouvement.
