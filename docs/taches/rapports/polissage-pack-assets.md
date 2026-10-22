# Packs d'assets pour les quatre clones (polissage, voie « sprites propres »)

Rapport de recherche du 2026-10-22, écrit pour la décision d'achat de William. Les pages de packs ont été lues
le jour même (texte de la page, JSON de la page itch pour le prix minimal, fichier `tile_list` du pack 0x72,
PDF de conditions de Pixel Crawler, PDF de licence Sonniss). Tout ce qui n'a **pas** pu être lu sur la page est
marqué `[À CONFIRMER]` ; tout ce qui est déduit est marqué `[INFERENCE]`. Les prix sont les **prix minimaux en
dollars** affichés par itch (`min_price`, hors TVA et frais de plateforme) ; ils peuvent bouger (soldes).

Contraintes de William rappelées : usage commercial, portage console possible
(`docs/plan-engine.md` §7), ni CC BY-NC ni GPL pour les assets, attribution acceptable, vue de dessus,
pixel art, tuiles de 16 px, personnages en cellule 32×32 (collider 20×20).

Aucune des licences lues ci-dessous ne mentionne de restriction de plateforme (console comprise) ; seule la
CC0 en donne la certitude. Pour les licences « maison » d'itch, une ligne de confirmation écrite de l'auteur
(« oui, embarqué dans un jeu console ») coûte peu et ferme le doute.

---

## 1. Besoins (tableau « classe d'entité × clone »)

Sources : `games/*/assets/characters/*.ron`, `games/*/assets/sprites/sprites.ron`, `games/*/assets/assets.yaml`,
`games/*/assets/ui/hud.ron`, `games/*/assets/maps/*.ldtk` (tilesets), `docs/taches/rapports/plan-polissage.inventaire.md`
§3, §10, §12, `docs/conventions.md` §2 et §83.

Légende : **propre** = asset définitif ; **remplacer** = emprunté, provisoire ou généré en attendant ;
**absent** = aucune image n'existe (carré de couleur, tuile vide ou rien).

| Classe d'entité | zombies | throne | gungeon | isaac |
|---|---|---|---|---|
| **Joueur** (teinté par handle, 4 joueurs) | 1 — **remplacer** : calques `Body/Hair/Shirt` de blackHUNTERdev (CC BY 4.0, placeholder) ; surcharge privée TheLazyStone non commitable (`assets.yaml` `thelazystone-postapocalypse`) | 1 (`pilote`) — **remplacer** : même source que zombies | 1 (`pistolero`) — **remplacer** : silhouette générée CC0 (`scripts/generer-sprites-gungeon.py`, `sprites/lisible/`) | 1 (`pleureur`) — **remplacer** : tuile Kenney Tiny Dungeon (`sprites/gungeon/`, CC0) |
| **Ennemis courants** (idle, marche, attaque, mort) | 3 familles : `zombie_1`, `zombie_2`, `zombie_full` — **remplacer** (blackHUNTERdev ; Zombie2 de source inconnue) | 10 : rat, chien, rodeur, pillard, cracheur, franc_tireur, arroseur, tourelle, brute, buffle — **remplacer** (tous des sprites zombie provisoires) ; +2 de labo (`cible`, `mannequin`) | 8 : bullet_kin, bullet_kin_squad, shotgun_kin, lanceur_kin, tourelle_kin, veteran_kin, bandana_kin, fuyard_kin (+`cible`) — **remplacer** (silhouettes générées) | 10 : suiveur, mouche, araignée, rampant, spectre, cracheur, arroseur, bourdon, follet, tourelle_croix (+3 cibles de labo) — **remplacer** (tuiles Kenney) |
| **Boss** | 1 (`boss_zombie`, `rooms/boss.ron`) — **remplacer** | 1 (`roi_rat`) — **remplacer** | 1 (`gatling`, immobile, 3 phases) — **remplacer** (généré) | 2 (`gardien` 3 phases, `colosse` immobile 44 PV) — **remplacer** |
| **Familiers** (suivent / orbitent) | — | — | — | 8 : anneau, cousin, escorte, frère, garde, ombre, orbe, sentinelle — **remplacer** |
| **Projectiles / effets** (balles, larmes, éclats, explosion, coup de mêlée) | 3 armes à distance, `slash` (source inconnue, licence inconnue) — balles **absentes** | idem + patterns `eventail`, `crachat`, `couronne` — balles **absentes** | `bullet_player`, `bullet_enemy` (générés), `shell`, `bomb` — **remplacer** | `tear` (feedback), `bullet_enemy`, `bomb` — **remplacer** |
| **Armes** (visuel rotatif ou icône) | 3 visuels (pistol, shotgun, machine_gun) + fusil sans sprite — **remplacer** | 3 visuels — **remplacer** | 19 icônes `weapon/*` (générées) — **remplacer** | pas d'icône d'arme (tirs de larmes ; `pistol` hérité) |
| **Objets / ramassables** | 5 power-ups (`insta_kill`, `double_points`, `max_ammo`, `carpenter`, `nuke`), munitions, armes murales — **absent** (pas d'entrée `sprites.ron`) | 7 power-ups, mutations (icônes) — **absent** | 20 objets (`objets/*.ron`) + `item/passive|active|consumable|key|bomb|heart|blank` — **remplacer** (génériques par classe) | **100 objets** (`item/*`, 103 entrées) + cœurs, pièces, clés, bombes — **remplacer** (121 fichiers générés, murs/portes compris) |
| **Portes, fenêtres, portail** | portes, **fenêtres** (centrales au jeu), barricades — **remplacer** (tuiles LDtk empruntées) ; portail n/a | portes ; portail de sortie d'étage — **absent** | `door_h`, `door_v`, `portal`, hub, `secret_wall` — **remplacer** (générés) | `door_h`, `door_v`, `portal`, porte à clé/secrète — **remplacer** (générés) |
| **Murs + sol (tileset auto-tuilable)** | tilesets LDtk `SunnyLand_by_Ansimuz-extended` (16 px) et `Dungeon_Tileset` (16 px) : **remplacer** ; pas de `wall_art` | idem (`caves/gabarit.ldtk`) : **remplacer** ; pas de `wall_art` | `wall_art` : `atlas_murs.png` généré, 36 tuiles ; sol = tilesets LDtk empruntés : **remplacer** | `wall_art` généré + sol emprunté : **remplacer** |
| **Props de couverture** | tables, caisses, perks (machines) — **absent** | rochers, barils — **absent** | `barrel`, `table`, `rock`, `chest`, `shelf`, `spikes` (+ `props/*.ron`) — **remplacer** (générés) | `rock`, `rock_loot`, `chest`, `shelf`, piques — **remplacer** (générés) |
| **Surfaces** (eau, fosse, sable) | — | — | 3 (`eau`, `fosse`, `sable`) | 1 (`piques`) |
| **HUD (icônes)** | 4 icônes de perks = carrés couleur + lettre (`hud.ron` `icons`) | rads, niveau, statuts : texte seul | passifs/actifs/clés/bombes : carrés couleur + lettre | cœurs, âme, pièces, clés, bombes, actif, minicarte : carrés couleur + lettre |

Trois constats qui pèsent sur le choix des packs :

1. **Les icônes de HUD n'ont pas de champ image** : `HudIcon` ne porte que `color` et `label`
   (`crates/game/src/ui/hud.rs:176-180`). Les icônes d'un pack (cœurs, clés, pièces, perks) demandent un petit
   travail moteur (ajouter `image: Option<String>` à `HudIcon` et au widget `Icons`/`Hearts`/`Counter`), pas
   seulement des fichiers.
2. **Les murs ne passent pas par le tileset LDtk** mais par `wall_art` : une auto-tuile en présentation qui
   attend une feuille de **36 tuiles dans un ordre fixe** (`crates/map_ldtk/src/wall_art.rs:10-20` : 0..32 dessus
   = rebords + coins, 32..36 face sud). Un tileset acheté ne s'y branche pas tel quel : il faut recomposer cette
   feuille (§5).
3. **Le joueur est teinté par handle** (`player_colors`, `docs/conventions.md` §83) : le sprite du joueur doit
   être clair/neutre, sinon la teinte donne de la boue. Un héros très coloré demande une variante désaturée.

Leçon déjà payée (`games/gungeon/assets/assets.yaml:1-9`, `docs/taches/m2-lisibilite-gungeon.md`) : les tuiles
Kenney Tiny Dungeon donnaient « tout ressemble au héros ». Un pack riche en **silhouettes distinctes** par
classe est donc plus important que sa seule beauté.

---

## 2. Candidats par clone

Colonnes de couverture : ✔ couvert, ◐ partiel, ✘ absent. Les lignes sont celles du §1.

### 2.1 Zombies (bunker / avant-poste post-apo, fonderie)

| | **Z1 — Post-Apocalypse Pixel Art** (déjà en surcharge privée) | **Z2 — Zombie Apocalypse AssetPack** | **Z3 — Zombie Apocalypse Tileset** | **Z4 (libre) — Top-down Shooter** |
|---|---|---|---|---|
| Auteur / lien | TheLazyStone — https://thelazystone.itch.io/post-apocalypse-pixel-art-asset-pack | Aymen B — https://pixelrogueknight.itch.io/zombie-apocalypse-assetpack | Ittai Manero — https://ittaimanero.itch.io/zombie-apocalypse-tileset | Kenney — https://kenney.nl/assets/top-down-shooter |
| Prix | **2 $** minimum pour usage commercial (gratuit non commercial) | **3 $** | gratuit (`min_price` 0) | gratuit |
| Licence (même page) | « Free for Non-Commercial Use », commercial dès 2 $, **« No Redistribution »** (original ou modifié), crédit facultatif | Commercial et modifications autorisés ; **interdit** de redistribuer, revendre ou empaqueter comme asset, même modifié ; « vendre seulement comme partie d'un projet » ; pas de NFT ni d'IA ; fichier `license.exe` dans le pack `[À CONFIRMER : non lu]` | Personnel et commercial, modifiable, crédit apprécié, **interdit** de redistribuer ou revendre, avec ou sans modification | **CC0** |
| Tailles | non annoncée sur la page ; le repack existant applique ×2 / ×1,6 / ×1,4 pour atteindre ~32 px `[INFERENCE : sprites ≈ 16-24 px]` | tags 16×16 et 32×32 | 16×16 (titre de la page : « 16x16 post-apocalyptic tileset » dans les listes itch) | non annoncée sur la page (pas de champ « Tile size ») `[À CONFIRMER]` |
| Joueur | ✔ Idle, Run, Punch, Pickup, Death (vue de côté, flip), casque ; armes Bat/Pistol/Gun/Shotgun (Idle/Run, Attack, Reload, Death, pompe) | ✔ 1 joueur : Idle 6 f., Run 8, Hit 3, Knocked 6, Death 8 | ◐ marche et dégâts seulement (haut-bas) | `[À CONFIRMER]` page sans détail (580 fichiers) |
| Ennemis | ✔ 3 zombies (Small, Axe, Big) : Idle, Run, **2 attaques**, Death | ✔ 4 zombies (dont poison et décomposé) + bandit + 2 PNJ ; **pas d'animation d'attaque listée** (Hit, Knocked, Death) | ✔ 4 zombies (petit, moyen, gros, **zombie-tourelle** qui crache) ; marche + dégâts ; pas d'idle/attaque/mort listés | `[INFERENCE]` zombies (tag) |
| Boss | ✘ (le Big, agrandi) | ✘ | ✘ | ✘ |
| Projectiles / VFX | ✘ (balles non listées) | ◐ armes avec tir 9-12 frames | ◐ explosion de grenade, sang animé | ◐ |
| Armes | ✔ 4 | ✔ pistolet, revolver, fusil, pompe, lance-roquettes, couteau | ✔ couteau, pistolet, pompe, grenade | ◐ |
| Objets | ✔ munitions, nourriture, armes ; indicateurs santé/munitions/faim, inventaire | ✔ medkit, batterie, munitions, caisse à outils, jerrican, grenades, gilet, carte, blueprint, crâne… (animation 7 f.) | ✔ argent, 3 munitions, 2 trousses, radio, jerrican, canette | ◐ |
| Portes / fenêtres | ✔ **fenêtres (21+), portes**, murs de brique (13), **« Buildable » : 12 murs bois + 12 renforcés (= barricades)** | ◐ portes de placards/casiers, panneaux | ◐ | ◐ |
| Murs + sol | ✔ sols (herbe, terre, routes), **550+ bâtiments** en 4 jeux de couleurs, 60 toits ; auto-tuile à recomposer pour `wall_art` | ✔ 3 environnements (hôpital abandonné, terrain vague, camp) + 350 tuiles | ✔ décor (ferme, station-service, routes, voitures, ponts) | ✔ tuiles (cf. tags « tile », « furniture ») |
| Props | ✔ poubelles, **distributeurs** (machines de perks), lampes, clôtures 30+, déchets 32, véhicules 9 | ✔ armoires, bureaux, distributeurs, voitures, barils, pneus, pompes à essence | ✔ caisses cassables, **baril explosif**, voitures | ✔ meubles |
| HUD | ◐ barres santé/munitions/faim, inventaire, craft, menu | ✔ barres de vie (4+4 icônes), 2 panneaux, 18 boutons | ✔ inventaire avec grandes icônes | ✘ |
| Manque | boss, balles, icônes de perks, portail ; pas de vue de haut pour les personnages (côté + flip, comme aujourd'hui) | attaque des zombies, boss | idle/attaque/mort, boss | cohérence 16 px à mesurer |
| Intégration | **déjà faite** : `scripts/repack-postapocalypse.py` écrit `sprites/postapo/` + `sprites.local.ron` (gitignorés) ; reste murs, fenêtres, props, objets | script de repack par feuille + 3 tilesets LDtk | idem | idem |

Colonne Z4 : la page Kenney n'annonce que 580 fichiers et les tags `top-down`, `tile`, `furniture`, `zombie` ; ses ◐ et ✔
sont des `[INFERENCE]` tirés de ces tags, à vérifier en ouvrant le zip avant de s'appuyer dessus.

Hors liste, **écartés** : *HD 8-Directional Zombie pack* de SmallScaleInt (https://smallscaleint.itch.io/hd-8-directional-top-down-zombie-pack,
29,99 $, licence commerciale sans crédit mais sprites de **128-192 px** : hors échelle de 32×32, mélange impossible
avec des tuiles de 16 px) ; *[16x16] Zombie Shooter* de LorisC (https://lorisc.itch.io/16x16-zombie-shooter,
4,99 $, 2 personnages seulement) ; *Wasteland Survivor Kit* de SakPix (https://sakpix.itch.io/wasteland-survivor-kit-post-apocalyptic-pixel-art-asset-pack,
10 $) : **page déclarant « AI Assisted, Graphics »**, exclu.

Option libre : **Z4 Kenney Top-down Shooter** (CC0) + **Z3** (gratuit mais licence propre). *Top Down Basic Set* de
tokka (https://tokka.itch.io/top-down-basic-set, gratuit, « both in free and in commercial projects », joueur, flic, zombie
basique) est trop mince pour porter un jeu.

### 2.2 Throne (désert, cavernes mutantes)

Aucun pack trouvé ne couvre à la fois désert, cavernes, mutants animés et armes : la couverture se compose de deux
auteurs. Les packs sont de style différent (voir le risque noté en §3).

| | **T1 — PostApo Lands SET1** (désert dévasté) + **SET2 Caves** | **T2 — Zombie Apocalypse AssetPack** (personnages) | **T3 — Wasteland Essentials (version premium)** | **T4 (libre) — 16x16 Desert Tileset** |
|---|---|---|---|---|
| Auteur / lien | Szadi art. — https://szadiart.itch.io/postapo-lands-set1 · https://szadiart.itch.io/postapo-lands-set2 | Aymen B — https://pixelrogueknight.itch.io/zombie-apocalypse-assetpack | Orven — https://cat-cap.itch.io/wasteland | Beast (OpenGameArt) — https://opengameart.org/content/desert-tileset-0 |
| Prix | SET1 **29,99 $** ; SET2 **15,99 $** | **3 $** | **2,99 $** (version gratuite : non commerciale) | gratuit |
| Licence (même page) | « License for Everyone » : personnel ou commercial, modifier, vendre des œuvres faites avec ; interdit : logo/marque, **revendre l'asset** (original ou modifié) ; crédit non requis | voir Z2 | premium : « any commercial or non-commercial project », modifiable, **pas de redistribution ni revente** ; la version gratuite est **non commerciale** | **CC-BY 3.0** (attribution) |
| Taille | 16×16 (« les 32×32 sont mis à l'échelle ») | 16×16 / 32×32 | 16×16 | 16×16 |
| Joueur / ennemis | ✘ aucun personnage | ✔ joueur, 4 zombies (poison jaune, décomposé), bandit en casque à gaz : idle, run, hit, knocked, death ; chunky, contours épais | ✔ survivants et ennemis animés (marche, attaque, idle) | ✘ |
| Boss | ✘ | ✘ | ✘ | ✘ |
| Projectiles / VFX | ✘ | ◐ tirs d'armes | ✘ | ✘ |
| Armes | ✘ | ✔ 6 armes avec tir | ◐ (fusil visible sur la planche) | ✘ |
| Objets | ◐ tonneaux toxiques, caisses | ✔ 14 objets animés | ◐ trousse, munitions | ✘ |
| Portes / portail | ◐ portes de bâtiments | ◐ | ◐ | ✘ |
| Murs + sol | ✔ **le plus proche de Throne** : sol désertique craquelé, falaises, arbres morts, rochers noirs, flaques ; SET2 : cavernes (trois couleurs de murs) | ✔ « Wasteland area » + hôpital + camp | ✔ terrain de base | ✔ sable, cactus, falaises |
| Props | ✔ pneus, bidons, ruines, clôtures, feu de camp animé | ✔ voitures, barils, pneus, pompes | ✔ sacs de sable, caisses, clôtures grillagées, feu de camp animé | ◐ |
| HUD | ✘ | ✔ barres de vie, panneaux, 18 boutons | ✘ | ✘ |
| Manque | personnages entiers | boss ; pas de monstres vraiment « mutants » hors zombies | peu de contenu | tout le reste |

Vérification visuelle faite (captures de page lues) : SET1 donne bien un désert dévasté détaillé, plus réaliste ;
les personnages de Z2 sont stylisés, trapus, aux contours épais. **Les deux styles ne se ressemblent pas** : à regarder
côte à côte avant d'acheter SET1 et SET2.

Autres candidats vus : *Rogue Adventure Wasteland 16x16* d'ELV Games (https://elvgames.itch.io/rogue-adventure-wasteland-16x16-rpg-top-down-tileset,
7,99 $, tuiles seulement ; **licence non lue sur la page** `[À CONFIRMER]`) ; *Pixel Crawler — Desert* d'Anokolisa
(https://anokolisa.itch.io/pixel-crawler-desert, 10 $, un seul ennemi, la momie, plus 25 armes de mêlée) ; *RPG Worlds
Desert* et *Endless Caves* de Szadi (12 $ chacun) sont en **32×32** : incompatibles avec la grille de 16. À écarter :
16Pixel (CC-BY-SA 4.0, https://opengameart.org/content/16x16-desert-tilesets), le partage à l'identique est à éviter.
Cethiel « Desert Assets » (CC0, https://opengameart.org/content/desert-assets) est en vue de côté (« Sidescroll »).

Option libre : T4 (terrain, CC-BY) + **Z3 d'Ittai** (gratuit, licence propre) pour les mutants (zombie-tourelle =
cracheur ; routes, voitures, baril explosif, caisses, radio, ramassables).

### 2.3 Gungeon (donjon de pierre, ennemis « balles »)

| | **G1 — 16x16 DungeonTileset II** | **G2 — Pixel Crawler, pack gratuit** | **G3 — Ninja Adventure** | **G4 — Guns (icônes d'armes)** |
|---|---|---|---|---|
| Auteur / lien | 0x72 — https://0x72.itch.io/dungeontileset-ii | Anokolisa — https://anokolisa.itch.io/free-pixel-art-asset-pack-topdown-tileset-rpg-16x16-sprites | pixel-boy et AAA — https://pixel-boy.itch.io/ninja-adventure-asset-pack | greenpixels_ — https://greenpixels.itch.io/pixel-art-assets-7 |
| Prix | gratuit | gratuit | gratuit (« name your own price », minimum 0) | gratuit |
| Licence (même page) | **CC0** (« CC-0 », crédit non nécessaire) | conditions dans un PDF lié (https://drive.google.com/file/d/17y1gjuwVirE8V79WcTL9tgTUtMu6R1je/view) : **projets commerciaux autorisés**, retouches autorisées, crédit non requis, **interdit de revendre les arts comme produit final** | **CC0** (« Creative Commons Zero v1.0 »), usage commercial explicite, attribution non requise | **CC BY 4.0** (attribution) |
| Taille | 16×16 ; personnages 16×28 à 32×36 | 16×16 | 16×16 | 16×16 à 41×16 |
| Joueur | ✔ 8 héros (elfes, chevaliers, nains, lézards, magiciens) : idle 4 f., run 4 f., **hit** (1 f.) ; **ni attaque ni mort** | ✔ 3 héros + base à 4 directions : Run, Walk, Idle, Hit, Collect, **Death**, Slice, Crush, Pierce | ✔ « 50+ characters with animations and faceset » | ✘ |
| Ennemis | ✔ 25 personnages au total (gobelin, imp, chort, orcs, squelette, zombies, ogre, nécromancien…) ; slugs, swampy, muddy | ◐ 8 ennemis : squelettes et orcs en 4 variantes (base, mage, guerrier, voleur) : Idle, Run, Death | ✔ « 30+ Monster with animations » | ✘ |
| Boss | ◐ big_demon, ogre (grands sprites) | ✘ | ✔ « 9 Bosses with animations » | ✘ |
| Projectiles / VFX | ◐ flèche, flèches de bow | ✘ | ✔ « 30+ visual effect » | ◐ munitions |
| Armes | ✘ tirs : épées, haches, masses, arc, bâtons magiques ; **aucune arme à feu** | ✘ 50 armes de mêlée, arcs, baguettes | ◐ armes de mêlée, magie | ✔ revolver, 2 pistolets, 2 fusils, sniper, SMG, pompe à canon scié, 6 munitions |
| Objets | ✔ coffres (3 états), fioles ×8, pièce animée, bombe, crâne, caisse, leviers, boutons | ◐ établi, forge, alchimie | ✔ « 60+ Items » | ✔ munitions |
| Portes / portail | ✔ cadre de porte gauche/droite/haut + battants ouvert/fermé, échelle, escaliers, trou | ◐ | ◐ | ✘ |
| Murs + sol | ✔ jeu d'arêtes `wall_edge_*` (coins, T, milieux), 8 sols, bannières, fontaines animées, goo, pics animés ; **v1.7 « autotiles »** | ✔ environnement « Dungeon » | ✔ « Tilesets (floor autotiling, exterior and interior) » | ✘ |
| Props | ✔ colonnes, caisses, bannières, piques | ✔ | ✔ | ✘ |
| HUD | ✔ cœurs plein/demi/vide | ✘ | ✔ « UI element », 2 polices | ✘ |
| Manque | attaque, mort, armes à feu, boss unique, clés | boss, objets, portes | style japonais (ninja, village) à recolorer pour un donjon de pierre | un jeu complet d'armes |

Contenu de G1 relevé dans le fichier `tile_list_v1.7` du pack (miroir
https://github.com/balagurf/The-Biding-of-Isaac-Caves---/blob/main/Sprites/0x72_DungeonTilesetII_v1.7/0x72_DungeonTilesetII_v1.7/tile_list_v1.7) :
370 images, 25 personnages à idle/run (4 f.), 8 avec `hit`, ni `attack` ni `death`, `ui_heart_full|half|empty`,
`coin_anim`, `bomb`, `chest_*_open_anim`, `crate`, `flask_*`, `doors_*`, `floor_1..8`, `floor_spikes_anim`,
`wall_edge_*`, `weapon_*` (épées, arc, bâtons ; pas de fusil), pas de clé.

Extension : **Pixel Crawler — Hideout** (10 $, https://anokolisa.itch.io/pixel-crawler-hideout : « Bandit Crew » de 4
types avec Death, Run, Idle, Hit) et **Cave** (10 $, https://anokolisa.itch.io/pixel-crawler-cave : « Fungus Crew » avec
Hit, Idle, Run, Death) ; **0x72 Sewers** (2 $, https://0x72.itch.io/16x16-dungeontileset-ii-sewers, 4 nouveaux
personnages animés, murs auto-tuilables ; **licence non lisible sur la page** `[À CONFIRMER]`).

Options libres : G1 + G2 + G4 (CC0 + conditions Anokolisa + CC BY) ; Ninja Adventure (CC0) ; Kenney Tiny Dungeon
(https://kenney.nl/assets/tiny-dungeon, CC0, 16×16, 130 fichiers, **déjà essayé et écarté**, statiques) ; Kenney
Roguelike Characters (https://kenney.nl/assets/roguelike-characters, CC0, 450 fichiers) et Roguelike RPG Pack
(https://kenney.nl/assets/roguelike-rpg-pack, CC0, 16×16, 1700 fichiers) ; DawnLike
(https://opengameart.org/content/dawnlike-16x16-universal-rogue-like-tileset-v181, CC-BY 4.0, 16×16, donjon,
monstres, armes, objets).

### 2.4 Isaac (sous-sol, larmes, cœurs)

| | **I1 — Ninja Adventure** | **I2 — DawnLike** (libre, attribution) | **I3 — Kenney Roguelike (Characters + RPG Pack)** | **I4 — DungeonTileset II + Sewers** |
|---|---|---|---|---|
| Auteur / lien | pixel-boy et AAA — https://pixel-boy.itch.io/ninja-adventure-asset-pack | DragonDePlatino — https://opengameart.org/content/dawnlike-16x16-universal-rogue-like-tileset-v181 | Kenney — https://kenney.nl/assets/roguelike-characters · https://kenney.nl/assets/roguelike-rpg-pack | 0x72 — https://0x72.itch.io/dungeontileset-ii · https://0x72.itch.io/16x16-dungeontileset-ii-sewers |
| Prix | gratuit | gratuit | gratuit | gratuit + 2 $ |
| Licence (même page) | **CC0** | **CC-BY 4.0** | **CC0** | CC0 (G1) ; Sewers `[À CONFIRMER]` |
| Taille | 16×16 | 16×16 | 16×16 (RPG Pack) | 16×16 |
| Joueur | ✔ 50+ personnages animés | ◐ planches de 2 frames (à vérifier au téléchargement `[À CONFIRMER]`) | ◐ statiques | ✔ idle/run |
| Ennemis | ✔ 30+ monstres animés | ✔ monstres (liste OGA : monster, character, dungeon, weapon, item) | ◐ statiques | ✔ 25 + slugs, swampy, muddy |
| Boss | ✔ 9 | ✘ | ✘ | ◐ |
| Familiers | ◐ animaux, mascottes (update #8 « Animals ») `[À CONFIRMER]` | ✘ | ✘ | ✘ |
| Projectiles / VFX | ✔ 30+ effets | ◐ | ✘ | ✘ |
| Objets | ✔ 60+ (sur **100** objets d'Isaac) | ✔ objets, potions | ✔ | ✔ fioles, coffres |
| Portes / portail | ◐ | ◐ | ◐ | ✔ |
| Murs + sol | ✔ tilesets avec sol auto-tuilé, intérieur et extérieur | ✔ donjon | ✔ | ✔ |
| HUD | ✔ **cœurs** (`heart`, `heart_2`, `heart_3` dans le projet Godot de l'auteur, https://github.com/pixel-boy/NinjaAdventure/tree/main/content/ui), UI | ✔ icônes | ◐ boutons, panneaux | ✔ cœurs plein/demi/vide |
| Style | cartoon mignon, coloré, ton japonais (ninja, samouraï) : plus proche du ton « mignon et glauque » d'Isaac que la pierre de Gungeon | pixel art classique | style Kenney, neutre | **identique à Gungeon** : à éviter pour que les deux clones ne se ressemblent pas |
| Manque | larmes, cœurs d'âme, bombes, clés (à dessiner ou à recolorer), objets 61 à 100 | animations, boss | animations, boss | cohérence avec Gungeon |

Écarté : *Binding Of Isaac Style Room Tilesets* de Taco (https://tacotry.itch.io/binding-of-isaac, gratuit, « please credit me »)
reprend l'apparence d'un jeu commercial (consigne Q9 : aucun visuel tiré d'un jeu commercial) et n'a ni licence ni animation.

Option libre : **I1 seul, il est déjà CC0.**

### 2.5 Sons (effets sonores complets par clone)

Le plan demande des effets sonores complets (inventaire §11 : manquent vague/salle/étage, achat, ramassage, boss,
explosion, bombe, menu ; la mort d'ennemi réutilise `death`).

| Pack | Auteur / lien | Prix | Licence (même page) | Contenu | Univers |
|---|---|---|---|---|---|
| Impact Sounds | Kenney — https://kenney.nl/assets/impact-sounds | gratuit | **CC0** | 130 fichiers (impacts, foley) | tous (coups, éclats, portes) |
| RPG Audio | Kenney — https://kenney.nl/assets/rpg-audio | gratuit | **CC0** | 50 fichiers (foley, pas, armes) | gungeon, isaac |
| Digital Audio et Sci-fi Sounds | Kenney — https://kenney.nl/assets/digital-audio · https://kenney.nl/assets/sci-fi-sounds | gratuit | **CC0** | 60 et 70 fichiers (lasers, moteurs) | throne (radiations, mutations, tirs d'énergie) |
| Interface Sounds | Kenney — https://kenney.nl/assets/interface-sounds | gratuit | **CC0** | 100 fichiers (menus, achat, ramassage) | tous (menu, achat) |
| Ninja Adventure (inclus) | https://pixel-boy.itch.io/ninja-adventure-asset-pack | gratuit | **CC0** | « 100+ Sound effect », 37 musiques | isaac, gungeon |
| Snake's Authentic Gun Sounds + SECOND | SnakeF8 — https://f8studios.itch.io/snakes-authentic-gun-sounds · https://f8studios.itch.io/snakes-second-authentic-gun-sounds-pack | gratuit | « All sounds can be used commercially, credit not needed » | 140+ et 166+ fichiers : tirs de .22 à pompe, rechargements, armement (WAV et MP3, avec et sans réverbération) | zombies, throne, gungeon |
| GDC Game Audio Bundle | Sonniss — https://gdc.sonniss.com/ | gratuit | royalty-free, **sans attribution**, usage commercial, jeux compris ; **ne pas revendre les sons seuls** (texte de la licence 2020 lu : https://ia802901.us.archive.org/34/items/GameAudioGDC/Licensing.pdf ; la page de la version 2.0 de 2026, https://sonniss.com/gdc-bundle-license/, n'a pas pu être lue : protection anti-robot `[À CONFIRMER]`) | 7,47 Go, 347+ fichiers (édition 2026) | throne, zombies (ambiances, explosions) |
| Undead Creatures SFX Pack | Leohpaz — https://leohpaz.itch.io/undead-creatures-sfx-pack | **3,99 $** | personnel et commercial, crédit facultatif, ne pas revendre ni distribuer | 44 + 111 sons : zombies (44), squelettes (37), spectres, nécromancien… | zombies, gungeon, isaac |
| Zombie Pack v1 + Gun Pack v1 | Cyber Apps — https://cyberofficial.itch.io/sfx-pack | gratuit | « free to use wherever, just give credit » : **pas de licence formelle** `[À CONFIRMER]` | 86 sons de zombies, 125 de tirs | zombies |
| Blood, Flesh, Bones 1 | Magic Sound Effects — https://magicsoundeffects.itch.io/blood-flesh-bones-sound-pack-1 | **15 $** | « personal use and **one commercial project** » : **une licence par jeu** | 300 sons (sang, chair, os) | isaac (gore et larmes) |
| Universal Sound Effects Bundle | ELV Games — https://elvgames.itch.io/universal-sound-effects-bundle | **29,99 $** | commercial autorisé, pas de revente, interdit pour entraîner une IA ; mention « Credits to EvlGames & Pegonthetrack » ambiguë `[À CONFIRMER]` (page d'un pack du lot : https://elvgames.itch.io/loot-inventory-sound-effects-universal-sound-effects) | bundle de dizaines de packs : pas, interfaces, effets de jeu, succès | tous (ramassage, achat, interface) |

Par univers (proposition) : **zombies** = Snake + Leohpaz (3,99 $) + Kenney Impact ; **throne** = Snake + Kenney
Digital Audio / Sci-fi + Sonniss ; **gungeon** = Snake + Kenney Impact et RPG Audio + effets Ninja Adventure ;
**isaac** = effets Ninja Adventure + Kenney Interface + (option) Magic Sound Effects 15 $.

---

## 3. Recommandation

Principe : **un auteur dominant par clone**, pour la cohérence ; des packs **différents** d'un clone à l'autre pour
que les quatre jeux ne se ressemblent pas ; la **CC0** chaque fois que la couverture est équivalente (portage console
sans question). Les prix sont en dollars, minimums itch, hors TVA.

| Clone | Pack(s) recommandé(s) | Pourquoi | Prix |
|---|---|---|---|
| **zombies** | **Z1 TheLazyStone** « Post-Apocalypse Pixel Art » | Seul pack lu qui couvre **fenêtres, portes, barricades (12 murs bois + 12 renforcés), distributeurs (perks), bâtiments, 3 zombies avec 2 attaques et mort, 4 armes** ; déjà branché par `scripts/repack-postapocalypse.py` (zéro mécanisme à créer). Restent à fabriquer : boss, balles, icônes. | **2 $** |
| **throne** | **T1 PostApo Lands SET1 + SET2 Caves** pour le décor, **T2 Aymen B « Zombie Apocalypse AssetPack »** pour les personnages | SET1 est le désert dévasté le plus proche de Throne (sol craquelé, rochers noirs, pneus, feu de camp), SET2 donne les cavernes ; Aymen B apporte joueur, 4 zombies (poison, décomposé), bandit, armes, objets, barres de vie, avec idle/run/hit/mort. **Risque** : styles différents (décor détaillé, personnages trapus) ; acheter d'abord Aymen B (3 $) et SET1 et regarder le mélange avant SET2. | **48,98 $** (29,99 + 15,99 + 3,00) |
| **gungeon** | **G1 0x72 DungeonTileset II** + **G2 Pixel Crawler (gratuit)** + **G4 greenpixels Guns** | Donjon de pierre à 16 px, 25 silhouettes distinctes (réponse au problème de lisibilité), coffres, portes, murs auto-tuilables, cœurs ; armes à feu 16 px (le seul manque des deux premiers) ; tout libre ou presque (CC0, conditions Anokolisa, CC BY 4.0). Restent : attaque, mort, boss, la famille des « balles », clés. | **0 $** (+10 $ par extension Pixel Crawler Hideout ou Cave, option) |
| **isaac** | **I1 Ninja Adventure** | CC0 : 50 personnages, 30 monstres, 9 boss, 60 objets, tilesets auto-tuilés, cœurs, UI, 30 effets, **100 sons**. Ton cartoon proche d'Isaac, distinct de Gungeon (pierre). Reste : larmes, cœurs d'âme, clés, bombes, objets 61 à 100, les familiers. | **0 $** (un don à l'auteur est sympathique) |
| **sons** | Snake (gratuit) + Kenney (gratuit) + Ninja Adventure (inclus) + Sonniss (gratuit) + **Leohpaz Undead Creatures** | Les tirs, impacts, interfaces et ambiances sont libres ; seul le bestiaire zombie/mort-vivant est payant. | **3,99 $** |

### Prix total par option

| Option | Contenu | Total |
|---|---|---|
| **A. Tout libre** | zombies : Z4 Kenney Top-down Shooter (CC0) · throne : T4 Beast (CC-BY 3.0) + Z3 Ittai (gratuit) · gungeon : G1 + G2 + G4 · isaac : I1 · sons : Kenney + Ninja + Snake + Sonniss | **0 $** |
| **B. Recommandée** | tableau ci-dessus : Z1 (2) + T1 SET1 (29,99) + SET2 (15,99) + T2 (3,00) + G1/G2/G4 (0) + I1 (0) + Leohpaz (3,99) | **54,97 $** |
| **B′. Recommandée, throne économique** | comme B mais throne = T2 Aymen B (3,00) + Z3 Ittai (0) au lieu de SET1 et SET2 | **8,99 $** |
| **C. Étendue** | B + zombies Aymen B (3,00) + throne Orven premium (2,99) + gungeon Hideout et Cave (10 + 10) + sons Magic Sound Effects (15) et ELV Universal (29,99) | **125,95 $** |

Ce qu'aucune option n'élimine : le §4.

---

## 4. Ce qu'il faudra fabriquer malgré tout

| À fabriquer | Clone(s) | Raison | Piste |
|---|---|---|---|
| **Boss uniques** : `boss_zombie`, `roi_rat`, `gatling`, `gardien`, `colosse` | tous | aucun pack lu ne livre un boss adapté (Ninja Adventure en a 9, au thème ninja) | agrandir et recolorer un grand sprite (zombie Big ; `ogre` / `big_demon` de 0x72 ; boss de Ninja pour Isaac), puis dessiner sa phase spéciale |
| **Famille des « balles »** (bullet_kin, squad, shotgun_kin, lanceur…) | gungeon | identité propre au jeu : aucun pack n'a de douille ni de balle qui marche | garder les silhouettes générées `sprites/lisible/` (CC0) ou les redessiner dans le style de G1 ; ce sont des classes de lisibilité (§83), pas du décor |
| **Animations manquantes** : attaque et mort | gungeon (G1 : `hit` seulement) | l'état `Attack` et `Dying_*` est lu par le moteur (`visuals.rs:247`) | réutiliser `hit` en recul, ou dessiner 3-4 frames ; Pixel Crawler a `Death` |
| **Projectiles** : balles, larmes, éclats | tous | non listés par les packs (sauf munitions de G4) | petits sprites simples ; les `bullet_*` et `tear` générés actuels restent valables, ajouter la teinte |
| **Cœurs d'âme, clés, bombes, objets 61 à 100** | isaac | Ninja n'a que 60+ objets ; clés et bombes absentes de G1 | recolorer les icônes du pack |
| **Familiers** (8) | isaac | pas de familier à orbite | recolorer des animaux ou des mascottes de Ninja, une silhouette par rôle |
| **Icônes de HUD** (perks, mutations, passifs) | zombies, throne, gungeon | `hud.ron` n'a que `color` + `label` ; peu d'icônes dans les packs | ajouter un champ `image` à `HudIcon` (travail moteur) puis découper les icônes du pack |
| **Atlas `wall_art` de 36 tuiles** | tous (les deux jeux avec `wall_art` et ceux qui l'adopteront) | l'auto-tuile attend un ordre précis, que ne suit aucun pack | script de recomposition (§5) ; pour zombies et throne, ajouter `wall_art` à leur `hud.ron` |
| **Variante claire du joueur** | tous | teinte par handle | désaturer le héros ou ajouter un masque de teinte |
| **Portail** et **hub** | gungeon, isaac, throne | aucun pack lu ne porte un portail de sortie d'étage | recolorer une arche ou une porte |

---

## 5. Étapes d'intégration

Mécanisme existant à connaître (lu dans le dépôt) :

- `scripts/repack-postapocalypse.py` (modèle) : pour chaque entité, colle les images de la vue de côté, **pieds
  centrés en bas**, dans une cellule commune, concatène `Idle | Run | Dying_1 | Attack` sur **une** ligne, écrit
  `<nom>.png`, `<nom>_sheet.ron` (`path`, `tile_size`, `columns`, `rows`, `animated`, `name: "body"`, `scale`,
  `anchor: Center`, offsets) et `<nom>_animation.ron` (`frame_duration`, `animations: { "Idle": (start, end) … }`),
  puis un `sprites/sprites.local.ron` ; les armes sont rognées en 4 lignes identiques (index d'arme 0..3).
- **Surcharge locale gitignorée** (`docs/conventions.md` §2, `crates/content/src/registry.rs:4115-4182`) : un
  `sprites.local.ron` à côté de `sprites.ron` remplace les entrées de même id ; un id absent du fichier public est
  refusé (`BrokenReference`) : **chaque entrée garde un repli public**. Dossier d'images gitignoré :
  `games/*/assets/sprites/postapo/` (`.gitignore:27`) ; fixture `sprite_sheet_local_overlay`.
- **Cette surcharge ne couvre que `sprites.ron`.** Ni l'atlas `wall_art` (`hud.ron`), ni les tilesets des `.ldtk`,
  ni les images d'icônes de HUD, ni les sons n'ont de surcharge locale.
- Le merge `better-assets` de `docs/taches.md` (ligne « fusion origin/main dans better-assets ») ne décrit que la
  fusion ; le mécanisme est celui de §2 de `docs/conventions.md`.
- États d'animation lus par le moteur : `Idle`, `Run`, `Dying_1` (ou tout `Dying*`), `Attack` ; portes `Idle` /
  `Open` / `Locked` / `Secret` (`world_sprites.rs:438-443`) ; coffre `Idle` / `Open`.

**Décision préalable, à prendre avant tout achat** : le dépôt est-il public ou privé ? Les licences d'itch lues
interdisent de **redistribuer le pack** (« No Redistribution », « cannot resell, redistribute… ») mais autorisent de
livrer un jeu qui l'embarque. Si le dépôt est **public**, les images payantes (TheLazyStone, Aymen B, Szadi, Orven,
Leohpaz) restent hors du dépôt, via la surcharge locale (comme aujourd'hui), et la CI et les autres postes voient le
repli public. Si le dépôt est **privé**, on peut les commiter, avec une seule restriction : ne jamais les publier.
Dans les deux cas, **garder les silhouettes générées CC0 comme repli public**.

Étapes par pack :

1. **Enregistrer la licence avant l'import** (`games/<jeu>/assets/assets.yaml`, règle V6) : une entrée par pack :
   `id`, `fichiers`, `source` (URL consultée, date), `auteur`, `licence` (texte et URL), `modifications`, `statut`.
   Pour CC BY (greenpixels, DawnLike, Beast) : crédit dans l'écran de crédits (obligatoire en console aussi).
2. **Retirer les placeholders** de chaque jeu : blackHUNTERdev (`Body/Hair/Shirt`, `Zombie*`, `Weapons.png`),
   `Slash_strip3.png` et `Zombie2.png` de licence inconnue (`assets.yaml`, entrée `zombie-shooter-source-inconnue`).
3. **Atlas de personnages** : un script `scripts/repack-<pack>.py` par pack, calqué sur `repack-postapocalypse.py`
   (même gabarit de sorties, même `ENTITIES`). Pour **0x72** (images séparées `*_anim_f0…f3`) : cellule de 32×32,
   pieds en bas au centre, `scale` choisi pour que la largeur du sprite approche le collider de 20 px (16 px → ×1,25
   ou ×1,5) ; préférer le **zoom de caméra entier** (`camera.ron`) au `scale` non entier pour ne pas flouter le pixel
   art. Pour **Pixel Crawler** et **Ninja Adventure** (feuilles par animation) : découpe en grille, état `Run` ← `Walk`,
   `Dying_1` ← `Death`, `Attack` ← `Slice` / attaque du pack.
4. **`sprites.ron`** : une entrée par id existant (`player`, `bullet_kin`…) pointant sur la nouvelle feuille ; les
   entrées générées restent le repli. Pour un pack non redistribuable : mêmes entrées dans `sprites.local.ron`.
   Lint : `make` du lint de contenu (`content::lint`) vérifie que les feuilles existent et qu'aucun id n'est répété.
5. **Armes** (`weapon/*`, 19 en gungeon) : rogner une image par arme (G4), 4 lignes identiques (index 0..3), `anchor:
   CenterLeft`, comme `weapon()` du script post-apo.
6. **Murs** : écrire `scripts/repack-wall-atlas.py` qui compose `atlas_murs.png` en **36 tuiles dans l'ordre de
   `wall_art.rs`** (0..32 dessus : `rebords + 8 × coins`, 32..36 face sud ; 36 et au-delà : tuiles annexes comme les
   piques d'`isaac`) à partir des `wall_edge_*`, `wall_mid`, `wall_left/right` du pack ; puis `wall_art: Some((layer:
   "Walls", atlas: "sprites/<pack>/atlas_murs.png", hide_layers: ["LevelConnection"]))` dans `hud.ron`
   (`zombies` et `throne` n'en ont pas encore). Lint : feuille présente, couche `Walls` présente dans une carte.
7. **Sol et tilesets LDtk** : les `.ldtk` référencent `../atlas/SunnyLand_by_Ansimuz-extended.png`,
   `Dungeon_Tileset.png` (16 px). Copier le tileset du pack en 16 px dans `assets/atlas/`, puis changer le `relPath`
   et refaire les `autoRuleGroups` dans LDtk (le CLI `alacod-map` relocalise les chemins à l'export,
   `docs/taches.md` ligne `alacod-map`) ; les tuiles de gabarit (`LevelConnection`, `Entities`) ne changent pas.
8. **Props, objets, portes** : feuilles dans `sprites.ron` (`barrel`, `table`, `rock`, `chest`, `shelf`, `spikes`,
   `door_h`, `door_v`, `portal`, `item/*`) ; états d'animation `Idle`/`Open`/`Locked`/`Secret` pour les portes.
9. **HUD** : ajouter `image: Option<String>` à `HudIcon` (`crates/game/src/ui/hud.rs:176`) et aux widgets qui
   l'utilisent (`Icons`, `Hearts`, `Counter`), lint des chemins ; puis `icons: { "heart": (image: "…") }`.
10. **Sons** : copier les `.ogg` sous `games/<jeu>/assets/sounds/` (les clés `sounds` de `feedback.ron` : tir,
    rechargement, coup, dégâts, mêlée, mort, dash, porte, `blank`) ; **créer les clés qui manquent** (vague, salle,
    étage, achat, ramassage, boss, explosion, bombe, menu : inventaire §11) ; régler `volumes` (−12 dB aujourd'hui) ;
    entrées `assets.yaml` (une par pack).
11. **Vérifier** : `ALACOD_MUTE=1` pour les captures, `make videos` et `make views` pour relire la lisibilité
    (silhouette par classe, §83), `make test_scenarios` (les sprites sont de la présentation : aucune trace d'état ne
    doit bouger), `make check_sim_boundary` inchangé.
12. **Ordre conseillé** : (a) zombies, car Z1 est déjà en place (il manque fenêtres, murs, props) ; (b) gungeon, car
    G1/G2/G4 sont libres et testables sans achat ; (c) isaac avec Ninja Adventure ; (d) throne en dernier (le
    mélange de deux auteurs demande un essai) ; en parallèle (e) le travail moteur d'icônes de HUD et (f) le
    script d'atlas de murs, communs aux quatre.
