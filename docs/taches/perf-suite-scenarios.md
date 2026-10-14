# perf-suite-scenarios — diviser par deux le temps de la suite de scénarios (outillage)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Headless, trace d'état et déterminisme »,
« Scénarios »), `docs/conventions.md` §5 (commandes make), §6 (CI lente) et §10 (preuve d'une
trace), `crates/scenario/src/runner.rs`, `crates/scenario/tests/scenarios.rs`,
`scripts/test-scenarios-sharded.sh`. Branche `perf-suite-scenarios` depuis `origin/main`.
Estimation : 3 j. Pas de section de conventions réservée : compléter §5 et §6.

## Contexte

La suite (243 scénarios) est le goulot de tout le projet : chaque vérification d'orchestrateur la
rejoue en entier, et chaque tâche la rejoue avant de livrer.
- Sur `orca` (8 cœurs, chargée) : ~3 900 à 5 000 s en 4 tranches.
- En cloud : ~2 800 s en un seul processus.

Mesure de la suite de M2-T7 (`target/metrics`, 4 tranches parallèles sur machine chargée) :
14 060 s de simulation cumulée. Les 10 scénarios les plus longs en font 31 % :
- `throne_three_floors` (5 000 frames, 6,4 fps) ;
- `throne_quad`, `throne_solo`, `clone_quad`, `clone_duo`, `throne_progression` ;
- `bench_floor`, `avant_poste_demo`, `clone_solo`, `throne_softlock_recul`.

Surtout, **même un scénario presque vide est lent** : `idle` tourne à ~10 fps (1 500 frames), et
la plupart des scénarios sont entre 8 et 13 fps. Le coût fixe par frame domine, pas la
simulation du contenu.

## À faire

1. **Mesurer d'abord**, sur une machine au calme (la session cloud) : `idle`, `throne_solo` et
   `bench_floor` seuls.
   - Profil par frame avec `perf` ou `cargo flamegraph` si disponibles, sinon des chronomètres par
     étape dans le runner.
   - Part de chaque poste : simulation, re-simulation du synctest (distance de vérification),
     snapshots et restauration rollback, checksum, trace d'état (hash à chaque frame, écriture),
     vérification des invariants, attentes, `FeedbackLog` et événements, journalisation.
   - Tableau au rapport, **avant toute optimisation**.
2. **Optimiser les postes qui dominent**, sans changer ce que la suite vérifie. Pistes à confirmer
   par la mesure :
   - hash d'état calculé une seule fois par frame et partagé ;
   - trace écrite en tampon plutôt que ligne à ligne ;
   - invariants qui parcourent toutes les entités à chaque frame ;
   - niveau de log en test ;
   - systèmes de présentation encore actifs en headless ;
   - allocations par frame dans le runner.
3. **Interdits sans accord de l'orchestrateur** :
   - réduire la distance de vérification du synctest ;
   - sauter des frames de vérification de trace ;
   - raccourcir un scénario ou retirer une attente ;
   - changer le profil de compilation `headless` de façon qui change la simulation.

   Ces pistes peuvent être chiffrées au rapport, pas appliquées.
4. **Ordonnancement des tranches** : `scripts/test-scenarios-sharded.sh` répartit un scénario sur N
   à tour de rôle. Le répartir plutôt par durée mesurée (les métriques de la dernière suite, ou
   `frames` en repli), pour que les N tranches finissent ensemble.
5. Résultat au rapport : temps de la suite complète avant/après sur la même machine, en un
   processus et en 4 tranches ; fps de `idle`, `throne_solo` et `bench_floor` avant/après.

## Règles

**Aucune trace ne doit bouger** : l'outillage de test ne change pas la simulation. Toute trace qui
bouge est un défaut, pas un bless. Les budgets (`tests/budgets.ron`) peuvent être relevés à la fin,
avec les mesures. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Optimisation de la simulation elle-même (flow field, collisions) au-delà de ce que le profil
désigne comme dominant dans le coût fixe ; perf d'un grand étage en jeu (critère de vague 2, à
part).

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ toutes
les deux heures et tue les processus d'arrière-plan : lancer les mesures et la suite au premier
plan) et finir par un commit `LIVRÉ perf-suite-scenarios` qui contient le rapport. La machine cloud
est plus calme que `orca` : c'est un avantage pour mesurer, à dire au rapport avec le nombre de
cœurs.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`) ; `make
test_scenarios_par` vert ; rapport `docs/taches/rapports/perf-suite-scenarios.md`.
