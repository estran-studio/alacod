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

`alacod run --sandbox [profil]` (`cargo run --features native`, `native` = `bevy/file_watcher` : **sans lui, pas de rechargement à chaud** des armes, personnages et caméra ; rien à ajouter dans `src/main.rs`) ouvre l'arène du jeu (`generate_template.map` de
`assets/game.ron`, ou `sandbox: (arena: "…")`), sans vagues ni étages, avec le **profil** `assets/sandbox/<profil>.ron`
(`arene` par défaut : solde de 1 000 et une cible `target` à 200 unités du joueur). Mode éditeur allumé : remote sur
`127.0.0.1:15702` (`ALACOD_REMOTE_PORT` pour un autre port), rechargement à chaud, local seulement.

- Profil : `alacod describe Sandbox` (champs : `players[]` avec `character`, `weapon`, `items`, `currency`, `modifiers`…,
  `characters[]` posés, `items[]`, `chests[]`, `shop`) ; `alacod new-content Sandbox lourd` en crée un.
- Pendant la partie, par `alacod remote` (autre terminal ; `--port` ou `ALACOD_REMOTE_PORT`) : `brief`, `step 30`, `profile [id]`,
  `give --weapon <id> | --item <id> | --set-currency N`, `spawn <personnage> --ahead`, `modifier MoveSpeed Mul 1.1`,
  `reload`, `restart` (rejoue le profil : ce qui a été donné ou spawné disparaît).
- Modifier le RON d'un profil, puis `reload` et `restart` ; `alacod lint` valide les ids (personnages, armes, objets).
- Guide complet : `{{engine}}/docs/editeur-vocal.md` §2.2 et `{{engine}}/docs/conventions.md` §87–88.

## Règles qui piègent

- Les valeurs `Fixed` s'écrivent en **chaînes** RON : `"1.5"`, jamais `1.5`.
- Simulation déterministe (rollback GGRS) : pas de `f32` ni de `HashMap` dans le code de simulation
  de l'engine. Le contenu RON n'est pas concerné, mais tout nouveau behavior en Rust l'est : voir
  `{{engine}}/CLAUDE.md` et `{{engine}}/docs/conventions.md` §4.
- Un comportement absent du vocabulaire demande du code dans l'engine, pas dans ce dépôt.

## Lancer

`alacod run` (rendu), `alacod run --sandbox` (bac à sable, rechargement à chaud), `alacod run --headless` (sans fenêtre ; `--seed N`, `--players N`, `-- args du jeu`).

## Version d'engine

Le jeu dépend de l'engine par un **tag git** (`Cargo.toml`). `alacod engine status` dit lequel et quelle racine est compilée ; `alacod engine pin <tag|rev>` change de version ;
`alacod engine local <chemin-de-l-engine>` compile une copie locale à la place (patch dans `.cargo/config.toml`, ignoré par git : ne jamais le committer) pour développer engine et jeu ensemble, `pin` le retire.
