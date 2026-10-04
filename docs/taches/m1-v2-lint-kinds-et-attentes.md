# m1-v2-lint-kinds-et-attentes — audit du lint des nouveaux kinds (T1.12) et des attentes de M1 (T1.15)

Lire d'abord `docs/taches/README.md` (agent **local**, branche depuis la tête livrée précédente,
`origin/main` mergé d'abord). Petite tâche (2 j), **ne déplace aucune trace**. Deux sujets, commits
séparés.

## Contexte

Plan §6 : « T1.12 Lint des nouveaux kinds — 2 j » et « T1.15 Attentes de M1 — `BulletCount`,
`HitsAtLeast`, `HasStatus`, `StatusStacks`, `EnemyState`, `EnemyDistance`, `FloorIndex`, `Clock`,
`CellState` ». Chaque tâche de la vague 1 a écrit son lint et ses attentes au fil de l'eau ; il reste à
**auditer** : chaque référence de contenu vers un autre contenu est-elle vérifiée, avec une fixture ?
Chaque attente de la liste existe-t-elle, est-elle dans `CLAUDE.md`, testée unitairement, et le
runner la rejoue-t-il au réenregistrement ?

## Décisions (fixées)

1. **T1.12 — tableau des références** : construire dans le rapport le tableau « kind source → champ →
   kind cible → règle de lint → fixture » pour tous les kinds de M1 : `Pattern` (projectile, arme),
   `Behavior::Shoot { weapon, pattern }`, `Behavior::Chase { profile }`, `variants` (skin, stat,
   LDtk `variant`), `Cave.characters`, `Surface.intgrid_value`, `Floors.levels` (`cave:<id>`
   compris), `Clock`/`Difficulty` (ids d'`entry`), `Effect` (pattern, stat, arme, mutation),
   `Progression` (mutations, `weapon_pool`, jauge), `Mutation`, `Status` (id dans `ApplyStatus`),
   `CharacterTest`/`generate_template` (carte, cible), `Scenario.characters`/`mode`. Pour chaque
   trou : règle `BrokenReference`/`OutOfRange`/`DuplicateId`/`Unsupported` + fixture
   (`crates/content/tests/lint_fixtures.rs`) + ligne dans le tableau du §3 des conventions.
   Aucune règle qui change un comportement de jeu.
2. **T1.15 — attentes** : vérifier pour chacune des neuf attentes du plan (+ `EnemyVariant`,
   `Gauge`, `Level`, `Mutations`, `EnemyContactBefore`, `EnemyNeverInWall`) : présence, test
   unitaire (`crates/combat/src/weapons/expectations.rs` ou `game::replay`), ligne dans `CLAUDE.md`,
   réenregistrement par le runner (`--save-scenario` conserve l'attente), forme RON documentée au §
   de sa tâche. Combler les manques ; **une seule liste** d'attentes dans `CLAUDE.md`, triée par
   tâche, avec le § de conventions de chacune.
3. **Hors périmètre** : nouvelles attentes, nouveaux kinds, correctifs de jeu (dettes).

## Règles

Deux compilations au plus ; purge + point d'état ; aucune trace bénie ni modifiée (`make
test_scenarios` vert sans bless, critère central) ; `docs/conventions.md` : tableau du §3 et une ligne
par § touché ; `CLAUDE.md` : liste des attentes ; `dettes.md`/`taches.md` : ne pas toucher.

## Livrer

Rapport `docs/taches/rapports/m1-v2-lint-kinds-et-attentes.md` avec les deux tableaux ; `git push -u
origin m1-v2-lint-kinds-et-attentes` ; `SendMessage` à `orch` : `LIVRÉ m1-v2-lint-kinds-et-attentes
<sha> : <une ligne>`. Ne merge pas.
