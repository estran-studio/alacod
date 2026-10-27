# X-3 « Verbes D1 » : rapport (voie C)

Branche `x-3-verbes-d1` (base `origin/main` `be732707`). Fiche : `docs/taches/plan-1837-cli.md` §4 (ligne X-3) et §5 ;
critère : `docs/plan-engine.md` §4.8. Section de conventions : **§90** (`docs/conventions.md`). Dettes : D149 à D152.

**Règle respectée** : rien d'un jeu externe dans l'engine (aucun nom, personnage, arme, sprite, valeur de réglage ni
« 120 bpm »). Le testbed prouve les verbes avec des noms neutres (`pouls`, `chronometre`, `parade`…) et des valeurs
arbitraires (battement de 40 frames, fenêtres 3 et 8, multiplicateurs 0,5 et 2).

## Ce qui est livré

| Verbe | Où | Contenu |
|---|---|---|
| Fenêtre de jugement | `sim_core::timing` (règle pure), `effects::Condition::Timing`, `effects::runtime::{TimingRefs, timing_holds}`, `run::clock::nearest_occurrence`, `game::effects_runtime::TimingSource`, lint, `describe` | repères `Clock`, `DashStart`, `MeleeStart` ; notes parfait / bien / raté ; durées en frames entières ; `OnDodge`, `OnReload`, `OnHit`, `Tick` comme « coup qui arrive » |
| Parade et renvoi | `combat::weapons::melee` (`MeleeReflect`, `melee_reflect_system`, `ProjectileReflected`) | changement d'équipe, de tireur et de direction (regard), dégâts × bonus sur « parfait », départage par `GgrsNetId` |
| Poids des armes | `combat::weapons` (`WeaponConfig::move_speed_mult`, `sync_carry_weight_system`) | l'architecture de modificateurs suffisait : un `Mul` sur `MoveSpeed`, source `weapon_weight:<arme>` |

Détails de conception, d'ordre et de lint : `docs/conventions.md` §90.

## Déterminisme et traces

- Aucun `f32` ; entiers `u32` pour les notes, `Fixed` pour dégâts et vitesses ; `BTreeSet` pour le départage ;
  `order_iter!`/`order_mut_iter!` partout où l'ordre compte (`sync_carry_weight_system` écrit seulement les
  modificateurs de son propre porteur : itération non ordonnée justifiée dans son commentaire).
- `FrameEvents<ProjectileReflected>` : file **neutre** (vide hors parade). Aucun type rollback nouveau.
- `MeleeWeaponConfig::reflect` et `WeaponConfig::move_speed_mult` : `Hash` et `Debug` manuels, rien d'écrit tant
  qu'ils sont absents (hash des armes existantes inchangé). `Condition::Timing` est ajoutée **en fin d'enum**.
- `TimingSource` est dérivée (lecture seule de `DashState`, `MeleeAttackState`, `Clock`, `ClockSchedule`), construite
  seulement pour un porteur dont un effet porte un `Timing` : aucun état caché, aucun calcul pour le contenu existant.
- Ordre : `melee_reflect_system` entre le déplacement des balles et leurs collisions ;
  `familiar_block_bullets_system` passe **après** lui (arête ajoutée dans `familiars.rs`, exigée par la détection
  d'ambiguïté de `GgrsSchedule`). `sync_carry_weight_system` ouvre le set `Weapon`. Ce sont les seuls systèmes ajoutés.

## Preuves

Scénarios nouveaux (testbed, traces écrites une fois ; aucune trace existante réécrite) :

| Scénario | Preuve |
|---|---|
| `fenetre_esquive` | dash parfait (+1), « bien » (+10), raté (+100) contre l'horloge `pouls` |
| `fenetre_rechargement` | rechargement actif : fin sur le battement (+1000), second rechargement hors fenêtre (rien), `Ammo` |
| `fenetre_fraction_esquive` | instant à `DashStart + 5` : +1 par dash, à la frame exacte |
| `parade_projectiles` | boule de feu renvoyée (équipe `Players`), tourelle −16 (parfait) puis −8 (bien) |
| `arme_lourde` / `arme_lourde_temoin` | `MoveSpeed` 75 contre 150, 68 contre 135 unités en 60 frames |

Tests unitaires : `sim_core::timing` (5), `effects::runtime::fenetre_de_jugement`, `effects::contracts` (RON, `offset`
facultatif), `run::clock::occurrence_la_plus_proche`, `combat::weapons::tests::le_poids_de_l_arme_active…`,
`combat::melee::reflect_tests`. Lint : fixtures `melee_reflect_mult_zero`, `weapon_move_speed_mult_zero`,
`effect_timing_invalid` ; `zombies_and_testbed_lint_without_error` reste vert.

Vérifications exécutées sur cette machine (VPS) : `cargo check` de `sim_core`, `combat`, `content`, `game`, `scenario` ;
`cargo test -p sim_core -p effects -p run`, `cargo test -p combat --lib` (121), `cargo test -p content --test
lint_fixtures` (178) ; les six scénarios ci-dessus (synctest compris) ; `alacod-gen` sur le testbed sans `--bless`.
Le statut de `make test_scenarios` complet, de `make schedule_order_diff` et de la capture Mac est dans le message de
livraison (section « Non exécuté »).

## Non livré, et pourquoi (pas de substitut factice)

- **Hit stop « cible seule » (D150)** et **musique calée sur l'horloge de simulation (D151)** : présentation pure,
  invérifiable sans écran ni sortie audio ; ce VPS n'a ni GPU (D60, D94) ni périphérique audio. Écrire un réglage ou un
  système sans pouvoir l'observer aurait été un no-op déguisé. Les dettes décrivent la conception et la preuve attendue.
- **Parade de mêlée contre mêlée (D149)** : la règle de contact entre deux hitbox n'est pas fixée par le design.
- **Repères `ReloadStart` et « coup qui arrive » d'un ennemi (D152)** : le rechargement actif est prouvé sur l'horloge.
- **Preuve visuelle du renvoi sur Mac** : voir le message de livraison.
