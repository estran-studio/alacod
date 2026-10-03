# Rapport — m1-v1a-projectiles-composables (T1.1, B5 v1)

- Fiche : `docs/taches/m1-v1a-projectiles-composables.md`
- Branche : `m1-v1a-projectiles-composables`, session cloud c1
- Tête du code vérifié : `0bbb8de` (merge de `origin/main` `b6a58a0` dans `bdbe9dd`) ; ce
  rapport est le commit suivant, sans changement de code.
- Bless à faire par l'orchestrateur : **seulement les 17 nouveaux scénarios**
  `tests/scenarios/generated/testbed/*.ron`, qui n'ont pas de trace. **Aucune trace existante
  ne change.**

## 1. Fait

### Simulation (`crates/combat`)

- `src/projectile.rs` (la majeure partie du travail) :
  - Données : `ProjectileSpec { modifiers, on_hit, on_expire }` (champ `projectile:` d'un mode
    de tir), `ProjectileDef` (table `projectiles` d'une arme, cible des patterns),
    `ExpireAction::Spawn(Pattern)`. Les contrats T1.0a ne sont pas renommés ;
    `ProjectileModifier`/`Pattern` passent de squelettes à types exécutés (mêmes variantes).
  - Composant rollback `Projectile`, posé à côté de `weapons::Bullet` uniquement pour une balle
    composable : rebonds/perforations restants, `Size`, durée de vie restante, `Homing`,
    `Gravity`, cibles déjà touchées (triées par `GgrsNetId`), `ended`, génération,
    multiplicateur de dégâts du tir, `on_hit`, `on_expire`, table de l'arme (`Arc`). `Debug`
    compact : la table n'apparaît que par ses clés.
  - `FrameEvents<ProjectileHit>` (coups qui portent un `on_hit`).
  - Systèmes, tous dans `RollbackSystemSet::Projectiles`, enchaînés :
    `projectile_collision_system` (personnages par `GgrsNetId` tant que `Pierce` le permet,
    jamais deux fois le même ; puis murs : `Bounce` réfléchit l'axe fautif et ramène à la
    position d'avant le déplacement, sinon fin), `apply_projectile_on_hit_system`,
    `projectile_expire_system` (`Lifetime`, puis `on_expire` et `despawn_rollback` pour
    **toute** fin : durée de vie, portée, mur, perforation épuisée), `projectile_steering_system`
    (`Gravity` puis `Homing`, appliqués au déplacement suivant).
  - Mathématiques pures testées : `resolve_modifiers`, `apply_gravity`, `steer_towards`,
    `reflect`/`bounce_axes`, `pattern_shots` (Aimed, Spread, Ring, Sequence ; sans aléa),
    `Projectile::register_hit`/`register_wall`/`tick_lifetime`/`radius`.
- `src/weapons/mod.rs` : `FiringModeConfig::projectile` et `WeaponConfig::projectiles`
  (`serde(default)`), avec **Hash et Debug manuels** qui ignorent ces champs vides — le
  contenu existant garde son hash (checksum) et sa ligne de trace détaillée. Tir : la balle
  reçoit `Projectile` si le mode en déclare un, `Size` agrandit collider et sprite.
  Déplacement : la portée atteinte *termine* un projectile composable (`ended`) au lieu de le
  détruire. Collisions des balles ordinaires : `Without<Projectile>`, sinon inchangées.
  Enregistrement `rollback_and_trace::<Projectile>()` et `add_frame_events::<ProjectileHit>()`.
- `src/weapons/expectations.rs` : `Expectation::BulletCount { count, projectile?, team?,
  at_frame }` (compte exact) et `Expectation::HitsAtLeast { entity, hits, at_frame }` avec
  `EntityRef::{NetId(n), Target}` (réexporté par `game::replay`).
- `Cargo.toml` : dépendance `effects` (pour `Action` ; pas de cycle).

### Scénarios (`crates/scenario`)

- `src/runner.rs` : évaluation de `BulletCount` (filtres id de projectile composable / équipe
  du tireur) et `HitsAtLeast` (`Target` = entité avec `HitCount` de plus petit `GgrsNetId`).
- `tests/expectations.rs` : `bullet_count_compte_les_projectiles_vivants` (balles ordinaires,
  filtres équipe/id, puis les 8 éclats de la première grenade en f113 et un compte faux qui
  échoue) et `hits_at_least_lit_le_compteur_de_la_cible` (Target, NetId, seuil dépassé,
  entité sans compteur).

### Contenu et lint

- `crates/content` : mirroirs `ProjectileSpecEntry`/`ProjectileDefEntry`/`PatternEntry`/
  `ProjectileModifierEntry`, `lint_weapon_projectiles` : modificateur répété, `Size <= 0`,
  `Homing` hors `]0, 1]`, `on_hit` sans modificateur, `Telegraph`/`Wait` en `on_expire`,
  `count = 0`, `spread`/`speed` < 0, projectile absent de la table (`BrokenReference`),
  `damage`/`speed` < 0 ou `range <= 0` d'une définition, cycle de `on_expire`. Fixtures
  `projectile_broken_reference`, `projectile_temporal_pattern`, `projectile_cycle` et le test
  `projectile_fixtures_have_a_single_rule_failure`.
- `games/testbed/.../weapons.ron` : `proj_bounce` (Bounce(3)), `proj_pierce` (Pierce(2)),
  `proj_size` (Size(3)), `proj_lifetime` (Lifetime(30)), `proj_homing` (Homing(0.1)),
  `proj_gravity` (Gravity(-60)) et `grenade` (Bounce(2), Pierce(8), Lifetime(50), `on_expire`
  → `explosion` : vitesse 0, Lifetime(0), Size(10), Pierce(16), `on_expire` → Ring de 8
  `eclat`). Chacune a `test:` ; la grenade a en plus `expect: [BulletCount(8 éclats, f113),
  BulletCount(0 éclat, f174), HitsAtLeast(Target, 11, f175)]`.
- `tests/scenarios/generated/testbed/` : 17 scénarios écrits par `make gen GAME=testbed`
  (les 7 nouvelles armes et, comme le générateur prend tout le registre du jeu, les 4 armes
  à distance et 6 de mêlée déjà partagées avec `zombies`).

### Docs

- `docs/conventions.md` §16 « Projectiles composables » (section distincte ; le conflit avec
  le §18 venu de `main` est résolu en gardant les deux).
- `CLAUDE.md` : liste des attentes.

### Décisions de gameplay (fixées ici, documentées au §16)

- Modificateur répété : le dernier l'emporte (lint : erreur).
- `Pierce(n)` : n personnages traversés, fin sur le (n+1)-ième ; une cible n'est touchée
  qu'une fois par projectile ; tous les personnages en contact dans la frame sont touchés.
- Personnages avant murs dans une même frame.
- `Bounce` : axe du déplacement qui entre dans le mur inversé (les deux dans un coin), retour
  à la position d'avant le déplacement.
- `Lifetime(n)` : n frames après celle du tir ; `Lifetime(0)` = une seule frame de collisions.
- `Homing(force)` : `v̂ + force·t̂` renormalisé, vitesse conservée, cible la plus proche
  touchable (hors `Neutral`, hors déjà touchées, égalité par `GgrsNetId`), sans portée de
  détection en v1.
- `Gravity(g)` : `v.y += g/3600` par frame (unités/s², positif vers le haut du monde).
- `on_hit` : seulement `TimedModifier`/`CurrencyMultiplier`, posés sur la cible touchée,
  source `projectile:<id>:<rang>`, **rafraîchis** (pas d'empilement).
- `on_expire` : à toute fin ; patterns instantanés seulement ; `Aimed` vise le personnage
  touchable le plus proche, sinon la direction du projectile ; `Ring` part de la direction du
  projectile (axe +x à l'arrêt), `every` ignoré (une salve) ; enfants hérités du tireur
  (source, équipe, tags, tir ami, multiplicateur `Damage`) ; garde-fou de 8 générations.
- Table de projectiles **par arme** en v1 (T1.2 pourra la remonter au niveau du jeu).
- `apply_projectile_on_hit_system` est dans `Projectiles` et non `Effects` : dans `Effects`,
  `GgrsSchedule` refuse l'ordre ambigu avec `game::powerups::apply_powerup_actions_system`
  (les deux écrivent `Modifiers`).

## 2. Vérifié (dans le clone cloud, état fusionné équivalent : le merge de `main` n'apporte
que docs, `scripts/nightly.sh` et `docker-compose.ci.yaml`)

- `cargo test -q --profile headless -p run -p combat -p game -p content -p map_ldtk -p sim_core
  -p stats -p bots -p effects --no-fail-fast` : **275 réussis / 0 échec / 1 ignoré**
  (dont 14 tests de `combat::projectile` : un par modificateur au moins, patterns, RON).
- `cargo test --profile headless -p scenario --no-fail-fast` : **47 réussis / 1 échec /
  7 ignorés**. `expectations` 34/34 (dont les deux nouveaux), `bots` 1, `determinism` 2,
  `generated` 1, `run` 3, `softlock` 2. Le seul échec est le test `scenarios` (= `make
  test_scenarios`) : **79 scénarios joués, les 62 existants (52 manuscrits + 10 générés
  zombies) verts avec leurs traces inchangées** ; les 17 de `generated/testbed/` échouent
  uniquement par « pas de trace de référence (lancer avec ALACOD_BLESS=1) » — attendu,
  non béni (règle cloud). Toutes leurs attentes passent (`alacod-gen --play`, ci-dessous).
- `make gen GAME=testbed` (`alacod-gen games/testbed --play`) : attentes `ok` pour les 17,
  trace `absente` pour les 17 (code de sortie 1 pour cette seule raison). Coups sur `target` :
  grenade 34 (600 frames), proj_bounce 5, proj_gravity 3 (400 f), proj_homing 4,
  proj_lifetime 2, proj_pierce 2, proj_size 3. Les fichiers commités sont la sortie de la dernière exécution (générateur déterministe).
- `make gen GAME=zombies` : 10 armes, attentes `ok`, traces `ok`, aucun fichier modifié.
- `make lint` : `games/zombies : aucune erreur`, `games/testbed : aucune erreur (7
  personnages, 11 armes, 6 armes de corps à corps, 0 vagues, 4 cartes)`.
- `cargo fmt --all -- --check` : rien. `./scripts/check-forbidden.sh` : 4 avertissements
  préexistants (HashSet 3, rand 1), aucun nouveau. `./scripts/check-rollback-registration.sh` :
  OK.
- **Scénario généré qui exerce Bounce, Pierce et une explosion** (`weapon_grenade`), relevé
  frame par frame avec une sonde (non commitée) : 1er tir explose en f111 (8 éclats en vol en
  f113) ; 2e tir rebondit sur le mur du haut en f160 (rebonds 2→1), traverse `target` vers f166
  (perforations 8→7) et explose en f172 sur elle ; ses 8 éclats la touchent en f174
  (`HitCount` 3 → 11). Les attentes `BulletCount` et `HitsAtLeast` de la grenade fixent ces
  points.
- **Preuve `trace-diff`** (README §5) : dumps `ALACOD_DUMP_TRACE` sur `main` `2db0b23`
  (worktree détaché, même target) et sur la branche, `scripts/trace-diff.py` :

  | Scénario | sans `--ignore` | `--ignore Projectile,FrameEvents<combat::projectile::ProjectileHit>` |
  |---|---|---|
  | `idle` | seule ligne en plus : `FrameEvents<combat::projectile::ProjectileHit>=FrameEvents([])` | identique (1499 frames) |
  | `points_on_kill` | idem | identique (599 frames) |
  | `two_players_shooting` (tir) | idem | identique (299 frames) |

  Et les `.trace` (checksum) des 62 scénarios existants sont identiques : rien à blesser
  côté existant.

## 3. Non fait / non vérifié ici

- **Bench** (`bench_bullets`, `ALACOD_BENCH_STRICT=1`) : interdit dans le cloud ; les fps
  affichés par `make test_scenarios` ici (machine différente, tout sous les planchers, ex.
  `bench_bullets` 43,5 fps) ne sont pas comparables. `bench_bullets` n'utilise que des balles
  ordinaires : seul coût ajouté pour elles, un filtre `Without<Projectile>` et un
  `Option<&mut Projectile>` dans le déplacement. « `bench_bullets` à 500 dans le budget »
  (taches.md) n'est pas traité ici (pas dans la fiche).
- **p2p** à deux clients : interdit dans le cloud. La simulation des scénarios existants est
  identique (traces) ; les projectiles composables ne sont tirés par aucune arme de `zombies`.
- Pas de vidéo ni de visuel dédié (explosion = sprite carré agrandi par `Size`).
- `on_hit` n'a pas d'arme de démonstration dans le testbed (testé par le lint et le code
  partagé `Action::as_modifier`, pas par un scénario).

## 4. Dettes et questions ouvertes

- Les 17 scénarios `generated/testbed/` incluent 10 doublons des scénarios
  `generated/zombies/` (mêmes armes, même arène) : ~80 s de plus par `make test_scenarios`.
  Le générateur n'a pas de filtre par arme ; à décider (filtre, ou ne générer pour le testbed
  que ses armes propres).
- La dispersion des armes non-fusil à pompe est un tirage ±0,5 rad qui ignore `spread` du
  RON (`weapon_rollback_system`, préexistant) : les armes de démo ratent souvent `target`,
  d'où des `min_hits` bas.
- Table de projectiles par arme : les ennemis (T1.2, `Shoot(pattern)`) auront besoin d'une
  table au niveau du jeu ou de l'ennemi.
- `Homing` sans portée ni cône : vise le plus proche partout dans la carte.
- Le README (§10 de conventions) dit qu'enregistrer un type au checksum change toutes les
  traces : ce n'est pas ce qui a été observé ici (traces identiques avec `Projectile` et
  `FrameEvents<ProjectileHit>` enregistrés au checksum). À vérifier côté orchestrateur.
- Environnement cloud : l'allowance disque (~30 Go) est saturée par les binaires de test du
  profil `headless` (~1 Go chacun) ; il a fallu supprimer les exécutables entre deux lots et
  compiler avec `CARGO_INCREMENTAL=0`.
