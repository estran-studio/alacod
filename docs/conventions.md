# Conventions de l'engine alacod

Référence pour ceux qui créent du contenu (cartes LDtk, sprites RON) et ceux qui ajoutent un vocabulaire à l'engine.

**Utilité** : lire cette page quand on crée une carte, importe des sprites, ou ajoute un effet/comportement/statut. Chaque section renvoie aux fichiers à lire pour comprendre le pattern.

**Numérotation du 2026-10-05** (m1-relecture-conventions) : « Feedback (présentation, T2.13) », ancien second §9 placé après le §10, devient le **§7** (il n'y avait pas de §7) et vient après le §6 ; §8 Combat, §9 Stats et §10 Blesser une trace gardent leur numéro, l'ancre `{#section7}` du §8 disparaît. Les fiches et rapports antérieurs qui citent « §9 » pour le feedback visent l'actuel §7 ; ceux qui citent « §7 » pour les stats ou « §8 » pour la preuve visent l'actuel §9 et l'actuel §10 (numérotation de M0).

**Sommaire** (un `## ` par entrée, dans l'ordre du fichier : les §40 et §41, §49 et §45, §51 et §50 y sont dans l'ordre où ils ont été écrits)

- §1. Cartes LDtk
- §2. Sprites et animations
- §3. Le dossier de jeu
- §4. Ajouter un vocabulaire à l'engine : checklist
- §5. Commandes make
- §6. CI lente (nuit) — T2.14
- §7. Feedback (présentation, T2.13)
- §8. Combat : équipes et dégâts (T1.1, chantier B1)
- §9. Stats et modificateurs (T1.2, chantier B2)
- §10. Blesser une trace : la preuve
- §11. Munitions et inventaire d'armes (T2.2, chantier B7)
- §12. Monnaie et achats (T2.3, chantier C5 v1)
- §13. État de run et mode `Waves` (T2.4, chantier F1)
- §14. Power-ups (T2.5, chantier C1 v0)
- §15. HUD : icônes, noms et sources (T2.12)
- §16. Projectiles composables (T1.1, chantier B5 v1)
- §17. Mode `Floors` (T1.8, chantier F1)
- §18. Équilibrage par joueurs (F5)
- §19. Statuts (T1.3, chantier B3)
- §20. Patterns, émetteurs et tir ennemi (T1.2, chantier B5 v1)
- §21. Terrain destructible et cavernes (T1.0b + T1.6, chantier E3)
- §22. Behaviors composables (T1.4, chantier D1)
- §23. Horloges et difficulté (T1.9, chantier F2)
- §24. Bots de validation (m0-v7 phase 2)
- §25. Variantes et élites (T1.5, chantier D2)
- §26. Surfaces (T1.7, chantier E4 v1)
- §27. Effets v1, jauges et mutations (T1.10, chantiers C1 v1 et C4 v1)
- §28. Générateur v1 et placement scripté (T1.13, voie V3)
- §29. Le jeu `throne` (T1.0c + T1.11, voie V2)
- §30. Écran de mutation et transition (T1.16, voie V4)
- §31. Feedback v1 (T1.17, voie V4)
- §32. HUD throne (T1.18, voie V4)
- §33. Restart en ligne (D14)
- §34. Course et esquive (dash)
- §35. Salles typées et verrouillées (M2-E1, chantier E1)
- §36. Objets et inventaire (M2-T0b, chantier C2)
- §37. Boss à phases (M2-T0c, chantier D4)
- §38. Profil de méta-progression (M2-T0c, chantier G1)
- §39. Le jeu `gungeon` (M2-T0d, squelette)
- §41. Projectiles v2 et blank (M2-T1, chantier B5 v2)
- §40. Grammaire d'étage (M2-T10, chantier E2)
- Notes essentielles
- §42. Effets v2 : déclencheurs, conditions, cooldowns et charges (M2-T3, chantier C1 v2)
- §43. Esquive v2 (M2-T2, chantier B6)
- §44. Objets v2 et synergies (M2-T4, chantier C2)
- §49. Formations simples : escouades en arc et tir alterné (M2-T8, chantier D1 v2)
- §45. Boss et arène (M2-T7, chantier D4)
- §46. Surfaces v2 : fosses, tables renversables, barils explosifs (M2-T11, chantier E4 v2)
- §47. Bot explorateur d'étages (M2-T16, chantier K2 v1)
- §48. Butin, coffres et clés (M2-T5, chantier C4)
- §51. Profil et déblocages qui enrichissent le butin (M2-T12, chantier G1)
- §50. Boutique et monnaie `shells` (M2-T6, chantier C5)
- §52. Contenu `gungeon` (M2-T14, quatre parties)
- §53. Lint des kinds M2 et atteignabilité statique (M2-T15, voie V2)
- §54. Générateur v2 : gabarits par objet, par salle et par boss (M2-T17, voie V3)
- §55. Feedback v2 : télégraphes de patterns, flash du blank, fin de boss (M2-T19, voie V4)
- §56. Écrans en bevy_hui et HUD gungeon (M2-T18, voie V4)
- §57. Hub : stations, marchand de déblocages, entrée en run (M2-T13, chantier G2)
- §60. Contrats des larmes et de l'empilement de projectiles (M3-T0a, chantier B5 v3)
- §61. Contrats des objets et de la santé (M3-T0b)
- §62. Contrats du monde : bombes, rochers, piques, portes verrouillées, grille (M3-T0c)
- §63. Contrats des familiers, orbitaux et champions v2 (M3-T0d)
- §64. Curses, pools à épuisement, accomplissements : contrats (M3-T0e)
- §65. Le jeu `isaac` : squelette (M3-T0f)
- §66. Larmes v3 : tir composé, `Spectral`, `Split`, `Orbit`, `Poison`, `Fear`, chance d'effet (M3-T1, chantier B5 v3)
- §67. Effets v3 : `OnHit`, `OnEvent`/`Emit`, `ApplyStatus`, `PerCopy` (M3-T2, chantier C1 v3)
- §68. Cœurs, i-frames, sous, bombes et clés au sol (M3-T3, chantiers C4 et C5)
- §69. Bombes, explosions, rochers cassés, piques, mur secret (M3-T4, chantiers E3 et E4 v2)
- §70. Étage en grille : planificateur, salle secrète, trésor à clé, salle post-boss (M3-T5, chantier E2 v2)
- §71. Familiers et orbitaux en jeu (M3-T6, chantier C3)
- §72. Champions et élites v2 en jeu (M3-T7, chantier D2 v2)
- §73. Pools par type de salle, épuisement à la génération, marché du diable (M3-T8, chantiers C4 v2 et C5 v2)
- §74. Curses d'étage en jeu : tirage, modificateur, étage agrandi, HUD masqué (M3-T9, chantier F4)
- §75. Accomplissements en jeu : faits de run, évaluation à `Ended`, profil v2 (M3-T10, chantier G1 v2)
- §76. Contenu `isaac` (M3-T11, quatre parties)
- §77. Lint des kinds M3 et atteignabilité des mécaniques (M3-T12, voie V2)
- §78. Générateur v3 : gabarits `TearsOnTarget`, `FamiliarHeld`, `ChampionForced` (M3-T13)
- §81. HUD `isaac` : cœurs, compteurs, objet actif, bandeau d'objet, minicarte typée (M3-T16, voie V4)
- §82. Feedback v3 : larmes lisibles, familiers, champions, bombes et explosions (M3-T17, voie V4)
- §83. Langage visuel provisoire : on doit comprendre ce qu'on voit (m2-lisibilite-gungeon, voie V4)

## 1. Cartes LDtk

**Version** : LDtk 1.5.3 (`jsonVersion` dans le fichier). Consulter `games/zombies/assets/exemples/test_map.ldtk`.

**Grille et taille** : tuiles de 16 pixels (`defaultGridSize: 16`). Un niveau LDtk = une salle de jeu. Les niveaux s'assemblent par leurs **connexions** (voir ci-dessous) ; le générateur (`crates/map/src/generation/`) en calcule position et taille dans le monde.

**Couches** : trois couches visibles dans l'éditeur.

- `Walls` (IntGrid, valeur 1) : les murs que les joueurs et ennemis ne traversent pas. Chaque cellule vaut 1 si c'est un mur, sinon absent (pas de 0 explicite). Détermine aussi le flow field (murs, portes fermées, obstacles cassables intacts) par ordre de priorité : murs > portes fermées > fenêtres intactes. Consulter `crates/map_ldtk/src/game/collider.rs` pour voir comment ces cellules sont converties en colliders physiques ; le flow field les utilise dans `crates/game/src/character/enemy/ai/navigation.rs` pour construire le graphe de navigation.
- `LevelConnection` (IntGrid, valeur 1) : indique les ouvertures (portes ou passages) sur chaque bord d'un niveau. Le générateur détecte ces connexions avec `scan_width_side` (`crates/map/src/generation/context.rs`) : un scan en ligne/colonne du bord compte les cellules contiguës valant 1. Chaque groupe continu = une ouverture possible vers un autre niveau. Exemple : une ouverture de trois cases sur le nord = trois cellules (1, 1, 1) consécutives dans la rangée du haut.
- `Entities` (couche objet Bevy LDtk) : emplacements des joueurs, ennemis, portes, fenêtres, armes et sodas. Chaque entité a une position (pixels dans le niveau) et une grille (cellule de 16 px du monde).

**Entités et leurs champs** (lus par `crates/map_ldtk/src/game/entity/*.rs`) :

- `DoorHorizontal`, `DoorVertical` (32×16 ou 16×32 px) : portes payantes. En éditeur LDtk, deux champs seulement : `price` (entier, coût en points), `electrify` (booléen, desserte électrique si vrai). Le générateur ajoute quatre champs aux portes qu'il place : `interactable` (booléen, défaut `true`), `paired_door_x`, `paired_door_y` (entiers), `paired_door_level` (chaîne) pour les portes appariées (une porte achetée ouvre son jumeau simultanément). Lus par `door_component_from_field` dans `crates/map_ldtk/src/game/entity/door.rs`. Les portes **bloquent le flow field** tant qu'elles ont un collider (portes fermées) ; une porte non interactive ne s'ouvre jamais (`CLAUDE.md`, section « Flow Field - Implémentation Actuelle »).
- `WindowHorizontal`, `WindowVertical` (16×32 ou 32×16 px) : vitres cassables. Aucun champ en éditeur. Santé fixée à 3 dans le code (`crates/map/src/game/entity/map/window.rs`). Les fenêtres intactes bloquent les ennemis (collision, mais permettent le tir à travers) ; cassées, elles bloquent les joueurs seulement (perméables aux ennemis). Réparation par interaction. Lus par `crates/map_ldtk/src/game/entity/window.rs`.
- `PlayerSpawn` (16×16 px) : points de départ des joueurs. Champ obligatoire : `index` (0, 1, 2, 3) = handle GGRS du joueur. Seul le niveau de départ a besoin de points de départ. Lus par `crates/map_ldtk/src/game/entity/player_spawn.rs`.
- `ZombieSpawn` (16×16 px) : emplacements de spawn des ennemis. Quatre champs entiers **optionnels**
  `active_x`, `active_y`, `active_width`, `active_height` (en pixels, **locaux au niveau** dans la
  bibliothèque, convertis en monde par l'assembleur) : quand ils sont tous présents, le mode vagues
  ne fait apparaître les ennemis de ce spawner que si un joueur est dans ce rectangle, sinon il est
  exclu. C'est ainsi qu'une section garde « ses » zombies (courettes derrière les fenêtres) sans en
  recevoir d'une section fermée. Aucun champ = comportement d'avant (choix par distance au joueur).
  Lus par `crates/map_ldtk/src/game/entity/enemy_spawn.rs`.
- `CharacterSpawn` (16×16 px, T2.9, testbed) : fait apparaître, une fois au chargement de la map (pas par les vagues), un personnage précis du registre. Trois champs : `character` (chaîne, `CharacterId`, obligatoire), `variant` (chaîne optionnelle, variante imposée, T1.5, §25) et `team` (chaîne optionnelle : `players`/`enemies`/`allies`/`neutral` ; absente ou vide → `CharacterConfig.team` du personnage, sinon `Enemies`). Lus par `crates/map_ldtk/src/game/entity/character_spawn.rs` ; spawné par `map_ldtk::game::local::spawn_characters_when_map_loaded` via `character::enemy::create::spawn_enemy`. Utilisé par `games/testbed/assets/testbed/*.ldtk` pour placer `dummy`/`target`/`follower`/`ally`/`civilian`/`breacher` (`games/testbed/assets/characters/*.ron`).
- `WeaponLocation` (16×16 px, T2.3, chantier C5 v1) : arme murale payante. Deux champs en éditeur : `weapon` (chaîne, `WeaponId` de `weapons.ron`), `price` (entier, coût en points). Propagée par le générateur de salles (modèle `CharacterSpawn`, voir `crates/map/src/generation/entity/weapon_location.rs`) ; au chargement de la map, fait apparaître un `weapons::WeaponPickup` mural (`price: Some(...)`, jamais consommé au ramassage — voir `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/weapon_location.rs`. `games/zombies/assets/exemples/test_map.ldtk` en pose quatre depuis T2.6 (pistol 500 dans le gabarit de départ `Level_0`, rifle 1200 et shotgun 1000 dans `Level_1`, machine_gun 1500 dans `Level_2` ; tableau dans `games/zombies/README.md`, qui rappelle aussi que la partie assemble des copies de ces gabarits ; depuis D10 le gabarit de départ `Level_0` n'est plus réutilisé, donc un seul mur `pistol`) ; `test_map_shop.ldtk` en pose une (shotgun, 500) pour les scénarios d'achat.
- `SodaLocation` (16×16 px, T2.3, chantier C5 v1) : machine à perk. Un champ en éditeur : `perk` (chaîne, `PerkId` de `economy/perks.ron`). Même propagation que `WeaponLocation` ; au chargement, fait apparaître une entité `game::economy::PerkMachine` avec `Interactable { interaction_type: Perk }` (voir `map_ldtk::game::local::spawn_soda_locations_when_map_loaded`). Lue par `crates/map_ldtk/src/game/entity/soda_location.rs`.
- `CrateLocation` (16×16 px) : **collectée mais non utilisée** (`crates/map_ldtk/src/map_const.rs` la déclare, le générateur en relève les positions dans le champ `crates` de `generation/from.rs`, aucun bundle implémenté en entity/*.rs). Réservée à un chantier futur (butin, C4).

**Champ de niveau** : le champ `spawn` (booléen) détermine le niveau de départ d'une run. Lu par `crates/map_ldtk/src/generation/from.rs` (`map_const::LEVEL_FIELD_SPAWN`) : si présent et `true`, le niveau est marqué `LevelType::Spawn`. **D10 (décision de William, 2026-10-14)** : le gabarit de départ n'est utilisé que comme salle de départ (`BasicMapGeneration::get_next_room_recursize` l'écarte des candidats des salles suivantes ; une connexion dont tous les gabarits compatibles sont des gabarits de départ est fermée comme `DeadEnd`). Avant, il pouvait réapparaître comme salle ordinaire (deux `Level_0` avec la seed des scénarios).

**Tailles de collision** : les colliders des entités (joueurs, ennemis) font 20×20 pixels aux pieds (voir `crates/game/src/character/config.rs`). Les sprites font 32×32 pixels. Les colliders de porte et fenêtre sont déterminés par leur taille LDtk (32×16 ou 16×32) ; l'engine les lit dans `crates/map_ldtk/src/game/entity/door.rs` et `window.rs`.

**Concevoir une bibliothèque de pièces (assemblage `BasicMapGeneration`, 2026-10-17).** C'est
l'algorithme de carte du projet, le seul pour les cartes de salles (les générateurs `building`
« Le Relais » et `compound` ont été retirés ce jour : grille fixe et niveau unique, sans
`LevelConnection`, ils ne permettaient pas de dessiner des pièces qui s'emboîtent). Gungeon y
ajoute la grammaire d'étage (§40 : types de salles obligatoires, boss en cul-de-sac), qui
assemble avec les mêmes prises. Code : `crates/map/src/generation/imp/basic.rs`, prises dans
`context.rs`, placement dans `room.rs`.

- **Une bibliothèque = un `.ldtk` ; une pièce = un niveau** (salle, couloir, section), de
  n'importe quelle taille (multiple de 16 px). Le générateur clone le niveau tel quel : murs,
  tuiles, entités.
- **Une prise** = une suite contiguë de cases à 1 de `LevelConnection` sur un bord. Signature :
  `côté @ décalage + longueur`, le décalage compté en cases depuis le coin haut-gauche le long
  du bord (x pour N/S, y pour E/W). `avant_poste` : `Depart` a `N@12+2`, `W@5+2`, `E@8+2`.
- **Règle d'emboîtement** (`Connection::are_matching`) : deux prises s'emboîtent si et seulement
  si leurs côtés sont opposés, **même décalage, même longueur**. Rien d'autre n'est regardé : ni
  la taille des pièces, ni leur nom. Deux pièces de tailles différentes s'emboîtent donc si leurs
  prises ont le même décalage ; un couloir étroit sert de **pièce de transition** entre deux
  formats quand ses prises reprennent les décalages de chaque format. Une pièce ne s'emboîte
  jamais avec elle-même.
- **Variété = plusieurs pièces par signature de prise.** Le générateur tire au hasard, parmi
  les pièces compatibles avec la prise choisie, laquelle poser : une signature portée par une
  seule paire de pièces donne toujours le même voisinage (c'est le cas d'`avant_poste`, dont
  les neuf signatures sont toutes uniques : les graines ne changent que la forme de l'arbre, pas
  qui touche qui). Pour des cartes différentes à chaque run : un petit jeu de signatures
  (ex. `N@12+2`/`S@12+2` et `W@8+2`/`E@8+2` pour tout le format 26×19) et beaucoup de pièces qui
  les portent, dont des **impasses** (une seule prise : fin de branche, salle à butin), des
  **couloirs** (deux prises opposées) et des **carrefours** (trois ou quatre prises).
- **Départ** : le champ de niveau `spawn: true` ; un seul gabarit de départ par carte (D10,
  jamais réutilisé ensuite ; plusieurs gabarits `spawn` = l'un d'eux tiré au sort). Lui seul
  porte les `PlayerSpawn` (`index` 0 à 3).
- **Une porte sur chaque prise** (`DoorHorizontal` 32×16 posé sur les deux cases d'une prise
  N/S, `DoorVertical` 16×32 sur une prise E/W). Le générateur apparie la porte avec celle de la
  pièce d'en face (une achetée ouvre les deux ; prix `750 + 250 × profondeur` en portes depuis
  le départ, `electrify` hors du départ). Une prise restée sans voisine (chevauchement, bord de
  carte, aucune pièce compatible, génération arrêtée) est **murée** : ses cases passent à 1
  dans `Walls` et sa porte n'est pas posée (`wall_unused_openings`, décision de William du
  2026-10-17 ; avant : porte non interactive et ouverture dessinée, encore disponible avec
  `wall_unused_openings: false`).
- **Boucles** (`close_loops`, 2026-10-17) : après le placement, deux pièces posées bord à bord
  dont des prises libres se font face (même emprise dans le monde : côtés opposés, même bord,
  même intervalle) sont reliées comme une connexion ordinaire : portes appariées, prix de
  chaque porte selon la profondeur de sa pièce. C'est ce qui donne les convergences et les
  raccourcis des cartes de CoD Zombies (deux ailes qui retombent dans la même cour) : sans
  cette passe l'assembleur ne produit que des arbres. Paires examinées dans l'ordre des pièces
  posées (déterministe) ; une prise déjà fermée (`DeadEnd`, `OutSide`) peut être rouverte par
  une boucle. Pour qu'une boucle soit possible, il faut que deux pièces puissent se retrouver
  côte à côte avec des prises de même signature face à face : encore une raison de partager
  les signatures.
- **Déroulé** : pièce de départ posée à une position tirée (alignée sur la grille), puis en
  boucle : prise libre tirée sur la pièce courante → pièce compatible tirée → collée bord à bord ;
  refusée si elle chevauche une pièce posée ou sort des bornes. Quand la pièce courante n'a plus
  de prise libre, une autre pièce encore ouverte est tirée. Arrêt à `max_rooms` ou quand plus
  rien n'est ouvert. Puis fermeture des boucles, murage des prises libres ; portes, fenêtres,
  apparitions, armes murales, sodas et props sont recopiés de chaque pièce posée.
- **Réglages par jeu** (`entry.map` de `game.ron`, `world::MapSettings`, tous optionnels) :
  `max_rooms` (10 ; `0` = sans plafond), `max_width`/`max_height` (1000 px : demi-largeur et
  demi-hauteur autour de l'origine, soit ~125 cases de côté), `aligned_openings` (`true` :
  règle d'origine, deux prises s'emboîtent au même décalage, pièces alignées coin haut-gauche ;
  `false` : côté opposé et même longueur suffisent, la pièce est décalée pour faire coïncider
  les prises — `get_connecting_room_position` sait déjà le faire), `close_loops` (`true`),
  `wall_unused_openings` (`true`). Un scénario joue avec les réglages de son jeu.
  ```ron
  entry: (
      start_map: "maps/avant_poste.ldtk",
      default_seed: 123456,
      mode: Waves,
      map: (max_rooms: 14, max_width: 1500, max_height: 1500, aligned_openings: false),
  ),
  ```
- **Juger une bibliothèque sans lancer le jeu** : `alacod-map` (`crates/map_ldtk/src/bin/alacod-map.rs`,
  bibliothèque `map_ldtk::preview`, aucun Bevy `App`) : `make map_render MAP=games/zombies
  SEEDS=1,2,3`. La cible compile avec `--no-default-features --features cli` : **aucune dépendance
  au crate `game`**, ni fenêtre ni simulation démarrée ; les dépendances Bevy transitives de `map`
  restent présentes à la compilation. Le dossier du jeu fournit `entry.start_map`, `entry.map`
  et sa graine par défaut si `SEEDS` est absent. Les désignations `floor:` et `cave:` sont résolues ;
  `ARGS="--floor etage"` choisit une grammaire différente. Avec `MAP=<bibliothèque.ldtk>`, les
  graines par défaut sont 1 à 6 et les réglages ceux de `MapSettings::default()` ;
  `ARGS="--game games/<jeu>"` lit plutôt les réglages de ce jeu.
  Le CLI écrit un PNG et un `.ldtk` par graine dans `target/map_preview/` (`ARGS="--out DIR"`
  pour changer de dossier). Le PNG dessine **les tuiles des calques LDtk**, auto-tuiles comprises,
  puis les annotations : index des pièces, prises reliées en vert, sans voisine en rouge,
  portes orange, joueurs bleus, zombies magenta, fenêtres cyan, départ bordé d'or.
  `--plain` force le plan abstrait ; un tileset PNG illisible est signalé. Les entités sont des
  marqueurs de conception, pas leurs sprites en jeu. Les auto-règles ne sont pas recalculées :
  les nouvelles cases murées sont dessinées sur fond sombre. La console compte les prises
  réellement murées (pas simplement les côtés sans voisin), les boucles et décrit chaque pièce.
  L'export LDtk relie les tilesets depuis son dossier réel, y compris derrière un lien symbolique.
  L'intégration du jeu reste disponible par la feature `runtime` de `map_ldtk`, activée par défaut.

  ```bash
  cargo run -p map_ldtk --no-default-features --features cli --bin alacod-map -- \
    games/zombies/assets/maps/avant_poste.ldtk --seeds 1,4 --out target/map_preview
  make map_render MAP=games/gungeon SEEDS=1 ARGS="--floor etage"
  ```

**Vérification** : `make map_render MAP=games/zombies SEEDS=1,2,3` (ci-dessus) pour voir les cartes ; `make map_preview ARGS=games/zombies/assets/exemples/test_map.ldtk` affiche une carte dans une fenêtre ; `make map_generation ARGS="games/zombies/assets/exemples/test_map.ldtk <sortie.ldtk> [graine]"` écrit seulement le `.ldtk` assemblé. Un scénario `idle` sur la carte rejouée en vidéo (`make play_scenario SCENARIO=idle`) vérifie l'assemblage et les entités en jeu.

---

## 2. Sprites et animations

**Format de planche** : grille régulière en PNG, décrite par deux fichiers RON (exemple : `games/zombies/assets/sprites/enemies/zombie/`).

**Table des feuilles (D3, kind `SpriteSheet`)** : `sprites/sprites.ron`, déclaré dans `game.ron`
(`(path: "sprites/sprites.ron", kind: "SpriteSheet")`), associe un **id** à une configuration
d'animation et à ses calques. L'id est celui que le contenu cite : `asset_name_ref` d'un
personnage, `sprite_config.name` d'une arme à distance, `slash` pour l'effet de coup de mêlée.
`game::global_asset` charge exactement cette table (aucun chemin de sprite dans le code).
```ron
({
    "player": (
        animation: "sprites/characters/player/player_animation.ron",
        layers: {
            "body": "sprites/characters/player/player_sheet.ron",
            "shadow": "sprites/characters/shadow/shadow_sheet.ron",
        },
    ),
    "pistol": (
        animation: "sprites/characters/player/player_animation.ron",
        layers: { "body": "sprites/weapons/pistol_sheet.ron" },
    ),
})
```
**Surcharge locale** (pack d'images non redistribuable) : un `sprites/sprites.local.ron` à côté de la
table, même format, **gitignoré** (`games/*/assets/sprites/*.local.ron`), remplace les entrées de même
id ; un id absent de la table publique est refusé (`BrokenReference` : chaque entrée garde un repli
public). Sans lui (CI, autres postes, amis), les sprites publics. Seul usage : le pack post-apo
TheLazyStone de zombies et throne, écrit avec ses PNG (`sprites/postapo/`, gitignoré) par
`scripts/repack-postapocalypse.py` ; fixture `sprite_sheet_local_overlay`.
Feuilles, animations et images sont rangées par entité sous `sprites/` : `characters/` (joueur,
ombre), `weapons/`, `enemies/`, `effects/`. Un personnage sans entrée n'a pas de sprite (les
personnages de laboratoire du testbed). Lint : fichiers d'une entrée présents (animation,
feuilles, image `path` de chaque feuille), `sprite_config.name` d'une arme connu de la table, id
répété refusé (tableau du §3). Les configs de contenu (`player_config.ron`, `weapons.ron`,
`zombie_*_config.ron`) restent sous `ZombieShooter/Sprites/**` (dossier historique) : D3 n'a
déplacé que les sprites.

`SpriteSheetConfig` (`crates/animation/src/lib.rs`, une planche = une couche) :
- `path` : chemin du PNG relatif au dossier `assets/` du jeu (`games/<jeu>/assets/`).
- `tile_size` : (largeur, hauteur) en pixels d'une case.
- `columns`, `rows` : grille de la planche.
- `anchor` : `Center`, `BottomLeft`, `BottomCenter`, etc. (enum `ConfigurableAnchor`). Les planches actuelles utilisent `Center` avec des `offset_*` (voir `sprites/characters/player/player_sheet.ron`).
- `offset_x`, `offset_y`, `offset_z` : translation du sprite par rapport à l'entité.
- `scale` : facteur appliqué au rendu.
- `animated` : `true` si la planche a plusieurs frames ; `false` sinon.
- `name` : nom interne du calque (exemple : `"body"`, `"shadow"`).

`AnimationMapConfig` (même dossier que la planche) :
- `frame_duration` : millisecondes par frame.
- `animations` : table `{ "nom": { start: 0, end: 6 } }` ou `{ "nom": { row: 0, end: 7 } }`. Format direct (indices absolus) ou basé sur les lignes (spécifier la ligne et le nombre de frames).
- `columns` : optionnel, dérivé de `SpriteSheetConfig` sinon.

**8 directions, 2 dessinées** : l'engine suit 8 directions par `FacingDirection` (`crates/sim_core/src/animation_state.rs`, état rollback ; déménagé d'`animation` par D69) mais dessine seulement 2 (gauche/droite, obtenues par retournement automatique). La planche n'a pas besoin de 8 orientations : elle est dessinée tournée vers la droite, et le rendu la retourne pour la gauche (`set_sprite_flip`).

**Calques (skins)** : un personnage peut avoir plusieurs calques animés (`body`, `shadow`, `effect`). Chaque calque est une `SpriteSheetConfig` et une `AnimationMapConfig` ; ils sont composés dans `CharacterConfig` (`crates/game/src/character/config.rs`).

**CharacterConfig** (RON, décrit un personnage) :
```ron
(
    movement: (...),
    asset_name_ref: "zombie_full",  // Identifiant unique
    collider: (shape: Rectangle(width: "20.", height: "20."), offset: (...)),
    scale: "1.0",
    base_health: (max: "50.0"),
    starting_skin: "1",
    skins: {
        "1": (layers: {
            "body": "",  // Clé dans le registre d'animation
            "shadow": "",
        })
    }
)
```

`scale` est **obligatoire** (D79, fermée) : `CharacterConfig.scale` n'a pas de défaut côté jeu, et le schéma
de contenu (`content::registry::CharacterFileSchema`) l'exige comme lui. Un personnage sans `scale` est
refusé par `alacod lint` (`Parse`, « `scale` est obligatoire pour un personnage, ex. `scale: "1.0"` »,
fixture `character_scale_missing`) au lieu d'échouer au chargement du jeu.

**Registre de simulation `CharacterConfigs`** (D69 étape 2, E3a) : la simulation ne lit jamais `Assets<CharacterConfig>`.
`CharacterConfig` est une donnée pure (`Deserialize`, sans `Asset`) ; l'application la charge par l'enveloppe
`CharacterConfigAsset` (`character/config_asset.rs`, `#[serde(transparent)]` : les `.ron` ne changent pas) et publie
`CharacterConfigs(BTreeMap<id, CharacterConfig>)` (clés : les `CharacterId` du registre de contenu, c'est-à-dire les
`asset_name_ref`). Un personnage porte `CharacterConfigKey(String)` (composant non rollback, posé par `create_character`,
remplace `CharacterConfigHandles`). Le système `sync_character_configs` tourne en `PreUpdate.before(RunGgrsSystems)` : il copie
toute entrée absente dont l'asset est lisible (chargement initial, sans dépendre d'un événement) et recopie sur
`AssetEvent::Modified` (rechargement à chaud : l'entrée recopiée remplace alors une surcharge) ; `Added`/`LoadedWithDependencies`
ne font rien, pour qu'une surcharge de scénario posée sur la ressource ne soit jamais écrasée par eux. Le chargement finit en `AppState::Loading` (tous les assets chargés)
avant le lobby, donc la ressource est pleine avant `resolve_balance_system` (`OnEnter(GameLoading)`), avant tout spawn de
personnage et avant le premier tick ; preuve : `crates/scenario/tests/character_configs.rs` (ressource relue à chaque tick,
dont le premier, pour zombies, testbed et isaac ; chemin `Modified` ; diagnostic ignoré `fill_timeline`). Les surcharges de
scénario (`CharacterConfigs` modifié par `run_with`) se posent après la synchro.

**Registre de simulation `WeaponRegistry`** (D69 étape 2, E3b) : la simulation ne lit jamais `Assets<WeaponsConfig>` ni
`Assets<MeleeWeaponsConfig>`. `WeaponRegistry { weapons, melee }` (`weapons.rs`, `BTreeMap<id, WeaponAsset | MeleeWeaponAsset>`, la table
de `weapons.ron` et celle de `melee_weapons.ron` du jeu) est lu par les créateurs (`create_player`, `spawn_enemy`, armes murales), le
butin (`loot`, `shop`, `champions`, `progression`, boss), les effets (`EffectAssets`, `SpawnPattern`), les bots, le runner de scénarios et le
son des armes. Les enveloppes `WeaponsConfig`/`MeleeWeaponsConfig` (`Asset`, `HashMap` bevy), `WeaponAssetsPlugin`, la visibilité des armes
portées et `sync_weapon_registry` vivent côté application (`weapon_assets.rs`). La synchro tourne en `PreUpdate.before(RunGgrsSystems)` : elle
copie une table **vide** dont l'asset est lisible (chargement initial, sans dépendre d'un événement) et **remplace** la table sur
`AssetEvent::Modified` (rechargement à chaud : une arme retirée du fichier disparaît, une surcharge de scénario aussi) ; `Added` ne fait rien,
donc une surcharge posée sur le registre n'est jamais écrasée. Les surcharges de scénario (`weapon_overrides`, `PlayerScript::weapon`) lisent et
écrivent le **registre** (`apply_weapon_overrides` attend qu'il soit rempli, avant la création des joueurs). Une arme absente du registre est ignorée
avec un `warn!` par les créateurs : seul un test sur la ressource voit un registre vide (`crates/scenario/tests/weapon_registry.rs`).
Rechargement des armes **déjà portées** : `weapons_config_update_system` (`Update`, inchangé par E3b) interroge `&Children` sur l'entité d'arme ; seules les
armes qui portent un sprite d'enfant (`attach_weapon_visuals`, avec rendu) sont mises à jour, pas en headless. Les armes créées après un rechargement, elles,
lisent le registre à jour.

**Équilibrage, économie, power-ups et vagues** (D69 étape 2, E3c) : `EconomyConfig`, `PerksConfig`, `PowerUpsConfig` et `WaveConfig` sont des
données de simulation pures (plus d'`Asset`) ; leurs enveloppes `#[serde(transparent)]` (`EconomyConfigAsset`, `PerksConfigAsset`
dans `economy_assets.rs`, `PowerUpsConfigAsset` dans `powerups_assets.rs`, `WaveConfigAsset` dans `waves/assets.rs`, **application**) portent
les `RonAssetPlugin` ; les `.ron` ne changent pas. La simulation lit **`ResolvedBalance`** (équilibrage : vagues, économie, perks, santé ;
`waves_enabled`, `economy.shells_drop`, `economy.shop_prices` ajoutés par E3c : `shell_drop_on_death_system`, `room_shop_system`, la machine à
vagues) et **`PowerUpsTable(Option<PowerUpsConfig>)`** (`powerups.rs`, `powerup_pickup_detect_system`, `apply_powerup_actions_system`,
`loot_drop_on_death_system`, le HUD et les repères au sol). Les deux sont remplies à `OnEnter(GameLoading)`, la table **après** l'équilibrage
(`powerups_assets::resolve_powerups_table_system`) : un rechargement à chaud de `items/powerups.ron` en cours de partie ne change donc plus
les power-ups avant la prochaine entrée en `GameLoading` (les systèmes lisaient l'asset à chaque frame ; `ResolvedBalance` était déjà figée,
F5). Surcharges de scénario : `wave_overrides` et `powerup_drop_chance_override` visent l'**enveloppe** (`Assets<…Asset>`, posée en `Update` dès
le chargement, donc avant `GameLoading` qui la résout) ; les placements de power-ups scénarisés lisent la table. Parité : test
`balance_assets::tests::resolved_balance_is_identical_through_the_envelope_and_the_table` (champ à champ, cinq jeux, 1 à 4 joueurs),
`powerups_assets::tests::table_is_the_envelope_content_field_by_field`, et `crates/scenario/tests/balance_tables.rs` (les ressources
`SessionPlayers`, `ResolvedBalance` et `PowerUpsTable` au premier tick, surcharges de scénario comprises : les replis des lecteurs masquent une table vide).

**`GlobalAsset`, ressource d'application** (D69 étape 2, E3d) : `game::global_asset::GlobalAsset` tient les `Handle` que l'application doit garder
chargés, en deux familles : la **présentation** (`spritesheets`, `animations`, `slash_effect_*`, lus par `character/visuals.rs`, `ui/weapon_visuals.rs`,
`world_sprites.rs`) et les **sources de synchro** (handles des enveloppes de personnages, d'armes, de vagues, d'économie, de perks et de power-ups, lus
par `sync_character_configs`, `sync_weapon_registry`, `resolve_balance_system` et `resolve_powerups_table_system`). Aucun fichier de simulation, aucun
lecteur externe (`bots`, `map_ldtk`, `examples`, le code de `scenario/src`) ne le lit : ils lisent `CharacterConfigs`, `WeaponRegistry`, `ResolvedBalance`
et `PowerUpsTable`, donc aucune `ContentData` n'a été créée (aucun lecteur de simulation ne subsistait). Les surcharges de vagues et de power-ups du runner
désignent l'enveloppe par `Assets<…Asset>` (un jeu n'en charge qu'une) ; les tests de `crates/scenario/tests` qui comparent registre et enveloppe
(`character_configs.rs`, `weapon_registry.rs`, `balance_tables.rs`) gardent `GlobalAsset` pour atteindre l'enveloppe. `loading_asset_system` ne
passe de `Loading` au lobby qu'une fois, en plus des assets chargés, `CharacterConfigs` plein et `WeaponRegistry` rempli (une table vide dans le fichier ne
bloque pas) : la première partie ne démarre jamais sur un registre vide.

**Armes** (exemple : `games/zombies/assets/weapons/melee/melee_weapons.ron`, table `{ "bare_hands": (...), ... }`) :
- `config` : `name`, `damage`, `range`, `attack_pattern` (enum : `SingleStrike`, `Combo(strikes_in_combo: N)`, `Sweep(arc_angle)`, `Thrust`), `attack_duration_frames`, `cooldown_frames`, `knockback_force`, `stamina_cost`. Tous les nombres décimaux en chaîne (`"100.0"`).
- `sprite_config` : `name`, `index` (première frame), `weapon_offset`. Pour une arme à distance, `name` est un id de la table `SpriteSheet` (D3).

**Fixed-point** : les valeurs de type `Fixed` s'écrivent en **chaîne** (`"100.0"`, jamais `100` ni `100.0` littéral). Les entiers (`frames`, `mag_size`, indices) s'écrivent nus. La sérialisation Bevy/RON (crate `fixed`) convertit les chaînes en `Fixed` au chargement, garantissant le déterminisme. Exemple : `damage: "10.0"` (Fixed), `attack_duration_frames: 10` (entier). Consulter `CLAUDE.md` §Déterminisme, règle 1 (Fixed-Point Math UNIQUEMENT).

**Vagues** (`games/zombies/assets/waves/wave_config.ron`) : RON, configuration simple. Champs : `base_enemies`, `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`, `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`, `min_player_distance`/`max_player_distance` (Fixed en chaîne), `wave_tiers` (liste de `{max_wave, enemy_probabilities}`), `health_multiplier_per_wave`, `damage_multiplier_per_wave`, `max_wave` (optionnel, T2.4, §13). Chaque champ numérique accepte aussi une expression (`NumOrExpr`, §18). Les tirages passent par le flux RNG `"waves"` (`RngStreams`).

---

## 3. Le dossier de jeu

**Depuis T0.3** : structure `games/<jeu>/` autonome. Chaque jeu a son propre crate binaire (`games/<jeu>/Cargo.toml`, `src/main.rs`), ses assets (`games/<jeu>/assets/`), et ses scénarios de test. Le premier jeu implémenté est `zombies` (T0.3) ; un testbed minimal (`testbed`) valide l'engine sans simulation complexe ; `throne` (§29) est le deuxième clone.

```
games/<jeu>/
├── Cargo.toml         # Dépend de engine (crates/game, etc.)
├── src/
│   └── main.rs        # Boilerplate de l'engine + jeu
├── assets/
│   ├── sprites/       # Planches PNG + RON configs
│   ├── maps/          # LDtk (.ldtk)
│   ├── sounds/        # Sons (.ogg)
│   ├── ui/            # Configs HUD/écrans (RON)
│   └── game.ron       # Manifeste : dossiers, ids de contenu
```

Les scénarios ne sont pas dans le dossier du jeu : `tests/scenarios/` (écrits) et
`tests/scenarios/generated/<jeu>/` (générés, `make gen`).

**`game.ron`** (manifeste du jeu, `crates/content/src/manifest.rs`, T1.5) : nom, dossiers de
contenu typés, point d'entrée (carte de départ, graine par défaut). Extrait simplifié
(`games/testbed/assets/game.ron`, qui déclare aussi `mode: Sandbox` et d'autres dossiers) :
```ron
(
    name: "testbed",
    content_folders: [
        (path: "ZombieShooter/Sprites/Character/player_config.ron", kind: "Character"),
        (path: "ZombieShooter/Sprites/Character/weapons.ron", kind: "Weapon"),
        (path: "weapons/melee/melee_weapons.ron", kind: "MeleeWeapon"),
        (path: "testbed", kind: "Map"),
        (path: "ui", kind: "Ui"),
        (path: "camera.ron", kind: "Camera"),
    ],
    entry: (
        start_map: "testbed/arena.ldtk",
        default_seed: 123456,
    ),
)
```
Chaque entrée de `content_folders` est un **fichier** ou un **dossier**, relatif à
`assets/`. Un dossier est scanné (non récursif) pour l'extension attendue par son `kind`
(`.ron`, sauf `Map` qui attend `.ldtk`) ; un fichier est lu tel quel. Les jeux actuels
mélangent encore plusieurs kinds dans un même dossier historique
(`ZombieShooter/Sprites/Character/` contient à la fois `player_config.ron` et des feuilles
de sprite) : dans ce cas le manifeste déclare le **fichier** précis plutôt que le dossier
entier (voir `games/zombies/assets/game.ron`). Les kinds connus (31 : `content::registry::KNOWN_KIND_NAMES` fait foi, lu dans cet ordre) : `Character`, `Weapon`,
`MeleeWeapon`, `Wave`, `Map`, `Ui`, `Camera`, `Economy`, `Perk`, `PowerUp`, `SpriteSheet` (D3), `Floors`, `Cave`,
`FloorGrammar`, `Room`, `Item`, `Surface`, `Pattern`, `Progression`, `Mutation`, `Status`, `Clock`, `Difficulty`,
`Synergy`, `LootTable`, `Unlock`, `Curse`, `Achievement`, `MetaEconomy`, `HubService`, `Prop` ; un
autre kind produit une erreur de lint (« kind inconnu ») plutôt qu'un échec RON générique.

**Le registre** (`content::registry::Registry`, chargé par `Registry::build`) est la
source de vérité des ids de contenu : `CharacterId`, `WeaponId`, `MeleeWeaponId`,
`EnemyId`, `WaveConfigId`, `MapId` (newtypes sur chaîne, `Ord`, `BTreeMap`). L'id d'un
personnage vient de son champ `asset_name_ref` (pas du nom de fichier, qui ne le reflète
pas toujours aujourd'hui) ; l'id d'une arme ou d'une arme de corps à corps vient de la clé
dans `weapons.ron`/`melee_weapons.ron` ; l'id d'une carte ou d'une config de vagues vient du
nom de fichier sans extension. Les joueurs reçoivent les armes déclarées dans le champ
`starting_weapons: [WeaponId]` de leur fichier `characters/*.ron` (dans l'ordre déclaré : la
première est l'arme active), pas tout `weapons.ron`.

**Le lint** (`content::lint::run`, appelé par `alacod lint`, par le jeu au démarrage et par
le rechargement à chaud) refuse une référence cassée (`starting_weapons` vers une arme
inconnue, `enemy_probabilities` d'une vague vers un personnage inconnu, `entry.start_map`
vers une carte non chargée), un id dupliqué, une valeur hors plage (santé > 0, vitesse
mouvement >= 0, cadence de tir > 0), un kind de dossier inconnu, ou un littéral RON nu
(entier ou flottant) là où une valeur `Fixed` est attendue (le projet exige une chaîne,
`"1.5"` : voir §2 ci-dessus et CLAUDE.md, règle 1). Chaque erreur nomme le fichier (relatif
à `assets/`) et un message précis (id, champ, valeur).

**Règles de lint** (`crates/content/src/lint.rs` pour les règles sémantiques,
`crates/content/src/registry.rs` pour les erreurs de chargement). Une fixture par règle sous
`crates/content/tests/fixtures/<fixture>/` (mini-jeu : `assets/game.ron` + contenu minimal,
un seul problème en plus de `start_map: "unused"` commun à toutes), un test par fixture dans
`crates/content/tests/lint_fixtures.rs`. Kinds d'erreur (`LintErrorKind`) : `Parse`,
`DuplicateId`, `BrokenReference`, `OutOfRange`, `UnknownKind`, `Unsupported` (T1.10).

| Contenu | Règle | Kind | Fixture |
|---|---|---|---|
| tout | RON invalide, littéral nu dans un champ `Fixed` | `Parse` | `float_literal` |
| `game.ron` | kind de dossier inconnu | `UnknownKind` | `unknown_kind` |
| `game.ron` | `entry.start_map` vers une carte non chargée | `BrokenReference` | (toutes) |
| `game.ron` | `entry.mode: Waves` sans dossier `Wave` (T2.4) | `BrokenReference` | `entry_mode_waves_without_waves` |
| personnage | id (`asset_name_ref`) dupliqué | `DuplicateId` | `duplicate_id` |
| personnage | `starting_weapons` vers une arme inconnue | `BrokenReference` | `broken_reference` |
| personnage | `starting_weapons.len() <= weapon_slots` (T3.4) | `OutOfRange` | `starting_weapons_exceed_slots` |
| personnage | `starting_skin` absent de `skins` | `BrokenReference` | — |
| personnage | `base_health.max > 0`, `movement.max_speed >= 0` | `OutOfRange` | `out_of_range` |
| personnage | `stats.<StatId> >= 0` (T1.2) | `OutOfRange` | `out_of_range_stat` |
| personnage | `bleedout_frames`, `revive_frames`, `weapon_slots` > 0 ; `downed_speed_mult` dans ]0, 1] | `OutOfRange` | — |
| arme | `firing_modes.<mode>.firing_rate > 0` ; gabarit `test:` (`frames > 0`, `min_hits <= max_hits`) | `OutOfRange` | — |
| personnage | `test:` (T1.13, §28) : `frames > 0`, au moins un gabarit (`still`/`moving`), `expect_*` seulement d'un gabarit actif | `OutOfRange` | `character_test_frames_zero`, `character_test_no_template`, `character_test_expect_inactive` |
| arme | `ammo_type` : variante inconnue (`Laser`) | `Parse` (nomme le champ) | `ammo_type_unknown` |
| arme | `ammo_type: Custom("")` (nom vide) (T2.8) | `OutOfRange` | `ammo_type_empty_custom` |
| arme, corps à corps | `friendly_fire` : valeur inconnue (enum fermé `Never`/`Always`/`Cursed`) (T2.8) | `Parse` (nomme le champ) | `friendly_fire_unknown` |
| arme | `audio_config.modes.<mode>.reloading`/`firing` : fichier absent de `assets/` (T2.8) | `BrokenReference` | `audio_missing_file` |
| arme | `audio_config.modes.<mode>` : mode absent de `firing_modes` (assets) | `BrokenReference` | `audio_unknown_mode` |
| arme | clé répétée dans `weapons.ron` (T3.4) | `DuplicateId` | `weapons_duplicate_key` |
| corps à corps | clé répétée dans `melee_weapons.ron` (T3.4) | `DuplicateId` | `melee_duplicate_key` |
| corps à corps | `damage >= 0` | `OutOfRange` | — |
| vagues | `enemy_probabilities` vers un personnage inconnu | `BrokenReference` | — |
| économie | `refill_price_ratio` dans [0, 1] (T2.8) | `OutOfRange` | `economy_ratio_out_of_range` |
| perk | `price > 0` | `OutOfRange` | — |
| perk | `modifiers` non vide (T2.8) | `OutOfRange` | `perk_no_modifiers` |
| perk | `op: Mul` exige `value > 0` (T2.8) | `OutOfRange` | `perk_mul_non_positive` |
| perk | `stat` inconnue (`StatId` : hors liste seulement via `Custom("...")`) | `Parse` (nomme le champ) | `perk_unknown_stat` |
| perk | id écrit deux fois dans `perks.ron` (T2.8) | `DuplicateId` | `perk_duplicate_id` |
| power-up | id dupliqué entre deux fichiers du dossier | `DuplicateId` | `powerup_duplicate_id` |
| power-up | clé répétée dans `items/powerups.ron` (T3.4) | `DuplicateId` | `powerups_duplicate_key` |
| power-up | `actions[].stat` inconnue | `Parse` | `powerup_unknown_stat` |
| power-up | `drop_chance` dans [0, 1] | `OutOfRange` | `powerup_drop_chance_out_of_range` |
| power-up | `weight > 0` | `OutOfRange` | `powerup_weight_zero` |
| power-up | `actions` non vide (T2.8) | `OutOfRange` | `powerup_no_actions` |
| power-up | `lifetime_frames > 0` (T2.8) | `OutOfRange` | `powerup_lifetime_zero` |
| power-up | `pickup_range > 0` (T2.8) | `OutOfRange` | `powerup_pickup_range_non_positive` |
| power-up | `TimedModifier`/`CurrencyMultiplier` : `frames > 0` (T2.8) | `OutOfRange` | `powerup_frames_zero` |
| power-up | `CurrencyMultiplier` : `factor > 0` (T2.8) | `OutOfRange` | `powerup_factor_non_positive` |
| power-up | `TimedModifier` avec `op: Mul` : `value > 0` (T3.4) | `OutOfRange` | `powerup_timed_mul_non_positive` |
| `ui/feedback.ron` | `hit_flash.frames`, `shake.frames` > 0 (T3.4) | `OutOfRange` | `feedback_frames_zero` |
| `ui/feedback.ron` | `shake.amplitude >= 0` (T3.4, zéro désactive la secousse) | `OutOfRange` | `feedback_amplitude_negative` |
| `ui/feedback.ron` | chaque fichier de `sounds` présent sous `assets/` (T3.4) | `BrokenReference` | `feedback_missing_sound` |
| `ui/feedback.ron` | arme de `by_weapon` connue du jeu (T1.17, §31) | `BrokenReference` | `feedback_by_weapon_unknown` |
| `ui/feedback.ron` | clé de `volumes` présente dans `sounds` (assets) ; valeur dans [-60, 12] dB | `BrokenReference` / `OutOfRange` | `feedback_volume_unknown_key` |
| `ui/feedback.ron` | surcharges `by_kind`/`by_weapon` dans les bornes (T1.17, §31) | `OutOfRange` | `feedback_override_out_of_range` |
| `ui/hud.ron` | source de widget inconnue (T1.18, §32) | `UnknownKind` | `hud_unknown_source` |
| power-up | `RefillAmmoOf` : munition déclarée par au moins une arme (D40, §14) | `BrokenReference` | `powerup_refill_ammo_unknown` |
| feuilles de sprites | `animation` et feuille de chaque calque présentes sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_file` |
| feuilles de sprites | image (`path`) de chaque feuille présente sous `assets/` (D3) | `BrokenReference` | `sprite_sheet_missing_image` |
| feuilles de sprites | id répété dans `sprites/sprites.ron` (D3) | `DuplicateId` | `sprite_sheet_duplicate_key` |
| arme | `sprite_config.name` absent de la table `SpriteSheet`, si le jeu en déclare une (D3) | `BrokenReference` | `weapon_sprite_unknown` |
| `game.ron` | `entry.mode: Floors` sans dossier `Floors` (T1.8) | `BrokenReference` | `entry_mode_floors_without_floors` |
| `game.ron` | `entry.clocks` vers une horloge inconnue ; `entry.difficulty` sans fichier `Difficulty` (T1.9, §23) | `BrokenReference` | `entry_clock_unknown`, `entry_difficulty_missing` |
| `game.ron` | `entry.progression` vers une progression inconnue (T1.10, §27 ; audit T1.12) | `BrokenReference` | `entry_progression_unknown` |
| `game.ron` | `generate_template` : carte connue, cible à `counts_hits` (T1.13, §28) | `BrokenReference` | `generate_template_unknown_map`, `generate_template_target_without_hits` |
| `game.ron` | `mechanics` : mécaniques d'une partie ordinaire que le contenu doit porter (M3-T12, §77) | avertissement `MechanicUnmet` (aucune erreur) | `mechanics_ok` et les tests de `m3_t12_fixtures.rs` |
| arme | projectiles composables (T1.1, §16) : projectile de pattern absent de la table, cycle de `on_expire`, pattern temporel ou `Scatter` en `on_expire`, `Size > 0`, `0 < Homing <= 1`, modificateur répété, `on_hit` limité aux actions à cible | `BrokenReference`, `OutOfRange` | `projectile_broken_reference`, `projectile_cycle`, `projectile_temporal_pattern`, `pattern_scatter_on_expire` |
| arme | `on_hit` `ApplyStatus` : statut inconnu, `stacks > 0` (T1.3, §19) | `BrokenReference`, `OutOfRange` | `status_unknown` |
| pattern | `Named` inconnu ou cyclique ; `count > 0`, `spread >= 0`, `speed >= 0` (T1.2, §20) | `BrokenReference`, `OutOfRange` | `pattern_unknown_name` |
| personnage | `ai.behaviors` `Shoot` : arme et pattern connus, projectiles du pattern dans la table de l'arme, `range > 0`, `cooldown_frames > 0` (T1.4, §22) | `BrokenReference`, `OutOfRange` | `behavior_shoot_unknown`, `ranged_projectile_missing`, `ranged_cooldown_zero` |
| personnage | `Melee` : arme de corps à corps connue ; `Chase` : profil connu ; `KeepDistance` : `min < max` ; `Charge` : télégraphe > 0 ; `targeting.ignore` : tag porté par un personnage (T1.4) | `BrokenReference`, `OutOfRange` | `behavior_melee_unknown`, `behavior_unknown_profile`, `behavior_keep_distance_inverted`, `behavior_charge_zero`, `targeting_unknown_tag` |
| personnage | `variants` : `chance` dans [0, 1], `weight > 0`, nom unique, skin connu, pas de `MoveSpeed` sur un IA (T1.5, §25) | `OutOfRange`, `DuplicateId`, `BrokenReference` | `variant_chance_out_of_range`, `variant_weight_zero`, `variant_duplicate`, `variant_skin_unknown`, `variant_move_speed_ai` |
| personnage | `effects` (T1.10, §27) : déclencheur, condition ou action non exécutés (`Unsupported`), `Tick(0)`, `Heal <= 0`, pattern et arme de `SpawnPattern` connus, jauge d'`OnGauge`/`GaugeAdd` connue | `Unsupported`, `OutOfRange`, `BrokenReference` | `effect_unsupported`, `effect_out_of_range`, `effect_broken_reference`, `progression_broken_reference` |
| carte | `CharacterSpawn` : personnage connu (audit T1.12) ; `variant` imposée déclarée par ce personnage (T1.5) | `BrokenReference` | `map_character_unknown`, `variant_ldtk_unknown` |
| caverne | au moins 16 × 16, `fill_ratio` dans [0, 1], `min_floor_ratio` dans [0, 0.9], `birth`/`survive` <= 8 ; `characters` connus ; gabarit `gabarit.ldtk` présent (T1.6, §21) | `OutOfRange`, `BrokenReference` | `cave_out_of_range`, `cave_unknown_character`, `cave_template_missing` |
| séquence `Floors` | `levels` non vide ; carte ou `cave:<id>` chargée (T1.8, §17) ; caverne sans ennemi (`characters` vide ou `enemy_spawns` nul) refusée sauf `transit: true` (D36, §21) | `OutOfRange`, `BrokenReference` | `floors_empty`, `floors_unknown_map`, `floors_unknown_cave`, `floors_cave_without_enemies` |
| surface | `intgrid_value > 0` et unique, facteurs > 0, tags non vides (T1.7, §26) | `DuplicateId`, `OutOfRange` | `surface_duplicate_value`, `surface_factor_non_positive` |
| horloge, difficulté | ids d'événements uniques, échéances croissantes, `repeat > 0` ; expression : identifiants admis, valeur > 0 aux bornes (T1.9, §23) | `DuplicateId`, `OutOfRange`, `Parse` | `clock_duplicate_id`, `clock_unordered`, `clock_repeat_zero`, `difficulty_unknown_identifier`, `difficulty_non_positive` |
| progression | `per_kill > 0`, `levels` croissants, `choices` dans [1, pool], `choice_frames > 0`, chance dans [0, 1] ; mutations et armes du pool connues (T1.10, §27) | `OutOfRange`, `BrokenReference` | `progression_out_of_range`, `progression_broken_reference` |
| mutation | `weight > 0`, `max_stacks > 0`, `effects` non vide (+ règles des effets) (T1.10) | `OutOfRange` | `mutation_out_of_range` |
| statut | `frames > 0` ; `Burn` : `damage > 0`, `period > 0` ; `Slow` : `factor` dans ]0, 1] (T1.3, §19) | `OutOfRange` | `status_out_of_range` |
| power-up | action propre aux projectiles ou aux effets (`DestroyTerrain`, `ApplyStatus`, `Modifier`, `Heal`, `SpawnPattern`, `GaugeAdd`) | `OutOfRange` | `powerup_apply_status` |

Les enums fermés (`FriendlyFire`, `ModifierOp`, `Action`) et les variantes nues inconnues des
enums ouverts (`AmmoType`, `StatId`, qui n'acceptent un nom libre que sous `Custom("...")`)
échouent au chargement RON : la règle est le parse lui-même, la fixture documente le message
(`champ ammo_type : ...`, `champ friendly_fire : ...`, `champ stat : ...`). Une table RON
(`{ "id": ... }`) désérialisée en `BTreeMap` garde silencieusement la dernière valeur d'une
clé répétée. Le registre lit donc `perks.ron`, `weapons.ron`, `melee_weapons.ron` et la table
`powerups` de `items/powerups.ron` en liste (`registry::KeyedEntries`) pour rapporter les
doublons, y compris dans un même fichier. La première entrée est conservée, la seconde
produit `DuplicateId`. Les fichiers `feedback.ron` et `hud.ron` déclarés sous le kind `Ui`
sont lus avec un schéma typé ; les autres fichiers Ui restent validés syntaxiquement.

**CLI** : `cargo run -p content --bin alacod --profile headless -- lint games/<jeu>` (code
de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon) ; `make lint` l'appelle pour
`zombies`, `testbed` et `throne`. Un hook `PostToolUse` (`.claude/settings.json`,
`scripts/lint-edited-game.sh`) relance ce lint en arrière-plan quand un fichier sous
`games/**` est édité.

**Pour un agent** : `alacod lint --json games/<jeu>` écrit un objet `{game, errors, warnings}` sur
stdout (`kind`, `file`, `message` par entrée ; même code de sortie que `lint`).
`alacod describe games/<jeu> [Kind] [--json]` (`crates/content/src/describe.rs`) inventorie le
contenu d'un jeu : kinds et dossiers, ids, champs de premier niveau **utilisés** (fréquence,
exemple), fichier modèle. Pour chaque kind couvert (tous sauf `Map`, `Ui`, `Camera`, test
`every_kind_has_a_schema`), il ajoute le **schéma exhaustif** dérivé des types Rust
(`crates/content/src/schema.rs`, `schemars`) : champs, types, requis ou défaut, variantes d'enum et
commentaires de doc ; les `Fixed` y sont des chaînes. `--json` donne le JSON Schema brut (`schema`).
Les types de contenu portent `#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]`
(feature `schema` de `bevy_fixed`, `sim_core`, `effects`, `items`, `world`, activée par `content`) :
**un nouveau champ ou une nouvelle variante apparaît donc seul dans `describe`**. Les plages que le lint
vérifie s'écrivent `[requis, lint : > 0]` : elles viennent de `#[schemars(extend("x-lint" = "> 0"))]`
posé sur le champ, **à maintenir à la main avec la règle de `lint.rs`** (une règle de lint ajoutée sans
annotation n'apparaît pas dans `describe`). Un champ `Fixed`
porte `#[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]`.

**Créer et tester** : `alacod new-content [jeu] <Kind> <id> [--from <modèle>]`
(`crates/content/src/scaffold.rs`) copie un modèle du jeu (fichier `<dossier>/<id>.ron`, ou entrée
dupliquée à la fin d'une table comme `weapons.ron`), réécrit l'id et le nom, relance le lint. Pas
pour `Map`, `Ui`, `Camera`, `Economy`, `Difficulty`. `alacod test [jeu] [id…] [--list] [--bless]
[--json]` joue les scénarios générés (§28, §54) par `alacod-gen --only` (le crate `scenario`
dépend de `content`, donc `alacod` appelle `cargo run -p scenario` dans l'engine : celui du jeu
(`Cargo.toml`), de `--engine`, ou d'où `alacod` a été compilé). Traces ignorées sauf `--bless` ;
sortie JSON : `scenario`, `frames`, `hits`, `expectations`, `expectations_ok`, `failures`. Un jeu
hors de l'engine est retrouvé par `ALACOD_GAME_NAME`/`ALACOD_GAME_DIR` (posés par `alacod-gen`) ;
ses scénarios sont écrits sous `<jeu>/tests/generated`. Les armes se jouent sur la
`generate_template` du manifeste (sinon dans le testbed de l'engine, où une arme nouvelle n'existe
pas).

**Jeu hors de l'engine** : `cargo install --path crates/content --bin alacod`, puis
`alacod new mon-jeu [--engine <racine>] [--git [--rev <sha>]]` copie `templates/starter/` (contenu minimal qui passe le
lint : joueur, quatre armes, une arène) dans `./mon-jeu`, avec un `Cargo.toml` à dépendances par
chemin vers l'engine (ou par git avec `--git` : URL de `origin` et révision, qui doit être poussée) (+ `[workspace]` et `[patch.crates-io]` `bevy_ecs_ldtk`, que cargo n'applique
qu'à la racine), le `Cargo.lock` de l'engine, un `CLAUDE.md` et un hook de lint. `lint` et
`describe` sans chemin trouvent le jeu depuis le dossier courant. Le gabarit est un instantané
d'assets du testbed : à resynchroniser si le format du contenu change (le lint du gabarit échoue
alors).

**Rechargement à chaud** : hors partie (`AppState::LobbyLocal`/`LobbyOnline`), si un
fichier de contenu suivi par bevy change (feature `native`, `bevy/file_watcher`), le
registre est relu depuis le disque et le lint relancé
(`crates/game/src/content_hot_reload.rs`, idiome `MessageReader<AssetEvent<T>>` repris de
`crates/game/src/ui/hud.rs`) ; un contenu invalide laisse l'ancien registre en place
(erreur journalisée). Jamais pendant une partie (`GgrsSchedule`) : les snapshots rollback ne
se réécrivent pas à chaud.
Le registre `CharacterConfigs` suit le même événement (`AssetEvent::Modified` d'un personnage) : il est recopié dans
`PreUpdate` (journal `personnage « … » rechargé`), hors partie comme en partie (dev seulement, comme avant avec `Assets`).

---

## 4. Ajouter un vocabulaire à l'engine : checklist

Exemple : ajouter un nouveau type d'ennemi, un effet, ou un statut. Suivre ce flux avant le code (plan §9.7). Chaque point est une décision qui touche l'architecture ; les valider avant d'écrire.

1. **Écrire le test (scénario)** : créer `tests/scenarios/<nom>.ron` avec attentes (`expect: [Expectation]`), invariants de frame, et commentaire « À regarder » décrivant ce qu'on doit observer à l'écran (quelles frames, quel comportement). Le scénario est le cahier des charges ; le code le réalise.
2. **Déclarer le kind** : enum Rust sérialisable en RON dans la crate de vocabulaire (`crates/combat`, `crates/behaviors` ou `crates/effects`). Un vocabulaire déclaré par un plugin s'enregistre auprès de `KindRegistry` (trait de `sim_core`, T0.2) ; un **kind de contenu** (dossier de `game.ron`) s'ajoute à `content::registry::KNOWN_KIND_NAMES` et au `match` de `Registry::build`.
3. **Composants et état** : déclarer tous les composants Bevy qui portent l'état. **Pas de HashMap, HashSet**, uniquement BTreeMap/BTreeSet pour l'ordre déterministe. Consulter `CLAUDE.md` Règles critiques du déterminisme.
4. **Enregistrement rollback** : `app.rollback_and_trace::<StatusComponent>()` pour chaque composant mutable, ou sa variante `_neutral` pour un type nouveau posé seulement sur de nouvelles entités (piège de parité du checksum) ; ressources : `app.rollback_and_trace_resource::<StatusResourceCache>()`. Aucun appel direct à `rollback_component_*`/`rollback_resource_*` (plan §9.6, K0, `scripts/check-rollback-registration.sh`). La règle complète et ses pièges font foi dans `CLAUDE.md` (« Checklist pour Nouveau Système GGRS »).
5. **Flux RNG dédié** (plan §4.6) : si le vocabulaire utilise l'aléatoire (variantes, direction), créer un `stream` unique (ex. `rng_streams.get_mut("status")`, `RngStreams`, `crates/bevy_fixed/src/rng.rs`). Consommer le RNG dans un ordre déterministe (après tri par `GgrsNetId`), jamais à la première occurrence.
6. **FrameEvents** : tous les événements émis (impact, mort, soin) passent par `FrameEvents<T>` (`crates/sim_core/src/frame_events.rs`, réexporté par `game::frame_events`). Lus par les systèmes de la simulation (`GgrsSchedule`), ordonnés après l'émetteur. La présentation en dérive (`Update`, jamais d'événements), restant juste après un rollback.
7. **Scénario et trace** : `make test_scenarios SCENARIO=<nom>` sans `BLESS` la première fois. Si divergence, corriger le code. Une fois vert : `BLESS=1 make test_scenarios SCENARIO=<nom>` écrit la trace de référence. Cette trace est comparée à chaque commit.
8. **Vidéo et validation** : `make videos SCENARIO=<nom>` produit la vidéo dans `target/videos/<commit>/` ; pour regarder des images fixes, `cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/<nom>.ron --capture <dossier> --every 2` (PNG 960×540, une image toutes les 2 frames). Moments clés détectés (vague, kills, coups reçus, morts, rechargements, changements d'arme, fenêtres, portes, armes lâchées/ramassées, power-ups, victoire/défaite/abandon) visibles et cliquables dans la page de revue.
9. **Doc** : ajouter une ligne ou un paragraphe à `docs/conventions.md` décrivant le kind, ses champs, les attentes, les invariants, et les fichiers lus.

---

### 4.3 Contrats de M1 (squelettes)

T1.0a fixe le vocabulaire ci-dessous. **La vague 1 étend chaque enum dans son crate** ;
un renommage doit être signalé dans son rapport. Aucun système n'exécute ces contrats
pendant la vague 0, et aucune trace de référence ne change.

| Crate | Contrats | Plugin / futurs sets |
|---|---|---|
| `combat` | `ProjectileModifier` : Bounce, Pierce, Size, Lifetime, Homing, Gravity ; `Pattern` : Aimed, Spread, Ring, Sequence, Telegraph, Wait (+ `Scatter`, `Named`, T1.2) ; `StatusDef` : Burn, Slow, Stun, Freeze ; état `Statuses(Vec<StatusEntry>)` (statut, piles, frame d'expiration) | `CombatPlugin` ; Projectiles, Effects, Status |
| `behaviors` | `Behavior` : Chase, KeepDistance, Strafe, Charge, Shoot, Melee, Flee, Wander ; `Perception` : Sight, Hearing ; `Targeting::Nearest` ; état `BehaviorState` (indice de règle, frame d'entrée, cible GgrsNetId) | `BehaviorsPlugin` ; EnemyAI |
| `effects` | `Effect { on, if, do }`, `On` (12 déclencheurs depuis `OnLevelUp`, T1.10), `Condition` (7 conditions), `GaugeThreshold::Above` ; réutilise `Action` des power-ups | `EffectsPlugin` ; Effects |

Les plugins enregistrent les noms Rust exacts (PascalCase) dans les catégories snake_case
`projectile_modifier`, `pattern`, `status`, `behavior`, `perception`, `targeting`,
`effect_trigger`, `effect_condition`. T1.12 a ajouté la validation des références dans le
contenu. `Kinds` et `KindRegistry` restent inchangés. `Statuses` et `BehaviorState` sont
enregistrés ensemble en rollback + checksum + trace ; `Statuses` est posé par l'`on_hit`
`ApplyStatus` depuis T1.3 (§19), `BehaviorState` reste sans porteur (T1.4 garde l'état dans
`BehaviorRuntime`, §22). Il n'y a pas de nouveau `RollbackSystemSet`.

Les quantités `Fixed` restent des chaînes (§2), les compteurs et durées des entiers en
frames, les angles des radians et les vitesses des unités/seconde. `Chase` porte un id de
profil (`"Ground"`, etc.) ; `Shoot`/`Melee` portent un id de pattern/arme. T1.4 les résoudra,
sans dépendance de `behaviors` vers `game` ou `combat`. `PerceptionConfig` compose une liste
de sens et `needs_light` (vue seulement). Les listes de séquence, conditions, actions et
statuts conservent l'ordre déclaré.

```ron
Sequence([Telegraph(60), Aimed(count: 4, spread: "0.1", projectile: "plomb"), Wait(180)])
Ring(count: 12, speed: "150", projectile: "braise", every: 90)
Chase(profile: "Ground")
Nearest(ignore: ["disguised", "ghost"])
(on: OnGauge("sacre", Above("0.75")), if: [HpBelow("0.2")], do: [RefillAmmo])
```

Le déplacement des armes met simulation, données des acteurs et géométrie de collision
dans `combat`. `game::weapons`, les anciens chemins des composants de personnage,
`game::collider` et `game::collision_grid` restent des réexports. Les attentes typées du
champ `test.expect` vivent dans `combat::weapons::expectations` et sont réexportées par
`game::replay` : métadonnées d'outillage, exclues du hash des armes, jamais exécutées par
`combat`. Les sprites attachés, slashs et HUD restent dans `game::ui` ; les handlers d'achat
et de ramassage et le chargement des armes murales restent dans `game`/`map_ldtk`.
`Interactable`/`InteractionType` sont des contrats `sim_core::interaction`, réexportés par
`game::interaction`, ce qui permet aussi à `combat` de créer une arme lâchée interactable.

---

## 5. Commandes make

| Cible | Rôle |
|---|---|
| `make check` | Format, tests unitaires, scénarios, motifs interdits en avertissement, enregistrement rollback (`check_rollback_registration`, strict) (la CI rapide) |
| `make test_scenarios` | Rejoue tous les scénarios en headless, compare les traces |
| `make test_scenarios_par [SHARDS=4]` | Même suite en N processus, tranches réparties par durée mesurée (`scripts/scenario-shard-plan.py`) |
| `make map_preview ARGS=<fichier.ldtk>` | Affiche une carte LDtk (fenêtre) |
| `make map_generation ARGS=<entrée.ldtk> <sortie.ldtk> <graine>` | Génère une carte assemblée par connexions (voir `examples/map_generation.rs` pour les arguments exacts) |
| `make map_render MAP=<jeu ou bibliothèque.ldtk> SEEDS=1,2,3 [ARGS="--floor etage --plain"]` | `alacod-map` : PNG avec tuiles et `.ldtk` assemblé par graine dans `target/map_preview/`, sans le jeu (§1, « Concevoir une bibliothèque de pièces ») |
| `make play_scenario SCENARIO=<nom>` | Rejoue le scénario avec rendu (l'option `--follow <handle>` du binaire suit un joueur) |
| `make record_session NAME=<nom>` | Enregistre une partie en scénario |
| `make remote [HEADLESS=1]` | Lance une partie en pause, pilotée par `scripts/alacod-remote` |
| `make videos [SCENARIO=<nom>]` | Encode les scénarios en vidéos (960×540, `target/videos/<commit>/`) ; plusieurs noms : `SCENARIO=a,b` ou `SCENARIO="a b"` ; `DRY_RUN=1` liste sans compiler (D23) |
| `make review_videos [TAILSCALE=1]` | Serveur de revue des vidéos (`localhost:8766`) |
| `make diff_log CID_1=alice CID_2=bob` | Compare les logs d'état entre deux clients (déterminisme) |

Voir `Makefile` pour les détails (cibles `test_multiplayer`, etc.).

**Parties de référence du clone `zombies` (T3.1)** : `clone_solo`, `clone_duo` et
`clone_quad` (`tests/scenarios/`, `exemples/test_map.ldtk`, graine 123456) suivent les
vagues 1 à 5 avec achats du pistolet et de Juggernog, Max Ammo et Nuke placés, et des
attentes datées. Le duo ajoute le tir ami bloqué et une réanimation ; le quad combine
deux bots `fonceur`, un `prudent` et un joueur scripté qui réanime un fonceur. Les
commentaires « À regarder » donnent les frames de revue, mesurées avec
`ALACOD_EVENTS=1 make test_scenarios SCENARIO=<nom>`. Les visées scriptées sont
enregistrées en headless, puis rejouées comme inputs fixes ; les bots du quad restent
pilotés par leur profil.

Les effectifs sont réduits par `wave_overrides`. Deux champs optionnels de ce réglage
complètent les effectifs et la préparation : `min_wave_delay_frames` raccourcit
l'attente entre vagues, `max_wave` fixe la condition de victoire. Absents, ils conservent
les valeurs du jeu. Solo et duo déclarent `max_wave: 5` : le mode `Waves` termine à
l'**entrée** en vague 5 et produit le résumé (la cinquième vague n'est pas combattue).
Le quad laisse cette limite absente et attend `RunState(Playing)` avec quatre joueurs
debout à l'entrée en vague 5. Aucune trace antérieure n'est blessée pour ces références.

**Vitesse de la suite (perf-suite-scenarios, 2026-10-14)** : 243 scénarios en ~17 min dans un
seul processus et ~5 min en 4 tranches sur 4 cœurs (avant : ~51 et ~12 min). Ce qui l'explique,
dans l'ordre du gain, et ce qu'il ne faut pas défaire :

- **Exécuteur monothread en headless** (`runner::single_threaded_schedules`) : la simulation
  compte des centaines de petits systèmes par frame, l'exécuteur multithread de Bevy y passait
  plus de temps à se réveiller (~1 500 `futex` par frame) qu'à calculer, et ses threads se
  disputaient les cœurs entre tranches parallèles. `ALACOD_EXECUTOR=multi` le rétablit. Un
  système sans ordre explicite ne change pas de résultat : `GgrsSchedule` refuse toute ambiguïté
  d'ordre (les 243 traces le prouvent).
- **`FlowFieldCache::layers` en `SharedFlowField`** (`Arc` + image des octets du `Hash`) : le
  snapshot rollback le clonait et le checksum le hachait 2 à 3 fois par frame (~23 000 entrées
  de `BTreeMap`, ~20 % d'une frame). Un champ de flux n'est jamais modifié après sa
  construction : toute nouvelle couche passe par `SharedFlowField::from`. Le flux d'octets du
  `Hash` est celui de `#[derive(Hash)]` sur `FlowField` (test
  `shared_flow_field_hash_equals_derived`) : aucune trace ne bouge.
- Contenu (`Registry`, `GameManifest`) chargé **une fois par processus** (`runner::load_content`),
  pas de lecture des périphériques (depuis D69 E4 `character::app_plugin::CharacterAppPlugin` n'ajoute ni leafwing
  ni `collect_device_inputs` en headless ; `skip_device_input` n'existe plus) et overlay de vagues (texte d'UI) coupé en headless.
- **Tranches par durée** : `scripts/test-scenarios-sharded.sh` lit `sim_seconds` dans
  `target/metrics/latest.json` (sinon les `frames` du `.ron`) et remplit les tranches du plus
  long au plus court (plus long d'abord, sur la tranche la moins chargée). Les métriques d'un
  run partiel écrasent `latest.json` : une suite complète les remet à jour. `SHARD_PLAN=roundrobin`
  rétablit un scénario sur N.
- `ALACOD_PROFILE=1` affiche, par scénario, le temps du runner par poste (`update` de
  Bevy, métriques, invariants) ; les invariants et les attentes pèsent moins de 0,5 % d'une frame.

**perf-navigation (2026-10-15)** : `build_flow_field` et `BotNavigation::build_field` calculent sur
tableaux `Vec` denses (sortie reconstruite en `BTreeMap` triée, même `Hash`) ; `throne_quad`
37,6 → 102 fps, `bench_floor` 96 → 121. Les calculs historiques (`build_flow_field_legacy`,
`build_field_legacy`) restent comme références de test : tout changement de l'un des deux doit
rester égal case par case. Détail : `docs/taches/rapports/perf-navigation.md`.

Ce qui restait avant perf-navigation était la simulation elle-même : `bench_floor` (profil callgrind, 600 frames) passe
~42 % de ses instructions dans `update_flow_field_system` (Dijkstra, dont ~22 % dans
`is_blocked`) et ~14 % dans la navigation des bots ; les `throne_*` ne sont que 1,5 à 2 fois
plus rapides qu'avant, pour la même raison probable (non profilés). Le détail chiffré est dans
`docs/taches/rapports/perf-suite-scenarios.md`.

---

## 6. CI lente (nuit) — T2.14

**Quoi** : chaque nuit (03:03 UTC) ou à la demande (`workflow_dispatch`), un runner auto-hébergé (`[self-hosted, alacod-builder]`) exécute un pipeline complet : benchmarks strict, simulation multi-bots, p2p headless (2 et 4 joueurs via allumette, détection des desyncs), génération des vidéos de tous les scénarios, page de revue interactive, et notes Markdown commitées dans `tests/review-notes/<commit>.md`.

**Où** : `.github/workflows/nightly.yaml` (workflow), `scripts/nightly.sh` (pipeline), `docker-compose.ci.yaml` (allumette headless), `Makefile` (`make nightly`, `make nightly_quick`).

**Comment lancer localement** (rapide, 2 graines, vague 2, vidéos idle+shoot_around) :
```bash
make nightly_quick
```

Résultats dans `target/nightly/<commit>/summary.md`, `sim.json`, `review/index.html`, `p2p-*.trace`, et `tests/review-notes/<commit>.md` (sans commit).

**Options** (full : 50 graines, vague 10, vidéos toutes, 2 et 4 joueurs p2p) :
```bash
NIGHTLY_SEEDS=1..100 NIGHTLY_BOTS=4 NIGHTLY_WAVE=15 NIGHTLY_P2P=2,4,8 NIGHTLY_VIDEOS=all bash scripts/nightly.sh
```

**Étapes du pipeline** :

1. **Bench** (seuils de `tests/budgets.ron`) : rejeu de tous les scénarios en headless, traces strictes ; les budgets absolus de `tests/budgets.ron` sont désactivés en CI (machine environ 4× plus lente, `scripts/nightly.sh`) : la régression de FPS se lit dans le tableau de `scenario-metrics.py`.

2. **Sim** (simulation bots) : centaines de parties sans rendu à N bots sur M graines jusqu'à la vague K, détecte les crashes et deadlocks ; JSON de métriques par graine (taux de survie, ennemis tués, vague atteinte, FPS moyenne). L'absence de crash = pas de desync en single-player.

3. **P2P headless** (2 et 4 joueurs) : lance une instance d'allumette (docker compose), puis N clients `zombies` en headless dans le même lobby sans rendu. Chaque client écrit sa trace d'état à chaque frame (`ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME=600`). À la fin, compare les traces : identiques = pas de desync, différentes = log la première frame qui diffère et échoue (information précieuse pour debug).

4. **Vidéos** : encode chaque scénario en MP4 (960×540, 60 FPS), stocke dans `target/videos/<commit>/`.

5. **Page de revue** : `scenario-review.py` génère une page HTML (`scripts/scenario-review.html`, copiée en `review/index.html` avec un `data.js`) avec tableau des scénarios (trace, FPS, moments clés détectés : vagues, kills, morts, tirs, rechargements, perte de santé) ; tableau d'historique par commit avec courbes FPS (benchmark vs sim).

6. **Notes** : Markdown généré (`tests/review-notes/<commit>.md`) avec lien vers la page de revue, timestamp, commit, résumé des étapes (✅/❌), métriques agrégées (FPS min par scénario, nombre de desyncs, p2p clients testés).

**Résultats** (artefacts GitHub Actions) :
- Vidéos MP4 (évaluables sans machine avec GPU) : `nightly/<commit>/videos/`, copiées de
  `target/videos/<commit>/` après `make videos` (`scripts/nightly.sh`, m1-dettes-doc-lot-2)
- Page de revue HTML (tableau des FPS, moments clés, comparaison historique)
- Traces P2P (fichiers texte, premiers desyncs identifiables avec `diff`)
- JSON de métriques (entrées pour graphiques, alertes de régression)

**Si p2p échoue** : examine les logs dans `logs/p2p-<N>-<cid>.log` (socket, timeout, spawn, GGRS frame error) ; les traces dans `target/nightly/<commit>/p2p-*.trace` montrent la divergence. Deux clients en headless => 10 minutes avec `--quick`.

---

## 7. Feedback (présentation, T2.13)

**Configuration RON** : fichier `games/<jeu>/assets/ui/feedback.ron` charge les paramètres de flash, secousse et sons.

```ron
(
    hit_flash: (
        frames: 4,                  // Durée du flash blanc (frames de simulation)
        color: (2.5, 2.5, 2.5),     // Multiplicateur des calques (> 1 = surexposé ; (1, 1, 1) invisible)
    ),
    shake: (
        frames: 8,                  // Durée de la secousse
        amplitude: 4.0,             // Amplitude du décalage en pixels
    ),
    sounds: {
        // Sons de base. Tir : clé de la famille du mode de tir actif de l'arme (Manual :
        // shot_pistol, Shotgun : shot_shotgun, Automatic/Burst : shot_smg), repli sur "shot".
        "shot_pistol": "sounds/shot_pistol.ogg",
        "shot_shotgun": "sounds/shot_shotgun.ogg",
        "shot_smg": "sounds/shot_smg.ogg",
        "reload": "sounds/reload.ogg",
        // Événements (lot B) : coup porté par un joueur local, dégâts reçus, ennemi tué,
        // dash, blank, porte ouverte.
        "hit": "sounds/hit.ogg",
        "hurt": "sounds/hurt.ogg",
        "death": "sounds/death.ogg",
        "dash": "sounds/dash.ogg",
        "blank": "sounds/blank.ogg",
        "door": "sounds/door.ogg",
        // Les clés absentes désactivent le son correspondant
    },
    // Volume par clé de `sounds`, en dB (0 : inchangé, plage [-60, 12]) ; un son d'arme
    // (`audio_config`) prend le volume de la clé qu'il remplace.
    volumes: {"hit": -3.0},
)
```

**Son propre d'une arme** (`weapons.ron`, champ `audio_config` de l'entrée, à côté de
`config`/`sprite_config`) : un tir et un rechargement par mode de tir, chacun facultatif.
```ron
"pistolet": (
    config: (...),
    sprite_config: (...),
    audio_config: (modes: {
        "default": (firing: "sounds/shot_pistol.ogg", reloading: "sounds/reload_pistolet.ogg"),
    }),
),
```
Priorité du tir : `firing` du mode actif de l'arme, sinon la clé de sa famille dans `sounds`,
sinon `shot` ; du rechargement : `reloading` du mode actif, sinon `reload`. `audio_config` reste
sur l'asset `WeaponAsset` (lu par `game::feedback::WeaponAudio` dans
`WeaponRegistry`) et n'est jamais copié dans le composant rollback `Weapon` : aucune
trace ne bouge. Lint : fichiers présents sous `assets/`, mode connu de `firing_modes`.

**Systèmes** : tous en `PostUpdate` (hors `GgrsSchedule`), lisant les événements de simulation dans `FrameEvents<T>` émis par `GgrsSchedule`. Les trois émetteurs sont :
- `DamageEvent` pour les impacts (flash, secousse si joueur local)
- Spawn de `Bullet` pour le son de tir (source = joueur local) : un son par tir (tireur, frame de création), mémorisé pour ne pas repartir quand le rollback de la session locale recrée les balles des dernières frames (elles redeviennent `Added<Bullet>`). Son : celui de l'arme active du tireur et de son mode actif à l'image rendue, selon la priorité ci-dessus (familles : `Shotgun` → `shot_shotgun`, `Automatic`/`Burst` → `shot_smg`, `Manual` → `shot_pistol` ; la balle ne porte pas son arme : un champ de plus ferait bouger les traces). Les échantillons sont des coups uniques courts (≤ 0,5 s, lot A de `docs/plan-assets.md`), joués en entier : plus de coupure en fondu.
- Passage de `WeaponInventory.reloading_ending_frame` de `None` à `Some` pour le son de rechargement (celui de l'arme active, priorité ci-dessus) : une fois par rechargement, comparé d'une image rendue à la suivante
- Sons d'événements (lot B), joués par `FeedbackSounds::play(clé)` ; chaque signal est lu dans l'état ou dans les `FrameEvents` de la dernière frame simulée, jamais écrit :
  - `hit` : un `DamageEvent` dont la source est un joueur local et la cible non, un son par frame simulée (gardé par `DamageEvent.frame` : une image rendue sans nouvelle frame voit encore les mêmes événements). L'événement précède la résolution : un coup annulé (invulnérabilité) sonne aussi.
  - `hurt` : `Health.current` d'un joueur local en baisse d'une image rendue à l'autre, à `max` égal ; au plus un son toutes les 15 frames (`HURT_SOUND_MIN_FRAMES` : les PV peuvent baisser deux frames de suite).
  - `death` : un ennemi portant `Death` et `RollbackDespawned` (requête qui nomme ce composant désactivant), une fois par `GgrsNetId`.
  - `dash` : front montant de `DashState::is_dashing` d'un joueur local.
  - `blank` : `FrameEvents<BlankUsed>` d'un joueur local, gardé par la frame de l'événement.
  - `door` : une porte avec collider à l'image précédente, sans collider maintenant ; rien pour l'ouverture d'office des portes de salle (`RoomStates::doors_opened` qui passe à `true`, début d'étage) ni à la première image (référence). Pas encore de son de coffre : les coffres (M2-T5) ne sont pas dans `main`.

**Composants non-rollback** :
- `HitFlash { until_frame, color }` : posé sur l'entité cible d'un `DamageEvent`, multiplie la couleur des calques enfants (`apply_layer_colors`) jusqu'à `until_frame`.
- `CameraShake { until_frame, frames, amplitude, applied }` : pose sur la caméra quand un joueur local prend des dégâts.

**Déterminisme** : la secousse applique un motif déterministe (décalage indexé par `FrameCount`) pour la reproductibilité des captures (`--capture` de `play_scenario`). Jamais de source aléatoire (`rand`, temps réel).

**Journal de preuve** : chaque effet écrit une ligne `info!("feedback f{frame} <effet> {net_id|kind}")` pour vérification sans écran.

**Feedback v1** (T1.17) : hit stop, chiffres de dégâts, télégraphe au sol, surcharges par genre et par arme, flash sur les calques enfants et journal `FeedbackLog` (headless compris) : voir §31, qui fait foi sur ce paragraphe.

## 8. Combat : équipes et dégâts (T1.1, chantier B1)

**Équipe** (`sim_core::team::Team` : `Players`, `Enemies`, `Allies`, `Neutral`) : composant statique posé une fois à la création, par `character::player::create::create_player` (`Team::Players`) et `character::enemy::create::spawn_enemy` (paramètre `team` : `Team::Enemies` pour les vagues, sinon `CharacterSpawn.team` ou `CharacterConfig::team` ; `Allies`/`Neutral` servent à `ally`/`civilian` du testbed, T2.9). Hors rollback (jamais muté en T1.1 ; voir la doc du composant pour la justification). `Allies` compte comme la même équipe que `Players` pour le tir ami ; `Enemies` est sa propre équipe. `Neutral` bloque toujours un coup (comme un mur) mais ne subit jamais de dégât.

**`DamageEvent`** (`sim_core::damage`, `FrameEvents<DamageEvent>`) : toute blessure passe par là — six émetteurs (`weapons::bullet_rollback_collision_system`, `weapons::melee::melee_hitbox_collision_system`, `character::enemy::ai::behavior::enemy_attack_damage_translate_system`, `combat::projectile::projectile_collision_system` (B5), `combat::projectile::status_tick_system` (T1.3), `character::enemy::ai::rules::charge_damage_translate_system` (T1.4)), un seul résolveur (`character::health::rollback_resolve_damage_events`, `RollbackSystemSet::CollisionDamage`, après les émetteurs, avant `DeathManagement`). Le résolveur applique `combat::damage::resolve_damage` (fonction pure, testée dans `crates/combat/src/damage.rs`) puis écrit `DamageAccumulator`/`HitBy` (appliqués à `Health` par `rollback_apply_accumulated_damage`, inchangé).

**Politique de tir ami** (`sim_core::damage::FriendlyFire` : `Never`/`Always`/`Cursed`, `#[serde(default)]` = `Never`) : champ RON `friendly_fire` sur `WeaponConfig` (`weapons.ron`), `MeleeWeaponConfig` (`melee_weapons.ron`) et `EnemyAiConfig` (en dur, pas encore RON). `Cursed` ne touche un allié que si la source porte le tag `cursed` (`combat::team::CURSED_TAG`). **D54** : quelle que soit la politique, une balle ne touche jamais son propre tireur (`bullet.source`) : exclu dans `bullet_rollback_collision_system`, dans la collision des projectiles composables et dans le choix de cible de `Homing`/`Aimed` (`nearest_target`) ; un test de tir ami a donc besoin d'un second tireur (scénario `dash_stun_buffer`, 3 joueurs).

**Tags et défenses** : `CharacterConfig` (`character/config.rs`) gagne trois champs RON, tous `#[serde(default)]` (vides) : `tags: [...]` (posé en composant `sim_core::tag::Tags`, ex. `["cursed"]`, `["zombie"]`), `immune_to: [...]` et `resistances: {"tag": "mult"}` (posés ensemble en composant `combat::damage::Defenses`). `Tags`/`Defenses` sont hors rollback, comme `Team`. Chaque `DamageEvent` porte `tags: Tags` = tags du personnage source **union** un tag de genre d'attaque posé par l'émetteur (`bullet` pour une balle, `melee` pour une attaque au corps à corps — arme ou griffe) : une griffe de zombie porte donc `melee` et `zombie` (le tag vient de `CharacterConfig::tags` du zombie). Une cible dont `immune_to` contient un de ces tags ne subit aucun dégât ; `resistances` multiplie le montant à la place. `DamageKind::True` ignore résistances, immunités et invulnérabilité (mais pas l'équipe/le tir ami).

**Invulnérabilité** : `Health.invulnerable_until_frame` (déjà présent, T0.2) est enfin lu par `resolve_damage` — aucun dégât tant que `frame <= invulnerable_until_frame`.

**Ce qui reste de `CollisionLayer`/`layer_matrix`** (`crates/combat/src/collider.rs`, `CollisionSettings::layer_matrix`) : uniquement les collisions **physiques** avec les murs (balles et personnages qui s'arrêtent sur un mur) et entre personnages/fenêtres. La décision « cette balle/cette mêlée touche-t-elle ce personnage » ne passe plus par la matrice : elle vient de `Team` + `combat::team::team_allows_hit` (équipe + politique de tir ami de l'arme), évaluée pour chaque personnage candidat indépendamment de son `CollisionLayer`.

**Scénarios de référence** (`tests/scenarios/`) : `friendly_fire_never.ron`, `friendly_fire_cursed.ron`, `immune_tag.ron`. Format `Scenario` étendu (`game::replay`) : `PlayerScript` gagne `tags`/`immune_to` (posés en composants après création par `scenario::runner::apply_player_overrides`) ; `WeaponOverride` gagne `friendly_fire` (posé sur l'arme entière, avant la création des joueurs, par `apply_weapon_overrides`).

**Bug pré-existant préservé (pas corrigé par T1.1)** : avant ce chantier, l'attaque directe de `EnemyAiConfig::attack_damage` (`enemy_attack_system`) n'avait presque jamais d'effet — elle écrivait dans `DamageAccumulator` sans `Option`, composant retiré dès qu'appliqué la même frame, donc absent la plupart du temps. Seule la griffe via hitbox (`MeleeWeaponConfig::damage`, ex. `zombie_claws` à 2.5) touchait vraiment le joueur. `enemy_attack_damage_translate_system` reproduit ce comportement à l'identique (garde explicite, voir sa doc) : le corriger changerait l'équilibrage (dégâts ennemis beaucoup plus fréquents) sans rapport avec ce chantier.

## 9. Stats et modificateurs (T1.2, chantier B2)

**Contrats** (`sim_core`) : `stats::{StatId, Stats}` (identifiants et valeurs de base, `BTreeMap<StatId, Fixed>`) et `modifier::{ModifierOp, ModifierSource, Modifier, Modifiers, resolve}`. `StatId` est un enum ouvert (`Custom(String)` pour un vocabulaire propre à un jeu) ; `ModifierOp` a quatre opérations `Set` (dernier gagne), `Add` (somme), `Pct` (pourcentage additif, `0.5` = +50 %, plusieurs `Pct` s'additionnent avant de multiplier une seule fois) et `Mul` (produit direct). `resolve(base, modifiers, frame)` calcule `(base + Σ Add) × (1 + Σ Pct) × Π Mul`, `Set` appliqué d'abord ; un `Pct` absent ne multiplie même pas par 1 (aucune dérive possible en l'absence de modificateur). Le crate ne pose rien sur aucune entité — c'est le rôle de `crates/stats` et de `character::create::create_character`.

**Branchement** (`crates/stats`) : `StatsPlugin` enregistre `Stats`/`Modifiers` en rollback (`RollbackTraceApp`, donc dans le `Checksum` GGRS et les traces) et ajoute `expire_modifiers_system` (`RollbackSystemSet::Status`), qui retire les modificateurs expirés de chaque entité à la frame courante (`Modifiers::retain_active`). `StatReader` (`SystemParam`) est le point de lecture unique : `stats.get(entity, &StatId::X, valeur_par_défaut)` résout la stat (base + modificateurs actifs) à la frame courante ; `stats.try_get(entity, &StatId::X)` renvoie `None` plutôt qu'un défaut si l'entité n'a pas la stat (utilisé quand réutiliser une valeur déjà en place comme base la ferait dériver sous un modificateur multiplicatif, voir `character::health::sync_health_from_stats`). Aucun système de `crates/game` ne rappelle `sim_core::modifier::resolve` directement (seule exception, assumée : `character::variant::variant_health`, santé de départ d'une variante au spawn, T1.5 — à ce moment `Stats`/`Modifiers` n'existent pas encore sur l'entité, il résout donc lui-même les modificateurs `MaxHealth` de la variante sur la santé de base).

**Modificateurs sourcés** : `Modifiers::push_from(source, stat, op, value, until)` construit et ajoute un modificateur ; `Modifiers::remove_by_source(&source)` retire tous les modificateurs d'une même source d'un coup — un statut qui expire ou une salle qu'on quitte appelle cette méthode avec le `ModifierSource` donné à la pose, sans connaître le détail des modificateurs posés (les chantiers de statuts B3 et de salles E1 s'en serviront).

**Stats posées à la création** (`character::create::create_character`) : pour tout personnage, `MoveSpeed`/`EnemyMoveSpeed` = `movement.max_speed`, `Acceleration` = `movement.acceleration`, `SprintMultiplier` = `movement.sprint_multiplier`, `MaxHealth` = santé max résolue (`ResolvedBalance::health_max_by_character`, F5, §18), `HealthRegen` = `base_health.regen_rate` si présent, et les multiplicateurs `FireRate`/`ReloadSpeed`/`Damage`/`Range` à 1 (produit neutre). Pour un ennemi (`enemy::create::spawn_enemy`), en plus : `SeparationDistance`/`SeparationForce`/`SlowDownDistance`/`OptimalAttackDistance` viennent de `PathfindingConfig::default()` (la constante jusque-là partagée par tous les ennemis, lue par `move_enemies`). `CharacterConfig::stats` (RON, `#[serde(default)]`) surcharge ensuite ces valeurs de base, en dernier — un personnage qui ne déclare rien dans `stats:` a un comportement strictement identique à avant ce chantier. Exemple RON :

```ron
stats: {
    MoveSpeed: "180.0",
    Custom("sacre"): "1.0",
},
```

**Lecture branchée** : `character::player::input::apply_inputs` (accélération, multiplicateur de sprint, vitesse max — aussi la vitesse de sortie du dash, §34), `character::enemy::ai::pathing::move_enemies` (vitesse, séparation, ralentissement, distance d'attaque optimale — la granularité de la grille spatiale de séparation reste la constante globale de `PathfindingConfig`, seulement pour le partitionnement, voir le commentaire dans `move_enemies`), `weapons::weapon_rollback_system`/`spawn_bullet_rollback` (cadence de tir, temps de rechargement, dégâts et portée d'une balle, multipliés par les stats du porteur) et `character::health::sync_health_from_stats` (`Health.max` et `HealthRegen.regen_rate`, recalculés chaque frame après l'expiration des modificateurs — `current` est borné à `max` s'il le dépasse, jamais relevé automatiquement). Le freinage (`movement.deceleration`, multiplié par l'adhérence `Acceleration` résolue / `movement.acceleration`, §34) et les champs de dash (`dash_distance`, `dash_duration_frames`, `dash_cooldown_frames`, `dash_iframes`, `dash_buffer_frames`) et de sprint (`sprint_acceleration_per_frame`/`sprint_deceleration_per_frame`) restent des lectures directes de `CharacterConfig` : ils ne sont pas dans la liste de stats de la tâche T1.2.

**Scénario de preuve** (`tests/scenarios/stat_move_speed.ron`) : deux joueurs avancent avec le même script d'inputs, l'un avec un modificateur `(stat: MoveSpeed, op: Mul, value: "0.5")` posé après création (`PlayerScript::modifiers`, comme `tags`/`immune_to`, appliqué par `scenario::runner::apply_player_overrides`), l'autre sans — `PlayerPosition` aux mêmes frames pour les deux, lecture directe de l'écart.

## 10. Blesser une trace : la preuve

Enregistrer un nouveau composant/ressource en rollback (`RollbackTraceApp`) change le `Checksum` GGRS de chaque frame, donc **toutes** les traces de référence (`tests/scenarios/*.trace`) changent, même quand aucune valeur de jeu ne bouge. Avant de blesser (`BLESS=1`), il faut prouver que c'est bien le cas : que la nouvelle trace ne diffère de l'ancienne que par le nouveau composant lui-même.

**Piège de parité des types vides** (mesuré le 2026-10-03, T1.2) : un type rollback enregistré
sous checksum mais porté par aucune entité ajoute une `ChecksumPart` constante `hash(0u64)`,
combinée par XOR : **un** type vide de plus change toutes les traces dès la frame 0, **deux** les
laissent intactes. Un composant qui n'est posé que sur de nouvelles entités s'enregistre avec la
variante **neutre** (`rollback_and_trace_neutral::<C>`, `rollback_and_trace_resource_neutral`) ;
on ne compense jamais par la parité et on ne passe jamais en `no_checksum` pour « sauver » des
traces. Détail et mesure : §20.

**Dump complet** (`ALACOD_DUMP_TRACE=<dossier>`) : `StateTraceRecorderPlugin::dump: Option<PathBuf>` (`crates/game/src/state_trace.rs`) fait garder à `StateTraceRecorder` la ligne détaillée (hash puis, comme en mode `full`, une ligne par ressource tracée et une ligne par entité rollback triée par `GgrsNetId`) de **toutes** les frames simulées, sans la limite de `history` (`HISTORY_LEN`, 400). `crates/scenario/src/runner.rs::build_app` lit la variable d'environnement ; `ScenarioOutcome::full_trace` porte ces lignes (`None` si `ALACOD_DUMP_TRACE` n'est pas défini) ; le test `scenarios` (`crates/scenario/tests/scenarios.rs`) les écrit dans `<dossier>/<scénario>.full` pour chaque scénario joué.

**Comparaison** (`scripts/trace-diff.py <a.full> <b.full> [--ignore Nom1,Nom2,...]`) : retire toujours le checksum de l'en-tête (il change dès qu'un composant est ajouté au rollback, ce n'est pas ce qu'on compare) et neutralise toute valeur `Entity` brute embarquée dans un Debug (`{index}v{generation}`, ex. `WeaponInventory.weapons: Vec<(Entity, Weapon)>`) : elle dépend du nombre exact d'entités déjà créées dans **ce process** au moment du spawn (minutage du chargement des assets), pas déterministe d'un lancement à l'autre même à code strictement identique — c'est déjà pour ça que `WeaponInventory` l'exclut de son `Hash` manuel (voir le `Hash` manuel de `WeaponInventory` dans `crates/combat/src/weapons/mod.rs`). Avec `--ignore`, retire en plus les paires `Nom=valeur` dont le dernier segment du nom (après `::`) est dans la liste — sur les lignes de ressource et d'entité (une ligne d'entité empile un tracer par composant présent, `--ignore` ne retire que ceux nommés). Lit les deux fichiers en flux (une frame à la fois) : un dump complet de 1500 frames pèse facilement plusieurs centaines de Mo. Affiche la première frame qui diffère et les lignes en cause ; code de sortie 1 si différent, 0 si identique.

**Méthode** : dumper les mêmes scénarios sur la référence (avant le chantier, ex. `main` dans un `git worktree add --detach`) et sur la branche, avec la même variable d'environnement, puis comparer en ignorant le(s) nouveau(x) composant(s)/ressource(s). Aucune différence : le chantier n'a changé que la présence du nouveau composant, jamais une valeur de jeu — on peut blesser en confiance. Une différence : une valeur de stat (ou autre) ne correspond pas exactement à la constante qu'elle remplace ; corriger avant de blesser.

### Exclusions assumées du checksum

Trois données sont délibérément hors du checksum GGRS — les inclure déplacerait les traces de référence sans changer aucun comportement de jeu (D5, m0-v10) :

- **`HitCount`** (`crates/game/src/character/health/mod.rs`, T2.9) : compteur de coups reçus, posé à la création uniquement si `CharacterConfig::counts_hits` (`crates/game/src/character/create.rs`, faux par défaut — aucun personnage zombie/joueur ne le pose ; seuls `target` (testbed) et `cible` (throne) le déclarent, `crates/scenario/src/generate.rs`). Enregistré via `rollback_and_trace_no_checksum::<HitCount>()` (`crates/game/src/character/mod.rs`) : rollback et trace, mais hors checksum. Il sert aux attentes `EntityHits` des scénarios (`crates/combat/src/weapons/expectations.rs`) ; aucune décision de jeu ne le lit.
- **`EnemyAiConfig::stationary`** (`crates/game/src/character/enemy/ai/state.rs`, T2.9) : immobilité totale d'un ennemi de testbed (`dummy`/`target`/`ally`/`civilian`), exclue du `Hash` manuel du composant. Aucun contenu zombie ne la pose ; hacher un champ de plus déplacerait le checksum de toute entité `Enemy` sans aucun changement de gameplay (vérifié empiriquement à l'époque : sans l'impl manuel, `idle.ron` diverge dès f181).
- **`RunHistory`** (`crates/game/src/profile.rs`, M3-T10, §75) : boss vaincus et objets tenus par joueur, lus **seulement** à `Ended` par `write_achievements_at_run_end` (`Update`). Enregistré via `rollback_and_trace_resource_no_checksum` : il suit les rollbacks mais n'entre pas dans le checksum. Dérivé d'états déjà sous checksum (`Health`/`Death` du boss, `Inventory`), jamais lu par `GgrsSchedule` ; l'inclure déplacerait les traces dès qu'un boss meurt.

## 11. Munitions et inventaire d'armes (T2.2, chantier B7)

**Type de munition** (`sim_core::ammo::AmmoType`) : enum ouvert (`Plomb`, `Balle`, `Cartouche`, `Energie`, `Special`, `Custom(String)`), comme `StatId`. Champ RON `ammo_type` sur `WeaponConfig` (`weapons.ron`), **obligatoire** pour toute arme à distance (pas de `#[serde(default)]` : une entrée sans `ammo_type` échoue au chargement RON, rapportée par `content::lint` comme n'importe quel autre littéral invalide). Contenu `zombies`/`testbed` : `pistol` → `Plomb`, `machine_gun` → `Balle`, `shotgun` → `Cartouche` (trois types distincts aujourd'hui, donc chaque arme a sa réserve propre en pratique). Exclu du hash manuel de `WeaponConfig` (comme `test`, voir la doc du champ `ammo_type` et le `Hash` manuel de `WeaponConfig` dans `crates/combat/src/weapons/mod.rs`) : une vraie valeur de gameplay, mais dont la valeur observable vit dans `AmmoReserves` (rollback), pas dans ce champ de config statique — l'y inclure ferait dériver le checksum de tous les scénarios existants sans qu'aucun comportement ne change.

**Réserves** (`combat::inventory::AmmoReserves`, composant rollback posé sur chaque joueur par `character::player::create::create_player`) : `BTreeMap<AmmoType, u32>`, une entrée par type de munition **effectivement représenté** parmi les armes de départ (rien pour un type jamais rencontré). Initialisée à la somme, par type, de `mag_limit × mag_size` du mode par défaut de chaque arme de départ (`weapons::default_mode_ammo_contribution`) — deux armes qui déclarent le même type s'additionnent (scénario `ammo_shared_reserve`). Une arme `Magless` (fusil à pompe : pas de chargeur séparé, tout son stock est `bullet_limit`) contribue `0` : son ravitaillement ne passe jamais par la réserve, avant comme après ce chantier.

**Rechargement** (`weapons::weapon_rollback_system`) : un rechargement (bouton ou chargeur vide) exige que la réserve du type de l'arme active contienne au moins un chargeur plein (`mag_size` unités) — `WeaponModeState::can_reload(mag, reserve)`. À la fin du délai de rechargement, l'arme retire `mag_size` unités de la réserve et remplit son chargeur (`WeaponModeState::reload`), exactement comme l'ancien `mag_quantity` (un chargeur entier par rechargement, jamais un appoint partiel) — seule l'unité change (munitions plutôt que nombre de chargeurs), pour un total de munitions tirables strictement identique. Une arme `Magless` ne passe jamais ce test (`can_reload` renvoie toujours faux pour `MagBulletConfig::Magless`) : son « rechargement » entre deux tirs (fusil à pompe) reste un délai de pompe qui ne touche jamais la réserve, comme avant.

**Emplacements** (`CharacterConfig::weapon_slots: u32`, `#[serde(default)]` = 2, validé par `content::lint` : `> 0` et `starting_weapons.len() <= weapon_slots`) : nombre d'armes à distance qu'un personnage peut porter. À l'exécution, borne le **ramassage** (voir plus bas) ; la création donne toutes les `starting_weapons`, le lint refuse désormais une liste trop longue (T3.4, D7). `zombies` et `testbed` déclarent `weapon_slots: 3` sur le joueur : ses trois armes de départ étaient déjà créées, aucune n'était perdue.

**Lâcher** (entrée clavier sur nouvel appui uniquement : relâcher puis rappuyer pour une autre arme ; `INPUT_DROP_WEAPON`, touche `L`, bouton `DropWeapon` des scénarios ; `weapons::weapon_drop_system`, avant `weapon_rollback_system` dans `GgrsSchedule`) : l'arme active est retirée de `WeaponInventory.weapons`, son entité despawn (`despawn_rollback`), et une entité rollback `weapons::WeaponPickup { weapon_id, mag_ammo }` apparaît au sol à la position du joueur — `mag_ammo` est celui du **mode actif** au moment du dépôt (un seul nombre, pas un par mode). Le rechargement en cours, s'il y en avait un, est annulé (`WeaponInventory::clear_reloading`) : l'arme qui rechargeait n'existe plus.

**Ramasser** (`INPUT_INTERACTION`, `InteractionType::Weapon`, `interaction::handle_weapon_pickup_interaction`) : équipe l'arme (devient active), restaure son chargeur (`WeaponPickup::mag_ammo`, via `spawn_weapon_for_player`, paramètre `initial_mag_ammo`). En dessous de `weapon_slots`, elle s'ajoute ; à `weapon_slots` ou au-delà, elle **remplace** l'arme active, qui tombe au sol à son tour (même mécanisme, `weapons::spawn_weapon_pickup`). La réserve de munitions n'est jamais transférée : seul le chargeur voyage avec l'arme, la réserve reste attachée au joueur.

**Réutilisation par T2.3** (armes murales) : `weapons::spawn_weapon_pickup(commands, weapon, mag_ammo, position, price, id_factory)` est le point d'entrée commun — `price: Option<u32>` vaut `None` pour un lâcher et `Some(prix)` pour une arme murale (T2.3, §12).

**Scénarios de référence** (`tests/scenarios/`) : `ammo_shared_reserve.ron` (deux armes forcées sur le même type via `weapon_overrides.ammo_type`, la réserve baisse quelle que soit l'arme qui recharge), `drop_pickup_swap.ron` (lâcher, `WeaponPickups`, ramasser). `weapon_overrides` gagne `ammo_type: Option<AmmoType>` (s'applique à l'arme entière, comme `friendly_fire`). Nouvelles attentes : `AmmoReserve { handle, ammo_type, amount, at_frame }` (exact, comme `Ammo`), `WeaponPickups { min, max, at_frame }` (nombre d'entités `WeaponPickup`, tous joueurs confondus). Nouveaux `GameEvents` : `drop`/`pickup` (détectés par `scenario::events::detect_events` sur l'apparition/disparition d'un `GgrsNetId` de `WeaponPickup`).

## 12. Monnaie et achats (T2.3, chantier C5 v1)

**Monnaie** (`run::currency::Currency`, crate minimal `crates/run`) : composant rollback `u32` posé sur chaque joueur à sa création (`character::player::create::create_player`), valeur de départ `CharacterConfig::starting_currency` (défaut 500, `zombies` le déclare explicitement dans `player_config.ron`). API saturante : `can_afford`/`spend` (refuse sans rien changer si le solde est insuffisant)/`earn`. Chaque variation passe par `FrameEvents<run::currency::CurrencyEvent>` (`{ handle, delta, reason }`, `delta > 0` = gain, `< 0` = achat, `== 0` = achat refusé faute de solde) — lu par le HUD (source `currency`), par `scenario::events::detect_events` (`GameEvents` : `points`/`purchase`/`purchase_refused`, classés par le signe de `delta`) et par T2.5 (power-ups).

**Points** (`game::economy`) : détectés au plus près de l'information disponible (kill dans `character::health::rollback_apply_accumulated_damage`, au moment où `Death` est posée sur une entité qui n'est pas un joueur avec `HitBy::Player` en dernier coup ; coup au but dans `rollback_resolve_damage_events`, à chaque `DamageEvent` résolu d'un joueur vers une cible qui n'en est pas un ; réparation de fenêtre dans `interaction::handle_window_repair`), poussés dans `FrameEvents<economy::PointsCredit>` et résolus une seule fois par `economy::award_points_system` (`RollbackSystemSet::Run`, en fin de frame) qui lit `economy.ron` et crédite `Currency`. Réparation plafonnable par joueur et par vague (`EconomyConfig::repair_points_cap_per_wave`, optionnel — `economy::RepairPointsTracking`, remise à zéro au changement de vague).

**Portes payantes** : `handle_door_interaction` débite `DoorConfig::cost` (existant, calculé par le générateur selon la profondeur de la salle — voir `BasicMapGeneration::get_doors`, 750 + 250×profondeur) avant d'ouvrir ; `cost <= 0` reste gratuit (comportement inchangé). Solde insuffisant : refus silencieux, porte fermée, `CurrencyEvent { delta: 0 }`.

**Armes murales** (`WeaponLocation`, §1) : `interaction::handle_weapon_pickup_interaction` distingue une arme murale (`weapons::WeaponPickup::price: Some(...)`) d'une arme au sol ordinaire (`None`, T2.2) — une arme murale n'est jamais consommée au ramassage (le mural reste achetable) et exige `Currency >= price`, ou `ResolvedEconomyConfig::refill_price(price)` (ratio `refill_price_ratio`, défaut 0.5×) si le joueur possède déjà une arme du même id, auquel cas l'interaction recharge la réserve de munitions du type au lieu d'équiper une seconde copie. Anti-rebond obligatoire (`WeaponPickup::can_buy_after_frame`, comme `WindowHealth::can_repair_after_frame`) : sans lui, maintenir Interaction facturerait le joueur à chaque frame.

**Perks** (`SodaLocation`, §1, kind de contenu `Perk`, `games/<jeu>/assets/economy/perks.ron`) : table `{ "id": (name: "...", price: N, modifiers: [(stat: ..., op: ..., value: "...")]) }` (fichier RON, newtype sur la table — parenthèse ouvrante/fermante autour de tout le fichier, comme `weapons.ron`). Achat = interaction (`InteractionType::Perk`) sur une entité `economy::PerkMachine` ; un seul achat par perk et par joueur (`run::perks::Perks`, `BTreeSet<String>`, vérifié **avant** de débiter — déjà possédé n'émet ni achat ni refus). Modificateurs posés permanents (`until: None`, `sim_core::modifier::ModifierSource::Named("perk:<id>")`) via `Modifiers::push_from`. Un perk qui relève `MaxHealth` (ex. Juggernog, `Mul "2.0"`) relève aussi `Health.current` de la même différence au moment de l'achat (`sync_health_from_stats`, `RollbackSystemSet::Status`, ne fait que plafonner `current` à la baisse, jamais à la hausse — voir sa doc) : décision propre à l'achat d'un perk (un statut temporaire qui relèverait `MaxHealth` puis expirerait ne doit pas, lui, soigner le joueur).

**Économie RON** (kind de contenu `Economy`, `games/<jeu>/assets/economy/economy.ron`, un seul fichier) : `kill_points` (défaut 60), `hit_points` (défaut 10), `repair_points` (défaut 10), `nuke_points` (défaut 400, D17 : crédité à chaque joueur vivant au ramassage d'un nuke), `repair_points_cap_per_wave` (`Option<u32>`, défaut aucun plafond), `refill_price_ratio` (`Fixed` en chaîne, défaut `"0.5"`). Chargé par le registre comme `Wave` (id = nom de fichier). Lint (`content::lint::lint_perks`, `lint_economy`) : `price > 0`, au moins un modificateur, `value > 0` pour un `op: Mul`, id unique dans `perks.ron` ; `refill_price_ratio` dans [0, 1] ; un `StatId` inconnu dans `perks.ron` échoue déjà au chargement RON (pas de variante fourre-tout implicite). Voir le tableau des règles, §3.

**Scénarios de référence** (`tests/scenarios/`) : `buy_door`, `buy_wall_weapon`, `buy_perk` (sur `games/zombies/assets/exemples/test_map_shop.ldtk`, copie de `test_map.ldtk` avec une `WeaponLocation`/`SodaLocation` dans la salle de départ — `test_map.ldtk` n'en avait pas avant T2.6, qui en a posé quatre de chaque et re-blessé toutes les traces de cette carte), `points_on_kill` et `shop_tour` (sur `test_map.ldtk`). Nouvelles attentes de scénario : `Expectation::Currency { handle, min, max, at_frame }`, `Expectation::Stat { handle, stat, value, at_frame }` (valeur résolue, comme `stats::StatReader`) ; `PlayerScript::currency: Option<u32>` (solde de départ, comme `weapon`).

## 13. État de run et mode `Waves` (T2.4, chantier F1)

**`Run`** (`run::run::Run`, crate `run`) : ressource rollback (checksum + trace, `RunPlugin`) qui représente la partie en cours — `seed: u32` (graine, dupliquée de `RunSeed` pour que `Run` seule décrive la partie), `mode: RunMode` (`Waves { config }` = le système de vagues actuel, `Sandbox` = aucune condition de fin hors défaite ; `Floors { config }` = séquence de niveaux, T1.8, §17 ; `Campaign` viendra en M2), `step: RunStep` (`Lobby` — valeur par défaut de l'enum, jamais celle d'un `Run` réellement inséré, voir plus bas —, `Playing { since_frame }`, `Ended { at_frame, outcome: RunEnd }`), `players: Vec<usize>` (handles GGRS triés), (`flags: BTreeSet<String>` : **retiré par D76**, rien ne l'écrivait ni ne le lisait ; `impl Hash for Run` écrit toujours le hash d'un ensemble vide à sa place pour que les traces ne bougent pas, test `hash_de_run_inchange_apres_le_retrait_de_flags` ; un futur champ de drapeaux reprend cette place) et `summary: Option<RunSummary>`. Remplace `combat::downed::RunOutcome` (T1.3), qui disparaît : la défaite est maintenant une valeur de `RunStep`, pas une ressource à part. **Décision** : `RunSummary` est un champ de `Run`, pas une ressource rollback séparée — un seul enregistrement, pas de valeur sentinelle à inventer pour « pas encore de résumé » (`RunEnd` n'a pas de variante neutre).

**Création** : `Run::new(seed, mode, players, since_frame)` construit toujours directement `Playing` (jamais `Lobby` dans ce chantier : réservé à un futur état où `Run` existerait déjà pendant un lobby interactif, M1/M2). Appelée par `game::jjrs::{local, p2p}` (`system_after_map_loaded_local`/`system_after_map_loaded`, `OnEnter(AppState::GameStarting)`), au même instant que `RunSeed`/`RngStreams` (T1.6) — avant que `GgrsSchedule` ne tourne pour la première fois (aucune session GGRS encore présente). `players` vient de `GgrsSessionBuilding.players[].handle`. `mode` vient de `game::run_state::resolve_run_mode(manifest, registry)` : `entry.mode` du manifeste (`content::manifest::EntryMode::Waves`/`Sandbox`/`Floors`, enum fermé comme `sim_core::damage::FriendlyFire`) s'il est présent, sinon `Waves` si le jeu déclare un dossier de contenu `Wave`, sinon `Sandbox` — `content::lint` refuse `entry.mode: Waves` sans dossier `Wave` déclaré (`lint_entry_point`), mais pas l'absence d'`entry.mode` avec un dossier `Wave` présent (c'est le chemin par défaut).

**Syntaxe RON du mode optionnel** (T3.4, D16) : `entry: (..., mode: Waves)`, `mode: Sandbox` ou `mode: Floors`. Le chargeur de manifeste active `implicit_some` sans directive en tête du fichier ; `Some(Waves)`/`Some(Sandbox)` et `None` restent acceptés. Les trois jeux déclarent leur mode explicitement (zombies `Waves`, testbed `Sandbox`, throne `Floors`). Sans ce champ, le mode est résolu comme décrit ci-dessus.

**Fin de partie** : deux voies, jamais concurrentes (`Run::is_playing()` gardé par les deux avant d'écrire `step`, une seule issue jamais réécrite).
- **Défaite** (universelle, tout mode confondu, T1.3) : `character::health::rollback_check_defeat` (`RollbackSystemSet::DeathManagement`, inchangé sinon) écrit `Ended { outcome: RunEnd::Defeat, .. }` dès que tous les joueurs en jeu sont à terre ou morts.
- **Victoire** (propre au mode, `run::modes::RunModeRules`, trait implémenté pour `RunMode`) : `game::run_state::check_run_victory_system` (`RollbackSystemSet::Run`, après `award_points_system`) appelle `run.mode.check_victory(&ctx)` avec un `RunContext` construit depuis `WaveState`/`ResolvedBalance::waves.max_wave` (et `FloorState` en `Floors`). Pour `Waves` : victoire si `WaveConfig::max_wave` (`Option<u32>`, `#[serde(default)]`, absent = jamais de victoire, comme avant ce chantier) est déclaré et atteint. `Sandbox` ne gagne jamais.

`game::run_state::finalize_run_summary_system` (`RollbackSystemSet::Run`, après les deux systèmes ci-dessus) calcule `Run.summary` une seule fois, dès que `step` devient `Ended` et que `summary` est encore `None` : `RunModeRules::summarize` construit `RunSummary { wave_reached, kills, points_total, frames, outcome }` (`wave_reached`/`kills` viennent de `WaveState` ; `points_total` = somme des soldes `Currency` de tous les joueurs à cet instant — pas un total cumulé des gains, qui n'existe nulle part ailleurs dans le jeu ; `frames` = `Run.step`'s `at_frame`).

**`waves/systems.rs`** : logique inchangée ; les systèmes de vagues ne lisent pas `Run.step`. Le spawn s’arrête quand il ne reste plus de joueur (`player_positions.is_empty()`), mais un run terminé avec des joueurs encore présents ne suspend pas les vagues. Cette limitation est conservée pour ne pas changer le gameplay ni les traces de ce chantier.

**Écran de fin** (`game::ui::game_over`) : dérivé de `Run.step` (pas de `RunOutcome`), affiche le résumé lisible une fois `Run.summary` disponible (`finalize_run_summary_system` tourne après la détection de fin, dans la même frame de simulation).

**Relance sans relancer le binaire** (`game::run_state::RunRequest`, hors rollback — ni checksum ni trace, comme `NextState`) : `Restart` (même config : même carte, même graine, mêmes joueurs ; en ligne, graine dérivée `restart_seed`, §33) ou `ToLobby` (retour `LobbyLocal`/`LobbyOnline` selon `OnlineState`), posée par le bouton de fin de partie ou la touche `R`. `apply_run_request_system` (`Update`, hors `GgrsSchedule`) consomme la commande et change d'`AppState` ; la destruction proprement dite se déclenche sur `OnExit(AppState::InGame)` (quelle que soit la cause), dans deux systèmes indépendants (aucun appel direct entre crates) :
- `game::run_state::cleanup_rollback_world_system` : détruit (`try_despawn`, silencieux sur une entité déjà partie — un despawn racine est récursif, voir sa doc) toute entité `Rollback` (joueurs, ennemis, balles, armes, murs, portes, machines à perk...), retire `Session<PeerConfig>` et `Run`, remet à zéro `FrameCount`, `GgrsNetIdFactory`, `WaveState`, `FloorState`, `run::Clock`, `FlowFieldCache`, `RepairPointsTracking`, `CollisionGrids`, `RollbackOrdered` (ressources rollback globales qu'aucun autre système ne réinitialise).
- `map_ldtk::game::plugin::despawn_ldtk_world_on_exit_ingame` : détruit l'entité racine LDtk (`LdtkProjectHandle`) et donc tout son arbre (niveaux, calques, tuiles, et toute entité LDtk encore dessous — portes, fenêtres, spawners, `WeaponLocation`/`SodaLocation`...). Vit dans `map_ldtk` (pas `game`) parce que `game` ne dépend pas de `bevy_ecs_ldtk`.

Ressources GGRS internes (`RollbackFrameCount`, `ConfirmedFrameCount`, `MaxPredictionWindow`, `LocalPlayers`) : pas touchées explicitement, `bevy_ggrs::schedule_systems::run_ggrs_schedules` les réinitialise de lui-même dès qu'aucune `Session` n'est présente. **Volontairement pas détruites** : `RunSeed`, `RngStreams`, `MapGenerationConfig` (via `LdtkGameMap`, jamais retirée de l'app), `GggrsSessionConfiguration`/`GgrsSessionBuilding` — c'est justement ce qui permet à `Restart` de rejouer « la même configuration » sans reconstruire un lobby : `jjrs::{local, p2p}` les écrase simplement des mêmes valeurs au prochain `OnEnter(AppState::GameStarting)`. `LdtkMapEntityLoadingRegistry` (état interne du chargement de carte, `map_ldtk`) est remise à zéro sur `OnEnter(AppState::GameLoading)` plutôt que sur `OnExit(InGame)` : sinon `wait_for_all_map_rollback_entity` la voit déjà `loading_complete` (partie précédente) et ne relit jamais les entités de la nouvelle carte.

**Restart en p2p** : livré depuis (D14, §33) pour `--matchbox` ; seul `--allumette` reste redirigé vers `ToLobby` par `apply_run_request_system` (averti par un `warn!`).

**Abandon et lobby local (D13)** : quitter vers le lobby une partie encore en cours la termine en `Ended { outcome: Abandon }` **avec son résumé** (`apply_run_request_system`, mêmes champs qu'une défaite via le calcul partagé `run_summary`). Un retour au lobby **local** pose `LocalLobbyHold { summary }` (hors rollback) : `setup_ggrs_local` attend qu'elle disparaisse (condition dans `core.rs`), l'écran du lobby (`ui/lobby.rs`) affiche le résumé de la partie quittée et la retire sur Entrée ou « Nouvelle partie ». Au premier lancement, pas de `LocalLobbyHold` : le lobby local démarre aussitôt, comme avant (scénarios, `make zombies`). Décision pure testée : `run_state::plan_run_request`.

**Scénarios et tests** : `run_lose_summary` (deux joueurs immobiles sur `test_map.ldtk`, même mise en scène que `downed_all_lose` — défaite complète à f1505) vérifie `RunState(step: Ended(Defeat), at_frame: ...)` et `RunSummary(wave_reached_min: 1, kills_min: 0, at_frame: ...)`. Nouvelles attentes de scénario : `Expectation::RunState { step: RunStepExpectation::{Playing, Ended(RunEnd)}, at_frame }` (vérification ponctuelle, comme `PlayerAlive` — `RunStep::Ended` ne change plus une fois posé) et `Expectation::RunSummary { wave_reached_min, kills_min, at_frame }` (bornes inférieures, échoue si `Run.summary` est encore `None`). `crates/scenario/tests/run.rs` : test d'intégration qui pilote `App::update()` directement (pas via `Scenario::expect`) — joue `idle` jusqu'à la défaite, pose `RunRequest::Restart`, mesure le temps mural jusqu'au retour en `InGame` à la frame 0 (doit rester sous dix secondes), rejoue 300 frames et compare la trace d'état (`StateTraceRecorder::lines_until`) à celle des 300 premières frames de la première partie — identiques si la relance n'a rien laissé traîner. `ToLobby` : vérifié au moins pour le changement d'état (`AppState` devient `LobbyLocal`).

## 14. Power-ups (T2.5, chantier C1 v0)

**Vocabulaire d'actions** (`crates/effects`, données pures ; `bevy` sert uniquement au
plugin de déclaration des kinds depuis T1.0a) : `effects::Action` est la liste fermée des effets qu'un power-up
peut appliquer, la graine du futur système de déclencheurs/effets de `docs/plan-engine.md`
§5 (« C1. Effets ») — pas de déclencheur ni de condition dans T2.5, seulement les
actions ; les contrats de M1 sont décrits au §4.3. Cinq variantes : `TimedModifier { stat, op, value, frames }` (modificateur de stat
temporaire, `frames` = durée relative à la frame de ramassage), `RefillAmmo`,
`RepairAllWindows`, `KillAllWaveEnemies`, `CurrencyMultiplier { factor, frames }`.
`RefillAmmoOf(munition)` (D40) : `RefillAmmo` restreint à une munition (armes et réserve de cette
munition ; un rechargement en cours n'est annulé que si l'arme active est de cette munition) ;
lint : munition déclarée par au moins une arme ; refusée dans un effet, comme `RefillAmmo`.
`Action::as_modifier(frame, source)` est l'« applicateur déterministe » pur (sans ECS,
testé unitairement) pour les deux variantes modifier-based ; les trois autres sont résolues
directement par `game::powerups` (accès à `AmmoReserves`/`WindowHealth`/`Team`, inconnus
d'`effects`).

Depuis T1.10, le même `Action` sert aux **effets** (`Effect { on, if, do }` exécutés, nouvelles
actions `Modifier`, `Heal`, `SpawnPattern`, `GaugeAdd`) : voir §27.

**Sémantique CoD (décision)** : un power-up ramassé s'applique à **tous les joueurs** de la
partie, jamais au seul joueur qui l'a ramassé — aucun des cinq power-ups de référence
(Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke) n'est individuel dans le jeu source.

**Table de contenu** (kind `PowerUp`, `games/<jeu>/assets/items/powerups.ron`, déclaré comme
un **dossier** dans `game.ron` — comme `characters/`, pas un fichier unique — pour qu'un
futur id dupliqué entre deux fichiers du dossier produise l'erreur `DuplicateId` habituelle) :
```ron
(
    drop_chance: "0.15",  // probabilité qu'un ennemi tué (équipe Enemies) laisse tomber un power-up
    powerups: {
        "insta_kill": (
            name: "Insta-Kill",
            weight: 15,            // poids de tirage parmi les power-ups, si un drop a lieu
            pickup_range: "30.0",  // portée de ramassage (Fixed), au passage, pas un bouton
            lifetime_frames: 1800, // durée de vie au sol avant disparition (≈ 30 s à 60 FPS)
            actions: [
                TimedModifier(stat: Damage, op: Set, value: "100.0", frames: 1800),
            ],
        ),
        // double_points: CurrencyMultiplier ; max_ammo: RefillAmmo ;
        // carpenter: RepairAllWindows ; nuke: KillAllWaveEnemies.
    },
)
```
`drop_chance` décide **si** un drop a lieu (tirage global), `weight` décide **lequel** parmi
la table (tirage pondéré, même méthode que `waves/wave_config.ron`/`select_enemy_type`).
Lint (`content::lint::lint_powerups`) : `drop_chance` dans `[0, 1]`, `weight > 0`,
`lifetime_frames > 0`, `pickup_range > 0`, au moins une action, `frames > 0` pour
`TimedModifier`/`CurrencyMultiplier` et `factor > 0` pour `CurrencyMultiplier` par
power-up (`LintErrorKind::OutOfRange`, T2.8 pour les cinq dernières), et `value > 0` pour un
`TimedModifier` avec `op: Mul` (T3.4) ; une référence de `StatId` inconnue dans
`actions[].stat` échoue déjà au chargement RON (`actions` réutilise le type réel
`effects::Action`, pas un miroir — comme `StatId` pour `perks.ron`), rapportée
`LintErrorKind::Parse` ; un id de power-up dupliqué entre deux fichiers du dossier `PowerUp`,
`LintErrorKind::DuplicateId` (même mécanisme que `characters/`/`weapons.ron`). Fixtures :
voir le tableau des règles de lint, §3.

**Entité rollback « power-up au sol »** (`game::powerups::PowerUpPickup { id, expires_at_frame }`,
posée par `spawn_powerup_pickup`, point d'entrée commun au drop et au placement scripté de
scénario — comme `weapons::spawn_weapon_pickup` pour lâcher/armes murales) : **pas** une
`Interactable` (contrairement aux armes/fenêtres/perks) — `powerup_pickup_detect_system`
(`RollbackSystemSet::Effects`) ramasse automatiquement au passage, dès qu'un joueur entre
dans `PowerUpDef::pickup_range` (le premier dans l'ordre `GgrsNetId` s'il y en a plusieurs à
portée la même frame), despawn immédiat (`despawn_rollback`) et émission de
`FrameEvents<PowerUpPickedUp>`, résolu par `apply_powerup_actions_system` (même set,
`.after`). `powerup_expiry_system` (même set) détruit un power-up jamais ramassé à
`expires_at_frame` : à cette frame le ramassage est fermé. `PowerUpPickup` est
enregistré par `rollback_and_trace` (rollback, checksum GGRS et trace). La file
`FrameEvents<PowerUpPickedUp>` est elle aussi checksummée. Tout bless exige la preuve
décrite au §10 ; les résultats figurent dans le rapport de tâche.

**Max Ammo** : remplit tous les modes de chaque arme portée, additionne les capacités
`mag_size × mag_limit` du mode par défaut **par type de munition**, puis relève chaque
réserve à cette somme sans diminuer une réserve déjà supérieure. Un rechargement en
cours est annulé pour éviter un débit après le remplissage. Les armes `Magless` ne
contribuent pas à la réserve, mais leur stock propre est rempli.

**Nuke : points** (D17) : au ramassage, `KillAllWaveEnemies` pousse un
`economy::PointsCredit::Nuke` par joueur vivant (ordre `GgrsNetId`), pas par ennemi tué ;
`award_points_system` le résout en fin de frame avec `EconomyConfig::nuke_points` (défaut 400,
comme CoD) et le multiplicateur de points du joueur (800 sous Double Points). Les ennemis tués
par le nuke ne rapportent pas de points de kill (`Death { last_hit_by: None }`).

**Rafraîchissement** (D18) : un power-up déjà actif ramassé à nouveau ne se cumule pas
(Double Points × Double Points ne fait pas × 4). Au ramassage, `apply_powerup_actions_system`
retire d'abord, sur chaque joueur, les modificateurs de source `powerup:<id>`
(`Modifiers::remove_by_source`), une fois pour toutes les actions du power-up, puis les
repose : la durée recommence à la frame du ramassage, comme dans CoD.

**Drop à la mort** (`game::powerups::loot_drop_on_death_system`,
`RollbackSystemSet::DeathManagement`, `.after(rollback_apply_accumulated_damage)
.before(rollback_apply_death)` — même contrainte que
`waves::systems::wave_enemy_death_tracking_system`, il lui faut la position de l'entité
avant qu'elle ne soit détruite) : portée **équipe `Enemies`** (pas seulement `WaveEnemy`,
absent des personnages de laboratoire du testbed placés par `CharacterSpawn` — sans quoi le
chemin « drop à la mort » ne serait pas prouvable dans le testbed, T2.9 n'y faisant jamais
tourner le mode vagues). Tirage déterministe dans le flux RNG nommé **`loot`**
(`bevy_fixed::rng::RngStreams`, T1.6, comme `"waves"`/`"weapons"`), **dans l'ordre des
`GgrsNetId`** des morts de la frame (`order_iter!`) : un tirage « un power-up tombe-t-il »
(comparé à `drop_chance`), puis, si oui, un tirage pondéré (`weight`) pour lequel.

**Réglages de scénario** (`game::replay::Scenario`, appliqués par `crates/scenario/src/runner.rs`,
même famille que `wave_overrides`/`weapon_overrides`) :
- `powerups: [(id, x, y, at_frame)]` (`PowerUpPlacement`) : fait apparaître un power-up à une
  position (`x` et `y` en chaînes Fixed) et une frame exactes, sans dépendre d'une carte
  LDtk ni du tirage RNG — pour
  prouver l'**effet** de chaque power-up indépendamment du mécanisme de drop. Contrairement
  aux autres réglages (`Update`, une fois avant la première frame), celui-ci tourne dans
  `GgrsSchedule` (`RollbackSystemSet::Effects`, avant la détection de ramassage) : la
  condition `frame == at_frame` est naturellement rollback-safe (même raisonnement qu'un
  spawn de vague), pas de `Local<bool>` par placement. La durée de vie est celle de la
  table du jeu, comme pour un drop réel.
- `powerup_drop_chance_override: Fixed` : force `PowerUpsConfig::drop_chance` pour ce
  scénario (`Update`, comme `apply_wave_overrides`) — pour prouver le chemin « drop à la
  mort » sans dépendre du tirage réel du jeu (scénario `powerup_drop_on_kill`, chance forcée
  à 1). Les scénarios de régression et les essais générés d'armes déclarent explicitement
  une chance nulle pour isoler leur comportement. Dans ce cas aucun tirage ni flux
  `loot` n'est créé. La table du jeu conserve sa chance de 15 %.

**Nouvelle attente** (`game::replay::Expectation::PowerUpPickups { min, max, at_frame }`) :
nombre de `PowerUpPickup` au sol, bornes `[min, max]` inclusives — même forme que
`WeaponPickups`. Les placements et la chance de drop sont préservés dans le scénario
réenregistré par le runner.

**Scénarios de référence** (`tests/scenarios/`, testbed, un par power-up + un pour le drop) :
`powerup_insta_kill`, `powerup_double_points`, `powerup_max_ammo`, `powerup_carpenter`
(seul à utiliser `testbed/window.ldtk` plutôt que `testbed/arena.ldtk` : il faut une fenêtre
à réparer, absente de l'arène — le `breacher` de `window.ldtk` la casse naturellement avant
que le scénario ne pose le power-up), `powerup_nuke`, `powerup_drop_on_kill` (chance forcée
à 1, tue `dummy` par tir soutenu, vérifie qu'un `PowerUpPickup` apparaît).

## 15. HUD : icônes, noms et sources (T2.12)

Le HUD est décrit par `games/<jeu>/assets/ui/hud.ron` (`crates/game/src/ui/hud.rs`, liste des
sources et des ancrages dans `CLAUDE.md` § HUD). Il ne fait que **lire** l'état à chaque
`Update` : aucun système dans `GgrsSchedule`, aucun état rollback, aucune trace touchée.
T1.18 : les sources du joueur passent par `game::ui::hud_model` (`HudSnapshot`) et la liste
des sources est fermée et lintée (voir §32).

Champs optionnels à la racine de `hud.ron`, à côté de `widgets` : `font`, `icons`, `names`,
`reference_size: Some((800.0, 540.0))`. Ce dernier ajuste `UiScale` aux petits viewports
sans agrandir les grands, et masque ce HUD pendant mutation/fin de run. Sans ce champ,
les anciens jeux conservent leur mise en page. Par widget : `background`,
`compact: true` (défaut false) : arme lisible sans underscores, munitions `chargeur / réserve`,
effets temporaires limités à deux lignes puis `+N effets`. Le nom de l'arme indique
`· recharge` uniquement pendant le rechargement. Le modèle complet reste accessible
aux scénarios/outils ; les barres ont un fond fixe sous le remplissage.

Les overlays techniques (frame/FPS, caméra, arme, inspecteur natif) sont optionnels
via `ALACOD_DEBUG_UI=1`. La feature native ne les active plus à elle seule. Le debug de
vagues de zombies suit ce défaut ; F3 conserve sa bascule explicite.
```ron
(
    // Police de tous les textes du HUD, relative à `assets/`. La police par défaut de Bevy
    // n'a ni accents (« À TERRE », « possédé ») ni tiret long : les trois jeux déclarent celle-ci.
    font: Some("fonts/FiraMono-Medium.ttf"),
    widgets: [
        (kind: Icons(source: "perks"), anchor: BottomLeft, offset: (16.0, 80.0),
         size: Some((200.0, 24.0)), color: None, font_size: None),
        (kind: Text(source: "prompt", prefix: Some("[H] ")), anchor: BottomCenter,
         offset: (0.0, 110.0), size: Some((480.0, 24.0)), color: Some("#f1c40f"), font_size: Some(18.0),
         // Fond derrière le texte (`#rrggbb` ou `#rrggbbaa`), masqué quand le texte est vide ;
         // avec un fond, la hauteur suit le nombre de lignes.
         background: Some("#000000a0")),
        // ...
    ],
    // Icône par id de perk (`economy/perks.ron`) : pas de sprite, un carré de couleur
    // (hex `#rrggbb`) et une étiquette courte (l'initiale).
    icons: {
        "juggernog": (color: "#c0392b", label: "J"),
    },
    // Nom affiché par id d'arme (`weapons.ron`), utilisé par la source `prompt`.
    names: {
        "shotgun": "fusil à pompe",
    },
)
```
- **Icône manquante** : un perk absent de `icons` est affiché en carré gris avec la première
  lettre de son id en majuscule (pas d'erreur).
- **Nom manquant** : une arme absente de `names` est affichée par son id.
- `Icons(source)` : `size.1` est le côté d'un carré ; la rangée s'étend vers la droite depuis
  l'ancre, quel que soit le nombre de perks. `Text(source: "perks")` affiche les étiquettes
  séparées par des espaces.
- M2-T18 : `Icons(source: "passives")` lit les objets passifs (ordre de ramassage,
  doublons conservés), avec les mêmes entrées `icons` et le même repli gris/initiale.
- `Map(source: "minimap")` dessine les rectangles des salles dans `size` : même échelle
  sur les deux axes, cadrage stable de l'étage, y monde inversé vers l'UI. Salle courante :
  bordure or ; verrouillée : rouge et `×` ; nettoyée : verte ; boss : violet et `B`.
  Les salles dormantes restent des silhouettes. Voir §56 pour la limite d'exploration.
- Les noms des perks et des power-ups viennent de leur propre contenu (`name` de `perks.ron`
  et de `items/powerups.ron`), pas de `hud.ron`.
- Prix « en icône » : préfixe `$` devant le montant (`Ouvrir — $750`).
- Valeur absente (D22) : les sources du joueur (`health`, `ammo`, `weapon`, `currency`)
  n'affichent rien quand il n'existe plus (mort) ou n'a pas d'arme, pas même leur préfixe :
  plus de « $ » ni de « ? | ? » sur le HUD d'un joueur mort.
- Placement : les textes de debug (`Wave 1 | PREP …` en bas à gauche, caméra en haut à
  gauche) occupent le bas de l'écran jusqu'à ~75 px : les perks (80) et le prompt (110) sont
  placés au-dessus. L'écran de fin utilise la même police (`ui/game_over.rs`).

Captures de référence : `docs/captures/hud-v1/` (prompt d'achat, perk et power-up actifs, à
terre), produites par `play_scenario --capture` (voir `CLAUDE.md` § Vidéos).

## 16. Projectiles composables (T1.1, chantier B5 v1)

Code : `crates/combat/src/projectile.rs` (données, mathématiques pures testées, systèmes),
branché par `combat::weapons::BaseWeaponGamePlugin`. Les contrats de T1.0a
(`ProjectileModifier`, `Pattern`, §4.3) sont exécutés ; aucun renommage, un ajout :
`ExpireAction::Spawn(Pattern)` (puis `ExpireAction::DestroyTerrain { radius }`, T1.6, §21).

**Contenu** : un mode de tir déclare `projectile:` (absent = balle ordinaire, exactement
comme avant, sans composant `Projectile`) ; l'arme déclare la table `projectiles` des
projectiles que ses patterns font naître (une table **par arme** en v1, résolue au tir et
portée par la balle ; T1.2 l'a gardée par arme : un émetteur tire depuis la table de son arme, §20).
```ron
"default": (
    // ... champs habituels du mode (bullet_type, range... : le projectile tiré)
    projectile: (
        modifiers: [Bounce(2), Pierce(8), Lifetime(70)],
        on_hit: [TimedModifier(stat: MoveSpeed, op: Mul, value: "0.5", frames: 60)],
        on_expire: [Spawn(Aimed(count: 1, spread: "0.0", projectile: "explosion"))],
    ),
),
// au niveau de `config`, à côté de `firing_modes` :
projectiles: {
    "explosion": (damage: "20.0", speed: "0.0", range: "1.0",
                  modifiers: [Lifetime(0), Size("10.0"), Pierce(16)],
                  on_expire: [Spawn(Ring(count: 8, speed: "240.0", projectile: "eclat", every: 0))]),
    "eclat": (damage: "4.0", speed: "240.0", range: "96.0"),
},
```

**Modificateurs** (décisions de gameplay ; un même modificateur répété : le dernier
l'emporte, le lint le refuse) :

| Modificateur | Effet |
|---|---|
| `Bounce(n)` | n rebonds sur les murs (`Wall` par `layer_matrix`) ; l'axe du déplacement qui entre dans le mur s'inverse (les deux dans un coin) et le projectile revient à sa position d'avant le déplacement. `n = 0` : fin au premier mur. |
| `Pierce(n)` | traverse n personnages, se termine sur le (n+1)-ième ; **jamais deux fois le même** (liste `hits`). Tous les personnages en contact dans une frame sont touchés, par `GgrsNetId`, tant que `Pierce` le permet. |
| `Size(f)` | rayon de collision × f (base 5, celui de `Standard`) et sprite × f. `f > 0`. |
| `Lifetime(n)` | n frames de vie après la frame de tir, fin à la suivante : `Lifetime(0)` = une seule frame de collisions. La portée (`range`) termine aussi le projectile. |
| `Homing(force)` | à chaque frame, la direction tourne vers le personnage le plus proche que l'équipe et le tir ami permettent de toucher (hors `Neutral`, hors cibles déjà traversées, égalité par `GgrsNetId`) : `v̂ + force·t̂` renormalisé, vitesse conservée. `0 < force <= 1`. Pas de portée de détection en v1. |
| `Gravity(g)` | accélération verticale constante de g unités/s² (positive vers le haut du monde) : `v.y += g/3600` par frame. |

**Ordre dans une frame** : `Weapon` (tir ; déplacement de toutes les balles, la portée
atteinte *termine* un projectile composable au lieu de le détruire ; collisions des balles
ordinaires, inchangées) → `Projectiles` (`projectile_collision_system` : personnages puis
murs ; `apply_projectile_on_hit_system` ; `status_tick_system` (T1.3, §19) ;
`projectile_wall_terrain_system` (T1.6, §21) ; `projectile_expire_system` : `Lifetime`,
`on_expire`, `despawn_rollback` ; `projectile_steering_system` : `Gravity` puis `Homing`,
appliqués au déplacement suivant) → `CollisionDamage` (les `DamageEvent` des projectiles).
`apply_projectile_on_hit_system` est dans `Projectiles` et non `Effects` : `Effects` contient
déjà `game::powerups::apply_powerup_actions_system`, qui écrit aussi `Modifiers` (ordre
ambigu refusé par `GgrsSchedule`).

**`on_hit: [Action]`** : actions de `effects::Action` posées sur **le personnage touché**
(s'il a des `Modifiers`), seulement les actions à modificateur (`TimedModifier`,
`CurrencyMultiplier`), plus `ApplyStatus` (T1.3, §19) et `DestroyTerrain` (T1.6, §21, qui ne
creuse que sur un mur) ; les autres (`RefillAmmo`, `RepairAllWindows`, `KillAllWaveEnemies`)
sont des effets globaux de power-up, refusés par le lint. Source du modificateur :
`projectile:<id>:<rang>` ; un nouveau coup du même projectile **rafraîchit** le
modificateur au lieu de l'empiler.

**`on_expire: [Spawn(pattern)]`** : joué à **toute** fin du projectile (durée de vie,
portée, mur sans rebond, perforation épuisée) au point de fin. Patterns instantanés
seulement, sans aléa : `Aimed` (éventail de `count` sur `spread` radians, centré sur le
personnage le plus proche, sinon sur la direction du projectile), `Spread` (même éventail,
centré sur la direction du projectile), `Ring` (`count` directions régulières à partir de
la direction du projectile — axe +x à l'arrêt —, vitesse du pattern, `every` ignoré : une
seule salve), `Sequence` de ceux-ci, `Named` (T1.2) ; `on_expire` accepte aussi `DestroyTerrain { radius }`
(T1.6, §21). `Telegraph`/`Wait` et `Scatter` (aléatoire) sont réservés aux émetteurs (T1.2)
et refusés par le lint. Les projectiles nés héritent du tireur (source, équipe, tags, tir
ami, multiplicateur de dégâts `Damage` du tir) ; garde-fou de 8 générations (le lint refuse
les cycles). Une **explosion** est un projectile de la table à vitesse nulle,
`Lifetime(0)`, grand `Size` et `Pierce` élevé.

**État rollback** : composant `combat::projectile::Projectile` (compteurs restants, cibles
déjà touchées, `ended`, génération, actions, table partagée par `Arc` ; Debug compact : la
table n'apparaît que par ses clés) et ressource `FrameEvents<ProjectileHit>`, enregistrés
par `RollbackTraceApp`. `FiringModeConfig`/`WeaponConfig` ont un `Hash` et un `Debug`
manuels qui ignorent `projectile`/`projectiles` vides : le contenu existant garde son hash
et sa ligne de trace détaillée (preuve `trace-diff --ignore
Projectile,FrameEvents<combat::projectile::ProjectileHit>`, §10).

**Attentes** (`crates/scenario`) : `BulletCount(count, projectile?, team?, at_frame)` —
nombre **exact** de balles vivantes, filtrées par id de projectile composable (l'arme qui a
tiré, ou l'entrée de la table pour un projectile né) et par équipe du tireur ;
`HitsAtLeast(entity, hits, at_frame)` — `HitCount` de `entity` ≥ `hits`, `entity` :
`NetId(n)` ou `Target` (l'entité qui compte ses coups de plus petit `GgrsNetId`, `target` du
testbed : utilisable dans le `test.expect` d'une arme, où le `GgrsNetId` dépend de l'arme).

**Lint** (`content::lint::lint_weapon_projectiles`) : modificateur répété, `Size <= 0`,
`Homing` hors `]0, 1]`, action `on_hit` sans modificateur, pattern temporel, `count = 0`,
`spread`/`speed` négatifs, projectile absent de la table (`BrokenReference`), définition
avec `damage`/`speed` négatif ou `range <= 0`, cycle de `on_expire`. Fixtures
`projectile_broken_reference`, `projectile_temporal_pattern`, `projectile_cycle`.

**Testbed** : une arme par modificateur (`proj_bounce`, `proj_pierce`, `proj_size`,
`proj_lifetime`, `proj_homing`, `proj_gravity`) et `grenade` (Bounce, Pierce, explosion,
éclats) dans `games/testbed/assets/ZombieShooter/Sprites/Character/weapons.ron`, chacune
avec `test:` et son scénario généré (`make gen GAME=testbed`,
`tests/scenarios/generated/testbed/`).


**Dispersion du tir** (D51, m1-d51-dispersion-ignoree) : `spread` d'un mode de tir
(`FiringModeConfig`) est en **radians, pleine largeur** : chaque balle d'un tir simple (tout mode
sauf `Shotgun`) part à `(r − ½) × spread` de la direction visée, `r` tiré dans le flux RNG
`weapons` (un tirage par balle), donc dans `[−spread/2, spread/2]` ; `spread` = 0 : tir exact
(`combat::weapons::single_shot_direction`). `Shotgun` garde son `spread_angle` (même formule par
plomb). Lint : `0 <= spread <= π` (fixture `weapon_spread_out_of_range`). Avant D51, l'angle
valait `(r − ½) × 1` (±0,5 rad) pour **toute** arme simple : `spread` n'était jamais appliqué.

Patterns joués **dans le temps** (`Telegraph`, `Wait`, `Ring.every`, `Scatter`, `Named`),
émetteurs et tir ennemi : voir §20 (T1.2).

## 17. Mode `Floors` (T1.8, chantier F1)

**Contenu.** Kind de dossier `Floors` (`content::registry::FloorsEntry`) : un fichier RON par
séquence de niveaux, id = nom de fichier sans extension.
```ron
// games/testbed/assets/floors/deux_niveaux.ron
(levels: ["testbed/floor_a.ldtk", "testbed/floor_b.ldtk"])
```
Les chemins sont relatifs à `assets/` (comme `entry.start_map`) ; chaque niveau est une carte
LDtk ordinaire (générée par gabarits comme les autres, même graine pour tous les niveaux).
Mode : `entry: (..., mode: Floors)` joue la première séquence (ordre des ids) ; un scénario
impose une séquence par son champ `floors: Some("deux_niveaux")` (`map` est alors ignorée), et
`alacod-sim` par `--floors <id>` (`game::run_state::FloorsOverride`, hors rollback). `RunMode::
Floors { config }` porte l'id de la séquence.

**Lint** (`content::lint::lint_floors`, `lint_entry_point`) :

| Règle | Kind | Fixture |
|---|---|---|
| `levels` non vide | `OutOfRange` | `floors_empty` |
| chaque niveau désigne une carte chargée (kind `Map`) | `BrokenReference` | `floors_unknown_map` |
| un niveau `cave:<id>` désigne une caverne chargée (kind `Cave`, T1.6, §21) | `BrokenReference` | `floors_unknown_cave` |
| `entry.mode: Floors` exige un dossier `Floors` | `BrokenReference` | `entry_mode_floors_without_floors` |

**Règles du mode.**
- **Portail** : il s'ouvre quand le niveau courant n'a plus aucune entité `Enemy`
  (`EntityCount(enemy) == 0` : alliés et civils du testbed compris — ne pas en placer dans un
  niveau `Floors`). Position : barycentre des `PlayerSpawn` du niveau (centre de la salle de
  départ), sans entité LDtk dédiée. Un joueur debout (ni à terre ni mort) à moins de 24 unités
  (`run::floors::PORTAL_RADIUS`) le franchit.
- **Passage** : à la frame du franchissement, toute entité rollback qui n'appartient pas à un
  joueur (ennemis, murs, portes, fenêtres, spawners, balles, objets au sol, power-ups...) est
  détruite (`despawn_rollback`), le flow field repart de zéro, les entités du niveau suivant sont
  créées et les joueurs placés sur ses `PlayerSpawn` (même handle ; repli sur le plus petit
  index). Les joueurs gardent santé, armes (enfants de l'entité joueur), munitions, monnaie,
  perks, modificateurs ; un joueur à terre le reste.
- **Numérotation** : `GgrsNetIdFactory` n'est jamais remise à zéro pendant la partie ; le
  nouveau niveau est numéroté dans le même ordre qu'au chargement (entités de carte du
  registre triées par nom puis position, murs par iid de niveau, personnages, armes murales et
  machines à perk triés par position) : les ids continuent ceux du niveau quitté.
- **Fin de séquence : boucle infinie au dernier niveau** (décision T1.8). Après le dernier
  niveau de la liste, chaque portail recharge le dernier niveau (`run::floors::level_for_floor`)
  et l'index continue de croître. Pas de victoire en `Floors` : la partie ne se termine que par
  la défaite (universelle, mêmes chemins que `Waves`) ou l'abandon.
- **Résumé** (`RunSummary`, mêmes chemins que `Waves`) : `floor_reached` = index atteint,
  `kills` = ennemis placés par les niveaux chargés moins ceux encore en vie, `wave_reached` = 0.
  L'écran de fin affiche « niveau N » (N = index + 1).

**État rollback.** `run::FloorState` (ressource : `index`, `anchor` — position du portail —,
`portal_open`, `enemies_placed`), enregistrée par `rollback_and_trace_resource_neutral` : sa
valeur par défaut (tout autre mode) contribue `0` au checksum GGRS, les traces des autres modes
ne changent pas ; `RunSummary::floor_reached` n'entre dans son hash que s'il est non nul (même
raison). Attente de scénario `FloorIndex(index, at_frame)` (ponctuelle) et `RunSummary(...,
floor_reached_min)`.

**Chargement : un monde LDtk par niveau.** À l'entrée de `GameLoading`,
`map_ldtk::game::floors::compute_floor_plan` pose `FloorPlan` (hors rollback) si la partie est
en `Floors` ; `loader::setup_generated_map` charge alors toutes les cartes distinctes de la
séquence, un monde chacun (`FloorWorld(emplacement)`, une carte répétée réutilise son monde),
superposés à l'origine ; seul le monde du niveau courant est visible (présentation). Le
chargement attend que tous les mondes aient leurs niveaux, puis ne crée les entités rollback
que du premier niveau. Le passage de niveau (`floor_transition_system`, `GgrsSchedule`,
`RollbackSystemSet::Run`) lit ces mondes déjà chargés, immuables et identiques sur tous les
clients : aucune attente de chargement pendant la partie, et un rollback qui remonte avant le
passage ressuscite l'ancien niveau. Hors `Floors`, la carte unique est le monde `FloorWorld(0)`
et le chargement est inchangé.

**Bots.** `BotView::portal` : un bot `fonceur` ou `prudent` sans ennemi visible marche vers le
portail ouvert (ligne droite, sans pathfinding : les niveaux du testbed sont des salles
ouvertes).

**Limites connues.** Les `RoomBounds` des niveaux non courants existent aussi (mondes
superposés) : les spawners de vagues (`ZombieSpawn`) ne sont pas pris en charge en `Floors`.
Restart p2p : comme les autres modes (§33). `alacod-sim --until-floor <n>` (T1.14, §24 v1)
arrête une graine au n-ième passage de portail.

## 18. Équilibrage par joueurs (F5)

**Décision (D25, le 2026-10-03) : F5 appartient à M0** — le plan §6 liste F5 dans le jalon M0
(`zombies` complet). Tâche dédiée : `docs/taches/m0-v11-f5-equilibrage-joueurs.md`.

Les champs numériques des **vagues** (`games/zombies/assets/waves/wave_config.ron`), des **prix**
(perks `assets/economy/perks.ron`, `assets/economy/economy.ron` ; les prix d'armes murales sont des champs LDtk, voir plus bas) et de la
**santé** des ennemis (`ZombieShooter/Sprites/Zombie/*_config.ron`) acceptent un littéral **ou** une
expression `content::expr` en chaîne RON (ex. `health: "120.0 + (players - 1) * 30"`). Règles :

- **Contexte** : l'identifiant `players` (nombre de joueurs de la run) est disponible partout où
  une expression est acceptée. C'est le seul identifiant, hors `difficulty.ron` (T1.9 : `floor`, `seconds`, `minutes`, `floor_seconds`, `floor_minutes`, §23).
- **Timing** : l'expression est évaluée **une seule fois, au démarrage de la run** (le nombre de
  joueurs de la session est connu : local `PlayersCount` (`--number-player`, variable `NUMBER_PLAYER` du Makefile), p2p `max_player`, scénario = ses
  joueurs), en valeurs concrètes. **Jamais d'`Expr` dans l'état rollback** : seules les valeurs
  évaluées entrent dans la simulation (déterminisme : mêmes configs + même nombre de joueurs
  partout ⇒ mêmes valeurs). Exception encadrée (T1.9, §23) : l'expression de difficulté est
  réévaluée **en simulation** chaque seconde, depuis l'asset immuable (`game::clock::
  DifficultyConfig`, hors rollback) ; seul son résultat `Fixed` entre dans l'état rollback
  (`run::Clock::difficulty`).
- **Erreurs** : une expression invalide (parse, division par zéro, identifiant inconnu) fait
  échouer le chargement — jamais de valeur par défaut silencieuse.
- **Traces** : les littéraux actuels restent valides et gardent leurs valeurs ; les 62 scénarios
  existants gardent leurs traces, sauf bless avec preuve (§10). La preuve de F5 passe par de
  nouveaux scénarios (même partie à 2 et 4 joueurs, attente qui diverge) plutôt que par la
  modification de valeurs existantes.

**Implémentation (m0-v11)** :

- **Type** : `content::expr::NumOrExpr` — `Integer(u32)` (entier RON nu, ex. `base_enemies: 6`),
  `Literal(Fixed)` (chaîne numérique, ex. `refill_price_ratio: "0.5"`), `Expression(NumExpr)`
  (chaîne RON évaluée, ex. `max: "10.0 + (players - 1) * 40.0"`). La désérialisation tente dans
  cet ordre : entier nu → `Integer` ; chaîne qui est un nombre → `Literal` ; sinon parse
  `Expr` → `Expression` (un échec de parse est un échec du chargement).
- **Point d'évaluation** : `game::balance_assets::resolve_balance_system` (application, E3c ; le calcul pur est
  `game::balance::resolve_balance`, simulation), enregistré sur
  `OnEnter(AppState::GameLoading)` — après le chargement des assets, avant le spawn des
  joueurs/personnages (Update de `GameLoading`) et avant la première vague. Il insère la
  ressource ordinaire (hors rollback, comme `Assets`) `game::balance::ResolvedBalance` :
  `waves`, `waves_enabled` (le jeu déclare un dossier `Wave`), `economy` (dont `shells_drop` et `shop_prices`),
  `perks` et `health_max_by_character`, en valeurs concrètes. Les systèmes
  de simulation lisent `ResolvedBalance`, jamais les assets d'origine — aucune expression dans
  l'état rollback. Une relance locale re-entre `GameLoading` et re-résout avec le même nombre
  de joueurs.
- **Nombre de joueurs** : `game::session::SessionPlayers(u32)`, ressource de simulation que l'application écrit
  juste avant (`balance_assets::publish_session_players`, `OnEnter(GameLoading)`, ordonné `.before` les deux résolutions) :
  `OnlineState::Online` → `ggrs_config.connection.max_player` (source
  autoritaire en ligne) ; sinon `PlayersCount` (partie locale ou scénario). `resolve_balance` et
  `clock::resolve_clocks_system` (difficulté) la lisent ; la simulation ne lit plus `OnlineState` ni
  `GggrsSessionConfiguration` pour cela.
- **Champs couverts** (rien d'autre — le reste des chantiers F est pour M1/M2) :
  - **Vagues** (`games/zombies/assets/waves/wave_config.ron`) : `base_enemies`,
    `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`,
    `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`,
    `min_player_distance`, `max_player_distance`, `health_multiplier_per_wave`,
    `damage_multiplier_per_wave`, `max_wave`, et par palier `max_wave` + poids des ennemis.
  - **Prix** : `games/zombies/assets/economy/economy.ron` (`kill_points`, `hit_points`,
    `repair_points`, `nuke_points`, `repair_points_cap_per_wave`, `refill_price_ratio`) et
    `games/zombies/assets/economy/perks.ron` (`price` de chaque perk). Les prix d'armes
    murales et de portes sont des champs **Int posés dans l'éditeur LDtk** (`WeaponLocation`,
    portes), pas du RON — ils n'acceptent pas d'expression (et `weapons.ron`/
    `melee_weapons.ron` ne portent aucun prix aujourd'hui) ; la recharge d'une arme murale
    reste `prix_achat × refill_price_ratio` (ratio résolu).
  - **Santé** : `base_health.max` de chaque personnage du registre (`CharacterConfig`) —
    ennemis comme « player », les deux chemins de spawn lisent
    `ResolvedBalance::health_max_by_character`.
- **Erreurs** : `content::expr::EvalError` — identifiant inconnu (seul `players` existe),
  division par zéro, valeur hors domaine (négatif pour un `u32` : erreur, jamais de clamp).
  `resolve_balance_system` panique en nommant le fichier, le champ et le nombre de joueurs.
  Tests unitaires : `content::expr` (désérialisation, résolution, erreurs) et `game::balance`
  (dépendance à `players`, erreurs = panic avec contexte).

### D61 : armure et bouclier

**Armure** (`StatId::Armor`, lue par `game::character::health::rollback_resolve_damage_events`, listée
dans `sim_core::stats::stat_reader`). Un dégât reçu devient `dégât × 100 / (100 + armure)`
(`combat::damage::after_armor`, formule de League of Legends : 100 d'armure = moitié des dégâts), en
`Fixed`, après `resolve_damage` (équipe, résistances, difficulté). Armure `<= 0` ou absente : le montant
est rendu tel quel (identité exacte, aucune trace ne bouge). `Armor` n'a pas de base dans `Stats` : un
modificateur (`objets/plaque`, `Modifier { stat: Armor, .. }`, perk, mutation) s'applique sur 0.
`DamageKind::True` ignore l'armure **et** le bouclier.

**Bouclier** (`combat::actors::Shield(u32)`), distinct de l'armure : chaque point absorbe **un coup
entier** (le dégât est annulé, le point disparaît, `SHIELD_IFRAMES` = 15 frames d'invulnérabilité
`Health.invulnerable_until_frame` pour qu'une rafale n'en consomme pas plusieurs). Composant
**neutre** (`rollback_and_trace_neutral`), posé au premier gain par `GainShield(n)` (variante **en fin
d'enum** d'`Action` : effet, objet `OnUse`/consommable, synergie ; refusée dans un power-up, `n = 0`
refusé par le lint). Le gain passe par `combat::actors::queue_gain_shield` (commande différée : ajoute à
un `Shield` existant). Ordre au résolveur : **bouclier d'abord, puis armure** sur ce qui passe. Trace :
`ggrs{f= shield_absorb target= left=}`.

**Contenu** (`testbed`) : `objets/plaque` (passif, +100 d'armure), `objets/egide` (consommable,
`GainShield(1)` au ramassage). Scénarios : `armor_reduces_damage`, `armor_true_damage` (chute en fosse
= dégât `True`, +100 d'armure par `modifiers`), `shield_absorbs_hit`, `shield_then_armor`. Lint :
fixture `shield_zero`. HUD : source `shield` laissée à M2-T18.

## 19. Statuts (T1.3, chantier B3)

Code : `combat::status` (règles pures, `Statuses`, `StatusLibrary`), `combat::projectile`
(`apply_projectile_on_hit_system`, `status_tick_system`, `status_motion_system`),
`game::statuses` (bibliothèque depuis le registre), `content::lint::{lint_statuses,
lint_weapon_statuses}`.

**Contenu** : kind `Status` (`statuses/<id>.ron`), l'id nomme le statut, `kind` son genre
(`StatusDef`) :
```ron
// games/testbed/assets/statuses/brulure.ron — game.ron : (path: "statuses", kind: "Status")
(kind: Burn, frames: 120, damage: "4", period: 30)
// lenteur : (kind: Slow, frames: 120, factor: "0.5") ; etourdi : (kind: Stun, frames: 60)
// gel : (kind: Freeze, frames: 90)
```
Lint : `frames > 0` ; `Burn` : `damage > 0`, `period > 0` ; `Slow` : `factor` dans `]0, 1]`
(fixtures `status_out_of_range`, `status_unknown`).

**Pose** : `Action::ApplyStatus { status, stacks }` (variant ajouté en fin d'`effects::Action` par T1.3, suivi depuis de `RefillAmmoOf`, D40) dans
l'`on_hit` d'un projectile composable, exécutée sur le personnage touché par
`apply_projectile_on_hit_system` (`RollbackSystemSet::Projectiles`). Statut inconnu ou `stacks`
0 : lint. Dans un effet (T1.10) ou un power-up : lint (`Unsupported` / refus) — v1 n'a pas de
cible pour un statut hors d'un coup. **Suite** : `OnDamageTaken` connaît sa source, un effet
« riposte » pourrait y poser un statut (v2) ; `OnKill` n'a pas de cible vivante.

**État** : composant `Statuses` (rollback + checksum, enregistrement de T1.0a **gardé tel quel** :
les traces de référence l'incluent déjà sans porteur ; la variante neutre, essayée, changeait le
checksum de toutes les frames de tous les scénarios). Un personnage n'en porte que si un statut
lui a été posé, le composant est retiré quand le dernier expire : aucune trace existante, zombies
compris, ne change. Entrée `StatusEntry { status, stacks, expires_at_frame, id, source,
source_team, next_tick_frame }`, dans l'ordre de pose.

**Règles** (pures, testées) :
- `Burn` : un tick de `damage` toutes les `period` frames depuis la pose, tant que le tick tombe
  au plus tard à l'expiration ; chaque tick est un `DamageEvent` de genre `Fire` (tags `status`,
  `<id>`), **crédité à la source** (kill compris), résolu dans la même frame par le résolveur
  unique (immunités, résistances, équipes ; source disparue : équipe relevée à la pose).
  Réapplication : durée restante + durée de base, plafonnée à 2 × la base ; `stacks` =
  applications actives, au plus 2.
- `Slow` : modificateurs `MoveSpeed` et `EnemyMoveSpeed` `Mul factor`, source
  `Named("status:<id>")`, jusqu'à l'expiration ; réapplication : rafraîchie.
- `Stun` : ni déplacement ni tir ; réapplication : rafraîchie. Joueur : inputs ignorés **dans la
  simulation** (`apply_inputs`, tir, mêlée ; visée gardée), jamais à la lecture des inputs (un
  rejeu `Scripted` doit rester identique). Ennemi : aucune règle de comportement retenue (une
  charge est interrompue), pas de déplacement (`move_enemies`), pas d'attaque, émetteur suspendu.
- `Freeze` : `Stun`, plus vitesse **et recul** remis à zéro chaque frame (`status_motion_system`,
  `Movement`, avant `move_characters`).

- `Charm` (suivi Q12, réponse de William du 2026-10-16 ; `StatusDef::Charm`, **en fin d'enum** : le `derive(Hash)` hache
  l'index de variante, les sept d'avant gardent le leur, et aucun contenu existant ne l'utilise, donc aucune trace ne
  bouge ; `StatusKindEntry::Charm`, `(kind: Charm, frames: 200)`). Réapplication : durée rafraîchie, jamais de pile.
  **Sans effet sur un joueur.** Un ennemi charmé sert le camp des joueurs pendant la durée :
  - **Cible** (`ai::behavior::enemy_target_selection`, `ai::charm::nearest_target`, pure) : l'ennemi **hostile le plus
    proche**, c'est-à-dire un autre ennemi vivant, **non charmé**, de l'équipe `Enemies` (jamais `Allies` ni `Neutral` :
    un coup `Players` ne les touche pas), hors salle dormante ; distance au carré en
    `FixedWide`, **égalité : le plus petit `GgrsNetId`** (indépendant de l'ordre d'itération). Portée illimitée, ni
    aggro ni vue. Cible stockée dans `EnemyTarget` avec `TargetType::Enemy` (variante ajoutée en fin d'enum) ; sans
    ennemi hostile : `Idle`. Une attaque en cours contre un ennemi encore présent va à son terme ; toute autre
    attaque (joueur, fenêtre) est abandonnée à la sélection suivante.
  - **Mouvement** (`pathing::move_enemies`, `ai::charm::charm_motion`, pure) : droit sur la cible à pleine vitesse, à
    l'arrêt à moins de `min(OptimalAttackDistance, attack_range)` (un ennemi à portée courte, `mouche` à 26, doit pouvoir
    frapper) ; glissement sur les murs comme tout ennemi, **sans champ de flux** (il ne
    mène qu'aux joueurs : un charmé ne contourne pas un mur). Il n'est pas repoussé par sa cible (la séparation entre
    ennemis le tenait à une quarantaine d'unités, hors de portée de corps à corps).
  - **Attaque** (`behavior::charm_attack`) : corps à corps seulement, portée `attack_range` et cadence
    `attack_cooldown_frames` de l'ennemi, `MonsterState::Attacking { target: AttackTarget::Enemy }` (variante ajoutée
    en fin d'enum) ; le dégât est traduit à la frame suivante (`enemy_attack_damage_translate_system`) avec
    `source_team: Players` (adversaire de sa cible : `team_allows_hit` le laisse passer). **Aucun tir et aucune
    attaque sur un joueur** : une salve commencée contre un joueur est interrompue (`Emitter` retiré), les règles de
    comportement de l'ennemi (`behaviors`) sont ignorées pendant le charme (aucune règle retenue : ni `Charge`, ni
    ouïe qui lui donnerait un joueur pour cible) ; sa **hitbox de mêlée** (`enemy_melee_attack_system`, qui ne
    regarde pas l'IA) est retenue tant qu'il est charmé (`held || charmed(statuses)` : état dérivé du statut, aucun
    composant à lever) : scénario
    `status_charm_contact` (un `coward` à griffes collé au joueur, aucun dégât pendant le charme). Peur et charme
    cumulés : `Fear` l'emporte (il fuit, ne frappe pas). Défaut **préexistant** relevé par le suivi, **corrigé par D90** : `Statuses` est retiré dès qu'il est vide
    (`combat::projectile`) et `rules::melee_hold_system` ne retrouvait plus alors un ennemi sans `BehaviorRuntime` : la
    `MeleeHold` posée par `Fear` ne se levait jamais. Le système voit maintenant aussi tout ennemi qui porte `MeleeHold`
    (`Or<(BehaviorRuntime, Statuses, MeleeHold)>`) et la lève à la fin de la peur. Preuve §10 : `status_fear` (première
    frame qui diffère : f301, la fin de la peur, `Statuses` et `MeleeHold` retirés ensemble ; hors `MeleeHold` les 399 frames
    sont identiques) et `generated/isaac/tears_rancune_vive` (f152 : la retenue levée avec la peur ; f153 : la
    `cible_mordante`, libérée une frame avant que la peur soit reposée, frappe le joueur, ce qui crée sa hitbox, et la
    retenue revient à f154) ; test `la_retenue_d_une_peur_finie_est_levee_sans_regles_de_comportement`. Le charme n'avait
    pas ce défaut (retenue dérivée du statut).
  - **Fin** : le statut retiré (`Statuses::tick`), la cible `Enemy` est effacée à la frame suivante et l'ennemi
    redevient hostile (joueur le plus proche, comme avant) sans état à nettoyer.
  Teinte rose des calques (`feedback::status_tint`, présentation). Scénarios `status_charm` (le `breacher` charmé frappe la
  `target` la plus proche, jamais la lointaine ni le joueur, puis se retourne contre le joueur à l'expiration) et son
  témoin `status_charm_temoin` ; arme `status_charm` (scénario généré `weapon_status_charm`), statut `charme`.

**Ordre** : pose (`Projectiles`, après les coups de la frame) → `status_tick_system` (même set,
juste après : ticks de `Burn` puis expiration, `expires_at_frame <= frame`, porteurs par
`GgrsNetId`) → résolution des dégâts (`CollisionDamage`, même frame). Un statut posé à la frame
`f` agit sur le déplacement à partir de `f + 1` (le set `Movement` est passé).

**Visuel dérivé** (`game::feedback`, hors simulation) : les calques d'un porteur prennent la
teinte de son statut le plus contraignant (gel, étourdi, brûlure, lenteur), relue chaque frame.

**Attentes** : `HasStatus(entity, status, present: true, at_frame)`, `StatusStacks(entity,
status, stacks, at_frame)` (`entity` : `NetId(n)` ou `Target`). **Testbed** : quatre statuts,
quatre armes `status_burn`, `status_slow`, `status_stun`, `status_freeze` dont le `test:` utilise
les deux attentes (scénarios générés `weapon_status_*`) ; scénarios écrits `status_slow_enemy`,
`status_freeze_enemy`.

## 20. Patterns, émetteurs et tir ennemi (T1.2, chantier B5 v1)

Code : `crates/combat/src/emitter.rs` (état `Emitter`, avancée pure testée, `emitter_system`),
`crates/combat/src/projectile.rs` (`Pattern`, `PatternLibrary`), `crates/combat/src/weapons/mod.rs`
(`spawn_bullet`, la **seule** fonction d'apparition d'une balle : tir de joueur, projectile né
d'un `on_expire`, tir d'émetteur), `crates/game/src/character/enemy/ai/behavior.rs`
(`ranged_attack`), `crates/game/src/patterns.rs` (bibliothèque construite depuis le registre).

**Patterns nommés** : kind de contenu `Pattern` (`(path: "patterns", kind: "Pattern")` dans
`game.ron`), un fichier `patterns/<nom>.ron` par pattern, id = nom de fichier. Contenu : un
`Pattern` RON, ex. `Ring(count: 8, speed: "90.0", projectile: "fireball", every: 60)`.
Référencés par la règle `Shoot(pattern: ...)` d'un personnage (§22) et par `Named("nom")` dans un pattern
(`on_expire` compris). La bibliothèque (`combat::projectile::PatternLibrary`) est une ressource
**hors rollback**, reconstruite à `OnEnter(GameLoading)` depuis le registre ; un nom inconnu
est une erreur de lint, jamais une panique en jeu (en jeu : avertissement, rien n'est tiré).

**Variantes ajoutées** (en fin d'enum : le `derive(Hash)` hache l'index de variante, le hash
des armes et projectiles existants ne bouge pas) : `Scatter(count, spread, projectile)` et
`Named("nom")`.

**Sémantique dans un émetteur** (le pattern, résolu, est aplati : les `Sequence` imbriquées
se déplient dans l'ordre RON) :

| Étape | Effet |
|---|---|
| `Aimed`, `Spread`, `Ring` (`every: 0`) | une salve, comme §16 ; durée 0 frame (l'étape suivante joue la même frame) |
| `Ring(every: n > 0)` | une salve tout de suite, puis une toutes les `n` frames **en tâche de fond**, jusqu'à la fin de la `Sequence` qui la contient ; un `Ring` **seul** est infini |
| `Scatter` | `count` tirs à des angles tirés dans `±spread/2` autour de la visée, flux RNG `"patterns"` |
| `Telegraph(n)` | `n` frames sans tir ; `Emitter::telegraphing() -> Option<frames restantes>` pour la présentation (dessin : T1.17) |
| `Wait(n)` | `n` frames sans tir |

La visée est **figée au départ** de la séquence (direction de la cible à la pose) : direction
de `Aimed`/`Scatter`, centre de `Spread`, premier rayon de `Ring`. L'émetteur est posé à la
frame `f` ; sa première étape joue à `f + 1` ; une étape après `Telegraph(n)` joue `n` frames
après l'entrée dans le télégraphe. Dans une frame, les étapes de la séquence passent avant
les couronnes de fond ; une couronne de fond ne tire plus la frame où la séquence finit
(`Sequence([Ring(every: 20), Wait(60)])` : trois salves). À la fin de la séquence,
`emitter_system` retire le composant.

**Projectiles tirés** : entrée `projectile` de la table `projectiles` de l'**arme de
l'émetteur** (§16), vitesse du `Ring` sinon celle de la définition ; nés au centre du tireur,
génération 0, équipe et tags du tireur (plus `bullet`), tir ami de l'arme, multiplicateur
`Damage` du tireur au départ. Ils suivent la même chaîne que les autres (`Weapon` puis
`Projectiles` : collisions par règles d'équipe §8, `on_hit`, `on_expire`, 8 générations).
`Bullet::player_handle` vaut `weapons::NO_PLAYER_HANDLE` (aucun joueur) ; il n'apparaît dans
aucun log, qui nomme le tireur par son `GgrsNetId` (`ggrs{f=… emitter net_id=… step=…}`).

**Déterminisme** : `emitter_system` (set `Weapon`, après tout tir de joueur et de mêlée)
avance les émetteurs par `GgrsNetId` ; le flux `"patterns"` n'est créé et consommé que par un
`Scatter` qui tire, et par rien d'autre : la suite des tirs d'un émetteur ne dépend que de la
graine de run et de l'ordre des émetteurs, **pas du nombre de joueurs** (test unitaire
`meme_graine_meme_tir_a_un_et_quatre_joueurs`, scénarios `enemy_ring`/`enemy_ring_quad`).

**Tir ennemi** : règle `Shoot` de `ai.behaviors` (T1.4, §22 ; remplace le champ `ai.ranged` de T1.2) :
```ron
behaviors: Some([Shoot(weapon: "fireball_gun", pattern: "ring_8", range: "260.0", cooldown_frames: 90)]),
```
`spawn_enemy` équipe `weapon` (active, dans `WeaponInventory` ; la griffe de corps à corps
reste posée selon `attack_range`) et pose `RangedAttackState`. `enemy_attack_system` joue le
tir **avant** le corps à corps (`ranged_attack`) :
- cible `Player` vivante, debout, à moins de `range`, refroidissement écoulé, état
  `Idle`/`Chasing` → pose un `Emitter` (visée = position de la cible),
  `MonsterState::Attacking { target: Player }` ;
- l'ennemi **ne bouge pas** tant que l'émetteur est posé (`move_enemies`, télégraphe compris) ;
- séquence interrompue si la cible meurt, passe à terre ou sort de `range` (l'émetteur est
  retiré ; le tireur mort emporte le sien) ; fin ou interruption → refroidissement de
  `cooldown_frames`, retour à `Chasing` ;
- un `Attacking` posé par un tir n'émet jamais le coup direct de corps à corps
  (`enemy_attack_damage_translate_system`).
Sans règle `Shoot` (`EnemyAiConfig::ranged` à `None` ; le champ RON `ai.ranged` n'existe plus
depuis T1.4, §22), rien ne change (hash de `EnemyAiConfig` : `ranged` n'est haché que s'il est
présent, hacher un `None` déplacerait le checksum de tous les ennemis).

**État rollback et checksum neutre** : `Emitter` (combat) et `RangedAttackState` (game) sont
enregistrés par `RollbackTraceApp::rollback_and_trace_neutral` : un composant que **aucune**
entité ne porte contribue `0` au checksum GGRS. Raison, vérifiée sur `idle` : bevy_ggrs 0.22
fait contribuer à un type enregistré **sans porteur** une part constante `K = hash(0u64)`,
**la même pour tous les types vides**, combinée par XOR — un type vide de plus déplace toutes
les traces dès la ligne 1, deux de plus s'annulent (trace identique). **Piège pour toute
voie** : un nouveau composant rollback absent du contenu existant passe par la variante
neutre, jamais par `rollback_and_trace` en comptant sur la parité.

**Lint** : `Scatter`/`Telegraph`/`Wait` refusés en `on_expire` (même au travers d'un `Named`) ;
pattern nommé inconnu (`BrokenReference`) ; cycle de `Named` ; `count = 0`, `spread`/`speed`
négatifs dans un pattern nommé ; règle `Shoot` : arme inconnue, pattern inconnu, projectile du
pattern absent de la table de l'arme (`BrokenReference`), `cooldown_frames = 0`,
`range <= 0`. Fixtures `pattern_unknown_name`, `pattern_scatter_on_expire`,
`ranged_projectile_missing`, `ranged_cooldown_zero`.

**Testbed** : arme `fireball_gun` (table `fireball`, `arrow`), patterns `ring_8`
(`Ring(count: 8, every: 60)`) et `volee` (`Sequence([Telegraph(30), Aimed(count: 3),
Wait(20), Scatter(count: 4)])`), personnages `turret` (immobile) et `archer` (poursuit),
carte `testbed/arena_tir.ldtk` (copie d'`arena`, tourelle au centre, archer à l'ouest ; le
joueur 0 est le plus proche de la tourelle, les joueurs 1 à 3 sont plus loin et hors des
rayons de la couronne). Scénarios `enemy_ring`, `enemy_ring_quad`, `enemy_telegraph`.

## 21. Terrain destructible et cavernes (T1.0b + T1.6, chantier E3)

Code : crate `world` (grille, générateur, destruction, `WorldPlugin`),
`crates/map_ldtk/src/generation/cave.rs` (niveau LDtk), `crates/map_ldtk/src/game/cave.rs`
(branchement ECS), `crates/map_ldtk/src/loader/mod.rs` (`resolve_map_config`, `CaveSlots`), `crates/game/src/cave_assets.rs`
(source d'assets `cave://`).

**Contenu.** Kind de dossier `Cave` (`content::registry::CaveEntry`) : un fichier RON par
caverne, id = nom de fichier sans extension, champs de `world::CaveConfig` ; le dossier contient
aussi le gabarit LDtk `gabarit.ldtk` (définitions de couches et d'entités ; ses tilesets sont
relatifs au dossier des cavernes, `../testbed/atlas/...` dans le testbed : un chemin qui ne se
résout pas bloque le chargement du niveau).
```ron
// games/testbed/assets/caves/petite.ron  — game.ron : (path: "caves", kind: "Cave")
(width: 48, height: 32, fill_ratio: "0.45", iterations: 4, birth: 5, survive: 4,
 min_floor_ratio: "0.3", enemy_spawns: 4)
```

**Désignation : `cave:<id>`** partout où une carte LDtk est attendue : `entry.start_map`,
`Scenario.map`, `--map` d'`alacod-sim`, `levels` d'une séquence `Floors` (ex.
`floors/caverne.ron` du testbed). Au chargement (`setup_generated_map`), la désignation devient
le chemin propre à la caverne `cave://<dossier>/<id>.ldtk` (servi avec le gabarit du dossier par
`game::cave_assets` : une caverne = un asset, sinon l'`AssetServer` rendait la première caverne
à tous les étages, m1-integration-scenarios) + `MapGenerationMode::Cave(config)` ; la ressource `MapGenerationConfig` garde
`cave:<id>` (un enregistrement se rejoue tel quel). La graine est `map_seed`, comme `Basic`.

**Lint** (`content::lint::lint_caves`, `lint_floors`, `lint_entry_point`) : au moins 16 × 16
cases, `fill_ratio` dans `[0, 1]`, `min_floor_ratio` dans `[0, 0.9]`, `birth`/`survive` ≤ 8,
gabarit présent (`BrokenReference`) ; `cave:<id>` inconnu dans `levels` ou `start_map` :
`BrokenReference`. D36 : dans `levels` d'une séquence `Floors`, une caverne **sans ennemi**
(`characters` vide ou `enemy_spawns = 0`) est refusée (`OutOfRange`) : le portail d'un niveau
`Floors` s'ouvre quand il ne reste aucun ennemi, donc aussitôt (et en boucle si c'est le dernier
niveau). Un passage voulu se déclare `transit: true` dans le RON de la caverne (champ de
`CaveConfig`, lint seul, non sérialisé quand il est faux) : `caves/petite.ron` du testbed
(séquence `floors/caverne.ron`). Les cartes LDtk d'une séquence ne sont pas contrôlées (il
faudrait lire leurs entités `CharacterSpawn`).

**Grille.** `world::CellGrid { width, height, cells }`, `CellKind { Floor, Wall, Rock }` :
`Wall` indestructible (bordure), `Rock` destructible. Cases de **16** unités (`GRID_CELL_SIZE`
de la navigation), origine (0, 0), **+y vers le haut** comme le monde et `GridPos` (la rangée 0
est en bas, au contraire des rangées LDtk). Ressource rollback + checksum + trace, enregistrée
en `rollback_and_trace_resource_neutral` : vide (défaut) hors caverne, contribution 0, traces
des autres cartes inchangées. Debug compact (une chaîne par rangée : `.` sol, `#` mur,
`r` roche). `world::Destructible` marque l'entité de niveau LDtk d'une caverne (pas une entité
par case). `RollbackSystemSet::World` : entre `Projectiles` et `CollisionDamage`, donc avant la
navigation (`EnemyAI`), qui voit une destruction dans la frame.

**Générateur** (`world::cave::generate(seed, &config)`, pur, `RollbackRng`) :
1. bordure `Wall`, intérieur `Rock` avec la probabilité `fill_ratio`, sinon `Floor` ;
2. `iterations` passes (voisinage de Moore, hors grille = solide) : `Floor` → `Rock` si au moins
   `birth` voisins solides ; `Rock` reste `Rock` si au moins `survive`, sinon `Floor` ;
3. connexité : la plus grande composante 4-connexe de sol est gardée (égalité : la première au
   balayage depuis le bas), les autres deviennent `Rock` ;
4. ratio de sol hors de `[min_floor_ratio, 0.9]` : nouvel essai avec la suite du même RNG (64 au
   plus). Testé sur 1 000 graines (connexité, bordure, ratio, spawns sur le sol).

**Points d'intérêt** (`world::points_of_interest`), choisis parmi les cases **dégagées** (sol
dont les 8 voisines sont du sol, `world::cave::is_open` : un corps de personnage de 20 × 20,
décalé vers le bas, y apparaît sans toucher de mur — sinon chaque déplacement serait refusé) :
`PlayerSpawn{index}` 0..3 = les plus proches du centre (distance au carré, puis y, puis x) ;
`ZombieSpawn` × `enemy_spawns` = les plus éloignées du `PlayerSpawn` 0 en distance de grille
(4-connexe), espacées d'au moins 8 cases (Tchebychev). Champ optionnel `characters: [...]`
(vide par défaut) : un `CharacterSpawn` (équipe `enemies`) par point `ZombieSpawn`, personnages
pris à tour de rôle (lint : personnages chargés) — un jeu sans vagues (testbed) peuple ainsi
une caverne (`caves/bench.ron` : six `follower`).
Dégagement des `ZombieSpawn` (D41, m1-d41-spawns-degages) : `CaveConfig::spawn_clearance` cases
(`world::is_open_within`), calculé par le registre depuis le plus grand corps **en jeu** (collider
× `scale`) de `characters` (`content::registry::spawn_clearance_for` : 1 jusqu'à 24 px du centre,
une case de plus par 16 px) ; 1 pour tout le contenu actuel (points inchangés).
Accessibilité des `ZombieSpawn` (D48, m1-d48-ennemis-hors-champ) : un point n'est retenu que si
le champ de flux du gabarit de navigation du plus grand corps de `characters` l'atteint depuis
les points des joueurs (`world::nav::nav_distances`). Les règles de passage (`blocked_for`,
`too_narrow`, coin coupé en diagonale) sont **les mêmes fonctions** que le champ de flux de
`game` (`world::nav`, appelées par `FlowFieldCache`) ; un test compare les deux sur les cavernes
de `throne`. `CaveConfig::nav_large`, calculé par le registre (collider × `scale` de plus de
20 px en largeur ou en hauteur, le seuil d'`AgentSize::Large`), choisit le gabarit. Sans cette
règle, une poche ouverte mais reliée par un couloir d'une case (petit gabarit) ou de deux cases
(grand) gardait l'ennemi hors de son champ, immobile et hors d'atteinte des bots (soft-locks des
graines 43 et 162 de `throne`). L'ordre des candidats reste la distance de grille. Lint : une
caverne peuplée dont une des graines de contrôle 1 à 5 n'a aucun point atteignable est refusée
(fixture `cave_spawns_unreachable`).

**Niveau LDtk.** Le callback du loader réécrit le niveau unique du gabarit
(`generation::cave::build_cave_ldtk`) : dimensions, IntGrid `Walls` (1 = `Wall` ou `Rock`),
entités, niveau en (0, 0) monde (`world_y = -px_hei`). Tout le reste (colliders fusionnés,
`load_intgrid_walls`, spawns, mode `Floors`) suit le chemin LDtk ordinaire. Au
`LdtkMapLoadingEvent`, `CellGrid` reçoit la même grille (emplacement 0) ; au passage d'un niveau
`Floors`, la grille du nouvel emplacement (vide pour une carte ordinaire).

**Destruction.** `effects::Action::DestroyTerrain { radius }` (rayon en unités, `Fixed` en
chaîne) : toute case `Rock` dont le centre est à **strictement** moins de `radius` de la
position de l'action devient `Floor` ; `Wall` ne change jamais. Positionnelle : réservée aux
projectiles (lint : refusée dans un power-up, `radius > 0`). Deux usages :
- `on_expire: [DestroyTerrain(radius: "40.0")]` (`combat::projectile::ExpireAction`) : au
  point de fin du projectile, quelle qu'en soit la cause (arme testbed `grenade_creuse`, sans
  rebond : elle s'arrête contre la roche et creuse) ;
- `on_hit: [DestroyTerrain(radius: "24.0")]` : au point d'impact sur un mur. Un mur touché
  émet `combat::projectile::ProjectileWallHit` (projectile, source, position au contact,
  actions `on_hit`) dans la branche `register_wall()`, **seulement** pour un projectile qui
  porte des actions `on_hit` (file neutre : vide, elle laisse les traces existantes intactes) ;
  `projectile_wall_terrain_system` en tire les demandes.

**Décision (D36) : `on_hit: [DestroyTerrain]` ne creuse que sur un mur.** Un projectile qui
touche un **personnage** émet `ProjectileHit`, dont `apply_projectile_on_hit_system` n'exécute
que `ApplyStatus` : `DestroyTerrain` y est ignoré, volontairement. Le terrain réagit au contact
du terrain ; creuser au point d'impact sur un personnage ouvrirait la roche autour de chaque
ennemi touché. Pour creuser à tout impact (mur ou personnage), utiliser `on_expire:
[DestroyTerrain]` : il part du point de fin du projectile, quelle qu'en soit la cause (un
projectile qui s'arrête sur un personnage expire là).

Les deux posent une `world::DestroyTerrainRequest` (file `FrameEvents` neutre) dans
`Projectiles` ; `apply_destroy_terrain_system` (`World`) l'applique et émet `TerrainDestroyed`
(cases creusées), moment clé `terrain` et métrique `terrain_destroyed` des scénarios.
`ExpireAction` a un `Hash` manuel : `#[derive(Hash)]` d'un enum à une seule variante n'écrit pas
le discriminant, ajouter une variante dérivée aurait déplacé le checksum de toute arme à
`on_expire` dès la frame 0 (règle générale : ajouter une variante à un enum haché qui n'en avait
qu'une change les traces).

**Murs et navigation après destruction** (`map_ldtk::game::cave`, set `World`) : quand la frame
a creusé, tous les `Wall` rollback (dans une caverne, ce sont ceux du niveau) sont détruits
(`despawn_rollback`) et recréés depuis `CellGrid` par la même fusion gloutonne
(`generate_collision_rectangles`), net ids `cave_wall_<frame>_<i>` ; les cases murées de la
navigation sont rechargées (`FlowFieldCache::reload_walls`, seule exception à « immutable after
load ») ; la signature des murs de `CollisionGrids` et `rebuild_blocked_cells` reconstruisent le
reste au pas suivant. `CellGrid`, murs et `FlowFieldCache` sont rollback : un rejeu qui remonte
avant la destruction les retrouve cohérents (test synctest
`destruction_creuse_et_reconstruit_les_murs_en_synctest`). Aucun état hors rollback ne doit
retenir « la dernière grille appliquée ». Coût nul hors destruction ; un flow field vraiment
incrémental n'est à faire que si `bench_cave` ne tient pas son budget. Rejeté : un collider par
case (des milliers d'entités rollback au checksum).

**Scénarios.** `explode_wall` (cratère observé par `CellState`), `bench_cave` (caverne `bench` :
quatre joueurs à grenades, six followers, ≥ 50 destructions, plancher de 40 fps simulés dans
`tests/budgets.ron`).

**Attente** `CellState(x, y, kind, at_frame)` : nature de la case à la frame exacte, en
coordonnées de grille ci-dessus ; échoue hors de la grille (donc toujours hors caverne).

**Présentation.** Le gabarit n'a pas de tuiles : un carré par case solide est dessiné depuis
`CellGrid` (`cave_cells_visual_system`, rendu seulement), reconstruit quand la grille change.

**Surfaces** : couche séparée `world::SurfaceGrid` (§26), indépendante de `CellGrid`.

**Limites.** Pas de surfaces de caverne (v2 de §26). (Les bots naviguent dans les cavernes
depuis m1-v3-bots-pathfinding, §24.) Dans
le testbed (pas de vagues), une caverne sans `characters` n'a pas d'ennemi : son portail
`Floors` s'ouvre aussitôt.

## 22. Behaviors composables (T1.4, chantier D1)

Code : `crates/behaviors` (vocabulaire `Behavior`, `Perception`, `Targeting` ; sélection pure
`select`/`applicable`, faits `SelectionContext`), `crates/game/src/character/enemy/ai/rules.rs`
(`behavior_select_system`, `behavior_motion`, `charge_damage_translate_system`,
`current_rule`), `state.rs` (`EnemyBehaviors`, `BehaviorRuntime`, `default_behaviors`).

**Contenu** : champs optionnels de `ai` (`EnemyAiConfigRon`) :
```ron
ai: Some((
    // ... champs habituels (aggro_range, attack_range, flee_threshold...)
    behaviors: Some([
        Shoot(weapon: "fireball_gun", pattern: "volee", range: "240.0", cooldown_frames: 90),
        KeepDistance(min: "120.0", max: "200.0"),
        Chase(profile: "Ground"),
    ]),
    perception: Some((senses: [Sight("80.0"), Hearing("300.0")])),
    targeting: Some(Nearest(ignore: ["ghost"])),
)),
```

**Sélection par priorité** : la liste est ordonnée par priorité (ordre RON) ; à chaque frame,
dans `RollbackSystemSet::EnemyAI`, la **première règle applicable** gagne (pas de `when:` en
v1). Applicabilité implicite :

| Règle | Applicable quand | Exécution |
|---|---|---|
| `Melee(arme)` | une cible (joueur, ou obstacle cassable sur la route si `can_break`) est à moins de `attack_range` | l'attaque de mêlée d'origine ; l'arme (`melee_weapons.ron`) est équipée au spawn — `zombie_claws` n'est plus codé en dur |
| `Shoot { weapon, pattern, range, cooldown_frames }` | séquence de tir en cours, ou cible vivante et debout à moins de `range`, refroidissement écoulé | `ranged_attack` (§20) ; se compile vers `EnemyAiConfig::ranged` |
| `Charge { telegraph }` | charge en cours, ou cible entre `attack_range` et 3 × `attack_range`, refroidissement écoulé | `telegraph` frames immobile, ruée en ligne droite vers la position **figée** de la cible à 3 × la vitesse jusqu'au contact (`attack_range`) ou 60 frames, murs respectés ; dégât de contact `attack_damage` (émis la frame suivante, `CollisionDamage`, tags `melee`+`charge`) ; refroidissement `attack_cooldown_frames` |
| `KeepDistance { min, max }` | cible à moins de `min`, ou déjà retenue et cible à moins de `max` (hystérésis) | recule : case voisine du champ de flux de **coût le plus élevé** (départage par `GridPos`), repli à l'opposé de la cible |
| `Flee` | santé ≤ `flee_threshold` × max | recule comme `KeepDistance` ; **pas d'attaque au corps à corps** tant que retenue (D38 : marqueur `MeleeHold`, rollback neutre, lu à la frame suivante) |
| `Strafe` | cible à moins de `Sight` | perpendiculaire à la cible, sens alterné toutes les 45 frames depuis l'entrée dans la règle |
| `Chase { profile }` | cible connue | `move_enemies` d'origine, sur le champ de flux de la **clé** de l'ennemi (D41 + D38, m1-navigation-profils-tailles) : `NavKey { profil (EnemyAiConfig::nav_profile), gabarit (Small ≤ 20 px de corps en jeu, Large au-delà : centre à une case des murs) }`, construite seulement pour les ennemis qui se déplacent ; un profil qui voit les mêmes obstacles que `GroundBreaker` sur la carte (pas de fenêtre ni de barricade) partage son champ (`FlowFieldCache::canonical`) |
| `Wander` | toujours (règle de fond) | direction tirée dans le flux RNG `"behaviors"` toutes les 60 frames (consommé seulement par `Wander`, ordre `GgrsNetId`), demi-vitesse |

Une règle absente n'existe pas pour ce personnage. Les déplacements imposés (`KeepDistance`,
`Flee`, `Strafe`, `Wander`, `Charge`) n'ont pas le ralentissement près du joueur de `Chase` et
passent par la même résolution de collision (glissement sur les murs).

**Liste par défaut** (personnage sans `behaviors:`, ex. testbed T2.9) : `[Flee]` si
`flee_threshold`, puis `Shoot` si `ranged`, puis `Melee("zombie_claws")` si
`attack_range > 0`, puis `Chase(profile: <movement_type>)` sauf si `stationary` — exactement le
comportement d'avant T1.4. Les zombies de `games/zombies` déclarent leur `ai` en entier
(valeurs du préréglage `zombie()`) et `behaviors: [Melee("zombie_claws"),
Chase(profile: "GroundBreaker")]` : « les zombies deviennent un fichier », traces identiques.

**Perception et ciblage** : `Sight(r)` = rayon seul, sans ligne de vue ni lumière ; se compile
vers `aggro_range` (repli). `Hearing(r)` : un tir de joueur né à moins de `r` rend le tireur
connu 120 frames, même hors de vue (seulement pour un ennemi à `BehaviorRuntime`). `needs_light` ignoré (E8). `Targeting::Nearest { ignore }`
(repli `ignore: []`) : l'algorithme d'origine (plus proche par le champ de flux, repli en ligne
droite, joueurs à terre ignorés) ; un joueur portant un tag d'`ignore` n'est jamais ciblé.

**Vue directe** (D66, champ `sight_range` de `ai`, chaîne `Fixed`, lint `>= 0`) : un joueur au-delà
de `aggro_range` mais à moins de `sight_range` est **ciblé** s'il est en ligne de vue dégagée
(`FlowFieldCache::line_of_sight`, segment ennemi→joueur contre `wall_cells` : murs de l'IntGrid,
terrain dynamique et portes fermées coupent le regard, une fenêtre non ; parcours de grille exact en entiers (bits des `Fixed` en `i128`, frontière x ou y la plus proche
décidée par produits croisés, sans quotient accumulé : verdict symétrique par construction) ; au passage
d'un coin exact, deux cases orthogonales murées coupent la vue, une seule non). Un ennemi déjà
en `Chasing` vers un joueur le garde tant que celui-ci est à moins de `sight_range`, coin ou pas
(sans quoi le champ de flux qui contourne un mur lui ferait perdre sa cible en route) : la mémoire
est `EnemyTarget`, déjà rollback, aucun état nouveau. `sight_range` absent (ou inférieur à
`aggro_range`) = `aggro_range` : le chemin d'avant, aucune trace ne bouge ; le champ est
statique et **exclu du `Hash`** de `EnemyAiConfig` (comme `stationary`) : `EnemyAiConfig` est bien rollbacké
et sous checksum, mais le champ est immuable après la création de l'ennemi (donnée de contenu, jamais écrite
en simulation), donc un rollback ne peut pas le faire diverger ; l'exclusion est un choix explicite qui garde
les traces des jeux sans vue directe. Ne change **que** les
ennemis dont le comportement dépend d'une cible : `move_enemies` suit le champ de flux vers le joueur
le plus proche quel que soit l'état (`Idle` ou `Chasing`), si bien qu'un `Chase` seul approche de toute
la carte, `aggro_range` ou non (constaté, D66) ; un ennemi à `Wander` (`rodeur`) ou un tireur, lui,
n'agit qu'avec une cible. `throne/rodeur` : `sight_range: "900.0"`. Outil : test ignoré
`sight_probe` (`ALACOD_SIGHT=<scénario>:<frame>`, ennemis, ligne de vue, cases d'un tir à `MIN`..`MAX`
unités, `ALACOD_SIGHT_HIDDEN=1` pour les cases cachées). Scénarios : `throne_vue_directe` (caverne
d'essai `essai_vue`, rodeur à 730 unités), `throne_vue_coupee` (témoin : mur sur le segment),
`throne_rodeur_vue_graine_4` (la vraie carte, `cave:niveau_1` graine 4).

**État et traces** (critère central : aucune trace existante ne change) :
- les règles résolues sont un composant **statique, hors rollback** (`EnemyBehaviors`, comme
  `Team`) : un ennemi existant ne gagne ni ne perd aucun composant rollback ;
- un ennemi qui ne liste que `Melee`/`Shoot`/`Chase` n'a **aucun état nouveau** : sa règle
  retenue est dérivée de son état (`current_rule` : séquence de tir → `Shoot`, `Attacking` →
  `Melee`, cible connue → `Chase`) ;
- les behaviors nouveaux gardent leur état dans `BehaviorRuntime` (règle retenue, frame
  d'entrée, phase de charge, errance, ouïe), **checksum neutre** (§20), posé seulement sur les
  ennemis qui en listent un ;
- `behaviors::BehaviorState` (contrat T1.0a) reste enregistré **tel quel** sous checksum
  ordinaire et n'est posé sur personne : le retirer, le passer en neutre ou le poser
  déplacerait toutes les traces par parité (§20, piège).
Code mort retiré : `enemy_movement_system`, `enemy_stun_recovery_system`, `apply_stun`,
`MonsterState::{Stunned, Breaching, Fleeing}` (jamais posés ; l'étourdissement est un statut,
T1.3). `enemy_target_selection` et `update_enemy_targets` ne font pas doublon (cible d'IA d'un
côté, point de chemin de `EnemyPath` de l'autre) : gardés. Dégât direct `attack_damage`
préservé tel quel (dette D28).

**Attentes** (`crates/scenario`) : `EnemyState(entity, behavior: "Chase", at_frame)` — règle
retenue (nom de variante) ; `EnemyDistance(entity, target: Player(h), min?, max?, at_frame)` ;
continues : `EnemyContactBefore(entity, frames)` (portée de mêlée d'un joueur atteinte au plus
tard à `frames`) et `EnemyNeverInWall(entity, from, to)` (collider sans chevauchement de `Wall`).
`entity` : `NetId(n)` ou `Target` (comme `HitsAtLeast`).

**Lint** : liste vide ; `Shoot` (règles `ranged` de T1.2) ; `Melee` arme inconnue ; `Chase`
profil inconnu (`Ground`, `Flying`, `Phasing`, `GroundBreaker`) ; `KeepDistance` `min >= max` ;
`Charge` télégraphe 0 ; tag d'`ignore` porté par aucun personnage. Fixtures
`behavior_melee_unknown`, `behavior_keep_distance_inverted`, `behavior_charge_zero`,
`behavior_unknown_profile`, `behavior_shoot_unknown`, `targeting_unknown_tag` (et `ranged_*` migrées vers `Shoot`).

**Testbed** : `kiter`, `charger`, `coward`, `drifter`, une carte chacun
(`testbed/arena_ia_keep|charge|flee|wander.ldtk` : un scénario ne peut pas faire apparaître de
personnage, une arène commune les ferait interagir), scénarios `enemy_keep_distance`,
`enemy_charge`, `enemy_flee`, `enemy_wander`.

## 23. Horloges et difficulté (T1.9, chantier F2)

Code : `crates/run/src/clock.rs` (`Clock`, `ClockDef`, `due_events`, `ClockFired`,
`FloorEntered`), `crates/game/src/clock.rs` (`resolve_clocks_system`, `clock_system`,
`DifficultyReader`, `scale`), `content::expr::difficulty_context`.

**Activation** : horloges et difficulté ne tournent que si une partie les demande — champs
`clocks: Some(["arene"])` et `difficulty: Some(true)` d'un scénario (`ClocksOverride`,
`DifficultyOverride`), ou `entry.clocks`/`entry.difficulty` du manifeste pour une partie jouée.
Déclarer les kinds dans `game.ron` ne suffit pas. Sans activation, `Clock` reste à sa valeur par
défaut (checksum **neutre**), aucun événement n'est émis, la difficulté vaut 1 : **aucune trace
ne change**.

**Horloges** (kind `Clock`, `clocks/<nom>.ron`) :
```ron
(
    scope: Floor,   // ou Run
    events: [
        (id: "tic", at: Seconds("1")),
        (id: "tac", at: Frames(120)),
        (id: "renfort", at: Seconds("2"), repeat: Some(Seconds("2"))),
    ],
)
```
Échéance relative au début de la portée (`Run` : frame 0 ; `Floor` : entrée dans l'étage
courant). Un événement répété a l'id `"<id>#<n>"` (`renfort#1` à 2 s, `renfort#2` à 4 s...), un
événement simple son id. `clock_system` (`RollbackSystemSet::FrameCounter`, avant l'incrément,
donc après le passage d'étage du set `Run`) : (1) si `FloorState::index` a changé, entrée
d'étage — temps d'étage remis à zéro, ids des horloges de portée `Floor` oubliés,
`FrameEvents<FloorEntered>` ; (2) événements échus par horloge (ordre des ids) puis par
événement (ordre du fichier) : id inséré dans `Clock::fired`, `FrameEvents<ClockFired>`, log
`ggrs{f=… clock id=…}` ; (3) difficulté. **`FloorEntered` n'est émis que si les horloges ou la
difficulté sont activées.** Aucun effet ne s'y attache encore : T1.10 est livrée sans
déclencheur d'étage ni d'horloge (`OnEvent` refusé `Unsupported`, v2, §27) ; un tel
déclencheur devra activer les horloges implicitement.

**Difficulté** (kind `Difficulty`, `difficulty.ron`) : `(value: "1 + floor * 0.5 + floor_minutes * 0.5")`,
une expression `content::expr` aux identifiants `players`, `floor`, `minutes`, `seconds`
(temps de run), `floor_minutes`, `floor_seconds` (temps d'étage) — **seulement pour ce kind**
(`players` reste le seul ailleurs, §18). Réévaluée en simulation chaque seconde et à chaque
entrée d'étage, depuis l'asset immuable (`DifficultyConfig`, hors rollback) ; le résultat seul
est dans `Clock::difficulty`. Consommateurs v1 : **santé des ennemis à l'apparition** (vagues,
spawners, `CharacterSpawn` : `health_max × difficulté`, évaluée à neuf pour l'étage où ils
apparaissent — ceux d'un nouvel étage apparaissent dans la frame du passage, avant la
réévaluation) et **dégâts infligés par les ennemis** (un seul point : le résolveur unique,
`rollback_resolve_damage_events`, pour les `DamageEvent` de `source_team == Enemies`).
`game::clock::scale` ne calcule rien quand la difficulté vaut exactement 1 (test : identique au
bit près).

**Attente et événement** : `Clock(id, fired: bool, at_frame)` ; moment clé `clock`
(`scenario::events`, libellé `horloge : <id>`).

**Lint** : horloges — ids d'événements uniques sur toutes les horloges (ils partagent
`Clock::fired`), échéances croissantes dans un fichier, `repeat > 0`, `entry.clocks` vers une
horloge connue ; difficulté — identifiants hors liste, valeur ≤ 0 pour `floor ∈ {0, 3}`,
`minutes ∈ {0, 10}`, `players ∈ {1, 4}`, `entry.difficulty` sans fichier. Fixtures
`clock_duplicate_id`, `clock_unordered`, `clock_repeat_zero`, `difficulty_unknown_identifier`,
`difficulty_non_positive`.

**Testbed** : horloge `arene`, `difficulty.ron`, séquence `deux_cibles` (`floor_a` puis
`floor_cible_b`) ; scénarios `clock_events`, `clock_floor_reset` (séquence `deux_niveaux`),
`difficulty_scales`. Dette D29 : les multiplicateurs par vague du clone zombies ne sont lus par
personne.

## 24. Bots de validation (m0-v7 phase 2)

Les profils RON `chasseur` et `acheteur` complètent `immobile`, `fonceur` et `prudent`.
Les profils v0 gardent leurs décisions. `alacod-sim --bots 4 --seeds 1..20 --until-wave 5`
utilise quatre `acheteur` par défaut ; `--profiles` permet les deux générations.
Pour comparer avec le digest M0, préciser `--map exemples/test_map.ldtk` : sans cet
argument, le manifeste choisit maintenant `maps/avant_poste.ldtk`.

`crates/bots/src/navigation.rs` réutilise `FlowField` et `GridPos` pour un Dijkstra
multi-source vers des postes de tir autour des ennemis, sur une grille physique de 8 px.
Le corps et son offset sont pris en compte ; murs, portes fermées et fenêtres, même cassées,
bloquent le mouvement du joueur. Les fenêtres permettent le tir. Les cibles sont triées par
`GgrsNetId`, et les diagonales ne coupent pas les coins. Le cache se reconstruit quand la
position en grille d'un ennemi, les colliders ou le corps changent. Le calcul se limite
à la composante physique accessible au joueur ; les champs d'approche restent en cache
tant que leurs clés et cette composante sont identiques. Il est dérivé dans
`ReadInputs`, hors simulation rollback, comme les inputs v0 : aucune décision ni RNG caché,
aucune ressource de jeu supplémentaire dans la trace. Le rejeu en `Scripted` capture
les inputs décidés et doit produire la même trace.

`crates/bots/src/hunter.rs` choisit le tir visible, la distance de combat, le changement
vers une arme approvisionnée, le rechargement et la réanimation. L'acheteur cherche aussi
les munitions, Juggernog et une porte abordable quand le champ est inaccessible (prix,
puis distance à l’ennemi, puis `GgrsNetId`). Les approches ponctuelles utilisent des
champs multi-source dérivés et partagés sur la même grille. Sans poste de tir accessible,
le bot s’approche jusqu’au rayon d’aggro d'un zombie `Idle` : il peut alors casser sa
fenêtre. Un zombie déjà en `Chasing` ne doit pas retenir le bot dans ce rayon quand il
n'existe toujours aucun poste de tir ; les portes et réparations restent possibles.
Sans poste de tir ni approche d'aggro, il cherche le point physiquement accessible le
plus proche du zombie. Cette investigation conserve la possibilité d'ouvrir une porte
ou de réparer, même quand le zombie inaccessible est à moins de 150 px.
Sans chemin d’approche, le bot répare aussi pour gagner les points de sa première porte.
Les interactions passent par le bouton existant : les règles de solde, coût, cooldown et
réanimation restent celles du jeu. Les unitaires couvrent navigation et décisions ;
`crates/scenario/tests/bots.rs` vérifie le rejeu v0 et v1.
Si une autre surface possède le prompt à portée du but choisi, le bot se rapproche
encore du but ou essaie une autre interaction, au lieu de rester immobile sans agir.
`crates/scenario/tests/hunter_doors.rs` vérifie que les quatre acheteurs finissent la
première vague des graines 2, 11 et 12 sans cap ni décès.
Les modes Manual, Shotgun et Burst relâchent le tir lorsque `WeaponState.is_firing`
est vrai, puis pressent de nouveau ; Automatic peut maintenir le bouton. Ce choix lit
l'état rollback de l'arme, sans compteur de pulsations caché dans le bot.

Le déplacement du joueur résout un mouvement diagonal bloqué en faisant glisser X,
puis en vérifiant Y à la position X obtenue. Vérifier les deux axes depuis la position
initiale pouvait autoriser leur combinaison à entrer dans le coin d'un mur. Les ennemis glissent
comme les joueurs (D39) : `move_enemies` et `move_characters` partagent
`combat::collider::slide_axes`.

En phase `Spawning`, après 600 frames sans spawn depuis le dernier spawn ou le début
de phase, une plage de distance vide utilise le spawner le plus proche d'un joueur
(égalité départagée par `GgrsNetId`). Les spawners dans la plage normale restent
prioritaires. Le drapeau `WaveState.spawn_fallback`, rollback et tracé avec cette
ressource, maintient ensuite la cadence normale pour cette vague et est remis à faux
dans `prepare_next_wave`. Le hash de `WaveState` conserve l'ordre historique des champs
et ajoute une contribution seulement si ce drapeau est actif, afin de conserver les
traces des vagues qui n'en ont pas besoin. La limite d'ennemis
simultanés et la taille des lots restent appliquées. Le test synctest
`crates/scenario/tests/spawn_stall.rs` force une plage vide et vérifie trois spawns,
sans desync, plutôt qu'un retour indéfini du système.

En `Spawning` ou `InProgress`, après 600 frames sans kill ni spawn, les zombies de
vague peuvent récupérer un point de steering sorti de sa case ou placé dans un mur.
`ai/pathing.rs` échantillonne des points entiers à l'intérieur de la prochaine case,
puis les étapes cardinales d'une diagonale, puis la case courante : un coin de mur
peut couper toutes les routes directes, et un point de la case courante dégage le
corps du coin avant le segment vers la case suivante. Le choix est stable (étape,
distance au point initial, x, y), et le collider avec son offset doit rester libre
sur chaque segment, vérifié à intervalles d'un pixel. Les fenêtres intactes restent
bloquantes. Une étape intermédiaire doit permettre de rejoindre un point libre dans
la case cible. Si la case cible n'offre aucun point libre pour le corps (un mur
couvre toute la case), le glissement le long d'un axe du déplacement de case est
essayé d'abord — une poche de coin est souvent ouverte latéralement (fenêtre cassée
dans le mur sous le coin) quand la diagonale est pincée — puis la rotation du cap
vers le waypoint par paliers de 45°, rotations proches d'abord, demi-tour en
dernier. La première direction dont le segment reste libre sur 24 px à intervalles
d'un pixel gagne, et le zombie glisse hors du coin avant que le champ ne reprenne. La direction récupérée évite le mélange de steering qui
réintroduisait le mur ; le mouvement conserve vitesse, séparation et collision
ordinaires. Le délai dépend des compteurs existants de `WaveState`, sans état caché
ou nouveau type rollback.

### v1 (T1.14)

**Esquive** (`prudent`, `crates/bots/src/dodge.rs`, pure en `Fixed`) : `BotView.projectiles` =
balles (`Bullet`, ordinaires ou composables) d'une équipe autre que celle des joueurs, à moins
de 320 px, les 16 plus proches, triées par `GgrsNetId` (`view::projectile_views`) ;
`BotView.body_radius` = demi-diagonale du collider du joueur. Un projectile qui **approche**
(`rel · v < 0`) menace si le point d'approche minimale de sa trajectoire linéaire dans les 30
prochaines frames est à moins de rayon du corps + taille du projectile + 8 px. Direction :
perpendiculaire à sa vitesse, du côté qui s'éloigne du point d'approche (à gauche s'il arrive
droit dessus), somme normalisée sur les menaces. Quand elle existe, elle remplace le
déplacement de `prudent` ; visée et tir vers l'ennemi le plus proche sont conservés.

**Gestion d'arme de `prudent`** (même règle que `chasseur`/`acheteur`) : recharge si le chargeur
est vide et la réserve le permet ; sinon passe à une arme utilisable (`switch_weapon`) ; ne
presse le tir d'une arme non automatique que détente relâchée (`WeaponState::is_firing` faux).
`fonceur`, `immobile`, `chasseur`, `acheteur` : inchangés.

**Complétion de niveau** : `prudent`/`fonceur` marchent vers l'ennemi le plus proche (la vue n'a
pas de limite de portée : `nearest_enemy` ne vaut `None` que s'il n'en reste aucun), puis vers
le portail ouvert ; en ligne droite jusqu'à la navigation ci-dessous. Approche du portail par `prudent` **pilotée en vitesse**
(m1-v3-bots-portail ; avant, freinage sous 48 px de T1.14) : les boutons ne donnent que le
signe de chaque axe et le corps garde son élan, si bien qu'à pleine vitesse le bot tournait
autour du portail sans entrer dans son rayon (24). Vitesse voulue = direction (la route
au-delà de 48 px, le portail en deçà) × min(120, 2 × distance) ; chaque axe est pressé selon
l'écart entre vitesse voulue et vitesse actuelle s'il dépasse 12 (`crates/bots/src/decide.rs`,
`approach_portal`). La vitesse transverse est annulée (plus d'orbite).

**Navigation de `prudent`/`fonceur`** (m1-v3-bots-pathfinding, suite de T1.14) : même calcul que
`chasseur` (`crates/bots/src/navigation.rs`, Dijkstra multi-source sur une grille de 8 px, murs,
fenêtres et portes, murs de caverne compris), dans un cache séparé (`DirectNavigation`), dérivé
hors rollback dans `ReadInputs`, **en mode `Floors` seulement** (en vagues, les ennemis
viennent aux joueurs : `enemy_visible` vaut vrai et `route` est absente, comportement de
T1.14 ; `crates/bots/src/input.rs`). `BotView` gagne `enemy_visible` (aucun `Wall`, élargi de
`SHOT_MARGIN` = 4 px, entre le joueur et l'ennemi le plus proche), `enemy_still` (face à un
ennemi immobile au-delà de `STILL_TARGET_DISTANCE` = 120 px, `prudent` s'approche par `route`
sans recul) et `route` (pas suivant vers le poste de tir de l'ennemi le plus
proche par le chemin, ou vers le point accessible le plus proche de lui ; sans ennemi, vers le
portail, rayon 16 ; composante de moins de 2 px annulée, sinon un pas d'un pixel devient une
diagonale contre un coin). Règles : l'esquive et le recul sous 180 restent prioritaires ;
ennemi caché ou au-delà de la bande de `prudent` (ou caché pour `fonceur`) → `route` ; ennemi
visible → ligne droite comme avant ; portail au-delà de 48 → `route`, puis l'approche pilotée.
Sans chemin (`route` absente), ligne droite en repli. Le champ n'est calculé que quand une règle
s'en sert. Critère à la livraison : `throne` 20/20 sur 20 graines (contre 17/20), `trois_niveaux` du
testbed conservé — mesure invalidée depuis (les trois étages chargeaient alors `niveau_1`,
m1-integration-scenarios) ; mesures actuelles : journal de `docs/taches.md`
(m1-integration-scenarios, m1-v3-bots-portail).

**`alacod-sim`** : `--until-floor <n>` (exige `--floors`) arrête la graine quand
`FloorState::index >= n` ; `--until-wave` ou `--until-floor` est obligatoire. `SimResult` gagne
`floor_frames` (frame de chaque passage de portail), `damage_taken` (somme des baisses de santé
des joueurs) et `dodges` (frames où l'esquive a remplacé le déplacement d'au moins un bot,
compteur `bots::BotStats` hors rollback). Soft-lock `Floors` : sans passage de niveau ni ennemi
en moins pendant 1 200 frames, arrêt avec `softlock` (instantané « précédent » à 600 frames).
`--log` (D49) : installe un subscriber de journaux sur stderr (`RUST_LOG`, défaut `info`, sans
horodatage ni couleur) ; sans lui, `alacod-sim` n'a aucun subscriber et `RUST_LOG` ne produit
rien. Sorties et JSON inchangés sans `--log`.
D43/D44 (m1-d43-d44-fin-de-partie) : la simulation s'arrête aussi dès que la partie est terminée
(`StopEarly::stop_when_run_ended`, `run_end` dans le JSON, colonne « fin ») ; la progression du
soft-lock compte aussi la santé totale des ennemis qui baisse et tout changement d'état d'un
joueur (`runner::FloorsProgress`).
`scripts/scenario-metrics.py` : colonnes `Niveau` et `Frames/niveau`, `Esquives`, résumé
« niveaux finis / graines ».

**Scénarios** : `bot_prudent_dodge` (arena_tir), son jumeau `bot_prudent_nododge` (inputs de
`prudent` sans esquive figés en `Scripted`), `bot_floors_three` (`trois_niveaux` = `floor_a`,
`floor_d`, `floor_c`).

**Armes des bots** (m1-v3-bots-armes, `crates/bots/src/arms.rs`, mode `Floors`, `prudent` et
`fonceur`) : chaque arme portée est notée par ses dégâts attendus par seconde sur la cible
courante, lus dans la config du mode de tir : dégâts par projectile × projectiles par tir
(plombs, rafale) × cadence × part qui touche, nuls hors portée ou sans munitions. Part qui touche
= `min(1, 2 × rayon / (distance × spread))` (angle uniforme sur la pleine largeur `spread`, §16 ;
`spread_angle` pour le fusil). Exemples (cible de rayon 12, armes de `throne`) : à 300 px,
mitraillette ≈ 43/s, lance-lames 36, laser 48 ; à 500 px, mitraillette ≈ 26 (test
`arms::tests::degats_attendus_avec_la_dispersion`). Changement d'arme : seulement pour un gain
≥ 1,5 × (ou si l'arme en main ne fait rien), pas avant 120 frames après le choix précédent
(`WeaponChoices`, hors rollback, frame de simulation, vidée à chaque entrée en partie) ;
`switch_weapon` pressé jusqu'à l'emplacement visé. Pas de tir au-delà de la portée de l'arme en
main (`BotView::fire_range`). Ramassage, sans ennemi visible à moins de 150 px : power-up à moins
de 96 px (partout à sec), arme au sol à moins de 96 px si elle bat l'arme en main à 250 px
(emplacements pleins : c'est elle qui tombe) ou si un emplacement est libre. Pas de cas « boss »
ni d'anticipation (m1-v3-bots-lead).

## 25. Variantes et élites (T1.5, chantier D2)

Code : `crates/game/src/character/variant.rs` (`VariantsConfig`, `draw_variant`,
`variant_health`, composant `Variant`), application dans `character::enemy::create::spawn_enemy`.

**Contenu** : champ optionnel `variants` d'un personnage (`CharacterConfig`) :
```ron
variants: Some((
    chance: "0.5",   // probabilité d'avoir une variante, [0, 1], défaut 1
    table: {
        "rapide": (weight: 1, modifiers: [(stat: EnemyMoveSpeed, op: Mul, value: "1.5")],
                   tags: ["rapide"], skin: Some("rapide")),
        "blinde": (modifiers: [(stat: MaxHealth, op: Mul, value: "2.0")], tags: ["champion"]),
    },
)),
```
`weight` (`u32`, défaut 1) pondère le tirage, dans l'ordre `BTreeMap` des noms ; `modifiers`
au format §9 ; `tags` en **union** avec ceux du personnage (immunités §8, `ignore` de §22) ;
`skin` (clé de `skins`) remplace `starting_skin`. Une **élite** est une variante ordinaire (tag
`champion` + modificateurs) : aucun mécanisme à part, pas de points ×2.

**Tirage** : au `spawn_enemy` de tout personnage qui déclare `variants` (vagues, spawners et
`CharacterSpawn`), un `RollbackRng` **local** de graine `variant_seed(run_seed, net_id)` = `fnv1a("variants:<run_seed>:<net_id>")`
replié sur 32 bits (`hash ^ hash >> 32`, comme `loot_seed` §48 et `curse_seed` §64) puis passé au
finaliseur `fmix32` de MurmurHash3 (D88, M3 : l'ancien `fnv1a("variants") ^ run_seed ^ net_id` suivi d'un
seul pas de congruentiel était quasi linéaire, 0,0004 par unité de graine, donc corrélé par étage ; le repli
FNV seul laissait encore 0,07 de corrélation entre net ids voisins, le finaliseur la ramène à zéro ; `run_seed` =
`RngStreams::run_seed`, la graine de carte ; `MapGenerationConfig::seed` pour un `CharacterSpawn`, même valeur) :
un tirage pour la `chance`, un pour le poids. Preuve statistique (`variant::tests::tirage_independant_par_graine_et_net_id`,
graines 1 à 20 000 × 16 net ids, chance 0,1) : 18,4 % des graines sans aucun champion (indépendance : 0,9^16 = 18,5 % ;
avant : 89 %), corrélation de Pearson entre net ids voisins de −0,013 à +0,009 ; `graines_voisines_tirages_differents`. `alacod-sim` compte les variantes
rencontrées par graine (colonne `variantes`, moment clé `variant_spawn`) et en donne la part des graines concernées (D88). Indépendant de l'ordre d'apparition et du moment de chargement de
la carte ; **`RngStreams` n'est jamais touché** (y créer un flux changerait le hash de la
ressource, donc toutes les traces). Le `GgrsNetId` du personnage est alloué par `spawn_enemy`
avant `create_character` (même valeur que s'il l'était dedans : c'est sa seule allocation)
pour que le tirage et le skin soient connus à la création. Un personnage **sans** `variants`
ne tire rien et suit le chemin d'origine.

**Variante imposée** : champ LDtk optionnel `variant` sur `CharacterSpawn` (§1) : rempli,
aucun tirage, la variante est imposée ; vide ou absent, tirage normal.

**Application** (seulement si une variante est choisie) : composant rollback `Variant(nom)`,
checksum **neutre** (§20) ; `Modifiers` = ceux de la variante, source `Named("variant:<nom>")`,
permanents ; `Tags` = union ; skin à la création. **Santé** : un ennemi n'a pas de stat de base
`MaxHealth` (sa santé vient de F5, §18) : pour un personnage **à variantes** seulement,
`spawn_enemy` pose la base `MaxHealth` = santé F5 et crée le personnage à la santé résolue par
les modificateurs `MaxHealth` de la variante (`sync_health_from_stats` le maintient ensuite).
La vitesse d'un ennemi est `EnemyMoveSpeed` (`MoveSpeed` est celle des joueurs).

**Attente et événement** : `EnemyVariant(entity, variant: Some("rapide") | None, at_frame)` ;
moment clé `variant_spawn` (`scenario::events`, libellé `variante <nom> (<net id>)`).

**Lint** : `chance` hors `[0, 1]`, `weight = 0`, nom en double, `skin` absent de `skins`,
`MoveSpeed` sur un personnage à `ai` (renvoie à `EnemyMoveSpeed`), stat inconnue (erreur de
chargement), `variant` LDtk vers un personnage ou une variante inconnus (lecture des
`CharacterSpawn` des `.ldtk`). Fixtures `variant_weight_zero`, `variant_chance_out_of_range`,
`variant_skin_unknown`, `variant_move_speed_ai`, `variant_duplicate`, `variant_ldtk_unknown`.

**Testbed** : `grunt` (`Chase`, variantes `rapide` et `blinde`), `grunt_plain` (le même sans
table), carte `testbed/arena_variantes.ldtk` (grunt `rapide` imposé, grunt `blinde` imposé,
`grunt_plain`, grunt tiré) ; scénarios `variant_fast`, `variant_none`, `variant_elite` (même
partie, chacun son sujet : leurs trois traces sont identiques, c'est voulu) et `variant_draw`
(autre graine de carte). `games/zombies` : aucune variante.

## 26. Surfaces (T1.7, chantier E4 v1)

Code : `world::surface` (grille, table, traduction), `game::character::surface` (système),
`map_ldtk::game::collider::surface_grid_of_levels` (lecture LDtk), `map_ldtk::loader::surface_table`.

**Contenu.** Kind de dossier `Surface` (`content::registry::SurfaceEntry`) : un fichier RON par
surface, id = nom de fichier sans extension.
```ron
// games/testbed/assets/surfaces/eau.ron — game.ron : (path: "surfaces", kind: "Surface")
(intgrid_value: 1, tags: ["eau"], move_speed: "0.5")
// glace : (intgrid_value: 3, tags: ["glace"], move_speed: "1.0", acceleration: Some("0.2"))
```
`move_speed` et `acceleration` (défaut `1.0`) sont des **facteurs abstraits** (`ModifierOp::Mul`)
traduits selon le personnage : joueur → `MoveSpeed` et `Acceleration` (qui règle aussi son
freinage : l'adhérence, §34) ; ennemi au sol →
`EnemyMoveSpeed` (pas d'accélération) ; ennemi volant (`MovementType::Flying`) → rien. Un facteur
`1` ne pose aucun modificateur. Lint (`content::lint::lint_surfaces`) : `intgrid_value` > 0 et
unique (`DuplicateId`), facteurs > 0, tags non vides (fixtures `surface_duplicate_value`,
`surface_factor_non_positive`).

**Source v1 : LDtk.** Couche IntGrid optionnelle **`Surfaces`** d'une carte (valeur IntGrid =
`intgrid_value`, 0 = pas de surface), lue avec les murs (chargement et passage de niveau
`Floors`). Une carte sans la couche laisse la grille vide. Cavernes avec surfaces : v2.

**Grille.** `world::SurfaceGrid` : **creuse** (`BTreeMap<(i32, i32), SurfaceId>`), en cases de
grille monde de 16 (+y vers le haut, même découpage que `CellGrid` mais sans origine imposée :
une carte LDtk est placée n'importe où, alignée sur 16 depuis m0-v7). `SurfaceId` = valeur
IntGrid (`u8` non nul) ; `world::SurfaceTable` (hors rollback, posée au chargement depuis le
registre) donne la définition. Ressource rollback, checksum **neutre** (vide = 0) : traces des
cartes sans surface inchangées. Indépendante de `CellGrid` (une case de roche creusée garde sa
surface, ou n'en a aucune).

**Application** (`surface_modifiers_system`, `RollbackSystemSet::Input`, avant `apply_inputs`,
`order_mut_iter!`) : case sous les pieds = centre du collider avec son offset. Les
modificateurs de source `ModifierSource::Named("surface")` présents sont comparés aux
modificateurs voulus pour cette case ; s'ils diffèrent, `remove_by_source` puis pose, **sans
`until`** ; sinon rien n'est écrit. Effet dans la frame, aucun état nouveau (`Modifiers` est
déjà rollback). Carte sans surface : le système ne fait que retirer d'éventuels restes (joueur
venu d'un niveau `Floors` à surfaces) ; un personnage hors surface n'est jamais touché.

**Attente** `CellState(x, y, surface: "eau", at_frame)` : nom de la surface de la case (`"aucune"`
= sans surface). `kind` y est optionnel (au moins un des deux).

**Scénarios** (testbed) : `surface_walk` (couloir à bandes eau / sable / glace de
`testbed/surfaces.ldtk` : 1,25 et 2,0 px/frame contre 2,5), `surface_none` (couloir nu, même
input), `surface_ice` (glissade au demi-tour : 30 frames contre 6 sur le sol), `surface_enemy`
(`testbed/surfaces_enemy.ldtk` : un breacher qui traverse l'eau touche le joueur 205 frames
après celui du couloir nu).

**v2 / hors périmètre.** Dash par surface (il couvre la même distance sur la glace), freinage
réglé à part de l'accélération par surface (l'adhérence règle les deux), coût de flow field par surface,
dangers (piques, fosses, barils, feu), neige et boue (contenu 1837), surfaces de caverne
(`CaveConfig.surfaces`).

## 27. Effets v1, jauges et mutations (T1.10, chantiers C1 v1 et C4 v1)

Code : `effects::runtime` et `effects::progression` (règles pures, testées sans Bevy),
`game::effects_runtime` (porteurs, `apply_effects_system`), `game::progression` (rads, niveaux,
choix, drop d'arme), `content::lint::{lint_effect, lint_progression, lint_mutations}`.

**Effet** : `Effect { on, if, do }` (§4.3) ; un porteur a `Effects(Vec<Effect>)` et `EffectState`
(frame de pose de chaque effet, dernière frame touchée, déclencheurs reportés), rollback,
checksum **neutre**. Posés par `CharacterConfig::effects` (champ optionnel), par une mutation
prise, ou par `PlayerScript::mutations` ; un personnage sans effet n'a aucun composant (traces
existantes inchangées).

- **Déclencheurs v1** : `OnKill` (tueur = premier `HitBy` de `Death::last_hit_by`, joueur par son
  handle, entité — dont un émetteur — par son net id), `OnDamageTaken` (frame où les dégâts
  accumulés s'appliquent : `HealthRegen::last_damage_frame`, sinon un `DamageEvent` qui vise le
  porteur), `Tick(n)` (toutes les `n` frames depuis la pose, jamais à la pose), `OnGauge(id,
  Above(x))` (jauge franchie à la hausse, `avant < x ≤ après`, frame suivante), `OnLevelUp`
  (nouveau variant, voir plus bas). Les autres (`OnHit`, `OnDodge`, `OnReload`, `OnRoomClear`,
  `OnPickup`, `OnUse`, `OnEvent`) : lint `Unsupported` (« v2 »).
- **Conditions v1** (ET) : `HpBelow(x)` (fraction de `Health.max`), `HasTag(t)` (porteur),
  `TargetTag(t)` (victime d'`OnKill`, source d'`OnDamageTaken`), `NotHitFor(n)`. `Carrying`,
  `SquadSize`, `TargetInRange` : lint `Unsupported`.
- **Actions** : `Action` étendu (même enum que les power-ups, §14) de `Modifier { stat, op,
  value }` (permanent, source `Named("effect:<net_id>:<index>")`), `Heal(x)` (borné à
  `Health.max`, jamais sous la santé courante), `SpawnPattern { pattern, weapon }` (un `Emitter`
  T1.2 sur le porteur, table `projectiles` de `weapon` ; ignoré si un émetteur joue déjà),
  `GaugeAdd(id, x)`. `TimedModifier`/`CurrencyMultiplier` s'appliquent au **porteur seul**
  (la règle « tous les joueurs » est celle des power-ups). `RefillAmmo`, `RefillAmmoOf`, `RepairAllWindows`,
  `KillAllWaveEnemies`, `DestroyTerrain`, `ApplyStatus` : lint `Unsupported` dans un effet.

**Ordre des sets (constaté, non modifié).** Les `FrameEvents` sont vidés au début de chaque frame
et les morts sont détruits dans `DeathManagement` (`rollback_apply_death`, même frame) : un
système dans `Effects` (avant `DeathManagement`) ne verrait **jamais** les kills. D'où :
`apply_effects_system` dans `RollbackSystemSet::DeathManagement`, après
`rollback_apply_accumulated_damage` (dégâts appliqués, `Death` posés) et avant
`rollback_apply_bleedout`, puis `init_progression_players` → `progression_system` →
`weapon_drop_on_death_system` (après `loot_drop_on_death_system` : le flux `loot` sert d'abord
aux power-ups). Ordre interne : porteurs par `GgrsNetId`, puis index d'effet, puis ordre des `do`.
**La mort de la frame prime** : un porteur posé `Death` ou `Downed` dans la frame ne déclenche
rien et n'est pas soigné (test `la_mort_de_la_frame_prime_sur_le_soin`).

**Progression (C4 v1)** : kind `Progression` (`progression/<id>.ron`), **opt-in** : la partie joue
celle que nomme `Scenario::progression`, sinon `entry.progression` du manifeste, sinon aucune
(testbed et zombies : aucune ; flux `loot` intact).
```ron
// games/testbed/assets/progression/base.ron — game.ron : (path: "progression", kind: "Progression")
(gauge: "rads", per_kill: "1", levels: ["2", "4"], choices: 3, choice_frames: 600,
 mutations: [],  // pool ; vide = toutes les mutations du jeu
 weapon_pool: [(level: 0, weapons: ["pistol"]), (level: 1, weapons: ["shotgun"])],
 weapon_drop_chance: "0.0")
```
Chaque joueur reçoit `Gauges` (la jauge `gauge`, bornée à `[0, max]`), `Level`, `Mutations`
(rollback, neutres). Chaque mort attribuée au joueur ajoute `per_kill` ; niveau = nombre de seuils
`levels` atteints (les `GaugeAdd` comptent aussi). Chaque niveau gagné ouvre un choix
(`MutationChoice { options, since_frame, pending, armed }`, neutre) : `choices` mutations
**distinctes** tirées dans le flux `loot` (pondérées par `weight`, `max_stacks` respecté, joueurs
par `GgrsNetId`) ; un niveau gagné pendant un choix s'empile (`pending`). La simulation ne se met
**jamais** en pause. Choix par les bits d'input 13–15 (`INPUT_CHOICE_A/B/C`, boutons `ChoiceA/B/C`,
touches 1/2/3), seulement si aucun bouton de choix n'a été tenu depuis l'ouverture (`armed`) ;
sans choix, la première option est prise à `since_frame + choice_frames`. La mutation prise
ajoute ses effets, est notée dans `Mutations`, et **`OnLevelUp` se déclenche à la frame
suivante** (pour tous les effets `OnLevelUp` du joueur, ceux de la nouvelle mutation compris).
Pool épuisé : pas de choix, `OnLevelUp` directement. Moments clés `levelup`, `mutation`.

`choice_timing: Immediate` est le défaut. `BetweenFloors` (revue humaine M1) conserve
les niveaux gagnés pendant le combat dans `PendingMutations` (rollback neutre, absent
quand vide), puis ouvre les choix après nettoyage de l'étage et ouverture du portail.
Le portail attend que tous les joueurs vivants aient résolu leurs choix ; les projectiles
hostiles restants sont supprimés et les dégâts des joueurs sont ignorés pendant cet
interlude. Le timeout par choix reste actif, les horloges continuent. Le tirage `loot`
est différé avec le choix, donc ses options et les combats suivants peuvent changer.
Cette politique ne s'applique qu'au mode `Floors` ; les autres modes gardent le choix
immédiat. `throne` l'active dans `progression/run.ron`.

**Mutation** : kind `Mutation` (`mutations/<id>.ron`) : `(name, weight: 1, tags, max_stacks: 1,
effects: [Effect])`.

**Drop d'arme** : chaque ennemi mort (ordre `GgrsNetId`) tire `weapon_drop_chance` (flux `loot`) ;
gagné, une arme parmi celles du `weapon_pool` dont `level ≤` niveau **max** des joueurs (niveau
mis à jour dans la même frame), posée par `spawn_weapon_pickup` (chargeur plein, gratuite). Chance
0 : aucun tirage.

**Lint** : déclencheur/condition/action `Unsupported`, `Tick(0)`, `Heal ≤ 0`, pattern ou arme de
`SpawnPattern` inconnus, jauge d'`OnGauge`/`GaugeAdd` qui n'est celle d'aucune progression ;
progression : `per_kill ≤ 0`, `levels` vides ou non strictement croissants, `choices` hors
`[1, pool]`, `choice_frames` 0, chance hors `[0, 1]`, mutation ou arme inconnue ; mutation :
`weight` 0, `max_stacks` 0, `effects` vide ; `entry.progression` vers une progression inconnue
(audit T1.12). Fixtures `effect_unsupported`, `effect_out_of_range`, `effect_broken_reference`,
`progression_out_of_range`, `progression_broken_reference`, `mutation_out_of_range`,
`entry_progression_unknown`.

**Scénarios** : attentes `Gauge(handle, id, min, max, at_frame)`, `Level(handle, level,
at_frame)`, `Mutations(handle, contains, count, at_frame)` ; champs `Scenario::progression` et
`PlayerScript::mutations`. Testbed : `effect_on_damage_taken` / `effect_none` (`pilote` contre
`dummy` : 92 contre 72 points), `effect_on_kill` (`vampire`), `effect_tick` (`tireur`, couronnes
f122/f242/f362), `levelup_choice` (`ChoiceC` → `coriace`), `levelup_timeout` (première option à
f704), `weapon_pool_drop` (progression `armes`).

**Hors périmètre** (écran de choix et HUD des rads livrés depuis : §30, §32) : déclencheurs et conditions v2,
objets passifs/actifs (M2), modificateurs de projectiles (M3, `isaac`), pause, `--progression` dans
`alacod-sim`.

## 28. Générateur v1 et placement scripté (T1.13, voie V3)

Code : `crates/scenario/src/generate.rs` (gabarits), `scenario::runner::
apply_scenario_character_placements` (placement), `map_ldtk::game::local::spawn_character`
(chemin commun avec les `CharacterSpawn` de carte), `game::character::config::CharacterTest`.

**Placement scripté.** `Scenario.characters: [(character: "grunt", x: "120", y: "-40",
at_frame: 5, variant: Some("blinde"), team: Some(Allies))]` (`variant`, `team` optionnels ;
coordonnées monde en chaînes `Fixed`). À `frame == at_frame`, dans l'ordre de déclaration, le
personnage est créé par **le même chemin qu'un `CharacterSpawn` de carte** : net id alloué à
cette frame, santé F5 × difficulté (§23), variante imposée ou tirée (§25), équipe du placement,
sinon `CharacterConfig.team`, sinon `Enemies`. Système dans `GgrsSchedule`,
`RollbackSystemSet::EnemySpawning` (comme un spawn de vague, rejoué à l'identique par un
rollback), avec une condition d'exécution : sans placement, il ne tourne pas, aucune trace
existante ne change. Le réenregistrement le conserve (`RecordedSettings::characters`). C'est
l'outil qui remplace « une arène LDtk par ennemi » ; les arènes de T1.4/T1.5 restent.

**`EntityRef::Placed(n)`** : le personnage créé par le placement d'index `n`. Le runner le
retrouve par un composant d'observation `ScriptedPlacement(n)` posé à la création, **hors
rollback** (non enregistré : ni checksum ni trace), reposé quand un rollback rejoue le
placement. Absent avant `at_frame` (une attente qui le vise échoue).

**`test:` d'un personnage** (opt-in, aucun effet en jeu) : `test: Some((frames: 600, still:
true, moving: true, expect_still: [...], expect_moving: [...]))` (`still`/`moving` vrais par
défaut). `make gen` écrit un scénario par gabarit actif :
`tests/scenarios/generated/<jeu>/enemy_<id>_still.ron` et `enemy_<id>_moving.ron`.

- **Gabarits** : `EnemyVsStillPlayer` (le joueur, armes de départ, ne fait rien) et
  `EnemyVsMovingPlayer` (il marche en carré, 90 frames par côté, en boucle, sans tirer). L'ennemi
  est placé à 200 unités à droite du **spawn du joueur** à la frame 1 ; le spawn est lu par une
  sonde d'une frame (`discover_enemy_probe`).
- **Attentes par défaut** (liste vide) : immobile → moment clé `Event(kind: "hit", "joueur 0
  touché")` avant la dernière frame (cumulatif : la santé du joueur se régénère, la fiche
  proposait `Health` à la dernière frame, qui ne prouvait rien — constaté sur `charger`) et
  `EnemyNeverInWall(Placed(0))` ; mobile →
  `PlayerAlive` à la dernière frame et `EnemyNeverInWall(Placed(0))`. Une liste explicite
  **remplace** les attentes par défaut de son gabarit (viser l'ennemi : `Placed(0)`).
- **Où** : les gabarits ennemis jouent dans le jeu énuméré lui-même (le registre du testbed n'a
  pas les personnages des autres jeux) : `testbed/arena.ldtk` pour le testbed,
  `exemples/test_map.ldtk` pour `zombies`, sinon `entry.start_map`. Jeu en mode `Waves` :
  `wave_overrides.grace_period_frames = frames + 1`, aucune vague ne s'ajoute à l'ennemi testé.

**`generate_template`** (manifeste, ajout de l'orchestrateur pour `games/throne`) :
`generate_template: (map: "maps/salle.ldtk", target: "cible")`. S'il est présent, les gabarits
d'armes **et** d'ennemis du jeu jouent dans ce jeu, sur `map` ; sinon le testbed comme avant
(`zombies` et `testbed` ne déclarent rien : leurs scénarios générés ne changent pas). Les inputs
du gabarit d'arme (visée, marche de la mêlée) sont réglés sur l'arène du testbed : la carte place
la cible (`target`, `counts_hits: true`) au même décalage du spawn du joueur, +128 et −48 en
coordonnées LDtk. Lint : carte connue, cible connue avec `counts_hits: true`
(`generate_template_unknown_map`, `generate_template_target_without_hits`).

**`Scenario.mode`** (suite de T1.13, `m1-dettes-lot-1`) : `mode: Some(Sandbox | Waves |
Floors)` l'emporte sur `entry.mode` du manifeste (le runner remplace `entry.mode` avant de
l'insérer) ; absent, comportement d'avant. `Floors` exige `floors` (échec du runner sinon). Le
générateur écrit `mode: Sandbox` dans les gabarits d'un jeu qui déclare `generate_template` (un
jeu en `Floors`, comme `throne`, jouerait sinon sa séquence au lieu de la carte du gabarit) ; les
générés de `zombies` et `testbed` n'ont pas le champ.

**Hors périmètre** : gabarits par statut (T1.3 mergée depuis, §19 ; ces gabarits ne sont pas écrits), vidéos des gabarits,
génération de cartes, bots dans les gabarits.

## 29. Le jeu `throne` (T1.0c + T1.11, voie V2)

`games/throne/` : le clone, **données seulement** (aucun code d'engine propre : `src/main.rs` est
celui du testbed renommé). Membre du workspace, `make throne` (fenêtré), `make lint` le linte avec
zombies et testbed. Une run = trois étages de caverne générés (mode `Floors`), sans vagues.

**Manifeste** : `entry: (start_map: "cave:niveau_1", default_seed: 123456, mode: Floors)` ;
séquence `floors/run.ron` = `cave:niveau_1`, `cave:niveau_2`, `cave:niveau_3` (même graine pour la
séquence, §17 et §21). Kinds : `Character`, `Weapon`, `MeleeWeapon`, `Pattern`, `Cave`, `Floors`,
`PowerUp`, `Economy`, `Ui`, `Camera`, `SpriteSheet`, `Map` (gabarit des scénarios générés),
`Progression`, `Mutation`, `Clock`, `Difficulty` (phase 2).

**Joueur** : `characters/pilote.ron` déclare le personnage d'id `player` (le moteur crée tous les
joueurs depuis lui), copie de `player` du testbed ; trois armes de départ (`mitraillette`,
`revolver`, `lance_lames`).

**Armes** (`weapons/weapons.ron`) : douze armes du joueur sur cinq munitions `Custom` —
`balles` (`revolver`, `mitraillette`), `obus` (`fusil_a_pompe`, `canon_ricochet` : `Bounce`),
`explosifs` (`lance_grenades` : `Bounce`+`Pierce`+`Lifetime`, explosion puis couronne d'éclats ;
`roquette` : souffle en `on_expire` ; `mortier` : `Gravity`, éventail de fragments), `energie`
(`laser` : `Pierce` ; `plasma` : `Size`+`Lifetime` ; `traqueur` : `Homing`), `lames`
(`lance_lames` : `Bounce`+`Pierce` ; `disque` : rafale, `Pierce`+`Lifetime`). Leurs `spread`
(mitraillette 0,15, disque 0,05, revolver 0,0005, les autres 0) ne sont appliqués que depuis D51
(§16) : avant, toutes tiraient à ±0,5 rad ; aucune valeur n'a été recalée à cette occasion. Plus `arsenal`,
l'arme des ennemis (sa table `projectiles` : `crachat`, `plomb`, `boule`). Mêlée
(`weapons/melee.ron`) : `bare_hands` (joueur), `griffes`, `crocs`, `massue`. Chaque arme à distance porte un
`test:` (jauges calées sur la mesure ; les armes de mêlée sont générées sans). **Gabarits** (§28, m1-throne-gen-et-d40) :
`generate_template: (map: "gabarit_armes.ldtk", target: "cible")` — `make gen GAME=throne` joue
les armes et les onze ennemis (dont `roi_rat` ; `test:` sur chacun, gabarits immobile et mobile) dans `throne`, sur
la carte gabarit (`cible` et `mannequin`, même disposition que l'arène du testbed), en mode
`Sandbox` imposé ; scénarios versionnés sous `tests/scenarios/generated/throne/`.

**Ennemis** (`characters/`, behaviors §22, variantes §25 ; aucun sprite) : mêlée `rat`
(`Melee("griffes")`, variante `rapide`), `chien` (rapide et fragile, variantes `rapide` et
`blinde`), `brute` (`massue`, variante `blinde`), `rodeur` (`Wander` hors de vue, `Sight(300)`) ;
tireurs `cracheur` (pattern `crachat` : `Aimed`), `arroseur` (`eventail` : `Spread`), `tourelle`
(immobile, `couronne` : `Ring` après télégraphe) ; chargeur `buffle` (`Charge`) ; kiter
`franc_tireur` (`KeepDistance` + `visee`) ; fuyard `pillard` (`Flee` sous 50 %).

**Cavernes** (`caves/`, gabarit `caves/gabarit.ldtk`, tilesets `atlas/`) : `niveau_1` 48 × 32,
4 ennemis (`rat`, `chien`, `rodeur`) ; `niveau_2` 56 × 40, 6 ennemis (premiers tireurs, chargeur,
fuyard) ; `niveau_3` 64 × 44, 8 ennemis (tous les profils). `fill_ratio` 0,38 (plus ouvert que le
testbed : les bots `prudent` vont en ligne droite). Pas de surface (impossible dans une caverne
générée, §26 v2).

**Boss** (m1-integration-scenarios) : `roi_rat`, en données seulement (360 PV, tag `champion`, `[Shoot(couronne), Charge, Chase]`, sans phases : D4, M2 ; corps de 20 × 20 avant `scale` 1.4, soit 28 px en jeu : gabarit `Large` de la navigation, D41 fermée par m1-navigation-profils-tailles), premier de `niveau_3.characters` avec `enemy_spawns` 9 : un exemplaire au point 0 (le plus éloigné du départ, toujours présent) ; la tourelle, seul ennemi fixe, est dernière (point le plus proche, ou absente quand la caverne n'a que huit points) ; le portail exige la mort du boss comme celle des autres, et la boucle infinie du dernier niveau en recrée un à chaque rechargement. Chaque caverne est un asset distinct (`cave://<dossier>/<id>.ldtk`, §21 ; avant, les trois étages chargeaient `niveau_1`).

**Butin** (`items/powerups.ron`, §14) : `munitions_balles`, `munitions_obus`,
`munitions_explosifs`, `munitions_energie`, `munitions_lames` (`RefillAmmoOf`, D40, poids 36,
12, 12, 12 et 24), `rage` (dégâts ×2) et `vitesse` (×1,3) (poids 25), `drop_chance` **0,40** (0,25 avant D50 : pénurie de munitions prouvée sur la graine 76, soft-lock des bots ; décision de William du 2026-10-14 : relever la chance, sans pondération par les réserves). Pas de `weapon_pool`.

**Assets** : tous des **placeholders** copiés du testbed (sprites du joueur et des armes, slash,
sons, police, tilesets), enregistrés dans `games/throne/assets/assets.yaml` (licences reprises de
zombies, `statut: placeholder`).

**Scénarios** : `throne_floor_1` (un bot `prudent` finit l'étage 1), `throne_three_floors` (deux
bots finissent la run), `throne_solo`, `throne_quad`, `throne_duo_defaite`, `throne_ammo_pickup`,
joués avec tout ce que le manifeste active (butin compris). Mesure : `alacod-sim --game throne
--bots 2 --profiles prudent,prudent --floors run --seeds 1..20 --until-floor 3 --max-frames
12000`. Les chiffres des phases 1 et 2 (19/20, 17/20) portaient sur trois fois `niveau_1` et
sont caducs ; mesures actuelles : journal de `docs/taches.md` (m1-integration-scenarios,
m1-v3-bots-portail).

**Phase 2 — progression, mutations, horloge, difficulté** (T1.9, T1.10 ; actives par le manifeste :
`entry: (..., progression: "run", clocks: ["etage"], difficulty: true)`, donc en partie jouée
**et** dans tous les scénarios de `throne`) :
- `progression/run.ron` (§27) : un rad par kill, niveaux à 3, 8 et 15 rads, trois mutations
  proposées, la première prise d'office après 600 frames (les bots ne choisissent pas) ;
  `weapon_pool` : munitions de départ au niveau 0, obus et lames au 1, énergie au 2, explosifs au
  3 ; `weapon_drop_chance` 0,08.
- Huit mutations (`mutations/`, effets v1 seulement) : `coriace` (OnLevelUp : +25 MaxHealth,
  soin 25), `vampire` (OnKill : soin 4), `tireur` (Tick(180) : salve de six éclats, pattern
  `salve`), `adrenaline` (touché sous 40 % : vitesse ×1,4 pendant 3 s), `rancune` (touché :
  dégâts ×1,5 pendant 2 s), `sang_froid` (Tick(60) sans coup depuis 5 s : soin 3), `chasseur`
  (tuer un champion : dégâts ×2 pendant 5 s), `irradie` (OnKill : un rad de plus).
- Horloge `clocks/etage.ron` (§23, portée `Floor`) : `alerte` à 15 s, `renfort` à 30 s puis
  toutes les 15 s. **Aucune action n'y est attachée** (`OnEvent` des effets est v2) : moments clés
  et attente `Clock` seulement.
- `difficulty.ron` : `1 + floor * 0.25 + floor_minutes * 0.25` (santé des ennemis à l'apparition,
  dégâts qu'ils infligent).
- Scénarios : `throne_progression` (niveau, rads, mutation prise d'office, horloge),
  `throne_mutation_choice` (inputs enregistrés d'un bot, `ChoiceB` → `chasseur`).
- Gabarit des scénarios générés : `gabarit_armes.ldtk` (copie de l'arène du testbed, `cible` à
  `counts_hits` à +128/−48 du spawn, `mannequin` ailleurs), joué en `Sandbox` imposé (voir
  **Gabarits** plus haut).

## 30. Écran de mutation et transition (T1.16, voie V4)

Présentation seule : aucune trace ne change, la simulation ne fait jamais de pause.

- **Modèle de vue** `game::ui::mutation_screen::MutationScreenView` (ressource `Update`, hors
  rollback, aussi remplie en headless par `MutationScreenModelPlugin`) : dérivé en lecture seule
  du `MutationChoice` du joueur affiché (`CameraFollowOverride`, sinon le joueur local de plus
  petit handle) : `open`, `options` (id, nom `Mutation.name`, description), `frames_left`
  (`since_frame + choice_frames − frame`), `total_frames`, `highlighted`. Fonctions pures
  testées : `build_view`, `apply_navigation`, `confirm_bit`, `confirm_input`.
- **Description** `effects::describe::describe_effect` : une phrase française par effet,
  « <déclencheur>[, <conditions>] : <actions>. » (« Quand vous êtes touché, sous 40 % de vie :
  vitesse × 1,4 pendant 3 s. »), virgule décimale, durées en secondes. Aucun champ de contenu
  en plus ; une variante nouvelle de `On`/`Condition`/`Action` ajoute sa phrase (le `match` est
  exhaustif).
- **Entrées** : ←/→ (flèches, D-pad) déplacent la surbrillance (état UI local, en boucle) ; un
  bouton `ChoiceA/B/C` tenu (1/2/3 ou script) la pose ; un nouveau choix la remet à 0. Valider
  (Entrée, A) ajoute à l'input du joueur affiché le bit de la carte surlignée
  (`read_local_inputs`) : seul chemin vers la simulation, le même que 1/2/3. Le D-pad et → gardent
  aussi leur action de déplacement (la partie continue pendant le choix).
- **Mise en page** (D71, §56) : gabarit `bevy_hui` `ui/mutation_screen.html` (throne, testbed),
  rempli par `game::ui::mutation_hui` depuis `MutationScreenView`. Remplace l'ancien
  `ui/mutation_screen.ron` et son schéma de lint (retirés). Absent : pas d'écran (le choix se
  fait aux touches 1/2/3 ou d'office).
- **Transition de niveau** `game::ui::floor_transition` : à chaque changement de
  `FloorState::index` lu côté présentation (pas `FloorEntered`), voile noir qui s'éclaircit
  linéairement en `floor_fade_seconds` (`camera.ron`, défaut 0,4) et caméra posée sur sa cible
  pendant le fondu (`floor_recenter`, défaut vrai). La première lecture ne déclenche rien.

## 31. Feedback v1 (T1.17, voie V4)

**Présentation seule.** Rien n'est rollback, rien n'entre dans le checksum ni dans les traces ;
la simulation ne lit jamais ces composants ou ressources. Le hit stop **ne fige jamais la
simulation** (rollback, p2p) : il gèle l'avance des animations et le suivi de caméra pendant
`n` ticks de **rendu** (`animation::AnimationFreeze`, un tick par `Update`, avancé en `First`) ;
`Time<Virtual>` n'est jamais touché (ggrs avance sur ce temps).

**`ui/feedback.ron`** (type partagé `content::feedback::FeedbackSettings`, enveloppé par l'asset
`game::feedback::FeedbackConfig`, lu tel quel par le lint) : `hit_flash` (frames, couleur :
multiplicateur linéaire des calques, au-delà de 1.0 le sprite est surexposé, `(1, 1, 1)` est
invisible), `shake` (frames, amplitude), `hit_stop_frames` (0 : aucun), `damage_numbers`,
`telegraph.color` (RGBA), `by_kind: {DamageKind: …}` et `by_weapon: {"arme": …}` (surcharges
`hit_stop_frames`/`hit_flash`/`shake`, champs optionnels), `sounds`. Résolution d'un coup
(`FeedbackSettings::resolve`), champ par champ : arme, puis genre, puis valeur globale.
Lint : frames > 0 (flash, secousse, surcharges comprises), amplitude ≥ 0, couleur du flash dans
`[0, 4]`, couleur du télégraphe dans `[0, 1]`, armes de `by_weapon` déclarées par le jeu (à
distance ou de corps à corps), sons présents.

**Déclencheurs** (par frame simulée) : flash sur la cible de chaque `DamageEvent` et chiffre si
`damage_numbers` ; au plus **une** secousse (cible = joueur local, ou tireur = joueur local quand
la secousse vient d'une surcharge : impact lourd) et **un** hit stop (source ou cible = joueur
local) par frame, le plus fort des coups de la frame.

**Limite connue : l'arme d'un coup.** `DamageEvent` ne porte pas l'arme (il est tracé : lui
ajouter un champ ferait bouger toutes les traces). L'arme retenue pour `by_weapon` est
l'émetteur de la source (`Emitter.weapon`) s'il existe, sinon **l'arme active** de son
inventaire au moment du coup (une balle tirée avant un changement d'arme est attribuée à la
nouvelle) ; aucune pour la mêlée (tag `melee`). Les projectiles infligent aujourd'hui des
dégâts `Physical` (`Fire` pour la brûlure) : `by_kind` sert surtout aux genres futurs, les
explosifs se règlent par `by_weapon`.

**Flash** : posé sur l'entité racine (`HitFlash`), appliqué aux **calques enfants** par
`apply_layer_colors` (même parcours que la teinte des statuts, §19 ; le flash l'emporte tant
qu'il dure). Avant T1.17, le flash cherchait un `Sprite` sur la racine, qui n'en a pas : il
n'était jamais visible. La secousse retire son décalage du tick précédent avant d'appliquer le
suivant (sans ça, les décalages s'additionnent quand la caméra ne suit plus, pendant un hit
stop), et décroît sur sa durée.

**Télégraphe au sol** (`telegraph_shape`, lecture seule de l'état) : phase `Telegraph` d'une
`Charge` (T1.4) → cercle au point visé figé, rayon `attack_range` ; pas `Telegraph` d'un
émetteur (`Emitter::telegraphing`, T1.2) → cercle autour du porteur, rayon la plus grande
portée de sa table de projectiles. Contour au rayon complet, disque intérieur qui grandit
jusqu'au déclenchement (gizmos).

**Preuve sans écran** : `game::feedback_log::FeedbackLog` (ressource de présentation, plugin
`FeedbackLogPlugin` actif aussi en headless ; depuis D69 E5b le journal est la moitié **simulation** de
l'ancien `feedback.rs` : formes, fonctions pures, `FeedbackConfigLoaded`, sans asset ; le chargeur de
`ui/feedback.ron` est `feedback::FeedbackAssetsPlugin`, côté application) reçoit les indices `{genre, frame, net_id,
position, valeur}` de chaque **nouvelle** frame simulée (`FrameCount` plus grand que la
dernière frame journalisée : les frames rejouées par un rollback ne sont pas journalisées deux
fois). Limites (présentation, sans conséquence) : un tick qui simule plusieurs frames ne voit
que les événements de la dernière ; un coup prédit puis annulé par un rollback p2p reste
journalisé. `scenario::events` en fait des moments clés `feedback` (`<genre> <net_id> (…)` :
« télégraphe 27 (29 frames, rayon 40) », « hit stop 22 (2 frames) », « secousse 22 (8 frames) » ;
flash et chiffre, un par coup, n'en font pas) ; attente `Event(kind: "feedback",
label_contains: …)`. Scénarios : `enemy_charge` (télégraphe de la ruée vu à f42, hit stop et
secousse au contact à f100), `weapon_grenade` généré (`test.expect` de la grenade : secousse et
hit stop de surcharge à la première explosion, f113). Ces attentes ne touchent pas aux traces.

Hors périmètre : sons (D32), particules, écran de mort.

## 32. HUD throne (T1.18, voie V4)

Présentation seule : aucune trace ne change.

- **Liste fermée** `content::ui::HUD_SOURCES` : `health`, `wave`, `ammo`, `weapon`, `enemies`,
  `players`, `currency`, `perks`, `downed`, `powerups`, `prompt`, et (T1.18) `rads`, `level`,
  `ammo_by_type`, `statuses`, `floor`, puis (M2-T18, §56) `blanks`, `keys`, `shells`,
  `passives`, `active_item`, `active_charge`, `minimap`. Le lint lit la source de chaque
  widget de `ui/hud.ron` (`HudFileSchema`) : source inconnue = `UnknownKind`
  (fixture `hud_unknown_source`).
- **Modèle** `game::ui::hud_model` : `hud_values(&HudPlayerInput) -> HudPlayerValues` (pure,
  testée sans rendu) et `player_source_text(source, prefix, values)` ; `HudSnapshot { frame,
  texts, bars, values }` mis à jour en `Update` par `HudModelPlugin`, **aussi en headless** ; le
  rendu (`ui::hud`) lit le snapshot. Joueur affiché : `CameraFollowOverride`, sinon le joueur
  local de plus petit handle.
- **Textes** : `rads` « 5 / 8 rads » (jauge `gauge` de la progression active / seuil du prochain
  niveau ; « 12 rads » au dernier niveau), barre = (valeur − seuil courant) / (seuil suivant −
  seuil courant), pleine au dernier ; `level` « Niv. 2 » ; `ammo_by_type` une ligne par munition
  `Custom` (« balles 120 », ordre des ids) ; `statuses` une ligne par statut dans l'ordre de pose
  (« brulure ×2 · 3 s », secondes arrondies au-dessus) ; `floor` « Étage 3 » (index + 1, mode
  `Floors` seulement). Sans valeur : texte vide, pas même le préfixe (D22).
- **Attente** `HudText { source, contains, at_frame }` : le texte (sans préfixe) contient
  `contains` ; lue dans `HudSnapshot`, hors simulation et hors trace.
- **Contenu après revue humaine (2026-10-14)** : `games/throne/assets/ui/hud.ron` —
  étage en haut à gauche ; vie, niveau et barres vie/rads en bas à gauche ; arme et
  chargeur/réserve en bas à droite ; statuts/power-ups contextuels en haut. Les textes
  joueurs/ennemis, rads numériques et réserves des armes inactives sont retirés du HUD
  courant. Les sources restent disponibles dans le modèle. Mutation/fin masquent le HUD.
  Les RON HUD de zombies et testbed restent inchangés.

### Polissage de revue M1 (2026-10-14)

Le rendu des cartes raccourcit les descriptions générées (« regagnez 25 points de vie »
→ « +25 PV »), en conservant déclencheurs, conditions, quantités et durées. Le modèle
`MutationScreenView` garde la description complète. L'écran de fin Floors affiche
étage, éliminations et durée min:sec plutôt que frames/score technique ; panneau centré.
`play_scenario --capture <dir> --size 640x360` vérifie les petits viewports ; défaut 960x540.
Ce sont des changements de présentation, aucune mécanique ni trace modifiée.

## 33. Restart en ligne (D14)

« Rejouer » à la fin d'une partie p2p (chemin `--matchbox`) relance une partie avec les mêmes
pairs, sans saisie de lobby et **sans message réseau** : l'accord vient du déterminisme
(`game::jjrs::restart`).

- **Compteur** `OnlineGames` (hors rollback) : parties **en ligne** jouées depuis la dernière
  entrée normale dans `LobbyOnline` (remis à 0 à cette entrée, +1 à `OnEnter(InGame)` en ligne).
  Les parties locales jouées avant la connexion ne comptent pas : les pairs ont donc le même
  compteur.
- **Rejouer** (`RunRequest::Restart` en ligne, `allumette_url` vide) : pas d'abandon (comme le
  restart local), `OnlineRestart { game: n }` puis `LobbyOnline`. Le socket de la partie est
  fermé à `OnExit(InGame)` ; le nouveau s'ouvre sur la salle `{lobby}-r{n}` (jamais la salle
  d'origine : aucun pair fantôme) ; `wait_for_players` et la création de session sont ceux du
  premier lancement.
- **Graine** de la partie relancée : `restart_seed(seed, n)` (`seed ^ fnv1a("restart") ^ n`,
  replié sur 32 bits) depuis la graine de carte ; le restart **local** garde la même graine.
- **Désaccord** : sans les pairs dans la salle de restart au bout de 30 s
  (`RESTART_TIMEOUT_SECS`), retour à la salle `{lobby}` d'origine et compteur à 0, comme
  « Lobby ». Le départ d'un pair après la fin de partie n'affiche plus « GAME DISCONNECTED »
  (l'écran de fin reste) ; l'overlay de déconnexion disparaît à la sortie de la partie.
- **Allumette** (`--allumette`) : pas en v1 (l'API joint un lobby par `game_id`, pas par nom ; le
  créateur bloque jusqu'à 60 s) : « Rejouer » y reste redirigé vers le lobby (dette).
- **Preuve** : `ALACOD_RESTART_AT_FRAME=r` (avec `ALACOD_STATE_TRACE`/`ALACOD_EXIT_AT_FRAME`) :
  trace de la partie 1 écrite dans `<fichier>-g1` à la frame `r`, `RunRequest::Restart` posé
  60 frames plus tard, partie 2 écrite dans `<fichier>-g2` à `ALACOD_EXIT_AT_FRAME` (compté depuis
  son début). `scripts/p2p-restart.sh` : deux clients, traces identiques partie par partie, partie 2
  différente de la 1 (graine dérivée) ; un client local dont les deux parties sont identiques
  (même graine : la sortie de partie remet tout à zéro).

## 34. Course et esquive (dash)

Code : `game::character::movement` (`MovementConfig`, `run_velocity`, `approach_axis`, `grip`),
`combat::actors::DashState` et `dash_step` (règles pures, testées sans Bevy),
`game::character::player::input::apply_inputs` (visée, dash puis course, un seul système).
Réglage des joueurs : `player_config.ron` (zombies, testbed), `characters/pilote.ron` (throne).
Les ennemis ne lisent que `max_speed` (leur IA fait le reste) ; le champ `friction` n'existe plus.

**Course.** Chaque axe va vers la vitesse visée (direction des touches normalisée × `MoveSpeed`
résolue × sprint ; nulle sans touche) : il accélère de `acceleration` (px/s²) vers une cible plus
rapide dans le même sens, freine de `deceleration` (px/s², absent = `acceleration`) vers une cible
plus lente, nulle ou opposée, sans jamais la dépasser. Par axe plutôt qu'en norme : arithmétique
`Fixed` exacte, sans racine, et un demi-tour sur un axe ne freine pas l'autre ; relâcher un axe le
freine même si l'autre est tenu. Réglage (150 px/s, 3000/3000) : pleine vitesse (2,5 px par frame)
en 3 frames, arrêt en 3, demi-tour en 6. Tenir une touche `n` frames déplace de **2,5 × `n` px**
(l'accélération et le freinage se compensent) : c'est la règle de calcul des positions attendues.

**Adhérence.** `grip` = stat `Acceleration` résolue / `movement.acceleration` (1 sans
modificateur) multiplie aussi le freinage : la glace (§26, ×0,2) met 15 frames à lancer ou arrêter
le joueur, 30 au demi-tour.

**Dash.** Part sur l'appui (front montant de `Dash` : tenir le bouton ne relance rien) et avance
dès cette frame ; direction : touches, sinon visée, sinon regard, inversée par `Modifier`. Il
couvre exactement `dash_distance` px en `dash_duration_frames` frames, par un pas qui décroît
linéairement de `2·d/n − course` jusqu'à la vitesse de course (`dash_step`) : on sort du dash en
courant, sans arrêt sec ni glissade ; sans touche à la sortie, la course freine (+2,5 px). Le pas
est une vitesse appliquée par `move_characters` : un dash s'arrête aux murs. `dash_cooldown_frames`
est compté depuis le départ. Un appui refusé (dash ou cooldown en cours) est gardé
`dash_buffer_frames` frames, celle de l'appui comprise, et part à la première frame permise.
Pendant le dash : pas de tir (le bouton de dash tenu, lui, ne bloque plus rien), aucun
ralentissement par les ennemis (on traverse la horde). À terre, étourdi ou gelé (T1.3, §19) : pas
de dash, pas même d'un appui gardé d'avant le statut (il vieillit et s'oublie). Un dash en cours
quand le statut tombe ne bouge plus (vitesse annulée par `status_motion_system`), mais ses frames,
ses i-frames et son cooldown s'écoulent. Réglage : 64 px en 8 frames (13,5 px la première),
cooldown 24, i-frames 6, buffer 8.

**I-frames.** Les `dash_iframes` premières frames du dash, celle de l'appui comprise :
`Health.invulnerable_until_frame` (déjà rollback) prend le maximum de sa valeur et de
`frame + dash_iframes − 1`. `Health::is_invulnerable_at(frame)` est le seul test : balles et
projectiles composables **traversent** le joueur (ni dégât ni arrêt, `Pierce` non consommé) ; les
hitbox de mêlée ne le voient pas (ni dégât ni recul, et il n'est pas marqué touché : une hitbox
encore active après les i-frames le touche) ; tout autre dégât est annulé au résolveur
(`rollback_resolve_damage_events`, sauf `DamageKind::True`). `Homing` et `Aimed` le visent
toujours.

**Scénarios** : `dash_wall` (64 px, puis butée contre le mur), `movement_melee` (sprint, dash
dans la direction des touches, appui refusé pendant le cooldown, appui gardé qui part à la fin du
cooldown), `surface_ice` (demi-tour sur la glace et sur le sol), `stat_move_speed`,
`dash_iframes` (un joueur plaqué contre un mur dashe dedans sous un tir ami continu, une balle
toutes les 2 frames : `NoDamageBetween` sur les 6 i-frames, coups juste avant et juste après ;
`dash_iframes: 0` le fait échouer, 3 coups de plus), `dash_stun_buffer` (un appui gardé pendant le
cooldown ne part pas une fois le joueur étourdi : la balle perçante tirée à bout portant le touche
à la frame où le dash fautif l'aurait rendu invulnérable ; sans la garde, 10 PV de plus),
`dash_aim_change` (la visée change pendant le dash, synctest).

**Recalage des scénarios (2026-10-04).** La course a changé toutes les trajectoires : positions
attendues recalculées (`at_frame N` = état après N frames, segments `from..to` à fin exclue, délai
d'input de 5 frames, 2,5 px par frame de touche), marches raccourcies quand elles visaient un
interactable (achats, réparation), générateur recalé (`WALK_*_FRAMES`). La visée des parties de
référence du clone (`clone_solo`, `clone_duo`, `clone_quad`) a été **ré-enregistrée** en headless à
partir de f1000 (outil temporaire, non versionné) : toutes les 15 frames, chaque joueur concerné
tire vers le zombie vivant le plus proche (égalité : plus petit `GgrsNetId`) s'il est à 400 px au
plus (joueur 3 du quad : 1000), pendant 15 frames (solo) ou 2 (duo, quad) ; `pan` = écart monde
arrondi, +y vers le haut. Mêmes attentes qualitatives (vagues, kills, victoire, joueurs debout),
soldes et frames recalés. Les bots joués en direct (`bot:`) gardent leurs scénarios, frames
recalées (portails, niveaux, mutations) ; ceux dont les inputs sont des enregistrements de bot figés
en `Scripted` ont été **réenregistrés** par la même commande `alacod-sim --save-scenario`
(`throne_mutation_choice`, avec `ChoiceB` réinséré à la frame de son choix ; `bot_prudent_nododge`,
avec le même diff temporaire qui désactive l'esquive).

## 35. Salles typées et verrouillées (M2-E1, chantier E1)

Voie B du prototype `m1-proto-etage-salles` : **un étage = un monde LDtk assemblé de salles**
(l'assembleur `Basic`, inchangé), chaque salle étant un gabarit typé. Code : contrats dans la
crate `world` (`world::rooms`, aucune dépendance au rendu), branchement ECS dans
`map_ldtk::game::rooms` (portes, bornes de salle), contenu dans `content::registry`
(`RoomEntry`) et `content::lint` (`lint_room_kinds`).

**Type de salle (liste ouverte).** Champ de niveau LDtk `room_kind` (chaîne) d'un gabarit, recopié
tel quel par l'assembleur (`fieldInstances` du gabarit) et posé en `world::RoomKind` sur l'entité
du niveau (`add_level_components.rs`, à côté de `RoomBounds`/`LevelId`). Les types sont les
fichiers d'un dossier de contenu de kind `Room` (`rooms/<id>.ron`, id = nom de fichier) :
```ron
// games/testbed/assets/rooms/combat.ron  — game.ron : (path: "rooms", kind: "Room")
(locks: true)   // verrouille et endort ses ennemis ; `locks: false` (défaut) : un lieu comme un autre
```
Types du testbed : `depart`, `combat` (`locks: true`), `recompense`. `boss`, `boutique`… s'ajoutent
par un fichier, sans code. Une carte sans `room_kind` n'a aucune salle typée : **aucun système du
module ne fait rien** (traces existantes inchangées).

**Lint.** `RoomTemplate` : le `room_kind` d'un niveau d'une carte du dossier `Map` doit être un
type chargé ; inconnu → `UnknownKind` (« RoomTemplate « SalleX » : room_kind « y » inconnu »).
Fixture `room_kind_unknown`.

**État rollback.** `world::RoomStates { doors_opened, rooms: BTreeMap<LevelId, RoomState> }`,
`RoomState { Dormant, Locked, Cleared }` (une salle absente de la table est `Dormant`), ressource
**neutre** (`rollback_and_trace_resource_neutral` : vide hors carte typée, contribution 0).
`world::RoomDormant` (marqueur sur les ennemis d'une salle dormante) : composant neutre.
`world::RoomChanged` (`FrameEvents`, neutre) : un changement d'état (frame, salle, type, avant,
après), pour les récompenses `OnRoomClear` à venir. `RoomKindTable` (types du jeu) et `DoorShape`
(collider de porte pour la refermer) sont statiques, hors rollback.

**Comportement** (`room_lock_system`, `RollbackSystemSet::Run`, après la transition d'étage ;
`room_dormancy_system`, `EnemySpawning`, avant l'IA) :
1. Au premier tick d'une carte typée, les portes posées au bord (à moins de 24 px) d'une salle
   typée s'ouvrent (collider et `Interactable` retirés) : une porte de salle est ouverte par défaut.
   **En `RunMode::Waves`, seules les salles verrouillantes (`locks: true`) s'ouvrent ainsi** ; les
   portes des salles non verrouillantes restent fermées et payantes (la progression économique se
   fait en amont, la porte de combat du boss reste gratuite). Autres modes (`Floors`, `Sandbox`,
   sans `Run`) : toutes les salles typées s'ouvrent comme avant.
2. Une salle dont le type verrouille passe `Dormant → Locked` quand un joueur **debout** est à plus
   de 24 px de ses bords et qu'un ennemi y vit : ses portes et leurs jumelles (au bord de la salle
   voisine) se referment (le champ de flux les voit comme des murs) ; puis `Locked → Cleared` quand
   plus aucun ennemi n'y vit : elles se rouvrent pour de bon.
3. **Jamais sur un occupant** : si un joueur ou un ennemi chevauche l'une des portes à refermer,
   aucune ne se ferme ; le verrouillage est réévalué à chaque frame (différé, trace
   `room_lock_deferred`).
4. **Coop** : au verrouillage, les joueurs hors de la salle (debout ou à terre) sont téléportés
   à l'entrée : 40 px dans la salle devant la porte la plus proche du joueur qui a déclenché le
   verrouillage (tangente du bord, 14 px d'écart, `GgrsNetId` croissant), vitesse annulée.
5. **Ennemis dormants** : tant qu'une salle verrouillante est `Dormant`, ses ennemis portent
   `RoomDormant` ; sélection de cible, comportements, déplacement et attaque
   (`enemy_target_selection`, `behavior_select_system`, `update_enemy_targets`, `move_enemies`,
   `enemy_attack_system`) les sautent (`Without<RoomDormant>`). Annuler leur vitesse ne suffisait
   pas : `move_enemies` écrit la position.

**Limites connues (E2 et suite).** Les portes d'une salle sont repérées par leur position (marge
de 24 px), pas par un lien explicite ; deux salles qui se touchent par un coin partagent
d'éventuelles portes voisines. Le mode `Floors` ne remet pas `RoomStates` à zéro au changement
d'étage. Pas de récompense, de minicarte ni de grammaire d'étage (E2).

**Scénarios et attentes.** `RoomState(kind: "combat", nth: 0, state: Locked, doors_closed:
Some(4), at_frame: 100)` : état de la `nth` salle du type (triées par position x puis y) et,
optionnel, nombre de portes fermées de la carte. Moment clé `room` (« salle combat verrouillée »,
« … nettoyée »). Cartes `testbed/salles.ldtk` (deux `dummy`) et `salles_poursuivants.ldtk` (deux
`follower`), produites par `docs/taches/rapports/m2-e1-salles.make_salles.py` depuis
`two_rooms_door.ldtk` ; scénarios `rooms_lock_solo`, `rooms_dormant`, `rooms_lock_coop`,
`rooms_door_occupied`. Diagnostic : `ALACOD_MAP_PROBE=<scénario>:<frame>` affiche aussi les salles
(état) et les ennemis (position, dormant).

D68 : si un ennemi chevauche une porte ouverte, une salle dormante réveille ses ennemis dès
qu'un joueur debout y entre (même marge `PLAYER_INSET` que le verrouillage). La porte
reste ouverte tant qu'un corps la chevauche. Un ennemi endormi sur une porte ne peut ainsi plus
empêcher son propre réveil. La dormance est recalculée avant l'IA à chaque frame, sans état caché.

## 36. Objets et inventaire (M2-T0b, chantier C2)

Contrats : crate `items` (simulation, sans rendu) ; runtime : `game::items` ; contenu :
`content::registry::ItemEntry` et `content::lint::lint_items`.

**Contenu.** Kind de dossier `Item` : un fichier RON par objet (`objets/<id>.ron`, id = nom de
fichier), champs de `items::ItemDef` :
```ron
(name: "Bottes", kind: Passive,                       // Passive | Active(charge) | Consumable
 modifiers: [(stat: MoveSpeed, op: Pct, value: "0.25")],   // permanents, source `item:<id>`
 effects: [],                                          // effets T1.10 tels quels (voir ci-dessous)
 tags: ["mobilite"], rarity: Common,                  // Common | Uncommon | Rare | Epic | Legendary
 pickup_range: "24.0")                                 // défaut 24
// actif : kind: Active(Rooms(1)) | Active(Damage(200)) | Active(Frames(600))
//   effects: [(on: OnUse, do: [TimedModifier(stat: Damage, op: Pct, value: "1.0", frames: 300)])]
```
`ActiveCharge` : `Rooms(n)` salles nettoyées (`world::RoomChanged` vers `Cleared`, §35),
`Damage(n)` dégâts infligés par le porteur (`DamageEvent`, partie entière), `Frames(n)`. L'actif est
prêt quand sa charge atteint `n` (plafonnée).

**Lint.** Id dupliqué (`DuplicateId`), RON ou stat inconnue (`Parse`), charge 0 et `pickup_range`
≤ 0 (`OutOfRange`), consommable avec modificateurs ou effets et tout effet autre que `OnUse` d'un
actif avec `TimedModifier`/`Modifier` (`Unsupported` : contrats des effets v2, M2-T3).

**Inventaire.** `items::Inventory { passives (ordre de ramassage), active: Option<ActiveSlot
{ item, charge }>, consumables: BTreeMap<id, n> }`, composant rollback **neutre**, posé au
**premier ramassage** (jamais sur un joueur qui n'a rien ramassé : contribution 0, traces
inchangées). `ItemPickup { item_id }` : objet au sol (rollback, neutre, `GgrsNetId`). `ItemTable`
(statique, hors rollback) est posée au chargement depuis le registre
(`map_ldtk::loader::item_table`). `ItemPicked` : `FrameEvents` neutre (moment clé `item_pickup`).

**Comportement** (`game::items`). Consommable : ramassé au contact (portée de l'objet), le plus petit
`GgrsNetId` à portée l'emporte. Passif/actif : `Interactable { Item }` (`InteractionType::Item`), ramassé
par Interaction ; un passif ajoute ses modificateurs à `Modifiers` ; un actif remplace l'actif tenu
(qui tombe aux pieds, charge perdue) avec une charge 0. Charge (`item_charge_system`, après `Run` : lit
les `RoomChanged` de la frame). Usage : bit `INPUT_USE_ACTIVE` avec l'actif chargé : les effets `OnUse`
s'appliquent, la charge retombe à 0.

**Input élargi (u16 → u32).** `BoxInput.buttons` passe de 16 à 32 bits (`BoxInput` : 8 → 12 octets en
mémoire, alignement 4) : `INPUT_USE_ACTIVE` (bit 16, touche Espace, bouton `UseActive`) et `INPUT_BLANK`
(bit 17, touche Q, bouton `Blank` ; contrat seul, l'effet vient de M2-T1). Les enregistrements ne
stockent que des noms de boutons : ils se rejouent à l'identique.

**Scénarios.** `items: [(id: "bottes", x: "-904.0", y: "-584.0", at_frame: 5)]` place un objet à une
position et une frame exactes (comme `powerups`). Attentes : `HasItem(handle, id)`,
`ItemCharge(handle, charge)` (échoue sans actif), `Consumable(handle, id, count)`. Moment clé
`item_pickup`. Scénarios : `item_passive`, `item_active_rooms` (carte de M2-E1), `item_consumable_race`,
`item_passive_race`. Objets du testbed : `bottes`, `fiole` (`Active(Rooms(1))`), `key`.
---

## 37. Boss à phases (M2-T0c, chantier D4)

Contrats : `behaviors::boss` (`BossDef`, `Phase`, `PhaseEnd`, `Timeline`, `BossState`,
`BossPhaseChanged`, fonctions pures `BossDef::next_phase` et `Timeline::due`) ; exécution :
`game::boss` ; contenu : champ `boss` d'un personnage RON ; lint : `content::lint::lint_boss`.

```ron
boss: Some((phases: [
    (until: Some(HealthBelow("0.5")),            // HealthBelow(ratio ]0, 1]) | AfterFrames(n > 0)
     behaviors: [Chase(profile: "Ground")],       // remplace les règles de l'ennemi (vide : inchangées)
     on_enter: [Heal("5.0")],                     // effects::Action, à l'entrée (y compris la phase 0)
     timeline: Some((events: [(at: 10, do: [SpawnPattern(pattern: "volee", weapon: "fireball_gun")])],
                     repeat: Some(150)))),        // frise rejouée toutes les 150 frames ; None : une fois
    (timeline: Some((events: [(at: 0, do: [SpawnPattern(pattern: "couronne", weapon: "fireball_gun")])],
                     repeat: Some(90)))),         // dernière phase : pas de `until`
]))
```
Les options d'un personnage s'écrivent `Some(...)` (pas d'`implicit_some`).

**Transitions.** `boss_phase_system` (`RollbackSystemSet::EnemySpawning`, après les dégâts de la
frame, avant l'IA) date l'entrée dans la phase 0 au premier tick, puis ne fait **qu'une transition
par frame**, dans l'ordre du RON : `HealthBelow` quand santé/santé max est **strictement** sous le
ratio, `AfterFrames` quand `frame - entrée >= n`. À l'entrée : `on_enter` puis, à chaque frame,
les événements dus de la frise. La frise n'a pas de curseur : `due(frame - entrée)` (modulo
`repeat`) se déduit de l'état, donc identique après un rollback. Trace
`ggrs{f=… boss_phase net_id=… from=… to=…}` ; `FrameEvents<BossPhaseChanged>` (neutre).

**Règles par phase.** `EnemyBehaviors::phase_rules` (statique) garde la liste de chaque phase ; l'IA
lit `EnemyBehaviors::active(BossState::phase)` (sélection, déplacement imposé, attaque,
`melee_hold`, attente `EnemyState`). **Limite levée par M2-T7 (§45)** : la règle `Shoot` d'une phase
porte sa propre table de tir (arme, pattern, portée, recharge) ; sans `Shoot` de phase, `ai.ranged` de
la config ; un tir peut aussi passer par la frise (`SpawnPattern`).

**Actions.** Même exécuteur que les effets de personnage (`effects_runtime::apply_action`) avec un
`EffectState` jetable : `GaugeAdd` n'a pas d'effet durable sur un boss ; `SpawnPattern` est ignoré
(avertissement) tant qu'un émetteur joue déjà. Le lint refuse en plus `RefillAmmo*`,
`RepairAllWindows`, `KillAllWaveEnemies`, `DestroyTerrain`, `ApplyStatus`, `CurrencyMultiplier`.

**État.** `BossState { phase, entered, started }` : composant rollback, checksum et trace en
variante **neutre** (seuls les boss en portent). `BossPlan` (la définition) est statique.

**Lint.** Au moins une phase ; seule la dernière peut ne pas avoir de `until` ; `HealthBelow` dans
`]0, 1]`, `AfterFrames` > 0 ; phase non vide ; frise : `repeat` > 0, `at` < `repeat`, événements non
vides ; règles et `SpawnPattern` (pattern et arme connus) comme dans un personnage. Fixtures
`boss_health_out_of_range`, `boss_phase_empty`, `boss_missing_until`, `boss_unknown_pattern`,
`boss_timeline_after_repeat`.

**Scénarios.** Attente `BossPhase(entity: Placed(0), phase: 1, at_frame: 100)` (échoue sans
`BossState` ou si l'entité est morte) ; moment clé `boss_phase` (« boss 36 : phase 0 → 1 »).
Personnage `testbed/assets/characters/gardien.ron` (120 PV, immobile : volée `volee` puis couronne
sous 50 %), scénario `boss_two_phases` (phase 1 à f90, mort à f133).

---

## 38. Profil de méta-progression (M2-T0c, chantier G1)

Crate `meta` (hors simulation, dépend de `run`) et glue `game::profile`. Décision §7 n° 8 : RON
versionné, un profil par joueur, écrit hors simulation.

```ron
(version: 1, id: "local-0", currencies: {"essence": 11}, unlocks: ["first_run", "first_victory"],
 stats: {"runs": 1, "kills": 0, "frames": 50, "wave_best": 1, "floor_best": 0, "victories": 1})
```
**Format.** `meta::Profile { version, id, currencies, unlocks, stats }` (`BTreeMap`/`BTreeSet`).
`Profile::from_ron` migre par `version` (v0 = sans champ : devient v1 ; version plus récente que le
jeu : `ProfileError::TooNew`, fichier non touché). Écriture atomique (`meta::save` : fichier
temporaire puis renommage) ; un fichier illisible est signalé et jamais écrasé.

**Dossier.** `ALACOD_PROFILE_DIR`, sinon `$XDG_DATA_HOME/alacod/profiles`, sinon
`~/.local/share/alacod/profiles` ; en headless (`ALACOD_HEADLESS`) rien n'est écrit sans
`ALACOD_PROFILE_DIR`. Identifiant : `local-<handle>` (la pubkey allumette n'est pas encore connue
ici).

**Écriture.** `game::profile::write_profiles_at_run_end` (`Update`, jamais `GgrsSchedule`) : quand
`Run` passe `Ended` avec un `RunSummary`, `Profile::apply_run` crédite chaque joueur local (sans
`LocalPlayer`, tous les joueurs de la run) puis le fichier est écrit, une fois par run. Règles
provisoires (M2-T12) : stats cumulées (`runs`, `kills`, `frames`, `victories`, meilleures
`wave_best`/`floor_best`), essence = 1 par run + 1 par kill + 10 par victoire, déblocages
`first_run`/`first_victory`. Lecture au lancement dans la ressource `meta::Profiles` ; **la
simulation ne la lit jamais** dans cette tâche.

**Scénarios.** Un scénario n'écrit rien, sauf `profile: true` (dossier temporaire propre au
scénario). Attente `ProfileHas(handle: 0, unlock: "first_run", currency: ("essence", 11),
at_frame: 60)` (déblocage et/ou monnaie minimale ; évaluée après la fin de run). Scénario
`profile_run_end` (`idle` de zombies avec `wave_overrides: (max_wave: 1)` : victoire immédiate).

---

## 39. Le jeu `gungeon` (M2-T0d, squelette)

Dossier `games/gungeon/` (binaire `gungeon`, `make gungeon`, copié de `throne` puis élagué),
manifeste `assets/game.ron`. Il montre les contrats de la vague 0 de M2 ensemble : salles typées
(§35), objets (§36), boss (§37) ; le profil (§38) s'écrit à la fin de la run.

**Contenu.**
- `characters/` : `pistolero` (le joueur, id moteur `player`, course nerveuse et roulade à i-frames
  du §34), `bullet_kin` (ennemi tireur : `Shoot` du pattern `visee`, `KeepDistance`, `Chase`),
  `gatling` (boss immobile ; trois phases depuis M2-T7, §45 : éventail, spirale, rage avec renfort), `cible` (cible des scénarios
  générés d'armes).
- `weapons/weapons.ron` : `pistolet` (le moteur n'a pas de munition infinie : réserve de 9999
  chargeurs de 12, rechargement rapide ; `Automatic` : tenir la gâchette tire en continu) et
  `arsenal` (projectiles des patterns ennemis) ; `patterns/` : `visee` (`Aimed`), `couronne`.
- `objets/` (kind `Item`) : `bottes` (passif), `fiole` (actif `Rooms(2)`), `key` et `blank`
  (consommables, compteurs seulement). `rooms/` (kind `Room`) : `depart`, `combat` et `boss`
  (verrouillantes : `locks: true`).
- `maps/etage_1.ldtk` : l'étage, trois salles `Depart`, `Combat` (deux `bullet_kin`) et `Boss`
  (`gatling`) assemblées par `Basic` (voie B), produit par
  `docs/taches/rapports/m2-t0d-squelette-gungeon.make_etage.py` (qui appelle le script de M2-E1) ;
  `maps/gabarit_armes.ldtk` : gabarit des scénarios générés.
- `floors/etage_1.ron` : séquence `Floors` à un étage avec **`victory_at_end: true`** (nouveau champ
  optionnel, faux par défaut : throne et le testbed bouclent toujours sur le dernier niveau) : le
  dernier niveau vidé de ses ennemis (le boss est le dernier) termine la partie par une victoire
  (`floors_victory`, `FloorPlan::victory_at_end`, `floor_portal_open_system`), au lieu d'ouvrir un
  portail. Défaite à la mort (universelle).
- Sprites, sons, interface et caméra : placeholders copiés de `throne` (`assets.yaml`), aucun asset
  nouveau ; le butin de power-ups est désactivé (`drop_chance: "0.0"`).

**Scénarios.** `gungeon_start` (le clone démarre : joueur vivant, trois salles dormantes, trois
ennemis), `gungeon_clear` (bottes par Interaction, clé au contact, verrouillage à f75, nettoyage à
f231, réouverture), `gungeon_boss` (salle du boss verrouillée f291, phase 1 f448, boss mort f568,
victoire f569), plus les scénarios générés de `make gen GAME=gungeon`
(`tests/scenarios/generated/gungeon/`). Un joueur scripté vise des positions mesurées (les
`bullet_kin` se cornent à l'est de leur salle par `KeepDistance`) ; les bots ne conviennent pas
encore (conçus pour les zombies et leurs fenêtres).

**Vérifications.** `make lint` couvre `games/gungeon` ; le test `content/tests/embedded.rs` compare
registre natif et registre embarqué pour `zombies`, `throne` et `gungeon` ; le jeu n'est pas encore
dans les scripts de build web (`zombies` et `throne` seulement).

## 41. Projectiles v2 et blank (M2-T1, chantier B5 v2)

Code : `combat::projectile::Pattern` (variantes), `combat::emitter` (état multi-salves),
`items::{BlankSpec, BlankUsed}`, `game::items::{blank_system, BlankState, FloorStartConsumables}`,
`content::lint` (règles), contenu `gungeon` et `testbed`.

**Patterns ajoutés** (en **fin** d'enum `Pattern` : le hash des variantes d'avant ne bouge pas) :

| Variante | Effet | Où |
|---|---|---|
| `Fan(count, spread, speed, projectile)` | `count` tirs répartis **exactement** sur `spread` radians (bornes comprises), centrés sur la visée, sans hasard (à la différence de `Scatter`) ; `speed` > 0 remplace celle du projectile, 0 = celle de la définition | émetteur **et** `on_expire` |
| `Spiral(count, step, shots, every, speed, projectile)` | `shots` salves espacées de `every` frames, chacune de `count` bras régulièrement répartis sur le cercle ; la salve `k` tourne de `k × step` radians par rapport à la visée figée | émetteur seulement |
| `Burst(count, per_burst, interval, spread, projectile)` | `count` salves de `per_burst` tirs en éventail de `spread`, espacées de `interval` frames | émetteur seulement |

Une étape multi-salves (`Spiral`, `Burst`) **bloque la séquence** comme un `Wait` : la salve 0 part tout de
suite, la suivante `every`/`interval` frames plus tard, l'étape suivante après la dernière salve (une salve
de fond `Ring.every` continue de tirer). L'état d'avancement est `Emitter::multi` (`MultiVolley { step,
fired }`), dans l'état rollback de l'émetteur ; il n'est **haché que s'il existe** (hacher un `None`
déplacerait le checksum de tous les émetteurs d'avant ; test `spirale_ne_change_pas_le_hash…`). Aucun hasard :
le flux RNG `patterns` n'est pas consommé. Frames mesurées (`pattern_spiral`) : émetteur posé à f6, télégraphe
de 10 frames, salves à f17, f23, … (une salve toutes les `every` frames).

**Lint.** `count`, `per_burst`, `shots` > 0 ; `every` > 0 (`Spiral`) et `interval` > 0 (`Burst`) ; `spread`,
`speed` ≥ 0 ; `Spiral`/`Burst` refusés en `on_expire` (même au travers d'un `Named`), `Fan` admis. Fixtures
`pattern_spiral_on_expire`, `pattern_burst_interval_zero`.

**Balles ennemies qui rebondissent.** `Bounce(n)` joue pour un projectile de n'importe quelle équipe (la
chaîne `projectile_collision_system` ne regarde pas l'équipe pour les murs) : aucun correctif nécessaire,
prouvé par `enemy_bullet_bounce`. `gungeon` : `salve` de `bullet_kin` tire des `ricochet` (`Bounce(2)`).

**Blank.** Le consommable `blank` porte `blank: Some((radius: "140.0", iframes: 10))` (`items::BlankSpec`
dans `objets/blank.ron`, lint : consommable seulement, `radius` > 0). Au **front montant** de `Blank`
(`INPUT_BLANK`, touche Q), `blank_system` (`RollbackSystemSet::Effects`, après l'usage d'actif) :
1. consomme un exemplaire du premier objet porteur de `blank` que le joueur possède (ordre des ids) ;
2. efface (`despawn_rollback`) toute balle dont l'équipe de tir n'est ni celle du joueur ni
   `Allies`/`Neutral`, dans `radius` du joueur (jamais celles des joueurs) ;
3. rend le joueur invulnérable `iframes` frames (`Health::invulnerable_until_frame`, comme le dash) ;
4. émet `FrameEvents<BlankUsed { frame, player_handle, x, y, radius, cleared }>` (neutre ; lu par la
   présentation et par `OnBlank` de M2-T3) et la trace `ggrs{f=… blank net_id=… item=… cleared=…}`.
Sans blank en inventaire (ou sans inventaire) : rien. **Mémoire du front montant** :
`BlankState { held, floor }`, composant rollback **neutre**, posé sur un joueur qui a un `Inventory`
**et** un motif d'en avoir (Blank appuyé, blank en poche, ou recharge d'étage dans le contenu) : un joueur
avec inventaire mais sans blank (objets du testbed) ne le porte pas, donc sa trace ne bouge pas (une
première version le posait sur tout inventaire et déplaçait 4 traces du testbed). Réécrit à chaque frame
**avant** tout retour anticipé (piège « donnée dérivée non rollbackée »). Seuls `gungeon` et les scénarios
qui donnent un blank en portent : les traces des autres jeux ne bougent pas.

**Piège des tables de projectiles.** Les `projectiles:` d'une arme sont dans l'état hashé de ses porteurs :
ajouter un projectile à une arme existante déplace les traces de tous ses porteurs (une première version
l'avait fait sur `fireball_gun` : 33 traces). Les tourelles de motifs du testbed ont donc leur propre arme
`tourelle_gun` (copie de `fireball_gun` + `brume`, `ricochet`).

**Preuve p2p.** `ALACOD_PRESS_BLANK_AT=<frame>` (`player/input.rs`, `press_blank_at`) ajoute `INPUT_BLANK` à
l'input neutre de cette frame : permet de jouer un blank dans un test p2p sans clavier
(`ALACOD_INPUT=neutral`).

**Consommables de départ et d'étage (données).** `CharacterConfig::starting_consumables: {"blank": 2}`
crée l'`Inventory` du joueur à la création (vide par défaut : aucun inventaire). `Floors` :
`consumables_per_floor: {"blank": 2}` dans `floors/<id>.ron` remonte chaque compteur **à au moins** `n` au
début de chaque étage suivant (ressource hors rollback `FloorStartConsumables`, posée par `map_ldtk` au
chargement ; `BlankState::floor` retient l'étage servi). Lint : id consommable connu (`BrokenReference`,
fixture `character_starting_consumable_unknown`).

**Présentation.** `game::items_view::BlankPresentationPlugin` (`PresentationPlugin`, absent en headless) : onde (cercle
qui s'étend jusqu'au rayon puis s'estompe, 24 frames) dérivée de `BlankUsed` dans `Update` (ressource
`LastBlank`, jamais lue par la simulation). Moment clé `blank` (scénarios, vidéos). HUD : pas de source
« compteur de blanks » dans `HUD_SOURCES` : à M2-T18.

**Scénarios.** Testbed (`arena.ldtk`, tourelles `tourelle_spirale|eventail|salve|ricochet|anneau`) :
`pattern_spiral` (salves f17-f47), `pattern_fan` (5 flèches f17), `pattern_burst` (2 flèches à f17, f25, f33),
`enemy_bullet_bounce` (rebond sur le mur sud vers f63), `blank_clears_bullets` (12 boules ennemies effacées
à f36, 2 balles du joueur gardées, compteur 2 → 1), `blank_empty` (sans blank, rien), `bench_bullets_room`
(535 balles ennemies vivantes, salle verrouillée, plancher `budgets.ron`). Piège de script : **des segments
d'input qui se chevauchent ne se combinent pas** (le premier gagne) : couper les segments (`Fire` seul,
puis `Fire, Blank` une frame, puis `Fire`). Diagnostic : test ignoré `bullet_lines`
(`ALACOD_BULLET_LINES=<scénario>:<de>:<à>` : balles, joueurs, inventaire, santé frame par frame).
`gungeon` : `bullet_kin` joue `salve`, le boss `gatling` `eventail` (phase 0) puis `spirale` (phase 1,
projectile `etincelle`), le pistolero a deux blanks (`gungeon_boss` en utilise deux contre la spirale).

---

## 40. Grammaire d'étage (M2-T10, chantier E2)

Un étage de salles typées, assemblé à partir de gabarits LDtk. Code : `world::FloorGrammar`
(donnée), `map::generation::floor` (planificateur), `map::generation::imp::floor`
(`FloorMapGeneration`, mode `MapGenerationMode::Floor(grammaire)` à côté de `Basic` et `Cave`).
`Basic` est inchangé (traces `zombies`/testbed identiques).

**Contenu.** Kind de dossier `FloorGrammar` : un fichier RON par grammaire, id = nom de fichier,
désignée comme carte par `floor:<id>` (`entry.start_map`, `Scenario.map`, `levels` d'une séquence
`Floors`) :

```ron
// floor_grammars/etage.ron — game.ron : (path: "floor_grammars", kind: "FloorGrammar")
(templates: "maps/salles.ldtk", grammar: (
    rooms: (8, 12),                                    // départ, boss et requises compris
    required: {"boss": 1, "boutique": 1, "recompense": 1},
    min_boss_distance: 3,                              // en salles
    // start: "depart", boss: "boss", filler: "combat", max_attempts: 64, loops: 0 (défauts)
))
```

`templates` est la carte LDtk dont chaque niveau est un gabarit, typé par son champ de niveau
`room_kind` (§35). Le départ est un gabarit `spawn: true`.

**Algorithme** (déterministe, `RollbackRng` de génération ; listes ordonnées, aucun `HashMap`) :
départ au centre ; remplissage par croissance aléatoire (tirage uniforme parmi les poses valides :
parent, connexion libre, gabarit compatible du bon type, sans chevauchement ni sortie de carte) ;
boss posé sur une salle de profondeur maximale, avec un gabarit à **une seule connexion**
(cul-de-sac, donc strictement le plus loin du départ) ; salles requises sur des salles de
profondeur ≤ celle du boss − 2 ; contrôle indépendant `validate_plan`. Échec : nouvelle tentative
avec une graine dérivée (`next_u32` du générateur principal), au plus `max_attempts`, puis
`FloorError::Exhausted` explicite (jamais de boucle infinie ; `NoTemplate` si un type n'a aucun
gabarit).

**Mesure** (1 000 graines, gabarits 10×10 tuiles du test) : génération moyenne 184 µs, max 11,8 ms,
1,05 tentative en moyenne, tailles 8 à 12 réparties uniformément.

**Boucles (`loops`, 2026-10-19).** Sans ce champ, l'étage est un arbre : la fermeture des boucles
(`close_loops`, §1) ne relie que les prises libres qui se touchent par hasard (0,40 boucle par
graine sur 40 graines de `la_fonderie`, 15/40 graines avec au moins une). Mesure du
2026-10-19 : sur 190 prises murées (20 graines), 156 donnent sur du vide et 34 sur une voisine
sans prise en face, aucune paire libre face à face n'est oubliée ; c'est le placement qui étale
l'arbre. Avec `loops: n`, tant que le plan a moins de `n` paires de prises libres face à face,
une salle de remplissage qui en crée une est préférée à toute autre ; le plan est refusé sous
`n` (nouvelle tentative). Les raccourcis comptent : le boss doit rester **strictement** la salle
la plus loin et à `min_boss_distance` au moins, **boucles comprises** (distance en largeur sur
l'arbre + les paires). Un poids proportionnel (essayé : `1 + biais × paires`) ne change presque
rien (0,40 → 0,55) : ces poses n'existent qu'une à trois fois par plan. `loops: 0` garde le
tirage et la consommation du RNG d'origine (traces gungeon inchangées). `la_fonderie` :
`loops: 2` exige `rooms: (10, 12)` (avec 9 à 11 salles, 2 boucles échouent sur la plupart des
graines) ; 200 graines : 2,11 boucles en moyenne, aucune sous 2.

**Lint** : `rooms` min > max ou < 2, `max_attempts` 0, `required` à 0 ou contenant le départ,
plus d'un boss, départ + boss + requises > max, `min_boss_distance` impossible (`OutOfRange`) ;
carte `templates` absente, type nécessaire (départ, boss, remplissage, requis) sans niveau de ce
`room_kind`, séquence `Floors` ou `start_map` vers une grammaire inconnue (`BrokenReference`).
Fixtures : `floor_bounds_inverted`, `floor_required_without_template`.

**Étapes 2 : `Floors` d'étages assemblés.** Une séquence `Floors` peut lister des `floor:<id>`
(`levels: ["floor:etage", "floor:etage"]`). Chaque entrée est **son propre monde** (emplacement) :

- *Un chemin d'asset par étage* : `floor://<emplacement>/<gabarits>` (`game::cave_assets`, comme
  `cave://` §21), lu physiquement dans le fichier des gabarits ; sans cela l'`AssetServer` rendrait
  le premier étage chargé à tous (`FloorPlan::slot_of_level` ne fusionne pas les entrées `floor:`).
- *Graine dérivée par emplacement* : `graine_de_la_run + emplacement × 0x3C6E_F372` (wrapping) ;
  l'emplacement 0 garde la graine de la run. Les ids réseau sont triés par contenu comme partout.
- *Ouvertures libres murées* : depuis le 2026-10-17, règle commune à `Basic` et aux étages
  (`MapSettings::wall_unused_openings`, par prise et sans porte, §1) ; avant, propre aux étages
  (`GeneratedMap::close_unused_openings`, par côté).
- *Salles de l'étage courant seulement* : les mondes de tous les emplacements existent dès le
  départ et se superposent dans l'espace ; `rooms::CurrentRooms` ne présente aux systèmes de salle
  que l'emplacement de l'étage en cours. Sans cela les salles de l'étage 2 se verrouillaient et se
  vidaient pendant l'étage 1. Les portes du nouvel étage sont rouvertes à la transition
  (`RoomStates::doors_opened` remis à faux).
- Portail à la mort du boss (barycentre des `PlayerSpawn`, donc au centre de la salle de départ),
  `victory_at_end` sur le dernier étage.

**Piège : la séquence par défaut.** `entry.mode: Floors` prend la **première séquence par ordre
d'id** du dossier `Floors` (`run_state::resolve_run_mode_with_floors`). Ajouter un fichier qui
passe avant l'ancien change la partie par défaut : les séquences de grammaire de `gungeon` sont
nommées `etages_*` pour que `etage_1` (T0d) reste la première. Même piège pour tout nouveau jeu.

**Gungeon.** Gabarits `games/gungeon/assets/maps/salles.ldtk` (quinze gabarits (8×8 tuiles à l'origine, **agrandis par m2-salles-murs-gungeon : voir « Échelle des gabarits » ci-dessous**), comptés dans les niveaux du fichier : 1 `depart`, 4 `combat`, 1 `recompense`, 1 `boutique`,
2 `boss`, 3 `arene`, 1 `escouade`, 1 `coffre`, 1 `secret` ; neuf à l'origine, douze avant M2-T14d, §52.d ;
les neuf d'origine viennent du script `docs/taches/rapports/m2-t10-grammaire-etage.make_salles.py`) : `Depart`,
`CombatEW/NS/T/Croix`, `Recompense`, `Boutique`, `BossW/N`, puis `ArenaEW/Croix` (M2-T7), `EscouadeEW` (M2-T8),
`Coffre`, `Secret` et `ArenaNS` (T14d). Grammaires `etage` (10–14 salles depuis T14d), `grand` (15–20) et `court`
(3 salles : départ, combat, boss) ; séquences `etages_grammaire`, `etages_grand`, `etages_court`,
`etages_court_deux`. T0d (`etage_1`) reste la séquence par défaut : les traces `gungeon_*` d'avant
ne bougent pas, la grammaire est activée par `floors: Some("etages_...")`.

**Scénarios.** `gungeon_floor_seed_{1,23,123}` : étage `court` (même forme, départ, combat en T,
boss à l'est, pour ces trois graines : relevé par `ALACOD_FLOOR_DUMP=court` du test
`gungeon_etages_de_grammaire`), joueur scripté jusqu'à la mort du boss. `gungeon_two_floors` :
graine 123, deux étages `court` de même forme, `FloorIndex`. `bench_floor` : étage `grand`, deux
bots `fonceur` (plancher dans `tests/budgets.ron`). L'atteinte du boss est scriptée : les bots
nettoient les salles de combat mais ne vont pas vers une salle dormante (bot explorateur : T16).

### Échelle des gabarits (m2-salles-murs-gungeon, 2026-10-18)

Revue M2 de William : « chaque salle est minuscule ». Mesure d'avant (`docs/taches/rapports/m2-salles-murs-gungeon.md` §2) :
les quinze gabarits de `salles.ldtk` faisaient **8×8 cases** (128×128 px, **six cases de sol de large**, 36 cases de sol), soit
**3 sprites de héros** (32 px) ou **4,8 colliders** (20 px) de large, 0,64 s de traversée à 150 px/s ; à l'écran (960×540,
`camera.ron` `min_zoom: 0.6` : 576×324 unités) une salle en occupait **22 % × 40 %**. `etage_1.ldtk` (squelette T0d, trois
salles 8×8) et `hub.ldtk` (256×192 px, déjà 16×12 cases) ne changent pas : ni l'un ni l'autre n'est joué dans une partie ordinaire
(`entry.floors: "etages_deux"`).

**Échelle retenue** (cases de 16 px, murs compris ; le script `docs/taches/rapports/m2-salles-murs-gungeon.make_gabarits_grands.py`
réécrit les quinze gabarits depuis la version 8×8, de façon déterministe) :

| type | gabarits | cases | surface ×ancienne | intérieur | écran (zoom 0,5) |
|---|---|---|---|---|---|
| départ | `Depart` | 14×12 | 2,6 | 12×10 | 47 % × 71 % |
| combat, escouade | `CombatEW/NS/T/Croix`, `EscouadeEW` | 16×12 | 3,0 | 14×10 (11 colliders de large) | 53 % × 71 % |
| récompense, secret | `Recompense`, `Secret` | 12×10 | 1,9 | 10×8 | 40 % × 59 % |
| boutique | `Boutique` | 14×10 | 2,2 | 12×8 | 47 % × 59 % |
| coffre | `Coffre` | 14×12 | 2,6 | 12×10 | 47 % × 71 % |
| arène | `ArenaEW/Croix/NS` | 20×14 | 4,4 | 18×12 (14 colliders) | 67 % × 83 % |
| boss | `BossW/N` | 22×16 | 5,5 | 20×14 (16 colliders) | 73 % × 95 % |

*Pourquoi ces chiffres.* Combat à ×3 (la fiche attendait 2 à 3) : à `min_zoom: 0.5` (480×270 unités à l'écran, 30×17 cases) une salle de
combat occupe un peu plus de la moitié de la largeur et 71 % de la hauteur : on la lit d'un coup d'œil, avec ses portes, sans voir
trois salles voisines (avant : quatre salles et demie dans la largeur). Les salles où l'on ne se bat pas (récompense, secret,
boutique) restent à ×2 : trois présentoirs espacés de 32 px (`SHELF_SPACING`) et un coffre n'ont pas besoin de plus. L'arène (750 frames à
tenir, sept ennemis en trois vagues dont une tourelle) et le boss (patterns, trois phases) demandent de la place pour esquiver : ×4,4 et ×5,5.
Le zoom `camera.ron` passe de 0,6 à 0,5 : le héros reste lisible (64 px de haut à l'écran) et la salle de combat tient dans l'image.
**Le contenu ne change pas** : mêmes ennemis, mêmes props, mêmes nombres ; les entités sont placées **proportionnellement** dans la salle
agrandie (centre de chaque entité conservé en fraction de l'intérieur) et vérifiées par le script (aucun recouvrement, aucun prop ni
apparition sur le passage de deux cases devant une ouverture). Les salles sont donc plus vides : le nombre d'ennemis par salle est une
décision d'équilibrage de William (D64), pas de cette tâche.

*Portes alignées.* Chaque côté ouvert a deux cases d'ouverture **au milieu** du côté. Des gabarits de tailles différentes ont donc leurs ouvertures
à des distances différentes du coin, que l'appariement imposait égales (`Connection::are_matching`, `starting_at`). Pour les grammaires
`Graph` (gungeon) l'appariement ne compare plus `starting_at` (`populate_level_connections_with(.., offset_free: true)`, activé par
`map_ldtk::generation::from` d'après `MapGenerationMode::Floor` + `FloorLayout::Graph` ; Basic, caverne et grille gardent l'appariement strict) et
**`Room::get_connecting_room_position`** décale la salle posée pour que les deux ouvertures soient face à face. Cette fonction avait deux
défauts jamais exercés (les ouvertures étaient toujours au même endroit, les salles de même taille) : elle prenait la taille de la voisine au lieu
de celle de la salle courante pour un côté est ou sud, et le signe du décalage le long du côté partagé était faux pour le sud et l'est (juste pour le nord et l'ouest). Réécrite, sans changement pour des salles
égales (test `la_voisine_se_pose_ouverture_contre_ouverture`) ; test de génération `gungeon_etages_de_grammaire` : toute ouverture restée libre
a, en face, une ouverture libre d'une autre salle, case contre case (étage, grand et court sur 2 000 graines).

*Étendue du monde.* `FloorGrammar::bounds: Option<(i32, i32)>` (pixels, de part et d'autre du départ ; absent : 1000×1000 comme avant) relève la boîte
dans laquelle le planificateur pose les salles : `etage` et `court_secret` à 3000/2000, `grand` à 4000 (les salles de 224 à 352 px la remplissent
plus vite). Lint : `bounds` négatif ou nul est refusé (`problems`).

### Échelle des gabarits d'`isaac` (m3-salles-murs-isaac, 2026-10-19)

Revue de William sur les vidéos `isaac` : « exactement le même problème de visuel que gungeon : pas d'asset pour les murs et autres, salles minuscules où
tout le monde se chevauche, il faut de grandes pièces ». Rapport chiffré : `docs/taches/rapports/m3-salles-murs-isaac.md`. Dans Isaac une salle de 13×7 cases fait
**un écran** et le personnage tient à peu près dans une case.

| | avant (T5, T11d1, T11d2) | après |
|---|---|---|
| unité de salle (`cell` des grammaires) | 15×9 cases de 16 px = 240×144 px | **30×18 = 480×288 px** (×2 uniforme) |
| intérieur (murs d'une case) | 13×7 = 91 cases | 28×16 = 448 cases (×4,9) |
| largeur de sol en héros (32 px) / colliders (20 px) | 6,5 × 3,5 / 10,4 × 5,6 | **14 × 8 / 22,4 × 12,8** |
| traversée de l'intérieur à 120 px/s | 1,7 s | 3,7 s |
| ouverture d'un côté d'unité | 3 cases (48 px) | **4 cases (64 px)**, centrée (13..16 sur 30, 7..10 sur 18) |
| une salle de l'unité à l'écran (960×540, `min_zoom` 0,6 : 576×324 unités) | 42 % × 44 % | **83 % × 89 %** |
| une salle 2×2 | 480×288 px | 960×576 px |
| étage 7×7 unités (`bounds`) | ±840 × ±504 px (monde par défaut ±1000) | ±1680 × ±1008 px, `bounds: Some((1800, 1200))` |

Le **facteur est 2 sur toutes les tailles** : un gabarit de 1×1, 2×1, 1×2 ou 2×2 unités reste un multiple de l'unité, la grille de M3-T5 (`FloorLayout::Grid`, ports centrés
sur chaque segment d'unité) ne change pas de code. Les ouvertures passent de 3 à 4 cases parce que 30 et 18 sont pairs (une ouverture de 3 cases ne se centre plus ; 6 cases
auraient fait un trou de 96 px dans un mur de 480). *Écart à la demande* : le texte de la fiche donnait « intérieur 26×14 cases » pour une unité de 480×288 ; avec un mur d'une case
l'intérieur de 30×18 est de 28×16 (26×14 supposerait deux cases de mur), c'est la valeur retenue.

Script : `docs/taches/rapports/m3-salles-murs-isaac.make_gabarits_grands.py` (déterministe, idempotent depuis la version 15×9, refuse un fichier déjà agrandi ; `git show 46a4e8f3:games/isaac/assets/maps/salles.ldtk`).
Ce qui change : la taille, l'anneau de murs, les ouvertures et les portes ; les apparitions d'ennemis sont placées **proportionnellement** dans l'intérieur ; les points de départ restent
groupés autour du centre (écarts ×2) ; la couleur du sol des niveaux (`#7B6A5F`, brun clair, plus clair que les murs). Les **rochers** gardent leur nombre, leur forme et leur écart au mur le plus proche (un rocher
reste une case de 16 px : le nombre de rochers est le contenu, `isaac_bomb_rock` compte 45 → 42) ; les **piques** (couche `Surfaces`, valeur 5) sont un terrain : chaque case devient un bloc 2×2 à 3×3 (6 → 33 cases dans `CombatPiques`),
leur part de la salle reste la même. Contenu inchangé : mêmes ennemis, mêmes types de salle, mêmes nombres de rochers. Le script **échoue** si une entité recouvre une autre entité, un rocher ou une pique ; si quoi que ce soit se trouve dans le couloir de passage devant une ouverture (largeur de
l'ouverture, 3 cases de profondeur) ; si un rocher (16 px) ou une pique (12 px) est à moins de la distance de T11d1 (§76.4) d'une ligne droite qu'un bot suit (ouverture ↔ ouverture, ouverture → ennemi, ouverture → centre) ; si les portes ne couvrent pas exactement les ouvertures.

*Moteur.* Aucune ligne de génération : `Connection::are_matching` exige la même distance au coin et les ouvertures sont au milieu de chaque côté d'unité, comme avant ; `FloorGrammar::bounds` (déjà là depuis gungeon) relève le monde à ±1800 × ±1200 pour
`sous_sol`, `sous_sol_grand` et `sous_sol_profond` (sept colonnes de 480 px ne tiennent pas dans ±1000) ; les grammaires de 3×3 unités tiennent dans le monde par défaut. **Caméra** : `CameraSettings::room_frame` (donnée de `camera.ron`, défaut non) cadre la
vue sur la salle où se tient la cible (`CameraRooms`, boîtes des salles de l'étage publiées par `map_ldtk::game::rooms::publish_camera_rooms` ; `frame_in_room`, pure, testée) : une salle de l'unité (480×288) tient dans la vue de 576×324 et la caméra y est fixe (une salle = un écran), une grande
salle est suivie sans montrer au-delà de ses murs. `min_zoom` reste 0,6. **Coût** : le champ de flux couvre toute la boîte englobante de la carte (4 fois plus de cases) : `FlowField` de 1 500 à 2 502 µs par frame sur `bench_floor_isaac` (mesuré en même temps, avant et après, sous charge ≈ 16 ; une partie de bot tourne à ~28 fps contre ~53 avant, à charge comparable) ; budgets de `tests/budgets.ron` à refaire au calme (D122). Les campagnes de bots passent `--explored-progress` (le détecteur de soft-lock de 600 frames ne voit pas une marche de 2 000 px : rapport §2).

## Notes essentielles

**`CrateLocation`** : collectée mais non utilisée, voir §1 (entités et vérification par un scénario `idle`).

**Colliders vs sprites** : voir §1 et `CLAUDE.md` (« Flow Field - Implémentation Actuelle ») ; le décalage du collider est `collider.offset` de `CharacterConfig`, celui du sprite `offset_x`/`offset_y` de `SpriteSheetConfig`.

**IntGrid** : les surfaces (E4 v1) ont leur propre couche IntGrid `Surfaces`, sans toucher `Walls` : voir §26.

**Fixed-point** : voir §2 et `CLAUDE.md` (Déterminisme, règles 1 et 7).

**Tests et scénarios** : `make test_scenarios` vérifie que les scénarios passent en synctest (mode déterministe local). Tout changement de code dans `GgrsSchedule` peut casser les traces `.trace` ; rebase sur `main` chaque jour et revalider en CI rapide (preuve d'un bless : §10). Une tâche de toute voie peut bénir des traces, **avec la preuve du §10**, bénies par l'orchestrateur (`BLESS=1`, justification dans le commit) ; V1 (simulation) est la voie qui les change le plus souvent.

**Séquence par défaut et preuve p2p (D59, 2026-10-15).** `entry.floors` (champ optionnel de
`game.ron`, `content::manifest::EntryPoint`) désigne la séquence du dossier `Floors` que `entry.mode:
Floors` joue par défaut ; absent, c'est l'ancien comportement (la première par ordre d'id). Lint : un id
absent du dossier est une erreur `BrokenReference`, le champ sans `mode: Floors` une erreur `Unsupported`
(fixtures `entry_floors_unknown` et `entry_floors_without_mode`). Un scénario garde la main par son champ
`floors`. `gungeon` déclare `floors: "etages_deux"` : la partie ordinaire démarre sur deux étages assemblés
par la grammaire `etage` (l'étage squelette `etage_1` reste jouable par `floors: Some("etage_1")`).
Diagnostic `ALACOD_CLEAR_ENEMIES_AT=<frame>` (`map_ldtk::game::floors`, avec `ALACOD_INPUT=neutral`, à régler
sur **tous** les clients) : à cette frame tous les ennemis disparaissent (`despawn_rollback`), le portail
(barycentre des `PlayerSpawn`) s'ouvre par la règle ordinaire et les clients traversent la transition sans
clavier. Preuve p2p de `gungeon` (2 clients, `ALACOD_CLEAR_ENEMIES_AT=200`, 900 frames) : séquence
`etages_deux` choisie, `floor_portal_open` puis `floor_transition from=0 to=1` à f200 sur les deux clients,
traces identiques sur 899 frames.

## 42. Effets v2 : déclencheurs, conditions, cooldowns et charges (M2-T3, chantier C1 v2)

Suite de §27 (effets v1) et §36 (objets). Code : contrats `effects::{On, Condition, Effect,
Recharge}`, règles pures `effects::runtime` (`responds_to`, `conditions_hold`, `Limits`,
`limits_allow/consume/recharge`), exécution `game::effects_runtime`
(`apply_effects_system` v1, `apply_triggered_effects_system` v2).

**Déclencheurs exécutés** (le lint refuse les autres) :

| Déclencheur | Source lue | Système / ordre |
|---|---|---|
| `OnRoomClear` | `FrameEvents<RoomChanged>` vers `Cleared`, **si le porteur est dans les bornes de cette salle** (iid de l'événement) | v2, après `Run` |
| `OnDodge` | `FrameEvents<sim_core::triggers::DashStarted>` (émis par `apply_inputs` au début du dash) | v2 |
| `OnReload` | `FrameEvents<sim_core::triggers::ReloadFinished>` (émis par `weapon_rollback_system` quand le chargeur est **réellement rempli** ; le délai de pompe d'une arme sans chargeur n'en émet pas) | v2 |
| `OnPickup` | `FrameEvents<ItemPicked>` du porteur (objet ramassé, **le sien compris**) | v2 |
| `OnFloorStart` | changement de `FloorState::index` vu par le porteur ; le premier étage compte pour un effet présent aux frames 0–1 (un effet vu plus tard, par exemple un objet ramassé en cours d'étage, ne se déclenche qu'au **prochain** étage) | v2 |
| `OnBlank` | `FrameEvents<BlankUsed>` (M2-T1) du porteur (`player_handle`) | v2, **après `blank_system`** (qui émet dans `Effects`) |

Les nouveaux variants (`OnFloorStart`, `OnBlank`) sont en **fin d'enum** `On`. Le système v2 tourne
`.after(RollbackSystemSet::Run).after(item_charge_system).after(blank_system).before(FrameCounter)` : `RoomChanged`
est émis dans `Run` et vidé au `FrameStart` suivant, les autres événements dans des sets plus tôt
de la même frame. Les événements `DashStarted`/`ReloadFinished` sont des `FrameEvents` neutres
(file vide : contribution 0 au checksum).

**Conditions** : `Carrying(item)` (objets tenus, passifs et actif) et `SquadSize(n)` (au moins `n`
joueurs en vie, ni `Death` ni `Downed`) sont exécutées. `TargetInRange` reste un contrat (aucune
position de cible dans les déclencheurs) : le lint la refuse.

**Cooldown et charges** : champs optionnels de l'effet, `cooldown: Some(frames)` (délai minimal entre
deux exécutions), `charges: Some(n)` (exécutions avant épuisement), `recharge: Some(PerRoom |
PerFloor)` (retour de **toutes** les charges à la salle nettoyée ou au début d'étage ; `PerRoom`
suit `RoomChanged` de n'importe quelle salle du porteur, `PerFloor` le changement d'index).
Sans ces champs l'effet est celui de la v1. La condition est évaluée **avant** les limites (une
condition fausse ne consomme rien) ; une exécution consomme la charge et arme le cooldown.
Lint : `cooldown = 0`, `charges = 0`, `recharge` sans `charges` sont refusés.
État : `EffectState::limits` (clé `effect:<index>` ou `item:<id>:<index>`).

**Pas de trace qui bouge** : `Effect` et `EffectState` ont un `impl Hash` manuel qui écrit les champs
v2 **seulement s'ils ne sont pas vides** ; un porteur sans effet v2 garde le hash d'avant
(`effet_v1_garde_son_hash`, `effect_state_sans_champ_v2_garde_le_hash_de_t1_10`).

**Effets des objets (choix : lecture dans l'`Inventory`)**. Un objet passif ou actif tenu applique
ses `effects` (hors `OnUse`, exécuté par `game::items`) au porteur : les systèmes lisent
l'`Inventory` à chaque frame au lieu de copier les effets dans `Effects` au ramassage. Raisons :
un objet perdu (actif remplacé) n'a rien à retirer ; pas de composant en plus sur le joueur (donc
pas de hash qui bouge pour les joueurs sans objet à effets) ; l'index d'effet est celui du
`ItemDef`, stable. Le porteur qui n'a pas d'`EffectState` en reçoit un au premier effet exécuté.
Source des modificateurs d'un effet d'objet : `item:<id>:<index>` (distincte de `item:<id>`,
celle des `modifiers` de l'objet). Ordre : porteurs par `GgrsNetId`, effets du personnage par
index puis ceux des objets (passifs dans l'ordre de ramassage, puis l'actif). Deux exemplaires du
même objet partagent leurs limites (clé identique). Un `Tick` d'objet compte depuis la première
frame où le système voit l'objet. Lint : un effet d'objet suit `lint_effect` ; `OnUse` n'a de sens
que sur un actif.

**Contenu** : `gungeon` : `trefle` (soin à la salle nettoyée, une fois par étage), `cartouchiere`
(cadence après un rechargement, cooldown 600), `amulette` (vie max et soin au ramassage, une charge).
`testbed` : neuf objets d'essai (dont `sceau`, `OnBlank`) (`anneau_salle`, `semelles`, `semelles_lentes`, `bandoulier`,
`bandoulier_comptee`, `talisman`, `boussole`, `porte_bonheur`), dont l'effet est un `Modifier(Damage,
Add, 1)` : la valeur de `Stat(Damage)` compte les exécutions.

**Scénarios** : `effect_on_room_clear`, `effect_on_dodge`, `effect_on_reload`, `effect_on_pickup`,
`effect_on_floor_start` (aussi la recharge `PerFloor`), `effect_cooldown`, `effect_charges`
(recharge `PerRoom`), `item_passive_effect` (`OnKill` d'un objet), `effect_on_blank` (passif
`sceau` : +1 de dégâts à chaque Blank utilisée, `testbed`).

**Règle d'engine : `add_frame_events_transient`** (M2-T3). Un événement émis dans des parties
existantes et lu seulement dans la même frame (`DashStarted`, `ReloadFinished`) s'enregistre avec
`add_frame_events_transient::<T>()` : file hors checksum et hors snapshot, vidée au `FrameStart`.
Les lecteurs sont ordonnés `.after` l'émetteur (ici le système d'effets v2, après `Run`, donc
après `Input` et `Weapon`) ; un lecteur placé avant verrait la file de la frame précédente. La
variante neutre ne convient pas : sa file non vide est hachée et déplace les traces existantes
dès qu'un dash ou un rechargement a lieu.

## 43. Esquive v2 (M2-T2, chantier B6)

Code : `combat::actors::DashExtra` (état), `game::character::movement` (`DashConfig`, `DashEffect`,
`IframesKind`, `MovementConfig::resolve_dash`), `game::character::player::input::apply_inputs`
(lancement), `game::character::dash::dash_shove_system` (bousculade), `content::lint::lint_dash`.

**`DashState` ne change pas.** Il est sous checksum et porté par tous les joueurs : le moindre champ
de plus déplacerait toutes les traces dès la frame 0. L'état nouveau vit dans un composant séparé,
`DashExtra { charges, max_charges, recharge_remaining, projectile_iframes_until, shove_force }`,
enregistré en variante **neutre** (`rollback_and_trace_neutral`) et posé à la création
(`character::create`) **seulement** si le personnage déclare `dash: Some(..)` ou
`movement.dash_charges`. Aucun porteur ailleurs : zombies, testbed, throne gardent leurs traces.
`is_dashing` passe à vrai à la frame de l'appui, comme en v1 (M2-T3 y branche `OnDodge`).

**Réglage.** `MovementConfig` gagne `dash_charges: Option<u32>` et `dash_recharge_frames`. Le RON
du personnage gagne `dash: Option<DashConfig>` ; chaque champ absent retombe sur le réglage v1 de
`movement` : `distance`, `duration_frames`, `cooldown_frames`, `iframes`, `iframes_kind`
(`All` v1 / `Projectiles`), `effect` (`Roll` v1 / `Shove` / `InPlace`, nouveaux effets en fin d'enum),
`shove_force`, `charges`, `recharge_frames`.
```ron
dash: Some((iframes_kind: Projectiles, charges: Some(2), recharge_frames: Some(60))),
```

**Charges.** Sans `charges` : v1 exact, le cooldown seul règle le rythme. Avec : un dash consomme une
charge (refusé à zéro, l'appui est gardé par le buffer comme en v1), `dash_cooldown_frames` reste
l'écart minimal entre deux dashs, une charge est rendue `recharge_frames` frames après la
consommation de la première manquante (le décompte repart tant qu'il en manque). Le test et le
décompte sont dans `apply_inputs`, avant le lancement : une charge rendue à la frame F est utilisable à F.

**Variantes.** `Roll` : v1. `Shove` : pendant le dash, chaque ennemi dont le collider touche celui du
joueur reçoit `Velocity.knockback = direction du dash × shove_force` (assigné, pas cumulé ; l'amortissement
habituel s'applique), ordre `GgrsNetId`. `InPlace` : distance 0 (les pas sont nuls), seulement les
i-frames ; pas de tir pendant la durée, comme tout dash.

**I-frames contre projectiles.** `iframes_kind: Projectiles` n'écrit **pas**
`Health.invulnerable_until_frame` : `DashExtra.projectile_iframes_until` (dernière frame comprise),
lu par la collision des balles (`weapons/mod.rs`) et des projectiles composables
(`projectile.rs`) ; elles traversent le porteur sans être consommées. La mêlée, les contacts, la
ruée (`Charge`) et tout autre dégât le touchent. `All` (défaut) : v1.

**Lint** (`OutOfRange`, seulement pour un personnage qui déclare `dash` ou `dash_charges`) :
`dash_charges = 0`, `dash_recharge_frames = 0` avec des charges, durée 0 hors `InPlace`, i-frames
plus longues que la durée. Fixtures `dash_charges_zero`, `dash_recharge_zero`, `dash_duration_zero`,
`dash_iframes_over_duration`.

**Scénarios** : champ `dash` de `PlayerScript` (réglage de scénario, `DashOverride` + `DashExtra`
posés avant la première frame par `apply_player_overrides` ; conservé au ré-enregistrement) pour
essayer une variante sans changer le contenu des jeux. `dash_charges` (gungeon : deux dashs à f15 et
f45, le troisième, refusé, part à f76 quand la charge de f15 est rendue), `dash_shove` (un
`follower` projeté, 52 px derrière contre 30 pour `Roll`), `dash_in_place` (position inchangée,
`NoDamageBetween` sur les i-frames), `dash_jump_bullets` (les balles de la `tourelle_salve` traversent,
la ruée du `charger` touche dans la même fenêtre).

**Contenu** : `gungeon/characters/pistolero.ron` : deux charges (une rendue toutes les 60 frames),
i-frames contre projectiles seulement. Les traces `gungeon` ne bougent qu'à cause de ce nouveau
composant (voir le rapport).

## 44. Objets v2 et synergies (M2-T4, chantier C2)

**Synergies.** Kind de contenu `Synergy` (dossier `synergies/`, `(path: "synergies", kind: "Synergy")`
dans `game.ron`), un fichier RON par synergie, id = nom de fichier, champs de `items::SynergyDef` :
`name`, `requires: [ids]` (deux objets distincts ou plus : passifs ou actif), `modifiers` et `effects`
(mêmes formes que `ItemDef`). Table `items::SynergyTable` (ressource hors rollback, posée au chargement
de la carte par `map_ldtk`, comme `ItemTable`).

Active **tant que tous** les objets requis sont tenus (`Inventory::has` : un passif ou l'actif
courant), retirée dès qu'un manque (l'actif remplacé tombe au sol). **État dérivé de l'`Inventory` à
chaque frame, aucun composant ni hash en plus** :
- `modifiers` : `game::items::synergy_modifiers_system` (`Effects`, après `blank_system`) pose les
  modificateurs de source `synergy:<id>` quand la synergie devient active, les retire (`remove_by_source`)
  quand elle cesse ; synergies par id, joueurs par `GgrsNetId`. Un joueur sans `Inventory` n'a pas de synergie.
- `effects` : `game::effects_runtime::listed_effects` ajoute, après les effets des objets, ceux des
  synergies actives (par id) sous la clé `synergy:<id>:<index>` (limites, `Tick`, source de modificateur
  `synergy:<id>:<index>`, comme les effets d'objet). Un effet de synergie suit `lint_effect` ; `OnUse` y est
  refusé. Le ramassage qui complète la paire déclenche déjà un `OnPickup` de la synergie (même frame).

**Actifs v2** (`ItemDef`, champs optionnels, v1 exact sans eux) :
- `uses_per_floor: Some(n)` : au plus `n` usages par étage du mode `Floors` (hors `Floors`, par partie).
  Mémoire dans `ActiveSlot::uses = Some((étage, usages))` : les usages d'un autre étage ne comptent plus,
  aucun reset à écrire. **`uses` n'entre dans le hash que s'il est posé** (`impl Hash` manuel d'`ActiveSlot`,
  comme `Effect` en §42) : le hash d'un actif v1 est celui d'avant. Un usage refusé garde la charge.
- `cost: Some((item, n))` : l'usage prélève `n` exemplaires du consommable `item` ; sans le stock, il est
  refusé (charge conservée). Lint : `item` connu et consommable, `n > 0`.
- `OnUse` accepte maintenant `Heal` (en plus de `Modifier` et `TimedModifier`) : `game::items::apply_instant_effects`.

**Consommables à effets.** Un consommable dont `effects` n'est pas vide (uniquement `OnUse`, lint) **s'applique
au ramassage** et n'entre pas dans l'inventaire : aucun bouton d'usage n'existe pour un consommable générique
(pas de nouveau bit d'input, accord d'orch). Actions : `Heal`, `Modifier`, `TimedModifier`. Un consommable
sans effet reste un compteur (`key`, `blank`, `obus`). Pas de refus de ramassage à pleine santé (le soin est
simplement borné).

**Lint** : synergie à objet inconnu (`BrokenReference`), à moins de deux objets distincts (`OutOfRange`, sauf transformation, voir plus bas),
à un consommable (`Unsupported`), deux synergies pour le même ensemble (`DuplicateId`), `OnUse` dans une
synergie ; `uses_per_floor = 0` (`OutOfRange`), `uses_per_floor`/`cost` sur un non-actif, `cost` vers un
objet inconnu ou non consommable. Fixtures `synergy_unknown_item`, `synergy_single_item`,
`synergy_duplicate_set`, `item_uses_per_floor_zero`, `item_cost_not_consumable`.

**Contenu.** `gungeon` : synergies `tresor_de_poche` (`trefle` + `amulette` : +10 de vie max) et
`tir_nourri` (`cartouchiere` + `etendard` : +25 % de cadence) ; actif `pierre_de_soin` (`Rooms(1)`, soin 20,
`uses_per_floor: 2`) ; consommable `ration` (soin 15 au ramassage). Pas de « bouclier temporaire » :
`StatId::Armor` n'est lue par aucun système (un actif sur cette stat serait sans effet), à reprendre quand
la réduction de dégâts existera. `testbed` : objets d'essai `lame`, `manche`, `braise`, `eclat`, `canon`,
`obus`, `ration`, synergies `lame_manche` (modificateur +2 et effet `OnPickup` +4) et `lame_braise` (+2).

**Scénarios** : `synergy_pair` (bonus avec les deux objets, absent avec un seul : dégâts 1 puis 7),
`synergy_lost` (actif remplacé : dégâts 3 puis 1), `item_active_per_floor` (trois appuis par étage, deux
effets, sur `deux_niveaux`), `item_active_cost` (obus 3 → 1, second usage refusé), `consumable_heal`
(santé 92 → 97 au ramassage). Aucune trace existante ne bouge : types neutres, champs optionnels hors hash.

### Transformations : `requires_tag` (suivi Q10, réponse de William du 2026-10-16)

Une **transformation** est une synergie dont la condition est un **tag** d'objet : `SynergyDef.requires_tag:
Some((tag: "champignon", count: 3))` (`items::TagRequirement`) ; champ optionnel, **absent** : la synergie d'avant, au bit
près (aucune trace ne bouge : définition de contenu, hors rollback). `requires` devient facultatif (`#[serde(default)]`) :
vide, la transformation n'exige que le tag ; non vide, les deux se **cumulent**. Une synergie sans aucune condition n'est
jamais active.

- **Règle** (`SynergyDef::is_active`, pure, utilisée par `SynergyTable::active` et par
  `game::items::synergy_modifiers_system`, donc par `projectile_mods`, les effets et les modificateurs) :
  active quand `Inventory::held_with_tag(items, tag) >= count`, soit le nombre d'objets **distincts** tenus
  (passifs et actif) dont `ItemDef.tags` contient `tag`. **Un exemplaire en double compte une fois**, un consommable ou
  un objet inconnu jamais ; l'actif perdu éteint la transformation. (`SynergyTable::active` prend désormais
  l'`ItemTable` : tous les appelants migrés.) **D91 (fermée)** : `held_with_tag` n'alloue plus (le `Vec` + tri +
  `dedup` par appel est remplacé par un balayage : un passif compte à sa première occurrence, l'actif s'il n'est pas
  déjà tenu en passif) ; son seul appelant est `SynergyDef::is_active`, donc `synergy_modifiers_system`, `projectile_mods`
  et les effets en profitent sans changer. Même résultat au bit près (test `held_with_tag_compte_les_objets_distincts`,
  scénarios de synergies et de transformations inchangés).
- **Effets** : ceux d'une synergie ordinaire (`modifiers` de source `synergy:<id>`, `effects`, `projectile`), posés et
  retirés à chaque frame depuis l'inventaire.
- **Teinte de présentation** `tint: Some("#rrggbb")` (champ optionnel de `SynergyDef`, hors simulation) : la première
  transformation active (par id) à `tint` colore les calques du joueur (`feedback::TransformationTint`, relue de l'état
  chaque tick, derrière la teinte d'un champion et devant celle du marqueur) et produit un moment clé `feedback`
  « transformation 8 (champignons) » (`FeedbackKind::Transformation`, une fois à l'acquisition).
- **Lint des mécaniques** (`content::mechanics`) : une transformation compte comme atteignable (ligne « transformations »)
  quand au moins `count` objets non consommables du tag sont atteignables par pool, départ ou étage ; sinon
  avertissement « seulement n objet(s) à source portent le tag ».
- **Lint** (`lint_synergies`) : `count = 0` (`OutOfRange`), tag que **aucun objet** ne porte (`BrokenReference`), `count`
  supérieur au nombre d'objets non consommables qui portent le tag (`OutOfRange` : jamais active), `tint` qui n'est pas
  `#rrggbb` (`OutOfRange`) ; « au moins deux objets distincts » ne vaut plus que sans `requires_tag` ; deux synergies de
  même ensemble **et** de même condition de tag restent un doublon. Fixtures `transformation_ok`, `transformation_bad`
  (`crates/content/tests/transformations.rs`).
- **Testbed** : objets `amanite`, `bolet`, `morille` (tag `champignon`, sans effet seuls), synergie `champignons`
  (`count: 3`, `MoveSpeed` `Pct` +0,3, teinte `#c07040`). Scénarios `transformation_three` (trois ramassages à f10, f40, f70 : la
  vitesse monte à la troisième prise) et son témoin `transformation_two` (deux champignons : rien) ; générés `item_amanite`,
  `item_bolet`, `item_morille`.

## 49. Formations simples : escouades en arc et tir alterné (M2-T8, chantier D1 v2)

Code : `behaviors::{Behavior::Formation, FormationShape}`, `game::character::enemy::ai::squad`
(`formation_system`, `SquadSlots`, `arc_places`, `fire_turn`, `nearest`), `rules::behavior_motion`
(déplacement), `rules::behavior_select_system` (tour de tir), `content::lint::lint_formation`.

**Contenu.** Un ennemi se déclare membre d'une escouade par `ai.squad: Some(("id", Some("rôle")))` et
liste une règle `Formation` (variante **en fin d'enum** `Behavior`, aucune discriminante existante ne bouge) :

```ron
ai: Some((
    squad: Some(("bullet_kins", Some("arc"))),
    behaviors: Some([
        Shoot(weapon: "arsenal", pattern: "salve", range: "240.0", cooldown_frames: 90),
        Formation(shape: Arc(radius: "56.0", spread: "1.6"), period: 40),   // volley: true : tir groupé
        Chase(profile: "Ground"),
    ]),
)),
```

**Escouade = donnée dérivée.** Les ennemis vivants (hors salle dormante) de même `squad` qui listent une
`Formation`, rangés par `GgrsNetId` (le rang est l'indice dans cet ordre). `formation_system`
(`EnemyAI`, après `enemy_target_selection`, avant `behavior_select_system`) vide puis réécrit
`SquadSlots` à chaque frame, avant tout retour anticipé : ressource **hors rollback**, aucun état caché,
aucun nouveau type sous checksum. Une escouade vit tant que ses membres vivent ; elle ne dépend pas de la
salle (une seule salle d'escouade est éveillée à la fois dans `gungeon`).

**Cible et places.** La cible de l'escouade est le joueur debout le plus proche de son centre (moyenne
des positions), à distance égale le plus petit `GgrsNetId` (`nearest`). L'arc est centré sur la cible,
son milieu pointe vers le centre de l'escouade ; la place du rang `k` sur `n` membres est l'angle
`spread × (k/(n−1) − 1/2)` (le membre seul est sur l'axe). Les places ne changent pas tant que l'effectif
ne change pas ; un membre mort réattribue les rangs au prochain tour. Angles et trigonométrie en `Fixed`
(`cos_fixed`/`sin_fixed`). Écart avec la fiche : « plus proche par le chemin » est ici la distance
euclidienne au centre de l'escouade (pas de flow field par escouade).

**Déplacement.** À plus de `2 × radius` de la cible : mouvement d'origine (flow field, comme `Chase`).
Plus près : droit vers la place, puis, à moins de 6 unités, `Strafe` léger (quart de vitesse, sens
alterné toutes les 45 frames). **Tir alterné** : `Shoot` n'est candidat que dans la fenêtre du membre,
`(frame / period) mod effectif == rang` (`fire_turn`, état dérivé, aucun curseur) ; une séquence en cours
va à son terme. `volley: true` garde le tir groupé. Effets de bord : un `Shoot` plus long que `period`
chevauche la fenêtre suivante (les patterns d'escouade tiennent dans une fenêtre).

**Lint** (`OutOfRange`, fixtures `formation_*`) : `radius ≤ 0`, `spread` hors ]0, 2π], `period = 0` sans
`volley`, formation sans `Shoot` ni `Chase` dans la liste, formation sans `ai.squad`.

**Testbed** : `escouade_tireur` (trois `salve` en fenêtres de 40, arc de 100). **Gungeon** :
`bullet_kin_squad` (arc de 40, la salle est petite), niveau `EscouadeEW` de `maps/salles.ldtk`
(`room_kind: escouade`, type `rooms/escouade.ron`), grammaire `court_escouade` et séquence
`etages_court_escouade`. Ni `bullet_kin` ni les grammaires `court`/`etage`/`grand` ne changent : le nouveau
type de salle n'est tiré par aucune.

**Attentes** : `EnemyDistance` accepte `target: Entity(<EntityRef>)` (écarts entre membres).
**Scénarios** : `formation_arc` (convergence à 100, écarts angulaires 59/113), `formation_member_dies`
(le rang 0 tombe, les deux restants se placent aux extrémités), `formation_alternate_fire` (2, 4, 6 puis 8
flèches au lieu de 6, 12, 18), `formation_two_players` (arc autour du joueur le plus proche),
`gungeon_squad`.

## 45. Boss et arène (M2-T7, chantier D4)

Complète §37 (phases, frise) et §35 (salles verrouillées) : tir **par phase**, salle de boss,
arène à tenir, actions de frise `SpawnEnemies` et `Telegraph`.

**Tir par phase** (lève la limite de §37). Une règle `Shoot` d'une phase (`Phase::behaviors`)
utilise **sa propre** table de tir : arme (donc table de projectiles), pattern, portée, recharge.
`EnemyBehaviors::phase_shoot(phase)` la rend, `enemy_attack_system` la préfère à `ai.ranged` ;
`spawn_enemy` équipe en plus (inactives) les armes des `Shoot` de phase. Sans `Shoot` de phase,
`ai.ranged` comme avant (aucune trace ne bouge). Personnage `testbed/gardien_tir`.

**Salle de boss.** Type de salle `boss` (`locks: true`, §35). Le boss **dort** tant que sa salle est
`Dormant` : `boss_phase_system` saute les entités `RoomDormant` (la phase 0 est donc datée à l'éveil,
pas au premier tick du jeu : le boss ne tire plus depuis une salle fermée). À sa mort,
`boss_defeated_system` (`DeathManagement`) émet `FrameEvents<BossDefeated>` (**transitoire** :
`add_frame_events_transient`, hors checksum et hors snapshot, vidé à la frame suivante) puis
`map_ldtk::game::rooms::boss_room_clear_system` tue les autres ennemis de sa salle (renforts compris) :
la salle passe `Cleared` dans la frame (portes rouvertes, `RoomChanged`). `floor_portal_open_system`
ne bouge pas : le portail ou la victoire de fin d'étage viennent après, comme avant. Trace
`ggrs{f=… boss_defeated net_id=…}` et `boss_room_clear`.

**Actions de frise** (variantes **en fin d'enum** `effects::Action`, le hash des autres ne bouge pas) :
- `Telegraph(n)` : le prochain `SpawnPattern` de la même liste est précédé d'un télégraphe de `n`
  frames (l'émetteur joue `Sequence([Telegraph(n), pattern])`) ; aucun type nouveau.
- `SpawnEnemies(id, count)` : `count` personnages `id` apparaissent dans la salle du boss, positions
  tirées du flux RNG `room:<niveau>` (`world::room_rng_stream`) sur une case libre du profil `Ground`
  (16 essais, repli sur le centre). Demande `game::boss::ReinforcementRequest` (transitoire) émise
  par `boss_phase_system` et exécutée dans la même frame par `boss_reinforcements_system` (`Run`).
Refusées partout ailleurs que dans un boss (power-up, effet de personnage : `OutOfRange`).

**Arène à tenir.** Champ optionnel `arena` d'un fichier `rooms/<id>.ron` :
`(arena: Some((clock: 600, waves: [(at: 0, enemies: [("dummy", 2)]), …])))`
(`world::ArenaDef`, `ArenaWave`). Une arène verrouille **dès qu'un joueur debout entre**, même vide
(`RoomKindTable::locks` est vrai), et lance son horloge (`ArenaClocks`, ressource rollback **neutre**,
frame de verrouillage par niveau). Vagues : `arena_waves_system` (`Run`) fait apparaître les ennemis
d'une vague quand `frame - début == at` (aucun curseur : tout se déduit de l'horloge et de l'état de
salle). À `clock` frames, si un joueur vit, les survivants meurent et la salle passe `Cleared`
(portes rouvertes). Si tous les joueurs meurent, défaite universelle. Trace
`ggrs{f=… arena_wave …}` et `arena_kill`.

**Lint.** `lint_arenas` : `clock` > 0, au moins une vague, `at` < `clock` et croissants, vague non
vide, personnage connu (`BrokenReference`), `count` > 0 ; dans une frise : `SpawnEnemies` d'un
personnage inconnu, `count` 0, `Telegraph(0)`, `Telegraph` sans `SpawnPattern` après lui. Fixtures
`arena_no_waves`, `arena_clock_zero`, `arena_unknown_enemy`, `arena_count_zero`,
`boss_spawn_enemies_unknown`, `boss_spawn_enemies_count_zero`, `boss_telegraph_alone`.

**Déterminisme.** Types nouveaux tous neutres ou transitoires (`ArenaClocks`, `BossDefeated`,
`ReinforcementRequest`) : aucune trace hors `gungeon` ne bouge. Les flux `room:<niveau>` ne naissent
qu'au premier renfort ou à la première vague. Apparitions dans l'ordre : demandes par `GgrsNetId` du
boss, salles triées par position puis identifiant.

**Contenu.** `testbed` : `gardien_tir`, `gardien_renforts`, types de salle `boss`, `arene`,
`arene_mortelle`, cartes `salles_boss*.ldtk` et `salles_arene*.ldtk`
(`docs/taches/rapports/m2-t7-boss-arene.make_cartes.py`).

**Scénarios.** `boss_room_lock` (le boss dort : aucune flèche avant le verrou f85 ; réouverture à sa mort f184),
`boss_phase_shoot` (flèches de `fireball_gun` en phase 0, balle `ricochet` de `tourelle_gun` en phase 1, projectile
qui n'existe que dans cette table), `boss_reinforcements` (1 → 3 → 0 ennemis), `arena_survive` (verrou f85, rouverte
f685 = 85 + 600), `arena_fail` (le joueur immobile meurt f1514, la salle ne se rouvre pas), `gungeon_boss_bot` (le bot
`fonceur` bat le `gatling` à f403, victoire).

**`gungeon`.** `gatling` : 120 PV (150 avant), phase 0 `eventail` ; phase 1 (sous 50 %) `spirale` ; phase 2 (sous
10 %, la rage) `SpawnEnemies(bullet_kin, 1)` puis `Telegraph(10)` + `couronne`. Les scénarios scriptés de M2-T0d,
T1, T3 et T10 gardent leurs entrées : un renfort plus tôt retardait la mort du boss de plus de 40 frames (il
bloque des balles et le joueur scripté meurt à quelques points de vie près) ; la rage se déclenche donc à la
dernière touche (sous 10 %) et les PV tombent à 120. Type de salle `arene` (`rooms/arene.ron`, horloge de 900
frames, 2 + 2 + 3 `bullet_kin`), deux gabarits `ArenaEW` et `ArenaCroix` dans `maps/salles.ldtk`
(`docs/taches/rapports/m2-t7-boss-arene.make_arene.py`), exigée une fois par la grammaire `etage`
(`required: {"arene": 1}`), validée sur 1 000 graines (`ALACOD_FLOOR_SEEDS=1000 cargo test -p map_ldtk
gungeon_etages_de_grammaire`). Aucun scénario ne joue encore l'arène de gungeon.
## 46. Surfaces v2 : fosses, tables renversables, barils explosifs (M2-T11, chantier E4 v2)

Code : `world::surface` (`PitDef`), `world::props` (`PropDef`, `TableState`, `BarrelState`,
`PitSafePosition`, règles pures `flip_direction` / `flipped_rect`), `game::props` (`PropsPlugin` :
`pit_system`, `table_flip_system`, `table_cover_system`, `barrel_explosion_system`),
`map_ldtk::game::entity::prop` (entités LDtk), `map::generation::entity::prop` (génération).
Script des cartes : `scripts/construire-cartes-t11.py`.

**Fosse.** Une surface (§26) peut porter `pit` ; la case IntGrid `Surfaces` correspondante est
une fosse :
```ron
// games/testbed/assets/surfaces/fosse.ron
(intgrid_value: 4, tags: ["fosse"], move_speed: "1.0",
 pit: (fall_damage: "10.0", dash_crosses: true))   // respawn: LastSafeCell (seul choix)
```
Un joueur (hors à terre) dont les pieds (§26) sont sur une fosse perd `fall_damage` PV (dégâts
`True`, tag `fall`), revient à sa dernière position sûre et sa vitesse est remise à zéro. Un dash en
cours traverse la fosse si `dash_crosses` ; il tombe s'il finit dessus. Position sûre =
`PitSafePosition` (composant rollback **neutre**, posé sur les joueurs seulement quand la carte a
au moins une case de fosse) : translation d'une frame où la case et ses 8 voisines n'étaient pas
des fosses, réécrite à chaque frame **avant** toute chute. Un joueur apparu dans une fosse n'a pas
de point de retour et ne tombe pas. Ennemis : les cases de fosse bloquent `Ground` et
`GroundBreaker` dans le champ de flux (`ObstacleType::Pit`), `Flying` et `Phasing` les passent (sans
pénalité) ; `EnemyNeverInWall` (§22) échoue aussi quand les pieds d'un ennemi au sol sont sur une
fosse. Sans la ligne de `update_flow_field_system`, `pit_enemy_path` échoue à f137.

**Objets de décor.** Kind `Prop` (`(path: "props", kind: "Prop")`), un fichier RON par objet, id =
nom de fichier, lu avec l'extension RON `UNWRAP_VARIANT_NEWTYPES` :
```ron
Table(bullets: 4)                                     // thickness: "6", range: "20" par défaut
Barrel(health: "20.0", radius: "48.0", damage: "30.0")
```
Entités LDtk `Table` et `Barrel` (aucun champ requis ; champ optionnel `prop` = id, défaut `table` /
`barrel`), lues par la génération (`PropConfig`, valeurs d'auteur portées telles quelles comme
`WeaponLocation`) puis créées par `spawn_map_items` avec `MapRollbackMarker` : `GgrsNetId` triés par
nom puis position comme les portes. Une entité dont l'id est inconnu ou d'un autre type reste
inerte (`warn!`). Lint : table à `bullets` 0, baril à `health`/`radius`/`damage` ≤ 0, fosse à
`fall_damage` ≤ 0 (`OutOfRange`), fosse sans `fall_damage` (`Parse`) ; fixtures `pit_*` et `prop_*`.

**Table.** Debout : marqueur `Window`, couche `window`, `Obstacle` `Barricade` non cassable,
`Interactable { Table }` (portée 20 du bord du collider). Les balles passent, les joueurs et les
zombies non. L'interaction (`InteractionType::Table`, variante ajoutée en fin d'enum) la renverse à
l'opposé du joueur (axe dominant, égalité : x) : elle perd `Window` et `Interactable`, gagne
`Wall` + couche `wall`, et son collider devient une bande de `thickness` au bord renversé sur toute
la longueur de ce bord. Chaque balle qui la touche (`combat::projectile::BulletWallHit`, file
**transitoire** hors checksum, émise par les collisions de balles ordinaires et composables) retire
1 à `bullets_left` ; à 0, `despawn_rollback()`. `TableState` : rollback neutre.

**Baril.** Personnage `Enemies` sans IA (Neutral ne subit jamais de dégâts) avec `Health`,
`BarrelState` (neutre) ; sans butin (power-ups, armes de progression ignorent `BarrelState`). À 0 PV
il garde son `Death` (santé figée à 0) : `rollback_apply_death` l'ignore. À la frame suivante
`barrel_explosion_system` (`CollisionDamage`, avant le résolveur) envoie `damage` (genre `Physical`,
tag `explosion`, tir ami `Always`) à tout ce qui a des PV dans `radius` (distance centre à centre,
hors barils déjà morts), crédité au dernier joueur qui l'a touché, puis `despawn_rollback()`. Les
barils touchés meurent dans la même frame (résolution avant `DeathManagement`) et explosent à la
suivante : chaîne déterministe (ordre `GgrsNetId`).

**Scénarios** (testbed, cartes `props_*.ldtk`) : `pit_fall` (100 → 90 PV, retour à x -150,6),
`pit_dash` (traversée, 100 PV), `pit_enemy_path` (le `follower` contourne), `table_flip_cover`
(cible à 5 coups avant, 5 pendant la couverture, 13 après la rupture), `barrel_chain` (f63 premier
baril à 0 PV, f64 second, f65 joueur 1 à 70). Nouvelle catégorie `EntityCount(kind: Barrel | Table)`.

**Gungeon.** `CombatEW` et `CombatNS` de `maps/salles.ldtk` portent deux barils et une table
(définitions d'entités ajoutées au projet LDtk) ; test `gungeon_decor_sur_1000_graines`
(`map_ldtk::generation`) : 1 000 graines de `etage`, 3 objets exactement par salle décorée, dans la
salle, sans recouvrir porte, spawn ni personnage. Seule `bench_floor` bouge (12 barils, 6 tables).

**Pièges.** (1) Une file `FrameEvents` « neutre » est hachée dès qu'elle n'est pas vide : un
événement émis dans des parties existantes (ici un mur touché) exige `add_frame_events_transient`.
(2) Un système qui pose un composant neutre sur les joueurs doit se garder sur une condition de
**carte** (case de fosse présente), pas de **jeu** (une surface fosse existe) : sinon les cartes à
eau du même jeu changent de trace. (3) `Team::Neutral` ne subit aucun dégât : un objet destructible
est `Enemies`. (4) Les bullets partent ~9 unités sous le centre du joueur : ne pas poser le joueur
en bord de couloir. (5) Deux worktrees sur un même `CARGO_TARGET_DIR` collisionnent (chemins
relatifs hachés pareil) : `find crates games -name '*.rs' -exec touch {} +` avant de reconstruire.
## 47. Bot explorateur d'étages (M2-T16, chantier K2 v1)

**Profil `explorateur`** (`BotProfile::Explorateur`, fin de l'enum, `bots::explorer`) : joue seul un étage
de salles typées du mode `Floors` (§35, §40). Aucune simulation touchée : il ne produit que des inputs, les
profils existants sont inchangés (aucune trace existante ne bouge).
- **Combat** : celui de `prudent` (chasse par le chemin, esquive, gestion d'arme, réanimation, ramassage au
  passage), contre les seuls ennemis **éveillés** : un ennemi `RoomDormant` n'est pas une cible mais une salle
  à visiter.
- **Blank** (§41) : appui quand une balle ennemie qui approche est à moins de `BLANK_DISTANCE` (48 px) et que
  le joueur a un blank ; front montant lu dans `BlankState::held` (état rollback).
- **Exploration**, sans ennemi éveillé : le but le plus proche **par le chemin** (champ à une source,
  `BotNavigation::costs_from`). Ordre déterministe : coût, objets avant salles, puis `GgrsNetId` de l'objet
  ou identifiant de salle (`LevelId`). Buts : objet au sol (pas d'actif si le joueur en tient déjà un :
  il échangerait sans fin), salle verrouillante à visiter (`Locked`, ou `Dormant` avec un ennemi dedans), la
  salle du boss en dernier. Portail ouvert : `prudent` va au portail.
- **Mémoire** : aucune. « Visitée » se lit dans l'état (`RoomState` `Cleared`, ou `Dormant` sans ennemi) ;
  les champs de navigation sont des caches dont la clé contient la géométrie.
- **Option** `ExplorerOptions::boss_direct` (`alacod-sim --boss-direct`) : droit au boss, sans autre salle
  ni objet.

**bot-explorateur-v2 (D65).**
- **Arène** : une salle `arene` dormante et vide est un but (`explorer::room_pending`) jusqu'à son
  nettoyage : elle ne se verrouille qu'à l'entrée d'un joueur et ses vagues naissent à ce moment.
  Même ordre que les autres salles (coût du chemin, puis identifiant), le boss reste en dernier.
- **Salles non verrouillantes** (départ, récompense, boutique) : un but jusqu'à ce qu'un joueur y
  entre (dans la salle réduite de 24 px) ; on n'y achète rien. **Seule mémoire du bot**,
  `ExplorerVisits` (ensemble `(index d'étage, salle)`, ressource hors rollback) : une salle sans
  état n'a rien à lire dans `RoomStates`. Elle ne sert qu'à produire des inputs, que GGRS enregistre ;
  un rollback ne rappelle pas le bot pour une frame passée.
- **Réanimation** : l'explorateur relève un coéquipier à terre sans attendre les 600 dernières frames
  d'urgence (`REVIVE_URGENT_FRAMES` reste la règle de `prudent`/`fonceur`). Il attendait l'urgence
  sans autre but : graine 175 à 2 bots, saignement fini à f2459, première interaction f1887.
- **Zone morte** : `surface_step` considère le bot arrivé quand son pas est sous `ROUTE_DEAD_ZONE`
  sur les deux axes ; sinon il restait immobile à 36 px de la portée, sans presser Interaction
  (graine 47 à 2 bots, `etages_deux`, soft-lock).
- **D68 : à terre, l'explorateur envoie un input neutre** jusqu'à sa réanimation. Il poursuivait
  sinon le repli de `prudent` ou le portail, quittant la portée de l'équipier. Le jeu continue
  d'autoriser le déplacement à terre pour les joueurs humains (`downed_speed_mult`).
- Campagne : `docs/taches/rapports/bot-explorateur-v2/campagne.sh` (lots de 10 graines).
- **M3-T14 (§79)** : porte à clé, bombes (mur secret, rochers) et fuite de la mèche, achats par politique, diable, sous au
  sol, objets refusés ; politique en données (`ExplorerPolicy`, `bot.ron`), campagnes `isaac`.

**`alacod-sim`** : `--floors <séquence> --until-floor N` existait (T1.14) ; le JSON gagne `outcome`
(`victoire`, `objectif`, `mort`, `soft-lock`, `plafond`, `desync`) et `rooms` (par étage : salles typées,
visitées, nettoyées, verrouillantes) ; un résumé par issue (graines) finit la sortie d'erreur. `--until-floor N`
s'arrête dès que `FloorState::index >= N` (index 0-based) : **atteindre l'étage 2 = `--until-floor 1`**. La
campagne : `alacod-sim --game gungeon --bots 1 --profiles explorateur --floors etages_court_deux --seeds A..B
--until-floor 1 --max-frames 20000`.

**Scénarios** : `bot_explorer_floor` (un bot, graine 1) et `bot_explorer_two` (deux bots) jouent
`etages_court_deux` sans input scripté ; ils vérifient l'état des salles, `FloorIndex` 1 et la survie. La
carte de l'étage suivant est chargée à l'avance : `RoomState` y désigne les salles de l'étage joué par
`nth: 1`. Les scénarios `gungeon_floor_seed_*` (atteinte scriptée) ne changent pas.

## 48. Butin, coffres et clés (M2-T5, chantier C4)

Où les objets (§36, §44) apparaissent : un tirage pondéré, seedé, **indépendant du moment**, des coffres
(dont certains à clé) et une récompense à la fin d'une salle de combat. Code : `items::loot` (contrats et
tirage pur), `game::loot` (systèmes), `map_ldtk::game::rooms::room_chest_system` (coffres des types de salle).

**Pool de butin** (kind `LootTable`, `loot/<id>.ron`, id = nom de fichier ; `game.ron` : `(path: "loot", kind: "LootTable")`) :

```ron
(
    entries: [(pick: Item("key"), weight: 3), (pick: Rarity(Rare), weight: 1)],
    exclude_held: false,                                   // défaut : faux
    reward: Some((chance: "0.33", chest_chance: "0.25")),  // récompense de salle (chest_rarity: Common)
)
```

- `pick: Item(id)` : un objet précis ; `pick: Rarity(r)` : **un** objet de cette rareté, uniforme parmi ceux de
  `items/` (le poids est celui de l'entrée, pas par objet : les rares restent rares quel que soit leur nombre).
- **Tirage** (`items::draw_loot`) : deux `next_u32` toujours consommés (l'entrée par poids, puis le candidat),
  entrées et candidats triés (BTreeMap), donc le flux avance de la même façon quels que soient les filtres.
- **Exclusion des objets tenus** : décision = **opt-in par pool** (`exclude_held`, faux par défaut). Elle rend
  le tirage dépendant de l'inventaire de l'ouvreur, donc du moment ; les pools de `gungeon` ne l'activent pas.
  Un actif tenu compte comme tenu. Si le filtre ne laisse rien, retombée sur le pool sans filtre de rareté ni
  d'exclusion (sauf récompense de salle : un consommable ou rien).
- **Rareté d'un coffre** (`Chest::rarity`) = rareté **minimale** de l'objet tiré (un consommable compte pour `Common`).

**Flux RNG dédié, indépendant du temps.** `items::loot_seed(graine de la partie, étage, clé de contenu)` =
FNV-1a de `loot:<graine>:<étage>:<clé>` replié sur 32 bits puis passé au finaliseur `fmix32` (D95 : sans lui, `champion:41` et
`champion:42` restaient corrélés, r jusqu'à 0,063 ; `loot_seed_numbered`/`loot_seed_parts` hachent les mêmes octets sans
construire la clé) ; le tirage lit un **`RollbackRng` local** de cette
graine. Il ne touche **aucun** flux de `RngStreams` : le flux nommé `"loot"` des power-ups est consommé à chaque
mort d'ennemi, il ferait dépendre l'objet du moment d'ouverture (et changerait le hash de la ressource, comme
`variant.rs` le note déjà). La clé de contenu est celle du coffre (`room:<salle>`, `reward:<salle>`, ou la clé
d'un placement de scénario), jamais son `Entity` ni son `GgrsNetId`. Résultat : un coffre ouvert à la frame 30 ou
à la frame 3000, par l'un ou l'autre joueur, donne le même objet (scénario `loot_seeded` : deux coffres de même
clé ouverts à 120 frames d'écart donnent le même objet ; `loot_seeded_other` : une autre graine, un autre objet).

**Pool d'un étage.** `Floors` : champ optionnel `loot: ["commun", "etage2"]` (index = étage, le dernier vaut
pour les suivants) → `game::loot::FloorLootTables` (hors rollback, posé par `map_ldtk::game::floors`). Un coffre
sans `table` utilise celui de l'étage, sinon `commun` ; la récompense de salle utilise celui de l'étage **s'il
définit `reward`**, sinon `salle`. Lint : pool inconnu = `BrokenReference`.

**Coffre** (`items::Chest { key, table, rarity, locked, opened }`, rollback + checksum + trace en variante
**neutre**) : entité au sol avec `Interactable { Chest }` (`InteractionType::Chest`, variante en fin d'enum :
les hash existants ne bougent pas), `GgrsNetId` par clé de contenu (`chest_<clé>`). `chest_interaction_system`
(`Interaction`, après `handle_item_interaction`) : le premier joueur (ordre `GgrsNetId`) l'emporte. **Verrouillé**
(`locked`) : consomme **un** consommable `key` ; sans clé, refus (rien ne change, `ggrs{chest_refused}`). Ouvert :
un `ItemPickup` au sol à 16 unités, `opened = true`, `Interactable` retiré (le prompt et la détection n'y
reviennent pas), `FrameEvents<ChestOpened>` (neutre). Un coffre ouvert reste en place.
Prompt HUD (source `prompt`) : « Ouvrir le coffre », « Fermé — 1 clé » (verrouillé, sans clé), « Ouvrir — 1 clé ».

**Coffre d'un type de salle.** `rooms/<id>.ron` : champ optionnel `chest: Some((table: None, locked: false,
rarity: Common))` (`world::RoomKindDef`, qui perd `Copy`). `room_chest_system` pose le coffre au centre de chaque
salle de l'étage courant qui n'en a pas encore (clé `room:<salle>`) : aucune entité LDtk à ajouter, tout
gabarit d'un type donné en reçoit un. `gungeon` : `recompense` (verrouillé, rareté mini Uncommon, donc au moins un
coffre à clé par étage dans la grammaire `etage` qui exige une récompense) et `boutique` (ouvert, en attendant M2-T6).

**Récompense de salle** (`room_reward_system`, après `Run`) : une salle de type `combat` qui passe `Cleared`
(`FrameEvents<RoomChanged>`) laisse, avec la chance `reward.chance` du pool (forcée par
`Scenario::room_reward_chance_override`), au centre de la salle : un coffre (`chest_chance`, pool de l'étage ou
`commun`) ou un consommable tiré dans le pool (`consumables_only`). Graine : clé `reward:<salle>`. Les clés
tombent donc comme récompense (`gungeon` : `salle.ron`, clé de poids 3).

**Lint** : pool vide, poids ≤ 0, objet inconnu (`BrokenReference`), rareté sans objet, chances hors de 0..=1,
coffre de salle à pool inconnu, **coffre verrouillé dans un jeu sans consommable `key`** (`BrokenReference`),
pool inconnu dans `loot` d'une séquence. Fixtures `loot_empty_table`, `loot_weight_zero`, `loot_unknown_item`,
`loot_locked_chest_without_key`.

**Scénario** : `Scenario::chests` (`ChestPlacement` : clé, pool, rareté, verrou, position, frame), comme `items` ;
`room_reward_chance_override`. Attente `ItemCount { handle, id, count, at_frame }` (exemplaires d'un objet tenus :
0 prouve un refus, 2 un même objet tiré deux fois). Scénarios : `loot_seeded`, `loot_seeded_other`, `chest_locked`,
`room_reward`, `loot_floor_table` (testbed : pools `commun`/`etage2`/`salle`, séquence `deux_butin`).

**Hors périmètre** : le butin du boss (table `boss` de `gungeon` prévue, branchée par M2-T7), la boutique (T6),
le HUD des clés (T18). Les deux types de salle `recompense` et `boutique` de `gungeon` gagnent un coffre : les
scénarios qui jouent ces gabarits (étages `etage`/`grand`) changent de trace, preuve dans le rapport.

## 51. Profil et déblocages qui enrichissent le butin (M2-T12, chantier G1)

**Kind `Unlock`** (`unlocks/<id>.ron`, `items::UnlockDef`) : `name`, `cost: Option<i64>` (essence ;
`None` = gratuit, conditions seules), `requires` (déblocages préalables), `stat: Option<(nom, n)>`
(statistique de profil minimale) et `grants` : `Item(id)` (l'objet quitte **tous** les pools tant que
le déblocage manque) ou `Loot(table, pick, weight)` (une entrée de plus dans le pool). Kind
`MetaEconomy` (`meta/essence.ron`, `(per_run, per_kill, per_victory)`) : l'essence gagnée par run,
qui remplace les règles provisoires de §38 (mêmes valeurs ; défaut identique sans fichier).

**`RunUnlocks`** (`items::RunUnlocks`, `BTreeSet<String>`) : les déblocages **de la partie**, figés au
lancement, ressource rollback **neutre** (`rollback_and_trace_resource_neutral`, contribution 0 si
vide) et statique pendant la run. Seule donnée du profil lue par la simulation.
- **Local** : union des profils des joueurs locaux (`profile::local_run_unlocks`).
- **En ligne** : le profil de l'**hôte** (plus petit handle), transmis avec la configuration de
  session comme la graine (`ALACOD_RUN_UNLOCKS`, ids séparés par des virgules ; jamais le profil
  du pair local). Choix de l'hôte plutôt que de l'intersection : un seul ensemble à transmettre,
  et un joueur qui rejoint n'appauvrit pas la partie.

**Tirage** (`items::draw_loot_with`) : retire les objets « hidden » (réservés à un déblocage absent)
et ajoute les entrées `Loot` des déblocages actifs ; le RNG consomme toujours ses deux `next_u32`,
donc une même graine donne la même suite de tirages avec ou sans déblocage, et seul l'objet change.
Ensemble vide et jeu sans `unlocks/` : comportement d'avant.

**Achat** hors simulation : `Profile::purchase` (déjà possédé, prérequis, statistique, essence ;
écriture atomique) et `alacod-profile unlock games/<jeu> <id> [--handle n]` (`list` pour l'état).

**Lint** (`lint_unlocks`, fixtures `unlock_*`) : objet ou table inconnus, prix ≤ 0, déblocage
préalable inconnu, cycle de prérequis.

**Contenu `gungeon`** : `tresor_trefle` (20 essence, retire le trèfle des pools), `etendard_royal`
(40, statistique `runs` ≥ 1, retire l'étendard), `coffre_riche` (60, exige le trèfle, entrée Rare
dans `commun`). `zombies`, `testbed`, `throne` : inchangés.

**Scénarios** : `Scenario.profile_setup` (profils de départ et achats, `profile: true`),
attente `RunUnlocks { has, lacks, at_frame }`. `unlock_enriches_pool` / `unlock_base_pool` (même
graine, mêmes coffres, l'un avec les trois déblocages : « amulette » ×2, l'autre sans :
« amulette » + « bottes »), `unlock_purchase`, `unlock_two_players_host` (union locale).

**Traces** : aucune trace existante n'a bougé.

### Armes aux sources (D62, `m2-armes-sources`)

Une arme a une source quand un pool de butin la nomme. Les dix-huit armes de `gungeon` (hors
`pistolet`, arme de départ, et `arsenal*`, contenu de test `test_only: true`) sont dans les pools.

- **Entrée de pool** `LootPick::Weapon("id", Rareté)` (`items::loot`). Le tirage la désigne par
  l'identifiant `weapon:<id>` (`items::loot_weapon`) ; la rareté joue comme celle d'un objet (filtre
  de rareté d'un coffre, prix d'un présentoir). Une récompense de salle sans coffre (consommables
  seulement) l'ignore. Le flux RNG avance comme avant (deux `next_u32` par tirage) : seul le candidat
  change.
- **Coffre** : l'arme tombe à côté du coffre (`game::loot::spawn_loot_weapon`), arme lâchée ordinaire
  (`price: None`, chargeur plein), ramassée par l'interaction habituelle, qui échange avec l'emplacement
  (`weapon_slots`, une seule arme pour le `pistolero`) : l'arme posée tombe à son tour.
  `ChestOpened::item_id` vaut `weapon:<id>`.
- **Boutique** : prix par rareté de l'arme (`economy.ron`, `shop_prices`). Acheter un présentoir
  d'arme la pose au pied de l'acheteur (log `shop_buy item=weapon:<id>`), à ramasser ensuite.
  `RoomShopDef` gagne `weapon_shelves` (les N premiers présentoirs ne tirent que des armes du pool :
  au moins une arme par boutique) et `floor_tables` (pool de chaque étage, le dernier vaut pour les
  suivants ; vide : `table`).
- **Pondération par étage** : `boutique` puis `boutique_profond` (armes rares plus lourdes), et
  `commun` puis `commun_profond` pour les coffres (`floors/etages_deux.ron`, champ `loot`).
- **Échange d'emplacement** : l'arme lâchée par un échange ne peut pas être ramassée pendant
  `WEAPON_SWAP_COOLDOWN_FRAMES` (30) frames (`WeaponPickup::can_buy_after_frame`, jusque-là réservé aux
  armes murales). Sans ce délai, Interaction maintenu rééquipait à chaque frame l'arme qu'on vient de
  poser (bug trouvé par le bot, emplacement unique).
- **Lint** : arme inconnue (`BrokenReference`), `weapon_shelves` > `shelves`, `floor_tables` inconnu,
  `weapon_shelves` sans arme dans le pool. `reach::loot_pool_weapons` compte les armes des pools :
  `alacod lint games/gungeon` n'a plus d'avertissement d'arme.
- **Atteignabilité** : le bot `explorateur` gagne l'option `ExplorerOptions::shopping`
  (`alacod-sim --shopping`) : ouvrir les coffres (clé en main si verrouillés) et acheter les
  présentoirs payables sont des buts d'exploration (Interaction sur la surface) ; l'arme qui tombe
  est ramassée par le combat de `prudent` (butin au sol). Faux par défaut : aucune trace de bot ne
  bouge. Test `explorer_finds_weapons_on_ordinary_floors` (`crates/scenario/tests/bots.rs`,
  `ALACOD_SHOP_SEEDS`, `ALACOD_SHOP_FRAMES`) : partie `etages_deux`, graine 3, une arme autre que le
  pistolet achetée à f69. Scénario `shop_weapon` : achat puis échange, bout en bout.

## 50. Boutique et monnaie `shells` (M2-T6, chantier C5)

Les ennemis lâchent des douilles (`shells`), dépensées sur les présentoirs d'une salle `boutique`.
Code : contrats dans `items::loot` (`Shelf`, `ShellPickup`, `RoomShopDef`), systèmes dans
`game::shop`, pose des présentoirs dans `map_ldtk::game::rooms::room_shop_system`.

**Monnaie au sol.** `economy.ron` : `shells_drop: Some((min, max))` (inclus, `None` par défaut :
aucun drop ; `min > max` refusé par le lint). `shell_drop_on_death_system`
(`DeathManagement`, même place que le drop de power-ups) tire le montant dans un `RollbackRng`
**local** de graine `items::loot_seed(graine, étage, "shells:<GgrsNetId du mort>")` : aucun flux de
`RngStreams` (le flux `"loot"` des power-ups garde sa consommation, donc ses traces). Équipe
`Enemies` seulement, jamais un baril. Le tas (`ShellPickup { amount }`, rollback **neutre**,
`GgrsNetId` `shells_<id du mort>`) se ramasse au contact (`shell_pickup_system`, `Effects`, portée
20, le plus petit `GgrsNetId` à portée) : `Currency::earn` + `CurrencyEvent` (raison `pickup`),
`despawn_rollback()`. Les points de kill (zombies) sont une autre source, inchangée.

**Boutique.** `rooms/<type>.ron` : `shop: Some((table: "boutique", shelves: 3))` (`world::RoomKindDef`).
`room_shop_system` pose, une fois par salle de ce type que l'étage présente, `shelves` présentoirs
alignés au centre (écart 32). Objets : `shop::draw_shelves`, tirage dans le pool `loot/<table>.ron`
avec un `RollbackRng` local de graine `loot_seed(graine, étage, "shop:<salle>")` — même mécanisme
que les coffres (§48), donc indépendant du moment et de qui entre ; un objet déjà tiré est redemandé
(8 essais au plus). Prix : `economy.ron` `shop_prices: {Common: 15, Uncommon: 30, Rare: 50}` par
rareté de l'objet (absent : `shop::DEFAULT_SHELF_PRICE` = 10). La salle n'est jamais verrouillée.

**Achat.** `items::Shelf { key, item_id, price, sold, retry_after }` (rollback **neutre**,
`GgrsNetId` `shelf_<clé>`, `Interactable { InteractionType::Shop }` — variante en fin d'enum),
`shelf_interaction_system` (`Interaction`, après le coffre) : `Currency::spend(prix)` puis l'objet est
donné comme un ramassage (passif : liste + modificateurs ; actif : remplace l'actif tenu, qui tombe
au sol ; consommable : stock ou effet immédiat) et `ItemPicked` est émis. `sold = true`,
`Interactable` retiré. Solde insuffisant : rien ne change, `CurrencyEvent { delta: 0, reason:
"shop_refused" }` et anti-rebond `retry_after = frame + 30`. Achat : `delta = -prix`, raison `shop`.
Prompt HUD (source `prompt`) : « Acheter Trèfle — 30 » ; rien pour un présentoir vendu.

**Lint.** Prix de `shop_prices` à 0, `shells_drop` min > max (`OutOfRange`) ; `shop` sans présentoir
(`shelves: 0`), pool de boutique vide (`OutOfRange`), pool inconnu (`BrokenReference`). Fixtures
`shop_price_zero`, `shop_shells_inverted`, `shop_no_shelves`, `shop_empty_table`, `shop_without_table`.

**Scénarios.** `Scenario::shop` (`ShopSetup`) : `shelves: [(key, item, price, x, y, at_frame)]` pose
des présentoirs, `shells_drop_override: Some((min, max))` force le drop (le testbed n'a pas de
`shells_drop` : ses traces ne bougent pas). Attentes : `Shelf(nth, item, price, sold, at_frame)`
(présentoirs triés par clé), `ShelfCount(count, at_frame)`, `ShellPiles(count, at_frame)`.
Scénarios : `shells_drop`, `shop_buy`, `shop_refused` (testbed), `shop_seeded` (gungeon, grammaire
`etage`).

**Hors périmètre** : marchand PNJ et déblocages méta (M2-T12, T13), vol, réduction de prix, HUD des
shells (M2-T18).
D68 : les drops de mort (douilles, power-ups et armes de progression) interrogent `With<Death>`
avant `rollback_apply_death`, qui désactive immédiatement le mort. Le drop reste unique et se
rejoue après restauration d'une mort posée en fin de frame (arène) ; les ticks `Added` ne font pas
partie du snapshot rollback et ne peuvent pas servir à détecter cette mort.


## 52. Contenu `gungeon` (M2-T14, quatre parties)

### 52.a Armes (T14a)

Données seulement (`games/gungeon/assets/weapons/weapons.ron`), aucune mécanique ajoutée au moteur.
Vingt armes à distance au total (compte du lint : `pistolet`, 18 nouvelles, `arsenal` des ennemis),
plus `bare_hands` (mêlée). Chacune compose des comportements existants et porte un `test:` (scénario
généré `weapon_<arme>`, `make gen GAME=gungeon`) avec bornes `min_hits`/`max_hits` mesurées.

| Arme | Munition | Caractère | Composition |
|---|---|---|---|
| `pistolet` | balles | arme de départ, cadence 3/s | `Automatic`, `Standard` |
| `revolver` | balles | coup lent et lourd | `Manual`, 30 dégâts, chargeur 6 |
| `pistolet_precis` | balles | balle perçante | `Piercing(1)` + `Pierce(1)` |
| `pistolet_jumele` | balles | deux balles par pression | `Burst(2)` |
| `carabine_rafale` | balles | rafale de trois | `Burst(3)` |
| `mitraillette` / `mitrailleuse_lourde` | balles | tir continu 10 et 14/s, dispersion | `Automatic`, `spread` |
| `rebondisseur` | balles | rebondit sur les murs | `Bounce(3)` |
| `fusil_pompe` / `double_canon` | cartouches | cône de 8 / 10 plombs | `Shotgun` |
| `fusil_precision` | calibre | traverse trois ennemis, très lent | `Piercing(3)` + `Pierce(3)` |
| `arc` / `arbalete` | fleches | flèche lente perçante / carreau lourd | `Piercing`, `Pierce`, `Size` |
| `laser` | energie | faisceau rapide qui traverse tout | `Automatic`, `Pierce(8)`, `Lifetime(30)` |
| `chercheur` | energie | tête chercheuse | `Homing("0.1")` |
| `boule_energie` | energie | gros projectile lent | `Size("3.0")`, `Pierce(8)` |
| `lance_roquettes` | roquettes | explose à l'impact | `on_expire: Spawn(Aimed → souffle)` |
| `grenadier` | roquettes | rebondit puis explose en éclats | `Bounce(2)`, explosion + `Ring` (§16) |
| `lance_flammes` | carburant | jet continu à portée courte | `Automatic`, `Lifetime(40)`, `Pierce(2)` |

Munitions : réserves distinctes par type `Custom` (`balles`, `cartouches`, `calibre`, `fleches`,
`energie`, `roquettes`, `carburant`). Pas de munition infinie dans le moteur (D à venir) : seul
`pistolet` a une réserve de 9999 chargeurs ; les autres ont une réserve finie.
`Burst` ne tire pas plusieurs balles par frame mais `pellets_per_shot` balles espacées de
`cooldown_frames` (voir `FiringMode::Burst`). Les traces `gungeon` existantes n'ont pas bougé
(les 5 scénarios générés d'avant sont identiques après `GEN_BLESS=1`).

### 52.b Objets et synergies (T14b)

Données seulement (`games/gungeon/assets/{objets,synergies}/`), aucune mécanique ajoutée. Vingt objets
au total (dix de plus), huit synergies (quatre de plus : la fiche dit « quatre synergies », lue comme
quatre en plus des deux de M2-T4). Les pools de butin (`commun`, `boutique`, `boss`) tirent par rareté :
les nouveaux objets y entrent sans toucher aux pools ; le prix en boutique suit la rareté (§50).

| Objet | Rareté | Genre | Effet |
|---|---|---|---|
| `gilet` | Commun | passif | `OnPickup` (une charge) : +25 d'armure (D61) |
| `egide` | Commun | consommable | `GainShield(1)` au ramassage (D61) |
| `lunettes` | Commun | passif | `Range` +20 % |
| `chargeur_rapide` | Commun | passif | `ReloadSpeed` +30 % |
| `roulade` | Peu commun | passif | `OnDodge`, cooldown 60 : cadence +30 % pendant 120 frames |
| `trophee` | Peu commun | passif | `OnKill`, cooldown 120 : soin 2 |
| `parchemin` | Peu commun | passif | `OnBlank`, cooldown 300 : `GainShield(1)` |
| `sablier` | Peu commun | actif `Frames(1800)` | vitesse +50 % pendant 240 frames |
| `bastion` | Rare | passif | vie max +10 ; `OnFloorStart` : `GainShield(1)` |
| `rancune` | Rare | passif | `OnDamageTaken` sous 50 % de vie, cooldown 600 : dégâts +50 % pendant 180 frames |

Synergies : `rempart` (`gilet` + `bastion` : +25 d'armure), `fuite_armee` (`bottes` + `roulade` : vitesse
+10 %), `sang_froid` (`trophee` + `rancune` : dégâts +20 %), `tireur_d_elite` (`lunettes` + `chargeur_rapide` :
dégâts +15 %).

**Piège (armure).** `Armor` n'a pas de valeur de base chez le joueur : le générateur (`expected_item_stats`)
refuse (`MissingStat`) un passif dont `modifiers` touche `Armor`. `gilet` pose donc son armure par un effet
`OnPickup` (source `item:gilet:0`) ; `rempart` (synergie, sans `test:`) peut l'utiliser en `modifiers`.
Les consommables (`egide`) n'ont pas de `test:` (comme `ration`, `blank`).

### 52.c Ennemis et boss (T14c)

Données seulement (`games/gungeon/assets/`), aucune mécanique de moteur.

**Ennemis** (`characters/`, un `test:` chacun, scénarios générés `enemy_<id>_still`/`_moving`) :
`shotgun_kin` (`Shoot` de `pompe` : télégraphe 25 puis sept chevrotines en `Fan`, portée 170 ; variante
`blinde`), `veteran_kin` (`visee` + `KeepDistance` 140-220 ; variantes `rapide` et `blinde`),
`bandana_kin` (`Charge`, télégraphe 30, 15 de dégât), `tourelle_kin` (immobile, `croix` : deux couronnes de
quatre boules), `fuyard_kin` (`Flee` sous 50 %, `rafale` de quatre balles ; variante `rapide`),
`lanceur_kin` (`KeepDistance` 150-240, `grenade_lente`). Avec `bullet_kin`, `bullet_kin_squad` et le boss
`gatling` : dix ennemis. Patterns nommés `pompe`, `rafale`, `croix`, `grenade_lente` ; arme ennemie
`arsenal_lourd` (table `chevrotine`, `grenade` → `explosion` → `eclat`), à part de `arsenal` pour que le hash
des ennemis existants ne bouge pas.

**Placement** (aucun gabarit ajouté, `docs/taches/rapports/m2-t14c-ennemis.make_placement.py`) :
`CombatEW` (+`shotgun_kin`, `fuyard_kin`), `CombatNS` (+`veteran_kin`, `tourelle_kin`), `CombatCroix`
(+`bandana_kin`, `lanceur_kin`), vagues de l'arène (`fuyard_kin` à 300, `bandana_kin`, `lanceur_kin`,
`tourelle_kin` à 600). `CombatT` et `EscouadeEW` ne bougent pas (les scénarios scriptés de l'étage `court` s'y
appuient). `alacod lint gungeon` : aucun nouvel avertissement d'ennemi.

**Boss `gatling`** : phase 1 = couronne puis spirale 60 frames plus tard (répétition 150) ; phases 0 et 2
inchangées. Un renfort `lanceur_kin` en phase 2 a été essayé : le bot meurt et `gungeon_boss_bot` dépasse son
`EntityCount`, retiré.

### 52.d Gabarits de salles et équilibrage de l'étage, D64 (T14d)

Données seulement (`games/gungeon/assets/`, scénarios, traces gungeon), aucun code de moteur ; le test de
grammaire `map_ldtk::generation::tests` a ses constantes mises à jour (bornes de `etage`, types exigés,
gabarits décorés). Scripts : `docs/taches/rapports/m2-t14d-gabarits.make_gabarits.py` (gabarits et couche
`Surfaces`) et `.make_ennemis.py` (composition des ennemis par gabarit), idempotents. Rapport et mesures :
`docs/taches/rapports/m2-t14d-gabarits.md`.

**Quinze gabarits** (8×8 cases, ouvertures de deux cases au milieu des côtés) : `Depart`, `CombatEW/NS/T/Croix`,
`Recompense`, `Boutique`, `BossW/N`, `ArenaEW/Croix`, `EscouadeEW` (douze avant T14d) et trois nouveaux :

| Gabarit | Type (`room_kind`) | Ouvertures | Contenu |
|---|---|---|---|
| `Coffre` | `coffre` | N E S W | trois tables, sable (`Surfaces`) autour du coffre central |
| `Secret` | `secret` | W (cul-de-sac) | trois tables, eau et fosse dans les angles |
| `ArenaNS` | `arene` | N S | deux barils et une table (les vagues viennent de `rooms/arene.ron`) |

Types de salle nouveaux (`rooms/`) : `coffre` (non verrouillant, coffre ouvert au centre, rareté mini Uncommon, pool
de l'étage) et `secret` (idem, rareté Rare, cul-de-sac). **« Secret » ne l'est que par sa place** : le mur secret
(`DoorLock::Secret`, §62, exécuté par M3-T4) exige une bombe et `gungeon` n'en donne aucune (D81) : un mur plein
que personne ne casserait. Un coffre de salle s'ouvre par Interaction : le
bot ne le fait qu'avec `alacod-sim --shopping`. Les tables ne sont jamais posées devant une ouverture (une table
sur les deux cases d'ouverture murerait la salle). Les accessoires sont toujours trois par salle décorée (le test
de décor compte `CombatEW`, `CombatNS`, `Coffre`, `Secret`, `ArenaNS`).

**Surfaces dans `gungeon`** : dossier `surfaces/` (`eau` 1, `sable` 2, `fosse` 4, copies de celles du testbed ;
pas de glace, aucun gabarit n'en pose), couche IntGrid `Surfaces` ajoutée au projet de `salles.ldtk` et aux
seuls niveaux `Coffre` et `Secret` ; la couche suit le gabarit dans l'étage assemblé (sonde `map_probe` : « fosse :
3 cases » sur la graine 28). `SurfaceGrid` est neutre quand elle est vide : les étages sans gabarit à surface
gardent leur hash.

**Grammaire `etage`** : de 10 à 14 salles, types exigés `boss`, `recompense`, `boutique`, `arene`, **`escouade`**,
`coffre`, `secret` (une fois chacun) ; le reste en `combat` (2 à 6 salles). La salle `escouade` de M2-T8 n'existait
que dans la grammaire `court_escouade`. Validée sur 1 000 graines par `gungeon_etages_de_grammaire`
(`ALACOD_FLOOR_SEEDS=1000`) et `gungeon_decor_sur_1000_graines`. `court`, `grand` et `court_escouade` ne
bougent pas (leurs scénarios scriptés gardent leur forme). `court_arene` et `court_coffre` (départ, la salle, boss)
portent les `test:` de `rooms/arene.ron` et `rooms/coffre.ron` ; `rooms/escouade.ron` joue `etages_court_escouade`.
Scénarios générés `room_arene`, `room_coffre`, `room_escouade`, `room_secret` (grammaire `court_secret`, 5 salles).

**D64, équilibrage** (leviers, tous en données). Un `bullet_kin` : 30 PV (40), salve de trois balles simples de
4 dégâts (trois salves de deux, 6 dégâts), recharge de 150 frames (90), `ricochet` : `Bounce(1)`, portée 320 (500) ;
idem `bullet_kin_squad`. Autres ennemis : `crachat` 4 (6), `chevrotine` 3 (5), explosion de `grenade` 10 (14) et
éclat 3 (4), `bandana_kin` 10 de dégât (15), `tourelle_kin` 60 PV (90) et `croix` avec la nouvelle boule `boulette`
(4 dégâts : `boule` reste celle du boss, non touchée). Gabarits (ennemis avant → après) : `CombatEW` 2 `bullet_kin` + `shotgun_kin` + `fuyard_kin` → 2 `bullet_kin` + `fuyard_kin` ;
`CombatNS` 2 `bullet_kin` + `veteran_kin` + `tourelle_kin` → 2 `bullet_kin` + `veteran_kin` ; `CombatCroix` 3 `bullet_kin` +
`bandana_kin` + `lanceur_kin` → 2 `bullet_kin` + `shotgun_kin` ; `CombatT` (3 `bullet_kin`) et `EscouadeEW` inchangés ; positions
aux coins de la salle. Arène :
horloge 750 (900), vagues à 0/250/500 (0/300/600), un `bullet_kin` au lieu de deux, un au lieu de trois. Soin :
chaque salle `combat` nettoyée laisse une récompense (`salle.ron` : chance 1, ration 6/9) et la `ration` rend 25 PV (15).
Joueur, boss, armes du joueur : intouchés (le renfort `bullet_kin` du boss profite de l'allègement).

**Mesures** (`alacod-sim`, `etages_deux`, 200 graines, explorateur, `until-floor 1`) : à 1 bot 200 morts sur 200 → 54
(27 %) ; à 2 bots 174 → 7 (3,5 %) ; 0 desync. Soft-locks : 0 avant, 4 sur 200 à 1 bot après, 0 à 2 bots ; cause
et pistes dans le rapport (bot : cycle limite devant une ouverture de deux cases, dette D82).

**Graines** (2 bots, `etages_deux`, `--shopping` ; `docs/taches/rapports/m2-t14d-gabarits/rencontre.py`) : 41 :
coffre secret f369, coffre f1045, escouade f1096-f1233, arène f1476-f2226, boutique f2334, baril f604 ; 13 : secret
f1100, coffre f3392, arène f2064-f2814, escouade f2971-f3257, boutique f1663.

## 53. Lint des kinds M2 et atteignabilité statique (M2-T15, voie V2)

Lint seulement : aucune simulation touchée, aucune trace ne bouge.

**Avertissements.** `content::warnings(&registry, &manifest) -> Vec<LintWarning>` (`crates/content/src/reach.rs`,
kinds `Unreachable` et `UnreadStat`). `alacod lint` les liste sur stderr (« avertissement : … ») et le
code de sortie ne change pas ; `LintError`/`load_and_lint` sont inchangés.

**Stat sans lecteur.** `sim_core::stats::stat_reader(&StatId) -> Option<&'static str>` : liste fermée
des stats lues par la simulation, avec l'endroit de lecture ; à tenir à jour **à la main** quand un
système lit une nouvelle stat. Un modificateur (ou une base `stats:`) sur une stat sans lecteur est une
erreur `LintErrorKind::UnreadStat` (`Luck`, tout `Custom` sauf `powerup_currency_multiplier`).
`Armor` est lue depuis D61 (§18, `character::health`) : plus d'exception.
Usages inspectés : `stats` et `effects` des personnages, variantes, perks, objets, synergies, mutations,
power-ups.

**Atteignabilité.** Racines : `entry.start_map` et chaque niveau de chaque séquence `Floors` (carte,
`cave:<id>`, `floor:<id>`). Il n'y a pas de séquence « par défaut » stable (le mode `Floors` prend la
première par ordre d'id, et `etage_1` de gungeon est une carte fixe) : toute séquence déclarée compte.
Types tirables d'une grammaire = `FloorGrammar::kinds_needed()`. Avertissements :
- type de salle (`rooms/<id>.ron`) qu'aucune grammaire ni carte posée ne peut tirer ;
- gabarit d'une carte `templates` dont le type n'est tiré par aucune grammaire atteinte ;
- ennemi (`ai` présent) absent des niveaux jouables (cartes posées, gabarits tirables), des arènes
  des salles tirables, des vagues, des cavernes et des renforts de boss (point fixe) ;
- objet sans source : `starting_consumables`, `consumables_per_floor`, **pool** (`reach::loot_pool_items`,
  `loot_pool_weapons` : branchés depuis M2-T5 et D62), déblocages (M3-T12, §77) ;
- arme sans source : `starting_weapons`, `WeaponLocation` des niveaux jouables, `weapon_pool` de
  progression, pools, armes citées par les règles des ennemis atteints (`Shoot`, phases de boss,
  `SpawnPattern` des effets, entrées de phase et frises). Un ennemi inatteint ou `test_only`
  ne compte pas comme source de ces armes.
`MapEntry::levels` (`MapLevelInfo` : type de salle, personnages, armes murales par niveau LDtk) porte
les données. Limites : pas de lecture des scénarios (une arme ou un objet donné par scénario seulement
est signalé), ni des cartes assemblées autrement que par les désignations ci-dessus ; la sandbox
`testbed` (arènes de test) produit donc beaucoup d'avertissements voulus.

**Contenu de test (D63).** Champ optionnel `test_only: bool` (défaut `false`) : au sommet d'un fichier
de personnage (`test_only: true,`), dans `config:` d'une arme (`config: (test_only: true, …)`). L'analyse
l'ignore : pas d'avertissement « ennemi » ou « arme » pour lui, et un personnage marqué n'est pas une
source d'armes. Lu par `content` seulement (`CharacterEntry::test_only`, `WeaponEntry::test_only`) : les
types de `game` et `combat` l'ignorent comme tout champ inconnu, donc aucun effet en jeu et aucune trace
qui bouge. Marqués : `zombie_scaled` (zombies), `cible` et `mannequin` (throne), `cible` (gungeon). Fixtures
`reach_enemy_weapons` (règles atteintes, phases et frises ; sources inatteintes et de test exclues),
`reach_test_only_character` et `reach_test_only_weapon` (jumelles de `reach_enemy_unplaced` et
`reach_weapon_no_source`, test `test_only_content_is_ignored_by_reach`).

**Audit des kinds M2** (références vérifiées / bornes vérifiées / fixtures) :

| Kind | Références | Bornes | Fixtures |
|---|---|---|---|
| `rooms` | `room_kind` des cartes (`lint_room_kinds`) ; ennemis d'arène | arène : clock>0, vagues datées, ordre, count>0 | `room_kind_unknown`, `arena_*` |
| `floor_grammars` | carte `templates`, gabarit par type nécessaire | `problems()` (rooms, requis, distance) | `floor_required_without_template`, `floor_bounds_inverted` |
| `floors` | carte, caverne, grammaire, `consumables_per_floor` | levels non vide, cavernes sans ennemi | `floors_*` |
| `items` | `cost`, stats (Parse) | charge, pickup_range, blank, uses_per_floor, cost>0 ; **stat lue (T15)** | `item_*`, `stat_unread` |
| `synergies` | objets requis | ≥2 objets distincts, doublon d'ensemble | `synergy_*` |
| `props` | — | rayons, dégâts > 0 | `prop_barrel_radius_non_positive`… |
| `surfaces` | — | doublon de valeur, facteurs > 0, fosse | `surface_*`, `pit_*` |
| personnages boss | patterns, ennemis de renfort, pas de répétition sans fin | santé, phases, `until`, count>0 | `boss_*` |
| profil | `entry.progression`, horloges | — | `entry_*` |
| stats (T15) | — | liste fermée `stat_reader` | `stat_unread`, `stat_armor_warning` |
| atteignabilité (T15) | racines → salles/gabarits/ennemis/objets/armes | — | `reach_*` |

Trous comblés : la lecture des stats (nouvelle règle) et l'atteignabilité (nouveaux avertissements).
Aucun autre trou trouvé dans l'audit (les `props` et le profil n'ont pas de référence à vérifier).

**Fixtures.** `stat_unread` (erreur), `stat_armor_warning` (avertissement), `reach_room_unreachable`
(type de salle et gabarit), `reach_enemy_unplaced`, `reach_item_no_source`, `reach_weapon_no_source`.

## 54. Générateur v2 : gabarits par objet, par salle et par boss (M2-T17, voie V3)

Code : `crates/scenario/src/generate.rs` (gabarits), `game::gen_test` (`ItemTest`, `RoomTest`,
`BossFightTest`), `scenario::runner::apply_player_overrides` (objets de départ), `content::lint`
(`lint_item_tests`, `lint_room_tests`, `lint_character_tests`). Complète §28 : mêmes règles
(opt-in par un champ `test:`, aucun effet en jeu, un scénario écrit par `make gen GAME=<jeu>` dans
`tests/scenarios/generated/<jeu>/`, champ absent = rien de généré : aucune trace existante ne bouge).
Contrairement aux gabarits d'armes, ces gabarits jouent **dans le jeu énuméré** (le registre du
testbed n'a ni les objets ni les salles de `gungeon`) : carte de `generate_template` si le jeu en
déclare une (`Sandbox` imposé), sinon l'arène du testbed pour le testbed.

**Objet** (`objets/<id>.ron`, gabarit `ItemHeld`, fichier `item_<id>.ron`) :
`test: Some((frames: 60, expect: [...]))` ; `frames > 16` pour un actif.
- `PlayerScript::items: ["<id>"]` (nouveau, vide par défaut) : le joueur tient l'objet **avant la
  première frame**, comme après un ramassage (passif : `Inventory` et modificateurs `item:<id>` ;
  actif : emplacement actif **chargé** à `ActiveCharge::required()` ; consommable : compteur). Les
  effets `OnPickup` ne sont pas rejoués.
- Attentes par défaut : `HasItem` à la frame 1 ; une `Stat` par stat modifiée (frame 1), attendue =
  `sim_core::modifier::resolve(base, modificateurs)` où `base` est lue par une **sonde** d'une frame
  sans l'objet (`probe_player_stats`) : la valeur ne vient pas de la partie qui tient l'objet ; un
  actif : `ItemCharge(required)` à la frame 5, `UseActive` une frame à `ITEM_USE_FRAME` (10), puis
  `ItemCharge(0)` à la frame 16 (omis pour `Frames(n)`, qui se recharge aussitôt, et pour un actif à
  `cost`, dont l'usage peut être refusé). `test.expect` s'**ajoute** aux attentes par défaut.

**Salle** (`rooms/<id>.ron`, gabarit `RoomClear`, `room_<id>.ron`) :
`test: Some((frames: 500, floors: "etages_court", seed: 1, expect: []))` ; exactement un de `floors`
(séquence `Floors` du jeu) ou `map` (carte LDtk, `Sandbox` imposé si le manifeste est en `Floors`) ;
`seed` = graine de carte (défaut 123456) : le gabarit LDtk tiré par la grammaire est donc fixe.
Un bot `explorateur` (§47), aucun input scripté. Attentes par défaut : moments clés `room` « salle
<id> verrouillée » puis « salle <id> nettoyée » avant `frames` (`Event`, cumulatif : une attente
`RoomState` ponctuelle dépendrait de la frame exacte du bot), et `PlayerAlive` à la dernière frame.
Une salle qui ne verrouille ni n'est une arène (`locks: false`, pas d'`arena`) : seulement
`PlayerAlive`.

**Boss** (personnage qui a `boss:`, gabarit `BossFight`, `boss_<id>.ron`) : le `test:` du personnage
gagne le champ `boss: Some((floors: None, map: None, seed: 123456, expect: []))` (`boss: Some(())`
suffit), `frames` du `test:` fixe la durée, `still: false, moving: false` coupent les gabarits
ennemis. Sans `floors` ni `map`, la partie du manifeste (`entry.start_map`, comme `gungeon_boss_bot`).
Bot `fonceur`, aucun input. Attentes par défaut : un `Event(boss_phase, "phase i-1 → i")` par phase
atteinte (cumulatif : `BossPhase` échoue une fois le boss mort), puis `Event(room, "salle boss
nettoyée")` si le jeu a un type de salle `boss` (§45).

**Lint.** `frames > 0` ; objet : pas sur un consommable (`Unsupported`) ; salle : exactement un de
`floors`/`map`, scène connue, **au moins un gabarit LDtk du type** dans les cartes de la scène
(`BrokenReference`, y compris les gabarits d'une grammaire `floor:<id>`) ; personnage : `test.boss`
seulement avec `boss:`, `floors`/`map` exclusifs et connus, un gabarit actif parmi `still`, `moving`
et `boss`. Fixtures `item_test_frames_zero`, `item_test_consumable`, `room_test_no_template`,
`room_test_both_scenes`, `room_test_frames_zero`, `boss_fight_without_boss`.

## 55. Feedback v2 : télégraphes de patterns, flash du blank, fin de boss (M2-T19, voie V4)

Complète §31. **Tout est présentation** : rien de rollback, rien dans le checksum ni dans les
traces, aucune simulation ralentie (`Time<Virtual>` n'est jamais touché : ggrs avance dessus). Les
trois effets s'écrivent d'abord dans `FeedbackLog` (headless compris), le rendu (`FeedbackPlugin`)
applique ensuite les indices ; `scenario::events` en fait des moments clés `feedback`.

**Réglages** (`ui/feedback.ron`, champs optionnels de `content::feedback::FeedbackSettings`,
défauts sûrs : les trois effets sont actifs sans rien écrire, `enabled: false` en coupe un) :
- `pattern_telegraph: (enabled, color: Option<rgba>)` ; couleur absente : celle du cercle.
- `blank: (enabled, ring_frames, color, flash_alpha)` : 24 frames, bleu clair, flash 0,15.
- `boss_end: (enabled, freeze_frames, zoom, zoom_frames, flash_alpha, flash_color)` : 30 ticks
  de rendu gelés, zoom ×1,5 relâché sur 60 frames, flash 0,6.
Lint (`FeedbackSettings::problems`, `OutOfRange`) : durées dans `[1, 600]` (`freeze_frames`
`[0, 600]`, > 0 si l'effet est actif), opacités et couleurs dans `[0, 1]`, `zoom` dans `[1, 4]`.
Fixtures `feedback_blank_frames_zero`, `feedback_boss_zoom_out_of_range`,
`feedback_boss_freeze_zero`.

**Télégraphe de forme.** Pendant le `Telegraph` d'un émetteur (et le `Telegraph(n)` d'une frise de
boss, §45 : même chemin, l'émetteur joue `Sequence([Telegraph(n), pattern])`), `pattern_shape`
(fonction pure) lit les étapes **après** le télégraphe, jusqu'au prochain `Telegraph`/`Wait` :
`Fan`/`Spread`/`Aimed` → rayons répartis sur l'ouverture autour de la visée figée ; `Spiral` →
rayons de la première salve (l'amorce) ; `Burst`/`Scatter` → cône (deux rayons de bord et arc).
`Ring` n'a pas de forme (le cercle v1 le dit). Gizmos, longueur = rayon du télégraphe v1 (portée du
tir annoncé, bornée par `Shoot.range`). Un indice `PatternTelegraph` par (émetteur, étape), au
début du télégraphe. Moment clé : « télégraphe fan 31 (5 rayons) », « télégraphe burst 31 (cône
0.50 rad) » (`31` : `GgrsNetId` de l'émetteur).

**Blank.** Dérivé de l'état : les charges de blank (`Inventory::consumable` de l'objet blank de la
table) d'un joueur baissent d'un tick à l'autre. Anneau (`blank_ring_radius`, ease-out jusqu'au
rayon de `BlankSpec`) et flash plein écran léger ; il **remplace** l'onde de `LastBlank` (M2-T1)
quand `blank.enabled`. Moment clé : « blank 0 (rayon 160) » (`0` : handle du joueur).

**Fin de boss.** Dérivé de l'état, jamais de `BossDefeated` (transitoire) : un boss (`BossPlan`)
vu vivant au tick précédent a disparu **et** le nombre de salles `Cleared` (`RoomStates`) a
augmenté. Gel de rendu de `freeze_frames` ticks (`AnimationFreeze`, comme le hit stop), zoom sur la
position du boss (`boss_zoom_factor` : monte sur le premier cinquième puis revient à ×1 ; échelle
de projection et décalage de la caméra retirés avant le suivi de caméra, `undo_boss_zoom`),
flash plein écran (`ScreenFlashNode`). Moment clé : « fin de boss 40 » (`40` : `GgrsNetId` du boss).

**Preuve sans écran.** Attentes `Event(kind: "feedback", label_contains: …)`, aucune trace ne
bouge. À voir sur un poste avec GPU : voir le rapport `docs/taches/rapports/m2-t19-feedback-v2.md`.

**Limites.** Comme §31 : un tick qui simule plusieurs frames ne journalise que la dernière ; un
indice prédit puis annulé par un rollback p2p reste dans le journal. Une fin de boss se détecte au
tick où le boss disparaît : un boss retiré sans qu'une salle passe `Cleared` ne déclenche rien.

## 56. Écrans en bevy_hui et HUD gungeon (M2-T18, voie V4)

**Décision de William, 2026-10-15 : `bevy_hui` pour les écrans.** Le HUD reste la couche
maison `ui/hud.ron` (§15). Le comparatif historique et ses mesures restent dans
`docs/taches/rapports/m2-t18-ui-comparatif.md` ; la livraison de la seconde partie est décrite
dans `docs/taches/rapports/m2-t18-hud-gungeon.md`.

### Écrans

`game/ui_hui` est activée par `render`, défaut des quatre jeux natifs et wasm et du rendu
scénario. Les exécutables headless ne chargent pas Hui. Il n'y a plus de sélecteur de backend,
de renderer `screens/custom.rs` ni de fichiers pause/inventaire RON.

Le modèle pur (`screens/model.rs`) conserve `ScreensState`, `UiAction`, `reduce`,
`pause_view`, `inventory_view` et `ScreenView`. Les sections nommées sont liées génériquement :
`Text` → `{nom}`, `Bar` → `{nom_label}` et `{nom_width}`, `List` → conteneur `id="nom"`.
Chaque ligne utilise `ui/hui_row.html`, avec marqueur, couleur et fond du focus. Un changement
de vue remplit à nouveau les listes ; un rechargement HTML recrée leurs lignes.
`sync_screen` tourne **avant `HuiSystems::Build`** : aucune commande de Hui ne doit
construire une ligne déjà détruite lorsque l'inventaire change.

- Pause : Reprendre, Recommencer (`RunRequest::Restart`), Lobby (`RunRequest::ToLobby`),
  Réglages (« Aucun réglage pour l'instant. »).
- Inventaire : armes et munitions, actif et charge, passifs, consommables. Joueur affiché :
  celui de `HudPlayer` (caméra forcée, sinon local de plus petit handle).
- Navigation : Échap / Start, I / Select ; ↑ ↓ / D-pad ; Entrée ou Espace / A ;
  Retour arrière / B. `ALACOD_INPUT=neutral` coupe la lecture des touches.
- **La simulation continue et ses touches restent actives** sous les menus, comme avant :
  aucun verrou d'input ni changement de trace n'est introduit.
- **Modèle et entrées séparés** (D69 E5a, E7) : le modèle (`ScreensPlugin`, `HudModelPlugin`,
  `MutationScreenModelPlugin`, ajoutés un à un par `core_sim::CoreSimPlugin`, aussi en headless) ne lit ni
  clavier ni manette. `ui::screens::input::read_nav_keys` (clavier, manette) pose les actions
  dans la file `UiActionQueue` (`.before(update_screens)`), `ui::mutation_input::read_mutation_nav`
  (`ActionState<PlayerAction>`) écrit la ressource `MutationNavInput { handle, held, step }` que
  `update_mutation_screen_view` lit (`.before(update_mutation_screen_view)`, seulement pour le
  joueur affiché). Les deux lecteurs sont dans `GameUiInputPlugin`, ajouté par `core::CoreAppPlugin` après `GameUiPlugin` (lobby, déconnexion) ;
  les deux sont hors `PresentationPlugin` : le lobby tourne aussi en
  headless p2p, les lecteurs n'y trouvent rien. `GameUiModelPlugin` n'existe plus (E7).
- Écran de fin : `ui/game_over_screen.html`, dans les quatre jeux. Résumé dérivé de
  `Run.step`/`Run.summary`, mis à jour sans reconstruire la racine ; rollback de l'issue :
  disparition de l'écran. Rejouer (R) et Lobby gardent leurs `RunRequest`, boutons souris
  inclus. La présentation est chargée seulement dans `PresentationPlugin`.
- **Écran de mutation (D71, 2026-10-15)** : `ui/mutation_screen.html` dans throne et testbed,
  adaptateur `game::ui::mutation_hui` (`MutationHuiPlugin`, dans `PresentationPlugin`, feature
  `ui_hui`). `MutationScreenView` et le protocole sont inchangés : valider émet toujours le bit
  `ChoiceA/B/C` dans l'input (`confirm_input`), l'adaptateur ne lit que la vue. Propriétés du
  gabarit : `name{i}`, `desc{i}`, `tint{i}`, `border{i}` (carte surlignée en or) pour trois cartes,
  `bar_width` (barre de temps, en %). Écran dérivé de l'état : un rollback qui annule le choix le
  fait disparaître. `mutation_screen.ron`, `MutationScreenLayout` et son lint sont retirés (aucune
  trace ne bouge). Tests : `mutation_hui::gabarit_des_jeux_construit_et_recompile` construit les
  vrais gabarits de throne et testbed.

Les fichiers HTML sont découverts directement dans les dossiers `Ui`, sans compagnon RON.
Le registre vérifie leur lecture UTF-8 ; le lint exige aussi `ui/hui_row.html` lisible
si pause/inventaire le référencent. Fixtures `html_screen_missing_row` et
`html_screens_valid`. Les inventaires de texte embarqués pour wasm incluent `.html`.
Ce lint ne valide pas la grammaire complète de Hui.

### HUD gungeon

Nouvelles sources calculées par `hud_values` (pure), présentes dans `HudSnapshot` aussi
en headless et dans la liste fermée `content::ui::HUD_SOURCES` :

| Source | Valeur |
|---|---|
| `blanks` | compteur `Inventory::consumables["blank"]` |
| `keys` | compteur `Inventory::consumables["key"]` |
| `bombs` | compteur `Inventory::consumables["bomb"]` (M3-T3, §68) |
| `shells` | monnaie `Currency` (les douilles de boutique la créditent) |
| `passives` | noms des passifs ; `Icons` utilise leurs ids |
| `active_item` | nom de l'objet actif |
| `active_charge` | charge / seuil, suffixe « · prêt » ; barre bornée entre 0 et 1 |
| `minimap` | type, état, marqueur boss et salle actuelle ; rendu spatial par `Map` |

Sans joueur/objet : textes vides et charge masquée. `HudRoomGeometry` reçoit uniquement
la géométrie de l'étage courant depuis `CurrentRooms`, en `Update` avant `HudModelSet`.
La carte se dérive de cette géométrie, de la position du joueur et de `RoomStates` :
aucun nouveau composant/état rollback.
**Mémoire de visite (D72, 2026-10-15)** : `RoomStates` ne retient pas une salle non combattante
restée `Dormant` (boutique, récompense). `hud_model::MinimapVisits` (ressource de présentation,
`Update`, hors rollback et hors trace) garde les ids des salles où un joueur **local** est entré
(`track_minimap_visits`, avant `update_hud_snapshot`) ; elle repart de zéro quand la géométrie de
l'étage change (nouvel étage, restart). Une salle visitée puis quittée reste « visitée ». Limite
assumée : un rollback ne défait pas une visite (l'exploration n'est pas un fait de simulation) ;
aucun état `Visited` rollback n'a été nécessaire. Preuve : scénario `minimap_shop_visit`
(gungeon, `etages_grammaire`, graine 12 : la boutique est voisine du départ ; attentes `HudText`
« boutique · inexplorée » f10, « visitée · actuelle » f90, « visitée » après la sortie f199, alors
que `RoomState` reste `Dormant`).

### Scénarios et revue visuelle

Le script `ui: [(at_frame: 10, action: Toggle(Pause)), …]` joue en `Update`, hors input
rollback. `ScreenText(screen, section, contains, closed, at_frame)` vérifie les menus ;
`HudText(source, contains, at_frame)` vérifie les sources du HUD. Les scénarios existants
`ui_inventory`, `gungeon_start`, `gungeon_clear`, `gungeon_boss` et `gungeon_two_floors`
portent les attentes HUD, sans modifier leurs traces.

```bash
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/ui_pause_menu.ron --capture /tmp/ui/pause --every 1
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/ui_inventory.ron --capture /tmp/ui/inventaire --every 1
cargo run -p scenario --profile headless --features render --bin play_scenario -- tests/scenarios/gungeon_boss_bot.ron --capture /tmp/ui/fin --every 10
```

Frames à regarder (`frame_NNNNN.png`) : pause 10, 20–22, 31, 40, 51, 62, 64 ;
inventaire/HUD 9 (actif prêt), 12, 20 (déchargé), 21–24 (focus), 31 (fermé).
Carte : `gungeon_clear` 100 (verrouillage), 650 (nettoyée) ;
`gungeon_boss` 320–450 (boss), 480–520 (blanks consommés) ;
`gungeon_two_floors` 800 (nouvel étage). Fin : `gungeon_boss_bot` 450.
Rechargement en jeu : `cargo run -p gungeon --features native`, ouvrir un écran puis
modifier son `.html`. Le rapport précise les captures effectivement observées.


## 57. Hub : stations, marchand de déblocages, entrée en run (M2-T13, chantier G2)

Le hub est une **session à part** : une salle LDtk où l'on se déplace et interagit (simulation
rollback, sans ennemi), puis la porte relance la session en run. Il ne change rien aux jeux sans
`entry.hub` (`zombies`, `testbed`, `throne`, `isaac`) ni aux scénarios sans champ `hub`.

**Données.** `game.ron` : `entry.hub: Some((map: "maps/hub.ldtk"))` et un dossier `(path: "hub",
kind: "HubService")`. `hub/<id>.ron` (`meta::HubServiceDef`) : `name`, `needs` (conditions de profil :
`Unlock("first_run")`, `Stat("runs", 2)`, toutes requises) et `action` : `EnterRun` (la porte) ou
`Merchant` (l'écran des déblocages). `meta::MILESTONES` (`first_run`, `first_victory`) sont des
conditions valides en plus des `Unlock` du jeu. **Lint** : carte du hub inconnue, hub sans service
`EnterRun` (`BrokenReference`) ; condition `Unlock` inconnue (`BrokenReference`) ; `Stat(_, 0)`
toujours vraie (`OutOfRange`). Fixtures `hub_unknown_unlock`, `hub_stat_zero`, `hub_without_door`.

**Carte.** `maps/hub.ldtk` (une salle 16×12, quatre `PlayerSpawn`) et l'entité LDtk `HubStation`
(champ `service` : id de `hub/<id>.ron`). Le hub est chargé avec `MapGenerationMode::Static` (variante
ajoutée en dernier) : la carte est jouée **telle que dessinée**, sans assemblage de salles.

**Stations dans la simulation.** `map_ldtk::game::local::spawn_hub_stations_when_map_loaded` pose une
`game::hub::HubStation { service }` (rollback neutre) + `Interactable { Hub }`
(`InteractionType::Hub`, en fin d'enum) par station dont le service est **ouvert** pour la session :
en local, si le profil d'un des joueurs locaux remplit ses conditions ; **en ligne, le profil de
l'hôte** reconstitué de la configuration de session (`ALACOD_RUN_UNLOCKS` pour les déblocages,
`ALACOD_HOST_STATS=nom:n,…` pour les statistiques ; stand-in de la synchronisation des profils,
hors périmètre). `GgrsNetId` triés par position puis service.
`game::hub::hub_station_system` (`Interaction`) écrit `HubState` (ressource rollback **neutre**,
vide par défaut) : `EnterRun` pose `exit: Some(HubExit { frame, handle })` (le premier ; **par défaut
n'importe quel joueur déclenche la porte**, à confirmer par William), `Merchant` enregistre
`visits[handle] = HubVisit { service, frame }`. Aucune autre règle : le profil n'est jamais lu dans
`GgrsSchedule`.

**Sortie : décision confirmée.** `game::hub::hub_exit_trigger` (`Update`) pose
`RunRequest::EnterRun` quand `HubState.exit.frame <= ConfirmedFrameCount` : un état prédit puis annulé
ne lance rien et les pairs qui confirment la même frame prennent la même décision au même moment.
`EnterRun` et `ToHub` passent par le chemin du restart (`plan_run_request` : `Restart` local = retour
direct en `GameLoading` ; en ligne, la salle `{lobby}-r{n}` et la graine dérivée de §33) ;
`apply_run_request_system` bascule `HubSession.in_hub` et la carte (`hub::apply_to_map_config`).
`HubState` est remis à zéro à la sortie d'`InGame`.

**Phase.** `game::hub::HubSession { hub_map, run_map, in_hub }` (hors rollback, survit aux relances)
est posée par `games/gungeon/src/main.rs` (le jeu démarre **au hub**) ou par un scénario
(`hub: Some((stay: false))`). Au hub le mode de run est `Sandbox` (`hub::resolve_session_mode`) ; la
run reprend le mode du manifeste (`Floors`, `entry.floors`). **Les outils sans binaire de jeu
(`alacod-sim`, `make gen`, les campagnes de bots, le runner de scénarios sans champ `hub`) ne posent pas
`HubSession` : ils jouent la run directement**, donc aucun faux soft-lock d'explorateur au hub (choix
retenu plutôt que d'apprendre la porte aux bots). Fin de run : le bouton « Rejouer (R) » de l'écran
de fin devient « Retour au hub (R) » (`RunRequest::ToHub`) quand `HubSession` existe ; le résumé de
la run reste affiché, le profil est déjà écrit.

**Marchand.** Écran `ScreenId::Merchant` (`ui/merchant_screen.html`, gabarit bevy_hui, même moteur
que la pause et l'inventaire, §56) : une visite `HubVisit` du joueur affiché, dont la frame est
confirmée, l'ouvre (`Screens::merchant_seen` : une visite ne rouvre pas l'écran). Lignes
(`model::merchant_lines`) : les `Unlock` du jeu (ordre des ids), état `Owned` / `Available` /
`Locked(raison)` calculé par `Profile::purchase` sur une copie. Entrée sur une ligne : commande
`ScreenCommand::Buy(n)` → `game::profile::purchase_for_handle` (hors simulation) : débit de l'essence,
profil écrit tout de suite (écriture atomique de §38), message affiché. **Un achat entre dans la
simulation partagée au passage de la porte** : `RunUnlocks` est figé au lancement de chaque session
(§51) et la run relit les profils au lancement ; au hub il ne change pas.

**Scénarios.** `Scenario::hub: Option<HubSetup>` (`stay: true` : la run n'est pas lancée, la preuve
reste dans la session du hub), attentes `HubExit { handle, exit_frame, none, at_frame }` et
`HubVisit { handle, service, at_frame }`. `hub_use_stations` (marchand puis porte), `hub_buy_unlock`
(écran, achat, `ProfileHas`, `RunUnlocks` inchangé). Avec `stay: true` le runner ne suit pas la relance
(la preuve reste dans la session du hub) ; avec `stay: false` il la suit (m2-full-run, ci-dessous). La
relance a aussi son test, `crates/scenario/tests/hub.rs` (porte → requête une fois la frame confirmée →
`Floors { etage_1 }` étage 0, `HubState` et `RoomStates` remis à zéro, `RunUnlocks` de la run contient le
déblocage acheté au hub).

**Limites.** Local : la graine de la run ne change pas d'une run à l'autre (le restart local garde la
graine, §33). En ligne : la porte est jouée par le chemin du restart de §33 (preuve p2p rejouée par
l'orchestrateur) ; les déblocages et services de l'hôte viennent de la configuration de session
(`ALACOD_RUN_UNLOCKS`, `ALACOD_HOST_STATS`), pas encore d'un échange réseau, et un achat de l'hôte
au hub n'atteint les pairs qu'avec la synchronisation des profils (hors périmètre).

### Run complète de M2 : `gungeon_full_run` (m2-full-run)

Deux scénarios attestent les mécaniques de M2 du tableau ci-dessous dans **une** partie de bots (graine 10, `etages_deux`,
`shopping: true`, profil à trois déblocages, hub puis run) : `gungeon_full_run` (2 bots `explorateur`, 7 900
frames, victoire f7200 depuis les gabarits agrandis, §40) et `gungeon_full_run_four` (4 bots, 7 000 frames, victoire f6844,
synctest). Les moments clés du premier sont dans `events.json` (page de revue), chaque ligne du tableau a une attente
**sauf le coffre à clé**, attesté par `gungeon_key_chest` (voir sa ligne).

| Mécanique | Frame (2 bots, frames de la run) | Attente |
|---|---|---|
| Entrée depuis le hub | marchand f80 et porte f175 du hub ; `hub` f1 | `HubVisit`, `HubExit(exit_frame: 175)`, `Event(hub)` |
| Déblocages lus du profil (M2-T12) | f1 | `RunUnlocks(has: 3)`, `Event(unlock)` |
| Escouade | verrou f356 → f506 | `SquadSize(bullet_kins)` = 3 (f380), 0 (f520) ; à 4 joueurs ≥ 2 (f380), 0 (f460) |
| Salle verrouillée puis nettoyée | f66 → f186 | `Event(room)` ×2 |
| Blank utilisé | f95, f107 | `Event(blank)` |
| Objet ramassé | clé f246, blank f1783, ration f3538 ; coffres libres : `bastion` f698 (joueur 1), `rancune` f6256 (joueur 0, étage 2) | `Event(item_pickup)`, `HasItem(bastion)`, `HasItem(rancune)`, `Consumable(key, 1)` |
| Achat en boutique | f1896-1922 (trèfle, présentoir 1 ; second étage f6117-6143) | `Event(purchase, shop)`, `Shelf(nth: 1, sold)` faux f1850 puis vrai f1950, `Consumable(blank)` 0 (f1700) → 1 (f1800) |
| Coffre à clé ouvert, clé consommée | **hors des runs de bots** : la limite de détour du bot écarte la salle de récompense (`recompense`, coffre `locked: true`) et la clé ramassée f246 reste au joueur (`Consumable(key)` = 1 jusqu'à la fin). **Attesté par `gungeon_key_chest`** (`game: "gungeon"`, coffre verrouillé, clé f5, interaction f10, objet `bastion` ramassé f45) | `Consumable(key)` 1 (f8) → 0 (f20), `Event(chest)`, `ItemCount(bastion)` 0 → 1 |
| Arène tenue | f2077 → f2827 (750 frames) | `Event(arena)`, `RoomState(arene, nth: 0, Cleared)` |
| Boss, phase 1 | boss f3650, phase 0 → 1 f3740, 1 → 2 f3761, nettoyée f3780 | `BossPhase(NetId(175), 1)` (f3750), `Event(boss_phase)` |
| Portail | f3781 | `Event(portal)` |
| Second étage | f4219, puis arène f5025-5775, boutique f6117-6143, coffre f6256, boss f7111 (phase 1 f7151) | `FloorIndex(1)`, `EventAfter(chest, purchase, arena, boss_phase)`, `BossPhase(NetId(607), 1)` |
| Victoire et profil | f7200 | `RunState(Ended(Victory))`, `ProfileHas(first_victory)` |

**Non attestées, et pourquoi** : (0) la **phase 2 du boss** n'est lue par aucune attente de ces runs : `BossPhase` y est lue en phase 1 (le boss ne fait « qu'une transition par frame », `game/src/boss.rs:7`, et disparaît à sa mort) ; le premier boss passe en phase 2 f3761 (moment clé) ; la phase 2 est attestée par `gungeon_boss` (f520) et `gungeon_boss_bot` (bot) ; (1) la **salle secrète** : absente de la graine 10 et, de toute façon, son mur ne
s'ouvre qu'à une explosion alors que gungeon ne donne aucune bombe (D81) ; (2) la **porte du hub conditionnée par un
déblocage** : elle n'existe pas, `hub/porte.ron` (`EnterRun`) n'a aucune condition ; seul le marchand exige `first_run`
(sa visite f120 le prouve pour le profil du scénario) ; l'effet des déblocages sur la run se prouve par `RunUnlocks`
(le trèfle quitte les pools sans `tresor_trefle`, §51), pas par la porte. Attente ajoutée `SquadSize { squad, min,
max, at_frame }` : ennemis vivants déclarant `ai.squad`.

**Runner : la relance du hub est suivie** (`HubSetup { stay: false, inputs }`). Le scénario se joue en deux
phases dans une même boucle : le hub (frames du hub ; seules `HubExit` et `HubVisit` sont évaluées, `inputs` :
une piste par joueur, remplace `players[i]` pendant le hub) puis, quand la porte a relancé la session
(`HubSession::in_hub` faux, `FrameCount` revenu à 0), la run (`swap_hub_inputs` rend aux joueurs leurs pistes ou
leurs bots). `frames`, les autres attentes, la **trace** (l'enregistreur est vidé à la relance), `sim_fps` et
`events.json` sont ceux de la run ; `play` et `capture` ne comptent `frames` et ne capturent qu'une fois la run
commencée (`HubPhaseOver`) : la vidéo montre la run seule, sans image du hub. Refusés explicitement
(échec du scénario) : les attentes continues et les réglages de joueur (`weapon`, `items`, `currency`…), qui
ne s'appliquent qu'à la première session. `HubSetup { stay: true }` et les scénarios sans hub : inchangés.

**`Scenario::shopping`** : le bot `explorateur` ouvre les coffres et achète les présentoirs (D62,
`ExplorerOptions::shopping`) sans code côté appelant ; faux par défaut, aucune trace de bot ne bouge.

**Moments clés ajoutés** (`crates/scenario/src/events.rs`) : `chest` (`ChestOpened`), `arena` (arène dont le
type porte `RoomKindDef::arena`, de `Locked` à `Cleared` : « arène tenue (750 frames) »), `unlock` (`RunUnlocks`
non vide, une fois), `hub` (« entrée en run depuis le hub », daté de la première frame de la run) ; l'achat
en boutique est le moment `purchase` existant (`CurrencyEvent` de raison `shop`). Attente `EventAfter` : un
moment clé survenu après une frame donnée (la seconde arène, le second boss). **Les moments clés tirés de
`FrameEvents` (`room`, `blank`, `item_pickup`, `boss_phase`, `arena`, `chest`) portent la frame de la
simulation qui les a émis, une de moins que celle lue après l'update : `by_frame` = frame affichée + 1.**

**Trois défauts trouvés en le faisant** (aucune trace existante ne bouge) :
1. *Hub → run : toutes les portes restaient fermées.* Le hub est une salle typée (`room_kind: depart`) : sa
   session posait `RoomStates::doors_opened`, que la relance ne remettait pas à zéro ; aucun bot ni joueur ne
   sortait de la salle de départ (mesure : 39 portes fermées contre 3 sans hub). Même défaut au restart
   (touche R) avec les salles nettoyées de la partie précédente. `cleanup_rollback_world_system` remet
   maintenant `RoomStates`, `ArenaClocks` et `DoorLocks` à zéro (assertions dans `tests/hub.rs`).
2. *Quatre joueurs sur `gungeon` : panique « pas de point de spawn pour le joueur 2 ».* Le gabarit `Depart` n'a que
   deux `PlayerSpawn` ; en ajouter décalait le barycentre du portail et la numérotation des entités de tous les
   étages (toutes les traces gungeon). `map_ldtk::game::local::player_spawn_point` : un joueur sans point
   propre prend celui de rang `handle % n` décalé de 24 unités vers le bas par tour, uniquement là où le
   chargement paniquait. À l'étage suivant, `floors.rs` laisse déjà « en place » un joueur sans point.
3. *`HashMap` des points de spawn* remplacé par un `BTreeMap` (deux entrées retirées de
   `determinism-allowlist.json`).

**Fuzz** : `ALACOD_FUZZ_SCENARIO=<nom>` (`fuzz_inputs`) joue le scénario avec des inputs aléatoires à la place de
ses bots (attentes retirées, pistes du hub conservées) : `ALACOD_FUZZ=0:8 ALACOD_FUZZ_SCENARIO=gungeon_full_run
cargo test -p scenario --profile headless --test fuzz_inputs -- --ignored --nocapture`.
Résultat (2026-10-16, 3 600 frames, synctest) : graines 0 à 7 sur `gungeon_full_run` et sur
`gungeon_full_run_four`, aucune divergence ; tous les joueurs restent en vie. La couverture n'est pas mesurée : des
inputs aléatoires ne cherchent pas à traverser les étages, les deux scénarios de bots le font (synctest, 0 desync).

**Vidéo et moments clés.** `scripts/scenario-video` ne tourne pas ici : `play_scenario --capture` ouvre une fenêtre
winit (cachée) et panique (`Failed to build event loop`, « neither WAYLAND… nor DISPLAY is set » : ni X11 ni Wayland, aucun `Xvfb`
installé, pas de droits root ; `lavapipe` est installé mais la fenêtre échoue avant). Les vidéos sont à rendre sur un
poste avec écran. Les moments clés se produisent sans rendu : `ALACOD_EVENTS_JSON=<dossier> ALACOD_SCENARIO=<nom>
make test_scenarios` écrit `<dossier>/<nom>.events.json`, au format de la capture ; ceux des deux scénarios sont dans
`docs/taches/rapports/m2-full-run/` (à copier dans `target/videos/<commit>/<nom>.events.json` pour la page de revue).

**Graine** : sur 64 graines à 2 bots (`alacod-sim --shopping`, 15 000 frames, avant la relance par le hub) : 48
victoires, 13 soft-locks (bot, D82), 3 défaites, 0 desync ; la 10 est une graine où tout arrive (coffre à clé,
achats, deux arènes, deux boss) et où les phases 0 → 1 → 2 du boss tombent aux deux étages ; elle gagne aussi à
4 joueurs.

### p2p réel de la run complète (m2-p2p-run-complete)

Preuves en ligne (transport WebRTC réel, rollbacks réels, sans affichage) de la run de M2 ; mesures et commandes
dans `docs/taches/rapports/m2-p2p-run-complete.md`.

- **Inputs des bots figés.** Les bots (`crates/bots`) ne sont pas dans le binaire du jeu : `scripts/p2p-gungeon-inputs.sh
  <joueurs> <graine> <frames> <sortie.ron>` joue la run par `alacod-sim --save-scenario` (l'enregistrement exact de ce qui
  part à GGRS) ; chaque client la rejoue (`ALACOD_INPUT=script:<fichier>`, handle GGRS = piste). Valable pour la même
  graine, les mêmes réglages et le même contenu seulement : après un changement de simulation, l'étage 2 n'est plus
  atteint, et `EXPECT_FLOORS` de `p2p-stress.sh` (nombre de `floor_transition` distincts du journal) échoue au lieu
  d'un faux vert.
- **`ALACOD_SKIP_HUB=1`** (gungeon, natif) : pas de `HubSession`, la partie démarre dans la run ; **`--seed <n>`** (tous les jeux, D105) : graine de
  carte imposée par **un seul** consommateur, `map_ldtk::game::local::apply_launch_seed` (`Startup`, installé par `LdtkLocalGamePlugin`,
  en local comme en ligne ; sans lui, `entry.default_seed`, sauf `zombies` en local, qui tire une graine au hasard : `choose_local_seed`
  ne garde que ce tirage et le titre de fenêtre, `--seed` passe par le système commun ; le runner de scénarios n'installe pas
  `BaseArgsPlugin`, donc aucune graine n'est imposée aux scénarios). `scripts/check-launch-seed.sh` : trois parties locales par jeu
  (graines A, B, A), frame 0 différente entre A et B, identique entre les deux A. Raison : une partie en
  ligne passée par le hub joue sa run sur une graine **dérivée** (`restart_seed`, §33) alors que la graine de carte
  du hub est conservée : un enregistrement de bots fait par `alacod-sim` (même graine partout) ne s'y rejoue pas à
  l'identique. Sans `ALACOD_SKIP_HUB`, le hub est joué pour de vrai : `ALACOD_INPUT=script:` lit les pistes `hub:
  Some((stay: false, inputs: …))` du scénario (`ScriptedHubInputs`) tant que `HubSession::in_hub`, puis les
  `players` ; `scripts/p2p-stress-inputs.py --hub-door` génère un script (hub : le joueur 0 va à la porte, run : entrées
  aléatoires).
- **Délai d'input des enregistrements de bots (D85, fermée).** `alacod-sim` joue en synctest avec `ALACOD_INPUT_DELAY` = 5 (défaut) : l'input
  décidé à la frame f est appliqué à f+5 ; les clients p2p des preuves tournent à 0. `scripts/p2p-gungeon-inputs.sh` décale donc les segments de
  `INPUT_SHIFT` (5) frames (0 pour des clients à délai 5) : sans cela la run rejouée n'est pas celle des bots (franchissement d'étage f3725 au
  lieu de f3731, aucune victoire après le hub). `HUB=1` enregistre une run lancée par la porte du hub en ligne (`ALACOD_SIM_HUB=1` : hub puis run
  dans `alacod-sim` ; `ALACOD_RUN_SEED_ONLINE_GAME=1` : graine de run `restart_seed(graine, 1)`, `jjrs::local`, natif, absente = inchangé) ;
  le client se lance alors sans `ALACOD_SKIP_HUB`.
- **Retour au lobby en ligne (D86, fermée).** Cause : « Lobby » rouvre la salle d'**origine**, où l'autre pair garde son socket de la partie finissante
  jusqu'à `OnExit(InGame)` ; le premier revenu s'y connectait et construisait sa session sur cet ancien identifiant. Correctif sans protocole
  nouveau (`jjrs::p2p`) : (a) `PreviousSessionPeers` mémorise les identifiants des pairs de chaque session construite
  (`system_after_map_loaded`) et `live_players` les retire de la liste du socket (`wait_for_players`, interface du lobby, construction de la
  session ; l'ordre trié des identifiants, donc les handles GGRS, est conservé) ; (b) une session p2p restée `Synchronizing` plus de
  `SYNC_TIMEOUT_SECS` (30 s d'**horloge murale**, `Instant` : `Time<Real>` avance de 1/60 s par update en headless ; `ALACOD_SYNC_TIMEOUT_SECS`
  pour les essais ; l'autre pair peut légitimement finir de charger) est abandonnée :
  `ggrs{f=… session_sync_timeout waited_secs=… players=…}` au journal, puis `RunRequest::ToLobby` ; le repli du restart (`{lobby}` après
  `RESTART_TIMEOUT_SECS` = 30 s, qui lit encore `Time<Real>` : trop court en headless, défaut antérieur non corrigé) est inchangé. Session seulement : aucune trace de scénario ne bouge. Essais :
  `scripts/p2p-lobby.sh` (requêtes à 100–400 ms d'écart, `ALACOD_RESTART_DELAY_MS`), `scripts/p2p-sync-timeout.sh` ((b) de bout en bout :
  un client suspendu avant sa session). `p2p-restart.sh` avec `RESTART_KIND=lobby` ; `EXPECT_VICTORY=1` exige la victoire avant la relance.
- **Connexion jamais établie au lobby (D109, fermée).** Sous perte de paquets (3 % : 2 à 5 % des démarrages), un pair annoncé par le signaling ne finit jamais sa négociation WebRTC et le
  lobby attendait sans fin (« Waiting for players: 1/2 »). Le socket matchbox n'expose pas ce pair : `jjrs::signal_watch::ObservedSignallerBuilder` (natif) enveloppe le signaller et
  note les `NewPeer`/`Signal` reçus ; `wait_for_players` borne l'attente du plus ancien pair vu, ni connecté ni d'une session passée, à `CONNECT_TIMEOUT_SECS` (15 s, **horloge murale**,
  hors simulation, `ALACOD_CONNECT_TIMEOUT_SECS`) : nouvelle tentative (socket neuf, même salle ; `ggrs{f=0 lobby_connect_timeout attempt=1/2 action=retry …}`), puis retour au lobby
  (`LobbyOnline` ré-entré ; `attempt=2/2 action=lobby` ; `ALACOD_CONNECT_ATTEMPTS`) ; en allumette et sur le web pas de réouverture. Écran du lobby : ligne d'état (« Connecting to peer... »,
  « new attempt (2/2) », « back to the lobby »). **Piège** : la nouvelle tentative d'un seul côté d'une connexion à moitié ouverte (un canal ouvert, l'autre pas) donnait deux cycles de
  30 s décalés qui ne se rejoignaient jamais ; une session `Synchronizing` est donc aussi abandonnée dès qu'un pair de la session en cours quitte la salle
  (`CurrentSessionPeers`, `ggrs{f=… session_peer_left …}`). Session seulement : aucune trace de scénario ne bouge. Preuves : `p2p-connect-timeout.sh`, `p2p-connect-starts.sh`
  (150 démarrages à 3 % joués, 7 retentés, 0 perdu) ; rapport `dettes-d105-d109.md`.
- **Admission au lobby (D116, corrigée ; ouverte sur l'arrivée tardive).** `wait_for_players` attendait sans fin quand des pairs en trop restaient dans la salle (D109 : « attend tant qu'il y a plus de pairs que
  `max_player` »). Politique : `jjrs::p2p::admit_players` garde les `max_player` **premiers** joueurs du socket (identifiants triés, le même ordre chez tous les pairs qui voient le même
  ensemble ; les handles GGRS en dépendent), les suivants sont **ignorés**. Les admis démarrent sans attendre les pairs en trop (`ggrs{f=0 lobby_admission peers=3 max=2 excess=1 local=admitted|ignored}`,
  journalisé une fois par changement) et `AdmittedPlayers` (ressource, fixée par `wait_for_players`) est reprise telle quelle par `system_after_map_loaded` : un pair arrivé pendant le chargement de la carte ne
  change ni les handles ni la taille de la session. L'ignoré attend une place (écran du lobby : « Room full (3 peers for 2 places): waiting for a free place »), puis, au bout de
  `ALACOD_CONNECT_TIMEOUT_SECS` (15 s, horloge murale), revient au lobby (`ggrs{f=0 lobby_room_full … action=lobby}`, « back to the lobby ») en tenant les admis pour occupés
  (`PreviousSessionPeers`) : sans cela son socket neuf les reverrait et le classerait parmi les admis d'une session que personne n'écoute (constaté : 1 essai sur 2 au premier jet).
  Un excédent de pairs **connectés** (`players.len() > max_player`, p. ex. le socket d'une partie finissante pas encore dans `PreviousSessionPeers`) est attendu `ADMISSION_SETTLE_SECS` depuis son apparition (`LobbyConnectWatch::excess_since`) ; le délai de D109 ne vaut que s'il manque des joueurs. Un pair vu par le signaling et pas encore connecté retarde l'admission d'au plus `ADMISSION_SETTLE_SECS` (3 s) : sans cette attente, deux clients qui ne voient pas le même ensemble
  choisissent des admis différents ; elle est bornée pour qu'un pair à moitié connecté qui ne finit jamais (D109) ne bloque pas les pairs connectés. **Limite assumée** : pas de protocole d'engagement entre pairs (une course sur 24 essais depuis la correction du premier jet : un client a démarré avant que le signaling lui annonce le troisième).
  Un pair qui arrive après le démarrage des autres voit trois pairs et se classe parmi les admis si son identifiant est bas : sa session ne se synchronise jamais et `abandon_unsynchronized_session`
  le ramène au lobby au bout de `SYNC_TIMEOUT_SECS` (30 s) ; même cas par construction avec allumette et le web (non essayés). Session seulement : aucune trace de scénario ne bouge.
  Preuves : `p2p-admission.sh` (trois clients, deux places : les deux admis jouent 1 500 frames aux traces identiques, l'ignoré revient au lobby ; `GHOST=1` : deux clients et un fantôme à moitié
  connecté jouent après 3 s), `p2p-connect-timeout.sh GHOST=1` (les deux délais complets `attempt=1/2 action=retry` puis `attempt=2/2 action=lobby`) ; rapport `dette-d116-lobby-admission.md`.
- **Entrées du joueur local en deux moitiés (D69 étape 2, E4).** La simulation (`character/player/input.rs`) ne nomme ni leafwing, ni `Window`/`Camera`, ni `ui::` : `read_local_inputs` (`ReadInputs`) construit `LocalInputs` d'après la source (`InputSource`) et, pour `Devices`, lit `Option<Res<DeviceInputs>>` (`BTreeMap<handle, BoxInput>`). L'application (`character/input_devices.rs`) l'écrit : `collect_device_inputs` (`ReadInputs`, **avant** `read_local_inputs`, remet la table à vide à chaque appel ; mêmes boutons, même `pan` — souris et caméra —, `confirm_input` de l'écran de mutation) ; `attach_local_player_input` pose `ActionState`/`InputMap` sur le joueur local (`PreUpdate` avant `RunGgrsSystems`, `Added<LocalPlayer>` : au plus une frame de rendu après sa création, D4) et `light::attach_player_light` la lumière (feature `lighting`). `character::app_plugin::CharacterAppPlugin { headless }` (juste après `BaseCharacterGamePlugin`) n'ajoute leafwing, `DeviceInputs` et ces systèmes **que hors headless** : en headless, un joueur local sous `Devices` envoie un input neutre (comme avant, `ActionState` n'y était jamais pressé) et `skip_device_input` n'existe plus. `create_character` ne pose plus `Visibility` (posée par `attach_character_visuals`) ni `SpatialAudioEmitter` (inerte). **Piège de capture** : une ressource nouvelle est une entité (+1 index), et `legibility.rs` déduit la phase du halo des objets de `entity.index % 16` : ajouter une ressource décale le halo d'une capture `gungeon_boss` sans toucher la simulation (preuve : base + ressource factice = 0 image différente) ; comparer des captures par `cmp` exige donc le même nombre de ressources. Rapport `dette-d69-e4-personnage.md`.
- **Session p2p en deux moitiés (D69 étape 2, E6).** `jjrs/p2p.rs` est la moitié **simulation**, sans socket : `PreviousSessionPeers`, `CurrentSessionPeers`, `SessionPeerLeft`, `live_players`, `admit_players`
  (D116), `AdmittedPlayers`, `pair_session_players` (appariement joueurs du socket ↔ `GggrsSessionConfiguration`, l'application passe la liste des pairs ; un `identity` optionnel pour le web),
  `start_online_run` (graine, `Run`, déblocages, `AppState::InGame`), `SYNC_TIMEOUT_*`, `SessionSyncWatch`, `reset_sync_watch`, `abandon_unsynchronized_session`, `session_builder`,
  `DISCONNECT_TIMEOUT`, `MAX_PREDICTION_WINDOW`, `secs_from_env` ; `PeerId` vient de `matchbox_protocol` (crate de données). `jjrs/p2p_socket.rs` est la moitié **application** (tout ce qui lit
  `MatchboxSocket`) : `start_matchbox_socket`, `wait_for_players`, `LobbyConnectWatch`, `build_p2p_session` (canal du socket, session GGRS, `Session::P2P`), `observe_session_peers` (écrit
  `SessionPeerLeft`), `drop_matchbox_socket` (ex-`restart.rs`, `OnExit(InGame)`), avec `signal_watch.rs`, `allumette.rs`, `browser.rs`. L'ancien `system_after_map_loaded` est `build_p2p_session`
  (app) puis `start_online_run` (sim), dans le même tuple `OnEnter(GameStarting)` ; `abandon_unsynchronized_session` lit `SessionPeerLeft` après `observe_session_peers` (même update, même frame).
  Télémétrie : `log_ggrs_events` écrit le message `DesyncObserved` (même idiome que `SessionLostEvent`), `jjrs/telemetry.rs` (app) le transmet à `telemetry` ; `GameArgsPlugin` (lu par `scenario`) ne
  pose plus `TelemetryConfig`, `args::telemetry_config::TelemetryConfigPlugin` (app, ajouté par `BaseArgsPlugin` après lui). `recording.rs` ne nomme plus `bevy::window::ExitSystems` (l'application pose
  `ExitSystems.before(ExitRequests)`). Web : `run_state::apply_run_request_system` écrit `ReturnToHostLobby` pour une session `HostedSession` (marqueur posé par `BaseArgsPlugin`),
  `jjrs::browser::return_to_host_lobby` appelle le crochet JS. Aucun type rollback touché, aucune trace ne bouge, ordre des schedules identique (`make schedule_order_diff`). Au passage en
  `game_app` (E9), `p2p_socket.rs`, `telemetry.rs`, `signal_watch.rs` et `telemetry_config.rs` suivent sans changement de contenu ; rapport `dette-d69-e6-session-reseau.md`.
- **Trace** : `StateTracePlugin` vide l'enregistreur à **chaque** sortie d'`InGame` (pas seulement au restart) : les frames de
  la run (qui repartent de 0 après le hub) ne se mêlent pas à celles du hub et ne comptent pas comme resimulées. Le compteur
  `resimulées` reste cumulé sur toute la durée du processus ; les rollbacks sont comptés **par partie** (`state_trace::RollbackStats`) et
  journalisés à la sortie : un rollback = une restauration d'état de bevy_ggrs (schedule `LoadWorld`, même nombre que `load ×N` du
  profil rollback) ; sa profondeur = frames resimulées = `dernière frame avancée avant la restauration − frame restaurée` (lue à la fin de
  chaque `AdvanceWorld`, indépendante de la trace : GGRS sauvegarde la frame courante même pendant un tick bloqué au seuil de prédiction,
  ce qui réenregistre une clé de trace sans rien resimuler) ; ligne
  `rollbacks : n=… frames=… profondeur_moy=… profondeur_max=… par_100_frames=… histogramme=profondeur:nombre,…`
  (`p2p-stress.sh` l'affiche par client).
- **`ALACOD_RESTART_KIND`** (avec `ALACOD_RESTART_AT_FRAME`) : requête posée à la frame de relance, `restart` (défaut),
  `lobby` (`RunRequest::ToLobby`) ou `hub` (`RunRequest::ToHub`) ; `ALACOD_RESTART_DELAY_MS` retarde la pose de la requête
  (temps réel, essais D86).
- **Scripts** (`scripts/p2p-lib.sh` en commun) : `p2p-stress.sh` (N clients `CLIENTS`, suspension du client 1,
  `NETEM`, `MATCHBOX_URL`, `GAME_ARGS`, `EXPECT_FLOORS`), `p2p-restart.sh <jeu>` (restart, lobby, hub ; `RESTART_DELAYS_MS`),
  `p2p-lobby.sh` (D86 : essais consécutifs du retour au lobby), `p2p-sync-timeout.sh` (abandon d'une session non synchronisée ; le client repris abandonne par `session_peer_left` ou `session_sync_timeout`),
  `p2p-connect-timeout.sh` (D109 : pair vu par le signaling jamais connecté, `ATTEMPTS=1|2` ; `GHOST=1` : second délai `attempt=2/2` avec un pair fantôme visible à chaque socket, `p2p-ghost-peer.mjs`), `p2p-admission.sh` (D116 : trois clients, salle à deux places ; `GHOST=1` : deux clients et un fantôme), `p2p-connect-starts.sh` (D109 : fréquence des démarrages bloqués,
  `PLAY_FRAMES`), `p2p-linux-box.sh` (les scripts `NETEM` depuis un Mac : conteneur Linux `NET_ADMIN`), `check-launch-seed.sh` (D105),
  `p2p-disconnect.sh` (pair tué), `p2p-machine.sh` (un client par machine), `nightly.sh` étape C2.
- **Deux machines (m2-p2p-deux-machines, 2026-10-18).** `scripts/p2p-machine.sh` lance **un** client sur chaque machine (`CLIENTS`, `LOBBY` et
  `MATCHBOX_URL` communs, même commit, mêmes inputs figés) ; `STRESS=1` suspend ce client par à-coups (SIGSTOP/SIGCONT, `SUSPEND_MS`, `RUN_MS`) et
  `MIN_RESIMULATED=n` échoue sous n frames resimulées. Constaté sur 8 parties : le VPS resimulait 0 frame dans 3 (`gung-b`, `gung-d`, `isa-b`), le Mac toujours ; [INFERENCE] le client en retard n'a rien à
  corriger, le plus rapide prédit les inputs de l'autre. Pour que **les deux** resimulent, suspendre longuement la machine qui resimule (Mac :
  `SUSPEND_MS=80 RUN_MS=40`, le VPS resimule alors 23 frames, le Mac 9 580). Signaling : `matchbox_server 0.0.0.0:3536` sur une machine (binaire extrait par `p2p-lib.sh`), `MATCHBOX_URL=ws://<ip publique>:3536`.
  `isaac` ignorait `--seed` (D108, corrigé par `m3-t14-bot-isaac`) ; throne et testbed aussi (D105, corrigée par `dettes-d105-d109` : un consommateur commun) : les enregistrements de
  bots (`alacod-sim --game isaac --floors sous_sol --until-floor 2 --seeds 5..5`, décalage d'input +5 comme `p2p-gungeon-inputs.sh`) se rejouent désormais avec `--seed 5`. Résultats : rapport `m2-p2p-deux-machines.md` §3.
- **D87 (silences sous perte, fermée).** Sous perte de paquets le pair se tait 0,5 à 1 s, 1,0 à 1,5 s ou 3,0 à 3,4 s ([INFERENCE] : retransmission rapide, retransmission SCTP de
  WebRTC après `RTO_MIN` = 1 s de `webrtc-sctp`, puis une seconde après 2 s de plus ; rien n'instrumente SCTP) ; GGRS déconnecte au bout de `p2p::DISCONNECT_TIMEOUT` (5 s, 2 s avant D87), et la
  notification `NetworkInterrupted` reste à 500 ms. `scripts/p2p-d87.sh` rejoue la matrice (`CELLS`, `VARIANTS` `timeout2000|timeoutNNNN|unreliable`,
  `REPS`), `scripts/p2p-silences.py` mesure les silences d'un journal (reprise = 500 ms + écart `NetworkInterrupted`→`NetworkResumed`) et distingue
  la déconnexion **en cours de partie** de la **fin de partie** (le pair a fini et quitté : le survivant attend le délai de déconnexion puis valide ses dernières frames avec les inputs par défaut du pair parti, d'où des dernières lignes de trace différentes ; 2 essais sur 31, à 2 s et à 8 s).
- **Réseau dégradé sans root** : `NETEM="delay 60ms 10ms loss 1%"` relance le script dans `unshare -Urn` (espaces de noms
  utilisateur et réseau sans privilège), met `lo` en `tc netem` (le délai s'applique à chaque sens : RTT = 2 × délai),
  ajoute une interface factice `dummy0` (WebRTC ignore la boucle locale) et y fait tourner le signaling (`matchbox_server`
  extrait de l'image Docker du compose, ou `MATCHBOX_BIN`). Aucun accès réseau extérieur dans cet espace.

## 60. Contrats des larmes et de l'empilement de projectiles (M3-T0a, chantier B5 v3)

Contrats posés pour M3-T1 (larmes v3), M3-T6 (familiers qui tirent), M3-T11a (contenu de larmes)
et M3-T15 (bench). **Aucune trace existante ne bouge** : variantes en fin d'enum, stats lues avec
une base 1 sans être posées dans `Stats`, champs d'objet optionnels hors rollback.

### Munitions infinies

`MagBulletConfig::Infinite` (en fin d'enum, `combat::weapons`) : le mode tire tant que la cadence
le permet ; jamais vide, jamais rechargé (pas même le délai de pompe d'un `Shotgun`), `mag_ammo` et
`mag_size` restent à 0 (attente `Ammo` à 0 ; le HUD affichera `∞` ou rien, M3-T16). Réserve
initiale : 0, comme `Magless`.

```ron
"larme": (config: (name: "larme", default_firing_mode: "default", firing_modes: {
    "default": (firing_rate: "3.0", firing_mode: Automatic(), /* … */ mag: Infinite),
})),
```

`ammo_type` devient facultatif dans le RON (`#[serde(default)]`, défaut `Custom("")`, la seule
valeur qu'un RON ne peut pas écrire : T2.8 la refuse) **seulement** pour une arme dont tous les
modes sont `Infinite`. Lint : `ammo_type` absent alors qu'un mode consomme (`Mag`, `Magless`) →
`Parse` (comme l'erreur RON d'avant) ; `ammo_type` donné alors que tous les modes sont `Infinite` →
`OutOfRange`. Une arme mixte (un mode `Mag`, un mode `Infinite`) garde son `ammo_type`.

### Stats de larmes et `Luck`

- `StatId::ShotSpeed` (vitesse de la balle) et `StatId::ShotSize` (rayon de collision et sprite,
  après `Size`), **après `Custom`** dans l'enum : l'`Ord` des clés de `Stats` et le hash des stats
  existantes ne changent pas. Lues au tir (`weapon_rollback_system`, `ShotStats`) par
  `StatReader::get(…, 1)` : base 1 sans être posées dans `Stats` (le `Stats` de tout personnage
  reste identique) ; à 1, aucun produit n'est calculé. `sim_core::stats::default_base` donne cette
  base aux lecteurs hors simulation (attente `Stat`, générateur `ItemHeld`).
- `Luck` a un lecteur de contrat : `effects::chance_with_luck(base, luck, par_point)` =
  `clamp(base + luck × par_point, 0, 1)` en `Fixed`, saturant. Rien ne l'appelle encore dans la
  simulation (M3-T1 : statut posé par une larme ; M3-T2 : effets d'objet). Le lint accepte donc
  désormais un modificateur sur `Luck` (`stat_reader`).

### Modificateurs de projectile portés par un objet

Les contrats `ProjectileModifier`, `Pattern`, `ExpireAction` et `ProjectileSpec` vivent désormais
dans `effects::projectile` (crate de simulation) et sont réexportés par `combat::projectile` :
`items` (simulation, sans rendu) peut les porter sans dépendre de `combat` (dette D69). Le `Hash`,
le `Debug` et le RON ne dépendent pas du crate de définition : aucune trace ne bouge.

```ron
// objets/<id>.ron (ou synergies/<id>.ron)
(name: "Ricochet", kind: Passive, projectile: Some((modifiers: [Bounce(2)], on_hit: [], on_expire: [])))
```

`ItemDef.projectile` et `SynergyDef.projectile` : `Option<ProjectileMods>` (`ProjectileMods` =
`ProjectileSpec` : `modifiers`, `on_hit`, `on_expire`), absent = v1 exacte. Ce sont des
définitions (`ItemTable`/`SynergyTable`, ressources hors rollback, sans `Hash`) : le champ n'entre
dans aucun checksum. **Le tir ne le lit pas encore** (M3-T1) : le lint vérifie sa forme (mêmes
plages qu'une arme, doublon refusé dans la liste d'un objet) puis le refuse (`Unsupported`).

### Composition (`combat::projectile::compose`)

`compose(base, ajouts) -> ResolvedModifiers`, pure, `Fixed` seulement. Chaque liste est d'abord
résolue seule (`resolve_modifiers` : dans une liste, le dernier l'emporte, le lint y refuse les
doublons), puis les sources se combinent (règles par défaut de Q3, retenues par William le
2026-10-15) :

| Modificateur | Règle entre sources | Pourquoi |
|---|---|---|
| `Bounce(n)`, `Pierce(n)` | addition (saturante) | deux objets de rebond rebondissent plus |
| `Size(f)` | produit | comme une stat multiplicative |
| `Lifetime(n)` | maximum des sources qui en ont un | une larme vit au moins aussi longtemps |
| `Homing(f)` | somme plafonnée à 1 | le guidage se renforce sans dépasser le virage complet |
| `Gravity(g)` | somme | |
| `Spectral` | présence | |
| `Split` | addition des `count` ; `spread`, `child` de la première source | addition des branches |
| `Orbit` | présence ; paramètres de la première source | |

Ordre des sources, déterministe : arme (`base`), passifs dans l'ordre de ramassage (un passif en
double compte deux fois), actif, synergies actives par id ; `items::projectile_sources(items,
synergies, inventory)` rend les listes des objets dans cet ordre. Sans ajout,
`compose(base, &[]) == resolve_modifiers(base)` exactement. `resolve_modifiers` reste l'unique
appel au tir (inchangé) ; `ResolvedModifiers` n'est plus `Copy` (il porte `split`, `orbit`,
`spectral`, que `Projectile::new` ne lit pas encore).

### Nouveaux modificateurs (contrats)

En fin d'enum `ProjectileModifier` (les six d'avant gardent leur hash, test
`variantes_historiques_gardent_leur_hash`) :

- `Spectral` : traverse murs et obstacles, pas les personnages ;
- `Split(count, spread, child: Some("id"))` : à l'impact sur un personnage, `count` projectiles en
  éventail de `spread` radians ; `child` = id de la table `projectiles` de l'arme, absent = le
  projectile par défaut du porteur (Q3) ;
- `Orbit(radius, frames_per_turn)` : tourne autour du tireur au lieu d'avancer.

Lint : formes et plages (`Split.count > 0`, `spread >= 0`, `child` présent dans la table de
l'arme ; `Orbit.radius > 0`, `frames_per_turn > 0`), puis `Unsupported` sur une arme tant que
M3-T1 ne les exécute pas. Kinds `projectile_modifier` : `Spectral`, `Split`, `Orbit` (et `Fan`,
`Spiral`, `Burst`, qui manquaient aux `pattern` déclarés par `CombatPlugin`).

### Testbed et scénarios

Arme `larme` (`Automatic`, 3 tirs/s, `mag: Infinite`, sans `ammo_type`) et objet `lentille`
(`ShotSpeed Pct 0.5`, `ShotSize Mul 1.5`). Le testbed n'a pas de choix de personnage joueur dans
un scénario : le « personnage d'essai » de la fiche est l'arme, donnée par `PlayerScript::weapon`.
Scénarios : `tears_infinite` (300 frames de tir, `Ammo` à 0 tout du long, les coups continuent),
`tears_shot_speed` (avec `lentille`, la larme touche `target` à une frame où, dans le témoin
`tears_shot_speed_sans_objet`, elle n'est pas encore arrivée).


## 61. Contrats des objets et de la santé (M3-T0b)

Contrats posés pour M3-T2 (effets v3), M3-T3 (cœurs et pickups), M3-T8 (marché du diable) et
le contenu Isaac. **Aucune trace existante ne bouge** : les champs nouveaux sont optionnels et
absents du contenu livré, les nouveaux types rollback sont neutres, et les variantes sont ajoutées
en fin d'enum.

### Inventaire et exemplaires

`Inventory::count(id)` compte les passifs dans leur ordre (doublons compris), l'actif s'il porte
`id`, et le compteur consommable `id`. La clé historique d'un effet est
`item:<id>:<index>` pour le premier exemplaire ; les suivants réservent
`item:<id>#<k>:<index>`, avec `k >= 1`. Le branchement dans `listed_effects` et l'exécution de
`PerCopy` restent M3-T2 : ce point ne change aucune partie existante.

`effects::Scale::PerCopy` est porté par l'action contractuelle `Scaled { action, scale }`.
`effects::scale_value` est la règle pure : `PerCopy` multiplie par `Inventory::count(id)`, le
champ absent ne modifie pas la valeur. `scale` est accepté seulement dans un effet d'objet ;
les power-ups, frises de boss, personnages, mutations et synergies le refusent au lint. L'action
enveloppée est validée comme une action nue (`Heal` > 0, références de `SpawnPattern`…), y compris
dans l'`OnUse` d'un objet. `Scale` est hors de l'exécution de cette dette.

### Prix et compteurs

`items::Price` est `Currency(u32)`, `Consumable(String, u32)` ou `MaxHealth(Fixed)`. Un
`Shelf` conserve `price: u32` pour son hash historique et reçoit `cost: Option<Price>` ; absent,
`cost` ne contribue pas au hash, et `Price::Currency` est équivalent à `price`. Les règles pures
`can_pay`/`pay` refusent un prix de vie qui tuerait le joueur et réduisent la vie courante et
maximale ensemble ; l'interaction de boutique est M3-T8. Un montant nul n'est jamais payable
(`can_pay` faux, `pay` sans effet et sans panique) ; `Price::problems` donne les erreurs de contenu
(montant nul, consommable inconnu, vie maximale hors `]0, max]`), sans surface de contenu avant T8.

`ItemDef.max_stack: Option<u32>` ne plafonne un consommable que lorsqu'il est présent (Isaac :
99). Absent conserve le comportement illimité actuel. Le ramassage au plafond laisse l'objet au
sol ; son exécution est M3-T3. `game::loot::BOMB_ITEM` réserve le nom `"bomb"`.

### I-frames et questions ouvertes

`CharacterConfig.hit_iframes: Option<u32>` est absent par défaut. Après un dégât final strictement
positif (après immunité, bouclier et armure), le résolveur pose `Health.invulnerable_until_frame`
à `event.frame + n`, en conservant la valeur la plus longue. Un coup absorbé par un bouclier ou
annulé par l'armure ne déclenche pas cette durée. La preuve future est le scénario
`hit_iframes` : deux balles à dix frames d'écart, la seconde n'enlève rien avec `Some(30)` (4992) ;
son témoin `hit_iframes_sans_iframes` (personnage `iframes_temoin`, sans `hit_iframes`) en perd
deux (4984). Le résolveur lui-même est testé par `crates/scenario/tests/hit_iframes.rs`.

**Q2 — modèle des cœurs (proposition par défaut).** Une demi-case vaut un point de `Health`,
les personnages livrés gardent des PV pairs, et `ExtraHealth { soul: Fixed }` absorbe les dégâts
point par point avant les PV ; il reste distinct de `Shield`, qui absorbe un coup entier.
T0b ne code pas cette couche.

**Q4 — partage en coop (bloqué).** Option par joueur : chaque `Inventory` reste local, et le
ramassage est attribué au premier `GgrsNetId` dans l'ordre déterministe. Option équipe : une
ressource rollback d'équipe porte sous, bombes et clés, les prompts lisent cette ressource et
un seul ramassage la crédite. T0b ne choisit ni ne code l'une des deux.

**Q16 — exemplaires.** Proposition par défaut : limites indépendantes pour tous les exemplaires,
avec preuve des traces gungeon touchées ; l'alternative est un champ par objet
`stack: Independent | Shared`. T0b réserve la clé indépendante mais ne branche pas les limites
dans `listed_effects` avant M3-T2.

## 62. Contrats du monde : bombes, rochers, piques, portes verrouillées, grille (M3-T0c)

Contrats seuls, sans exécution : M3-T4 (bombes, rochers cassés, mur secret, piques) et M3-T5
(planificateur en grille) les exécutent. **Aucune trace existante ne bouge** : tout type nouveau est
neutre ou absent du contenu existant, l'input `Bomb` n'est lu par personne.

### Explosion (`effects::Action::Explode`)

`Explode { radius, damage, destroys, friendly_fire }`, en fin d'enum `Action` (les hashs d'avant ne bougent pas ; `friendly_fire` : D113, voir « Action `Explode` »).
Règle pure `world::explosion_plan(grid, center, radius, destroys, candidates)` : les cibles sont les
personnages à PV dont le centre est à **strictement** moins de `radius` (carré en `FixedWide`), rendus
dans l'ordre croissant de `GgrsNetId.0` ; `destroys` casse aussi les rochers de `CellGrid` du rayon
(`destroy_terrain`). Elle vit dans `world` (comme `destroy_terrain`, dans la frontière `SIM`), pas dans
`combat` : elle ne connaît que des identifiants et des positions. Lint : `radius > 0`, `damage >= 0`,
puis `Unsupported` dans un effet tant que M3-T4 ne l'exécute pas ; refusée dans un power-up. Le baril
**ne bascule pas** dessus ici (sa trace ne doit pas bouger) : M3-T4 l'a fait (§69), traces identiques. Moment clé
`explosion` (`world::Exploded`, `FrameEvents` neutre) déclaré, émis en T4.

### Bombe

Bit d'input `INPUT_BOMB` (18), bouton `Bomb` des scénarios (`game::replay::Button`), touche **E**
(présentation ; la manette attend Q17). `CharacterConfig.bomb: Option<world::BombDef>`
(`BombDef { fuse_frames, radius, damage }`, absent = pas de bombe, aucun contenu existant n'en a) ;
composant `world::Bomb { owner: GgrsNetId, fuse_until, radius, damage }`, rollback **neutre**.
`world::bomb_requested(buttons, def, bit)` est la lecture de l'input : faux sans `bomb`. Lint :
`fuse_frames > 0`, `radius > 0`, `damage >= 0`. `fuzz_inputs` : le bit 18 n'a aucun effet sans `bomb`
(personne ne le lit), donc aucune desync possible avant T4.

### Rochers de salle

Couche IntGrid optionnelle **`Rocks`** d'un gabarit LDtk (1 = rocher, 2 = rocher à butin ; le nom
final dépend de Q9). **Écart à la proposition de la fiche** (remplir `CellGrid`) : un rocher est **une
entité rollback** `world::Rock { cell, loot }` (neutre) avec collider (couche des murs) et `Obstacle`
`Wall`, créée à côté des murs (`map_ldtk::game::collider::spawn_level_rocks`), numérotée
`rock_<x>_<y>` dans l'ordre de lecture de la couche, niveaux triés par iid. Raison : `CellGrid` est
dense, ancrée à l'origine et non négative, et `rebuild_cave_walls_system` détruit **tous** les murs pour
les recréer depuis la grille et `reload_walls` remplace les cases murées de la navigation par celles
de la grille : sur un étage assemblé (salles à des positions quelconques, murs IntGrid propres) cela
effacerait les murs des salles. Le champ de flux voit l'`Obstacle` sans code de plus. Un niveau sans
la couche ne crée aucune entité ni aucun id : traces inchangées. M3-T4 détruira un rocher par
`despawn_rollback` (explosion `destroys`). Attente : `EntityCount(kind: Rock)`.

### Piques

`SurfaceDef.hazard: Option<HazardDef { damage, period }>` (fin de structure, absent des surfaces
existantes ; fichier `surfaces/*.ron`, champ `hazard`). Règle pure `HazardDef::is_due(last_hit,
frame)` : un coup au plus toutes les `period` frames. Lint : `damage > 0`, `period > 0`. Dégâts `True`
ou typés (demi-cœurs : Q2) et exécution en T4.

### Portes verrouillées

`RoomKindDef.door_lock: Option<DoorLock>`, `DoorLock { Key(n), Secret }` (`rooms/*.ron`, hors
sérialisation quand absent). `Key(n)` : porte fermée jusqu'à une Interaction avec `n` clés
(`KEY_ITEM`, consommées à l'ouverture) ; `Secret` : mur plein jusqu'à une explosion `destroys`. État
rollback **neutre** `world::DoorLocks` (ensemble des salles dont la porte est ouverte, vide par
défaut). Lint : `Key(0)`. Attente : `DoorLocked(kind, nth, locked, at_frame)` (même tri que
`RoomState`). Types réservés dans le testbed, aucun dans `gungeon` : `rooms/secrete.ron` (`Secret`),
`tresor.ron` (`Key(1)`), `diable.ron` (sémantique en T8).

### Disposition en grille

`FloorGrammar.layout: Option<FloorLayout>`, `FloorLayout { Graph (défaut), Grid { cell, max } }`
(Q8, proposition retenue : `cell` = 15 × 9 cases, une seule taille ; **30 × 18 depuis m3-salles-murs-isaac, §40**). Lint (`lint_floor_grammars`) :
`cell` et `max` > 0, `max` contient au moins `rooms.0` salles, et chaque gabarit de la grammaire
fait exactement `cell` cases avec une ouverture centrée sur chacun des quatre côtés. `plan_floor`
**refusait** `Grid` (`FloorError::Unsupported`) tant que M3-T5 n'était pas livrée : le planificateur est
`map::generation::grid` (§70), le lint des gabarits passe aux multiples de `cell` et aux ouvertures par
case de salle (§70), `Grid` gagne `loops`, `secret` et `post_boss`.

### Testbed et scénarios

Carte `testbed/rocks_static.ldtk` (`scripts/construire-cartes-t0c.py`) et scénario `rocks_static` :
le joueur marche vers trois rochers qui ferment un couloir et s'y arrête ; quatre rochers comptés
(dont un à butin) à f5 et à la fin ; `joueur_hors_mur` tient.


## 63. Contrats des familiers, orbitaux et champions v2 (M3-T0d)

Contrats seuls : rien n'est rencontré en partie ordinaire, aucune trace ne bouge. M3-T6 exécute les
familiers, M3-T7 les champions.

**Behaviors de familier** (`behaviors::Behavior`, variantes **en fin d'enum**) : `Follow { gap }` (suit
le porteur à `gap` unités, en file derrière les autres familiers du même porteur : rang = ordre
d'apparition), `Orbit { radius, frames_per_turn }` (tourne autour du porteur, phase = rang × 2π /
nombre d'orbitaux, par `fixed_math`), `MirrorShot { every }` (tire quand le porteur tire, dans
sa direction de visée, au plus une fois tous les `every` frames ; **Q13** : les larmes du porteur, voir §71 « Q13 »), `BlockBullets` (une balle hostile qui touche le familier est détruite par
`despawn_rollback`, comme le blank ; le familier est immunisé). `Behavior::is_familiar`. `select`
ne retient jamais ces règles (un familier n'a pas d'IA de poursuite) ; kinds `Follow`, `Orbit`,
`MirrorShot`, `BlockBullets` déclarés (`register_kinds`). Un familier est un **personnage** de
`characters/` (équipe `Players` ; l'IA ennemie ne cible que `With<Player>`, un familier n'est donc ni
visé ni attaqué : le ciblage ne change pas ici, allié qui combat = D5).

**Lien au porteur.** `behaviors::FamiliarOf { owner: GgrsNetId, rank: u32 }` : composant rollback
**neutre** (`rollback_and_trace_neutral`), contribution 0 sans porteur. Créé et détruit par les
objets en M3-T6 ; son `GgrsNetId` viendra de `id_factory` au ramassage, dans l'ordre déterministe des
ramassages de la frame.

**Champ d'objet.** `ItemDef.familiar: Option<FamiliarRef { character, per_copy }>` (`per_copy` : 1 par
défaut, par exemplaire tenu). Définition hors rollback et hors hash. **Lint** : personnage inconnu
(`BrokenReference`), personnage sans behavior de familier, `per_copy = 0`, `gap`/`radius` ≤ 0,
`frames_per_turn = 0`, `every = 0` ; réciproquement un behavior de
familier sur un personnage qu'aucun objet ne cite est refusé (`Unsupported`, « sans porteur »).

**Champions v2.** Champs optionnels de `VariantDef` (hors hash : le composant `Variant(nom)` ne change
pas) : `on_death: [Action]` (vide par défaut), `behaviors: Option<Vec<Behavior>>` (remplace la liste
du personnage), `loot: Option<String>` (pool de butin lâché à la mort, graine de §48 avec la clé de
contenu `champion:<net_id>`), `tint: Option<String>` (`#rrggbb`, présentation seule). `chance`
accepte un nombre (valeur inchangée) **ou une expression** (`NumOrExpr`, §18) avec les identifiants
`players` et `floor` ; `VariantsConfig::chance_at(players, floor)`. Tant que M3-T7 n'exécute rien, le
tirage de la variante résout l'expression avec 1 joueur à l'étage 0 (`draw_variant`) ; un nombre est
toujours exact, le tirage de la graine de §25 est donc inchangé.
**Nouvelle action** `Action::SpawnCharacters { id, count }` (fin d'enum) : fait apparaître `count`
personnages à la position de l'action, à des **décalages fixes** autour du point, sans RNG (justification
contre `SpawnEnemies`, qui tire ses positions dans le flux RNG de la salle `room:<niveau>` : la mort d'un
champion ne doit pas déplacer ce tirage). **Lint** des actions à la mort : seules `Explode`, `SpawnCharacters`
et `SpawnPattern` sont admises (`Unsupported` sinon ; `SpawnCharacters` est refusée dans un effet, un
power-up et une frise de boss) ; `Explode` rayon > 0 et dégâts ≥ 0, `SpawnCharacters` personnage connu et
`count > 0`, `SpawnPattern` pattern et arme connus, pool de butin connu, couleur `#rrggbb`, `behaviors`
non vide et valides (mêmes règles que `ai.behaviors`), `chance` dans [0, 1] pour 1 et 4 joueurs aux
étages 0 et 5.

**Preuves.** Fixtures `familiar_ok`, `familiar_unknown_character`, `familiar_orphan_behavior`,
`familiar_without_behavior`, `familiar_gap_zero`, `champion_ok`, `champion_bad_death`,
`variant_chance_expression_range`, `variant_chance_expression_unknown` ; contenu `testbed` : personnage
`compagnon` (`Follow` + `MirrorShot`) et objet `compagnon` (`familiar`), accepté par le lint,
sans scénario de jeu (le ramassage ne crée rien encore). Attente de scénario `FamiliarCount { handle,
count, at_frame }` (familiers dont le porteur est le joueur ; 0 tant que M3-T6 n'existe pas).

## 64. Curses, pools à épuisement, accomplissements : contrats (M3-T0e)

Contrats et règles pures seuls : rien n'est rencontré en partie ordinaire, **aucune trace ne bouge**
(`curses: None`, `exhaust: false`, ressources neutres vides). Le tirage d'une curse et ses effets sont
M3-T9 (HUD : T16), l'épuisement en jeu et la salle du diable M3-T8, l'écriture des accomplissements dans
le profil M3-T10. Réponses de William du 2026-10-16 : **Q5** (épuisement par run et par pool, tirage
indépendant du moment), **Q7** (les cinq curses), **Q6** (le diable refuse si le paiement tuerait ; les
cœurs d'âme paient à la place : T8).

### Curses (`run::curse`, kind `Curse`)

- **Contenu** `curses/<id>.ron` (id = nom de fichier ; `game.ron` : `(path: "curses", kind: "Curse")`) :
  `CurseDef { weight: u32, effects: [CurseEffect] }`. `CurseEffect`, liste fermée, variantes **en fin
  d'enum** : `HideMinimap`, `HideHealth`, `HideItemNames` (présentation seule : lues par le HUD, T16),
  `FloorRooms("2.0")` (multiplicateur du nombre de salles : lu par la grammaire en grille, T5/T9),
  `Modifier(stat, op, "valeur")` (posé sur chaque joueur pour la durée de l'étage ;
  `CurseEffect::to_modifier(id)` : source `curse:<id>`, comme `item:<id>`, sans expiration, retiré à la
  sortie de l'étage par T9).
- **Les cinq de Q7** (exemples du `testbed`) : `darkness` (`HideMinimap`), `unknown` (`HideHealth`),
  `blind` (`HideItemNames`), `labyrinth` (`FloorRooms("2.0")`), `maze` (`Modifier(MoveSpeed, Mul,
  "0.85")`). `Modifier` est la variante générique (Maze/Lost d'Isaac : vitesse ou chance en moins) ; ne pas
  en coder d'autres que ces cinq sans nouvelle réponse.
- **Tirage par étage** : champ optionnel `curses: Some((chance: "0.5", pool: ["darkness", …]))` d'une
  séquence `Floors` (`FloorsEntry::curses`, `run::CurseConfig`) ; absent = aucune curse (toutes les
  séquences existantes). `run::draw_curse(config, defs, graine, étage)` (M3-T9 : graine de carte, §74) : RNG **local**
  `RollbackRng::new(curse_seed)` avec `curse_seed = fnv1a("curse:<graine>:<étage>")` replié sur 32 bits
  puis passé au finaliseur `fmix32` (`bevy_fixed::rng::seed_from_hash`, D95 ; même graine que `loot_seed` et
  `variant_seed`, sans allocation), **aucun flux de `RngStreams`** (en créer un changerait le hash de la
  ressource et toutes les traces). Deux tirages toujours consommés, dans l'ordre : la chance
  (`next_fixed() < chance` : `0` ne tombe jamais, `1` toujours), puis le choix pondéré dans l'ordre du
  pool (`next_u32() % Σ poids`). Une entrée inconnue ou de poids `0` est ignorée (le lint la refuse) ; sans
  entrée valide, pas de curse. Le résultat ne dépend que de la graine et de l'étage : pas du moment où
  l'étage est entré, ni de l'ordre des étages.
- **État** `run::RunCurses { floors: BTreeMap<u32 étage, String curse>, entered }` (`entered` ajouté par M3-T9, §74) : ressource rollback **neutre**
  (`RunPlugin`, `rollback_and_trace_resource_neutral`), vide donc de hash `0` tant que T9 n'a rien écrit ;
  remise à zéro par la relance (`game::run_state::cleanup_rollback_world_system`). T9 écrit l'entrée de
  l'étage à l'entrée dans l'étage (événement `FloorEntered`) avec `draw_curse`.
- **Lint** (`lint_curses`) : `weight = 0`, `effects` vide, `FloorRooms` ≤ 0, `Modifier` `Mul` ≤ 0
  (`OutOfRange`) ; `curses.chance` hors [0, 1], `curses.pool` vide (`OutOfRange`) ; curse du pool inconnue
  (`BrokenReference`).

### Pools : `table` suffit, et l'épuisement

- **Pas de champ `pool`.** Un coffre (`RoomChestDef.table`), une boutique (`RoomShopDef.table`,
  `floor_tables`) et une séquence (`FloorsEntry.loot`) nomment déjà un pool de butin
  (`loot/<id>.ron`) ; un type de salle n'a rien de plus à porter. Les pools du boss et du diable (T8) sont
  de nouvelles tables nommées par la même clé. Le « pool » de l'épuisement est donc **le nom d'une table**.
- **`LootTableDef.exhaust: bool`** (défaut faux : comportement d'avant, hors de tout hash et de toute
  trace) : un objet sorti du pool n'y revient plus pendant la run. **Par run et par pool** (Q5) : un pool
  partagé par plusieurs salles est le même pour toutes. `weapons_only` garde le drapeau.
- **État** `items::PoolState { seen: BTreeMap<pool, BTreeSet<objet>>, assigned }` (M3-T8, §73 : `assigned` conserve
  l'objet assigné à chaque emplacement ; tuple `PoolState(BTreeMap)` avant T8) : ressource rollback **neutre**
  (`LootPlugin`), vide tant qu'aucun pool n'a `exhaust` ; remise à zéro à chaque relance. Les armes y sont
  nommées `weapon:<id>` (comme le tirage). `count(pool)` est l'**index d'épuisement** du prochain tirage ;
  `record(pool, objet)` renvoie vrai pour une nouveauté.
- **Règle pure** `items::draw_exhausting(table, items, held, rareté, consommables, hidden, seen, rng)` :
  `draw_loot_with` la rappelle avec un `seen` vide. Si `table.exhaust`, les objets de `seen` ne sont plus
  candidats ; sinon `seen` est ignoré. Le flux avance toujours de deux tirages, quel que soit `seen` (comme
  `draw_loot`). Quand l'épuisement vide les
  candidats : d'abord les assouplissements de `draw_loot` (rareté du coffre puis `Common`, exclusion des
  objets tenus), **un objet neuf plus commun passe avant de répéter un objet vu**, puis, pool entièrement vu,
  le pool **repart complet** (un objet déjà sorti peut ressortir ; jamais vide si le pool ne l'est pas). Une
  récompense de salle (`consumables_only`) ne descend jamais sous les consommables. Les déblocages
  (`unlocked_table`) étendent le pool avant le filtre.
- **Le choix est une course pondérée, stable à l'ajout (D96, 2026-10-17).** Les deux tirages du flux forment une
  clé de base de 64 bits. Niveau 1 : chaque entrée joue **autant de billets que son poids**, de clé
  `fmix64(fmix64(base ^ contenu de l'entrée) ^ n° du billet × φ)` ; la plus petite clé gagne, donc l'entrée gagne avec la probabilité
  exacte `poids / somme` (une clé divisée par le poids ne la donnerait pas : poids 1 et 2 → 3/4, non 2/3). Niveau 2 : parmi
  les candidats de l'entrée gagnante (une entrée `Rarity` en a plusieurs), la plus petite clé `hash(base, entrée, identifiant)`
  gagne : uniforme. L'identité d'une entrée est le **contenu** de son `pick`, jamais son rang (les entrées d'un déblocage
  s'ajoutent dans l'ordre des identifiants de déblocage) ; deux entrées de même `pick` valent une entrée de poids somme, et
  l'ordre des entrées n'a aucun effet. Avant D96, le choix était `tirage % somme des poids` : débloquer un objet changeait la
  somme (10 → 12 pour le pool `boss` d'`isaac`) et **redistribuait tous les emplacements** (mesuré : 251 emplacements sur 400
  passaient à un objet non débloqué).
  **Contrat** (l'ordre de tirage et `PoolState::assign` sont inchangés) : *à graine, `seen`, passe de rareté et exclusions égaux,
  ajouter une entrée, débloquer un objet caché, ajouter un candidat à une entrée `Rarity` ou augmenter un poids ne change
  l'objet d'un emplacement que si c'est pour un objet qu'apporte l'ajout* (l'objet d'une entrée `Item`, l'arme ou le tas de sous d'une
  entrée `Weapon` ou `Currency` ; pour une entrée `Rarity` ajoutée, alourdie ou fusionnée avec une entrée de même contenu, **n'importe lequel
  de ses candidats**, y compris un qui existait déjà ; inversement, retirer ce qui n'a pas gagné ne change rien). Un
  contenu qui s'enrichit (T11b, T11c… : des objets ajoutés aux pools) ne fait donc plus bouger que les emplacements qui prennent
  le nouvel objet, et un déblocage de profil (§75) ne déplace que vers ce qu'il apporte. **Limites** : (1) dans une suite
  d'emplacements à épuisement, l'objet qu'un déblocage déplace revient dans le pool et peut sortir à un emplacement
  **suivant** (`seen` n'est plus le même) ; la garantie est par tirage, à `seen` égal ; (2) si l'ajout rend non vide une passe
  plus stricte (rareté d'un coffre), c'est cette passe qui tire ; (3) coût : deux `fmix64` par unité de poids du pool (`race_key`), d'où
  `items::MAX_LOOT_WEIGHT` (1 000), refusé au lint (`OutOfRange`) et plafonné au tirage ; (4) une boutique **sans épuisement** (`game::shop::draw_shelves`) enchaîne ses présentoirs sur un seul flux, avec relance des doublons (au plus 8 essais) : un ajout qui change le présentoir 0 change le nombre d'essais, donc le tirage, des suivants ; la garantie vaut par tirage, pas pour une boutique entière. Proportions mesurées sur 100 000
  tirages (`the_race_keeps_the_weighted_proportions`) : voir le rapport `dette-d95-d96-tirage-butin`.
- **Graine** `items::exhaust_seed(graine, étage, clé, index)` : `loot_seed` pour l'index `0` (le premier
  tirage d'un pool qui devient `exhaust` donne l'objet qu'il aurait donné sans `exhaust`),
  puis `loot_seed(…, "<clé>#<index>")` (mêmes octets, sans allocation).
  `PoolState::next_seed(pool, graine, étage, clé)` prend `count(pool)` comme index.

**Deux façons d'exécuter l'épuisement (décision de T8 : A, voir §73 ; le tableau est celui de T0e).**

| | A. tirage à la génération de l'étage (proposé) | B. tirage à l'ouverture |
|---|---|---|
| Quand | à la création des emplacements (salles dans l'ordre du plan, graine de l'emplacement) | quand le joueur ouvre le coffre / entre dans la boutique |
| Index d'épuisement | rang de l'emplacement dans le plan : **indépendant du moment** (Q5) | nombre d'objets déjà sortis à l'ouverture : dépend de l'ordre des ouvertures |
| Coût | l'objet d'un coffre jamais ouvert est quand même sorti du pool ; il faut **porter l'objet assigné** (un champ optionnel de `Chest`/`Shelf` hors hash quand vide, ou une table d'assignation dans `PoolState`) puisque l'ouverture ne tire plus | rien à porter (le tirage actuel de `chest_interaction_system`) |
| Rollback | l'état (`PoolState` + assignations) est écrit une seule fois par étage, à la génération | écrit à chaque ouverture : un rollback qui remonte avant une ouverture doit défaire `PoolState` (il le fait, c'est une ressource rollback) |

L'option A est la seule qui tienne la règle « un coffre ouvert tôt ou tard donne le même objet » de §48 avec
un pool qui s'épuise ; la règle pure sert les deux (T8 choisit en passant `seen` de la génération ou de
l'ouverture). **Décision de T8 : option A**, exécutée et documentée au §73 (assignation à la génération de
l'étage, conservée dans `PoolState::assigned`).

### Accomplissements (`meta::achievement`, kind `Achievement`)

- **Contenu** `achievements/<id>.ron` : `AchievementDef { when: [AchievementCondition], unlocks: [id d'Unlock] }`.
  Conditions, cumulatives (ET) : `Victory` (issue `RunEnd::Victory`), `FloorAtLeast(n)` (même index que
  `RunSummary::floor_reached` et l'attente `FloorIndex` : `0` = premier étage), `BossDefeated(id)` (un
  personnage à `boss:`), `ItemHeld(id)` (objet tenu à la fin).
- **Règle pure** `meta::achieved(&RunFacts, &AchievementDef)` ; une définition sans condition n'est jamais
  accomplie. **Hors simulation** : évalué à la fin de la run, comme l'écriture du profil (`game::profile`) ;
  `meta` ne lit ni n'écrit rien, T10 réunit les faits et ajoute les `unlocks` au profil.
- **`RunFacts` plutôt qu'un `RunSummary` élargi.** La fiche proposait des champs optionnels de `RunSummary`
  (boss vaincus, objets tenus). `RunSummary` est `Copy`, dans `Run` (rollback, haché) : y ajouter des listes
  de chaînes l'alourdirait pour un état que la simulation ne lit jamais. `RunFacts { summary, bosses_defeated,
  items_held }` est hors rollback ; T10 lit les boss vaincus (`BossDefeated`, hors checksum, §45) et
  l'`Inventory` des joueurs à `Ended`.
- **Lint** (`lint_achievements`) : `when` vide, `unlocks` vide (`OutOfRange`) ; déblocage, objet ou boss
  inconnu (`BrokenReference`).

### Attentes de scénario

`Curse { floor, id: Some("darkness") | None, at_frame }` (curse de l'étage `floor`, index de `FloorIndex`,
ou absente) et `PoolSeen { pool, item, seen, at_frame }` (`item` est ou non sorti du pool). Elles lisent
`RunCurses` et `PoolState`. `PoolState` n'est rempli qu'à partir de T8 ; `RunCurses` l'est par M3-T9 (§74) : un scénario `.ron` ne peut donc
encore y poser que `None` / `seen: false` (ce que fait le témoin ci-dessous) ; le test
`crates/scenario/tests/curses_pools.rs` les remplit depuis un système de `GgrsSchedule`.

### Déterminisme et traces

Aucune trace existante ne bouge : `RunCurses` et `PoolState` sont enregistrées par
`rollback_and_trace_resource_neutral` (contribution `0` à leur valeur par défaut, aucune parité de type vide
à compenser), `LootTableDef.exhaust` et `FloorsEntry.curses` sont des définitions hors hash, les RNG de
curse et de pool sont locaux. `RunSummary` est inchangé. Les kinds `Curse` et `Achievement` ont leur schéma
(`every_kind_has_a_schema`).

### Preuves

Unitaires : `run::curse` (chance 0 et 1, indépendance de l'ordre des étages, pondération, entrées
inutilisables, le tirage de chance ne dépend pas du pool, `to_modifier`), `items::loot` (`exhaust` faux
= `draw_loot`, objets vus exclus puis pool qui repart complet, armes, rareté assouplie avant répétition,
consommables, flux identique quel que soit `seen`, graine à l'index 0 = `loot_seed`, ordre du plan sans
doublon), `meta::achievement` (chaque condition, seuil inclusif, ET, sans condition). Fixtures de lint :
`curse_invalid_values`, `floors_curses_invalid`, `achievement_invalid` (`crates/content/tests/
m3_t0e_fixtures.rs`). Contenu `testbed` : cinq curses, séquence `maudit` (`curses: Some(…)`), pool
`epuisable` (`exhaust: true`), accomplissement `deux_etages` et déblocage `manche_epuisable` (un `Loot` sur
`epuisable` seulement : aucun pool existant ne change). Test `curses_pools` : les ressources écrites en
cours de partie traversent le synctest sans divergence, les attentes les lisent (et échouent avec la valeur
observée), la relance les remet à zéro.

### Atteignabilité

Contrats : rien en partie ordinaire. T9 tire une curse dans la séquence `Floors` d'`isaac` (graine nommée),
T8 active `exhaust` sur le pool du trésor, T10 écrit un accomplissement à la fin d'une partie ordinaire.

## 65. Le jeu `isaac` : squelette (M3-T0f)

Le clone `isaac` démarre, sur la grammaire d'étage **en arbre existante** (`Graph`) : rien de nouveau
côté moteur. Modèle : `gungeon` (§39). Données et un `main.rs` seulement (`games/isaac/`, membre du
workspace par `games/*`, `make isaac`, `make lint` ×5 avec `games/isaac`).

- **Manifeste** `games/isaac/assets/game.ron` : mode `Floors`, séquence `floors/sous_sol.ron` (nom
  provisoire, Q9) d'un seul étage `floor:sous_sol`, `victory_at_end` ; `generate_template` copié de
  `gungeon` (scénarios générés `make gen GAME=isaac`).
- **Personnage** `pleureur` (`asset_name_ref: "player"`) : 6 PV (Q2, en attendant), aucun dash ni
  sprint, arme `larmes` (`Automatic`, 3 tirs/s, chargeur 9999 faute de munition infinie ici : le
  `mag: Infinite` de M3-T0a viendra avec M3-T11a). Contact des ennemis : `ai.attack_damage` (1 PV) et
  `attack_cooldown_frames` (120) ; **l'arme de mêlée nommée dans `Melee("contact")` ne fixe ni les
  dégâts ni la cadence** (`enemy_attack_damage_translate_system` lit l'`EnemyAiConfig`, défaut 10 PV et
  60 frames) : ne pas s'y fier pour équilibrer un ennemi au corps à corps.
- **Ennemis** : `suiveur` (`Chase` profil `Ground` + contact), `mouche` (`Chase` profil `Flying` +
  contact), boss `gardien` immobile à deux phases (éventail, puis couronne sous 50 % de santé).
  Valeurs choisies pour que les bots `fonceur` (scénario généré `boss_gardien`) et `explorateur` (6 PV, aucune
  esquive du premier) finissent l'étage : 10, 4 et 40 PV, un coup de contact toutes les 120 frames.
- **Gabarits** `maps/salles.ldtk`, produit par `docs/taches/rapports/m4-t0f-squelette-isaac.make_salles.py` :
  salles de **15 × 9 cases** (Q8, préparation de la grille de M3-T5), types `depart`, `combat`
  (un suiveur, une mouche), `boss`. Ouvertures de 3 cases **centrées** (colonnes 6-8 au nord et au sud,
  rangées 3-5 à l'ouest et à l'est). `depart` et `combat` ont les quatre ; le boss a **une seule** ouverture
  (quatre gabarits `BossN/E/S/W`) car le planificateur en arbre exige un boss à une connexion
  (`floor.rs`, « cul-de-sac ») : la grille de M3-T5 apportera ses propres gabarits.
- **Grammaire** `floor_grammars/sous_sol.ron` : 5 à 7 salles, un boss requis à au moins 2 salles du
  départ, `Graph`. Test `map_ldtk::generation::tests::isaac_etages_de_grammaire` : 1 000 graines
  (`ALACOD_FLOOR_SEEDS`, 500 par défaut, doublées par la graine dérivée), 5 à 7 salles, un seul boss,
  toutes de 240 × 144, sans chevauchement.
- **Sprites, sons, UI** : copiés de `gungeon` (placeholders Kenney CC0 et sons du registre), registre
  `assets.yaml` de `isaac` renvoyant à celui de `gungeon` ; aucun nom ni visuel tiré du jeu commercial.
- **Scénarios** : `isaac_boots` (démarrage, tir continu sans rechargement), `isaac_floor_seed_1`
  (le bot `explorateur` bat l'étage, victoire à f1009) ; générés : `weapon_*`, `enemy_*`, `room_combat`,
  `boss_gardien`.

## 66. Larmes v3 : tir composé, `Spectral`, `Split`, `Orbit`, `Poison`, `Fear`, chance d'effet (M3-T1, chantier B5 v3)

M3-T1 fait **lire au tir** les contrats de §60 et ajoute les statuts v2 et la chance d'effet.
**Aucune trace existante ne bouge** (suite complète sans `BLESS`) : tout passe par des objets, des
modificateurs ou des champs que le contenu d'avant n'a pas. Scénarios neufs seulement : douze
(`tears_*`, `status_poison`, `status_fear`) et les générés `weapon_status_poison`/`weapon_status_fear`,
`item_*` des neuf objets d'essai.

### Tir composé

`weapon_rollback_system` lit, au tir d'un joueur, `items::Inventory` (le tireur), `ItemTable` et
`SynergyTable` (`items::projectile_mods` : passifs dans l'ordre de ramassage, un passif en double
compte deux fois, puis l'actif, puis les synergies actives par id). Les `ProjectileMods` **non vides**
sont composés avec le `projectile:` du mode de tir par `Projectile::composed` (règles de `compose`,
§60) ; leur `on_hit` et leur `on_expire` s'**ajoutent** à ceux de l'arme, dans le même ordre.

- Aucun objet qui ajoute quelque chose (cas de tout le contenu d'avant) : chemin d'origine,
  `Projectile::new` ou balle ordinaire sans `Projectile`, au bit près.
- Une balle ordinaire devient un `Projectile` (id : le nom de l'arme) dès qu'un objet ajoute quelque
  chose ; `Size` agrandit alors collider et sprite, comme pour une arme.
- `Projectile` (rollback) gagne `spectral`, `split`, `orbit`, `luck` ; son `Hash` est **manuel** :
  ces champs ne sont hachés que s'ils sont actifs, donc le hash d'un projectile d'avant M3-T1 est
  inchangé (même règle pour le `Debug` du dump détaillé).
- `combat` dépend maintenant de `items` (crate de simulation, sans rendu : `check_sim_boundary` inchangé).
- Familiers (Q13, question ouverte) : le tir d'un familier n'est **pas** composé ici ; `Projectile::composed`
  prend ses ajouts en paramètre, M3-T6 y passera `items::projectile_mods` du porteur s'il retient l'héritage.

### `Spectral`, `Split`, `Orbit`

- **`Spectral`** : `projectile_collision_system` saute la branche des murs (rebond, arrêt, `BulletWallHit`) ;
  les personnages touchent toujours. La portée (`range`) borne encore le vol.
- **`Split { count, spread, child }`** : à chaque impact sur un personnage (après `register_hit`),
  `spawn_split_children` fait naître `count` projectiles en éventail de `spread` radians centré sur la
  direction de vol, à la position du parent, un `GgrsNetId` chacun dans l'ordre de l'éventail.
  `child: Some(id)` : l'entrée `id` de la table `projectiles` de l'arme (dégâts, vitesse, portée et
  modificateurs de sa définition) ; `child: None` (**Q3 : le projectile par défaut du porteur**) : une
  **copie du parent** (mêmes dégâts, vitesse et portée, modificateurs et `on_hit` restants, sans `Split`,
  sans `Orbit`, sans `on_expire`). Dans les deux cas l'enfant **hérite de la liste des cibles déjà touchées**
  (il ne retouche pas la cible qu'il quitte) et compte une génération de plus : au-delà de `MAX_GENERATION`
  (8) la division est ignorée avec un `warn!`. L'enfant n'hérite pas de `ShotSize`. Lint : un cycle de
  `Split` dans la table (`a → b → a`) est refusé (`OutOfRange`).
- **`Orbit { radius, frames_per_turn }`** : `projectile_steering_system` fixe chaque frame la vitesse
  de la frame suivante à l'écart jusqu'à `combat::projectile::orbit_position` : angle
  `phase + 2π · (âge mod frames_per_turn) / frames_per_turn` (sens trigonométrique, `phase` = direction
  de tir, fixée à l'apparition), rayon qui croît de 0 à `radius` pendant 10 frames
  (`ORBIT_RAMP_FRAMES`), centre = position du **tireur** à cette frame. Ni gravité ni guidage en orbite
  (lint : `Orbit` avec `Homing` ou `Gravity` dans la même liste est refusé). Le projectile ne s'arrête que
  par un coup (`Pierce`), un mur (sauf `Spectral`), `Lifetime`, la portée (la longueur d'arc compte) ou la
  disparition du tireur (portée mise à 0).

### Statuts `Poison` et `Fear`

`StatusDef::{Poison, Fear}` en fin d'enum (`status_def_ron_round_trip` ; `StatusDef` avait quatre variantes,
le `Hash` dérivé garde l'index des anciennes). Contenu : `(kind: Poison, frames: 120, damage: "2", period: 30)`,
`(kind: Fear, frames: 150)` ; le lint exige `damage > 0` et `period > 0` pour `Poison`, `frames > 0` pour les deux.

- **`Poison`** : exactement les règles de `Burn` (ticks, piles jusqu'à 2, durée plafonnée à 2 × la base,
  dernière source créditée) ; dégâts de genre `DamageKind::Poison` (`status_tick_system`).
- **`Fear`** : `Statuses::feared()`. L'ennemi **fuit** sa cible : `move_enemies` impose le recul de
  `Flee` (`ai::rules::flee_motion`, extrait de `behavior_motion` : champ de recul de l'ennemi, sinon à
  l'opposé de la cible) qu'il liste `Flee` ou non ; `enemy_attack_system` ne lance plus d'attaque au corps à
  corps et `melee_hold_system` pose `MeleeHold` (mêlée d'arme) tant que la peur dure. Réapplication :
  durée rafraîchie ; sans effet sur un joueur. **Charme : Q12**, hors tâche (rien codé).

### Chance d'effet

`Action::Chance { base, per_luck, then }` (en fin d'enum `Action`) enveloppe une action d'`on_hit`
(`ApplyStatus`, `TimedModifier`, `CurrencyMultiplier` ; ni une autre `Chance` ni `DestroyTerrain`) :

```ron
projectile: Some((on_hit: [Chance(base: "0.5", per_luck: "0.125", then: ApplyStatus(status: "poison", stacks: 1))]))
```

Chance finale `effects::chance_with_luck(base, Luck, per_luck)` (§60), `Luck` lue **au tir** et gardée dans
`Projectile::luck` (seulement pour un projectile qui porte une `Chance`). Le tirage est un RNG **local** :
`effects::chance::roll_chance(run_seed, net id du projectile, rang de l'action)`, graine mélangée
(`chance_seed`, splitmix64 : deux net ids voisins ne donnent pas deux tirages voisins), aucun `RngStreams`
consommé (la graine de run est lue seule). Il est résolu dans `projectile_collision_system` (le seul à
connaître le net id) : l'événement `ProjectileHit` porte l'action `then` si le tirage réussit, la `Chance`
elle-même sinon (les systèmes d'application l'ignorent), ce qui garde le **rang** de chaque action, donc la
source `projectile:<id>:<rang>` des modificateurs posés. Lint : `base` dans `[0, 1]`, statut vérifié à travers
la `Chance`. Refusée (`Unsupported`) dans un effet, un power-up et une frise de boss.

### Lint

`Spectral`, `Split` et `Orbit` ne sont plus refusés dans une arme ; un objet ou une synergie à `projectile:`
est vérifié sans être refusé : mêmes plages qu'une arme, doublon refusé **dans une liste**, `on_hit` comme
une arme (statut connu), et, faute de table `projectiles` (celle de l'arme qui tire), `Split` **sans** `child`
et `on_expire` **sans** `Spawn` (`DestroyTerrain` seulement). Fixtures : `projectile_contract_ok`,
`item_projectile_ok`, `item_projectile_no_table`, `item_projectile_out_of_range`,
`projectile_chance_out_of_range`, `projectile_split_cycle`, `status_poison_fear_out_of_range`.

`sim_core::stats::default_base` donne 0 à `Luck` (attente `Stat`, générateur `ItemHeld`).

### Testbed et scénarios

Carte `testbed/props_room.ldtk` (salle de 28 × 9 cases, un joueur en (696, -200), un `follower` en
(328, -200) ; monde = `(x_ldtk + 280, -120 - y_ldtk)`), cibles `target` posées par `characters`. Objets
d'essai (`tags: ["larmes"]`) : `guide` (`Homing`), `perforant` (`Pierce(1)`), `rebond` (`Bounce(2)`),
`spectre` (`Spectral`), `division` (`Split` en 3, 0,6 rad), `orbite` (rayon 40, un tour en 60 frames),
`venin` (`Chance` 0,5, +0,125 par `Luck`, `ApplyStatus poison`), `trefle` (`Luck` +4). Armes `status_poison`,
`status_fear`. Statuts `poison`, `peur`.

| Scénario | Prouve |
|---|---|
| `tears_item_homing`, `..._sans_objet` | un objet ajoute `Homing` à une arme sans modificateur : touche à f58 ; témoin : aucun coup |
| `tears_stack_two_copies` | deux `perforant` : `Pierce` 1 + 1, la larme touche trois cibles sur quatre |
| `tears_stack_three` | `rebond` + `guide` + `division` : une larme, impact, trois larmes de génération 1 qui se courbent vers le `follower` |
| `tears_spectral_wall`, `..._sans_objet` | `Spectral` franchit le mur de la salle et touche la cible derrière (f49) ; témoin : arrêtée par le mur |
| `tears_split` | une larme, trois enfants qui touchent chacun sa cible (f72), personne touché deux fois |
| `tears_orbit` | la larme tourne autour du joueur et touche la cible derrière lui (demi-tour, f48) |
| `status_poison` | ticks de 2 toutes les 30 frames, santé exacte de la cible, expiration |
| `status_fear` | le `follower` touché fuit 150 frames (distance ≥ 360 à f300), reprend sa marche, ne touche pas le joueur |
| `tears_luck_chance` | six larmes, chance 0,5 : quatre cibles empoisonnées (graine de run, net id, rang) |
| `tears_luck_stat` | même tir avec `trefle` (`Luck` 4 → chance 1) : les six |

Diagnostic : `ALACOD_PROJECTILES=<scénario>:<de>:<à> cargo test -p scenario --profile headless
--no-default-features --test scenarios projectile_probe -- --ignored --nocapture` (projectiles, coups,
statuts frame par frame) ; `map_probe` liste aussi les personnages non joueurs.

### Visée à quatre directions, `aim: FourWay` (suivi Q17, réponse de William du 2026-10-16)

Option de personnage (**donnée**) : `aim: Free` (défaut, 360°, tout le contenu d'avant) ou `aim: FourWay` (`isaac` :
`pleureur`). Type `sim_core::aim::AimMode` (`CharacterConfig::aim`, miroir `content::CharacterEntry::aim`, schéma
généré). **Hors hash et hors trace** : composant **statique, non rollback** (comme `Team`, voir `sim_core::aim`), posé à
la création du joueur **seulement pour `FourWay`** (`character::player::create`) ; un personnage sans `aim:` ne porte
rien, aucune trace d'un autre jeu ne bouge (suite complète, 0 `BLESS` hors la liste ci-dessous).

**Quantification dans `weapon_rollback_system`** (`sim_core::aim::aim_direction(mode, pan_x, pan_y)`, un seul code pour
le tir du joueur et le `MirrorShot` de ses familiers, §71) : `FourWay` ne garde que l'**axe dominant** du `pan`
(comparaison des entiers `|pan_x|` et `|pan_y|`, avant toute normalisation : ni division ni arrondi, `Fixed` exact), avec
le signe de sa composante, vecteur unitaire `(±1, 0)` ou `(0, ±1)`. **Égalité exacte des deux axes (diagonale parfaite) :
l'horizontale l'emporte** ; `pan` nul : pas de visée (aucun tir, comme avant). `Free` : la normalisation d'avant, au bit
près. La dispersion (`spread`) et les plombs d'un fusil à pompe tournent ensuite autour de cette direction, comme avant.
Limite connue : la **visée affichée** (rotation du sprite d'arme, `CursorPosition` dérivé du `pan` réel) reste celle de la
souris ; seule la direction des balles est quantifiée.

**Scénarios** : `PlayerScript::aim: Option<AimMode>` (réglage de joueur, comme `bomb`) : `Some(FourWay)` pose la visée,
`Some(Free)` **retire** celle du personnage. `aim_four_way` (testbed, trois tirs en biais à 17°, 17° et 45° exact : les
trois larmes partent plein ouest et touchent la cible alignée aux frames 80, 105 et 129 ; la diagonale parfaite vérifie le
choix de l'horizontale) et son témoin `aim_four_way_libre` (visée libre : aucun coup).

**Bot** (`crates/bots`) : `BotView::aim` (lu sur le composant du joueur), `decide::apply_four_way` appliqué à la sortie de
`decide` pour tous les profils : le bot écrit la direction que le moteur retiendra (127 sur l'axe dominant), et, cible à
moins de 320 sur cet axe **avec ligne dégagée** (`enemy_visible && enemy_shootable` : derrière un mur, il garde sa route) :
hors de la file de tir (écart perpendiculaire > 14, rayon de larme 5 + demi-corps ~10 − 1) il relâche la gâchette et se
décale **vers la cible** sur l'axe perpendiculaire (le mouvement du profil y est remplacé, celui sur l'axe de tir est
gardé) ; dans la file il tire comme avant et ne dérive plus en perpendiculaire. `Free` : l'input du profil, inchangé.
Le bot reste efficace mais plus lent : sur `grid_post_boss` (graine 8, avant la fusion de T11c) le boss verrouille sa salle à
f712 au lieu de f628, passe en phase 1 à f825 au lieu de f740 et meurt à f945 au lieu de f862.

**`isaac`** : `pleureur` active `aim: FourWay`. **Générateur** (`alacod-gen`) : les scénarios générés d'un jeu dont le
joueur vise à quatre directions épinglent `aim: Some(Free)` sur les joueurs scriptés qui tirent : la cible de l'arène
(+128, −48 : `AIM_AT_TARGET`, carte `gabarit_armes.ldtk`) est en biais et ne peut pas être touchée à quatre directions ; ces
scénarios prouvent un contenu (arme, larme, familier, statut), pas la visée. Ils sont **bit-identiques** à avant (le
composant n'existe pas), leurs `.ron` gagnent la ligne `aim: Free`. Épinglés pour la même raison (`explorateur` : la
grammaire de la grille est leur objet, pas la visée) : `grid_floor_seed_1`, `grid_floor_seed_23`, `grid_floor_seed_123`,
`grid_post_boss`, `isaac_floor_seed_1` (traces inchangées).

**Traces bougées, avec preuve §5** (dumps `ALACOD_DUMP_TRACE`, `main` `2d30a1c1` contre la branche, `scripts/trace-diff.py`) :
`generated/isaac/boss_gardien` et `generated/isaac/room_combat` (bots `fonceur` et `explorateur` d'`isaac`, qui tirent à
quatre directions). Première frame qui diffère : **5**, la première où le bot a un ennemi en vue : le `pan` écrit par le bot
est quantifié, donc le `CursorPosition` dérivé change et, avec lui, la rotation du sprite d'arme (`FixedTransform3D` de
`larmes` : `x_axis (-0,986, -0,167)` devient `(-0,999, 0,039)` pour `boss_gardien` ; `FacingDirection` `DownLeft` devient
`Down` pour `room_combat`). Les attentes tiennent (le boss de `boss_gardien` tombe à f499, `room_combat` nettoyée). Aucun autre
scénario d'`isaac` ne bouge.

### Atteignabilité

Rencontré quand M3-T11a met les objets de larmes dans le pool trésor d'`isaac` ; d'ici là, scénarios
testbed seulement.

## 67. Effets v3 : `OnHit`, `OnEvent`/`Emit`, `ApplyStatus`, `PerCopy` (M3-T2, chantier C1 v3)

Exécute les contrats de §61. **Aucune trace existante ne bouge** : variantes en fin d'enum, champs
optionnels, files transitoires (hors checksum, vidées au `FrameStart`).

- **`OnHit`** : le porteur a infligé un dégât **réel** (> 0, après immunité, bouclier, armure) à une autre
  entité. `character::health::rollback_resolve_damage_events` émet `effects_v3::HitLanded`, lu par
  `apply_effects_system`. Jamais pour un dégât que la source s'inflige. La condition `TargetInRange` lit
  la position de la cible (`OnHit`, `OnKill`, `OnDamageTaken`) ; fausse sans cible.
- **`ApplyStatus { status, stacks }`** dans un effet : statut posé sur la **cible** du déclencheur, donc
  `OnHit` ou `OnDamageTaken` (la source du coup) seulement. **Pas `OnKill`** : la cible est morte dans la
  frame, le statut ne serait jamais posé (lint : refusé) ; pour un effet à la mort, `ApplyStatusSelf` pose
  sur le porteur. Posé par `effects_v3::effect_status_system` avec la règle des `on_hit` de projectile
  (auteur crédité des dégâts, `Slow` pose son modificateur de vitesse). Statut inconnu et `stacks = 0` : lint.
- **`OnPickup` filtré** : champ `item: Some("bottes")` de l'**effet** (pas une variante `OnPickup(Some(item))` :
  l'enum `On` et le hash des effets existants ne bougent pas). Sans filtre, tout ramassage ; avec, le seul
  objet nommé (le sien compris). Lint : `item` hors `OnPickup` et objet inconnu refusés. Hors hash quand absent.
- **`Emit(nom)` / `OnEvent(nom)`** : les effets `OnEvent` du même porteur répondent aux `Emit` de ses
  effets, dans la même frame, par tours (`MAX_EVENT_ROUNDS` = 8, au-delà les émissions sont abandonnées
  avec un `warn!`). Le lint refuse un nom vide ; il ne vérifie pas qu'un `OnEvent` a un émetteur.
- **`Scaled { PerCopy }`** : l'action enveloppée n'est exécutée **qu'une fois par objet** (par le premier
  exemplaire), multipliée par le nombre d'exemplaires (`Inventory::count`). `scale: None` est transparent.
  **Actions linéaires seulement** (`effects::actions::is_linear`) : `Heal`, `GainShield`,
  `Modifier`/`TimedModifier` d'op `Add` ou `Pct`, `ApplyStatus`, `ApplyStatusSelf`, `Explode` (dégâts, pas le
  rayon). Le lint refuse toute autre action sous `PerCopy` (`Modifier` `Mul`/`Set`, `Emit`, `RefillAmmo`…) :
  l'exécution ne la sauterait pas sans rien dire.
- **Composition `Emit` × `OnEvent` par exemplaire : linéaire.** Une action non `Scaled` s'exécute **par
  exemplaire** (n exemplaires = n `Emit`), mais un effet `OnEvent` **écoute une fois par objet** (le récepteur
  n'est pas dupliqué ; `listed_effects` ne le liste que pour le premier exemplaire). Exemple, la `vipere`
  (`OnHit : Emit("coup")`, `OnEvent("coup") : +1 de dégâts`) : 1 exemplaire = 1 émission × 1 récepteur = +1 ;
  2 exemplaires = 2 × 1 = **+2** (et non +4) ; seul `Scaled(PerCopy)` multiplie.
- **Limites par exemplaire (Q16)** : `ItemDef::stack` (`Shared`, défaut, ou `Independent`). `Shared` :
  clé `item:<id>:<index>` commune, les `cooldown`/`charges` sont partagés ; `Independent` : clé
  `item:<id>#k:<index>` par exemplaire.
- **Ordre d'exécution** : porteurs par `GgrsNetId` ; pour chacun, effets du personnage, puis objets (passifs
  dans l'ordre de ramassage, puis l'actif), exemplaire après exemplaire, puis synergies. Les `Emit` sont
  traités après (`apply_event_effects_system`), puis les statuts (`effect_status_system`).
- **Scénarios** (chacun avec son témoin `_temoin`, sauf `effect_on_hit`/`effect_per_copy` dont le témoin est
  la frame d'avant le coup) : `effect_on_hit` (`vipere`), `effect_on_event` (`echo`), `effect_target_in_range`
  (`portee`), `effect_apply_status` (`morsure`), `effect_pickup_filtered` (`filtre`), `effect_per_copy`,
  `effect_scale_per_copy` (`empile`). Unitaires : `effects_v3::tests`, `effects_runtime::tests`,
  `effects::runtime::tests`, `crates/scenario/tests/effects_v3.rs` (`HitLanded` dans le vrai résolveur).
- **Échelle** : `effects_hundred_items` (100 objets : 97 `cent_*` de charge et 3 observateurs à une charge, un par
  famille `Tick`, `OnHit`, `OnKill`, dont la conséquence visible est comparée au témoin ; `machine_gun` sur un
  `dummy` puis une `target`, le `follower` de la carte meurt aussi) et son témoin sans objets. `advance_ms_avg`, trois exécutions du même run, machine
  partagée (indicatif) : **0,19 ms** avec les 100 objets, **0,12 ms** sans, 0,145 ms pour `four_players_idle`
  (115 entités). Surcoût ≈ 0,07 ms par frame sur 16,7 : `listed_effects` n'est pas précalculé.
- Hors périmètre : exécution des effets d'objet par les pools Isaac (T11b), transformations (Q10, non retenues
  ici : `transformation_three` non livré).

## 68. Cœurs, i-frames, sous, bombes et clés au sol (M3-T3, chantiers C4 et C5)

Exécute les contrats de §61 (Q2, Q4, `max_stack`, `hit_iframes`). Code : `combat::{actors::ExtraHealth,
damage::apply_layers}`, `character::health::rollback_resolve_damage_events`, `effects::Action::GainSoul`,
`items::{ItemDef::accepts, LootPick::Currency}`, `game::{items, shop, loot, champions}`,
`map_ldtk::game::rooms::rock_loot_system`. **Aucune trace existante ne bouge** hors `isaac` (voir « Traces ») :
tout type nouveau est neutre ou absent des jeux existants, les variantes sont ajoutées en fin d'enum.

### Couche de vie `ExtraHealth` (Q2 de William)

Un demi-cœur vaut **1 PV** (`Health` en points, les PV d'`isaac` restent pairs). `combat::actors::ExtraHealth
{ soul: Fixed }` est un composant rollback **neutre** (`rollback_and_trace_neutral`), posé au premier gain
(`combat::actors::queue_gain_soul`, comme `Shield`) : sans porteur il ne contribue pas au checksum. Il n'entre
pas dans `Health.max`, ne se régénère pas, n'est borné par rien (le plafond de 12 cœurs d'Isaac est un
réglage de T16) et reste posé à `soul == 0`.

**Ordre retenu des couches de défense**, en un seul endroit pur et testé, `combat::damage::apply_layers`
(appelé par `rollback_resolve_damage_events`, après `resolve_damage`) :

1. **bouclier** (`Shield`) : un point absorbe le coup **entier** (brèves i-frames `SHIELD_IFRAMES`) ; ni
   l'armure ni l'âme ne sont touchées ;
2. **armure** (`stat Armor`, `after_armor`) : réduction proportionnelle ;
3. **`ExtraHealth`** : absorbe ce qui reste **point par point** ;
4. **PV** : ce qui reste après l'âme, via `DamageAccumulator` (donc mort, à terre et `last_hit_by` comme avant).

`DamageKind::True` ignore le bouclier et l'armure (des atténuations) mais **pas** `ExtraHealth` (c'est de la
vie : une chute ou des piques entament d'abord les cœurs d'âme, comme dans Isaac). Un coup que l'âme absorbe
en entier **est un coup** : `hit_iframes` démarrent, `HitLanded` (donc `OnHit`) et `HitCount` le comptent, le
crédit de points aussi ; en revanche rien n'est accumulé (`DamageAccumulator` intact : ni mort, ni `last_hit_by`,
ni délai de régénération). **`OnDamageTaken` dépend du porteur** (`effects_runtime`, §25) : avec `HealthRegen`
il se lit sur la perte de PV réellement appliquée, donc un coup absorbé en entier par l'âme ne le déclenche pas ;
**sans `HealthRegen` — le cas de `pleureur`, le joueur d'`isaac` — il se déclenche sur tout `DamageEvent` qui vise
le porteur**, donc aussi sur un coup que l'âme absorbe en entier (et sur un coup ignoré par les i-frames). **La
règle du §25 n'est pas modifiée** par cette tâche : c'est la documentation qui est corrigée. Un objet d'`isaac`
qui doit réagir aux seules pertes de PV devra attendre une règle unique (hors fiche).

Sans bouclier, avec une armure ≤ 0 et sans âme, le montant reste exactement celui de `resolve_damage` (identité,
aucune trace ne bouge). Traces de jeu : `ggrs{f=… soul_absorb target=… took=… left=…}` (et `shield_absorb` comme avant).

**Consommable « cœur d'âme »** : action `effects::Action::GainSoul(Fixed)` (variante **en fin d'enum**, linéaire
donc acceptée sous `Scaled(PerCopy)` dans un effet, décrite par `describe_action`), exécutée dans un effet de
personnage ou d'objet (`effects_runtime`) et à `OnUse` d'un consommable (`items::apply_instant_effects`) ;
refusée dans un power-up et dans les actions où `GainShield` l'est ; lint `GainSoul = 0` (`OutOfRange`).
**`Scaled` est refusé à `OnUse`** (consommable et actif, lint `Unsupported`, fixture `scaled_in_onuse`) :
`apply_instant_effects` n'exécute pas l'enveloppe, donc `OnUse: [Scaled(GainSoul(..))]`, `Scaled(Heal(..))` ou
`Scaled(GainShield(..))` aurait été accepté puis sans effet (T0b avait fait déballer l'enveloppe par le lint ; aucun
contenu ne l'utilisait). Le sens de `PerCopy` pour un consommable consommé au ramassage reste à décider si un
besoin apparaît. Un consommable à
effets s'applique au ramassage et n'entre pas dans l'inventaire (§44). `testbed` : `coeur_ame` (4 points) ;
`isaac` : `coeur_ame` (2 points = un cœur entier).

**Attente de scénario** `ExtraHealth(handle, n, at_frame)` : points de vie d'âme du joueur (0 sans le composant),
égalité exacte. **T8 (Q6)** : `ExtraHealth::soul` est un champ public de `combat` ; `Price::MaxHealth` reçoit déjà
`health` et `max_health` en `Fixed` et T8 y ajoutera `soul` à l'appel de `can_pay`/`pay` (« les cœurs d'âme
paient à la place s'ils sont présents ») : rien n'est décidé ici sur la règle de paiement.

### Plafonds et refus au ramassage

`ItemDef::accepts(stock, at_full_health)` (pure) : faux pour un consommable dont le compteur a atteint
`max_stack`, et, quand l'objet déclare **`refuse_when_full: true`**, à pleine vie (`Health.current >= max`).
`refuse_when_full` (champ de fin d'`ItemDef`, hors hash, faux par défaut) est réservé aux consommables qui
soignent (lint : exige un effet `Heal`) ; **un cœur reste donc au sol seulement pour les consommables qui le
déclarent** (`coeur`) : la `ration` garde son comportement (ramassée sans effet à pleine vie).
`item_consumable_pickup_system` prend le **premier joueur à portée (ordre `GgrsNetId`) qui peut le prendre** :
un joueur au plafond ou à pleine vie laisse le tas à un autre joueur à portée, sinon l'objet reste au sol et
sera ramassé plus tard. Sans `max_stack` ni `refuse_when_full`, `accepts` est toujours vrai : le premier joueur
à portée, comme avant. **Boutique** : un présentoir de consommable que l'acheteur ne peut pas prendre ne vend
pas : rien n'est dépensé, `CurrencyEvent { delta: 0, reason: "shop_full" }` et anti-rebond `retry_after`
(`ggrs{shop_full}`). Lint : `max_stack` déjà vérifié (T0b) ; `starting_consumables` qui dépasse `max_stack`
(`OutOfRange`), `refuse_when_full` sans `Heal` (`Unsupported`). `isaac` : `key` et `bomb` plafonnés à 99 ;
`testbed` : `bomb` à 3.

### Sous au sol (`LootPick::Currency`)

Entrée de pool `pick: Currency(n)` (variante **en fin** de `LootPick`, `n > 0` au lint). Le tirage la désigne par
`currency:<n>` (`items::loot_currency`, comme `weapon:<id>`) ; elle compte comme un **consommable `Common`** : elle
sort d'une récompense de salle ou d'un rocher (`consumables_only`), pas d'un coffre de rareté minimale
supérieure. Les consommateurs du tirage posent un **`ShellPickup`** (la même entité que les douilles des
ennemis, M2-T6 §50 : `game::shop::spawn_shell_pickup`, ramassé au contact par `shell_pickup_system`, crédité à
`Currency` par `CurrencyEvent { reason: "pickup" }`) : coffre (`chest_interaction_system`), récompense de salle
(`room_reward_system`), butin de champion (`champions::drop_loot`), butin de rocher (`rock_loot_system`). Un pool
de **boutique** ne peut pas en porter (lint `Unsupported`, y compris les entrées ajoutées par un déblocage) :
un présentoir ne vend pas de sous. La mort d'un ennemi laisse déjà des sous par `economy.ron`
`shells_drop` (§50, `isaac` : 2 à 5) ; un champion peut en lâcher par son pool. `isaac` : `loot/salle.ron`
(récompense de salle) gagne `Currency(1)`, `coeur`, `coeur_ame` et `bomb` aux poids 4, 3, 1 et 2.

**Épuisement (M3-T0e, §64)** : `Currency` est un `pick` ordinaire pour `draw_exhausting` et `draw_loot_with` (candidat `currency:<n>`, filtres de rareté et de consommable inchangés) mais n'est **jamais « vu »** : `PoolState::record` ignore un `currency:<n>` (renvoie `false`, aucun état). Un tas de sous n'épuise donc pas un pool `exhaust: true` et reste tirable à chaque emplacement ; la variante reste en fin de `LootPick` (T0e n'a ajouté aucune variante).

### Partage en coop (Q4 de William) : par joueur

Sous, bombes et clés sont **par joueur** (l'`Inventory` et le `Currency` de chacun, comme avant) : celui qui
ramasse possède. Rien de nouveau à coder, sinon l'ordre de ramassage déjà déterministe (plus petit `GgrsNetId`
à portée qui peut prendre l'objet ; coffre : premier événement d'interaction). Prompts et HUD lisent le joueur
concerné (prompt du coffre : les clés de l'interacteur) ; le HUD gagne la source **`bombs`** (compteur `bomb` du
joueur affiché, à côté de `keys` et `shells`, liste fermée `content::ui::HUD_SOURCES`) ; `isaac/ui/hud.ron`
l'affiche. Aucune ressource d'équipe (l'option « équipe » de §61 n'est pas retenue).

### I-frames de `isaac`

`pleureur` (le personnage joueur) pose `hit_iframes: Some(60)` : une seconde d'invulnérabilité après avoir
perdu de la vie (résolveur de T0b, §61). Deux ennemis au contact qui frappent à quelques frames d'écart ne
retirent plus qu'un coup (`hit_iframes_isaac` : 4 PV à f130 et 3 PV à f260, contre 3 et 1 sans la donnée —
témoin joué par `crates/scenario/tests/extra_health.rs`, `isaac_without_hit_iframes_loses_more`).

### Traces

Fusion avec `origin/main` `64ec961c` (M3-T5 : `isaac` en grille) : **onze traces `isaac` bougent** (plus `hit_iframes_isaac`,
neuve) ; aucune autre. Preuve §5 : dumps détaillés (`ALACOD_DUMP_TRACE`) des onze scénarios sur trois états, comparés par
`scripts/trace-diff.py` sans `--ignore` : (1) `main` ; (2) la branche **sans** `hit_iframes` de `pleureur` (reste : le pool
`loot/salle.ron` d'`isaac` et le code) ; (3) la branche complète. Deux causes distinctes, jamais confondues :

- **`hit_iframes`** (2 → 3) : première frame qui diffère = frame du premier coup reçu par le joueur, **une seule ligne** (le
  joueur), **un seul champ** : `Health.invulnerable_until_frame` `None` → `Some(f+60)` ;
- **pool `salle`** (1 → 2) : première frame qui diffère = frame où la récompense de salle est tirée, **une seule ligne** :
  l'objet au sol change (`item_key`/`item_blank`/`item_ration` → `item_bomb`/`item_ration`, même position).

| Trace | 1 → 3 : première frame et cause | autre cause, plus tard |
|---|---|---|
| `grid_floor_seed_1` | f250, `hit_iframes` (→ 310) | pool `salle` f529 (clé → bombe, (248, −8)) |
| `grid_floor_seed_123` | f365, `hit_iframes` (→ 425) | aucune (pool seul : identique à `main`) |
| `grid_floor_seed_23` | f254, `hit_iframes` (→ 314) | pool `salle` f357 (blank → ration, (248, 136)) |
| `grid_post_boss` | f352, **pool `salle`** (clé → bombe, (−232, 136)) | `hit_iframes` f845 (→ 905) |
| `isaac_floor_seed_1` | f250, `hit_iframes` (→ 310) | pool `salle` f529 (clé → bombe) |
| `generated/isaac/boss_gardien` | f160, `hit_iframes` (→ 220) | pool `salle` f163 (ration → bombe, (−232, −8)) |
| `generated/isaac/enemy_mouche_moving` | f73, `hit_iframes` (→ 133) | aucune |
| `generated/isaac/enemy_mouche_still` | f202, `hit_iframes` (→ 262) | aucune |
| `generated/isaac/enemy_suiveur_moving` | f77, `hit_iframes` (→ 137) | aucune |
| `generated/isaac/enemy_suiveur_still` | f264, `hit_iframes` (→ 324) | aucune |
| `generated/isaac/room_combat` | f250, `hit_iframes` (→ 310) | aucune (pool seul : identique) |

Le pool `salle` **fait donc bouger des `grid_*`** (`grid_floor_seed_1`, `grid_floor_seed_23`, `grid_post_boss`) et
`isaac_floor_seed_1`, `boss_gardien` : ces traces sont bénies pour les deux causes. `hit_iframes_isaac` n'existe pas
sur `main` : sa trace est neuve (la carte `map_seed: 1` est maintenant la grille). Ses ennemis, posés à (20, −8) et
(−36, −8), restent à 28 px du joueur (départ (−8, −8), ancre de l'étage inchangée) : les attentes (6, 5, 4, 4, 3 PV)
passent sans changement, rien à recaler. Toutes les autres traces (`grid_secret_*`, `grid_treasure_*`, `isaac_boots`,
générées `weapon_*`, testbed, zombies, gungeon, throne, autres jeux) sont identiques.

### Scénarios (testbed sauf mention)

| Scénario | Prouve |
|---|---|
| `soul_heart_absorbs` / `_temoin` | la `tourelle_spirale` retire 8 PV par volée ; avec un `coeur_ame` (4) : ExtraHealth 4 puis 0, santé 100 → 96 → 88 ; sans : 92 → 84 |
| `heart_full_stays` / `_temoin` | à pleine vie le `coeur` (`refuse_when_full`) reste au sol et soigne dès le premier coup (96) ; la `ration` ramassée à f5 est perdue (92) |
| `coin_pickup` | un coffre du pool `pieces` (`Currency(7)`) laisse un tas de 7 sous, ramassé : 500 → 507 |
| `room_reward_coins` | l'étage `sous_salle` (pool `pieces_salle`, `Currency(3)`) : la salle `combat` nettoyée laisse un `ShellPickup` de 3 sous (f480), ramassé aussitôt (1120 → 1123), aucune clé ; chemin `room_reward_system` |
| `champion_drops_coins` | `elite_sous` (variante `sous`, pool `pieces`) : à sa mort (f54) un tas de 7 sous (`champions::drop_loot`), ramassé (580 → 587) ; chemin champion |
| `shop_full` / `_temoin` | un présentoir de `bomb` (5 sous) : avec 3 bombes (plafond) l'achat est refusé sans dépense (500, 3 bombes, présentoir invendu) ; avec 2 bombes il passe (495, 3 bombes, vendu) |
| `bomb_pickup_cap` | au plafond (3) la bombe du sol est refusée (3 à f29) ; après la pose d'une bombe (explosion), elle est ramassée (3 à f40) |
| `coop_pickups_per_player` | deux joueurs : une clé chacun, la bombe du milieu va à P0 (plafond 3), la seconde est refusée par P0 et prise par P1 ; 7 sous pour P1 seul ; HUD sur P0 |
| `hit_iframes_isaac` (`isaac`) | voir ci-dessus |

Les sous du **rocher à butin** passent par le même `items::loot_currency` + `shop::spawn_shell_pickup` que la
récompense de salle et le champion, mais **aucun scénario ne les observe** : un étage dont le pool est `pieces_salle`
sur la carte `rocks_static` ne laisse pas le joueur bouger (non élucidé), le scénario a été retiré.

Unitaires : `combat::damage::layer_tests` (ordre bouclier → armure → âme → PV, `True`, coup absorbé en entier),
`items::tests::accepts_refuses_a_full_counter_and_a_wasted_heal`, `items::loot::tests`
(`currency_pick_is_a_common_consumable…`), `crates/scenario/tests/extra_health.rs` (les mêmes règles dans le vrai
résolveur, dont `a_hit_absorbed_by_the_shield_does_not_start_the_hit_iframes` : bouclier d'un point, coups à f12 et
f30, attendu `(4992, 0, 0)` ; 5000 si le coup absorbé posait `hit_iframes`), `crates/content/tests/m3_t3_fixtures.rs`
(8 fixtures : `item_refuse_when_full_without_heal`, `item_gain_soul_zero`, `loot_currency_zero`, `shop_currency_pool`,
`character_starting_over_max_stack`, `unlock_currency_zero`, `unlock_currency_shop_pool`, `scaled_in_onuse`).

### Atteignabilité et dettes

Récompense de salle `combat` d'`isaac` (cœurs, sous, bombes, clés) ; mort d'ennemi d'`isaac` (`shells_drop`) ;
tables de champions et d'ennemis (M3-T11c). `OnDamageTaken` et cœurs d'âme : voir « Ordre retenu » ci-dessus
(sans `HealthRegen`, un coup absorbé par l'âme le déclenche) ; `ExtraHealth` n'a pas de plafond ; la pose de
bombe d'`isaac` (`BombCarrier`) est M3-T11 ; HUD des cœurs : T16.

## 69. Bombes, explosions, rochers cassés, piques, mur secret (M3-T4, chantiers E3 et E4 v2)

Exécute les contrats de §62. Code : `game::bombs` (`BombsPlugin`), `world::{bomb, explosion, surface}`,
`map_ldtk::game::rooms::secret_wall_system`, `game::effects_runtime` (action `Explode`). **Aucune trace
existante ne bouge** : tout état nouveau est neutre ou transitoire et absent des jeux sans `bomb`, sans
rocher, sans piques ni salle secrète ; le baril garde sa trace (`barrel_chain`, `bench_floor`) en changeant
de code.

### Bombe

`CharacterConfig.bomb` donne au joueur un composant `world::BombCarrier { def, held }` (rollback **neutre**,
`PlayerScript::bomb` le pose aussi en scénario). `bomb_place_system` (`Effects`, après
`synergy_modifiers_system`, joueurs par `GgrsNetId`) : au **front montant** de `INPUT_BOMB` (`held` réécrit à
chaque frame, avant tout retour anticipé), joueur debout, avec au moins un consommable `bomb` dans son
`Inventory` (`starting_consumables` ou `PlayerScript::items`), il en consomme un et pose une `world::Bomb`
aux pieds (centre du collider, offset compris) : entité rollback neutre numérotée `bomb_<poseur>`,
mèche de `fuse_frames`. `BombDef.friendly_fire` (fin de structure, défaut **`Always`**, Q4 : la bombe blesse
son poseur et les autres joueurs, comme Isaac ; `Never` pour la couper). Sans `BombCarrier` rien ne lit le bit.

### Explosion

Trois sources, une règle. `world::ExplosionRequest { id, source, source_team, x, y, radius, damage,
destroys, friendly_fire }` (`FrameEvents` **transitoire**, hors checksum : un baril en émet dans des parties
existantes) est émise par `bomb_fuse_system` (bombe dont la mèche est écoulée, puis `despawn_rollback`), par
le baril mort (lu dans `explosion_system`) et par l'action `Explode` (voir plus bas). Deux étages :

1. `World` (`bomb_fuse_system` puis `explosion_terrain_system`, après `pit_system`, **avant**
   `apply_destroy_terrain_system`) : si `destroys`, terrain de caverne (`DestroyTerrainRequest` : les murs
   de `CellGrid` et le champ de flux sont refaits dans la frame, comme pour `DestroyTerrain`) et rochers.
2. `CollisionDamage` (`explosion_system`, à l'emplacement de l'ancien `barrel_explosion_system`, avant
   `rollback_resolve_damage_events`) : les demandes de la frame (bombes + barils morts) sont **triées par
   `GgrsNetId` de l'objet** (tri stable) ; chacune envoie `damage` (`Physical`, tag `explosion`, source =
   le poseur, donc crédité) à tout personnage à PV dont le centre est à **strictement** moins de `radius`
   (`world::within_radius`), sauf l'objet lui-même, un personnage déjà à l'agonie (`Death`) et un personnage
   **invulnérable** (`Health::is_invulnerable_at`, comme les balles : c'est ce qui protège les familiers de
   M3-T6, `invulnerable_until_frame = u32::MAX`). Elle émet `world::Exploded` (transitoire, moment clé
   `explosion`) ; un baril est détruit par `despawn_rollback()` avec sa demande. Une chaîne de barils
   reste « un baril par frame » (le voisin touché meurt dans la frame, explose à la suivante).

Écart avec le baril d'avant : le rayon est **strict** (`<`) et non inclusif, et un personnage invulnérable
n'est plus visé (le résolveur l'ignorait déjà) ; aucune des scènes existantes ne place un personnage à
exactement `radius` (traces de `barrel_chain` et des cartes à barils identiques).

### Rochers et butin

`explosion_terrain_system` casse chaque `world::Rock` dont le centre est à moins de `radius`
(`world::cells_in_radius` : rendus par `(y, x)` croissants, un rocher cassé une seule fois par frame) par
`despawn_rollback()`. Le champ de flux relit les `Obstacle` à chaque mise à jour : le passage s'ouvre sans code
de plus (`rock_flow_field`). Chaque rocher cassé émet un `game::bombs::RockBroken` (transitoire) ;
`map_ldtk::game::rooms::rock_loot_system` (`World`, après `explosion_terrain_system`, qui seul voit `CurrentRooms`)
tire le butin d'un rocher à butin (`loot`) : un `ItemPickup` tiré par
`items::draw_loot_with` (consommables seulement) avec un `RollbackRng` **local** de graine
`loot_seed(graine, étage, "rock:<salle>:<x>:<y>")` (`game::bombs::rock_loot_key`, case de grille monde ;
`<salle>` = identifiant de la plus petite salle **de l'étage courant** dont les bornes contiennent le rocher, `-`
sinon : `game::bombs::room_of`, nourri par `CurrentRooms` ; en `Floors` les mondes se superposent, une salle d'un
étage inactif donnerait une clé fausse) ; pool =
celui de l'étage s'il existe, sinon `salle` (aucune chance : le tirage est certain). Q : pool propre aux
rochers ?

### Mur secret

Une salle dont le type porte `door_lock: Secret` garde ses portes **et leurs jumelles** (celles du bord de la
salle voisine) fermées : au premier tick, `room_lock_system` les laisse closes et leur retire seulement
`Interactable` (aucune Interaction ne les ouvre) ; les autres portes de salle s'ouvrent comme avant. Les deux
branches `Locked → Cleared` (fin d'une salle de combat ou d'une arène **voisine**) **et le verrouillage** d'une salle
voisine (`to_close`) ignorent aussi les portes secrètes (`secret_door`), ouvertes ou fermées : leur état est du seul
ressort de `secret_wall_system` (`secret_wall_open_then_fight` : mur ouvert, puis la salle voisine se verrouille et se
nettoie, il reste ouvert ; témoin `_sans_bombe`) : nettoyer la salle d'à côté n'ouvre pas le mur (`secret_wall_after_clear`). Le mode
`Floors` remet `DoorLocks` à zéro à la transition d'étage (`floor_transition_system`) : les portes renaissent
fermées, et un étage répété (mêmes `LevelId`) ne les croit pas déjà ouvertes (`secret_wall_floors`).
`secret_wall_system` (`World`, après `bomb_fuse_system`) : une `ExplosionRequest` qui `destroys` et dont le
rayon touche le **bord du collider** d'une de ces portes (`world::rect_within_radius`, strict) ouvre
**toutes** les portes de la salle (colliders retirés) et la range dans `DoorLocks` (`DoorLocked(kind, nth,
locked: false)`). Le champ de flux relit les portes à chaque mise à jour : les deux salles sont reliées
(`secret_wall_bomb`, un ennemi de la salle secrète rejoint le joueur). Placement par la grammaire : M3-T5 (§70).

### Piques

`hazard_system` (`World`, après `pit_system`, seulement si la carte a une case de piques) : un joueur debout
dont les pieds sont sur une surface à `hazard` reçoit `damage` (`True`, tag `spikes`, source = lui-même) si
`HazardDef::applies` : une fois toutes les `period` frames (`world::HazardState { last_hit }`, rollback
neutre posé au premier coup), sauf pendant un dash si `dash_crosses` (champ de fin de structure, faux par
défaut : le dash ne franchit pas les piques, il y est blessé quand il n'a pas d'i-frames). Surface du
testbed : `surfaces/piques.ron` (valeur IntGrid 5, 5 PV, 30 frames).

### Action `Explode`

`effects::Action::Explode { radius, damage, destroys }` est exécutée dans un effet de personnage, d'objet
tenu, de mutation ou une frise de boss (`apply_action_sourced`) : une **bombe à mèche d'une frame** est
posée au point du porteur (commande différée : `GgrsNetIdFactory` et position se lisent à l'application),
équipe du porteur, tir ami `Always` ; elle explose à la frame suivante par le même chemin. Le porteur n'est
pas exempté : au centre du rayon, il prend les dégâts comme les autres (`explode_effect`). Le lint ne la
refuse plus dans un effet (rayon > 0, dégâts >= 0 inchangés) ; les power-ups la refusent toujours.
**`OnUse` d'un objet (D80, fermée)** : `items::apply_instant_effects` n'a plus sa propre liste d'actions
(`Heal`, `GainShield`, `TimedModifier`, `Modifier`) : chaque action passe par `apply_action_sourced`, la même
exécution que les effets de personnage et d'objet tenu (source `item:<id>`, comme avant ; `Heal` sans `Health` :
sans effet). `Explode` est donc acceptée en `OnUse` d'un actif ou d'un consommable (lint : rayon > 0, dégâts >= 0) :
bombe à mèche d'une frame au point du porteur **lu à l'application** (commande différée), créditée à lui, de son
équipe. Un actif « bombe » : `testbed/objets/bombe_active.ron`, scénarios `item_active_bomb_rock` (les trois
rochers du couloir cassés à la frame suivant l'usage) et `item_active_bomb_enemy` (un `dummy` à 22 px blessé, le
porteur aussi : tir ami `Always`, un `dummy` hors rayon intact) ; fixtures `explode_onuse_ok`,
`explode_onuse_radius_zero`. Les actions que le lint refuse à `OnUse` (`GaugeAdd`, `SpawnPattern`, `Scaled`…) ne
sont toujours pas admises : l'exécution commune les refuserait par un `warn!` (action non exécutée), pas en
silence. Les traces existantes ne bougent pas (72 scénarios d'objets, boutiques, bombes et boss rejoués). `Explode`
en mort de champion : M3-T7.

**Tir ami (`friendly_fire`, D113)** : `Explode(radius, damage, destroys, friendly_fire: Never | Always)`, champ optionnel
(`ExplodeFriendlyFire`, `effects::actions`), **défaut `Always` = comportement d'avant** : l'explosion blesse tout personnage
à PV du rayon, porteur et alliés compris. `Never` pose la `Bomb` avec `FriendlyFire::Never` : les `DamageEvent` de
l'explosion sont refusés par `resolve_damage` pour tout personnage du **même bord** que le porteur (`Players`/`Allies`
ensemble, ou deux `Enemies`) : le porteur et ses coéquipiers sont épargnés, un adversaire prend les dégâts normalement.
Même champ pour l'`Explode` d'une mort de champion (`death_plan`). **Hash manuel** (règle « champ nouveau sous
checksum ») : `Always` n'écrit rien (les hashs d'avant ne bougent pas), `Never` écrit un octet ; `describe` ajoute
« épargne les alliés » ; le lint n'a rien à refuser (deux valeurs, toutes valides). Le champ passe par `scaled_action`.
`nova`, `bombe_fiole` et `epines_de_chair` sont à `Never` et n'ont plus de `GainShield` de contournement ; preuve coop :
`isaac_nova_coop` (deux joueurs à 6 PV à portée de `nova`, tous deux à 6 PV après l'onde ; avec l'ancien contenu, le
coéquipier mourait). Traces bougées (preuve §5, dumps `ALACOD_DUMP_TRACE`, `scripts/trace-diff.py`, objets d'avant contre
objets de la branche, même moteur) : `isaac_nova` et `isaac_bombe_fiole` f35, `generated/isaac/item_nova` et
`item_bombe_fiole` f15, `generated/isaac/tears_epines_de_chair` f2 : à chaque fois la première frame qui diffère est la
**pose de la bombe de l'objet** (`friendly_fire: Always` devenu `Never`, plus le `Shield` en moins pour les deux actifs) ;
`item_epines_de_chair` (porteur sans coup reçu) identique ; aucune autre trace ne bouge (suite complète rejouée).
Détail : `docs/taches/rapports/dette-d113-tir-ami-explode.md`.

### Balle explosive (`BulletType::Explosive`, D76 fermée)

`blast_radius` (et `explosive_damage_multiplier`) n'étaient lus par rien : une balle `Explosive` ne faisait que
disparaître. **Branchée sur l'explosion commune, sans second chemin** (choix : la variante existe déjà dans le schéma
RON des armes, la retirer aurait décalé le discriminant de `Piercing` dans le `derive(Hash)` de `BulletType`, donc
déplacé toutes les traces à balle perforante). Quand une balle `Explosive` se termine, `combat::weapons::spawn_blast`
pose **la même bombe à mèche d'une frame** que l'action `Explode` et les champions (`world::Bomb`, `fuse_until =
frame + 1`, numérotée `blast_<balle>`, créditée au tireur, de son équipe, politique de tir ami de l'arme) à la position
de la balle : rayon `blast_radius`, dégâts `Bullet.damage × explosive_damage_multiplier` (la cible touchée prend le coup
**puis** le souffle), tir ami `Cursed` **résolu au tir** (`Always` si le tireur porte `cursed`, `Never` sinon : le souffle n'émet que le tag `explosion`, test `souffle_cursed_suit_les_tags_du_tireur`), `destroys: false` (ni rochers ni terrain : `on_expire: [DestroyTerrain]` le demande
explicitement). Trois sites, tous dans l'ordre `GgrsNetId` du système : l'arrêt contre un personnage ou un mur d'une
balle ordinaire (`bullet_rollback_collision_system`), sa fin de portée (`bullet_rollback_system`), la fin d'un projectile
composable (`projectile_expire_system`, après ses `on_expire`, qui couvre sa touche, son mur et sa durée de vie). Une
balle `Piercing` ou `Standard` ne pose rien. Une balle explosive **annulée** (blank, effacement hostile, bouclier de familier) disparaît sans exploser : choix
assumé (relecture de D76), l'explosion est celle d'une balle qui se termine, pas d'une balle détruite. Les `GgrsNetId` ne sont alloués que quand une balle explosive se termine :
**aucune trace existante ne bouge** (aucun contenu d'un jeu ne déclarait `Explosive`, vérifié par `grep` sur `games/`,
`templates/`, `tests/` : seules les deux armes de testbed ajoutées ici). Testbed : armes `balle_explosive` (sans
`projectile:`, portée 100) et `balle_explosive_composee` (`Lifetime(20)`) (scénarios générés `weapon_balle_explosive` et `weapon_balle_explosive_composee`, `make gen GAME=testbed`), scénarios `explosive_bullet_hit` (le premier
`dummy` 200 → 185 : balle puis souffle ; le deuxième, derrière, 200 → 190 : souffle seul ; le troisième, hors rayon, et
le joueur intacts), `explosive_bullet_range` et `explosive_bullet_composed` (la balle s'éteint sans toucher : le `dummy`
voisin prend le souffle seul, 200 → 190). Pas de lint propre à `Explosive` (aucun lint de `bullet_type` n'existe).

### Déterminisme

`Fixed` partout (`FixedWide` pour les carrés), `order_iter!`/`order_mut_iter!`, `BTreeMap`/`BTreeSet`,
`despawn_rollback()` pour bombes, barils et rochers, un seul `RollbackRng` local par rocher à butin.
`ExplosionRequest` et `Exploded` sont transitoires (vidés au `FrameStart`, jamais lus avant leur émetteur) :
`explosion_system` est après `bomb_fuse_system` et les dégâts d'une frame se résolvent dans la frame. Types
neutres sans porteur hors des cartes concernées : `BombCarrier`, `Bomb`, `HazardState`, `Rock` (T0c),
`DoorLocks` (T0c). `PlayerScript::bomb` est un champ de scénario sans effet sur les parties enregistrées.
`fuzz_inputs` : `ALACOD_FUZZ_BOMBS=1` joue un personnage à huit bombes dans la salle à rochers du testbed
(`rock_flow.ldtk`) avec des appuis aléatoires sur `Bomb`.

### Testbed et scénarios

Cartes `testbed/spikes.ldtk`, `testbed/rock_flow.ldtk`, `testbed/salles_secret.ldtk`
(`scripts/construire-cartes-t4.py`) ; `rocks_static.ldtk` (T0c) sert aux bombes et au butin. Objet
`objets/bomb.ron` (consommable `bomb`), surface `surfaces/piques.ron`.

| Scénario | Prouve |
|---|---|
| `bomb_place_explode`, `bomb_hurts_owner` | pose au front montant, compteur `bomb` 1 → 0, explosion 20 frames après (f35, `explosion`) ; un mannequin à 35 px perd 50 PV, un autre à 100 px non ; tir ami coupé : le poseur intact, actif : il perd 50 |
| `bomb_breaks_rock` | les trois rochers du couloir disparaissent, le rocher à butin (50 px) reste, le joueur passe ; aucun butin |
| `bomb_digs_cave` | caverne `petite` : la bombe creuse (26, 15) et (27, 15) (7 cases détruites, f96) par la même requête que la grenade ; le joueur traverse |
| `rock_loot` | rayon 60 : le rocher à butin laisse une clé (`loot_seed`), ramassée (f156) |
| `bomb_chain_barrel` | bombe → premier baril (f80) → second (f81) → joueur 1 touché (100 → 70, f82) |
| `secret_wall_bomb`, `secret_wall_closed` | porte fermée (`DoorLocked`), joueur et ennemi bloqués ; la bombe l'ouvre (f95) : ils passent ; témoin sans bombe |
| `spikes_damage` | 5 PV à f71 puis toutes les 30 frames (95, 90, … 75 à f199), santé intacte avant |
| `secret_wall_after_clear` | la salle de combat voisine nettoyée (f489) rouvre ses portes ordinaires, pas la porte secrète : le joueur s'arrête contre elle (x -714,8), aucune bombe |
| `secret_wall_floors` | deux étages identiques (`deux_secrets`) : porte ouverte par une bombe à l'étage 0, de nouveau fermée à l'étage 1 (f306), rouverte par la seconde bombe (f395) |
| `explode_effect` | un effet `Explode` (`kamikaze`, `Tick(40)`, une charge) : une seule explosion (f47), joueur 100 → 70, porteur 50 → 20 (aucune exemption), `dummy` hors rayon intact |
| `rock_flow_field`, `rock_flow_field_blocked` | le `follower` prend le détour ; la bombe rouvre le passage direct : distance 59,7 à f240 contre 109,2 ; il rejoint le joueur vers f280 contre f360 |

Unitaires : `game::effects_runtime` (`Explode` pose une bombe à mèche d'une frame, équipe et crédit du porteur,
`destroys`), `game::bombs::room_of` (sélection de la salle parmi celles de l'étage courant), `world::explosion` (rayon strict, rectangle touché, cases triées), `world::surface`
(`HazardDef::applies`), `game::bombs` (`explosion_system` : ordre des ids, invulnérables et mourants
épargnés, ordre des explosions ; clé de butin), `content` (`explode_effect_ok`).

### Questions pour William

Q4 (reprise de §62) : tir ami de la bombe, proposition `Always` ; pool de butin propre aux rochers (le pool
`salle` ne donne qu'une clé) ; dégâts des piques `True` en points (demi-cœurs : Q2) ; mur secret ouvert par
une explosion de baril (aujourd'hui non : le baril n'a pas `destroys`).

### Atteignabilité

Rencontré quand `isaac` donne des bombes de départ à son personnage (dette : le faire change ses traces,
décision de l'orchestrateur : faite par M3-T11d1, §76.4), que M3-T11d1 a placé rochers et piques dans les gabarits `combat` ; M3-T5 a placé
le mur secret (§70 : une salle secrète par étage de `isaac`, graines 25 et 39 depuis M3-T11d1, 5 et 8 avant : `grid_secret_*`).

## 70. Étage en grille : planificateur, salle secrète, trésor à clé, salle post-boss (M3-T5, chantier E2 v2)

Exécute `FloorLayout::Grid` (contrat de §62) : un étage d'Isaac pose ses salles sur une grille de cases de
même taille, une porte par côté partagé relié, des boucles possibles, des culs-de-sac pour les salles
spéciales, une salle secrète voisine de plusieurs salles. Code : `map::generation::grid` (planificateur et
contrôle indépendant), `map::generation::floor` (dispatch, `FloorPlan`), `map::generation::imp::floor`,
`map_ldtk::generation::to` (portes de plus, murage exact), `map_ldtk::game::rooms` (portes à clé, minicarte),
`content::lint` (gabarits et clés). **`Graph` est inchangé** : mêmes plans, mêmes traces pour `gungeon`,
`zombies`, `throne` et `testbed` (preuve en fin de §).

### Contenu : la grammaire

```ron
// games/isaac/assets/floor_grammars/sous_sol.ron
(templates: "maps/salles.ldtk", grammar: (
    rooms: (8, 12),                                   // départ, boss, requises, secrète, diable compris
    required: {"boss": 1, "tresor": 1, "secrete": 1}, // en grille : exactement ce nombre
    min_boss_distance: 3,
    layout: Some(Grid(
        cell: (15, 9),                                // une case de salle, en cases de 16 px (Q8)
        max: (7, 7),                                  // étendue de la grille, en cases de salle
        loops: 1,                                     // boucles au plus (défaut 0 : un arbre)
        secret: "secrete",                            // type de la salle secrète (défaut)
        post_boss: Some((kind: "diable", chance: "0.33")), // salle post-boss, absente : aucune
    )),
))
```

Réponse de William à Q8 : **case unique et grandes salles 2×1, 1×2, 2×2 dès T5** (formes en L hors périmètre).
Un gabarit couvre un rectangle de cases de salle : sa taille en cases de 16 px est un **multiple de `cell`**, et
**chaque côté de chaque case de salle** a une ouverture `LevelConnection` **centrée sur ce segment** (un 2×1 :
deux ouvertures au nord, deux au sud, une à l'ouest et à l'est). Cette ouverture est un *port* : `(côté, case le
long du côté)`. Lecture de « ouverture centrée sur chaque côté » pour une grande salle : **par segment de case**,
seule façon d'aligner ses portes sur celles de voisines 1×1 ; à relire (Q8 ne tranchait pas ce point). Deux salles ne
se relient que par des ouvertures **de même largeur** (sinon leurs bords ne coïncident pas : le contrôle indépendant
refuserait la porte, `Layout::width` l'exclut dès la pose ; le lint exige une même largeur pour tous les gabarits d'une
grammaire en grille). **L'étendue `max` est bornée par le monde** : le départ est à la case `(max - taille) / 2` et le
monde fait ±1000 px (`max_width`/`max_heigth`), si bien que huit colonnes de 240 px ne tiennent pas (la huitième est
refusée par `Layout::free`) : `sous_sol_grand` est en 7×7, comme `sous_sol`. Un
gabarit n'a plus besoin d'être un cul-de-sac : le boss et le trésor ont leurs quatre ouvertures, le plan n'en
relie qu'une, le chargeur mure les autres. Les quatre `BossN/E/S/W` de T0f disparaissent.

### Algorithme (`grid::attempt`, une tentative = une graine)

1. **Départ** (gabarit `Spawn` du type de départ) au centre de la grille, aligné sur les tuiles comme en `Graph`.
2. **Remplissage** : croissance, une pose = tirage d'un gabarit parmi ceux qui ont au moins une pose valide (un 2×2
   n'est pas favorisé par ses huit ports), puis d'une pose de ce gabarit. Une pose est valide si le rectangle est
   dans la grille et dans le monde (`max_width`/`max_heigth`), sans salle, et **ne touche que son parent** (règle
   d'Isaac). Sauf à `loops` pas de croissance tirés d'un flux local, où la pose peut toucher **une seconde** salle
   ordinaire, reliée par une porte de plus (une boucle ; ports libres face à face exigés).
3. **Boss** sur une salle de profondeur maximale (distance en salles, boucles comprises) : le cul-de-sac le plus loin ;
   `min_boss_distance` ; strictement le plus loin de l'étage.
4. **Salle post-boss** : la chance (`post_boss.chance`) est tirée **une fois par étage** par un flux local de la
   graine de l'emplacement (avant les tentatives : toutes les tentatives voient la même décision) ; la place est
   réservée dans `rooms` ; la case est accrochée au boss (seul voisin) et n'est pas comptée comme « plus loin que le
   boss ». Le boss a donc un voisin de plus, elle.
5. **Salles requises** (hors boss et secrète) : culs-de-sac accrochés à une salle ordinaire (départ ou remplissage)
   de profondeur ≤ celle du boss − 2.
6. **Salle secrète** (`required` la demande) : un rectangle libre, sans voisin boss ni post-boss, qui touche **au
   moins deux salles ordinaires** par un port libre de part et d'autre ; une porte par voisine (la première dans
   l'ordre des ports). Les candidats sont énumérés dans l'ordre (`BTreeSet`), tirés comme les poses.
7. **Contrôle indépendant** `validate_grid_plan`, relu depuis les `Room` (positions, connexions) et non depuis l'état du
   planificateur : bornes, types **exacts**, un seul départ en tête, un seul boss, alignement sur la grille et
   tailles multiples, aucun chevauchement ni identifiant en double, **portes réciproques entre salles voisines aux
   ouvertures face à face** (aucune porte vers le vide), connexité hors salle secrète, arêtes ≤ arbre + `loops`, deux
   salles voisines (hors secrète) toujours reliées, boss en cul-de-sac strictement le plus loin (distances sans la
   secrète : un raccourci par elle ne compte pas) à la salle post-boss près, requises en culs-de-sac, secrète à ≥ 2
   portes toutes ordinaires et loin du boss. Un plan refusé relance une tentative.

**Déterminisme.** `BTreeMap`/`BTreeSet`, `Vec` parcourus dans l'ordre de pose, aucun `HashMap`. Le RNG d'une
tentative est découpé en **flux locaux** par usage (croissance, pas de boucle, secrète, départ, identifiants de salle
par indice) dérivés de la graine de la tentative (`mix`) : ajouter un tirage à l'un ne déplace pas les autres. Les
tirages bornés prennent les bits de poids fort du LCG (`below` : `RollbackRng::next_u32_range` prend un modulo, dont
les bits de poids faible alternent). Identifiants de salle `seed ^ (indice + 1) × 0x9E3779B9` : injectif, jamais deux
salles au même identifiant. Graine par emplacement : inchangée (§40).

**Terminaison (le risque de la fiche).** Aucune boucle `while`. La croissance est un `for` borné par le nombre de
salles ; une pose énumère un nombre fini de candidats (parents × ports × gabarits × ports) ; les pas de boucle sont
tirés sans remise ; les tentatives sont bornées par `max_attempts` (`FloorError::Exhausted`, `NoTemplate` si un type
n'a aucun gabarit à ports). `grid::step_bound(grammaire, gabarits)` majore les évaluations de candidats ;
`FloorPlan::steps` les compte, tentatives échouées comprises. Tests : 1 000 graines sous la borne (maximum observé
353 pour une borne de 77 millions, gabarits du test), grammaire impossible (grille d'une case, distance du boss
irréaliste, plus de requises que de salles) : échec explicite sous la borne.

### Plan : de l'arbre au graphe

`PlannedRoom.links: Vec<PlannedLink { mine, other, theirs }>` : les portes qui ne sont l'arête de pose (parent) d'aucune
salle (boucle, voisines supplémentaires de la salle secrète), portées par la salle posée en dernier des deux ; vide en
`Graph`. `IMapGeneration::get_links` et `IMapGenerator::add_link` les rendent après les salles (`map_generation`).
En `Basic` et `Graph`, `get_links` rend aussi les prises libres qui se touchent après la pose (fermeture des boucles,
§1, `MapSettings::close_loops`) ; une grille ne les ferme pas (`FloorMapGeneration` désactive `close_loops` et
`wall_unused_openings` : son plan fixe ses boucles). `Room::connections` finales portent déjà toutes les portes :
l'appariement des portes (`get_doors`) est inchangé.

**Murage exact.** En grille le chargeur appelle `close_unused_openings_exact` : chaque salle générée garde la liste
de ses ouvertures sans porte du plan (lue dans ses connexions finales) et les mure. `Basic` et `Graph` murent leurs
prises libres pendant `map_generation` (`IMapGeneration::unused_openings`, `IMapGenerator::close_opening`, §1) ;
l'ancien murage par direction (`close_unused_openings`) est retiré (fusion de `better-assets`, 2026-10-19).

### Salle secrète

Type `secrete` (`rooms/secrete.ron`, `door_lock: Secret`) : une par étage si `required` la demande, murée (§69 : une
explosion `destroys` ouvre toutes ses portes). Elle n'est reliée qu'à des salles **ordinaires** (départ, combat) :
seules, les portes du mur secret restent closes (`doors_closed` de `isaac_boots`, graine 1 : six portes, les quatre de la salle secrète dont deux murées et les deux jumelles de ses deux voisines).
**Accessible à la bombe** : ses voisines sont atteignables du départ sans la franchir (testé sur 1 000 graines, deux
grammaires) et `grid_secret_bomb` l'ouvre par une vraie explosion. **Minicarte** (M2-T18, `hud_model::minimap_rooms`) :
elle lit déjà les bornes de chaque salle et `RoomStates`, donc dessine une grille (grandes salles à leur taille)
sans changement ; l'adaptateur de carte (`update_hud_room_geometry`) ne lui donne pas la salle secrète **tant que son
mur est fermé** (`hidden_from_minimap`, unitaire) : elle apparaît à l'ouverture (`DoorLocks`).

### Trésor à clé

`RoomKindDef.lock_from_floor: u32` (défaut 0, champ de fin de structure absent du contenu existant) : étage de la
séquence `Floors` (indice, 0 = le premier) à partir duquel `door_lock` s'applique ; `RoomKindTable::door_lock_at(kind,
floor)`. `rooms/tresor.ron` d'`isaac` : `door_lock: Some(Key(1)), lock_from_floor: 1` (Q6 : fermé à partir du second
étage). **`DoorLock::Key` n'était exécuté par personne** (T0c le disait pour T4, qui n'a fait que `Secret`) :
- `InteractionType::KeyDoor` (fin d'enum) : au premier tick, la porte d'une salle `Key(n)` de l'étage courant **et sa
  jumelle** (`DoorConfig::paired_door` renseigné : les ouvertures murées restent des portes de salle ordinaires)
  gardent leur collider et leur `Interactable` devient `KeyDoor` (même portée) : `handle_door_interaction` ne les voit
  plus, donc **aucun coût de porte** n'est débité. Prompt du HUD « Ouvrir — clé (n) » / « Fermé — clé (0) » (§81).
- `key_door_system` (`Interaction`, après tous les handlers qui écrivent `Inventory`) : une Interaction sur une porte
  `KeyDoor` consomme `n` clés **de l'interacteur** (Q4 : les clés sont par joueur), retire collider, couche et
  `Interactable` de toutes les portes scellées de la salle, inscrit la salle dans `DoorLocks`
  (`ggrs{f=… key_door_open room=… by=…}`) ; sans clé suffisante : `key_door_refused`, rien ne change.
- ni l'ouverture d'office, ni le verrouillage d'une salle de combat voisine, ni sa fin (`Locked → Cleared`) ne touchent
  une porte à clé, ouverte ou fermée (même règle que le mur secret).
- `DoorLocked` (attente) et le lint lisent `door_lock_at` avec l'étage courant.
**Clé obtenable** : lint (§ plus bas) et donnée : `consumables_per_floor: {"key": 1}` de la séquence `sous_sol`
(servi à l'arrivée du second étage) ; `pleureur.ron` gagne `starting_consumables: {"key": 0}` : `blank_system` ne sert
que les joueurs qui **ont** un `Inventory` (posé au premier ramassage ou par `starting_consumables`), sans quoi la clé du
second étage n'aurait servi personne avant le premier ramassage.

### Salle post-boss

Type `diable` (`rooms/diable.ron`, vide ; le marché est T8). Tirée par `post_boss.chance` (un tiers pour `isaac`), une fois
par étage, accrochée au boss, jamais voisine de la salle secrète. Le boss verrouille sa salle comme avant ; à sa mort ses
portes s'ouvrent, celle de la salle post-boss comprise.

### Lint

`lint_floor_grammars` (grammaires en grille) : taille d'un gabarit d'un type nécessaire multiple de `cell`, ouverture
centrée sur **chaque segment de chaque côté** (message : côté et segment), `FloorGrammar::problems()` : `secret` égal au
départ/boss/remplissage, plus d'une secrète requise, `post_boss.chance` hors [0, 1], `post_boss.kind` déjà pris ;
`lock_from_floor` sans `door_lock` ou sur `Secret` (le mur secret ne lit pas l'étage) ; **clé obtenable** : un type `Key(n)` exigé par une grammaire que nomme une séquence
`Floors` doit avoir une source de ≥ n clés pour l'étage où le verrou s'applique (`consumables_per_floor["key"]` servi à
partir de l'étage 1 seulement, pool de butin qui donne `key`, `starting_consumables`). Fixtures : `grid_template_size_not_multiple`,
`grid_template_missing_opening`, `grid_post_boss_chance_out_of_range`, `lock_from_floor_without_lock`,
`key_room_without_key_source`, `key_room_key_at_floor_zero_only` ; cas valides `grid_large_rooms_ok`,
`key_room_with_key_source`.

### `isaac` en grille

Gabarits `maps/salles.ldtk` (T5 : `docs/taches/rapports/m3-t5-grammaire-grille.make_salles.py` ; **vingt depuis M3-T11d1**,
§76.4, script `m3-t11d1-gabarits.make_salles.py`, qui reprend les dix de T5 tels quels) : `Depart`, `Combat`, `Boss`
(1×1), `BossGrand` (2×2), `Tresor` (1×1), `TresorLong` (2×1), `TresorHaut` (1×2), `Secrete`, `Diable`, `Transit`, plus huit
gabarits de combat, `BossAbris` et `Boutique`. Grammaires :
`sous_sol` (8 à 12 salles, 7×7), `sous_sol_grand` (12 à 15, 8×8, deux boucles), `sous_sol_court` (3 salles, désignée par
`entry.start_map = floor:sous_sol_court` pour les scénarios générés sans carte : `boss_gardien`), `sous_sol_transit` (un
départ et une salle sans ennemi : portail ouvert aussitôt). Séquences : `sous_sol` (**défaut : deux étages**, clé au second),
`sous_sol_un`, `sous_sol_grand`, `sous_sol_court`, `sous_sol_cle` (étage de passage puis `sous_sol` : le second étage des
scénarios à clé sans jouer le premier). Objet `bomb` (copie du testbed : la bombe des scénarios ; `pleureur` en a une au départ depuis M3-T11d1, D81). **Les traces d'isaac bougent** : (1) la carte est une grille ; (2) le joueur a un `Inventory`
(`starting_consumables`). Preuve et traces re-bénies : rapport `docs/taches/rapports/m3-t5-grammaire-grille.md`.

### Scénarios et attente `RoomLayout`

Attente nouvelle `RoomLayout(kind, nth, size: Some((w, h)), touches: [types], apart: [types], at_frame)` (même tri que
`RoomState`, géométrie relue sur `RoomBounds`, à employer sur un étage unique : les mondes des étages se superposent) :
taille en pixels (2×2 = 480 × 288) et voisinage par un bord commun.

| Scénario | Prouve |
|---|---|
| `grid_floor_seed_1`, `_23`, `_123` | un étage entier joué par le bot `explorateur` jusqu'à la victoire (M3-T11d1 : vingt gabarits, plans refaits ; `Armor +50` de D88 pour la graine 1, aucune armure pour les graines 23 et 123) : boss 1×1 (graine 1, trésor 1×2, quatre combats, boutique), boss 1×1 avec salle du diable et trésor 2×1 (graine 23), **boss 2×2** avec salle du diable (graine 123) ; tailles, voisinages, secrète fermée, trésor ouvert au premier étage |
| `grid_secret_room` | graine 25 (graine 5 avant M3-T11d1) : secrète placée (voisine du départ et d'un combat, loin du boss), fermée, rien n'y entre |
| `grid_secret_bomb`, `grid_secret_no_bomb` | graine 39 (graine 8 avant) : **la bombe de départ de `pleureur`** (D81 : mèche 75 frames, rayon 56) posée f35, le joueur s'écarte, l'explosion (f110) ouvre toutes les portes de la secrète sans le blesser, il y entre ; témoin sans pose : il s'arrête contre le mur |
| `grid_treasure_key`, `grid_treasure_floor0` | séquence `sous_sol_cle`, graine -1013904097 (plan du second étage = graine 145 ; -1013904216 et 26 avant M3-T11d1) : le trésor est fermé à clé, la clé du second étage l'ouvre (f56), le joueur entre ; témoin du premier étage : porte ordinaire |
| `grid_post_boss` | graine 8 : la salle du diable collée au boss et à lui seul, boss verrouillé puis nettoyé (f1383, f1726), butin ramassé (f1785), victoire f1727 ; `Armor +100` |
| `isaac_floor_seed_1`, `isaac_boots` | partie ordinaire (séquence par défaut) : premier étage nettoyé, portail, second étage : trésor fermé, clé servie, secrète fermée ; M3-T11d1 : curses `maze` puis `blind`, boutique de trois présentoirs par étage |

Tests : `map::generation::grid` (unitaires : ports par forme, `port_cell`/`origin_for` réciproques, pose d'un 2×2 sur un
rectangle libre qui ne touche que le parent, 1 000 graines, boucles bornées, chance 0 et 1, déterminisme, grammaires
impossibles, plan altéré refusé), `map_ldtk::generation` (`isaac_etages_de_grille_sur_1000_graines` : 2 × 1 000 graines
sur les gabarits réels, `isaac_grille_de_12_a_15_salles_sous_trois_fois_grand_de_gungeon`, M3-T11d1 : génération entière 2,1 à 2,4 × `grand` de gungeon, planificateur 0,14 → 0,67 ms et 278 → 2 396 évaluations de candidats par étage ; borne de 2 à 3, voir D107), `map_ldtk::game::rooms`
(minicarte), `world::rooms` (`door_lock_at`), `content` (lint et fixtures).

### Mesures

1 000 graines, gabarits du test (départ, combat, boss 1×1 et 2×2, trésor 1×1/2×1/1×2, secrète, diable) : 1,08 tentative en
moyenne, 353 évaluations de candidats au plus ; gabarits d'`isaac` (2 grammaires × 1 000 graines) : salle du diable 304
fois sur 1 000 (un tiers), boucle dans 510 étages `sous_sol` sur 1 000 et 929 `sous_sol_grand` ; génération d'un étage de 12 à
15 salles en grille 0,80 ms contre 0,99 ms pour `grand` de gungeon (rapport 0,81, machine partagée ; fiche : moins de 2).

### Questions pour William

Ouverture « centrée sur chaque côté » d'une grande salle : lue par segment de case de salle (voir plus haut). Bot et
trésor à clé : l'`explorateur` les ouvre depuis M3-T14 (§79). Aucune salle secrète n'apparaît sur la minicarte de la
fiche T16 tant que le mur est fermé : T16 reprendra le dessin.

### Atteignabilité

La séquence par défaut d'`isaac` est en grille : toute partie ordinaire la rencontre (deux étages). Secrète : graines 25 et 39
(`grid_secret_*`, M3-T11d1 : 5 et 8 avant les vingt gabarits) ; trésor à clé : second étage de toute partie par défaut, graine -1013904097 de `sous_sol_cle` ;
post-boss : graines 8, 23, 123 (celles-ci n'ont pas bougé de rôle : le plan change, la salle du diable reste).

## 71. Familiers et orbitaux en jeu (M3-T6, chantier C3)

M3-T6 exécute les contrats de §63 : `Follow`, `Orbit`, `MirrorShot`, `BlockBullets`, `FamiliarOf`,
`ItemDef.familiar`. **Aucune trace existante ne bouge** : aucun objet existant n'a `familiar`, donc aucun
`FamiliarOf` ni `FamiliarGun` (neutres) n'existe, et les systèmes sortent sans rien toucher quand il n'y a
pas de familier (`game::familiars`, plugin `FamiliarsPlugin`, dans `RollbackSystemSet::Weapon`).

### Apparition : dérivée de l'état

`familiar_sync_system` compare chaque frame, par joueur, les familiers **voulus** (`desired_familiars` : les objets
tenus dans l'ordre passifs (ordre de ramassage, un passif en double compte deux fois) puis actif ; `per_copy`
fois le personnage de chaque objet à `familiar`) aux familiers **existants** (`FamiliarOf`, triés par rang).
`reconcile` garde le plus long préfixe commun (par personnage) : le reste est détruit par `despawn_rollback()`
puis recréé. Ramasser un objet crée ses familiers (rangs à la suite), perdre l'actif remplacé détruit ceux de
l'actif (queue de liste : les autres gardent leur rang) ; perdre un passif du milieu refait les rangs suivants.
Pas d'événement à rejouer : un rollback avant le ramassage retombe sur le même état. `GgrsNetId` (étiquette =
id du personnage) : porteurs par `GgrsNetId` croissant, puis rang croissant ; le familier naît **sur le porteur**.
Porteur à terre : ses familiers ne bougent ni ne tirent ; porteur mort (entité détruite) : ils restent en place,
immobiles (proposition de la fiche, retenue).

### Un familier est un personnage, ni `Enemy` ni `Player`

Créé par `create_character` (`character/` du jeu), de l'équipe de sa fiche (sinon celle du porteur), avec :
`FamiliarOf { owner, rank }` (rollback, neutre), `FamiliarBehaviors` (liste `ai.behaviors`, **statique**, comme
`EnemyBehaviors`), `FamiliarGun` (neutre, seulement avec `MirrorShot`). Sans `Enemy` : ni compté par les salles,
les vagues, le HUD ; sans `Player` : l'IA ennemie ne le cible pas. **Aucun obstacle** : couche `environment` (aucune
entrée dans la matrice de collision), donc ni joueur, ni ennemi, ni balle n'est gêné par un familier ; il traverse
aussi les murs (il vole). **Immunisé** : `Health.invulnerable_until_frame = u32::MAX`, que les collisions de balles,
de projectiles et de mêlée respectent déjà (traversé, jamais touché). Limite : `Homing`/`Aimed` hostiles « visent
toujours » un personnage invulnérable (§16) : une balle à tête chercheuse peut chasser un familier.

### Comportements (`behaviors::familiar`, pures, `Fixed` seulement)

- **`Orbit { radius, frames_per_turn }`** : `orbit_position(centre, radius, fpt, index, count, frame)` = centre +
  `radius` × (cos, sin) de `index × 2π / count + 2π × (frame mod fpt) / fpt` (sens trigonométrique). `index` = rang
  parmi les orbitaux du porteur, `count` = leur nombre. **Sans état** : se déduit du numéro de frame (fraction de
  tour en entiers : `Fixed` est un I16F16, un tour de plus de 32 767 frames ne déborde pas), angle replié dans
  `[-π, π]` (le CORDIC perd sa précision au-delà de 3π/2). Rang = ordre d'apparition.
- **`Follow { gap }`** : `follow_step(position, ancre, gap)` : immobile à `gap` ou moins de l'ancre, sinon avance de
  `max(5, excédent / 4)` unités par frame sans dépasser l'excédent. Les suiveurs d'un porteur, par rang croissant,
  forment une **file** : le premier vise le porteur, chacun des suivants le précédent (position déjà mise à jour dans
  la frame). Pas de mémoire de trajectoire : la file se déduit des positions courantes.
- **`MirrorShot { every }`** : quand le porteur a **réellement tiré** cette frame, le familier tire **les larmes du
  porteur** (Q13, réponse de William du 2026-10-16, reprise de T6 : `weapon` a disparu du contrat, un familier n'a plus
  d'arme propre) : l'arme **active** du porteur dans son **mode actif**, composée avec ses objets et synergies par
  `combat::weapons::shot_projectile` (`Projectile::composed` de §66 + `items::projectile_mods`, même code que le tir du
  joueur : `carried_projectile_mods`, `tear_motion`, `aim_direction` sont partagés avec `weapon_rollback_system`) et avec
  ses stats de larmes (`Damage`, `Range`, `ShotSpeed`, `ShotSize`, `Luck`), dans la direction de visée du porteur
  (`pan` de l'input), au plus une fois par `every` frames (`FamiliarGun`). Le déclencheur est l'événement transitoire
  `sim_core::triggers::WeaponFired { frame, player_handle }`, émis par `weapon_rollback_system` quand la balle part
  (cadence respectée, chargeur non vide) et lu la même frame (`owner_fired`, pure) : une gâchette tenue que la cadence
  refuse n'émet rien (à la frame 0, `last_fire_frame` valait 0 par défaut et faisait croire à un tir). Événement
  transitoire : hors checksum, hors snapshot. Un seul tir par déclenchement (pas de plombs multiples pour un fusil à
  pompe), ni dispersion ni recul (aucun RNG consommé), ni chargeur ni cadence du porteur consommés. La balle porte le
  `GgrsNetId` et l'équipe du **porteur** (les kills et les points lui reviennent, elle ne le touche jamais),
  `player_handle` du porteur, les tags du familier, `friendly_fire` de l'arme du porteur. Porteur sans arme (inventaire
  d'armes vide) : pas de tir. **Traces** : un familier dont le porteur tient la même arme que l'ancien `weapon` du
  familier et aucun objet de larmes tire **exactement** la même balle qu'avant (`isaac` : `frere` tirait `larmes`, ses
  traces ne bougent pas) ; le testbed (`compagnon` tirait `pistol`, dégâts 15 vitesse 400) tire désormais `larme`
  (3,5 / 200) : preuve au § « Q13 : traces bougées » ci-dessous.
- **`BlockBullets`** : entre le mouvement des balles et leurs collisions, une balle (ordinaire ou composable) que
  `team_allows_hit` dit hostile au familier et dont le **disque balayé** de la frame (segment ancienne → nouvelle
  position, rayon de la balle) touche son collider est détruite par `despawn_rollback()` (sans `on_expire`), comme le
  blank. L'AABB balayée ne sert qu'à présélectionner ; le contact se décide par `swept_hits_collider` (pure,
  `FixedWide`) : rond = distance du centre au segment < R + r ; rectangle = segment qui le traverse, ou distance
  segment–rectangle < r (bouts du segment et coins). Une balle qui frôle un coin n'est donc plus détruite.

Ordre dans `Weapon` : `weapon_rollback_system` → `familiar_sync_system` → `familiar_motion_system` →
`familiar_mirror_shot_system` → `bullet_rollback_system` → `familiar_block_bullets_system` →
`bullet_rollback_collision_system`. Les `GgrsNetId` des balles de familier sont donc alloués entre ceux des tirs de
joueurs et le mouvement des balles, dans l'ordre porteur puis rang.

### Lint et attentes

Inchangé pour l'essentiel (§63). Un personnage cité par un objet `familiar` n'est plus signalé « n'apparaît dans
aucune salle » (son atteignabilité est celle de l'objet). Attente de scénario **`FamiliarDistance { handle, rank,
to_rank, min, max, at_frame }`** : distance du familier de rang `rank` du joueur `handle` à son porteur ou au
familier de rang `to_rank` (avec `FamiliarCount` de §63).

### Testbed et scénarios

Personnages `compagnon` (`Follow` 28 + `MirrorShot` tous les 12), `orbital` (`Orbit` 36, un tour en 120 frames,
`BlockBullets`), `rempart` (`Orbit` 24 en 36 000 frames : quasi immobile à l'est du porteur, `BlockBullets`). Objets
`compagnon`, `compagnon_actif` (actif), `orbital`, `essaim` (`per_copy: 2`), `trio` (`per_copy: 3`), `rempart`.

| Scénario | Prouve |
|---|---|
| `familiar_follow` | apparaît sur le porteur, reste dans sa laisse, rattrape et garde 28 unités |
| `familiar_mirror_shot`, `..._sans_objet` | une larme du porteur, deux coups sur la cible (le familier f81, le porteur f83 ; avant Q13, `pistol` : f58) ; témoin : un seul |
| `familiar_mirror_items`, `..._sans_objet` | Q13 : avec `perforant` (`Pierce(1)`), la larme du porteur **et** celle du familier traversent la première cible et s'arrêtent dans la deuxième (deux coups chacune, f50 et f62, la troisième jamais touchée) ; témoin sans `perforant` : deux coups sur la première, rien sur les autres |
| `familiar_two_copies` | deux `compagnon` en file (rangs 0, 1) et un `trio` (rangs 2 à 4, à 120°) |
| `familiar_lost_with_active` | l'actif remplacé détruit son familier, l'orbital passif reste |
| `orbital_blocks_bullet`, `..._sans_objet` | l'orbital arrête la boule de feu de la tourelle (f125 : au bord, 19 en vol comme le témoin ; f126 : 18 contre 19), santé 100 à f200 contre 74 sans objet |
| `familiar_mirror_cadence` | gâchette tenue dès f0 : la cadence refuse le tir jusqu'à f20, le familier ne tire pas avant ; deux balles à f20 (le familier tire désormais la larme du porteur : trace rebénie, Q13) |
| `familiar_quad` | quatre joueurs, deux familiers chacun, synctest (resimulation + checksums à chaque frame) |

Diagnostic : `projectile_probe` (`ALACOD_PROJECTILES=<scénario>:<de>:<à>`) affiche aussi joueurs et familiers
(porteur, rang, position) toutes les 10 frames.

### Q13 : `MirrorShot` tire les larmes du porteur (suivi `m3-suivis-q10-q12-q13-q17`)

Contrat : `Behavior::MirrorShot { every }` (le champ `weapon` disparaît : migration de `BehaviorEntry`, du lint — plus
d'« arme inconnue » — et des deux personnages `compagnon` du testbed et `frere` d'`isaac`). Exécution : voir la puce
`MirrorShot` ci-dessus. Les trois aides communes (`aim_direction`, `tear_motion`, `carried_projectile_mods` /
`shot_projectile`) ont été extraites de `weapon_rollback_system` sans changer une valeur (les 600 autres traces sont
inchangées).

**Traces bougées, avec preuve §5** (dumps complets `ALACOD_DUMP_TRACE`, `main` `2d30a1c1` contre la branche,
`scripts/trace-diff.py` : première frame qui diffère = première frame où un familier tire, une seule ligne en
désaccord : la `Bullet` du familier, `damage: 15, speed: 400` (le `pistol` de l'ancien `compagnon`) devenue `damage: 3.5,
speed: 200` (la `larme` du porteur)) :

| Trace | Première frame qui diffère | Cause |
|---|---|---|
| `familiar_mirror_cadence` | 20 | premier tir du porteur à f20, donc premier tir miroir |
| `familiar_mirror_shot` | 35 | idem (tir demandé f30, balles parties f35) |
| `familiar_quad` | 105 | premiers tirs miroirs des joueurs 1 et 3 |
| `champion_explodes`, `champion_explodes_sans_variante` | 25 | idem ; la larme du familier est plus lente : le champion meurt à f35 au lieu de f29, attentes décalées |
| `generated/testbed/familiar_compagnon` | 21 | idem ; `min_hits` 9 → 8 (`objets/compagnon.ron`) et témoin `_sans_objet` `max` 8 → 7 (dérivé) |

Rien d'autre ne bouge : `familiar_frere` d'`isaac` (le frère tirait déjà `larmes`, le porteur a `larmes`, aucun objet de
larmes) est **identique au bit près** ; les scénarios de `tears_*` et le tir de joueur n'ont pas changé. Neufs :
`familiar_mirror_items` et son témoin `familiar_mirror_items_sans_objet`.

### Atteignabilité

M3-T11b (§76.2) met trois familiers (`frere`, `orbe`, `nuee`) dans le pool trésor d'`isaac` : dette fermée.


## 72. Champions et élites v2 en jeu (M3-T7, chantier D2 v2)

M3-T7 exécute les champs de `VariantDef` de §63 : `on_death`, `behaviors`, `loot`, `chance` en expression ;
`tint` est lu par la présentation seule (T17 le dessine : aucune lecture ici, attente nulle). **Aucune trace
existante ne bouge** : un personnage sans variante, ou dont la variante n'a aucun champ v2, ne passe par rien de
neuf (`champion_death_system` sort sans rien toucher), `PatternGhost` est neutre. Le
tirage de §25 (`RollbackRng` local, jamais `RngStreams` ; graine `fnv1a("variants:<graine>:<net id>")` depuis D88) est inchangé.

### `chance` en expression : valeur figée à l'apparition

`draw_variant(config, forced, graine, net id, VariantContext { players, floor })` : `players` = joueurs de la session
(`PlayersCount`, au moins 1), `floor` = étage de l'apparition (`run::FloorState::index`, 0 hors mode `Floors` ;
`variant_context_at(étage suivant)` au passage de niveau). `DifficultyReader` porte le contexte (`variant_context_at`) ;
les cinq sites d'apparition (`spawn_enemy` : vagues, spawners, renforts de salle, `CharacterSpawn`, placements de
scénario) le passent. Un littéral est exact : même tirage qu'avant, quels que soient joueurs et étage. Le lint
(§63) borne l'expression dans [0, 1] pour 1 et 4 joueurs aux étages 0 et 5 : une expression qui dépasserait 1 à
l'étage 5 est une erreur de contenu (`0,1 + floor × 0,18`). Variante **imposée** (`CharacterSpawn.variant`, placement
de scénario) : aucun tirage, la chance ne compte pas.

Ancien piège de §25, **corrigé par la dette D88** : la graine `fnv ^ graine ^ net id` donnait, via un générateur congruentiel,
des valeurs voisines à des net ids consécutifs (quarante ennemis posés d'un coup tiraient presque le même nombre ; les
scénarios de chance choisissaient donc la graine 1942 pour répartir les tirages de part et d'autre des seuils). Depuis D88
(graine par hachage de `variants:<graine>:<net id>`, §25) les tirages sont indépendants ; les scénarios de chance prennent la graine 110
(4 champions sur 40 à l'étage 0, 11 sur 40 à l'étage 1) et leurs attentes sont celles du nouveau tirage.

### `behaviors`

Ceux de la variante **remplacent** ceux du personnage à la création (`enemy::create::spawn_enemy`, après le tirage) :
`EnemyBehaviors::rules`, et `ai.ranged` suit la première règle `Shoot` comme pour un personnage. `BehaviorRuntime`
est posé si la nouvelle liste en a besoin. Un personnage dont la variante n'a pas de `behaviors` garde le chemin d'avant.

### `on_death`

`champion_death_system` (`RollbackSystemSet::DeathManagement`, après `rollback_check_defeat` et
`shell_drop_on_death_system`, avant `rollback_apply_death`, comme `boss_defeated_system`) : pour chaque mort qui porte
`Variant`, par `GgrsNetId` croissant, **à la frame de sa mort** :

- **`SpawnCharacters { id, count }`** : `count` personnages `id` de l'**équipe du mort**, santé résolue (F5 × difficulté),
  par `spawn_enemy`, à des **décalages fixes** (`split_offsets` : un anneau de 14 unités, rang 0 vers +x, aucune
  consommation de RNG), `GgrsNetId` alloués dans l'ordre (mort, puis rang `k` ; clé de contenu `champion:<net id>:<k>`
  dans les logs `champion_split`). Ils sont créés **sans variante** (`NO_VARIANT` = `"-"`, absente de toute table) : pas de
  division en cascade, même si la chance du personnage est de 1.
- **`Explode { radius, damage, destroys }`** : **le mécanisme de M3-T4 (§69), sans second chemin** : une **bombe à mèche
  d'une frame** (`world::Bomb`, `fuse_until = mort + 1`, comme l'`Action::Explode` des effets) posée à la position du mort,
  **de son équipe**, tir ami `Always`, créditée au dernier joueur qui l'a touché (points des victimes), sinon au mort ;
  `destroys` est transmis. `game::bombs::bomb_fuse_system` puis `explosion_system` l'exécutent à la frame suivante (une
  `ExplosionRequest`, un `DamageEvent` par cible, un `Exploded`, rayon strict, exclusions `Death` et **invulnérables** :
  un familier immunisé n'est jamais touché, scénario `champion_explodes`). Une seule émission de dégâts et d'`Exploded`
  par explosion ; `PendingBlasts` et `champion_blast_system` (premier jet de T7) n'existent plus. L'équipe du mort
  (`Enemies`) est celle de la source : la difficulté d'étage multiplie donc les dégâts, comme ceux de tout ennemi.
- **`SpawnPattern { pattern, weapon }`** : un émetteur **sans personnage** (`PatternGhost`, rollback neutre) à la position du
  mort, de l'équipe du mort, joue le pattern avec la table de projectiles de l'arme (`effects_runtime::spawn_pattern`, le
  chemin des effets), puis `pattern_ghost_cleanup_system` le détruit (`despawn_rollback`) quand son `Emitter` est fini. Les
  projectiles naissent à la frame suivante (`Weapon`). Lint ajouté : un pattern **infini** (`Ring` avec `every > 0`, y compris
  atteint par une chaîne `Named`, profondeur bornée ; chaîne cyclique ou inconnue refusée aussi) est
  refusé à la mort (`Unsupported`) : l'émetteur n'a pas de personnage pour l'arrêter ; utiliser `every: 0`.
- Toute autre action est refusée par le lint (§63) et, si elle arrivait, ignorée avec un `warn!`.

### `loot`

La table de la variante (`loot/<table>.ron`) est tirée à la mort par `items::draw_loot_with` avec un `RollbackRng` de graine
`items::loot_seed(graine de run, étage, "champion:<net id>")` (§48 : indépendant du moment, jamais `RngStreams`), les
déblocages actifs de la run appliqués comme pour un coffre ; l'objet (ou l'arme, `items::loot_weapon`) est posé au sol
16 unités à l'est du mort (`spawn_item_pickup`, `spawn_loot_weapon`) et se ramasse comme n'importe quel objet.

### Testbed et scénarios

Personnages `elite_explosif` (variantes `explosif`, `salve`, `nu`), `elite_divise` (`divise`, `nu`), `elite_riche` (`riche`,
`nu`), `elite_comportement` (`distant`, `nu`), `elite_chance` (chance `0,1 + 0,18 × floor`) ; pattern `couronne_unique` ;
séquence `deux_vides` (deux niveaux sans ennemi, le portail s'ouvre aussitôt). Les scénarios **forcent** la variante
(`variant:` du placement) et gardent un témoin `_sans_variante` (variante `nu`, sans effet).

| Scénario | Prouve |
|---|---|
| `champion_explodes`, `..._sans_variante` | `Explode` à la frame suivant la mort (f29 → f30) : joueur 90, cible proche 4990, lointaine 5000 ; le familier (10 PV, immunisé) survit ; témoin : personne blessé |
| `champion_splits`, `..._sans_variante` | trois `elite_divise` en anneau de 14 unités à la mort (f55), sans variante ; témoin : rien |
| `champion_drops`, `..._sans_variante` | l'objet `manche` (graine `champion:14`, D95/D96 : `bottes` avant) est lâché, ramassé à f120 ; témoin : rien |
| `champion_behaviors`, `..._sans_variante` | `KeepDistance` de la variante au lieu de `Chase` : le champion reste à plus de 190 unités ; témoin : il rejoint le joueur |
| `champion_pattern` | `SpawnPattern` : huit boules de feu en couronne à f56, une seule couronne, émetteur détruit |
| `champion_chance_floor_0`, `champion_chance_floor` | `chance` en expression : 2 champions sur 40 à l'étage 0 (10 %), 23 sur 40 à l'étage 1 (28 %) |

Diagnostic : `map_probe` (`ALACOD_MAP_PROBE=<scénario>:<frame>`) liste aussi la variante des personnages.

### Atteignabilité

M3-T11c (§76.3) donne des champions à trois ennemis des salles `combat` d'`isaac` : atteignables par graine choisie (1730) **avant D88** ; depuis D88 (tirage des variantes par hachage, §25) ils sortent avec la graine par défaut et sur la majorité des graines (mesure au rapport `dette-d88-tirage-variantes.md`).

## 73. Pools par type de salle, épuisement à la génération, marché du diable (M3-T8, chantiers C4 v2 et C5 v2)

Exécute les contrats de §64 (`exhaust`, `PoolState`, option A/B) et de §61 (`Price`, `can_pay`/`pay`) avec les
réponses de William du 2026-10-16 : **Q5** (épuisement par run et par pool : trésor, boutique, boss, diable) et **Q6**
(le diable refuse si le paiement tuerait, sinon la vie maximale baisse définitivement ; les cœurs d'âme paient à la
place s'ils sont présents). Code : `items::{PoolState::assign, BossDropDef, RoomShopDef::cost, Price::{can_pay, pay}}`,
`game::{loot::assign_slot, shop::{assign_shelves, pay_cost}}`, `map_ldtk::game::rooms::{room_chest_system,
room_shop_system, room_boss_slot_system, boss_drop_system}`. **Un pool sans `exhaust` et une boutique à prix en douilles
gardent leur comportement et leurs traces** : tout champ nouveau est optionnel et absent du contenu existant.

### Option A retenue : l'objet est tiré à la génération de l'étage

Option A de §64 (option B, tirage à l'ouverture, écartée : elle ne tient pas Q5, le même coffre donnerait un autre objet
selon l'ordre des visites). **Un emplacement** est une clé de contenu (`room:<salle>` coffre de salle, `shelf:<salle>:<i>`
présentoir, `boss:<salle>` butin du boss, la clé d'un placement de scénario). Quand le pool de l'emplacement a `exhaust`,
l'objet est assigné **à la création de l'emplacement** : `PoolState::assign` tire sur la graine
`exhaust_seed(graine, étage, clé, nombre d'objets déjà sortis du pool)` avec `draw_exhausting` (sans l'objet déjà sorti),
retire l'objet du pool (`record`) et **conserve l'assignation** dans `PoolState::assigned` (`"<étage>:<clé>"` → objet).
Ouvrir un coffre, mourir pour un boss ou acheter ne tire plus rien : l'objet vient de `assigned`, donc **ne dépend ni du
moment, ni de l'ordre des visites, ni du joueur** (scénarios `pool_exhaust_time_independent_ab`/`_ba`).

- **Ordre canonique.** Les salles de l'étage courant se parcourent dans l'ordre `(x, y, identifiant)` (`sorted_rooms`) ; les
  trois systèmes (coffres, puis présentoirs, puis butin du boss, `Run`) l'appliquent dans cet ordre. L'ordre de tirage est
  donc une fonction du plan et des types de salle, jamais du jeu. Un pool partagé par deux types de salle
  (`coffre_a` et `coffre_b` du testbed) s'épuise dans cet ordre. Un placement de scénario est assigné à sa pose, dans
  l'ordre de la liste (un scénario fixe lui-même le moment de la pose).
- **Par run, par pool.** `PoolState` est une ressource de la run (remise à zéro à la relance, §64) : le trésor du second
  étage ne redonne pas celui du premier (`isaac_floor_seed_1` : `tresor` et `boss` comptent 1 objet à f5, 2 à la
  génération du second étage). La clé d'assignation porte l'étage : deux étages qui rechargent la même carte (mêmes
  identifiants de salle) ne se confondent pas.
- **Pool vidé.** Quand tous les objets du pool sont sortis, il repart complet (§64) : un objet déjà sorti peut ressortir, rien
  de plus n'entre dans `seen`. `pool_unlock_temoin` : quatre coffres, trois objets, le quatrième répète `bottes`.
- **Déblocages (§51).** `unlocked_table` étend le pool et `hidden_items` retire les objets réservés avant le filtre, comme
  pour tout autre tirage (`pool_unlock` : le déblocage `manche_epuisable` donne quatre objets distincts, `manche` compris).
  Le chemin des boutiques sans `exhaust` (`draw_shelves`) ignore toujours les déblocages (comportement d'avant, dette
  ancienne).
- **Ce qui n'épuise pas.** La récompense de salle (`reward:<salle>`, `room_reward_system`), les rochers et les champions
  tirent à l'usage sur leur graine, avec `seen` vide : ils sont créés en cours d'étage, leur rang dépendrait de l'ordre des
  salles nettoyées, donc du moment (Q5 interdit). Un coffre de récompense ouvert lit `assigned` s'il y trouve sa clé (jamais
  aujourd'hui) puis tire comme avant. Un pool qui a `exhaust` et `exclude_held` est refusé au lint : l'inventaire de l'ouvreur
  est inconnu à la génération.

### Pools par type de salle

Aucun champ `pool` (§64) : le pool est la table nommée par le type de salle. M3-T8 ajoute au contenu de type de salle
(`rooms/<id>.ron`, `world::RoomKindDef`) :

```ron
(locks: true, boss_drop: Some((table: "boss", rarity: Common)))                              // butin du boss
(locks: false, shop: Some((table: "diable", shelves: 2, cost: Some(MaxHealth("2")))))         // marché du diable
```

- **`boss_drop`** (`items::BossDropDef { table, rarity }`) : à la mort du boss (`FrameEvents<BossDefeated>`, `Run` suit
  `DeathManagement` dans la même frame), `boss_drop_system` (ordonné **avant** `floor_transition_system` : un joueur au portail change d'étage dans la même frame, l'événement se lit dans l'étage d'origine) pose l'objet au centre de la salle qui contient le boss : celui
  que `room_boss_slot_system` avait assigné à la génération (pool à épuisement ; un boss mort au tout premier tick, avant ce
  système, est assigné par `boss_drop_system` lui-même avec la même fonction idempotente), sinon un tirage sur la graine
  `boss:<salle>` (pool sans épuisement, indépendant du moment). Pose commune avec le coffre : `game::loot::spawn_drawn_loot`
  (objet, arme ou tas de sous).
- **`shop.cost`** (`items::RoomShopDef::cost: Option<Price>`) : prix **de chaque présentoir** de ce type de salle, quelle que
  soit la rareté. Absent : les prix en douilles par rareté de `economy.ron`, comme avant. `Currency(n)` fixe un prix en
  douilles (`Shelf::price`) ; `MaxHealth(n)` et `Consumable(id, n)` vont dans `Shelf::cost` (le prix en douilles vaut alors 0).
  Le pool d'une boutique peut s'épuiser comme les autres (`assign_shelves` : un emplacement `shelf:<salle>:<i>` par
  présentoir, sur sa propre graine ; les `weapon_shelves` restent réservés aux armes).
- **Secrète et trésor** : les coffres de ces types (`chest.table`) nomment déjà leur pool. `isaac` : `tresor`, `boss`, `diable`
  (`exhaust: true`), `secrete` (consommables, sans épuisement), `salle` (récompense de salle, inchangé).

### Marché du diable : `Price` payé en vie maximale ou en âme

`shelf_interaction_system` (inchangé pour un présentoir sans `cost`) paie un présentoir à `cost` par `game::shop::pay_cost`
(`Price::pay` sur les valeurs du payeur, écrites seulement si le paiement réussit). **Règle de Q6, en un seul endroit
(`Price::{soul_split, can_pay, pay}`, testé)** : les points d'`ExtraHealth` partent **d'abord** (comme au combat, §68) ; ce
qu'ils ne couvrent pas se paie en vie maximale ; **ce reste est refusé s'il tuerait** (`vie courante <= reste`) ou dépasse la
vie maximale. `shelf_interaction_system` refuse aussi si la **stat `MaxHealth` résolue après** la perte serait ≤ 0
(`shop::max_health_after` : un `Set`/`Mul` de `MaxHealth` amplifie la perte ; scénario `devil_deal_refused_multiplied`).
Un payeur à 1 PV sans âme est donc toujours refusé, avec une âme qui couvre le prix il est servi sans perdre une
goutte. Le choix « âme partielle puis reste en vie maximale » est de T8 (Q6 dit seulement « à la place s'ils sont présents ») ;
à relire. Refus : rien ne change (ni vie, ni âme, ni monnaie), `CurrencyEvent { delta: 0, reason: "shop_refused" }`, anti-rebond
`retry_after` de 30 frames, trace `ggrs{… shop_refused … cost=…}`. Succès : trace `shop_buy … soul=<points> max_health_lost=<PV>`.

**La perte de vie maximale est définitive** : `sync_health_from_stats` réécrit `Health.max` depuis la stat `MaxHealth` à
chaque frame, donc abaisser `Health.max` seul se serait défait au tick suivant. Le paiement pose donc un modificateur
permanent `MaxHealth Add −perte` de source `price:<clé du présentoir>` (et abaisse aussi la vie courante de la perte). Un
modificateur multiplicatif sur `MaxHealth` (`Mul`) amplifie la perte réelle de la stat, `sync_health_from_stats` borne alors
la vie courante ; aucun contenu n'en a. Le prix d'un cœur est écrit `MaxHealth("2")` (Q2 : 2 PV = un cœur, `items::HEART_POINTS`).

`Price::Consumable` est exécuté de même (`pay` retire le stock de l'inventaire du payeur ; sans `Inventory`, refus) :
`a_consumable_price_needs_the_stock_and_spends_it`, aucun contenu ne l'emploie encore.

**Prompt du HUD** (`game::ui::hud`, source `prompt`, présentation) : « Acheter Pacte rouge — 1 cœur » (`items::heart_label` :
½ cœur, 1 cœur, 1½ cœur, 2 cœurs…), « — 3 Bombe » pour un consommable, les douilles comme avant. Le prompt est calculé par
les widgets : il n'est pas lisible par `HudText` en headless, d'où des tests unitaires (`ui::hud`). Les autres affichages en
cœurs (barre de vie, prix à l'écran) restent T16.

### Salle du diable (post-boss de T5)

`rooms/diable.ron` d'`isaac` : deux présentoirs du pool `diable` à `MaxHealth("2")`, posés au centre de chaque salle de ce
type que l'étage présente (la salle post-boss de T5, `post_boss.chance` = un tiers, graines 8, 23 et 123 de `sous_sol_un`, encore vraies avec les vingt gabarits). **La chance reste
celle de la grammaire** (tirée une fois par étage à la génération du plan, T5). La proposition de la fiche (réduire la chance
si le joueur a subi des dégâts au boss) demanderait de tirer la salle **après** le combat alors que le plan est posé avant : non
retenue, non confirmée par William (Q6 ne la mentionne pas), à reprendre s'il la veut. Salles d'ange : hors périmètre (Q6).

### Lint et schémas

`exhaust` avec `exclude_held` (`Unsupported`), `shop.cost` (`Price::problems` : montant nul, vie maximale hors `]0, max]` du
joueur `player`, `OutOfRange` ; consommable inconnu, `BrokenReference`), `boss_drop.table` inconnu (`BrokenReference`).
Fixtures `loot_exhaust_exclude_held`, `shop_cost_invalid`, `boss_drop_unknown_pool` (`crates/content/tests/m3_t8_fixtures.rs`).

### Contenu

`isaac` : pools `tresor` (4), `boss` (3), `diable` (4) à `exhaust`, `secrete` (consommables), onze objets passifs de **donnée
d'attente** (une stat chacun : `semelles_lestees`, `pile_rapide`, `lunette_longue`, `trefle_chat`, `griffe_acier`,
`reservoir_vital`, `oeil_vif`, `pacte_rouge`, `cornes_noires`, `queue_fourchue`, `regard_maudit`) : le contenu des pools est
M3-T11 (T11d1 : §76.4), il faut seulement des candidats distincts pour que l'épuisement soit jouable de bout en bout. Salles : `tresor`
(pool `tresor`), `secrete` (pool `secrete`), `boss` (`boss_drop`), `diable` (marché). **Boutique `isaac`** (M3-T11d1, §76.4) : type `boutique`, pool `boutique`, gabarit `Boutique` ; le moteur l'exécute (`assign_shelves`, `shop.cost`) mais le testbed n'a aucune salle `shop` : `cost` n'est exercé que par le diable d'`isaac` (et des présentoirs posés par scénario) ; la branche `Currency(n)` de `shop.cost` n'est exercée par aucun scénario (dette). `testbed` : types de
salle `coffre_a`/`coffre_b`, carte `salles_coffres.ldtk`, étage `coffres_epuisables`, pool `epuisable_temoin` (jumeau sans
`exhaust`).

### Bot

Le bot `explorateur` ne vise pas un présentoir à `cost` sans politique (son `price` vaut 0 : il l'aurait cru gratuit) ; la décision du
diable est celle de M3-T14 (§79 : `DevilPolicy`, défaut `Never`). Il **ramasse** le butin du boss : les parties `grid_*` de l'`explorateur` en sont retardées (voir « Traces »).

### Attentes de scénario

`PoolCount { pool, count, at_frame }` (objets **distincts** sortis du pool), `ItemsOnFloor { id, count, at_frame }` (objets
`id` au sol : le butin d'un boss), `Shelf` gagne `cost: Some(MaxHealth("2"))` ; `Scenario::shop.shelves[].cost` pose un
présentoir à `cost`.

### Scénarios (testbed sauf mention)

| Scénario | Prouve |
|---|---|
| `pool_exhaust_two_chests` / `_temoin` | deux coffres (`k1`, `k14`) du pool à épuisement : `lame` et `bottes`, deux objets sortis ; témoin (pool jumeau sans `exhaust`) : `lame` ×2, `PoolState` vide (D96 : `k5` → `k14`, le tirage ayant changé) |
| `pool_exhaust_two_rooms` | deux salles de types différents d'une carte générée partagent le pool : deux objets sortis dès f10, avant tout coffre ouvert |
| `pool_exhaust_time_independent_ab` / `_ba` | `k1` → `lame`, `k2` → `boussole` que le joueur ouvre `k1` d'abord ou `k2` d'abord |
| `pool_unlock` / `_temoin` | le déblocage `manche_epuisable` : quatre objets distincts dont `manche` ; sans : trois puis `bottes` répété |
| `devil_deal_pay_health` | un cœur payé : vie 100 → 98 et vie maximale 98 définitivement (f30 et f110), l'objet est tenu, douilles intactes |
| `devil_deal_refused_lethal` | à 1 PV (`MaxHealth` `Set 1`) deux tentatives (f10, f60) refusées : rien ne bouge |
| `devil_deal_soul_saves` | témoin : même joueur à 1 PV avec un `coeur_ame` : l'achat passe, âme 4 → 2, vie 1/1 |
| `devil_deal_soul_first` | l'âme paie d'abord (4 → 2, vie et maximum à 100), puis un prix de 6 : l'âme (2) part toute, 4 PV de vie maximale (96) |
| `devil_deal_isaac` (`isaac`) | le cœur d'`isaac` : 6/6 → 4/4 |
| `devil_room_chance` / `devil_room_absent` (`isaac`) | graine 8 : salle du diable, deux présentoirs à un cœur, `pacte_rouge` et `queue_fourchue` sortis du pool `diable`, `bottines` du trésor, `sang_chaud` du boss, trois présentoirs de boutique (M3-T11d1 : les objets tirés suivent le plan et les pools, D96) ; graine 5 : aucune salle, rien tiré dans `diable` |
| `grid_post_boss`, `grid_floor_seed_23` (`isaac`) | le butin du boss tombe (graine 8 : `sang_chaud`, ramassé f1785, victoire f1727 ; graine 23 : `griffe_acier`, ramassé f1537, victoire f1485 ; chronologies de M3-T11d1, vingt gabarits) et le bot le ramasse |
| `isaac_floor_seed_1` (`isaac`) | `tresor` et `boss` : 1 objet sorti à f5, 2 à la génération du second étage (par run) |

Unitaires : `items::loot` (`assign_*` : sans `exhaust` rien n'est écrit, objets distincts, idempotence, premier emplacement =
tirage d'avant `exhaust`, par run entre étages, pool vidé, objets cachés), `items` (`Price` avec âme : refus mortel, âme
d'abord, âme partielle, refus sans effet de bord ; `heart_label`), `game::shop` (`pay_cost` : cœur, refus à 1 et 2 PV, âme,
consommable, douilles ; `assign_shelves` sur deux salles), `ui::hud` (prompt en cœurs).

### Traces

(voir le rapport `docs/taches/rapports/m3-t8-pools-marche-diable.md`)

### Dettes connues (M3-T8)

- `max_health_after` ne regarde que la frame courante : un bonus `MaxHealth` temporaire qui expire rend a posteriori le paiement mortel.
- Le butin du boss est détruit par `floor_transition_system` si un joueur est déjà sur le portail à la frame de la mort (le drop est posé avant la transition, mais l'objet vit dans l'étage quitté).
- Garde « butin déjà tombé » : une fois par salle **et par frame** seulement (deux `BossPlan` morts à des frames différentes dans la même salle lâcheraient deux fois).

### Atteignabilité

Trésor, boss et secrète : toute partie ordinaire d'`isaac` (deux étages) ; diable : graines 8, 23 et 123 de `sous_sol_un`, ou
toute partie par défaut dont un étage tire la salle (`devil_room_chance`) ; le bot qui achète au diable : M3-T14 (§79, `bot_isaac_devil`) ; la boutique
d'`isaac` (gabarit, type, pool `boutique`) : M3-T11d1 (§76.4).

## 74. Curses d'étage en jeu : tirage, modificateur, étage agrandi, HUD masqué (M3-T9, chantier F4)

Exécute les contrats de §64 (les cinq curses de Q7). Code : `game::curses` (tirage et modificateurs),
`world::FloorGrammar::scaled_rooms` + `map_ldtk::loader::with_curse_floor_rooms` (étage agrandi),
`game::ui::hud_model` et `game::ui::hud` (présentation). **Une séquence `Floors` sans champ `curses` ne tire
rien** : `CurseTable` est vide, `curse_system` rend la main avant de toucher une ressource ou un composant, le HUD
lit des valeurs absentes ; aucune trace existante ne bouge (preuve en fin de §).

### Tirage : quand et par quoi

`curse_system` (`GgrsSchedule`, `RollbackSystemSet::FrameCounter`, avant l'incrément de frame, donc **après** le
passage d'étage du set `Run`) compare `FloorState::index` au dernier étage traité (`RunCurses::entered`, rollback) :
à l'entrée dans un étage, **y compris le premier** (frame 0), il tire `run::draw_curse` (RNG local de graine
`curse_seed(graine, étage)`, jamais `RngStreams`), écrit `RunCurses` et la trace
`ggrs{f=… curse floor=… id=<id|none>}`. La fiche disait « à `FloorEntered` » : cet événement n'est émis que quand les
horloges ou la difficulté sont activées (§23) et jamais pour le premier étage ; le tirage détecte donc lui-même
l'entrée, à la même frame que `FloorEntered` l'aurait été. `RunCurses` devient `{ floors: BTreeMap<étage, id>, entered:
Option<u32> }` (T0e : tuple `BTreeMap` seul) : sans `entered`, « tiré, pas de curse » ne se distinguerait pas de « pas
encore tiré » et le tirage serait refait à chaque frame. Toujours neutre (`Default` : hash 0) ; **elle ne l'est plus
dès qu'une séquence a un champ `curses`** : `entered` s'écrit à la frame 0, donc les traces d'une partie à curses
changent (`curse_pool_unset` de T0e, seul scénario dans ce cas sur `main`, est retiré : son propos — « rien n'exécute le
tirage » — n'est plus vrai). Moment clé `curse` (« étage 2 : curse « darkness » ») ; les étages sans curse n'en sont pas.
`CurseTable` (hors rollback : champ `curses` de la séquence + `curses/*.ron` du registre) est posée à `compute_floor_plan`
comme `FloorLootTables`.

**Graine du tirage = graine de carte** (`game::curses::curse_map_seed`, `MapGenerationConfig::seed as u32`), pas
`RunSeed` : la session remplace `RunSeed` par `restart_seed(graine de carte, n° de partie)` après un « Rejouer » en
ligne ou une entrée du hub (§33), alors que les cartes sont assemblées avec la graine de carte inchangée. Avec `RunSeed`,
`FloorRooms` (génération) et `curse_system` (entrée) auraient lu deux graines : un étage doublé sans « Labyrinth »
annoncée, ou l'inverse. Même expression des deux côtés, test `the_draw_follows_the_map_seed_not_the_derived_run_seed`.
Conséquence assumée : un « Rejouer » rejoue les mêmes curses (comme il rejoue les mêmes cartes ; le butin, lui, suit
`RunSeed`).

### Les cinq effets

| Curse (Q7) | Effet | Où | Quand |
|---|---|---|---|
| `Modifier(stat, op, v)` | modificateur de source `curse:<id>` sur chaque joueur | `curse_system` | posé à l'entrée de l'étage ; **tout** `curse:*` est retiré à l'entrée de l'étage suivant avant la pose de la nouvelle ; un joueur qui n'a pas le compte attendu (apparu après l'entrée) le reçoit à la frame suivante ; rien n'est écrit tant que le compte est bon (pas de `Changed`) |
| `FloorRooms(×m)` | `rooms` de la grammaire multiplié | `with_curse_floor_rooms`, `setup_generated_map` | **à la génération** : toutes les cartes d'une séquence sont assemblées au chargement de la partie, avant la première frame. Le tirage ne dépendant que de (graine de carte, étage), le planificateur retrouve la curse que `curse_system` écrira à l'entrée |
| `HideMinimap` | `hud_values.minimap` vide | `Update` | tant que `RunCurses` donne la curse à l'étage courant |
| `HideHealth` | `hud_values.health` absent : ni texte ni barre (D22) | `Update` | idem |
| `HideItemNames` | nom de l'objet ou de l'arme du prompt → `???` (`Weapon`, `Shelf` ; prix et reste du texte intacts) | `Update` (`ui::hud`) | idem |

**Étage et emplacement.** Le monde d'un emplacement `s` de la séquence est assemblé avec la curse de l'**étage `s`**
(un emplacement par `floor:<id>` de `levels`). Au-delà de la séquence, `level_for_floor` boucle sur le dernier niveau :
ces étages réutilisent le monde déjà assemblé et ne reçoivent **pas** de `FloorRooms` de leur propre tirage (leurs
modificateurs et leur HUD, eux, suivent leur tirage). Sans effet sur une carte qui n'est pas une grammaire (`floor:<id>`).

**`FloorGrammar::scaled_rooms(m)`** (pure, arithmétique `Fixed`) : bornes de `rooms` × `m` arrondies au plus proche, puis
**ramenées à une grammaire satisfiable** : au moins départ + boss + requises (et `min_boss_distance + 1`), au plus la
capacité de la grille (`max.0 × max.1`) ; `m = 1` rend la grammaire inchangée. `isaac/sous_sol` ×2 : 16 à 24 salles. Preuves :
1 000 graines en grille 7 × 7 (`mille_graines_doublees_par_la_curse_respectent_la_grille` : validées, sous la borne de
travail, 16 à 24 salles) ; 2 × 200 graines sur les gabarits réels, en grille (`isaac`) et en arbre (`gungeon/etage`, 20 à
28) : `la_curse_labyrinth_double_les_salles_sans_rendre_l_etage_impossible`. **Limite** : un multiplicateur que la grille ne
peut pas tenir (×100 sur 7 × 7, jusqu'à 49 salles) échoue explicitement (`FloorError::Exhausted`, comme toute grammaire
impossible) ; seules les valeurs jusqu'à ×2 sont prouvées sur `isaac`.

### Présentation

`game::curses::CurseHides { minimap, health, item_names }` (pur, `CurseHides::of(effects)`), lu depuis `RunCurses` à chaque
`Update` : jamais d'un événement, donc juste après un rollback. `HudPlayerInput::curse` (id et définition de la curse de
l'étage courant) alimente `hud_values` : `health` et `minimap` sont **absents** (même chemin que « joueur mort » :
barre à zéro, texte vide sans préfixe). Nouvelle source `curses` (`content::ui::HUD_SOURCES`, `PLAYER_SOURCES`) : l'id de la
curse de l'étage courant, vide sans curse ; `isaac/ui/hud.ron` la montre (« Malédiction : darkness »). Le prompt d'objet est
calculé dans `ui::hud` (hors `HudSnapshot`) : `HideItemNames` y est prouvé par test unitaire, pas par `HudText`. La machine à
perk, les portes et les coffres n'ont pas de nom d'objet : inchangés. Les écrans de pause et d'inventaire (§56) ne lisent
pas `RunCurses` (hors périmètre : ils n'affichent ni la santé, ni la minicarte, ni le nom des objets du sol).

### Contenu et lint

- `testbed` : `floors/maudit.ron` (chance 0,5, les cinq curses), `floors/maudit_maze.ron` (chance 0,5, `maze` seule).
- `isaac` : dossier `curses/` (les cinq de Q7, copies de celles du testbed), `floors/sous_sol_maudit.ron` (un étage
  `floor:sous_sol`, chance **1** : la graine choisit la curse). La séquence par défaut d'`isaac` (`floors/sous_sol.ron`) a des `curses` depuis M3-T11d1 (chance 0,3 par étage, les cinq selon leurs poids, §76.4) ;
  les séquences de test `sous_sol_un`, `_cle`, `_court`, `_grand` n'en ont pas.
- Lint : `reach::stat_uses` compte désormais le `Modifier` d'une curse : une stat que la simulation ne lit pas
  (`stat_reader`) est une erreur `UnreadStat`.
- Attentes : `RoomCount(min, max)` (nombre de `RoomBounds`), `HudText(empty: true)` (texte vide : une source masquée ;
  `contains: ""` ne prouvait rien).

### Scénarios (graines choisies par `draw_curse`, vérifiées en jeu)

| Scénario | Prouve |
|---|---|
| `curse_drawn_seeded`, `_other` | `maudit`, graines 2 (darkness puis rien) et 22 (labyrinth puis darkness) : `Curse` aux deux étages, source `curses` du HUD, moment clé `curse` ; le test `curses` rejoue la graine 2 deux fois (même trace), la graine 22 (autre trace) et la graine 5 (aucune curse, aucun moment clé) |
| `curse_modifier` | `maudit_maze`, graine 2 : `MoveSpeed` 150 → 127,5009155 à l'entrée de l'étage 0, **150** au passage du portail ; le test `curses` mesure l'abscisse du joueur : sous `maze` il avance de 70 à 95 % de la distance libre |
| `curse_hides_minimap` | `isaac/sous_sol_maudit`, graine 2 : `minimap` **vide**, santé « 6/6 » intacte, 8 à 12 salles |
| `curse_hides_health` | graine 5 (unknown) : `health` **vide** le joueur vivant, minicarte affichée |
| `curse_blind` | graine 3 (blind) : témoin, santé et minicarte visibles, 8 à 12 salles |
| `curse_bigger_floor` | graine 6 (labyrinth) : `RoomCount` 16 à 24 là où l'étage ordinaire n'a jamais plus de 12 ; boss, trésor, secrète présents |

D95 (2026-10-17) : `curse_seed` passe par `fmix32`, les graines ci-dessus sont les plus petites qui donnent l'issue voulue avec le nouveau tirage (les anciennes : 3, 2, 3, 1, 7, 5, 2).

### Déterminisme

RNG local par (graine, étage) ; `order_mut_iter!` pour les joueurs ; `BTreeMap`, `Fixed` ; trace `ggrs{f=… curse …}` ; aucun
flux de `RngStreams`. Hors séquence à curses, aucune ressource ne bouge : `RunCurses` n'est pas écrite (test
`a_sequence_without_curses_draws_nothing_and_leaves_the_state_neutral`, vérifié à chaque frame), `CurseTable` est vide.

### Atteignabilité

`isaac/sous_sol_maudit` joue les cinq curses par graine ; la séquence par défaut d'`isaac` les tire depuis M3-T11d1 (graines 1, 2, 13, 19, 42 : §76.4). Le testbed joue `maudit` et `maudit_maze` à deux étages.

## 75. Accomplissements en jeu : faits de run, évaluation à `Ended`, profil v2 (M3-T10, chantier G1 v2)

Code : `game::profile` (`RunHistory`, `record_held_items`, `write_achievements_at_run_end`, `grant_achievements`,
`summary_is_final`, `AchievementsEarned`), `meta` (`Profile::achievements`, `Profile::grant_achievement`,
`Achievements`, `PROFILE_VERSION` 2), `game::boss::boss_defeated_system`, HUD (`achievements`), écran de fin,
contenu `isaac` et `gungeon`. Contrats : §64 (`AchievementDef`, `RunFacts`, `achieved`), §38 (profil), §51 (déblocages).

### Faits de la run : `RunHistory`

`BossDefeated` est un événement **transitoire** (§45) : il ne laisse aucun historique. `game::profile::RunHistory`
`{ bosses_defeated, items_held }` en est un, écrit par la simulation :

- **boss vaincus** : `boss_defeated_system` (`DeathManagement`) y inscrit le **nom du personnage** (le `String` de
  `GgrsNetId`, celui de `characters/<id>.ron` : ce que `BossDefeated("gardien")` désigne). Aucun changement de l'événement.
- **objets tenus** : `record_held_items` (`GgrsSchedule`, `FrameCounter`) recopie, à chaque frame de partie en cours,
  l'union des passifs et de l'actif, **par joueur** (`held_by_player`, union à l'évaluation : `RunHistory::items_held()`), des joueurs qui ont un `Inventory` (comparée avant de cloner : aucun `String` copié tant
  que l'ensemble ne change pas). Un joueur mort perd son inventaire : la **dernière valeur connue reste** (« tenu à la fin » =
  tenu à la dernière frame où quelqu'un tenait quelque chose) ; rien n'est écrit une fois la run terminée (sauf la frame même de la fin : `run_counts_at`). Même garde pour les
  boss, défaite posée à cette frame comprise : la simulation continue après `Ended`, un boss tué ensuite (poison, explosion) n'est pas un fait de la run.
- **étages, temps, issue** : déjà dans `RunSummary` (`floor_reached`, `frames`, `outcome` : la défaite est une issue, il
  n'y a pas de compteur de morts à ajouter).

**Rollback sans checksum** (`RollbackTraceApp::rollback_and_trace_resource_no_checksum`, nouveau : `Clone + Debug`, la
variante `Copy` existait). Choix entre « hors rollback » et « rollback neutre » de la fiche : l'historique est écrit **dans**
`GgrsSchedule` ; hors rollback, une prédiction ratée (boss « mort » dans une frame resimulée autrement) resterait dans
l'ensemble et accorderait à tort un accomplissement écrit sur disque. En rollback, le rollback le défait. Sans checksum :
rien en simulation ne le lit, donc aucune trace ni aucun checksum ne bouge, quel que soit le contenu (une ressource neutre
bougerait dès qu'un boss meurt). Contrepartie : un desync de cet historique ne serait pas détecté, ce qui est sans effet
sur la simulation. Remise à zéro à la relance (`run_state::cleanup_rollback_world_system`).

### Évaluation hors simulation

`write_achievements_at_run_end` (`Update`, chaîné après l'écriture d'essence de §38 et après `apply_run_request_system`, jamais
`GgrsSchedule`) : quand `Run` est `Ended` avec un `RunSummary`, **une fois par run** et une fois la fin **confirmée**
(`summary_is_final` : `summary.frames <= ConfirmedFrameCount`, qui vaut -1 hors session ; un **abandon** est définitif tout de
suite : décision locale, la session est détruite à la frame suivante), il construit `RunFacts { summary, bosses_defeated,
items_held }` et, pour chaque joueur de la **session** (`bevy_ggrs::LocalPlayers` : tous en partie locale ou synctest, seulement
les joueurs locaux en p2p, morts compris : un mort n'a plus d'entité), appelle `grant_achievements` :
chaque accomplissement de la table `meta::Achievements` (chargée du registre avec les autres tables, hors rollback) dont
`meta::achieved` est vrai et que le profil n'a pas encore obtenu ajoute son id à `Profile::achievements` et ses `unlocks` à
`Profile::unlocks`, puis le profil est écrit (atomique, §38). Pourquoi attendre la confirmation : sur disque, un
accomplissement ne se reprend pas, et une fin vue pendant une prédiction peut être défaite. Contrepartie : quitter l'écran de
fin (Rejouer, Lobby) **avant** la confirmation (quelques frames en local, un aller-retour réseau en p2p) perd l'évaluation.
**Abandon sans confirmation** : c'est une décision locale du joueur, la session est détruite à la frame suivante et la confirmation n'arriverait jamais ; les faits peuvent donc venir de frames encore prédites (surtout en p2p) et un accomplissement peut être accordé sur un boss « vaincu » par une prédiction ratée. Accepté : l'alternative (ne rien évaluer à l'abandon) perd des accomplissements réels, et le cas demande un abandon pile pendant une prédiction fausse.

Même règle d'`ended_handles` pour l'essence (§38) : un joueur mort la reçoit désormais aussi. `AchievementsEarned`
(`{handle → ids}`, présentation, vidé à la relance) garde les accomplissements **nouveaux** de cette fin.

**Profil v2** : `PROFILE_VERSION` 2, champ `achievements: BTreeSet<String>` (`#[serde(default)]`). Un fichier v0 ou v1 se
lit tel quel (champ absent : aucun accomplissement) et se réécrit en v2 ; un fichier plus récent que le jeu reste `TooNew`
et n'est pas touché : `meta::save` **refuse** de remplacer un fichier existant illisible ou plus récent (`TooNew`), que `Profiles::load_dir`
a écarté au lancement (la partie joue alors avec un profil vierge qui n'est pas écrit : sans cette garde, un build v1 relancé après un
build v2 aurait détruit le profil v2 à la première fin de run). Un accomplissement déjà obtenu ne rajoute rien : un déblocage retiré à la main ne revient pas.

**Entrée dans la run suivante** : par `RunUnlocks` (§51), **sans autre chemin**. La relance relit le profil
(`jjrs::local`), filtré sur les `Unlock` du jeu : les ids d'accomplissements (`Profile::achievements`) n'y entrent jamais, seuls
leurs déblocages. Le marchand du hub (§57) et `alacod-profile unlock` **ne vendent pas** un déblocage qu'un accomplissement
accorde (`Registry::achievement_unlocks`) : on l'obtient en jouant.

**En ligne : règle de l'hôte** (décision du 2026-10-15). Les faits sont ceux de la partie commune, identiques chez tous les pairs
(l'historique suit les rollbacks, n'est plus écrit après `Ended`, et la fin est évaluée une fois confirmée) : chaque pair n'écrit que le profil de **ses** joueurs
locaux (`LocalPlayers`). Ce qui entre dans la run suivante est inchangé : le profil de **l'hôte** (`ALACOD_RUN_UNLOCKS`, §51), jamais celui du pair
local. L'accomplissement d'un invité est donc dans son fichier, et ne sert que quand c'est lui qui héberge. `ItemHeld` lit
l'union des objets des joueurs de la run (contrat de §64), pas ceux d'un joueur.

### Présentation

Source HUD `achievements` (`content::ui::HUD_SOURCES`, source du joueur affiché) : une ligne par accomplissement **obtenu à
cette fin de run** (`AchievementDef::name`, nouveau champ optionnel : l'id sinon) ; vide en cours de partie, donc rien (et vide aussi quand le joueur affiché est mort : le HUD ne montre plus rien de lui, l'écran de fin
liste tout). L'écran
de fin (`ui::game_over`) l'ajoute sous le résumé (« Accomplissements : * … »). Pas d'écran des accomplissements (hors périmètre) :
le profil les garde (`Profile::achievements`).

### Attentes de scénario

`ProfileHas` gagne `achievement: Some("id")` (obtenu), `lacks_unlocks` et `lacks_achievements` (absents du profil) ;
`HudText(source: "achievements", contains | empty)`. Un scénario n'écrit toujours rien sans `profile: true`.

### Contenu

`isaac` : quatre accomplissements. Les objets débloqués sont des **objets neufs**, ajoutés aux pools **et** cachés par un `Item`
(§51) : un pool dont les candidats d'une entrée sont tous cachés ne compte pas cette entrée (`draw_exhausting`), donc, profil
vide, le tirage est celui d'avant : **aucune trace d'`isaac` ne bouge**. Retirer un objet existant d'un pool décalerait les
tirages de tous les scénarios.

| Accomplissement | Condition | Déblocage | Ce qui entre |
|---|---|---|---|
| `gardien_vaincu` | `BossDefeated("gardien")` | `medaille_libre` | « Médaille du gardien » (`medaille`, +10 % dégâts) dans le pool `boss` |
| `run_finie` | `Victory` | `couronne_libre` | « Couronne de fin » (`couronne`, +15 % portée) dans le pool `tresor` |
| `second_etage` | `FloorAtLeast(1)` | `bague_libre` | « Bague des profondeurs » (`bague`, +10 % cadence) dans le pool `tresor` |
| `semelles_aux_pieds` | `ItemHeld("semelles_lestees")` | `bombes_en_reserve` | la bombe (poids 3) dans le pool `salle` (`Loot` : rien n'est caché) |

`gungeon` : quatre accomplissements dont les déblocages sont des entrées `Loot` (rien ne quitte un pool : aucune trace ne
bouge) ; le marchand ne les propose pas.

| Accomplissement | Condition | Déblocage | Ce qui entre |
|---|---|---|---|
| `gatling_vaincu` | `BossDefeated("gatling")` | `butin_gatling` | `parchemin` dans `coffre_etage1` et `coffre_etage2` |
| `gungeon_termine` | `Victory` | `butin_victoire` | `sablier` dans `coffre_etage2` |
| `second_etage` | `FloorAtLeast(1)` | `butin_second_etage` | `trophee` dans `coffre_etage2` |
| `bottes_aux_pieds` | `ItemHeld("bottes")` | `butin_bottes` | `cartouchiere` dans `coffre_etage1` et `coffre_etage2` |

Le lint d'`isaac` ne signale plus « aucun accomplissement dont le déblocage donne un objet ou une entrée à un pool tiré »
(T12, §77). Le « Q10 » de la fiche (tenir une transformation) n'a pas de réponse : `semelles_aux_pieds` tient un objet.

### Preuves

Unitaires : `meta` (migration v1 → v2, déblocages accordés une fois), `game::profile` (accomplissements atteints seulement et
une fois, fin confirmée, noms affichés, objets tenus et dernière valeur après la mort, rien après la fin), écran de fin.
Scénarios `achievement_unlocks` (isaac, graine 8, `sous_sol_un` : boss vaincu + victoire accordent deux accomplissements, le
troisième manque, `RunUnlocks` ne bouge pas pendant la run, HUD) et `achievement_not_met` (isaac, défaite à l'étage 0 : aucun
accomplissement, essence créditée, HUD vide). Test `crates/scenario/tests/achievements.rs` : deux runs enchaînées, la relance
voit les déblocages dans `RunUnlocks` et, **à graine égale** (400 graines), les objets entrent dans les pools `boss` et `tresor`,
pas ceux dont l'accomplissement manque ; l'historique de la première run n'a pas survécu à la relance.

**Contrat du tirage : un déblocage ne déplace que vers l'objet débloqué (D96, 2026-10-17).** À graine, emplacement et `seen` égaux, ce
que débloque un accomplissement ne change l'objet d'un emplacement que si c'est pour un objet qu'il apporte (l'objet débloqué d'une entrée
`Item` ; pour une entrée `Rarity` ajoutée ou alourdie, **n'importe lequel de ses candidats**, y compris un qui existait déjà) : une graine partagée donne
le même butin à deux profils, aux objets débloqués près. Ce n'est **pas** un effet du hasard : c'est la course pondérée de
`draw_exhausting` (§64), et `les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante` l'exige (400 emplacements par
pool, un `PoolState` neuf par emplacement). Avec l'ancien choix `tirage % somme des poids`, la propriété n'était garantie par rien : elle
ne tenait que pour un contenu de pool donné (le test, vert sur la branche de T10, est rouge sur `main` depuis le merge 3f37ba43 :
débloquer `medaille` faisait passer la somme du pool `boss` de 10 à 12 et 251 emplacements sur 400 changeaient pour un objet non débloqué ; D96). Limite dite au §64 : le long d'une suite
d'emplacements à épuisement, l'objet déplacé revient dans le pool et peut ressortir plus loin.

### Déterminisme et traces

Rien de non rollbacké n'est lu par la simulation. `RunHistory` est rollback sans checksum, `RunUnlocks` inchangée,
`AchievementsEarned` et `Achievements` hors rollback (présentation et profil). Aucune trace existante n'a bougé (suite
complète sans BLESS) ; seules les traces des deux scénarios nouveaux sont ajoutées.

### Limites

- Pas de hub dans `isaac` : un déblocage d'accomplissement n'y a d'autre chemin que le profil (voulu, fiche).
- L'ensemble en ligne vient de la variable d'environnement de l'hôte (§51) : l'échange réseau des profils n'existe pas.
- `ItemHeld` est l'union des joueurs, pas par joueur ; un accomplissement par joueur (« tenir X en coop ») viendrait avec un
  `RunFacts` par joueur.

## 76. Contenu `isaac` (M3-T11, quatre parties)

### 76.1 T11a : larmes et stats (`m3-t11a-larmes`)

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`) ; aucun code moteur.

- **`larmes` en `mag: Infinite`** (§60) : plus d'`ammo_type`, plus de chargeur de 9 999. Les stats de larmes sont des
  multiplicateurs de base 1 que posent les objets : `Damage` (3,5), `FireRate` (3 tirs/s), `Range` (300), `ShotSpeed` (260),
  `ShotSize` (collision et sprite), `Luck` (chance des `Chance`). `MoveSpeed` est celle du personnage.
- **Statuts** `statuses/` (copies du testbed : `poison`, `brulure`, `gel`, `peur`, `lenteur`), déclarés dans `game.ron`.
- **19 objets** `objets/*.ron` (tag `larmes`), tous dans le pool `loot/tresor.ron` (entrées `Item(id)` : un `Rarity(…)` piocherait
  aussi clé, bombe et cœurs, de rareté `Common` ; poids commun 4, peu commun 2, rare 1 ; `exhaust: true` (T8 : un objet sorti n'y revient plus par run, donc pas de doublon par ce pool), pas d'`exclude_held`. Il contient aussi les quatre passifs de T8). `rooms/tresor.ron` y pointe (`chest.table: "tresor"`) ; la salle secrète garde `salle`.
  Boutique : pool `boutique` de M3-T11d1 (§76.4).
- **Lisibilité (§82)** : `bullet_player`/`bullet_enemy` d'`isaac` utilisent le halo blanc de M3-T17
  (`sprites/feedback/tear.png`, copie du testbed, `assets.yaml`) : teinte par statut/guidage, opacité spectrale, taille par
  division/perforation, sans code. Les objets n'ont pas de sprite propre (aucun champ d'image dans `ItemDef`).
- **Lab** : `characters/cible_fragile.ron` (15 PV, `test_only`) et l'arme `larmes_courtes` (portée 100, `test_only`) : l'arène
  (416 x 304) est plus petite que la portée de `larmes`, `Range` ne se prouve qu'avec une arme courte.

| Objet | Effet | Rareté | Graine `alacod-sim` (premier coffre de trésor) | Scénarios générés |
|---|---|---|---|---|
| `encre` | `Damage` +50 % | Common | 48 | `item_encre`, `tears_encre` (+ `_sans_objet`) |
| `ressort` | `FireRate` +50 % | Common | 35 | `item_ressort`, `tears_ressort` |
| `longue_vue` | `Range` +100 % | Common | 44 | `item_longue_vue`, `tears_longue_vue` |
| `vent_arriere` | `ShotSpeed` +50 % | Common | 1 | `item_vent_arriere`, `tears_vent_arriere` |
| `goutte` | `ShotSize` ×1,5, `Damage` +20 % | Uncommon | 11 | `item_goutte`, `tears_goutte` |
| `bottines` | `MoveSpeed` +25 % | Common | 4 | `item_bottines` |
| `trefle_noir` | `Luck` +2 | Uncommon | 53 | `item_trefle_noir` |
| `plomb_fondu` | `Damage` +100 %, `FireRate` −30 % | Rare | 56 | `item_plomb_fondu` |
| `oeil` | `Homing` | Common | 95 | `item_oeil`, `tears_oeil` |
| `epine` | `Pierce(1)` | Uncommon | 47 | `item_epine`, `tears_epine` |
| `ricochet` | `Bounce(2)` | Common | 66 | `item_ricochet`, `tears_ricochet` |
| `fantome` | `Spectral` | Rare | 69 | `item_fantome`, `tears_fantome` |
| `eclat` | `Split(3)` | Rare | 62 | `item_eclat`, `tears_eclat` |
| `satellite` | `Orbit` + `Lifetime(240)` | Rare | 77 | `item_satellite`, `tears_satellite` |
| `venin` | `Chance` 50 % (+12,5 %/`Luck`) poison | Uncommon | 28 | `item_venin`, `tears_venin` |
| `braise` | brûlure à l'impact | Uncommon | 103 | `item_braise`, `tears_braise` |
| `givre` | `Chance` 30 % gel | Uncommon | 59 | `item_givre`, `tears_givre` |
| `epouvante` | `Chance` 30 % peur | Uncommon | 71 | `item_epouvante`, `tears_epouvante` |
| `glu` | lenteur à l'impact | Common | 7 | `item_glu`, `tears_glu` |

Graines : `alacod-sim --game isaac --bots 1 --profiles explorateur --shopping --seeds N..N --floors sous_sol --until-floor 2
--max-frames 12000 --log`, la ligne `game::loot: … slot_assign pool=tresor floor=0 … item=<id>` (balayage 1..400 de M3-T11d1, plan à vingt gabarits et boutique : l'objet est
assigné à la génération de l'étage, `--max-frames 6` suffit ; première graine qui donne l'objet au trésor du premier étage, ouvert). Les seuils des `tears:` sont mesurés avec témoin
(commentaire de chaque `test:`) ; `tears_longue_vue` utilise `larmes_courtes`, `tears_goutte` une cible posée à 22 unités de la
ligne de tir (touchée seulement par la larme ×1,5).

**Traces** : le passage en `Infinite` change l'état de l'arme dès f0 pour tout scénario `isaac` qui a le personnage ; le pool
`tresor` change l'objet des coffres de trésor (preuve §5 au rapport `m3-t11a-larmes.md`). Les autres jeux ne bougent pas.

Dettes : `Chance` dépend du net id des larmes (un changement de cadence déplace
les tirages : les seuils de `venin`, `givre`, `epouvante` sont « au moins un statut à f300 ») ; pas de sprite d'objet.

### 76.2 T11b : effets, familiers, actifs (`m3-t11b-effets`)

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`) ; aucun code moteur. Tout passe par des vocabulaires déjà livrés :
effets v3 (§67), familiers (§71), cœurs d'âme (§68), pools à épuisement (§73), générateur v3 (§78).

- **Dix objets**, tous avec `test:` ; poids par rareté 4 / 2 / 1 dans leur pool (`exhaust: true`, aucun `exclude_held`).
  Pool `tresor` (`loot/tresor.ron`, 23 → 31 entrées) : `frere`, `orbe`, `nuee`, `crocs`, `pouls`, `rancune_vive`, `reliquaire`,
  `calice_ame`. Pool `boss` (`loot/boss.ron`, 3 → 5 entrées ; les trois passifs de T8, peu communs, passent de poids 1 à 2) : `sang_chaud`,
  `calice_ardent`.
- **Familiers** (`characters/`, fiche de §71 : équipe `Players`, `stationary`, collider 12 x 12, immunisés) : `frere` (`Follow(gap: 28)` +
  `MirrorShot(every: 12)` : les larmes du porteur, son arme et ses objets, Q13) ; `orbe` (`Orbit(radius: 36, frames_per_turn: 120)`
  + `BlockBullets`). Lab : `cible_mordante` (`test_only`, immobile, 5 000 PV, `counts_hits`, mêlée au contact à 40 : le générateur retrouve ses
  cibles **par position**, une cible mobile n'a pas de compteur dans la sonde).
- **Synergie** `synergies/garde_rapprochee.ron` (`requires: ["frere", "orbe"]`, deux objets du pool `tresor`, `Damage` +20 %) ; dossier `synergies`
  déclaré dans `game.ron`. Preuve : scénario `isaac_synergy_garde` (frère ramassé f10 : dégâts 1, un familier ; orbe f40 : deux familiers, dégâts 1,2).
- **Cœurs d'âme** : `coeur_ame` est déjà dans le pool de récompense `loot/salle.ron` (T3, poids 1) ; `reliquaire` et `calice_ame` en donnent aussi
  (`GainSoul`). `salle.ron` n'a pas changé (aucune trace de plus ne bouge).

| Objet | Effet | Pool (rareté, poids) | Graine `alacod-sim` | Scénarios |
|---|---|---|---|---|
| `frere` | familier `Follow` + `MirrorShot` | tresor (Common, 4) | 18 (coffre) | `item_frere`, `familiar_frere` (+ `_sans_objet` : ≥ 6 coups contre 5), `isaac_synergy_garde` |
| `orbe` | familier `Orbit` + `BlockBullets` | tresor (Uncommon, 2) | 13 (coffre) | `item_orbe`, `familiar_orbe` (`FamiliarDistance` 30–40) |
| `nuee` | trois `orbe` (`per_copy: 3`) | tresor (Rare, 1) | 5 | `item_nuee`, `familiar_nuee` (rangs 0 et 1 à 60–64) |
| `crocs` | `OnHit` : `Scaled(PerCopy) ApplyStatus poison` | tresor (Common, 4) | 14 | `item_crocs`, `tears_crocs` (poison / témoin sans) |
| `pouls` | `Tick(90)` : `TimedModifier` `MoveSpeed` +30 % 60 frames | tresor (Common, 4) | 3 | `item_pouls` (`MoveSpeed` 120 à f60, 156,00037 à f100) |
| `rancune_vive` | `OnDamageTaken` : `ApplyStatus peur` sur l'auteur du coup | tresor (Uncommon, 2) | 51 | `item_rancune_vive`, `tears_rancune_vive` (`cible_mordante` : peur / témoin sans) |
| `reliquaire` | `OnPickup` filtré `item: Some("reliquaire")` : `GainSoul(4)` (sans filtre, chaque ramassage suivant rendrait 4 points : relecture) | tresor (Common, 4) | 10 | `item_reliquaire`, `isaac_reliquaire` (ramassé au sol, puis un autre objet : rien de plus) |
| `calice_ame` | actif `Active(Damage(30))`, `OnUse` : `GainSoul(2)` | tresor (Rare, 1) | 2 | `item_calice_ame` (charge, usage, `ExtraHealth` 2) |
| `sang_chaud` | `OnKill` : `TimedModifier` `FireRate` +50 % 300 frames | boss (Uncommon, 2) | 8 (butin du boss, `slot_assign pool=boss`) | `item_sang_chaud`, `tears_sang_chaud` (`cible_fragile` tuée : `FireRate` 1,5 / témoin 1) |
| `calice_ardent` | actif `Active(Rooms(2))`, `OnUse` : `Damage` +100 % 300 frames | boss (Uncommon, 2) | 4 (butin du boss) | `item_calice_ardent` (`Damage` 2 après l'usage) |

Graines : `alacod-sim --game isaac --bots 1 --profiles explorateur --shopping --seeds N..N --floors sous_sol --until-floor 2 --max-frames 12000
--log` (M3-T11d1 : balayage 1..400 de la ligne `slot_assign pool=tresor` ou `pool=boss` du premier étage, `--max-frames 6` ; les anciennes graines de T11b n'étaient plus justes : le plan et le pool `boutique` ont changé). Les 10 objets sortent.

**Limites de preuve.** Un objet **donné au départ** (`PlayerScript::items`) ne déclenche pas `OnPickup` : `reliquaire` se prouve par un scénario
écrit (`isaac_reliquaire`, objet posé sous le joueur) et son `test:` ne génère que la tenue ; l'attente `ExtraHealth` n'y est donc pas. `Stat` d'une
vitesse est en unités/s (`MoveSpeed` 120), pas en multiplicateur : le seuil de `pouls` est une valeur mesurée.
Lint : `make lint GAME=isaac` perd les avertissements « aucun objet à effets » (trésor, boss), « aucune synergie atteignable » et « familiers : 0 sur 2 »
(la boutique est faite par M3-T11d1, §76.4).

**Traces** : le pool `tresor` change les coffres de trésor et le pool `boss` le butin du boss (tirage à la génération de l'étage : l'objet tiré dépend
de la taille du pool) ; preuve §5 au rapport `m3-t11b-effets.md`. Les autres jeux ne bougent pas.

Dettes : pas de transformation (Q10 non retenue) ; la synergie n'est jouée qu'en scénario (le bot ne rouvre pas le trésor du
second étage, deux objets du même pool ne sortent pas dans une même partie du bot) ; pas de sprite d'objet ni de familier propre (placeholders).

### 76.3 T11c : ennemis, champions, boss (`m3-t11c-ennemis`)

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`) ; aucun code moteur. Vocabulaires déjà livrés : behaviors (§22), vue directe
D66 (`sight_range`, §22), variantes v2 (§72), boss à phases et renforts (§37, §45), générateur v3 (§78).

- **Sept ennemis** (`characters/`, un `test:` chacun, scénarios `enemy_<id>_still`/`_moving`) : `suiveur` et `mouche` (T0f), et cinq nouveaux.
  `rampant` (`Charge(telegraph: 30)` + `Chase` Ground, 6 PV, 1 PV de dégât), `spectre` (`Chase` profil `Phasing` : ignore fenêtres, barricades, eau et fosses, **pas les murs ni les rochers** (`navigation.rs`, `Phasing` ignore tout sauf `Wall`) ; 8 PV,
  contact 1 PV, aucun `sight_range` : son `aggro_range` de 500 couvre toute salle de 240 unités), `cracheur` (`Shoot` du pattern `crachat` + `Chase` Ground, 6 PV,
  `aggro_range` 130, `sight_range` 360), `tourelle_croix` (immobile, `croix` : deux couronnes de quatre boules, `aggro_range` 100, `sight_range` 260,
  12 PV), `bourdon` (`Chase` profil `Flying` + `Shoot` du pattern `rafale`, 6 PV, `aggro_range` 140, `sight_range` 360). **`sight_range`
  cohérent** : un tireur sans portée de flair (`aggro_range` bas) mais à vue longue ne cible qu'en ligne de vue dégagée (un coin le met hors de
  portée) ; un ennemi qui flaire toute la salle (`spectre`, `aggro_range` 500) n'en a pas. Dans `isaac`, aucun obstacle franchissable par `Phasing` n'est posé : le `spectre` se comporte comme un `Chase` Ground (dette moteur/contenu : un vrai fantôme à travers rochers exigerait un obstacle `Wall` traversable).
- **Patterns** (`patterns/`) : `crachat` (télégraphe 15, un tir visé), `croix` (deux couronnes de quatre, 25 frames d'écart), `rafale` (trois tirs
  visés, un par 8 frames), `couronne_unique` (une couronne de huit boules, `every: 0` : le motif de mort d'un champion), `spirale` (deux bras,
  six salves, 60 frames). Projectiles de l'`arsenal` : `crachat` (dégât 1 : un demi-cœur d'isaac, vitesse 110) en plus de `plomb` et `boule`.
- **Trois ennemis à champions** (`variants:`, `chance` en expression par étage, `tint`, un effet de mort chacun, tous dans le gabarit de combat) :
  `suiveur` → `explosif` (`0,1 + floor × 0,15`, `Explode` rayon 44, dégât 2, `destroys: false` : la salle secrète reste à la bombe) ;
  `mouche` → `essaim` (`0,12 + floor × 0,1`, `SpawnCharacters` : deux `mouche` ordinaires) ; `tourelle_croix` → `salve` (`0,15 + floor × 0,12`,
  `SpawnPattern` `couronne_unique`). Générateur v3 : `champions: [(variant, kill: true, weapon: Some("larmes"))]` → `champion_suiveur_explosif`,
  `champion_mouche_essaim`, `champion_tourelle_croix_salve` (la sonde relève la mort et attend l'explosion, les deux mouches nées, la couronne).
- **Placement** (aucun gabarit ajouté, `docs/taches/rapports/m3-t11c-ennemis.make_placement.py`) : le gabarit `Combat` garde `suiveur` et `mouche`
  et gagne `tourelle_croix` (13, 7) : trois ennemis par salle de combat, 16 ennemis (au lieu de 11) à l'étage de la graine 1. **Le reste vient du boss**.
- **Boss `gardien`** (3 phases, `boss_drop` du pool `boss` inchangé : `rooms/boss.ron`) : phase 0 (≥ 60 %) éventail ; phase 1 (< 60 %) `rampant` et
  `cracheur` en renfort (`SpawnEnemies`) + couronne ; phase 2 (< 25 %) `spectre` et `bourdon` en renfort + spirale (télégraphe de 10 frames).
  Aucune vague d'arène : `isaac` n'a pas de type de salle `arene` (gabarit : T11d2 ; la boutique, les rochers et les piques de T11d1 n'y changent rien).

| Ennemi / variante | Comportement | Gabarit / vague | Graine `alacod-sim` | Scénario généré |
|---|---|---|---|---|
| `suiveur` | `Chase` Ground, contact | `Combat` | 2 (salle verrouillée f57) | `enemy_suiveur_still`/`_moving` |
| `suiveur` / `explosif` | explosion à la mort | `Combat` | 1730 | `champion_suiveur_explosif` |
| `mouche` | volante, contact | `Combat` | 2 | `enemy_mouche_*` |
| `mouche` / `essaim` | deux mouches à la mort | `Combat` | 1730 | `champion_mouche_essaim` |
| `tourelle_croix` | immobile, `croix`, vue directe | `Combat` | 2 | `enemy_tourelle_croix_*` |
| `tourelle_croix` / `salve` | couronne à la mort | `Combat` | 1730 | `champion_tourelle_croix_salve` |
| `rampant` | charge | boss, phase 1 | 2 (renfort f2370) | `enemy_rampant_*` |
| `cracheur` | tireur, vue directe | boss, phase 1 | 2 | `enemy_cracheur_*` |
| `spectre` | `Phasing` | boss, phase 2 | 2 | `enemy_spectre_*` |
| `bourdon` | volant, tireur | boss, phase 2 | 2 | `enemy_bourdon_*` |
| `gardien` | 3 phases, renforts, spirale | `Boss` | 2 | `boss_gardien` |

**Tirage des variantes : D88 fermée** (2026-10-17, `docs/taches/rapports/dette-d88-tirage-variantes.md`). Le tirage (`variant.rs`) était `fnv1a("variants") ^ graine ^ net id` suivi d'un seul
pas de congruentiel : quasi linéaire (0,0004 par unité de graine) et corrélé par étage (89 % des graines sans aucun champion, 9 % avec tous ; la graine par défaut `123456` n'en
donnait aucun au premier étage ; seule la graine 1730 atteignait les trois variantes). Il hache maintenant `variants:<graine>:<net id>` (§25). Atteignabilité mesurée :
`alacod-sim --game isaac --bots 1 --profiles explorateur --seeds 1..50 --floors sous_sol --until-floor 2 --max-frames 12000` : variantes rencontrées sur **47 graines sur 50** (94 %,
259 au total, 11 au plus sur une graine ; absentes des graines 3, 29 et 42), graine par défaut comprise (colonne `variantes` du rapport par graine et ligne de synthèse de `alacod-sim`,
D88). Le bot `explorateur` en subit les conséquences : 10 victoires, 37 morts, 3 soft-locks (graines 9, 42, 43) contre 17, 30 et 3 avant D88 (équilibrage : hors boucle de développement, décision de William du 2026-10-18).

**Équilibrage** (campagne courte, `explorateur`, 50 graines `--floors sous_sol --until-floor 2 --max-frames 12000`, §76.3 du rapport) : l'`explorateur` meurt
dès qu'une salle de combat compte quatre ennemis (essais : 16 morts sur 16 à quatre ennemis, 5 types ; cracheur, rampant ou tourelle seuls : 9 à 15 morts
sur 16 avec un boss à renforts doubles), d'où les trois ennemis par salle et les renforts du boss. Mesure finale : 30 morts, 17 victoires, 3 soft-locks
(9 et 43 existent sur `main` à l'identique, 42 est de la même famille : plafond sans progrès, ennemis hors de vue, aucun dégât). Les 5 scénarios
`grid_floor_seed_1`/`_123`/`_23`, `grid_post_boss`, `isaac_floor_seed_1` (chronologie du boss) ont leurs attentes recalées (16 ennemis, boss verrouillé vers f2021 au lieu de f1340 pour la graine 1).

Lint : `make lint GAME=isaac` perd l'avertissement « aucun ennemi des salles de la partie ordinaire ne porte `variants` » (§72 : dette fermée).

**Traces** : un ennemi de plus par salle de combat décale les net ids de tout ce qui suit (preuve §5 au rapport `m3-t11c-ennemis.md` : `FloorState.
enemies_placed` 11 → 16, 7 → 10 ; seul changement : les cinq `tourelle_croix`) ; `generated/isaac/weapon_arsenal.trace` bouge aussi (entrée `crachat` ajoutée à la table de projectiles de l'`arsenal`). Les autres jeux ne bougent pas.

Dettes : pas de gabarit de combat par ennemi (neuf `Combat…` de M3-T11d1, §76.4, mêlent six des sept ennemis : `spectre` n'est dans aucun gabarit) ; pas de salle à vagues ; tirage des variantes groupé pour les petites graines ; le soft-lock
de l'`explorateur` (graines 9, 42, 43) relève de T14 ; D88 (tirage des variantes) fermée le 2026-10-17 ; écart de périmètre : 7 ennemis et 1 boss livrés sur 10 et 2 (le reste en T11d2 (volume Q1), les gabarits variés sont faits par T11d1 ; 60 % de morts et 34 % de victoires à 1 bot, sous la cible Q14 : l'équilibrage n'est plus un objectif (décision de William du 2026-10-18, chiffres au rapport de T11d1).

### 76.4 T11d1 : structure de la partie ordinaire (`m3-t11d1-gabarits`)

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`) ; aucun code moteur : un test de `map_ldtk` ajusté (gabarits, borne de temps) et un
diagnostic ignoré du harnais (`floor_probe`). Décision de William du 2026-10-18 : T11d est **coupé en deux** ; T11d1 (ici) fait la structure de la partie ordinaire,
**T11d2** (autre branche) le volume de Q1 (objets 49 → 100, ennemis 7 → 10, boss 1 → 2). L'équilibrage n'est pas un objectif : victoires et morts sont
rapportés, aucun réglage n'a visé une fourchette.

**Vingt gabarits** (`maps/salles.ldtk`, `docs/taches/rapports/m3-t11d1-gabarits.make_salles.py`, qui reprend les dix de T5 octet pour octet et vérifie les dix
autres : ouvertures reliées, aucun obstacle sur une ouverture ou devant elle). Neuf gabarits de combat (au lieu d'un), trois de boss (dont le grand), trois de
trésor, une secrète, un diable, une **boutique**, un départ, un transit :

| Gabarit | Cases | Contenu |
|---|---|---|
| `Combat` | 1×1 | inchangé (T0f, T11c) : `suiveur`, `mouche`, `tourelle_croix` |
| `CombatRochers` | 1×1 | un amas de rochers (trois cases en L) dans chaque angle ; `suiveur`, `mouche`, `tourelle_croix` au centre |
| `CombatButin` | 1×1 | deux angles de rochers dont le coin est un **rocher à butin** ; `suiveur`, `mouche`, `cracheur` |
| `CombatPiques` | 1×1 | deux angles de **piques** (nord-ouest, sud-est) ; `suiveur`, `suiveur`, `bourdon` |
| `CombatMixte` | 1×1 | rochers (un amas plein, un à butin), piques dans les deux autres angles ; `suiveur`, `rampant`, `mouche` |
| `CombatCroix` | 1×1 | deux angles de rochers ; `tourelle_croix`, `cracheur`, `mouche` |
| `CombatLong` | 2×1 | deux angles de rochers à butin, deux de piques ; `suiveur` ×2, `mouche`, `cracheur` |
| `CombatHaut` | 1×2 | trois angles de rochers (deux à butin), un de piques ; `suiveur`, `mouche`, `rampant`, `tourelle_croix` |
| `CombatGrand` | 2×2 | quatre angles de rochers (deux à butin) ; `suiveur`, `mouche`, `cracheur`, `bourdon` |
| `Boss`, `BossGrand` | 1×1, 2×2 | inchangés : `gardien` au centre |
| `BossAbris` | 1×1 | `gardien` au centre, un amas de rochers dans chaque angle |
| `Tresor`, `TresorLong`, `TresorHaut` | 1×1, 2×1, 1×2 | inchangés |
| `Secrete`, `Diable`, `Transit`, `Depart` | 1×1 | inchangés |
| `Boutique` | 1×1 | vide (les présentoirs sont posés par le type de salle) |

Les neuf gabarits de combat sont tirés au hasard (un gabarit par pose, uniforme) : sur 300 graines de `sous_sol_un`, 139 à 183 salles chacun. **Rochers et piques
sont dans les angles** : le bot `explorateur` ne voit pas les rochers (ils n'ont pas le marqueur `Wall` que `bots::input` lit pour son maillage), marche en ligne
droite et reste collé à un rocher posé sur son chemin (premier essai, rochers au centre : étage sans progrès après 700 frames) ; le script refuse tout rocher à
moins d'une case (piques : trois quarts) d'une ligne droite entre deux ouvertures, une ouverture et un ennemi ou le centre. Casser un rocher n'est donc jamais
nécessaire pour traverser une salle : la bombe ouvre le butin et la salle secrète. Nouvelles couches du projet LDtk : `Rocks` (1 rocher, 2 rocher à butin, §62) et
`Surfaces` (5 piques) ; `surfaces/piques.ron` (valeur 5, 1 PV tous les 60 frames, dégâts `True` : l'âme les absorbe d'abord ; les `hit_iframes` de `pleureur`
recouvrent la période) ; dossier `surfaces` déclaré dans `game.ron`.

**Boutique** : type `boutique` (`rooms/boutique.ron` : jamais verrouillée, `shop` de trois présentoirs du pool `boutique`, prix en sous par rareté de
`economy.ron` : 15, 30, 50), exigé par la grammaire `sous_sol` (`required: {boss, tresor, secrete, boutique}` : cul-de-sac derrière une salle ordinaire, une par
étage ; `sous_sol_grand`, `_court`, `_transit` n'en ont pas). Pool `loot/boutique.ron`, 26 entrées, `exhaust: true` (Q5 : par run et par pool, un consommable compte
comme un objet vu) : les **treize objets de larmes** (chacun est aussi au trésor : deux exemplaires possibles sans pool qui les exclue, règle `Tears` du lint),
quatre passifs à effets (`crocs`, `pouls`, `rancune_vive`, `reliquaire`), les trois familiers, deux objets de stats (`encre`, `bottines`) et quatre consommables (`key`,
`bomb`, `coeur`, `coeur_ame`), poids 4, 2, 1 par rareté, 3, 3, 3, 2 pour les consommables ; jamais de `Currency` (lint). La boutique **vend en sous** les objets : les
sous viennent de la mort des ennemis (`shells_drop`, 2 à 5) et des récompenses de salle (`Currency(1)`). Récompenses de salle (`loot/salle.ron`, T3) et coffre secret
(`loot/secrete.ron`, T8) : **inchangés** (cœurs, sous, bombes, clés déjà cohérents avec §68 et §73).

**Curses dans la séquence par défaut** : `floors/sous_sol.ron` gagne `curses: Some((chance: "0.3", pool: ["darkness", "unknown", "blind", "labyrinth", "maze"]))` : une
curse sur 30 % des étages (graine de carte et étage, §74, les cinq selon leurs poids : `darkness` 3, `unknown` 2, `blind` 2, `maze` 2, `labyrinth` 1 ; 112 étages maudits sur 400 graines au premier étage). Les séquences de test (`sous_sol_un`, `_cle`, `_court`, `_grand`) n'en ont pas :
leurs scénarios ne bougent que par les gabarits. Conséquence propre à la séquence par défaut : `RunCurses::entered` s'écrit dès la frame 0, même sans curse tirée
(§74), donc toutes ses traces changent.

**Bombe de départ (D81, fermée)** : `pleureur.ron` gagne `bomb: Some((fuse_frames: 75, radius: "56.0", damage: "4.0"))` (tir ami `Always`, défaut de Q4 : la bombe
blesse son poseur, 4 PV sur 6 ; mèche de 1,25 s ; rayon 3,5 cases : l'explosion touche le bord de la porte secrète d'où l'on pose la bombe et casse les rochers voisins)
et `starting_consumables: {"key": 0, "bomb": 1}` (objet `bomb`, plafond 99). Comme Isaac, une bombe au départ. **Réserve pour T11d2 et T14** : `damage: "4.0"` enlève **deux cœurs** au poseur (sur trois), contre un seul dans Isaac ; valeur de départ, non réglée (équilibrage hors objectif).

**Lint** : `alacod lint games/isaac` passe de **24 avertissements à 0** (24 mesurés sur `main` avant la tâche : boutique absente et ses quatre règles, rochers et piques absents des
gabarits `combat`, aucune bombe jouable, salle secrète sans bombe, curses de la séquence par défaut, treize objets de larmes sans second pool). Aucun avertissement
d'atteignabilité, hors `test_only`, ne reste. `testbed` (115), `gungeon` (1) : inchangés ; `zombies`, `throne` : aucun.

**Scénarios** : recalés (graines, chronologies, comptes d'ennemis ; les plans de toutes les graines ont changé : `GgrsNetId`, gabarits tirés au hasard, boutique) :
`devil_room_chance` (cinq présentoirs : trois de la boutique, deux du diable), `devil_room_absent` (trois : la boutique seule, aucun du diable), `grid_floor_seed_1` (14 ennemis, boss f2245, victoire f2674), `_23` (boss 1×1
désormais, victoire f1485), `_123` (porte maintenant le **boss 2×2** de l'ancienne 23, victoire f1502), `grid_post_boss` (`Armor +100`, victoire f1727),
`grid_secret_room` (graine 25), `grid_secret_bomb` (graine 39, **bombe de départ**, sans `bomb:` ni `items:` du scénario : l'explosion f110 ouvre la salle,
le joueur sorti du rayon n'est pas blessé, il entre dans la secrète : f339), `grid_secret_no_bomb` (même graine, la bombe n'est pas posée : aucune explosion, la porte reste fermée, le joueur s'arrête contre le mur), `grid_treasure_key`/`_floor0` (graine 145, -1013904097 pour `sous_sol_cle`), `isaac_floor_seed_1` (curses `maze` puis `blind`,
boss f2500, « niveau 1 » f3220, boutique 3 puis 6 objets sortis), `isaac_hud_items` (source `curses` : `maze`). Neufs : `isaac_shop_buy` (graine 2 : le joueur à 60 sous achète
`reliquaire` à 15 sous, `ExtraHealth` 4), `isaac_bomb_rock` (graine 4 : la bombe de départ casse l'amas de rochers d'un `CombatButin`, 45 → 42 rochers, le rocher à butin
laisse une `ration`, le joueur hors du rayon n'est pas blessé), `isaac_spikes` (graine 10 : un demi-cœur dès f108, f168, f228, f288 sur l'amas de piques d'un `CombatPiques`) et son témoin `isaac_spikes_temoin` (même marche arrêtée avant l'amas : 6 PV jusqu'à f170, un ennemi arrive vers f172 : les deux premiers coups sont ceux des piques ; `HazardState.last_hit` du dump : 107, 167, 227, 287).
Diagnostic `floor_probe` (`ALACOD_FLOOR_PROBE=<séquence>:<de>-<à>`, `crates/scenario/tests/scenarios.rs`) : le plan d'un étage par graine (type, taille, voisines, rochers,
piques, ennemis, présentoirs) sans jouer la partie, pour choisir une graine. Test `map_ldtk::generation::isaac_etages_de_grille_sur_1000_graines` : `Combat…` (neuf
gabarits) compte comme salle ordinaire, une boutique par `sous_sol` (cul-de-sac derrière une salle ordinaire) ; `…_sous_trois_fois_grand_de_gungeon` : trois chronomètres ((a) clone + `from_map`, (p) planificateur, (b) génération entière), borne 3 sur (b), garde sur le nombre d'évaluations de candidats (moyenne sous 3 500, mesuré 2 396, 278 avec dix gabarits) :
cause de l'écart avec la borne d'origine de 2 (2,1 à 2,4 mesuré contre 0,8 avec dix gabarits) **dans le planificateur** : un gabarit tiré parmi les poses valides, neuf gabarits de combat au lieu d'un (D107).

**Traces** : toutes les traces `isaac` bougent (124 : 120 rebénies, 4 neuves) ; zombies, throne, gungeon, testbed **identiques** (suite complète de 653 scénarios).
Preuve §5, douze scénarios (`generated/isaac/{weapon_larmes, enemy_suiveur_still, tears_encre, champion_suiveur_explosif, familiar_frere, room_combat, boss_gardien}`,
`hit_iframes_isaac`, `isaac_hud_items`, `isaac_boots`, `grid_secret_no_bomb`, `devil_room_absent`), dumps `ALACOD_DUMP_TRACE` du même binaire avec `ALACOD_GAME_DIR` pointé
sur cinq états de `games/isaac` : (1) `main` ; (2) **bombe seule** (`pleureur.ron`, `bomb.ron`) ; (3) bombe + curses de la séquence par défaut ; (4) la branche ; (5) vingt gabarits **sans** boutique exigée (état intermédiaire entre 3 et 4).
`scripts/trace-diff.py … --ignore Inventory,BombCarrier` : 1 → 2 **identiques sur les douze** (`Inventory` gagne `bomb: 1`, `BombCarrier` est neutre : seules ces lignes
bougent) ; 1 → 4 identiques sur cinq scénarios en carte de test (`gabarit_armes` : aucune boutique, curse ni gabarit n'y entre). 2 → 3 : sur `hit_iframes_isaac`,
`isaac_hud_items`, `isaac_boots`, `room_combat` (séquence par défaut, graine 1) f0 : `RunCurses { floors: {0: "maze"}, entered: Some(0) }` (au lieu de `{}`, `None`) et
`Modifiers` du joueur gagne `MoveSpeed ×0,85` de source `curse:maze` ; identiques sur `boss_gardien`, `devil_room_absent`, `grid_secret_no_bomb` (séquences sans curse). 3 → 5 (séquence par défaut) dès f0 : le plan change. **Gabarits et boutique sont séparés par un cinquième état** (« vingt gabarits, boutique non exigée » : la grammaire sans `boutique` dans `required`, sur `hit_iframes_isaac`, `isaac_hud_items`, `isaac_boots`) : 3 → gabarits seuls : `FloorState.enemies_placed` 16 → 17, `GgrsNetIdFactory.counter` 148 → 193 (le plan change, `PoolState` aussi) ; gabarits → boutique exigée (la branche) : `enemies_placed` 17 → 14, `PoolState` gagne `boutique`. Chacun déplace le plan à lui seul (le planificateur tire un gabarit parmi neuf, puis la boutique prend une place de cul-de-sac).

**Atteignabilité** (tableau du §8.1 de `docs/taches.md`, une ligne par mécanique, avec les graines `alacod-sim --game isaac --floors sous_sol --until-floor 2 --shopping` (séquence par
défaut ; plan et butin de l'étage 0 lus à la génération, `--max-frames 6`, balayage 1..400) et le scénario qui la prouve à la main tant que le bot de T14 n'est pas sur `main`) :

| Mécanique | Graine (étage 1) | Scénario |
|---|---|---|
| Larmes modifiées, empilement | 1 : trésor `vent_arriere`, boutique `givre` ; deux exemplaires par deux pools : 71 (`epouvante`), 76 (`vent_arriere`), 95 (`oeil`) | `isaac_shop_buy`, `tears_*`, `tears_stack_two_copies` (testbed) |
| Effets v3 | trésor `crocs` 14, `pouls` 3 ; boutique `rancune_vive` 8 ; boss `sang_chaud` 8, `calice_ardent` 4 | `item_*`, `tears_crocs`, `tears_rancune_vive`, `tears_sang_chaud` |
| Familiers, orbitaux | 9 (trésor `nuee`, boutique `frere`), 13 (`orbe` aux deux), 18 (`frere` aux deux) | `isaac_synergy_garde`, `familiar_*` |
| Bombes | toute graine (bombe de départ) | `grid_secret_bomb`, `isaac_bomb_rock` |
| Rochers, rochers à butin | 4 : `CombatButin` au nord du départ | `isaac_bomb_rock` (45 → 42, une `ration`) |
| Piques | 10 : `CombatPiques` au nord du départ | `isaac_spikes`, `isaac_spikes_temoin` |
| Salle secrète | toute graine ; sud du départ : 39 ; est : 25 | `grid_secret_bomb`, `grid_secret_room`, `grid_secret_no_bomb` |
| Clés, trésor fermé | `key` en boutique : 1 ; clé de récompense : 14, 20, 22 ; second étage de toute partie | `grid_treasure_key`, `isaac_floor_seed_1` |
| Cœurs, sous, boutique | cœur de récompense 1, 3, 14 ; cœur d'âme 3, 23, 32 ; sous à chaque mort ; boutique : toute graine | `isaac_shop_buy`, `isaac_floor_seed_1` |
| Pools à épuisement | toute partie (trésor, boss 1 → 2, boutique 3 → 6 au second étage) | `isaac_floor_seed_1`, `devil_room_chance` |
| Marché du diable | 8, 13, 16 | `devil_room_chance`, `devil_deal_isaac` ; l'achat par le bot : T14 |
| Champions | 1 (`essaim`, `salve`), 4 (`explosif`) | `champion_*` |
| Curses | 1 (`maze` puis `blind`), 2 (`darkness`), 13 (`blind`), 19 (`labyrinth`), 42 (`unknown`) | `isaac_floor_seed_1`, `isaac_hud_items` |
| Accomplissements | deux parties d'affilée | `achievement_unlocks` (une run), test `crates/scenario/tests/achievements.rs` (deux runs enchaînées) |

**Campagne de contrôle** (bot actuel, séquence par défaut, `--shopping`, lots de 20 graines, `docs/taches/rapports/m3-t11d1-gabarits.campagne.sh`, résumé graine par graine :
`m3-t11d1-gabarits.campagne.tsv`) : **1 bot, graines 1 à 200 : 0 desync, 12 victoires, 169 morts, 19 « soft-locks »** (64 graines franchissent le premier étage) ; **2 bots, graines 1 à 200 :
0 desync, 72 victoires (36 %), 102 morts, 26 « soft-locks »** (141 franchissent le premier étage). Chaque « soft-lock » a été rejoué en scénario, **prolongé de 2 000 frames sans détecteur**
(`until_floor` absent), et classé :

| Famille | 1 bot | 2 bots | Preuve |
|---|---|---|---|
| Cœur à pleine vie, au sol | 5 (39, 63, 112, 122, 171) | 3 (63, 144, 152) | ne reprend pas, vivant ; `coeur` à 4 à 11 unités (D99) |
| Cœur à pleine vie, en boutique | 1 (149) | 2 (86, 149) | ne reprend pas ; `purchase_refused … shop_full` toutes les 30 frames (D99) |
| `gardien` intact | 1 (4) | 0 | ne reprend pas ; 48 unités du boss, visée à quatre directions (D99) |
| Réanimation interrompue par l'esquive | 0 | 1 (84) | ne reprend pas ; le coéquipier à terre meurt au saignement (f7076) puis l'autre (f7380) (D99) |
| Long trajet pris pour un soft-lock | 12 | 20 | **28 reprennent** (76 à 759 frames après l'arrêt : un ennemi meurt, une salle est nettoyée, ou l'étage change), **4 se résolvent par la défaite** du bot (1 bot 30, 35, 196 ; 2 bots 159, 95 à 337 frames après) : **aucune ne gèle** (D100) |

**Cause de 84 (2 bots, second étage, boss en phase 2).** Le joueur 0 tombe à f5276 (saignement jusqu'à f7075). Le joueur 1 est collé à lui (11 unités) et **tient `Interaction` pendant 119 frames, la lâche 31, tient 119, la lâche 31…**
(cycle de 150 frames = la répétition de l'éventail du boss) : la progression de la réanimation (`handle_revive_interaction`, 180 frames **consécutives**) atteint au plus 119 puis retombe à 0, 9 fois. Interactable visé :
le seul à portée, le joueur à terre (net id 147, `Revive`, prompt « Réanimer », `view.revive = (direction nulle, interact = vrai)` à chaque frame du relevé) ; aucun présentoir, cœur ou porte ne capte l'interaction (les entités
à moins de 85 unités sont le joueur à terre, un familier `orbe`, des balles et un mur). Ennemi le plus proche : le `gardien`, 177,7 unités, en vue, **au-delà de `REVIVE_SAFE_DISTANCE` (150)** : pas de « menace »
au sens de la réanimation. `Downed { since_frame: 5275, bleedout_at_frame: 7075 }`. Les 31 frames sans `Interaction` sont celles où `decide_prudent` exécute `crate::dodge::dodge` (un projectile du boss approche dans les
30 prochaines frames, direction d'esquive `(0, 1)`, le bot ne bouge pas : position constante) : la branche d'esquive précède la réanimation et remplace tout l'input. **Cause : le bot (D99), pas le contenu.**
**Cause de 86 (2 bots, premier étage).** Les deux joueurs (6 PV et 4 PV sur 6) sont collés au présentoir 0 de la boutique (un `coeur`, 2,2 et 2,8 unités) ; le joueur 0, à pleine vie, le vise et reçoit `shop_full` toutes
les 30 frames (103 refus en 3 000 frames) ; le joueur 1, qui pourrait l'acheter, ne l'obtient jamais. **Bot (D99) : cœur en boutique, deux joueurs.**

Les chiffres sont des informations, pas des objectifs (décision de William). La campagne est à refaire avec le bot de T14 (cœurs, bombe, clés, achats) : mêmes commandes.

**Intégration de T14 (D106, `m3-t14-bot-isaac` fusionné, §79).** Le bot de T14 pose des bombes par défaut dès qu'un joueur en porte une : la bombe de départ de `pleureur` les active dans tous les scénarios isaac à `explorateur`.
**Quinze traces rebénies** (quatorze, plus `bench_floor_isaac` de T15 à la fusion de `main`) (parmi elles, huit `bot_isaac_*` joués sur `isaac`, recalés sur les graines des vingt gabarits et la bombe de départ réelle, sans kit d'essai ; les deux `bot_isaac_rock*` de T14, joués sur la carte `testbed` dont le personnage n'a pas de bombe de départ, gardent leur kit et ne bougent pas ; `grid_floor_seed_1/23/123`, `grid_post_boss` à `Armor +200`, `isaac_floor_seed_1`, `achievement_unlocks`) ;
**preuve §5** : bombe portée avec les décisions de bombe du bot coupées (`bombs: (secret: false, rocks: false)`) = identique à l'état sans bombe sur neuf scénarios ; avec elles, la première frame qui diffère est le **trajet vers le mur secret**, 13 à 350 frames
**avant** la pose (le critère de T14 « première frame = première bombe posée » est trop strict). **Campagnes `s1`, `i2`, `i4` rejouées sur le contenu livré** (séquence par défaut, bombe de départ réelle, `--pickup-progress`, sans `--bombs`) : s1 **29 victoires, 171 morts**, i2 **115 victoires, 85 morts**,
i4 **20 victoires sur 20** ; **0 desync, 0 soft-lock** (420 graines) ; les 45 graines laissées en soft-lock par le bot d'avant T14 finissent toutes en victoire ou en mort. D106 et D100 fermées, D99 réduite à l'esquive qui coupe la réanimation. Atteignabilité du bot : mur secret à la bombe 187 graines sur 200 (s1),
porte à clé 51, achats en boutique 143 ; **aucune bombe de rocher en partie ordinaire** (rochers dans les angles : 45 poses du joueur (`bomb_place`) = 45 murs secrets ouverts sur 40 graines ; le compteur `bombe` du JSON compte aussi les bombes de champions ; rochers cassés par le souffle : 29 dans 12 graines) ; achat au diable : `Never` par défaut (Q14). Détail : rapport § 10, résumé `m3-t11d1-gabarits.campagne-t14.tsv`.

**Dettes** : D99 (la réanimation du bot est interrompue par l'esquive ; ses autres blocages de la première campagne, cœur à pleine vie, visée du `gardien`, rochers, sont fermés par T14), D100 fermée (détecteur : `--pickup-progress`), D107 (le planificateur de grille évalue ×8,6 candidats avec neuf gabarits de combat) ; le bot n'achète pas au diable et pose la bombe de départ contre un mur secret (T14, intégré, D106) ; la boutique n'exerce pas
`shop.cost: Currency(n)` (les prix passent par `economy.ron`) ; D93 reste ouverte (quatre scénarios gagnent par armure).

### 76.5 T11d2 : volume de Q1 (`m3-t11d2-volume`)

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`) ; aucun code Rust. Décision de William du 2026-10-18 : le volume de Q1 reste le critère de sortie de M3 ;
**pas d'équilibrage** (victoires et morts rapportés, rien n'est réglé pour une fourchette). Rapport : `docs/taches/rapports/m3-t11d2-volume.md`.

| Q1 | Avant | Après | Cible |
|---|---|---|---|
| objets | 49 | **100** | 100 |
| objets qui modifient les larmes (tag `larmes` ; dont 23 à `projectile`) | 19 | **37** | ≥ 30 |
| familiers ou orbitaux | 3 | **9** | 8 |
| actifs | 2 | **11** | 10 |
| ennemis | 7 | **10** (`araignee`, `follet`, `arroseur`) | 10 |
| ennemis à champions | 3 | 3 (`suiveur`, `mouche`, `tourelle_croix`, §76.3) | 3 |
| boss à phases | 1 | **2** (`colosse`) | 2 |
| gabarits (`salles.ldtk`) | 20 | 23 (20 + trois de boss `boss_profond`) | 20 |

**Objets** (51, `objets/*.ron`, un `test:` chacun sauf les trois consommables, que `ItemHeld` refuse : preuve par `isaac_consommables_lot`, comme `coeur` et `ration` ; les quatorze passifs de stats de T8 et T10 qui n'en avaient pas en ont maintenant un, `item_*`).
Poids de pool 4 / 2 / 1 par rareté (consommables : 3 en boutique pour `pain`, 2 pour `eclat_ame` et `festin`, 3 / 2 / 1 en salle secrète). Le **tableau objet → pools → scénario** est au rapport (§ 2). Répartition :

| Pool | Entrées avant | Après | Nouveaux objets |
|---|---|---|---|
| `tresor` | 33 | 71 | 38 (larmes 13, familiers 5, actifs 6, effets 11, stats 3) |
| `boutique` | 26 | 55 | 29 (les douze objets de larmes du trésor, familiers 4, actifs 3, effets 4, stats 3, consommables 3) |
| `boss` | 6 | 11 | `coeur_de_pierre`, `egide`, `bombe_fiole`, `escorte`, `cendre_froide` |
| `diable` | 4 | 9 | `grappe`, `acide`, `philtre`, `epines_de_chair`, `grimoire` |
| `secrete` (sans épuisement) | 5 | 10 | `pain`, `eclat_ame`, `festin`, `pot_de_miel`, `fer_a_cheval` |
| `salle` (récompense) | 7 | 8 | `pain` |

Les objets de larmes du trésor sont aussi en boutique (règle `Tears` du lint : deux exemplaires possibles) ; `cendre_froide` (boss), `grappe`, `acide`, `philtre` (diable) n'ont qu'un pool d'épuisement et n'y sont pas soumis.
Trois statuts nouveaux (`statuses/` : `etourdi` Stun 60 frames, `charme` Charm 200, `saignement` Poison 1 PV / 15 frames pendant 180), trois ennemis, un boss, deux patterns (`gerbe`, `tourbillon`).

**Mécaniques manquantes (dettes, pas de code ici)** : `SpawnPattern` et `Emit`/`OnEvent` sont refusés à `OnUse` d'un objet : `nova` est un `Explode` (rayon 100) et `hydre` pose directement ses deux actions (D112) ; l'`Explode` d'un objet ou d'un effet a un `friendly_fire` (D113 : `Never` pour `nova`, `bombe_fiole` et `epines_de_chair`, qui n'ont plus de `GainShield` de contournement ; un coéquipier n'est plus blessé, `isaac_nova_coop`) ; reste de D113 : la charge `Damage` compte tous les dégâts dont le porteur est la source, ses propres explosions comprises : `nova` se charge donc par salles (`Active(Rooms(2))`, revue) ; aucune attente `Shield`, aucun départ blessé, `familiar:` sans input ni balle hostile, pas de `test:` pour un consommable : scénarios écrits à la main (D114) ;
le second boss passe par un type de salle propre `boss_profond` (D115, réécrite). Objets sans sprite propre (inchangé, T17).

**Ennemis** (`characters/`, un `test:` chacun, `enemy_<id>_still/_moving`) : `araignee` (sol, 4 PV, 66 unités/s, contact), `follet` (volant `Flying`, 5 PV, `Charge` télégraphe 25 + contact), `arroseur` (tireur au sol, 8 PV, `KeepDistance` 80 à 140, `gerbe` : télégraphe 18 puis cinq crachats en éventail exact sur 1 rad, `sight_range` 300). Placés **sans changer le nombre d'ennemis par salle**, en remplacement d'une mouche ou d'un suiveur :
`CombatRochers` (follet), `CombatButin` (araignee), `CombatPiques` (arroseur), `CombatMixte` (araignee), `CombatCroix` (follet), `CombatLong` (arroseur), `CombatHaut` (araignee), `CombatGrand` (follet) ; `Combat` (inchangé) et `spectre` (renfort seulement) comme avant.
Script : `docs/taches/rapports/m3-t11d2-volume.make_salles.py` (reprend celui de T11d1, mêmes vérifications).

**Boss `colosse`** (`characters/colosse.ron`, 44 PV contre 40, immobile, trois phases, type de salle `boss_profond`, butin du pool `boss` de `rooms/boss_profond.ron`) : phase 0 (≥ 65 %) une gerbe toutes les 150 frames ; phase 1 (< 65 %) deux `araignee` en renfort, croix à f0 et gerbe à f75 ; phase 2 (< 30 %) un `follet` et un `arroseur` en renfort, `tourbillon` (spirale de deux bras, 48 frames) précédé d'un télégraphe de 10 frames.
Premier jet (50 PV, trois araignées, deux follets, spirale à trois bras) : le bot `fonceur` du scénario généré mourait en phase 2 ; allégé jusqu'à ce que `boss_colosse` le batte (victoire f449) ; aucune fourchette visée.
**Rotation (justifiée)** : le `colosse` est le boss du **second étage** de la séquence par défaut (`floors/sous_sol.ron` : `floor:sous_sol` puis `floor:sous_sol_profond`), le `gardien` celui du premier : un boss par étage (comme Isaac), plutôt que le hasard, parce que (1) la difficulté monte avec l'étage (44 PV contre 40, renforts plus nombreux ; le butin vient du même pool `boss`, épuisé par run), (2) toutes les séquences de test (`sous_sol_un`, `_court`, `_cle`, `_grand`, `_maudit`) et leurs traces
gardent le `gardien` : seuls les scénarios qui traversent le premier boss de la séquence par défaut voient le second, (3) un tirage au hasard parmi des gabarits de boss de même type ferait du boss d'un scénario de grille une loterie de graine. Mécanisme (revue) : `FloorGrammar::boss` existait déjà et le planificateur choisit les gabarits de boss par ce type : `floor_grammars/sous_sol_profond.ron` (et `_court`) déclarent `boss: "boss_profond"` et `required: {"boss_profond": 1, …}`, trois gabarits `BossProfond`, `BossProfondGrand`, `BossProfondAbris` (type `boss_profond`, `colosse`) s'ajoutent à `salles.ldtk` (23 gabarits ; les vingt premiers inchangés), `rooms/boss_profond.ron` (verrouille, butin du pool `boss`). `sous_sol_profond_court` (départ, combat, boss) sert le scénario `boss_colosse`.
**Code (petit, hors simulation)** : la minicarte ne reconnaît plus le boss par le nom `boss` : `RoomKindDef::boss` (RON `boss: true`, faux par défaut, hors hash), `RoomKindTable::is_boss`, `HudRoomBounds::boss` rempli par `map_ldtk::game::rooms` ; les `boss.ron` de `gungeon`, `testbed` (et `boss_butin`) et d'`isaac` posent `boss: true` ; `room_icons` d'`isaac` gagne `boss_profond`. Le gabarit `BossFight` du générateur lit `BossFightTest::room` (type de la salle du boss, `boss` par défaut) pour l'attente « salle <type> nettoyée » (`colosse` : `room: Some("boss_profond")`). D115 (réduite à un lint).

**Scénarios** recalés (graines, chronologies, objets de butin ou de présentoir) : `grid_floor_seed_123`, `_23`, `grid_post_boss` (butin `bombe_fiole`), `isaac_floor_seed_1`, `bot_isaac_floor` (le bot meurt au premier combat du second étage sans `Armor` : `Armor +50`, D93, il bat ensuite le `colosse` f5442), `bot_isaac_two`, `isaac_shop_buy`, `devil_room_chance`, `isaac_spikes_temoin` (marche raccourcie : l'`arroseur` du gabarit `CombatPiques` arrive plus tôt) ;
écrits : `isaac_fiole_vive`, `isaac_egide` (+ témoin), `isaac_bombe_fiole`, `isaac_nova`, `isaac_sablier` (+ `isaac_sablier_temoin`, partagé avec `isaac_cuirasse` et `isaac_vengeance`), `isaac_bol_de_soupe` (+ témoin), `isaac_bouclier_de_fortune`, `isaac_familiers_file`, `isaac_consommables_lot`, `isaac_ombre_bloque` (+ témoin).

**Traces** : 40 traces `isaac` bougent (écrites à la main ou générées), 134 sont neuves (118 générées, 16 écrites à la main) ; zombies, throne, gungeon, testbed identiques. Preuve §5 (dumps `ALACOD_DUMP_TRACE` du même binaire, `ALACOD_GAME_DIR` sur quatre états de `games/isaac`, `scripts/trace-diff.py … --ignore Inventory,BombCarrier`,
six scénarios `hit_iframes_isaac`, `isaac_boots`, `grid_secret_no_bomb`, `devil_room_absent`, `generated/isaac/{boss_gardien, weapon_larmes}`) : (1) `main` → **pools et objets** : première frame qui diffère = 0, `PoolState` et le contenu des présentoirs seulement (`weapon_larmes` identique) ; (2) → **ennemis et gabarits** : f0, les ennemis de salle (`suiveur`/`mouche` → `araignee`/`follet`/`arroseur`) ;
(3) → **boss, grammaire profonde, séquence** : **identiques** sur les six (aucun n'atteint le second étage), le changement ne se voit que dans les scénarios qui franchissent le premier boss de la séquence par défaut (`isaac_floor_seed_1`, `bot_isaac_floor`) ; les `grid_floor_seed_*` jouent `sous_sol_un` (un étage, `gardien`) et ne bougent que par les pools et les ennemis.
**Revue de livraison, traces rebénies (5, cause)** : `item_nova`, `isaac_nova` (`nova` : `Active(Rooms(2))` au lieu de `Damage(60)`), `item_grimoire` (`grimoire` : `Rooms(3)`, `GainSoul(1)` + `Luck`), `generated/isaac/boss_colosse` et `bot_isaac_floor` (type de salle `boss_profond` : seule la **chaîne** `kind` de `RoomChanged` change, `"boss"` → `"boss_profond"`, première frame f245 et f5273 ; plan, ids et butin identiques ; le renommage des sept objets et le retrait de `larmes` de `fer_a_cheval` ne bougent aucune trace).
Preuve §5 (dumps du même binaire, `isaac_floor_seed_1`, S2 = pools, objets, ennemis, `salles.ldtk` d'avant les gabarits de boss, séquence à deux `sous_sol` ; contre la branche) : **première frame qui diffère = 3752**, la génération du second étage ; **une seule entité change : le boss net id 584, `gardien` → `colosse`** ; `isaac_floor_seed_1` (4 000 frames) n'atteint pas la salle de ce boss.

**Atteignabilité** : `alacod lint games/isaac` : 0 erreur, 0 avertissement ; chaque objet est dans un pool tiré par la séquence par défaut (tableau du rapport). **Campagnes de T14** (`m3-t11d2-volume.campagne-t14.sh`, 200 graines à 1 et 2 bots, séquence par défaut, `--pickup-progress`, victoires en information) : s1 **24 victoires, 175 morts, 0 desync**, un « soft-lock » de détecteur (graine 63, long trajet D100 : la partie reprend et se résout, boss tué f2738) ; i2 **95 victoires, 105 morts, 0 soft-lock, 0 desync** ; murs secrets ouverts à la bombe 182 et 195 graines, portes à clé 43 et 128, achats 130 et 180 graines. Détail : rapport § 6, `m3-t11d2-volume.campagne-t14.tsv`.

## 77. Lint des kinds M3 et atteignabilité des mécaniques (M3-T12, voie V2)

Lint seulement : `content` (plus la déclaration `mechanics` de `games/isaac/assets/game.ron`, une ligne de
`crates/game/src/run_state.rs` dans un test pour le littéral du manifeste) ; aucune simulation touchée, **aucune
trace ne bouge**. Code : `crates/content/src/mechanics.rs` (avertissements), `reach.rs` (sources d'objets et d'armes,
stats), `lint.rs` (trois règles de plus), tests `crates/content/tests/m3_t12_fixtures.rs` sur la fixture
`mechanics_ok`.

### Audit des règles de lint des kinds M3

Une ligne par mécanique de M3 : ce que le lint refuse, la fixture qui le prouve. Les règles de T0a à T7 existaient ;
les lignes marquées **T12** sont les trous trouvés par l'audit.

| Mécanique | Règle (niveau) | Fixture / test |
|---|---|---|
| Munitions infinies (§60) | `ammo_type` absent alors qu'un mode consomme (`Parse`), donné alors que tous les modes sont `Infinite` (`OutOfRange`) | `weapon_ammo_type_missing`, `weapon_infinite_with_ammo_type` |
| Larmes, `ProjectileMods` (§60, §66) | formes et plages d'un objet ou d'une synergie (`Homing`, `Bounce` répété, `Split.count`, `Orbit.radius`…), `child` et `on_expire: Spawn` sans table de projectiles (`Unsupported`), `Chance` hors [0, 1] | `projectile_split_out_of_range`, `item_projectile_out_of_range`, `item_projectile_no_table`, `item_projectile_ok`, `projectile_chance_out_of_range` |
| Stats de larmes, `Luck` (§60) | liste fermée `stat_reader` : **T12** test `every_named_stat_has_a_reader` (liste des stats nommées tenue à la main ; son `match` sans `_` fait échouer la compilation à l'ajout d'une variante de `StatId`, ce qui rappelle d'y décider si elle a un lecteur) ; **T12** `stat_uses` lit aussi les modificateurs de curses et les actions d'entrée et de frise des phases de boss (un `Modifier` y est accepté) | `stat_unread`, `unread_stats_are_found_in_curses_and_boss_phases`, test `reach::tests` |
| Bombes (§62) | `fuse_frames > 0`, `radius > 0`, `damage >= 0` (`OutOfRange`) ; `Explode` : rayon, dégâts (`OutOfRange`), `friendly_fire` `Never`/`Always` (D113), refusée dans un power-up | `bomb_fuse_zero`, `explode_radius_zero`, `explode_effect_ok` |
| Rochers (§62) | **T12** couche `Rocks` : valeur autre que 1 et 2 (ignorée au chargement : `OutOfRange`) | `rocks_and_surfaces_layers_are_checked` |
| Piques (§62) | `damage > 0`, `period > 0` (`OutOfRange`) ; **T12** couche `Surfaces` : valeur sans surface déclarée (`BrokenReference`) | `hazard_period_zero`, `rocks_and_surfaces_layers_are_checked` |
| Portes verrouillées (§62, §70) | `Key(0)`, `lock_from_floor` sans `door_lock` ou sur `Secret`, clé introuvable à chaque étage verrouillé | `door_lock_key_zero`, `key_room_*` |
| Grille (§62, §70) | `cell`, `max`, `post_boss.chance`, gabarits multiples de `cell`, ouvertures centrées et de même largeur, types distincts ; **T12** la salle secrète `required` d'une grille a `door_lock: Secret` (sinon posée ouverte : `OutOfRange`) | `grid_*`, `secret_room_not_required_or_not_walled` |
| Familiers (§63) | personnage inconnu ou sans behavior de familier, `per_copy = 0`, `gap`/`radius` ≤ 0, `every = 0`, behavior de familier sans porteur | `familiar_*` |
| Champions (§63, §72) | actions de mort admises (`Explode`, `SpawnCharacters`, `SpawnPattern`), pattern fini, pool de butin connu, `#rrggbb`, `chance` en expression dans [0, 1] pour 1 et 4 joueurs aux étages 0 et 5 | `champion_ok`, `champion_bad_death`, `champion_death_infinite_pattern*`, `variant_chance_expression_*` |
| Curses (§64) | poids 0, aucun effet, `FloorRooms` ≤ 0, `Modifier Mul` ≤ 0, chance hors [0, 1], pool vide ou inconnu | `curse_invalid_values`, `floors_curses_invalid` |
| Pools à épuisement (§64) | **T12** `exhaust: true` sur un pool qui n'a que des `Currency` (jamais « vu » : `OutOfRange`) | `exhaust_pool_of_coins_only_is_an_error` |
| Accomplissements, déblocages (§64) | `when`/`unlocks` vides, objet, boss ou déblocage inconnu | `achievement_invalid` |
| `Price` (§61, §73) | la surface de contenu est `shop.cost` d'un type de salle (T8) : `Price::problems` (montant nul, consommable inconnu, vie max hors `]0, max]` : `OutOfRange` / `BrokenReference`), `boss_drop.table` inconnu ; T12 n'ajoute rien (le trou de la première lecture, « aucune surface avant T8 », est fermé par T8) | `shop_cost_invalid`, `boss_drop_unknown_pool` (`m3_t8_fixtures.rs`) ; `items` : `price_problems_report_unknown_consumable_and_out_of_range_health` |
| `ExtraHealth` (§61, §68) | `GainSoul > 0` dans un objet, une synergie, un personnage ; refusée dans un power-up | `item_gain_soul_zero` |
| `hit_iframes` (§61) | `Some(0)` (`OutOfRange`) | `character_hit_iframes_zero` |
| Plafond de compteur (§61, §68) | `max_stack` 0 ou sur un non-consommable, `starting_consumables` au-dessus du plafond, `refuse_when_full` sans soin | `item_max_stack_zero`, `character_starting_over_max_stack`, `item_refuse_when_full_without_heal` |
| `Scaled(PerCopy)` (§61) | refusée hors d'un effet d'objet, action enveloppée validée, linéaire seulement | `scaled_*`, `effect_per_copy_*` |

### `mechanics` : les mécaniques d'une partie ordinaire

Le manifeste gagne `mechanics: [Mechanic]` (défaut vide : aucune vérification, tous les jeux avant `isaac`).
`content::Mechanic` a une variante par ligne du tableau « Atteignabilité » de `docs/taches.md` §8.1 : `Tears`,
`Effects`, `Familiars`, `Bombs`, `SecretRoom`, `Keys`, `Hearts`, `ExhaustPools`, `DevilMarket`, `Champions`,
`Curses`, `Achievements`. Pourquoi une déclaration : le lint ne peut pas deviner qu'`isaac` *doit* avoir un
familier ; sans elle, un jeu sans contenu de familier est muet, et « `make lint GAME=isaac` sans avertissement
d'atteignabilité » (critère de sortie M3, T11) serait vrai par vacuité. `isaac` déclare les douze.

**Partie ordinaire** (`mechanics::Play::ordinary`) : en mode `Floors`, la séquence de `entry.floors` (à défaut la
première par ordre d'id, comme le jeu), sans `FloorsOverride` ni les autres séquences ; sinon la carte de départ.
Les règles « toujours actives » ci-dessous gardent, comme §53, **toute** séquence déclarée comme racine (un
scénario ou le lobby peut en imposer une autre) ; seules les règles déclarées sont restreintes à la partie ordinaire.

**Sites de tirage** (`Play::draws`) : un pool est tiré par le coffre (`chest`) et la boutique d'un type de salle atteint
(`floor_tables` quand il y en a, sinon `table` : `RoomShopDef::table_for_floor` ignore `table` dès qu'il y a des
`floor_tables`), par la récompense des salles `combat` (le pool d'étage s'il a `reward`, **sinon** `salle` : `salle` n'est
tiré que pour une séquence sans `loot` ou à pool d'étage sans `reward`) et par le coffre de cette récompense quand
`chest_chance > 0` (pool d'étage ou `commun`), par le butin du boss (`boss_drop`, T8) d'un type de salle atteint, par les
rochers à butin des niveaux joués, par le butin (`loot`) des variantes à chance non nulle des ennemis atteints. Un coffre sans
`table` tire le `loot` de l'étage de chaque séquence, `commun` pour une séquence sans `loot` (`game::loot::chest_table_id`).
Chaque site porte un nombre de tirages par partie, au moins (salles exigées × étages de la partie, trois pour le remplissage).

Kind d'avertissement `LintWarningKind::MechanicUnmet` (JSON : `"kind": "MechanicUnmet"`), message « mécanique
« <nom> » : … » qui nomme la salle ou le pool qui devrait porter la mécanique ; fichier : le type de salle, le pool ou
la séquence fautif, `game.ron` quand le manque est global.

| Règle | Quand | Avertit si | Fixture (`m3_t12_fixtures.rs`) |
|---|---|---|---|
| pool jamais tiré | toujours | un pool n'a aucun site de tirage | `pool_that_nothing_draws`, `pool_of_a_default_chest_is_drawn` |
| familier sans objet atteignable | toujours | tous les objets qui citent le personnage (hors `test_only`, D63) sont hors des pools tirés, du départ des personnages jouables, des étages et des déblocages | `familiar_whose_items_have_no_source`, `test_only_familiar_is_not_warned` |
| synergie à objet sans source | toujours | un objet requis n'a aucune source | `synergy_with_an_item_that_has_no_source` |
| salle spéciale vide | toujours, grammaire en grille | une salle exigée, la salle secrète ou la salle post-boss à chance > 0 n'a ni `chest`, `shop`, `arena`, `boss_drop` ni `locks` | `special_room_that_carries_nothing` |
| mur secret sans bombe | toujours | un type à `door_lock: Secret` est atteint sans bombe jouable (consommable `bomb` **et** personnage à `bomb`) ni objet à `Explode { destroys: true }` | `secret_wall_without_a_bomb` |
| curse jamais tirée | toujours | aucune séquence ne la met dans `curses.pool` (chance > 0) | `curse_that_no_sequence_draws` |
| variante à chance nulle | toujours | `variants.chance` nulle pour 1 et 4 joueurs aux étages 0 et 5 | `variant_with_a_zero_chance` |
| accomplissement inaccomplissable | toujours | `FloorAtLeast(n)` au-delà d'une séquence finie (`victory_at_end`), `BossDefeated` d'un boss jamais posé, `ItemHeld` d'un objet sans source | `achievement_with_an_impossible_condition` |
| déblocage jamais accordé | toujours | ni accomplissement ni service `Merchant` ne l'accorde | `unlock_that_nothing_grants` |
| `Tears` | déclarée | aucun objet de larmes (`projectile`, `ShotSpeed`, `ShotSize`) dans le pool d'une salle exigée à coffre (trésor) ni dans une boutique ; ou l'objet ne peut pas être tenu en deux exemplaires : chaque pool qui le donne a `exclude_held` ou `exhaust` (un objet sorti ne revient pas avant que le pool soit vu en entier) et il n'y a pas deux pools sans `exclude_held` | `tears_missing_from_the_*`, `tears_that_cannot_be_held_twice`, `tears_in_a_single_exhausting_pool_cannot_stack` |
| `Effects` | déclarée | aucun objet à effets dans le pool trésor, boutique ou boss ; aucune synergie dont tous les objets sont atteignables | `effects_missing_from_the_boss_pool_and_no_transformation` |
| `Familiars` | déclarée | moins de deux objets à `familiar` dans le pool trésor, aucun dans une boutique | `familiars_missing_from_the_treasure_and_shop_pools` |
| `Bombs` | déclarée | pas de bombe de départ (personnage jouable), d'étage, de pool tiré (récompense, coffre, boutique, rocher, champion) ni de déblocage ; aucun personnage jouable à `bomb` (un personnage `test_only` ne compte pas, D63) ; aucun rocher ni pique (surface à `hazard`) dans un gabarit du type de remplissage | `bombs_rocks_and_spikes`, `test_only_characters_give_no_bomb_and_wrapped_explosions_count` |
| `SecretRoom` | déclarée | pas de grammaire en grille ; la salle secrète n'est pas dans `required` ou n'est pas murée ; aucune explosion atteignable | `secret_room_not_required_or_not_walled` |
| `Keys` | déclarée | pas de clé dans la récompense de combat ni dans une boutique ; aucune salle exigée à `door_lock: Key` | `keys_in_rewards_shops_and_locked_treasure` |
| `Hearts` | déclarée | pas de `Currency` ni de cœur (`Heal`, `GainSoul`) dans la récompense de combat ; aucune boutique exigée (`required`) : une salle à `shop` payé en vie maximale (le diable) n'est pas la boutique | `hearts_coins_and_shop`, `devil_market_is_not_the_shop` |
| `ExhaustPools` | déclarée | aucun pool `exhaust` tiré au moins deux fois par partie ordinaire | `exhausting_pool_drawn_once` |
| `DevilMarket` | déclarée | aucune salle post-boss à chance > 0 dans une grammaire de la partie ordinaire ; ou son type de salle n'a pas de `shop` à `cost` (présentoirs payés, T8 : sans prix ce n'est qu'une boutique) | `devil_market_without_a_post_boss_room`, `devil_market_needs_priced_shelves` |
| `Champions` | déclarée | aucun ennemi atteint, hors `test_only`, ne porte une variante à chance non nulle | `champions_on_no_reachable_enemy` |
| `Curses` | déclarée | la séquence ordinaire n'a pas de `curses` (chance > 0, une curse de poids > 0) | `curses_only_count_in_the_ordinary_sequence` |
| `Achievements` | déclarée | aucun accomplissement aux conditions remplissables (§ ci-dessus) dont un déblocage donne un objet ou une entrée à un pool tiré | `achievements_without_a_visible_unlock`, `achievement_with_an_impossible_condition_is_not_visible` |

Les avertissements M2 nomment maintenant la mécanique : « type de salle « secrete » : … ; mécanique : salle secrète
(murée, ouverte à la bombe) » (`room_mechanic`), « objet « x » : aucune source … ; mécanique : familiers, effets »
(`item_mechanics`). Fixture de base `mechanics_ok` : un jeu en grille (départ, combat avec rocher, rocher à butin et
pique, boss, trésor à clé, secrète, boutique, diable) qui porte les douze mécaniques, sans erreur ni avertissement
(`base_is_clean`) ; chaque test lui retire un élément dans un dossier temporaire. Sans `mechanics`, aucune règle
déclarée ne parle (`undeclared_mechanics_are_not_checked`). `alacod lint --json` rend `MechanicUnmet`
(`lint_json_reports_mechanic_warnings`).

### Sources d'objets et d'armes

`reach::loot_pool_items` (pools, entrées `Item` et `Rarity`) et `loot_pool_weapons` (entrées `Weapon`) sont tous deux
branchés depuis D62 : le constat de la fiche (« le second renvoie un ensemble vide ») était périmé, rien à y ajouter
pour les armes. Ajouté : `unlock_grant_sources` (les entrées qu'un déblocage ajoute à un pool, `UnlockGrant::Loot`,
comptent comme sources d'objets et d'armes : sans elles, un objet que seul un accomplissement donne était signalé
« sans source ») et `item_sources` (départ, étage, pools, déblocages) partagé par les règles. Les sources M3 de la
fiche (rochers à butin, champions, salles trésor, diable, secrète) sont **toutes des pools nommés** : leur contenu est
déjà dans `loot_pool_items` ; ce qui manquait était de savoir si quelqu'un tire le pool (règle « pool jamais tiré »).

### Limites

Statique : une salle posée par la grammaire n'est pas une salle *rencontrée* (probabilités, graines : bot T14 et
campagnes T18) ; l'ordre des étages (`lock_from_floor` contre la longueur d'une séquence)
n'est vérifié que pour les clés (`lint_key_sources`) ; le nombre de tirages par partie est un minimum prudent ; une entrée
qu'un déblocage ajoute à un pool (`UnlockGrant::Loot`) compte comme source d'objet sans vérifier que le déblocage est
accordé (la règle « déblocage jamais accordé » le dit à part).

### Atteignabilité (jeux livrés)

`zombies`, `throne` : aucun avertissement. `gungeon` : un, **réel** (pool `boss` jamais branché : le butin du boss est
`boss_drop` de T8, `gungeon` ne le pose pas ; aucune tâche M3 ne le ferme : dette). `testbed` (bac à sable, beaucoup
d'avertissements voulus) : dix de plus (pools d'essai non tirés dont `epuisable_temoin`, familiers et synergie d'essai, mur
secret sans bombe). `isaac` : **0** depuis M3-T11d1 (§76.4 ; 24 mesurés sur `main` avant la tâche : boutique, curses, bombe, rochers, piques, secrète sans bombe et l'empilement des
objets de larmes) ; la liste des 18 de T12 reste au rapport `docs/taches/rapports/m3-t12-lint-atteignabilite.md`.

## 78. Générateur v3 : gabarits `TearsOnTarget`, `FamiliarHeld`, `ChampionForced` (M3-T13)

Code : `crates/scenario/src/generate.rs` (`build_tears_scenario`, `build_familiar_scenario`,
`build_champion_scenario`, `champion_death_expectations`, `discover_target`), `game::gen_test`
(`TearsTest`, `FamiliarTest`, `ChampionTest`), `game::character::config::CharacterTest::champions`,
`content::lint` (`lint_item_tests`, `lint_character_tests`). Complète §28 et §54 : mêmes règles
(opt-in par `test:`, aucun effet en jeu, un scénario écrit par `make gen GAME=<jeu>`, champ absent =
rien de généré, les scénarios et traces existants ne bougent pas). **Aucune attente nouvelle** : les
gabarits n'emploient que `HasItem`, `Stat`, `EntityHits`, `HitsAtLeast`, `FamiliarCount`,
`EnemyVariant`, `EntityCount`, `Event`, `PlayerAlive`.

Les trois gabarits sont des **champs ajoutés à des `test:` existants** : un objet ou un personnage
déjà généré garde son fichier octet pour octet. Ils jouent dans l'arène de `generate_template`
(`WeaponArena`, `target`/`cible` avec `counts_hits`) pour les objets, dans l'arène du gabarit ennemi
(§28) pour les champions. Les nouveaux sujets sont rangés **après** les anciens dans `--list`.

**Larmes** (`objets/<id>.ron`, fichiers `tears_<id>.ron` **et** `tears_<id>_sans_objet.ron`) :
`test: Some((frames: 60, tears: Some((weapon: Some("larme"), frames: Some(150), targets: [(character: "target", dx: -48, dy: -18)], hits: [(target: 0, with_item: 5, without_item: 0)], expect: [...], control: [...]))))`.
Le joueur tient l'objet (`PlayerScript::items`), tire par impulsions (`weapon`, visée `aim` : par
défaut vers la cible de l'arène) ; le **témoin** rejoue le même tir, le même placement, sans l'objet :
la preuve de l'effet est la différence entre les deux fichiers, dans le dépôt. Attentes avec l'objet :
`HasItem`, une `Stat` par modificateur de stat (valeur sondée comme `ItemHeld`, §54), chaque `hits`
(`EntityHits ≥ with_item`), puis `expect` ; témoin : chaque `hits` (`EntityHits ≤ without_item`),
puis `control` (le contraire de `expect`). `targets` : personnages à `counts_hits` posés à `(dx, dy)`
unités monde de la cible de l'arène (cible d'index 0 de `hits`, les posées ont l'index `n + 1`, et
`Placed(n)` dans `expect`/`control`) ; leur santé règle le test ; les net ids sont retrouvés par
position (`discover_hit_net_ids`). `frames` propre au gabarit (défaut `test.frames`) : le `ItemHeld`
d'un objet existant ne change pas. Le lint exige `with_item > 0` et `with_item > without_item` (un
seuil que le témoin passerait est refusé), `expect` et `control` ensemble, au moins un `hits` ou un
`expect`. Un objet dont l'effet ne change pas les coups prouve par `expect`/`control` (ex.
`BulletCount` exact, `HasStatus` / `present: false`).

**Familier** (objet à `familiar:`, fichier `familiar_<id>.ron`, et `familiar_<id>_sans_objet.ron` si
`min_hits > 0`) :
`test: Some((frames: 60, familiar: Some((weapon: ..., frames: Some(120), min_hits: 9, expect: [...]))))`.
Attentes : `HasItem`, `FamiliarCount = per_copy` à la frame 5 et à l'avant-dernière frame ; si
`min_hits > 0`, le joueur tire aussi et la cible reçoit au moins `min_hits` coups ; le témoin (même
tir sans l'objet) en reçoit au plus `min_hits - 1` et n'a aucun familier : fixer `min_hits` d'après
le témoin. `familiar:` sur un objet sans `familiar:` : erreur de lint.

**Champion** (personnage à `variants:`, fichier `champion_<id>_<variante>.ron`) :
`test: Some((frames: 150, still: false, moving: false, champions: [(variant: "explosif", kill: true, weapon: Some("larme")), (variant: "nu")]))`.
L'ennemi apparaît à 200 unités à droite du joueur avec la variante imposée
(`CharacterPlacement::variant`, §63) ; attente `EnemyVariant(Placed(0))` à la frame 2. `kill: true` :
une **sonde** (`probe_champion_death`) joue la partie, relève la frame de la mort (première frame sans
`Variant`) et ce qui l'entoure ; le scénario attend alors **la mort** : `EntityCount(Enemy)` exact
avant (frame − 1) et après (frame + 1 : avant − 1 + personnages nés), puis chaque effet de mort
déclaré : `Explode` → moment clé `explosion` ; `SpawnCharacters` → `count` ennemis de plus ;
`SpawnPattern` → `BulletCount` des balles de l'équipe ennemie (0 avant, le nombre mesuré après) ;
`loot` → le joueur tire jusqu'à la mort puis marche vers +x et appuie sur Interaction (80 frames de
marche), `HasItem` de l'objet ramassé (`frames` assez long). Une variante sans effet traduisible
est refusée par le lint (`kill: true`), et un effet qui n'a laissé aucune trace à la sonde (motif sans
balle, butin non ramassé) fait échouer la génération : rien n'est ignoré en silence. Une variante
répétée ou absente de la table est une erreur ; `still: false, moving: false` : pas de gabarit
`enemy_<id>_*` en plus.

**Lint** (`content`) : voir ci-dessus ; `tears.frames`, `familiar.frames > 0` ; armes connues ;
cibles `targets` connues et à `counts_hits` ; `hits.target` dans les cibles ; `champions` : variantes
de la table, sans doublon, `weapon` seulement avec `kill` et connue, `kill` seulement avec un effet de
mort traduisible. Fixtures : `champion_test_{unknown_variant,duplicate_variant,unknown_weapon,
kill_untranslatable,weapon_without_kill}`, `tears_test_{min_hits_zero,hits_not_above_control,
nothing_proven,expect_without_control,frames_zero,hit_target_out_of_range,weapon_unknown,
target_unknown,target_no_counts_hits}`, `familiar_test_{without_familiar,frames_zero}`.
Le schéma de `alacod describe` (kind `Item`, `Character`) expose les nouveaux champs.

**Limites des preuves.** Le lint vérifie la cohérence des seuils **déclarés** (`with_item >
without_item`, `expect`/`control` ensemble) ; leur adéquation aux parties réelles n'est prouvée qu'en
jouant les scénarios (`make gen` joue l'objet et son témoin). Une preuve par `BulletCount` exact à
une frame (`rebond`, `spectre` : 1 balle en vol contre 0 à la dernière frame) dépend de la phase
tir/vol/durée de vie : un changement de cadence, de vitesse ou de `lifetime` de l'arme la fait
basculer (les traces et le témoin le signalent) ; préférer un compte de coups quand l'effet en change.

**Échelle** : le nom d'un fichier est une fonction de l'id seul (`tears_<id>`, `familiar_<id>`,
`champion_<id>_<variante>`) ; ajouter un objet ordinaire = un `.ron` de contenu avec `test:`, jamais
un scénario écrit à la main. Mesures : voir `docs/taches/rapports/m3-t13-generateur-v3.md`.

## 79. Bot `explorateur` v3 pour `isaac` : bombes, clés, achats, diable (M3-T14, chantier K2 v3)

Complète §24 et §47 (`explorateur`, D65). Code : `crates/bots/src/explorer.rs` (réécrit : `ExplorerWorld::frame`,
`GoalContext`, `ordered_goals`, `rock_goal`, `explore_step`, `flee_step`), `crates/bots/src/input.rs`, `game::replay`
(`ExplorerPolicy`), `scenario::{runner, events}`, `alacod-sim`. **Le bot ne produit que des inputs** (`ReadInputs`, hors
`GgrsSchedule`) : aucune simulation touchée, aucun type rollback nouveau. **Aucune trace existante ne bouge** (la suite entière
rejouée sans BLESS, dont les 25 scénarios à `explorateur` de `main` : `bot_explorer_*`, `grid_floor_seed_*`, `grid_post_boss`,
`gungeon_*`, `isaac_floor_seed_1`, `achievement_unlocks`, les `room_*` générés : traces identiques). Les décisions nouvelles sont **inertes sans leur objet** : aucun porteur
de bombe, porte à clé, salle secrète, rocher ni présentoir à `cost` dans `gungeon`, `throne`, `zombies` ou `testbed`
(hors cartes d'essai). **Cette inertie tient parce que personne ne porte de bombe** : `BombPolicy::default()` a `secret` et `rocks` vrais.
Quand `pleureur` portera `bomb` (T11d1), les scénarios isaac à `explorateur` en politique par défaut poseront des bombes et leurs traces
bougeront : le défaut reste tel quel, **la branche mergée en second rebénit ces traces avec preuve §5** et rejoue les campagnes `s1`, `i2`, `i4` : **fait par M3-T11d1 (§76.4, D106 fermée)**.

### Politique en données : `ExplorerPolicy`

```ron
// games/isaac/assets/bot.ron (lu par alacod-sim ; Scenario::explorer : Some((…)) pour un scénario)
(
    shopping: true,                                   // coffres et présentoirs (D62)
    shop: HeartsThenCheapest(low_health: "0.5"),      // Nearest (défaut, ordre d'avant) | HeartsThenCheapest
    detour: (shells: Some(400)),                      // limite de détour (unique) : sous à au plus 400 de coût de chemin (10 par case de 8 px)
    devil: Never,                                     // Never | IfHealthAbove("4.0")
    bombs: (secret: true, secret_min_neighbours: 2, rocks: true, reserve: 0),
    oscillation: Hysteresis,                          // anti-oscillation (une stratégie : Off | Hysteresis | Patience, ci-dessous)
    combat_aware: true,                               // fuit toute bombe allumée, s'approche pour des larmes en orbite
)
```

`Default` = comportement d'avant M3-T14 (aucun achat, ordre du plus proche) : un jeu sans `bot.ron` ne change pas.
`alacod-sim` : `--policy <fichier>` (sinon `bot.ron` du jeu), `--shopping`, `--devil never|above:<n>`,
`--bombs <n>` (voir « Bombes d'essai »), `--pickup-progress` (soft-lock, ci-dessous). Le JSON rapporte `mechanics` (décomptes
`mur_secret`, `porte_a_cle`, `bombe`, `coffre`, `achat`, `diable`, `objet`…) lus dans les moments clés.

### Décisions

- **Porte à clé** : une porte `InteractionType::KeyDoor` fermée (et pas déjà dans `DoorLocks`) est un but d'objet (approche du
  collider à portée d'interaction − 8, Interaction tenue) quand le joueur a **au moins `n` clés**, `n` lu dans
  `RoomKindTable::door_lock_at(kind, étage)` comme `key_door_system` (salles triées par position puis identifiant : la même
  qui lit le prix). Sans la clé : pas un but (sinon `key_door_refused` en boucle).
- **Mur secret** : la salle est « probable » quand son type porte `door_lock: Secret`, qu'elle n'est pas dans `DoorLocks` et
  qu'elle touche **au moins `secret_min_neighbours` autres salles** par un bord commun (règle de la grille, §70). Le but est un
  point à moins de `rayon − 8 − |décalage du collider|` du bord d'une de ses portes closes (l'explosion touche **strictement** à
  moins du rayon, `world::rect_within_radius`). Le bot lit **le type de la salle dans le contenu** (la minicarte ne la montre pas
  tant que le mur est fermé) : une inférence sans cette lecture (case vide voisine de deux salles) est une dette.
- **Rocher** : seulement quand plus aucun but ordinaire n'est atteignable et qu'il en reste (salle à visiter, objet, portail). Un
  second champ de navigation **sans les rochers** (`RockNavigation`) dit ce qu'une bombe ouvrirait ; le rocher visé est le
  **premier du chemin** vers le but le moins cher de ce champ (cases du chemin à moins de 14 du bord de son collider, 16 × 16), la
  zone de pose est à portée de bombe de son centre (`cells_in_radius` : centre à moins du rayon). Un but que le champ réel atteint
  (détour) n'est jamais une raison de bombe.
- **Pose et mèche** : le bot presse `Bomb` à l'arrivée (front montant : `BombCarrier::held` lu), puis **fuit** (`flee_step`, prioritaire
  sur tout, combat compris) vers la case accessible la moins chère à plus de `rayon + 16 + 8` de **toutes** les bombes, sinon à
  l'opposé de la bombe. Tir ami `Always` : sans la fuite la bombe lui retire 4 PV sur 6 (`bot_isaac_secret` : vie pleine à f100).
  **Un seul poseur par mur** : un mur ou un rocher dont un joueur de **plus petit handle**, porteur de bombe et debout, est déjà à
  portée de pose (+ 40) est le sien (`GoalContext::peers` : deux bots arrivés ensemble en posaient chacun une, `bot_isaac_two`).
  Pas de nouveau but de bombe tant qu'une mèche brûle.
- **Objets refusés** : un consommable que le jeu refuserait (`ItemDef::accepts` : stock au `max_stack`, cœur `refuse_when_full` à
  pleine vie) n'est **pas un but** (`PickupRule`) : le bot attendrait sans fin sur le tas. Vaut pour les présentoirs (`shop_full`).
- **Achats en douilles** (`shop: HeartsThenCheapest { low_health }`, par joueur : il n'achète que ce qu'il paie) : **un cœur d'abord**
  quand sa vie est au plus `low_health × max` (et qu'elle n'est pas pleine), **sinon l'objet le moins cher**, puis le plus proche ;
  un seul présentoir à la fois, les autres suivent. `Nearest` garde l'ordre d'avant (D62 : le plus riche des joueurs paie).
- **Sous au sol** : `ShellPickup` à moins de `n` de coût de chemin (contact). Sans limite, un étage nettoyé envoyait le bot
  ramasser des tas aux quatre coins (graines 79, 89, 96, 132 de la première campagne : 1 200 frames sans kill).
- **Anti-blocage de combat** (`oscillation: Hysteresis`, `ExplorerJitter.1`, `BotView::{engage_reach, engage_strafe}`) : un combat où l'empreinte de
  la vie des ennemis (`ExplorerWorld::enemy_progress` : nombre et somme des vies courantes) ne bouge plus depuis 240 frames est un blocage :
  les larmes s'arrêtent sur un coin de mur ou un pilier (`ldtk_wall_1x3` devant la tourelle de la graine 63), le bot tire sans fin vers un
  ennemi qui ne perd rien (graines 86, 178, 182 de la première campagne sur le contenu T11c : le boss `gardien` restait à 14,8 PV). Le bot
  **s'approche** alors de l'ennemi par le chemin (`navigate` calcule la route même sans ligne cachée) et, après 90 frames, **le contourne**
  (tangente, sens alterné toutes les 90 frames, à moins de 60 de lui). Levé dès qu'un coup au but change l'empreinte, ou à la fin du combat.
  Un ennemi qu'aucune ligne de tir atteignable ne touche (tourelle derrière un pilier, graine 63) reste un défaut de contenu T11c/T11d.
- **`combat_aware`** (faux par défaut) : la fuite d'une bombe allumée ne vaut plus que pour le porteur d'une bombe **ou** sous `combat_aware` (un champion
  explosif, D88, en pose une : sans la politique, `grid_post_boss` et `grid_floor_seed_23` divergeaient de `main` dès f767 et f2371 ; avec, les traces
  de tous les scénarios des autres jeux et de la grille sont celles de `main`) ; de même `orbit_reach` n'est lu que sous `combat_aware`. Visée
  `FourWay` (`pleureur`, suivi Q17) : un combat **engagé** (`engage_reach`) n'écrit que la direction de tir, sans la file de tir qui figeait le bot, et
  s'approche **en ligne droite** quand la ligne est dégagée (la route contournait un coin que les larmes ne passent pas ; graines 63, 95, 119).
- **Poursuite sur place** (`ExplorerJitter.2/.3`, `pursue`, sous `Hysteresis`) : 300 frames sans quitter un rayon de 24 en marchant vers un but qui n'est
  pas une salle **et dont le pas est réellement exécuté** (ni réanimation, ni butin, ni esquive, pas un pas nul) écartent ce but pour ce joueur
  (`explore_step(…, avoid)`) ; graine 103 d'`isaac` à deux bots : un tas de sous contre un mur que la navigation croit libre, deux buts alternés. La
  mémoire du bot (`ExplorerJitter` : champs `sway`, `stalls`, `pursuits`, `banned_goals`, `snaps`, `floor`) est **remise à zéro à chaque partie** (`OnEnter(InGame)` : les `GgrsNetId` repartent de zéro) ; buts écartés, poursuites, blocages et tenues sont **oubliés à chaque changement d'étage** (`enter_floor` : l'identité d'un mur est sa zone, qui peut revenir à l'identique).
- **Réanimation à portée** (`revive_step`, `snap`, sous `combat_aware`) : déjà à portée de la surface que le jeu sélectionnerait, Interaction tenue
  sans bouger (le pas d'approche visait portée − 8 et faisait osciller le bot de ±9 : la réanimation, tenue plusieurs frames, ne finissait jamais) ;
  **sans marge** (une marge de 5 recréait le blocage : 4 soft-locks à 2 bots) mais **tenue bornée** (`ExplorerJitter::snap_hold` : 240 frames puis 60
  d'approche, car `Rect::distance` mesure la boîte d'un collider circulaire et le jeu le cercle) ; `BotView::hold_revive` : sous visée `FourWay`, le bot qui tient la réanimation ne se
  décale pas pour s'aligner sur un ennemi.
- **Larmes en orbite** (`BotView::orbit_reach`, lu par `ExplorerWorld::orbit_reach` : `Orbit` des objets et synergies tenus, composés comme au
  tir par `combat::projectile::compose`) : `satellite` (T11b) fait tourner les larmes à 40 unités du porteur ; un boss immobile à 94 unités ne
  perdait plus un point de vie, `prudent` ne s'approchant d'un ennemi immobile que jusqu'à 120 (graine 41 d'`isaac` sans bombe, soft-lock
  de 600 frames). Avec une orbite, `decide_prudent` s'approche jusqu'à `orbit_reach` (immobile ou non) et ne recule plus devant un ennemi
  mobile ; sans orbite (`None`), rien ne change pour aucun profil ni aucun jeu.
- **Anti-jitter** (`oscillation: Hysteresis`, seconde mémoire du bot après `ExplorerVisits`, `ExplorerJitter`, hors rollback) : un bot qui veut
  avancer mais reste dans un rayon de 12 pendant 40 frames balance entre deux buts (ou deux routes) de coût presque égal dont l'ordre
  s'inverse à chaque pas, la commande arrivant avec le délai d'input. Il **s'attache** alors à son but du moment (`Step::target`,
  passé en premier à `explore_step` tant qu'il est atteignable) et **tient** son prochain point de passage 60 frames au plus.
  Découvert sur la graine 19 d'`isaac` sans bombe (deux salles équidistantes, bot immobile 600 frames, soft-lock) ; inerte tant qu'il
  n'y a pas de balancement, et faux par défaut (aucun autre jeu ne change).
- **Diable** (`Price::MaxHealth(perte)`) : `Never`, ou `IfHealthAbove(n)` : achète si l'âme couvre tout le prix, ou si la vie
  courante dépasse **strictement** `n` et dépasse le reste à payer (le jeu refuse un paiement qui tuerait, §73). **Défaut : `Never`**
  (Q14, à confirmer par William) : sur les 55 graines de la campagne qui ont une salle du diable, **contenu de T11b (2026-10-17)** :
  55 victoires avec `Never`, 48 victoires et 7 morts avec `IfHealthAbove("4.0")` ; **contenu final (D69, 2026-10-18)** : 5 victoires
  contre 5, aucun signal (le bot meurt avant le présentoir : 4 achats en 3 graines) ; les objets du diable sont encore des données
  d'attente (un cœur de moins ne se rachète pas, T11d). Écart connu avec le jeu : `shop::max_health_after` refuse aussi un `Mul`/`Set` de `MaxHealth` qui annulerait la
  stat ; le bot ne le modélise pas (aucun contenu `isaac` n'en a ; dette).

### Bombes d'essai

**Depuis M3-T11d1 (§76.4, D81 fermée), `pleureur` porte une bombe de départ réelle** (`bomb` : mèche 75, rayon 56, 4 PV, tir ami `Always` ; `starting_consumables: {"bomb": 1}`) : les campagnes d'`isaac` jouent la configuration « livrée »
**sans kit** (`kit de bombes None` au journal d'`alacod-sim`) et les scénarios n'ont plus besoin de `PlayerScript::bomb` ni de `items: ["bomb"]`. Le kit `--bombs <n>` reste disponible pour un jeu dont le personnage n'a pas de bombe (mèche 60, rayon 56, dégâts 4, tir ami `Always`,
une bombe dans `items`). Le bot lit la `BombDef` du `BombCarrier` (rayon, mèche) : rien à changer.

### Soft-lock et diagnostic

`alacod-sim --pickup-progress` (`StopEarly::pickups_are_progress`, **faux par défaut** : les autres campagnes gardent leur détecteur) : un
changement d'inventaire, de solde ou de compteur de consommables d'un joueur est une **progression** du soft-lock `Floors`
(`FloorsProgress::stock`, empreinte par joueur triée par handle). Sans lui, les détours légitimes d'un étage nettoyé (ramasser un objet
loin, revenir au portail) dépassaient les 1 200 frames sans kill. Un vrai blocage (bot immobile, boucle sans ramassage) reste détecté ;
une boucle qui ramasse et lâche sans fin finit au plafond (`plafond`, classé échec).
`ALACOD_BOT_DEBUG=1` : l'`explorateur` écrit son pas d'exploration à chaque frame sur stderr (hors simulation).
Moments clés ajoutés (`scenario::events`) : `door_lock` (« mur secret ouvert » / « porte à clé ouverte », lus dans `DoorLocks`),
`bomb` (bombe posée), `shelf` (présentoir vendu, douilles ou diable) ; ni rollback ni trace.

### Scénarios

| Scénario | Prouve |
|---|---|
| `bot_isaac_floor` | partie ordinaire (graine 6, séquence par défaut `sous_sol`, bombe de départ réelle, politique de `bot.ron`) jouée jusqu'à la victoire (f4815) : bombe contre le mur secret aux deux étages, coffre, boutique, porte à clé (f3268) ; les attentes ne nomment aucun objet (le tirage du butin change : D95/D96) |
| `bot_isaac_secret` | graine 39 de `sous_sol_un`, la bombe de départ : le bot pose (f19), fuit (vie pleine), le mur s'ouvre (f94), coffre secret ouvert (f1412) |
| `bot_isaac_key` | `sous_sol_cle` (graine -1013903988, plan 254 : aucune secrète près du départ, pas de détour à la bombe) : le bot ouvre la porte à clé du trésor (f20), le coffre (f81) |
| `bot_isaac_two` | deux bots (graine 39, `sous_sol_un`), **une seule bombe** posée au mur secret (le joueur 1 garde la sienne), trois achats en boutique (f1755 à f1792), victoire f2224 ; le refus du diable est prouvé par `bot_isaac_devil_never` |
| `bot_isaac_devil` / `_devil_never` | présentoir à `MaxHealth("2")` posé au pied du joueur : `IfHealthAbove("4.0")` l'achète (f12, 6/6 → 4/4), `Never` non |
| `bot_isaac_rock` / `_rock_sans_bombe` | carte `testbed/bot_rocks.ldtk` (`scripts/construire-cartes-t14.py`) : portail derrière deux rochers sans détour ; avec une bombe le bot les casse (f48 → f107) et passe (« niveau 1 », f184) ; sans bombe il reste devant |
| `bot_isaac_shop` | trois présentoirs en douilles (un cœur à pleine vie, un trèfle proche, une bombe moins chère) : la bombe d'abord (f35), le trèfle (f88), le cœur refusé |
| `bot_isaac_shop_blesse` | blessé (6 PV sur 8), le cœur à 9 douilles **d'abord** (f80), puis la bombe (f135) |

Tests unitaires (`bots::explorer`) : zone et ordre des buts, cœur refusé à pleine vie / stock au plafond, porte à clé, achats, diable
(`devil_accepts`), mur secret (stock, réserve, mèche en cours), premier rocher du chemin, but atteignable par détour, voisinage de
salles (`share_an_edge`), fuite, anti-jitter (`Hysteresis`, garde bornée), patience (`ExplorerJitter::patience`), détour borné (`Detour`), anti-blocage de combat (`engage`), poursuite sur place (`pursue`), identité
des buts. `runner::floors_progress_tests` : un ramassage compte. `decide::larmes_en_orbite_le_bot_s_approche`.

### Réconciliation avec m2-salles-murs-gungeon (`ExplorerOptions::steady` éclaté en politique)

La branche `m2-salles-murs-gungeon` avait ajouté des décisions d'`explorateur` pour les salles agrandies derrière un seul booléen `ExplorerOptions::steady`,
posé par le runner et `alacod-sim` quand `scenario.game == "gungeon"` : **un nom de jeu dans le code**. Il n'existe plus. Chaque décision est un champ de
`ExplorerPolicy`, posé **en données** par gungeon (`games/gungeon/assets/bot.ron` ; `Scenario::explorer` des scénarios gungeon écrits à la main ; les scénarios
**générés** lisent le `bot.ron` du jeu, `runner::game_bot_policy`, par `ExplorerPolicy::scene()` : ni achat, ni sous, ni diable, ni bombe, un gabarit teste une salle) :

| Champ | Rôle | gungeon | isaac |
|---|---|---|---|
| `detour: (shells, optional, last_stretch)` | **une seule limite de détour** : un but facultatif de coût de chemin supérieur à la borne n'est pas un but. `shells` : les sous au sol ; `optional` : objets, présentoirs, coffres et salles sans verrou tant qu'une salle de combat reste ; `last_stretch` : idem quand plus aucune salle de combat ne reste (ligne droite jusqu'au boss) | `optional: 600`, `last_stretch: 200` | `shells: 400` |
| `lead` | anticipe l'élan transverse (`explorer::lead`) devant une ouverture de 32 px (D82) | vrai | faux |
| `oscillation` | **une seule stratégie anti-oscillation** (`Off` \| `Hysteresis` \| `Patience`), une seule mémoire (`ExplorerJitter`) | `Patience` | `Hysteresis` |
| `weapon_reserve` | ni achat ni ramassage d'une arme sans réserve de munition ; ramassage « à sec » | vrai | faux |

`Detour` remplace `ExplorerPolicy::shells` et les constantes `OPTIONAL_DETOUR`/`LAST_STRETCH_DETOUR` : `GoalContext::detour(frame)` donne la borne, `ordered_goals`
l'applique aux objets, présentoirs et salles. **Anti-oscillation : un seul emplacement, pas deux mécanismes parallèles**, mais deux stratégies parce qu'aucune ne
remplace l'autre sans bouger des traces (essais du 2026-10-18, 15 scénarios gungeon et 9 isaac) : `Hysteresis` à la place de `Patience` dans gungeon déplace
`gungeon_full_run` (ligne 1969 de la trace, événements `portal`, `purchase`, `chest`, boss absents) et `gungeon_full_run_four` ; `Patience` à la place de `Hysteresis`
dans isaac déplace `bot_isaac_floor` (ligne 3904) et `bot_isaac_two` (ligne 637). La campagne gungeon reste verte avec `Hysteresis` seul (20 graines à 1 et 2 bots, 40 victoires,
0 desync, mais des durées différentes sur chaque graine) : supprimer `Patience` est possible au prix de **rebénir deux scénarios `gungeon_full_run*` avec leurs attentes**,
décision laissée à la revue (dette D110 ; sortie : une stratégie choisie, campagnes rejouées, deux scénarios rebénis avec preuve §5).

**Soft-lock `Floors` : « cases explorées »** : entrer dans une case de 32 px jamais visitée n'est une progression que sous `StopEarly::explored_is_progress`
(`alacod-sim --explored-progress`, faux par défaut). Les campagnes gungeon la passent ; isaac et throne gardent leur détecteur d'origine (santé, mort, ramassage).
Commande de la campagne de contrôle : `alacod-sim --game gungeon --bots N --profiles explorateur[,explorateur] --floors etages_deux --until-floor 5 --seeds 1..20
--max-frames 20000 --shopping --explored-progress`. Mesure de la réconciliation : durées (frames) et issues **identiques graine par graine** aux campagnes de
`m2-salles-murs-gungeon` (gungeon, 20 graines × 1 et 2 bots) et de T14 (`isaac` `s1`, 20 graines).

### Campagnes et atteignabilité

Chiffres, tueurs et graines : `docs/taches/rapports/m3-t14-bot-isaac.md` (scripts `campagne.sh`, `analyse.py`, `morts.py`,
`diable.sh` à côté). Atteignabilité : toute partie ordinaire d'`isaac` rencontre la porte à clé (second étage), les coffres et les sous ;
bombes sur le mur secret : toute graine avec `--bombs 1` (195 graines sur 200 ; graine 8 de `sous_sol_un`, `bot_isaac_secret`) ; diable : les 55
graines à salle du diable (8, 9, 13, 15, 16, 23, 26…), mais le bot n'y achète que 4 fois en 3 graines (23, 28, 181) : il meurt avant dans 90 % des
parties (Q14, au rapport). **Campagne finale (contenu de `main` au 2026-10-18, D88/D95/D96, visée `FourWay`)** : 1 bot sans bombe 12 victoires / 188 morts /
0 soft-lock / 0 desync ; 1 bot avec la bombe d'essai 18 / 182 / 0 / 0 ; 2 bots 116 / 84 / 0 / 0 ; 4 bots (20 graines) 18 / 2 / 0 / 0.

## 80. Bench de larmes, de familiers et d'étage en grille ; budgets par poste (M3-T15, D84)

**Mesure seulement** : quatre scénarios neufs, budgets provisoires dans `tests/budgets.ron`, une ligne de diagnostic dans le runner.
**Aucune trace existante ne bouge.** Rapport et tableaux : `docs/taches/rapports/m3-t15-bench-larmes.md`.

### Scénarios

| scénario | carte | contenu | pic mesuré (poste VPS, relevé 3) |
|---|---|---|---|
| `bench_tears_trio` | `testbed/arena` | un joueur, `division` + `rebond` + `guide` sur la `larme` (20 coups/s), 30 `target` (5 000 PV, immobiles) | 86 entités, 24 balles |
| `bench_tears` | `testbed/arena` | **quatre** joueurs (synctest), dix objets de larmes chacun (`division` ×2, `guide`, `rebond` ×2, `perforant`, `spectre`, `venin`, `lentille`, `trefle`) + `essaim` (2 familiers), mêmes 30 cibles | 506 entités, **432 balles** |
| `bench_familiars` | `testbed/arena` | quatre joueurs × `essaim` (deux familiers), `larme` sans objet | 63 entités |
| `bench_floor_isaac` | `isaac`, `sous_sol_grand` (12 à 15 salles, §70), graine 1 | deux bots `explorateur`, 3 000 frames, 6 `suiveur` champions `explosif` posés à f5 (`characters` avec `variant`) | 225 entités, 31 ennemis |

La cadence de la `larme` est portée à 20 coups/s par `weapon_overrides` (3 par défaut : 24 balles au plus avec trois objets).
Les 432 balles de `bench_tears` viennent de la chaîne de `Split` (deux `division`, `MAX_GENERATION` = 8), pas du nombre de
tireurs : `guide` fait toucher les cibles plus tôt (sans lui : 1 080 balles).
Les champions de `bench_floor_isaac` sont posés (la chance `0,1 + 0,15 × étage` n'en tire aucun sur la graine 1) ; leurs
explosions pèsent aussi dans la mesure.

### Mesurer

- `make bench` (strict) : une suite complète en un processus, ~40 à 52 minutes sur le poste VPS ; la mesure vaut sa charge :
  noter `uptime` au départ et à la fin (le processus de bench pèse lui-même ~1,1). Relevés à 2,7 de charge : jusqu'à 2 fois plus lents qu'à 0,1.
- `ALACOD_SYSTEM_PROFILE=1 ALACOD_SCENARIO=<nom>` : le runner imprime désormais `profil sets : …` (temps par `RollbackSystemSet` et
  tranche `FlowField`, frames comptées en simulation + resimulation du synctest) juste avant la ligne du scénario ; `snapshot_heaviest`
  est déjà dans les métriques.
- Attribution mesurée de `bench_tears` (rollback ×12 = 56 ms sur le VPS) : `Projectiles` 2,29 ms par frame sur 3,9 (~5 µs par balle et par
  frame) ; sans `division` : 39 balles, 18,9 ms ; sans `guide` : 1 080 balles, 78,9 ms. Dette D97, **réduite** (ci-dessous).

### D97 : coût des larmes empilées (dette-d97-cout-larmes)

Rapport : `docs/taches/rapports/dette-d97-cout-larmes.md`. **Aucune trace ne bouge** ; Mac M1 Pro partagé, mesures relatives (binaires A et B
alternés) : pire rollback de `bench_tears` **24,3 → 11,2 ms (−54 %)**, simulation 1,94 → 0,84 ms par frame, `save`/`load` inchangés
(1,1 / 1,0 ms), population inchangée (432 balles). Le profil échantillonné (`sample` de macOS) montrait trois coûts :

- **guidage** (`projectile_steering_system`, 40 %) : tri de tous les personnages et quatre racines carrées *par balle* ;
- **`FixedVec2::length_squared`/`length`** (22 %) : un aller-retour `f32` par composante (arrondi à 24 bits significatifs : D111) et une racine bit à bit ;
- **collision** (16 %) : chaque balle balayait `CollisionGrids::characters`, qui contient aussi les 432 balles.

Règles qui en restent :

- `CollisionGrids::targets` = `characters` **sans les balles**, reconstruite avec lui (un seul tri) : à employer pour toute requête dont le `.get()` exige
  `Without<Bullet>`, `Team` ou `Health` (balles, projectiles composables) ; `characters` reste pour ceux qui comptent aussi les balles (`move_characters`,
  fenêtres des ennemis). Les entrées portent le `GgrsNetId.0` seul, **triées** : `iter_overlapping` n'alloue ni ne retrie.
- Un système qui itère des projectiles sans que l'ordre change le résultat (chaque balle n'écrit que son propre état et ne lit que des personnages) n'a pas à passer par
  `order_mut_iter!` : trier 432 balles par frame est un coût ; la règle d'ordre de `CLAUDE.md` vaut pour tout ce qui alloue des `GgrsNetId`, tire au RNG ou émet
  des événements (`projectile_collision_system`, `projectile_expire_system`, `bullet_rollback_system` restent triés).
- `nearest_target` est un minimum (distance, `GgrsNetId`) : il ne dépend pas de l'ordre d'itération, donc pas de tri préalable des cibles.
- `FixedVec2`/`FixedVec3` `length_squared`, `length`, `normalize`, `normalize_or_zero` sont en entiers, **au bit près** de l'ancien code (oracle gardé dans les tests de
  `bevy_fixed`) : ne pas « corriger » l'arrondi à 24 bits sans `BLESS` global (D111). `FixedVec2::length_and_direction` = `(length, normalize_or_zero)` avec une seule racine.
- Mesure de coût d'un composant rollback (clone, hash) : une sonde jetable à la population maximale (404 balles : clone `Projectile` 71 µs, `Bullet` 34 µs, hash 121 et 65 µs,
  soit 291 µs de `save`) ; `save`/`load` sont surtout `bevy_ggrs` (un système par type, ~100 types × 506 porteurs).

### Budgets par poste

`tests/budgets.ron` : 49 entrées recalibrées par D84 (2026-10-18, VPS Debian 8 cœurs, trois `make bench` strict + un quatrième vert d'un seul tenant) : plancher = moitié de la pire mesure (arrondie au-dessus), plafond de rollback = double. Les scénarios `generated/<jeu>/<nom>` se désignent par leur **id complet** (une clé nue ne correspond à rien et retombe sur le défaut). La charge 1 min du bench seul est de 2 à 3 (pointes 14) : le critère « `uptime` < 1,5 » ne se tient pas, on joint la charge **hors bench** (`rapports/m3-t15-d84-charge/`). Règle : plancher = moitié de la pire mesure, plafond = double,
jamais plus de 2 × la mesure au calme ; on ne relâche pas un seuil pour passer.



## 81. HUD `isaac` : cœurs, compteurs, objet actif, bandeau d'objet, minicarte typée (M3-T16, voie V4)

Présentation seule : `game::ui::{hud, hud_model}`, `content::ui`, `games/isaac/assets/ui/hud.ron`. **Aucune trace ne
bouge** (aucun état rollback, aucun système de la simulation touché ; les deux scénarios neufs ont leur propre trace).
Rien de spécifique à `isaac` dans le code : un jeu reprend ces sources en les posant dans son `hud.ron`.

### Sources nouvelles (liste fermée `content::ui::HUD_SOURCES`, lint `UnknownKind`)

| Source | Rendu | Attente `HudText` |
|---|---|---|
| `hearts` | widget `Hearts(source)` : rangée de carrés (côté `size.1`), un par cœur, **rouges** (vides compris, jusqu'à la vie maximale) puis **d'âme**, rempli à moitié pour un demi-cœur, passe à la ligne à la largeur `size.0` ; texte `Text(source: "hearts")` : « 2,5 / 3 · âme 1 » | `isaac_hud_items` (f3 « 3 / 3 », f30 « 2,5 / 3 », f100 « 2,5 / 3 · âme 1 »), `isaac_reliquaire` (« 3 / 3 · âme 2 »), `curse_hides_health` (`empty: true`) |
| `coins` | `Counter(source)` : carré `icons.coins` + nombre (sous du joueur) ; la rangée est masquée sans valeur (D22). Alias de `shells` (même valeur), gardé pour l'icône par source : dette, à fusionner si `shells` disparaît des HUD | `isaac_hud_items` (« 12 ») |
| `keys`, `bombs` (existantes) | `Counter` : carré `icons.keys` / `icons.bombs` + nombre | `isaac_hud_items` (« 2 », « 3 ») |
| `item_banner` | `Text` sur fond, centré en haut : nom de l'objet ramassé pendant 3 s | `isaac_hud_items` (vide f3, « Bottines légères » f30, vide f230), `isaac_reliquaire` (« Reliquaire » f20, « Pouls » f60), `isaac_banner_blind` (« ??? ») |
| `active_item` (existante) | `Icons(source: "active_item")` : le carré de l'objet actif (couleur et lettre de `icons[<id>]`) | `isaac_hud_items` (« Calice d'âme ») |
| `active_charge` (existante) | texte « 30 / 30 · prêt » + jauge | `isaac_hud_items` (« prêt ») |
| `minimap` (existante) | lettre et bordure teintée par type de salle (`room_icons`) | `isaac_hud_items` (« tresor · inexplorée », « boss · boss · inexplorée ») |
| `curses` (existante, §74) | texte « Malédiction : blind » | `isaac_hud_items` (`empty: true` sans curse), `curse_*` |
| `ammo` (existante) | **vide pour une arme `Infinite`** (ni chargeur ni réserve : l'ancien « 0 \| 0 » des larmes) | `isaac_hud_items` (`empty: true`) |

**Cœurs** (`HudHearts`, `HudHearts::slots`) : `Health` et `ExtraHealth` (§68) en points de vie, un cœur valant
`items::HEART_POINTS` = 2 points (Q2 : un demi-cœur = 1 PV). C'est une convention du moteur (`heart_label` des prix en
cœurs la partage), pas un réglage du RON. Couleurs : `icons.heart`, `icons.soul_heart`, `icons.heart_empty` de `hud.ron`
(défauts dans `heart_colors`). La curse `HideHealth` vide `hearts` comme `health`. **Pas de plafond de 12 cœurs** : la
fiche de M3-T3 (§68) renvoyait à T16 « le plafond de 12 cœurs d'Isaac » ; borner le gameplay n'est pas de la
présentation, donc T16 n'en pose aucun (`ExtraHealth` et `Health.max` restent non bornés) et la rangée **passe à la
ligne** au lieu de tronquer. Un plafond reste à décider si William le veut, côté simulation.

**Bandeau d'objet** (`ItemBannerState`, `track_item_banner`, `Update`, ressource hors rollback) : dérivé de l'**état
d'inventaire** du joueur affiché, jamais d'un événement de la simulation : un passif dont le nombre d'exemplaires
augmente, ou un actif qui change, depuis l'`Update` précédent, annonce l'objet pendant `ITEM_BANNER_FRAMES` (180 frames)
**depuis la frame du ramassage** (`frame - from`). Les objets reçus dans les `ITEM_BANNER_START_GRACE` (5) frames qui
suivent la première observation d'un joueur (objets de départ, `PlayerScript::items`) ne sont pas annoncés ; un
consommable (clé, bombe, cœur) n'est jamais annoncé. Le nom vient de `ItemTable`, remplacé par « ??? »
(`HIDDEN_ITEM_NAME`) sous `CurseHides::item_names` ; la source est vide sinon. Limite : un rollback qui retire puis
redonne l'objet le réannonce, et un objet annoncé qui n'est plus tenu (prédiction annulée) éteint le bandeau ; un
joueur sans `Inventory` (créé au premier ramassage) est suivi avec un inventaire vide : son premier objet est annoncé ; la mémoire (`ItemBannerState`) n'est pas rollbackée, comme `MinimapVisits` (D72).

**Prompts** (`ui::hud::Prompt`, hors snapshot : testés unitairement, pas par `HudText`) : dette de T9 (§74) — un objet
au sol (`InteractionType::Item`, composant `items::ItemPickup`) n'avait aucun prompt, donc aucun nom à cacher ; il
affiche « Ramasser Reliquaire » (« Ramasser ??? » sous `HideItemNames`, `Prompt::hide_name`). Dette de T5 (§70) : la
porte à clé affiche les **clés du joueur** : « Ouvrir — clé (2) » avec au moins une clé, « Fermé — clé (0) » sans
(comme « Fermé — 1 clé » du coffre). Le coût `n` de la porte (`DoorLock::Key(n)`) n'est pas affiché : le HUD ne connaît
pas la salle de la porte ; `isaac` n'a que `Key(1)`.

**Minicarte** : la géométrie, les salles visitées/actuelle (bordure or épaisse) et la salle secrète n'apparaissant
qu'ouverte (§70) existaient ; T16 ajoute `room_icons` à la racine de `hud.ron` (type de salle → `(color, label)`) :
la lettre est écrite dans la salle (`T` trésor, `S` boutique, `B` boss, `?` secrète, `D` diable pour `isaac`) et la
bordure teintée (la salle courante garde l'or) ; une salle verrouillée ajoute « × ». Sans `room_icons`, comportement
d'avant (`B` pour le boss, `×` verrouillée) : `gungeon` et les autres jeux ne changent pas. `boutique` existe dans
`isaac` depuis M3-T11d1 (§76.4) : l'icône déclarée par T16 s'affiche.

### Scénarios et captures

`isaac_hud_items` (6 PV, 12 sous, `calice_ame`, 2 clés, 3 bombes ; bottines ramassées f6, `suiveur` + `mouche` au contact
dès f7, cœur d'âme ramassé f60), `isaac_banner_blind` (« ??? » sous `blind`), `isaac_reliquaire` (§76.2, attentes ajoutées).
Captures `play_scenario --capture … --at` et vidéo : `docs/captures/m3-t16-hud-isaac/`, rapport
`docs/taches/rapports/m3-t16-hud-isaac.md`.

## 82. Feedback v3 : larmes lisibles, familiers, champions, bombes et explosions (M3-T17, voie V4)

Complète §31 et §55 : mêmes règles. **Tout est présentation** : rien de rollback, rien dans le checksum ni
dans les traces (aucune trace n'a bougé : suite complète sans `BLESS`), rien dans `GgrsSchedule`, aucun crate
de simulation n'importe du rendu (`make check_sim_boundary`). Chaque effet est **dérivé de l'état** (composants
`Projectile`, `Bomb`, `FamiliarOf`, `Variant`, ressource `DoorLocks`) ou des `FrameEvents` **transitoires** que
la simulation émet déjà (`Exploded`, `RockBroken`) : un rollback qui rejoue la même situation redonne les mêmes
indices. Code : `game::feedback_v3` (fonctions pures et journal), `game::feedback_log` (journal v1/v2), `game::feedback` (rendu des explosions, teinte
des champions), `game::world_sprites` (larmes, bombe, rocher), `game::character::visuals::orient_familiars`.

### Journal et moments clés

`feedback_v3::record_v3_cues` (`PostUpdate`, après `record_feedback_cues`, headless compris) remplit
`FeedbackLog` ; sept genres de plus dans `FeedbackKind` et un champ `note: Option<String>` dans `FeedbackCue`.
`scenario::events` en fait des moments clés `feedback` (`feedback_v3::cue_label`) :

| Genre | Quand | Libellé |
|---|---|---|
| `Tear` | première frame d'une larme à modificateurs (`TearLook::is_plain` faux) | « larme 15 (guidée, division) » : liste des traits, dans l'ordre guidée, spectrale, orbite, division, perforante, statut, génération n |
| `Familiar` | un familier apparaît (`FamiliarOf`) | « familier 14 (suiveur, porteur 8, rang 0) » ; « orbital » pour `Orbit` |
| `Champion` | un personnage à `Variant` dont la variante a un `tint` apparaît | « champion 15 (explosif) » |
| `BombFuse` | une `Bomb` apparaît | « mèche 17 (19 frames) » (frames restantes à la pose) |
| `Explosion` | un `Exploded` | « anneau d'explosion (rayon 48) » |
| `RockBroken` | un `RockBroken` | « rocher cassé (19,44) » (case de grille) |
| `SecretWall` | une salle entre dans `DoorLocks` | « mur secret ouvert (<iid de la salle>) » |

Une larme **plate** (aucun trait) ne produit rien : le contenu d'avant garde ses journaux. La mémoire (ids déjà
vus, salles déjà ouvertes) est remise à zéro quand le compteur de frames recule (nouvelle partie) ; un
rollback qui retombe avant l'ouverture d'un mur la retire de `DoorLocks` et la mémoire suit. Limites de §31
reprises : un tick qui simule plusieurs frames ne journalise que les `FrameEvents` de la dernière ; un indice
prédit puis annulé par un rollback p2p reste dans le journal.

### Larmes

`feedback_v3::tear_look(projectile, statuts, base)` (pure) : **couleur** = celle du premier statut posé par l'`on_hit`
(`Chance` déballée, `StatusLibrary` : brûlure orange, poison vert, gel bleu, peur violet, lenteur, étourdissement),
sinon turquoise si guidée, sinon la couleur d'équipe d'avant (jaune joueur, rouge ennemi) ; **opacité** 0,5 si
`Spectral` ; **taille** (facteur sur `custom_size`, déjà agrandi par `Size` : `Projectile::size`) : ×1,25 `Split`,
×1,15 `Pierce`, ×0,8 par génération (plancher 0,5). `world_sprites::style_tears` l'applique **une fois** par larme,
après `skin_bullets` (qui pose l'image et la couleur d'équipe). Une larme plate n'est pas touchée.

### Familiers et orbitaux

Sprites provisoires par l'entrée `compagnon`/`orbital`/`rempart` de la table `SpriteSheet` du jeu (visuels
`character::visuals`, comme les ennemis : un jeu sans entrée garde le rendu d'avant, aucun). **Orientation**
(`orient_familiars`, après `set_sprite_flip`) : un suiveur regarde du côté où va son porteur, à défaut vers lui
(`follower_faces_left`) ; un orbital tourne sur lui-même, la pointe vers l'extérieur du cercle (`outward_angle`).
Seuls les calques à image sont touchés (pas la barre de vie).

### Champions

`feedback::tint_champions` pose `ChampionTint(Option<Color>)` (présentation, une fois) d'après `VariantDef::tint` de
la variante (`#rrggbb`, lu dans `CharacterConfig::variants` par `feedback_v3::variant_tint`) ; `apply_layer_colors`
l'utilise comme **couleur de base** des calques, devant `MarkerTint` (D66) ; statuts et flash passent encore devant.
Les variantes du testbed ont une teinte (sauf `nu`, le témoin).

### Bombe, explosion, rocher, mur secret

- **Mèche.** `world_sprites::sync_world_sprites` : l'enfant-sprite de la `Bomb` (entrée `bomb`) clignote selon les
  frames restantes `fuse_until - frame` : demi-période `clamp(restant / 4, 2, 8)` frames (`blink_half_period`), rouge
  (1, 0,25, 0,2) en phase allumée, gris sinon, taille jusqu'à +25 % sur les 30 dernières frames (`fuse_blink`).
- **Explosion** (`ExplosionFxConfig`, `explosion:` de `ui/feedback.ron`, optionnel, actif par défaut) : deux anneaux
  (plein rayon et 0,6 plus clair) sur `ring_frames` (18), orange ; secousse de `shake_frames` (10) d'amplitude
  `shake_amplitude` (8) **pour un rayon de 100**, proportionnelle au rayon, bornée à [1, 3 × l'amplitude]
  (`explosion_shake`). Valable pour la bombe, le baril et l'explosion d'un champion (même `Exploded`).
- **Rocher cassé.** Sprite de rocher (entrée `rock`), puis six éclats par rocher (`debris_offset` : directions
  réparties, vitesse selon l'indice, freinage ; `debris_alpha` : pleins puis estompés sur 24 frames), dérivés de
  l'âge en frames de simulation, détruits à l'échéance. La **tuile** de la carte (couche IntGrid `Rocks`) est dessinée
  ssi un `world::Rock` vivant est à moins de 4 unités de son centre (`map_ldtk::plugins::sync_rock_tiles`, chaque tick,
  **dérivée de l'état**, jamais d'un indice : juste quand un tick simule plusieurs frames ou quand un rollback annule
  la casse ; seules les tuiles non nulles de la couche `Rocks` sont touchées).
- **Mur secret.** Au tick où la salle entre dans `DoorLocks`, nuage de douze éclats ocre et anneau de poussière de rayon
  48 à l'endroit de l'explosion de la même frame (sans explosion, seul le journal change).

Lint (`FeedbackSettings::problems`, `OutOfRange`) : `explosion.ring_frames` dans `[1, 600]`, `shake_frames` dans `[0, 600]`,
`shake_amplitude >= 0`, couleur dans `[0, 1]`. Fixtures `feedback_explosion_frames_zero`,
`feedback_explosion_amplitude_negative`, `feedback_explosion_shake_frames_too_long`,
`feedback_explosion_color_out_of_range`.

### Assets provisoires

`games/testbed/assets/sprites/feedback/` : `tear`, `bomb`, `rock`, `companion`, `orbital`, `champion` (PNG), feuilles
`*_sheet.ron`, `anim_still.ron` ; entrées `bullet_player`, `bullet_enemy`, `bomb`, `rock`, `compagnon`, `orbital`,
`rempart`, `elite_*` de `sprites/sprites.ron`. **Images originales du projet**, dessinées par
`scripts/generer-sprites-feedback.py` (déterministe, CC0, `assets.yaml`), en niveaux de blanc pour que la teinte du
rendu se voie ; statut `placeholder`, à remplacer par la voie V6. Les autres jeux n'ont pas ces entrées : rendu d'avant.

### Preuve sans écran

Attentes `Event(kind: "feedback", label_contains: …)` sur **11 scénarios** (14 attentes) : les genres qui prouvent une
**logique de feedback propre** : `Tear` (`tear_look` : `tears_*`) et `Champion` (teinte résolue par `variant_tint` :
`champion_*`), commentaire `// M3-T17` en tête de chacun, **traces inchangées**. Les genres qui ne font que **redire un
événement d'état** (`Explosion` = `Exploded`, `SecretWall` = `DoorLocks`, `Familiar` = `FamiliarOf`, `BombFuse` = `Bomb`,
`RockBroken` = `RockBroken`) sont journalisés (moments clés des vidéos) mais **ne sont plus attendus** : ils seraient
tautologiques. Le rendu qu'ils pilotent (anneau conditionné à `explosion.enabled` et au brouillard, éclats, secousse,
clignotement, orientation) n'a pas de preuve headless : preuve = captures et vidéos
(`docs/taches/rapports/m3-t17-feedback-v3/`). Unitaires : `game::feedback_v3` (look d'une larme, mèche, secousse, éclats,
teinte, orientation) et `map_ldtk::plugins` (tuile d'un rocher : retirée puis ressuscitée).

### Limites

- Aucun son d'explosion ni de mèche (aucun asset licencié retenu : `sounds` n'a pas de clé).
- L'orbital et l'élite sont visibles dans le testbed seulement ; `isaac` aura ses propres entrées avec M3-T11.
- Une bombe de champion (mèche d'une frame) apparaît une frame avec son sprite et un indice « mèche (0 frames) ».
- Le mur secret n'a pas de sprite de porte propre : seule la poussière annonce l'ouverture.

## 83. Langage visuel provisoire : on doit comprendre ce qu'on voit (m2-lisibilite-gungeon, voie V4)

Constat de William sur les vidéos `gungeon_full_run` (2026-10-17) : « je ne comprends absolument rien ».
Trois causes, toutes corrigées en **présentation seule** (aucune trace ne bouge, rien dans
`GgrsSchedule`, aucun état rollback) :

1. **La caméra était figée après le hub.** `camera_control_system` ne choisissait sa cible qu'une fois ;
   la relance hub → run recrée les joueurs, la cible mémorisée n'existait plus et la caméra restait où
   le hub l'avait laissée : les vidéos ne montraient plus les joueurs (flèches rouge et bleue de
   `player_indicator_system` sur le bord). Une cible disparue est maintenant oubliée et re-choisie.
2. **Tout se ressemblait** : héros, ennemis, objets partageaient les cases de Tiny Dungeon. Un langage
   visuel **par classe d'entité** les remplace ; il est généré par `scripts/generer-sprites-gungeon.py`
   (stdlib seule, CC0, `--check`, `--preview`) dans `games/gungeon/assets/sprites/lisible/` et déclaré dans
   `sprites.ron` (bloc `// >>> généré`). Les sprites de Kenney du lot C sont retirés.
3. **Rien n'indiquait ce qui est ramassable, achetable, dangereux.** Halo, pointeur, étiquettes, légende.

### Les classes (id de `sprites.ron` = classe)

| Classe | Silhouette / couleur | Id |
|---|---|---|
| joueur | pistolero (grand chapeau, poncho), **clair** : teinté par handle (0 bleu, 1 vert, 2 jaune, 3 violet, `player_colors` de `hud.ron`) | `player` |
| bullet kin | balle de laiton ronde, jaune | `bullet_kin` |
| escouade | petite balle pointue, orange, trois traits blancs | `bullet_kin_squad` |
| shotgun kin | cartouche rouge large, culot de laiton | `shotgun_kin` |
| lanceur | corps rond violet, tube de mortier | `lanceur_kin` |
| tourelle (stationnaire) | cube d'acier sur trépied, lentille rouge | `tourelle_kin` |
| vétéran | carrure large, casque à crête, chevrons, vert olive | `veteran_kin` |
| bandana | élancé turquoise, bandeau rouge aux pans | `bandana_kin` |
| fuyard | goutte bleu pâle, traits de vitesse | `fuyard_kin` |
| cible | disque rouge et blanc sur pied | `cible` |
| boss | forteresse cramoisie 64 px, couronne dorée (`atlas_boss`) | `gatling` |
| tir ennemi | orbe **rond rouge** (taille = racine du dégât) | `bullet_enemy` |
| tir allié | **losange** blanc bleuté | `bullet_player` |
| douille | pièce jaune | `shell` |
| clé / bombe / cœur / blank | clé d'or / bombe noire à mèche / cœur rouge / disque bleu | `item/key`, `item/bomb`, `item/heart`, `item/blank` |
| objet passif / actif / consommable | carte à initiale / hexagone / fiole, teinte stable par id (`item/<id>`) | `item/<id>`, `item/passive`, `item/active`, `item/consumable` |
| arme au sol | silhouette par famille (pistolet, revolver, fusil, fusil à pompe, arc, énergie, lanceur, flammes), orange | `weapon/<id>`, `weapon` |
| coffre | bois, bandes d'or, cadenas ; ouvert : couvercle relevé (`Open`) | `chest` |
| présentoir | socle de pierre, plateau doré, étiquette | `shelf` |
| baril / table / rocher / mur fissuré | baril rouge à flamme / table de bois / bloc gris / brique fissurée | `barrel`, `table`, `rock`, `secret_wall` |
| porte | fermée (planches) / ouverte (deux montants, sol visible) / à clé (cadenas rouge, chaînes) ; 16×32 et 32×16 | `door_v`, `door_h` (`Idle`, `Open`, `Locked`) |
| portail | spirale violette et cyan 64 px | `portal` |
| halo, pointeur | disque doux blanc, chevron blanc (la teinte vient du rendu) | `halo`, `aim` |

Eau, sable et fosse restent les couleurs d'IntGrid de la couche `Surfaces` (`#3b7dd8`, `#d8c27a`, `#202020`),
nommées dans la légende. Les piques ne sont pas redessinés (surface à `hazard`, aucun jeu n'en pose).

### Données et systèmes génériques

- **`sprites.ron`** : une entrée par classe. Un objet au sol cherche `item/<id>`, puis `item/<genre>`
  (`passive`, `active`, `consumable`), puis `item` ; une arme `weapon/<id>` puis `weapon`
  (`world_sprites::item_candidates`, `weapon_candidates`). Une entité sans entrée garde son rendu d'avant
  (autres jeux inchangés).
- **`hud.ron`** (champs optionnels de `HudConfig`) : `player_colors`, `names` (id → nom des étiquettes :
  `player`, `shell`, `chest`, `chest_locked`, `chest_open`, `barrel`, `table`, `rock`, `portal`, ids de
  personnages et d'armes), `legend` (groupes `(title, column, entries: [(label, sprite, anim, tint,
  color)])`), `hide_markers` (cache les tuiles d'apparition des ennemis, repères d'auteur). Widget
  `Legend(source: "legend")`, source ajoutée à `HUD_SOURCES`. Le lint refuse un `sprite` de légende absent de
  `sprites.ron`.
- **`game::legibility`** (`LegibilityPlugin`, dans `PresentationPlugin` : absent en headless) :
  - pointeur de visée (`AimPivot`, enfant de chaque ennemi) : visée figée de l'émetteur, sinon dernière
    position connue de la cible (`EnemyTarget`), sinon cible de déplacement ; caché sans cible ;
  - halo pulsant (`Halo`, enfant) sous douilles, objets (couleur par rareté), armes, power-ups, coffres,
    présentoirs ; il respire sur `FrameCount` (image n = frame n en capture) ; masqué à l'ouverture du coffre
    ou à la vente ;
  - couleur par handle : `MarkerTint` posé sur les joueurs (relu par `feedback::apply_layer_colors`, qui
    n'efface plus la couleur des barres de vie : rouge ennemi, verte joueur) ;
  - **étiquettes** (`Label`, entités indépendantes comme `PowerUpLabel`) : nom de l'ennemi, du joueur
    (`J 1`), de l'objet, de l'arme (+ prix d'une arme murale), de la douille (`+n`), du coffre (clé, vide),
    du présentoir (nom + prix), baril, table, rocher, porte fermée ou à clé (une porte ouverte n'en a pas),
    portail (`WorldLabel`). Désencombrées (une étiquette qui en recouvre une autre monte) et masquées hors
    de la vue (`FogView::sees`). **F6** les bascule ; `ALACOD_LABELS=1` les allume au départ ;
    `play_scenario --capture` et `make videos` les allument d'office (`ALACOD_LABELS=0` les coupe) ;
  - **légende** : **F7** (`LegendOpen`) ouvre le panneau du HUD, reconstruit à chaque ouverture.
- **`world_sprites`** : `SpriteTable` (feuille et case d'un id), `SpriteRequest` (sprite demandé par id sur
  une entité de carte : le portail), portes (`DoorSprite`, état d'après `Collider` et `Interactable` :
  sans collider = ouverte, avec collider et interaction = à clé, collider seul = fermée par sa salle),
  `hide_replaced_map_tiles` (la tuile de l'entité LDtk d'origine devient transparente quand son entité
  rollback porte l'image du monde). Projectiles : une feuille déclarée `own_colors: true`
  (`SpriteSheetConfig`, gungeon) porte sa couleur (pas de teinte d'équipe) et sa taille suit le dégât
  (`5 + 1,6·√dégât`, bornée à 8..22) ; sans le champ (défaut), teinte d'équipe et halo agrandi comme avant
  (testbed, isaac, throne, zombies inchangés) ; `style_tears` (M3-T17) s'applique ensuite. Une table ou un
  baril (entités rollback sans `Transform`) pose son image sur l'entité LDtk d'origine (`MapHeld`) et la cache à sa destruction (`reap_held_sprites`) ; l'image d'une porte ouverte reste
  `Visible` malgré le parent caché par `interaction_view::update_door_visuals`.
- **Composants non rollback posés sur des entités rollback** : `AimPointer`, `PickupHalo`, `DoorSprite`,
  `WorldSprite`, `MarkerTint` (joueurs) : tous hors checksum et hors snapshot, recréés pour une entité
  ressuscitée ; les enfants (pointeur, halo) se cachent quand leur parent est désactivé.
- **Dettes assumées (écarts à la fiche)** :
  - `AimPointer`, `PickupHalo`, `DoorSprite` (et `WorldSprite`, `MarkerTint`) sont posés sur des entités
    rollback, hors checksum et snapshot (comme `WorldSprite` avant eux) ; un composant de présentation sur
    une entité rollback n'est pas un état, mais la fiche demandait de n'en poser aucun nouveau.
  - Couleurs de catégorie du halo, du pointeur et des étiquettes en dur dans `legibility.rs` (voie V6 :
    les passer dans `hud.ron`).
  - Piques non dessinés (aucun jeu n'en pose ; le sprite `spikes` est généré mais non branché).
  - Coût des étiquettes : `gather_labels` et `sync_labels` cherchent linéairement (`wishes.iter().find`),
    `declutter` est quadratique ; non mesuré, à mesurer au-delà de ~50 étiquettes (salles d'arène, étage
    entier visible) avant d'indexer par entité.
  - Les images de porte, de table et de baril sont des enfants de l'entité LDtk persistante (l'entité
    rollback n'a pas de `Transform`) : leur visibilité est **héritée** du monde de l'étage (`Inherited`,
    jamais `Visible`) et `reap_held_sprites` les cache quand l'entité rollback propriétaire n'existe plus
    (changement d'étage, création annulée par un rollback). Test : `world_sprites::tests`.

### Preuves et vidéos

`play_scenario --capture <dir> --at 42,157,…` ne capture que ces frames (et s'arrête après la dernière) ;
`--legend` ouvre la légende sans étiquettes. `scripts/scenario-video` rend, pour chaque scénario dont le jeu
déclare un widget `Legend`, l'**image de légende** (frame 40, par le moteur) et la place en tête de la
vidéo pendant `LEGEND_SECONDS` (1,2 s) : `ffmpeg` sans `drawtext` suffit (concat, aucun texte incrusté par
ffmpeg). La page de revue lit `<nom>.legend` (secondes de tête) et compte frames et moments clés depuis la fin
de la légende. Seul `render` ajoute la légende (pas `views` ni `compare`). Rapport : `docs/taches/rapports/m2-lisibilite-gungeon.md`.

### Ajouter une classe

1. Dessiner le sprite dans `scripts/generer-sprites-gungeon.py` (primitives `Img`, contour automatique), le
   déclarer dans `PERSOS` / `world_sprites()`, relancer le script, `--preview /tmp/apercu.png` pour le voir.
2. Ajouter la ligne de légende dans `ui/hud.ron` et, si besoin, un nom dans `names`.
3. `python3 -m unittest scripts/tests/test_sprites_gungeon.py` (sprites à jour, CC0 dans `assets.yaml`,
   légende résolue, silhouettes distinctes). Un autre jeu fait de même avec ses propres entrées : le code
   ne connaît aucune classe, seulement des ids.

**Remplacement par la voie V6** : un fichier `png` (et sa feuille) remplace la classe ; l'id de
`sprites.ron` ne change pas.

### Murs dessinés (m2-salles-murs-gungeon, présentation seule)

Constat de William (revue M2, 2026-10-18) : « les murs n'ont pas de sprite » (sol gris, extérieur sombre, aucun
mur). Cause : les gabarits LDtk ne portent **aucune tuile** de mur (`autoLayerTiles` vide : la couche IntGrid
`Walls` est écrite par script, puis l'assembleur de salles la modifie : ouvertures murées, cases de sol), donc
des règles d'auto-couche LDtk n'auraient rien à habiller. **Choix : auto-tuile en présentation**, dérivée des cases
de la carte *assemblée* (`map_ldtk::wall_art`, `LdtkRoguePlugin`) : correcte pour toute carte générée, aucun
fichier de carte à retoucher, aucune trace qui bouge (rien dans `GgrsSchedule`, aucun état rollback).

- **Activation par donnée** (`ui/hud.ron`, champ optionnel `wall_art`, `game::ui::hud::WallArt`) :
  `wall_art: Some((layer: "Walls", atlas: "sprites/lisible/atlas_murs.png", hide_layers: ["LevelConnection"]))`
  (`tile` 16, `columns` 8, `z` 1,5 par défaut). Sans le champ, ou sans rendu (headless : `HudConfigHandle` absent,
  le système ne tourne pas), rien ne change : zombies, throne, isaac, testbed identiques. Le lint (`hud_wall_art_missing`)
  refuse une feuille absente, un `tile` ou un `columns` nul ; `wall_cells` ignore une couche qui n'est pas une grille `largeur × hauteur`.
- **Système** `wall_art::draw_walls` : à l'apparition d'un niveau (entité `LevelIid` avec ses couches), lit la couche
  `layer` dans le projet (`Assets<LdtkProject>`, monde de l'étage par `FloorSlots`), choisit une tuile par case non
  nulle et pose un sprite par case sous une racine `WallArtRoot`, **enfant du niveau** (il hérite de sa visibilité :
  étage quitté caché, comme les portes). Un niveau déjà dessiné se reconnaît à son enfant `WallArtRoot` (pas de marqueur à tenir à jour). Hors du niveau, une case compte comme un mur. Le lint vérifie que `layer` est une couche IntGrid d'au moins une carte du jeu.
- **Choix de la tuile** (`wall_tile`, pure, testée) : vue de dessus avec **face de mur** côté sud. Case dont la voisine
  du sud est un sol : **face** (briques ; chapeau latéral côté ouest/est si la voisine est un sol : 4 tuiles, indices
  32 à 35) ; sinon **dessus** (liseré clair + contour sombre sur chaque bord qui touche le sol, nord/est/ouest ; angle
  rentrant nord-est/nord-ouest quand les deux voisines sont des murs mais pas la diagonale : 8 × 4 tuiles, indices 0 à 31).
- **`hide_layers`** : cache les couches de métadonnées que LDtk peint par défaut. `LevelConnection` (ouvertures des
  gabarits, valeur de couleur noire) se voyait en noir par-dessus le sol dès que la porte ouverte laissait voir la case.
- **Sprites** : `scripts/generer-sprites-gungeon.py` (CC0, même style et même script que le reste de §83) écrit
  `atlas_murs.png` (36 tuiles de 16 px). Les **portes** (`door_v`/`door_h`) gagnent deux jambages de pierre aux bouts
  (les couleurs de la face de mur ; ouverte = les jambages seuls), le **mur fissuré** (`secret_wall`) reprend les briques
  du mur et sa fissure claire : tous trois se raccordent aux murs.
- **Limite** : le sol reste la couleur de fond du niveau (`#696A79`), sans motif ni ombre portée du mur.

### Murs, portes, rochers et piques d'`isaac` (m3-salles-murs-isaac, présentation seule, aucune trace ne bouge de ce fait)

Même mécanisme que ci-dessus (**aucun second mécanisme**) ; seuls changent la palette, les sprites et trois extensions génériques, activées par donnée.

- **`ui/hud.ron`** d'`isaac` : `wall_art: Some((layer: "Walls", atlas: "sprites/lisible/atlas_murs.png", hide_layers: ["LevelConnection", "Rocks", "Surfaces"], cells: [(layer: "Surfaces", value: 5, tile: 36)]))`,
  `hide_markers: true` (les tuiles d'apparition des ennemis : repères d'auteur) et `names` (étiquettes F6 : `shell` « Sous », `chest`, `rock`, `portal`).
- **Générateur** : `scripts/generer-sprites-isaac.py` **importe** `scripts/generer-sprites-gungeon.py` (primitives, PNG, atlas, `wall_cap`/`wall_face`, objets) et remplace les couleurs : palette « sous-sol » (face de brique brun-rouge `#4e3228`, dessus `#34231d`,
  sol `#7B6A5F`), distincte du pourpre de gungeon. `--check`, `--preview`, test `scripts/tests/test_sprites_isaac.py` (fichiers à jour, déterminisme, aucun orphelin, CC0 dans `assets.yaml`, palette distincte, tuiles lues par le moteur).
  Sorties sous `games/isaac/assets/sprites/lisible/` et bloc `// >>> généré` de `sprites.ron` : `atlas_murs.png` (36 tuiles de mur + **tuile 36 = piques**), `atlas_porte_v.png` (16×64) et `atlas_porte_h.png` (64×16) à **quatre états** (`Idle` planches cerclées de fer,
  `Open` jambages seuls, `Locked` chaînes et cadenas rouge, `Secret` mur de briques fissuré), `atlas_monde.png` (32 px : `shell`, `chest` (`Idle`/`Open`), `shelf`, `rock`, `rock_loot`, `bomb`, `item/<id>` pour les 100 objets d'`objets/*.ron` (cartes, hexagones, fioles, cœurs rouge, d'âme…), `item/passive|active|consumable`), `atlas_portail.png`.
  Les personnages (héros, ennemis, boss) gardent leurs tuiles Kenney (`sprites/gungeon/`) : ce ne sont pas des objets « sans sprite ».
- **Trois extensions génériques** (les autres jeux ne changent pas) :
  1. `wall_art.cells: [(layer, value, tile, z)]` (`game::ui::hud::CellArt`) : une tuile fixe de la feuille par valeur d'une autre couche IntGrid (`wall_art::value_cells`, pure, testée). Lint : la couche d'une case annexe doit exister
     dans une carte (fixture `hud_wall_art_cell_layer_unknown`). Les piques d'`isaac` (§69, `Surfaces` valeur 5) sont dessinées ainsi ; la couche d'origine est cachée par `hide_layers` (rendu IntGrid colorié de LDtk).
  2. **Rochers** : l'image du rocher est celle de l'entité (`world_sprites`, id `rock`), un **rocher à butin** (`world::Rock::loot`) cherche d'abord `rock_loot`, puis `rock` ; la couche `Rocks` (tuiles coloriées de LDtk) est cachée par `hide_layers`. Un rocher détruit disparaît avec son entité (inchangé).
  3. **Mur secret** : `game::visual_requests::SecretDoor`, posé en présentation (`map_ldtk::game::rooms::mark_secret_doors`, `Update`, hors `GgrsSchedule`, mêmes portes que `secret_wall_system` : la porte d'une salle `door_lock: Secret` et sa jumelle) ;
     `world_sprites::sync_door_sprites` montre l'état `Secret` d'une porte fermée sans `Interactable` marquée `SecretDoor` (repli `Idle` si la feuille n'a pas l'état : gungeon inchangé), `Open` dès que la bombe retire le collider. Le mur secret
     avait pour seul signe la poussière (§82, « n'a pas de sprite de porte propre ») : il se lit maintenant comme un mur fissuré.
- **Limites** : le sol est la couleur de fond du niveau (`#7B6A5F`), sans motif ni ombre portée ; rochers de 16 px dans des salles de 480×288 (un rocher reste une case de 16 unités, plus petit que le héros de 32 px : le nombre de rochers est le contenu des gabarits) ;
  le sprite `chest` montre un cadenas même sur un coffre non verrouillé (les coffres du trésor et de la secrète d'`isaac` ont `locked: false`) ; halo, pointeur de visée et légende (F7) n'existent pas pour `isaac` (langage visuel des classes d'ennemis : seul gungeon l'a). Rendu jugé par un **rendu synthétique**
  (`docs/taches/rapports/m3-salles-murs-isaac/rendu_*.png`, même logique de tuiles que le moteur, pas une capture du jeu) : captures impossibles sur le VPS (D94).

## 84. Éditeur par l'IA : mode éditeur, commandes de partie, `Registry` hors lobby (K5-1, K5-2)

Guide d'usage : `docs/editeur-vocal.md`. Plan : `docs/taches/plan-editeur-vocal.md` (D1 à D7). K5-1 n'ajoute aucun système dans `GgrsSchedule` ni ressource rollback ; K5-2 ajoute deux systèmes
**inertes sans commande** (`run_if`) et une ressource **non rollback** (`GameCommands`) : aucun type rollback nouveau, aucune parité de types vides, traces inchangées.

- **Commandes** (K5-2) : `game::game_commands::GameCommands` = `BTreeMap<frame, Vec<GameCommand>>`, **non rollback, hors checksum, jamais dépilée** (une file dépilée perdrait le spawn à la resimulation) ;
  vidée à `OnEnter(GameLoading)`. `alacod/spawn_character|powerup|item` valident l'id, écrivent à `frame + 1` et ajoutent le placement équivalent à l'`InputRecorder` (`record_command`).
  Les systèmes `map_ldtk::game::commands::{apply_game_commands_spawn (EnemySpawning, après vagues et spawners), apply_game_commands_pickups (Effects, avant ramassages)}` (non déplacés : `LevelSpawnAssets`
  vit dans `map_ldtk`) reprennent l'ordre des placements du runner et journalisent `ggrs{f=… editor_command kind=… rank=…}`. Les coordonnées f32 du JSON sont converties en `Fixed` dans le handler.
  « Devant moi » n'existe que côté script : `alacod-remote spawn … --ahead [PX]` (60 px dans le `facing` du joueur, lu dans `alacod/state`).
- **`Stats` vivantes** (K5-2) : `character::config_asset::refresh_live_stats` (`PreUpdate`, avant `RunGgrsSystems`, même garde `session_hot_edit_allowed`) réécrit sur `AssetEvent::Modified` les bases de `Stats`
  (`create::base_stats`, partagée avec `create_character`) des entités portant la clé rechargée, dans l'ordre `GgrsNetId`, par `Stats::overlay` (les défauts de création d'un ennemi restent) ;
  ni `Modifiers` ni `Health`.
- **Mode éditeur** (`game::editor_mode`, `ALACOD_EDITOR=1`, ressource `EditorMode`) : implique le remote ; `RemoteClock` démarre **en marche** ; `InputSource::Remote` n'est **pas** posé (le clavier reste la source,
  `read_local_inputs` laisse `RemoteInputs::held` prendre la main par handle, un `alacod/input` vide la rend) ; `play_check_distance_from_env` renvoie **0** même avec `ALACOD_STATE_TRACE`
  (un `ALACOD_CHECK_DISTANCE` explicite reste prioritaire). Hors mode éditeur, `ALACOD_REMOTE=1` se comporte comme avant (pause au départ, clavier coupé).
- **Garde `hot_edit_allowed(online, check_distance)`** : les écritures hors simulation (`Modifiers`, `Registry`, `RunRequest`) ne sont licites qu'en session locale à `check_distance = 0` (à d > 0, le synctest recharge
  un snapshot et perd l'écriture ; en ligne, elle diverge entre pairs). Les méthodes d'édition appellent `remote::guard_hot_edit`, qui lit `OnlineState` et `GggrsSessionConfiguration.connection.check_distance` ;
  erreur JSON-RPC `INVALID_REQUEST` qui nomme la cause (`EditorRefusal`). Ne pas l'assouplir.
- **Méthodes** : `alacod/restart {kind?, seed?}` (`RunRequest` ; `seed` écrit `MapGenerationConfig.seed`, refusé avec `ToLobby`), `alacod/set_modifier {handle?|net_id?, stat, op, value: "Fixed", source?, until_frames?}`
  (`apply_set_modifier`, `ModifierSource::Named`, défaut `editor`), `alacod/clear_modifiers`, `alacod/reload_content`, `alacod/spawn_character`, `alacod/spawn_powerup`, `alacod/spawn_item` ;
  `alacod/state` enrichi (`editor`, `online`, `check_distance`, `game`, `map`, `seed`, `run.step`, `config` des ennemis) ;
  `alacod/state` expose aussi `commands` (frames futures) et `facing` par joueur.
- **`Registry` hors lobby** (`content_hot_reload`) : le système ne tourne plus sous `in_lobby` ; il remplace `Registry` **et** `GameManifest` quand un asset sentinelle (personnages, armes, mêlée, vagues) émet `Modified`
  **ou** quand la scrutation (natif, **mode éditeur seulement**, `Instant`, 1 s : `Time` est figé par la pause du remote) voit changer l'empreinte (chemin, mtime, taille) de `game.ron` et des fichiers déclarés par `content_folders`.
  **Garde du remplacement automatique** : hors lobby, il n'a lieu que si `session_hot_edit_allowed` est `Ok` (hors ligne, `check_distance = 0`, `OnlineState` et `GggrsSessionConfiguration` lisibles) ; sinon `warn!` une fois « registre non rechargé : <cause> » (en ligne, aucun pair ne diverge). Au lobby : comportement d'avant K5-1.
  Lint rouge : l'ancien registre reste (`ERROR contenu invalide, registre non rechargé`). Aucun lecteur de `Registry` ne tourne dans `GgrsSchedule` : les tables dérivées (`StatusLibrary`, patterns, progression,
  horloges…) ne se recalculent qu'à `OnEnter(GameLoading)` : au `restart` **et au changement d'étage** (mode `Floors`) ; l'écran de mutation et l'écran de fin peuvent afficher transitoirement le contenu neuf. Limite : `GlobalAsset` lit le registre au `Startup` seulement (un personnage ou une arme **ajouté** exige une relance du binaire).
- **Tests** : unitaires `editor_mode`, `content_hot_reload`, `remote::parse_stat` ; `crates/scenario/tests/editor_reload.rs` (scrutation, événement d'asset, relance, fichier invalide ; `set_modifier` à d = 0 et cas d'erreur).

