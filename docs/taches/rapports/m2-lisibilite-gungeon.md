# Rapport m2-lisibilite-gungeon (langage visuel provisoire lisible, présentation seule, conventions §83)

## 1. Fait

**Cause racine trouvée en route : la caméra.** `camera_control_system` ne choisissait sa cible qu'une fois ;
la relance hub → run recrée les joueurs, la cible mémorisée n'existait plus, la caméra restait figée à la
position du hub (flèches rouge/bleue de `player_indicator_system` au bord). Les vidéos `gungeon_full_run` ne
montraient donc jamais les joueurs. Une cible disparue est oubliée et re-choisie (`camera/mod.rs`).
Les captures « avant » (`avant_apres/`, moitié gauche) le montrent.

**Langage visuel par classe, en données** (rien de spécifique à gungeon dans le moteur) :

| Classe | Silhouette / couleur | Id `sprites.ron` | Fichier |
|---|---|---|---|
| joueur | pistolero clair, teinté par handle (0 bleu, 1 vert, 2 jaune, 3 violet) | `player` | `sprites/lisible/atlas_persos.png`, teinte : `player_colors` de `ui/hud.ron` |
| bullet kin | balle de laiton ronde, jaune | `bullet_kin` | `atlas_persos.png` |
| escouade | petite balle pointue orange, 3 traits | `bullet_kin_squad` | idem |
| shotgun kin | cartouche rouge large, culot de laiton | `shotgun_kin` | idem |
| lanceur | corps rond violet, tube de mortier | `lanceur_kin` | idem |
| tourelle (stationnaire) | cube d'acier sur trépied, lentille rouge | `tourelle_kin` | idem |
| vétéran | carrure large, casque à crête, vert olive | `veteran_kin` | idem |
| bandana | élancé turquoise, bandeau rouge | `bandana_kin` | idem |
| fuyard | goutte bleu pâle, traits de vitesse | `fuyard_kin` | idem |
| cible | disque rouge/blanc sur pied | `cible` | idem |
| boss | forteresse cramoisie 64 px, couronne dorée | `gatling` | `atlas_boss.png` |
| tir ennemi / allié | orbe rond rouge / losange blanc-bleu, taille = √dégât (8..22) | `bullet_enemy`, `bullet_player` | `bullet_*.png` |
| ramassables | douille, clé, bombe, cœur, blank, carte (passif), hexagone (actif), fiole (consommable), 19 silhouettes d'armes ; **halo pulsant** teinté par rareté/catégorie | `shell`, `item/<id>`, `weapon/<id>`… | `atlas_monde.png`, halo : `legibility.rs` |
| coffre / présentoir | cadenas ; ouvert = couvercle relevé / socle + prix | `chest`, `shelf` | `atlas_monde.png` |
| baril / table / rocher / mur fissuré / bombe posée | rouge à flamme / bois / gris / brique fissurée / sphère à mèche | `barrel`, `table`, `rock`, `secret_wall`, `bomb` | idem |
| portes | fermée (planches) / ouverte / à clé (cadenas rouge), 16×32 et 32×16 | `door_v`, `door_h` (`Idle`/`Open`/`Locked`) | `atlas_porte_v.png`, `atlas_porte_h.png` |
| portail | spirale violette et cyan | `portal` | `atlas_portail.png` |
| eau / sable / fosse | couleurs IntGrid des surfaces, nommées dans la légende | — | `maps/*.ldtk` |

Non fait : **piques** (aucun jeu n'en pose, la surface `hazard` n'a pas de sprite) et teinte
des variantes/champions (déjà faite par M3-T17, `ChampionTint`, inchangée).

**Systèmes génériques** (`crates/game/src/legibility.rs`, `LegibilityPlugin` dans `PresentationPlugin`,
absent en headless) : pointeur de visée rouge sur chaque ennemi (visée de l'émetteur → `EnemyTarget` →
`EnemyPath`, caché sans cible) ; halo pulsant ; teinte par handle (`MarkerTint`) ; étiquettes **F6** /
`ALACOD_LABELS=1` (nom, prix, état ; désencombrées ; masquées hors vue ; allumées d'office dans
`play_scenario --capture` et `make videos`, `ALACOD_LABELS=0` les coupe) ; légende **F7** (widget
`Legend(source: "legend")` de `hud.ron`, `HUD_SOURCES`, lint d'un sprite de légende inconnu + fixture
`hud_legend_unknown_sprite`). `world_sprites.rs` : `SpriteTable`, id candidats `item/<id>` → `item/<genre>` →
`item`, armes `weapon/<id>`, portes d'état, `SpriteRequest` (portail), tuile LDtk d'origine rendue
transparente. `feedback::apply_layer_colors` n'écrase plus la couleur des barres de vie.

**Outillage** : `scripts/generer-sprites-gungeon.py` (stdlib seule, encodeur PNG intégré, `--check`,
`--preview`, supprime les orphelins), 88 fichiers dans `games/gungeon/assets/sprites/lisible/` + bloc
généré de `sprites.ron` ; `assets.yaml` : entrée `sprites-lisibilite-gungeon` (CC0, « placeholder généré,
à remplacer par la voie V6 ») ; les sprites Kenney du lot C sont **retirés** (illisibles ici). Test
`scripts/tests/test_sprites_gungeon.py`. `play_scenario --at 42,157,…` (captures ciblées) et `--legend` ;
`scripts/scenario-video` place l'**image de légende** (rendue par le moteur, frame 40, sans étiquettes) en
tête de chaque vidéo (1,2 s) ; il marche avec un ffmpeg sans `drawtext` (concat). Scénario
`tests/scenarios/gungeon_classes.ron` (toutes les classes sur une image, nouvelle trace).
Doc : `docs/conventions.md` §83, `CLAUDE.md` (F6/F7).

## 2. Preuves (dans `docs/taches/rapports/m2-lisibilite-gungeon/`)

- `apres/frame_<n>.png` : `gungeon_full_run` avec étiquettes à f42 (salle verrouillée), f157 (clé), f1890
  (arène), f2825 (boutique), f2928 (coffre), f3439 (boss), f3489 (portail, au fondu de changement d'étage),
  + f3620 (portail visible).
- `avant_apres/` : côte à côte (barre rouge = main `ad2be1c2`, barre verte = branche) : `gungeon_full_run_f*.png`
  aux mêmes frames, et par classe sur `gungeon_classes` f40 : `joueurs`, `ennemis`, `ramassables`,
  `coffre_presentoirs`, `classes_scene_entiere`. (« Avant » n'a pas `--at` : captures de la partie entière.)
- `tete_gungeon_full_run.png` : image de légende en tête de vidéo.
- Vidéos `gungeon_full_run.mp4` et `gungeon_full_run_four.mp4` (7,3 et 6,8 Mo) : **ré-encodées** (x264 crf 31,
  ≤ 700 kbit/s) pour tenir sous 10 Mo ; les originaux de `make videos SCENARIO=gungeon_full_run,gungeon_full_run_four`
  (24,5 et 23,6 Mo, + `montage.mp4`) sont dans `target/videos/ba12e80a/` (machine b1, non versionnés).

## 3. Vérifié (résultats réels, suite rejouée **sur la fusion avec main `d1b1954a`**, après relecture)

| Commande | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | 3 tranches ok, **536 scénarios**, 237 s ; **aucune trace existante bougée** (`git status` propre après la suite ; seule `gungeon_classes.trace`, scénario nouveau) |
| `make test_crates` | **1 124 tests passés, 0 échec** |
| `make lint` ×5 (zombies, testbed, throne, gungeon, isaac) | « aucune erreur » ×5 |
| `make gen GAME=<jeu>` ×5 | exit 0 ×5, aucun fichier généré modifié |
| `cargo fmt --all -- --check` | exit 0 |
| `make check_sim_boundary` | OK (11 crates, 5 en dette D69) |
| `check-forbidden.sh`, `check-rollback-registration.sh` | OK, OK |
| `make test_tools` | 25 tests OK (dont `test_sprites_gungeon.py`, 6) |
| `make videos SCENARIO=gungeon_full_run,gungeon_full_run_four` | rendues ; originaux 24,5 et 23,6 Mo dans `target/videos/ba12e80a/` |

Pas de p2p ni de bench : présentation seule, rien n'est dans `GgrsSchedule` ni rollback, machine partagée.
**Relecture indépendante** (agent `reviewer`) : aucun P1 sur la simulation ; trouvé et corrigé : **P1** `skin_bullets`
changeait les balles des autres jeux (testbed, isaac) → `own_colors` opt-in par feuille ; **P2** étiquettes de
porte jamais affichées (`GlobalTransform` absent), image de porte ouverte cachée par le parent, image de table/baril
restée après destruction, page de revue décalée par la légende de tête (fichier `<nom>.legend`) ; **P3** halos et
pointeurs cachés avec leur porteur, composants non rollback documentés au §83. Un test (`legibility`) dépendait de
l'ordre des entités : corrigé. Trouvé en plus : étiquettes créées avant le chargement de `hud.ron` (police sans accents).

## 4. Non fait / limites / dettes
- Rendu jugé sur captures et vidéos par l'agent ; **William revoit les vidéos** (critère de sortie).
- Étiquettes : encore denses quand beaucoup d'entités se touchent (salle d'escouade, table + coffre) ;
  le désencombrement ne gère que le chevauchement vertical simple.
- Sprites dessinés par primitives (32 px) : lisibles, pas beaux ; V6 les remplace fichier par fichier
  (l'id de `sprites.ron` ne change pas).
- Autres jeux (isaac, testbed, throne, zombies) : aucune entrée `halo`/`aim`/portes dans leurs `sprites.ron`,
  donc rendu inchangé ; leur donner le langage visuel = dette (copier le script, `hud.ron`).
- `make videos` : un premier essai a échoué avec `ui/hud.ron` « Unexpected variant Legend » (artefact de build
  incrémental après un redémarrage de la machine ; reconstruction propre, non reproduit). Les vidéos ont été
  rendues avant la dernière fusion de main (m3-t8, sans effet visuel sur gungeon).
- Tables, barils : images posées sur l'entité LDtk d'origine (pas de `Transform` sur l'entité rollback).
- Atteignabilité M2 : `gungeon_full_run` et `_four` (frames listées ci-dessus) ; classes : `gungeon_classes`.
