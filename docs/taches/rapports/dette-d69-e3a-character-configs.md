# dette-d69-e3a-character-configs : configuration des personnages en ressource de simulation (D69 étape 2, E3a)

Branche `dette-d69-e3a-character-configs`, base `5fa55fcb` (`origin/main` après E1 et E2). Fiche `docs/taches/dette-d69-etape2.md` § E3a.
Refactoring pur : aucune trace ne bouge, aucun `BLESS`. Casse R3 (`Assets<CharacterConfig>` dans la simulation).

## 1. Ce qui change

- `character/config.rs` : `CharacterConfig` perd `Asset, TypePath` (reste `Deserialize, Debug, Clone`). Nouveaux : `CharacterConfigKey(pub String)`
  (`Component`, la clé est l'`asset_name_ref`, remplace `CharacterConfigHandles`) et `CharacterConfigs(pub BTreeMap<String, CharacterConfig>)`
  (`Resource`, clés = celles de `GlobalAsset::character_configs`, méthode `get(&str)`). `CharacterConfigHandles` **n'était pas rollback**
  (aucun enregistrement dans `character/mod.rs`) : aucun `Hash`/`Debug` rollback ne change ; `CharacterConfigs` n'est ni rollback ni checksum
  (écrite hors `GgrsSchedule`, donnée statique d'une partie comme l'était `Assets<CharacterConfig>`).
- `character/config_asset.rs` (nouveau, app) : `CharacterConfigAsset(pub CharacterConfig)` (`Asset, TypePath, Deserialize`,
  `#[serde(transparent)]` : les `.ron` ne changent pas), `CharacterConfigAssetPlugin` (`RonAssetPlugin::<CharacterConfigAsset>`, déplacé de
  `character/mod.rs`, au même endroit de `BaseCharacterGamePlugin::build`, avec `init_resource::<CharacterConfigs>()`) et
  `sync_character_configs` (`PreUpdate.before(bevy_ggrs::RunGgrsSystems)`).
- `GlobalAsset::character_configs` : `HashMap<String, Handle<CharacterConfigAsset>>`, reste la source de la synchro (E3d). L'exception
  `determinism-allowlist.json` suit le texte des deux lignes modifiées (`global_asset.rs`, `config.rs`).
- `scenario/src/generate.rs` (parse `CharacterConfig` en RON directement) : inchangé.

### Sites migrés (`Res<Assets<CharacterConfig>>` + `CharacterConfigHandles` → `Res<CharacterConfigs>` + `CharacterConfigKey`)

| groupe | fichiers |
|---|---|
| les 4 systèmes de la fiche | `player/input.rs` (`apply_inputs`), `health/mod.rs` (`rollback_resolve_damage_events`, `rollback_apply_accumulated_damage`), `ai/pathing.rs` (`move_enemies`) : **plus aucun `Assets`/`Handle`** (grep 0) |
| autres lecteurs de simulation | `interaction.rs` (`handle_revive_interaction`, `handle_weapon_pickup_interaction`), `champions.rs` (`champion_death_system`, `DivisionAssets.characters_asset` retiré : il lisait deux fois la même chose), `familiars.rs`, `waves/systems.rs` (`SpawnAssets.character_configs`), `balance.rs` (`resolve_balance_system` : itère toujours les clés de `GlobalAsset::character_configs`, lit `CharacterConfigs`, panique comme avant si une entrée manque) |
| créateurs | `character/create.rs` (`create_character` perd son paramètre `GlobalAsset`, ne lisait que lui pour les configs ; panique inchangée si `config_name` ou `asset_name_ref` manque), `enemy/create.rs` (`spawn_enemy`), `enemy/spawning.rs`, `player/create.rs` (`create_player`), `familiars::spawn_familiar` (perd `GlobalAsset`) |
| présentation | `character/visuals.rs` (`attach_character_visuals`, `spawn_death_ghosts`), `feedback.rs` (`tint_champions`), `feedback_v3.rs` (`variant_tint(configs, nom, variante)` perd `GlobalAsset`), `ui/hud.rs` (`revive_frames`), `content_hot_reload.rs` (`AssetEvent<CharacterConfigAsset>`) |
| appelants externes | `bots/src/input.rs`, `map_ldtk/game/{local,rooms}.rs` (`LevelSpawnAssets.character_configs`), `scenario/src/runner.rs` (placements), `scenario/tests/{expectations,extra_health,scenarios}.rs` (les surcharges portent maintenant sur `CharacterConfigs` ; le diagnostic ignoré `sight_probe` affiche l'id du personnage au lieu du chemin de son asset), `examples/character_tester.rs` |

`grep -rn "Assets<CharacterConfig>\|CharacterConfigHandles" crates games examples templates` (`--include=*.rs`) ne renvoie plus que deux commentaires de doc (`config.rs`) ;
`Assets<CharacterConfigAsset>` (enveloppe, `config_asset.rs` et le test de rechargement) est le seul `Assets` de personnage restant, côté application.

## 2. Moment du remplissage (décision D2) : preuve

`sync_character_configs` tourne **à chaque frame**, en `PreUpdate`, avant `RunGgrsSystems`, donc avant tout tick de `GgrsSchedule` quel que soit
le chemin, car il est dans `BaseCharacterGamePlugin` (jeu natif avec ou sans rendu, headless, `run_with`, p2p) :

- **entrée absente** dont l'asset est lisible : copiée. Chemin du chargement initial ; il ne dépend d'aucun `AssetEvent` (les événements
  d'un asset sont écrits en `PostUpdate`, une frame après son insertion : s'y fier aurait créé une fenêtre d'une frame) ;
- **`AssetEvent::Modified`** : l'entrée de l'asset modifié est recopiée (rechargement à chaud, `file_watcher`). `Added` et
  `LoadedWithDependencies` ne font **rien** : une surcharge de scénario posée sur la ressource après le remplissage n'est jamais écrasée.

Argument par construction : `loading_asset_system` n'autorise `Loading → Lobby` qu'une fois tous les assets chargés ; le `StateTransition` suivant
a lieu après le `PreUpdate` qui remplit ; `OnEnter(GameLoading)` (`resolve_balance_system`) et tout créateur de personnage (`create_character`)
**paniquent** (`unwrap`/`panic!`, pas de repli) si l'entrée manque ; la ressource n'est jamais vidée (seulement insérée ou recopiée). Un
personnage qui porte `CharacterConfigKey` a donc forcément été créé avec son entrée présente, et les lecteurs à repli (`apply_inputs` `continue`,
`move_enemies` vitesse de repli, défauts de `health`) la voient à chaque tick. Mesures :

| chemin | preuve | résultat |
|---|---|---|
| headless, `run_with` (scénario, surcharges de `scenario`) | `crates/scenario/tests/character_configs.rs` : un système en `GgrsSchedule`/`FrameStart` relit `CharacterConfigs` **à chaque exécution, dont la première**, pour chaque entité `CharacterConfigKey` ; au premier tick la ressource a autant d'entrées que `GlobalAsset::character_configs` ; au moins un ennemi existe, et pour zombies et `champion_splits` au moins un est **créé par la simulation** (vu après le premier tick) | `four_players_idle` 400 frames (zombies : joueurs de carte, ennemis de vague après le délai de grâce de 180 frames), `hit_iframes` (testbed : `CharacterSpawn` de carte), `hit_iframes_isaac` (isaac), `champion_splits` (`spawn_enemy` depuis `GgrsSchedule`) : 4 verts, 0 violation |
| frames d'app (`fill_timeline`, test ignoré rejouable) | `four_players_idle` | frame d'app 2 : `CharacterConfigs` rempli (5 entrées) **et** `LobbyLocal` ; 3 : `GameLoading` ; 18 : 4 premiers personnages créés ; 20 : `GameStarting` ; **21 : premier tick**, `InGame` |
| surcharges de scénario | le test `expectations` (D66/D88 : variantes et `sight_range` retirés des fiches « avant le premier spawn ») et `extra_health` (`hit_iframes` effacé aux frames 0-1) posent leur surcharge sur `CharacterConfigs` ; leurs scénarios passent (suite complète) | verts, traces inchangées |
| surcharge qui survit, et ordre `before` | `asset_modification_refreshes_registry_but_overrides_survive` : surcharge posée sur la ressource (f10) intacte à f29 (aucun `Modified`), puis `Assets::get_mut` à f30 → `AssetEvent::Modified` → ressource recopiée ; **la première exécution de `GgrsSchedule` après la modification voit déjà la valeur rechargée et la précédente voyait la surcharge**. Mutation jouée : synchro posée `.after(RunGgrsSystems)` au lieu de `.before` → le test échoue (`left: Some(7)`, `right: Some(9)`), remise en `.before` → vert | vert |
| binaire natif `games/zombies` (`cargo run --profile headless --features native`, `ALACOD_REMOTE=1`) | journal : `loading of asset is done` à 19:37:18.067, cinq lignes `personnage « … » : CharacterConfigs rempli` à .078, `Map is loaded` à .146, `start local connection` à .151 | rempli avant la carte et la session |
| avec rendu | `play_scenario --features render --capture` (`two_players_shooting`, `champion_splits`) : visuels des personnages (sprites, barres de vie, teintes) lus dans `CharacterConfigs` | **150 + 80 images identiques à `origin/main` (`cmp`)** |
| p2p réel (headless, deux clients, rollbacks réels) | `p2p-stress.sh` zombies et isaac | voir § 4 |

### Rechargement à chaud : essai manuel

Binaire natif zombies, `--features native` (`bevy/file_watcher`), `ALACOD_REMOTE=1` (partie pilotée par `scripts/alacod-remote`), fichier
`games/zombies/assets/ZombieShooter/Sprites/Zombie/zombie_full_config.ron` modifié sur disque en cours de partie (`max_speed` 80 → 240, restauré ensuite ;
`git status games/` propre à la fin) :

- journal : `personnage « zombie_full » rechargé : CharacterConfigs mis à jour` dans la seconde (19:38:30), puis une seconde fois à la restauration (19:38:49).
  Idem pour `player_config.ron` (19:37:45 et 19:38:16) ;
- effet : vitesse mesurée sur `alacod/state` à intervalles de 6 frames : le zombie 78, créé **avant** la modification, reste à **80 px/s** (sa stat
  `EnemyMoveSpeed` est posée à la création, comme avant E3a) ; les zombies 80, 83 et 89, créés **après**, courent à **240 px/s** (231 à 240 pendant la marche).
  `CharacterConfigs` a donc été mis à jour en partie, lu par `spawn_enemy`/`create_character`. Le même essai sur `player_config.ron` (`max_speed` 150 → 225)
  ne change pas la vitesse du joueur déjà créé (74,9 px par 30 frames avant et après) : `apply_inputs` lit la stat `MoveSpeed`, comme avant E3a.

Aucun scénario ne joue ce chemin : le test `asset_modification_refreshes_registry_but_overrides_survive` en couvre le mécanisme (`Assets::get_mut` →
`AssetEvent::Modified` → ressource), l'essai ci-dessus le fichier réel.

## 3. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`5fa55fcb`) | après |
|---|---|---|
| fichiers du crate | 119 | 120 (`character/config_asset.rs`) |
| fichiers à fuite | **59** | **55** |
| erreurs de résolution | **781** | **762** |
| dont `asset` | 176 | 156 |

Fichiers **libérés de toute fuite** : `character/config.rs`, `character/health/mod.rs`, `character/enemy/ai/pathing.rs`, `familiars.rs`,
`feedback_v3.rs`. Fichiers à fuite en moins d'erreurs (`asset`) : `ui/hud.rs` 208 → 207, `character/visuals.rs` 19 → 17, `interaction.rs` 7 → 5, `champions.rs` 5 → 3,
`balance.rs` 4 → 3, `character/create.rs` 4 → 3, `character/player/create.rs` 4 → 3, `enemy/create.rs` 3 → 2, `enemy/spawning.rs` 3 → 2, `waves/systems.rs` 3 → 2,
`character/player/input.rs` 6 → 5, `feedback.rs` 39 → 38 ; nouveau : `character/config_asset.rs` (6, l'enveloppe, app par construction).
Il reste pour ces fichiers les `Assets<WeaponsConfig>`/`GlobalAsset` (E3b, E3d) et la présentation.

**Cliquet inchangé (21 fichiers).** Aucun des cinq fichiers libérés ne peut y entrer : la sonde exige aussi que sa fermeture interne (`use crate::…`) soit dans la liste.
`config.rs` dépend de `health/mod.rs` et `collider/mod.rs` (`spawn_test_wall`, sprite) ; `health/mod.rs` de `interaction.rs`, `progression.rs`, `economy.rs`,
`effects_v3.rs`, `clock.rs` ; `pathing.rs` de `collider/mod.rs`, `waves/mod.rs`, `state.rs`, `rules.rs`… ; `familiars.rs` de `balance.rs`, `create.rs`, `collider/mod.rs` ;
`feedback_v3.rs` de `feedback.rs`, `bombs.rs`. Ces fermetures contiennent encore des fichiers à fuite (E3b, E3d, E4, E6). Commentaire daté dans `scripts/d69-sim-files.txt`.
La fiche l'annonçait (« `ai/pathing.rs` si E3b n'y touche plus… ») ; à relancer le calcul de fermeture à chaque étape suivante.

## 4. Vérifications (résultats réels)

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **sortie 0** : 4 520 lignes de schedules, 645 types rollback identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS (sur la branche, `main` n'a pas bougé : `5fa55fcb`) | 3 tranches `ok`, **668 scénarios**, 267 s ; **aucun `.trace` modifié** |
| `make test_crates` | `exit 0` (82 résultats `test result: ok`, dont les 5 tests de `character_configs`) |
| `cargo fmt --all -- --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` (25 tests), `make check_sim_boundary` | OK (13 crates SIM, 2 en dette) |
| `make lint` ×5 | 0 erreur (zombies 5 personnages, testbed 39, throne 14, gungeon 11, isaac 14) |
| `make gen GAME=…` ×5 | 313 scénarios générés joués, tous `ok ok` (zombies 16, testbed 104, throne 39, gungeon 59, isaac 95), aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` | OK / OK (9 graines, 8 explosions par graine avec bombes, sans divergence) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e3a cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` | OK ×3 |
| **p2p-stress zombies** (`tests/scenarios/p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3572`, `COMPOSE_PROJECT_NAME=b3`) | **2 102 frames resimulées** (client 0, 16 rollbacks, profondeur max 11), codes de sortie 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| **p2p-stress isaac** (idem) | **1 796 frames resimulées** (16 rollbacks, profondeur max 11), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| captures `play_scenario --capture --every 2` avant/après (`two_players_shooting`, `champion_splits`) | 150 / 80 images, **0 différente** (`cmp`) |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 21 fichiers au cliquet ; § 3 |
| acceptation « 4 systèmes sans `Assets` » | `grep -c 'Assets<\|Handle<\|AssetServer'` : 0 dans `input.rs`, `health/mod.rs`, `pathing.rs` |

Mesures faites sur la branche à la fin du code ; seuls des fichiers de doc ont changé depuis (conventions, CLAUDE.md, dettes, cliquet, allowlist de déterminisme pour
deux lignes de texte). La charge machine pendant les p2p était de 7 à 9 (Mac partagé).

## 5. Décisions et limites

- **Fiche contre annexe** : la fiche dit `BTreeMap<String, CharacterConfig>`, l'annexe `Arc<CharacterConfig>` ; suivi la fiche (un clone par entrée au remplissage, jamais par frame).
- **Portée des tests** (relecture indépendante) : au premier tick, seul `first_tick` (longueur du registre contre celle de `GlobalAsset::character_configs`) discrimine ; la liste de
  violations par personnage est redondante avec les `panic!` de `create_character` et de `resolve_balance_system`, qui refusent déjà une entrée absente. Les quatre scénarios ne
  sont pas sensibles à l'ordre de la synchro par rapport à `RunGgrsSystems` (19 frames d'app séparent le remplissage du premier tick) : cet ordre est couvert par le test de rechargement
  (mutation jouée, § 2).
- **`Modified` seulement, pas `Added`** : voir § 2. Conséquence : si un jour un autre système réécrivait un asset de personnage en cours de partie
  (`Assets::get_mut`), la ressource le suivrait ; aucun système n'en écrit.
- **Le rechargement à chaud n'est pas limité au lobby** pour `CharacterConfigs` (il ne l'était pas non plus pour `Assets<CharacterConfig>`, que bevy mettait à jour
  en partie) ; développement seulement. `content_hot_reload.rs`, lui, relit toujours le registre en lobby seulement.
- **`create_character`, `spawn_familiar`, `variant_tint`, `tint_champions` perdent `GlobalAsset`** : ils ne s'en servaient que pour retrouver le handle. `spawn_enemy`/`create_player`
  le gardent (armes : E3b).
- `std::sync::Mutex` dans `character_configs.rs` (test) : convention des autres tests du crate `scenario` (`extra_health.rs`, `hit_iframes.rs`) ; `parking_lot` n'est
  une dépendance directe d'aucun crate du workspace.
- Non fait, hors périmètre (fiche) : armes (E3b), `GlobalAsset` (E3d), `Visibility`/`ActionState` des spawns (E4).

## 6. Relecture indépendante

Agent `reviewer` en lecture seule sur le diff non commité (sans cargo) : verdict **aucun bloquant**, six points OK (équivalence des replis et des panics site par site, logique et
moment de la synchro, premier tick par chemin, rollback, écarts à la fiche), cinq mineurs traités : (8) le test zombies ne jouait que 120 frames alors que le délai de grâce des
vagues est de 180 → 400 frames et vérification qu'un ennemi est créé après le premier tick ; (9) l'ordre `before` n'était pas testé → assertion dans le test de rechargement, mutation
`.after` jouée ; (11) règle de `CLAUDE.md` nuancée (cible, atteinte pour les personnages seulement) ; (12)-(13) rapport corrigé (motif du `grep`, « vagues ») ; (3) changement du
diagnostic `sight_probe` signalé. Les chiffres du § 4 ne sont pas vérifiés par le relecteur (aucun cargo) : ce sont les miens.
