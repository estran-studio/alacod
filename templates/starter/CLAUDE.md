# {{name}}

Jeu construit sur l'engine alacod (`{{engine}}`). Le contenu est **data-driven** : des fichiers RON
sous `assets/`, déclarés dans `assets/game.ron`, assemblent des behaviors codés dans l'engine.

## Boucle de travail

1. `alacod describe` : kinds du jeu ; `alacod describe <Kind>` : ids, champs **possibles** (types,
   variantes, plages du lint) et ceux utilisés, fichier modèle. Ne pas inventer de champ.
2. `alacod new-content <Kind> <id> [--from <modèle>]` : crée l'entrée en copiant un modèle qui passe
   le lint (fichier du kind, ou entrée de table pour `Weapon`…). Puis l'adapter.
3. Éditer le RON. Un hook relance `alacod lint` après chaque édition ; corriger jusqu'à zéro erreur
   (`alacod lint --json` pour une sortie structurée).
4. `alacod test <id>` : joue le scénario généré de l'entrée (arme : tir sur la cible ; objet, salle,
   boss, personnage : leur `test:`). `alacod test` seul les joue tous, `--list` les liste. Un test
   « ok » sans attente ne prouve rien : renseigner `test:` dans l'entrée.

## Bac à sable : essayer le jeu sans le code

`alacod sandbox` (dans le dossier du jeu ; = `cargo run --features native -- --sandbox [profil]`, `alacod sandbox --profile lourd` pour un autre profil ; `native` = `bevy/file_watcher` : **sans lui, pas de rechargement à chaud** des armes, personnages et caméra ; rien à ajouter dans `src/main.rs`) ouvre l'**arène vide** du jeu
(`sandbox: (arena: "maps/arene_vide.ldtk")` de `assets/game.ron` : des murs et le joueur, rien d'autre ; `alacod new-arena` la régénère, `--size 40x30` pour une autre taille), sans vagues ni étages, avec le **profil** `assets/sandbox/<profil>.ron`
(`arene` par défaut : solde de 1 000, aucun ennemi). Mode éditeur allumé : remote sur
`127.0.0.1:15702` (`ALACOD_REMOTE_PORT` pour un autre port), rechargement à chaud, local seulement. `--detach` lance le jeu en arrière-plan et rend la main quand il répond.

- Profil : `alacod describe Sandbox` (champs : `players[]` avec `character`, `weapon`, `items`, `currency`, `modifiers`…,
  `characters[]` posés, `items[]`, `chests[]`, `shop`) ; `alacod new-content Sandbox lourd` en crée un.
- Pendant la partie, par `alacod remote …` : `brief`, `profile [id]`,
  `give --weapon <id> | --item <id> | --set-currency N`, `spawn <personnage> --ahead` (ex. `spawn target --ahead` : la cible des armes), `modifier MoveSpeed Mul 1.1`,
  `reload`, `restart` (rejoue le profil : ce qui a été donné ou spawné disparaît), `screenshot <png>`.
- Modifier le RON d'un profil, puis `reload` et `restart` ; `alacod lint` valide les ids (personnages, armes, objets).
- Guide complet : `{{engine}}/docs/editeur-vocal.md` §2.2–2.3 et `{{engine}}/docs/conventions.md` §87–89.

## Règles qui piègent

- Les valeurs `Fixed` s'écrivent en **chaînes** RON : `"1.5"`, jamais `1.5`.
- Simulation déterministe (rollback GGRS) : pas de `f32` ni de `HashMap` dans le code de simulation
  de l'engine. Le contenu RON n'est pas concerné, mais tout nouveau behavior en Rust l'est : voir
  `{{engine}}/CLAUDE.md` et `{{engine}}/docs/conventions.md` §4.
- Un comportement absent du vocabulaire : d'abord le critère de `{{engine}}/docs/plan-engine.md` §4.8 (contenu d'abord, puis verbes d'engine
  si deux jeux s'en servent) ; sinon c'est un **plugin Rust de ce jeu** (section suivante), jamais une copie de l'engine.

## Plugin Rust du jeu (mécanique propre à ce jeu)

`src/main.rs` ne fait que `entry::run(GameSpec::new("{{name}}", env!("CARGO_MANIFEST_DIR")))` : l'engine fournit la partie, `--sandbox`, le runner
de scénarios et le lint. Pour du code propre au jeu, implémenter `game::extension::GamePlugin` et l'ajouter : `.plugin(MonPlugin)`.

- `extension()` déclare le vocabulaire : `content_kind("Caisse")`, `behavior("id", CustomBehaviorImpl { applicable, motion })`,
  `condition("id", fn)`, `action("id")`, `lint_rule("nom", fn)`. **Chaque nom doit aussi être déclaré dans `assets/game.ron`**
  (`plugin: (kinds: [..], behaviors: [..], actions: [..], conditions: [..], lint_rules: [..])`) : `alacod lint` ne voit que ça, et le jeu refuse de
  démarrer si les deux listes diffèrent.
- `build(app)` pose la **simulation** : composants et ressources rollback (`rollback_and_trace*`, `_neutral` pour un type posé sur de nouvelles
  entités), systèmes `GgrsSchedule` (placés par `RollbackSystemSet` ; ceux qui exécutent une action `Custom` : `.after(game::effects_v3::effect_status_system)`, dernier de la chaîne d'effets,
  lecture par `game::extension::custom_actions`). Règles du déterminisme : `{{engine}}/CLAUDE.md` (pas de `f32`, `order_iter!`, `despawn_rollback`, `BTreeMap`).
- `assemble(app)` pose le reste de la partie jouée (ressources, systèmes `Update`, présentation) ; jamais appelé par les scénarios.
- Dans le RON : `Custom(id: "rush", params: {"within": "450.0"})` dans `ai.behaviors`, `if: [..]` et `do: [..]` d'un effet d'objet ou de personnage.
  Les valeurs sont des chaînes (`CustomParams::fixed("within")`, `::uint("n")`). Un kind de plugin charge `.ron` dans `Registry::plugin_content`.
- **Tout plugin se prouve par un scénario** joué par le binaire du jeu : `cargo run --features native -- scenarios play [fichier.ron] [--bless]`
  (traces `<scénario>.trace` à côté), `cargo run -- scenarios gen --play` (scénarios générés), `cargo run -- lint` (règles Rust du plugin comprises).
  Un exemple complet : `{{engine}}/crates/content/tests/fixtures/plugin_game/` et `{{engine}}/docs/conventions.md` §91.

## Lancer

`cargo run` (rendu), `alacod sandbox` (bac à sable, rechargement à chaud ; `cargo run --features native -- --sandbox`) ou `ALACOD_HEADLESS=1 cargo run --no-default-features --features native`.
