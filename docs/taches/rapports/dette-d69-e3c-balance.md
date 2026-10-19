# dette-d69-e3c-balance : économie, power-ups, vagues et équilibrage en données de simulation (D69 étape 2, E3c)

Branche `dette-d69-e3c-balance`, base `3f3285cb` (`origin/main` après E3b). Fiche `docs/taches/dette-d69-etape2.md` § E3c. Refactoring pur : aucune trace ne bouge, aucun `BLESS`. Casse R15 et R16
(`Assets<WaveConfig|EconomyConfig|PerksConfig|PowerUpsConfig>` et `OnlineState`/`GggrsSessionConfiguration` dans la simulation).

## 1. Ce qui change

### Types de donnée de simulation (plus d'`Asset`) et enveloppes (application)

| donnée (simulation) | enveloppe `Asset` (application, `#[serde(transparent)]`) | plugin de chargement |
|---|---|---|
| `EconomyConfig` (`economy.rs`) | `EconomyConfigAsset` (`economy_assets.rs`, nouveau) | `EconomyAssetsPlugin` (`RonAssetPlugin` ×2), juste avant `EconomyPlugin` |
| `PerksConfig` (`economy.rs`) | `PerksConfigAsset` (idem) | idem |
| `PowerUpsConfig` (`powerups.rs`) | `PowerUpsConfigAsset` (`powerups_assets.rs`, nouveau) | `PowerUpsAssetsPlugin`, juste avant `PowerUpsPlugin` |
| `WaveConfig` (`waves/config.rs`) | `WaveConfigAsset` (`waves/assets.rs`, nouveau) | `WaveAssetsPlugin`, juste avant `WaveSystemPlugin` |

`GlobalAsset::{wave_config, economy_config, perks_config, powerups_config}` gardent leurs `Handle` (sources de la résolution, E3d) mais visent les enveloppes. Les `.ron` ne changent pas (`transparent`). Aucun type rollback créé,
retiré, renommé ni déplacé ; aucun système n'entre dans `GgrsSchedule` ; `EconomyPlugin` et `PowerUpsPlugin` gardent leur position dans `CoreSetupPlugin::build` (les `RonAssetPlugin` ne sont pas des types rollback).

### Ressources lues par la simulation

- **`ResolvedBalance`** (`balance.rs`) gagne `waves_enabled: bool` (remplace `global_assets.wave_config.is_none()` dans `wave_state_machine_system` et `wave_spawning_system`), `economy.shells_drop: Option<(u32, u32)>` (lu par
  `shop::shell_drop_on_death_system`) et `economy.shop_prices: BTreeMap<Rarity, u32>` (lu par `shop::shelf_price` et `map_ldtk::game::rooms::room_shop_system`). Les types résolus dérivent `PartialEq` (test de parité) ; `shelf_price` prend
  une `&ResolvedEconomyConfig` (plus d'`Option` : l'économie par défaut a des prix vides, même repli).
- **`PowerUpsTable(Option<PowerUpsConfig>)`** (`powerups.rs`, `init_resource` dans `PowerUpsPlugin`) : lu par `powerup_pickup_detect_system`, `apply_powerup_actions_system`, `loot_drop_on_death_system`, le HUD (`ui/hud.rs`), les repères au sol
  (`powerup_visuals.rs`) et `apply_scenario_powerup_placements` du runner. Remplie par `powerups_assets::resolve_powerups_table_system`, `OnEnter(GameLoading)`, `.after(resolve_balance_system)`.
- **`SessionPlayers(u32)`** (`session.rs`) : le nombre de joueurs de la session (`Online` → `max_player` de GGRS, sinon `PlayersCount`), écrit par l'**application** (`balance_assets::publish_session_players`, `OnEnter(GameLoading)`, les deux
  résolutions `.after`). `clock::resolve_clocks_system` et `balance_assets::resolve_balance_system` le lisent ; ni `OnlineState` ni `GggrsSessionConfiguration` ne subsistent dans `clock.rs` ni `balance.rs`. Sans valeur par défaut : une résolution lancée sans elle
  panique (Bevy, ressource absente) au lieu de résoudre pour zéro joueur.

### `balance.rs` / `balance_assets.rs`

Les types résolus et `resolve_waves` / `resolve_economy` / `resolve_perks` restent dans `balance.rs`. Leur assemblage devient une **fonction pure** `resolve_balance(players, waves, economy, perks, characters, character_ids) -> ResolvedBalance`
(configurations `Option<&…>` : absente = `Default`, comme avant ; `waves_enabled = waves.is_some()`). `resolve_balance_system` (même nom, même `OnEnter(GameLoading)`) passe dans `balance_assets.rs` (app) : il lit les enveloppes, les `CharacterConfigs` et
`SessionPlayers`, appelle `resolve_balance`. `core.rs`, `examples/character_tester.rs` (`.after(...)`) migrés vers `balance_assets::resolve_balance_system` ; pas de réexport sous l'ancien chemin.

### Autres lecteurs migrés

`global_asset.rs` (types de handles), `content_hot_reload.rs` (`AssetEvent<WaveConfigAsset>`), `scenario/src/runner.rs` (voir § 2), `docs/conventions.md` (§ équilibrage, § 18), `CLAUDE.md` (« Contenu lu par la simulation »).

Aucun `Assets<WaveConfig|EconomyConfig|PerksConfig|PowerUpsConfig>` (ni `Res<GlobalAsset>` pour eux) ne reste hors des fichiers d'enveloppe, de `balance_assets.rs`, de `global_asset.rs` et du runner de scénarios (qui visent l'enveloppe, § 2).

## 2. Décisions

- **Surcharges de scénario : l'enveloppe, avant la résolution** (fiche « documenter lequel »). `apply_wave_overrides` et `apply_powerup_drop_chance_override` restent des systèmes `Update` qui écrivent dans `Assets<WaveConfigAsset>` /
  `Assets<PowerUpsConfigAsset>` dès que l'asset est lisible, bien avant `OnEnter(GameLoading)` (chargement → lobby → carte → `GameLoading`), qui copie ensuite l'asset surchargé dans `ResolvedBalance` / `PowerUpsTable`. C'était déjà vrai pour les vagues
  (F5) ; pour les power-ups, la lecture était paresseuse (à chaque frame), elle ne l'est plus. Preuve : `bench_horde` (surcharge de vagues) et `powerup_drop_on_kill` (surcharge de `drop_chance` à 1) donnent les mêmes valeurs avant/après (§ 3.1) et gardent leur trace.
  `apply_scenario_powerup_placements` (`GgrsSchedule`, déjà là) lit la table.
- **`PowerUpsTable` figée pour la partie** : avant, les trois systèmes lisaient l'asset à chaque frame, donc un rechargement à chaud de `items/powerups.ron` en cours de partie changeait les power-ups. Désormais la table est recopiée à chaque entrée en
  `GameLoading` (donc aussi au `Restart` local) : même régime que `ResolvedBalance` (F5), documenté dans `docs/conventions.md` et sur `PowerUpsTable`. Changement de comportement hors partie scénarisée, volontaire (la fiche impose la table résolue à `OnEnter(GameLoading)`).
- **`waves_enabled` = « l'asset de vagues est lisible »** au lieu de « le handle existe ». Équivalent : `loading_asset_system` ne quitte `Loading` qu'une fois les quatre configurations chargées et `GlobalAsset` garde des handles forts (relecture indépendante).
- **`publish_session_players` dans `balance_assets.rs`** (pas dans `core.rs`) : c'est le pont app → simulation de l'équilibrage ; `core.rs` n'a que l'enregistrement et l'ordre.
- **Cliquet** : `clock.rs` et `waves/config.rs` entrent (22 → 24 fichiers). `balance.rs`, `economy.rs`, `powerups.rs`, `shop.rs`, `waves/mod.rs` sont libérés de toute fuite mais n'entrent pas : leur fermeture interne contient des fichiers à fuite ou hors liste
  (`character/config.rs` ← `ai/mod.rs`, `character/health/mod.rs` ← `effects_v3.rs`, `interaction.rs`, `items.rs`, `effects_runtime.rs`, `waves/mod.rs` ← `enemy/spawning.rs`…). `scripts/d69-sim-files.txt` en garde la trace.
- **`scripts/determinism-allowlist.json`** : l'exception de `waves/config.rs` suit son import (`use bevy::platform::collections::HashMap;`, `TypePath` retiré) ; aucune exception ajoutée.

## 3. Preuves

### 3.1 Parité de `ResolvedBalance` : avant/après sur la simulation elle-même

Test jetable (non commité) `crates/scenario/tests/zz_balance_dump.rs` : à la **première exécution de `GgrsSchedule`** de dix scénarios (`four_players_idle` 4 joueurs et `two_players_shooting` 2 joueurs de zombies, `aim_four_way` testbed,
`throne_duo_defaite`, `bench_floor` gungeon, `achievement_unlocks` isaac, `bench_horde` : surcharge de vagues, `powerup_drop_on_kill` : surcharge du drop, `shop_buy`, `coin_pickup`), il écrit vagues (tous les champs et paliers triés), économie, perks, santé de tous les
personnages, « le jeu a-t-il des vagues », `shells_drop`, `shop_prices` et la table de power-ups complète (`drop_chance`, id, nom, poids, portée, durée, actions). Joué sur `3f3285cb` (lecture des assets, avant) puis sur l'arbre E3c (lecture de `ResolvedBalance` /
`PowerUpsTable`, après) : **14 605 octets de part et d'autre, `diff` vide**. Les cas qui distinguent : jeux sans vagues (`wave_config_is_none: true`, testbed/throne/gungeon/isaac), gungeon (`shells_drop: Some((2, 5))`, `shop_prices: {Common: 15, Uncommon: 30, Rare: 50}`),
jeu sans `powerups.ron` (`powerups: None` : isaac, `achievement_unlocks`), santé en expression de `players` (`zombie_scaled` : 130 à 4 joueurs, 50 à 2), `bench_horde` (60 ennemis de base, 80 simultanés), `powerup_drop_on_kill` (`drop_chance` 1 au lieu de 0).

### 3.2 Parité en test unitaire (commité)

- `balance_assets::tests::resolved_balance_is_identical_through_the_envelope_and_the_table` : pour chacun des cinq jeux, de 1 à 4 joueurs, sur les vrais `wave_config.ron`, `economy.ron`, `perks.ron` et deux vrais personnages (`player`, `zombie_scaled`), le `ResolvedBalance`
  calculé à partir du **contenu de l'enveloppe** est égal à celui calculé à partir de la **configuration lue directement** (le chemin d'avant), **champ à champ** (déstructuration exhaustive de `ResolvedBalance`, `ResolvedWaveConfig`, `ResolvedEconomyConfig` : un champ de plus casse la
  compilation) ; les trois champs nouveaux sont comparés à ce que lisaient les systèmes avant (`shells_drop`, `shop_prices` de l'asset d'économie, `waves_enabled` = présence de la configuration de vagues) ; la santé est comparée à l'expression évaluée à la main
  (`10 + 40 × (players − 1)`). Le test exige de rencontrer un jeu avec vagues, un sans, un avec drop de `shells`, un avec prix.
- **Mutations jouées** (balance.rs restauré ensuite) : `shells_drop: None` → échec (`gungeon à 1 joueur(s) : left None, right Some((2, 5))`) ; `waves_enabled = waves.is_none()` → échec ; `shop_prices` non copié → échec (`left: {}, right: {Common: 15, Uncommon: 30, Rare: 50}`).
- `balance_assets::tests::economy_literals_do_not_depend_on_the_player_count` (`kill_points` en expression suit `players`, `shells_drop` et `shop_prices` non) et `online_uses_the_ggrs_player_count_otherwise_the_local_one` (`session_players`).
- `powerups_assets::tests::table_is_the_envelope_content_field_by_field` : la table publiée par `resolve_powerups_table_system` (via `Assets<…Asset>` et un `GlobalAsset`) égale la configuration lue directement, champ à champ (`PowerUpDef` déstructuré), pour tous les `items/powerups.ron` des jeux ;
  `game_without_powerups_clears_the_table` (jeu sans dossier `PowerUp` ou partie précédente : table vide).
- **Tests inline de `powerups.rs` réécrits sur la table** : `world_with_config` insère `PowerUpsTable(Some(…))` à la place d'`Assets<PowerUpsConfig>` + `GlobalAsset` ; les deux tests qui lisaient `weapon_assets::WeaponsConfig` (l'enveloppe, app) lisent la table d'armes de donnée (`WeaponAsset`). Les 9 tests passent, couverture inchangée.
- `shop::tests::price_falls_back_to_default` réécrit sur `ResolvedEconomyConfig` (via `resolve_balance`).
- **Test de scénario `crates/scenario/tests/balance_tables.rs`** (5 tests, dans le style de `character_configs.rs` / `weapon_registry.rs` : les replis des lecteurs masquent une table vide, donc on regarde les ressources elles-mêmes **au premier tick de `GgrsSchedule`**) :
  `SessionPlayers` vaut 4 puis 2 (`four_players_idle`, `two_players_shooting`) et pilote la résolution (`zombie_scaled` : 130 puis 50) ; `ResolvedBalance` est posée (`waves_enabled`, points, perks) et `PowerUpsTable` pleine pour zombies ; gungeon : pas de vagues, `shells_drop` `(2, 5)`,
  prix `[15, 30, 50]` ; isaac (sans `powerups.ron`) : table vide ; surcharges de scénario atteignent les tables (`powerup_drop_on_kill` : `drop_chance` 1 contre 0 dans le scénario voisin du testbed ; `bench_horde` : `base_enemies` 60 contre 3 dans `four_players_idle`). **Mutation jouée** : `resolve_powerups_table_system`
  retiré de `OnEnter(GameLoading)` → 3 tests sur 5 échouent (`PowerUpsTable remplie avant le premier tick`), `core.rs` restauré. Le test utilise `parking_lot::Mutex` (règle du dépôt pour un verrou immédiatement déverrouillé) : `crates/scenario/Cargo.toml` gagne
  `[dev-dependencies] parking_lot = "0.12"` (déjà dans le graphe de bevy, `Cargo.lock` : une ligne).

### 3.3 Ordre des systèmes et types rollback

`make schedule_order_diff BASE=origin/main` : **sortie 0**, 4 520 lignes de schedules et 645 types rollback identiques (cinq jeux), exécuté deux fois (avant et après les retouches de revue) ; le test de scénario `balance_tables.rs` (§ 3.2) et son `dev-dependency` ne changent pas l'ordre.

## 4. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`3f3285cb`) | après |
|---|---|---|
| fichiers du crate | 122 | 126 (+ `balance_assets.rs`, `economy_assets.rs`, `powerups_assets.rs`, `waves/assets.rs`) |
| fichiers à fuite | **49** | **47** |
| erreurs de résolution | **762** | **757** |
| dont `asset` | 144 | 139 |
| cliquet (`scripts/d69-sim-files.txt`) | 22 | **24** (+ `clock.rs`, `waves/config.rs`) |

Fichiers **libérés de toute fuite** : `balance.rs` (3), `economy.rs` (3), `powerups.rs` (5), `shop.rs` (1), `waves/config.rs` (1), `waves/mod.rs` (1) ; moins d'erreurs : `ui/hud.rs` 207 → 206, `powerup_visuals.rs` 10 → 9. Nouveaux fichiers à fuite, des enveloppes (app par construction) :
`balance_assets.rs` (3), `economy_assets.rs` (3), `powerups_assets.rs` (3), `waves/assets.rs` (2). `waves/systems.rs` n'a plus de `GlobalAsset`.

## 5. Vérifications (résultats réels)

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **sortie 0** (4 520 lignes, 645 types), deux fois |
| `make test_scenarios_par SHARDS=3` sans BLESS (scénarios power-ups, boutique, vagues, `wave_*`, `powerup_*`, `shop_*`, `bench_horde`, `four_players_*` inclus) | 3 tranches `ok`, **802 scénarios**, **aucun `.trace` modifié** (`git status` propre hors code et docs) ; **deux fois** : 361 s sur l'arbre avant les retouches de revue, 372 s sur l'arbre final (sans les tests ajoutés ensuite, qui ne touchent aucun code de simulation) |
| `make test_crates` | 83 résultats `ok` ; un seul échec : le test de temps D117 de `map_ldtk` sous charge (voir ci-dessous), qui passe seul. Un premier passage, avant l'ajout de `balance_tables.rs` (5 tests, passés à part : `cargo test -p scenario --test balance_tables`), était `exit 0` |
| `cargo fmt --all -- --check`, `make check_sim_boundary` (13 crates SIM, 2 en dette), `check-rollback-registration.sh` | OK |
| `make lint` ×5 | 0 erreur (zombies 5 personnages 4 armes 1 vagues, testbed 39/25, throne 14/13, gungeon 11/21, isaac 24/3) |
| `make gen GAME=…` ×5 | tous `ok ok`, aucune trace différente : zombies 16, testbed 104, throne 39, gungeon 59, isaac 213 scénarios générés |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` | OK (68,8 s) / OK (59,2 s) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e3c cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` (features par défaut et `--no-default-features`) | OK ×4 |
| **p2p-stress zombies** (`tests/scenarios/p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3572`, `COMPOSE_PROJECT_NAME=b3`) | **1 917 frames resimulées** (client 0 : 21 rollbacks, profondeur max 11), codes de sortie 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| p2p-stress isaac (idem) | 1 846 frames resimulées (21 rollbacks), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0, 24 fichiers au cliquet, § 4 |

**Rouges déjà présents sur `origin/main`, hors périmètre** : `scripts/check-forbidden.sh` et `make test_tools` (`test_current_repository_has_no_unreviewed_or_stale_exception`) échouent sur six `Instant::now()` non listés dans `scripts/determinism-allowlist.json`
(`jjrs/p2p.rs:470,497,531`, `jjrs/signal_watch.rs:159,180`, `ui/lobby.rs:220`), arrivés avec le commit `6d27cdf0` de `dettes-d105-d109` (b0) ; aucun fichier de `jjrs/` ni `ui/lobby.rs` n'est dans mon diff, et `check-forbidden.sh` ne signale rien d'autre
(l'exception de `waves/config.rs`, que mon import déplacé rendait périmée, est corrigée). **D117** : `make test_crates` (dernier passage, après tous les ajouts de tests) a **un** échec, `map_ldtk::generation::tests::isaac_grille_de_12_a_15_salles_sous_trois_fois_grand_de_gungeon` (borne de temps de génération, charge machine 17 à 29 pendant le run) ; le même test **passe seul** (0,41 s) ; dette D117 en cours chez b1, non comptée comme échec de cette livraison.

Disque : 14–16 Go libres pendant la livraison (sous le seuil de 20 Go) ; aucun exécutable `deps/<nom>-<hash>` périmé à supprimer (un seul par nom dans `headless/deps` et `debug/deps`).

## 6. Limites

- **Rechargement à chaud des power-ups pendant une partie** : plus effectif avant la prochaine entrée en `GameLoading` (§ 2) ; hors partie (lobby), `content_hot_reload` recharge toujours le registre de contenu. Aucun scénario ne recharge un asset.
- `ResolvedBalance` n'est comparée à l'ancien chemin que sur les cinq jeux livrés et dix scénarios (§ 3) ; une expression de contenu qui n'existe nulle part aujourd'hui n'est couverte que par `economy_literals_do_not_depend_on_the_player_count` et les tests existants de `balance.rs`.
- Non fait, hors périmètre : `GlobalAsset` réduit à la présentation (E3d), `weapons_config_update_system` (E3b), `scenario` « simulation seule » (D1).
- Le test jetable de parité (§ 3.1) n'est pas conservé : il compare deux arbres, ce qu'un test commité ne peut pas faire ; la parité commitée est § 3.2.

## 7. Relecture indépendante

Agent `reviewer` en lecture seule sur le diff non commité (sans cargo) : **aucun P0, P1 ni P2**. Trois P3, traités : (1) `powerups_assets::tests::table_is_the_envelope_content_field_by_field` comparait la table à l'enveloppe (clone de son original) → compare maintenant à la
configuration lue directement ; (2) `health_max_by_character` était vide des deux côtés dans le test de parité → deux vrais personnages (`player`, `zombie_scaled`) et une santé évaluée à la main ; la doc d'un test qui disait l'inverse de ce qu'il prouve est corrigée ; (3) `#[allow(clippy::too_many_arguments)]` inutile sur
`resolve_balance_system` (7 paramètres) retiré. Points confirmés par le relecteur : équivalence site par site (économie sans `economy.ron`, surcharge de drop, `waves_enabled`, power-ups, horloges), points de synchronisation automatiques entre les systèmes ordonnés de `OnEnter(GameLoading)`
(`Commands` visibles), `PowerUpsTable` toujours créée par `init_resource` là où l'asset l'était, redémarrage local (`GameLoading` re-résout depuis l'asset déjà surchargé), aucun lecteur oublié. Les chiffres du § 5 sont les miens, pas ceux du relecteur.
Ajoutés **après** la relecture, donc non relus : le test de scénario `crates/scenario/tests/balance_tables.rs`, le `dev-dependency` `parking_lot` et les corrections des trois P3 (tests et un attribut, aucun code de simulation).

## 8. Réserve de la revue E3b (rechargement à chaud d'une arme à distance)

Fermée en premier : section « Complément » (§ 7) de `docs/taches/rapports/dette-d69-e3b-weapon-registry.md`. Essai manuel sur `games/zombies` natif piloté par `scripts/alacod-remote` : un ennemi à distance monté temporairement (motif `volee`, table `projectiles` sur
le `pistol`, règle `Shoot` sur `zombie_full`), `weapons.ron` modifié en cours de partie (`arrow.damage` 5 → 20). Les zombies créés **avant** le rechargement tirent à `−5`, un zombie créé **après** (`zombie_full-85`) tire à **`−20`** (`<- 20 dmg from (zombie_full-85)`, santé 23 → 3) :
la copie du registre à la création de l'arme à distance porte bien la valeur rechargée. Montage annulé (`git checkout games/zombies/assets`).
