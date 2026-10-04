# Rapport m1-v1a-patterns — patterns, émetteurs et tir ennemi (T1.2)

**Branche** `m1-v1a-patterns-tir-ennemi`, partie de `main` `58db32c` ; `origin/main`
`23a43fc` mergé juste avant la livraison (merge `068dac8`, sans conflit : `CLAUDE.md`,
checklist « piège de parité », et `examples/character_tester.rs`, aucun code de simulation).
m0-v7 p2 n'était pas encore sur `origin/main` au moment de livrer.

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

Sur le commit d'implémentation `4a7bddb` (le merge final n'apporte aucun code de simulation).

- **`make test_scenarios`** : 86 scénarios joués (82 existants + 4 nouveaux), **0 « trace
  différente »** : les 82 traces existantes sont inchangées, sans bless ni preuve. Seuls
  échecs : `enemy_ring`, `enemy_ring_quad`, `enemy_telegraph`, `weapon_fireball_gun` sur « pas
  de trace de référence » (nouvelles traces, toutes leurs attentes passent) — **4 bless
  demandés à l'orchestrateur**.
- **Tests des crates** (`cargo test -q --profile headless -p scenario -p run -p combat -p game
  -p content -p map_ldtk -p sim_core -p stats -p bots -p effects -p utils`) : tous verts hors
  `scenarios` (ci-dessus) ; dont `combat` 66 (9 tests d'émetteur, test de contrat des kinds),
  `lint_fixtures` 48 (4 nouvelles fixtures).
- Tests unitaires de la fiche : `ring_every_seul_est_infini`,
  `ring_every_dans_une_sequence_tourne_jusqu_a_la_fin`, `ring_sans_every_tire_une_fois_et_finit`,
  `sequence_joue_les_etapes_instantanees_la_meme_frame`, `telegraph_retarde_et_se_lit`,
  `wait_ne_tire_pas_et_n_est_pas_un_telegraphe`, `scatter_tire_dans_l_eventail_et_consomme_le_flux`,
  **`meme_graine_meme_tir_a_un_et_quatre_joueurs`**, `named_se_resout_par_la_bibliotheque`,
  `couronne_visant_vers_le_haut_ne_deborde_pas`. Helper d'apparition partagé : les trois
  appelants passent par `weapons::spawn_bullet` (structure du code ; les balles joueur et les
  projectiles nés gardent leurs traces, donc leurs valeurs, au bit près).
- **`make lint`** : `games/zombies : aucune erreur` ; `games/testbed : aucune erreur (9
  personnages, 12 armes, …, 8 cartes)` (kind `Pattern` déclaré par le testbed).
- **`cargo fmt --all -- --check`** : propre (après `make fmt`).
- **`check-forbidden.sh`** : 4 occurrences, identiques à main ; **`check-rollback-registration.sh`** :
  OK. (`make scripts` cité par la fiche n'existe pas dans le Makefile : ce sont ces deux
  scripts, README §4.)
- **`make gen GAME=zombies`** : 10/10, attentes et traces `ok`, aucun fichier modifié.
  **`make gen GAME=testbed`** : 17 armes `ok`/`ok` sans modification ; `fireball_gun` (nouvelle
  arme) : attentes `ok`, trace `absente` → nouveau fichier
  `tests/scenarios/generated/testbed/weapon_fireball_gun.ron` (4e nouvelle trace). Son `test:`
  demande 2 coups (comme `rifle`/`proj_lifetime` : 2 coups mesurés en 200 frames).
- **Bench** (mesuré pendant la suite, **machine non calme** : b1 jouait sa suite, charge ~5) :
  `bench_bullets` 111,9 fps (budget 70), `bench_horde` 65,9 fps (budget 38) — dans le budget
  même sous charge. Le run `ALACOD_BENCH_STRICT=1` machine calme n'a pas pu être fait.

## 3. Non fait / incertain

- Bench strict (`ALACOD_BENCH_STRICT=1`) machine calme : non fait (machine partagée occupée) ;
  chiffres ci-dessus mesurés sous charge.
- p2p à deux clients : non rejoué ici (la recette de l'orchestrateur au merge).
- Le gizmo de debug du télégraphe (« au plus ») n'est pas ajouté.
- Pas de preuve `trace-diff` : aucune trace existante n'a changé.

## 4. Dettes, questions ouvertes

- Le dessin du télégraphe (T1.17) lit `Emitter::telegraphing()` ; aucun gizmo de debug ajouté.
- `direction_of` : le CORDIC reste imprécis près de 2π (ex. 6,2831 rad → y = -0,007) ; le
  repli ne s'applique qu'au-delà de 2π pour ne rien changer au contenu existant.
- Dette D27 (dispersion joueur qui ignore `spread`) non touchée, comme demandé.
