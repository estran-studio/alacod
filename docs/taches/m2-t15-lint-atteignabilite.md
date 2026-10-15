# m2-t15-lint-atteignabilite — lint des kinds M2 et atteignabilité du contenu (M2-T15, V2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §3 (dossier de jeu), §4
(checklist d'un vocabulaire), §35 à §47 (kinds M2 : salles, objets, boss, profil, grammaire,
effets v2, esquive v2, synergies, arène, surfaces v2, bot explorateur), et `docs/taches.md` §7,
« Atteignabilité ». Branche `m2-t15-lint-atteignabilite` depuis `origin/main`. Estimation : 2 j.
Section de conventions réservée : **§53**.

## Contexte

Chaque tâche M2 a ajouté ses propres règles de lint (`crates/content/src/lint.rs`). Personne n'a
encore regardé l'ensemble. Deux trous sont connus :
- **une stat sans lecteur** : `Armor` n'était lue par aucun système, et le lint ne l'a pas vu (D61) ;
- **du contenu valide jamais rencontré** dans une partie ordinaire. La règle d'atteignabilité
  (`docs/taches.md` §7) exige qu'une mécanique mergée soit rencontrée dans une partie `gungeon`
  ordinaire. Aujourd'hui, seule une relecture humaine le vérifie.

## À faire

1. **Audit** : pour chaque kind de contenu M2 (`rooms`, `floor_grammars`, `floors`, `items`,
   `synergies`, `props`, `surfaces`, personnages boss, arènes, profil…), un tableau au rapport et en
   §53. Trois colonnes : références vérifiées, bornes vérifiées, fixtures existantes. Les trous
   trouvés sont comblés : nouvelle règle et fixture refusée, sur le modèle des fixtures existantes de
   `crates/content/tests/lint_fixtures.rs`.
2. **Atteignabilité**, en avertissement (nouveau niveau `Warning` si le lint n'en a pas ; un
   avertissement n'échoue pas `make lint`, mais il est listé) :
   - un type de salle (`rooms/<id>.ron`) qu'aucune grammaire de la séquence par défaut ne peut tirer ;
   - un gabarit LDtk de `salles.ldtk` dont le type n'est tiré par aucune grammaire ;
   - un personnage ennemi qui n'apparaît dans aucune salle, aucune arène, aucune vague, aucun
     renfort de boss ;
   - un objet qu'aucune source ne donne (départ, salle, coffre ou pool quand M2-T5 sera mergée :
     coder la source « pool » de façon à la brancher en dix lignes, et le dire au rapport) ;
   - une arme qu'aucun personnage, aucun mur ni aucun pool ne donne.

   Calcul statique sur le contenu lu par le lint, sans jouer de partie. Résultat sur les quatre jeux
   au rapport, avec chaque avertissement et sa cause.
3. **Stat sans lecteur** : liste fermée des `StatId` lues par la simulation (constante dans
   `sim_core::stats`, tenue à jour à la main, avec un commentaire qui dit où chaque stat est lue).
   Un modificateur sur une stat hors de la liste devient une erreur. Si D61 (b0, armure) n'est pas
   encore mergée, `Armor` reste hors liste et les contenus qui la modifient sortent en erreur :
   dans ce cas, mettre la règle en avertissement pour `Armor` seulement et le dire.
4. **Fixtures** : une par nouvelle règle, et une pour l'avertissement d'atteignabilité.

## Règles

Lint seulement : aucune trace ne doit bouger, aucun système de simulation n'est modifié. Si un
avertissement révèle un vrai trou de contenu (un ennemi jamais placé, par exemple), ne pas
corriger le contenu : le lister au rapport, l'orchestrateur en fera une dette.
`cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Atteignabilité dynamique (jouer des parties : c'est le bot explorateur, M2-T16), contenu nouveau,
équilibrage.

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ toutes
les deux heures et tue les processus d'arrière-plan : lancer la suite au premier plan) et finir par
un commit `LIVRÉ m2-t15-lint-atteignabilite` qui contient le rapport.

## Vérification

README §4 (`make test_crates` dont `lint_fixtures`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4). La suite
de scénarios complète n'est pas exigée si aucun fichier de simulation ni de contenu n'est touché ;
le dire au rapport. Rapport `docs/taches/rapports/m2-t15-lint-atteignabilite.md`, conventions §53.
