# dette-d97-cout-larmes : rapport

Branche `dette-d97-cout-larmes` depuis `origin/main` `98e65184`. Worker b0. Dette D97 (`docs/taches/dettes.md`), conventions §66 et §80.
Poste : **Mac Apple M1 Pro, 8 cœurs, 16 Go, partagé** (b1 joue les campagnes isaac de T11d1, b3 construit T14), profil `headless`
(crates du workspace en `opt-level = 1`, dépendances en 3), exécuteur monothread. **Les mesures sont relatives** (avant/après sur le même
poste, dans la même fenêtre, binaires A et B alternés, 3 à 5 relevés chacun, médiane) ; la charge (`uptime`, 1 min) est notée à chaque tableau.
Les budgets absolus (`tests/budgets.ron`) ne sont pas touchés (D84, b6 sur le VPS).

## 1. Résultat

| `bench_tears` (432 balles au pic, 506 entités) | avant (A, `98e65184`) | après (F) | écart |
|---|---|---|---|
| **`rollback_ms_worst`** (restauration + 12 frames) | 24,3 ms | **11,2 ms** | **−54 %** |
| **`advance_ms_avg`** (une frame de simulation) | 1,94 ms | **0,84 ms** | **−57 %** |
| `save_ms_avg` | 1,14 ms | 1,22 ms | inchangé (bruit) |
| `load_ms_avg` | 0,95 ms | 1,11 ms | inchangé (bruit) |
| snapshot | 217 KiB | 217 KiB | inchangé |
| fps (frames simulées par seconde, synctest compris) | 93,8 | 126,7 | ×1,35 |

Médiane de 5 relevés alternés A/F, charge 9,0 / 8,2 / 7,7 / 8,6 / 11,2 au départ de chaque tour (bruyant : pire rollback de A 19,8–25,5,
de F 7,8–12,3). Objectif de la fiche : −25 % sur `rollback_ms_worst` et `advance_ms_avg` : **atteint, plus de deux fois**. Aucune trace n'a bougé (§5).

Projection sur le poste de référence (VPS, 56,3 ms : `load` 3,57 + 12 × `advance` 4,39) **[INFERENCE, non mesurée ici]** : à rapport égal
(simulation ×0,43), `advance` ≈ 1,9 ms et pire rollback ≈ **26 ms**, sous les 50 ms (3 ticks). Le rejouer sur le VPS (b6, recalibrage D84)
confirme ou non ; D97 est donc **réduite**, pas fermée (voir §7).

### Étapes (même mesure, quatre binaires alternés, médiane de 5, charge 32 → 17 : très chargée)

| binaire | contenu | `advance` | `rollback_ms_worst` |
|---|---|---|---|
| A | `98e65184` | 1,83 | 22,8 |
| B1 | grille `targets`, requêtes sans allocation, guidage sans tri par balle, visée de `on_expire` à la demande | 1,24 | 15,9 |
| B2 | + `length`/`length_squared`/`normalize` en entiers, une racine pour la vitesse du guidage | 0,78 | 10,3 |
| B3 (= F) | + racine entière à graine flottante (corrigée) | 0,73 | 9,8 |

Relevés moins chargés (charge 3 à 10, 3 tours) : A 1,52–1,55 / 18,9–19,3 ; B1 1,01–1,06 / 12,8–13,5 ; B2 0,61 / 8,0.

### Les autres bancs ne régressent pas (A contre F, médiane de 3, mêmes conditions)

| scénario | `advance` A → F | pire rollback A → F |
|---|---|---|
| bench_bullets_room | 1,03 → 0,43 | 13,3 → 6,0 |
| bench_horde | 1,36 → 0,62 | 17,0 → 8,1 |
| bench_tears_trio | 0,30 → 0,22 | 3,9 → 2,9 |
| bench_floor_isaac | 0,25 → 0,22 | 3,5 → 3,1 |
| four_players_idle | 0,11 → 0,10 | 1,6 → 1,5 |
| bench_cave | 0,18 → 0,19 | 2,6 → 2,6 (bruit) |

## 2. Où passait le temps (profil avant)

Outils : `ALACOD_SYSTEM_PROFILE=1` (par `RollbackSystemSet`), puis `sample` de macOS (`/usr/bin/sample`, échantillon de 1 ms sur le thread de
simulation ; `xctrace` demande Xcode, `samply` n'est pas installé). Binaire A, fenêtre de 5 s, charge ≈ 10 à 30.

### Par set (µs par frame de simulation, resimulation du synctest comprise)

| set | A (calme) | A (charge 9) | F (charge ~10) |
|---|---|---|---|
| **Projectiles** | **1 080** | 1 232 | 302 |
| Weapon | 138 | 168 | 121 |
| EnemyAI | 77 | 90 | 67 |
| CollisionDamage | 64 | 79 | 77 |
| DeathManagement | 59 | 74 | 84 |

### Par système (échantillons du thread de simulation, même fenêtre de 5 s, avant → après)

| système | A | F |
|---|---|---|
| `projectile_steering_system` | **369** | 79 |
| `projectile_collision_system` | **148** | 46 |
| `bullet_rollback_system` | 31 | 19 |
| `apply_effects_system` | 17 | 21 |
| `apply_projectile_on_hit_system` | 10 | 15 |
| total occupé du thread (save/load compris) | 912 | 649 |

### Les ~5 µs par balle et par frame (VPS), élément par élément

1. **Pilotage (`Homing`) : 40 % des échantillons.** Pour *chaque* balle guidée, `order_iter!(target_query)` rassemblait et triait tous les
   personnages (≈ 40 : joueurs, familiers, 30 cibles) avant `nearest_target`, puis `steer_towards` faisait **4 racines carrées**
   (`length`, `normalize_or_zero` ×3, chacun recalculant `length_squared` puis `length`).
2. **`FixedVec2::length_squared`/`length` : 22 % des échantillons de A en propre** (`fixed::sqrt::u64` 83, `length_squared` 68 + `to_float_kind` 53) : chaque composante
   passait par un **aller-retour `f32`** (`Fixed::to_num::<f32>()` puis `FixedWide::from_num`), la racine par un autre, et la racine `fixed` est une boucle bit à bit.
3. **Collision : 16 % (148 échantillons, dont 74 en propre dans `query_aabb`).** `CollisionGrids::characters` contient *tout* collider rollback, **les 432 balles comprises** (voulu
   pour `move_characters`, qui compte les contacts d'ennemis) ; chaque balle le balayait en entier (balles × 506 AABB), collectait un `Vec`, le triait, puis
   la reconstruction (deux fois par frame) clonait un `GgrsNetId` (une `String`) par entrée.
4. **Expiration :** `projectile_expire_system` calculait `nearest_target` (tri de tous les personnages) et `normalize_or_zero` pour chaque projectile
   terminé, alors que seul un `on_expire: Spawn` s'en sert.
5. `Projectile::composed` (§66) ne tourne qu'**au tir** (≈ 80 tirs par seconde pour quatre joueurs à 20 coups/s) : aucun échantillon, hors du problème.
   Pas d'allocation par balle et par frame dans `Projectiles` hors `spawn_*` et `despawn`.

### Où passent `save` et `load` (snapshot de 217 KiB)

Sonde jetable (test ignoré retiré) à la population maximale (404 balles et projectiles, frame 119), µs par frame de sauvegarde, 200 répétitions :

| coût | µs |
|---|---|
| `clone` de tous les `Projectile` | 71 |
| `clone` de toutes les `Bullet` | 34 (dont `Tags` 25, `source` 9) |
| `Hash` de tous les `Projectile` | 121 (dont `on_hit` 28, `hits` 18, `id` 8, `table` 6) |
| `Hash` de toutes les `Bullet` | 65 (dont `Tags` 17, `source` 9) |
| **total balles** | **291** sur `save` ≈ 800 µs (36 %), et ≈ 105 µs de `clone` sur `load` ≈ 700 µs (15 %) |

Le reste est dans `bevy_ggrs` (un système par type rollback, ~100 types : chaque `load` parcourt tous les 506 porteurs de `Rollback`, chaque `save`
reconstruit une `HashMap<RollbackId, _>` et hache) : hors de notre code. **`save`/`load` n'entrent pas dans `rollback_ms_worst`** (= `load` + 12 × `advance`),
dont la simulation fait 90 %.

## 3. Optimisations (aucune ne change un résultat)

| fichier | changement |
|---|---|
| `crates/combat/src/collision_grid.rs` | `CollisionGrids::targets` : `characters` **sans les balles** (même ordre, un seul tri, `Has<Bullet>`) ; `projectile_collision_system` et `bullet_rollback_collision_system` le balayent (leur `Query` exige déjà `Without<Bullet>`, `Team`, `Health` : même ensemble de cibles). Entrées = `net_id: usize` seul (plus de `String` clonée), toujours triées (insertion à sa place) : `iter_overlapping` ne retrie ni n'alloue ; `query_aabb` (autres consommateurs) en est le `collect`. |
| `crates/combat/src/projectile.rs` | guidage : une liste de cibles par frame, **sans tri** (`nearest_target` est un minimum (distance, `GgrsNetId`), indépendant de l'ordre ; chaque balle n'écrit que sa vitesse) ; `projectile_expire_system` : visée et direction **à la demande** (premier `Spawn`) ; `steer_towards` : une racine pour `length` et `normalize_or_zero` de la vitesse (`FixedVec2::length_and_direction`). |
| `crates/bevy_fixed/src/fixed_math.rs` | `length_squared*`, `length`, `normalize`, `normalize_or_zero` (2D et 3D) **en entiers, au bit près** de l'ancien aller-retour `f32` (arrondi à 24 bits significatifs, égalité vers le pair), racine entière exacte (graine `f64` corrigée en entiers), `normalize_or_zero` ne calcule qu'un carré et qu'une racine. Oracle : l'ancien code est gardé **dans les tests** (4 tests). |

Preuves d'égalité de `bevy_fixed` (`cargo test -p bevy_fixed`) : carrés 2D (et `_as_wide`) et 3D sur 600 000 vecteurs de magnitudes de 1 à 32 bits,
cas limites compris (`i32::MIN`, 2²⁴ ± k) ; `length` d'un carré : **tous** les carrés `< 2²³` (toutes les égalités d'arrondi des petites longueurs)
et 2 000 000 de carrés tirés jusqu'à la limite de `Fixed` ; `normalize`, `normalize_or_zero`, `length_and_direction` sur 400 000 vecteurs ; racine entière
sur 1 000 000 de valeurs de 96 bits, carrés parfaits et voisins. Au-delà de `Fixed::MAX` la conversion d'origine est reprise telle quelle (même panique en debug).

Non fait (justifié) :

- **`Split` non borné** : décision de gameplay de William, pas touchée. L'optimisation suffit (§1) : la population reste de 432 balles, le chiffrage d'un plafond n'est pas
  nécessaire. Pour mémoire (M3-T15, VPS) : sans `division` 39 balles et 18,9 ms.
- **Hash et clone des balles** (§2, 291 µs de `save`) : un `Hash` par lots d'octets (le flux `SeaHasher` est équivalent par concaténation) gagnerait ~100 µs de `save`, un
  `clone_from` manuel ~50 µs de `load` ; ni l'un ni l'autre n'entre dans `rollback_ms_worst` ; non fait, chiffré pour la suite.
- Un `Hash`/`GgrsNetId` à nom partagé (`Arc<str>`) toucherait tout le moteur : hors tâche.

## 4. Piège trouvé (dette D111, non corrigé)

`FixedVec2::length_squared` (et `length`) **arrondissait chaque composante à 24 bits significatifs** (`f32`) : au-delà de 256 px d'écart sur un axe, les bits bas du carré se perdent. La conversion IEEE
est déterministe (aucun risque de désynchronisation), mais la règle « jamais de `f32` dans la simulation » n'était pas tenue, et corriger changerait des traces (`nearest_target`, `Homing`, distances
d'IA). L'implémentation en entiers **reproduit** ce comportement au bit près (c'est la condition de « aucune trace ne bouge ») et le dit en commentaire.

## 5. Preuve d'identité (résultats réels)

| commande | résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans `BLESS`) | 3 tranches `ok`, **654 scénarios**, 335 s ; `git status` propre ensuite : **aucune trace écrite ni différente** (dont `bench_tears.trace`, comparée aussi par `make test_scenarios SCENARIO=bench_tears` à chaque étape) |
| `ALACOD_FUZZ=0:8 cargo test -p scenario --profile headless --test fuzz_inputs -- --ignored` | 8 graines × 3 600 frames, `ok`, aucune divergence de synctest |
| idem avec `ALACOD_FUZZ_SCENARIO=bench_tears ALACOD_FUZZ=0:4 ALACOD_FUZZ_FRAMES=600` | 4 graines, quatre joueurs à inputs aléatoires, `ok` |
| `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (`MATCHBOX_PORT=3570`, `COMPOSE_PROJECT_NAME=b0`) | **2 139 frames resimulées** (client 0, 15 rollbacks de profondeur 11), 0 `DesyncDetected`, codes de sortie 0 0, 2 999 lignes identiques |
| `make test_crates` | **1 252 tests, 0 échec** (1 247 avant + 5 neufs : 4 de `bevy_fixed`, 1 de `collision_grid`) |
| `make lint` (zombies, testbed, throne, gungeon, isaac) | code 0, aucun avertissement neuf |
| `cargo fmt --all -- --check`, `make check_sim_boundary` (13 crates, 2 en dette D69), `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` | code 0 |

## 6. Non fait / incertain

- **Pas de mesure sur le VPS** : le poste de référence n'est pas accessible d'ici ; la projection (≈ 26 ms) est une extrapolation par rapport (§1). Le Mac est partagé : charge 8 à 32 pendant les relevés, relatifs seulement.
- `save`/`load` inchangés (§2, §3) : `save` 1,1 ms par frame pèse autant que la simulation (0,84 ms) ; D97 garde ce reste.
- Le p2p joué est celui de `isaac` (`p2p_smoke`), pas un p2p chargé d'objets de larmes (le script rejoue des inputs, pas des objets) ; les objets de larmes sont couverts par les 654 scénarios, par `bench_tears` en synctest et par le fuzz `bench_tears`.

## 7. Dettes

**D97 réduite** : l'objectif (pire rollback sous 3 ticks sans changer une trace) est atteint sur le Mac (−54 % ; 11,2 ms contre 24,3) et projeté sous 50 ms sur le VPS ; **à fermer par b6** après confirmation sur le VPS (D84).
Reste : `save` des balles (291 µs, hash et clone), population de `Split` (décision de William, plus nécessaire pour le seuil). **D111** (neuve) : `length_squared` par `f32`.
