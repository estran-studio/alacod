# dette-d95-d96-tirage-butin — D95 (graines de butin et de curse décorrélées) et D96 (déblocage et tirage stable) (b1)

Branche `dette-d95-d96-tirage-butin`, depuis `origin/main` f1d5a093 (qui contient D88). Conventions : `docs/conventions.md` §48 (graine du butin), §64 (curses, pools à épuisement, **contrat du tirage**), §73 (pools par salle), §75 (accomplissements).

## 1. D96 — le test `achievements` rouge sur main

### 1.1 Cause (sonde)

`crates/scenario/tests/achievements.rs::les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante` est rouge sur `main` depuis le merge de T10 (3f37ba43) : « emplacement 1 : reservoir_vital n'est pas un objet débloqué ». Reproduit sur f1d5a093 (4 min de build, puis 7 s de test).

Sonde (le test lui-même, instrumenté puis rétabli : pools réellement tirés et nombre d'emplacements qui bougent entre profil vide et profil débloqué, 400 emplacements par pool) :

- les entrées du pool `boss` sont **les mêmes** dans les deux cas (cinq objets de poids 2 plus `medaille`, poids 2) ; ce qui change, c'est `hidden` : `medaille` est cachée tant que `medaille_libre` manque. Sans elle, `draw_exhausting` a un poids total de **10** ; avec elle, **12** ;
- `draw_exhausting` choisissait `pick % total` (puis `pick_candidate % candidats`). Changer `total` redistribue **tous** les emplacements : pool `boss`, **66** emplacements inchangés, 83 vers `medaille`, **251 vers un autre objet** ; pool `tresor`, 14 inchangés, 7 vers `couronne`, **379 vers un autre objet**.

Cause confirmée : le modulo. Ce n'est pas un défaut de T10 ni de T11 : la propriété que le test exige n'était garantie par rien, elle tenait pour un contenu de pool donné.

### 1.2 Décision : la propriété est un contrat du moteur

§75 ne la promettait pas en toutes lettres (« à graine égale, les objets entrent dans les pools »), mais le test, écrit par T10, la lisait ainsi, et la valeur est réelle :

- une **graine partagée** donne le même butin à deux profils, aux objets débloqués près (graine du jour, run à défi, comparaison de parties) ;
- un **contenu qui s'enrichit** (T11b, T11c, T11d… ajoutent des objets aux pools) ne fait plus bouger que les emplacements qui prennent le nouvel objet : sans cela, chaque ajout d'un objet rebénit toutes les traces à butin du jeu (c'est ce qui s'est passé : 44 traces isaac rebénies par T11b) ;
- le coût est localisé : un tirage, une fois.

J'ai donc garanti la propriété (et gardé le test tel quel : il vérifie bien ce qui est garanti) ; contrat écrit au §64 (« Le choix est une course pondérée, stable à l'ajout ») et au §75 (« Contrat du tirage »), avec ses limites.

### 1.3 Schéma de tirage

Les deux tirages du flux (`pick`, `pick_candidate`, toujours consommés : la façon dont le flux avance ne change pas) forment une clé de base de 64 bits. **Course à deux niveaux** :

1. **L'entrée** : chaque entrée joue **autant de billets que son poids** ; billet *j* de clé `fmix64(fmix64(base ^ identité de l'entrée) ^ j × φ)` ; la plus petite clé gagne. L'entrée gagne avec la probabilité **exacte** `poids / somme` (minimum de `w` clés uniformes), en entiers `u64`, sans flottant ni `Fixed`.
2. **Le candidat** : parmi les candidats de l'entrée gagnante (plusieurs pour `Rarity`), la plus petite clé `fmix64(fmix64((base ^ séparateur) ^ identité) ^ hash(id) × φ)` gagne : uniforme.

- **Pourquoi des billets et pas « clé mise à l'échelle par le poids »** (`clé / poids`, le schéma suggéré) : le minimum de `u₁/w₁` et `u₂/w₂` ne donne pas `w/Σw` ; pour des poids 1 et 2, P(le poids 2 gagne) = 3/4 au lieu de 2/3. Les billets sont exacts. Coût : un `fmix64` par unité de poids du pool (poids du contenu : 1 à 6, une centaine d'entrées au plus) ; `items::MAX_LOOT_WEIGHT` = 1 000, refusé au lint (`OutOfRange`, pools et `UnlockGrant::Loot`) et plafonné au tirage (test `an_absurd_weight_is_capped` : `i32::MAX` ne boucle pas).
- **Identité d'une entrée = le contenu de son `pick`**, jamais son rang : les entrées d'un déblocage s'ajoutent dans l'ordre des identifiants de déblocage (`UnlockTable` est un `BTreeMap`), un rang changerait quand un autre déblocage s'ajoute. Deux entrées de même `pick` valent une entrée de poids somme ; l'ordre des entrées n'a aucun effet (testé).
- Inchangés : l'ordre canonique de génération, `PoolState::assign`, `exhaust_seed` (index), les passes d'assouplissement (rareté, objets tenus, `seen`), le flux de deux tirages.

### 1.4 Proportions (100 000 tirages par pool ; `the_race_keeps_the_weighted_proportions`)

| pool | objet | mesuré | attendu |
|---|---|---|---|
| `a` poids 1, `key` poids 2, `Rarity(Rare)` = {b, c} poids 3 | a | 0,1665 | 0,1667 |
| | key | 0,3325 | 0,3333 |
| | b | 0,2489 | 0,2500 |
| | c | 0,2521 | 0,2500 |
| quatre objets, poids 1, 2, 3, 4 | x0 | 0,1017 | 0,1000 |
| | x1 | 0,2011 | 0,2000 |
| | x2 | 0,2978 | 0,3000 |
| | x3 | 0,3993 | 0,4000 |

Borne du test : ± 0,008 (un écart-type vaut au plus 0,0016).

### 1.5 La propriété garantie, mesurée (`adding_to_a_pool_only_moves_slots_to_the_added_item`, 5 000 emplacements)

| ajout | emplacements qui bougent | vers |
|---|---|---|
| une entrée ajoutée à la fin (et la même au début : tirages identiques, le rang n'y change rien) | 820 | tous vers `x4` |
| un objet caché qui se débloque | 858 | tous vers `x5` (caché : tirages identiques à la base) |
| un candidat de plus dans l'entrée `Rarity(Rare)` | 224 | tous vers `r3` |
| un poids augmenté (1 → 4) | 1 019 | tous vers `x1` |

Aussi : l'ordre des entrées inversé, ou une entrée scindée en deux de même `pick`, donne exactement les mêmes tirages. Le test `achievements` (400 emplacements × deux pools) passe, sans modification.

**Limites** (dites au §64) : (1) le long d'une suite d'emplacements **à épuisement**, l'objet qu'un déblocage déplace revient dans le pool et peut ressortir à un emplacement suivant (`seen` n'est plus le même) : la garantie est par tirage, à `seen` égal ; (2) si l'ajout rend non vide une passe plus stricte (rareté d'un coffre), c'est cette passe qui tire ; (3) pour une entrée `Rarity` alourdie ou ajoutée, « l'objet ajouté » est un candidat de cette entrée (`unlock_enriches_pool` : le coffre riche ajoute une entrée Rare).

## 2. D95 — `loot_seed` et `curse_seed` décorrélés

### 2.1 Correctif

`bevy_fixed::rng` : `fmix32`, `fmix64`, `seed_from_hash(hash)` = repli 64 → 32 bits puis `fmix32` (l'idiome de `variant_seed`, qui l'utilise maintenant ; graines inchangées). `items::loot_seed`, `items::exhaust_seed` (index > 0), `run::curse_seed` : même formule, **sans allocation** (FNV incrémental `fnv1a_extend` / `fnv1a_extend_decimal` de D88, mêmes octets que l'ancien `format!`). Nouveaux : `items::loot_seed_numbered(graine, étage, "shells:", n)` et `items::loot_seed_parts(graine, étage, &["shop:", salle])`, utilisés par les sites qui construisaient une chaîne : `game::shop` (drop de douilles, à chaque mort d'ennemi ; boutique), `game::champions` (butin de champion ; la ligne de trace garde la clé `champion:<id>` par argument de format, sans chaîne). Les clés stockées ou rejouées (`room:<salle>`, `rock:<salle>:<x>:<y>`, `reward:`, `boss:`, `shelf:`) restent des `String` : elles servent aussi de clé d'assignation, une fois par emplacement.

### 2.2 Mesure refaite en Rust (pas une réplique Python)

Tests `items::loot::tests::loot_seeds_of_neighbouring_keys_are_independent` et `run::curse::tests::curse_seeds_of_neighbouring_floors_are_independent`, sur le modèle de `variant::tests` : 16 clés voisines, indicateurs de p = 0,1 (première valeur du flux, `next_fixed < 0,1` : la chance d'une curse ou d'un champion ; puis le **tirage de butin entier**, dix armes de poids 1), corrélation de Pearson entre clés voisines, **fréquence de chaque clé, la dernière comprise** (P3 de la revue de D88), part des graines sans occurrence (attendu 0,9¹⁶ = 18,5 %). Chaque test confronte l'ancienne formule (copiée dans le test) : il doit la **refuser** (|r| max > 0,03), sinon le détecteur est aveugle.

Corrélation de Pearson (|r| max entre clés voisines ; première valeur du flux, **100 000 graines**, bruit d'échantillon ≈ 0,003) :

| famille de clés | avant | après |
|---|---|---|
| `champion:100+i` | 0,0466 | 0,0063 |
| `champion:1000+i` | 0,0489 | 0,0069 |
| `shells:100+i` | 0,0485 | 0,0061 |
| `shells:37+i` | 0,0451 | 0,0065 |
| `rock:3:10:5+i` | 0,0477 | 0,0057 |
| `rock:3:2+i:5` | 0,0068 | 0,0057 |
| `room:salle_i` | 0,0515 | 0,0073 |
| `curse:<graine>:<étage>`, chance 0,1 (20 000 graines, 16 étages) | 0,0593 | 0,0121 |
| `curse:<graine>:<étage>`, **choix** a/b (poids 1 et 3, p = 0,25) | **0,3366** | 0,0153 |

- Fréquence de chaque clé après : [0,0988 ; 0,1017] au pire (100 000 graines) pour les sept familles de butin, [0,0972 ; 0,1047] pour la curse ; part des graines sans occurrence 18,4 à 18,6 % (attendu 18,5 %).
- Le tirage de butin entier (la course, 20 000 graines, bruit ≈ 0,007) donne |r| ≤ 0,017 avant **et** après : la course mélange déjà les clés avec `fmix64` ; D95 corrige la graine, la course la protège.
- **Trouvé en route, plus grave que le seuil de la dette** : le **choix** de la curse (`next_u32() % Σ poids`, le deuxième tirage du LCG) était corrélé à **0,34** entre étages voisins (le LCG `x ← 1664525·x + c` ne mélange pas les bits bas : les deux derniers bits du deuxième tirage sont fonction des deux derniers de la graine). La chance (premier tirage, bits hauts), seule mesurée par la dette, ne montrait que 0,06. Corrigé par la même graine ; `draw_curse` garde son tirage.
- Un premier essai à 20 000 graines donnait 0,0279 pour `shells:37+i` après correctif (seuil 0,03) : un écart de bruit (4 σ) ; à 100 000 graines, quatre fenêtres disjointes donnent |r| ≤ 0,007. D'où les 100 000 graines du test (le premier indicateur est bon marché).

## 3. Traces

Résultat des **rejeux sans BLESS** : seuls les scénarios qui tirent un butin, une curse ou un drop de douilles bougent ; zombies et throne **aucun**.

### 3.1 69 traces rebénies (54 écrites à la main, 15 générées)

- **isaac (34)** : `achievement_not_met`, `achievement_unlocks`, `curse_bigger_floor`, `curse_blind`, `curse_hides_health`, `curse_hides_minimap`, `devil_deal_isaac`, `devil_room_absent`, `devil_room_chance`, `grid_floor_seed_1`, `grid_floor_seed_123`, `grid_floor_seed_23`, `grid_post_boss`, `grid_secret_bomb`, `grid_secret_no_bomb`, `grid_secret_room`, `grid_treasure_floor0`, `grid_treasure_key`, `hit_iframes_isaac`, `isaac_banner_blind`, `isaac_boots`, `isaac_floor_seed_1`, `isaac_hud_items`, `isaac_reliquaire`, `isaac_synergy_garde`, et générés `boss_gardien`, `champion_mouche_essaim`, `champion_suiveur_explosif`, `champion_tourelle_croix_salve`, `room_combat`, `tears_encre`, `tears_encre_sans_objet`, `tears_sang_chaud`, `tears_sang_chaud_sans_objet`.
- **gungeon (22)** : `bench_floor`, `bot_explorer_floor`, `bot_explorer_two`, `gungeon_boss`, `gungeon_boss_bot`, `gungeon_clear`, `gungeon_floor_seed_1`, `gungeon_floor_seed_123`, `gungeon_floor_seed_23`, `gungeon_floor_start`, `gungeon_full_run`, `gungeon_full_run_four`, `gungeon_two_floors`, `minimap_shop_visit`, `shop_seeded`, `unlock_base_pool`, `unlock_enriches_pool`, et générés `boss_gatling`, `room_arene`, `room_combat`, `room_escouade`, `room_secret`.
- **testbed (13)** : `champion_drops`, `chest_locked`, `curse_drawn_seeded`, `curse_drawn_seeded_other`, `curse_modifier`, `loot_floor_table`, `loot_seeded`, `loot_seeded_other`, `pool_exhaust_two_chests`, `pool_exhaust_two_chests_temoin`, `pool_exhaust_two_rooms`, `pool_unlock`, et généré `champion_elite_riche_riche` (son `.ron` aussi : la sonde du générateur mesure l'objet lâché, `boussole` → `bottes`).

Procédure : les sept scénarios dont la graine, un objet ou une chronologie change ont été recalibrés d'abord (§4) puis rebénis un par un ; les 27 traces restantes (seule la trace diffère, attentes vertes) l'ont été d'un bloc (`SCENARIO=<liste> BLESS=1`) après un rejeu sans BLESS qui montrait **uniquement** « trace différente » ; les 15 générées par `make gen GAME=… GEN_BLESS=1` (cinq jeux : seules ces 15 ont bougé).

### 3.2 Preuve §5 par jeu (dumps `ALACOD_DUMP_TRACE`, avant = binaire de `main` f1d5a093 compilé dans un worktree, après = la branche, `scripts/trace-diff.py`)

| jeu | scénario | première frame qui diffère | ce qui diffère à cette frame |
|---|---|---|---|
| testbed | `loot_seeded` | 35 (ouverture du coffre `k`) | `ChestOpened.item_id` (`boussole` → `manche`) et le `ItemPickup` posé (même position, même net id 29) : rien d'autre |
| testbed | `champion_drops` | 54 (mort du champion) | l'objet lâché (net id 17 : `bottes` → `manche`), position identique |
| testbed | `pool_exhaust_two_rooms` | 0 (génération de l'étage) | `PoolState` seul : `epuisable` `{bottes, lame}` → `{boussole, lame}`, assignations des deux salles |
| gungeon | `shop_seeded` | 0 (génération) | les trois `Shelf` (net ids 192-194) : `item_id` et `price` (`arbalete` 15 / `mitraillette` 30 / `revolver` 15 → `pistolet_precis` 15 / `pistolet_jumele` 15 / `rebondisseur` 30) ; clés de présentoir, positions identiques |
| gungeon | `generated/gungeon/room_combat` | 169 (mort d'un ennemi) | `ShellPickup.amount` (net id 56 : 3 → 5) : la graine `shells:<net id>` |
| gungeon | `generated/gungeon/boss_gatling` | 140 | `ShellPickup.amount` (net id 45 : 4 → 2) |
| isaac | `grid_treasure_floor0` | 0 | `PoolState` (`boss`, `diable`, `tresor`) et le `Shelf` du diable (`regard_maudit` → `cornes_noires`) : rien d'autre |
| isaac | `isaac_hud_items` | 0 | `PoolState` seul (`tresor` : `trefle_noir` → `vent_arriere`, `boss` identique) |
| isaac | `generated/isaac/room_combat` | 0 | `PoolState` seul (idem) |
| isaac | `generated/isaac/champion_mouche_essaim`, `…/tears_encre_sans_objet` | 70, 118 | `ShellPickup.amount` (4 → 2), position identique |
| isaac (curse) | `curse_hides_health` joué à la **graine d'avant** (7), copie temporaire du `.ron` d'origine | 0 | `RunCurses` (`{0: unknown}` → `{0: darkness}`) et `PoolState` (`boss`, `tresor`) : rien d'autre à la première frame |

Cause unique dans chaque cas : le tirage (butin à l'ouverture, assignation à la génération, drop de douilles, curse d'étage). La première frame qui diffère est celle où le tirage a lieu. Les conséquences (objet ramassé, curse qui masque la minicarte, inventaire) viennent après.

**Témoins sans butin tiré, dump complet, « identique »** : `idle` (zombies, 1 499 frames), `two_players_shooting` (zombies, 299), `throne_solo` (throne, 4 409), `testbed_dummy_shoot` (179), `gungeon_start` (119), `shells_drop` (le drop de douilles est à montant constant : 699), `generated/isaac/enemy_mouche_still`, `generated/gungeon/enemy_shotgun_kin_still`, `generated/testbed/enemy_grunt_still` (599 chacun : ennemis à variantes sans champion).

## 4. Scénarios recalibrés (chaque `.ron` modifié porte une ligne `D95/D96 (2026-10-17, …)` ; aucune attente supprimée, trois sens modifiés dits plus bas)

Les nombres de graines et d'objets sont des instantanés du tirage ; un tirage qui change (D95, D96) les déplace. Méthode : la graine **ou** la valeur attendue, selon ce que le scénario prouve.

**Graine changée (le scénario prouve une issue précise)** — graine de carte = graine du tirage ; la graine de carte détermine aussi le niveau, donc la position du joueur : la changer n'est possible que si la carte reste valable (c'est le cas des scénarios de curse, pas des coffres du testbed, dont les coordonnées sont celles du niveau de la graine 1) :
- `curse_blind` 5 → 3 (`blind`), `isaac_banner_blind` 5 → 3, `curse_bigger_floor` 2 → 6 (`labyrinth`, `RoomCount` 16..24), `curse_hides_health` 7 → 5 (`unknown`), `curse_hides_minimap` 1 → 2 (`darkness`), `curse_drawn_seeded` 3 → 2 (`darkness` puis rien), `curse_drawn_seeded_other` 2 → 22 (`labyrinth` puis `darkness`), `curse_modifier` 3 → 2 (`maze` puis rien) : plus petite graine qui convient, mesurée par une sonde Rust jetable ; aucune frame ni attente ne bouge.
- `pool_exhaust_two_chests` et son témoin : la **clé** `k5` devient `k14` (la graine 1 reste : une autre graine change de niveau et le joueur n'a plus de coffre sous les pieds) : avec le nouveau tirage `k1` et `k5` donnent deux objets différents au témoin sans épuisement ; `k1` et `k14` donnent bien deux `lame`, et le jumeau à épuisement `lame` puis `bottes` : **mêmes attentes qu'avant**.
- `crates/scenario/tests/curses.rs` : `curse_drawn_seeded_other` joue la graine 22 ; la graine « sans aucune curse » passe de 1 à 5 (mesurée par une sonde Rust jetable : la graine 1 tire désormais `maze` à l'étage 0) ; commentaire de `curse_modifier` (graine 2).

**Valeur attendue re-mesurée (même sens)** : `loot_seeded` (`boussole` → `manche`), `loot_seeded_other` (`manche` → `bottes`, toujours différent de `loot_seeded`), `loot_floor_table` (`bottes` → `manche`, pool `commun`), `chest_locked` (`boussole` → `lame`), `champion_drops` (`bottes` → `manche`), `shop_seeded` (trois armes ci-dessus, prix qui suivent la rareté), `unlock_base_pool` (amulette 1 → 0, bastion 1 → 0, lunettes 3 → 1) et `unlock_enriches_pool` (amulette 2 → 1, bastion 2, lunettes 2 → 0) : `roulade` (2 contre 3) ajoutée aux deux pour garder une différence non nulle entre le profil de base et le profil débloqué ; `devil_room_chance` (graine 8 gardée, la grille est son objet : `PoolSeen` diable `queue_fourchue` + `cornes_noires`, trésor `bottines`, boss `oeil_vif`).

**Chronologies re-mesurées** (le bot suit un autre chemin avec d'autres objets) : `grid_post_boss` (graine 8 : boss verrouillé f1033, mort f1400, butin `oeil_vif`, victoire f2117 ; `frames` 2150 → 2200), `grid_floor_seed_23` (butin `sang_chaud` au lieu de `reservoir_vital`, victoire f2335 ; `RoomState Locked` du boss lu à f1020, **borne** : la frame exacte du verrouillage n'est pas un moment clé), `bot_explorer_floor` (`FloorIndex(0)` à f650, le portail passe à f663), `gungeon_full_run` (graine 10 gardée : victoire f7316 contre f7730) et `gungeon_full_run_four` (victoire f6091 contre f6455).

**Écarts de sens dans les deux runs gungeon, à valider (détail dans chaque `.ron`)** :
1. La première salle donne une ration au lieu d'une clé (clé ramassée f2248 au lieu de f157) et le coffre à clé donne `double_canon` (f2821) au lieu d'`arc` ; la boutique du premier étage vend un blank et une arme, plus gilet ni roulade.
2. **`HasItem(gilet)` et `HasItem(roulade)` à f3090, qui prouvaient qu'un achat atteint l'inventaire, n'existent plus** : ces objets ne sont plus en boutique. Remplacés par `HasItem(rancune)` (joueur 0, f4800, coffre) et `roulade` (coffre, f4300), qui ne prouvent pas un achat ; **la preuve d'achat est restaurée par `Consumable(blank)` du joueur acheteur** : 0 à f2685 puis 1 à f2700 (`gungeon_full_run`, achat vu à f2687) ; 0 à f2585 puis 1 à f2600, joueur 3 (`gungeon_full_run_four`, achats f2587, f2611, f2648). Rien n'est retiré, mais le *même* objet n'est plus prouvé.
3. **Phase 2 du boss non observée** : `BossPhase` est lue en phase 1 au lieu de 2 et l'événement « phase 1 → 2 » devient « phase 0 → 1 », dans les deux runs. Mesuré sur un dump du premier boss de `gungeon_full_run` (jusqu'à f3398, pas du second boss) : **120 PV au plus, 60 de f3330 à f3353, 30 à f3354 (phase 1), mort à f3355** : deux coups de 30 PV à une frame d'écart. [INFÉRENCE] le boss franchit le seuil de la phase 2 et meurt avant qu'elle soit évaluée ; l'arme qui frappe n'est pas identifiée dans le dump, les seuils de phase n'ont pas été relus, et rien ne prouve que ce soit du seul fait du butin (aucune mesure avant D95/D96 de ces PV). Le second boss meurt 36 frames après sa phase 1 sans phase 2 (PV non mesurés). La phase 2 reste prouvée par `gungeon_boss`, `gungeon_boss_bot` et `boss_two_phases`. Une autre graine (essayée : 1 à 6) change toute la carte ; non retenue.

**Générés** : seul `champion_elite_riche_riche` change de `.ron` (`HasItem` `bottes`).

## 5. Résultats (réels)

Tout a tourné sur le code final (dernier commit avant `LIVRÉ`), machine chargée (charge 20 à 50) ; `origin/main` n'a pas bougé depuis f1d5a093 (T14 non mergé).

- `make test_scenarios_par SHARDS=3`, **sans BLESS** : **642 scénarios verts**, 3 tranches, 345 s. Première passe (avant recalibrage) : 68 en échec ; deuxième (après les recalibrages et le bless des quinze scénarios de curses/isaac et des treize de gungeon) : 27 « trace différente » et rien d'autre, blessés d'un bloc (§3.1).
- `make test_crates` : exit 0, **1 217 réussis, 0 échec**, 14 ignorés. La première passe avait un échec, `curses::same_seed_same_curse_other_seed_other_curse` (graines 2 → 22 et « aucune curse » 1 → 5, §4), corrigé. `achievements::les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante` (D96) passe sans modification. Les 51 tests de `items` (statistiques comprises) : 8,2 s.
- `make lint` ×5 : 0 erreur (avertissements : testbed 112, gungeon 1, isaac 24 ; zombies et throne aucun). `make gen GAME=…` ×5 : zombies 16, testbed 102, throne 39, gungeon 59, isaac 95 scénarios, tous « ok ok » (aucune trace différente, aucune erreur).
- `cargo fmt --all -- --check` propre ; `make check_sim_boundary` (11 crates respectent la frontière, 5 en dette D69, inchangé) ; `check-forbidden.sh` et `check-rollback-registration.sh` OK ; `make test_tools` 25 tests OK.
- Fuzz `ALACOD_FUZZ=0:8` (3 600 frames par graine, synctest) : aucune divergence ; `ALACOD_FUZZ=0:4 ALACOD_FUZZ_SCENARIO=isaac_floor_seed_1` (isaac, avec butin et champions) : aucune divergence.
- `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (`MATCHBOX_PORT=3571`, `COMPOSE_PROJECT_NAME=b1`) : **1 583 frames resimulées** (client 0 ; une première exécution avant la revue : 1 680), 0 desync, codes de sortie 0 0, 2 999 lignes identiques. L'assignation du butin à la génération de l'étage est dans la trace comparée.

## 6. Relecture indépendante (agent `reviewer`, lecture seule)

Verdict : **mergeable après correction des P2**, aucun P0 ni P1. Tout est traité :

- **P2, contrat plus fort que le code pour `Rarity`** (un poids augmenté ou fusionné sur une entrée `Rarity` déplace vers n'importe lequel de ses candidats, y compris un ancien) : doc de `draw_exhausting`, §64 et §75 précisés ; nouveau test `rarity_weapon_and_currency_additions_stay_inside_what_they_bring` (fusion et ajout d'une entrée `Rarity`, `Weapon`, `Currency`).
- **P2, lint sans fixture** : `loot_weight_over_cap` et `unlock_weight_over_cap` (`crates/content/tests/fixtures`, tests `loot_weight_over_cap_fixture_reports_out_of_range`, `unlock_weight_over_cap_fixture_reports_out_of_range`).
- **P2, docs périmées** (`conventions.md` : `champion_drops`, `pool_exhaust_two_chests`, `devil_room_chance`, `grid_post_boss`, tableau des curses du §74, victoires f7316 / f6091 de §57) : corrigées.
- **P2, « aucune attente retirée ni affaiblie » faux** : reformulé (§4) ; la preuve d'achat de `gungeon_full_run` / `_four` est **restaurée** par `Consumable(blank)` du joueur acheteur (0 → 1) ; la phase 2 du boss est dite perdue, avec la mesure de PV du premier boss et l'inférence marquée comme telle.
- **P2, rapport inachevé** : ce §5 et le journal de `docs/taches.md` §10.
- **P3** : somme de poids plafonnée aussi après fusion (`MAX_LOOT_WEIGHT`) ; annotation de schéma `x-lint` de `LootEntry.weight` : « > 0, ≤ 1000 » ; ordre des variantes de `Rarity` documenté comme donnée (le tirage hache `*rarity as u8`) ; durée des tests mesurée (8,2 s) ; les graines de curse « plus petite qui convient » sont des instantanés, dits au §74 (note D95).

## 7. Dettes et remarques

- **D95 et D96 fermées** (`docs/taches/dettes.md`).
- **À valider** : les deux runs gungeon ne montrent plus la phase 2 du premier boss (§4, point 3). Si l'équilibrage des armes ou des boss change, relire ces attentes.
- Le **choix** de la curse (`% Σ poids`, deuxième tirage du LCG) reste un modulo sur les bits bas : sans importance ici (la graine est maintenant bien mélangée, r 0,015), mais un LCG ne mélange pas ses bits bas ; une évolution de `RollbackRng` serait un changement de toutes les traces, hors périmètre.
- **Ne pas merger avant T14** (`m3-t14-bot-isaac`, b3, qui rebénit les `bot_isaac_*`) : si T14 est mergé avant, fusionner `origin/main` et rebénir à nouveau ([INFÉRENCE] : les `bot_isaac_*` parcourent des étages à butin, leurs traces bougeront ; non vérifié, T14 n'est pas dans cette branche).
- Outillage de preuve : un worktree `base-d95` (main f1d5a093) a servi aux dumps « avant » ; supprimé avant livraison.
