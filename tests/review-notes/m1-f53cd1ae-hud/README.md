# HUD et quantité de texte — revue du 2026-10-14

William : « Pour les HUD, il a bcp de overlap entre les elements » et
« il a bcp de pollisage possible aussi, il a bcp trop de texte et information aussi.
J'aimerais qu'on travailles les 2 points, ». Correctifs de présentation autorisés.
Gravité proposée : à corriger ; confirmation visuelle finale encore attendue.
Code livré séparément dans `3ae7d2e5`.

## Analyse vidéo

Sources : `docs/digests/videos/throne_solo.mp4` (8 s, 20 s) et `throne_quad.mp4`
(10 s), puis rejeu du début de la deuxième partie humaine (graine 123456,
original conservé dans `../m1-f53cd1ae-run-02/original.ron`, f0–3700).
Les vidéos du digest sont anciennes : leurs timings de mutation/ennemis ne
représentent pas les correctifs de gameplay actuels. Les trois captures `digest-*`
conservent ce contexte. Leur bande titre/commit/frame en haut est une annotation
ajoutée par l'encodeur vidéo, pas un widget du HUD.

Le rejeu actuel avec les features du lancement natif confirme :

- **Inspecteur moteur** : panneau World Inspector ouvert dans le quart haut gauche
  alors que la partie est lancée avec `make throne --features native`.
- **Chevauchement bas gauche** : Weapon/Ammo/reload de debug sur vie, niveau et rads.
- **Chevauchement bas droite** : frame/FPS/version/ennemis sur les munitions actives.
- **Doublons permanents** : chargeur/réserve affichés deux fois, toutes les réserves
  inactives en plus, compteur de joueurs solo et compteur d'ennemis.
- **Modals surchargés** : le HUD reste autour des cartes et de l'écran de fin ;
  descriptions longues, résumé avec frames et score technique plutôt que durée.
- **Tailles fixes** : cartes pouvant sortir d'un petit viewport, textes multiline
  sans limite dans les coins.

La capture du runner masque déjà la ligne caméra et le debug de vagues : ceux-ci
sont examinés dans le code aussi. Le défaut caméra était activé en jeu natif.

## Première passe livrée

- Overlays frame/FPS, caméra, arme et inspecteur disponibles via `ALACOD_DEBUG_UI=1`,
  sans activation implicite par la feature native. F3 garde le debug de vagues.
- **HUD throne** : cinq textes permanents (étage, vie, niveau, arme, chargeur/réserve),
  deux barres avec piste de fond fixe ; ressources séparées dans les deux coins bas.
- Rads numériques, réserves inactives et compteurs joueurs/ennemis retirés du HUD
  courant ; toutes les valeurs restent dans le modèle pour les outils/scénarios.
- Effets temporaires : deux lignes et `+N effets` si besoin ; prompt seulement à portée.
  `· recharge` apparaît pendant le rechargement, sans remettre le compteur technique.
- Mutation/fin masquent le HUD ; cartes sur fond assombri, titre/aide plus courts,
  descriptions abrégées en gardant conditions, nombres et durées, police de texte augmentée.
- Fin : panneau centré, étage et éliminations, durée min:sec, deux boutons plus compacts.
- HUD/modals réduits quand le viewport est petit, à partir de la référence logique
  800×540. Capture `--size 640x360` pour le vérifier ; captures standard 960×540.

Aucune mécanique, graine, arme, mutation ou trace de simulation modifiée.
Les RON HUD de zombies/testbed/gungeon restent inchangés ; le nettoyage des overlays
techniques est partagé. Le compteur debug de vagues de zombies est optionnel au démarrage.
Le texte compact des cartes et le panneau de fin sont des composants de présentation partagés.

## Preuves à regarder

- `avant-combat.png` / `apres-combat.png` : même partie, f480.
- `avant-mutation.png` / `apres-mutation.png` : même partie, f600.
- `avant-vie-basse.png` / `apres-vie-basse.png` : même partie, f3240.
- `avant-fin.png` / `apres-fin.png` : même partie, f3600.
- `apres-mutation-640x360.png` : les trois cartes et l'aide restent dans le viewport.
- `apres-quad.png` : HUD sur vue à quatre bots ; caméra/lecture des autres joueurs
  restent selon le comportement existant, ce n'est pas un nouveau HUD individuel par joueur.
- `apres.mp4` : début de la partie humaine rejoué, captures à **4 images/s**. À utiliser
  pour juger la disposition et les textes, pas la fluidité de l'animation ou le FPS du jeu.

Reproduction :

```sh
cargo build -p scenario --profile headless --features render,game/debug_ui --bin play_scenario
# Copier l'original humain en /tmp et limiter frames à 3700, sans modifier l'original.
target/headless/play_scenario /tmp/human-02.ron --capture /tmp/hud --every 15
target/headless/play_scenario /tmp/human-02.ron --capture /tmp/hud-small --every 60 --size 640x360
```

`validation.json` : 112 tests game, 84 tests content unitaires, 86 fixtures et
2 smokes content verts ; neuf scénarios throne et deux zombies verts avec
`ALACOD_BLESS=0`, aucune trace modifiée. Les événements du rejeu humain avant/après
sont identiques frame par frame ; `replay-events.json` est joint. Builds natifs
throne/zombies verts ; le debug explicite est encore affichable (smoke à f60).

Le rejeu confirme aussi le résultat de la deuxième run : entrée au troisième étage
f2062, défaite observée f3393 (résumé de run à 3392 frames). Les constats humains
sur les bugs de gameplay restent à recueillir ; ce résultat ne valide pas M1 à lui seul.
