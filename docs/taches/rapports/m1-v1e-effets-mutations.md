# Rapport — m1-v1e-effets-mutations : effets v1 et mutations (T1.10, C1 v1 et C4 v1)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport) ; branche partie de
`4f52d2b` (LIVRÉ T1.14), base `origin/main` `e26a3e3` (T1.14, T1.4, T1.5 mergées ; T1.9 non),
merge `98de6d2`. Fiche : [m1-v1e-effets-mutations](../m1-v1e-effets-mutations.md). Worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date :
2026-10-04 (nuit).

## État en cours

- **Fait** : tout le périmètre de la fiche (effets v1, progression, mutations, choix par input,
  drop d'arme par niveau), sept scénarios, §27, `CLAUDE.md`.
- **Chiffres** : toutes les traces existantes identiques sans bless ; 7 traces nouvelles à
  bénir ; 471 tests de crates réussis (seul échec : `scenarios`, traces nouvelles).
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **effects** : `Action` étendu (en fin d'enum) de `Modifier`, `Heal`, `SpawnPattern { pattern,
  weapon }`, `GaugeAdd` ; `On::OnLevelUp` (kind `effect_trigger`, 12 déclencheurs) ;
  `effects::runtime` (déclencheurs et conditions v1, `Tick` depuis la pose, soin borné) et
  `effects::progression` (niveau, tirage pondéré sans remise avec `max_stacks`, choix par bouton
  ou expiration, pool d'armes par niveau), purs et testés.
- **game::effects_runtime** : `Effects`, `EffectState`, `Gauges` (rollback, neutres) ;
  `apply_effects_system` ; `kills_by_killer` partagé. Dégâts subis : `HealthRegen` si présent,
  sinon un `DamageEvent` qui vise le porteur (mannequins sans régénération).
- **game::progression** : `ProgressionTable` (hors rollback, `OnEnter(GameLoading)`), `Level`,
  `Mutations`, `MutationChoice` (neutres) ; `init_progression_players`, `progression_system`,
  `weapon_drop_on_death_system` (chaînés, condition d'exécution : progression active).
- **Input** : bits 13/14/15 `INPUT_CHOICE_A/B/C`, `Button::ChoiceA/B/C` (`ALL_BUTTONS` à 18),
  touches 1/2/3.
- **content** : `CharacterConfig.effects`, kinds `Progression` (plusieurs fichiers, un actif) et
  `Mutation`, `LintErrorKind::Unsupported`, `lint_effect`/`lint_progression`/`lint_mutations`,
  cinq fixtures ; `entry.progression` au manifeste.
- **Scénarios** : `Scenario::progression`, `PlayerScript::mutations` ; attentes `Gauge`, `Level`,
  `Mutations` (avec `count`) ; moments clés `levelup`, `mutation`.
- **Testbed** : personnages `pilote` et `cible`, cartes `testbed/effets.ldtk` et
  `testbed/progression.ldtk` (copies d'`arena`), pattern `couronne`, mutations `coriace`,
  `vampire`, `tireur`, progressions `base` et `armes`. Zombies : rien.
- **Docs** : `docs/conventions.md` §27 (+ renvoi au §14), `CLAUDE.md` (attentes, champs, bits).

## Décisions et amendements

- **Placement** (accepté par l'orchestrateur) : `DeathManagement`, après les dégâts accumulés,
  avant le saignement et la mort, et non dans `Effects` : les morts et les `FrameEvents` n'y
  survivraient pas. Progression après `loot_drop_on_death_system` (flux `loot`, power-ups
  d'abord).
- **La mort de la frame prime** sur le soin (`Death`/`Downed` : rien ne se déclenche), test
  `la_mort_de_la_frame_prime_sur_le_soin`.
- **Progression opt-in** : sinon le testbed, qui la déclare, aurait changé ses traces. Plusieurs
  fichiers `progression/` autorisés (au lieu d'un seul) pour tester le drop (`armes`) sans
  toucher `base`.
- **`OnLevelUp`** se déclenche à la frame qui suit le **choix** de la mutation du niveau, pour
  tous les effets `OnLevelUp` du joueur (la nouvelle mutation comprise) ; pool épuisé :
  directement.
- **Choix armé** : un bouton de choix ne compte que s'il a été relâché depuis l'ouverture (un
  bouton tenu ne choisit pas deux niveaux d'affilée).
- **Attentes en `handle`** (comme `Currency`, `Stat`), pas `player`. `coriace` : `+50
  MaxHealth` (`Add`) et `Heal 50`, pas `Mul 1.5` (qui se cumulerait à chaque niveau).
- **`tireur`** : pattern `couronne` (une salve) plutôt que `ring_6` (inexistant ; `ring_8` est
  infini, l'émetteur ne se terminerait jamais).
- **`pilote`** : mannequin (copie de `dummy`) sur une carte dédiée plutôt que personnage
  joueur : le personnage d'un joueur est figé à `player`.

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=3` puis `2`, profil
headless, sur l'état fusionné (`98de6d2`, puis `cargo fmt`).

- `make test_scenarios` (sans bless) : **toutes les traces existantes identiques** ; échecs
  uniquement « pas de trace de référence » pour les 7 scénarios de T1.10. Aucune attente en
  échec. `bench_horde` 35,6 fps sous charge (plancher 38 ; aucun porteur d'effet, systèmes de
  progression sous condition d'exécution).
- Scénarios : `effect_on_damage_taken` 92 points contre 72 pour `effect_none` (16 coups de 8,
  les 4 derniers sous 50 % soignés de 5) ; `effect_on_kill` 100 à f150 contre 97,5 sans
  mutation ; `effect_tick` salves à f122, f242, f362 ; `levelup_choice` niveau 1 à f104, options
  [vampire, tireur, coriace], `ChoiceC` → `coriace`, MaxHealth 150, santé 110,8 ;
  `levelup_timeout` `vampire` à f704 (104 + 600), aucune mutation à f703 ; `weapon_pool_drop`
  rien au niveau 0, `shotgun` à f104 (niveau 1) et f155.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk
  -p map -p sim_core -p stats -p bots -p effects -p behaviors -p world -p utils --no-fail-fast` :
  **471 réussis, 1 échec, 9 ignorés** ; l'échec est `scenarios`, uniquement sur les traces
  nouvelles. Dont : `effects` 20 (déclencheurs, conditions, `Tick`, `NotHitFor`, soin, tirage
  déterministe et `max_stacks`, choix, pool d'armes), fixtures de lint, attentes
  (`progression_expectations_pass_and_fail`), test ECS de la mort qui prime.
- `make lint` sans erreur (deux jeux) ; `cargo fmt --all -- --check` vide ;
  `check-forbidden.sh` 4 occurrences préexistantes ; `check-rollback-registration.sh` OK ;
  `make gen GAME=testbed` et `GAME=zombies` code 0, aucun fichier modifié ; `cargo build
  --profile headless --examples` code 0.

## Non fait / non vérifié

- Écran de choix, HUD des rads (T1.16, T1.18) : en jeu fenêtré, seules les touches 1/2/3
  existent, sans affichage des options.
- Pas de test p2p ; bench strict au calme non fait ; pas d'option `--progression` dans
  `alacod-sim`.
- Traces nouvelles non bénies (à l'orchestrateur).

## Dettes / questions ouvertes

- Un joueur avec progression reçoit `Effects`/`EffectState` vides : `apply_effects_system`
  tourne alors à chaque frame pour lui (coût faible, mesuré nulle part).
- `per_kill` compte toute mort attribuée au joueur (y compris un allié en tir ami).
- `MutationChoice` est réécrit chaque frame pendant un choix (commande `insert`) ; neutre, sans
  effet sur les traces.
