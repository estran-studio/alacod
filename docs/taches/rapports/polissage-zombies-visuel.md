# polissage-zombies-visuel (S1-8) — rapport

Branche `polissage-zombies-visuel`, clone `zombies`, fiche `docs/taches/polissage-zombies-visuel.md`. Présentation seule : aucune trace ne bouge.

## 1. Audit J1 (état « avant », 2026-10-28, `origin/main` 00dc83ce)

Sources lues : `games/zombies/assets/{sprites/sprites.ron, assets.yaml, ui/hud.ron, maps/*.ldtk, ZombieShooter/Sprites/**}`,
code de rendu (`world_sprites.rs`, `legibility.rs`, `powerup_visuals.rs`, `interaction_view.rs`, `map_ldtk::wall_art`,
`map_ldtk::game::entity/*`), pack `~/Downloads/PostApocalypse_AssetPack_v1.1.2` (lu seulement), captures
`play_scenario` du 2026-10-28 (repli public, puis avec la surcharge locale du pack).

**Définitions.** *propre* : image dessinée pour ce rôle, propre au clone, licence lisible dans `assets.yaml`.
*emprunté* : tuile d'un atlas d'exemple ou d'un pack générique, ou fichier **octet-identique dans d'autres jeux**
(md5), ou licence inconnue. *absent* : aucune image (carré de couleur, gizmo, texte, tuile d'éditeur cachée).
« Moteur » : états d'animation que le moteur joue réellement : `Idle`, `Run`, `Dying_*` (`update_animation_state`,
`character/visuals.rs`) ; **aucun état `Attack`** n'existe côté simulation (`AnimationState` est rollback : on n'en
ajoute pas ici). Une animation d'attaque présente dans une feuille n'est donc jamais jouée.

### 1.1 Tableau classe × état × animations (avant)

Animations : I = idle, M = marche, A = attaque, D = mort ; nombre d'images entre parenthèses.

| # | Classe | État | Source actuelle (fichier, licence) | Animations présentes | Le pack couvre | Le pack ne couvre pas |
|---|---|---|---|---|---|---|
| 1 | Joueur (`player`, calques body/shirt/hair) | **emprunté** | `sprites/characters/player/*` blackHUNTERdev CC BY 4.0, **identique** (md5) dans gungeon, isaac, testbed, throne ; 1 seule peau pour 4 joueurs, pas de teinte par handle | I(4) M(4) D(4) A✗ ; `Dying_1` = images 8–11 sur une feuille de 10 colonnes (2 hors feuille, **à vérifier**) | Character/Main : I(6) M(6) D(6) + Punch(4) + Pick-up(3), vues côté/haut/bas, casque | attaque non jouée (moteur) ; teinte par handle à vérifier sur ce sprite sombre |
| 2 | Ombre (`shadow`) | emprunté (secondaire) | `characters/shadow/Shadow.png` blackHUNTERdev, identique dans 5 jeux | statique | ✗ (aucune ombre) | à garder (disque 16 px, hors périmètre) |
| 3 | Armes portées (`pistol`, `machine_gun`, `shotgun`) | **emprunté** | `weapons/Weapons.png` (14×8, 4 lignes) blackHUNTERdev, identique dans 5 jeux ; `rifle` réutilise `shotgun` | statique | Guns/Pistol, Gun, Shotgun (main + arme, tir, rechargement), Bat | `rifle` (reste sur `shotgun`) ; tir/recharge non joués |
| 4 | Zombie 1 (`zombie_1`) et dur (`zombie_2`) | **emprunté** | `enemies/zombie/Zombie.png` (identique dans throne), `ZombieHard.png` ; blackHUNTERdev | I(4) M(4) D(2) A✗ | Zombie_Small : I(6) M(6) D 2×(6–7) A 2×(4–11) ; Zombie_Axe : I(6) M(8) D A(7) | attaque non jouée (moteur) |
| 5 | Zombie complet (`zombie_full`) | **emprunté, licence inconnue** | `enemies/zombie/Zombie2.png` (288×384), entrée `zombie-shooter-source-inconnue` | I(8) M(5) D(1 + 5) A✗ | Zombie_Big : I(6) M(8) D 2×(7–8) A 2×(8–15) | idem |
| 6 | Boss (`boss_zombie`, salle `boss`) | **emprunté** | même feuille que `zombie_full`, id distinct, échelle 1,0 | idem 5 | seulement Zombie_Big agrandi (le plan, §4 de `polissage-pack-assets.md`, le dit « à fabriquer ») | **boss propre** : à décider (agrandir/teinter le Big = sprite dérivé du pack, jamais dessiné) |
| 7 | Projectiles (balles joueur et ennemi) | **absent** | aucune entrée `bullet_*` dans `sprites.ron` : `attach_bullet_visuals` pose un carré teinté par équipe | — | Guns/Bullets (1–4 px), Enemies/Shot (crachat, 3 images) | balle lisible à 32 px d'échelle : à régler à l'intégration |
| 8 | Effet de mêlée (`slash`) | emprunté, **licence inconnue** | `effects/slash/Slash_strip3.png`, identique dans 5 jeux | 3 images de coup, `frame_duration: 3` | ✗ | effet de coup (le pack n'a que l'animation Punch du joueur) |
| 9 | Armes murales et lâchées | **emprunté** | tuile Dungeon (80,128) de l'entité LDtk `WeaponLocation` + le petit sprite de `Weapons.png` ; **aucune** entrée `weapon/<id>` | statique | icônes Pistol, Gun, Shotgun, Bat (`UI/Inventory/Objects`, `Objects/Pickable`) | `rifle` |
| 10 | Power-ups (5) | **absent** | `powerup_visuals.rs` : cercle + croix en gizmo vert + texte ; `PowerUpPickup` n'entre pas dans `attach_world_sprites` | — | `max_ammo` ← Ammo-crate ; `carpenter` ← icône Wooden-wall | `insta_kill`, `double_points`, `nuke` ✗ ; chemin moteur : `world_sprites` ne demande pas `powerup/<id>` |
| 11 | Distributeurs de perks (`SodaLocation`, 4 perks) | **emprunté** | tuile Dungeon (144,128) de l'éditeur LDtk | statique | **✗** : aucun distributeur ni machine dans le pack (contrairement au §2.1 de `polissage-pack-assets.md`) | **tout** : décision de William (voir §1.3) |
| 12 | Portes (`DoorHorizontal/Vertical`) | **emprunté** | tuiles Dungeon (96,96 32×16 ; 128,64 16×32) de l'éditeur ; pas d'entrée `door_h/door_v` : un seul état visible (fermée), ouverte = cachée | I(1) | Door_1…6 (beige, entrouverte, condamnée, métal, rouillée), Gate bois/renforcé + animation 7 images, grillage + cadenas | aucun (états `Idle`/`Open`/`Locked` gérés par `world_sprites`) |
| 13 | Fenêtres (`WindowHorizontal/Vertical`, 3 PV) | **emprunté** | tuiles Dungeon (144,64 16×32 ; 80,112 32×16) + barre verte de 2 px ; **aucun état visuel** (intacte / abîmée / cassée) | I(1) | 21 fenêtres : intacte, **condamnée**, brisée, ouverte (3 teintes de façade) | état par PV : **chemin moteur manquant** (`world_sprites` ne gère que les portes) |
| 14 | Barricades | sans objet | aucune entité `Barricade` posée par les cartes de zombies (la fenêtre est la barricade ; `Obstacle::barricade` ne sert qu'aux tables) | — | Wooden/Reinforced wall (déjà là si besoin) | — |
| 15 | Murs | **emprunté** | `la_fonderie.ldtk` : autotuile `Walls` = 4 tuiles (1, 10, 15, 41) de `Dungeon_Tileset.png` (Pixel_Poem, licence propre, partagée avec gungeon, isaac, testbed) ; `avant_poste.ldtk`, `arene_vide.ldtk` (bac à sable) : tuiles de **SunnyLand** (Ansimuz, plateforme) ; pas de `wall_art` | statique | Tiles/Buildings ×4 couleurs, Brick-Wall, Roof, Wire-Fence | l'**auto-tuile à 36 tuiles** de `wall_art` n'existe dans aucun pack : recomposition (§1.3) |
| 16 | Sol | **emprunté** (fonderie) / **absent** (autres) | `la_fonderie` : couche `Sol`, 9 tuiles Dungeon (6–8, 16–18, 26–28) ; `avant_poste`, `arene_vide` : aucun, fond uni `#696A79` | statique | Background_*_TileSet (24×17 tuiles de 16 px : dalles, pavés, béton, 3 teintes) | un « remplissage de sol » n'existe pas côté moteur pour les cartes sans couche `Sol` |
| 17 | Mobilier (couche `Mobilier`, fonderie) | **emprunté** | 3 tuiles Dungeon (2, 3, 4), 1 342 instances | statique | Tiles/Buildings (fauteuil, meubles), Container ×24, Vehicles | — |
| 18 | Props table/baril, portail | sans objet | aucune carte de zombies n'en pose (`Table`, `Barrel`, `Portal` : mode `Waves`) | — | — | — |
| 19 | Repères d'auteur (`ZombieSpawn`, `PlayerSpawn`, `CharacterSpawn`) | invisibles | `EnemySpawnBundle` n'a pas de sprite ; `hide_markers` **absent** du `hud.ron` de zombies (les `CharacterSpawn` des 2 salles scriptées seraient visibles) | — | — | `hide_markers: true` à poser |
| 20 | HUD (`hud.ron`) | **absent** (icônes) / propre (texte) | barre de vie = rectangle, perks = carré de couleur + lettre (`icons`), police FiraMono (SIL OFL, propre) ; `HudIcon` n'a que `color` + `label` | — | HP-Bar, cœurs plein/demi/vide, indicateurs de balle (pistolet, mitraillette, fusil), icônes d'inventaire (trousse, caisse de munitions, bandage) | icônes des 4 perks (Juggernog, Speed Cola, Double Tap, Stamin-Up) ✗ ; `HudIcon` image : **moteur** |
| 21 | Lisibilité coop | **absent** | `hud.ron` de zombies : pas de `player_colors`, `legend`, `hide_markers`, `names` ; pas de feuille `halo` dans `sprites.ron` (donc `attach_halos` retourne tout de suite) : 4 joueurs identiques à l'écran sauf l'étiquette « J n » | — | `halo` : disque doux générique CC0 (`games/gungeon/assets/sprites/lisible/`, commité, repris tel quel) | — |

Comptes (21 lignes) : **emprunté** 14 (1–6, 8, 9, 11–13, 15, 16 en partie, 17) ; **absent** 4 (7, 10, 20 pour les icônes, 21) ;
sans objet ou invisible 3 (14, 18, 19) ; **propre : 0** (hors la police du HUD).
Animations : le moteur ne joue que `Idle`/`Run`/`Dying_*` : la colonne « A » est vide par construction, quelle que soit la feuille.

### 1.2 Ce que la surcharge locale du pack couvre déjà (mesuré)

`python3 scripts/repack-postapocalypse.py <pack> zombies` écrit `sprites/postapo/*.png` et `sprites/sprites.local.ron`
(gitignorés) : lignes **1, 3 (3 armes sur 4), 4, 5, 6 (le Big agrandi)**. Capture `/tmp` du 2026-10-28 : les quatre joueurs
restent identiques et sombres (pas de `player_colors`). Rien d'autre n'est couvert (images de HUD, murs, sol, fenêtres, portes,
power-ups, balles, halo) : la surcharge ne vise que `sprites.ron`.

### 1.3 Choix proposés (pour William, via le coordinateur)

Le dépôt est public ; seul `sprites.ron` a une surcharge. Trois options pour murs, sol, fenêtres, HUD :

- **A (défaut)** — étendre la surcharge : `ui/hud.local.ron` (gitignoré, même principe que `sprites.local.ron`) pour
  `wall_art`, `icons[].image`, `floor` ; les PNG du pack dans `sprites/postapo/` (gitignoré) ; repli public commité =
  l'état actuel (sans `wall_art`, icônes carrées, tuiles Dungeon). Moteur : `HudIcon.image`, fusion de `hud.local.ron`,
  extension `wall_art.floor` (remplissage de sol, `map_ldtk::wall_art`), états de fenêtre et `powerup/<id>` dans `world_sprites`.
- **B** — repli public **propre** aussi : un atlas CC0 de remplacement (Kenney) pour que la CI et les amis voient un jeu sans
  tuiles empruntées. Plus de fichiers, un second style à tenir ; non retenu sans décision.
- **C** — commiter les images payantes : **exclu** (dépôt public, « No Redistribution »).

Décisions à prendre (défaut entre parenthèses) : distributeurs de perks absents du pack (garder la tuile Dungeon, marquée
« emprunté » au tableau final / aucune image)  ; boss (Big agrandi et teinté par donnée / rien) ; build web public avec
les images du pack (attend la confirmation écrite de l'auteur, décision 1 du § 0.3).

### 1.4 Réestimation

Fiche : 2 j ; plan : 3 j. Après audit : **≈ 4 j** (J1 audit 0,5 · J2 surcharge étendue + `HudIcon.image` + intégration
personnages/armes 1,25 · J3 fenêtres/portes/power-ups/halos/légende 1 · J4 atlas de murs + sol + captures + rapport 1,25).
Risque : l'atlas de murs (36 tuiles dans un ordre fixe) à composer **à partir de tuiles du pack seulement**.

(Les sections 2 « Après » et 3 « Preuves » sont ajoutées au fil des jalons.)
