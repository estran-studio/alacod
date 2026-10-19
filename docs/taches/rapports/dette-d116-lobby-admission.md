# dette-d116-lobby-admission — politique d'admission du lobby en ligne (D116)

Worker b0, 2026-10-18, branche `dette-d116-lobby-admission` (depuis `origin/main` `2b15376a`, fusionnée deux fois avec `origin/main` (dernière fusion : `8ca55686`) avant les vérifications finales). Mac M1 Pro partagé
(charge 9 à 46 pendant les essais : b1 et b3). Signaling `matchbox_server` d'OrbStack (`b0-signaling`, port 3570, déjà en ligne : les scripts l'utilisent par `MATCHBOX_URL`).

## 1. Résultat

| Point demandé | État |
|---|---|
| Politique d'admission déterministe, identique chez tous les clients | **faite** : `jjrs::p2p::admit_players` garde les `max_player` premiers joueurs du socket (identifiants triés, ordre commun), les autres sont ignorés et prévenus (§2) |
| Les admis jouent, traces identiques | **prouvé** : `scripts/p2p-admission.sh`, trois clients dans une salle à deux places, 23 essais verts, dont 1 course perdue sur 24 depuis la correction du premier jet (§3) |
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
| Build fusionné avec `5823c032`, 6 essais | 6 verts, 0 perdu |
| Après la relecture (§7), 7 essais | 6 verts, **1 course perdue** (essai 6) : le client 2 a démarré avec le client 1 avant que le signaling lui annonce le troisième ; le client 1, qui voyait trois pairs, a choisi d'autres admis ; les deux sessions abandonnées par `session_peer_left` (limite du §2, pas de protocole d'engagement) ; total 23 verts, 1 course perdue sur 24 (hors premier jet) |
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
| `make test_scenarios_par SHARDS=3` (sans BLESS, sur la fusion avec `8ca55686`, après la relecture) | 802 scénarios, 3 tranches ok, 492 s (569 s sur la fusion précédente) ; `git status` propre |
| `make test_crates` (sur la même fusion) | tous les `test result` ok, 0 échec |
| `make lint` ×5 | zombies, testbed (115 avertissements), throne, gungeon (1), isaac : 0 erreur |
| `scripts/check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` | verts après correction (ci-dessous) |
| p2p : `p2p-lobby.sh` (3 essais), `p2p-restart.sh`, `p2p-disconnect.sh`, `p2p-sync-timeout.sh`, `p2p-stress.sh` zombies (1 653 / 0 frames resimulées) et isaac (1 642 / 1) | verts, traces identiques sur 2 999 lignes |

Deux corrections hors périmètre, trouvées par ces portes (héritées de D109, que j'avais livrée) :
- `check-forbidden.sh` échouait sur `main` : six `Instant::now()` du lobby (`p2p.rs`, `signal_watch.rs`, `ui/lobby.rs`) n'étaient pas dans `determinism-allowlist.json` ; exceptions ajoutées (horloge murale du lobby, présentation du réseau).
- `p2p-sync-timeout.sh` échouait sur `main` : depuis D109 le client repris abandonne par `session_peer_left` (le client 0 a déjà quitté la salle), pas par son propre `session_sync_timeout` ; le script accepte l'un ou l'autre pour le client 1 et exige `session_sync_timeout` du client 0.

## 6. Limites et dettes

- Arrivée tardive (§2) : pas d'engagement entre pairs ; repli par l'abandon de session. Fermer ce cas demande un message de contrôle (second canal matchbox) : hors périmètre.
- Allumette et web non essayés.
- Un ignoré qui revient au lobby ne reboucle pas : son socket neuf ne revoit pas les admis (`PreviousSessionPeers`), il attend dans le lobby (« Waiting for players », avis « Room full … back to the lobby » 20 s) qu'un pair neuf arrive ; si la salle se remplit encore de pairs neufs, il est de nouveau ignoré et un nouveau délai court.

## 7. Relecture indépendante

Relecteur (agent `reviewer`, lecture seule, sur `b992df81`) : aucun bloquant ; quatre défauts mineurs, tous corrigés :
1. `settling` portait sur le pair en attente le plus **ancien** : un fantôme à moitié connecté resté plus de 3 s annulait l'attente d'un pair arrivé après lui. Corrigé : `PeerSightings::youngest_pending` (le plus récent), test ajouté.
2. `room_full` n'était pas remis à zéro quand un client ignoré se retrouvait seul (les admis partis) : le compte à rebours de la salle suivante aurait repris à l'ancienne date et banni des pairs libres. Corrigé : remis à zéro dans la branche « moins de joueurs que de places ».
3. `p2p-admission.sh` classait « course perdue » toute session perdue, y compris celle du client ignoré revenu au lobby (la signature du bug du premier jet : faux vert possible). Corrigé : un client ignoré qui démarre une partie ou perd une session après `lobby_room_full` fait **échouer** le script ; la course perdue ne regarde que les admis.
4. `--players "$(p2p_players 2)"` passé en un seul argument dans `p2p-connect-timeout.sh GHOST=1` : corrigé.
Remarque du relecteur, non corrigée : sur le web, un intrus absent de `data-participants` classé parmi les admis bloque au `return` de `browser_mapping` sans délai (cas d'avant D116, non régressif) ; un filtre par le mapping avant `admit_players` le fermerait.

## 8. Seconde revue (OK avec réserves) : correctifs

Rien dans `GgrsSchedule`, aucune trace ne bouge, une session incompatible donne un blocage borné et jamais un desync. Réserves corrigées :
1. **P2 — excédent connecté d'emblée.** L'attente « Too many peers » de `main` couvrait le socket d'une partie finissante (§4.5 de `dettes-d105-d109.md`) : encore connecté, pas encore dans `PreviousSessionPeers`, il était admis d'emblée s'il se classait parmi les `max_player` premiers, et `ADMISSION_SETTLE_SECS` ne le couvrait pas (il est connecté, donc pas « en attente »). Maintenant tout excédent (`players.len() > max_player`) est attendu `ADMISSION_SETTLE_SECS` : `LobbyConnectWatch::excess_since` (`Option<Instant>`), posé à la première frame d'excédent, remis à zéro dans `begin()` et dès qu'il n'y a plus d'excédent ; `settling |= excess_since.elapsed() < ADMISSION_SETTLE_SECS`.
2. **P3 — délai de D109 pendant l'attente d'admission.** Il ne s'applique plus que si `players.len() < num_players` (`pending.filter(|_| players.len() < num_players)…`).
3. **P3 — `admission_logged`** remis à `None` quand `players.len() < num_players`, avec `room_full`.
4. **P3 — `p2p-admission.sh`** échoue maintenant quand plus d'un essai sur quatre est perdu par course (`lost * 4 > attempt`) ; `RACE_RETRIES` (6) reste un arrêt de sécurité.
5. **P3 — `GHOST=1`** : un fantôme qui n'a pas visé les deux clients (moins de deux « candidat ICE sans suite » dans `ghost.log`) est un **échec**. (Le critère « journal du client : pair vu par le signaling » aurait été faux : l'autre client réel est aussi un pair vu.)
6. **Doc** : §6 corrigé (l'ignoré ne reboucle pas), raison de l'exception `Instant` de `t0` (`signal_watch.rs`, `#[cfg(test)]`) corrigée dans l'allowlist (et celle de `saw`), D116 **rouverte sur le seul point de l'arrivée tardive**.

Vérifications (build `zombies` fait avant, sur le code corrigé) :

| Script | Résultat |
|---|---|
| `cargo test -p game` | 264 passés, 0 échec ; `cargo fmt --check` OK |
| `p2p-admission.sh` `TRIALS=12` | **12 verts, 2 courses perdues sur 14** (≤ 1 sur 4), traces des deux admis identiques (1 499 lignes), 0 `DesyncDetected` |
| `p2p-admission.sh` `GHOST=1` ×3 | 3 / 3 OK, traces identiques (1 499 lignes), fantôme visant les deux clients |
| `p2p-connect-timeout.sh` `ATTEMPTS=2` ×6 (le cas du P2, dépendant du timing) | 6 / 6 OK (traces identiques, 299 lignes) ; **1 course perdue** du script (« le client 1 a connecté son pair avant d'être suspendu », rejouée par le script lui-même) sur 7 lancements |
| `ATTEMPTS=1` ×2 | 2 / 2 OK |
| `GHOST=1` ×2 | 2 / 2 : `attempt=1/2 action=retry` puis `attempt=2/2 action=lobby` |
| `p2p-lobby.sh` (3), `p2p-sync-timeout.sh`, `p2p-restart.sh` | verts |

Non couvert : le P2 n'a pas de test dédié (le socket d'une partie finissante qui n'a pas encore quitté la salle n'est pas reproduit isolément ; `p2p-lobby.sh` le traverse : « pairs obsolètes ignorés : 1 » par tentative, 0 abandon de session).
