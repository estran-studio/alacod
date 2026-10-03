# Rapport m1-v1a-patterns — patterns, émetteurs et tir ennemi (T1.2)

**Branche** `m1-v1a-patterns-tir-ennemi`, partie de `main` `58db32c` ; base à la livraison :
_(complété au merge final d'`origin/main`)_.

## 1. Fait

### Checksum neutre pour les composants (et le piège de parité, vérifié)

- **Constat** (bevy_ggrs 0.22, `ComponentChecksumPlugin`) : un type de composant enregistré
  avec checksum mais porté par **aucune** entité contribue une `ChecksumPart` constante
  `K = hash(0u64)`, **identique pour tous les types vides** ; les parts sont combinées par XOR.
- **Vérifié sur `idle`** (sondes temporaires, non commitées) :
  - +1 type vide (`rollback_and_trace::<Sonde>()`) : `trace différente de la référence à la
    ligne 1` (attendu `…ed0b956eef6ad74e`, obtenu `…f1d5703c4b07ae11`) ;
  - +2 types vides : trace **identique**, test vert.
  Seule la parité du nombre de types vides compte : c'est ce qui a permis à T1.1 d'ajouter
  `Projectile` sans re-bless zombies, et ce qui a forcé T2.9 à passer `HitCount` hors checksum.
- **Réponse** : `RollbackTraceApp::rollback_and_trace_neutral::<C>()`
  (`crates/utils/src/rollback.rs`) — même hachage par entité que bevy_ggrs (ordre
  `RollbackOrdered`, XOR, repli final), mais `ChecksumPart(0)` (neutre) quand aucune entité ne
  porte le type. `Emitter` et `RangedAttackState` passent par là : vérifiés au checksum dès
  qu'ils existent, sans toucher aux traces existantes quelle que soit la parité.
- Écrit dans `docs/conventions.md` §20 (« Piège pour toute voie »).

### Émetteurs (`crates/combat/src/emitter.rs`)

- `Emitter` (composant rollback, checksum neutre) : pattern résolu et aplati (`Arc`, hors
  checksum : représenté par son nom), arme et table `projectiles` (`Arc`, par ses clés),
  multiplicateur `Damage`, tir ami, **visée figée au départ**, frame de départ, étape
  suivante, frames restantes de l'étape bloquante, drapeau télégraphe, couronnes de fond
  (`RingRepeat` : étape, prochaine frame, salves tirées), `finished`. `Hash`/`Debug` manuels.
- `Emitter::tick(frame, random)` : avancée **pure** d'une frame (testable sans Bevy) ;
  `Emitter::telegraphing() -> Option<frames restantes>`.
- `emitter_system` : set `Weapon`, après tout tir de joueur et de mêlée, `order_mut_iter!`
  par `GgrsNetId` ; projectiles au centre du tireur via le helper partagé ; retire l'émetteur
  à la fin de sa séquence. Le flux `"patterns"` n'est créé et consommé que par un `Scatter`
  qui tire.
- Sémantique (Q2 de l'orchestrateur) : un `Ring(every > 0)` dans une `Sequence` tourne en
  tâche de fond jusqu'à la fin de la séquence ; seul, il est infini. Détails §20.

### Une seule fonction d'apparition (`weapons::spawn_bullet`)

Extraite de `spawn_bullet_rollback` : `BulletSpawn` (position, rotation, vitesse, type,
dégât et portée finaux, tireur, équipe, tags, tir ami, projectile, préfixe d'id). Rayon,
sprite, couleur et marqueurs dérivent du type de balle et du projectile, comme avant.
Appelants : tir de joueur (`spawn_bullet_rollback`, qui ne calcule plus que la bouche du
canon), `projectile::spawn_child_projectile`, `emitter_system`. Les balles de joueur et les
projectiles nés sont identiques au bit près (vitesses calculées par chaque appelant comme
avant, mêmes composants, même ordre d'allocation des `GgrsNetId`) — vérifié par les traces.
Seul changement visible : le log d'apparition nomme le tireur par son `GgrsNetId` (avant : le
handle du joueur), et `NO_PLAYER_HANDLE` (tir d'émetteur) n'apparaît dans aucun log.

### Patterns (`crates/combat/src/projectile.rs`)

- Variantes `Scatter { count, spread, projectile }` et `Named(String)` ajoutées **en fin
  d'enum** (le `derive(Hash)` hache l'index de variante : hash des armes et projectiles
  existants inchangé). Kinds `pattern` : 8.
- `PatternLibrary` (ressource hors rollback) et `resolve_pattern` : résolution récursive des
  `Named`, profondeur bornée (cycle → erreur, jamais de boucle). Utilisée au départ d'un
  émetteur et à l'expiration d'un projectile (`on_expire: [Spawn(Named("..."))]`).
- **Correctif latent de T1.1** : `direction_of` ramène dans `[-2π, 2π]` un angle qui en sort.
  Le CORDIC de `fixed_math` perd sa précision puis **panique (débordement `fixed`)** au-delà :
  une couronne dont le premier rayon part à π/2 (tourelle visant vers le haut) atteint
  7,07 rad au 8e rayon. Aucun contenu existant n'atteint 2π (couronnes d'explosion partant de
  l'axe +x, au plus 7τ/8) : valeurs inchangées au bit près (traces vérifiées). Test
  `couronne_visant_vers_le_haut_ne_deborde_pas`.

### Tir ennemi (`crates/game`)

- `EnemyAiConfig::ranged: Option<RangedAttack>` (forme RON `RangedAttackRon`, `range` en
  chaîne) ; `Hash` manuel : `ranged` haché **seulement s'il est présent** (hacher un `None`
  ajoute des octets et déplacerait le checksum de tous les ennemis existants).
- `RangedAttackState { target, ready_at }` (composant rollback, checksum neutre), posé par
  `spawn_enemy` sur les seuls personnages à `ranged`, qui équipe aussi l'arme (active, dans
  `WeaponInventory`, via `spawn_weapon_for_player`). **Correction de la fiche** : l'inventaire
  des ennemis existants est vide — la griffe est une arme de corps à corps enfant
  (`spawn_melee_weapon_for_character`), hors `WeaponInventory`.
- `behavior::ranged_attack`, appelé par `enemy_attack_system` **avant** le corps à corps :
  départ (cible `Player` vivante, debout, à moins de `range`, refroidissement écoulé, état
  `Idle`/`Chasing`) → `Emitter` posé, `Attacking { target: Player }` ; arrêt (Q3) si la
  cible meurt, passe à terre ou sort de `range` (tireur mort : l'émetteur part avec lui) ;
  fin ou arrêt → refroidissement, `Chasing`. `move_enemies` immobilise l'ennemi tant qu'un
  émetteur est posé. `enemy_attack_damage_translate_system` ignore un `Attacking` posé par un
  tir (aucun coup direct de corps à corps).
- `game::patterns` : `PatternLibrary` construite depuis le registre de contenu à
  `OnEnter(GameLoading)` (à côté de l'équilibrage F5).

### Contenu (`crates/content`) et lint

- Kind `Pattern` (`patterns/<nom>.ron`, id = nom de fichier) ; `PatternEntry` étendu
  (`Scatter`, `Named`) ; `CharacterEntry::ranged` (lu dans `ai.ranged`).
- Règles : `Scatter`/temporels refusés en `on_expire` (y compris au travers d'un `Named`),
  pattern nommé inconnu, cycle de `Named`, valeurs d'un pattern nommé, `ai.ranged` (arme
  inconnue, pattern inconnu, projectile du pattern absent de la table de l'arme,
  `cooldown_frames = 0`, `range <= 0`). Fixtures `pattern_unknown_name`,
  `pattern_scatter_on_expire`, `ranged_projectile_missing`, `ranged_cooldown_zero`.

### Testbed et scénarios

- `fireball_gun` (`weapons.ron` du testbed, table `fireball` et `arrow`, `test:`), patterns
  `ring_8` (`Ring(count: 8, speed: "60.0", every: 60)`) et `volee` (`Sequence([Telegraph(30),
  Aimed(count: 3, spread: "0.3"), Wait(20), Scatter(count: 4, spread: "0.6")])`),
  personnages `turret` (immobile) et `archer` (poursuit), kind `Pattern` déclaré par
  `games/testbed/assets/game.ron` (zombies n'en a pas besoin).
- `testbed/arena_tir.ldtk` : copie d'`arena` (identifiants régénérés, uuid5 déterministes),
  tourelle au centre de la partie ouverte de la salle, archer au sud-est ; joueur 0 à
  96 unités à gauche de la tourelle, joueurs 1 à 3 plus loin et entre deux rayons de la
  couronne. `arena.ldtk` n'est pas touché.
- Scénarios (frames mesurées) :
  - `enemy_ring` : couronnes à f1, f61, f121 ; `BulletCount(fireball)` = 0 à f0, **8 à f2**,
    8 à f60, **16 à f62** ; `Health(0) <= 99` à f90 (la boule de feu visée touche vers
    f82) ; joueur vivant à f130.
  - `enemy_ring_quad` : la même partie à 4 joueurs, **mêmes attentes aux mêmes frames**,
    toutes vertes ; la tourelle vise le joueur 0 dans les deux cas.
  - `enemy_telegraph` : volée de l'archer, `BulletCount(arrow)` = **0 à f15 et f30**
    (télégraphe), **3 à f32 et f50** (après `Aimed`), **7 à f52** (après `Scatter`) ;
    `Health(0) <= 99` à f140 ; nouvelle volée à f141 (refroidissement de 90 après f51).
  - Attente sur le joueur : `Health` (les joueurs ne comptent pas leurs coups,
    `HitsAtLeast` ne s'applique pas). Dans `enemy_telegraph`, la tourelle de la même carte
    tire aussi : la baisse de santé vient des deux (honnête : pas attribuable à l'archer seul).

## 2. Vérifié (résultats réels)

_(complété après les exécutions finales)_

## 3. Non fait / incertain

_(complété)_

## 4. Dettes, questions ouvertes

- Le dessin du télégraphe (T1.17) lit `Emitter::telegraphing()` ; aucun gizmo de debug ajouté.
- `direction_of` : le CORDIC reste imprécis près de 2π (ex. 6,2831 rad → y = -0,007) ; le
  repli ne s'applique qu'au-delà de 2π pour ne rien changer au contenu existant.
- Dette D27 (dispersion joueur qui ignore `spread`) non touchée, comme demandé.
