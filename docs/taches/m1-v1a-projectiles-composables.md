# m1-v1a — projectiles composables (T1.1) + attentes `BulletCount`/`HitsAtLeast`

## Contexte

M1, Vague 1, voie V1a combat (plan §6, B5 v1). T1.0a (`51d70b6`) a créé les contrats : les
kinds `ProjectileModifier` (Bounce, Pierce, Size, Lifetime, Homing, Gravity), `Pattern` (Aimed,
Spread, Ring, Sequence, Telegraph, Wait) et `StatusDef` existent et sont enregistrés
(`Kinds`/`KindRegistry`), mais **aucun n'est implémenté ni posé** sur le moindre projectile.
T1.1 implémente les modificateurs de projectile pour de vrai. Les attentes `BulletCount` et
`HitsAtLeast` (liste T1.15) n'existent pas encore : **cette fiche les revendique**.

## Périmètre

1. **Les six modificateurs** sur le cycle de vie du projectile (crate `combat`, chantier B5 v1) :
   - `Bounce(n)` : n rebonds sur les murs (n = 0 → aucun) ;
   - `Pierce(n)` : traverse n ennemis avant de disparaître ;
   - `Size` : facteur de rayon (collision comprise) ;
   - `Lifetime` : durée de vie en frames, expiration propre ;
   - `Homing(force)` : vire vers la cible la plus proche (choix déterministe : tri par
     `GgrsNetId`) ;
   - `Gravity` : accélération verticale constante.
2. **`on_hit: [Action]`** (actions existantes de la crate `effects`) et
   **`on_expire: [Spawn(pattern)]`** ; une **explosion** = projectile à durée nulle dont
   `on_expire` spawn un pattern.
3. **Contenu testbed** : une arme de démonstration par modificateur dans
   `games/testbed/assets/weapons.ron` (champs `test:` et lint verts, comme les armes
   existantes), plus une arme « grenade » (explosion).
4. **Scénario généré** dans le testbed qui exerce au moins Bounce, Pierce et une explosion, et
   qui utilise les deux attentes.
5. **Attentes** `BulletCount(n)` (nombre de projectiles vivants d'un type/faction à une frame
   donnée) et `HitsAtLeast(entity, n)` (le compteur `HitCount` de la cible ≥ n, voir
   `crates/combat/src/weapons/expectations.rs`) dans `crates/scenario`, avec un test unitaire
   chacune.

Tout nouvel état de simulation va en rollback + checksum + trace (`RollbackTraceApp`, jamais
d'appel direct hors de `crates/utils/src/rollback.rs`). Décisions de gameplay fixées dans la
fiche, pas inventées.

## Règles cloud (spécifiques à une session cloud)

- **Clone froid** de `https://github.com/estran-studio/alacod`, branche
  `m1-v1a-projectiles-composables` créée depuis `origin/main`. Pas de worktree ni de target
  amorcé : compilation complète depuis zéro, c'est normal. `CARGO_BUILD_JOBS=4`.
- **Pas de bench** (machine différente : les chiffres ne sont pas comparables ; l'orchestrateur
  rejouera `bench_bullets` localement). Pas de p2p, pas de `make test_multiplayer`.
- **Ne jamais blesser une trace** (`BLESS=1` interdit). Les nouveaux composants rollback
  changeront les traces : fournir la **preuve** (README §5) : dumps avant/après sur 2-3
  scénarios clés (`idle`, `points_on_kill`, un scénario qui tire) et
  `scripts/trace-diff.py --ignore <nouveaux composants>` → le diff se réduit au nouveau
  composant. L'orchestrateur bénit.
- `docs/conventions.md` : ajouter un **§ numéroté distinct** (§16 « Projectiles composables ») —
  le fichier est aussi modifié par d'autres branches (m0-v7, m0-v10) : ne toucher qu'à ta
  section, et merger `origin/main` dans la branche juste avant de livrer.
- Une tâche à la fois : ne commencer aucune autre tâche de toi-même.

## Critères d'acceptation

1. Un test unitaire par modificateur (au moins) ; chaque arme testbed passe le lint et son
   `test:`.
2. Le scénario généré du testbed est vert **hors traces à blesser** (avec la preuve du point
   suivant, l'échec de trace attendu est documenté, pas caché).
3. `BulletCount` et `HitsAtLeast` sont utilisés par ce scénario et testés unitairement.
4. Preuve `trace-diff` complète pour le bless à venir (voir règles cloud).
5. Tests des crates verts, `make lint`, `make fmt`, `make scripts`, `make gen` sans
   modification.

## Livrer

Rapport `docs/taches/rapports/m1-v1a-projectiles-composables.md` **sur la branche, commité et
poussé** (format README §7, sha de tête en en-tête). Pousse la branche. Termine ta réponse
finale par la ligne :

```
LIVRÉ m1-v1a-projectiles-composables <sha> : <résumé une ligne>
```

William la relaiera à l'orchestrateur. Ne merge pas, ne bénis pas, n'attends pas de réponse
dans la même session.
