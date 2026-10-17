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
| 3. Nightly complet `NIGHTLY_P2P_GUNGEON=1` | **lancé et terminé** : code 0, 18 924 s (5 h 15) ; étapes C et D non exécutées utilement (D88), §5 |
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
  `SYNC_TIMEOUT_SECS` = **30 s** (temps réel ; surcharge d'essai `ALACOD_SYNC_TIMEOUT_SECS`) pose `RunRequest::ToLobby` et journalise
  `ggrs{f=0 session_sync_timeout waited_secs=4 players=2} session non synchronisée, abandon et retour au lobby`. 30 s : l'autre pair peut
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
| zombies (partie 1 : 299 lignes, partie 2 : 299) | 1 | 150 | 133 ms | 1 | VERT |
| | 2 | 225 | 250 ms | 1 | VERT |
| | 3 | 250 | 347 ms | 1 | VERT |
| | 4 | 275 | 305 ms | 1 | VERT |
| | 5 | 350 | 365 ms | 1 | VERT |
| gungeon, **après la victoire** (graine 10, inputs figés décalés, requête f7260 ; partie 1 : 7 199 lignes, partie 2 : 1 499) | 1 | 150 | 168 ms | 1 | VERT |
| | 2 | 225 | 255 ms | 1 | VERT |
| | 3 | 250 | 287 ms | 1 | VERT |
| | 4 | 275 | 273 ms | 1 | VERT |
| | 5 | 350 | 309 ms | 1 | VERT |

Chaque essai vert exige (`p2p-restart.sh`) : les deux clients sortent en 0, aucun `DesyncDetected`, **traces identiques entre les deux clients pour la
partie 1 et pour la partie 2**, partie 2 = partie 1 (graine d'origine, retour au lobby d'origine). Sur gungeon : victoire f7143 chez les deux clients
(`EXPECT_VICTORY=1`). Les 10 essais ont vu l'ancien socket de l'autre pair (1 pair ignoré, côté du client revenu le premier) : la course se produit à tous les coups
dans ces écarts, le filtre est donc réellement exercé. Un premier lot zombies (avant d'ajouter la bande d'écart au script) : 5/5 verts aussi, écarts
réels 96, 55, 257, 463, 371 ms (deux hors de la bande, d'où la correction du script : l'écart demandé se mêle au décalage de frames des pairs).
**Rejeu sur le binaire final** (après la dernière modification du code, `state_trace.rs`, qui ne touche pas la session) : `p2p-lobby.sh zombies` **5/5 dans la bande**
en 7 tentatives (144, 227, 305, 136, 108 ms comptés ; 93 et 53 ms verts mais hors bande, non comptés ; la tentative à 53 ms n'a pas rencontré l'ancien socket :
0 pair ignoré, donc pas de course), `p2p-sync-timeout.sh` OK (deux abandons, traces identiques), `p2p-restart.sh` OK (partie 2 ≠ partie 1, local identique).
Les essais gungeon (fin de run) ont été faits sur le binaire d'avant cette dernière modification.

**(b) de bout en bout** : `scripts/p2p-sync-timeout.sh zombies` (`ALACOD_SYNC_TIMEOUT_SECS=4`) : le client 1 est suspendu (SIGSTOP) dès qu'il voit
son pair, 8 s ; le client 0 construit sa session, reste `Synchronizing`, abandonne (« waited_secs=4 ») et revient au lobby ; le client 1, repris,
construit sa session sur l'identifiant que le 0 a fermé, abandonne à son tour ; les deux se retrouvent (1 pair ignoré), jouent 299 frames :
**traces identiques**, 0 `DesyncDetected`, « p2p-sync-timeout : OK ».

**Restart ordinaire rejoué** (`scripts/p2p-restart.sh zombies`, 600 frames) : traces identiques pour les deux parties, partie 2 ≠ partie 1, local
identique : OK.

## 4. Rollbacks : profondeur et fréquence (point 4 de `docs/etat-engine.md`)

**Définition** (compteur ajouté au journal de fin, `state_trace::RollbackStats`, ligne « rollbacks : n=… ») : un rollback = une restauration d'état de
bevy_ggrs (schedule `LoadWorld` ; `n` = le `load ×N` du profil de rollback, vérifié égal sur les 6 clients) ; **profondeur** = frames déjà
enregistrées que la trace revoit après la restauration, jusqu'à la première frame nouvelle (0 : GGRS restaure sans rien rejouer) ; fréquence = `n`
pour 100 frames de la partie. `sans_restauration` = frames réenregistrées **sans** restauration préalable : [INFERENCE non vérifiée] re-sauvegardes
de GGRS hors rollback ; mesuré à part pour ne pas gonfler les profondeurs (`n × moyenne + sans_restauration` = compteur « frames resimulées »).

Conditions : 2 clients, gungeon graine 10 `etages_deux`, inputs de bots figés (`p2p-gungeon-inputs.sh`, décalés de 5 frames), `ALACOD_INPUT_DELAY=0`,
4 500 frames, transition d'étage f3730 franchie par les deux, traces identiques, 0 `DesyncDetected`, fenêtre de prédiction 12 (`MAX_PREDICTION_WINDOW`).
Binaire final, un essai par ligne (**une** exécution par condition, pas une campagne).

| Condition (charge machine au lancement) | Client | Rollbacks n (par 100 frames) | Profondeur moy. / médiane / p90 / p99 / max | Profondeur 0 | `sans_restauration` | Frames resimulées |
|---|---|---|---|---|---|---|
| **Naturelle**, boucle locale, sans stress ni netem (2,95) | 0 | 781 (17,4) | 0,63 / 0 / 2 / 6 / 10 | 497 (64 %) | 0 | 491 |
| | 1 | 713 (15,9) | 0,58 / 0 / 2 / 5 / 9 | 454 (64 %) | 3 (suite max 2) | 416 |
| **netem** RTT 120 ms ± 20 ms, perte 1 % (**12,06**, machine chargée) | 0 | 1 040 (23,1) | 1,91 / 1 / 4 / 9 / 11 | 186 (18 %) | 154 (suite max 49) | 2 136 |
| | 1 | 1 075 (23,9) | 1,87 / 1 / 4 / 9 / 11 | 179 (17 %) | 140 (suite max 47) | 2 151 |
| **Stress** SIGSTOP du client 1 (12 ms / 30 ms) (5,37) | 0 | 1 185 (26,3) | 4,26 / 3 / 10 / 11 / 11 | 98 (8 %) | 715 (suite max 5) | 5 761 |
| | 1 | 45 (1,0) | 0,02 / 0 / 0 / 1 / 1 | 44 (98 %) | 0 | 1 |

Lecture : (i) même **sans** réseau dégradé, GGRS restaure l'état environ 16 fois par 100 frames (une fois sur 6 frames), dont près des deux tiers sans rien
rejouer (restauration seule : 2,8 ms en moyenne, jusqu'à 18,8 ms, relevés du profil) ; la profondeur reste ≤ 10 ; (ii) un RTT de 120 ms avec
1 % de perte pousse la profondeur moyenne à ~1,9 (~2,3 frames resimulées par rollback non vide) et le p90 à 4, et la fréquence à ~23/100 frames ;
(iii) le stress à suspensions (artificiel : le client 1 est arrêté 12 ms sur 42, soit 29 % du temps) donne des profondeurs jusqu'à 11 : **aucune profondeur ne
dépasse 11 (fenêtre : 12)**. Coût d'une restauration relevé en parallèle : 2,8 à 3,3 ms en moyenne, 10 à 29 ms au pire ; simulation d'une frame 1,5 à
2,6 ms ; sauvegarde 2,7 à 6,4 ms (machine chargée, valeurs indicatives, pas une mesure de performance). La charge de la machine a varié (2,95 à 12,06) :
l'essai netem a été fait sous charge 12, ses nombres (fréquence, `sans_restauration`) ne sont pas ceux d'une machine au calme. Pas de comparaison de
fréquence entre conditions sans plusieurs répétitions (non faites).
Deux mesures antérieures du même jour sur un binaire intermédiaire (compteur par « suites de frames » au lieu des restaurations) ont été **écartées** :
leur `n` ne coïncidait pas avec `load ×N` ; le compteur final l'égale.

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

Ce n'est pas du nouveau : C et D ne tournent pas sur ce poste (pas d'affichage, pas de dépôt allumette). **D88** (cause, acceptation) ; le nightly sort en succès
alors que D n'a rien produit, et le gabarit des notes écrit « Videos ✅ » en dur (D88 aussi). La couverture réelle de cette nuit : bench + traces, bots, p2p gungeon
en réel avec rollbacks ; pas : allumette, vidéos, réseau dégradé du nightly. C2 a duré 5 430 s (compilation à froid et attente de calme `uptime < 3` comprises).
Le nightly a tourné sur `10816160`, **avant** la dernière version du compteur de rollbacks (journal de fin seulement : restaurations `LoadWorld` au lieu de suites de frames) ;
les p2p de cette dernière version sont ceux du §3 et du §4 (mesures faites avec elle). La suite de scénarios, elle, est rejouée sur le code final (§6).

## 6. Vérifications

Sur le code final (voir `## Summary` pour le sha) : `make test_scenarios_par SHARDS=3` sans BLESS, `make test_crates`, `make test_tools`, `cargo fmt --all -- --check`,
`bash -n` des scripts : résultats ci-dessous.

Sur le code final (état `10816160` + `state_trace.rs` + documentation, avant le commit de livraison) :

| Vérification | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **501 scénarios verts**, 1 363 s, 3 tranches ok ; `git status` : aucune `.trace` modifiée |
| `make test_crates` | code 0 ; **1 053 tests passés, 0 échec** (dont `live_players_ignores_previous_session_peers_and_keeps_order`, `sync_timeout_is_reached_only_after_the_delay`, `rollbacks_depth_and_histogram`) |
| `make test_tools` | 19 tests OK |
| `cargo fmt --all -- --check` | propre |
| `bash -n` des scripts modifiés ou ajoutés (`p2p-lobby.sh`, `p2p-sync-timeout.sh`, `p2p-restart.sh`, `p2p-stress.sh`) | OK |
| `p2p-lobby.sh` zombies, gungeon ; `p2p-sync-timeout.sh` ; `p2p-restart.sh` ; `p2p-stress.sh` (3 conditions) | §3 et §4 |

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
- `sans_restauration` : cause non établie (§4).
- Les enregistrements de bots (inputs figés) sont caducs au moindre changement de simulation.
