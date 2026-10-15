# perf-navigation — flow field et navigation des bots en grille dense (outillage et simulation, sans changement d'octet)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Flow Field - Implémentation Actuelle »,
« Déterminisme »), `docs/conventions.md` §5, §6 et §10 (preuve d'une trace), le rapport
`docs/taches/rapports/perf-suite-scenarios.md` (§ 5, « Pistes non appliquées »),
`crates/game/src/character/enemy/ai/navigation.rs` et `crates/bots/src/navigation.rs`.
Branche `perf-navigation` depuis `origin/main`, **une fois perf-suite-scenarios et m1-revue-humaine mergées** (toutes deux touchent `navigation.rs`). Estimation : 3 j. Pas de
section de conventions réservée : compléter « Vitesse de la suite » de §5.

## Contexte

Après perf-suite-scenarios, le coût restant des scénarios lourds est dans la navigation (callgrind,
`bench_floor`) :
- `update_flow_field_system` = **42 %** des instructions, dont `is_blocked` = 22 %. Le Dijkstra et
  ses voisins sont stockés en `BTreeMap`/`BTreeSet` (`FlowField`, `wall_distance`, `blocked_cells`) ;
- `bots::navigation::update_for` / `build_field` = **14 %**, davantage pour `throne_*` et `clone_quad`.

Les `throne_*` font la moitié du temps des dix scénarios les plus longs (`throne_quad` à 28 fps).

## À faire

1. **Mesurer** `bench_floor`, `throne_quad`, `throne_three_floors`, `clone_quad` avant (fps, et un
   profil callgrind ou chronomètres).
2. **Structures denses** pour les calculs : une grille `Vec` indexée par case sur la boîte englobante
   (les `BTreeMap`/`BTreeSet` restent permis pour l'état rollback si les changer déplace le hash).
   - Dijkstra : tas trié par (coût, case) inchangé ; mêmes voisins dans le même ordre, mêmes
     départages. La sortie (directions, coûts, `owners`) doit être **identique case par case**.
   - `is_blocked`, `wall_distance` : tableau précalculé au lieu de recherches dans des arbres.
   - Bots : même chose pour `build_field`.
3. **Hash et rollback** : si une structure rollback change de forme, son `Hash` doit produire le
   **même flux d'octets** qu'avant (modèle : `SharedFlowField` de perf-suite-scenarios, test
   d'égalité des hash). Sinon garder la structure rollback telle quelle et ne densifier que le calcul.
4. Test unitaire : sur plusieurs cartes (zombies, throne, gungeon, testbed), l'ancien et le nouveau
   calcul donnent le même champ (garder l'ancien en `#[cfg(test)]` comme référence).
5. Rapport : fps avant/après des quatre scénarios, temps de la suite en 1 processus et en 4 tranches.

## Règles

**Aucune trace ne doit bouger** : une trace qui bouge est un défaut, jamais un bless. Pas de `f32`,
pas de `HashMap`/`HashSet`, ordre de départage par `GridPos` comme aujourd'hui. Ne pas toucher à la
distance de vérification du synctest (décision de l'orchestrateur, 2026-10-14 : elle reste à 2).
`fuzz_inputs` (`ALACOD_FUZZ=0:8`) sur deux scénarios à zombies. `cargo fmt --all -- --check`.

## Hors périmètre

Changer le comportement des zombies ou des bots, le pas de mise à jour du flow field (toutes les 30
frames), la taille des cases.

## Session cloud

Tâche prévue pour une session Claude Code cloud : ignorer les consignes `orca` du README (§1) ; pas
de Docker : le p2p (le flow field est rollback) est rejoué par l'orchestrateur, le dire au rapport.
Pousser la branche régulièrement (commits WIP acceptés ; le conteneur redémarre environ toutes les deux
heures : lancer la suite au premier plan). Finir par un commit `LIVRÉ perf-navigation` qui contient le
rapport `docs/taches/rapports/perf-navigation.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ;
`make test_scenarios_par` vert.
