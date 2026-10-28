# X-4b « Sprites : import » — brouillon conservé (2026-10-27)

- **Branche poussée, jamais à fusionner** : `x-4b-brouillon-sur-x2a` @ `1beaa90b`. **Base `adc042d4` (intégration X-1 + X-2a), pas `main`.**
- **Non vérifié** : brouillon d'un sous-agent arrêté par quota, commité tel quel. Aucun build ni test lu, aucune revue.
- **Contenu** : `crates/content/src/sprite_import.rs` (869 lignes : `import_sprite`, `SpriteImportOptions`, `SpriteAnchor`, `SpriteImportResult`,
  `SpriteImportError`, `frame_index`). Il lit un manifeste `*.sprite.json`, vérifie les directions, les lignes et la taille des PNG, et refuse sans laisser de fichier
  partiel. Il copie les PNG et écrit une `SpriteSheetConfig` et une `AnimationMapConfig` **par clip**. Il insère `<id>_<clip>` dans `sprites.ron` en gardant les commentaires.
  `scale` et `anchor` sont des options explicites ; le `pivot` et le `ppm` du manifeste sont ignorés.
- `crates/animation/src/lib.rs` : `AnimationClipMeta` (`frame_duration` et `looping` par clip, facultatifs) et `advance_frame`.
- `crates/content/tests/sprite_import.rs` : 10 tests sur PNG synthétiques. Ils couvrent 8 directions et 1/4/5, la correspondance atlas/ligne, les lignes ou directions
  invalides, la taille de PNG fausse, le manifeste malformé, le lint du jeu généré, la fusion de la table et le réimport.
- `docs/conventions.md` : §93 (numéro à revoir sur `main`).
- **Pas de câblage CLI** : `alacod import-sprite` n'existe pas dans le brouillon.
- **Limite de conception** : une entrée `sprites.ron` par clip. Un personnage, qui n'a qu'un `asset_name_ref`, ne passe pas d'une planche à l'autre
  (idle → marche) ; le brief X-4b doit trancher : plusieurs feuilles par entrée, ou atlas.
- **`alacod.rs` de `main` suffit** : il suffit d'y ajouter une sous-commande clap `import-sprite` ; rien de X-2a n'est requis. Le conflit viendra plus tard, si X-2a dégèle,
  car X-2a déplace ce fichier dans `bin/alacod/main.rs`.
- **Reprise** : brief séparé, worktree depuis `main`. Reprendre le module et les tests à la main, en lecture seule, ne jamais rebaser la branche. Compiler, tester et prouver
  sur une planche réelle à 8 directions, puis afficher le résultat sur le Mac.
