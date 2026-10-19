# dette-d69-e3d-global-asset : `GlobalAsset` réduit à une ressource d'application (D69 étape 2, E3d)

Branche `dette-d69-e3d-global-asset`, base `a6dd3496` (`origin/main` après E3c). Fiche `docs/taches/dette-d69-etape2.md` § E3d. Refactoring pur : aucune trace ne bouge, aucun `BLESS`. Premier point : le P3 de la revue E3c.

## 0. P3 de la revue E3c : `parking_lot`

`parking_lot` en dev-dependency de `crates/scenario` (une ligne de `Cargo.toml`, une de `Cargo.lock`) était superflu et contraire à la convention (E3a et E3b : `std::sync::Mutex`). Retiré avec la section `[dev-dependencies]` devenue vide ;
`crates/scenario/tests/balance_tables.rs` passe à `std::sync::{Arc, Mutex}` (`sink.lock().expect(…)`, `Arc::try_unwrap(…).into_inner().expect(…)`). Les cinq tests de `balance_tables.rs` passent (`make test_crates`).

## 1. Ce qui change

### `GlobalAsset` (`game::global_asset`), ressource d'application

La structure garde ses champs ; sa documentation les range en deux familles, toutes deux tenues pour que les assets restent chargés :

| famille | champs | lecteurs |
|---|---|---|
| présentation | `spritesheets`, `animations`, `slash_effect_spritesheet`, `slash_effect_animation` | `character/visuals.rs`, `ui/weapon_visuals.rs`, `world_sprites.rs` |
| sources de synchro (enveloppes) | `character_configs`, `weapons`, `melee_weapons`, `wave_config`, `economy_config`, `perks_config`, `powerups_config` | `character/config_asset.rs`, `weapon_assets.rs`, `balance_assets.rs`, `powerups_assets.rs` |

C'est la lecture de la fiche (« les `Handle<…Config>` des enveloppes restent pour la synchro ») : ces handles ne quittent pas la ressource, elle n'est lue par aucun fichier de simulation. `grep -rn GlobalAsset crates games examples templates` : `global_asset.rs`, `core.rs` (enregistrement), `lib.rs`,
les sept fichiers d'application du tableau, et les trois tests de scénario qui comparent registre et enveloppe (`character_configs.rs`, `weapon_registry.rs`, `balance_tables.rs`, qui atteignent l'enveloppe par son handle). Plus aucun commentaire de fichier de simulation ne le cite
(`balance.rs`, `run_state.rs`, `character/appearance.rs`, `character/config.rs`, `character/player/create.rs`, `character/mod.rs`, `content/registry.rs` reformulés).

**`ContentData` : aucune.** Après E3a-c aucun lecteur de simulation ne subsiste (grep ci-dessus) ; la fiche n'en demande une que dans ce cas.

### Lecteurs externes

| lecteur | avant | après |
|---|---|---|
| `bots/src/input.rs`, `examples/character_tester.rs`, `map_ldtk/game/rooms.rs` | déjà migrés par E3a-c (aucune occurrence) | inchangés |
| `map_ldtk/game/local.rs` | `LevelSpawnAssets.global_assets` + `contains_key` pour refuser un personnage inconnu | champ retiré ; `spawn_character` lit `CharacterConfigs::get` (même refus, même `warn!`, la configuration sert aussi à `team`) |
| `scenario/src/runner.rs` | `apply_wave_overrides` et `apply_powerup_drop_chance_override` : handle de `GlobalAsset` | l'enveloppe est désignée par `Assets::<WaveConfigAsset / PowerUpsConfigAsset>::ids().next()` ; sans enveloppe (jeu sans `Wave` / `PowerUp`) la surcharge ne trouve rien et reste en attente, comme avant |

Les surcharges visent toujours l'enveloppe, avant la résolution de `GameLoading` (décision E3c). Un jeu ne charge qu'une enveloppe de chaque type : `GlobalAsset::create` est le seul `load` de ces assets (confirmé par la relecture).

### Sortie de `Loading`

`loading_asset_system` n'annonce le lobby qu'une fois, en plus des assets chargés, **`CharacterConfigs` contenant chaque personnage déclaré et `WeaponRegistry` rempli** (`registries_published` / `table_published` : une table d'armes vide dans le fichier ne bloque pas, la synchro ne copie jamais une table vide). Ordre des schedules
(`PreUpdate` → `StateTransition` → `Update`) : la synchro tournait déjà avant la transition, le garde le rend explicite et le fait échouer bruyamment (jeu bloqué en `Loading`) au lieu d'un registre vide masqué par les replis des lecteurs. Deux tests inline : `a_table_is_published_once_readable_and_copied`,
`loading_waits_for_every_declared_character` (un autre personnage ne publie pas celui qui est déclaré). `resolve_balance_system` lit toujours `GlobalAsset::character_configs.keys()` pour les ids de `resolve_balance` : le `panic!` de `balance.rs` sur une santé non résolue garde son rôle de filet (un premier essai avec les clés de `CharacterConfigs`, qui le rendait inatteignable, a été
retiré sur remarque de la relecture).

Aucun type rollback, aucun système de `GgrsSchedule`, aucun plugin déplacé : `core.rs` n'est pas touché (b0 y fusionne E6).

## 2. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`a6dd3496`) | après |
|---|---|---|
| fichiers à fuite / fichiers du crate | 47 / 126 | 47 / 126 |
| erreurs de résolution | 757 | 759 |
| dont `asset` | 139 | 141 |
| cliquet (`scripts/d69-sim-files.txt`) | 24 | 24 (sortie 0) |

Seul `global_asset.rs` (APP) bouge : 17 → 19 erreurs (`Assets<WeaponsConfig>` et `Assets<MeleeWeaponsConfig>` du garde de sortie de `Loading`). Aucun fichier n'est libéré.

**Erreurs `asset` par classe du plan (`dette-d69-plan-etape2.md` § 2.1), avant puis après :**

| classe | fichiers | erreurs `asset` |
|---|---|---|
| **SIM** | **0 → 0** | **0 → 0** |
| MIXTE (`feedback.rs` 11, `character/visuals.rs` 7, `room_fog.rs` 3, `core.rs` 3, `frame.rs` 1) | 5 → 5 | 25 → 25 |
| APP | 12 → 12 | 85 → 87 |
| nouveaux fichiers d'application (enveloppes et synchro) | 6 → 6 | 29 → 29 |

Le critère « aucune erreur `asset` dans un fichier classé SIM » est tenu, mais **il l'était déjà à l'arrivée sur `a6dd3496`** : les 27 erreurs de 12 fichiers SIM citées par la fiche ont été fermées par E3a (personnages), E3b (armes) et E3c (économie, power-ups, vagues). E3d ne libère donc aucun fichier et n'ajoute rien au cliquet (24 fichiers). Les 25 erreurs MIXTE se
coupent en E5b/E4/E6 (`feedback.rs`, `core.rs`, `room_fog.rs`, `frame.rs`) ou sont la partie application de `character/visuals.rs`.

Pour mémoire, `curses.rs` et `performance.rs` (SIM au plan) sont sans fuite et ne dépendent que de fichiers du cliquet : ils pourraient y entrer ; ce n'est pas un effet d'E3d, non ajoutés.

## 3. Vérifications (résultats réels)

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **sortie 0**, 4 520 lignes de schedules et 645 types rollback identiques (cinq jeux), exécuté deux fois (arbre avant et arbre final) |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, **802 scénarios**, aucun `.trace` modifié, deux fois (353 s et 434 s) |
| `make test_crates` | **exit 0**, 84 résultats, **1 299 passés, 0 échec** (1 297 avant, +2 tests de `global_asset.rs`). Un premier passage a échoué au link (`ld: write() failed, errno=28`, disque plein) : rejoué après nettoyage des exécutables périmés |
| `cargo fmt --all -- --check`, `make check_sim_boundary`, `check-rollback-registration.sh` | OK |
| `make lint` ×5 | 0 erreur (zombies 5 personnages 4 armes 1 vagues, testbed 39/25, throne 14/13, gungeon 11/21, isaac 24/3) |
| `make gen GAME=…` ×5 | tous `ok`, aucune trace générée différente (`git status` propre) |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` | OK (53,4 s) / OK (45,9 s) |
| `cargo check --workspace --all-targets` (défaut et `--no-default-features`), `cargo check -p game --features lighting,ui_hui,debug_ui`, build web `APP_VERSION=e3d cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` | OK ×4 |
| **p2p-stress zombies** (`tests/scenarios/p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3572`, `COMPOSE_PROJECT_NAME=b3`) | **1 782 frames resimulées** (client 0 : 21 rollbacks, profondeur max 11), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| p2p-stress isaac (idem) | 1 698 frames resimulées (16 rollbacks), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0, 24 fichiers au cliquet, § 2 |

**Rouges déjà présents sur `origin/main`, hors périmètre** : `scripts/check-forbidden.sh` et `make test_tools` (`test_current_repository_has_no_unreviewed_or_stale_exception`) échouent sur les six `Instant::now()` non listés (`jjrs/p2p.rs:470,497,531`, `jjrs/signal_watch.rs:159,180`, `ui/lobby.rs:220`), comme dans le rapport E3c ; `check-forbidden.sh` ne signale rien d'autre (aucun
de ces fichiers n'est dans mon diff, l'exception de `global_asset.rs` reste valide).

Disque : le disque du Mac est tombé à 12–17 Go pendant la livraison (autres voies) ; 11 Go d'exécutables `headless/deps/<nom>-<hash>` périmés supprimés dans mon seul target (le plus récent par nom conservé, hors de tout build).

## 4. Limites

- Les surcharges du runner reposent sur « un jeu ne charge qu'une enveloppe de vagues et de power-ups » (vrai, un seul `load` dans `GlobalAsset::create`) ; deux enveloppes du même type feraient choisir la première de `Assets`, sans ordre garanti. Hors simulation (runner de tests).
- Le garde de `Loading` est redondant avec l'ordre actuel des schedules (la synchro précède déjà `StateTransition`) : il ne protège que d'un changement d'ordonnancement futur. Son seul effet observable est un blocage franc en `Loading` plutôt qu'un registre vide ; aucun scénario n'a de registre jamais publié à tester sans
  casser la synchro (les tests de premier tick `character_configs.rs`, `weapon_registry.rs`, `balance_tables.rs` prouvent la publication).
- `GlobalAsset` reste dans `game` (E9 le déplace avec les fichiers d'application) ; les trois tests de comparaison registre/enveloppe le lisent.

## 5. Relecture indépendante

Agent `reviewer`, lecture seule, diff non commité (sans cargo) : **aucun P0 ni P1**. Deux P2, traités : (1) `resolve_balance_system` passait les clés du registre à `resolve_balance`, ce qui rendait inatteignable le `panic!` de santé non résolue et sa doc tautologique → retour aux ids de `GlobalAsset::character_configs`, doc de `balance.rs` reformulée ; (2) la moitié « personnages » du garde de
sortie de `Loading` n'était pas testée → test `loading_waits_for_every_declared_character`. Points confirmés : équivalence des deux surcharges du runner (un seul `load` de chaque enveloppe, rechargement à chaud sans nouvel asset, jeux sans vagues/power-ups), `spawn_character`, absence de blocage permanent du garde (tables vides, clés identiques), aucun lecteur oublié, `check-forbidden` inchangé, ordre
`PreUpdate` → `StateTransition` → `Update` (`bevy_app` 0.19.1). P3 traités : commentaires obsolètes (`balance.rs`, `player/create.rs`, `character/mod.rs`, `content/registry.rs`), ligne E3d de `scripts/d69-sim-files.txt`, ce rapport ; P3 non traité : un `debug_assert` sur l'unicité de l'enveloppe dans le runner (§ 4).
Les chiffres du § 3 sont les miens, pas ceux du relecteur. Ajoutés **après** la relecture, donc non relus : le test `loading_waits_for_every_declared_character`, la remise des ids de `GlobalAsset`, les commentaires.
