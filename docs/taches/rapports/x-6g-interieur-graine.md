# X-6g — Intérieur par graine (rapport)

Branche `wquintal/x-6g-interieur-graine`. Décision de William (2026-11-21), lot (2) ; jalon **D3 dim 6 déc**. Conventions : `docs/conventions.md` § 129 ; dettes D226–D227.

**Ce qui existait déjà** : la grammaire d'étage § 40 (`FloorMapGeneration`) connaît les types de pièces (`room_kind`), pose le départ au centre, les salles requises, le boss en cul-de-sac, avec nouvelles tentatives et échec explicite ; la graine par défaut (`entry.default_seed`) rejouée à chaque relance.
**Ce qui manquait** : un nombre exact de sorties du départ, « exactement » (et non « au moins ») par type requis, deux types dans deux branches différentes ; `Basic` ne connaît pas les types et tire son départ dans la boîte (cause des défauts de 1837).
**Ce qui est ajouté** : trois champs optionnels de la grammaire (`start_exits`, `exact`, `apart`), leur contrôle `validate_plan`, leur lint ; aucun générateur neuf, défaut inchangé.

## (a) Relevé, avant tout code

**`BasicMapGeneration`** (`crates/map/src/generation/imp/basic.rs`)
- `get_spawning_room` (l. 304) : gabarit `Spawn` tiré au hasard, puis **position tirée** `x ∈ [-max_width, max_width - largeur]`, `y` idem (l. 323-330), calée sur la grille de tuiles
  (`snap_to_tile_grid`, l. 292). Le hall n'est donc jamais « au centre » : il peut toucher un bord de la boîte.
- `get_next_room_recursize` (l. 109) : parent tiré dans `rooms_possible`, prise libre tirée, **gabarit compatible tiré sans regarder son type** (`room_kind` n'est lu
  nulle part dans `Basic`), chevauchement → `DeadEnd` (D46), `Room::is_outside` → `OutSide` ; la salle n'est posée que si sa pose est dans la boîte. **Aucune garantie de contenu** :
  un type n'existe dans la carte que si le tirage le choisit ; pas de nombre de sorties exact ; pas de nouvelle tentative (une génération = un tirage).
- `Room::is_outside` (`room.rs` l. 155) ne compare que le **coin haut-gauche** à ±`max_width` × ±`max_heigth` (`world::MapSettings`, 1000 par défaut).

**Grammaire d'étage § 40** (`crates/map/src/generation/floor.rs`, `imp/floor.rs`, donnée `world::FloorGrammar`)
- `try_attempt` (l. 329) : départ **au centre** (`Position(snap(-w/2), snap(-h/2))`, l. 363) — le défaut de position de `Basic` n'y existe pas ; remplissage ; boss en cul-de-sac sur une
  salle de profondeur maximale ; salles requises sur parents de profondeur ≤ boss − 2 (l. 415) ; `validate_plan` (l. 430) ; nouvelles tentatives bornées (`plan_floor`, l. 521-550, graine de
  tentative = `rng.next_u32()` du générateur principal), `FloorError::Exhausted` explicite.
- **Manque** : `validate_plan` exige *au moins* le nombre de salles d'un type (`count(kind) < required`, l. 440), pas *exactement* ; aucune notion de nombre de sorties du départ (le départ
  prend autant de salles que le tirage uniforme sur ses prises libres en donne) ; aucune notion de branche (une salle requise peut s'accrocher sur le chemin du boss).

**Relance et graine**
- `LdtkGameMap.seed` ← `manifest.entry.default_seed` (`games/*/src/main.rs`, ex. `games/testbed/src/main.rs:60`) ; `--seed` la remplace au `Startup` (`map_ldtk::game::local::apply_launch_seed`).
  `MapGenerationConfig.seed` est lu par `configure_map` ; `RunRequest::Restart` **ne la touche pas** (`run_state.rs`, en-tête : « même carte, même graine ») ; seul `alacod/restart --seed` la remplace
  (`remote.rs` l. 517). Seul `games/zombies` tire une graine au hasard (`choose_local_seed`, politique propre au jeu). **Une graine par défaut rechargée à chaque relance existe déjà** (donnée :
  `entry.default_seed`) ; X-6g n'ajoute aucun champ, il le prouve (scénarios) et le documente. Étage assemblé : graine dérivée `graine_de_la_run + emplacement × 0x3C6E_F372` (§ 40).

**Défauts relevés par 1837** (`36ab2386`, `loge_interieur.ldtk` : 26 gabarits de 20×14 cases, `Hall` spawn ; `alacod-map --seeds 1,2,3,4,5,6` reproduit, `target/headless/alacod-map`)
- *Une seule salle du chef en six graines* : **cause = `Basic` ne connaît pas les types** (point ci-dessus). Relevé : chef (`Chef_*`) dans la graine 1 seulement (graine 4 : une salle de la clé, graine 2 : une, graines 3 et 5 : aucune des deux).
  Les trois pièces « réunion », « cuisine », « réserve » ont toutes `room_kind = combat` dans ce fichier : aucun mécanisme de type ne peut les distinguer tant qu'ils ne portent pas chacun leur `room_kind` (à changer dans **son** contenu).
- *Graine 5 à 4 salles, hall au bord haut* : **cause confirmée** = `get_spawning_room` tire y ∈ [−1000, 1000 − 224] ; la graine 5 pose le hall à y = −928 (case −58) ; la salle suivante au nord
  serait à y = −1152 < −1000 → `is_outside` → prise `OutSide` murée (`n@8+3 (murée)` sur le hall et sa voisine), et l'arbre se coupe (4 salles). Test : voir (b).


## Choix
Extension de la **grammaire d'étage § 40** (le départ y est déjà au centre ; `Basic` ne connaît pas les types), trois champs optionnels de `world::FloorGrammar` : `start_exits`, `exact`, `apart` (`docs/conventions.md` § 129).
Défaut inchangé : `Basic` n'est pas touché (dette D227, cause établie et couverte par le test `le_depart_de_basic_est_tire_jusqu_au_bord_donc_sa_voisine_peut_sortir_de_la_carte`).

## (b) Campagne (test ignoré `campagne_interieur`, `crates/map_ldtk/src/generation/mod.rs`, jugé sur les salles réellement générées)
| Contenu | Graines | Conformes | Tentatives moy / max | Plan moy / max |
|---|---|---|---|---|
| testbed `floor:interieur` | 200 | **200/200** | 8,12 / 46 | 0,25 ms / 1,2 ms |
| testbed | 1 000 | 1 000/1 000 | 8,12 / 53 | 0,25 ms / 1,4 ms |
| 1837 `loge_interieur.ldtk` (`36ab2386`), `room_kind` d'origine, types `recompense` + `chef` | 200 | 200/200 | 1,54 / 6 | 0,8 ms / 3,2 ms |
| 1837, `Reunion`/`Cuisine_*`/`Reserve_*` rangés en `reunion`/`cuisine`/`reserve` | 200 | 200/200 | 15,4 / 86 | 2,7 ms / 14 ms |

**Pour 1837** : tout est `combat` sauf hall, clé (`recompense`) et chef ; la garantie « réunion, cuisine, réserve exactement une fois » n'est possible que si ces trois gabarits portent chacun leur propre `room_kind`
(`reunion`, `cuisine`, `reserve`, avec un fichier `rooms/<type>.ron` chacun) ; grammaire : `rooms: (8, 10)`, `required` des cinq types, `boss: "chef"`, `min_boss_distance: 2`, `start_exits: Some(3)`, `exact: true`,
`apart: Some(("recompense", "chef"))`, `max_attempts: 256`. Leur hall a quatre prises (une est murée). Les gabarits `Passage`/`Angle`/`Commune` restent `combat`.

## (c) Scénarios (traces neuves, un et deux joueurs) : `interieur_arrivee[_duo]`, `interieur_parcours[_duo]`, `interieur_porte_sans_cle[_duo]`, `interieur_graine_3[_duo]` ; test `la_graine_par_defaut_rejoue_le_meme_interieur_et_une_graine_explicite_un_autre` (relance).

## (d) Capture du vrai jeu (testbed release natif, `alacod remote`, `game.ron` temporaire `Floors`/`interieur_seul`, rétabli) : `preuves/x-6g/interieur_reel.mp4` (33 s, 330 images, 123 Ko) :
hall au centre, branche nord (salle de la clé), retour, branche est jusqu'à la porte verrouillée de la finale (graine 123456). Images fixes `reel_*.png`. Rendus PNG de six graines : `preuves/x-6g/interieur_graine_{1..6}.png` (`alacod-map`).
`target/release` supprimé.

## (e) Traces : 36 scénarios des familles touchées (gungeon, grid/isaac, throne, zombies, testbed, `key_door_*`, `sortie_porte_*`) rejoués : **aucune trace modifiée, aucun BLESS** (hors 8 traces neuves).
Aucun système ni type rollback ajouté (donnée de génération et scénarios seulement) ; `scenario/schedule_order` non touché.

## (f) Contrôle rapide découpé : voir ci-dessous.
