# Durcissement P2P, CI et preuves de validation

Branche : `hardening-engine-p2p-ci`. Base : `b917c38f` (2026-10-14).
Demande de William : préparer des modifications pour la pérennité de l’engine, à revoir dans
sa session d’orchestration. Pas de merge sur main dans cette tâche.

## Changements à reviewer

1. **Détection P2P active.** Le builder commun natif/navigateur utilise `desync_interval` (10
   frames dans les arguments par défaut), refuse zéro et fixe explicitement les 60 Hz de l’engine.
   GGRS était resté sur `DesyncDetection::Off` dans ce chemin. Trois tests utilisent le builder
   de production avec deux pairs en mémoire : intervalle invalide, inputs retardés avec rollback
   sans desync, checksum divergent détecté. Aucune modification des inputs de jeu ni des types hashés.
2. **Destruction des balles ordonnée par NetId.** Remplacement du HashSet puis tri par
   `Entity.index()` par un BTreeMap indexé par `GgrsNetId.0`. Les Entity sont locales au client.
   Même politique de collision et `despawn_rollback()` ; aucune trace ne doit être bénie pour ce lot.
3. **CI complète.** `test_crates` couvre tout le workspace sans features par défaut, au profil
   headless, avec exclusion exacte de la suite `scenarios` jouée séparément. La CI utilise cette
   cible, teste les outils et le lint. L’ancien `cargo test` à la racine ne couvrait pas les tests
   unitaires de toutes les crates (dont ceux de game). `make gungeon` transmet aussi
   `DEV_FEATURES`, comme les autres jeux, pour éviter une seconde compilation de Bevy en local.
4. **Contrôle de déterminisme strict.** Imports groupés, collections Bevy, noms non qualifiés,
   RNG système et horloge sont détectés dans les crates concernées ; les nouveaux crates sont
   contrôlés par défaut. Exceptions exactes avec compte et raison, y compris les mesures de tests
   et les maps de chargement/présentation. Une exception périmée échoue aussi. Suppression d’un
   import HashSet inutilisé. Le contrôle est lexical, pas une preuve de déterminisme.
5. **Traces P2P vérifiées.** Le nightly supprime les anciens fichiers, impose des inputs neutres
   et exige 599 frames contiguës de chaque pair pour le smoke à 600 frames. Absence, fichier vide,
   trace tronquée ou divergence ne peut plus passer comme succès.
6. **Scénarios et métriques fiables.** Les identifiants des scénarios générés incluent le jeu
   (`generated/<jeu>/<nom>`). Les noms courts uniques restent acceptés. Six noms de fichiers
   étaient partagés par plusieurs jeux ; le BTreeMap des métriques pouvait les écraser. Le filtre
   refuse noms inconnus/ambigus, les partitions invalides échouent, le rendu actif ne transforme
   plus la suite en succès sans exécution. Les métriques shardées exigent toutes les parts du run,
   refusent les doublons et n’écrasent pas latest.json après échec. Le script transmet aussi
   `DYLD_LIBRARY_PATH` au binaire de test direct sur macOS.
7. **Performance explicite.** `make perf_review PERF_BASELINE=...` compare des mesures choisies,
   avec contrôle des frames/joueurs, refus des données absentes/invalides et seuil de régression
   (10 % par défaut). Objectif absolu optionnel, indépendant des budgets historiques. Aucun seuil
   historique relevé, aucune revendication de gain de vitesse.
8. **Documentation d’état et de sortie.** `docs/etat-engine.md`, README, ancien milestone,
   préambule du plan et consignes de validation : distinguer livré, accessible, testé et validé
   humainement. Suite proposée : D59, boucle M2, calibration/profilage, frontières des dépendances.

## Vérification

Toutes les commandes ci-dessous ont été exécutées localement ; aucune référence `.trace` réécrite.

- Trois tests P2P ciblés : verts (inputs variables, retards, rollback, divergence injectée).
- Treize tests Python des garde-fous et comparateurs : verts.
- Filtres inconnus/ambigus, slice hors bornes, shard malformé : quatre exécutions négatives
  du runner retournent 101. Script sharding avec zéro tranche : retour 2.
- Deux tests ciblés de sélection/partition : verts.
- `make format`, `make check_forbidden`, `make check_rollback_registration` : verts.
  Contrôle lexical : 36 occurrences historiques explicitement justifiées, aucune nouvelle implicite.
  `bash -n` sur les trois scripts shell touchés et `git diff --check` : verts.
- `ALACOD_DYLIB=0 scripts/test-scenarios-sharded.sh 4` : **255 scénarios verts**, sans bless,
  529 s au total, quatre tranches ; **255 métriques distinctes** après fusion. Aucun `.trace` changé.
- `ALACOD_DYLIB=0 make test_crates` : **728 tests réussis, zéro échec, 10 ignorés**
  (tests et doc-tests du workspace ; suite `scenarios` exclue exactement, jouée ci-dessus).
- `ALACOD_DYLIB=0 make lint` : **zéro erreur** dans les quatre jeux ; avertissements
  zombies 1, testbed 61, throne 3, gungeon 11 (aucune logique de lint ni aucun contenu changé).
- Deux scénarios `weapon_pistol` sélectionnés par ids complets : verts, traces inchangées.
- `make -n gungeon` avec `ALACOD_DYLIB=1` puis `0` : présence puis absence de `game/dev-dylib`, comme attendu.
- Environnement : macOS aarch64, Rust `1.101.0-nightly (0abfedbc7 2026-10-02)`, builds statiques
  (`ALACOD_DYLIB=0`). Suites sous charge : les FPS de ce run ne sont pas une référence de performance.

## Limites et reprise par l’orchestrateur

- Docker est installé mais son daemon ne répond pas sur cette machine : WebRTC/allumette et le
  restart P2P réel restent à exécuter. Les tests en mémoire ne valident pas le NAT ni le navigateur.
- Pas de build wasm (target non installé), pas de validation entre architectures ni de run humaine.
  Le build dylib n’a pas été reconstruit ; vérification statique des commandes et variables seulement.
- Les suites tournent sous charge ; leurs débits ne constituent pas une calibration. Refaire les
  benches au calme sur une machine nommée avant de fixer un objectif ou de conclure sur la fluidité.
- Aucun asset, contenu de jeu ou fichier `.trace` changé. D59, D61, D62, D60 et les autres tâches
  de contenu restent dans leurs fiches ; ne pas les considérer résolues par ce lot.
- La frontière ECS seule reste un chantier : ce lot documente les dépendances de présentation
  encore présentes, sans prétendre les avoir supprimées ni avoir validé un portage console.
- Les consommateurs externes de métriques doivent utiliser les nouveaux ids des scénarios générés ;
  refaire leur référence pour comparer ces scénarios. Les ids des scénarios manuscrits restent identiques.

## Dépendances : signal remonté au push

GitHub a signalé **58 alertes de dépendances sur la branche par défaut** (24 hautes,
24 modérées, 10 faibles) lors du push. Ce compteur est une notification de l’hébergement,
pas un audit réalisé dans cette tâche. Trier les [alertes Dependabot](https://github.com/estran-studio/alacod/security/dependabot)
par composant, exposition réelle et correctif disponible dans une tâche dédiée ; ne pas assimiler
ce compteur à 58 vulnérabilités exploitables de la simulation. Aucune dépendance mise à jour ici.

Pour la revue : comparer à `b917c38f`, lancer `ALACOD_DYLIB=0 make check` (ou les étapes séparées
avec les features habituelles du poste), puis la recette P2P à 2/4 et le restart. Ne pas fusionner
sur la seule base des tests en mémoire.

## Deuxième passe (Claude, 2026-10-14) : revue du lot et compléments

Revue du lot Codex contre `origin/main` puis fusion de `main` (perf-suite, M2-T14a/T16/T17/T19 ;
302 scénarios après fusion, 13 noms courts partagés entre jeux).

Ce que le lot avait juste : la détection p2p était bien absente (`DesyncDetection::Off` par
défaut), le `session_builder` commun couvre natif et wasm, le `BTreeMap` par `GgrsNetId` est
correct (les balles ne se touchent jamais entre elles : `contains_key` est mort, avant comme
après), les scripts Python sont sains. Le test de divergence prouve la configuration GGRS, pas
la chaîne Bevy (`Checksum` → `cell.save` → `log_ggrs_events`) : c'est la raison du point 5.

Ce qui manquait ou cassait :

1. **`main.yaml` n'était pas touché.** Les branches sont fusionnées sans PR : `pr.yaml` n'a plus
   tourné depuis le 2026-10-03, et `main.yaml` gardait `cargo test` nu et `check-forbidden` en
   `continue-on-error`. Les deux appellent désormais `checks.yaml` (workflow réutilisable) :
   mêmes étapes bloquantes, suite de scénarios en quatre jobs par durée estimée, `timeout-minutes`
   et annulation des runs de PR dépassés. Mesuré avant : 26 à 38 min de scénarios en un job,
   ~45 min par push.
2. **Le nightly n'a jamais été vert** : annulé chaque nuit du 07 au 14 octobre à 6 h 00
   (timeout GitHub par défaut), la suite seule prenant 14 337 s sur le runner ; ni p2p, ni vidéos,
   ni notes n'ont jamais été produits. `timeout-minutes: 720`, suite en tranches
   (`NIGHTLY_SHARDS`, défaut 2). À observer une nuit avant d'y croire.
3. **Le smoke p2p ne rollbackait jamais** (`ALACOD_INPUT=neutral` : prédictions toujours
   justes). Nouveau : `ALACOD_INPUT=script:<scénario.ron>` (le `PlayerScript` de chaque joueur
   local, natif), `ALACOD_INPUT_DELAY` (0 sur boucle locale = chaque changement d'input est prédit
   puis resimulé), scénario `p2p_smoke.ron` (4 joueurs, inputs qui changent toutes les 20 à 40
   frames, phases décalées), compteur « frames resimulées après rollback » dans le log de trace et
   dans le résumé du nightly. Mesuré sur deux clients headless par `matchbox_server` local :
   délai 5 → 0 frame resimulée ; délai 0 → 8 et 11 frames resimulées, traces identiques
   (`check-p2p-traces.py --frames 599`).
4. **Un desync ne faisait rien** (journal + télémétrie). `SessionLostEvent { title, message }`
   remplace `GameDisconnectedEvent` : écran « DESYNC » en jeu, `AppExit` code 3
   (`SESSION_LOST_EXIT_CODE`) en headless pour le desync seulement (une déconnexion en headless
   est normale : le pair a fini sa trace). Le nom du pair est résolu par `handles_by_address`
   (avant : premier joueur distant, faux dès trois joueurs).
5. **Exercice de desync** : `ALACOD_DEBUG_DESYNC_AT=<frame>` altère la santé du joueur local sur
   ce client seulement ; les deux pairs doivent sortir en code 3. Voir la vérification ci-dessous.
6. **Régression du lot** : `ALACOD_DUMP_TRACE` ne créait pas le sous-dossier des ids
   `generated/<jeu>/<nom>` (panique). **Régression de fusion** : `scenario-shard-plan.py` (main,
   perf-suite) nommait les tranches par nom court : les 13 homonymes se confondaient (un des deux
   disparaissait du plan, l'autre était refusé comme ambigu par le runner du lot). Corrigé par
   l'id complet, test Python ajouté (`ShardPlanTests`).

### Vérification de la deuxième passe

- `make test_tools` : 14 tests Python verts (dont le nouveau).
- `cargo test -p game` (headless) : 133 tests verts, dont les 3 p2p en mémoire.
- `p2p_smoke` : trace de référence créée (nouveau scénario, aucune trace existante modifiée) ;
  600 frames, quatre joueurs vivants à la frame 300.
- p2p réel à deux clients headless (`matchbox_server` local, `ws://127.0.0.1:3536`) : voir
  le point 3 ci-dessus ; avec `ALACOD_DEBUG_DESYNC_AT=200` sur un client : voir ci-dessous.
- Suite complète en tranches et `make test_crates` : voir ci-dessous.

Non fait ici : allumette/WebRTC via docker (daemon absent), deux machines, wasm, nuit verte
observée, mesure du coût des snapshots (`docs/etat-engine.md`, revue d'architecture).

## Troisième passe (Claude, 2026-10-15) : mesure du rollback et frontière compilée

Demande de William : les points 1 (coût des snapshots) et 2 (frontière simulation/présentation)
de la revue passent dans le flow ; le point 3 (politique de session) reste une dette.

1. **`utils::snapshot_profile`** : chronos aux bornes de `SaveWorld` (checksum, clone),
   `LoadWorld`, `AdvanceWorld` ; inventaire des types rollback enregistrés par
   `RollbackTraceApp` (porteurs × `size_of`, échantillon toutes les 30 sauvegardes). Toujours
   actif dans le runner : métriques `save_ms_avg/max`, `checksum_ms_avg`, `load_ms_avg/max`,
   `advance_ms_avg/max`, `snapshot_bytes_max`, `snapshot_heaviest`, `rollback_ms_worst`
   (restauration + `MAX_PREDICTION_WINDOW` = 12 frames). Tableau `scenario-metrics.py`
   (colonnes sim/save/load/rb×12), `perf-review.py` (hausse > 10 % d'un coût par frame =
   échec ; `--max-rollback-ms`, `PERF_MAX_ROLLBACK_MS`), `tests/budgets.ron` (`max_rollback_ms`
   sur les cinq `bench_*`, au double de la mesure ; `implicit_some`). Jeu :
   `ALACOD_SNAPSHOT_PROFILE=1`, résumé et cinq types les plus lourds dans le journal à la sortie ;
   le nightly l'active sur le smoke p2p et reprend la ligne dans son résumé.
2. **`utils::system_profile`** (`ALACOD_SYSTEM_PROFILE=1`) : temps par `RollbackSystemSet`
   par un système-frontière entre sets voisins (aucune ambiguïté d'ordre), plus une tranche
   `FlowField` ; rapport à la sortie. `RollbackSystemSet::name()` ajouté.
3. **Constat** (p2p_smoke, 4 joueurs qui bougent, 150 entités, 4 zombies) : simulation
   7,06 ms/frame contre 0,11 à l'arrêt ; 98 % dans `update_flow_field_system` (895
   reconstructions pour 1 794 frames : `FlowFieldCache` dérive `Default` → `update_interval`
   0, le `new()` à 30 frames n'est appelé nulle part). Pire rollback 85 ms. Dette **D67** avec
   deux sorties chiffrées ; aucune trace changée ici (une cadence serait un changement de
   gameplay, un bless de toute la suite : à trancher).
4. **Exécuteur monothread des schedules du rollback dans le jeu**
   (`game::core::single_threaded_rollback_schedules`, `finish` de `CoreSetupPlugin`) : A/B sur
   le p2p réel, délai 0 : save 1,28 → 0,47 ms, simulation 7,1 → 5,9 ms, traces identiques.
   `ALACOD_EXECUTOR=multi` rétablit l'ancien.
5. **`scripts/check-sim-boundary.sh`** (`make check_sim_boundary`, `make check`, CI) : graphe
   du crate seul sans crate de présentation + `cargo check --no-default-features` des crates
   SIM (35 s). `utils` rendu ECS seul : `logs.rs`, `web/` déménagés dans `game`
   (`game::logs`, `game::web`), `camera/tod.rs` supprimé (aucun usage), features `keyboard`/
   `bevy_camera` et dépendances de journal retirées de `utils`. 11 crates propres, 5 en dette
   **D66** (listés `PENDING`, affichés sans bloquer ; un crate non classé échoue).

Vérification : `make test_tools` 17 ✅ ; `utils` 3 tests ✅ ; `make check_sim_boundary` ✅ ;
`make format`, `check_forbidden`, `check_rollback_registration` ✅ ; cinq `bench_*` mesurés ;
p2p réel à deux clients (profil, A/B exécuteur) traces identiques ; suite complète en tranches :
303 scénarios verts en 124 s, aucune trace changée (le profil n'entre pas dans l'état) ; tests
unitaires de `utils`, `sim_core`, `game`, `scenario`, `content` verts.
