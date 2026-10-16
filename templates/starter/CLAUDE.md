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

## Règles qui piègent

- Les valeurs `Fixed` s'écrivent en **chaînes** RON : `"1.5"`, jamais `1.5`.
- Simulation déterministe (rollback GGRS) : pas de `f32` ni de `HashMap` dans le code de simulation
  de l'engine. Le contenu RON n'est pas concerné, mais tout nouveau behavior en Rust l'est : voir
  `{{engine}}/CLAUDE.md` et `{{engine}}/docs/conventions.md` §4.
- Un comportement absent du vocabulaire demande du code dans l'engine, pas dans ce dépôt.

## Lancer

`cargo run` (rendu) ou `ALACOD_HEADLESS=1 cargo run --no-default-features`.
