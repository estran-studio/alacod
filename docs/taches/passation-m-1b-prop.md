# Passation M-1b « Personnage prop » (jalon S1 dim 15)

Worktree `~/orca/workspaces/alacod/m-1b-prop`, branche `wquintal/m-1b-prop` (partie d'`origin/main` b6bbe24d). Spec : `~/orca/runs/alpha/m-1b-prop.md` ;
demande : `git -C ~/Project/bascanada/1837 show origin/main:docs/demandes/prop-image.md` ; consignes : `~/orca/runs/alpha/commun.md`.
**Rien n'a été compilé** : aucun cargo lancé, aucune trace jouée.

## Fait (édité, NON compilé)
- `crates/game/src/character/config.rs` : champ `pub prop: bool` (`#[serde(default)]`) après `counts_hits`.
- `crates/game/src/character/enemy/create.rs` : au début de `spawn_enemy`, si `config.prop` → chemin séparé : `create_character(..., stat_defaults vide, variante None)` +
  `insert(team)` puis `return`. Donc ni `Enemy`, ni `EnemyPath`/`MonsterState`/`EnemyAiConfig`/`EnemyBehaviors`, ni arme, ni `HarmoniumTag`. Le chemin des ennemis n'est pas touché.
- `crates/game/src/character/visuals.rs` (`attach_character_visuals`) : `if config.prop { continue; }` juste avant la création de la barre de vie (sprites déjà attachés avant).
- `crates/content/src/registry.rs` : `prop` dans `CharacterEntry` (pub), dans le schéma serde/schemars (`#[serde(default)]`, donc visible dans `alacod describe Character`) et dans le constructeur.

## Pas fait
1. **Lint** (`crates/content/src/lint.rs`, `lint_characters`, après le bloc `bomb` vers la ligne 409, avant « Référence : starting_weapons ») : refuser `prop: true` avec `counts_hits`,
   `starting_weapons` non vide, `boss.is_some()`. Prévu : `LintErrorKind::Unsupported`, message « personnage « X » : `prop: true` est incompatible avec `<champ>` ». L'édition a été rejetée (mauvais tag), rien n'est écrit.
   Fixtures : `crates/content/tests/fixtures/<nom>/assets/{game.ron,characters/x.ron}` (modèle : `bomb_fuse_zero`) + une ligne dans le tableau de `m1_audit_fixtures`
   (`crates/content/tests/lint_fixtures.rs` ~l.150) : trois fixtures `prop_counts_hits`, `prop_starting_weapons`, `prop_boss`, et une `prop_ok` (aucune erreur).
2. **Contenu neutre du testbed** : `games/testbed/assets/characters/prop-test.ron` (modèle : `dummy.ron`, mais SANS `ai` ; `prop: true`, collider rectangle 20×20, `base_health` 200, `skins {"1": (layers: {})}`)
   et `wall-test.ron` (autre collider) ; vérifier que `prop: true` + `team` absent donne `Team::Enemies` (ciblable par les balles). Les RON de personnages se lisent en `implicit_some`.
3. **Scénarios neutres** (`tests/scenarios/`, forme : `explosive_bullet_hit.ron`, map `testbed/rocks_static.ldtk`, `characters: [(character: "prop-test", x, y, at_frame: 5)]`) :
   - `prop_no_enemy` : 5 props + 1 `dummy`, `EntityCount(kind: Enemy, min: 1, max: 1, ...)`, `EntityCount(kind: Rollback ...)` ; ajouter **11 props** dans une variante pour dépasser le plafond global de 10.
   - `prop_stops_bullet` : un prop sur la trajectoire, un `dummy` derrière ; `EntityHealth` du prop baisse, celui du `dummy` reste à 200 (même impact qu'un `dummy` devant).
   Traces neuves : `BLESS=1 SCENARIO=prop_no_enemy,prop_stops_bullet make test_scenarios`.
4. Test `alacod new-content Character <id> --from prop-test` (copie textuelle de l'entrée, `crates/content/src/scaffold.rs`, test dans `crates/content/tests/cli.rs`) ; `alacod describe Character` montre `prop`.
5. Capture : `alacod sandbox testbed` + `alacod remote screenshot` avec un prop (`alacod remote spawn prop-test --ahead`), sans barre rouge ; la placer dans `docs/taches/rapports/`.
6. Preuve : `make test_scenarios SCENARIO=<neufs>,<2-3 existants avec décor/spawner : ex. explosive_bullet_hit,testbed_window,four_players_idle>` sans trace existante modifiée ;
   `make schedule_order_diff BASE=origin/main` ; `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make check_rapide` → journal
   `~/orca/runs/alpha/logs/m-1b-prop-check-rapide-<sha>.log`. Rapport `docs/taches/rapports/m-1b-prop.md`, § 112 de `docs/conventions.md`, ligne `LIVRÉ` à la fin de `docs/taches.md` (§10),
   suppression de cette note dans le commit du rapport, `status` « E prêt <sha> », un seul `worker_done`.

## Prochaine étape exacte
`cd ~/orca/workspaces/alacod/m-1b-prop && df -h /System/Volumes/Data` (sweep `scripts/target-sweep.sh target` sous 40 Go, `ask` sous 30), puis
`env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 cargo check -p game -p content` pour compiler le WIP, puis les points 1 à 6.

## Constats pour le rapport (lecteurs de `Enemy`, vérifiés par lecture, pas par exécution)
- Un prop sans `Enemy` sort de : séparation ennemi-ennemi de `move_enemies` (`pathing.rs`, seule collision molle entre ennemis : un prop ne repousse plus les ennemis voisins ; les murs/fenêtres
  restent des `Wall`/`Window`), ciblage (`enemy_target_selection`, charme), `max_enemies` global du spawner (**constante 10 dans `spawning.rs`**, le champ `EnemySpawnerComponent::max_enemies` n'y est
  PAS lu), vagues, salles (`rooms.rs` : un prop ne bloque plus l'« ennemis morts »), HUD `enemies`, bots (`hunter`, `explorer`), `remote brief`, fantômes/son de mort (`visuals.rs`, `feedback.rs`), étiquettes de lisibilité.
- Inchangés : collisions du joueur (par couche `enemy_layer`, pas par `Enemy`), balles (requête `Team`+`Health`+`Collider`), dégâts, mort.
- `enemy_spawn_from_spawners_system` ne tourne jamais en scénario (`WaveModeEnabled(true)` forcé) : la preuve « le spawner continue » passe par `EntityCount(kind: Enemy)` (même marqueur que le plafond) et non par un spawner joué.
- Nom : `prop` collisionne avec le kind `Prop` (barils, tables, `lint_props`) ; garder `prop: bool` (forme de la demande), le dire dans le rapport en trois lignes.

## Pièges
- Édition par `edit` : relire le fichier pour le tag avant chaque PUT (les tags changent) ; un fichier édité en plusieurs PUT : du bas vers le haut.
- Tuple `Bundle` ≤ 15 éléments ; un type rollback neuf = variante neutre (ici aucun type neuf).
- Ne jamais `BLESS` une trace existante ; si une trace existante bouge, le dire dans le rapport (aucune ne devrait : `prop` absent ⇒ chemin inchangé).
- Un seul cargo à la fois, `ALACOD_MUTE=1`, pas de pkill par motif, jamais `--force`.
