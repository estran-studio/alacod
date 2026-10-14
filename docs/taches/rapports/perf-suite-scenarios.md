# Rapport — perf-suite-scenarios (outillage : diviser par deux le temps de la suite)

Branche `perf-suite-scenarios` depuis `origin/main` (2936b72). Session cloud : **4 cœurs, 15 Go**,
sans autre charge (aucun job concurrent pendant les mesures ; une seule commande lourde à la
fois). Toutes les durées sont des durées réelles de cette machine, profil `headless`, Bevy en
bibliothèque partagée (`game/dev-dylib`).

## Résultat

| Suite complète (243 scénarios, 0 `BLESS`) | avant (`origin/main`) | après | gain |
|---|---|---|---|
| 1 processus | 3 065 s (51 min) | **1 038 s** (17 min) | ×2,95 |
| 4 tranches (`test_scenarios_par`) | 701 s (11,7 min) | **285 s** (4,8 min) | ×2,46 |

Les deux « avant » sont le binaire de `origin/main` construit dans un worktree et rejoué sur la
même machine (tranches à tour de rôle pour le 4 tranches, comme l'ancien script). **Aucune trace
n'a bougé** : les 243 scénarios sont verts contre leurs `.trace` de référence, en un processus
comme en 4 tranches, sans `BLESS`.

| fps simulés (1 processus, au calme) | avant | après | gain |
|---|---|---|---|
| `idle` | 54,4 (49,9 à la première mesure) | 224,8 | ×4,1 |
| `throne_solo` | 31,5 (30,4) | 65,7 | ×2,1 |
| `bench_floor` | 38,1 (36,5) | 90,4 | ×2,4 |
| `bench_bullets` | 44,3 | 161,9 | ×3,7 |
| `clone_solo` | 54,5 | 252,1 | ×4,6 |
| `clone_duo` | 51,5 | 240,9 | ×4,7 |
| `avant_poste_demo` | 64,1 | 253,1 | ×3,9 |
| `clone_quad` | 44,5 | 145,1 | ×3,3 |
| `bench_horde` | 22,6 | 44,7 | ×2,0 |
| `throne_progression` | 33,1 | 62,2 | ×1,9 |
| `throne_softlock_recul` | 29,3 | 53,1 | ×1,8 |
| `throne_three_floors` | 23,7 | 40,4 | ×1,7 |
| `throne_quad` | 18,6 | 27,9 | ×1,5 |

Les `throne_*` gagnent le moins : leur coût est dans la simulation elle-même (voir « Pistes non
appliquées »). Gain par scénario (temps réel, 243 scénarios) : de ×1,5 (`throne_quad`) à ×5,5
(`blank_floor_refill`), aucun scénario plus lent.

## 1. Mesure avant toute optimisation

Outils : pas de `perf` ni de `cargo flamegraph` dans l'image ; `strace -c -f`, `valgrind
--tool=callgrind` (compté en instructions, exact mais 50× plus lent) et des chronomètres dans le
runner (`ALACOD_PROFILE=1`, ajouté, voir plus bas). Le runner mesure aussi les postes de
l'environnement de test ; la répartition de la simulation vient de callgrind.

### 1.1 Le coût fixe est dans l'ordonnanceur de Bevy

`idle` (1 500 frames, 49,9 fps = 20 ms par frame, 7 ennemis, 135 entités) :

| Mesure | Résultat |
|---|---|
| temps utilisateur / système du test | 197 s / 88 s sur 235 s de mur (`idle`, `throne_solo`, `bench_floor`) : **le tiers du temps est du noyau** |
| `strace -c -f` sur `idle` | 2 204 566 appels `futex` pour 1 500 frames = **1 470 par frame** (89 % du temps de syscall) |
| `idle` sur 1 cœur (`taskset -c 0`) | **78,7 fps** (contre 51,8 sur 4 cœurs) |
| `idle` sur 2 cœurs | 49,8 fps |
| `idle`, exécuteur monothread sur 4 cœurs | 154 fps |

Plus de cœurs ralentissaient : la simulation compte des centaines de petits systèmes par frame,
l'exécuteur multithread de Bevy les réveille un à un. Et 4 tranches parallèles, chacune avec ses
threads, se battaient pour les 4 cœurs.

### 1.2 Distance de vérification du synctest (mesurée, **non modifiée**)

Même scénario `idle`, exécuteur monothread, `ALACOD_CHECK_DISTANCE` :

| distance | ms par frame | fps |
|---|---|---|
| 1 | 4,66 | 214,6 |
| 2 (défaut) | 6,81 | 146,8 |
| 4 | 11,2 | 89,2 |

Soit **≈ 2,2 ms par pas de vérification** (re-simulation + restauration + sauvegarde +
checksum) pour ≈ 2,4 ms fixes : à la distance 2, la re-simulation pèse environ deux tiers d'une
frame. C'est le poste le plus gros qui reste, et il est **interdit sans accord** (§ 3 de la
fiche) ; chiffré ici seulement : distance 1 = ×1,46 sur `idle`.

### 1.3 Répartition d'une frame d'`idle` (callgrind, 500 frames, après l'exécuteur monothread)

18,87 G d'instructions au total :

| Poste | Part |
|---|---|
| checksum GGRS du `FlowFieldCache` (`default_hasher`, 3 `BTreeMap` de ~23 000 entrées, 2 à 3 fois par frame) | **23,6 %** |
| `GgrsSchedule` (simulation, ordinaire) + re-simulations | ≈ 8,6 % + part de `handle_requests` |
| `handle_requests` de bevy_ggrs (save / load / avance) au total | 55,5 % |
| snapshot du `FlowFieldCache` (clone) + restauration | 4,4 % + 3,5 % |
| `update_flow_field_system` (Dijkstra, toutes les 30 frames ou changement de cible) | 5,3 % |
| leafwing `update_action_state` (`handle_clashes`, périphériques inutilisés en scénario) | 5,8 % |
| `build_app` (charge le contenu : `content::load_and_lint`, plugins) | 8,3 % (≈ 0,45 s par scénario) |
| texte d'UI Bevy (`measure_text_system`, overlay de vagues) | ≈ 11 % (relevé après les premiers gains) |
| invariants, métriques, attentes du runner | **0,4 %** (0,04 s sur 10,2 s ; `ALACOD_PROFILE=1`) |
| trace d'état (`record_state`, ligne de hash) | < 1 % (jamais en tête du profil) |

Les pistes de la fiche, confirmées ou non par la mesure :
- hash d'état calculé une fois par frame : **non** (déjà un seul `Checksum` par sauvegarde ; le
  coût était dans le *contenu* haché, pas dans le nombre de hash) ;
- trace écrite en tampon : **sans objet** (la trace est en mémoire, écrite seulement avec
  `ALACOD_DUMP_TRACE`) ;
- invariants : **0,4 %**, rien à gagner ;
- niveau de log en test : pas dans le profil ;
- systèmes de présentation actifs en headless : **oui, deux** (texte d'UI, périphériques) ;
- allocations par frame dans le runner : négligeables ; celles des snapshots, oui (clone du
  `FlowFieldCache`).

## 2. Ce qui a été fait

Chaque changement est dans l'outillage de test, ou sans effet sur les octets hachés :

1. **Exécuteur monothread en headless** (`crates/scenario/src/runner.rs`,
   `single_threaded_schedules`) : `SingleThreadedExecutor` sur tous les schedules de
   `build_app(headless = true)`. `ALACOD_EXECUTOR=multi` rétablit le comportement précédent. Le
   jeu lancé par `play_scenario`/`zombies` n'est pas touché. `idle` : 52 → 154 fps ; temps système
   de 88 s à 1 s. Sans risque pour le déterminisme : `GgrsSchedule` refuse toute ambiguïté d'ordre
   (`ambiguity_detection: Error`) ; les traces le confirment.
2. **`FlowFieldCache::layers` : `SharedFlowField`** (`crates/game/src/character/enemy/ai/navigation.rs`) :
   chaque champ de flux est derrière un `Arc` (clone du cache en O(1), snapshot restauré = même
   champ) avec l'image des octets de son `Hash`, calculée une fois (`OnceLock`). Le `Hash` réécrit
   cette image : même flux d'octets que `#[derive(Hash)]` sur `FlowField`, donc même checksum.
   `seahash` est insensible au découpage des écritures (son test `chunked_equiv`), et le test
   `shared_flow_field_hash_equals_derived` compare les deux valeurs. `Debug` délégué (trace
   détaillée inchangée). `idle` : 146 → 203 fps.
   *Écarté après mesure* : mémoïser le hash par `Res::is_changed()` (en synctest chaque frame
   restaure la ressource, donc la marque changée : gain nul) ; écrire les octets dans un tampon
   avant `seahash` (le coût était dans le parcours des `BTreeMap`, pas dans les appels : 450 µs
   de remplissage contre 28 µs de hash pour 184 Ko).
3. **Lecture des périphériques coupée en headless** (`game::character::skip_device_input`,
   appelée par le runner) : l'ensemble `InputManagerSystem::Update` de leafwing ne tourne plus ;
   `ActionState` n'est lu qu'avec `InputSource::Devices` (`player/input.rs`), jamais en scénario.
4. **Overlay de vagues coupé en headless** (`WaveDebugEnabled(!headless)`) : du texte d'UI Bevy
   mesuré à chaque frame, purement visuel (gardé pour `play_scenario` et les vidéos).
5. **Contenu chargé une fois par processus** (`runner::load_content`) : `Registry` et
   `GameManifest` sont clonés d'un cache (seul un chargement sans erreur est mis en cache). Sur
   11 petits scénarios joués dans un processus : 133 s → 35 s avec l'ensemble des gains.
6. **Tranches par durée mesurée** (`scripts/test-scenarios-sharded.sh`,
   `scripts/scenario-shard-plan.py`) : coût = `sim_seconds` de `target/metrics/latest.json` + 1 s
   de chargement, sinon `frames`/60 depuis le `.ron` ; plus long d'abord sur la tranche la moins
   chargée (déterministe, tri par durée puis nom). Le script passe à chaque processus sa liste
   (`ALACOD_SCENARIO=a,b,…`) ; le binaire n'applique plus l'alternance quand cette liste est
   donnée. `SHARD_PLAN=roundrobin` rétablit l'ancien comportement. Tranches de 272 à 285 s au
   deuxième passage (le plan s'appuie sur la suite précédente) ; premier passage avec des
   métriques d'avant les gains : 231 à 337 s.
7. **Corrigé en passant** : `make test_scenarios_par` échouait (`./target` relatif exporté par le
   Makefile, journaux écrits depuis `crates/scenario`) ; le chemin est maintenant rendu absolu.
8. `ALACOD_PROFILE=1` : temps du runner par poste (`update`, métriques, invariants), sans effet
   sur la simulation.
9. `tests/budgets.ron` : planchers relevés à la moitié de la mesure (24 scénarios) et trois
   scénarios `throne_*` lourds ajoutés (ils passaient sous le défaut de 40 fps en `make bench`).
   `docs/conventions.md` § 5 : commande `test_scenarios_par` et paragraphe « Vitesse de la suite ».

## 3. Vérification (commandes réellement lancées)

| Commande | Résultat |
|---|---|
| suite complète, 1 processus, sans `BLESS` | `test result: ok`, 243 scénarios, 1 038 s |
| `make test_scenarios_par` (4 tranches) | 4 × « ok », 243 scénarios fusionnés, 285 s |
| `make test_crates` | exit 0 ; **667 réussis, 0 échec, 10 ignorés** (dont `shared_flow_field_hash_equals_derived`, relancé à part : réussi) |
| `make lint` ×4 (zombies, testbed, throne, gungeon) | « aucune erreur » pour les quatre |
| `cargo fmt --all -- --check` | rien à afficher |
| `./scripts/check-forbidden.sh` | 6 occurrences, **identique à `origin/main`** (même sortie, vérifiée dans le worktree de base) ; le « 4 » de la fiche est périmé |
| `./scripts/check-rollback-registration.sh` | « OK » |
| `cargo check --workspace --all-targets --no-default-features` (profil `headless`) | compile |
| budgets stricts (`ALACOD_BENCH_STRICT=1`, les 33 scénarios de `budgets.ron`) | `test result: ok`, aucun budget franchi |

## 4. Non vérifié / hors de portée ici

- **p2p à deux clients (Docker) et restart en ligne** : non rejoués. La simulation n'a pas changé
  d'octet ; seuls changent `FlowFieldCache::layers` (même flux de hash) et l'exécuteur du runner de
  scénarios (le jeu en réseau garde l'exécuteur par défaut). À rejouer à la revue par précaution
  (`FlowFieldCache` est rollback).
- **Bench au calme sur orca** : les planchers de `budgets.ron` sont
  établis sur 4 cœurs cloud, un par un ; non confirmés ailleurs. La CI nocturne garde ses budgets
  absolus désactivés (§ 6).
- Les `.trace` : aucune réécrite (aucun `BLESS`).
- Le gain « ×2 » est mesuré sur 4 cœurs ; avec davantage de cœurs, les tranches ont plus à gagner.

## 5. Pistes non appliquées (chiffrées)

| Piste | Gain estimé | Pourquoi pas ici |
|---|---|---|
| distance de vérification du synctest 2 → 1 | ×1,46 sur `idle` (4,66 contre 6,81 ms par frame) | interdit sans accord de l'orchestrateur ; à chiffrer seulement |
| `build_flow_field` (Dijkstra) en grille dense au lieu de `BTreeMap`/`BTreeSet` | `bench_floor` : `update_flow_field_system` = 42 % des instructions, `is_blocked` = 22 % | simulation elle-même, hors périmètre ; sortie identique à prouver sur les traces des 243 scénarios |
| navigation des bots (`bots::navigation::update_for` / `build_field`) | 14 % de `bench_floor`, plus pour `throne_*` et `clone_quad` | outillage de bots, hors fiche |
| `wall_distance` (`BTreeMap<GridPos,u8>`) et `FlowField` internes en `Vec` | partie des 42 % ci-dessus | idem |
| construction de l'app (`CorePipelinePlugin`, shaders, `MipGenerationPlugin` en headless) | ≈ 7 % de `build_app`, 0,2 s par scénario | plugins de rendu de `DefaultPlugins` chargés sans GPU ; à faire avec un jeu de plugins headless |

## 6. Dettes et questions

- Le plan des tranches dépend de `target/metrics/latest.json` : un run partiel (`SCENARIO=idle`)
  écrase ce fichier et dégrade le plan suivant (retour aux `frames`). Un fichier de coûts versionné
  (`tests/scenario-costs.ron`) éviterait la dépendance au target ; non fait pour ne pas ajouter un
  fichier à maintenir.
- Les `throne_*` (50 % du temps des 10 plus longs scénarios) demanderaient une tâche dédiée à la
  navigation (bots et flow field).
- Atteignabilité (M2) : sans objet, tâche d'outillage.
