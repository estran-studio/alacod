# Polissage de throne : visuel (S2-10)

Branche `polissage-throne-visuel` (depuis `origin/polissage-integration` a7a7a49b). Fiche : `runs/alacod-polissage/b1-s2-10-throne-visuel.md`.
Packs payants (licence achetée par William, **redistribution interdite**) : Aymen B « UPDATE 2_ZombiePack » et Ittai Manero
« Zombie Apocalypse Tileset » ; ils passent par la surcharge locale gitignorée (conventions § 92), jamais dans le dépôt.

## 1. Audit J1 (état « avant », 2026-10-29)

Sources lues : `games/throne/assets/{sprites/sprites.ron, characters/*.ron, weapons/weapons.ron, items/powerups.ron, ui/hud.ron, assets.yaml,
caves/gabarit.ldtk}`, `map_ldtk::game::floors` (portail), `game::world_sprites` / `legibility` (ids cherchés), les deux archives (listées, puis
les dossiers utiles extraits **fichier par fichier** sous `~/orca/workspaces/alacod/packs/{aymen-b,ittai}/`, hors dépôt), captures
`play_scenario` de `throne_solo`, `throne_quad` et `throne_duo_defaite` (repli public : throne n'a **aucune** surcharge locale à ce jour).

Mêmes définitions que `polissage-zombies-visuel.md` § 1 : *propre* = image dessinée pour ce rôle, propre au clone, licence lisible ;
*emprunté* = atlas d'exemple, pack générique, fichier octet-identique dans d'autres jeux, licence inconnue ; *absent* = aucune image.
Animations jouées par le moteur : `Idle`, `Run`, `Dying_*` ; `Attack` seulement par la pose de présentation (`PresentationClip`, S1-8),
**si** la feuille la définit.

### 1.1 Ce que couvrent les deux packs (mesuré)

| | **Aymen B** (cases de 32 × 32, 1 vue de face, miroir pour la direction) | **Ittai Manero** (sprites séparés de 8 à 16 px, 3/4) |
|---|---|---|
| Personnages | Joueur (idle 6, run 8, mort 8, touché 3, assommé 6) ; Bandit à masque à gaz (idem) ; Zombie 1 (vert), 2 (vert sombre), 3 (pustules jaunes), 4 (os, brun-rouge) : idle 6, run 8, mort 8 (×2 pour 1 et 2), touché 3 ; 2 PNJ | Joueur (marche 9), grand zombie (9), enfant (9), maigre (9), **zombie tourelle** (idle 12 + « vomi » 7 images + touché 9), versions touchées, cadavres, oiseaux, épouvantail (2), boîte de pièces |
| Attaque | **aucune** (« Hit » = encaisser) | tourelle : 7 images de crachat ; couteau (3 arcs, effet) |
| Armes | Hache, couteau, pistolet, revolver, fusil à pompe, fusil d'assaut, sniper, lance-roquettes (7 images « flicker » + tir avec éclair) ; planche 32 × 32 | pistolet et fusil à pompe en main du joueur (5 et 6 images), objets ramassables et armes (20) |
| Projectiles et effets | balles (standard, 3 lueurs orange / cyan / orange, longue, grenade), impact (5 images), explosion 48 × 48 (7 images), fumée | explosion (6), fumée (6), sang (5), fût explosif (4) |
| Objets | 36 objets 16 × 16 animés (trousse, 4 caisses de munitions, grenades, dynamite, batterie, bidon, crâne, radio, électronique, gilet, masque…), **3 coffres** (rouge, vert, métal) | boîte d'objets qui apparaît + pièces (animation), pierre tombale, inventaire (15) |
| Décor | sol désertique (Wasteland, 32 × 32 : 40 tuiles de sable), voitures, bidons, pneus, containers, pompes à essence ; **intérieur** (murs de labo gris, meubles) ; camp | terrain (4 sols bruns 16 px, 12 sentiers, 1 mur de falaise), **bâtiments modulaires** (29 + 15 pièces : toit béton, façade), granges (58 : toits bois et tuiles, façades), clôtures (31), buissons, arbres, route (26), voitures, tracteur, station-service, eau, pont |
| IU | barres de vie, panneaux, boutons | éléments d'IU (23), inventaire (15) |
| **Absent des deux** | **boss** ; ombre ; effet de mêlée « slash » ; **portail** ; auto-tuile de mur de caverne à 36 tuiles ; arme laser, plasma, mortier, canon rebondissant, traqueur, lames, disque, arsenal (aucun dessin) ; cible d'entraînement (Ittai : canette « Shootable Coke Can », 5 images, non extraite) | idem |

Écart d'échelle : les cases du jeu font 16 px, un personnage Aymen en occupe environ 20 × 26 sur 32, un personnage Ittai 12 × 15. Mélanger
les deux packs sur un même écran est le risque de style (déjà noté dans `polissage-pack-assets.md` § 3) : Aymen a un contour noir net, Ittai des aplats
sans contour.

### 1.2 Tableau classe × état × animations (avant)

Animations : I = idle, M = marche, A = attaque, D = mort.

| # | Classe | État | Source actuelle (fichier, licence) | Animations | Le pack couvre | Le pack ne couvre pas |
|---|---|---|---|---|---|---|
| 1 | Pilote (`player`, 4 joueurs) | **emprunté** | `sprites/characters/player/*` blackHUNTERdev CC BY 4.0, octet-identique dans gungeon, isaac, testbed, zombies ; une seule peau, aucune teinte par numéro (`hud.ron` sans `player_colors`) | I(4) M(4) D(4) A✗ | Aymen : Player I 6, M 8, D 8, touché 3 | attaque ; mains et arme (voir 3) |
| 2 | Ombre | emprunté (secondaire) | `characters/shadow/Shadow.png` blackHUNTERdev | statique | ✗ | à garder |
| 3 | Armes portées (3 `sprite_config` : `pistol`, `machine_gun`, `shotgun` pour **13 armes**) | **emprunté** | `weapons/Weapons.png` blackHUNTERdev, identique dans 5 jeux | statique | Aymen : pistolet, revolver, pompe, fusil d'assaut, sniper, lance-roquettes, hache, couteau (+ tir) | laser, plasma, mortier, canon rebondissant, traqueur, lames, disque, arsenal ; le joueur Aymen n'a pas de main : l'arme est un sprite séparé à poser |
| 4 | Rat, chien, rôdeur, pillard, cracheur, franc-tireur, arroseur (7 ennemis) | **emprunté** | **une seule** feuille, `enemies/zombie/Zombie.png` (blackHUNTERdev, identique dans zombies) : sept ennemis d'aspect identique | I(4) M(4) D(2) A✗ | Aymen : Zombie 1–4, Bandit : 6 peaux distinctes ; Ittai : enfant, maigre | attaque ; une peau propre par ennemi (6 peaux pour 7 + 2 ennemis) |
| 5 | Brute, buffle (2 ennemis) | **emprunté** | `ZombieHard.png` (identique dans zombies) | idem | Aymen : Zombie 4 ; Ittai : grand zombie | attaque |
| 6 | **Tourelle** | **emprunté** | `ZombieHard.png` + `marker` (teinte bleue, échelle 1,25) : un zombie qui bouge pas (dette D66, dernier remède) | I(4) M(4) D(2) | **Ittai : zombie tourelle** (idle 12, crachat 7, touché 9) | rien d'Aymen |
| 7 | Roi rat (boss de l'étage 3, champion) | **emprunté** | `ZombieHard.png`, `scale 1.4` (donnée), sans `marker` ni teinte | idem 5 | rien de propre au boss : Zombie 4 ou grand zombie agrandi (dérivé, jamais dessiné) | **boss propre** |
| 8 | Cible, mannequin (ennemis d'essai immobiles) | **absent** | aucune entrée dans `sprites.ron` : pas de sprite | — | Ittai : épouvantail (2), canette (5) ; Aymen : PNJ | à choisir |
| 9 | Projectiles (balles joueur et ennemi, 13 armes) | **absent** | aucune entrée `bullet_*` : carré **noir** de 3 px (`attach_bullet_visuals`) ; une seule image par équipe possible (le moteur ne choisit pas par arme) | — | Aymen : balle, lueurs orange et cyan, grenade, roquette, explosion, impact | image par arme (moteur : un id par équipe) ; couleur et taille par `tear_look` |
| 10 | Effet de mêlée (`slash`) | emprunté, **licence inconnue** | `effects/slash/Slash_strip3.png`, identique dans 5 jeux | 3 images | Ittai : coup de couteau (3 arcs, blancs) | — |
| 11 | Armes au sol (13) | **emprunté** | tuile Dungeon de l'entité LDtk `WeaponLocation` + petit sprite de `Weapons.png` ; aucune entrée `weapon/<id>` | statique | 8 icônes Aymen (voir 3) | 5 armes sans dessin |
| 12 | Power-ups (7 : 5 munitions, rage, vitesse) | **absent** | `powerup_visuals.rs` : cercle et croix en gizmo vert + texte ; pas d'entrée `powerup/<id>` | — | Aymen : caisses de munitions (4 types), batterie, bidon, trousse | rage, vitesse : à choisir (crâne, bidon) |
| 13 | Coffres (`Chest`) | **absent** | pas d'entrée `chest` | — | Aymen : 3 coffres (fermé, 7 images ; la planche d'effets liste aussi un coffre ouvert 32 × 16, non vérifié) ; Ittai : boîte d'objets qui apparaît | — |
| 14 | Objets et pièces au sol (`item`, `shell`) | **absent** | pas d'entrées `item/<id>` ni `shell` | — | Aymen : 36 objets 16 × 16 ; Ittai : pièces animées | — |
| 15 | Portail de l'étage (`portal`) | **absent** | `floors.rs` : carré violet de 40 px, id `portal` déjà demandé (`SpriteRequest`) | — | ✗ aucun portail dans les packs (Ittai : pont, eau) | **tout** : aucune image propre ; garder le carré, ou un assemblage (à décider) |
| 16 | Murs de caverne | **emprunté** | `caves/gabarit.ldtk` : `Walls` en IntGrid, rendu IntGrid de LDtk (aplat brun) ; tuiles Dungeon (Pixel_Poem) et SunnyLand ; pas de `wall_art` | statique | Ittai : toits de bâtiment et façades, falaise ; Aymen : murs d'intérieur | **auto-tuile de caverne** : recomposition (comme zombies) |
| 17 | Sol | **absent** | fond uni de LDtk (`#696A79`), aucune tuile | — | Aymen : sable de désert (40 tuiles 32 × 32 = 160 de 16 px) ; Ittai : 4 sols bruns | — |
| 18 | Portes, fenêtres, props, rochers | sans objet | les cavernes générées de throne n'en posent pas (à confirmer sur les captures : aucune n'apparaît ; le gabarit déclare `DoorHorizontal/Vertical`) | — | — | — |
| 19 | Repères d'auteur (`PlayerSpawn`, `CharacterSpawn`) | invisibles | `EnemySpawnBundle` sans sprite | — | — | `hide_markers` absent du `hud.ron` (b3, S2-8) |
| 20 | HUD (`hud.ron`) | texte propre, icônes **absentes** | barres de vie, de rads (rectangles), police FiraMono (OFL) ; pas d'`icons`, de `wall_art`, de `player_colors`, de `legend`, de `names` | — | Aymen : barres de vie, icônes de munitions et d'objets | `hud.ron` est à **b3** (S2-8) : tout passe par `ui/hud.local.ron` |
| 21 | Rads (jauge de progression) | texte + barre | `Bar(source: "rads")` rectangle vert | — | — | — |
| 22 | Mutations (écran de choix, 8) | texte | `ui/mutation_screen.html`, aucune icône | — | icônes Aymen possibles | optionnel |

Comptes (22 lignes) : **emprunté** 8 (1, 3, 4, 5, 6, 7, 11, 16) + 2 secondaires (2, 10) ; **absent** 8 (8, 9, 12, 13, 14, 15, 17, 20) ;
sans objet ou invisible 2 (18, 19), texte seul 2 (21, 22) ; **propre : 0**.
Animations : sans pack, l'ennemi n'a que I(4) M(4) D(2), un joueur n'a pas d'attaque ; tout throne tient sur **deux feuilles d'ennemis** (zombie et zombie dur) pour douze personnages.

### 1.3 Ce que j'envisage (par défaut, sans attendre)

| Classe | Source |
|---|---|
| Pilote | Aymen Player (I 6, M 8, D 8), teinte par numéro de joueur |
| Rat, chien, rôdeur, pillard, cracheur, franc-tireur, arroseur, brute, buffle | Aymen Zombie 1–4 + Bandit, une peau par rôle, teinte et échelle de présentation par `marker` (donnée) pour séparer les voisins : **6 peaux pour 9 ennemis** (limite dite telle quelle) |
| **Tourelle** | **Ittai : zombie tourelle** (idle 12, crachat 7 en `Attack` si la pose de présentation s'applique à un tireur, sinon idle seul), agrandi par un facteur entier |
| Roi rat | Aymen Zombie 4 agrandi + teinte chaude par `marker` (comme le boss de zombies) |
| Armes | pistolet, revolver, pompe, fusil d'assaut, sniper, lance-roquettes d'Aymen posés sur le joueur ; les 5 armes sans dessin gardent l'arme du type voisin, nommées « emprunté » |
| Projectiles, objets, coffres, power-ups | Aymen (balles, lueurs, caisses, coffres) |
| Murs et sol | sol : Aymen Wasteland (sable) ; murs : Ittai (toit béton + façade) recomposés en auto-tuile de `wall_art`, dans `ui/hud.local.ron` |
| Portail, cible, mannequin, slash | non couverts ou à choisir (Ittai : épouvantail, canette) |

### 1.4 Réestimation (fin J1)

Estimation initiale : 3 jours. Reste : persos, ennemis et tourelle (1 j), armes et projectiles (0,75 j), murs, sol, objets, coffres et icônes (1 j), captures 1/2/4 joueurs et lint avec et sans surcharge (0,5 j),
rapport et vérifications (0,5 j) : **3,5 à 4 jours** (au lieu de 3), plus l'étape 2 de S1-9 et l'interlude des distributeurs (fait).
