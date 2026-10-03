SHA de tête vérifiée avant le commit de ce rapport : `581dfe1` (assets + scénario).
Fiche : `docs/taches/m0-v8-carte-zombies.md` — m0-v8, voie V6, assets seulement.
Branche : `m0-v8-carte-zombies` ; référence `main` : `05fd7f9`.
Date : 2026-10-02 ; agent : Claude Code (voie V6).

Le commit de livraison ajoute uniquement ce rapport et les captures au SHA vérifié
ci-dessus ; son SHA est donné dans la ligne LIVRÉ.

## Fait

- `games/zombies/assets/assets.yaml` (nouveau) : registre des licences de **tous** les
  binaires du jeu, packs pré-existants compris — 11 entrées : ZombieShooter
  (blackHUNTERdev, CC BY 4.0), SunnyLand (Ansimuz, CC0), Dungeon_asset (Pixel_Poem,
  licence propre, voir dettes), NuclearBlaze (deepnight, CC BY-SA), icônes Caz
  (CC BY 4.0), 3 sons (2 Pixabay, 1 freesound CC BY), Fira Mono (OFL 1.1), et les
  3 cartes LDtk (contenu interne). Chaque entrée : id, fichiers, source réellement
  consultée, auteur, licence, modifications. Aucun asset extrait des jeux de
  référence.
- `games/zombies/assets/game.ron` : l'unique exception additive autorisée,
  `(path: "maps", kind: "Map")` (+ son commentaire), rien d'autre dans le fichier.
- `games/zombies/assets/maps/avant_poste.ldtk` (nouveau, 433 Ko, LDtk 1.5.3, niveau
  unique, tuiles 16 px, 9 salles) : boucle classique Depart (spawn) → Couloir /
  Cuisine / Chambre (portes 1000) → Poste (salle à tenir, **4 fenêtres**, porte
  1250), Armurerie (2 armes murales), Cave (1250) → Cellier, Jug (**SodaLocation
  juggernog**, porte 1500). Entités : 4 `PlayerSpawn` (index 0..3), 5
  `ZombieSpawn`, 4 `WeaponLocation` (pistol 500 / machine_gun 1500 / shotgun 1000 /
  rifle 2000), 1 `SodaLocation`, 3 `CrateLocation`, 2 fenêtres dans Depart, portes
  `price`/`electrify` croissants. Générée hors dépôt (`gen_map.py`, /tmp) sur la
  structure de `exemples/test_map.ldtk` ; tilesets = ceux déjà câblés de test_map
  (SunnyLand + Dungeon_asset) via `relPath ../exemples/` — zéro câblage nouveau,
  zéro binaire ajouté.
- `tests/scenarios/avant_poste_demo.ron` (nouveau, **livré sans trace** — fiche §4,
  l'orchestrateur blesse) : 1 joueur, machine_gun (mag_limit 12), 3600 frames. Le
  joueur monte au nord de Depart (10,5)→(10,2) et tient la fenêtre ouest (tir
  continu par fenêtres de transit, pans calibrés en y-up, ~1880 frames de tir) ;
  WaveOverride : grace 150, délai 60, 2 zombies + variance(0..2) par vague,
  `max_wave: 5` → victoire du mode Waves à l'entrée en vague 5. Attentes :
  `PlayerAlive(3600)`, `WaveAtLeast(1,500)`, `WaveAtLeast(5,3600)`,
  `KillsAtLeast(8,3600)`, `RunState(Ended(Victory),3600)`,
  `RunSummary(wave_reached_min:5, kills_min:8, 3600)`.
- Aucun code d'engine, aucun `.trace`, `docs/taches.md` et `start_map` intacts.

## Vérifié

Toutes les commandes ont été précédées de `source ../env.sh` (worktree
`m0-v8-carte-zombies`), profil `headless`, `CARGO_BUILD_JOBS=4`, une invocation
Cargo à la fois.

- **`make lint`** : `games/zombies : aucune erreur (4 personnages, 4 armes, 6 armes
  de corps à corps, 1 vagues, 3 cartes)` — les 3 cartes = test_map, test_map_shop,
  avant_poste ; `games/testbed : aucune erreur` aussi.
- **`make test_scenarios`** (trace bénie localement pour la vérification, puis
  **supprimée**) : `test scenarios ... ok`, 62 scénarios, 455,5 s, 0 échec — les
  61 existants restent verts (aucune trace existante modifiée : `git status` final
  sans `.trace`).
- **Bless local** : `avant_poste_demo: frame 3600 : vague 5, 14 ennemis tués,
  joueurs vivants [0] | frames=3600 fps=87.2 entités_max=127 balles_max=7
  ennemis_max=4 (41.7s)` — les 6 attentes passent (dont `Ended(Victory)`).
- **Capture** (`../target/headless/play_scenario ... --capture /tmp/m0-v8/work/caps5
  --every 40`) : 90 PNG 960×540 + `events.json` (172 événements). Chronologie :
  vagues 1-5 arrivées f151/540/1164/1845/2386, terminées f330/954/1635/2176 ;
  **victoire f2236** (entrée en vague 5 = `max_wave 5`) ; 14 kills (dernier
  f2556) ; joueur touché 5 fois en vague 5 (f2520+), debout à f3600 ;
  rechargements f1755/2200/2465 (réserve 210→180→150). 5 captures < 100 Ko dans
  `docs/captures/m0-v8-carte-zombies/` (README avec les commandes).
- **Diagnostics de blocage** (sur la carte, via `ALACOD_NAV` nav_map/nav_stats) :
  zombie 234 bloqué au spawner (24,14) 2378 frames (« sprite dans un mur » 2437
  frames, max 10,5 px) — voisin est = mur ; après déplacement en (21,13), zombie
  232 bloqué pareil (« dans un mur » 2437 frames, max 16,0 px) — voisin nord =
  bande murale à fenêtres (rangée 12). Corrigé en (21,14) ; après correction,
  aucun zombie bloqué (0 frame), 6/9 zombies au contact du joueur. Règle retenue
  et commentée dans le générateur : les 4 voisins d'un spawner doivent être libres
  (le sprite 32×32 déborde de 8 px dans chaque voisin).
- **`cargo fmt --all -- --check`** : rien à formater (rc 0).
- **`scripts/check-rollback-registration.sh`** : OK (aucun appel direct hors
  `crates/utils/src/rollback.rs`).
- **`scripts/check-forbidden.sh`** : 4 occurrences, toutes dans des fichiers
  d'engine pré-existants (`crates/game/src/character/enemy/ai/state.rs`,
  `crates/map_ldtk/src/game/plugin.rs` ×2, `crates/bevy_fixed/src/math.rs`) —
  aucun fichier de cette tâche ; le script est un warning par défaut, pas un échec.
- **`make map_preview`** : échoue (pré-existant, voir Non fait) — erreurs exactes
  relevées ci-dessous.

## Non fait / incertain / non vérifié

- **`map_preview` ne rend pas** — l'acceptation « map_preview rend la carte » ne
  peut pas être vérifiée telle quelle : l'exemple échoue avant tout rendu avec
  `Path not found: .../alacod/assets/games/zombies/assets/exemples/test_map.ldtk`
  (l'exemple résout `alacod/assets/`, dossier inexistant) puis panique
  `Parameter Res<CollisionSettings> failed validation: Resource does not exist`
  dans `map_ldtk::game::plugin::wait_for_all_map_rollback_entity`. Les deux erreurs
  sont dans du code d'engine, hors périmètre V6 (assets seulement). **Substitut
  homologué pour le rendu** : `play_scenario --capture` (même pipeline de rendu
  960×540), captures dans `docs/captures/m0-v8-carte-zombies/`.
- **Validation 20 graines** et `--map` d'`alacod-sim` : à l'orchestrateur (hors de
  cette tâche, fiche §Acceptation).
- **Licence Pixel_Poem** (Dungeon_Tileset.png) : licence propre de l'auteur, pas
  CC0/CC-BY — l'auteur a autorisé en commentaire de la page l'hébergement dans un
  dépôt open source ; à valider par l'équipe.
- **Zombie2.png et Slash_strip3.png** : source non identifiée de façon fiable
  (absents du pack blackHUNTERdev) — à identifier ou remplacer.
- **Portes / perks / achats non exercés** : la démo défend sans bouger — aucune
  porte ouverte, aucun achat, le déblocage salle-à-salle n'est pas prouvé par une
  partie. Le flow field atteint 375 cellules (toutes les salles joignables,
  vérifié en nav_map), mais pas en jouant.
- **Suite `cargo test` complète non exécutée** sous mes yeux (seulement
  `-p scenario`) : aucun code modifié par cette tâche, mais je n'affirme pas le
  reste du workspace vert.
- La démo dure 3600 frames là où la fiche suggérait ~2000 : la victoire tombe
  f2236 (proche du ~2000 visé) et la capture continue jusqu'à f3600 pour montrer
  la vague 5 nettoyée et le joueur debout — choix assumé, 90 captures complètes.

## Dettes laissées, questions ouvertes

- `map_preview` cassé (engine, pré-existant) : à réparer hors V6 — résoudre
  `alacod/assets/` et insérer `CollisionSettings` dans l'exemple.
- Deux dettes de licence (Pixel_Poem à valider ; Zombie2/Slash_strip3 sans
  source) — consignées dans `assets.yaml`, à trancher avant tout usage officiel.
- `gen_map.py` vit dans /tmp (hors dépôt) : la carte est régénérable mais le
  script n'est pas versionné — à intégrer si la voie V6 veut itérer sur le layout.
- La bande physique pleine largeur à la rangée 17 du monde (« p » en nav_map)
  n'a pas d'explication trouvée dans l'IntGrid (propre) ni dans `collider.rs` —
  contournée par la règle des spawners (≥ 2 tuiles de tout mur), à comprendre si
  quelqu'un retouche la carte.
- La démo n'ouvre jamais de porte : une preuve de progression salle-à-salle en
  jeu reste à faire (démo suivante ou m0-v7).
