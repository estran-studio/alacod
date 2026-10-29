# polissage-feel — rapport S1-6 (voie b0, branche `polissage-feel`)

Fiche : `docs/taches/polissage-feel.md`, réduite à **zombies puis throne** (`plan-polissage` § 0). Base : `origin/polissage-integration`
(`ebab067a`, `main` + lot 0 + S1-1). Machine : Mac (M4 Max, écran intégré Liquid Retina XDR). Date : 2026-10-28. Conventions : § 94.

Entrée de William (2026-10-28, manette physique dans Chrome, build `e4-manette`) : « ça marche assez bien », mais **un peu trop
rigide**. Décision du coordinateur (2026-10-28, réponse à l'`ask`) : ni déplacement analogique ni changement d'accélération dans S1-6 ;
William rejuge la rigidité à la manette avec l'interpolation pendant l'E4 de vendredi ; l'analogique, s'il le veut, est une tâche à part.

## 1. Issue par point de la fiche

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Audit chiffré, vidéo « avant » | fait | § 2 (diagnostic), § 3 (tableau par clone) ; `feel_zombies.avant.mp4` |
| 2 | Interpolation d'affichage (`PostUpdate`, `Transform` seulement, coupée au téléport, à l'étage, au rollback) | fait ; mesurée à 60 Hz (la fenêtre native du Mac présente à 60 Hz, mesuré) ; 120 Hz : test unitaire, et à voir dans Chrome à l'E4 | § 4 ; `make test_scenarios_par` 811 scénarios verts sans `BLESS` ; `make schedule_order_diff` identique |
| 3 | Réglages par clone en données | fait pour la présentation (`feedback.ron`, `camera.ron`) ; `movement` inchangé (décision ci-dessus) | § 3, § 5 |
| 4 | Flash de bouche, recul visuel, effet de mort, par arme (`by_weapon`) | fait | § 5 ; `zombies_flash_eclats_f990-f1900.png`, `throne_flash_f14-f300.png` |
| 5 | Vidéo « après », jugement de William (course, dash, tir) | vidéo faite ; **jugement : non vérifié** (E4 de vendredi 30) | `feel_zombies.apres.mp4`, `feel_zombies.avant-apres.mp4` |

Preuves hors dépôt (sprites du jeu, vidéos lourdes), sur le Mac : **`~/orca/workspaces/alacod/captures/polissage-feel/`**. Vidéos
(60 s, 60 images/s, une image par frame simulée, rendu hors écran 960×540, sons coupés) : `feel_zombies.avant.mp4` (binaire de
`ebab067a`), `feel_zombies.apres.mp4` (branche), `feel_zombies.avant-apres.mp4` (côte à côte, avant à gauche). Elles montrent le flash, le recul et
les éclats ; **pas l'interpolation** : une capture à une image par frame simulée n'a rien à interpoler (elle est coupée en capture, § 4).
L'interpolation se prouve par la mesure (§ 4) et se juge à la main (E4).

Scénario de la vidéo et de la mesure : `tests/scenarios/feel_zombies.ron` (arène vide de zombies, 3600 frames : course dans les huit
directions, quatre dashes, mitraillette, fusil à pompe, esquive, pistolet, 17 zombies posés dont 16 tués). Sa trace
`feel_zombies.trace` est **neuve** (aucune trace existante n'a bougé).

## 2. Diagnostic de la rigidité (mesuré)

Outil : `ALACOD_PERF_CSV` écrit maintenant par image la position **rendue** du joueur local et de la caméra et l'`alpha` de
l'interpolation ; `scripts/frame-pacing.py` en tire la régularité du mouvement pendant la course (images où le joueur se déplace entre
40 et 400 px/s). Partie jouée en vrai temps, fenêtre native, vsync : `play_scenario tests/scenarios/feel_zombies.ron` (binaire
`--profile headless --features render`), deux passes sans et deux avec interpolation (`ALACOD_INTERPOLATION=0` pour « sans »).

| Passe | images/s | frames simulées par image 0 / 1 / 2 / 3+ | images en course | **figées** | **à-coups** | cv du pas | retard moyen |
|---|---|---|---|---|---|---|---|
| sans interpolation, passe 1 | 60 | 10 % / 80 % / 10 % / 0 % | 873 | **21 %** | **59 %** | 0,70 | 0 ms |
| sans interpolation, passe 2 | 60 | 16 % / 68 % / 16 % / 0 % | 872 | **8 %** | **24 %** | 0,43 | 0 ms |
| interpolation, `alpha` brut (1re version) | 61 | 3 % / 94 % / 3 % / 0 % | 870 | 2 % | 42 % | 0,34 | 6 ms |
| interpolation lissée, passe 1 | 60 | 3 % / 94 % / 3 % / 0 % | 868 | **1 %** | **13 %** | 0,35 | 10 ms |
| interpolation lissée, passe 2 | 60 | 21 % / 57 % / 21 % / 1 % | 872 | **1 %** | **12 %** | 0,27 | 10 ms |

« Figées » : le joueur ne bouge pas à l'écran pendant une image alors qu'il court. « À-coups » : le pas affiché s'écarte de plus de
0,5 px de la moyenne des deux images de part et d'autre (un pas de 0 ou de 5 px au milieu de pas de 2,5 px). Fichiers :
`~/orca/workspaces/alacod/captures/polissage-feel/pacing/{pacing_off,pacing_on,pacing_on2,final_off,final_on}.csv`.

Ce que ça dit :
1. **Affichage (la cause mesurée)** : la simulation avance à 60 Hz par un accumulateur de temps réel (`bevy_ggrs`) et l'écran présente
   à 60 Hz, mais l'intervalle de la boucle principale varie de **11 à 23 ms** d'une image à l'autre. Une image sur cinq à une sur trois
   montre alors **0 ou 2** frames simulées : sans interpolation, le joueur fige puis saute (8 à 21 % d'images figées en course, 24 à
   59 % d'à-coups selon la phase de la passe). Sur un écran à 120 Hz (Chrome sur cet écran ProMotion [INFERENCE : non mesuré ici, la
   fenêtre native est à 60 Hz]), chaque frame serait montrée deux fois : un à-coup une image sur deux. **Corrigé en présentation**
   (§ 4) : 1 % d'images figées, 12-13 % d'à-coups, au prix de 10 ms de retard moyen de l'image sur la simulation.
2. **Stick (non corrigé, décision)** : le stick gauche est numérisé en **huit directions** (seuil 0,4 par axe,
   `character::control::STICK_MOVE_THRESHOLD` : la diagonale couvre 24° à 66°, chaque axe pur 47°) et la vitesse est **tout ou rien**
   (0 ou 150 px/s, atteinte en 3 frames). Un stick à peine incliné lance le joueur à pleine vitesse en 50 ms ; un stick à 20° du
   cardinal le fait courir droit. Seul un déplacement analogique le règle : `BoxInput` gagne deux `i8` (format réseau changé, vieux
   clients incompatibles), `Segment`/`PlayerScript` (champ `move` optionnel), enregistrement, bots, chemin de course ; les traces ne
   restent identiques que si le chemin numérique reste bit à bit, sinon `BLESS` de tous les jeux. ≈ 1 j. **Pas fait** (réponse du
   coordinateur).
3. **Accélération** : 3000 px/s² en accélération et en freinage (150 px/s en 3 frames, arrêt en 3, demi-tour en 6) : nerveux par
   choix (movement-feel, 2026-10-04 : « on patine » à 150 px/s²), validé par William au clavier. La baisser adoucirait aussi le
   clavier et bougerait les traces de zombies **et** de throne (le `pilote` a les mêmes valeurs). **Inchangée** (décision).
4. **Caméra** : `lerp_speed` 12, lissage exponentiel indépendant de la fréquence (63 % de l'écart en 83 ms) ; elle suit le
   `Transform` du joueur, donc maintenant sa position interpolée. Rien à corriger mesuré : l'écart à l'écran joueur−caméra varie de
   1,60 px par image sans interpolation à 1,21 px avec (passes finales).

## 3. Valeurs effectives par clone

Mouvement (personnages joueurs, `ZombieShooter/Sprites/Character/player_config.ron` et `throne/characters/pilote.ron`), feedback
(`ui/feedback.ron`), caméra (`camera.ron`). En gras, ce que S1-6 ajoute.

| Valeur | zombies | throne | écart et motif |
|---|---|---|---|
| course : accélération / freinage | 3000 / 3000 px/s² (vitesse max en 3 frames, arrêt en 3, demi-tour en 6) | idem | aucun ; inchangé (§ 2.3) |
| vitesse max ; sprint | 150 px/s ; ×2 (+0,1 par frame, −0,2) | idem | aucun |
| dash | 64 px en 8 frames, cooldown 24, i-frames 6, appui gardé 8 | idem | aucun |
| knockback | amortissement 0,85 par frame (ressource du moteur, `KnockbackDampingConfig`, même partout) | idem | moteur |
| stick | 8 directions, seuil 0,4 par axe ; visée 0,3 ; gâchette 0,5 / 0,35 (S1-1) | idem | moteur |
| flash à l'impact | 4 frames, ×2,5 | idem | aucun |
| secousse | 8 frames, amplitude 12 | idem ; explosifs 16 / 20 (`by_weapon`) | throne a des explosifs |
| hit stop | 2 frames | 2 ; explosifs 6, fusil à pompe 4 | idem |
| caméra | `lerp_speed` 12, `min_zoom` 0,6 | idem, fondu d'étage 0,4 s | throne a des étages |
| **interpolation d'affichage** | **activée, coupure à 48 px** (`camera.ron`) | **idem** | aucun |
| **flash de bouche** (frames, longueur px) | **3 / 10 ; mitraillette 2 / 9 ; fusil à pompe 4 / 16** (orange) | **3 / 12 ; mitraillette 2 / 9 ; fusil à pompe 4 / 18 ; lance-grenades et roquette 5 / 18 ; laser bleu, plasma vert** | zombies : on mitraille longtemps une horde, tir sec et lisible ; throne : arsenal varié et plus lourd, chaque arme se reconnaît |
| **recul visuel de l'arme** (px, frames) | **2 / 6 ; mitraillette 1,5 / 4 ; fusil à pompe 4 / 10** | **3 / 6 ; fusil à pompe 5 / 10 ; explosifs 6 / 12 ; mitraillette 1,5 / 4 ; laser 1 / 4 ; plasma 4 / 8** | idem |
| **éclats de mort** | **8 éclats rouge sombre de 3 px** | **10 éclats verts de 3 px** | zombies saignent ; mutants de throne |
| ennemis (vitesse max lue par l'IA) | `zombie_1` 120, `zombie_full` 80, `zombie_hard` 140, boss 55 px/s | 40 à 90 px/s (`rat` 60, `chien` 90, `brute` 40…) | contenu, non touché |

Le champ `recoil` des modes de tir de `weapons.ron` (ex. pistolet 3, mitraillette 1,5) existe et entre dans le hash de `Weapon`, mais
**aucun système ne le lit** : il n'a pas été réutilisé pour le recul visuel (donnée de gameplay, hors présentation) ; le recul visuel vit
dans `feedback.ron`.

## 4. Interpolation d'affichage

`crates/game/src/display_interpolation.rs`, `DisplayInterpolationPlugin` dans `PresentationPlugin` (absent en headless) :
- `advance_interpolation_clock` puis `interpolate_transforms`, chaînés en `PostUpdate`, après `sync_bevy_transforms_from_fixed` et
  avant la propagation ; x/y seulement, rotation et échelle telles que simulées.
- Horloge : recopie de l'accumulateur privé de `bevy_ggrs` (un pas retiré par frame simulée observée, borné à `[0, pas]`), puis
  `alpha` lissé (delta en moyenne glissante, rappel de 10 % vers l'accumulateur) : sans lissage, le pas affiché variait de 1,6 à 3,5 px
  pour 2,5 px par frame (ligne « `alpha` brut » du tableau § 2, 42 % d'à-coups).
- Coupures : téléport (> 48 px en une frame), retour de frame (nouvelle partie), rollback d'une partie en ligne (`note_rollback` dans
  `LoadWorld`, ajouté par ce plugin : absent de l'app headless, donc de `schedule_order_diff`), capture de `play_scenario`.
- Tests : `display_interpolation::tests` (120 Hz : moitiés de frame ; delta irrégulier 12 / 21,3 ms : pas régulier à 0,15 frame près ;
  session qui attend ; nouvelle partie ; téléport). Rien n'est rollback : `make test_scenarios_par` **811 scénarios verts sans
  `BLESS`** (414 s, 4 tranches, `target/metrics/d9467975`), `make schedule_order_diff BASE=origin/polissage-integration` :
  « OK : ordre des systèmes et des types rollback identiques (4550 lignes de schedules, 645 types) ».

## 5. Flash de bouche, recul, éclats de mort

`crates/game/src/feedback_shots.rs` (`ShotFeedbackPlugin`, ajouté par `feedback::FeedbackPlugin`, rendu seulement), types et lint dans
`crates/content/src/feedback.rs` (`MuzzleFlashConfig`, `RecoilConfig`, `DeathFxConfig`, `FeedbackSettings::resolve_shot`,
`check_shot`, `check_death`). Dérivés de l'état : `Added<Bullet>` (un flash et un recul par tireur et frame de création, comme le son de
tir), disparition d'un ennemi d'une image à l'autre (comme le son `death`). Le recul ne déplace que le `Transform` du sprite de l'arme,
réécrit à chaque image : la balle part droit. Sans les champs, rien ne change (gungeon, isaac, testbed). Les clés de sons de
`feedback.rs` (S1-7, b2) ne sont pas touchées ; `crates/content/src/feedback.rs` gagne trois champs hors sons (dit au coordinateur).
Tests : `feedback_shots::tests` (durée et retour en arrière, sens et retour du recul).

## 6. Vérifications

- `cargo test -p game --lib -- display_interpolation feedback_shots` : 7 tests verts.
- `make test_scenarios_par SHARDS=4` : 811 scénarios verts, sans `BLESS` (la trace neuve `feel_zombies` écrite une fois avec
  `ALACOD_SCENARIO=feel_zombies ALACOD_BLESS=1`, puis rejouée verte).
- `make schedule_order_diff BASE=origin/polissage-integration` : identique.
- Capture de throne (`throne_mutation_choice`, f14-f300) : flash de la mitraillette visible ; mort f163.
- `make -k check` (Mac, 2026-10-28, 23 min) : vert sauf `test_tools`, dont les trois échecs sont ceux du Mac connus
  (`test_claude_say`, voix système), sans lien avec ce lot. Dernières lignes utiles :

```
Ran 38 tests in 2.152s
FAILED (failures=3)                       # test_claude_say (voix du Mac), connu
make: *** [test_tools] Error 1
OK: aucun nouveau motif interdit ; 51 occurrences auditées dans determinism-allowlist.json
OK: aucun appel direct à rollback_component_with*/rollback_resource_with* hors de crates/utils/src/rollback.rs
OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69
(test_crates : aucune ligne « FAILED »)
test scenarios ... ok
test result: ok. 5 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 992.24s
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)
games/throne : aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes)
```

## 7. E4 de S1-1 (vendredi 30) : préparation

Le build `e4-feel` (cette branche, zombies et throne ; `WEB_BUILD_ID=e4-feel WEB_OPTIMIZE=0 WEB_ENABLE_SOLO=1
WASM_BINDGEN=/private/tmp/wbg/wasm-bindgen-0.2.129-aarch64-apple-darwin/wasm-bindgen node scripts/build-web-games.mjs zombies throne`,
puis `git checkout website/static/releases.json`) s'ajoute à `e4-manette` sur le serveur de S1-1 (`127.0.0.1:8777`, PID 22332,
vivant le 2026-10-28 au soir). William joue l'E4 de S1-1 (protocole `polissage-manette.md` § 7) sur `e4-feel`, et compare la rigidité
avec `e4-manette` (même partie, sans interpolation ni flash) :
- zombies : `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-feel` (avant : `build=e4-manette`) ;
- throne : `http://127.0.0.1:8777/loader.html?game=throne&build=e4-feel`.
Démarrage vérifié dans un Chromium sans écran (module wasm chargé, canevas 1600×900, journaux du jeu) ; le rendu et la manette ne
se vérifient que dans le vrai Chrome de William. Résultat à consigner dans `plan-polissage-notes.md` et dans une section « E4 » de
`polissage-manette.md`.

## 8. Reste

- Jugement de William sur course, dash et tir (« agréable » ou pas), et sur la rigidité à la manette avec l'interpolation, à l'E4 de
  vendredi 30. S'il trouve encore le stick rigide : déplacement analogique, tâche à part (≈ 1 j, § 2.2).
- Mesure à 120 Hz : la fenêtre native du Mac présente à 60 Hz ; Chrome sur l'écran ProMotion est à mesurer (S1-5 : `ALACOD_PERF_CSV`
  est natif seulement, à instrumenter en wasm).
- Lot 2 (traînées, douilles) : renvoyé en P8.
- Aucune dette ouverte.
