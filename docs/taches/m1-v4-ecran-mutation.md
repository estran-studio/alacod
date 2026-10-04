# m1-v4-ecran-mutation — écran de mutation et transition de niveau (T1.16, voie V4)

Lire d'abord `docs/taches/README.md` (agent **local**, `origin/main` mergé). Tâche moyenne (3 j),
**présentation seule** : aucune trace ne change (critère central). Prérequis : T1.10 (mutations,
`MutationChoice`, bits 13–15), T1.8 (`FloorState`).

## Décisions (fixées après proposition de b1)

1. **Modèle de vue pur** : `MutationScreenView { open, options: Vec<(id, nom, description)>,
   frames_left, highlighted: usize }` dérivé du `MutationChoice` du joueur local (`Update`, hors
   rollback, lecture seule), testé sans rendu, aussi rempli en headless.
2. **Description** : `describe_effect(&Effect) -> String` pure, en français, testée (une phrase par
   déclencheur/action v1) ; `Mutation.name` existe. Aucun champ de contenu en plus.
3. **Entrées** : D-pad / flèches gauche-droite surlignent (état UI local), A / Entrée valident →
   émettent `ChoiceA/B/C` (bits 13–15) ; touches 1/2/3 conservées. La simulation **ne fait jamais de
   pause** ; une barre de temps lit `choice_frames` et `since_frame`.
4. **Mise en page RON** : `ui/mutation_screen.ron` (ancrages, police, trois cartes), kind `Ui`
   existant, lint (police présente, trois emplacements).
5. **Transition de niveau** : à chaque changement de `FloorState::index` lu côté présentation (pas
   `FloorEntered`, émis seulement avec horloges/difficulté), fondu noir 0,4 s et recentrage caméra sur
   le joueur local ; réglages dans `camera.ron`.
6. **Validation sans écran** : tests du modèle et de la description ; captures hors écran
   (`capture_frames` du runner, PNG) de `levelup_choice` à f104 (ouvert), f134 (surbrillance), f140
   (fermé) et d'un passage de portail de `bot_floors_three`, jointes au rapport ; si la capture exige
   un GPU indisponible, le dire et livrer les tests seuls (William valide à l'écran).

## Règles

Deux compilations au plus ; purge + point d'état ; `docs/conventions.md` : **uniquement** §30 « Écran de
mutation et transition » ; `dettes.md`/`taches.md` : ne pas toucher. Traces : `make test_scenarios`
vert sans bless.

## Livrer

Rapport `docs/taches/rapports/m1-v4-ecran-mutation.md` (captures ou raison de leur absence) ; `git push
-u origin m1-v4-ecran-mutation` ; `SendMessage` à `orch` : `LIVRÉ m1-v4-ecran-mutation <sha> : <une
ligne>`. Ne merge pas.
