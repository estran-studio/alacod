# Rapport m2-t0c-contrats-boss-profil — contrats de boss et de profil (M2-T0c)

Branche `m2-t0c-contrats-boss-profil` (base `d88cb33`, sans E1 ni T0b : l'orchestrateur fusionne au
merge), b1. Conventions : `docs/conventions.md` §37 (boss) et §38 (profil).

## 1. Fait

**Boss** (`behaviors::boss`, `game::boss`) :
- Contrats `BossDef { phases }`, `Phase { until, behaviors, on_enter, timeline }`,
  `PhaseEnd` (`HealthBelow` | `AfterFrames`), `Timeline { events, repeat }`, `BossState`
  (rollback + checksum + trace en variante **neutre**), `FrameEvents<BossPhaseChanged>` (neutre).
  Fonctions pures testées : `BossDef::next_phase`, `Timeline::due`.
- `boss_phase_system` (`EnemySpawning`) : phase 0 au premier tick, **une transition par frame** dans
  l'ordre du RON, `on_enter` puis frise, trace `ggrs{f= boss_phase net_id= from= to=}`.
- Règles de comportement par phase : `EnemyBehaviors::phase_rules` + `active(phase)` ; l'IA
  (`behavior_select_system`, `behavior_motion`, `enemy_attack_system`, `melee_hold_system`,
  attente `EnemyState`) lit les règles de la phase courante. Champ `boss` du personnage RON.
- Lint `lint_boss` + 5 fixtures (`boss_health_out_of_range`, `boss_phase_empty`,
  `boss_missing_until`, `boss_unknown_pattern`, `boss_timeline_after_repeat`) ; `lint_rules`
  extrait de `lint_behaviors` et partagé avec les phases.
- Attente `BossPhase`, moment clé `boss_phase`.

**Profil** (`crates/meta`, `game::profile`) :
- `Profile { version, id, currencies, unlocks, stats }`, RON versionné (migration v0 → v1, version
  plus récente refusée), écriture atomique, dossier `ALACOD_PROFILE_DIR` (défaut : dossier de
  données de l'utilisateur ; rien en headless sans variable), `Profiles` (ressource hors rollback).
- `write_profiles_at_run_end` (`Update`, jamais `GgrsSchedule`) : crédite et écrit une fois par run
  depuis `RunSummary` ; la simulation ne lit jamais les profils.
- Scénario : champ `profile: true` (dossier temporaire ; sinon rien d'écrit), attente `ProfileHas`.

**Bout en bout (testbed)** : `characters/gardien.ron` (120 PV, immobile, volée `volee` puis
couronne sous 50 %) ; `boss_two_phases` : phase 1 à f90 (moment clé `boss_phase`), couronne de 6
boules de feu à f93, mort du boss à f133 ; `profile_run_end` : victoire par `max_wave: 1`,
profil `local-0` avec `first_run`, `first_victory` et 11 d'essence.

## 2. Vérifié

- `make test_scenarios` complet **sans BLESS** : vert (EXIT 0), aucune trace existante modifiée
  (seules `boss_two_phases.trace` et `profile_run_end.trace` nouvelles).
- `cargo test` run, combat, game, content, map_ldtk, sim_core, stats, bots, effects, world,
  behaviors, meta : 528 réussis, 0 échec, 1 ignoré.
- `make lint` sans erreur sur les 3 jeux ; `check-rollback-registration` OK ; `check-forbidden` :
  4 avertissements préexistants ; `cargo fmt --all -- --check` propre.
- Tests de `scenario` : voir §5 (79 réussis).

## 3. Non fait / incertain

- p2p à deux clients : non lancé (le port Docker est réservé par l'orchestrateur) ; aucun changement
  d'input ni de session ici, mais deux types rollback neutres nouveaux (`BossState`, frame events) :
  traces identiques, à rejouer en p2p par l'orchestrateur.
- La table de tir d'une règle `Shoot` n'est pas par phase (voir §37) ; les tirs de phase passent par
  la frise (`SpawnPattern`).
- `BossState` n'a pas de curseur de frise (déduit de `entered`) : écart volontaire à la fiche.
- La pubkey allumette n'est pas connue ici : identifiant `local-<handle>`.
- Règles de récompense du profil (`apply_run`) provisoires (M2-T12).

## 4. Dettes / questions

- À fusionner avec E1 (`rooms`) et T0b (`items`) : `Scenario` gagne `profile` (T0b a ajouté `items`) ;
  les littéraux `Scenario { .. }` des tests demandent les deux champs (`items: vec![]`,
  `profile: false`) — conflit mécanique attendu dans `recording.rs`, `generate.rs`, `alacod-sim.rs` et
  quatre fichiers de tests.
- Disque : alertes à 100 % pendant la tâche (binaires de test de `scenario` ~1,1 Go chacun).

## 5. Tests de `scenario`

`cargo test` scenario (lib + expectations, cave, cave_floors, determinism, floors, generated,
hunter_doors, placement, run, softlock, spawn_stall, bots) : 79 réussis, 0 échec. `fuzz_inputs` non
relancé (l'input n'a pas changé dans cette tâche).
