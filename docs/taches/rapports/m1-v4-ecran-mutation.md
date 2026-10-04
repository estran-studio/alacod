# Rapport — m1-v4-ecran-mutation (T1.16, voie V4, §30)

Branche partie de `m1-v2-lint-kinds-et-attentes`, `origin/main` (5cad4b3) mergé.

## État

Livrée : suite headless verte, aucune trace déplacée, captures hors écran faites (GPU disponible).

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

## Vérifié

`CARGO_BUILD_JOBS=2`, profil `headless` :

- `cargo test -p effects describe` (2), `cargo test -p game ui::` (18 : modèle de vue, navigation,
  validation, fondu, `camera.ron` des trois jeux, description de toutes les mutations de testbed et
  throne), `cargo test -p content` (dont `mutation_screen_fixtures` et le schéma des deux
  `mutation_screen.ron`) : verts.
- `make test_scenarios` : vert, **aucune trace déplacée**, aucun bless (le modèle de vue tourne
  aussi en headless, en lecture seule).
- Tests des crates, `make lint` (trois jeux), `cargo fmt --check` (après `cargo fmt`),
  `check-forbidden`, `check-rollback-registration`, `cargo check -p throne`, exemples : verts.
- `make gen` testbed et zombies : vert, rien de régénéré ; `GAME=throne` : panique connue sur
  `arsenal`, sans lien.

## Captures

`play_scenario --capture` (`--profile headless --features render`), images dans
`m1-v4-ecran-mutation/` :

- `levelup_choice_f104.png` : choix ouvert (niveau 1 à f104), trois cartes Vampire / Tireur /
  Coriace avec leur description générée, Vampire surlignée par défaut, barre de temps pleine.
- `levelup_choice_f134.png` : le script tient `ChoiceC` → Coriace surlignée, barre entamée.
- `levelup_choice_f140.png` : choix résolu, écran fermé.
- `bot_floors_three_f388.png` / `f392` / `f402` : fin du niveau 1 (portail), puis niveau 2
  (changement à f390) : f392 voile sombre et caméra déjà sur les joueurs au nouveau niveau, f402
  voile dissipé.

Descriptions vues à l'écran : « À chaque ennemi tué : regagnez 10 points de vie. », « Toutes les
2 s : une salve part de vous (couronne). », « À chaque niveau : +50 vie max, regagnez 50 points de
vie. ».

## À valider à l'écran (William)

- Le fondu paraît plus court que 0,4 s dans la capture (presque dissipé 12 frames après) : le
  temps de présentation de la capture ne suit pas exactement une frame de simulation par mise à
  jour. Réglable dans `camera.ron` (`floor_fade_seconds`).
- Les cartes sont légèrement transparentes (`card_color` `#141414e6`) : le jeu reste visible
  derrière ; à durcir dans `ui/mutation_screen.ron` si gênant.
- Pendant le choix, le D-pad et → déplacent aussi le joueur (la partie continue).
