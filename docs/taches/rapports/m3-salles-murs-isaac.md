# Rapport m3-salles-murs-isaac (murs dessinés, salles à l'échelle, traces, campagnes, p2p)

Revue de William sur les vidéos `isaac` (2026-10-19) : « exactement le même problème de visuel que gungeon : pas d'asset pour les murs et autres, salles minuscules où tout
le monde se chevauche, il faut de grandes pièces ». Même traitement que `m2-salles-murs-gungeon` (`docs/taches/rapports/m2-salles-murs-gungeon.md`, §1 murs, §2 échelle, §6 limites),
avec le **même mécanisme** (aucun second mécanisme). Conventions : `docs/conventions.md` §40 « Échelle des gabarits d'`isaac` » et §83 « Murs, portes, rochers et piques d'`isaac` ».
Base : `origin/main 46a4e8f3`. Poste : VPS (8 cœurs, partagé avec un autre worker : charge 8 à 16 pendant tout le travail, tous les fps ci-dessous en dépendent).

## 1. Murs, portes, rochers, piques et « autres » (présentation seule)

**Mécanisme** : `ui/hud.ron` d'`isaac` gagne `wall_art` (auto-tuile des murs d'après la couche IntGrid `Walls`, `map_ldtk::wall_art`), `hide_markers: true` et `names`. Générateur :
`scripts/generer-sprites-isaac.py`, qui **importe** `scripts/generer-sprites-gungeon.py` (primitives, PNG, atlas, tuiles de mur) et ne remplace que la palette (« sous-sol » : briques brun-rouge `#4e3228`,
dessus `#34231d`, sol `#7B6A5F`, contre le pourpre de gungeon) et ce qu'`isaac` pose. 121 fichiers générés sous `games/isaac/assets/sprites/lisible/` (atlas, feuilles, animations) + le bloc `// >>> généré` de
`sprites.ron` ; `--check`, `--preview` et `scripts/tests/test_sprites_isaac.py` (8 tests : fichiers à jour, déterminisme, aucun orphelin, CC0 dans `assets.yaml`, palette distincte de gungeon, tuiles lues par le moteur,
portes aux quatre états, ids attendus par le moteur). Planche de contrôle : `m3-salles-murs-isaac/apercu_sprites.png`.

**Relevé des objets du monde d'`isaac` sans sprite propre.** Captures impossibles sur le VPS (D94) : le relevé est fait **sur le code et les données** (quelles entités de la partie ordinaire n'ont pas d'entrée dans `sprites.ron`,
`games/isaac/assets/sprites/sprites.ron` n'avait que des personnages, les balles et `slash`), pas sur une image ; l'aspect « d'avant » est donc celui du repli du moteur (tuile de la carte ou carré de couleur) [INFERENCE, non capturé].

| Objet du monde d'`isaac` | Sprite avant | Sprite après |
|---|---|---|
| murs de salle | aucun (couche IntGrid `Walls` noire) | `atlas_murs.png` : 36 tuiles de 16 px auto-tuilées (face de brique au sud d'un sol, dessus ailleurs) |
| portes (4 cases) | aucun | `door_v` / `door_h` : 16×64 et 64×16, états `Idle` (planches cerclées de fer), `Open` (jambages seuls), `Locked` (chaînes et cadenas rouge) |
| **mur secret** (porte de la salle `secrete` et sa jumelle) | aucun (seule signalisation : la poussière, §82) | état `Secret` des mêmes portes : mur de briques fissuré, fissure claire ; ouvert par la bombe = `Open` |
| rochers et rochers à butin | tuile IntGrid coloriée (`#8A6A4A`, `#C9A227`) | `rock`, `rock_loot` (veine dorée) ; la couche `Rocks` est cachée |
| piques (`Surfaces`, valeur 5) | tuile IntGrid grise (`#9A9A9A`) | tuile 36 de `atlas_murs.png` (plaque d'acier, quatre pointes rouges) ; la couche `Surfaces` est cachée |
| sous (`ShellPickup`) | aucun | `shell` |
| coffre | aucun | `chest` (`Idle`, `Open`) |
| présentoir | aucun | `shelf` |
| bombe posée | aucun | `bomb` (la mèche qui clignote est du moteur, §82) |
| objets au sol (100 objets de `objets/*.ron`) | aucun | `item/<id>` : carte (passif), hexagone (actif), fiole (consommable), cœur rouge/bleu/orange (`coeur`, `coeur_ame`, `eclat_ame`, `festin`, `pain`, `ration`), clé, bombe, blank ; repli `item/passive|active|consumable` |
| portail | aucun | `portal` (spirale) |
| apparitions d'ennemis (tuiles de repère d'auteur) | visibles | cachées (`hide_markers`) |

Hors relevé, **inchangés** : personnages, ennemis et boss d'`isaac` (tuiles Kenney `sprites/gungeon/`, ils ont déjà une entrée), larmes (halo `sprites/feedback/`). **Pas faits** (limites, D123) : halo des ramassables, pointeur de visée,
légende (F7) et couleur par joueur (le langage visuel des classes d'ennemis est celui de gungeon), sol sans motif ni ombre portée.

**Trois extensions génériques du moteur, activées par donnée** (les autres jeux ne changent pas) : `wall_art.cells` (une tuile fixe par valeur d'une autre couche IntGrid, pour les piques ; `wall_art::value_cells` testée ; lint de la couche,
fixture `hud_wall_art_cell_layer_unknown`), rocher à butin → `rock_loot` puis `rock` (`world_sprites`), et `SecretDoor` (marqueur de présentation posé par `map_ldtk::game::rooms::mark_secret_doors`, lu par
`world_sprites::sync_door_sprites` : repli `Idle` si la feuille n'a pas l'état `Secret`).

**Rendu jugé sur un rendu synthétique** (`m3-salles-murs-isaac/rendu_synthetique_combat_mixte.png` et `…_combat_piques_mur_fissure.png`) : un script jetable recompose une salle du fichier de gabarits avec la logique de tuiles du moteur
(`wall_tile`) et les atlas générés (un carré bleu de 32 px figure le héros). **Ce n'est pas une capture du jeu.** Les captures du jeu se font sur le Mac (commandes au §7).

## 2. Salles à l'échelle

### Mesure d'avant et d'après (`salles.ldtk`, 23 gabarits)

| | avant (T5, T11d1, T11d2) | après |
|---|---|---|
| unité de salle, `cell` des grammaires | 15×9 cases de 16 px = 240×144 px | **30×18 = 480×288 px** (×2 uniforme) |
| gabarits | 1×1, 2×1, 1×2, 2×2 unités (23 gabarits) | les mêmes, ×2 (23 gabarits, `iid` et champs inchangés) |
| intérieur (mur d'une case) | 13×7 = 91 cases | 28×16 = 448 cases (×4,9) |
| largeur de sol en héros (32 px) / colliders (20 px) | 6,5 × 3,5 / 10,4 × 5,6 | **14 × 8 / 22,4 × 12,8** |
| traversée de l'intérieur à 120 px/s (`pleureur`) | 1,7 s | 3,7 s |
| ouverture d'un côté d'unité | 3 cases (48 px) | 4 cases (64 px), centrée |
| une salle de l'unité à l'écran (960×540, `min_zoom` 0,6 → 576×324 unités visibles) | 42 % × 44 % | **83 % × 89 %** (caméra cadrée sur la salle, §3) |
| rochers | 6 à 12 par gabarit, amas de 3 cases en L dans les angles | **mêmes nombres, mêmes formes**, mêmes angles, collés au même mur |
| piques (cases de `Surfaces`) | 3 à 6 | 21 à 33 (chaque case → bloc de 2×2 à 3×3 : la part de la salle reste la même) |
| étage 7×7 unités | ±840 × ±504 px | ±1 680 × ±1 008 px (`bounds: Some((1800, 1200))`) |

Échelle retenue : **×2 uniforme**, comme le proposait la fiche, de sorte que toutes les tailles restent des multiples de l'unité (la grille de M3-T5 ne change pas de code). *Écart à la demande* : la fiche donnait « intérieur 26×14 cases » pour
480×288 ; avec un mur d'une case l'intérieur de 30×18 est de **28×16** (26×14 supposerait deux cases de mur) : c'est la valeur retenue (14 × 8 héros). Les ouvertures passent de 3 à **4 cases** parce que 30 et 18 sont pairs
(un port de 3 cases ne se centre plus ; 6 cases = un trou de 96 px dans un mur de 480).

Script : `docs/taches/rapports/m3-salles-murs-isaac.make_gabarits_grands.py` (déterministe, idempotent depuis la version 15×9, refuse un fichier déjà agrandi). Contenu inchangé : mêmes ennemis (positions proportionnelles dans l'intérieur), mêmes
types de salle, mêmes rochers. Il **vérifie** : anneau de murs de la source = bord moins ouvertures ; aucune entité sur une autre, un rocher ou une pique ; rien dans le couloir de passage (largeur de l'ouverture, trois cases de profondeur) ;
rien sur les lignes droites qu'un bot suit (rochers à moins de 16 px, piques à moins de 12 px : règle de T11d1) ; portes = ouvertures. Les 23 gabarits passent tous.

### Ce que cela a demandé dans le moteur

- **Rien dans la génération** : les ports centrés de la grille (`FloorLayout::Grid`) tiennent avec `cell: (30, 18)` ; les six grammaires isaac changent de `cell` ; `FloorGrammar::bounds` (déjà là depuis gungeon) passe à `(1800, 1200)` pour
  `sous_sol`, `sous_sol_grand` et `sous_sol_profond` (sept colonnes de 480 px autour d'un départ centré = ±1 680 px, le monde par défaut est ±1 000) ; les grammaires de 3×3 unités tiennent dans le défaut.
- **Tests de `map_ldtk`** (`generation/mod.rs`) : `isaac_floor` prend l'étendue de la grammaire (comme le chargeur), grille de 480×288, ouvertures de 4 cases ; `isaac_etages_de_grille_sur_1000_graines` et
  `la_curse_labyrinth_double_les_salles…` repassent (2 396 évaluations de candidats par étage comme avant : le plan de chaque graine est inchangé) ; le test de coût `isaac_grille_de_12_a_15_salles_sous_trois_fois…` : (b) génération entière isaac 2,7 ms, rapport 1,94 à gungeon (borne 3).
- **Caméra** : `CameraSettings::room_frame` (donnée de `camera.ron`, défaut non) cadre la vue sur la salle où se tient la cible : `CameraRooms` (boîtes des salles de l'étage, publiées par `map_ldtk::game::rooms::publish_camera_rooms`),
  `frame_in_room` (pure, 3 tests). Une salle de l'unité (480×288) tient dans la vue de 576×324 et la caméra y est fixe (une salle = un écran, comme Isaac) ; une grande salle (960×576) est suivie sans montrer au-delà de ses murs. `min_zoom` reste 0,6.
  Présentation seule, non joué avec rendu dans cette passe (D94) : testé sur la fonction pure seulement.
- **Coût** : le champ de flux couvre la boîte englobante de la carte (4 fois plus de cases). `bench_floor_isaac`, profil `FlowField` par frame : 1 500 µs avant (assets d'avant, même binaire) contre 2 502 µs après, **mesurés en même temps sous charge ≈ 16** (une seule paire,
  bruit fort) ; relevé au calme, une fois : 25,6 fps, pire rollback 34,3 ms (budgets 26 fps et 40 ms : l'avertissement « budget » apparaît de peu). **Dette D122** (budgets à refaire au calme, coût du champ de flux, simulation de bot ~2× plus lente : 28 fps contre 53 sous charge comparable).

### Soft-lock : le détecteur ne passait pas à l'échelle (même cause que gungeon, `m2-salles-murs-gungeon` §3.1)

Première partie à 1 bot après le changement (graine 2) : « soft-lock » à f14 875, alors que le bot marchait vers le boss (2 000 px entre le coffre secret et la salle du boss ; ses positions de f13 920 à f14 874 le montrent, `ALACOD_BOT_DEBUG=1`) : le détecteur
(600 frames sans mort, dégât ni ramassage) ne voit pas une marche. Les campagnes ci-dessous passent donc **`--explored-progress`** (entrer dans une case de 32 px jamais visitée est une progression), seul écart aux commandes de T18.

## 3. Traces rebénies et preuve §5

**62 traces rebénies, toutes `isaac`** (`git diff --name-only 46a4e8f3 -- '*.trace'` : 61 + `isaac_quad`), 48 scénarios dont les attentes ont été recalées (aucune supprimée : voir §5) ; **aucune trace hors `isaac` ne bouge** (`git status` après la suite complète : §6).

**Preuve §5, par famille** (même binaire de test, `ALACOD_DUMP_TRACE`, assets d'avant (`ALACOD_GAME_NAME=isaac ALACOD_GAME_DIR=<games/isaac de 46a4e8f3>`) contre assets d'après, `scripts/trace-diff.py` ; le **même fichier de scénario des deux côtés** :
on choisit donc les scénarios dont le `.ron` n'a pas changé) :

| Famille | Scénario | Première frame qui diffère | Ce qui diffère (frame 0) | Types tracés avant / après |
|---|---|---|---|---|
| générés | `generated/isaac/room_combat`, `boss_gardien`, `boss_colosse` | **0** | positions des `character_spawn`, `p_spawn` et des joueurs ; colliders des murs (`ldtk_wall_*`) et des portes ; `FlowFieldCache`, `SurfaceGrid`, compteur d'ids | 92 / 91 (`MeleeHitbox` seulement avant : conséquence), 88 / 88, 88 / 88 |
| grille | `grid_secret_room` | **0** | idem | 85 / 85 |
| curses | `curse_blind` | **0** | idem (+ `AnimationState` des ennemis, dérivé de leur position) | 84 / 84 |
| diable | `devil_room_absent` | **0** | idem | 84 / 84 |
| bot | `bot_isaac_devil_never` | **0** | idem | 88 / 87 (`Projectile` seulement avant : conséquence) |

**Cause unique : la géométrie des gabarits** (positions des entités posées par les salles, colliders de murs et de portes, `SurfaceGrid`, `FlowFieldCache`, compteur d'ids) dès la frame 0 ; **aucun type de composant ou de ressource n'est ajouté ou retiré par le changement**
(les deux écarts d'un type sont des types conditionnels à la partie : un projectile ou un coup de mêlée qui n'a pas eu lieu dans l'un des deux runs). Résumé : `m3-salles-murs-isaac/preuve_5_resume.txt`. Pour les scénarios dont le script d'inputs ou les attentes ont été recalés,
la même cause vaut (rien d'autre n'a bougé dans le moteur de simulation : voir le diff), mais la comparaison directe est impossible (le `.ron` diffère).
