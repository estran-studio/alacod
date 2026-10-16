# M3-T5 : étage en grille, salle secrète, trésor à clé, salle post-boss

Branche `m3-t5-grammaire-grille` (depuis `origin/main` a9432ed9, qui contient M3-T0c, M3-T4 et M2-T14d).
Conventions : `docs/conventions.md` §70. Session b3 (OMP), travaux délégués à deux sous-agents (lint de la crate
`content` ; portes à clé et cartes du testbed) puis relecture indépendante (agent `reviewer`).
Réponse de William à Q8 appliquée : case unique **et** grandes salles 2×1, 1×2, 2×2 dès cette tâche.

## 1. Fait

| Volet | Où | Contenu |
|---|---|---|
| Planificateur en grille | `map::generation::grid` (nouveau), `floor::plan_floor` | croissance sur grille, règle d'Isaac (une pose ne touche que son parent), boucles (`loops`) aux pas tirés par un flux local, boss sur la profondeur maximale, salle post-boss (chance tirée une fois par étage), requises en culs-de-sac, salle secrète voisine d'au moins deux salles ordinaires, contrôle indépendant `validate_grid_plan`, borne `step_bound` / compteur `FloorPlan::steps` |
| Grandes salles | idem | un gabarit = un rectangle de cases (ports par segment de case, tirage d'abord du gabarit puis de la pose : un 2×2 n'est pas favorisé par ses ports), lint « multiple de `cell`, ouverture centrée par case de salle » |
| Graphe de portes | `PlannedRoom.links`, `IMapGeneration::get_links`, `IMapGenerator::add_link`, `GeneratedMap::close_unused_openings_exact` | portes de plus (boucles, voisines de la secrète), murage exact des ouvertures sans porte ; `Graph` garde `close_unused_openings` et ne produit aucun lien |
| Contrat | `world::FloorLayout::Grid { cell, max, loops, secret, post_boss }`, `PostBoss`, `RoomKindDef.lock_from_floor`, `RoomKindTable::door_lock_at` | champs à valeur par défaut, contenu existant inchangé |
| Trésor à clé | `map_ldtk::game::rooms` (`key_door_system`, `room_lock_system`), `InteractionType::KeyDoor`, prompt HUD | **`DoorLock::Key` n'était exécuté par personne** (T0c le renvoyait à T4, qui n'a fait que `Secret`) ; une porte à clé n'est jamais ouverte ni refermée par un combat voisin ; `lock_from_floor` (Q6 : fermé dès le second étage) |
| Salle secrète | grammaire + `rooms/secrete.ron` + minicarte (`hidden_from_minimap`) | une par étage, murée (`Secret`), jamais voisine du boss, accessible à la bombe ; n'est dessinée sur la minicarte qu'ouverte |
| Salle post-boss | grammaire + `rooms/diable.ron` | case accrochée au boss, chance `0.33`, vide (le contenu est T8) |
| Lint | `content` | gabarits en grille (taille, ouvertures par segment, même largeur d'ouverture), `post_boss`/`secret`, `lock_from_floor`, clé obtenable ; 11 fixtures |
| `isaac` | `games/isaac` | gabarits 1×1/2×1/1×2/2×2 (script `m3-t5-grammaire-grille.make_salles.py`), grammaires `sous_sol`, `sous_sol_grand`, `sous_sol_court`, `sous_sol_transit`, séquences (défaut : **deux étages**), types `tresor`/`secrete`/`diable`/`transit`, objet `bomb`, `starting_consumables: {"key": 0}` |
| Attente | `Expectation::RoomLayout` | taille et voisinage d'une salle typée |
| Scénarios | `tests/scenarios/grid_*`, `key_door_*` (testbed) | voir §70 |

## 2. Décisions et écarts à relire

- **Grande salle et « ouverture centrée sur chaque côté »** : lu **par segment de case de salle** (un 2×1 a deux ouvertures au
  nord et au sud) ; seule façon d'aligner ses portes sur celles de voisines 1×1. Q8 ne tranchait pas.
- **`DoorLock::Key` exécuté ici** (non prévu par la fiche, qui le croyait fait) : `InteractionType::KeyDoor` en fin d'enum,
  `key_door_system`, exclusion des portes à clé de l'ouverture d'office, du verrouillage et du nettoyage des salles voisines.
  Aucun coût de porte n'est débité (le handler de portes payantes ne voit pas `KeyDoor`).
- **`lock_from_floor`** (champ de `RoomKindDef`) porte « à partir de l'étage donné en données » : un type `tresor` unique, fermé
  à partir de l'indice d'étage 1.
- **`pleureur.ron` gagne `starting_consumables: {"key": 0}`** : `blank_system` ne sert `consumables_per_floor` qu'aux joueurs qui
  ont un `Inventory` ; sans cela la clé du second étage n'aurait servi personne avant un premier ramassage. Cause, avec la
  carte, du déplacement de **toutes** les traces d'`isaac` (voir §4).
- **La séquence par défaut d'`isaac` passe à deux étages** (`sous_sol` ×2, graine dérivée par emplacement, clé au second) : seule
  façon qu'une partie ordinaire rencontre le trésor à clé « dès l'étage 2 ». Les scénarios d'un seul étage (`grid_floor_*`, `grid_secret_*`,
  `grid_post_boss`) désignent `sous_sol_un`.
- **`entry.start_map` d'`isaac` = `floor:sous_sol_court`** (nouvelle séquence du même nom) : `maps/salles.ldtk` contient maintenant dix
  gabarits dont des 2×2 ; assemblé par `Basic` (le chemin des scénarios générés sans carte), le boss n'était plus atteint en
  1 500 frames par le bot `fonceur` (`boss_gardien`). Un étage court de trois salles le rétablit, comme `court` de gungeon.
- **`sous_sol_grand` est en 7×7** : relevé par la relecture, huit colonnes de 240 px ne tiennent pas autour d'un départ centré dans
  les ±1000 px du monde (`MapGenerationConfig::max_width`) ; la huitième colonne était refusée en silence. Documenté en §70.
- **Le nombre de salles** est tiré par `next_u32_range_inclusive` (modulo) et non par `below` : premier tirage d'une tentative à graine
  déjà mélangée, sans effet pour un intervalle de 5 ; l'aligner sur `below` aurait déplacé toutes les graines nommées plus bas.
- **Un étage `transit`** (départ + salle sans ennemi) sert aux scénarios du trésor à clé : sans lui, atteindre le second étage demande
  de vaincre un boss (≈ 1 500 frames).

## 3. Vérifié (résultats réels ; **suite verte sur la fusion avec `main` 0a719425**, qui contient `outillage-p2p-stress`)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : vert, 495 scénarios, 217 s sur la fusion (263 s avant : même suite) ; aucune trace existante n'a bougé hors `isaac`.
- `make test_crates` : 1 007 réussis, 0 échec, 13 ignorés (fusion).
- `make lint` (zombies, testbed, throne, gungeon, isaac) : « aucune erreur » ×5 ; `make gen GAME=…` ×5 : sortie 0, aucun diff hors
  `isaac` (régénéré et re-bénit).
- `cargo fmt --all -- --check`, `make check_sim_boundary` (« OK : 11 crates, 5 en dette D69 »), `scripts/check-forbidden.sh` (exception
  `Instant::now` de `map_ldtk/src/generation/mod.rs` portée de 1 à 2 pour le test de performance), `scripts/check-rollback-registration.sh`, `make test_tools` (17 tests) : verts.
- `fuzz_inputs` `ALACOD_FUZZ=0:8` (8 graines × 3 600 frames) : sans divergence.
- **Planificateur, 1 000 graines** (gabarits du test) : 1,08 tentative en moyenne, 353 évaluations de candidats au plus (borne `step_bound` : 77 414 400),
  boss 1×1 647 fois et 2×2 353, trésor 1×1/2×1/1×2 344/309/347, salle du diable 495 fois sur 1 000 (chance 0,5), boucle dans 822 étages sur 1 000
  (`loops: 2`) ; grammaires impossibles (grille d'une case, distance 40, plus de requises que de salles) : `Exhausted` sous la borne.
- **Gabarits réels d'`isaac`** (`isaac_etages_de_grille_sur_1000_graines`, 2 grammaires × 1 000 graines) : bornes, un boss, un trésor, une secrète, au plus un
  diable collé au boss, alignement, aucun chevauchement, chaque ouverture non murée est une porte (trois cases), secrète à ≥ 2 portes ordinaires
  atteignables sans la franchir, trésor en cul-de-sac, boss le plus loin ; diable 304/1 000, boucle 510 (`sous_sol`) et 929 (`sous_sol_grand`) sur 1 000.
- **Performance** : génération d'un étage de 12 à 15 salles en grille 0,80 ms contre 0,99 ms pour `grand` de gungeon (rapport 0,81, machine partagée ; fiche : < 2).
- **`Graph` strictement inchangé** : empreinte de la carte générée (gabarit, position, identifiants, voisins, murs et entités de chaque niveau) sur
  1 000 graines pour `etage`, `grand`, `court` de gungeon et `avant_poste` (Basic), identique sur `origin/main` et sur la branche
  (`b2c1a0ae243e2ea8`, `8ca93997592c6a1e`, `f524adea8fc1fa9d`, `bfafb4d2f9c4e26b`) ; sonde jetable, retirée.
- **p2p** : voir §6.

## 4. Traces bougées et preuve (§5 du README)

Seules les traces d'`isaac` bougent : `isaac_boots`, `isaac_floor_seed_1`, `generated/isaac/*` (dix scénarios). Deux causes :
(1) la carte est une grille ; (2) le joueur a un `Inventory` (`starting_consumables`).

Dumps détaillés `main` (a9432ed9, worktree à part) contre la branche :
- `idle` (zombies, 1 499 frames) et `gungeon_floor_seed_1` (799 frames, étage `Graph`) : **octets identiques** (`cmp`) et « identique » à `trace-diff.py`.
- `generated/isaac/weapon_larmes` (carte `gabarit_armes` inchangée) : `trace-diff.py` signale une différence dès la frame 0 sur les lignes du joueur ;
  avec `--ignore Inventory`, « identique (199 frames comparées) » : le seul écart est l'`Inventory` posé par `starting_consumables`.
- `isaac_boots` : écart dès la frame 0 sur les portes et points d'apparition (positions de la grille), 70 → 133 entités.
Re-blessées : `isaac_boots`, `isaac_floor_seed_1`, les dix `generated/isaac/*` ; neuves : `grid_*` (9), `key_door_*` (4).

## 5. Revue indépendante (agent `reviewer`, lecture seule) : constats et suites

Aucun bloquant. Corrigés : (1) lint « clé obtenable » laissait passer un trésor fermé dès l'étage 0 et signalait à tort `lock_from_floor` hors de la
séquence (règle refaite par placement `i ≥ lock_from_floor`, 2 fixtures) ; (2) des largeurs d'ouverture différentes passaient le lint puis échouaient en
`Exhausted` (le planificateur compare désormais la largeur des deux ports ; le lint exige une largeur unique ; test + fixture) ; (3) `sous_sol_grand` en 7×7 ;
(4) un test de `validate_grid_plan` ne vérifiait rien (`let _ = …`) : le refus « boucles » est testé ; (5) panique par indice si un type requis est le
type de remplissage : `depths.get(…)` et nouveau problème de grammaire.

## 6. p2p

Sur la fusion avec `main` (qui porte maintenant `scripts/p2p-stress.sh`) : `scripts/p2p-stress.sh isaac <inputs> 3000` avec `scripts/p2p-stress-inputs.py 3000 --pool
Up,Down,Left,Right,Fire,Interaction,Sprint` (`ALACOD_INPUT_DELAY=0`, entrées qui changent toutes les 1 à 3 frames, client 1 suspendu par `SIGSTOP`/`SIGCONT`, Docker OrbStack,
`MATCHBOX_PORT=3576`, `COMPOSE_PROJECT_NAME=b3`). Manifeste d'`isaac` temporairement sur la séquence `sous_sol_cle` et la graine -1013904216 (second étage à trésor fermé ;
annulé par `git checkout`) : la clé est servie à f1 et la porte à clé s'ouvre dès f9 (`key_door_open`, rejouée par les rollbacks).
Résultat : codes de sortie 0 et 0, `DesyncDetected` 0, **7 794 frames resimulées** (client 0 ; client 1 : 3), déconnexions 0, **traces identiques sur 2 999 lignes** :
« p2p-stress : OK ». (Avant la fusion, avec une copie jetable du script et le `state_trace.rs` de `outillage-p2p-stress` : 7 936 frames resimulées, 2 999 lignes identiques ;
avec le `state_trace.rs` de l'ancien `main` : traces différentes sur les sept dernières frames seulement, prédictions non confirmées écrites à la sortie, le défaut que cette
branche corrige.) Non rejoué : `scripts/p2p-restart.sh` (la session et le restart ne changent pas).

## 7. Non fait / non vérifié / incertain

- Bot : `explorateur` n'ouvre pas les portes à clé et n'utilise pas de bombe (T14). Le trésor et la secrète ne sont donc jamais ouverts en partie de bots.
- Salle du diable : accrochée au boss mais **jamais franchie** par un scénario (le bot gagne à la mort du dernier ennemi) ; son accessibilité repose sur
  le test de plan (voisine unique du boss) et sur l'ouverture des portes du boss à son nettoyage (même code que les autres salles).
- `cargo clippy` absent du nightly de la machine : non vérifié.
- Minicarte : rendu non regardé à l'écran (aucune session graphique) ; vérifié par les unitaires `minicarte_dessine_une_grille_de_salles_de_tailles_differentes` et
  `la_salle_secrete_n_apparait_sur_la_minicarte_qu_ouverte`.
- Formes en L, curse « étage agrandi » (T9), contenu de la salle du diable (T8) : hors périmètre.

## 8. Dettes et questions

- Q (William) : lecture « par segment » de l'ouverture centrée d'une grande salle (§2) ; trésor fermé à clé « dès l'étage 2 » donné par `lock_from_floor: 1` sur le
  type (un seul type) plutôt que par deux types.
- Dette : le HUD affiche « Ouvrir — clé » sans le nombre de clés requis (`Key(n)` avec n > 1) ; T16.
- Dette : une grammaire en grille dont `max` dépasse ce que permet `max_width` perd ses dernières colonnes sans le dire (pas de lint : le lint ne connaît pas `max_width`).
- Dette : `isaac` ne donne toujours pas de bombes au départ (décision de l'orchestrateur, §69) : la secrète ne s'ouvre qu'avec `PlayerScript::bomb`.

## 9. Atteignabilité

La séquence par défaut d'`isaac` (`entry.floors = sous_sol`) est en grille : toute partie ordinaire la rencontre, avec son trésor fermé à clé au second étage
(clé remise à l'arrivée). Graines nommées : secrète fermée `grid_floor_seed_1` (graine 1), `grid_secret_room` (5), ouverte à la bombe `grid_secret_bomb` (8) ; trésor à
clé `grid_treasure_key` (séquence `sous_sol_cle`, graine -1013904216) et partie ordinaire `isaac_floor_seed_1` (graine 1, second étage atteint à f1919) ; post-boss graines 8
(`grid_post_boss`), 23 et 123 (`grid_floor_seed_23`, `_123`) ; grand boss 2×2 graine 23 ; boucle graines 1 et 123.

## 10. Suite de la revue (2e passage)

1. `plan_grid` n'exige le gabarit post-boss que si `chance > 0` (comme `kinds_needed`) ; test `post_boss_de_chance_nulle_n_exige_aucun_gabarit` (plan valide sans `diable`, `NoTemplate` à chance 0,5).
2. `Secret` + `lock_from_floor > 0` : **refusé au lint** (fixture `lock_from_floor_on_secret`). Choix : §70 ne définit `lock_from_floor` que pour `Key` (« trésor fermé dès le second étage ») ; faire lire l'étage au mur secret ajouterait un comportement non demandé, et `is_secret` sert aussi à la minicarte et au verrouillage voisin.
3. Test `la_borne_de_pas_tient_avec_des_boucles` : `loops: 3`, post-boss, 1 000 graines, `steps <= step_bound`.
4. `cargo test -p map -p world -p content -p map_ldtk` : 360 réussis, 0 échec ; `make test_scenarios SCENARIO=isaac_boots,isaac_floor_seed_1` sans BLESS : vert ; `make lint` ×5 : aucune erreur ; fmt propre.
