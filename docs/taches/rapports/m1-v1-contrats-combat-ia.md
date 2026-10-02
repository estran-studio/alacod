# Rapport — T1.0a : contrats de combat et d'IA

**SHA de la tête de branche vérifiée : `bd65d34847806787ce6f28f8430ad7b868812ecd`.**
Fiche : [T1.0a-contrats-combat-ia](../T1.0a-contrats-combat-ia.md).
Branche : `m1-v1-contrats-combat-ia` ; agent : Codex ; date : 2026-10-02.
Base intégrée : `main` / `origin/main` à `f170a92fc1b979e69027a54dffcff5394c3c7e18`.
Le SHA ci-dessus désigne le code testé, avant le commit de ce rapport. Le SHA de
livraison désigne ensuite ce commit documentaire, sans changement de code.

## Fait

- Déplacement de `game::weapons::{mod, melee}` dans `combat::weapons`, avec
  `pub use combat::weapons` dans `game`. Premier commit `7b1eb46`, puis preuve
  des 61 scénarios **avant** l'ajout des squelettes. Aucun bless.
- Présentation conservée dans `game` : HUD dans `ui/weapon_hud.rs`, attachement,
  orientation des sprites et effets de slash dans `ui/weapon_visuals.rs`.
  `WeaponPresentationPlugin` est monté par le plugin de présentation.
- Les armes dépendaient aussi des acteurs, des entrées de session, des colliders,
  de la grille de collision et des attentes du générateur. Pour éviter le cycle,
  leurs données et fonctions géométriques existantes passent dans
  `combat::{actors, collider, collision_grid}` ; `game` les réexporte. Les attentes
  typées passent dans `combat::weapons::expectations` et sont réexportées par
  `game::replay`. Les systèmes de personnage et de session restent dans `game`.
  Les algorithmes, composants et règles de hash déplacés sont conservés.
- `Interactable` / `InteractionType` passent dans `sim_core::interaction`, avec
  réexports dans `game`. Le spawn commun de pickup reste dans `combat` ; le
  chargement des machines et les systèmes d'achat restent dans `map_ldtk` / `game`.
- `combat` reçoit `ProjectileModifier`, `Pattern`, `StatusDef`, `StatusEntry`,
  `Statuses` et `CombatPlugin`. Le nouveau crate `behaviors` contient `Behavior`,
  `Perception`, `PerceptionConfig`, `Targeting`, `BehaviorState`, `BehaviorsPlugin`.
  `effects` reçoit `On`, `GaugeThreshold`, `Condition`, `Effect`, `EffectsPlugin` ;
  l'`Action` existante est réutilisée et gagne les dérivés `Eq` / `Hash`.
- Dérivés de contrat, nombres `Fixed` sérialisés en chaînes, durées en frames,
  références de profils/patterns/armes en identifiants `String`. `PerceptionConfig`
  porte `needs_light`. `Effect` expose les champs RON `on`, `if`, `do`.
- **45 kinds dans 8 catégories**, catégories snake_case et noms de variantes
  PascalCase. Les trois plugins sont montés dans `game::core`. `Statuses` et
  `BehaviorState` sont enregistrés en rollback et trace via l'API effective
  `RollbackTraceApp::rollback_and_trace::<T>()` ; aucun composant n'est posé et
  aucun système M1 n'est ajouté.
- **13 nouveaux tests** : 9 allers-retours RON d'enums, 1 contrôle des champs RON
  d'`Effect`, 3 tests de kinds par plugin (et de trace des deux nouveaux états).
  Les tests couvrent toutes les variantes initiales, dont un pattern imbriqué.
- Documentation des propriétaires, unités, choix d'identifiants et sets futurs
  dans les crates et `docs/conventions.md` §4.3 ; table des crates et statut
  `en cours (branche)` dans `docs/taches.md`. Aucun changement de `Kinds`,
  `KindRegistry`, `RollbackSystemSet` ou du journal §10.
- `main` a été intégré **dans la branche de tâche** (`922236a`) avant les contrats.
  Le commit des contrats est `bd65d34`. Aucun merge de cette branche dans `main`.

## Vérifié

Toutes les commandes cargo/make ont tourné dans le worktree de la tâche, une
compilation à la fois, avec le target déjà amorcé et les variables suivantes :

```bash
cd /home/wq/Project/bascanada/alacod_tasks/m1-v1-contrats-combat-ia/alacod
source ../env.sh
export CARGO_BUILD_JOBS=4
# Pour les vérifications finales, après l'incident d'espace disque décrit ci-dessous :
export RUSTC_WRAPPER="$PWD/../rustc-compact.py"
```

Les logs sont conservés dans le dossier parent du worktree. Les résultats du
tableau suivent l'ordre du README §4 ; le test du déplacement précède les contrats.

| Commande | Résultat observé | Log local |
|---|---|---|
| `make test_scenarios` (déplacement seul) | **61 scénarios**, 2 tests réussis, 0 échec, 7 ignorés, **326,68 s**, sans `BLESS` | `../deplacement-scenarios.log` |
| `make test_scenarios` (contrats montés) | **61 scénarios**, 2 tests réussis, 0 échec, 7 ignorés, **323,66 s**, sans `BLESS` | `../final-scenarios.log` |
| `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots -p effects -p behaviors --no-fail-fast` | **288 réussis / 0 échec / 8 ignorés**, dont les 13 nouveaux tests ; scénarios inclus rejoués en 321,02 s | `../standard-tests.log` |
| `make lint` | **0 erreur** pour chacun des deux jeux : zombies (4 personnages, 4 armes à distance, 6 de mêlée, 1 table de vagues, 2 cartes), testbed (7, 4, 6, 0, 4) | `../lint.log` |
| `cargo fmt --all -- --check` | Sortie **0**, aucune sortie texte | `../fmt.log` |
| `./scripts/check-forbidden.sh` | **4 occurrences préexistantes**, aucune nouvelle : 3 `std::collections::HashSet`, 1 commentaire `rand::` | `../forbidden.log` |
| `./scripts/check-rollback-registration.sh` | Sortie **0**, message **OK** | `../registration.log` |
| `make gen GAME=zombies` | **10 armes**, attentes et traces toutes `ok`, aucune modification des scénarios ni traces | `../gen.log` |
| `ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py` | Sortie **0**, **61 scénarios**, tous les budgets passent ; tests 2/0/7 en **324,48 s** ; tableau généré | `../bench.log`, `../metrics.log` |
| `cargo tree -p combat` puis `cargo tree -p game` | **0 entrée `game`** dans l'arbre de combat ; **1 entrée `combat`** dans celui de game | `../combat-tree.log`, `../game-tree.log` |

Bench sans autre compilation active : `bench_bullets` **106,0 fps** (plancher 70),
`bench_horde` **67,2 fps** (plancher 38), `bots_four_mixed` **102,0 fps**.
Le journal de `main` pour T3.4 donne respectivement 104,7 / 66,7 / 109,5 fps :
les deux scénarios de bench restent comparables. Ce n'est pas un benchmark A/B
de `main` et de la branche sur la même session. L'état des processus au lancement
est conservé dans `../bench-machine.log`.

### P2p à deux clients

Commande exécutée : `../verify-p2p.sh > ../p2p.log 2>&1`.
Le script construit une fois le binaire puis lance deux exemplaires directs,
afin de conserver une seule compilation à la fois. Il reprend le protocole
du README avec un projet Compose isolé, l'image de signaling déjà disponible
(`alacod-signaling:latest`, `--no-build`) et les traces dans le dossier de tâche.
Les commandes essentielles du script sont :

```bash
cargo build -q -p zombies --profile headless --no-default-features
docker compose -p alacod-m1-contracts -f docker-compose.ci.yaml \
  -f ../p2p-compose.yaml up -d --no-build signaling
# Après 3 secondes, même lobby pour les deux clients, i = 0 puis 1 :
timeout 180 env ALACOD_HEADLESS=1 ALACOD_INPUT=neutral \
  ALACOD_STATE_TRACE="$PWD/../p2p/client-$i.trace" \
  ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x \
  "$CARGO_TARGET_DIR/headless/zombies" --matchbox ws://127.0.0.1:3536 \
  --lobby "$lobby" --number-player 2 --players localhost remote \
  --cid "client_$i" --name "client_$i"
# Les deux processus sont attendus et leurs codes contrôlés, puis :
wc -l ../p2p/client-{0,1}.trace
cmp ../p2p/client-0.trace ../p2p/client-1.trace
docker compose -p alacod-m1-contracts -f docker-compose.ci.yaml \
  -f ../p2p-compose.yaml down
```

Résultat : **2 sorties 0**, **599 lignes par client**, `cmp` retourne **0**.
SHA256 identique des deux fichiers :
`a44f5b4f531e0aceb25ac3f66a9be9b02d15eaf9cea84e1c4ba679575a3b0bce`.
Le conteneur et le réseau créés pour ce test ont été supprimés par Compose.
Logs et traces : `../p2p.log`, `../p2p/client-{0,1}.{log,trace}`.

### Preuve de neutralité des traces

`git diff --exit-code main -- tests/scenarios` retourne **0**. Un contrôle Python
complémentaire compare chaque fichier suivi avec `git show main:<chemin>` :
**61 fichiers `.trace`, tous identiques octet pour octet**.
SHA256 du corpus (chemins triés par `git ls-files`, NUL, contenu) :
`24b0053ea271b05eb9e40b0bf9879e50631d39adb60eaf14ef64ff90bd0ce315`.
Aucune trace réécrite, aucun `BLESS`, aucun besoin de `trace-diff` / dump détaillé.

Attention au mécanisme de checksum existant : dans cette version de bevy_ggrs,
un type de composant enregistré sans entité contribue au checksum même à vide.
Les deux nouveaux types vides ont la même contribution et s'annulent dans le XOR
global. Ils sont montés ensemble ; l'identité effective des 61 traces est la preuve.
Enregistrer un seul de ces types ne garantit donc pas à lui seul la neutralité.

Le checkout principal est resté propre. `git diff --check` retourne **0** ; le diff
des scénarios, de `kinds.rs` et de `system_set.rs` contre `main` est vide. `main`
local et distant étaient encore à `f170a92` lors du contrôle de livraison.

### Incidents de vérification

- Un essai ciblé des seuls crates combat/behaviors/effects a changé l'unification
  des features Bevy et commencé à compiler d'autres variantes de dépendances.
  Il a été interrompu (sortie **130**) ; il **ne compte pas** comme vérification.
  La suite complète prescrite a ensuite passé.
- Le premier lien du runner final a échoué par manque d'espace disque (signal du
  linker), avant l'exécution des tests. Nettoyage limité aux exécutables obsolètes
  et au fichier de lien incomplet **dans le target de cette tâche**, puis reprise.
  Le wrapper local `../rustc-compact.py` retire uniquement les sections de debug
  des exécutables (`-Wl,--strip-debug`, puis `strip --strip-debug`), sans changer
  l'optimisation du profil headless. Il n'est pas livré dans le dépôt. Les résultats
  verts ci-dessus sont ceux des reprises effectivement terminées.

## Non fait / incertain / non vérifié

- Aucun système d'exécution des contrats M1, aucune tâche suivante commencée.
- Pas de rendu, de vidéo ni de revue visuelle du HUD ou des effets de slash ;
  la vérification décrite ici est headless. Le mode `debug_ui` n'a pas été compilé.
- Pas de CI distante observée et pas de nouveau benchmark A/B sur `main`.
- Pas de dump détaillé / `trace-diff`, puisqu'aucune référence n'a été changée.
- Aucun merge dans `main`, aucune ligne du journal §10 écrite. Ces étapes et le
  passage du statut à `mergée` appartiennent à l'orchestrateur après sa revue.

## Dettes et questions ouvertes

- Les ids de profil, projectile, pattern et arme restent des chaînes : résolution
  par les propriétaires de vague 1 ; validation des références de kinds par T1.12.
- `combat` dépend maintenant aussi d'`animation`, des types de session
  ggrs/matchbox, de `stats` et de `run` (métadonnée `RunEnd` des attentes d'arme).
  Il n'a aucun chemin vers `game`. Un découpage ultérieur des données de config
  pourrait réduire ces dépendances ; cette tâche privilégie le déplacement neutre.
- Les particularités de simulation héritées des armes (dont les conversions
  numériques existantes) sont conservées ; aucun nettoyage comportemental ni
  changement de checksum historique n'est inclus.
- Les deux nouveaux états rollback devront être remplis par T1.3 / T1.4, avec
  leur preuve de changement de traces. Ils ne doivent pas être montés isolément
  en supposant qu'un enregistrement vide est toujours neutre pour le checksum.
