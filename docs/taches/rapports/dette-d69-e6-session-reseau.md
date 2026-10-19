# dette-d69-e6-session-reseau — D69 étape 2, E6 : session, réseau, arguments, télémétrie

Worker b0, 2026-10-18/19, branche `dette-d69-e6-session-reseau` (depuis `dette-d116-lobby-admission` `1e52c2a8`, elle-même fusionnée avec `origin/main` `8ca55686` ; `origin/main` n'a pas bougé depuis).
Mac arm64 partagé (charge 9 à 46 : b1 et b3). Fiche : `docs/taches/dette-d69-etape2.md` § E6, recettes `annexe-session.md` §2.2-§2.5 et §5.

## 1. Résultat

| À faire (fiche) | État |
|---|---|
| 1. `jjrs/p2p.rs` en deux moitiés ; `wait_for_players` et `system_after_map_loaded` coupés ; appariement joueurs ↔ configuration en fonction de simulation sans socket | **fait** : §2 |
| 2. `drop_matchbox_socket` → app, son `OnExit(InGame)` suit | **fait** : `jjrs/p2p_socket.rs`, enregistré dans `core.rs` |
| 3. `log_ggrs_events` sans télémétrie, message `DesyncObserved`, `jjrs/telemetry.rs` : `forward_desync_telemetry` | **fait** |
| 4. `GameArgsPlugin` ne pose plus `TelemetryConfig` ; `TelemetryConfigPlugin(GameArgs)` app, ajouté par `BaseArgsPlugin` après lui ; `scenario` sans télémétrie | **fait** : `args/telemetry_config.rs` ; `TelemetryPlugin` n'est jamais ajouté par `scenario` (grep : seuls les champs `telemetry*` de `GameArgs` y sont posés) |
| 5. `recording.rs` : `.after(bevy::window::ExitSystems)` retiré | **fait**, l'ordre d'avant est conservé par `ExitSystems.before(ExitRequests)` côté application (`core.rs`) |
| 6. `run_state.rs` (wasm) : message de simulation consommé par un système app | **fait** : `ReturnToHostLobby` + marqueur `HostedSession`, `jjrs::browser::return_to_host_lobby` ; **compilé seulement** (build web), pas exécuté |
| 7. `frame.rs` : sans `increase_frame_system` (E1), tout app | **rien à faire** : seul `core.rs` (`FrameDebugUIPlugin`) l'utilise |
| 8. `p2p_tests.rs` : `PeerId` de `matchbox_protocol` | **fait**, dépendance `matchbox_protocol` ajoutée à `game` (même révision que `combat`) |
| Porte commune, p2p, build web | **verts** : §4, §5 |
| Cliquet | 22 → **28 fichiers** : §3 |

## 2. Coupes

Une moitié à son chemin final, l'autre dans un fichier de nom nouveau (la fiche impose le chemin final `jjrs/p2p.rs` pour la moitié application, mais le chemin existe déjà côté simulation tant que `game` est un
seul crate) : la moitié application s'appelle **`jjrs/p2p_socket.rs`** (E9 : `git mv` vers `game_app/src/jjrs/p2p_socket.rs`).

| Moitié | Contenu |
|---|---|
| **simulation** `jjrs/p2p.rs` (+ `p2p_tests.rs`) | `PreviousSessionPeers`, `CurrentSessionPeers`, **`SessionPeerLeft`** (nouvelle ressource), `live_players`, `Admission`/`admit_players`/`AdmittedPlayers` (D116), `DISCONNECT_TIMEOUT(_ENV)`, `disconnect_timeout`, `secs_from_env`, **`pair_session_players`** (appariement, sans socket), **`start_online_run`**, `SYNC_TIMEOUT_*`, `SessionSyncWatch`, `sync_timed_out`, `reset_sync_watch`, `abandon_unsynchronized_session` (sans socket), `MAX_PREDICTION_WINDOW`, `session_builder` ; `PeerId` = `matchbox_protocol::PeerId` |
| **application** `jjrs/p2p_socket.rs` (+ `p2p_socket_tests.rs`) | `CHANNEL_ENV`, `channel_config`, `default_ice_server`, `CONNECT_*`, `ADMISSION_SETTLE_SECS`, `ConnectNotice`, `RoomFull`, `ConnectTimeoutAction`, `LobbyConnectWatch` (porte `SharedSightings` de `signal_watch`), `connect_*`, `build_socket`, `start_matchbox_socket`, `open_matchbox_socket`, `wait_for_players`, `session_peer_left`, **`observe_session_peers`**, **`build_p2p_session`**, **`drop_matchbox_socket`** (venu de `restart.rs`) |
| autres fichiers application | `jjrs/signal_watch.rs` (D109, ses dépendances natives `async-tungstenite`, `futures-util`, `futures-timer` sont celles de la moitié application), `jjrs/allumette.rs`, `jjrs/browser.rs`, `jjrs/telemetry.rs` (**nouveau**), `args/telemetry_config.rs` (**nouveau**) |

Décisions de coupe :
- **`system_after_map_loaded`** → `build_p2p_session` (app : canal du socket, `session_builder`, `previous`/`current`, `Session::P2P`) et `start_online_run` (sim : graine, `RngStreams`, `Run`, déblocages, `AppState::InGame`), dans le même tuple `OnEnter(GameStarting)` que `system_after_map_loaded_local`. Aucun ordre n'existait entre les systèmes du tuple, les commandes sont appliquées au même point de synchronisation ; ordre des deux nouveaux systèmes sans effet.
- **`wait_for_players`** reste en application (il lit `socket.players()`, `connected_peers`, `try_update_peers`) et appelle la simulation : `live_players`, `admit_players`, `pair_session_players(&players, &config, identity)` ; `identity` est la seule porte du web (`browser_mapping`, `data-participants`).
- **`abandon_unsynchronized_session`** : l'ancien système lisait le socket (`try_update_peers`) pendant `Synchronizing` ; `observe_session_peers` (app) fait cette lecture, mêmes gardes (en ligne, pas de `RunRequest`, session P2P en `Synchronizing`) et écrit `SessionPeerLeft` ; la simulation décide (`session_peer_left` ou `session_sync_timeout`) ; les deux sont chaînés dans le même update. `reset_sync_watch` remet `SessionPeerLeft` à faux à chaque session neuve. La fiche ne le prévoyait pas : `session_peer_left` (D109) est postérieur à la fiche ; `session_peer_left(&[(PeerId, PeerState)])` reste côté application, `PeerState` est un type de `matchbox_socket`.
- **Télémétrie** : `log_ggrs_events` n'a plus de paramètre `telemetry`/`chrono` ; il écrit `DesyncObserved { frame, local_checksum, remote_checksum, addr_debug }` au même instant que l'erreur journalisée ; `forward_desync_telemetry` (app, `.chain()` après lui, `InGame`) construit le `TelemetryEvent` (horloge `chrono` comprise) et l'envoie. Sans `TelemetrySender` (scénarios), les messages sont lus et ignorés.
- **D109/D116 côté application** : `LobbyConnectWatch`, `ConnectNotice`, `RoomFull`, `connect_status_text` (ligne d'état du lobby, `ui/lobby.rs` migré) ; l'admission (`admit_players`, `AdmittedPlayers`) est de la simulation pure.
- **Imports morts retirés** (signalés par `unused_imports` avant la sonde) : `collider::spawn_test_wall` de `jjrs/mod.rs` et de `jjrs/local.rs` (ils gardaient `jjrs/mod.rs` hors du cliquet), `SinkExt` de `signal_watch.rs`.
- `scripts/determinism-allowlist.json` : les six `Instant::now()` du lobby suivent `p2p.rs` → `p2p_socket.rs`.

## 3. Sonde `make probe_game_sim --tests` (avant / après)

| | avant (`1e52c2a8`) | après |
|---|---|---|
| fichiers à fuite / total | 50 / 122 | **49 / 126** (4 fichiers application ajoutés : `p2p_socket.rs`, `p2p_socket_tests.rs`, `telemetry.rs`, `telemetry_config.rs`) |
| erreurs de résolution | 791 | **789** |
| catégories | crate d'E/S 17, fenêtre 10, réseau 8 | crate d'E/S 16, fenêtre 10, réseau 7 |

Fichiers sortis de la liste des fuites : `jjrs/mod.rs` (4), `jjrs/p2p.rs` (2), `jjrs/p2p_tests.rs` (2), `jjrs/restart.rs` (1), `recording.rs` (1) ; `args/mod.rs` 2 → 1 (reste `telemetry::TelemetryPlugin` de `BaseArgsPlugin`, application, à E9) ; `core.rs` 11 → 12 (le `bevy::window::ExitSystems` posé côté application).

**Cliquet** (`scripts/d69-sim-files.txt`, 22 → **28 fichiers**) : + `jjrs/mod.rs`, `jjrs/restart.rs`, `jjrs/p2p_tests.rs`, `curses.rs`, `clock.rs`, `character/enemy/ai/mod.rs` (les deux derniers : leur fermeture, sans fuite, est devenue fermée avec `jjrs/mod.rs` ; calculé par point fixe sur la sonde). Fichiers de la fiche qui **n'entrent pas** (sans fuite propre, fermeture interne à fuite) :
`jjrs/p2p.rs`, `jjrs/local.rs`, `profile.rs`, `run_state.rs`, `hub.rs`, `state_trace.rs`, `recording.rs` — la chaîne est `run_state.rs` → `balance.rs`, `economy.rs` (asset, E3c), `waves/mod.rs` (asset) ; `hub.rs` → `interaction.rs`, `shop.rs` (E5c) ; `recording.rs` → `character/player/input.rs` (entrée, E4), `replay.rs`, `progression.rs`. `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` : sortie 0.

## 4. Vérifications

| Commande | Résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **OK** : ordre des systèmes et des types rollback identiques (4 520 lignes de schedules, 645 types) |
| `make test_scenarios_par SHARDS=3` (sans BLESS) | 802 scénarios, 3 tranches ok, 445 s ; `git status` propre hors docs |
| `make test_crates` | sortie 0, 83 `test result: ok`, aucun échec |
| `make lint` ×5 | zombies, testbed, throne, gungeon, isaac : 0 erreur |
| `make gen GAME=…` ×5 | sortie 0 : zombies 16, testbed 104, throne 39, gungeon 59, isaac 213 scénarios générés, tous `ok ok`, aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / avec `ALACOD_FUZZ_BOMBS=1` (`--ignored`) | OK / OK |
| `cargo fmt --all -- --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` | verts |
| `make check_sim_boundary` | OK : 13 crates, 2 en dette D69 |
| build web `APP_VERSION=v0.0.0 cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` | sortie 0 (compile `HostedSession`, `ReturnToHostLobby`, `pair_session_players` avec `identity` web, `build_p2p_session` wasm) |

## 5. p2p réel (signaling `matchbox_server` local, port 3570)

| Script | Résultat |
|---|---|
| `p2p-stress.sh zombies` (3 000 frames) | codes 0 0, 0 `DesyncDetected`, **2 056** / 0 frames resimulées, traces identiques (2 999 lignes, `check-p2p-traces.py`) |
| `p2p-stress.sh isaac` (`ALACOD_SKIP_HUB=1`) | codes 0 0, 0 `DesyncDetected`, **1 842** / 0 frames resimulées, traces identiques (2 999 lignes) |
| `p2p-restart.sh` | OK (partie 2 identique à la partie 1, 599 lignes) |
| `p2p-lobby.sh` (5 essais) | 5 / 5 verts consécutifs dans la bande 100-400 ms, 0 abandon de session |
| `p2p-disconnect.sh` (victime tuée à 15 s) | invariants OK, survivant terminé, 0 `DesyncDetected` |
| `p2p-sync-timeout.sh` | OK, traces identiques (299 lignes) |
| `p2p-connect-timeout.sh` `ATTEMPTS=1` / `2` | OK / OK, traces identiques (299 lignes) |
| `p2p-connect-timeout.sh GHOST=1` (D116) | `attempt=1/2 action=retry` puis `attempt=2/2 action=lobby` |
| `p2p-admission.sh` (trois clients, deux places, 6 verts demandés) | 6 verts, **2 courses perdues** sur 8 (limite assumée de D116 : un client a démarré avant que le signaling lui annonce le troisième) ; traces des deux admis identiques (1 499 lignes), 0 `DesyncDetected` ; `GHOST=1` OK |

## 6. Limites

- Le web n'est **que compilé** : `HostedSession` / `ReturnToHostLobby` / le chemin `identity` de `pair_session_players` n'ont pas été exécutés dans un navigateur.
- `core.rs` reste mixte (E7) : les enregistrements de la moitié application (`start_matchbox_socket`, `wait_for_players`, `build_p2p_session`, `observe_session_peers`, `forward_desync_telemetry`, `LobbyConnectWatch`) y sont encore, avec les chemins corrigés.
- `args/mod.rs` garde un `telemetry::TelemetryPlugin` (dans `BaseArgsPlugin`, application) : E9.
- Cliquet : sept fichiers de la fiche (`jjrs/p2p.rs`, `jjrs/local.rs`, `recording.rs`, `run_state.rs`, `profile.rs` ; `hub.rs`, `state_trace.rs`) sont sans fuite propre mais pas fermés (§3).

## 7. Relecture indépendante

Relecteur (agent `reviewer`, lecture seule, sur `9b76c88b`) : **aucun défaut**. Équivalence constatée par relecture de l'ancien et du nouveau code : `system_after_map_loaded` → `build_p2p_session` + `start_online_run` (mêmes gardes, mêmes commandes,
rien ne dépend de l'ordre des deux systèmes du tuple `OnEnter`) ; `observe_session_peers` → `abandon_unsynchronized_session` (`try_update_peers` appelé dans les mêmes cas, un départ ni perdu ni compté deux fois) ; `pair_session_players` natif et web
(`identity` rejoint l'ancien bloc `browser_mapping`) ; aucune fuite restante (`bevy_matchbox`, `telemetry`, `bevy::window`) dans `p2p.rs`, `mod.rs`, `restart.rs`, `recording.rs`, `run_state.rs` ; `ExitSystems.before(ExitRequests)` sans cycle (bevy_ecs 0.19.1 relie les voisins d'un set vide),
seul changement d'ordre : `write_trace_at_exit_frame` après `exit_on_all_closed`, deux écritures d'`AppExit` sans effet visible ; `apply_run_request_system` à 16 paramètres sous wasm, au maximum de Bevy.
