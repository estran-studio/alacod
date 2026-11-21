# Revue X-6g — verdict

**Verdict : merge après corrections** (aucun P1 ; 2 P2 de lint/doc à corriger, conflit trivial `propositions.md` à résoudre au merge).

Revue en lecture seule, branche `3975cb5e` contre `origin/main` (`30e2603d`, avancé depuis `18d576c4`). Aucun cargo/make lancé : tests, campagne 200/200 et traces **non rejoués** par moi ; je m'appuie sur le journal `x-6g-check-rapide-716cd159.log` (59 binaires, 0 échec, `schedule_order` vert) et la lecture du code. `interieur_reel.mp4` non visionné (images fixes et six PNG vus).

## 1. Défaut inchangé au bit près : OK
Lu ligne à ligne (`crates/map/src/generation/floor.rs` `try_attempt`, `candidates`, `validate_plan`) :
- `start_open` vaut vrai sans `start_exits` (`is_none_or`) → les filtres `open || p.depth > 0` sont neutres ; le `retain(parent == 0)` n'est évalué que si `start_exits.is_some()` (le `&&` court-circuite avant le `any`) ; aucun tirage ajouté, même ordre.
- `avoid` vaut `None` sans `apart` ; `branches` est un `Vec` vide (aucun calcul) ; le prédicat `avoid.is_none_or(..)` est vrai. Le changement de signature `parent_ok(usize, &PlannedRoom)` ne modifie ni l'ordre des candidats ni les tirages.
- `validate_plan` : les trois ajouts sont gardés par `grammar.exact` / `Some(..)` ; aucune condition évaluée pour tous. `distances_with_loops` refactorisé en `adjacency_with_loops` : même construction d'adjacence.
- `grid.rs` : seulement trois champs ajoutés aux gabarits de test. `imp/basic.rs` (+45) : **uniquement un test** (`le_depart_de_basic_…`, dans `mod tests`) ; code de production de `Basic` intact.
- Ce que les 36 scénarios ne couvrent pas : un contenu qui désérialise `FloorGrammar` avec `#[serde(default)]` → `None`/`false` (`skip_serializing_if` : un `Hash`/dump de grammaire reste identique). Je n'ai rien trouvé qui change un ordre de tri.

## 2. Déterminisme : OK
- `BTreeMap`/`BTreeSet`/`Vec` seulement, pas de `HashMap`/`HashSet`, pas de `f32` dans la génération (le `as f64` n'est que dans le test de campagne). RNG : aucun tirage nouveau hors du chemin `start_exits`, où l'ordre est déterministe (candidats déjà ordonnés).
- Échec borné : `plan_floor` → `FloorError::Exhausted{attempts}` (test `branches_impossibles_echouent_apres_n_tentatives`, `exact_refuse_un_type_en_trop`). Pas de boucle non bornée (DFS sur `seen`).
- `Instant::now` ajouté (`crates/map_ldtk/src/generation/mod.rs:1048`) est dans `campagne_interieur`, test `#[ignore]` de `mod tests` : hors simulation ; allowlist 4 → 5 juste (les 5 occurrences sont toutes dans des tests).

## 3. Conformité fiche/décisions : OK sauf P2
- Salle centrale : départ à `snap(-taille/2)` (existant), test unitaire (-80,-80) × 200 graines ; campagne sur les salles réellement générées (centre à ±16 px). Sorties exactes : `start_exit_count` = enfants + boucles touchant le départ, contrôlé dans `validate_plan`, retenté sinon.
- Types exacts : `exact` + `count != required` refusé. Boss dans `required` (le testbed le met) : couvert.
- **« Branche » dans `validate_plan` : juste.** Le contrôle (floor.rs:535-) fait un DFS sur le graphe réel (arbre **et boucles**), départ retiré : plus strict que « sous-arbres de deux enfants différents ». À la pose, `branch_of` ne regarde que l'arbre (heuristique d'orientation) ; une boucle qui relierait les deux branches est rattrapée par `validate_plan` puis retentée. Cohérent avec le doc §129.
- Graine par défaut / explicite / relance : test `la_graine_par_defaut_rejoue_le_meme_interieur_et_une_graine_explicite_un_autre` ; `interieur_graine_3` (map_seed 3). Porte refusée sans clé : `interieur_porte_sans_cle` (`key_refused`, battants `Locked`, clé 0). Enchaînement extérieur → porte → intérieur : `interieur_arrivee/parcours` (`FloorIndex` 0 puis 1 à f275, `PlayerPosition` dans le hall).
- Les 8 scénarios testent ce que leur nom dit (attentes et frames cohérentes avec les entrées ; versions `_duo` = même script + joueur 1 vivant/en place). Écarts mineurs en P3.
- Extension minimale : aucun générateur neuf, pas de code mort (`border_openings`, `branch_of`, `adjacency_with_loops` tous utilisés).

## 4. Rien de 1837 dans l'engine : OK
Types `salon/atelier/cellier/cle/finale`, clé `cle_test`, `depart` : neutres. Mentions de 1837 seulement dans des commentaires de test (`basic.rs`, D227), le rapport, §129 et les dettes (contexte, pas de contenu). Aucun import de présentation ; `content`/`map`/`world` restent SIM ; pas de rollback ajouté.

## 5. Preuve
- Campagne : `cargo test -p map_ldtk --profile headless --no-default-features campagne_interieur -- --ignored --nocapture` (variables `ALACOD_CAMPAGNE_*`, 200 graines par défaut ; jugée sur les salles générées, pas le plan). Rapport : 200/200 (8,12 tentatives en moyenne, 46 au plus), 1 000/1 000. **Non rejouée par moi.**
- Six PNG vus : chaque hall est au centre (cadre jaune) avec exactement 3 portes ouvertes (vertes) et 1 prise murée (rouge) ; salles toutes distinctes, 9-10 salles. Limite : aucune légende (quelle salle est la clé, la finale, le salon…) → l'image seule ne prouve pas `apart`/`exact` (P3).
- `reel_hall.png` : hall du vrai jeu natif, joueur + un ennemi ; cohérent avec la description.

---
## P1
Aucun.

## P2
1. **`docs/conventions.md:9277` affirme un lint qui n'existe pas** : « un type de remplissage qui est aussi requis rend `exact` impossible ». `crates/world/src/floor_grammar.rs:269` ne le fait que dans la branche `FloorLayout::Grid`. En `Graph` (seul cas où `exact` agit), `required: {"combat": 1}` avec `filler: "combat"` passe le lint et échoue au chargement (`Exhausted`, panique « grammaire d'étage impossible »). *Correction* : dans `problems()` (près de `floor_grammar.rs:298`), `if self.exact && self.required.contains_key(&self.filler) { out.push(...) }` + test dans `sorties_et_branches_incoherentes_sont_refusees` ; ou retirer la phrase du doc.
2. **Combinaison impossible non lintée : `apart` avec `start_exits: Some(1)`** (une seule branche possible). `floor_grammar.rs:298-325` ne la voit pas ; le test `branches_impossibles_echouent_apres_n_tentatives` (floor.rs) prouve qu'elle n'échoue qu'à l'exécution après `max_attempts`. *Correction* : `if self.apart.is_some() && self.start_exits.is_some_and(|n| n < 2)` → problème « apart exige au moins 2 sorties du départ » (+ test). Même logique : `start_exits` > prises du gabarit est déjà lint (`lint.rs` : bien).

## P3
1. `crates/scenario/tests/floors.rs:~360` : le test passe `Some(123456)` explicitement à `map_seed` — il prouve que la **relance rejoue la graine du scénario**, pas que `entry.default_seed` du manifeste du testbed vaut 123456 ni qu'il est lu. Ajouter une assertion sur `entry.default_seed` (ou lancer sans `map_seed`). De plus `if map_ldtk::RENDER_ENABLED { return; }` fait passer le test sans rien vérifier en build rendu : le dire dans un `eprintln!`/`#[cfg]`.
2. `floor.rs:535` (`validate_plan`/`apart`) et `floor.rs` `attempt.placed.iter().position(..)` ne regardent que **la première** salle de chaque type ; sans `exact`, un doublon d'un type `apart` n'est pas contrôlé. Lint : exiger `exact` avec `apart` (ou contrôler toutes les paires).
3. `crates/content/src/registry.rs:1468` `border_openings` : une case ouverte au coin compte dans deux côtés (double comptage possible) ; le lint `start_exits` pourrait accepter un gabarit à une prise de moins. Cas improbable en LDtk ; à noter dans le doc de la fonction ou compter les coins une fois.
4. `crates/map/src/generation/imp/basic.rs` (test +45) : épingle un défaut connu (D227) — il cassera le jour où `Basic` centre son départ ; le marquer « à réécrire avec D227 » (le commentaire le dit presque, pas explicitement).
5. `tests/scenarios/interieur_arrivee.ron` : commentaire « neuf à dix salles s'ajoutent » vs `RoomCount(min: 9, max: Some(12))` qui inclut l'extérieur ; et « les autres [portes] sont ouvertes » alors que seules `MapDoor` 0, 1, 3 sont attendues. Aligner commentaire/attentes.
6. Les PNG de six graines n'ont pas de légende (types par salle) : régénérer avec les étiquettes `cle`/`finale`/`salon`… pour que l'image prouve `apart`.
7. Merge : `propositions.md` en conflit avec `origin/main` (ajouts en fin de fichier des deux côtés, résolution = garder les deux) ; `docs/taches/plan-1837-cli.md:222` (« en cours ») et la fiche § 6 sont à mettre à jour par le coordinateur (le fichier a bougé sur main).
