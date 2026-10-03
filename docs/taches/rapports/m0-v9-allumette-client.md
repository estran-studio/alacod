SHA de tête vérifiée avant le commit de ce rapport : `7ab256c` (code).
Fiche : `docs/taches/m0-v9-allumette-client.md` — m0-v9, D12, client allumette natif.
Branche : `m0-v9-allumette-client` ; référence `main` : `b021dc5`.
Date : 2026-10-03 ; agent : Claude Code (voie b0, tâche m0-v9).

Le commit de livraison ajoute ce rapport et la mise à jour de `dettes.md` (D12)
au SHA vérifié ci-dessus ; son SHA est donné dans la ligne LIVRÉ.

## Fait

- `crates/game/src/jjrs/allumette.rs` (nouveau, ~480 lignes avec tests) : client HTTP
  allumette complet, **hors simulation** (tourne uniquement en phase de lobby, rien
  dans le rollback GGRS). Flux : paire Ed25519 éphémère (`getrandom` 32 octets →
  `SigningKey::from_bytes`) → `POST /auth/challenge` → signature `verify_strict` de
  la chaîne challenge telle quelle (octets UTF-8, rien d'autre) → `POST /auth/login`
  → JWT → lobby (`--lobby <uuid>` = join direct ; sinon découverte du premier lobby
  `Waiting` de `game_id "zombies"` puis join, ou création) → attente du complet par
  le créateur (`GET /lobbies` toutes les secondes, 60 sondages max, car la topologie
  marque le lobby `InProgress` dès que le propriétaire connecte le WS) →
  `GET /ice-servers` → URL `ws(s)://hôte/<JWT>` (dérivée de l'URL HTTP, slash final
  toléré). Encodages base64 STANDARD (même alphabet que le serveur, base64 0.21).
  4 tests unitaires : dérivation http→ws, fixture `LobbyResponse` au format exact du
  serveur (champs ignorés inclus), filtre de découverte (game_id + Waiting),
  aller-retour de signature `verify_strict` (dont rejet d'un message voisin).
- `crates/game/src/jjrs/p2p.rs` : `start_matchbox_socket` prend
  `Option<Res<AllumetteConfig>>` (natif uniquement, cfg-gated) — en mode allumette,
  URL `ws(s)://hôte/JWT` et ICE de l'API au lieu de `{matchbox_url}/{lobby}` et du
  STUN en dur ; le chemin `--matchbox` historique est **inchangé** (STUN Google
  extrait en `default_ice_server()`, même code qu'avant). Le builder matchbox
  n'accepte qu'un seul `RtcIceServerConfig` : la première entrée de `/ice-servers`
  est utilisée, repli sur le STUN par défaut si l'API n'en renvoie aucune.
- `crates/game/src/core.rs` : en natif, `OnEnter(LobbyOnline)` chaîne
  `start_allumette_flow` puis `start_matchbox_socket` (l'insertion de
  `AllumetteConfig` est visible du second via l'`apply_deferred` du chaînage) ;
  wasm garde le système unique historique.
- `crates/game/src/args/cli.rs` + `args/mod.rs` : flag `--allumette <url>` (short
  `-a`, `conflicts_with = "matchbox"`) ; `GameArgs.allumette` rempli en natif, vide
  en wasm ; `OnlineState::Online` et `GggrsSessionConfiguration` alimentés par
  matchbox **ou** allumette. `crates/scenario/src/runner.rs` : champ `allumette:
  String::new()` (constructeur de test inchangé dans les faits).
- `crates/game/src/jjrs/mod.rs` : `pub mod allumette` cfg-gated natif ;
  `GggrsSessionConfiguration.allumette_url`.
- `crates/game/Cargo.toml` : 4 dépendances sous
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` — `ureq 2.12.1`
  (HTTP bloquant, pas de tokio dans le jeu), `ed25519-dalek 2.2.0`,
  `base64 0.21.7` (même version que le serveur), `getrandom 0.2.17` (entropie ;
  évite `rand::`, interdit par `check-forbidden.sh` dans le jeu).
- Aucune UI, aucun `.trace` modifié, `docs/taches.md` intact, serveur allumette
  non patché (le contrat a été validé par scripts python hors dépôt).

## Vérifié

Toutes les commandes ont été précédées de `source ../env.sh` (worktree
`m0-v9-allumette-client`), profil `headless`, `CARGO_BUILD_JOBS=4`, une
invocation Cargo à la fois.

- **`cargo test --profile headless -p game --lib`** : `test result: ok. 26 passed;
  0 failed` — dont les 4 tests allumette :
  `select_waiting_lobby_filters_game_and_status`, `lobby_response_from_server_fixture`,
  `http_to_ws_derive`, `challenge_signature_roundtrip`, tous `ok`.
- **`cargo build -q -p zombies --profile headless --no-default-features`** : rc 0,
  binaire `target/headless/zombies` (1 011 199 696 octets) ; warnings uniquement
  pré-existants (`map_ldtk`), aucun dans les fichiers de cette tâche.
- **`make test_scenarios`** : `test scenarios ... ok`, `test result: ok. 2 passed;
  0 failed; 7 ignored`, 414,85 s, rc 0 — **`git status` final : 0 fichier `.trace`
  modifié sur 62 bénis** (la preuve « les 62 traces ne bougent pas »).
- **Suite complète** (`cargo test -q --profile headless -p scenario -p run -p
  combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots -p effects
  --no-fail-fast`) : rc 0, **0 échec** — 31 binaires de test `ok` (dont
  `scenario` 438,5 s, `bots` 119,2 s ; 16+44+64+36+11+26+13+32+39 tests unitaires
  et d'intégration, 0 failed partout).
- **Recette réelle via allumette** (serveur `allumette_server` frais sur 0.0.0.0:3536,
  puis deux clients headless décalés de 6 s — le créateur d'abord, le rejoigneur
  ensuite — `ALACOD_HEADLESS=1 ALACOD_STATE_TRACE=/tmp/allu-$i.trace
  ALACOD_EXIT_AT_FRAME=600 target/headless/zombies --allumette
  http://127.0.0.1:3536 --number-player 2 --players localhost remote`) : les deux
  clients jouent la partie complète jusqu'à la frame 600 (log ggrs identique des
  deux côtés), **`cmp /tmp/allu-0.trace /tmp/allu-1.trace` : identiques** (23 850
  octets chacun). Log serveur : deux `Found player_id`, `Owner connected — lobby
  marked InProgress`, aucun `error` ; deux `WARN
  ResetWithoutClosingHandshake` bénins (le client quitte le processus à la frame
  600 sans handshake de fermeture WS).
- **Recette §4 `--matchbox` rejouée** (`docker compose -f docker-compose.ci.yaml up
  -d signaling`, deux clients `--matchbox ws://127.0.0.1:3536 --lobby test-$$`,
  binaire déjà compilé, aucune compilation) : **`cmp /tmp/p2p-0.trace
  /tmp/p2p-1.trace` : identiques** (23 850 octets) — le chemin historique n'a pas
  bougé.
- **`make lint`** : `games/zombies : aucune erreur (4 personnages, 4 armes, 6 armes
  de corps à corps, 1 vagues, 3 cartes)` et `games/testbed : aucune erreur`, rc 0.
- **`cargo fmt --all -- --check`** : rc 0 (après un `cargo fmt --all` appliqué à
  `allumette.rs` — retours à la ligne uniquement ; les tests et le binaire ont été
  rejoués/reconstruits sur le code formaté : `cargo test -p game --lib` re-vert
  (26 passed, 0 failed) et build `zombies` rc 0).
- **`scripts/check-forbidden.sh`** : 4 occurrences, **toutes pré-existantes**
  (`character/enemy/ai/state.rs` HashSet, `map_ldtk/src/game/plugin.rs` ×2,
  `bevy_fixed/src/math.rs` commentaire `rand::`) — aucune dans les fichiers de
  cette tâche ; le script est un warning par défaut, pas un échec.
- **`scripts/check-rollback-registration.sh`** : `OK: aucun appel direct à
  rollback_component_with*/rollback_resource_with* hors de crates/utils/src/rollback.rs`.
- **`make gen GAME=zombies`** : rc 0, `git status` avant/après identique — aucun
  scénario généré ne change (aucune modification de contenu).

## Non fait / incertain / non vérifié

- **`InProgress` vu depuis le jeu** : le log serveur de la recette réelle montre
  bien `Owner connected — lobby marked InProgress`, mais le client n'a pas de
  vérification du statut — le statut est un garde serveur, pas une attente client.
- **Retour au lobby après déconnexion** : l'appartenance au lobby survit à une
  déconnexion (contrat serveur), mais le jeu sort à la frame 600 — le retour en
  lobby n'est pas exercé par cette tâche.
- **Reconnexion / lobby plein / lobby disparu** : chemins d'erreur couverts par
  `AllumetteError` + `exit(1)` net (pas de socket sans JWT), mais pas rejoués en
  partie réelle.
- **Un seul serveur ICE effectif** : le builder matchbox du fork n'accepte qu'un
  seul `RtcIceServerConfig` (`ice_server()` remplace) — la première entrée de
  `/ice-servers` est prise, le reste est ignoré (documenté dans le code).
- **Course de création** : deux clients démarrés strictement en même temps peuvent
  chacun créer un lobby (le second créera le sien faute de lobby `Waiting`) —
  limite v1 connue, les recettes démarrent les clients en décalé.

## Dettes laissées, questions ouvertes

- `getrandom` au lieu de `rand` pour la paire Ed25519 : `check-forbidden.sh`
  interdit `rand::` dans `crates/game/src` — `SigningKey::from_bytes` sur 32
  octets `getrandom` est équivalent (les deux passent par le RNG du système).
- Le commentaire de `docker-compose.ci.yaml` (service `signaling`) dit encore que
  « le jeu natif n'implémente pas encore » le flux allumette — **devenu faux**,
  mais laissé intact (la recette CI ne doit pas bouger) ; à mettre à jour quand la
  CI de nuit passera au profil `allumette` (reste de la ligne D12).
- Serveur allumette : les lobbies `Waiting` orphelins (créateur parti sans
  rejoigneurs) ne sont pas nettoyés — côté serveur, hors périmètre.
- `start_allumette_flow` est bloquant dans `OnEnter(LobbyOnline)` (sondage 1 s ×
  60) : c'est voulu pour la phase de lobby headless, mais un client fenêtré
  gèlerait sa boucle pendant l'attente du complet — à rendre asynchrone si une UI
  de lobby allumette arrive.
- D12 : cellule de suivi passée à « fait dans `m0-v9-allumette-client` ».
