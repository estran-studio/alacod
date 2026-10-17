# dette-d86-lobby-en-ligne — retour au lobby en ligne, mesure des rollbacks, nightly complet

Worker b2, 2026-10-17, branche `dette-d86-lobby-en-ligne` (depuis `origin/main` `25c0e49b`). Une machine : VPS Debian 8 cœurs, Docker
(signaling `matchbox_server` du compose), sans affichage, **partagée** : un autre worker compilait en permanence (charge 5 à 12 selon les
essais ; la charge au lancement de chaque essai est dans les journaux des scripts et rappelée ci-dessous quand elle compte).

## 1. Résultat

| Point demandé | État |
|---|---|
| 1. D86 : (a) ignorer les pairs de la session finie, (b) abandon d'une session non synchronisée, repli du restart gardé | **fait et fermé** (D86, `dettes.md`) |
| 1. Test : requêtes à 100–400 ms d'écart, 5 essais consécutifs verts, traces identiques parties 1 et 2 | **fait** : 5/5 sur zombies et 5/5 sur gungeon après la victoire ; avant le correctif : rouge au 1er essai |
| 1. Rejouer `p2p-restart.sh` | **fait** (restart ordinaire, zombies : OK) |
| 1. Aucune trace de scénario ne bouge | **vérifié** (501 scénarios, `git status` propre) |
| 2. Mesure des rollbacks (profondeur, fréquence) | **faite** : §4 ; compteur ajouté au journal de fin |
| 3. Nightly complet `NIGHTLY_P2P_GUNGEON=1` | **lancé et terminé** : code 0, 18 924 s (5 h 15) ; étapes C et D non exécutées utilement (D89), §5 |
| 4. Suites, rapport, dettes, fiche, journal | §6 |

## 2. Cause et correctif

Cause (rapport `m2-p2p-run-complete` §7.4, **reconfirmée** : le binaire d'avant le correctif, rejoué par `p2p-lobby.sh`, échoue au 1er essai
avec des requêtes à 157 ms d'écart ; les deux journaux s'arrêtent à « Map loading complete, transitioning to GameStarting state »).
« Lobby » rouvre la salle d'origine ; l'ancien socket de l'autre pair y vit jusqu'à `OnExit(InGame)` ; `wait_for_players` accepte tout pair
présent : le premier revenu bâtit sa session sur un identifiant que l'autre ferme aussitôt.

Correctif (`crates/game/src/jjrs/p2p.rs`, `core.rs`, `ui/lobby.rs` ; **session seulement**, aucun état rollback, aucun format de trace) :

- **(a)** `PreviousSessionPeers` (ressource) : `system_after_map_loaded` y ajoute les identifiants matchbox des pairs de chaque session qu'il
  construit ; `live_players(socket.players(), &stale)` les retire de la liste (même ordre trié, donc mêmes handles GGRS chez tous les pairs).
  Utilisée par `wait_for_players` (compte et construction de `GgrsSessionBuilding`), `system_after_map_loaded` et l'interface du lobby.
  Le journal dit « peer … d'une session précédente ignoré (D86) » (une fois par pair, sans `ggrs{…}` : l'identifiant diffère d'un client à
  l'autre, la ligne ne se comparerait pas). Mémoriser à la **construction de la session** plutôt qu'à la sortie de partie couvre aussi un pair
  que le serveur liste encore alors que le canal de données est déjà fermé.
- **(b)** `abandon_unsynchronized_session` (`Update`, `InGame`, en ligne) : une session p2p restée `Synchronizing` plus de
  `SYNC_TIMEOUT_SECS` = **30 s** (horloge **murale** : `Instant`, surcharge d'essai `ALACOD_SYNC_TIMEOUT_SECS`) pose `RunRequest::ToLobby` et journalise
  `ggrs{f=0 session_sync_timeout waited_secs=4.0 players=2} session non synchronisée, abandon et retour au lobby`. Pas `Time<Real>` : la relecture a montré qu'en
  headless bevy l'avance de 1/60 s par update (`TimeUpdateStrategy::ManualDuration`), donc « 4 s » durait 1,3 s réelles (corrigé). 30 s : l'autre pair peut
  légitimement finir de charger sa carte (valeur du repli du restart ; non mesurée sur de vraies cartes lentes). La fermeture du socket et les
  pairs devenus obsolètes passent par le même chemin que (a). Repli du restart (`{lobby}` après `RESTART_TIMEOUT_SECS` = 30 s) **inchangé**.
- Outils d'essai : `ALACOD_RESTART_DELAY_MS` (`state_trace.rs`, retarde la pose de `ToLobby` de ce temps réel), `RESTART_DELAYS_MS`
  (`p2p-restart.sh`), `scripts/p2p-lobby.sh`, `scripts/p2p-sync-timeout.sh`.
- Tests unitaires : `live_players_ignores_previous_session_peers_and_keeps_order`, `sync_timeout_is_reached_only_after_the_delay`.

## 3. Preuves

Signaling Docker local (`COMPOSE_PROJECT_NAME=b2`, port 3566), `ALACOD_HEADLESS=1`, clients `--profile headless`.

**Avant le correctif** (`p2p.rs`, `core.rs`, `lobby.rs`, `p2p_tests.rs` ramenés à `25c0e49b`, le reste de la branche gardé) :
`TIMEOUT=100 scripts/p2p-lobby.sh zombies` → tentative 1 **ROUGE**, écart réel 157 ms, 0 pair ignoré, les deux clients figés à `GameStarting`.

**Après**, `scripts/p2p-lobby.sh` (`RESTART_KIND=lobby`, un client retardé de 150 à 350 ms, tour à tour ; l'écart **réel** est lu dans les
journaux et doit tomber dans 100–400 ms pour compter ; sinon l'essai se rejoue) :

| Jeu | Essai | Écart demandé | Écart réel | Pairs obsolètes ignorés (somme des 2 clients) | Verdict |
|---|---|---|---|---|---|
| zombies (partie 1 : 299 lignes, partie 2 : 299) | tentative 2 | 225 | 338 ms | 1 | VERT |
| | 3 | 250 | 264 ms | 1 | VERT |
| | 4 | 275 | 208 ms | 1 | VERT |
| | 7 | 225 | 332 ms | 1 | VERT |
| | 8 | 250 | 318 ms | 1 | VERT |
| gungeon, **après la victoire** (graine 10, inputs figés décalés, requête f7260 ; partie 1 : 7 199 lignes, partie 2 : 1 499) | 1 | 150 | 126 ms | 1 | VERT |
| | 2 | 225 | 273 ms | 1 | VERT |
| | 3 | 250 | 292 ms | 1 | VERT |
| | 4 | 275 | 300 ms | 1 | VERT |
| | 5 | 350 | 298 ms | 1 | VERT |

Binaire final (celui des mesures du §4). Zombies : 8 tentatives pour 5 dans la bande, toutes vertes (les tentatives 1, 5 et 6, à 627, 482 et 75 ms, sont hors bande et
non comptées ; l'écart demandé se mêle au décalage de frames des pairs). Chaque essai vert exige (`p2p-restart.sh`) : les deux clients sortent en 0, aucun `DesyncDetected`,
**traces identiques entre les deux clients pour la partie 1 et pour la partie 2**, partie 2 = partie 1 (graine d'origine, retour au lobby d'origine). Sur gungeon :
victoire f7143 chez les deux clients (`EXPECT_VICTORY=1`). Les 10 essais comptés (et les 3 hors bande de zombies) ont vu l'ancien socket de l'autre pair (1 pair ignoré, côté
du client revenu le premier) : la course se produit presque à chaque fois dans ces écarts, le filtre est donc réellement exercé.

**Séries précédentes** (binaires intermédiaires, mêmes scripts, même verdict) : zombies 5/5 verts à 96, 55, 257, 463, 371 ms (premier lot, avant la bande d'écart du script) puis 5/5 dans la bande à
133, 250, 347, 305, 365 ms ; gungeon après la victoire 5/5 à 168, 255, 287, 273, 309 ms ; un rejeu intermédiaire de zombies : 5/5 en 7 tentatives (144, 227, 305, 136, 108 ms comptés ; la tentative à 53 ms
n'a pas rencontré l'ancien socket : 0 pair ignoré, pas de course). Au total, **aucun essai rouge après le correctif**, sur 4 séries.

**(b) de bout en bout** : `scripts/p2p-sync-timeout.sh zombies` (`ALACOD_SYNC_TIMEOUT_SECS=4`, binaire final) : le client 1 est suspendu (SIGSTOP) dès qu'il voit
son pair, 8 s ; le client 0 construit sa session, reste `Synchronizing`, abandonne (`waited_secs=4.0`, horloge murale) et revient au lobby ; le client 1, repris,
construit sa session sur l'identifiant que le 0 a fermé, abandonne à son tour (`waited_secs=4.0`) ; les deux se retrouvent (1 pair ignoré), jouent 299 frames :
**traces identiques**, 0 `DesyncDetected`, « p2p-sync-timeout : OK ».

**Restart ordinaire rejoué** (`scripts/p2p-restart.sh zombies`, 600 frames) : traces identiques pour les deux parties, partie 2 ≠ partie 1, local
identique : OK.

## 4. Rollbacks : profondeur et fréquence (point 4 de `docs/etat-engine.md`)

**Définition** (compteur ajouté au journal de fin, `state_trace::RollbackStats`, ligne « rollbacks : n=… ») : un rollback = une restauration d'état de
bevy_ggrs (schedule `LoadWorld` ; `n` = le `load ×N` du profil de rollback, vérifié égal sur les 6 clients) ; **profondeur** = frames resimulées par ce rollback (≥ 1) =
frames déjà enregistrées que la trace revoit après la restauration **plus une** (la sauvegarde qui suit la restauration porte une clé de trace neuve ; lecture de `ggrs` 0.13 et
`bevy_ggrs` 0.22 par la relecture, cohérente avec le maximum observé de 12 = fenêtre de prédiction) ; fréquence = `n` pour 100 frames de la partie. `sans_restauration` = frames
réenregistrées sans restauration : [INFERENCE, lecture du code, non vérifiée par une exécution instrumentée] re-sauvegardes de la frame courante par bevy_ggrs à chaque tick où GGRS saute
l'`Advance` au seuil de prédiction (pas des resimulations) ; mesuré à part (le compteur historique « frames resimulées » = frames revues + ces re-sauvegardes).

Conditions : 2 clients, gungeon graine 10 `etages_deux`, inputs de bots figés (`p2p-gungeon-inputs.sh`, décalés de 5 frames), `ALACOD_INPUT_DELAY=0`,
4 500 frames, transition d'étage franchie par les deux, traces identiques, 0 `DesyncDetected`, fenêtre de prédiction 12 (`MAX_PREDICTION_WINDOW`).
Binaire final, un essai par ligne (**une** exécution par condition, pas une campagne).

| Condition (charge machine au lancement) | Client | Rollbacks n (par 100 frames) | Profondeur moy. / médiane / p90 / p99 / max | Profondeur 1 | `sans_restauration` | Frames revues (compteur historique) |
|---|---|---|---|---|---|---|
| **Naturelle**, boucle locale, sans stress ni netem (2,97) | 0 | 848 (18,9) | 1,14 / 1 / 2 / 3 / 4 | 741 (87 %) | 0 | 122 |
| | 1 | 839 (18,7) | 1,15 / 1 / 2 / 3 / 4 | 726 (87 %) | 0 | 126 |
| **netem** RTT 120 ms ± 20 ms, perte 1 % (2,39) | 0 | 904 (20,1) | 4,50 / 4 / 9 / 12 / 12 | 93 (10 %) | 813 (suite max 96) | 3 975 |
| | 1 | 915 (20,3) | 4,53 / 4 / 9 / 12 / 12 | 104 (11 %) | 827 (suite max 95) | 4 055 |
| **Stress** SIGSTOP du client 1 (12 ms / 30 ms) (2,89) | 0 | 1 262 (28,1) | 4,12 / 3 / 9 / 12 / 12 | 134 (11 %) | 457 (suite max 3) | 4 399 |
| | 1 | 82 (1,8) | 1,06 / 1 / 1 / 2 / 2 | 77 (94 %) | 0 | 5 |

Lecture : (i) même **sans** réseau dégradé, sur la boucle locale, GGRS restaure l'état environ 19 fois par 100 frames (une fois sur 5 frames) ; 87 % de ces rollbacks ne
resimulent qu'**une** frame (délai d'input 0 : l'input distant arrive une frame après la prédiction), le p99 est 3, le pire 4 ; (ii) un RTT de 120 ms avec 1 % de perte ne change presque
pas la fréquence (~20/100 frames) mais **déplace la profondeur** : médiane 4, p90 9, p99 et maximum 12 (la fenêtre de prédiction entière, 21 et 28 fois sous netem : le pire cas
`rollback_ms_worst` du budget est atteint), et il fait apparaître de longues suites de re-sauvegardes sans restauration (jusqu'à 96 ticks consécutifs, [INFERENCE] ticks bloqués au seuil de prédiction pendant
les 5 interruptions de 500 ms et plus) ; (iii) le stress à suspensions (artificiel : le client 1 est arrêté 12 ms sur 42, soit 29 % du temps) concentre les rollbacks sur le client qui devance
(1 262 contre 82) avec la même profondeur maximale 12 ; 63 `WaitRecommendation` pour ce client, 0 pour l'autre. **Aucune profondeur ne dépasse 12.**
Coût relevé au profil (moyenne ; pire cas) : restauration 2,0 à 2,6 ms ; 6,6 à 20,6 ms ; simulation d'une frame 1,6 à 2,7 ms ; sauvegarde 2,8 à 4,8 ms (machine partagée, charge 2,4 à 2,97 au lancement,
valeurs indicatives, pas une mesure de performance).
**Variabilité** : c'est une exécution par condition et deux exécutions d'une même condition ne sont pas équivalentes. La même condition « naturelle », mesurée plus tôt avec un binaire dont le
compteur différait d'une unité de profondeur (profondeur = frames revues, donc +1 pour comparer), avait donné 781 / 713 rollbacks (17,4 / 15,9) mais seulement 64 % de profondeur 1 et un maximum de 11 / 10 : la
distribution de profondeur dépend de la charge et du minutage, pas seulement de la condition. Pas de conclusion de tendance sans répétitions (non faites).
Deux mesures intermédiaires du même jour (compteur par « suites de frames ») ont été **écartées** : elles comptaient les re-sauvegardes au seuil dans le rollback ouvert (profondeur maximale 101 et 91 sous
netem, fenêtre de 12), d'où la ligne `sans_restauration`.

## 5. Nightly complet

`CARGO_TARGET_DIR` du worker, `NIGHTLY_P2P_GUNGEON=1`, sans `--quick`, sur `10816160` (début 04:14 UTC), au calme relatif (autre worker actif : charge 5 à 10) :
**code 0, 18 924 s (5 h 15)**. Résumé (`target/nightly/10816160/summary.md`) :

| Étape | Résultat | Durée |
|---|---|---|
| A. Bench (2 tranches, traces strictes) | ✅ | 1 462 s |
| B. Sim (4 bots, graines 1..50, vague 10) | ✅ | 11 287 s |
| C. P2P headless (allumette, 2 et 4 joueurs) | ⚠ **SKIPPED** : `../allumette` introuvable | — |
| **C2. P2P gungeon** (stress SIGSTOP, transition d'étage, traces identiques) | ✅ 5 597 / 26 frames resimulées, 1 / 1 transition d'étage ; `netem` **SKIPPED** (non demandé) | 5 430 s |
| D. Vidéos | ⚠ **PARTIAL** : 0 vidéo, `bevy_winit` : ni `DISPLAY` ni `WAYLAND_DISPLAY` | 743 s |
| E. Page de revue, F. Notes | générées | — |

Ce n'est pas du nouveau : C et D ne tournent pas sur ce poste (pas d'affichage, pas de dépôt allumette). **D89** (cause, acceptation) ; le nightly sort en succès
alors que D n'a rien produit, et le gabarit des notes écrit « Videos ✅ » en dur (D89 aussi). La couverture réelle de cette nuit : bench + traces, bots, p2p gungeon
en réel avec rollbacks ; pas : allumette, vidéos, réseau dégradé du nightly. C2 a duré 5 430 s (compilation à froid et attente de calme `uptime < 3` comprises).
Le nightly a tourné sur `10816160`, **avant** les derniers changements du code (horloge murale de (b), compteur de rollbacks : journal de fin seulement) ; il n'exerce pas le retour au lobby.
Les p2p de ces changements sont ceux du §3 et du §4 ; la suite de scénarios, elle, est rejouée sur le code final (§6).

## 6. Vérifications

Sur le code final (livraison `5dc1d91a` + correctifs de la relecture : horloge murale de (b), profondeur des rollbacks, scripts, documentation) :

| Vérification | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **501 scénarios verts**, 1 175 s, 3 tranches ok ; `git status` : aucune `.trace` modifiée |
| `make test_crates` | code 0 ; **1 053 tests passés, 0 échec** (dont `live_players_ignores_previous_session_peers_and_keeps_order`, `sync_timeout_is_reached_only_after_the_delay`, `rollbacks_depth_and_histogram`) |
| `make test_tools` | 19 tests OK |
| `cargo fmt --all -- --check` | propre |
| `bash -n` des scripts modifiés ou ajoutés (`p2p-lobby.sh`, `p2p-sync-timeout.sh`, `p2p-restart.sh`, `p2p-stress.sh`) | OK |
| `p2p-lobby.sh` zombies et gungeon (fin de run) ; `p2p-sync-timeout.sh` ; `p2p-restart.sh` ; `p2p-stress.sh` (3 conditions) | §3 et §4, binaire final |

Non relancés sur le code final : `make check_sim_boundary` (les fichiers touchés sont dans `game`, crate `PENDING` ; aucune dépendance ajoutée) et le nightly (voir §5).

Accident de disque en cours de route : le disque partagé est tombé à 0 octet libre (autre worker : 77 Go de target), deux `make test_crates` ont échoué à l'édition
de liens (« No space left on device ») ; `scripts/target-sweep.sh`, la suppression de `debug/` (profil `test` de mon premier `cargo test`, jamais nécessaire ensuite), de
`headless/examples/` et des binaires de test du crate `scenario` hors `scenarios` ont libéré de la place ; tout a été reconstruit et rejoué.
Incident d'outillage : vers 12:55 le binaire `~/.cargo/bin/rustup` a disparu (les liens `cargo`, `rustc`… pointent dans le vide : « command not found: cargo ») alors que le toolchain
`nightly` (mis à jour à 07:36) est intact ; je ne l'ai pas réparé (poste partagé) : le rejeu final des essais p2p a mis le `bin/` du toolchain en tête du `PATH`.

## 7. Limites

- Une machine, un VPS partagé : charge variable, mesures de §4 à une exécution par condition ; aucune conclusion sur un vrai WAN, NAT, arm64.
- (b) abandonne après 30 s une session qui n'est pas **légitimement** lente à démarrer : non mesuré sur des cartes lentes de jeu réel (fenêtre, assets).
- Le retour au lobby en **allumette** n'est pas couvert (le flux est un autre chemin, non touché).
- `sans_restauration` : cause déduite du code (re-sauvegardes au seuil de prédiction), non vérifiée par une exécution instrumentée.
- Le repli du restart (`RESTART_TIMEOUT_SECS`) lit encore `Time<Real>` : en headless, 30 « secondes » durent ~10 s réelles (défaut antérieur à ce chantier, constaté à la relecture, non corrigé : hors périmètre).
- Les enregistrements de bots (inputs figés) sont caducs au moindre changement de simulation.

## 8. Relecture indépendante (agent `reviewer`, lecture seule, avant merge)

Elle n'a trouvé **aucun bloquant** et a vérifié, sans défaut : toutes les lectures de `socket.players()` filtrent (a) ; pas de pair légitime retiré (identifiants neufs à chaque socket, y compris
salles `-r{n}`, hub, 2e partie) ; handles GGRS identiques entre `GgrsSessionBuilding` et la session ; garde-fous de (b) (hors ligne, synctest, `RunRequest` déjà posée, remise à zéro, pas de boucle d'abandons) ;
rien dans `GgrsSchedule`, l'état rollback, le checksum ni les traces ; chiffres du rapport contre les journaux. Deux défauts **importants**, corrigés :

1. **Délai de (b) compté en updates en headless** (`Time<Real>` avance de 1/60 s par update) : « 4 s » = 1,3 s réelles. Corrigé : `Instant` (horloge murale), `waited_secs` réel ; test unitaire
   et `p2p-sync-timeout.sh` rejoués (`waited_secs=4.0`).
2. **Profondeur des rollbacks décalée de 1** : la restauration de la frame `f` est suivie d'une sauvegarde sur une clé neuve, donc la trace ne revoit que `profondeur − 1` frames ; la « profondeur 0 »
   n'existait pas (une frame resimulée) et le maximum n'était pas 11 mais 12 = la fenêtre entière. Corrigé : profondeur ≥ 1, test, **toutes les mesures du §4 refaites** (et les tableaux de
   `etat-engine.md`, du rapport `m2-p2p-run-complete` §8, de `conventions.md`), `sans_restauration` re-expliqué (inférence).

Mineurs corrigés : `p2p-sync-timeout.sh` (`grep` sans correspondance sous `pipefail`), en-tête de `p2p-restart.sh` (`RESTART_DELAYS_MS` vide par défaut), garde `GAPS_MS` vide de `p2p-lobby.sh`,
valeur invalide de `ALACOD_SYNC_TIMEOUT_SECS` (avertissement et 30 s), coûts du §4 (valeurs des essais finaux), raison de l'écart des mesures intermédiaires, renvoi à un `## Summary` inexistant.
Non corrigés : minuit dans le calcul de l'écart de `p2p-lobby.sh` (un essai à cheval sur minuit est rejoué, jamais un faux vert) ; `summary(end)` divise par les frames écrites (négligeable).
