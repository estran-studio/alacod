# m1-v1a-patterns — patterns, émetteurs et tir ennemi (T1.2)

Lire d'abord `docs/taches/README.md` (agent **local** : la tâche compile, joue les scénarios et
le bench). Branche : `m1-v1a-patterns-tir-ennemi`, worktree
`alacod_tasks/m1-v1a-patterns-tir-ennemi/`, créés par l'orchestrateur (`scripts/task-new.sh`,
target amorcé, `source ../env.sh`).

## Contexte

M1, Vague 1, voie V1a combat (plan §6, chantier B5 v1, suite de T1.1). T1.1 (mergée `ea1b222`,
conventions §16) a implémenté les modificateurs de projectile et un `Pattern` **instantané et
sans aléa** : `pattern_shots(pattern, forward, aim) -> Vec<Shot>` (`crates/combat/src/projectile.rs`
l.433, `collect_shots` l.439-502), appelé une seule fois par `projectile_expire_system` pour
`ExpireAction::Spawn(Pattern)` ; `Telegraph(n)` et `Wait(n)` ne produisent rien (« réservés aux
émetteurs (T1.2) »), `Ring.every` est ignoré. L'enum `Pattern` (l.75) : `Aimed{count, spread,
projectile}`, `Spread{count, spread, projectile}`, `Ring{count, speed, projectile, every}`,
`Sequence(Vec<Pattern>)`, `Telegraph(u32)`, `Wait(u32)` ; miroir de lint `PatternEntry`
(`crates/content/src/registry.rs` l.193), `lint_expire_pattern` (`crates/content/src/lint.rs`
l.411) refuse les patterns temporels en `on_expire`.

Le tir aujourd'hui est **réservé aux joueurs** : `weapon_rollback_system`
(`crates/combat/src/weapons/mod.rs` l.983) itère les joueurs par `player.handle`, dispersion par
le flux RNG `"weapons"` (`RngStreams::get_mut`, `crates/bevy_fixed/src/rng.rs` l.36-44 : un flux
nommé est créé à la demande, graine `fnv1a(nom) ^ run_seed`), apparition par
`spawn_bullet_rollback` (l.690, privé, 19 paramètres) ; enfants de pattern par
`spawn_child_projectile` (projectile.rs l.557). Flux existants : `weapons`, `waves`, `loot`.
**Aucun flux `patterns`.**

Les ennemis attaquent **au corps à corps seulement** : `enemy_attack_system`
(`crates/game/src/character/enemy/ai/behavior.rs` l.322-480) passe `MonsterState::Attacking`
quand la cible `Player` est à moins de `attack_range` après `attack_cooldown_frames`
(`EnemyAiConfig`, `ai/state.rs` l.82-111, forme RON `EnemyAiConfigRon` l.272 tous champs
optionnels), la hitbox vient de `enemy_melee_attack_system` (`crates/combat/src/weapons/melee.rs`
l.613) avec l'arme `zombie_claws` que `spawn_enemy` (`enemy/create.rs` l.65-142) met dans
l'inventaire si `attack_range > 0`. Les ennemis **portent donc déjà une arme** par
`WeaponInventory`. Le set `EnemyAI` vient après `Weapon`/`Projectiles` dans
`RollbackSystemSet::ORDER` (`crates/sim_core/src/system_set.rs`). Les zombies de `games/zombies`
n'ont pas de champ `ai:` (repli `EnemyAiConfig::zombie()`). Le squelette `Behavior::Shoot(String)`
existe dans `crates/behaviors` (T1.0a) mais **T1.4 (behaviors composables, voie V1c) n'est pas
dans cette fiche** : ici l'ennemi tire par sa machine d'état actuelle.

Testbed : personnages de labo `games/testbed/assets/characters/*.ron` (`dummy`, `target` avec
`counts_hits`, `follower`, `ally`, `civilian`, `breacher`), placés par l'entité LDtk
`CharacterSpawn{character, team}` (conventions §1) dans `testbed/arena.ldtk` ; armes de démo
`proj_*` et `grenade` dans le `weapons.ron` du testbed avec `test:` ; scénarios `testbed_*`.

## Décisions (fixées ici, pas à réinventer)

1. **Émetteur** : composant rollback `Emitter` (crate `combat`, `rollback_and_trace_component`)
   posé sur une entité qui tire selon un pattern : pattern (référence `Arc` à la définition,
   hors checksum comme la table de projectiles), **état** (index dans la `Sequence`, frames
   restantes de l'étape, répétitions de `Ring.every` déjà faites, frame de départ, cible visée
   figée au départ de la séquence), tout en `Fixed`/entiers. Un système `emitter_system` dans le
   set `Weapon` (après `weapon_rollback_system`), `order_mut_iter!` par `GgrsNetId`, fait
   avancer chaque émetteur d'une frame et tire via **un helper partagé** extrait de
   `spawn_bullet_rollback` (publié dans `combat`, utilisé par les joueurs, par
   `spawn_child_projectile` et par les émetteurs : une seule fonction d'apparition de
   projectile). Le projectile tiré vient de la **table `projectiles` de l'arme portée** par
   l'émetteur (`WeaponInventory`, clé `projectile` du pattern), équipe = celle de l'émetteur
   (`Team`, règles de dégâts §8 : les balles ennemies touchent les joueurs, pas les ennemis, selon
   la politique de tir ami de l'arme).
2. **Sémantique des patterns** (étend §16 sans casser `on_expire`) :
   - `Aimed`, `Spread`, `Ring` : comme §16 pour une salve ; `Ring.every = n > 0` répète la
     couronne toutes les `n` frames tant que la séquence dure (`Ring` seul = infini, dans une
     `Sequence` = jusqu'à l'étape suivante — préciser au § conventions) ;
   - `Sequence([...])` : étapes dans l'ordre, une étape instantanée dure 0 frame ;
   - `Telegraph(n)` : `n` frames d'avertissement sans tir, l'état « télégraphe en cours » est
     lisible par la présentation (`Emitter::telegraphing() -> Option<frames_restantes>`) — le
     dessin (cercle au sol) est T1.17, ici au plus un gizmo de debug ;
   - `Wait(n)` : `n` frames sans tir ;
   - **nouvelle variante `Scatter{count, spread, projectile}`** : `count` tirs à des angles
     tirés au hasard dans `±spread/2` autour de la visée, via le **flux RNG `"patterns"`**
     (`RngStreams::get_mut("patterns")`), consommé **uniquement** par `emitter_system` dans
     l'ordre des `GgrsNetId` des émetteurs. `Scatter`, `Telegraph`, `Wait` sont refusés en
     `on_expire` par le lint (fixture), comme les temporels aujourd'hui.
   - À la fin de la séquence, l'émetteur est retiré (ou, pour un ennemi, repasse en
     refroidissement : voir 3).
   **Critère « même graine = même tir à 1 et à 4 joueurs »** : le flux `patterns` n'est consommé
   par rien d'autre, donc la suite de tirs d'un émetteur ne dépend que de la graine de run et de
   l'ordre des émetteurs — pas du nombre de joueurs. Test unitaire : deux mondes, 1 et 4
   joueurs, même graine, un émetteur `Scatter` → mêmes angles ; et scénarios 5.
3. **Tir ennemi** : nouveau champ RON **optionnel** dans la config IA du personnage
   (`EnemyAiConfigRon`, conventions §2 pour les `Fixed`) :
   `ranged: Some((weapon: "fireball_gun", pattern: "ring_8", range: "260.0",
   cooldown_frames: 90))`. `spawn_enemy` équipe alors `weapon` (en plus ou à la place de
   `zombie_claws` selon `attack_range`). `enemy_attack_system` : cible `Player` à moins de
   `range` et refroidissement écoulé → pose un `Emitter` avec le pattern (visée = position de la
   cible au départ), `MonsterState::Attacking{target: Player}` ; **l'ennemi ne bouge pas**
   pendant télégraphe et tir (`KeepDistance`/`Strafe` = T1.4) ; fin de séquence → retour à
   `Chasing`, refroidissement. **Sans champ `ranged`, rien ne change** : zombies et traces
   intactes (le `Hash` manuel de `EnemyAiConfig` inclut le nouveau champ, mais il est `None` pour
   tous les personnages existants ; vérifier que la valeur de hash d'un `None` ne modifie pas le
   checksum existant — sinon, preuve `trace-diff` et le dire).
4. **Patterns nommés = kind de contenu** : `(path: "patterns", kind: "Pattern")` dans
   `game.ron` (conventions §3, checklist §4) : fichiers `games/<jeu>/assets/patterns/<nom>.ron`
   contenant un `Pattern` ; `ranged.pattern` les référence par nom ; `on_expire` continue
   d'accepter un `Pattern` inline (§16) **et** accepte `Named("nom")`. Lint : nom inconnu,
   `projectile` absent de la table de l'arme, `Scatter`/temporels en `on_expire`, `cooldown_frames`
   = 0, `range` ≤ 0 ; fixtures `crates/content/tests/fixtures/<erreur>/`.
5. **Contenu et scénarios testbed** (nouvelle arène `testbed/arena_tir.ldtk`, copie d'`arena`
   avec deux `CharacterSpawn` — ne pas toucher `arena.ldtk`, ses scénarios gardent leurs traces) :
   - arme `fireball_gun` (testbed `weapons.ron`, table `projectiles` avec `fireball`, `test:`
     et lint verts) ;
   - personnages `turret` (`stationary`, `ranged` → `ring_8` : `Ring{count: 8, every: 60}`) et
     `archer` (`ranged` → `volee` : `Sequence([Telegraph(30), Aimed{count: 3, spread: "0.3"},
     Wait(20), Scatter{count: 4, spread: "0.6"}])`) ;
   - `enemy_ring` : un joueur immobile face à la tourelle ; `BulletCount` = 8 après la première
     couronne, 16 après la deuxième (avant expiration), `Health` du joueur qui baisse ;
   - `enemy_ring_quad` : **la même arène à 4 joueurs**, mêmes frames, mêmes `BulletCount`
     (même graine = même tir quel que soit le nombre de joueurs) ;
   - `enemy_telegraph` : l'archer ; `BulletCount` = 0 pendant le télégraphe, 3 après `Aimed`,
     7 après `Scatter` (avant expiration) ; `HitsAtLeast` ou `Health` sur le joueur.
   Trois nouvelles traces = pas de preuve requise ; bénies par l'orchestrateur.
6. **Hors périmètre** : behaviors composables et `KeepDistance` (T1.4), parade/renvoi (B5 v2),
   dette **D27** (la dispersion joueur ignore `spread` : la corriger change toutes les traces de
   tir — **ne pas y toucher**), visuel du télégraphe (T1.17), bots qui esquivent (T1.14),
   `games/throne` (T1.0c). Le testbed garde `weapon_slots` et ses personnages existants intacts.

## Déterminisme (rappel, `CLAUDE.md`)

`Fixed` partout dans `GgrsSchedule`, `BTreeMap`, `order_iter!`/`order_mut_iter!` avec
`&GgrsNetId` en tête, RNG par `RngStreams` (flux `patterns`, jamais `rand`), `despawn_rollback()`,
`FrameEvents<T>`, enregistrement par `rollback_and_trace_*` (`crates/utils/src/rollback.rs`),
logs `ggrs{f=… emitter net_id=… step=…}` avec `GgrsNetId`. Les projectiles ennemis passent par la
même chaîne `Projectiles` que ceux des joueurs (collision, `on_hit`, expiration, 8 générations).

## Règles du worktree

- Une compilation à la fois pour toi, deux au plus sur la machine (b1 travaille en parallèle
  sur la voie V1d) : `pgrep -x cargo` avant de lancer, `--profile headless`,
  `CARGO_BUILD_JOBS=4`, `source ../env.sh`, jamais depuis un target vide. Dumps `.full` de
  300 Mo à 1 Go : à supprimer au fur et à mesure (disque serré).
- **Aucune trace bénie par l'agent.** Attendu : les 80 traces existantes **inchangées**
  (composant `Emitter` absent des entités existantes, champ `ranged` à `None`, flux `patterns`
  jamais consommé sans émetteur). Sinon : preuve `trace-diff` (README §5) + justification dans le
  commit ; l'orchestrateur bénit.
- `docs/conventions.md` : ajouter **uniquement** un §20 « Patterns, émetteurs et tir ennemi »
  (§19 statuts et §21 cavernes sont pris par d'autres branches) ; compléter §16 seulement par
  un renvoi d'une ligne vers §20. Mettre à jour la liste des attentes de `CLAUDE.md` seulement
  si tu en ajoutes une (aucune n'est demandée). `docs/taches.md` : ne pas toucher.
- Merger `origin/main` dans la branche juste avant de livrer (règle m0-v9) ; en cas de conflit,
  garder les deux côtés (c3 « statuts » touche aussi `combat`, `effects` et le testbed). Ne pas
  merger dans main, ne pas supprimer le worktree.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : un par variante d'émetteur (`Ring.every`, `Sequence`, `Telegraph`, `Wait`,
   `Scatter`), « même graine à 1 et 4 joueurs », helper d'apparition partagé (une balle joueur et
   une balle d'émetteur passent par la même fonction), lint des nouveaux cas (fixtures).
2. `enemy_ring`, `enemy_ring_quad`, `enemy_telegraph` verts avec les attentes du point 5 ; les
   `BulletCount` de `enemy_ring` et `enemy_ring_quad` sont **identiques aux mêmes frames**.
3. Les 80 scénarios existants verts, **traces inchangées** (ou preuve jointe) ; `bench_bullets`
   reste dans son budget (`ALACOD_BENCH_STRICT=1`, chiffre dans le rapport, machine calme :
   vérifier qu'aucune sim de b1 ne tourne).
4. Suite des crates verte, `make lint` (deux jeux, kind `Pattern` déclaré par le testbed ;
   `games/zombies` n'a pas besoin de dossier `patterns`), `make fmt`, `make scripts`,
   `make gen GAME=zombies` et `GAME=testbed` sans modification des scénarios existants (le
   `fireball_gun` est une arme d'ennemi : exclue du générateur `WeaponOnTarget`, ou avec un
   scénario généré vert qui est alors une 4e nouvelle trace — à dire dans le rapport).
5. §20 écrit ; rapport honnête.

## Livrer

Rapport `docs/taches/rapports/m1-v1a-patterns-tir-ennemi.md` sur la branche (README §7 : fait /
vérifié avec chiffres / non fait / dettes ; sha de tête, base `origin/main`). Commits en
français avec attribution. `git push -u origin m1-v1a-patterns-tir-ennemi`, puis `SendMessage`
à `orch` : `LIVRÉ m1-v1a-patterns-tir-ennemi <sha> : <une ligne>` (ou `BLOQUÉ … : <pourquoi>`
après 30 minutes de blocage). Ne merge pas, ne bénis pas, attends la réponse.
