# m1-restart-p2p — « Rejouer » en ligne relance une partie avec les mêmes pairs (D14)

Lire d'abord `docs/taches/README.md` (agent **local**, `origin/main` mergé). Tâche moyenne (3 j),
touche la **session** (pas la simulation) : la recette p2p du README §4 est obligatoire, étendue
(point 5). Dette D14, reportée de M0 (« relancer une partie en ligne demande de recréer la session
GGRS entre les pairs »).

## Contexte

`RunRequest::Restart` (`run_state.rs`) rejoue la même configuration en local sans repasser par le
lobby ; en ligne (`OnlineState::Online`) il est **redirigé vers `LobbyOnline`** (`plan_run_request`,
`restart_redirected`). `start_matchbox_socket` (`jjrs/p2p.rs`) ouvre `{matchbox_url}/{lobby}` (ou
l'URL allumette avec JWT), `wait_for_players` construit la session GGRS quand le lobby est plein.
L'écran de fin (`ui/game_over.rs`) a un bouton « Rejouer (R) » et un bouton « Lobby ».

## Décisions (proposées ; l'agent confirme ou amende en dix lignes avant de coder)

1. **Accord sans message réseau** : les deux clients ont vu la même fin de partie (déterminisme). Le
   compteur de parties jouées dans le processus `games_played` (ressource hors rollback) est identique
   des deux côtés. « Rejouer » en ligne : quitter la session GGRS, rouvrir le socket matchbox sur la
   salle **`{lobby}-r{games_played}`**, attendre les pairs comme au premier lancement, nouvelle session
   GGRS, graine `run_seed ^ fnv1a("restart") ^ games_played`, même carte et mêmes réglages.
2. **Désaccord** : si l'autre joueur a choisi « Lobby » (ou n'arrive pas), le joueur qui attend voit
   « En attente des autres joueurs… » avec un délai (`camera.ron`/`ui` : 30 s) puis retourne à
   `LobbyOnline`. Aucun état de jeu n'est partagé entre les deux parties (les joueurs repartent à neuf,
   comme un restart local).
3. **Chemin allumette** (`--allumette`) : v1 = même mécanique avec une **nouvelle salle** créée par
   l'API (`start_allumette_flow` rejoué avec un nom dérivé) si c'est simple ; sinon « Rejouer » reste
   redirigé vers le lobby en allumette et c'est écrit (dette). Le chemin `--matchbox` est l'objectif.
4. **Nettoyage** : réutiliser `cleanup_rollback_world_system` (OnExit InGame) ; vérifier que `RngStreams`,
   `GgrsNetIdFactory`, `FlowFieldCache`, `Clock`, `FloorState`, `Gauges` repartent à zéro — un test
   d'intégration local « restart deux fois, traces des deux parties identiques à deux parties
   fraîches » (`ALACOD_RESTART_AT_FRAME`, point 5).
5. **Preuve p2p** : variable de test `ALACOD_RESTART_AT_FRAME=600` (hors rollback, comme
   `ALACOD_EXIT_AT_FRAME`) : chaque client pose `RunRequest::Restart` à cette frame ; la recette du
   README §4 étendue joue deux parties (600 + 600 frames, `ALACOD_EXIT_AT_FRAME` relatif à la partie
   courante) et compare les **deux** traces de chaque client (`ALACOD_STATE_TRACE` suffixé `-g1`, `-g2`) :
   identiques entre clients, et la partie 2 identique à une partie fraîche de même graine jouée en
   local (`scenario` ou synctest). Script `scripts/p2p-restart.sh` (bash) ajouté au README §4.
6. **Hors périmètre** : spectateurs, changement de carte au restart, vote en jeu, écran d'attente
   élaboré.

## Règles

Deux compilations au plus ; purge + point d'état ; aucune trace de scénario ne change (la simulation
n'est pas touchée) ; `docs/conventions.md` : §33 « Restart en ligne » ; `docs/taches/README.md` §4 :
la recette étendue ; `dettes.md`/`taches.md` : ne pas toucher.

## Livrer

Rapport `docs/taches/rapports/m1-restart-p2p.md` (sorties réelles de `scripts/p2p-restart.sh`) ; `git push
-u origin m1-restart-p2p` ; `SendMessage` à `orch` : `LIVRÉ m1-restart-p2p <sha> : <une ligne>`.
