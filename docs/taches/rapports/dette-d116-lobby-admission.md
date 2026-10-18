# dette-d116-lobby-admission — politique d'admission du lobby en ligne (D116)

Worker b0, 2026-10-18, branche `dette-d116-lobby-admission` (depuis `origin/main` `2b15376a`, fusionnée avec `5823c032` avant les vérifications finales). Mac M1 Pro partagé
(charge 9 à 46 pendant les essais : b1 et b3). Signaling `matchbox_server` d'OrbStack (`b0-signaling`, port 3570, déjà en ligne : les scripts l'utilisent par `MATCHBOX_URL`).

## 1. Résultat

| Point demandé | État |
|---|---|
| Politique d'admission déterministe, identique chez tous les clients | **faite** : `jjrs::p2p::admit_players` garde les `max_player` premiers joueurs du socket (identifiants triés, ordre commun), les autres sont ignorés et prévenus (§2) |
| Les admis jouent, traces identiques | **prouvé** : `scripts/p2p-admission.sh`, trois clients dans une salle à deux places, 17 essais verts sur 17 (§3) |
| Le troisième retourne au lobby ou attend proprement | **fait** : ligne d'état « Room full », puis retour au lobby au délai de D109, en tenant les admis pour occupés (§2, §3) |
| Réserve de validation : second délai complet `attempt=2/2` avec un pair resté visible | **fermée** : `scripts/p2p-connect-timeout.sh` avec `GHOST=1` (§4) |
| Aucune trace ne bouge | **vérifié** : suite 802 scénarios sans BLESS, `git status` propre (§5) |

## 2. Politique

`wait_for_players` (natif et web) :

1. `live_players(socket.players(), previous)` : joueurs du socket, sans les pairs d'une session déjà jouée (D86). `socket.players()` est trié par identifiant, le joueur local compris : même ordre chez tous les pairs qui voient le même ensemble, et les handles GGRS en dépendent.
2. **Moins de joueurs que `max_player`** : attente, comme avant (délai de D109 sur un pair vu par le signaling et jamais connecté).
3. **Pairs en trop** (`admit_players`) : les `max_player` premiers sont admis, les autres **ignorés**. Journal `ggrs{f=0 lobby_admission peers=3 max=2 excess=1 local=admitted|ignored}`, une fois par changement.
   - Admis : démarrent **sans attendre** les pairs en trop. `AdmittedPlayers` (nouvelle ressource) fixe la liste et `system_after_map_loaded` la reprend telle quelle : un pair arrivé pendant le chargement de la carte ne change ni les handles ni la taille de la session (avant : `socket.players()` relu, panique possible).
   - Ignoré : `LobbyConnectWatch::room_full` (ligne d'état du lobby : « Room full (3 peers for 2 places): waiting for a free place »), attend une place ; au bout de `ALACOD_CONNECT_TIMEOUT_SECS` (15 s, horloge murale, comme D109) : `ggrs{f=0 lobby_room_full peers=3 max=2 action=lobby waited_secs=3.0}`, avis « back to the lobby » 20 s, `LobbyOnline` ré-entré.
   - Les admis rejoignent `PreviousSessionPeers` au retour au lobby de l'ignoré. **Piège constaté au premier jet** : sans cela, le socket neuf de l'ignoré revoit les deux pairs en jeu, se classe parmi les admis (2 chances sur 3) et construit une session que personne n'écoute (`session_peer_left` 12 s plus tard, quand les autres ont fini) ; 1 essai sur 2.
4. **Pair vu par le signaling, pas encore connecté** : retarde l'admission d'au plus `ADMISSION_SETTLE_SECS` (3 s). Sans cela, deux clients qui ne voient pas le même ensemble (A voit {A,B}, B voit {A,B,C}) choisiraient des admis différents. L'attente est **bornée** : un fantôme à moitié connecté qui ne finit jamais (D109) ne doit pas empêcher deux pairs connectés de jouer (essai `GHOST=1`, §3). Avant ce délai, une première version attendait sans borne : elle aurait remplacé « attente infinie avec des pairs en trop » par « paire jamais démarrée avec un fantôme », régression par rapport à `main`.

Limite assumée (pas de protocole d'engagement entre pairs) : la décision porte sur les pairs connectés au moment où elle est prise. Un pair qui arrive après le démarrage des deux autres voit trois pairs et se classe parmi les admis si son identifiant est bas (2 chances sur 3) ; sa session ne se synchronise jamais et `abandon_unsynchronized_session` le ramène au lobby au bout de `SYNC_TIMEOUT_SECS` (30 s) ou de `session_peer_left`. Observé une fois (essai 1 du premier jet, §2 point 3), pas mesuré en fréquence. Allumette (HTTP, jeton) et web : même code, **non essayés** (pas de serveur allumette ici ; le web n'a pas de signaling observé, donc pas de point 4).

## 3. Preuves

`scripts/p2p-admission.sh` (nouveau) : trois clients headless lancés ensemble (`--number-player 2`, même salle), partie de 1 500 frames, `ALACOD_CONNECT_TIMEOUT_SECS=3` pour l'ignoré.
Vert si exactement un client est ignoré et revenu au lobby, les deux autres sortent en code 0, démarrent une partie, sans `DesyncDetected`, aux traces identiques ; un essai où un client a vu un autre ensemble est « course perdue » et se rejoue.

| Essai | Résultat |
|---|---|
| Premier jet (sans `PreviousSessionPeers`, sans délai) | 1 essai perdu sur 2 : l'ignoré, revenu au lobby, est admis à une session sans pair (§2 point 3) |
| Après correction, 6 essais | 6 verts, 0 perdu ; ignorés : clients 0, 2, 0, 0, 0, 1 ; traces des admis identiques (1 499 lignes), 0 `DesyncDetected` |
| Avec `ADMISSION_SETTLE_SECS`, 5 essais | 5 verts, 0 perdu |
| Build fusionné avec `5823c032`, 6 essais | 6 verts, 0 perdu (17 verts sur 17 au total hors premier jet) |
| `GHOST=1` : deux clients et un fantôme à moitié connecté (`p2p-ghost-peer.mjs`) | les deux jouent 1 499 frames aux traces identiques, aucun `lobby_connect_timeout` (le fantôme ne les bloque pas) |

Journal de l'ignoré (essai 6 du premier lot de six) :
```
ggrs{f=0 lobby_admission peers=3 max=2 excess=1 local=ignored} salle pleine : ce client est ignoré
ggrs{f=0 lobby_room_full peers=3 max=2 action=lobby waited_secs=3.0} salle pleine, ce client est resté ignoré : retour au lobby
```
Journal d'un admis : `ggrs{f=0 lobby_admission peers=3 max=2 excess=1 local=admitted} pairs en trop ignorés, la partie démarre avec les premiers par identifiant`.

Tests (`jjrs/p2p_tests.rs`) : `admission_keeps_the_first_players_by_peer_id`, `admission_is_identical_for_every_client_seeing_the_same_peers` (trois vues, mêmes admis, un seul ignoré), `lobby_status_line_reports_slow_peers_and_abandoned_attempts` (ligne « Room full » ajoutée).

## 4. Réserve de validation de D109 : le second délai

`p2p-connect-timeout.sh` ne pouvait exercer que `ATTEMPTS=1` : après une nouvelle tentative, un client suspendu (SIGSTOP) ne se fait plus annoncer par le signaling (il n'est connu que par l'offre qu'il envoie). Nouveau `GHOST=1` : `scripts/p2p-ghost-peer.mjs` (Node, WebSocket) rejoint la salle et envoie un candidat ICE à **chaque** pair qui arrive, sans jamais répondre (une offre invalide ferait paniquer le socket matchbox, `native.rs:258` : constaté). Résultat (délai 4 s) :
```
ggrs{f=0 lobby_connect_timeout attempt=1/2 action=retry waited_secs=4.0 pending=1 connected=1} … nouvelle tentative
ggrs{f=0 lobby_connect_timeout attempt=2/2 action=lobby waited_secs=4.0 pending=1 connected=1} … retour au lobby
```
Deux délais complets, le second sur le socket de la nouvelle tentative, puis retour au lobby. `ATTEMPTS=1` et `ATTEMPTS=2` (nominal) restent verts (traces identiques, 299 lignes).

## 5. Vérifications

| Commande | Résultat |
|---|---|
| `cargo fmt --all -- --check` | sortie 0 |
| `make check_sim_boundary` | OK : 13 crates, 2 en dette D69 |
| `make test_scenarios_par SHARDS=3` (sans BLESS, sur la fusion avec `5823c032`) | 802 scénarios, 3 tranches ok, 569 s ; `git status` propre |
| `make test_crates` | tous les `test result` ok, 0 échec |
| `make lint` ×5 | zombies, testbed (115 avertissements), throne, gungeon (1), isaac : 0 erreur |
| `scripts/check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` | verts après correction (ci-dessous) |
| p2p : `p2p-lobby.sh` (3 essais), `p2p-restart.sh`, `p2p-disconnect.sh`, `p2p-sync-timeout.sh`, `p2p-stress.sh` zombies (1 653 / 0 frames resimulées) et isaac (1 642 / 1) | verts, traces identiques sur 2 999 lignes |

Deux corrections hors périmètre, trouvées par ces portes (héritées de D109, que j'avais livrée) :
- `check-forbidden.sh` échouait sur `main` : six `Instant::now()` du lobby (`p2p.rs`, `signal_watch.rs`, `ui/lobby.rs`) n'étaient pas dans `determinism-allowlist.json` ; exceptions ajoutées (horloge murale du lobby, présentation du réseau).
- `p2p-sync-timeout.sh` échouait sur `main` : depuis D109 le client repris abandonne par `session_peer_left` (le client 0 a déjà quitté la salle), pas par son propre `session_sync_timeout` ; le script accepte l'un ou l'autre pour le client 1 et exige `session_sync_timeout` du client 0.

## 6. Limites et dettes

- Arrivée tardive (§2) : pas d'engagement entre pairs ; repli par l'abandon de session. Fermer ce cas demande un message de contrôle (second canal matchbox) : hors périmètre.
- Allumette et web non essayés.
- Un ignoré qui revient au lobby retombe en boucle tant que la salle reste pleine (un cycle par délai) ; la ligne d'état le dit, rien d'autre.
