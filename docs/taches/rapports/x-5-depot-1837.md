# X-5 — Création de 1837 et kit de la cour (VPS)

Base `origin/main` `7c707277`, branche `wquintal/x-5-depot-1837`. Fiche : `docs/taches/plan-1837-cli.md` (lot X-5, § 3 « Décisions de William », § 5, § 6), spec du run
`/home/debian/orca/runs/x-5/spec-x5.md`. Convention : `docs/conventions.md` § 107. **Simulation intouchée** : aucun type rollback, aucun système `GgrsSchedule`, aucune trace
de clone ne bouge ; seule la trace de `sprite_import_fantassin` disparaît avec le revert. Aucun `BLESS`. `make check` : lancé par le coordinateur sur la tête poussée.

## Partie E (engine)

| Point | Issue | Preuve |
|---|---|---|
| E1 retrait de l'exception du testbed | fait | `git revert` de `a81523fe`, `7b5d9d20`, `a878966c` (trois commits `Revert …`) : huit fichiers supprimés (personnage, feuille, animation, PNG, scénario, trace, entrée `sprites.ron` et `assets.yaml`). `docs/conventions.md` §91 (« Preuve sur un vrai manifeste ») et `plan-1837-cli.md` § 5 (exception levée) corrigés ; `x-4b-import-sprite.md` inchangé. `alacod lint games/testbed` : « aucune erreur (40 personnages, 25 armes…) ». |
| E2 `alacod new 1837` | fait | `new_game` accepte un nom qui commence par un chiffre ; `cargo_package_name` : `2048` → paquet et binaire `jeu-2048` (préfixe `jeu-` si le nom commence par un chiffre, sinon inchangé) ; `templates/starter/Cargo.toml` : `name = "{{package}}"`, `{{name}}` reste le nom du jeu (dossier, `game.ron`, titre de fenêtre, messages). Test `crates/content/tests/cli.rs::new_accepts_a_numeric_game_name_with_a_valid_cargo_package` (nom neutre `2048` ; `cargo metadata --no-deps` accepte le manifeste ; lint vert). `cargo test -p content --test cli` : 12 passés, 2 ignorés. `make test_alacod_cli` : 2 passés (`new_game_opens_its_sandbox_without_any_added_code`, `test_command_plays_generated_scenarios…`), 2164 s. |
| E3 import de décor | fait (la partie J l'a exigé) | `alacod import-sprite decor-mur-rondins.sprite.json` refusait : « clip « static » : fps = 0 : nombre fini > 0 ». Deux compléments génériques : `fps = 0` + une colonne = clip fixe ; `--direction <DIR>` garde une seule ligne (décor figé dans une orientation, `mirror: false`). Tests synthétiques neutres `crates/content/tests/sprite_import.rs` : `static_clip_with_zero_fps_imports_as_one_image`, `zero_fps_with_several_columns_or_a_negative_fps_fails_without_artifacts`, `direction_keeps_only_that_row_of_every_clip` ; 25 passés. |
| E4 docs et dettes | fait | `docs/conventions.md` § 107 (+ renvoi au § de `alacod new`) ; `templates/starter/CLAUDE.md` (nom du paquet) ; dettes D186–D189 dans `docs/taches/dettes.md`. |
| E5 vérifications | fait | `cargo test -p content --test cli`, `--test sprite_import`, `--lib sprite_import` verts ; `make test_alacod_cli` vert ; `make test_tools` : 50 tests OK ; `alacod lint games/testbed` : 0 erreur. Pas de `make check` (coordinateur). |

## Partie J (dépôt 1837)

Dépôt privé `estran-studio/1837`, branche `main`, SHA de tête au moment de ce rapport : `2ee5017fa51a800ddf8c490d0ee025abc9f984ee`. Engine épinglé : `6617cde175a83e000c8f107043ec669db514ddb3` (tête de cette branche, **pas encore un
`main`** : le `rev` sera réépinglé sur le `main` de l'engine après le merge du coordinateur ; les sept lignes `git = …, rev = …` de `Cargo.toml` et `Cargo.lock`).

| Point | Issue | Preuve |
|---|---|---|
| J1 création | fait | `alacod new 1837 --git --rev 6617cde1` (binaire construit depuis la branche E) dans `/home/debian/orca`, remote `origin` = `https://github.com/estran-studio/1837.git` ; paquet et binaire `jeu-1837` ; aucun `path` vers l'engine dans `Cargo.toml`. Premier commit `af56054`. Cold build release : 21 min 09 s (8 cœurs, `CARGO_BUILD_JOBS=4`). |
| J2 kit importé | fait (loge : pas de planche) | `scripts/importer-kit.sh` (20 imports, un `alacod import-sprite` chacun, `--overwrite`). **Échelle** : `--units-per-meter 24` pour tout le kit (cases de 16 unités = 0,67 m ; porte de 1 m = 24 unités ; Frère 37,7 unités = 2,4 cases ; atlas du Frère 5632×6120 px, `--max-side 6144`). Frère : id `player` (68 clips, huit directions, `Idle=idle`, `Run=marche`, `Shoot=tir`, `--pivot-at 0,-13`), fantassin (29 clips, `Idle=fusil-idle`, `Run=fusil-marche`, `Shoot=tir`), `hache-poc`, `fusil-chasse`, **décor (16 images)** : `decor-mur-rondins` (h, v), `decor-mur-porte` (h), `decor-mur-coin` (e, n, w, s), `decor-barriere` (h, v), `decor-sapin-grand`, `decor-sapin-petit`, `decor-tonneau`, `decor-caisse`, `decor-lanterne-poteau`, `decor-tas-foin`, `decor-charrette-foin` (choisis dans `decor-*` : cour d'une loge en hiver ; non importés : borne, souche, rocher, roseaux, forge, enclume, chaînes, poêle). Licences : `assets/assets.yaml` (20 entrées : « William Quintal (estran-studio), pipeline 1867_lore » ; « © 2026 estran-studio, tous droits réservés »). Les couches `armes/` et les mains ne sont pas importées (D156) : on voit le Frère avec un fusil cuit dans le rendu d'`idle`, la pose `tir` quand il tire, **aucune hache** à la main ni pose de coup. **Loge** : aucun manifeste dans `1867_lore/assets/sprites/` (brief `loge-auberge.yaml` en brouillon, porte art du 9 nov) ; non inventée, écrit dans l'état de X-5 de `docs/tranche.md`. |
| J3 la cour | fait | `maps/cour.ldtk` (34×22 cases, `alacod new-arena --size 34x22 --players 2`, deux `PlayerSpawn`) + décor et fantassin posés par `scripts/poser-decor.py` (55 `CharacterSpawn`) ; `maps/cour_decor.ldtk` + profil `sandbox/cour.ron` (fantassin posé par le profil) pour `alacod sandbox --profile cour`. Deux joueurs : `--players localhost localhost` donne deux Frères aux deux points (`docs/preuves/x-5-cour-2joueurs.png`). Armes : `fusil_chasse`, `bare_hands` = hache, valeurs de `game-feel.md` §§ 3, 4, 7 et `armes.md`, le reste **provisoire** dans le RON. |
| J4 `docs/tranche.md` | fait | copie telle quelle de `runs/x-5/tranche.md` (commence par « Objectif studio : O3 »), section « État de X-5 » remplie (fait, manque, loge, échelle, commande du soir, `rev`), puis plan par jour de `runs/x-5/plan-jour.md` versé après le calendrier (commit `2ee5017`, demande du coordinateur). |
| J5 `CLAUDE.md` | fait | 66 lignes (≤ 150) : renvoie à `docs/tranche.md`, `1867_lore` (lecture seule, définition du niveau écrite par William), règles GGRS de l'engine (lien, pas de copie), `alacod --help` ; règle « rien de 1837 dans l'engine, tout verbe nouveau passe par l'engine ». |
| J6 build du soir | fait (Mac non essayé) | `./jouer.sh` (Cargo seul) : `git pull --ff-only` puis `cargo run --release` ; avec un nom de lobby : `-- --number-player 2 --matchbox wss://allumette.estran.studio --lobby <nom> --players localhost remote` (`MATCHBOX_URL` remplace le serveur). En tête de `README.md` et dans `tranche.md`. Pas lancé sur le Mac. |
| J7 vérifications VPS | fait, un écart | `alacod lint` : « aucune erreur (19 personnages, 5 armes, 6 armes de corps à corps, 0 vagues, 3 cartes) » ; le jeu construit ; `./jouer.sh` sous Xvfb + lavapipe (`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`) atteint la cour ; **`./jouer.sh x5-lobby` sur deux instances avec un `matchbox_server` local (`MATCHBOX_URL=ws://127.0.0.1:3537`) : les deux entrent en partie (≈ 2 370 lignes `ggrs{f=` chacune en 45 s)** ; deux instances `ALACOD_HEADLESS=1` (build `--no-default-features --features native`, 24 min) dans le même lobby : 600 frames, **599 lignes de trace identiques** (`check-p2p-traces.py --frames 599` : « OK: 2 pairs, 599 frames contiguës et identiques », aucun `DesyncDetected`, copie dans `docs/preuves/x-5-p2p-600-frames.trace`). **Écarts** : zéro frame resimulée (inputs neutres, aucun rollback réel : pas la preuve d'un rollback) ; le bac à sable (`--sandbox`) refuse `--matchbox` (« session locale seulement »), d'où la cour comme partie ordinaire (le Frère est le personnage `player`, la carte pose le fantassin) ; la build headless d'un jeu externe exige `--no-default-features` (la build de rendu panique sous `ALACOD_HEADLESS=1` : « No sub-app with label 'RenderApp' »). |
| J8 preuve | fait | `docs/preuves/x-5-cour.png` (800×600, prise par `alacod remote screenshot` pendant `./jouer.sh`, rendu lavapipe) : le Frère, le fantassin, la palissade, le tonneau, la caisse, le foin, la charrette, un sapin, vus à la même échelle (Frère ≈ 55 px, mur de rondins ≈ 50 px, tonneau ≈ 40 px) ; vue et jugée à l'œil : les deux personnages sont visibles et posés au sol. `x-5-cour-2joueurs.png` : deux Frères. Transcript de cinq lignes ci-dessous. |
| J9 push | fait | `git push origin main` : `2ee5017fa51a800ddf8c490d0ee025abc9f984ee` (à rejoindre : `main` de l'engine après merge pour le `rev`). |

Transcript de la commande du soir (VPS, `./jouer.sh` sous Xvfb, après la première compilation) :

```
$ ./jouer.sh
Already up to date.
    Finished `release` profile [optimized] target(s) in 0.68s
     Running `target/release/jeu-1837`
INFO game::global_asset: loading of asset is done , now entering lobby
INFO map_ldtk::game::local: Map is loaded with 55 character spawns   (puis la partie : Joueurs 1, Ennemis 55, fusil_chasse 1 | 12)
```

### Ce qu'il faut savoir

- **Décor = personnage neutre** (D186) : « Ennemis 55 » au HUD, barres de vie rouges au-dessus de chaque décor, pas de tri en y (une palissade passe devant ou derrière selon l'ordre de dessin). Visible sur la capture.
- **Positions à la case de 16 unités** pour tout ce que pose une carte : une palissade de murs de 24 unités se chevauche de 8 unités entre deux cases.
- **`player`** : le Frère a l'id `player` (seul personnage qu'un joueur prend hors bac à sable), sa planche est importée `--id player`.
- **Atlas du Frère 5632×6120 px** (≈ 138 Mo) : trop gros pour WebGL2 ; à réduire pour le build Chrome (M-1).
- **Cargo affiche `error: missing key for inline table element`** sur `templates/starter/Cargo.toml` à chaque `cargo run` du jeu (inoffensif, D189 a) ; renommer le gabarit en `Cargo.toml.tmpl` serait la correction (hors X-5).
- Les `rev` : 1837 dépend de `6617cde1` (branche) ; à réépingler sur le `main` de l'engine dès son merge.

## Journal

`docs/taches.md` § 10 : ligne `LIVRÉ wquintal/x-5-depot-1837` ci-dessous.

## Clôture (coordinateur, 2026-11-06)

- Partie E mergée : `main` `0e7c8d58` (merge de `51d87e0a`), puis `748deacb` (cible `check_rapide`). Vérification avant merge : **contrôle rapide** (décision de William du
  6 nov) sur `51d87e0a`, vert : format 3 s, test_tools 5 s, check_forbidden 1 s, check_rollback_registration 1 s, check_sim_boundary 8 s, lint 70 s, test_crates 434 s
  (522 s, cible chaude ; journal `~/orca/runs/x-5/logs/check-rapide-51d87e0a.log`). La suite de scénarios passe la nuit (timer `alacod-nuit`, première nuit le 7 nov).
- 1837 réépinglé sur `0e7c8d58` (un `main` poussé) : `estran-studio/1837` `07a2084` (build release `--locked`), puis `0ac2f5b` (tranche : règles de vérification et du plan).
  Les mentions de `6617cde1` ci-dessus datent d'avant ce réépinglage.
- Preuve du dim 8 : `./jouer.sh` (README du dépôt 1837) et `docs/preuves/x-5-cour.png` (rendu Xvfb + lavapipe, VPS). Non vu sur le Mac : la partie du soir de William fait foi.
