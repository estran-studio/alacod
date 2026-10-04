# m1-v1e-effets-mutations — effets v1 et mutations (T1.10, chantiers C1 v1 et C4 v1)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de l'agent qui la prend, branche
`m1-v1e-effets-mutations` créée depuis `origin/main`, même target que sa tâche précédente).
**Prérequis** : T1.8 `Floors` (mergée), T1.2 émetteurs et patterns (mergée), T1.4 behaviors
(`EnemyState`, mergée ou livrée : ne pas en dépendre pour le code). T1.9 horloges n'est **pas**
faite : ne pas en dépendre. Tâche moyenne (4 j), voie V1e run.

## Contexte

Plan §6 : « T1.10 Effets v1 et mutations (C1 v1, C4 v1) — `Effect { on, if, do }` avec
`OnLevelUp`, `OnKill`, `OnDamageTaken`, `Tick` ; actions `Modifier`, `Heal`, `SpawnProjectile` ;
rads → niveau → choix parmi trois mutations tirées d'un pool (flux `loot`) ; pool d'armes par
niveau. Acceptation : scénario par déclencheur ; `Event(levelup)` ; le choix passe par un input
dédié (scriptable). »

Ce qui existe : `crates/effects` — `Action` (cinq variantes des power-ups, `as_modifier` pur,
conventions §14), contrats `Effect { on, if, do }`, `On` (onze déclencheurs), `Condition` (sept),
kinds `effect_trigger`/`effect_condition` déclarés par `EffectsPlugin`, **aucune exécution** ;
`sim_core::gauge::Gauge { value, min, max, floor, thresholds: Vec<(Fixed, GaugeEvent)> }` avec
`add → Vec<GaugeCrossing>`, **utilisé nulle part** ; `Modifiers`/`StatReader` (§9) ; flux RNG
`loot` (drop des power-ups, `powerups.rs` l.474) ; `RollbackSystemSet::Effects` (power-ups) ;
`FrameEvents<Death>` (`last_hit_by`), dégâts accumulés puis appliqués dans `DeathManagement` ;
`BoxInput { buttons: u16 (bits 0–12 pris), pan, fire, switch_weapon }` et `replay::Button` ;
attente `Event(kind)` et `detect_events` (`kill`, `hit`, `floor`, `portal`…) ; `CharacterConfig`
(`stats`, `tags`, `starting_weapons`) ; `combat::emitter::Emitter` (T1.2 : tirer un `Pattern`
depuis une entité) ; `weapons::spawn_weapon_pickup`.

## Décisions (proposées par l'orchestrateur ; l'agent confirme ou amende en dix lignes avant de coder)

1. **Déclencheurs exécutés en v1** : `OnKill` (le tueur est `Death.last_hit_by`), `OnDamageTaken`
   (la victime, à la frame où les dégâts accumulés s'appliquent), `Tick(n)` (toutes les `n`
   frames depuis la pose de l'effet), `OnGauge(id, Above(x))` (jauge franchie à la hausse, via
   `GaugeCrossing`) et **`OnLevelUp`** (nouveau variant d'`On`, kind `effect_trigger`). Les six
   autres (`OnHit`, `OnDodge`, `OnReload`, `OnRoomClear`, `OnPickup`, `OnUse`, `OnEvent`)
   restent des contrats : le **lint refuse** leur présence dans un effet v1 (`Unsupported`, message
   « v2 »), jamais un silence.
2. **Conditions exécutées** : `HpBelow(x)` (fraction de `MaxHealth`), `HasTag(t)` (porteur),
   `TargetTag(t)` (victime d'`OnKill` / source d'`OnDamageTaken`), `NotHitFor(n)`. `Carrying`,
   `SquadSize`, `TargetInRange` : lint `Unsupported`. Toutes les conditions d'un effet doivent
   être vraies (ET).
3. **Actions** : `Action` **étendu** (même enum, §14 : « jamais un type séparé ») de `Modifier {
   stat, op, value }` (permanent, `source: Named("effect:<porteur>:<index>")`), `Heal(Fixed)`
   (points de vie, borné à `MaxHealth`, passe par le chemin de soin existant), `SpawnPattern
   (pattern_id)` (un `Emitter` éphémère T1.2 à la position du porteur, flux `patterns`), et
   `GaugeAdd(id, Fixed)`. Les cinq actions des power-ups restent valables dans un effet
   (`TimedModifier` sur le porteur seulement, pas sur tous les joueurs : la règle « tous les
   joueurs » est celle des power-ups, documentée §14, pas celle des effets).
4. **Porteur et état** : composant rollback **neutre** `Effects(Vec<Effect>)` (`rollback_and_trace_
   neutral`), posé seulement si le personnage déclare `effects: [...]` (`CharacterConfig`, nouveau
   champ optionnel) ou reçoit une mutation ; état d'exécution dans `EffectState` neutre
   (`BTreeMap<u32 index, u32 last_fired_frame>`, plus `last_hit_frame` pour `NotHitFor`). Un
   personnage sans effet n'a aucun composant : **traces existantes inchangées** (zombies compris).
5. **Système** : `apply_effects_system` dans `RollbackSystemSet::Effects`, `order_mut_iter!` par
   `GgrsNetId` du porteur, puis index d'effet, puis ordre des `do`. Il lit les `FrameEvents` de la
   frame (morts, dégâts) : si le set `Effects` **précède** `DeathManagement` dans la frame, les
   `OnKill`/`OnDamageTaken` se déclenchent à la frame suivante — constater, documenter au §
   conventions, **ne pas réordonner les sets** (traces).
6. **Rads et niveaux (C4 v1)** : kind **`Progression`** (fichier `progression.ron`, déclaré par
   `game.ron`, optionnel) : `( gauge: "rads", per_kill: "1", levels: ["3", "7", "12", ...],
   choices: 3, choice_frames: 600, mutations: "mutations", weapon_pool: [(level: 0, weapons:
   ["pistolet"]), (level: 2, weapons: ["fusil"])] )`. Chaque joueur d'un jeu **avec**
   `Progression` reçoit `Gauges(BTreeMap<String, Gauge>)` neutre (seuils = `levels`, événement
   `GaugeEvent("levelup")`) ; `OnKill` par un joueur ajoute `per_kill` à la jauge (système dédié,
   même set, pas un effet de contenu) ; un franchissement = `LevelUp` : `Level(u32)` neutre
   incrémenté, `Event("levelup")`, puis **choix**.
7. **Choix de mutation** : kind **`Mutation`** (dossier `mutations/`, `( name, weight: 1, tags,
   max_stacks: 1, effects: [Effect] )`). Au `LevelUp`, tirage de `choices` mutations distinctes
   dans le pool (pondéré, `max_stacks` respecté) dans le flux **`loot`** (ordre `GgrsNetId` des
   joueurs montés de niveau dans la frame) → composant neutre `MutationChoice { options: Vec<id>,
   since_frame }`. La simulation **ne se met pas en pause** (p2p). Le joueur choisit par trois
   nouveaux bits d'input `INPUT_CHOICE_A/B/C` (bits 13, 14, 15 de `buttons`, `Button::ChoiceA/B/C`
   dans les scripts, touches 1/2/3 en présentation) ; sans choix après `choice_frames`, la
   première option est prise (déterministe). La mutation choisie : ses `effects` sont ajoutés à
   `Effects`, `Mutations(Vec<id>)` neutre, `Event("mutation")`. Un deuxième `LevelUp` pendant un
   choix s'empile (`pending: u32`).
8. **Pool d'armes par niveau** : `weapon_pool` est consulté par un nouveau **drop d'arme à la
   mort** (`weapon_drop_chance` dans `Progression`, défaut 0 ; tirage flux `loot`, ordre `GgrsNetId`
   des morts, comme §14) : l'arme tirée parmi celles dont `level ≤` niveau **max** des joueurs,
   posée par `spawn_weapon_pickup`. Rien n'est tiré quand la chance est 0 ou sans `Progression`.
9. **Attentes** : `Event("levelup")`/`Event("mutation")` existants par kind ; nouvelles :
   `Gauge { player, id, min, max, at_frame }`, `Level { player, level, at_frame }`, `Mutations {
   player, contains: Vec<id>, at_frame }` (`game::replay`, liste `CLAUDE.md`, tests unitaires).
10. **Contenu et scénarios testbed** (zombies : **rien**) : `progression.ron` (par_kill 1, niveaux
    `["2","4"]`, 600 frames), mutations `coriace` (`Modifier MaxHealth Mul 1.5` + `Heal`),
    `vampire` (`OnKill → Heal 10`), `tireur` (`Tick(120) → SpawnPattern(ring_6)`), personnage
    `pilote` (copie de `player` avec `effects: [(on: OnDamageTaken, if: [HpBelow("0.5")], do:
    [Heal("5")])]`). Scénarios : `effect_on_kill` (`vampire` imposé via `mutations` du
    `PlayerScript`, kills → `Health` remonte), `effect_on_damage_taken` (`pilote` touché sous 50 %
    → `Health` > sans effet, scénario jumeau `effect_none`), `effect_tick` (`BulletCount` toutes
    les 120 frames), `levelup_choice` (deux kills → `Event(levelup)`, `Gauge`, `Level 1`, script
    presse `ChoiceB` à f+30 → `Mutations contains`, `Stat`), `levelup_timeout` (aucun bouton →
    première option à `since_frame + 600`). Six nouvelles traces (bless orchestrateur).
11. **Lint** (`lint_effects`, `lint_progression`, `lint_mutations`) : déclencheur/condition
    `Unsupported`, `Tick(0)`, `Heal ≤ 0`, pattern/stat/arme/mutation inconnus, `levels` non
    croissants, `choices` 0 ou > pool, `weight` 0, `max_stacks` 0, `per_kill ≤ 0` ; fixtures.
12. **Hors périmètre** : écran de choix à trois cartes (T1.16), HUD rads (T1.18), déclencheurs et
    conditions v2 (liste du point 1 et 2), objets passifs/actifs (M2), empilement de modificateurs
    de projectiles (M4), pause de la simulation.

## Traces attendues

Aucune trace existante ne change : composants neutres absents des personnages sans effet, flux
`loot` et `patterns` tirés seulement avec `Progression`/`SpawnPattern`, zombies sans effet ni
progression. Vérifiable par `make test_scenarios` vert sans bless (critère central).

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ; aucune
trace bénie par l'agent ; `docs/conventions.md` : **uniquement** §27 « Effets v1, jauges et
mutations » (+ une ligne au §14 renvoyant au §27) ; `CLAUDE.md` : attentes et bits d'input ;
`docs/taches.md` : ne pas toucher. Merger `origin/main` juste avant de livrer ; conflits : garder
les deux.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : chaque déclencheur et condition v1, `Tick` depuis la pose, `NotHitFor`,
   `Heal` borné, tirage des choix déterministe (graine ⇒ mêmes options, `max_stacks`), choix par
   bouton et par expiration, `weapon_pool` filtré par niveau, lint (`Unsupported` et le reste).
2. Six scénarios verts ; **tous les scénarios existants verts sans trace modifiée**.
3. `bench_horde` dans son budget (aucun porteur d'effet : coût nul) ; chiffres sous charge acceptés.
4. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` sans
   modification ; exemples racine compilés.
5. §27 écrit (ordre des sets constaté inclus) ; rapport honnête avec point d'état.

## Livrer

Rapport `docs/taches/rapports/m1-v1e-effets-mutations.md` sur la branche (README §7 ; sha de tête,
base `origin/main`). `git push -u origin m1-v1e-effets-mutations`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v1e-effets-mutations <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
