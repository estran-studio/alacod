# m0-v9 — client allumette dans le jeu natif (D12)

> Branche : `m0-v9-allumette-client` · Worktree : `alacod_tasks/m0-v9-allumette-client/alacod`
> (source `env.sh` du dossier de tâche avant de compiler) · Début : 2026-10-03 (orchestrateur).

## Contexte

Dette **D12** (`docs/taches/dettes.md`) : le jeu natif ne parle pas à **allumette**. Aujourd'hui
le client se connecte en matchbox nu : `--matchbox ws://hôte` et le nom de salle est mis dans le
chemin du WebSocket (`crates/game/src/jjrs/p2p.rs`, `start_matchbox_socket` :
`format!("{}/{}", matchbox_url, lobby)`). Or allumette (sous-repo `../allumette`, crate
`allumette_server` v0.11.0, matchbox_signaling 0.13) exige un **JWT dans le chemin du WebSocket**,
obtenu par un flux HTTP d'authentification par challenge — c'est pour ça que la CI de nuit
(T2.14) utilise un `matchbox_server` nu.

## Le contrat exact du serveur

Tout est dans `../allumette/src/` — vérifie les lignes citées (elles peuvent avoir bougé d'un
commit) : `lib.rs` (routes `app()`, handlers ~395-870), `auth.rs` (challenge, JWT, vérification
Ed25519), `lobby.rs` (DTO), `topology.rs` (mise en relation des pairs).

1. `POST {base}/auth/challenge` → `{"challenge": "<32 caractères alphanumériques>"}` — expire
   après 60 s, **usage unique** (consommé par le login).
2. `POST {base}/auth/login`, corps JSON
   `{"public_key_b64", "username", "challenge", "signature_b64"}` → `{"token": "<JWT HS256>"}`.
   - `signature_b64` = signature Ed25519 (`verify_strict` côté serveur) du message qui est **la
     chaîne challenge telle quelle** (ses octets UTF-8, sans rien d'autre).
   - Encodages en base64 **STANDARD** — le serveur utilise base64 0.21.
   - JWT : `sub` = `public_key_b64`, `username`, `exp` = +24 h ; secret `JWT_SECRET`
     (défaut `test-secret-key-for-development-only`).
   - 401 sur challenge invalide/expiré ou signature invalide.
3. `GET {base}/lobbies` (Bearer optionnel) → tableau de `LobbyResponse` :
   `{id (UUID), game_id, is_owner, player_count, players: [{publicKey, is_you}], status:
   "Waiting"|"InProgress", is_private, is_whitelisted}`. `players` est **vide** tant qu'on n'est
   pas membre du lobby (les clés des autres ne sont pas exposées).
4. `POST {base}/lobbies` (Bearer), corps `{"is_private", "game_id", "whitelist"?}` →
   `LobbyResponse` — le créateur est ajouté d'office ; 409 s'il est déjà dans un lobby.
5. `POST {base}/lobbies/{id}/join` (Bearer) → rejoint le lobby.
6. `POST {base}/lobbies/{id}/start` (Bearer) — **inutile pour le flux du jeu** : la topologie
   marque le lobby `InProgress` toute seule quand son **propriétaire** connecte son WebSocket
   (et interdit les nouvelles arrivées).
7. `GET {base}/ice-servers` (Bearer) → `[{"urls": [...], "username"?, "credential"?}]`
   (STUN par défaut + TURN si le serveur est configuré).
8. **WebSocket** : `ws(s)://{hôte}/{JWT}` — le jeton est **dans le chemin** (il n'y a pas de nom
   de salle) ; le serveur décode `sub` → pubkey, retrouve le lobby du joueur (inscrit par l'API
   HTTP) et met en relation les membres connectés d'un même lobby (`NewPeer`/`PeerLeft` entre
   eux seulement). L'appartenance au lobby survit à une déconnexion (partie suivante).

`{base}` = URL http(s) de l'API ; l'URL du WebSocket s'en déduit : `http`→`ws`, `https`→`wss`,
chemin = `/` + JWT.

## Périmètre (v1)

Fichiers attendus : `crates/game/src/args/cli.rs` (+ `args/mod.rs` pour la config),
**nouveau module** `crates/game/src/jjrs/allumette.rs` (+ tests unitaires dans le module),
`crates/game/Cargo.toml` (dépendances), docs (cette fiche, `dettes.md` ligne D12).

**Ne pas toucher** : le chemin `--matchbox` existant (la CI et la recette p2p du README §4 en
dépendent), `crates/game/src/ui/lobby.rs` (UI), les scénarios et leurs traces, le serveur
allumette lui-même (si tu trouves un bug bloquant côté serveur : `BLOQUÉ` avec le détail, ne le
patche pas toi-même).

Ce que ça doit faire :
- Nouveau flag `--allumette <base-http-url>` (clap, `conflicts_with` `--matchbox`).
- En mode allumette, **avant** d'ouvrir le socket (phase lobby, hors simulation) : générer une
  paire Ed25519 éphémère (une par partie, pas de persistance en v1) ; challenge → login (jeton) ;
  rejoindre ou créer un lobby de `game_id` **`"zombies"`** ; récupérer `/ice-servers`.
  - `--lobby <uuid>` fourni = rejoindre ce lobby précis (il doit exister, ex. créé par une UI).
  - Sinon, découverte automatique : chaque client **liste** (`GET /lobbies`, filtre
    `game_id == "zombies"` et `status == "Waiting"`) et **rejoint** s'il en trouve un ; sinon il
    **crée** (`is_private: false`). Deux clients démarrés en même temps peuvent chacun créer
    (course connue, limite v1 documentée) : dans les tests et les recettes, démarrez les
    clients en décalé de quelques secondes, comme la recette p2p du README §4 le fait déjà.
  - **Le créateur attend que son lobby soit au complet avant d'ouvrir le WebSocket** : il
    connaît l'id (réponse du `POST /lobbies`) et interroge `GET /lobbies` jusqu'à
    `player_count == --number-player` (avec un délai raisonnable) — sinon la topologie marque le
    lobby `InProgress` dès que le propriétaire connecte et les autres ne peuvent plus rejoindre.
    Les rejoigneurs, eux, ouvrent leur WebSocket dès que le join a réussi.
- `start_matchbox_socket` (`jjrs/p2p.rs`) consomme alors l'URL `ws(s)://hôte/JWT` et les ICE
  reçus de l'API (au lieu de `{matchbox_url}/{lobby}` et du STUN Google en dur). Le plus propre :
  une ressource (ex. `AllumetteConfig { ws_url, ice_servers }`) remplie pendant le flux HTTP et
  lue par `start_matchbox_socket`.
- Client HTTP : **ureq 2.x** (bloquant, léger — pas de tokio dans le jeu). Dépendances
  nouvelles : `ureq`, `ed25519-dalek` 2.x, `rand` 0.8 (génération de la paire), `base64`
  (**STANDARD**, aligné sur la version du serveur). Rien de tout ça n'entre dans la simulation :
  pas de contrainte fixed-point ici, et aucune trace ne doit bouger.

## Critères d'acceptation

1. **Traces intactes** : `make test_scenarios` vert, `git status` sans aucun `.trace` modifié
   (le mode allumette est hors simulation — c'est la preuve principale).
2. **`--matchbox` inchangé** : la recette p2p du README §4 (matchbox nu en Docker + deux clients
   headless + `cmp` des traces) doit rester jouable — l'orchestrateur la rejoue après merge.
3. **Partie réelle via allumette** : serveur local déjà compilé par l'orchestrateur
   (`../allumette/target/debug/allumette_server`, écoute sur `0.0.0.0:3536` par défaut), deux
   clients zombies headless, démarrés en décalé (recette p2p) :
   `ALACOD_HEADLESS=1 ALACOD_STATE_TRACE=/tmp/allu-$i.trace ALACOD_EXIT_AT_FRAME=600
   target/headless/zombies --allumette http://127.0.0.1:3536 --number-player 2
   --players localhost remote` puis `cmp` des deux traces : **identiques**. Joue-le toi-même et
   mets les commandes exactes + le résultat dans le rapport (si tu n'y arrives pas : « non
   fait » honnête, l'orchestrateur le jouera).
4. **Tests unitaires** des parties pures : dérivation http→ws, désérialisation d'un
   `LobbyResponse` depuis une fixture JSON, signature du challenge qui se vérifie
   (round-trip signer/vérifier en local).
5. Suite standard verte : `cargo test` de la crate `game` au moins, `make lint` des deux jeux,
   `cargo fmt --all -- --check`, `./scripts/check-forbidden.sh`,
   `./scripts/check-rollback-registration.sh`, `make gen GAME=zombies` sans modification.

## Règles du worktree (rappel README §1)

- `source ../env.sh` (exporte `CARGO_TARGET_DIR` vers le target amorcé de la tâche) avant toute
  commande cargo. Toujours `--profile headless`, `--no-default-features` pour les binaires de
  jeu. Une seule commande cargo à la fois. **Codex compile en parallèle dans son worktree : ne
  lance jamais de compile supplémentaire en même temps que la tienne** — la tienne est déjà la
  2e autorisée.
- Pas de `git stash`, pas de push sur `main`, pas de modification de `docs/taches.md` (journal
  de l'orchestrateur) — sauf `dettes.md` : passe la ligne D12 à « fait dans
  `m0-v9-allumette-client` » en fin de lot.

## Livrer

1. Commits propres sur la branche (messages en français, attribution
   `Co-Authored-By: Claude Code <noreply@anthropic.com>` en fin de message), puis
   `git push -u origin m0-v9-allumette-client`.
2. Rapport `docs/taches/rapports/m0-v9-allumette-client.md` **sur la branche, commité et
   poussé** : format README §7 (fait / vérifié avec les commandes et leurs chiffres / non fait /
   dettes), sha de tête en tête de rapport.
3. Termine par `LIVRÉ m0-v9-allumette-client <sha>` (ou `BLOQUÉ … : <pourquoi>`).
4. Attends la réponse. Ne merge pas.
