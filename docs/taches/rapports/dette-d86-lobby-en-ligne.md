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
| 3. Nightly complet `NIGHTLY_P2P_GUNGEON=1` | **lancé et terminé** : code 0, 18 924 s (5 h 15) ; étapes C et D non exécutées utilement (D94), §5 |
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
| zombies (partie 1 : 299 lignes, partie 2 : 299) — binaire final | 1 | 150 | 152 ms | 1 | VERT |
| | 2 | 225 | 240 ms | 1 | VERT |
| | 3 | 250 | 331 ms | 1 | VERT |
| | 4 | 275 | 267 ms | 1 | VERT |
| | 5 | 350 | 385 ms | 1 | VERT |
| gungeon, **après la victoire** (binaire de `3eb60a86`, avant les derniers changements, voir ci-dessous ; graine 10, inputs figés décalés, requête f7260 ; parties 1 et 2 : 7 199 et 1 499 lignes) | 1 | 150 | 126 ms | 1 | VERT |
| | 2 | 225 | 273 ms | 1 | VERT |
| | 3 | 250 | 292 ms | 1 | VERT |
| | 4 | 275 | 300 ms | 1 | VERT |
| | 5 | 350 | 298 ms | 1 | VERT |

Zombies : 5 tentatives pour 5 dans la bande, toutes vertes (binaire final : `origin/main` `4630291a` fusionné, profondeur des rollbacks corrigée). Chaque essai vert exige (`p2p-restart.sh`) : les deux clients sortent en 0,
aucun `DesyncDetected`, **traces identiques entre les deux clients pour la partie 1 et pour la partie 2**, partie 2 = partie 1 (graine d'origine, retour au lobby d'origine). Sur gungeon :
victoire f7143 chez les deux clients (`EXPECT_VICTORY=1`) ; cette série n'a **pas** été rejouée sur le binaire final (12 min par série, et les derniers changements ne touchent pas la session : horloge murale de
(b) déjà incluse, compteur de rollbacks, fusions de `main`) ; le code de session de cette série est celui d'aujourd'hui (`jjrs/p2p.rs`, `core.rs`, `ui/lobby.rs` inchangés depuis `3eb60a86`). Les 10 essais comptés ont vu
l'ancien socket de l'autre pair (1 pair ignoré, côté du client revenu le premier) : la course se produit à chaque fois dans ces écarts, le filtre est donc réellement exercé.

**Séries précédentes** (binaires intermédiaires, mêmes scripts, même verdict) : zombies 5/5 verts à 96, 55, 257, 463, 371 ms (premier lot, avant la bande d'écart du script), puis 5/5 dans la bande à
133, 250, 347, 305, 365 ms, puis 8 tentatives pour 5 dans la bande (338, 264, 208, 332, 318 ms ; 627, 482, 75 ms hors bande, non comptées, toutes vertes), puis 6 tentatives (239, 291, 268, 392, 166 ms ; 87 ms hors bande) ;
gungeon après la victoire 5/5 à 168, 255, 287, 273, 309 ms avant celle du tableau. Une tentative à 53 ms n'a pas rencontré l'ancien socket (0 pair ignoré, pas de course). Au total, **aucun essai rouge après le
correctif**, sur 6 séries.

**(b) de bout en bout** : `scripts/p2p-sync-timeout.sh zombies` (`ALACOD_SYNC_TIMEOUT_SECS=4`, binaire final) : le client 1 est suspendu (SIGSTOP) dès qu'il voit
son pair, 8 s ; le client 0 construit sa session, reste `Synchronizing`, abandonne (`waited_secs=4.0`, horloge murale) et revient au lobby ; le client 1, repris,
construit sa session sur l'identifiant que le 0 a fermé, abandonne à son tour (`waited_secs=4.0`) ; les deux se retrouvent (1 pair ignoré), jouent 299 frames :
**traces identiques**, 0 `DesyncDetected`, « p2p-sync-timeout : OK ».

**Restart ordinaire rejoué** (`scripts/p2p-restart.sh zombies`, 600 frames) : traces identiques pour les deux parties, partie 2 ≠ partie 1, local
identique : OK.

## 4. Rollbacks : profondeur et fréquence (point 4 de `docs/etat-engine.md`)

**Définition** (compteur ajouté au journal de fin, `state_trace::RollbackStats`, ligne « rollbacks : n=… ») : un rollback = une restauration d'état de
bevy_ggrs (schedule `LoadWorld` ; `n` = le `load ×N` du profil de rollback, égal sur les 6 clients) ; **profondeur** = frames que GGRS fait resimuler =
`dernière frame avancée avant la restauration − frame restaurée` (`RollbackFrameCount` lu à la fin de chaque `AdvanceWorld`, puis à `LoadWorld`) ; fréquence = `n` pour
100 frames de la partie. **Cette définition ne dépend pas de la trace.** Les deux versions précédentes du compteur la déduisaient des clés de trace revues (frame déjà enregistrée = resimulée) :
or GGRS sauvegarde la frame courante à chaque tick, y compris un tick bloqué au seuil de prédiction qui n'avance pas (`handle_rollback_and_save` avant `advance_rollback_frame`,
`ggrs` 0.13 `p2p_session.rs:855-968`) ; ces re-sauvegardes étaient comptées comme resimulées (profondeur 101 et 91 sous netem à la 1re version ; surestimation à la 2e : exemple frame 100 déjà
sauvegardée, restauration à 98, deux `Advance` jusqu'à 100 = 2 frames resimulées, comptées 3). **Toutes les profondeurs publiées avant la revue 2 étaient fausses** (rapport et tableaux de `etat-engine.md`,
`m2-p2p-run-complete` §8). Test unitaire `rollback_depth_is_frames_advanced_not_trace_keys` (cas du seuil : profondeur exacte 2, puis 1, puis 0). Le compteur historique « frames resimulées » du journal de fin
(clés de trace revues) n'a pas changé de définition ; il n'est plus utilisé pour la profondeur.

Conditions : 2 clients, gungeon graine 10 `etages_deux`, inputs de bots figés (`p2p-gungeon-inputs.sh`, décalés de 5 frames), `ALACOD_INPUT_DELAY=0`,
4 500 frames, transition d'étage franchie par les deux (f3497 : les inputs ont dû être **réenregistrés** sur `origin/main` `4630291a`, le contenu ayant changé depuis `803cc9f0`, D88 et D95), traces identiques,
0 `DesyncDetected`, fenêtre de prédiction 12 (`MAX_PREDICTION_WINDOW`). Binaire final (profondeur par frames avancées, `origin/main` `4630291a` fusionné). La machine était partagée avec b6 (benchs) :
ces essais n'ont été lancés qu'hors de ses mesures, charge au lancement 2,6 à 2,8.

| Condition (charge au lancement) | Client | Rollbacks n (par 100 frames) | Profondeur moy. / médiane / p90 / p99 / max | Profondeur 1 | Frames revues (compteur historique) |
|---|---|---|---|---|---|
| **Naturelle**, boucle locale, sans stress ni netem (2,67) | 0 | 732 (16,3) | 1,27 / 1 / 2 / 3 / 4 | 576 (79 %) | 199 |
| | 1 | 705 (15,7) | 1,24 / 1 / 2 / 4 / 7 | 577 (82 %) | 168 |
| **netem** RTT 120 ms ± 20 ms, perte 1 % (2,81) | 0 | 822 (18,3) | 4,04 / 3 / 8 / 11 / 11 | 102 (12 %) | 3 090 |
| | 1 | 840 (18,7) | 3,98 / 3 / 8 / 11 / 11 | 116 (14 %) | 3 150 |
| **Stress** SIGSTOP du client 1 (12 ms / 30 ms) (2,64) | 0 | 1 099 (24,4) | 5,14 / 4 / 10 / 11 / 11 | 79 (7 %) | 5 199 |
| | 1 | 53 (1,2) | 1,11 / 1 / 1 / 3 / 3 | 49 (92 %) | 6 |

**Répétitions** (trois exécutions par condition avec le compteur corrigé : v5 sur `803cc9f0`, v6 et v7 sur `4630291a` ; v6 avec des inputs périmés, EXPECT_FLOORS rouge, traces identiques, 0 desync, gardée
pour la distribution) : rollbacks par 100 frames (client 0) **naturelle** 17,9 / 16,4 / 16,3, profondeur moyenne 1,35 / 1,45 / 1,27, maximum 11 / 10 / 4 ; **netem** 18,7 / 22,5 / 18,3, moyenne 4,67 / 3,80 / 4,04,
maximum 11 / 11 / 11 ; **stress** 27,7 / 28,9 / 24,4, moyenne 3,81 / 4,62 / 5,14, maximum 11 / 11 / 11. La médiane est 1 (naturelle) et 3 à 4 (netem, stress client 0) dans les trois séries.

**Avant / après la correction** (l'« avant » est le compteur par clés de trace de la revue 1, une exécution ; l'« après » est la dernière série v7) :

| Condition | Avant : rollbacks/100 frames ; profondeur moy. ; max | Après : rollbacks/100 frames ; profondeur moy. ; max |
|---|---|---|
| Naturelle | 18,9 / 18,7 ; 1,14 / 1,15 ; 4 / 4 | 16,3 / 15,7 ; 1,27 / 1,24 ; 4 / 7 |
| netem | 20,1 / 20,3 ; 4,50 / 4,53 ; 12 / 12 | 18,3 / 18,7 ; 4,04 / 3,98 ; 11 / 11 |
| Stress (client 0 / 1) | 28,1 / 1,8 ; 4,12 / 1,06 ; 12 / 2 | 24,4 / 1,2 ; 5,14 / 1,11 ; 11 / 3 |

Lecture : (i) même **sans** réseau dégradé, sur la boucle locale, GGRS restaure l'état environ 16 fois par 100 frames (une fois sur 6 frames) ; 80 % de ces rollbacks ne
resimulent qu'**une** frame (délai d'input 0 : l'input distant arrive une frame après la prédiction), le p99 est 3 à 5 ; (ii) un RTT de 120 ms avec 1 % de perte ne change presque
pas la fréquence (~18 à 22/100 frames) mais **déplace la profondeur** : médiane 3 à 4, p90 8 à 10, p99 11, moyenne ~4, et seuls 9 à 14 % des rollbacks ne resimulent qu'une frame ; (iii) le stress à
suspensions (artificiel : le client 1 est arrêté 12 ms sur 42, soit 29 % du temps) concentre les rollbacks sur le client qui devance (1 099 contre 53) ; 69 `WaitRecommendation` pour ce client, 0 pour l'autre.
**La profondeur maximale mesurée est 11, jamais 12** (fenêtre de prédiction 12) sur les neuf exécutions : le pire cas du budget `rollback_ms_worst` (restauration + 12 frames) n'a pas été atteint.
La relecture avait supposé le contraire en lisant le code ; la mesure ne le confirme pas.
Coût relevé au profil (moyenne ; pire cas, série v7) : restauration 2,0 à 3,0 ms ; 7,6 à 23,5 ms ; simulation d'une frame 1,7 à 2,8 ms ; sauvegarde 3,0 à 4,7 ms (machine partagée, valeurs indicatives,
pas une mesure de performance).
**Variabilité** : la médiane et la forme de la distribution tiennent d'une exécution à l'autre, pas le maximum de la condition « naturelle » (4, 10, 11) ni la fréquence sous netem (18 à 22) : pas de conclusion
de tendance fine entre conditions, trois exécutions chacune, une machine, une graine.

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

Ce n'est pas du nouveau : C et D ne tournent pas sur ce poste (pas d'affichage, pas de dépôt allumette). **D94** (cause, acceptation) ; le nightly sort en succès
alors que D n'a rien produit, et le gabarit des notes écrit « Videos ✅ » en dur (D94 aussi). La couverture réelle de cette nuit : bench + traces, bots, p2p gungeon
en réel avec rollbacks ; pas : allumette, vidéos, réseau dégradé du nightly. C2 a duré 5 430 s (compilation à froid et attente de calme `uptime < 3` comprises).
Le nightly a tourné sur `10816160`, **avant** les derniers changements du code (horloge murale de (b), compteur de rollbacks : journal de fin seulement) ; il n'exerce pas le retour au lobby.
Les p2p de ces changements sont ceux du §3 et du §4 ; la suite de scénarios, elle, est rejouée sur le code final (§6).

## 6. Vérifications

Sur le code final (`origin/main` `4630291a` fusionné + profondeur des rollbacks par frames avancées ; seules des modifications de documentation suivent les vérifications) :

| Vérification | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **642 scénarios verts**, 1 077 s, 3 tranches ok ; aucune `.trace` modifiée (`git status` propre hors documentation) |
| `make test_crates` | code 0 ; **1 221 tests passés, 0 échec** (dont `live_players_ignores_previous_session_peers_and_keeps_order`, `sync_timeout_is_reached_only_after_the_delay`, `rollback_depth_is_frames_advanced_not_trace_keys`) |
| `make test_tools` | 19 tests OK |
| `cargo fmt --all -- --check` | propre |
| `bash -n` des scripts modifiés ou ajoutés | OK |
| `p2p-lobby.sh` zombies (5/5) ; `p2p-sync-timeout.sh` ; `p2p-restart.sh` ; `p2p-stress.sh` gungeon (3 conditions, 3 exécutions) | §3 et §4, binaire final |

Incident de la fusion de `803cc9f0` : `make test_scenarios_par` y a rendu `achievement_unlocks` rouge (trace différente à la ligne 6) ; **ce n'est pas mon changement** : le même scénario, joué sur `origin/main`
`803cc9f0` seul (worktree propre), échoue avec les mêmes empreintes (attendu `…1318af9a…`, obtenu `…b18a996a…`) ; `main` l'a corrigé depuis (D96, `4cd1b245`). Aussi : les inputs de bots figés de la série
précédente étaient périmés sur `main` (aucun franchissement d'étage, `EXPECT_FLOORS` rouge, traces identiques, 0 desync) : réenregistrés (franchissement f3497).

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
- Profondeur des rollbacks : exacte par construction (frames avancées) ; trois exécutions par condition, une machine, une graine ; la fréquence et le maximum varient d'une exécution à l'autre (§4).
- Le repli du restart (`RESTART_TIMEOUT_SECS`) lit encore `Time<Real>` : en headless, 30 « secondes » durent ~10 s réelles (défaut antérieur à ce chantier, constaté à la relecture, non corrigé : hors périmètre).
- Les enregistrements de bots (inputs figés) sont caducs au moindre changement de simulation.

## 8. Relecture indépendante (agent `reviewer`, lecture seule, avant merge)

Elle n'a trouvé **aucun bloquant** et a vérifié, sans défaut : toutes les lectures de `socket.players()` filtrent (a) ; pas de pair légitime retiré (identifiants neufs à chaque socket, y compris
salles `-r{n}`, hub, 2e partie) ; handles GGRS identiques entre `GgrsSessionBuilding` et la session ; garde-fous de (b) (hors ligne, synctest, `RunRequest` déjà posée, remise à zéro, pas de boucle d'abandons) ;
rien dans `GgrsSchedule`, l'état rollback, le checksum ni les traces ; chiffres du rapport contre les journaux. Deux défauts **importants**, corrigés :

1. **Délai de (b) compté en updates en headless** (`Time<Real>` avance de 1/60 s par update) : « 4 s » = 1,3 s réelles. Corrigé : `Instant` (horloge murale), `waited_secs` réel ; test unitaire
   et `p2p-sync-timeout.sh` rejoués (`waited_secs=4.0`).
2. **Profondeur des rollbacks décalée de 1** (corrigée une première fois : profondeur ≥ 1 en ajoutant une unité à la profondeur déduite de la trace) : voir la revue 2.

Mineurs corrigés : `p2p-sync-timeout.sh` (`grep` sans correspondance sous `pipefail`), en-tête de `p2p-restart.sh` (`RESTART_DELAYS_MS` vide par défaut), garde `GAPS_MS` vide de `p2p-lobby.sh`,
valeur invalide de `ALACOD_SYNC_TIMEOUT_SECS` (avertissement et 30 s), coûts du §4 (valeurs des essais finaux), raison de l'écart des mesures intermédiaires, renvoi à un `## Summary` inexistant.
Non corrigés : minuit dans le calcul de l'écart de `p2p-lobby.sh` (un essai à cheval sur minuit est rejoué, jamais un faux vert) ; `summary(end)` divise par les frames écrites (négligeable).

### Revue 2 (bloquant, corrigé)
La relecture suivante a montré que **déduire la profondeur des clés de trace revues est faux par construction** : GGRS sauvegarde la frame courante à chaque tick, même bloqué au seuil de prédiction
(`handle_rollback_and_save` avant la décision de ne pas avancer, `ggrs` 0.13 `p2p_session.rs:855-968`) ; frame 100 déjà sauvegardée, restauration à 98, deux `Advance` : 2 frames resimulées, comptées 3, et le
rollback restait « ouvert » jusqu'à une clé nouvelle. Correction : `RollbackStats` ne lit plus la trace ; profondeur = `dernière frame avancée − frame restaurée` (`RollbackFrameCount` à la fin de
chaque `AdvanceWorld`, puis à `LoadWorld`), `sans_restauration` supprimé ; test `rollback_depth_is_frames_advanced_not_trace_keys` (cas du seuil : profondeurs exactes 2, 1, 0). **Mesures refaites**
(§4, tableau avant/après) et tableaux de `etat-engine.md` et de `m2-p2p-run-complete` §8 mis à jour ; correction notable : la profondeur maximale est **11, pas 12**.
Aussi : ma dette de nightly est renumérotée **D94** (D88 à D92 : suivis de main, D93 : `dette-d88`).
