# X-6g — Intérieur par graine (rapport)

Branche `wquintal/x-6g-interieur-graine`. Décision de William (2026-11-21), lot (2) ; jalon **D3 dim 6 déc**. Conventions : `docs/conventions.md` § 129 ; dettes D226–D227.

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

(le reste du rapport est complété au fil des points)
