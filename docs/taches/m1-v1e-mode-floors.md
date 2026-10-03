# m1-v1e — mode `Floors` (T1.8) + attente `FloorIndex`

## Contexte

M1, Vague 1, voie V1e run (plan §6, F1). La ressource `Run` (T2.4) porte déjà
`mode: RunMode::{Waves, Sandbox}` et `step: RunStep::{Lobby, Playing, Ended}` avec `RunSummary`.
T1.8 ajoute le mode `Floors` : une séquence de niveaux, un portail à la fin de chacun, et la
continuité des net ids d'un niveau à l'autre. L'attente `FloorIndex` (liste T1.15) n'existe pas
encore : **cette fiche la revendique**.

## Périmètre

1. **`RunMode::Floors { config }`** + manifeste `game.ron` : liste ordonnée de niveaux (cartes
   LDtk), monde par niveau. Un niveau vide = fin de séquence (comportement au dernier niveau :
   boucle infinie au dernier niveau atteint — préciser ce choix dans le § conventions).
2. **Portail de fin de niveau** : apparaît (position déterministe, gabarit ou centre de la
   salle de départ) quand `EntityCount(enemy) == 0` dans le niveau courant ; le franchir charge
   le niveau suivant **sans recréer le process** : destruction propre des entités du niveau
   précédent, continuité des net ids (numérotation triée par contenu, comme le plan l'exige —
   voir `GgrsNetIdFactory`), les joueurs gardent santé/armes/monnaie.
3. **Défaite et résumé** : `RunSummary` existant, mêmes chemins que `Waves`.
4. **Attente `FloorIndex(n)`** dans `crates/scenario` (ressource ou composant lu à la frame
   donnée), testée unitairement.
5. **Scénario `portal_next_floor`** (testbed : deux petits niveaux) qui vérifie le passage
   0 → 1 ; et **`alacod-sim` : les bots finissent trois niveaux** avec les profils existants
   (`--until-floor` n'est pas demandé : T1.14, hors périmètre).

Tout nouvel état de simulation va en rollback + checksum + trace (`RollbackTraceApp`, jamais
d'appel direct hors de `crates/utils/src/rollback.rs`). Le mode `Waves` et ses traces ne
changent pas.

## Règles cloud (spécifiques à une session cloud)

- **Clone froid** de `https://github.com/estran-studio/alacod`, branche `m1-v1e-mode-floors`
  créée depuis `origin/main`. Pas de worktree ni de target amorcé : compilation complète depuis
  zéro, c'est normal. `CARGO_BUILD_JOBS=4`.
- **Pas de bench, pas de p2p** (l'orchestrateur les rejoue localement).
- **Ne jamais blesser une trace** (`BLESS=1` interdit). Si un nouvel état rollback change les
  traces : fournir la **preuve** (README §5) avec `scripts/trace-diff.py --ignore <nouveau
  composant>`. L'orchestrateur bénit.
- `docs/conventions.md` : ajouter un **§ numéroté distinct** (§17 « Mode Floors ») — le fichier
  est aussi modifié par d'autres branches (m0-v7, m0-v10, m1-v1a) : ne toucher qu'à ta section,
  et merger `origin/main` dans la branche juste avant de livrer.
- Une tâche à la fois : ne commencer aucune autre tâche de toi-même.

## Critères d'acceptation

1. Scénario `portal_next_floor` vert : deux niveaux, `FloorIndex` 0 → 1 vérifié par l'attente,
   résumé en fin de partie.
2. `alacod-sim` : les bots finissent trois niveaux (toute graine), **ou** BLOQUÉ explicite et
   honnête si les profils de bots actuels n'y arrivent pas — ne rien inventer.
3. Tests unitaires du chargement de niveau et de la continuité des net ids ; le mode `Waves`
   reste vert (62 scénarios inchangés hors bless éventuel, preuve à l'appui).
4. Tests des crates verts, `make lint`, `make fmt`, `make scripts`, `make gen` sans
   modification.

## Livrer

Rapport `docs/taches/rapports/m1-v1e-mode-floors.md` **sur la branche, commité et poussé**
(format README §7, sha de tête en en-tête). Pousse la branche. Termine ta réponse finale par la
ligne :

```
LIVRÉ m1-v1e-mode-floors <sha> : <résumé une ligne>
```

William la relaiera à l'orchestrateur. Ne merge pas, ne bénis pas, n'attends pas de réponse
dans la même session.
