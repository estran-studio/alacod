# Revue d'une partie gungeon complète jouée par 2 bots `explorateur`

Local, non commité. Base : main @ 3bab4764 (aucun code modifié).

## Partie filmée

- Graine **10**, séquence `etages_deux`, 2 × `explorateur`, issue **victoire** à f9108 (≈ 152 s à 60 Hz).
- Seule victoire sur 20 graines (1..20) ; les 19 autres sont des `Defeat` entre f539 et f3535, tous à étage 0.
  (Le JSON dit `morts 1/2` dans 18 cas sur 19 : la défaite est déclarée avec un seul joueur « mort », l'autre à terre ? à confirmer.)
- Étage 1 : 9 salles (6 verrouillantes), portail f3122, passage f3272. Étage 2 : 12 salles (9 verrouillantes), boss nettoyé f9108.
- Dégâts subis : 1228. Mises à terre : 8 (toutes le joueur 0), 8 réanimations. Blanks utilisés : 5 (tous avant f762).
- Fichiers (local) :
  - scénario : `tests/scenarios/revue_gungeon_bots.ron` (graine 10, non commité) ; sim JSON : `/tmp/gungeon-bots/sim.json`
  - images : `target/videos/3bab4764/frames/revue_gungeon_bots/frame_NNNNN.png` (4555 images, 1 sur 2) + `events.json`
  - vue joueur 0 : `.../frames/revue_gungeon_bots.p0/` (identique à la vue principale) ; **vue joueur 1 non capturée** (voir « Outils »).
  - vidéo **sans texte incrusté** : `/tmp/gungeon-bots/revue_gungeon_bots.sans-texte.mp4` (encodée à la main, voir « Outils »).

## Constats

1. **Échelle / caméra (f70, f300, f1000, f3500)** : le joueur fait ≈ 50 px sur 960×540 ; l'écran montre environ 19×11 cases. Une salle de combat ne tient pas dans le cadre : les ennemis arrivent par le haut/bords (f70 : ≈ 17 ennemis dont plusieurs coupés par le bord). Pour un bullet hell, la part visible est faible. Suggestion : caméra (`camera.ron`) dézoomée ou cadrage par salle verrouillée (cadre fixe sur la salle), à essayer avant de toucher aux sprites.
2. **Balles ennemies illisibles (f70, f3500, f5300)** : les projectiles ennemis sont de petits points rouges d'environ 3 px, à peine distincts du sol ; le chiffre de dégâts et les sprites les couvrent. Les télégraphes (248 événements) ne se voient presque pas hors f1566/f5300 (grands cercles rouges qui sortent du cadre). Suggestion : contenu RON (taille et couleur des projectiles, contour clair), et rayon de télégraphe borné au cadre.
3. **Boss trop vite tués (f2943→3121, f9000→9108)** : 3,0 s puis 1,8 s de salle de boss ; phases 0→1→2 à f3050/f3101 et f9048/f9088, soit 51 et 40 frames entre phases. On ne voit aucune phase. Les points de vie du boss sont à revoir (RON), ou les dégâts des bots (munitions affichées `119988` : réserve quasi infinie).
3b. **Boss peu imposant (f3050, f9048, f9088)** : sprite ≈ 95 px, soit 2 × le joueur ; le joueur se tient *dans* le sprite du boss (superposition, aucune distance). Salle de boss étroite (≈ 214 px de large visibles). Suggestion : boss plus grand, salle plus large (contenu).
4. **Carrés rouges avec pictogramme d'arme (f0, f3280)** : au chargement de l'étage 2, de grands carrés rouges (≈ 55 px) marqués d'un petit pistolet occupent des salles ; ils sont aussi présents sur la vue d'ensemble de f0. Ressemblent à des marqueurs de spawner/placeholder visibles en jeu, ou à des sprites manquants. À vérifier : ils disparaissent quand les ennemis naissent (absents à f70). Suggestion : dette moteur (masquer en jeu) si ce sont des marqueurs LDtk.
5. **Frame 0 = carte entière en zoom arrière** (f0) avant que la caméra se règle : une image de saut de caméra au début.
6. **Rythme** : étage 1 = 3272 frames (54 s) ; étage 2 = 5837 frames (97 s). Salles de combat nettoyées en 120 à 450 frames. Au centre de l'étage 2 (f4074→7186) on compte 7 mises à terre pour 6 salles, soit un cycle « à terre / réanimation » de 300 à 450 frames (≈ 5 à 7 s) qui domine la durée ; le joueur 1 reste seul à tenir la salle.
7. **Difficulté** : 19 défaites sur 20, 18 avant f3400, toutes à l'étage 0 ; 5 graines (1, 3, 7, 11, 14) tombent entre f539 et f811 (où, non vérifié : seules les images de la graine 10 ont été regardées). Sur la graine 10 : premier verrouillage f42, 17 ennemis d'un coup à f70, 2 blanks consommés dès f70 et f82. Cohérent avec D64. Dans la partie filmée, le joueur 0 tombe 8 fois, le joueur 1 jamais (le joueur 0 est presque toujours celui qui attire le feu).
8. **Économie inutilisée** : « Douilles » passe de 520 (f70) à 3293 (f9088) sans une dépense ; `Clés 0` et `Blanks 0` presque tout le long ; un seul objet ramassé de toute la partie (blank, f565). Aucun événement boutique/coffre. Les coffres (f0, f3050, f9048) restent fermés (pas de clé). La boutique et les coffres ne sont donc pas exercés par le bot, la revue ne peut pas juger leur lisibilité. Suggestion : donner au bot l'achat (déjà `acheteur` en zombies) ou un test dédié.
9. **HUD** : le HUD n'affiche que la santé du joueur suivi (82/100…) ; l'autre joueur n'a pas de barre : impossible de voir qu'il est à terre. La mini-carte (haut droite) couvre des ennemis (f3500 : deux zombies sous la mini-carte). Santé en barre dorée/rouge à bas niveau lisible (bon point), vignette de dégâts f1566 lisible.
10. **Murs/entités** : aucun sprite dans un mur repéré dans les 20 frames regardées ; les trous noirs (carrés noirs 50 px) sont lisibles. Pas de contrôle systématique des 4555 images.

## Outils (erreurs exactes)

- `alacod-sim` : la commande de la consigne échoue sans condition d'arrêt : `alacod-sim : --until-wave <n> (Waves) ou --until-floor <n> (Floors) obligatoire`. J'ai ajouté `--until-floor 99` (jamais atteint : la fin de partie arrête la graine). Le reste est identique.
- Sous zsh, `$(./scripts/dev-features.sh features)` non cité ne se découpe pas : `error: unexpected argument '--features game/dev-dylib'`. Contournement de ma ligne de commande : `${=DEV}`. `make sim` n'est pas touché.
- `make videos` : `dyld: Library not loaded: @rpath/libstd-c36b5870a20e184c.dylib ... no LC_RPATH's found` (code 134). Cause : `build_player` (`scripts/scenario-video`) exporte `DYLD_LIBRARY_PATH` dans une substitution `$(...)` donc perdu, et sous macOS SIP retire `DYLD_*` à travers `make` et `/usr/bin/env`. Contournement : variable posée dans l'environnement et script lancé avec `/opt/homebrew/bin/bash scripts/scenario-video render ...` (pas de code modifié). **Bug d'outillage macOS à corriger.**
- Encodage : `[AVFilterGraph] No such filter: 'drawtext'` : le ffmpeg Homebrew installé n'a pas freetype. Aucun `.mp4` produit par le script, ni de montage ; `make views` capture la vue du joueur 0 puis s'arrête (`exit=8`), **la vue du joueur 1 n'est pas capturée**. L'encodage manuel est `ffmpeg -framerate 30 -pattern_type glob -i 'frames/revue_gungeon_bots/frame_*.png' -c:v libx264 -crf 20 -pix_fmt yuv420p` (sans la ligne de texte « À regarder »). `make review_videos` non lancé (aucune vidéo dans `target/videos/`).
- `tests/scenarios/revue_gungeon_bots.ron` est dans l'arbre de travail (non suivi) : un `make test_scenarios` complet le jouerait sans trace de référence ; le supprimer avant.

## Résumé (5 lignes pour l'orchestrateur)

1. gungeon `etages_deux`, 2 × explorateur, 20 graines : 1 victoire (graine 10, 9108 frames ≈ 152 s), 19 défaites (f539 à f3535, étage 0) ; dégâts 1228, 8 mises à terre sur la partie filmée.
2. Échelle : joueur ≈ 50 px sur 960×540, salle non visible en entier, balles ennemies ≈ 3 px quasi invisibles : régler la caméra (cadrage de salle) et les projectiles (RON) avant d'ajouter du bullet hell.
3. Boss : morts en 3,0 s et 1,8 s (phases à 40–50 frames d'écart), sprite 2 × le joueur, le joueur le recouvre : revoir PV/taille du boss et de sa salle.
4. À vérifier : carrés rouges à pictogramme d'arme visibles en jeu (f0, f3280), HUD sans la santé du second joueur, mini-carte sur les ennemis ; boutique, coffres et clés non exercés (3293 douilles non dépensées).
5. Outillage macOS : `scenario-video` échoue (DYLD perdu dans le sous-shell/SIP, ffmpeg sans `drawtext`) ; images et mp4 sans texte livrés à la main, vue du joueur 1 non capturée.
