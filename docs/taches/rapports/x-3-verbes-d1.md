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

Vérifications exécutées (VPS, sauf mention) :

- `cargo check` de `sim_core`, `combat`, `content`, `game`, `scenario` ; `cargo test -p sim_core -p effects -p run`,
  `cargo test -p combat --lib` (121), `cargo test -p content --test lint_fixtures` (178) ; `cargo fmt --check`,
  `scripts/check-forbidden.sh` et `scripts/check-rollback-registration.sh` verts.
- **Suite de scénarios complète, sans `BLESS`, en quatre tranches** (`ALACOD_SCENARIO_SLICE=k/4`, profil `headless`,
  synctest compris) : 4 tranches sur 4 vertes, **aucune trace existante modifiée** (`git status` propre, seuls les
  `.trace` des scénarios nouveaux sont ajoutés). Les deux derniers changements (enregistrement de
  `FrameEvents<ProjectileReflected>` déplacé dans `CoreSimPlugin`, peau des projectiles renvoyés) sont postérieurs aux
  tranches 0 et 1 ; ils ont été revérifiés par un rejeu de quatorze scénarios (les six nouveaux, `enemy_ring`,
  `aim_four_way`, `clock_events`, `effect_on_dodge`, `effect_on_reload`, `movement_melee`, `familiar_mirror_items`,
  `generated/testbed/weapon_pistolet_lourd`), verts.
- **Ordre de la simulation** (Mac, `crates/scenario/tests/schedule_order.rs`, deux cibles séparées : `make
  schedule_order_diff` partage le sien entre base et tête et compilait de mélange, D133) : la liste des **types
  rollback** gagne exactement une ligne par jeu, `FrameEvents<ProjectileReflected>`, juste avant les cinq de clôture
  (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams`) ; la liste des systèmes gagne
  `sync_carry_weight_system`, `melee_reflect_system`, `clear_frame_events<ProjectileReflected>`, et
  `familiar_block_bullets_system` passe après la parade. Le reste du diff des schedules est du renumérotage de
  systèmes sans arête entre eux (le tri topologique de bevy ne suit pas l'ordre d'insertion, `CLAUDE.md`) : ce sont les
  seuls systèmes ajoutés, ordre nouveau justifié.
- **Preuve visuelle** : voir plus bas (Mac).

## Non livré, et pourquoi (pas de substitut factice)

- **Hit stop « cible seule » (D150)** et **musique calée sur l'horloge de simulation (D151)** : présentation pure,
  invérifiable sans écran ni sortie audio ; ce VPS n'a ni GPU (D60, D94) ni périphérique audio. Écrire un réglage ou un
  système sans pouvoir l'observer aurait été un no-op déguisé. Les dettes décrivent la conception et la preuve attendue.
- **Parade de mêlée contre mêlée (D149)** : la règle de contact entre deux hitbox n'est pas fixée par le design.
- **Repères `ReloadStart` et « coup qui arrive » d'un ennemi (D152)** : le rechargement actif est prouvé sur l'horloge.

## Preuve visuelle (Mac)

`play_scenario tests/scenarios/parade_projectiles.ron --capture <dossier> --every 1` sur le Mac (worktree dédié,
cible clonée, `CARGO_BUILD_JOBS=4`), 261 images. Les boules de feu ennemies sont rouges ; **celle qui a été renvoyée est
jaune** (peau de l'équipe `Players`) et repart vers la droite :
`x-3-verbes-d1-renvoi-f70.png` (f70 : boule jaune à côté de l'arc du coup) et `x-3-verbes-d1-renvoi-f90.png` (f90 :
la même, plus loin, vers la tourelle).

Cette capture a montré un défaut de **présentation** : la peau d'un projectile n'était posée qu'une fois, donc une balle
renvoyée gardait sa peau ennemie (rouge). Correctif (`game::world_sprites::reskin_reflected_bullets`, composant
`SkinnedAs`, présentation seule : les marqueurs de peau et de style sont retirés quand l'équipe change, la taille d'avant
la peau est rendue, `skin_bullets` et `style_tears` repassent une fois). Aucune simulation ni trace touchée.

## Suite : revue indépendante et pièces manquantes (2026-10-27, même branche)

La revue indépendante a relevé quatre bloqueurs (P1, trois P2), tous corrigés ; les trois dettes
restantes (D149, D150, D151) sont livrées, D152 reste ouverte et rétrécie.

### Correctifs de revue

- **P1 — `TimingSource` exigeait `Clock`** : `clock: Res<run::Clock>` paniquait dans une application
  de test qui monte les effets sans horloge. `clock`, `floor` et `schedule` sont `Option` : sans
  eux le repère `Clock(..)` est inconnu (condition fausse). Test
  `une_fenetre_de_jugement_sans_horloge_montee_ne_declenche_rien_et_ne_panique_pas`.
- **P2 — la parade tournait avant la création/expiration des hitbox** : un contact à la frame du
  coup ne pouvait pas être parfait, une hitbox expirée renvoyait encore. Ordre du set `Weapon` :
  création (joueur, ennemi), expiration, parade de mêlée, renvoi, collisions de balles, collisions
  de mêlée ; le repère de note est `MeleeHitbox::created_frame` (plus de `MeleeAttackState` lu par
  le renvoi). Scénarios `parade_projectiles_instantanee` (contact f64 = parfait × 2, impossible
  avant) et `parade_projectiles_expiree` (le second coup expire à f94 avant l'arrivée de la boule :
  tourelle à 484, inchangée — avec l'ancien ordre elle tombait à 476). Tests unitaires
  `un_contact_a_la_frame_du_coup_est_renvoye_parfait` et `une_hitbox_expiree_ne_renvoie_plus_rien`
  (`update` puis `reflect` enchaînés).
- **P2 — horloge `Floor` lue avant sa remise à zéro le jour du passage** : `TimingSource::beat`
  vaut 0 pour le temps d'étage quand `FloorState` a déjà changé d'index (comme `clock_system` à la
  même frame). Tests `l_horloge_d_etage_est_deja_remise_a_zero_pour_debut_d_etage` et Run associé ;
  scénario `fenetre_etage` (deux entrées, 8 puis 15 de dégâts, jamais +70).
- **P2 — reskin d'une balle renvoyée sans peau du joueur** : `SkinnedAs` garde l'état d'avant la
  peau (image, couleur, côté) et le rend avant reskin ; test
  `un_projectile_renvoye_retrouve_son_etat_d_avant_la_peau`.

### Livré en plus

- **D149 — parade de mêlée contre mêlée** : `MeleeWeaponConfig::parry` (`perfect_frames`,
  `perfect_knockback`), `melee_parry_system` (annulation avant blessure, contacts décidés puis
  appliqués, mutuelle symétrique, recul parfait sur l'axe), `FrameEvents<MeleeParried>` (neutre,
  moment clé `parry`), lint `perfect_knockback >= 0` (fixture
  `melee_parry_knockback_negative`), schéma, §90. Scénario `parade_melee` (parfait repoussé f46,
  bien au contact f104, seul le troisième coup passe : 97,5 PV). D149 fermée.
- **D150 — hit stop « cible seule »** : `hit_stop_scope` (`Global`/`Target`, global par défaut,
  surchargeable), `animation::AnimationHold` (gel par entité, ticks de rendu), jamais le joueur
  local ni la caméra ; indice `extra: 1` dans le journal, moment `feedback` « cible seule ».
  Testbed : `parade_melee` en `Target`. D150 fermée.
- **D151 — musique calée sur l'horloge** : `music` dans `feedback.ron` (piste, horloge à `repeat`,
  événement, `loop_events`, `max_drift_ms`, volume), `game::music_sync` (lecture seule de la
  simulation, pause/reprise/recalage, journal `music f… position=… expected=… drift_ms=…`),
  lint (boucle, références, bornes), piste `metronome.ogg` (CC0,
  `scripts/generer-son-metronome.sh`), testbed branché sur `pouls`. Fonctions pures testées.
  D151 fermée.
- **Notes de jugement en `FrameEvents`** : `TimingJudged` (neutre, moment clé `timing`), filtre
  `Good` (« bien » seulement, le ET d'avant disparaît de `chronometre`), `TimingReference` et
  `Condition::Timing` inchangés pour le contenu d'avant (`Good` ajouté en fin d'enum).

### Reste ouvert

- **D152 (rétrécie)** : pas de repères `ReloadStart` ni de « coup qui arrive » ennemi. Les actions
  D1 documentées (`design/game-feel.md` : gigue sur le temps, renvoi au timing, rechargement
  lisible) n'en exigent aucun : la gigue se juge sur l'horloge, le renvoi sur le début du coup, le
  rechargement actif sur l'horloge (`fenetre_rechargement`).
- **Preuve Mac** : captures du gel « cible seule » et mesure de dérive audio sur périphérique réel,
  voir § suivant. Si indisponible, la limite exacte est écrite ici, sans preuve factice.

### Vérifications (VPS)

- Unitaires (profil `headless`, `--no-default-features`) : `combat` (dont 5 tests X-3 suite),
  `effects`, `content` (dont fixture `melee_parry_knockback_negative`), `sim_core`, `run`,
  `animation`, `game` (dont `TimingSource` sans horloge, horloge d'étage, `music_sync`,
  reskin) : À REMPLIR.
- Scénarios nouveaux (10) : verts, traces écrites une fois : `parade_projectiles_instantanee`,
  `parade_projectiles_expiree`, `parade_melee`, `fenetre_etage` (+ 6 de X-3).
- Suite complète : À REMPLIR (aucune trace existante réécrite).
- `cargo fmt --check`, `scripts/check-forbidden.sh`,
  `scripts/check-rollback-registration.sh` : À REMPLIR.
- Types rollback nouveaux (neutres, après `ProjectileReflected`, avant les cinq de clôture) :
  `FrameEvents<TimingJudged>`, `FrameEvents<MeleeParried>`. Systèmes ajoutés : `melee_parry_system`,
  ordre du set `Weapon` réordonné (voir §90) : le diff `schedule_order_diff` gagne ces lignes, rien
  d'autre.
