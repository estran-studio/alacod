# X-4b — Sprites : import (lot 1, VPS)

Base `origin/main` `00dc83ce`, branche `wquintal/x-4b-import-sprite`. Fiche : `docs/taches/plan-1837-cli.md` (lot X-4b, décision du 2026-10-27),
`docs/taches/rapports/x-4b-arret.md`. Convention : `docs/conventions.md` §91. **Simulation intouchée** : aucun type rollback ajouté ou
modifié, aucun système `GgrsSchedule` ajouté, aucune trace existante réécrite, aucun BLESS. Le brouillon `x-4b-brouillon-sur-x2a` (`1beaa90b`) n'a servi que de
matière lue à la main (jamais rebasé, fusionné ni cherry-pické) ; il n'avait jamais été compilé : son insertion dans `sprites.ron` cherchait le mauvais
délimiteur (`}` puis `)` au lieu de `})`) et son remplacement perdait la clé de l'entrée, deux défauts corrigés et couverts par des tests.

## Décision D-X4b-1 (appliquée)

Un atlas réduit composé à l'import, une entrée `sprites.ron` par personnage : chaque planche est réduite cellule par cellule (alpha prémultiplié, Lanczos3,
déterministe), chaque clip devient un bloc de colonnes × lignes de directions dans **un seul PNG** de côté ≤ 4096 (`--max-side`). Une `SpriteSheetConfig`
(calque `body`, `directions` dans l'ordre du manifeste), une `AnimationMapConfig` (chaque clip en `Direct` sur la première ligne de son bloc, états
`--state Idle=idle --state Run=marche --state Shoot=epauler` en plus, **aucun clip abandonné**). Détail, formules et limites : §91.

- **Durée et boucle par clip** : `animation::AnimationClipMeta` (`frame_duration`, `looping`), facultatif et rétrocompatible ; les cartes existantes n'ont pas de `clips`
  et `animate_sprite_system` n'écrit alors pas le timer. Test : `map_without_clips_deserializes_and_keeps_global_timing`.
- **`Shoot`** : état de **présentation**. `animation::PresentationClip` (non rollback) ; `game::character::visuals::present_shooting` (`Update`) le pose quand l'arme
  active a tiré depuis moins de 20 frames (`WeaponState::last_fire_frame`, `FrameCount`, existants). `AnimationState` n'est pas touché ; sans clé `Shoot`
  dans la carte rien ne change (aucune carte existante n'en définit : vérifié par recherche dans `crates/` et `games/`, seul `Behavior::Shoot` des ennemis
  porte ce nom, sans rapport avec les cartes d'animation).
- **Échelle et pivot** : `scale = (U / ppm) / facteur` ; pivot du manifeste → `anchor: Center` + `offset_x/offset_y` en unités monde ; **le pivot du manifeste est aux pieds** (point de contact au sol, correction du 2026-10-28 :
  la première version disait « à mi-hauteur »), posé par `--pivot-at X,Y` (unités locales de l'entité, défaut `0,0`). La hauteur de silhouette est mesurée et reportée.

## Preuve du lot 1 : le vrai fantassin dans le testbed (commit de tête, retirable)

Tout ce qui vient du fantassin est dans **un seul commit en tête de branche** (« X-4b : fantassin-de-ligne importé dans le testbed (retirable) ») : dossier `games/testbed/assets/sprites/fantassin-de-ligne/`, entrée de `sprites.ron`, `characters/fantassin-de-ligne.ron`, entrée d'`assets.yaml`, scénario `sprite_import_fantassin` et sa trace. Si William répond « import local non commité », ce commit se retire sans toucher l'outil, les tests synthétiques ni la doc ; les commits d'en dessous sont verts sans lui (vérifié, voir plus bas).

Manifeste `origin/main` `9cc19d7` de `1867_lore` (lu, jamais modifié) : cellule 376×372, `ppm` 91, pivot (186,72 ; 196,34), dix clips de 8×8 cases (3008×2976 px).
Commande et sortie (première étape avec U provisoire = 30 : silhouette de 143 px source → 47,1 unités ; U = 32 × 91 / 143 ≈ 20,4 donne **32,1 unités, deux cases**, comme la boussole) :

```text
$ alacod import-sprite fantassin-de-ligne.sprite.json games/testbed --id fantassin-de-ligne --units-per-meter 20.4 --state Idle=idle --state Run=marche --state Shoot=epauler --overwrite
import fantassin-de-ligne : facteur 0.3441, cellule 129×128 px (source 376×372)
atlas 3096×4096 px : 3×4 blocs, 24 colonnes × 32 lignes de cases, 10 clips
échelle 0.65151095 ; pivot aux pieds (point de contact au sol) posé en (0, 0) unités locales (--pivot-at) : offset (0.28694504, 2.317978) unités
silhouette de « idle » : 143 px source, 49.2 px d'atlas, 32.1 unités affichées (réglez --units-per-meter)
```

La suite de la sortie liste les dix clips (images, durée, boucle, états : `idle` 16..=23 250 ms boucle [Idle], `marche` 200..=207 125 ms boucle [Run], `epauler` 8..=15
125 ms une fois [Shoot], etc.) puis les quatre fichiers écrits. Atlas de 2,1 Mo sur disque, 3096×4096 px (≈ 50 Mo en mémoire vidéo, au lieu de ≈ 360 Mo pour dix planches).

Diff de `games/testbed/assets/sprites/sprites.ron` :

```diff
@@ -86,4 +86,8 @@
         animation: "sprites/effects/slash/slash_animation.ron",
         layers: { "body": "sprites/effects/slash/slash_sheet.ron" },
     ),
+    "fantassin-de-ligne": (
+        animation: "sprites/fantassin-de-ligne/fantassin-de-ligne_animation.ron",
+        layers: { "body": "sprites/fantassin-de-ligne/fantassin-de-ligne_sheet.ron" },
+    ),
 })
```

`alacod lint games/testbed` : aucune erreur (41 personnages) ; `characters/fantassin-de-ligne.ron` copie `boussole.ron` avec `asset_name_ref: "fantassin-de-ligne"` et la
mitrailleuse du testbed (tir tenu). `assets.yaml` : licence et auteur « **à confirmer par William** » (question ouverte du coordinateur, rien d'inventé).

Réimport de la revue (commit B, pivot aux pieds posé au bas du collider ; seul `fantassin-de-ligne_sheet.ron` change, le PNG et l'animation sont identiques) :

```text
$ alacod import-sprite fantassin-de-ligne.sprite.json games/testbed --id fantassin-de-ligne --units-per-meter 20.4 --pivot-at 0,-13 --state Idle=idle --state Run=marche --state Shoot=epauler --overwrite
échelle 0.65151095 ; pivot aux pieds (point de contact au sol) posé en (0, -13) unités locales (--pivot-at) : offset (0.28694504, -10.682022) unités
```

## Scénario du lot 2 (Mac)

`tests/scenarios/sprite_import_fantassin.ron` : `Idle` (visée seule), `Run` (déplacement + visée) puis `Shoot` (Fire tenu 66 frames, pause de 30) dans les huit directions, 740 frames, commentaire `// À regarder :` avec les frames. Le runner **exige** une trace de référence (« pas de trace de référence »), donc la seule trace neuve,
`tests/scenarios/sprite_import_fantassin.trace`, est écrite ici, sur le VPS, par le seul BLESS permis :

```sh
BLESS=1 SCENARIO=sprite_import_fantassin make test_scenarios
```

(740 frames, joueur vivant, invariants tenus). Les traces étant identiques entre x86-64 et arm64 (§57), le Mac la relit sans la réécrire ; `git status` ne montre que ce fichier de trace nouveau.

Capture réelle (une image par direction et par pose : idle f35…245, run f275…485, shoot f552…1434 : `epauler` joué en entier et tenu, 62 frames après le début du tir de chaque direction ; frames du retour à `Idle` : section « Revue indépendante ») :

```sh
CARGO_BUILD_JOBS=4 cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/sprite_import_fantassin.ron \
  --capture target/videos/x-4b-import-sprite --at 35,65,95,125,155,185,215,245,275,305,335,365,395,425,455,485,552,678,804,930,1056,1182,1308,1434 --size 960x540
# mosaïque 8 colonnes × 3 lignes (idle, run, shoot) ; ajuster le motif aux noms d'images produits
ffmpeg -y -pattern_type glob -i 'target/videos/x-4b-import-sprite/*.png' -vf "scale=480:-1,tile=8x3" -frames:v 1 docs/taches/rapports/x-4b-import-sprite.png
```

À vérifier à l'écran : silhouette de ≈ 2 cases (32 unités), pivot aux pieds (voir le constat 7 de la revue : `--pivot-at 0,-13`), huit lignes
distinctes, `marche` jambes animées, `epauler` joué une fois puis tenu.

## Dettes

- **D156** mains et ancres des manifestes non importées (design composable non tranché).
- **D157** clips sans état de simulation (`touche`, `mort`, `roulade`, `rechargement`, `baionnette`, `idle2`, `victoire`) : jouables par leur nom, rien ne les choisit.

## Vérifications

- `env -u CI CARGO_BUILD_JOBS=4 cargo test -p content --test sprite_import` : 15 verts (1/4/5/8 directions, correspondance atlas/ligne dont un bloc hors colonne 0, réduction et couleurs,
  déterminisme à l'octet près, échelle/pivot/silhouette, états, refus sans artefact : lignes, direction, taille de PNG, manifeste mal formé, PNG absent, atlas impossible ; lint du jeu généré, fusion de table, réimport) ; 6 unitaires du module.
- `cargo test -p animation` : 11 verts (rétrocompatibilité sans `clips`, rythme par clip, `advance_frame`, clé de présentation, `Shoot` joué une fois puis tenu, sans clé rien ne change).
- `cargo test -p game --lib -- shoot_pose` : 2 verts.
- `make test_tools` : 38 tests verts.
- `make test_scenarios_par` (811 scénarios, sans BLESS) : vert ; `git status` ne montre **aucune trace existante modifiée**, seulement la trace neuve `sprite_import_fantassin.trace`.
- `make check` : vert (`cargo fmt` a aussi remis sur une ligne une constante de `crates/content/tests/lint_fixtures.rs`, dérive de format déjà présente sur `main` depuis X-4a).
- `make check_sim_boundary` (dans `make check`) : vert, `content` reste `SIM` (la crate `image` n'est prise qu'avec la feature `png`).
- **Sans le fantassin** (avant-dernier commit, même arbre de travail sans le commit de tête, un seul cargo) : `alacod lint games/testbed` sans erreur (40 personnages) et `cargo test -p content --test sprite_import` : 15 verts.

## Lot 2 — capture Mac

Base `a878966c` (arm64, macOS, écran déverrouillé ; aucun code ni donnée touchés, aucun BLESS).

- **Trace** : `env -u CI CARGO_BUILD_JOBS=4 SCENARIO=sprite_import_fantassin make test_scenarios` (sans BLESS) : `sprite_import_fantassin` vert, 740 frames, joueur vivant ; `git status` sans aucune trace modifiée (la trace x86-64 du VPS est relue à l'identique sur arm64).
- **Capture** (24 images 960×540, luminance moyenne 70–87 sur chacune, sprite visible sur toutes, aucune image noire) :

```sh
env -u CI CARGO_BUILD_JOBS=4 ALACOD_MUTE=1 cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/sprite_import_fantassin.ron \
  --capture target/videos/x-4b-import-sprite --at 35,65,95,125,155,185,215,245,275,305,335,365,395,425,455,485,504,534,564,594,624,654,684,714 --size 960x540
ffmpeg -y -pattern_type glob -i 'target/videos/x-4b-import-sprite/frame_*.png' -vf "scale=480:-1,tile=8x3" -frames:v 1 docs/taches/rapports/x-4b-import-sprite.png
```

Mosaïque : `docs/taches/rapports/x-4b-import-sprite.png` (8×3, lignes idle, run, shoot ; colonnes E, NE, N, NW, W, SW, S, SE). Pour juger le temps (marche, `epauler`), 20 images de plus ont été prises à part
(`--at 435…463` et `490…560`, non versées) : mêmes commande et scénario.

Constat (échelle mesurée : salle de 416 unités = 693 px, soit 1,667 px/unité, une case de 16 unités = 26,7 px) :

1. **Hauteur** : bonnet à poil au bas des bottes ≈ 52 px, soit ≈ 31 unités ≈ 1,95 case : conforme aux ≈ 2 cases (32 unités). Le sprite est plus étroit que haut (≈ 19 unités de large de face).
2. **Pieds / pivot** : le pivot **n'est pas** à mi-hauteur du dessin autour du centre du joueur. Le centre du joueur (case de départ `[160,80]`, centre à l'écran (480,270)) est ≈ 12,5 unités **sous** le milieu du sprite : le haut du bonnet est ≈ 28 unités au-dessus du centre, les bottes ≈ 3 unités sous le centre, donc **dans** la case (≈ 5 unités au-dessus de son bord bas), pas « un peu sous la case ». Les pieds restent ≈ 13 unités au-dessus du bas du collider (20×20, `offset_y` −6). Aucun `offset_y` n'a été touché.
3. **Huit directions** : huit images distinctes à l'idle, au pas et au tir ; de dos au N, de face au S, profils E/W, trois-quarts aux diagonales, NE/NW et SE/SW bien différents. Écart : l'arme (sprite d'arme du jeu, pas de l'atlas) est dessinée à part, **détachée sous les pieds** (horizontale à l'E/W, verticale au N/S, diagonale ailleurs) et jamais dans les mains du fantassin.
4. **Marche** : `marche` est animée (S, f435 → f463 : jambes qui s'alternent, buste qui bouge, pied avant puis arrière) ; à chaque direction du run la pose diffère de l'idle (jambes écartées contre jambes jointes).
5. **`epauler`** : joué une fois puis tenu. À l'E, f490 course, f498-506 le bras se lève, f510-522 trois images identiques (pose tenue, plus de mouvement) ; même chose aux autres directions (f534, f564… bras tendu). Écart : la pose reste tenue au moins 8 frames après la fin du Fire (f514 → f522 encore tenue) ; le retour à `Idle` n'a pas été observé image par image (aux directions S et SE, f684/f714, la pose se confond avec l'idle à cette échelle, sans bras visible).

Commit de la mosaïque et du constat : `2e5d6772` (cette ligne ajoutée par le commit suivant).

## Revue indépendante (2026-10-28)

Commits revus : `f7f04e58` (outil, présentation, tests, doc) et `a878966c` (fantassin, retirable) ; capture Mac du lot 2 : `23337437`, `d74cb17e`. Sept constats (six de la revue,
un de la capture Mac), tous corrigés ; chacun a un test, vérifié **en échec sur l'ancien code** (les tests de `content` et celui d'`animation` ont été rejoués contre le code d'avant).
Les correctifs s'ajoutent **au-dessus** de `d74cb17e`, sans réécrire d'historique poussé, en deux commits : **A** « X-4b revue : import transactionnel, table et manifeste analysés,
clés échappées, durée d'une seconde, `--pivot-at` » (outil, tests, doc : aucun fichier du fantassin) et **B** « X-4b revue : fantassin (scénario epauler par direction, pivot aux pieds,
trace neuve) — retirable » (seulement `tests/scenarios/sprite_import_fantassin.ron` et `.trace` et les fichiers de `games/testbed/assets/sprites/fantassin-de-ligne/`). Leurs SHA dépendent de ce rapport :
ils sont dans le message de fin de tâche et par `git log`. **Retirer le fantassin = `git revert` de `a878966c` et de B**, sans toucher à l'outil, aux tests synthétiques, à la doc ni à la capture.
Aucune dette nouvelle (rien n'est reporté).

| # | Constat | Correctif | Test |
|---|---|---|---|
| 1 P1 | `std::fs::write` tronquait les destinations ; un échec avec `--overwrite` supprimait d'anciens fichiers au lieu de les restaurer, `sprites.ron` pouvait rester tronqué | `commit_files` : tout est écrit en fichiers temporaires du même dossier, l'ancien fichier est mis de côté puis le nouveau renommé (`sprites.ron` en dernier) ; échec = renommages inverses, temporaires et dossiers créés retirés, état d'avant intact octet pour octet | `failed_overwrite_leaves_the_previous_import_intact_byte_for_byte` (panne simulée à chacun des 4 pas, comparaison des octets), `failed_first_import_leaves_no_file_and_no_directory`, `unwritable_table_directory_fails_before_touching_anything` (dossier en lecture seule réel) |
| 2 P2 | `check_animation_config_reload_system` prenait toute durée d'une seconde pour « non initialisé » : un clip à 1 fps était remis à la durée globale à chaque mise à jour et n'avançait jamais | vrai marqueur `AnimationTimer::configured`, posé par le rechargement et par `animate_sprite_system` quand il écrit la durée du clip | `a_one_second_clip_advances_with_both_systems` (les deux systèmes, configuration chargée par l'`AssetServer`, 600 ms par mise à jour) |
| 3 P2 | `entry_span` trouvait `"id": (` dans un commentaire | analyse de la table (`scan_table`) : commentaires, chaînes et valeurs imbriquées sautés, premier niveau seulement ; la table produite est relue (`verify_table`) avant écriture | `commented_entries_are_not_mistaken_for_real_ones` |
| 4 P2 | la table `SpriteSheet` était cherchée par `path:` dans les 400 octets avant `"SpriteSheet"` (ordre des champs, coupure UTF-8) | `GameManifest::parse` : première déclaration `kind: "SpriteSheet"` des `content_folders` | `sprite_sheet_declaration_is_read_by_the_manifest_parser` (`(kind, path)` inversés, déclaration avant, accents) |
| 5 P2 | une barre oblique inverse dans un nom de clip ou d'état donnait une échappée invalide ou une autre clé | chaque clé écrite passe par la sérialisation RON (`ron_key`) ; feuille, animation et table sont relues avant écriture (refus sinon) | `backslash_in_clip_and_state_names_is_escaped_in_ron` |
| 6 P2 | le scénario ne laissait jamais retomber la pose `Shoot` (24 frames de tir, 6 de pause, pose tenue 20 frames, `epauler` = 60 frames) | **B** : 66 frames de tir par direction (`epauler` joué en entier puis tenu), **pause de 60 frames** (≥ `SHOOT_POSE_FRAMES` + 10, et assez pour voir `Idle` revenir : voir ci-dessous), grand chargeur par `weapon_overrides` ; en-tête `À regarder`, frames `--at` et trace refaits (le seul BLESS : `sprite_import_fantassin`) | le scénario lui-même (1510 frames, joueur vivant) |
| 7 (Mac) | **le pivot du manifeste est aux pieds** (point de contact au sol), pas à mi-hauteur : mesure sur `fantassin-de-ligne_idle.png` à `9cc19d7`, première image de chaque direction, le pixel opaque le plus bas est 6 à 18 px sous le pivot, le plus haut 118 à 125 px au-dessus ; sur le Mac les bottes flottaient ≈ 13 unités au-dessus du bas du collider (défaut de placement et de documentation, pas d'échelle : 1,95 case mesurée est conforme) | option `--pivot-at X,Y` (unités monde locales à l'entité, défaut `0,0` : comportement d'avant) : `offset = (centre − pivot) × échelle + pivot_at`, affichée par la commande ; « mi-hauteur » corrigé en « aux pieds » dans la feuille générée, la sortie, le §91 et ce rapport ; **B** réimporte le fantassin avec `--pivot-at 0,-13` (le bas du collider du testbed, 20×20 à `offset_y` −6, est à −16 ; les bottes y arrivent comme les sprites du testbed) | `pivot_at_shifts_the_offset_by_the_given_amount_and_sign` |

### Frames à regarder pour la nouvelle capture du Mac (scénario de B, 1510 frames)

Tir par direction i (0 = E … 7 = SE) : début `490 + 126 i`, Fire tenu 66 frames, pause de 60 frames. La pose `Shoot` retombe 21 frames après le dernier tir, et la première image
d'`Idle` n'apparaît qu'au tick suivant du clip `idle` (250 ms = 15 frames) : **`Idle` revient visible vers `fin_de_tir + 36`, jusqu'au début du tir suivant (`fin_de_tir + 60`)**.
Fins de tir : f556, f682, f808, f934, f1060, f1186, f1312, f1438 ; `Idle` visible de f592 à f616, f718 à f742, f844 à f868, f970 à f994, f1096 à f1120, f1222 à f1246, f1348 à f1372,
f1474 à f1498. `epauler` joué en entier et tenu : `début + 62` (f552, f678, f804, f930, f1056, f1182, f1308, f1434). Capture conseillée :
`--at 35,65,95,125,155,185,215,245,275,305,335,365,395,425,455,485,552,678,804,930,1056,1182,1308,1434` (mosaïque idle, run, shoot), plus `--at 604,730,856,982,1108,1234,1360,1486` pour le retour à `Idle`.
