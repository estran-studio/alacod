# État de l’engine et critères de sortie

Revue du 2026-10-14, base `b917c38f`. Cette page synthétise l’état connu ; les preuves détaillées
restent dans les rapports de tâches. Les tableaux de `plan-engine.md` décrivent principalement
l’écart initial du 2026-09-28 : ne pas les lire comme l’inventaire actuel.

## Avancement connu

| Partie | Livré | Validation disponible | Prochaine preuve requise |
|---|---|---|---|
| Socle | Fixed-point, rollback, RNG par flux, événements de frame, traces, invariants, lint, scénarios, bots | Suite de scénarios et campagnes documentées | Maintenir ces preuves sur chaque état fusionné |
| M0 / zombies | Boucle de vagues, achats, portes/fenêtres, perks, à terre/réanimation, restart | Campagnes de bots, traces P2P à 2 et 4 clients | Runs humaines sur plusieurs machines et validation du ressenti |
| M1 / throne | Trois cavernes, destruction, armes/munitions, mutations, portail, boss, HUD | Digest : 198/200 runs à 2 bots, 200/200 à 4, sans soft-lock ni desync sur la révision mesurée | Revue humaine finale, équilibrage, nouvelles campagnes après changements de contenu |
| M2 / gungeon | Salles et grammaires, patterns, blanks, esquives, effets, synergies, boss, props, formations, bot explorateur | Scénarios dédiés ; campagne explorateur sur une version antérieure du boss | Butin/coffres/boutique, contenu accessible, profil/hub/UI selon fiches, puis run intégrée P2P |
| M3–M5 | M3 `isaac` en fiches, contrats en cours (T0) ; M4 (1837) attend la bible ; plan et contrats partiels réutilisables | Pas de validation de clone complet | Terminer les critères de M2 avant d’étendre fortement le vocabulaire |

Les réussites de bots sont des résultats fonctionnels sur une configuration donnée ; elles ne
mesurent ni le plaisir, ni la difficulté pour un humain. « Zéro desync » en synctest n'équivaut
pas à une validation de traversée NAT ou de réseau dégradé.

## Ce que la CI prouvait réellement (mesuré le 2026-10-14, `gh run list`)

| Étage | Constat | Conséquence |
|---|---|---|
| `pr.yaml` | dernier run le 2026-10-03 : les branches sont fusionnées par merge direct, sans PR | les contrôles « bloquants » d'une PR ne s'exécutaient jamais |
| `main.yaml` (push) | `cargo test` nu + `check-forbidden` en `continue-on-error` ; ~45 min par push, dont 26 à 38 min de suite de scénarios en un seul job | l'étage « rapide » du plan (§9.9, < 5 min) n'existait pas ; les garde-fous ne bloquaient pas |
| `nightly.yaml` | **annulé chaque nuit du 07 au 14 octobre à 6 h 00 pile** (timeout GitHub par défaut) pendant l'étape B : la suite seule prenait 14 337 s sur le runner | aucun p2p réel, aucune vidéo, aucune note de revue produits par la CI depuis la mise en place (une seule note, écrite à la main le 2026-10-02) |
| Smoke p2p nocturne | `ALACOD_INPUT=neutral` : prédictions toujours justes, aucun rollback resimulé sur le transport réel | la seule preuve de rollback p2p était le test en mémoire |
| `DesyncDetected` | journal + télémétrie seulement : la partie continuait, chaque pair dans son propre état | un desync en partie réelle n'arrêtait rien, et un smoke headless pouvait finir « vert » |

Corrigé sur cette branche : `checks.yaml` partagé par PR et main (format, outils, garde-fous,
workspace, lint ; suite de scénarios en quatre tranches par durée estimée, jobs limités à 60 min) ;
nightly à 720 min avec la suite en tranches ; smoke p2p à inputs scriptés (`p2p_smoke.ron`) et
compteur de frames resimulées dans le log de trace ; `DesyncDetected` → écran « DESYNC » et code
de sortie 3 en headless. Reste à **observer** une nuit verte avant de considérer N5/N7 couverts.

## Revue d'architecture (2026-10-14)

Direction : juste. Déterminisme vérifiable, scénarios comme tests, clones comme intégration et
contenu en RON sont exactement ce qu'il faut pour un engine de roguelike coop en rollback ; peu
de projets ont ce filet. Les trois risques qui pèsent le plus sur « l'ultime engine p2p » :

1. **Le coût du rollback est maintenant mesuré, et ce n'est pas les snapshots.** Décision du
   2026-10-15 (William) : points 1 et 2 passent dans le flow ; le point 3 reste une dette.
   `utils::snapshot_profile` chronomètre `SaveWorld`/`LoadWorld`/`AdvanceWorld` dans chaque
   scénario (métriques `save_ms_avg`, `load_ms_avg`, `advance_ms_avg`, `rollback_ms_worst` =
   restauration + 12 frames resimulées ; plafonds `max_rollback_ms` dans `tests/budgets.ron`,
   régression de plus de 10 % refusée par `make perf_review`) et dans le jeu
   (`ALACOD_SNAPSHOT_PROFILE=1`, repris par le résumé du nightly sur le p2p réel). Mesures du
   2026-10-15, poste M4, ms par frame simulée :

   | scénario | entités | save | load | simulation | pire rollback (×12) | snapshot |
   |---|---|---|---|---|---|---|
   | bench_bullets_room (535 balles) | 570 | 0,60 | 0,46 | 1,05 | 13,0 | ~198 KiB |
   | bench_horde (61 zombies, 3 joueurs) | 272 | 0,46 | 0,45 | 2,91 | 35,3 | ~88 KiB |
   | four_players_idle | 141 | 0,20 | 0,17 | 0,11 | 1,5 | ~29 KiB |
   | **p2p_smoke (4 joueurs qui bougent, 4 zombies)** | 150 | 0,51 | 0,38 | **7,06** | **85,1** | ~30 KiB |

   Les snapshots coûtent ~0,5 ms (dont 0,37 de checksum) même à 570 entités : élaguer
   `Weapon`/`EnemyAiConfig`/`GgrsNetId` (une `String` clonée par entité) est secondaire. Ce qui
   compte, c'est la frame de simulation quand des **joueurs bougent** : 64× plus chère qu'à
   l'arrêt, et `ALACOD_SYSTEM_PROFILE=1` (temps par `RollbackSystemSet`) l'attribue à 98 % au
   champ de flux, reconstruit à chaque changement de case d'un joueur parce que
   `FlowFieldCache::update_interval` vaut 0 par `Default` (dette **D70**, deux sorties
   chiffrées). En ligne, un rollback de 12 frames coûte cinq ticks avant toute horde. Autre
   constat du p2p réel : l'exécuteur multithread de Bevy sur les schedules du rollback coûtait
   2,7× sur la sauvegarde (1,28 → 0,47 ms) ; `game::core::single_threaded_rollback_schedules`
   les passe en monothread dans le jeu comme dans les scénarios (`ALACOD_EXECUTOR=multi` pour
   comparer).
2. **La frontière simulation/présentation est prouvée par la compilation pour 13 crates,
   en dette pour 2.** `make check_sim_boundary` (`scripts/check-sim-boundary.sh`, `make check`
   et CI) : `cargo tree` du crate seul sans crate de présentation, puis `cargo check
   --no-default-features` ECS seul. Propres : `sim_core`, `stats`, `effects`, `behaviors`,
   `world`, `run`, `items`, `meta`, `content`, `bevy_fixed`, `utils`, **`combat`** et **`map`**
   (étape 1 de D69, 2026-10-18 : `combat` ne tire plus que 147 crates au lieu de 505 ;
   `animation`, qui ne porte plus que de la présentation, est classé `APP`). Dette **D69**,
   affichée sans bloquer : `map_ldtk` (`bevy_ecs_ldtk`, rendu) et `game` (≈ 47 k lignes,
   simulation et UI/caméra/audio/réseau dans le même crate). Un crate nouveau doit être classé.
   Sort de D69 : `game_sim`/`game_app` (étape 2, chiffrée dans `docs/taches/dettes.md`).
3. **La partie en ligne n'a pas de politique de session** (dette, pas prioritaire : décision du
   2026-10-15). Une déconnexion termine la partie
   pour tout le monde (overlay et sortie du processus) alors que GGRS continue sans le pair
   parti ; pas de spectateur (`PlayerType::Spectator` n'est jamais ajouté au builder), pas de
   réglage de `disconnect_timeout`, flux allumette bloquant sur le thread principal avec
   `process::exit`. Pour une coop à 4, décider : continuer à 3, ou fin de partie ; le code fait
   aujourd'hui le choix le plus brutal par défaut.

Risques de second rang : la suite de scénarios est le goulot de tout (1 à 2 h par vérification
sur orca, 4 h sur le runner) et son coût fixe par frame domine (synctest ×2, checksum, trace,
invariants) ; la boucle gungeon n'était rejouée qu'en synctest (D59, D62 : objets sans source) ; depuis
m2-p2p-run-complete (2026-10-16) elle l'est en p2p réel sur une machine : 2 clients de l'entrée à la victoire (f7138) puis
restart, 4 clients jusqu'après la transition d'étage (4 500 frames, sans fin de run), transition d'étage avec rollbacks réels,
réseau dégradé 120 ms / 20 ms / 1 %, le tout **sans le hub** pour les runs à bots (D85) ; pas encore entre deux machines ; les dettes de jouabilité zombies tranchées le 10-14
(D28, D29) changeront les traces `test_map` ; le `rust-toolchain` nightly n'est pas épinglé par
date (stabilité des hash entre machines, §5 du contrat de sortie).

## Contrat de sortie d’un jalon

Chaque clôture fournit un court dossier de preuve avec commit, jeu, commandes, graines, machines,
plateformes, options de build et limites. Un point non exécuté reste « non vérifié ».

1. **Boucle accessible.** Une partie lancée normalement depuis le manifeste rencontre les
   mécaniques annoncées. Les overrides de scénarios restent des tests complémentaires. Un objet
   seulement défini en RON ne compte pas comme contenu jouable. Le lint et les campagnes de bots
   couvrent atteignabilité, transitions et absence de soft-lock.
2. **Régression.** Tests du workspace, scénarios sans bless, lint et contrôles statiques passent.
   Un bless est lié à un changement voulu et à sa preuve (§10 des conventions). Le format historique
   d’un checksum peut évoluer avec une migration explicitement revue ; ne pas ajouter de conventions
   de hash complexes uniquement pour éviter un changement de référence légitime.
3. **P2P intégré.** Au moins une run complète à deux, puis quatre joueurs lorsque le clone les
   supporte, avec combat, choix/objets, boss, transitions et restart. Inclure deux machines distinctes.
   Garder des inputs actifs : le smoke de 600 frames à inputs neutres ne valide pas cette boucle.
4. **Réseau dégradé.** Mesurer et documenter les conditions, puis tester retard, jitter et pertes.
   Matrice de départ proposée : RTT 60/120/200 ms, jitter 0/20 ms, pertes 0/1/3 %, séparément puis
   quelques combinaisons représentatives. Mesurer stalls, profondeur/fréquence des rollbacks et
   récupération ; les conditions hors cible peuvent ralentir, mais ne doivent pas masquer un desync.
   Les tests en mémoire couvrent le protocole et la resimulation ; ils ne remplacent pas WebRTC/NAT.
   *Mesuré le 2026-10-17* (m2-p2p-run-complete §8, `dette-d86-lobby-en-ligne` ; 2 clients, gungeon graine 10, 4 500 frames, une exécution par condition,
   une machine partagée ; rollback = restauration d'état (`LoadWorld`), profondeur = frames resimulées par rollback (≥ 1) ; le jeu journalise désormais « rollbacks : n=… » à la sortie) :

   | Condition (charge au lancement) | Rollbacks / 100 frames (clients 0, 1) | Profondeur moyenne | médiane / p90 / max | Profondeur 1 |
   |---|---|---|---|---|
   | Boucle locale, sans stress (2,08) | 17,9 ; 18,3 | 1,35 ; 1,32 | 1 / 2 / 11 ; 8 | 80 % |
   | netem RTT 120 ms ± 20 ms, perte 1 % (2,79) | 18,7 ; 20,4 | 4,67 ; 4,99 | 4 / 9–10 / 11 | 9–13 % |
   | Stress SIGSTOP du client 1 (2,10) | 27,7 ; 2,4 | 3,81 ; 1,11 | 3 / 8 / 11 (client 0) | 15 % ; 90 % |

   Profondeur maximale mesurée 11, jamais 12 (fenêtre de prédiction 12). Profondeur = frames resimulées = dernière frame avancée − frame restaurée (corrigée à la revue 2 : une première version la déduisait
   des clés de trace et la surestimait). Pas encore mesurés : matrice RTT 60/200 ms et pertes 3 % (D87), répétitions (une seule exécution par condition ; deux exécutions d'une même condition ont donné des
   distributions différentes), vrai WAN.
5. **Plateformes.** Pour chaque combinaison annoncée (natif, navigateur, architectures différentes),
   comparer une même partie et ses états confirmés. Le fixed-point seul ne prouve pas la compatibilité
   des builds ni la stabilité des fonctions de hash entre plateformes/toolchains.
6. **Performance.** Sur une machine cible nommée et au calme, publier simulation, sauvegarde et
   restauration des snapshots, mémoire et coût des ticks avec resimulation. À 60 Hz, un tick dispose
   de 16,7 ms ; réserver une marge pour rejouer les frames prédites. Mesurer percentiles et pics sur
   horde, bullet hell, étage complet, explosions en chaîne et transitions. Les valeurs du harness
   headless/synctest sont des indicateurs de régression, pas une mesure directe du budget de production.
7. **Humain.** Jouer en coop, vérifier esquive/visée, lisibilité des dangers et du HUD, progression et
   redémarrage. Consigner les problèmes observés avant de déclarer le jalon terminé.

## Ordre recommandé après cette branche

1. Revue et intégration du durcissement P2P/CI ; exécuter le transport réel sur le poste
   d’orchestration (Docker opérationnel) puis entre machines.
2. D59 : choix explicite de la séquence d’étages dans le manifeste et grammaire par défaut de
   gungeon, avec transition en P2P. Garder la décision et la fiche existantes comme autorité.
3. M2-T5/T6/T14 : brancher les sources de butin et l’économie ; fermer D62, puis refaire une
   campagne sur le boss actuel et la boucle ordinaire. Terminer les autres livrables de M2.
4. Calibrer une référence de performance reproductible, profiler le coût dominant et corriger
   selon les mesures. Ne pas relever artificiellement les seuils de `tests/budgets.ron`.
5. Réduire progressivement les dépendances de présentation dans les modules de simulation :
   `combat` et `map` sont maintenant des crates `SIM` (D69, étape 1) ; reste `game` (simulation et
   présentation dans le même crate) et `map_ldtk` (D69, étape 2). Le headless actuel évite
   fenêtre/GPU à l’exécution ; la compilation ECS seule est prouvée pour 13 crates, pas encore pour
   `game`, donc pas encore un portage console.
6. Faire assembler un petit jeu par une autre personne : une arme, un ennemi, un objet et une salle.
   Consigner le Rust nécessaire, les erreurs de contenu et les difficultés rencontrées.

## Outils de consolidation

- `make check` : format, tests des outils, garde-fous, workspace headless, scénarios, lint.
- `make test_tools` : tests rapides sans compilation de l’engine.
- Les scénarios générés ont pour id `generated/<jeu>/<nom>` : pas d’écrasement de métriques
  entre jeux. Les filtres inconnus/ambigus et les tranches invalides échouent explicitement.
  Le merge des métriques shardées refuse les parts absentes ou les collisions.
- `scripts/check-forbidden.sh` : strict par défaut, couvre aussi les imports groupés/Bevy et les
  nouveaux crates. Les exceptions sont des lignes exactes avec compte et justification dans
  `scripts/determinism-allowlist.json`. Réduire/retirer une entrée périmée ; ne pas régénérer cette
  liste pour faire passer une nouvelle violation. Ce contrôle lexical ne remplace pas une revue.
- `python3 scripts/check-p2p-traces.py --frames 599 client0.trace client1.trace` : traces complètes,
  contiguës et identiques d’un smoke démarré à froid avec `ALACOD_EXIT_AT_FRAME=600`.

Pour une comparaison de performance, copier la référence **avant** de relancer les mesures :

```bash
cp target/metrics/latest.json /chemin/persistant/reference-machine.json
# Même machine, options, instrumentation, scénarios et nombre de frames ; sans autres builds.
ALACOD_DYLIB=0 make test_scenarios SCENARIO=bench_bullets,bench_bullets_room,bench_horde,bench_cave,bench_floor
make perf_review PERF_BASELINE=/chemin/persistant/reference-machine.json
```

`perf_review` refuse mesures manquantes/non finies, charges de frames/joueurs différentes et
régressions de plus de 10 %. `PERF_MAX_REGRESSION`, `PERF_SCENARIOS`, `PERF_CURRENT` et
`PERF_MIN_SIM_FPS` permettent un contrat explicite. L’objectif absolu est optionnel et doit être
calibré ; les budgets historiques et la mesure d’un tick de production restent distincts.
Ne pas comparer des mesures issues d’une suite shardée à une suite séquentielle.
