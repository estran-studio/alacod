# dettes-d105-d109 — `--seed` partout (D105), attente bornée d'un pair vu mais jamais connecté (D109)

Worker b0, 2026-10-18, branche `dettes-d105-d109` (depuis `origin/main` `02bf46b6`, fusionnée avec `8e25731d` avant les vérifications finales). Mac M1 Pro partagé
(charge 2 à 56 selon l'heure : b1 et b3), sans affichage utilisable pour le lobby. Réseau dégradé : conteneur Linux arm64 OrbStack (`tc netem` sur `lo`),
car macOS n'a ni `netem` ni `unshare` : voir `scripts/p2p-linux-box.sh` (§3).

## 1. Résultat

| Point demandé | État |
|---|---|
| D105 : un seul consommateur de `--seed`, copies de `gungeon`/`isaac` supprimées | **fait et fermé** : `map_ldtk::game::local::apply_launch_seed`, installé par `LdtkLocalGamePlugin` (`Startup`) |
| D105 : `zombies::choose_local_seed` unifié ? | **non, réduit à ce qui lui est propre** : le tirage aléatoire sans `--seed` (les autres jeux gardent `entry.default_seed`) et le titre de fenêtre ; l'application de `--seed` passe par le système commun (§2) |
| D105 : deux `--seed` → deux frames 0 ; même graine → même frame 0, pour chaque jeu | **fait** : `scripts/check-launch-seed.sh`, 5 jeux, §2 |
| D105 : aucune trace ne bouge | **vérifié** : `make test_scenarios_par SHARDS=3` sans BLESS (§5) ; le runner n'installe pas `BaseArgsPlugin`, `LaunchSeed` n'y existe pas |
| D109 : fréquence mesurée | **faite** : §3 |
| D109 : attente bornée, nouvelle tentative puis retour au lobby, horloge murale | **fait** (`jjrs::p2p`, `jjrs::signal_watch`), délai 15 s ; journal `ggrs{f=0 lobby_connect_timeout …}` ; écran du lobby (§4) |
| D109 : script de preuve | `scripts/p2p-connect-timeout.sh` (deux modes), §4 |
| Chemin nominal inchangé : `p2p-lobby.sh`, `p2p-restart.sh` verts | **vérifié** (§5) |

## 2. D105

Cause : `LaunchSeed` n'était lu que par `zombies`, `gungeon` et `isaac` (deux copies de `apply_launch_seed`) ; `throne` et `testbed` l'ignoraient.
Correctif : `apply_launch_seed(Option<Res<LaunchSeed>>, ResMut<LdtkGameMap>)` dans `crates/map_ldtk/src/game/local.rs`, ajouté en `Startup` par `LdtkLocalGamePlugin`
(les cinq jeux l'installent) ; copies de `games/gungeon` et `games/isaac` supprimées (coupure franche). `Option<Res<…>>` : le runner de scénarios (`crates/scenario`) n'installe pas
`BaseArgsPlugin`, donc aucune graine ne s'impose aux scénarios.

`zombies` : la sémantique n'est **pas** la même. `choose_local_seed` (`OnEnter(LobbyLocal)`, hors ligne) **tire** une graine aléatoire quand `--seed` manque (une partie locale neuve
est différente à chaque fois), alors que les autres jeux gardent `entry.default_seed`. Il ne garde donc que le tirage (si `LaunchSeed` est vide) et le titre de la fenêtre ;
l'application de `--seed` est celle du système commun. Effet de bord voulu : `zombies --seed N` en **ligne** est maintenant honoré (avant : ignoré hors ligne locale).

Preuve (`scripts/check-launch-seed.sh`, trois parties locales headless par jeu : graine 1, graine 2, graine 1 ; ligne « frame 0 : hash, entités » de la trace d'état) :

| Jeu | graine 1 | graine 2 | graine 1 (bis) | verdict |
|---|---|---|---|---|
| zombies | `…24af3f506cc63a97` (62) | `…e3c9ed438d79aa50` (69) | identique à la 1re | OK |
| gungeon | `…1d5986a3cf176ce4` (193) | `…3378c71b9e9ad92e` (195) | identique | OK |
| isaac | `…1500857041fea91a` (179) | `…445d96c77f9d661c` (165) | identique | OK |
| throne | `…a151c38d4aea1489` (81) | `…037f9d34856ae847` (73) | identique | OK |
| testbed | `…6db687967a4c4208` (35) | `…443cafce2f04bfeb` (35) | identique | OK |

**Avant** (système commun désactivé, `throne` et `testbed` recompilés) : `throne` 85 entités, hash `…a606449a3dc9c5b6` pour les graines 1, 2 et 1 ; `testbed` `…310c9267a8f3f1b3`
pour les trois : « ÉCHEC (graines 1 et 2 : même frame 0) », la graine du manifeste est toujours jouée. (Piège relevé : restaurer le fichier par `mv` lui rend son ancienne date de
modification, cargo croit le binaire à jour ; un `touch` est nécessaire avant de recompiler.)

## 3. D109 : fréquence mesurée

Outil : `scripts/p2p-connect-starts.sh` (N démarrages de deux clients dans une salle neuve, signaling `matchbox_server` local, temps jusqu'à « All 2 players are connected »
chez les deux, résolution 0,1 s ; avec `PLAY_FRAMES` le démarrage est joué jusqu'à la frame n et les traces comparées) lancé dans un conteneur Linux par `scripts/p2p-linux-box.sh`
(copie de l'arbre de travail, `cargo build --profile headless`, `tc qdisc add dev lo root netem …`, `matchbox_server` Linux arm64). `gungeon`. `netem` sur `lo` s'applique à chaque paquet de chaque
sens, signaling compris (même modèle que `p2p-d87.sh`). Charge du conteneur au lancement 1,3 à 2,8.

**Binaire d'avant** (`02bf46b6`, sans D109) :

| Perte (délai par sens) | Démarrages | Bloqués (> 75 s) | Connectés | Temps de connexion (max des deux clients) |
|---|---|---|---|---|
| 1 % (60 ms ± 10) | 30 | **0** | 30 | médiane 1,8 s, p90 2,6 s, max 4,7 s |
| 3 % (100 ms ± 10) | 30 + 70 = **100** | **2** (2 %) | 98 | médiane 3,3 s, p90 4,5 s ; **un** démarrage à 35,6 s (cf. ci-dessous), les autres ≤ 6,7 s |

Les 24 démarrages à 3 % du rapport `m2-p2p-deux-machines` (1 bloqué) sont cohérents. Les deux démarrages bloqués : les deux clients restent à « Waiting for players: 1/2 connected »
(24 000 lignes), le signaling a vu les deux (`NewPeer`). Le démarrage à 35,6 s est une connexion **à moitié ouverte** : un seul des deux côtés voit son canal s'ouvrir ; ce client
construit une session sur un pair qui ne répond pas, la session `Synchronizing` expire après 30 s (D86), il revient au lobby et rejoint l'autre, resté en attente.

**Binaire de D109** (délai 15 s, une nouvelle tentative) :

| Perte | Démarrages | Nouvelle tentative déclenchée | Bloqués | Notes |
|---|---|---|---|---|
| 1 % | 30 | 0 | 0 | max 3,6 s |
| 3 % | 30 + 100 + 100 (sans le correctif du §4.2) | 8 (3,5 %) | 0 | les démarrages sans tentative : max 8,0 s |
| 3 % | **150** (binaire final, chaque démarrage joué jusqu'à la frame 120) | **7 (4,7 %)** | **0** | **150 parties jouées jusqu'au bout, traces des deux clients identiques, 0 perdue** ; connexions retentées à 18,4–20,1 s (15 s + ~3 s) |

Le 1 % n'a jamais déclenché de nouvelle tentative (60 démarrages sur les deux binaires, max 4,7 s) : le délai de 15 s est près de 2× le pire temps de connexion sans anomalie mesuré (p90 5,3 s à 3 %, jusqu'à 8,0 s
sans anomalie) et ne touche pas le chemin nominal. Fréquence de l'anomalie à 3 % : **2 à 5 %** des démarrages (2/100 bloqués + 1/100 lent sur le binaire d'avant ;
15/380 retentés avec D109 sur 380 démarrages à 3 %), **0 sur 60 à 1 %**.

Limite : une machine, un conteneur ; la mesure ne dit rien d'un vrai WAN, d'un NAT ou du `matchbox_server` public. Le 1 % est trop rare pour trancher entre 0 et ~3 % (0/60 : borne haute à 95 % ≈ 5 %).

## 4. D109 : correctif

### 4.1 Observer le signaling (`jjrs/signal_watch.rs`)

Le socket matchbox n'expose que les pairs **connectés** (`MatchboxSocket::players`) ; un pair annoncé par le serveur dont la négociation ne finit jamais est invisible. Ce n'est pas observable
par l'API publique, et le constructeur du signaller WebSocket natif est privé. `ObservedSignallerBuilder` (natif) réimplémente le signaller de matchbox (même protocole, mêmes reconnexions)
et note dans un `PeerSightings` partagé (`Arc<Mutex<…>>`) chaque `NewPeer` / premier `Signal` reçu (instant mural) ; `PeerLeft` le retire. Trois dépendances directes de plus pour `game`
(`async-tungstenite`, `futures-util`, `futures-timer`, mêmes versions et features que `matchbox_socket` : aucun crate de plus dans `Cargo.lock`). Sur le web, rien n'observe (le lobby y garde son comportement
d'avant : `cargo check -p game --target wasm32-unknown-unknown` passe).

Piège de protocole (matchbox) : le serveur n'envoie `NewPeer` qu'aux pairs **déjà présents** (qui deviennent offreurs) ; un nouveau venu n'apprend l'existence d'un pair que par son offre. Conséquence : après une
nouvelle tentative, un pair **gelé** (suspendu) n'est plus vu ; un pair vivant l'est.

### 4.2 Lobby (`jjrs/p2p.rs`, `ui/lobby.rs`)

- `LobbyConnectWatch` : sightings du socket courant, essai courant, délai. `wait_for_players` calcule, quand il manque des joueurs, l'attente du pair vu le plus ancien **ni connecté ni d'une session passée**
  (`PreviousSessionPeers`, D86). Elle atteint `CONNECT_TIMEOUT_SECS` = **15 s** (horloge murale `Instant`, comme `SYNC_TIMEOUT_SECS` ; `ALACOD_CONNECT_TIMEOUT_SECS` pour les essais) :
  1. essai 1 sur `CONNECT_ATTEMPTS` = 2 : **nouvelle tentative**, un socket neuf sur la même salle ; journal `ggrs{f=0 lobby_connect_timeout attempt=1/2 action=retry waited_secs=15.0 pending=1 connected=1}` ;
  2. essai 2 : **retour au lobby** : `LobbyOnline` ré-entré (socket et flux allumette repris de zéro, `OnlineRestart` retiré), journal `… attempt=2/2 action=lobby …`.
  En allumette (URL à jeton) et sur le web, pas de réouverture : directement le retour au lobby. `ALACOD_CONNECT_ATTEMPTS=1` (essai) : le premier délai ramène au lobby.
- Écran du lobby : ligne d'état sous la liste des joueurs : « Connecting to peer... N s » dès 3 s d'attente d'un pair vu, « Peer connection failed: new attempt (2/2) » puis
  « Peer connection failed after N attempt(s): back to the lobby » (20 s). Texte testé par `lobby_status_line_reports_slow_peers_and_abandoned_attempts` ; **non vérifié à l'écran** (aucun lobby en ligne rendu
  sur ce poste ; le texte est fabriqué par la fonction pure testée et posé par un système de dix lignes).
- Journal des pairs : « pair(s) vu(s) par le signaling, connexion en cours (D109) » (une fois par socket) et « peer … connected (vu par le signaling depuis x s) ».

### 4.3 Défaut trouvé à la mesure, corrigé : la connexion à moitié ouverte

Première version (nouvelle tentative seule), 59 démarrages joués à 3 % : 6 retentés, dont **1 perdu** (démarrage 38, ≥ 4 cycles de 33 s sans jamais jouer, tué à 240 s). Mécanisme : un client voit sa connexion s'ouvrir (3 s) et construit sa
session sur le pair ; l'autre, resté bloqué, rouvre un socket à 15 s : l'ancien pair disparaît de la salle, le premier reste en `Synchronizing` 30 s (D86) alors que le second se synchronise sur le **socket de partie** du premier,
etc. : les deux cycles ont la même période (32,5 s) et un décalage constant de 15 s, ils ne se rejoignent jamais. Avant D109 cette connexion se rattrapait par le délai D86 (35,6 s observé) : la nouvelle tentative
l'avait **rendue pire**.
Correctif : `abandon_unsynchronized_session` abandonne aussi la session **dès qu'un pair de la session en cours quitte la salle** (`CurrentSessionPeers`, `session_peer_left` ; `PeerState::Disconnected` relevé par
`try_update_peers` pendant `Synchronizing` seulement) : son identifiant ne reviendra pas, attendre 30 s ne sert à rien. Journal `ggrs{f=… session_peer_left waited_secs=13.0 players=2}`. Mesuré : 5 des 7
démarrages retentés du lot final étaient des connexions à moitié ouvertes, 5 `session_peer_left`, retour 200 ms après la nouvelle tentative de l'autre, 7/7 parties jouées jusqu'au bout.
Test : `only_a_departing_current_session_peer_abandons_the_session`.

### 4.4 Preuve de bout en bout (`scripts/p2p-connect-timeout.sh [jeu]`, zombies)

Deux clients, le client 1 **suspendu** (SIGSTOP) dès qu'il a vu son pair par le signaling : connexion annoncée, négociation inachevée, c'est la situation de D109. `ALACOD_CONNECT_TIMEOUT_SECS=4`. Le client 0 doit
journaliser l'abandon ; le client 1 est repris ; les deux jouent jusqu'à la frame 299 : traces identiques, 0 `DesyncDetected`.

| Mode | Journal du client 0 | Résultat |
|---|---|---|
| `ATTEMPTS=2` (défaut du jeu) | `ggrs{f=0 lobby_connect_timeout attempt=1/2 action=retry waited_secs=4.0 pending=1 connected=1}` | « p2p-connect-timeout : OK », 299 lignes identiques |
| `ATTEMPTS=1` | `ggrs{f=0 lobby_connect_timeout attempt=1/1 action=lobby waited_secs=4.0 pending=1 connected=1}`, puis un second « start p2p connection » 1 ms plus tard (lobby ré-entré) | « p2p-connect-timeout : OK », 299 lignes identiques |

Le second délai du mode par défaut (`attempt=2/2 action=lobby`) n'est pas exercé de bout en bout : avec un pair **gelé** il ne se produit pas (cf. §4.1, le nouveau venu ne revoit pas le pair gelé) ; `ATTEMPTS=1` exerce la même branche de code
(retour au lobby), et `connect_timeout_action(2, 2, true) == ToLobby` est testé.

### 4.5 Après la fusion avec `5fa55fcb` : un pair de trop, et la panique de matchbox

`origin/main` a avancé deux fois pendant la tâche (`8e25731d`, puis `5fa55fcb` : D69 E1/E2, 80 fichiers ; fusions sans conflit). Sur la seconde, un `p2p-connect-timeout.sh` a révélé un **crash** : après le
retour au lobby, le client 1 voyait **deux** pairs vivants (le socket de partie d'avant et le neuf de l'autre client : l'ancien n'avait pas encore quitté la salle, D86 ne marque que les pairs dont une
session a été construite), `players.len() >= max_player` passait et `system_after_map_loaded` paniquait (`failed to add player`, handle hors `num_players`). Défaut **préexistant** (la condition était `<`),
que les sockets supplémentaires de D109 rendent plus probable. Correctif : `wait_for_players` **attend** (`Too many peers: 3/2 connected, waiting for a stale socket to leave`) tant qu'il y a plus de pairs
que `max_player` ; le chemin nominal (égalité) ne change pas.
Aussi dans les journaux (`thread 'async-compat/tokio-1' panicked … native.rs:545 … TrySendError { kind: Disconnected }`) : une panique de **matchbox** (callback `on_open` d'un canal dont le socket vient d'être détruit,
sa tâche tokio) à chaque socket fermé en pleine négociation ; elle ne tue que cette tâche, le processus continue (les parties se jouent ensuite, traces identiques). Third-party (dépôt `AdamWhitehurst/matchbox`),
non corrigée ici.

## 5. Vérifications

La branche a été vérifiée sur la fusion avec `8e25731d` puis, `main` ayant encore avancé, sur la fusion avec `5fa55fcb` ; les lignes ci-dessous sont celles de la **fusion finale** sauf mention.

| Vérification | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` sans BLESS | **668 scénarios verts**, 3 tranches, 290 s (fusion `5fa55fcb`) ; aucune `.trace` modifiée (`git status` propre hors documentation). Non rejoué après le dernier correctif (§4.5, lobby en ligne seulement, aucun chemin de scénario) |
| `make test_crates` | code 0 ; **1 275 tests passés, 0 échec**, rejoué après le dernier correctif (dont `connect_timeout_retries_once_then_returns_to_lobby`, `lobby_status_line_reports_slow_peers_and_abandoned_attempts`, `only_a_departing_current_session_peer_abandons_the_session`, `pending_is_the_oldest_peer_neither_connected_nor_ignored`) |
| `make lint` (×5 : zombies, testbed, throne, gungeon, isaac) | 5 « aucune erreur » |
| `cargo fmt --all -- --check` | propre |
| `cargo check -p game --target wasm32-unknown-unknown` | passe (avant la fusion `5fa55fcb`) |
| `scripts/check-launch-seed.sh` | OK (§2), rejoué sur la fusion finale |
| `scripts/p2p-stress.sh gungeon` (graine 10, bots figés, 4 500 frames, `STRESS=1`, `EXPECT_FLOORS=1`) | OK : **10 899 frames resimulées** (dernier passage ; 4 947 / 31 sur la fusion précédente), 0 `DesyncDetected`, 4 499 frames identiques, passage d'étage resimulé |
| `scripts/p2p-lobby.sh zombies` | 5/5 verts dans la bande 100–400 ms (dernier passage) |
| `scripts/p2p-restart.sh zombies` | OK (dernier passage) |
| `scripts/p2p-connect-timeout.sh zombies` | **10/10** OK sur le script final (5 × `ATTEMPTS=2`, 5 × `ATTEMPTS=1`) ; il rejoue de lui-même un essai quand la course de la suspension est perdue (6 essais sur 16 : le client 1 avait fini sa négociation, ou le client 0 sa moitié, avant le SIGSTOP) : un essai perdu n'est pas un essai de D109 |
| Démarrages nominaux avec D109 | 1 % : 30/30 sans tentative superflue ; 3 % : 150 joués (§3) |

## 6. Limites

- Pas de vérification à l'écran de la ligne d'état du lobby (aucun rendu d'un lobby en ligne sur ce poste) ; texte testé en unité.
- Une machine, un noyau Linux de VM : la fréquence mesurée (2 à 5 % à 3 % de perte) n'est pas celle d'un WAN. 15 s n'a pas été comparé à d'autres délais (le temps normal tient en 7 s au pire, la mesure ne distingue pas 10 s de 15 s).
- La nouvelle tentative rompt aussi les connexions déjà ouvertes d'une partie à 3 joueurs ou plus (un socket neuf) : non mesuré à N > 2.
- Web et allumette : pas d'observation des pairs (web), retour direct au lobby (allumette) ; non essayés.
- Le lot de 150 démarrages (§3) a tourné **avant** la correction du §4.5 (l'attente d'un pair de trop) ; il n'a pas été rejoué.
- `p2p-connect-timeout.sh` fabrique le pair bloqué par une suspension (le serveur voit le pair, la négociation n'avance pas) ; la perte réelle de paquets est exercée par `p2p-connect-starts.sh`, sans scénario déterministe.

## 7. Outils ajoutés

- `scripts/check-launch-seed.sh` (D105).
- `scripts/p2p-connect-starts.sh` (mesure, `PLAY_FRAMES` pour jouer chaque démarrage), `scripts/p2p-linux-box.sh` (conteneur Linux avec `NET_ADMIN` pour les scripts p2p `NETEM` depuis un Mac : copie du dépôt, compilation, `tc` sur `lo`, `matchbox_server` Linux),
  `scripts/p2p-connect-timeout.sh` (preuve de D109).
- Variables d'essai : `ALACOD_CONNECT_TIMEOUT_SECS`, `ALACOD_CONNECT_ATTEMPTS`.
