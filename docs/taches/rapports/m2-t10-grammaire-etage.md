# M2-T10 — Grammaire d'étage (étape 1)

Branche `m2-t10-grammaire-etage`. Conventions : `docs/conventions.md` §40.

## Livré (étape 1)

- `MapGenerationMode::Floor(FloorGrammar)` ; `Basic` intact.
- `world::FloorGrammar` (données), `map::generation::floor` (planificateur, `validate_plan`),
  `FloorMapGeneration` (rejoue le plan par `get_next_room`).
- Champ `room_kind` dans `AvailableLevel` (lu depuis LDtk).
- Kind de contenu `FloorGrammar`, désignation `floor:<id>`, lint + 2 fixtures.

## Mesures (test `mille_graines_respectent_la_grammaire`)

1 000 graines : moyenne 183,5 µs, max 11,8 ms, 1,05 tentative en moyenne ; tailles 8:189, 9:204,
10:202, 11:198, 12:207. Chaque plan passe `validate_plan` (connexité, types requis, boss à distance
maximale et cul-de-sac, aucun chevauchement, bornes). Déterminisme : même graine → même étage.

## Reste (étape 2)

Gabarits gungeon et script LDtk, `Floors` multi-étages (un chemin d'asset par étage), `bench_floor`,
scénarios `gungeon_floor_seed_<n>` et `gungeon_two_floors`, p2p.
