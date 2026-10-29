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

### 1.4 Réestimation (J1)

Fiche : 2 j ; plan : 3 j. Après audit : **≈ 4 j** (J1 audit 0,5 · J2 surcharge étendue + `HudIcon.image` + intégration
personnages/armes 1,25 · J3 fenêtres/portes/power-ups/halos/légende 1 · J4 atlas de murs + sol + captures + rapport 1,25).
Risque : l'atlas de murs (36 tuiles dans un ordre fixe) à composer **à partir de tuiles du pack seulement**.

## 2. Tableau après (J2–J3, 2026-10-28)

Deux états sont mesurés, chacun avec ses captures (voir §4) : **surcharge locale** (le pack TheLazyStone installé par `scripts/repack-postapocalypse.py … zombies`, hors dépôt) et **repli public**
(ce que voient la CI, les amis, un build web public : seulement les fichiers commités).

| # | Classe | Avant | Après, avec le pack | Après, repli public | Preuve (captures) |
|---|---|---|---|---|---|
| 1 | Joueur | emprunté | **propre** : Character/Main (I 6, M 6, D 6 images), teinté par numéro de joueur | emprunté (blackHUNTERdev, partagé) + couleur par numéro | `4j_f00700`, `2j_f00200` |
| 2 | Ombre | emprunté | emprunté (secondaire : le pack n'en a pas) | emprunté | — |
| 3 | Armes portées | emprunté | **propre** : Pistol, Gun, Shotgun (mains + arme) ; `rifle` reste sur le fusil à pompe | emprunté | `4j_tir_f00120` |
| 4 | Zombie, zombie coriace | emprunté | **propre** : Zombie_Small, Zombie_Axe (I 6, M 6–8, D 6, **A 4–7 jouée**, pose d'attaque de présentation) | emprunté | `4j_f00700`, `1j_f00650` |
| 5 | Zombie brute | emprunté, licence inconnue | **propre** : Zombie_Big (**A 8 jouée**) | `ZombieHard.png` (licence connue ; `Zombie2.png` retiré) | `1j_f00650` |
| 6 | Boss | emprunté | **propre, dérivé du pack** : Zombie_Big, teinte chaude et échelle 1,5 **par donnée** (`marker`, `boss_zombie_config.ron`) | `ZombieHard.png` + même `marker` | — (non capturé : salle finale hors des scénarios joués) |
| 7 | Projectiles | absent | **propre** : balle du joueur (Guns/Bullets ×3), crachat ennemi (Enemies/Shot ×2) | absent (carré teinté d'équipe, comme avant) | `4j_tir_f00120` |
| 8 | Coup de mêlée (`slash`) | emprunté, licence inconnue | **inchangé** (le pack n'en a pas) : D162 | inchangé | — |
| 9 | Armes murales / lâchées | emprunté | **propre** : icône Pistol / Gun / Shotgun + halo orange | emprunté + halo | `1j_f00100`, `4j_f00060` |
| 10 | Power-ups | absent | 2 sur 5 **propres** (`max_ammo` ← caisse de munitions, `carpenter` ← mur de bois), halo vert ; `insta_kill`, `double_points`, `nuke` **absents** (gizmo, D162) | absent ×5 + halo | scénario jeté `/tmp` (captures hors dépôt) |
| 11 | Distributeurs de perks | emprunté | **propre, CC0** : machines de karsiori, une par perk, repli public **et** surcharge (§ 7) | **propre** (même image) | `1j_f00200` |
| 12 | Portes | emprunté | **propre** : barricades de bois, `Idle` / `Open` / `Locked` (cadenas) ; **une paire collée = une porte double** (battants symétriques, une étiquette, rangée d'images pour les portes de 64) | emprunté (tuile Dungeon) ; une seule étiquette par paire (`double_doors`) | `4j_f00700`, `2j_f00200` |
| 13 | Fenêtres | emprunté | **propre**, 4 états d'après les PV (`Idle`, `Damaged`, `Critical`, `Broken`) : image du pack ×2 entier (30 × 30) + teinte d'état ; la barre de 2 px reste, secondaire | emprunté + barre verte (comme avant) | `surcharge-locale_fenetre_4_etats_…` (hors dépôt) |
| 14 | Barricades | sans objet | sans objet | sans objet | — |
| 15 | Murs | emprunté | **propre** : toit rouge (Roof_TileSet) + face de brique (Brick-Wall_TileSet), `wall_art` sur la couche `Walls` de **toutes** les cartes de zombies | emprunté (Dungeon / SunnyLand) | `4j_f00130`, `1j_f00100` |
| 16 | Sol | emprunté / absent | **propre** : 4 tuiles de Background_Bleak-Yellow posées sous le sol que la carte peint (`mask_layer: Sol` ; toute case non murée pour les cartes sans couche `Sol`, voir « Limite du sol » ci-dessous) | emprunté / fond uni (comme avant) | idem |
| 17 | Mobilier de la fonderie | emprunté | **remplacé** : la couche de tuiles `Mobilier` (brique de mur Dungeon) est cachée, `wall_art` dessine les murs | emprunté | idem |
| 18 | Props table/baril, portail | sans objet | sans objet | sans objet | — |
| 19 | Repères d'auteur | visibles pour `CharacterSpawn` | **cachés** (`hide_markers`) | cachés | — |
| 20 | HUD | absent (icônes) | **propre** : 4 icônes de perks (First-Aid-Kit, Canned-soup, Bullet-box, Canned-food) via `HudIcon.image` ; barre de vie inchangée | carrés de couleur + lettre (comme avant) | `1j_f00200` |
| 21 | Lisibilité coop | absent | **fait** : `player_colors`, légende F7, halos, noms F6, `hide_markers` | **fait** (même données publiques) | `4j_legende_f00040`, `4j_f00700` |

Classes principales avec le pack (joueur, zombies, boss, armes, projectiles, portes, fenêtres, murs, sol, HUD) : **aucune case « emprunté »**. Restent hors du pack, dites telles : ombre (secondaire), `slash`, distributeurs de perks,
trois power-ups (D162) ; l'animation d'attaque, absente du moteur au §1, est jouée par une **pose de présentation** (§ 2 bis) ; la ligne « A » du §1 est donc pleine avec le pack, vide en repli public.
Repli public : les classes 1–6, 9, 12, 13, 15–17 restent **empruntées** (l'état d'avant, avec `Zombie2.png` en moins), mais la lisibilité coop (21) et le halo (9, 10) sont aussi acquis sans le pack.

### 2 bis. Suite de S1-8 (portes doubles, fenêtres, zombie qui frappe ; 2026-10-28)

**Portes (audit).** Dans les `.ldtk` bruts aucune porte n'est collée à une autre (chacune est au bord de sa salle) ; la paire naît à l'assemblage. Sondage de présentation jeté (`DOORPROBE` dans `world_sprites`, retiré) sur les cartes assemblées :
`avant_poste` **8 paires** (ids 1-2, 3-4, 5-6, 7-8, 9-10, 11-14, 12-15, 13-16 ; collider 16 × 32), `la_fonderie` **12 paires** (collider 16 × 64 ou 64 × 16), `buy_door` (`test_map_shop`) 1. Toutes les paires sont **jumelles** (`paired_door` mutuel), à 16 unités l'une de l'autre, avec des prix
différents (750 / 1000, 1000 / 1250…). **Simulation** : deux entités rollback, deux colliders, deux `Interactable`, deux prix ; `handle_door_interaction` fait payer la porte actionnée seule et retire `Collider` / `CollisionLayer` / `Interactable` aux deux dans la même frame (`paired_door`) : elles s'ouvrent toujours ensemble.
**Conclusion : ni carte ni simulation à changer** (aucun `ask`, aucune trace, aucun `BLESS`).
**Remède en présentation** (`hud.ron` `double_doors: true`, faux par défaut : gungeon inchangé) : `DoorPair` (composant de présentation, hors rollback) posé par `pair_doors` d'après `door_pairs` (pure, testée : désignation mutuelle **et** centres à moins de 40 unités), le second battant retourné (`mirror_door_pairs`), une étiquette F6 au milieu de la paire
(`legibility`), une rangée d'images pour une porte plus longue que son image (les portes de la fonderie : 64 pour une image de 32 ; avant, la moitié de l'ouverture n'avait pas d'image). **État affiché** : chaque battant suit l'état de sa porte (`Idle` / `Locked` / `Open`) : si les deux états diffèrent un jour, l'image le montre ; en jeu ils sont toujours égaux.
Captures (hors dépôt) : `surcharge-locale_porte_double_fermee_f00140.png`, `…_ouverte_f00200.png` (`buy_door`), `surcharge-locale_1j_fonderie_portes_f00900.png` (portes de 64, deux rangées).

**Fenêtres.** Les quatre états existaient mais l'image de 15 px les rendait indiscernables au jeu (≈ 2,4 px par unité à 960 × 540 : planches d'un pixel). Corrigé : même image de 32 × 32 pour `window_h` et `window_v` (fenêtre du pack ×2 entier, 30 × 30, sans pixel dessiné) et **teinte d'état**
(`window_tint` : blanc, chaud, rouge, sombre). Orientation vérifiée : une entité plus large que haute prend `window_v`, l'autre `window_h` ; les deux ont la même image, donc rien à tourner. Capture des quatre états, fenêtre `window_repair` à f300 (intacte), f372 (abîmée), f392 (critique), f450 (brisée) :
`surcharge-locale_fenetre_4_etats_Idle-Damaged-Critical-Broken_f300-372-392-450.png`. **Barre de vie** : conservée (2 px, secondaire) ; le sprite porte l'état, la barre la valeur exacte. **Repli public** : inchangé (tuile de la carte + barre).
Limite : la fenêtre d'un mur qui court en largeur (la seule vue dans les scénarios) montre sa face ; le pack n'a pas de fenêtre vue de côté, `window_h` en mur vertical reste approximée (D163).

**Pose d'attaque.** `animation::PresentationClip` (clé de clip de présentation, hors rollback, même type que le `Shoot` de X-4b) ; `character::visuals::present_attacking` (`Update`) la pose à `"Attack"` tant que `MonsterState::Attacking { last_attack_frame }` date de moins de `ATTACK_POSE_FRAMES` = 48 frames (le délai entre deux coups du zombie est de 45),
sinon `None` ; `animate_sprite_system` joue cette clé **si** la carte d'animation la définit (`AnimationMapConfig::playing_key`) : la surcharge du pack oui (`Attack` de `zombie_small` 4 images, `zombie_axe` 7, `zombie_big` 8), les feuilles publiques non (rien ne change). Elle couvre l'attaque d'une fenêtre **et** d'un joueur (même état).
Tests : `un_coup_reste_en_cours_la_duree_de_la_pose`, `la_pose_d_attaque_derive_de_l_etat_d_attaque` ; vu en jeu (sondage jeté : `Attack` posé à f317, retiré à f365 sur `window_repair`).
**Où ça rejoint X-4b** : la branche `wquintal/x-4b-import-sprite` définit le même `PresentationClip` et `playing_key` dans `crates/animation/src/lib.rs` et une pose `Shoot` dans `character/visuals.rs` (`present_shooting`, `SHOOT_POSE_FRAMES`). Conflit attendu à ces deux endroits : **garder une seule définition** de `PresentationClip` / `playing_key` (la leur gère aussi `clips`, durée et boucle par clip), ajouter `present_attacking` à côté de `present_shooting` ; ils ne se disputent aucune entité (un zombie n'a pas d'arme à distance).
**Le reste de la séquence** (la fenêtre cède, le zombie entre) est déjà de la simulation, inchangée. **Vidéo** (hors dépôt, 14 s à demi-vitesse, 420 images, étiquettes coupées) : `~/orca/workspaces/alacod/captures/polissage-zombies-visuel/surcharge-locale_fenetre_zombie_frappe_brise_entre_demi-vitesse.mp4` (scénario jeté dérivé de `window_repair` sans réparation, joueur immobile :
le zombie frappe la fenêtre avec la hache levée, la brise, entre et attaque le joueur).

### Limite du sol (retouche de relecture, 2026-10-28)

Constat du coordinateur : avec la surcharge, le sol couvrait aussi le hors-salle (`surcharge-locale_4j_f00700.png`, dallage jusqu'aux bords de l'écran). Deux cas, mesurés (`DBG` jetable dans `draw_walls`, retiré) :

- **`la_fonderie`** (le jeu : `start_map: floor:la_fonderie`) : les niveaux assemblés gardent leur couche `Sol` (441 à 1 067 tuiles par niveau) ; `FloorArt::mask_layer: Some("Sol")` limite le sol aux cases qu'elle peint. Capture
  `surcharge-locale_1j_fonderie_bot_f00300.png` (`la_fonderie_demo`) : sol dans les salles, **extérieur resté sombre**. Test `le_sol_peint_par_la_carte_laisse_le_hors_salle_vide`.
- **`test_map`** (scénarios `clone_*`, `four_players_*`, dont la capture citée), `avant_poste`, `arene_vide` : aucune couche `Sol` (le niveau n'a que `Entities`, `LevelConnection`, `Walls`). Le « hors-salle » n'y est pas distinguable d'une salle : la grille de `test_map` est ouverte
  (lignes entières sans mur) et **ces cases sont franchissables** (seuls les murs de `Walls` ont un collider) ; le sol reste donc sous toute case non murée, sinon un joueur marcherait sur le fond du niveau. **Voulu** : c'est la carte de test, pas celle du jeu.
  Le dallage étendu de la capture citée est donc conservé pour ces cartes ; les captures `*_4j_f00700`, `*_1j_*`, `*_2j_*` y sont identiques (octet pour octet, sauf `1j_f00650`, zombies non déterministes à l'image).

## 3. Par point de la fiche

| Point | Issue | Preuve |
|---|---|---|
| 1. Audit chiffré | fait (J1, relu par le coordinateur) | §1, commit `199ce4ac` |
| 2. Sol et murs propres : `wall_art`, fenêtres, portes | fait avec la surcharge locale ; repli public inchangé | `wall_art.floor` + `hide_layers` (§92), tests `le_sol_couvre_les_cases_vides_et_seulement_elles`, `la_tuile_de_sol_ne_depend_que_de_la_position`, `l_etat_visuel_d_une_fenetre_suit_sa_sante` ; captures §4 |
| 3. Joueur et ennemis propres, animés | fait avec la surcharge locale (I/M/D ; **A non jouée** : le moteur n'a pas d'état d'attaque) ; boss dérivé par donnée | captures §4 |
| 4. Lisibilité coop : `player_colors`, `legend`, halos | fait, public | `repli-public_4j_legende_f00040`, `repli-public_4j_f00700` |
| 5. Captures 1/2/4 joueurs | faites : `clone_solo`, `clone_duo`, `clone_quad`, `four_players_shooting`, `four_players_idle` ; **vidéo non faite** (J5 du plan : « captures et vidéo 60 s zombies ») | §4 |
| `HudIcon` image (`Icons`, `Hearts`, `Counter`) | fait ; `Icons` vu en jeu (icône du Juggernog), `Hearts` et `Counter` non vus en jeu (zombies n'en a pas) mais testés (`a_heart_image_needs_every_state_it_draws`) | tests `crates/game/src/ui/hud.rs` |
| Surcharge locale étendue (`hud.local.ron`, ids ajoutés) | fait ; lint vert avec et sans | fixtures `hud_local_overlay_missing_image`, `sprite_sheet_local_overlay` |

## 4. Captures

Sources : `tests/scenarios/{clone_solo,clone_duo,clone_quad,four_players_shooting,four_players_idle}.ron`, `play_scenario --capture … --at … --follow 0`, `ALACOD_MUTE=1`, binaire `.target-mac/debug/play_scenario`.

- **Repli public** (commité) : `docs/captures/polissage-zombies-visuel/repli-public_<n>j_f<frame>.png` (1 joueur : f100, f200, f650 ; 2 : f60, f200, f600 ; 4 : f60, f130, f700 ; tir à 4 : f120 ; légende F7 à 4 : f40) et
  `avant-origin-main_4j_idle_f00590.png` (état de `origin/main` avant le lot).
- **Surcharge locale** (hors dépôt ; refaites après la retouche du sol : les 11 de `clone_*`/`four_players_*`, identiques sauf `1j_f00650` ; **nouvelles** : `surcharge-locale_1j_fonderie_bot_f00150/f00300/f00900.png`, le seul changement visible), décision de William du 2026-10-28 : rien de payant dans le dépôt public, même en capture) : `~/orca/workspaces/alacod/captures/polissage-zombies-visuel/surcharge-locale_<n>j_f<frame>.png`
  (mêmes scénarios et frames) et `avant-surcharge-locale-ancienne_4j_idle_f00590.png` (l'ancienne surcharge de `main`, sans murs ni sol). Elles iront dans le dépôt privé d'assets quand il existera.
- Captures d'états rares (fenêtre `window_repair` f300–545, power-ups d'un scénario jeté dérivé de `clone_solo`) : `/tmp/b1/after/` (non conservées).

## 5. Ce qui reste (estimation)

| Reste | Estimation |
|---|---|
| ~~Distributeurs de perks~~ : **faits** (interlude du 2026-10-29, § 7) | — |
| Images pour `insta_kill`, `double_points`, `nuke` et pour `slash` (aucune dans le pack) | 0,5 j une fois choisies |
| Vidéo de 60 s zombies, repli public et surcharge locale (J5 du plan) | 0,5 j |
| D161 : exclure la surcharge locale des trois copies d'assets d'un build web (`build-web-games.mjs`, `embed-game-content.rs`, `Makefile`) ; **à faire avant tout build web** d'un poste qui a le pack | 0,25 j |
| Portes vues de dessus, fenêtre d'un mur vertical, troisième planche, sol varié (D163) : sans pack mieux adapté, rien | — |
| Boss : capture dans la salle finale (marqueur D66 vu seulement dans le code) | 0,25 j |
| Dépôt privé d'assets : reprendre l'arborescence du §92 de `docs/conventions.md` | à faire par le coordinateur |

## 6. Preuves d'exécution

Commit de tête de la vérification : `28660d93` (Mac arm64, `CARGO_TARGET_DIR=~/orca/workspaces/alacod/.target-mac`, `env -u CI`).

```
$ make format check_forbidden check_rollback_registration check_sim_boundary test_crates test_scenarios_par lint     (EXIT=0, 22 min)
OK: aucun nouveau motif interdit ; 51 occurrences auditées dans determinism-allowlist.json
OK: aucun appel direct à rollback_component_with*/rollback_resource_with* hors de crates/utils/src/rollback.rs
OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69
cargo test -q --workspace --profile headless --no-default-features --no-fail-fast -- --exact --skip scenarios   (93 « test result: ok », 0 failed ;
  content/lint_fixtures : 176 passed, dont hud_local_overlay_fixture_reports_missing_atlas_and_icon_image et sprite_sheet_local_overlay_replaces_and_adds_ids)
tranche 0/4 : ok   tranche 1/4 : ok   tranche 2/4 : ok   tranche 3/4 : ok
métriques : 810 scénarios fusionnés ; suite en 4 tranches : 438 s        (sans BLESS : aucune trace n'a bougé)
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)    + testbed, throne, gungeon, isaac : aucune erreur
$ alacod lint games/zombies   avec puis sans sprites.local.ron / hud.local.ron : « aucune erreur » dans les deux cas ; lint_fixtures sans surcharge : 176 passed
```

`make check` complet n'est **pas** vert sur ce Mac pour une raison d'environnement sans rapport avec le lot : `make test_tools` échoue sur 3 tests de `scripts/tests/test_claude_say.py` (le test fige `/usr/bin/python3`, qui est le
Python 3.9.6 du système ici, et `scripts/claude-say.sh` exige 3.10 ; fichiers non touchés). Les 35 autres tests Python passent. Le reste de `make check` (liste ci-dessus) est vert ; `test_scenarios_par` remplace `test_scenarios` (même suite, 4 tranches,
recommandé par le `Makefile`).

Unitaires ajoutés : `wall_art::{le_sol_couvre_les_cases_vides_et_seulement_elles, la_tuile_de_sol_ne_depend_que_de_la_position}`, `world_sprites::{l_etat_visuel_d_une_fenetre_suit_sa_sante, un_power_up_cherche_son_id_puis_le_generique}`,
`hud::{the_local_overlay_replaces_wall_art_and_merges_icons_by_id, a_heart_image_needs_every_state_it_draws}`, fixture de lint `hud_local_overlay_missing_image`.

### Suite de S1-8 (portes doubles, fenêtres, pose d'attaque)

```
$ make format check_forbidden check_rollback_registration check_sim_boundary test_crates test_scenarios_par lint     (EXIT=0)
OK: aucun nouveau motif interdit ; OK : aucun appel direct à rollback_*_with* ; OK : frontière simulation/présentation respectée par 13 crates
93 « test result: ok » (test_crates), 0 failed ; tranche 0/4 … 3/4 : ok ; suite en 4 tranches : 333 s (sans BLESS : aucune trace n'a bougé)
games/{zombies,testbed,throne,gungeon,isaac} : aucune erreur ; lint zombies avec et sans sprites.local.ron / hud.local.ron : « aucune erreur »
$ schedule_order_diff (base origin/main, tête = arbre de travail ; base et tête construites l'une après l'autre avec un `touch` entre les deux, D133) : diff -r base head → IDENTIQUE (5 jeux : schedules et types rollback)
```

`make test_tools` reste en échec sur ce Mac pour les 3 tests de `test_claude_say.py` (`/usr/bin/python3` = 3.9.6, le script exige 3.10), sans rapport avec le lot. `make schedule_order_diff` tel quel échoue à la tête (D133 : artefacts de la base plus récents) ; contourné comme ci-dessus.
Unitaires ajoutés : `world_sprites::{deux_portes_collees_qui_se_designent_forment_une_porte_double, chaque_etat_de_fenetre_a_sa_teinte}`, `visuals::attack_pose_tests::{un_coup_reste_en_cours_la_duree_de_la_pose, la_pose_d_attaque_derive_de_l_etat_d_attaque}`.

## 7. Interlude : distributeurs de perks en CC0 (2026-10-29, branche `polissage-zombies-distributeurs`)

Décision de William (29 oct) : les distributeurs prennent une source CC0, commitable (repli public **et** surcharge).

**Candidates**

| | Source | Licence (lue) | Contenu | Échelle |
|---|---|---|---|---|
| **1 (retenue)** | « Pixel Art Vending Machines », karsiori, https://karsiori.itch.io/pixel-art-vending-machines (archive de 130 ko téléchargée et ouverte le 2026-10-29) | **CC0**, page itch (section « License info »), usage commercial libre ; l'archive n'a pas de fichier de licence | 6 machines, 45 variantes, vitre / sans vitre, vue de face, 45 × 94 à 50 × 89 px | réduite à 48 px de haut (Lanczos), 1 case = 16 px, joueur 20–32 px |
| 2 | Kenney « Roguelike Modern City », https://kenney.nl/assets/roguelike-modern-city | CC0 (page relue) | 1 036 tuiles urbaines 16 × 16 | **non retenue** : l'aperçu ne montre aucun distributeur reconnaissable, archive non ouverte (téléchargement refusé sans navigateur) |

**Choix** : candidate 1, quatre variantes de la couleur de l'icône du perk (Juggernog rouge = 3.1, Speed Cola vert = 3.2,
Double Tap orange = 1.1, Stamin-Up jaune-olive = 3.3). Script `scripts/repack-distributeurs.py` (réduction seule, aucun pixel
dessiné), images `games/zombies/assets/sprites/props/perk_machine_<perk>.png` **commitées**, entrées `perk_machine/<perk>` de
`sprites/sprites.ron`, entrée `vending-machines-karsiori` d'`assets.yaml` avant l'import.

**Moteur** (présentation seule) : `world_sprites::attach_world_sprites` pose sur l'entité `PerkMachine` l'image
`perk_machine/<perk>` puis `perk_machine` (`perk_machine_candidates`, test) ; `hide_soda_tiles` rend transparente la tuile de
l'entité LDtk `SodaLocation` quand l'une de ces images existe (sans image : la tuile reste, comme avant).

**Avant / après** : « emprunté » (tuile Dungeon de l'éditeur) → « propre, CC0 » ; captures `docs/captures/polissage-zombies-distributeurs/repli-public_1j_f00200.png`
(commitée), surcharge dans `~/orca/workspaces/alacod/captures/polissage-zombies-distributeurs/`. Ferme la partie « distributeurs » de D162 ;
restent `slash` et trois power-ups.
