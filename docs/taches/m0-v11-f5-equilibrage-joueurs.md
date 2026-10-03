# m0-v11 — F5 : équilibrage par nombre de joueurs (vagues, prix, santé)

## Contexte

Le digest M0 (`docs/digests/m0-fin-de-vague-2.md`) a repéré que F5 (chantier F du plan §5 :
« `players` disponible dans les expressions partout où ça compte ») est listé dans le jalon M0
(plan §6) mais n'a jamais été porté : `content::expr` sait parser `players` (tests unitaires,
T1.4/T1.0a) mais **aucun RON de la simulation n'est évalué par `Expr`** — vagues, prix et santé
ne dépendent pas du nombre de joueurs. La dette D25 demande d'écrire la décision M0 ou M1 puis
de dédier une tâche. **Décision écrite le 2026-10-03 dans `docs/conventions.md` §18 : F5 ∈ M0.**
C'est cette tâche.

## Périmètre

### A. Évaluation par la simulation

1. Faire en sorte que les champs visés (B) acceptent un littéral **ou** une expression
   `content::expr` (`Expr` se désérialise d'une chaîne RON, ex. `"120.0 + (players - 1) * 30"`).
   Les littéraux actuels doivent rester valides et garder leurs valeurs.
2. Évaluer les expressions **une seule fois, au démarrage de la run** — quand le nombre de
   joueurs de la session est connu (local : `NUMBER_PLAYER`/config ; p2p :
   `ggrs_config.connection.max_player`, voir `crates/game/src/jjrs/p2p.rs` ; scénario : ses
   joueurs, voir `crates/scenario/src/runner.rs` et `crates/game/src/recording.rs`) — en valeurs
   concrètes, avant la première frame de simulation. **Jamais d'`Expr` dans l'état rollback** :
   seules les valeurs évaluées y entrent (déterminisme : mêmes configs + même nombre de joueurs
   partout ⇒ mêmes valeurs, voir `conventions.md` §18).
3. Erreur d'expression (parse, division par zéro, identifiant inconnu) = échec du chargement,
   jamais de valeur par défaut silencieuse. Un test unitaire le vérifie.

### B. Les trois familles de champs

- **Vagues** : `games/zombies/assets/waves/wave_config.ron` (`base_enemies`, `enemies_per_wave`,
  `max_concurrent_enemies`, `spawn_batch_size`, timings — ce qui est lisible et numérique).
- **Prix** : armes (`games/zombies/assets/weapons/` et `weapons.ron`), perks
  (`games/zombies/assets/economy/perks.ron`), `games/zombies/assets/economy/economy.ron`
  (coût des portes/achats, prix par joueur s'il y a lieu).
- **Santé** : `health` des personnages ennemis
  (`games/zombies/assets/ZombieShooter/Sprites/Zombie/*_config.ron`).

Ne pas étendre à d'autres familles de champs (le reste des chantiers F est pour M1/M2).

### C. Preuve de contenu

Au moins **une** valeur du contenu zombies doit réellement utiliser `players`, et être
démontrée par des scénarios : par exemple la santé d'un zombie qui grandit avec le nombre de
joueurs, vérifiée par la même partie à 2 et 4 joueurs avec une attente qui diverge
(`EntityHealth`/`EntityCount`/`Currency` selon la valeur choisie). Les 62 scénarios existants
gardent leurs traces, sauf bless avec preuve — voir Règles de traces.

### D. Documentation

Compléter `docs/conventions.md` §18 (déjà créé par l'orchestrateur) avec les détails
d'implémentation : le type d'enum/desérialisation choisi, le point exact d'évaluation au
démarrage de run, la liste des champs couverts par famille, les erreurs possibles.

## Règles de traces

- **Préférer** de nouveaux scénarios + une nouvelle valeur de contenu pour la preuve : les 62
  traces existantes restent intactes.
- Si une valeur existante doit dépendre de `players` (ex. la santé des zombies actuels) et fait
  changer des scénarios existants à 2/4 joueurs : bless avec la preuve standard
  (`conventions.md` §10 « Blesser une trace : la preuve ») et le dire dans le rapport.
- Tout bless se fait par l'orchestrateur, jamais par l'agent (voir Livrer).

## Critères d'acceptation

1. Les trois familles acceptent littéral + expression ; expressions évaluées une fois au
   démarrage de la run avec `players` en contexte ; erreur de parse = échec au chargement
   (testé).
2. Tests unitaires dans `crates/content` (et le crate où vit l'évaluation) : contexte
   `players`, erreurs, littéraux inchangés.
3. Preuve de contenu : au moins une valeur zombies dépend de `players`, avec des scénarios
   nouveaux verts (2 vs 4 joueurs, attente qui diverge) ; 62 scénarios verts, traces inchangées
   sauf bless prouvé.
4. Suite complète des crates verte, `make lint`, `make fmt`, `make scripts`, `make gen` sans
   modification inattendue.
5. Le chemin p2p évalue aussi au démarrage de la run (même code d'évaluation partagé) ;
   l'orchestrateur rejouera une recette N=2 au LIVRÉ.

## Règles du worktree

- Le worktree `alacod_tasks/m0-v11-f5-equilibrage-joueurs/` est créé par l'orchestrateur avec
  un target amorcé : `source ../env.sh` avant toute compilation, `CARGO_BUILD_JOBS=4`, jamais
  depuis un target vide.
- Une seule compilation à la fois machine-wide : b1 valide m0-v7-p2 (et l'orchestrateur le
  vérifie) — vérifier qu'aucun autre cargo ne tourne avant de lancer le tien (`pgrep -x cargo`),
  et libérer vite.
- `docs/conventions.md` : écrire **uniquement dans la section §18** (§16/§17 sont réservées aux
  voies M1 de c1/c2 ; b1 a aussi des modifications en cours). Merger `origin/main` dans la
  branche juste avant de livrer (règle d'hygiène) ; en cas de conflit, garder les deux.
- Ne pas merger toi-même ; ne pas bénir de trace.

## Livrer

`LIVRÉ m0-v11-f5-equilibrage-joueurs <sha>` — avec le rapport
`docs/taches/rapports/m0-v11-f5-equilibrage-joueurs.md` sur la branche : ce qui a été fait
(A-D), la valeur de contenu qui utilise `players` et où, les scénarios de preuve (noms,
attentes, frames), l'état des traces (aucune changée / liste des bless demandés avec preuve),
tests/lint/fmt/scripts/gen, et la base de la branche (doit être `origin/main` à jour).
