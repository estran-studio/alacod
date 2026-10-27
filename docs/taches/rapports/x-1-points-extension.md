# Rapport x-1-points-extension (b6) : points d'extension d'un plugin de jeu

Fiche : `docs/taches/plan-1837-cli.md` (§1 critères 1 et 4, §3 D-d et D-e, §4 lot X-1, §5), `docs/plan-engine.md` §4.8, `CLAUDE.md`.
Branche `x-1-points-extension` depuis `25af5b5f`. Convention : `docs/conventions.md` §89. Dettes prises : **D139** (règles de lint Rust seulement dans le binaire du jeu, voir §6) ;
D133 fermée. Aucun nom, contenu ni réglage d'un jeu externe dans l'engine (la fixture s'appelle `plugdemo`).

Reprise : le premier worker a été interrompu (429) après ≈ 50 fichiers modifiés non commités. Ses éditions ont été conservées telles quelles ; la reprise a seulement relu, compilé, corrigé
ce que les preuves ont montré faux (§5) et écrit les docs, le rapport et le journal.

## 1. Ce qui existe

| Brique | Où | Rôle |
|---|---|---|
| Entrée unique (D-e) | `crates/entry` (`entry::run(GameSpec)`) | Un seul chemin pour la partie, `--sandbox`, les bots, `<jeu> lint [--json]`, `<jeu> scenarios gen\|play`. Installe le plugin **avant** de charger le contenu. Les cinq `games/*` et `templates/starter` sont migrés (cutover : plus aucun assemblage manuel de `CoreSetupPlugin`/`LdtkLocalGamePlugin` dans un `main.rs`). Web : `GameSpec::embedded`. |
| Contrat du plugin | `crates/game/src/extension.rs` | `GamePlugin { extension(), build(), assemble() }`, `Extension` (kinds, behaviors, conditions, actions, règles de lint), `CustomVerbs` (ressource immuable), `custom_actions`, `install`/`uninstall`. |
| Point d'insertion rollback | `core_sim.rs` : `extension::build_plugins(app)` | Appelé **avant** les cinq enregistrements de clôture (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams`), donc dans toute application (partie, runner, bots). Sans plugin : rien n'est enregistré. |
| `Custom` ouverts | `behaviors::Behavior::Custom`, `effects::Condition::Custom`, `effects::Action::Custom`, `content::registry::BehaviorEntry::Custom` ; paramètres `sim_core::custom::CustomParams` (`BTreeMap<String,String>`, `Fixed`/entiers en chaînes) | Ajoutés en fin d'enum (hashs des variantes d'avant inchangés). Un `Custom` non déclaré = `LintErrorKind::UndeclaredCustom`. `On` : non ouvert (aucun besoin ; voir §6). |
| Kinds et règles de lint d'un plugin | `content::manifest::PluginDecl` (`plugin:` de `game.ron`), `content::plugin`, `Registry::plugin_content` | Voir §2. |
| D133 | `scripts/schedule-order-diff.sh` | Base compilée dans `<target>-schedule-base` (checkout `…-src` stable). |

`Custom` n'est admis que dans un effet de personnage ou d'objet : refusé au lint dans un power-up, une frise de boss (messages dédiés). `Action::Custom` ne s'exécute pas dans `apply_action_sourced` (comme `Emit`) : l'engine pose
`EffectOut::Custom { carrier, target, id, params }` dans `FrameEvents<EffectOut>` ; le plugin le lit avec `custom_actions`.

## 2. Choix : comment `alacod lint` connaît les déclarations d'un plugin

Deux options : (a) déclarer les **noms** en données dans `game.ron` ; (b) déléguer tout le lint au binaire du jeu. **Choix : (a) + (b) pour ce que (a) ne peut pas faire.**

- `alacod lint` est un binaire **sans** le code du jeu. Avec (a) seul, il refuse déjà tout `Custom` ou kind non déclaré, valide la syntaxe RON des kinds du plugin et leurs ids, et signale par un avertissement `PluginRulesNotRun` les règles Rust qu'il ne peut pas exécuter.
- Le binaire du jeu (`entry`) installe ce que son code fournit (`content::plugin::install`) et `lint::run` compare **dans les deux sens** `game.ron` ↔ `extension()` (`PluginMismatch`) puis exécute les règles Rust (`LintErrorKind::PluginRule`). La partie, `--sandbox`, le runner et `<jeu> lint` refusent de démarrer sur une divergence.
- Raison : (b) seul rendrait `alacod lint` (hook d'édition de l'agent, CI) aveugle ou dépendant d'une compilation du jeu ; (a) seul ne pourrait pas vérifier des références propres au plugin. L'état est global au processus (`parking_lot::RwLock`) parce que `load_and_lint` est appelé de dix endroits (jeu, rechargement à chaud, runner, générateur) qui n'ont pas à se passer le plugin ; un jeu = un binaire.
- Limite (D139) : `alacod lint` n'exécute pas les règles Rust ; l'étape X-2b (`alacod test` par le binaire du jeu) peut appeler `<jeu> lint`.

## 3. Ordre d'enregistrement rollback : mesure (`crates/scenario/tests/plugin_order.rs`)

Même scénario (synctest, distance 2, rollbacks réels), un type rollback à hash **actif** (`Beat`), enregistré (1) par un plugin avant la clôture (`GamePlugin::build`), (2) après `CoreSetupPlugin` (donc après les cinq de clôture).

- **Trace d'état (checksum)** : identique à chaque frame (119 lignes) : les parts du checksum se combinent par XOR, l'ordre d'enregistrement n'y entre pas.
- **Liste des types** (`TracedTypes`, ce que compare `make schedule_order_diff`) : `Beat` est à la position `len - 6` (avant) ou `len - 1` (après) ; mêmes types, ordre seul différent.
- Conclusion : l'ordre n'a **pas d'effet sur la simulation ni sur le checksum**, seulement sur la liste de référence ; la règle de clôture reste utile parce qu'un type en queue de liste d'un clone déplacerait cette liste. Un plugin s'enregistre donc avant la clôture (c'est ce que fait `build`) ; le faire après est déconseillé, pas dangereux.

## 4. Preuve de bout en bout (fixture `crates/content/tests/fixtures/plugin_game`, jeu `plugdemo`)

Plugin (`src/plugin.rs`) : behavior `rush` (retenu quand la cible est à moins de `within`, droit sur elle à `speed`×), condition `after_frame`, action `grant_bounty` (un Luck de `weight` points lu dans le kind `Crate`, composant rollback neutre `Bounty`),
kind de contenu `Crate`, règle de lint `crate_weights`. Contenu : `characters/rodeur.ron` (behavior), `objets/prime.ron` (condition + action), `crates/coffre.ron`. Scénario `tests/scenarios/plugin.ron` (4 attentes).

Tests (`crates/content/tests/cli.rs`) :
- `alacod_lint_knows_the_plugin_vocabulary_from_game_ron` et `custom_action_is_refused_outside_effects` (rapides, `make test_crates`) : `alacod lint` accepte le jeu, avertit `PluginRulesNotRun`, refuse chacun des trois `Custom` non déclarés, un kind non déclaré, un kind du plugin qui reprend un nom de l'engine, un `Custom` dans un power-up.
- `plugin_game_runs_its_scenario_with_its_own_binary` (`#[ignore]`, `make test_alacod_cli`) : compile le jeu hors de l'engine ; **son binaire** lint (règle Rust exécutée), joue son scénario (`scenarios play --bless`, 4 attentes vertes, trace de 199 lignes) puis le rejoue contre la trace (`"trace":"ok"`), joue les scénarios générés de l'engine (`scenarios gen --play --only pistol`), et refuse un contenu que sa règle Rust rejette (alors que `alacod lint` l'accepte) ainsi que de démarrer dessus (`--sandbox`).

Résultats mesurés (VPS, profil `headless`, `CARGO_TARGET_DIR=…/b6/target`) : e2e `1 passed … finished in 22.66s` (compilation du jeu déjà en cache) ; `plugin_order` `1 passed` ; `alacod_lint_knows_…` + `custom_action_is_refused_…` `2 passed`.

## 5. Défauts trouvés par les preuves, corrigés

1. **Ambiguïté d'ordre dans `GgrsSchedule`** (le détecteur de bevy est en `Error`) : `grant_bounty_system` n'était ordonné qu'après `apply_triggered_effects_system` ; il entrait en conflit avec `apply_event_effects_system` et `effect_status_system` (`Modifiers`, `FrameEvents<EffectOut>`). Les `Custom` des tours `OnEvent` sont aussi émis par `apply_event_effects_system` : **le lecteur d'`Action::Custom` doit se placer `.after(game::effects_v3::effect_status_system)`** (dernier de la chaîne d'effets). Corrigé dans la fixture, `extension.rs`, `effects_v3.rs`, `CLAUDE.md`, `templates/starter/CLAUDE.md`, conventions §89.
2. **Longueur de trace** : `state_trace` n'écrit que les frames confirmées, soit `frames - 1` lignes (documenté par `outillage-p2p-stress`) ; deux assertions disaient `frames` (120 et 200). Corrigées (119 et 199).
3. **Attente de distance mal calibrée** : rôdeur à 400 unités, 200 frames : `rush` ×3 (accélération comprise) n'arrive pas (258 unités restantes à f199). Rôdeur rapproché à 200 unités : `rush` l'atteint, un `Chase` seul (≈ 100 unités à 30 u/s) non : l'attente `EnemyDistance max 40` à f199 est discriminante.
4. `scripts/check-sim-boundary.sh` : le nouveau crate `entry` n'était classé ni SIM, ni PENDING, ni APP (le script échoue pour tout crate non classé) : classé `APP` (outillage au-dessus de `game`/`map_ldtk`/`scenario`).
5. `plugin_order.rs` : `uninstall()` en tête et en fin (un plugin installé fuirait vers un autre test du même processus).

## 6. Dettes et limites

- **D139** : les règles de lint Rust d'un plugin ne tournent que dans le binaire du jeu ; `alacod lint` les annonce (`PluginRulesNotRun`) sans les exécuter. Fermer en X-2b (`alacod test`/`alacod lint` appelant `<jeu> lint`).
- `On` n'a pas de `Custom` : aucun besoin avant X-3 ; un déclencheur propre à un jeu passe par `OnEvent(nom)` + `Emit`, qui existent.
- Un plugin par processus (`extension::install` est global) ; deux plugins dans un même `GameSpec` sont fusionnés et un id en double panique à l'installation.
- Les kinds d'un plugin sont chargés **non typés** (`PluginContentEntry.text`) : le sens des valeurs est la règle de lint du plugin ; `alacod describe` ne connaît pas leur schéma.
- `alacod.rs` non touché (X-2a/X-2b). `GamePlugin::assemble` n'est jamais appelé par le runner de scénarios : un jeu ne peut pas y mettre de la simulation.

## 7. Vérifications (VPS Debian x86-64, profil `headless`, `CARGO_TARGET_DIR=/home/debian/orca/workers/b6/target`, jamais de BLESS)

| Contrôle | Résultat |
|---|---|
| `make schedule_order_diff BASE=25af5b5f` (D133, **un seul appel**, base et tête en targets séparés) | `OK : ordre des systèmes et des types rollback identiques (4550 lignes de schedules, 645 types)`, exit 0 (≈ 14 min la première fois, base compilée à part) |
| `make test_scenarios_par SHARDS=3 BLESS=` | 809 scénarios, 3 tranches `ok`, 2 127 s, **aucune trace modifiée** (`git status` : aucun `.trace`) |
| `cargo test -p content --test cli -- --ignored plugin_game_runs_its_scenario_with_its_own_binary` | `1 passed` (22,7 s, jeu déjà compilé ; la compilation à froid a pris 515 s et 756 s lors des deux essais rouges du §5) |
| `plugin_order` | `1 passed` (trace identique, listes de types différentes) |
| `cargo test` par paquet (un à la fois, `-- --exact --skip scenarios`) | sim_core 47, effects 38, behaviors 17, content 108 + 13 + 2 + 171 + 6 + 5 (3 `#[ignore]` : `make test_alacod_cli`), game 298, scenario (hors suite) 6 + 1 + 1, map_ldtk 15, entry 0, bots 99 : 0 échec |
| Build headless des cinq jeux (`zombies gungeon isaac throne testbed`) | exit 0 |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh` (51 occurrences auditées), `scripts/check-rollback-registration.sh`, `make test_tools` (38), `scripts/check-sim-boundary.sh` (13 crates SIM, `entry` classé APP) | verts |

| `new_game_opens_its_sandbox_without_any_added_code` (`make test_alacod_cli` : `alacod new demo`, nouveau gabarit à `entry::run`, compilation, `--sandbox` headless, `alacod-remote`) | `1 passed … finished in 524.03s` : le gabarit migré garde le bac à sable sans une ligne de code ajoutée |

Non vérifié : `make bench` (budgets), vidéos, p2p (Docker/matchbox), build wasm (`GameSpec::embedded` n'est compilé que pour la cible web : écrit, **pas compilé ici**).

Risques restants : (1) le chemin wasm de `entry` ; (2) un seul plugin installé par processus (état global) ; (3) les règles de lint Rust ne tournent pas dans `alacod lint` (D139) ; (4) un lecteur d'`Action::Custom` mal ordonné est refusé au démarrage par le détecteur d'ambiguïtés de bevy (message clair, mais à documenter dans chaque plugin : fait au §89).
