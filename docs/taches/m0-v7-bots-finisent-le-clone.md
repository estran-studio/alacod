# m0-v7 — Les bots finissent le clone (critère de sortie de M0)

Lire d'abord `docs/taches/README.md` (agent **local**, pas la variante cloud : la tâche
compile, joue les scénarios et `alacod-sim`). Branche : `m0-v7-bots-finisent-le-clone`
(créée depuis `main` par l'orchestrateur, worktree et target amorcés par
`scripts/task-new.sh`). Indépendant de T1.0a (contrats M1) : les fichiers des bots
(`crates/bots`, `crates/scenario/src/bin/alacod-sim.rs`, pathfinding ennemi) ne bougent pas
dans T1.0a.

C'est la tâche qui ferme le dernier critère de sortie de M0 manquant (plan §9.8) :
« **les bots finissent le clone sur 200 graines sans softlock ni desync** ».
État (digest `docs/digests/m0-fin-de-vague-2.md`, 20 graines, 4 bots, jusqu'à la vague 5,
plafond de 20 000 frames) : **0 desync** ; vagues atteintes : 1 (5 graines), 2 (11), 3 (3),
4 (1) ; **aucune graine n'atteint la vague 5** ; 12 graines finissent par la mort des quatre
bots ; **8 graines bloquées au plafond sans finir leur vague** (graines 4, 11, 12, 13, 16,
17, 18, 19, dont 5 avec les quatre bots en vie), cause non diagnostiquée. Dette D20
(`docs/taches/dettes.md`).

Les bots v0 (T2.11, `crates/bots/src/decide.rs` : `decide(profile, view, rng) -> BoxInput`,
profils `immobile`, `fonceur`, `prudent`) n'ont **pas de pathfinding** : ils avancent en
ligne droite vers l'ennemi le plus proche. Seuls les zombies en ont, via le
`FlowFieldCache` (`crates/game/src/character/enemy/ai/navigation.rs`, Dijkstra multi-source
depuis les joueurs, portes fermées = cases bloquées). Les scénarios du clone utilisent des
bots (`clone_quad` : fonceur, prudent) : leur trace est bénie sur ce comportement.

## Phase 1 — Diagnostic des 8 graines bloquées (obligatoire avant tout correctif)

1. Rejouer localement les 8 graines bloquées (4, 11, 12, 13, 16, 17, 18, 19), commande du
   digest :
   `cargo run -q -p scenario --profile headless --bin alacod-sim -- --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile --seeds <n> --until-wave 5 --max-frames 20000 --json …`
2. Pour chaque graine bloquée, établir **où sont les zombies restants et pourquoi la vague
   ne finit pas** : zombies derrière une porte payante jamais ouverte ? dans une pièce que
   les bots ne visitent pas ? hors les murs (spawner mal placé) ? vague qui n'avance plus
   alors que tout est mort (bug de fin de vague) ? kite sans fin (un prudent qui fuit, des
   zombies qui le suivent, personne ne meurt) ?
   Outils existants : `map_probe` et `nav_stats` (`crates/scenario/tests/scenarios.rs`,
   tests ignorés, `ALACOD_NAV=…`), le mode remote (`make remote`, `scripts/alacod-remote
   state`), et la sauvegarde d'un run en scénario (T2.11) rejoué visuellement avec
   `make play_scenario`. Ajouter à `alacod-sim` un **dump de softlock** : quand une graine
   atteint `--max-frames`, le JSON dit pourquoi (ennemis restants avec positions, phase de
   vague, portes fermées, joueurs vivants avec positions) — il reste ensuite en permanence
   (un blocage futur s'explique de lui-même).
3. Écrire le diagnostic dans le rapport (§ « Diagnostic ») : une ligne par graine bloquée,
   la cause retenue, et la conclusion générale (hypothèse dominante). Mettre à jour la
   ligne D20 de `docs/taches/dettes.md` avec la conclusion (colonne acceptation : « fait
   dans m0-v7 » seulement une fois la phase 2 livrée ; sinon « diagnostic : … »).

## Phase 2 — Correctif (selon le diagnostic ; ce qui suit est la forme attendue)

1. **Pathfinding des bots.** Réutiliser la machinerie du `FlowFieldCache` pour les bots :
   un champ par `NavProfile` déjà calculé pour les zombies va des joueurs vers l'ennemi ;
   il faut l'équivalent vers les zombies (multi-source depuis les ennemis, ordre
   déterministe par `GgrsNetId`, mise à jour quand un ennemi change de case ou quand les
   cases bloquées changent — mêmes règles que le champ existant : BTreeMap, pas de f32,
   `rollback_and_trace_resource`, checklist GGRS de `CLAUDE.md`). Si une solution moins
   coûteuse s'impose (ex. suivre le champ existant à l'envers), l'écrire et la justifier
   dans le rapport : elle doit gérer portes fermées, fenêtres et coins comme le champ
   zombie le fait.
2. **Nouveaux profils de bots** (noms proposés : `chasseur` — suit le champ vers le zombie
   le plus proche, tire en se déplaçant, répare quand libre ; `acheteur` — chasseur qui
   ouvre la porte payante la moins chère quand il est bloqué, puis achète arme et
   Juggernog quand il a les points). **Les profils v0 (`immobile`, `fonceur`, `prudent`)
   gardent leur comportement exact** : `clone_quad` les met en scène et sa trace ne doit
   pas changer. `alacod-sim` bascule sur le nouveau line-up par défaut (le line-up v0 ne
   finit pas le clone) ; les deux restent jouables via `--profiles`.
3. **Portes payantes et achats.** Le bot déclenche l'action d'interaction existante
   (`fonceur` le fait déjà pour les réparations de fenêtres : le chemin existe) sur une
   porte payante quand son chemin est bloqué, avec vérification du solde (l'économie est
   partagée : `PointsCredit`, `economy.rs`). S'il le faut pour survivre : réanimation d'un
   coéquipier à terre (le prompt « Réanimer » existe déjà).
4. **Fin de vague garantie.** Si le diagnostic trouve un bug de vague (la vague ne se
   termine pas alors que ses zombies sont morts), le corriger. Si des zombies restent hors
   d'atteinte par construction (ex. spawner derrière une porte non interactive), corriger
   la donnée ou la règle, avec preuve `trace-diff` si une trace en dépend.
5. **Tests** : tests unitaires des nouveaux profils dans `crates/bots` (sur le modèle des
   tests existants de `decide.rs`) ; le dump de softlock a son test ou est couvert par un
   run bloqué artificiellement.

## Règles

- Une seule compilation à la fois sur la machine, `--profile headless`, `CARGO_BUILD_JOBS=4`,
  `source env.sh` du worktree. L'orchestrateur vérifie T1.0a sur le checkout principal au
  début de la tâche : ta compilation dans le worktree est la deuxième autorisée en même
  temps, pas de troisième.
- **Aucune trace bénie par l'agent.** Toute trace modifiée (attendue : aucune — les profils
  v0 ne changent pas et `alacod-sim` n'a pas de trace) exige la preuve `trace-diff` du
  README §5 et sa justification dans le commit ; l'orchestrateur blesse à la vérification.
- Les 61 scénarios restent verts sur le code livré (le vérifier dans le worktree avec
  `make test_scenarios` avant de livrer ; les métriques du M4 peuvent servir de base).
- Pas de modification de `docs/taches.md` (journal de l'orchestrateur) ; `dettes.md` :
  seulement la ligne D20.
- Un commit par étape cohérente (messages en français, attribution en fin de message),
  poussés sur la branche. Rapport `docs/taches/rapports/m0-v7-bots-finisent-le-clone.md`
  sur la branche (README §7), en tête le sha et la fiche, avec la section « Diagnostic »
  et le tableau des 20 graines de validation.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

- `alacod-sim`, 20 graines (1..20), line-up par défaut : **0 desync, 0 graine bloquée au
  plafond** ; chaque graine se termine par la victoire (entrée en vague 5) ou la mort des
  quatre bots ; tableau par graine dans le rapport. La majorité des graines gagne (des
  bots qui finissent le clone, pas seulement qui meurent proprement).
- Les 61 scénarios verts, **traces inchangées** (ou preuve `trace-diff` jointe) ;
  `clone_quad` en particulier intact.
- Tests des crates verts (dont les nouveaux tests des profils) ; `make lint`,
  `cargo fmt --all -- --check`, `check-forbidden.sh` sans nouvelle occurrence,
  `check-rollback-registration.sh`, `make gen GAME=zombies` sans modification.
- Le diagnostic des 8 graines est écrit, la ligne D20 à jour.
- Après merge, l'orchestrateur rejoue le critère complet : **200 graines** sans softlock
  ni desync (run long, hors de la tâche).
