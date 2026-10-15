# Rapport — perf-navigation (flow field et navigation des bots sur grilles denses)

Branche `perf-navigation` depuis `origin/main` (8f3fd270), session cloud : 4 cœurs, 15 Go, profil
`headless`, Bevy en bibliothèque partagée. Une seule commande lourde à la fois.

## Fait

1. **`build_flow_field`** (`crates/game/src/character/enemy/ai/navigation.rs`) :
   - la grille de cases bloquées / cassables est remplie en **parcourant** `wall_cells` et
     `blocked_cells` (au lieu d'une recherche d'arbre par case et par type d'obstacle) ;
   - Dijkstra sur tableaux `Vec` indexés par case (coût, case suivante, propriétaire) ; tas trié
     par (coût, `GridPos`), mêmes voisins dans le même ordre, mêmes départages ; la vérification
     de coin utilise la grille de dégagement ;
   - les trois `BTreeMap` de `FlowField` sont reconstruites d'un bloc depuis une suite triée
     (x puis y) : **même contenu, même `Hash`** ; `FlowField`, `SharedFlowField` et le rollback ne
     changent pas de forme ;
   - cible hors de la boîte de recherche (cas sans murs) : repli sur le calcul historique, gardé
     (`build_flow_field_legacy`, aussi référence des tests).
2. **Bots** (`crates/bots/src/navigation.rs`) : `blocked` et `reachable` en `CellGrid` (grille
   dense de booléens), `build_field` en tableaux denses, sortie reconstruite en `BTreeMap` triée.
   Le profil callgrind de `throne_quad` (400 frames) montrait `BotNavigation::build_field` à
   **44 %** des instructions (`step_free` → `BTreeSet::contains`) et `update_flow_field_system` à
   13,6 % ; c'est le bot, pas le flow field zombie, qui dominait les `throne_*`.
3. Tests : `optimized_geometry_matches_legacy_flow_fields` (existant, 24 cartes × 4 profils × 2
   tailles × 4/8 directions, champs et hash identiques) ; nouveau `dense_field_matches_legacy_field`
   (bots, 12 géométries, coûts/directions/propriétaires identiques à `build_field_legacy`,
   `#[cfg(test)]`).

## Mesures (fps simulés, un scénario à la fois au calme, même machine)

| Scénario | avant (`origin/main`) | flow field seul | flow field + bots |
|---|---|---|---|
| `bench_floor` | 96,1 | 114,7 | **121,0** |
| `throne_quad` | 37,6 | 45,5 | **102,2** |
| `throne_three_floors` | 79,2 | 96,8 | **131,9** |
| `clone_quad` | 178,5 | 183,7 | **180,1** (bruit) |

Suite complète sur la fusion avec `origin/main` (b527bdb1) : **311 scénarios, 829 s en un
processus** ; `make test_scenarios_par` (4 tranches) : **217 s**. Le temps « avant » de la suite
n'a pas été remesuré sur cette base (15 min de plus) : le dernier chiffre connu est celui de
perf-suite-scenarios (243 scénarios, 1 038 s et 285 s), donc non comparable à nombre égal.

## Vérifié

- **Aucune trace n'a bougé** : suite complète sans `BLESS`, `test result: ok`, 311 scénarios, sur
  la fusion avec `origin/main` b527bdb1 (qui n'avait pas bougé à la fin) ; aussi verte en 4
  tranches (`tranche 0..3/4 : ok`).
- `fuzz_inputs` `ALACOD_FUZZ=0:8`, normal puis `ALACOD_FUZZ_SPARSE=1` : 2 × `ok`, aucune
  divergence du synctest (carte du jeu, zombies).
- `make test_crates` : 709 réussis, 0 échec, 10 ignorés. `make lint` : aucune erreur (zombies,
  testbed, throne, gungeon). `make gen` ×4 : exit 0, aucun fichier modifié. `cargo fmt --all --
  --check` : propre. `check-rollback-registration.sh` : OK. `check-forbidden.sh` : 9 occurrences
  (HashSet, `Instant::now`, `rand::`), toutes dans des fichiers que cette tâche ne touche pas.

## Non vérifié / hors de portée ici

- **p2p à deux clients et restart en ligne** (Docker) : non rejoués ; `FlowFieldCache` est
  rollback, mais son contenu et son `Hash` sont inchangés (tests d'égalité ci-dessus). À rejouer
  par l'orchestrateur.
- `ALACOD_BENCH_STRICT=1` : non lancé ; les planchers de `tests/budgets.ron` n'ont pas été
  relevés (les fps ont augmenté, aucun plancher n'est franchi par le bas).
- Bot : l'égalité dense/historique est testée sur 12 géométries synthétiques et par les traces,
  pas sur les cartes complètes.

## Dettes / pistes

- Il reste 13 % de `update_flow_field_system` (copie de `wall_cells`/`blocked_cells` pour
  comparer à chaque pas, reconstruction des `BTreeMap` de sortie) ; une sortie dense demanderait
  de changer `FlowField` et son `Hash` (hors périmètre, aucun gain nécessaire).
- Un profil callgrind après changement n'a pas été refait (fps mesurés à la place).
- Atteignabilité (M2) : sans objet, tâche d'outillage.
