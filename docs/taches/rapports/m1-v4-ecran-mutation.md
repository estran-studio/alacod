# Rapport — m1-v4-ecran-mutation (T1.16, voie V4, §30)

Branche partie de `m1-v2-lint-kinds-et-attentes` (8a55477, `origin/main` dbc3de8 mergé).

## État en cours

Code écrit, **pas encore compilé** (attente du « feu vert » d'orch). Reste : compilation, tests,
suite complète (aucune trace ne doit bouger), captures hors écran si le GPU le permet.

## Fait

- `effects::describe` : description française d'un effet (déclencheur, conditions, actions),
  testée sur les mutations de throne ; `game::ui::mutation_screen` : test qui décrit toutes les
  mutations de testbed et throne.
- `game::ui::mutation_screen` : modèle de vue pur `MutationScreenView` (+ `build_view`,
  `apply_navigation`, `confirm_bit`, `confirm_input`, tests), plugin de modèle (headless aussi)
  et plugin de rendu (trois cartes, surbrillance, barre de temps).
- Entrées : `ChoicePrev`/`ChoiceNext` (←/→, D-pad), `ChoiceConfirm` (Entrée, A) → bit de la
  carte surlignée dans `read_local_inputs`.
- `ui/mutation_screen.ron` (testbed, throne), schéma `content::ui::MutationScreenLayout`, lint
  (police, trois emplacements, tailles) + fixtures `mutation_screen_font_missing`,
  `mutation_screen_two_slots`.
- `game::ui::floor_transition` : fondu noir 0,4 s et recentrage caméra au changement de
  `FloorState::index` ; `floor_fade_seconds`/`floor_recenter` dans `camera.ron` (défauts serde).
- `docs/conventions.md` §30.

## Écarts

- Le lint de `mutation_screen.ron` n'a pas de ligne dans le tableau §3 : la fiche limite la doc
  à §30 (la règle y est décrite).
