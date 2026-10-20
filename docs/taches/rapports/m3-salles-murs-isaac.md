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

**62 traces rebénies, toutes `isaac`** (`git diff --name-only 46a4e8f3 -- '*.trace'` : 61 + `isaac_quad`), 49 fichiers de scénarios modifiés dont les attentes ont été recalées (aucune supprimée : voir §5) ; **aucune trace hors `isaac` ne bouge** (`git status` après la suite complète : §6).

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

## 4. Campagnes `alacod-sim` (T18 §2) rejouées sur les salles ×2

**Commande** : `docs/taches/rapports/m3-salles-murs-isaac/campagne.sh <préfixe> <bots> <de> <à>` = celle de T18 (`alacod-sim --game isaac --bots N --profiles explorateur… --floors sous_sol --seeds A..B --until-floor 5 --max-frames 20000
--pickup-progress --events …`, séquence par défaut, politique de `bot.ron`, bombe de départ réelle, lots de 20 graines) **plus `--explored-progress`**, sans `--log` (le tueur de chaque joueur n'est pas relevé : pas de tableau des causes de défaite). Binaire : l'arbre de la branche
(`alacod-sim` compilé de l'arbre de la branche avant la passe de formatage et les tests de `map_ldtk` : simulation identique à celle livrée, seuls la présentation, les tests et les données de scénarios ont bougé depuis). Les chiffres d'« avant » sont ceux de T18 (`m3-t18-integration-sortie/{s1,d2,q4}.tsv`, binaire de `main`) : **vérifiés identiques graine pour graine** sur les graines 2, 3 et 4 (1 bot) rejouées ici avec le binaire d'avant et les assets d'avant (`ALACOD_GAME_DIR`).
Script de dépouillement : `m3-salles-murs-isaac/resultats.py` ; résultats par graine : `m3-salles-murs-isaac/{s1,d2,q4}.tsv` (issue, frames, étage, desync, dégâts) ; moments clés et JSON bruts non versionnés.

| Campagne | Graines | Victoires | Parties perdues | Soft-locks | Desyncs | Étage 1 franchi | Frames de victoire (médiane) |
|---|---|---|---|---|---|---|---|
| s1 avant (T18) | 1–200, 1 bot | 24 (12,0 %) | 175 | 1 (graine 63) | 0 | 79 (39,5 %) | 6 147 |
| **s1 après** | 1–200, 1 bot | **138** + 2 « plafond » gagnées à 40 000 frames = **140 (70,0 %)** | **60** | **0** | **0** | **177 (88,5 %)** | 12 443 |
| d2 avant (T18) | 1–200, 2 bots | 95 (47,5 %) | 105 | 0 | 0 | 160 (80,0 %) | 6 565 |
| **d2 après** | 1–200, 2 bots | **183** + 2 « plafond » gagnées à 40 000 frames = **185 (92,5 %)** | **14** | **1 (graine 178)** | **0** | **198 (99,0 %)** | 11 780 |
| q4 avant (T18) | 1–20, 4 bots | 15 (75 %) | 5 | 0 | 0 | 20 | 6 076 |
| **q4 après** | 1–20, 4 bots | **18 (90 %)** | **2** | **0** | **0** | **20** | 11 124 |

- **0 desync sur 420 graines.** Les parties durent ~1,9 fois plus (médiane de victoire 12 443 frames à 1 bot).
- **Quatre « plafond » (20 000 frames atteintes sans issue)** : s1 graines 53 et 101, d2 graines 44 et 101. Ce ne sont pas des blocages : les bots y nettoient encore des salles à f19 500 ; rejouées avec `--max-frames 40000`, **les quatre gagnent** (f20 570, f24 607, f24 204, f23 397).
  Le plafond de 20 000 frames de T18 est devenu serré ; il n'est pas relevé dans le script.
- **Le détecteur sans `--explored-progress`** classait déjà la graine 2 à 1 bot en « soft-lock » à f14 875 alors que le bot marchait vers le boss (§2) : sans l'option, les chiffres ci-dessus ne seraient pas comparables.
- **Un vrai soft-lock, d2 graine 178 (D124)** : f12 908 sur le `colosse` (44 PV restants), les deux joueurs vivants, le plus proche à 191 px **en vue** ; aucun kill de f12 308 à f12 908. Journal de f11 800 à f12 900 : le joueur 122 tire (`Standard-672`… `-693`, depuis (−300 ; 852))
  et ses larmes sont **« stopped by a wall »** : elles s'arrêtent contre un mur (collider `Wall` : mur de salle ou rocher) avant le boss (en (−488 ; 872), 20 px plus haut : visée à quatre directions). C'est un défaut du bot (D99 : « visée à quatre directions »), pas du moteur ; **0 soft-lock à 1 bot** (la graine 63 de D121, soft-lock avant, est maintenant une partie perdue : le plan et les trajets ont changé).
- **Parties perdues** : s1 60 (23 au premier étage, 37 au second), d2 14 (2 et 12), q4 2 (au second). Dégâts subis moyens par partie : 5,2 (s1), 8,2 (d2), 10,8 (q4). Les salles agrandies sont **plus vides** (mêmes ennemis dans plus d'espace) et le bot n'est plus submergé :
  l'équilibrage (nombre d'ennemis par salle) est la décision de William (D64, comme pour gungeon), cette tâche ne le règle pas. **La fourchette de victoires de Q14 reste non fixée** ; D119 (le bot ignore les piques) pèse toujours sur ces taux.

## 5. Scénarios

### Les cinq runs complètes (critères 3 et 5 de T18) : graines rechoisies, aucune armure, aucun `modifiers`

Méthode de T18 : `choisir_graines.py` lit les moments clés de la campagne s1 et prend la plus petite graine qui réunit les moments du scénario ; une graine rejouée en scénario refait la partie de la campagne (vérifié : toutes les attentes recalées sur les frames de la campagne passent au premier essai).

| Scénario | Graine (avant → après) | Frames (avant → après) | Ce qui est prouvé (inchangé sauf mention) |
|---|---|---|---|
| `isaac_full_run` | 120 → **122** (la seule des 200 qui réunit les moments) | 5 700 → 12 400 | salle verrouillée puis nettoyée, bombe → mur secret **et trois rochers cassés par la même explosion** (f477), trois achats en sous dont deux objets de larmes (`glu`, `encre`, `talisman` à la place de `pain`), cœur, `gardien` (phases, portail), second étage avec `explosif` et `essaim` tués, second mur secret, porte à clé, familier acheté (`cousin` à la place de `frere`), `colosse` (phases), victoire f12 293 |
| `isaac_devil_run` | 103 → **103** | 6 870 → 12 200 | les mêmes objets du diable (`cornes_noires`, `queue_fourchue`), victoire f12 135 ; **écart** : chacun des deux joueurs paie un présentoir en vie maximale (le joueur 1 `cornes_noires`, le joueur 0 `queue_fourchue` : 7 → 5 et 6 → 4 PV) au lieu du seul joueur 0 (6 → 4 → 2) ; 6 graines sur 7 essayées achètent au diable (3 sur 11 avant) |
| `isaac_curse_run` | 2 → **2** | 4 150 → 7 200 | curse `darkness` au premier étage (minicarte vide f30, f2000, f4000, f7000), aucune au second (minicarte revenue f7155), mur secret f597, `gardien` f5390–f5450, portail f6052 |
| `isaac_duo` | 2 → **198** (20 graines jouées avec les `items`) | 3 600 → 9 700 | familiers et larmes des deux joueurs dès f1, boss et portail, second étage, `explosif` tué (f8264), joueur 0 à terre (f9111) puis réanimé (f9593) ; **écart** : la mise à terre ne vient plus de l'explosion du champion (mort 800 frames avant) |
| `isaac_quad` | 1 → **163** (15 graines jouées avec les `items`) | 3 300 → 9 300 | cinq familiers dès f1, larmes qui se divisent, guidées, perforantes, quatre objets tenus, boutique, boss et portail, second étage, un joueur à terre puis réanimé (joueur 1, f4977 → f5297) ; **écarts** : un seul achat de familier (`frere`, joueur 1, f6569) au lieu de `cousin` (joueur 3) ; la mise à terre est celle du joueur 1 (le 2 avant) |

Les objets de départ rendent les bots plus solides : sur quinze graines à quatre bots avec les `items`, **une seule** met un joueur à terre (163) ; à deux bots, une sur vingt (198). Le rebond de `billard` n'a toujours pas d'indice propre (§82).

### Les autres scénarios `isaac` (recalés par trois workers, un tableau par famille)

**Cause commune** : tout ce qui est posé en coordonnées absolues contre le joueur a glissé de 16 px (le premier `PlayerSpawn` est à ±32 px du centre de la salle, au lieu de ±16), les trajets sont ~2 fois plus longs, les ennemis arrivent plus tard. 49 `.ron` modifiés au total (avec les cinq runs complètes), **aucune attente supprimée**.

| Famille | Scénarios | Ce qui a changé |
|---|---|---|
| tailles `RoomLayout` | `grid_floor_seed_{1,23,123}`, `grid_post_boss`, `grid_secret_room`, `devil_room_chance`, `isaac_shop_buy` | `size` doublées (le plan de chaque graine est inchangé : 2 396 évaluations de candidats par étage comme avant) |
| objets et effets au pied du joueur | `isaac_egide(_temoin)`, `isaac_fiole_vive`, `hit_iframes_isaac`, `isaac_hud_items`, `isaac_nova`, `isaac_bombe_fiole`, `isaac_consommables_lot`, `isaac_bouclier_de_fortune`, `isaac_ombre_bloque(_sans_objet)`, `isaac_reliquaire`, `isaac_synergy_garde`, `isaac_banner_blind` | positions décalées de −16 px (mêmes distances, mêmes frames) ; `cible_mordante` rapprochée (à 48 px elle ne mordait plus) |
| parties courtes de bot | `isaac_sablier(_temoin)`, `isaac_cuirasse`, `isaac_vengeance` | 500 → 600 frames, attentes recalées sur les événements (verrouillage f102, coup f187, kills f191/253/337) ; une `cible_mordante` ajoutée à f400 : après le premier nettoyage plus rien ne touche le bot (graines 1 à 21 essayées) |
| marches écrites à la main | `isaac_spikes(_temoin)`, `isaac_bomb_rock`, `isaac_shop_buy`, `isaac_bol_de_soupe(_temoin)` | marches réécrites (amas ~2 fois plus loin), `Right` f1–8 pour franchir le portail |
| plan de grille, bots longs | `grid_floor_seed_*`, `grid_post_boss`, `isaac_floor_seed_1`, `achievement_unlocks`, `bot_isaac_{floor,two,secret,key}` | `frames` allongées (3 500 → 4 750, 5 700 → 9 800…), attentes datées recalées sur la frame observée, mêmes graines |
| `grid_secret_*`, `grid_treasure_*`, `devil_deal_isaac` | | marche vers le mur secret ou la porte à clé refaite (trajets ×2, position finale (−20 ; −399,3)), présentoir au pied du joueur |
| trace seule | `grid_secret_room`, `devil_room_{absent,chance}`, `curse_{bigger_floor,blind,hides_health,hides_minimap}`, `achievement_not_met`, `bot_isaac_{devil,devil_never,shop,shop_blesse}`, `generated/isaac/{room_combat,boss_gardien,boss_colosse}`, `bench_floor_isaac` | aucune attente n'échouait |

**Deux attentes qui ne prouvent plus tout à fait la même chose** (dites, pas retirées) : `isaac_bomb_rock` prouvait « le rocher à butin laisse une `ration` » (`ItemsOnFloor(ration)`) ; le butin dépend de la clé `rock:<salle>:<x>:<y>` que la géométrie change, le tirage donne maintenant un tas de
sous : l'attente est `ShellPiles` 0 puis 1 (trois rochers cassés, 45 → 42, joueur hors du rayon, inchangés). `isaac_spikes_temoin` ne couvre plus que les trois premières périodes de piques (f191, f251, f311) : un ennemi arrive à f357 et les i-frames du coup de f311 absorbent la quatrième.
Aussi : le portail d'un étage de passage (`sous_sol_cle`) est au barycentre des deux `PlayerSpawn`, maintenant à 32 px du premier joueur (rayon de 24 px) : les scénarios qui franchissent le portail à f1 ajoutent `Right` f1–8.

## 6. p2p réel et vérification (résultats réels, VPS chargé)

- **p2p, 2 clients, `scripts/p2p-stress.sh isaac`** : inputs de deux bots figés par `scripts/p2p-gungeon-inputs.sh` (`GAME=isaac FLOORS=sous_sol UNTIL_FLOOR=5 SIM_ARGS=--pickup-progress`, **graine 23**, 3 500 frames, passage d'étage à f2984), délai d'input 0, un client suspendu en boucle,
  `MATCHBOX_PORT=3568 COMPOSE_PROJECT_NAME=b6 EXPECT_FLOORS=1 GAME_ARGS="--seed 23"` : **`p2p-stress : OK`**, codes de sortie 0 0, **0 `DesyncDetected`**, 5 171 et 83 frames resimulées (client 0 : 360 rollbacks, profondeur moyenne 9,3, maximale 11), **transition d'étage f2983 sur les deux clients**, 3 499 lignes de trace contiguës et identiques.
  Une machine seulement (D73 : deux machines non rejouées).
- `make test_scenarios_par SHARDS=3` **sans BLESS** : **3 tranches ok, 807 scénarios, 3 165 s**, `git status` : **aucune trace modifiée** (les 62 traces `isaac` rebénies sont déjà dans l'arbre, aucune trace hors `isaac` n'a bougé).
- `make test_crates` : lancé tel quel, il remplit le disque partagé (les exécutables de test font 1 Go chacun, ~38 Go) et le lieur meurt (`Bus error`, `No space left on device`, deux fois) ; rejoué **un paquet (et un test d'intégration) à la fois** avec suppression des exécutables de plus de 300 Mo après chaque étape (même commande
  `cargo test -q --profile headless --no-default-features --no-fail-fast … -- --exact --skip scenarios`) : **1 246 tests passés, 0 échec, 18 ignorés**. Deux tests de `map_ldtk` échouaient d'abord (`isaac_etages_de_grille_sur_1000_graines`, `la_curse_labyrinth_double_les_salles…` : l'unité 240×144, les trois cases d'ouverture et l'étendue ±1000 y étaient écrites en dur) : corrigés (§2), verts.
  Les tests nouveaux : `camera::tests` (3), `wall_art::tests::une_couche_annexe_ne_dessine_que_la_valeur_demandee`, `lint_fixtures::hud_wall_art_fixture_reports_unknown_cell_layer`, `scripts/tests/test_sprites_isaac.py` (8).
- `make lint` (les cinq jeux) : **0 erreur** chacun ; `make gen GAME=zombies|testbed|throne|gungeon|isaac` : **rc 0**, 16, 104, 39, 59 et **213** scénarios générés « ok ok », aucun fichier modifié ensuite ; `cargo fmt --all -- --check` : propre ; `scripts/check-forbidden.sh` : OK (47 occurrences auditées) ;
  `make check_sim_boundary` : OK (13 crates `SIM`, 2 en dette D69) ; `make test_tools` : 33 tests OK ; `scripts/check-rollback-registration.sh` : OK.
- **Non fait** : captures et vidéos (D94 : impossible sur le VPS), deux machines pour le p2p, `make bench` strict (D122), réseau dégradé pour `isaac`.

## 7. Limites, dettes et ce que William regarde sur le Mac

Dettes ouvertes par cette tâche (`docs/taches/dettes.md`) : **D122** (coût du champ de flux et budgets d'`isaac` à refaire au calme), **D123** (limites visuelles : personnages Kenney, pas de halo ni de légende, sol uni, rochers de 16 px, coffre cadenassé), **D124** (soft-lock du bot à deux joueurs devant le `colosse`,
graine 178 ; plafond de 20 000 frames des campagnes), **D125** (cadrage de caméra par salle et mur fissuré non vus avec rendu). D121 (soft-lock du bot sur un tas de sous, graine 63) ne se reproduit plus (la graine 63 est une partie perdue) mais la cause n'est pas corrigée dans le bot.

**Commandes de capture (Mac, avec affichage ; `make videos` rend `target/videos/<sha>/`)** :

```
make videos SCENARIO="isaac_full_run,isaac_devil_run,isaac_curse_run,isaac_duo,isaac_quad"
make views SCENARIO=isaac_duo ; make views SCENARIO=isaac_quad        # une vue par joueur
make review_videos                                                      # page de revue (http://localhost:8766)
make compare_video SCENARIO=isaac_curse_run BASE=46a4e8f3               # avant/après (même graine 2 des deux côtés)
# captures ciblées (image n = frame n, 960×540, étiquettes allumées d'office) :
cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/isaac_full_run.ron --capture /tmp/isaac_full --at 60,402,477,500,773,5040,6000,6916,6991,12293 --follow 0
cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/grid_secret_bomb.ron --capture /tmp/isaac_secret --at 5,66,140,145,200 --follow 0
cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/isaac_spikes.ron --capture /tmp/isaac_spikes --at 100,191,251 --follow 0
cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/isaac_bomb_rock.ron --capture /tmp/isaac_rocks --at 205,285,290 --follow 0
```

Ce que William regarde (frames de `isaac_full_run`, graine 122, et des scénarios courts) :

| Où | Ce qu'il faut juger |
|---|---|
| `isaac_full_run` f60 | une salle de l'unité (480×288) **entière dans l'image**, murs de brique, portes de 64 px, sol brun clair ; le héros fait 1/14 de la largeur de sol ; la caméra ne bouge pas tant que le joueur reste dans la salle |
| f402 à f477 | la bombe posée, son explosion (f476), le **mur fissuré** qui s'ouvre en porte (f477) et les trois rochers cassés au même instant ; les rochers se lisent-ils (16 px, gris à facettes, butin doré) ? |
| f773 (boutique) | présentoirs, objets au sol (`item/<id>`), sous (`shell`), bandeau d'objet |
| f5040 à f6000 | le `gardien` meurt, le portail, la transition d'étage (fondu), la salle suivante vue d'un seul tenant ; une salle 2×2 (960×576) est-elle suivie sans montrer au-delà de ses murs ? |
| `grid_secret_bomb` f66–f145 | le mur fissuré **avant** l'explosion (porte secrète fermée, reconnaissable), puis la porte ouverte |
| `isaac_spikes` f100–f251 | le bloc de piques (plaque d'acier, pointes rouges) et le demi-cœur perdu à chaque période (f191, f251) |
| `isaac_duo` / `isaac_quad` (`make views`) | quatre joueurs dans une salle de 480×288 : plus de chevauchement ? l'écran reste-t-il lisible (zoom 0,6, caméra fixe par salle) ? |

Jouer : `make isaac` (solo), `APP_VERSION=dev cargo run -p isaac --features native -- --local-port 7000 --players localhost localhost` (deux héros, mêmes touches : `docs/jouer.md`), graines proposées : 122 (la partie de `isaac_full_run`), 103 (salle du diable au premier étage), 2 (curse `darkness`).
