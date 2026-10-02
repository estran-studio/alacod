# T3.2 — vidéos et digest de fin de M0, livraison

Tête de branche vérifiée : `30778b8` (merge de `main` f170a92 dans `m0-v4-videos-digest`), sur
les commits de travail `0c5e954` et `50ec94e`. Le SHA livré ajoute ensuite le commit de ce
rapport, purement documentaire.

- Fiche : `docs/taches/T3.2-videos-digest.md`.
- Base : `main` `7df8a28`, puis `4f0d550` par avance rapide avant tout commit (T3.4 mergée
  pendant la tâche), puis `f170a92` par merge.
- Date : 2026-10-02. Agent : Claude (Claude Code, session cloud, 4 cœurs, 15 Go).

**Vérification standard verte, hors p2p et bench (non faisables ici). Aucune trace modifiée.**
Aucun merge dans `main`, `docs/taches.md` non modifié, aucune autre fiche commencée.

## 1. Fait

Livrables de la fiche :

1. **Vidéos** du clone, rendues sur `4f0d550` dans `target/videos/4f0d550/` par
   `make videos SCENARIO="clone_solo clone_duo clone_quad"` (Xvfb + Vulkan logiciel lavapipe).
   Plus `make views SCENARIO=clone_quad`, parce que la vue principale suit le joueur 0, un bot,
   et cache les achats du joueur 3.

   | Vidéo | Durée | Taille | Moments clés |
   |---|---|---|---|
   | `clone_solo` | 50,8 s | 1,8 Mo | 127 |
   | `clone_duo` | 77,8 s | 3,0 Mo | 128 |
   | `clone_quad` | 72,9 s | 4,6 Mo | 240 |
   | `montage` | 77,8 s | 3,3 Mo | — |
   | `clone_quad.vues` | 72,9 s | 15,6 Mo | 240 |

   Durées et moments clés sont identiques à ceux de T3.1. Tout ce qui fait moins de 5 Mo est
   copié dans `docs/digests/videos/` avec ses `events.json`. `clone_quad.vues` dépasse cette
   limite : lien seulement.
2. **Comparaison avec la vague 0** : `make compare_video SCENARIO=idle BASE=4e93269` produit
   `idle_4e93269_vs_4f0d550.mp4` (25 s, 3,5 Mo, copiée). Le clone était superficiel
   (154 commits) : `git fetch --unshallow` a été nécessaire pour avoir `4e93269`.
3. **`alacod-sim`**, commande de la fiche, 20 graines, sur `7df8a28` : code de sortie 0,
   **0 desync**, JSON dans `docs/digests/m0-fin-de-vague-2.sim.json`, tableau dans le digest.
4. **Digest** `docs/digests/m0-fin-de-vague-2.md` :
   - ce que le clone sait faire, une ligne par chantier avec sa tâche ;
   - les chiffres et les vidéos, avec la comparaison ;
   - ce qui manque pour M0, critère par critère du plan §9.8 ;
   - les dettes ouvertes et les trouvailles.
5. **Notes de revue** `tests/review-notes/4f0d550.md`, dans le format de `nightly.sh`
   (sections A, B, C puis vidéos) : ce qui est visible et ce qui paraît anormal, vidéo par
   vidéo, à compléter par T3.3.
6. **D14** fermée dans `docs/taches/dettes.md` (« fait dans T3.2 »). Le digest dit que le
   restart p2p n'est pas supporté (redirigé vers le lobby, `run_state.rs`) et qu'il est
   reporté à M1. La ligne correspondante dans la section M1 de `docs/taches.md` revient à
   l'orchestrateur.

Corrections faites en cours de route :

- **`scripts/scenario-video compare` produisait une comparaison fausse.** Le premier essai
  jouait le code de la base des deux côtés.
  - Preuve : les deux moitiés étaient identiques à 12-76 pixels près sur 480 000, et les
    `events.json` étaient identiques.
  - Cause : les crates du workspace ont le même hash de compilation dans les deux arbres, donc
    le même emplacement dans le `target` partagé. Leur empreinte renvoie aux sources de l'arbre
    compilé en dernier : cargo affichait `Fresh utils (/home/user/alacod/crates/utils)` avec la
    bibliothèque de la base, et la compilation de HEAD échouait sur `utils::rollback`.
  - Aggravant : l'échec était ignoré, car bash n'hérite pas de `set -e` dans `$(…)`.
  - Correctifs (`0c5e954`) : `shopt -s inherit_errexit`, et `touch` des sources `.rs` des deux
    arbres avant chaque compilation de `compare`.
  - Résultat : les deux moitiés diffèrent de 9 000 à 63 000 pixels, avec 8 moments clés pour la
    base contre 14 pour main.
  - Effet de bord : `make videos` et `make views` s'arrêtent aussi désormais si la compilation
    échoue.
- **Commentaire « À regarder » d'`idle.ron`** : il donnait encore les frames de la vague 0
  (f329, f399, f775, f1052). Remplacé par celles relevées sur `main` (f381, f465, f873, f1117).
  Commentaire seulement : la trace est inchangée et le scénario reste vert.

Décisions :

- **`alacod-sim` n'a pas été rejoué sur `4f0d550`.** Joué sur `7df8a28` (1 h 55) ; T3.4 n'a
  ensuite touché que des messages de log (`println!` devenus `debug!`), le lint, le testbed et
  la CI. L'orchestrateur a constaté les 61 traces identiques, et mes 61 scénarios sur
  `4f0d550` le confirment : la simulation est la même. Le dire plutôt que de rejouer 2 h.
- **Liste des noms dans `make videos`** : avec des virgules (`SCENARIO=a,b,c`, comme l'écrit la
  fiche), le script cherche un seul scénario de ce nom. J'ai utilisé des espaces.

## 2. Vérifié (dans ce clone)

| Commande | Résultat |
|---|---|
| `make test_scenarios` sur `7df8a28`, au départ | 61 scénarios, `2 passed; 0 failed; 7 ignored` (17 min à froid) |
| `make test_scenarios` sur `4f0d550` + T3.2 | 61 scénarios, `2 passed; 0 failed; 7 ignored` (13 min) ; `git status` : aucun `.trace` modifié |
| tests des dix crates (§4) sur `7df8a28` | 265 réussis, 0 échec, 8 ignorés |
| tests des dix crates (§4) sur `4f0d550` + T3.2 | **275 réussis, 0 échec, 8 ignorés** (chiffre de l'orchestrateur après T3.4) |
| `make lint` | zombies et testbed : « aucune erreur » |
| `cargo fmt --all -- --check` | rien à afficher |
| `./scripts/check-forbidden.sh` | 4 avertissements, ceux qui existaient déjà |
| `./scripts/check-rollback-registration.sh` | OK |
| `alacod-sim`, 20 graines (commande de la fiche) | code de sortie 0, 0 desync ; vagues 1 (5 graines), 2 (11), 3 (3), 4 (1) ; 12 graines finies par la mort des bots, 8 bloquées au plafond de 20 000 frames |
| vidéos | rendues et regardées sur des planches de 12 images, prises aux frames annoncées par chaque « À regarder » : conformes (détail dans les notes) |
| CI GitHub de `main` (outil GitHub) | « Tests & Format » vert sur `7df8a28` ; Nightly CI : 15 exécutions, aucune verte |

Lancées une seule fois, sans le relancer après le merge, car il n'a apporté que la CI
(`.github/workflows/nightly.yaml`) et une fiche (`docs/taches/m0-v6-dettes-powerups.md`), sans
code. `make gen` non lancé : aucune arme n'a changé.

Les fps de `make test_scenarios` dans le cloud (scénarios en parallèle sur 4 cœurs) mettent
24 scénarios sur 61 sous leur plancher, dont `bench_horde` à 37,9 pour un plancher de 38.
Simple avertissement hors `ALACOD_BENCH_STRICT`, et sans valeur de bench.

## 3. Non fait, non vérifié

- **p2p à deux clients** et **bench strict** : non faisables dans le cloud. Le digest cite le
  dernier bench au calme du journal (T3.4).
- **Page de revue non ouverte dans un navigateur** (pas d'écran). Je l'ai seulement regénérée
  (`target/videos/index.html`). Les vidéos n'ont pas été regardées en continu : seulement sur
  des planches d'images aux moments clés. Il n'y avait pas de son.
- **Vue `--follow 1` de `clone_duo` non rendue** : le joueur à terre n'y est pas suivi par la
  caméra, c'est noté pour T3.3.
- 200 graines (critère de sortie de M0) : non tentées, 20 prennent déjà 2 h ici.
- Je n'ai pas diagnostiqué les 8 graines bloquées.

## 4. Dettes et questions ouvertes (pour l'orchestrateur)

- **Bots** : 8 graines sur 20 bloquées au plafond sans finir leur vague (4, 11, 12, 13, 16, 17,
  18, 19), aucune vague 5. Le critère « les bots finissent le clone » est loin.
- **F5** (équilibrage par nombre de joueurs) est listé dans M0 (plan §6), mais aucune tâche ne
  l'a porté. `content::expr` n'est évalué nulle part dans la simulation.
- **CI de nuit** : jamais verte sur `main`. Des correctifs du runner sont arrivés pendant la
  tâche ; à revérifier.
- **Moments clés** (`crates/scenario/src/events.rs`) :
  - aucun pour la victoire ni pour le ramassage d'un power-up ;
  - les armes murales sont étiquetées « tombée au sol » à la frame 0.
- **HUD** :
  - « $ » sans montant et « ? | ? » une fois le joueur mort ;
  - « Juggernog — possédé » reste affiché tant qu'on est devant la machine ;
  - les power-ups instantanés n'ont aucun retour visuel, et il n'y a pas de sprite de pickup ;
  - l'état du coéquipier n'est affiché nulle part.
- **Écran de fin** : « 3016 frames » est le numéro de la frame de fin (`RunStep::Ended::at_frame`)
  et non une durée. Le libellé est trompeur.
- **Outillage** :
  - quand `CARGO_TARGET_DIR` n'est pas défini, les métriques de `make test_scenarios` vont dans
    `crates/scenario/target/metrics/`, où `scripts/scenario-metrics.py` ne les trouve pas ;
  - la fiche écrit `SCENARIO=a,b,c` avec des virgules.
- **Nommage** : le plan appelle « T3.4 » la fermeture des notes, mais la fiche T3.4 a servi à un
  lot de dettes. La fermeture des notes de `tests/review-notes/4f0d550.md` reste à attribuer.
- **Cloud** : le conteneur a redémarré une fois pendant les tests (relancés, verts), et le disque
  a frôlé la saturation à trois reprises. Les artefacts des crates du workspace ont été supprimés
  entre les étapes, sans effet sur les résultats.
