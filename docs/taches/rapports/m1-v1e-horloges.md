# Rapport m1-v1e-horloges — horloges d'étage et de run, difficulté (T1.9, F2)

**Branche** `m1-v1e-horloges`, partie de la tête livrée de T1.5 `148a990`, même worktree et
même target. Fiche : `docs/taches/m1-v1e-horloges.md`. `origin/main` mergé deux fois avant la
livraison : `6de2960` (T1.7 surfaces ; conflits `CLAUDE.md`, `conventions.md`,
`combat::expectations`, `character/mod.rs`) puis `e26a3e3` (T1.4, T1.5, T1.14 ; conflits
`CLAUDE.md`, `conventions.md`), en gardant les deux côtés à chaque fois ; sections de
`conventions.md` dans l'ordre 20, 21, 22, 23 (horloges), 24, 25, 26.

## État en cours

- **Fait** : tout le périmètre de la fiche (§1), vérifié sur l'état fusionné avec `e26a3e3`
  (§2). Livré.
- **Target** purgé après la suite complète (13 Go, /home 73 Go libres).
- **Reste à l'orchestrateur** : bless des 3 traces T1.9 (`clock_events`, `clock_floor_reset`,
  `difficulty_scales`), bench strict, p2p, merge.

## 1. Fait

- **État rollback** `run::Clock` (`crates/run/src/clock.rs`) : `run_started_frame`,
  `floor_started_frame`, `floor_index`, `fired: BTreeSet<String>`, `difficulty: Fixed` (défaut
  1). Checksum **neutre** (`rollback_and_trace_resource_neutral`) : la valeur par défaut
  contribue 0 (test `horloge_par_defaut_neutre`).
- **Activation, zéro changement** (amendement accepté) : horloges et difficulté ne tournent que
  si une partie les demande (champs `clocks`/`difficulty` d'un scénario ou d'un enregistrement,
  `entry.clocks`/`entry.difficulty` du manifeste). Sans activation, `Clock` reste par défaut et
  aucun `ClockFired`/`FloorEntered` n'est émis. `FloorEntered` n'est émis qu'avec activation ;
  T1.10 (`OnFloorEntered`) devra l'activer implicitement (écrit au §23 et dans la doc du type).
- **Kind `Clock`** (`clocks/<nom>.ron`) : `scope: Run | Floor`, événements
  `(id, at: Frames(n) | Seconds("x"), repeat)` ; un événement répété a l'id `"<id>#<n>"`.
  Règles pures dans `run::clock::due_events` (rattrapage de plusieurs échéances, ordre du
  fichier), remise à zéro de la portée `Floor` au passage d'étage (`Clock::enter_floor`).
  Événements `ClockFired` et `FloorEntered` en `FrameEvents` neutres ; moment clé `clock` dans
  `scenario::events`.
- **Kind `Difficulty`** (`difficulty.ron`, `value` = expression `content::expr`) : identifiants
  `players`, `floor`, `run_seconds`, `run_minutes`, `floor_seconds`, `floor_minutes`
  (`content::expr::difficulty_context`). Réévaluée **en simulation** chaque seconde depuis l'asset
  immuable (`game::clock::DifficultyConfig`, hors rollback) ; seul le résultat `Fixed` entre dans
  `Clock::difficulty` (exception encadrée, phrase ajoutée au §18).
- **Application** : santé des ennemis × difficulté à l'apparition (vagues, spawners, personnages de
  carte — à l'étage suivant, la difficulté de l'étage d'arrivée, `DifficultyReader::at_floor`) ;
  dégâts des ennemis × difficulté dans le résolveur unique de dégâts. `game::clock::scale` rend la
  valeur **inchangée** si la difficulté vaut 1 (test `difficulte_un_identique_au_bit_pres`).
- **Système** `game::clock::clock_system` dans `RollbackSystemSet::FrameCounter`, avant
  `increase_frame_system` ; horloges résolues à `OnEnter(GameLoading)` (`resolve_clocks_system`).
- **Lint** : identifiant d'événement en double, événements non ordonnés, `repeat` nul,
  identifiant inconnu dans l'expression de difficulté, difficulté non positive (évaluée sur une
  grille de contextes). Fixtures `clock_duplicate_id`, `clock_unordered`, `clock_repeat_zero`,
  `difficulty_unknown_identifier`, `difficulty_non_positive`.
- **Attente** `Clock { id, fired, at_frame }` (`combat::weapons::expectations`).
- **Contenu testbed** : `clocks/arene.ron` (portée `Floor` : `tic` 1 s, `tac` 120 frames,
  `renfort` 2 s puis toutes les 2 s), `difficulty.ron` (`1 + floor * 0.5 + floor_minutes * 0.5`),
  séquence `floors/deux_cibles.ron` (`floor_a` puis la nouvelle carte `testbed/floor_cible_b.ldtk`).
  `games/zombies` : rien d'activé.
- **Scénarios** :
  - `clock_events` : `tic` faux à f59, vrai à f61 ; `tac` faux à f119, vrai à f121 ;
    `renfort#1` vrai et `renfort#2` faux à f181 ; moment clé `clock` « tic » avant f65.
  - `clock_floor_reset` (partie de `portal_next_floor` + horloge) : `tic` vrai à f220 (étage 0),
    faux à f240 (étage 1, juste après le portail), revrai à f310.
  - `difficulty_scales` : follower de l'étage 0 (`NetId(27)`) à 60 PV à f2, étage 1 à f240,
    follower de l'étage 1 (`NetId(79)`) à 90 PV (60 × 1,5) à f240.
- `docs/conventions.md` §23 (+ phrase §18) ; `CLAUDE.md` : `Clock` ; `dettes.md` : D29
  (multiplicateurs par vague de zombies calculés mais lus par personne).

## 2. Vérifié (résultats réels)

Sur l'état fusionné avec `origin/main` `e26a3e3` :

- **Scénarios** (via la suite des crates) : 109 joués, **0 « trace différente »**, **0 attente
  en échec** ; seuls sans trace : `clock_events`, `clock_floor_reset`, `difficulty_scales`
  (3 bless demandés). Aucune trace bénie par moi. Aucune trace existante ne change : sans
  activation, `Clock` reste par défaut (checksum neutre) et aucun événement n'est émis.
- **Tests des crates** (`scenario run combat game content map_ldtk map sim_core stats bots
  effects utils behaviors world`, `--include-ignored`) : 480 tests verts hors `scenarios`
  (ci-dessus) — `run` 23, `content` 76, `lint_fixtures` 65, `game` 66 (dont
  `difficulte_un_identique_au_bit_pres`), `scenario --test expectations` 40. Seul autre échec : le
  doctest `rust,ignore` de `game::waves` que `--include-ignored` force à compiler, identique dans
  main (préexistant).
- **`make lint`** : `games/zombies` et `games/testbed` sans erreur (17 cartes testbed).
  **`fmt`** propre ; **`check_forbidden`** 4 occurrences (identique à main) ;
  **`check_rollback_registration`** OK.
- **`make gen`** : zombies 10/10 et testbed 20/20 (`ok`/`ok`), aucun fichier généré modifié.
- **Exemples racine** : `cargo build --examples --profile headless --no-default-features` OK.

## 3. Écarts, non fait / incertain

- **`difficulty_scales` mesure le follower, pas la cible** : la fiche suggérait la cible de
  5 000 PV, mais le portail ne s'ouvre qu'une fois l'étage 0 nettoyé ; il faut donc un ennemi
  tuable. Le follower (60 PV) sert aux deux étages.
- **Bench strict** et **p2p à deux clients** : non faits (pour l'orchestrateur). Sans activation,
  le coût est une lecture de ressource par frame et un test `difficulty == 1` par dégât.

## 4. Dettes, questions ouvertes

- D29 (ci-dessus) : décision de William.
- Le HUD (source `clock`), la nuit/marée et les événements qui déclenchent du contenu
  (`OnClock` → spawns) sont hors périmètre (T1.10 et suites).
