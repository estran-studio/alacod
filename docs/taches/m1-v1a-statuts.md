# m1-v1a-statuts — statuts Burn/Slow/Stun/Freeze (T1.3) + attentes `HasStatus`/`StatusStacks`

## Contexte

M1, Vague 1, voie V1a combat (plan §6, B3). T1.0a (`51d70b6`) a créé le kind `StatusDef` et
l'a enregistré (`Kinds`/`KindRegistry`), mais **aucun statut n'est implémenté ni posé sur la
moindre entité**. T1.1 (projectiles composables, voie V1a) est mergée sur `origin/main` : ses
`on_hit`/`on_expire` (actions de la crate `effects`) existent et peuvent déclencher des statuts,
mais cette fiche n'en dépend pas — les statuts se posent aussi par arme/contact/config. Les
attentes `HasStatus` et `StatusStacks` (liste T1.15) n'existent pas encore : **cette fiche les
revendique**.

## Périmètre

1. **Composant rollback `Statuses`** sur les entités (crate `combat`, chantier B3) : une liste
   de statuts sourcés `{ kind, source: Option<EntityRef>, reste_frames }` — rollback +
   checksum + trace (`RollbackTraceApp`, jamais d'appel direct hors de
   `crates/utils/src/rollback.rs`).
2. **Les quatre statuts** et leurs règles (fixées ici, pas inventées) :
   - `Burn` : N ticks de dégâts (dégâts/tick et période fixés dans le RON du `StatusDef`) ;
     réapplication = **empilement des ticks restants** (plafond : 2× la durée de base) ;
   - `Slow(facteur)` : vitesse de déplacement × facteur ; réapplication **rafraîchit** la durée ;
   - `Stun` : ni mouvement ni tir ; réapplication rafraîchit la durée ;
   - `Freeze` : comme `Stun`, plus **recul annulé** (momentum gelé) ; réapplication rafraîchit.
   À expiration, le statut se retire proprement (fin de frame déterministe).
3. **Application** : via les `Action` existantes de la crate `effects` (une action « pose un
   statut » réutilisable par `on_hit`/`on_expire` des projectiles), et par config là où ça a du
   sens (au minimum les projectiles ; contact arme/ennemi si le chantier B3 le prévoit).
4. **Visuel dérivé** : le rendu lit `Statuses` (teinte/particules par statut), sans état de
   rendu propre.
5. **Contenu testbed** : un `StatusDef` par statut dans `games/testbed/assets/` (champs `test:`
   et lint verts), une arme par statut qui le pose, et un scénario généré par statut
   (`status_burn`, `status_slow`, `status_stun`, `status_freeze`) qui utilise les deux attentes.
6. **Attentes** `HasStatus(entity, statut)` et `StatusStacks(entity, statut, n)` dans
   `crates/scenario` (sur le modèle des attentes de T1.1), ré-exportées par `game::replay`,
   avec un test unitaire chacune.

Tout nouvel état de simulation va en rollback + checksum + trace. Décisions de gameplay fixées
dans cette fiche, pas inventées.

## Règles cloud (spécifiques à une session cloud)

- **Clone froid** de `https://github.com/estran-studio/alacod`, branche `m1-v1a-statuts` créée
  depuis `origin/main`. Pas de worktree ni de target amorcé : compilation complète depuis zéro,
  c'est normal. `CARGO_BUILD_JOBS=4`.
- **Pas de bench** (machine différente : les chiffres ne sont pas comparables ; l'orchestrateur
  rejouera les benchs localement). Pas de p2p, pas de `make test_multiplayer`.
- **Ne jamais blesser une trace** (`BLESS=1` interdit). Le composant `Statuses` changera les
  traces : fournir la **preuve** (README §5) : dumps avant/après sur 2-3 scénarios clés
  (`idle`, `points_on_kill`, un scénario qui tire) et `scripts/trace-diff.py --ignore
  <nouveaux composants>` → le diff se réduit au nouveau composant. L'orchestrateur bénit.
- **Le push est refusé** depuis une session cloud (proxy git : seul le meta-repo est autorisé).
  À la fin, crée un bundle : `git bundle create m1-v1a-statuts.bundle origin/main..HEAD`, annonce
  son chemin dans ta réponse finale (il sera relayé via Syncthing). Si par chance le push passe,
  pousse aussi la branche — dans tous les cas, bundle.
- `docs/conventions.md` : ajouter un **§ numéroté distinct** (§19 « Statuts (T1.3) ») — le
  fichier est aussi modifié par d'autres branches : ne toucher qu'à ta section, et merger
  `origin/main` dans la branche juste avant de livrer.
- Une tâche à la fois : ne commencer aucune autre tâche de toi-même.

## Critères d'acceptation

1. Un test unitaire par statut (tick, expiration, empilement Burn, rafraîchissement
   Slow/Stun/Freeze, sources) ; chaque `StatusDef` testbed passe le lint et son `test:`.
2. Les quatre scénarios générés du testbed sont verts **hors traces à blesser** (avec la preuve
   du point 4, l'échec de trace attendu est documenté, pas caché).
3. `HasStatus` et `StatusStacks` sont utilisés par ces scénarios et testés unitairement.
4. Preuve `trace-diff` complète pour le bless à venir (voir règles cloud).
5. Tests des crates verts, `make lint`, `make fmt`, `make scripts`, `make gen` sans
   modification.

## Livrer

Rapport `docs/taches/rapports/m1-v1a-statuts.md` **sur la branche, commité** (format README §7,
sha de tête en en-tête). Crée le bundle git (règles cloud) et termine ta réponse finale par la
ligne :

```
LIVRÉ m1-v1a-statuts <sha> : <résumé une ligne>
```

William la relaiera à l'orchestrateur. Ne merge pas, ne bénis pas, n'attends pas de réponse
dans la même session.
