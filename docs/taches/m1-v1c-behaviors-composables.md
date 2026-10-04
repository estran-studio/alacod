# m1-v1c-behaviors — behaviors composables v1 (T1.4) + attentes `EnemyState`/`EnemyDistance`

Lire d'abord `docs/taches/README.md` (agent **local** : la tâche compile, joue les scénarios et
le bench). Branche : `m1-v1c-behaviors-composables`, worktree
`alacod_tasks/m1-v1c-behaviors-composables/`, créés par l'orchestrateur (`scripts/task-new.sh`,
target amorcé, `source ../env.sh`). **Prérequis mergés** : T1.2 (émetteurs, `ai.ranged`,
`PatternLibrary`, conventions §20) et m0-v7 p2 (bots chasseur/acheteur, secours de vague,
snap de la salle de spawn).

## Contexte

M1, Vague 1, voie V1c ennemis (plan §6, chantier D1 : « remplace `behavior.rs` et `pathing.rs` ;
les zombies actuels deviennent un fichier »). T1.0a (`51d70b6`) a posé le contrat dans
`crates/behaviors/src/lib.rs` : `Behavior { Chase{profile}, KeepDistance{min,max}, Strafe,
Charge{telegraph}, Shoot(String), Melee(String), Flee, Wander }`, `Perception { Sight(Fixed),
Hearing(Fixed) }` + `PerceptionConfig`, `Targeting::Nearest{ignore: Vec<Tag>}`, composant
`BehaviorState { selected_rule, since_frame, target }` **enregistré sous checksum ordinaire et
porté par personne** (il compte donc dans la parité actuelle des types vides, CLAUDE.md
checklist) ; kinds `behavior`/`perception`/`targeting` ; **aucun système**.

L'IA actuelle (`crates/game/src/character/enemy/ai/`) : `MonsterState { Idle, Chasing,
Attacking{target, last_attack_frame}, Stunned, Breaching, Fleeing, Dead }` (`state.rs`),
`EnemyAiConfig` (`movement_type`, `aggro_range`, `attack_range`, `attack_cooldown_frames`,
`can_break`, `attack_through`, `ignores`, `path_through_breakables`, `flee_threshold`,
`attack_damage`, `friendly_fire`, `stationary`, et depuis T1.2 `ranged: Option<RangedAttack>`),
préréglages `zombie()`/`flying()`/`ghost()`/`tank()`. Set `EnemyAI` (`character/mod.rs`) :
`enemy_target_selection` (`behavior.rs` : joueur le plus proche **par le chemin** via
`FlowFieldCache::owners`, repli en ligne droite, joueurs à terre ignorés, fenêtre sur la route
ciblée si à portée), `update_enemy_targets` + `move_enemies` (`pathing.rs` : flow field,
`steering_point`, séparation, portes), `enemy_attack_system` (mêlée avec refroidissement,
obstacles via `ObstacleAttackEvent`, et depuis T1.2 `ranged_attack` qui pose un `Emitter`),
`enemy_attack_damage_translate_system`. **Code mort constaté** (b0, lecture) :
`enemy_movement_system`, `enemy_stun_recovery_system`, `apply_stun` ne sont enregistrés nulle
part ; `Fleeing`, `Breaching`, `Stunned` ne sont jamais posés ; `flee_threshold` n'est lu nulle
part ; deux ciblages font doublon. Les zombies de `games/zombies` n'ont pas de champ `ai:`
(repli `zombie()`) ; `zombie_claws` est codé en dur dans `spawn_enemy`.

Les attentes `EnemyState` et `EnemyDistance` (liste T1.15) n'existent pas : **cette fiche les
revendique**, plus deux diagnostics de navigation (`EnemyContactBefore`, `EnemyNeverInWall`).

## Décisions (fixées ici après revue avec b0, pas à réinventer)

1. **Traces identiques au bit près, sans bless.** Les zombies et les personnages existants ne
   portent **aucun nouveau composant** et n'en perdent aucun : leur liste `behaviors:` se
   **compile vers l'état existant** (`EnemyAiConfig`, `MonsterState`, `EnemyTarget`,
   `EnemyPath`…). Les behaviors nouveaux (`KeepDistance`, `Strafe`, `Charge`, `Flee`, `Wander`)
   gardent leur état dans des composants **neutres** (`rollback_and_trace_neutral::<C>`, T1.2)
   posés **seulement** sur les personnages qui les listent. L'enregistrement de `BehaviorState`
   reste **tel quel** (le passer en neutre, le retirer ou le poser déplacerait les 82 traces par
   parité) ; il n'est posé sur personne en v1 et le § conventions le dit. Résultat vérifiable :
   `make test_scenarios` vert sans aucune trace changée, `alacod-sim` 20 graines identiques à
   main.
2. **Un seul chemin de code, sélection par priorité.** Un personnage sans `behaviors:` reçoit
   au chargement la **liste par défaut dérivée de son `EnemyAiConfig`** (exactement :
   `[Flee]` si `flee_threshold` est `Some`, puis `Shoot{…}` si `ranged` est `Some`, puis
   `Melee("zombie_claws")` si `attack_range > 0`, puis `Chase{profile: <movement_type>}` sauf
   si `stationary`) ; avec `behaviors:`, sa liste. L'ordre RON est la priorité ; chaque behavior
   a une **applicabilité implicite** (pas de `when:` en v1) et la première applicable gagne, à
   chaque frame, dans `RollbackSystemSet::EnemyAI`, `order_mut_iter!` par `GgrsNetId` :
   - `Melee(arme)` : cible (joueur, ou obstacle cassable sur la route si `can_break`) à portée
     de l'arme ; exécution = l'attaque de mêlée actuelle, l'arme devient une donnée
     (`zombie_claws` n'est plus codé en dur).
   - `Shoot { weapon, pattern, range, cooldown_frames }` : cible vivante, debout, à moins de
     `range`, refroidissement écoulé ; exécution = `ranged_attack` de T1.2 (pose l'`Emitter`).
     **Remplace `ai.ranged`** (T1.0a disait `Shoot(String)` : le contrat change, c'est permis) ;
     `turret`/`archer` migrent ; les 4 traces de T1.2 restent identiques.
   - `Charge { telegraph }` : cible entre `attack_range` et 3 × `attack_range` ; `telegraph`
     frames immobile (lisible par la présentation comme le télégraphe d'émetteur, T1.17
     dessinera), puis ruée en ligne droite vers la position **figée** de la cible à 3 × la
     vitesse jusqu'au contact ou 60 frames, murs respectés (arrêt) ; refroidissement
     `attack_cooldown_frames`.
   - `KeepDistance { min, max }` : cible connue à moins de `min` → recule (flow field inversé :
     case voisine de coût le plus élevé, départage par `GridPos`) ; la bande `[min, max]` fait
     l'hystérésis, pas de mécanisme générique.
   - `Flee` : santé ≤ `flee_threshold` × max ; recule comme ci-dessus ; **sans** ressusciter
     `MonsterState::Fleeing`.
   - `Strafe` : cible à moins de `Sight` ; déplacement perpendiculaire à la cible, sens alterné
     toutes les 45 frames (parité de `frames_depuis_entrée / 45`), vitesse normale, murs
     respectés.
   - `Chase { profile }` : cible connue ; exécution = `move_enemies` actuel avec le
     `NavProfile` nommé.
   - `Wander` : toujours applicable (règle de fond) ; direction tirée dans le **flux RNG
     `"behaviors"`** toutes les 60 frames (consommé uniquement par `Wander`, ordre
     `GgrsNetId`), vitesse moitié, murs respectés.
   Une règle absente n'existe pas pour ce personnage.
3. **Code mort supprimé** (sans effet sur les traces : aucun type enregistré ne bouge) :
   `enemy_movement_system`, `enemy_stun_recovery_system`, `apply_stun`, les variantes
   `Fleeing`/`Breaching`/`Stunned` jamais posées (vérifier qu'aucun `Hash` manuel ni aucune
   sérialisation ne dépend des index de variantes avant de les retirer — sinon les garder et le
   dire), le ciblage en doublon (`update_enemy_targets` ou `enemy_target_selection`, garder
   celui qui produit l'état actuel, prouvé par les traces). L'étourdissement est un statut
   (T1.3, c3), pas un `MonsterState`.
4. **Perception et ciblage, identiques pour les zombies.** `perception` optionnel : `Sight(r)`
   = rayon seul, sans ligne de vue ni lumière (c'est `aggro_range` ; repli `Sight(aggro_range)`) ;
   `Hearing(r)` : un projectile né à moins de `r` rend le tireur **connu** 120 frames même hors
   de vue (événement d'apparition existant du set `Weapon`) ; `needs_light` ignoré (E8).
   `targeting` optionnel, repli `Nearest { ignore: [] }` = l'algorithme actuel (plus proche par
   le flow field, repli en ligne droite, joueurs à terre ignorés) ; `ignore` exclut par `Tag`.
5. **Bug préservé, dette** : le dégât direct `attack_damage` ne s'applique presque jamais
   (hérité de T1.1) ; on le garde tel quel (traces identiques), `Melee` = hitbox de l'arme
   seulement ; nouvelle ligne **D28** dans `dettes.md` (la seule ligne que la tâche y ajoute).
6. **Attentes** (crate `scenario`, modèle `FloorIndex`/`BulletCount`, ré-export `game::replay`,
   liste `CLAUDE.md`, un test unitaire chacune) :
   - `EnemyState { entity, behavior: "Chase", at_frame }` : le behavior retenu (nom de
     variante) pour un ennemi désigné par net id ou par `Target` comme `HitsAtLeast` ;
   - `EnemyDistance { entity, target: Player(h), min, max, at_frame }` ;
   - diagnostics de navigation : `EnemyContactBefore { entity, frames }` (l'ennemi atteint la
     portée de mêlée d'un joueur avant la frame donnée) et `EnemyNeverInWall { entity, from, to }`
     (son collider ne chevauche aucun `Wall` sur l'intervalle ; attente continue comme
     `NoDamageBetween`).
7. **Contenu et scénarios testbed** (nouvelle arène `testbed/arena_ia.ldtk`, copie d'`arena`
   avec `CharacterSpawn` dédiés ; `arena.ldtk` et `arena_tir.ldtk` intacts) : un personnage et
   un scénario par behavior nouveau : `kiter` (`[Shoot{fireball_gun, volee, …},
   KeepDistance(min: "120", max: "200"), Chase]`) → `enemy_keep_distance` (`EnemyDistance`
   dans [110, 210] à trois frames, `EnemyState` `KeepDistance` puis `Shoot`) ; `charger`
   (`[Charge(telegraph: 30), Chase]`) → `enemy_charge` (`EnemyState` `Charge`, `Health` du
   joueur qui baisse, `EnemyContactBefore`) ; `coward` (`[Flee, Melee("zombie_claws"), Chase]`,
   `flee_threshold` 0,5) → `enemy_flee` (`EnemyState` `Flee`, `EnemyDistance` qui croît) ;
   `drifter` (`[Strafe, Wander]`) → `enemy_wander` (`EnemyState`, `EnemyNeverInWall`, position
   qui change). Quatre nouvelles traces, pas de preuve requise (bless orchestrateur).
   `games/zombies/.../zombie_config.ron`, `zombie_full_config.ron`, `zombie_hard_config.ron`
   reçoivent `ai: Some((... behaviors: [Melee("zombie_claws"), Chase(profile: "GroundBreaker")]))`
   explicite (valeurs du préréglage recopiées) : c'est « les zombies deviennent un fichier »,
   traces identiques.
8. **Lint** (`crates/content`) : `Shoot` arme/pattern inconnus ou projectile absent de la table
   (reprend les règles `ranged` de T1.2), `Melee(arme)` inconnue, `Chase` profil inconnu,
   `KeepDistance` min ≥ max, `Charge` télégraphe 0, liste vide, tag d'`ignore` inconnu ; fixtures.
9. **Hors périmètre** : variantes et élites (T1.5), formations/patrouilles/escorte (D1 v2),
   boss (D4), `needs_light`/bruit (E8), bots joueurs (T1.14), réécriture de la navigation
   (`pathing.rs` devient un exécuteur, renommer est permis, réécrire non).

## Déterminisme (rappel, `CLAUDE.md`)

`Fixed`, `BTreeMap`, `order_mut_iter!` avec `&GgrsNetId` en tête, départage par net id, RNG
par `RngStreams` (flux `behaviors`, jamais `rand`), `despawn_rollback()`, `FrameEvents<T>`,
enregistrement par `rollback_and_trace_*` (**neutre** pour tout nouveau composant), logs
`ggrs{f=… behavior net_id=… rule=…}`. Checklist « piège de parité » : ne retire ni n'ajoute
aucun type **ordinaire** sous checksum.

## Règles du worktree

- Deux compilations au plus sur la machine (`pgrep -x cargo`), `--profile headless`,
  `CARGO_BUILD_JOBS=4`, `source ../env.sh`, jamais depuis un target vide ; générations
  périmées de `target/headless/build/scenario` purgées entre deux runs (disque serré).
- **Aucune trace bénie par l'agent**, et **aucune trace existante ne doit changer** : c'est le
  critère central. Si une trace change, c'est un bug à corriger, pas une preuve à fournir.
- `docs/conventions.md` : ajouter **uniquement** un §22 « Behaviors composables » (§19 statuts,
  §20 patterns, §21 cavernes sont pris) et retoucher §20 d'une ligne (`ai.ranged` → `Shoot`) ;
  liste des attentes de `CLAUDE.md` : ajouter les quatre. `docs/taches.md` : ne pas toucher ;
  `dettes.md` : seulement D28.
- Merger `origin/main` juste avant de livrer ; conflits : garder les deux. Ne pas merger dans
  main, ne pas supprimer le worktree.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : liste par défaut dérivée de chaque préréglage (`zombie`, `flying`,
   `ghost`, `tank`), sélection par priorité (table de cas), chaque règle (applicabilité,
   exécution), `Wander` déterministe (même graine ⇒ mêmes directions), les quatre attentes,
   fixtures de lint.
2. Les 4 scénarios testbed verts ; **les scénarios existants verts sans aucune trace
   modifiée** (dont les 4 de T1.2 après la migration `turret`/`archer`) ; `alacod-sim`
   20 graines : mêmes vagues/frames/kills que main (tableau dans le rapport).
3. `bench_horde` dans son budget (`ALACOD_BENCH_STRICT=1`, machine calme, chiffre dans le
   rapport : le sélecteur coûte par ennemi et par frame, il doit rester marginal).
4. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` pour les
   deux jeux sans modification des scénarios existants.
5. §22 écrit, D28 ajoutée ; rapport honnête (code mort retiré listé).

## Livrer

Rapport `docs/taches/rapports/m1-v1c-behaviors-composables.md` sur la branche (README §7 ; sha
de tête, base `origin/main`). Commits en français avec attribution. `git push -u origin
m1-v1c-behaviors-composables`, puis `SendMessage` à `orch` : `LIVRÉ m1-v1c-behaviors-composables
<sha> : <une ligne>` (ou `BLOQUÉ … : <pourquoi>` après 30 minutes de blocage). Ne merge pas, ne
bénis pas, attends la réponse.
