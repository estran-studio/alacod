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
- **Échelle et pivot** : `scale = (U / ppm) / facteur` ; pivot du manifeste → `anchor: Center` + `offset_x/offset_y` en unités monde ; **le pivot est à mi-hauteur, pas aux pieds**
  (l'`offset_y` écrit remonte le sprite de 2,3 unités sous le centre de la case, pour le fantassin). La hauteur de silhouette est mesurée et reportée.

## Preuve du lot 1 : le vrai fantassin dans le testbed (commit de tête, retirable)

Tout ce qui vient du fantassin est dans **un seul commit en tête de branche** (« X-4b : fantassin-de-ligne importé dans le testbed (retirable) ») : dossier `games/testbed/assets/sprites/fantassin-de-ligne/`, entrée de `sprites.ron`, `characters/fantassin-de-ligne.ron`, entrée d'`assets.yaml`, scénario `sprite_import_fantassin` et sa trace. Si William répond « import local non commité », ce commit se retire sans toucher l'outil, les tests synthétiques ni la doc ; les commits d'en dessous sont verts sans lui (vérifié, voir plus bas).

Manifeste `origin/main` `9cc19d7` de `1867_lore` (lu, jamais modifié) : cellule 376×372, `ppm` 91, pivot (186,72 ; 196,34), dix clips de 8×8 cases (3008×2976 px).
Commande et sortie (première étape avec U provisoire = 30 : silhouette de 143 px source → 47,1 unités ; U = 32 × 91 / 143 ≈ 20,4 donne **32,1 unités, deux cases**, comme la boussole) :

```text
$ alacod import-sprite fantassin-de-ligne.sprite.json games/testbed --id fantassin-de-ligne --units-per-meter 20.4 --state Idle=idle --state Run=marche --state Shoot=epauler --overwrite
import fantassin-de-ligne : facteur 0.3441, cellule 129×128 px (source 376×372)
atlas 3096×4096 px : 3×4 blocs, 24 colonnes × 32 lignes de cases, 10 clips
échelle 0.65151095 ; pivot à mi-hauteur (pas aux pieds) : offset (0.28694504, 2.317978) unités
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

## Scénario du lot 2 (Mac)

`tests/scenarios/sprite_import_fantassin.ron` : `Idle` (visée seule), `Run` (déplacement + visée) puis `Shoot` (Fire tenu 66 frames, pause de 30) dans les huit directions, 740 frames, commentaire `// À regarder :` avec les frames. Le runner **exige** une trace de référence (« pas de trace de référence »), donc la seule trace neuve,
`tests/scenarios/sprite_import_fantassin.trace`, est écrite ici, sur le VPS, par le seul BLESS permis :

```sh
BLESS=1 SCENARIO=sprite_import_fantassin make test_scenarios
```

(740 frames, joueur vivant, invariants tenus). Les traces étant identiques entre x86-64 et arm64 (§57), le Mac la relit sans la réécrire ; `git status` ne montre que ce fichier de trace nouveau.

Capture réelle (une image par direction et par pose : idle f35…245, run f275…485, shoot f552…1224 : `epauler` joué en entier et tenu, 62 frames après le début du tir de chaque direction) :

```sh
CARGO_BUILD_JOBS=4 cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/sprite_import_fantassin.ron \
  --capture target/videos/x-4b-import-sprite --at 35,65,95,125,155,185,215,245,275,305,335,365,395,425,455,485,552,648,744,840,936,1032,1128,1224 --size 960x540
# mosaïque 8 colonnes × 3 lignes (idle, run, shoot) ; ajuster le motif aux noms d'images produits
ffmpeg -y -pattern_type glob -i 'target/videos/x-4b-import-sprite/*.png' -vf "scale=480:-1,tile=8x3" -frames:v 1 docs/taches/rapports/x-4b-import-sprite.png
```

À vérifier à l'écran : silhouette de ≈ 2 cases (32 unités), pivot à mi-hauteur (les pieds flottent un peu sous la case, à régler par `offset_y`), huit lignes
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

## Revue indépendante (2026-10-28)

Commits revus : `f7f04e58` (outil, présentation, tests, doc) et `a878966c` (fantassin, retirable). Six constats, tous corrigés ; chacun a un test, vérifié **en échec sur
l'ancien code** (les cinq tests de `content` et celui d'`animation` ont été rejoués contre le code d'avant). Nouveaux commits : « X-4b revue : import transactionnel, table et
manifeste analysés, clés échappées, durée d'une seconde » (sous le fantassin) puis le commit de tête du fantassin, réécrit pour le correctif 6 ; leurs SHA dépendent de ce
rapport et sont donnés dans le message de fin de tâche et par `git log`. Aucune dette nouvelle (rien n'est reporté).

| # | Constat | Correctif | Test |
|---|---|---|---|
| 1 P1 | `std::fs::write` tronquait les destinations ; un échec avec `--overwrite` supprimait d'anciens fichiers au lieu de les restaurer, `sprites.ron` pouvait rester tronqué | `commit_files` : tout est écrit en fichiers temporaires du même dossier, l'ancien fichier est mis de côté puis le nouveau renommé (`sprites.ron` en dernier) ; échec = renommages inverses, temporaires et dossiers créés retirés, état d'avant intact octet pour octet | `failed_overwrite_leaves_the_previous_import_intact_byte_for_byte` (panne simulée à chacun des 4 pas, comparaison des octets), `failed_first_import_leaves_no_file_and_no_directory`, `unwritable_table_directory_fails_before_touching_anything` (dossier en lecture seule réel) |
| 2 P2 | `check_animation_config_reload_system` prenait toute durée d'une seconde pour « non initialisé » : un clip à 1 fps était remis à la durée globale à chaque mise à jour et n'avançait jamais | vrai marqueur `AnimationTimer::configured`, posé par le rechargement et par `animate_sprite_system` quand il écrit la durée du clip | `a_one_second_clip_advances_with_both_systems` (les deux systèmes, configuration chargée par l'`AssetServer`, 600 ms par mise à jour) |
| 3 P2 | `entry_span` trouvait `"id": (` dans un commentaire | analyse de la table (`scan_table`) : commentaires, chaînes et valeurs imbriquées sautés, premier niveau seulement ; la table produite est relue (`verify_table`) avant écriture | `commented_entries_are_not_mistaken_for_real_ones` |
| 4 P2 | la table `SpriteSheet` était cherchée par `path:` dans les 400 octets avant `"SpriteSheet"` (ordre des champs, coupure UTF-8) | `GameManifest::parse` : première déclaration `kind: "SpriteSheet"` des `content_folders` | `sprite_sheet_declaration_is_read_by_the_manifest_parser` (`(kind, path)` inversés, déclaration avant, accents) |
| 5 P2 | une barre oblique inverse dans un nom de clip ou d'état donnait une échappée invalide ou une autre clé | chaque clé écrite passe par la sérialisation RON (`ron_key`) ; feuille, animation et table sont relues avant écriture (refus sinon) | `backslash_in_clip_and_state_names_is_escaped_in_ron` |
| 6 P2 | le scénario ne laissait jamais retomber la pose `Shoot` (24 frames de tir, 6 de pause, pose tenue 20 frames, `epauler` = 60 frames) | 66 frames de tir par direction (`epauler` joué en entier puis tenu), 30 frames de pause qui relâchent la pose, grand chargeur par `weapon_overrides` ; en-tête `À regarder`, frames `--at` et trace refaits (le seul BLESS : `sprite_import_fantassin`) | le scénario lui-même (1270 frames, joueur vivant) |
