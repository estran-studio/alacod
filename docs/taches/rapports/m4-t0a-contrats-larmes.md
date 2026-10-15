# M4-T0a : contrats des larmes et de l'empilement de projectiles

Branche `m4-t0a-contrats-larmes` (depuis `origin/main` `f81caa7`). Conventions : `docs/conventions.md`
§60. Session Claude Code cloud (pas de Docker : p2p non rejoué ici). Questions Q1 à Q18 : propositions
par défaut retenues (décision de William du 2026-10-15) ; pour cette fiche, Q3 (règles de composition,
`child` absent = projectile par défaut du porteur).

## 1. Fait

- **Munitions infinies** : `MagBulletConfig::Infinite` en fin d'enum (`combat::weapons`). Jamais vide, jamais
  rechargé (ni délai de pompe d'un `Shotgun`), `mag_ammo`/`mag_size` à 0, contribution de réserve 0
  (comme `Magless`) ; `default_mode_capacity`, `powerups` (Max Ammo) traitent la variante.
  `WeaponConfig.ammo_type` devient `#[serde(default = "no_ammo_type")]` (`Custom("")`, valeur qu'aucun RON
  ne peut écrire : T2.8 la refuse) **pour une arme dont tous les modes sont `Infinite`**. Lint
  (miroir `WeaponConfigSchema.ammo_type: Option`, `FiringModeSchema.mag`) : `ammo_type` absent alors qu'un
  mode consomme → `Parse` (l'erreur RON d'avant, devenue erreur de lint) ; `ammo_type` donné alors que
  tous les modes sont `Infinite` → `OutOfRange`. Décision : une arme mixte (un mode `Mag`, un mode
  `Infinite`) garde son `ammo_type` (la fiche ne tranche pas ce cas).
- **Stats de larmes** : `StatId::ShotSpeed`, `StatId::ShotSize` **après `Custom`** (ordre des clés de `Stats`
  et hash des stats existantes inchangés). Lues au tir (`ShotStats`, `StatReader::get(…, 1)`) : vitesse de
  la balle ; rayon de collision et sprite (après `Size`, nouveau champ `BulletSpawn::shot_size`, 1 pour les
  émetteurs et les projectiles enfants). **Base 1 sans être posées dans `Stats`** (sinon le `Stats` de
  tout personnage, rollback, changerait de hash) ; à 1 aucun produit n'est calculé.
  `sim_core::stats::default_base` donne cette base à l'attente `Stat` et au générateur `ItemHeld`.
- **`Luck` lue** : `effects::chance_with_luck(base, luck, par_point)` = `clamp(base + luck × par_point, 0, 1)`
  en `Fixed`, saturant (3 unitaires). `stat_reader` déclare `Luck`, `ShotSpeed`, `ShotSize`. Personne ne
  l'appelle encore dans la simulation. La fixture `stat_unread` passe à une stat `Custom` (Luck n'est plus
  refusée).
- **Modificateurs portés par un objet** : `ItemDef.projectile` et `SynergyDef.projectile` :
  `Option<ProjectileMods>` (`ProjectileMods` = `ProjectileSpec` : `modifiers`, `on_hit`, `on_expire`),
  `#[serde(default)]`, absent = v1 exacte. **Hors rollback** : `ItemDef`/`SynergyDef` sont des définitions
  (ressources `ItemTable`/`SynergyTable`, sans `Hash`), le champ n'entre dans aucun checksum.
  `items::projectile_sources(items, synergies, inventory)` rend les listes dans l'ordre de composition
  (passifs par ramassage, doublons compris, actif, synergies par id). Le tir ne lit rien (M4-T1) : le lint
  vérifie la forme puis refuse (`Unsupported`).
- **Déplacement de contrats** (décision technique, non demandée par la fiche) : `ProjectileModifier`,
  `Pattern`, `ExpireAction`, `ProjectileSpec` passent de `combat::projectile` à `effects::projectile`,
  réexportés par `combat::projectile` (chemins inchangés). Raison : `items` est un crate `SIM` de
  `check-sim-boundary` ; dépendre de `combat` (dette D69, rendu) l'aurait fait échouer. `derive(Hash/Debug/
  Serialize)` ne dépend pas du crate de définition : aucune trace ne bouge (vérifié par la suite).
- **Composition** : `combat::projectile::compose(base, ajouts) -> ResolvedModifiers`, pure. Chaque liste est
  résolue seule (`resolve_modifiers`, le dernier l'emporte), puis : `Bounce`/`Pierce` addition saturante,
  `Size` produit, `Lifetime` maximum, `Homing` somme plafonnée à 1, `Gravity` somme, `Spectral` présence,
  `Split` addition des `count` (forme de la première source), `Orbit` présence (paramètres de la première
  source). Sans ajout : exactement `resolve_modifiers(base)`. `resolve_modifiers` reste le seul appel au
  tir. `ResolvedModifiers` n'est plus `Copy` (il porte `split`). 10 unitaires (identité, chaque règle,
  ordre des sources, passif en double, doublon dans une liste, hash des variantes historiques).
- **Contrats `Spectral`, `Split { count, spread, child }`, `Orbit { radius, frames_per_turn }`** en fin
  d'enum `ProjectileModifier`. Lint : plages (`count > 0`, `spread >= 0`, `child` dans la table de l'arme,
  `radius > 0`, `frames_per_turn > 0`) puis `Unsupported` sur une arme.
- **Kinds** : `projectile_modifier` + `Spectral`, `Split`, `Orbit` ; `pattern` + `Fan`, `Spiral`, `Burst`
  (manquaient, constat de la fiche).
- **Testbed** : arme `larme` (`Automatic`, 3 tirs/s, `mag: Infinite`, sans `ammo_type`, `test:` 8 coups) et
  objet `lentille` (`ShotSpeed Pct 0.5`, `ShotSize Mul 1.5`, `test:`). Le testbed n'a pas de choix de
  personnage joueur dans un scénario : le « personnage d'essai `larme` » de la fiche est l'arme donnée par
  `PlayerScript::weapon`.
- **Scénarios** (traces neuves seulement) : `tears_infinite` (300 frames de tir, `Ammo` 0 à f5/100/200/309,
  coups exacts 3/8/13 à f100/200/309 : une larme toutes les 20 frames, aucun palier de rechargement),
  `tears_shot_speed` (avec `lentille` : `Stat` 1,5 ×2, aucun coup à f44, touché à f46, un seul coup),
  témoin `tears_shot_speed_sans_objet` (`Stat` 1,0 ×2, aucun coup à f50 ni f56, touché à f58), générés
  `generated/testbed/weapon_larme` et `generated/testbed/item_lentille`.
- **Fixtures de lint** (6) : `weapon_infinite_with_ammo_type`, `weapon_ammo_type_missing`,
  `projectile_contract_unsupported`, `projectile_split_out_of_range`, `item_projectile_unsupported`,
  `item_projectile_out_of_range`.
- Doc : `docs/conventions.md` §60. Liste des attentes de `CLAUDE.md` inchangée (aucune attente nouvelle ;
  `Stat` résout désormais `ShotSpeed`/`ShotSize` avec leur base 1).

## 2. Vérifié (résultats réels)

RESULTATS

## 3. Non fait, incertain, non vérifié

- **p2p à deux clients** (Docker) : non rejoué ici (session cloud), à faire par l'orchestrateur. Le tir des
  armes existantes n'a changé que par des branches inactives (`is_infinite()` faux, stats à 1 sans produit).
- **Bench au calme** (`ALACOD_BENCH_STRICT=1`) : non fait (machine cloud partagée).
- Pas de preuve `trace-diff` : aucune trace existante n'a été bénie, il n'y avait rien à prouver.
- Les bots (`bots::input`, `bots::hunter`) liront une arme `Infinite` comme vide (`mag_ammo` 0, réserve 0) :
  sans conséquence aujourd'hui (aucun bot n'a `larme`), à traiter avec M4-T14 (bot `isaac`).
- HUD : une arme `Infinite` affiche `0 | 0` (M4-T16 : `∞` ou rien).

## 4. Dettes et questions

- **Constat (non corrigé, comme demandé)** : `ReloadSpeed` multiplie la **durée** du rechargement
  (`weapons/mod.rs`, `reload_time_seconds.saturating_mul(reload_speed_mult)`) ; l'objet gungeon
  `chargeur_rapide` (`Pct 0.3`, « 30 % plus rapide ») ralentit donc le rechargement. Déjà noté D74 au
  journal ; numérotation par l'orchestrateur.
- **Préexistant sur `origin/main`** (`f81caa7`, vérifié dans un worktree propre de `main`) :
  `scripts/check-forbidden.sh` échoue sur `crates/game/src/performance.rs` (`Instant::now()`, 3
  occurrences non autorisées, commit `41d3859`) et `make test_tools` échoue sur le test qui vérifie la même
  liste d'exceptions (`test_current_repository_has_no_unreviewed_or_stale_exception`). Hors périmètre ;
  rien de cette tâche n'y touche.
- Règles de composition non couvertes par la fiche, tranchées ici (à confirmer par William si besoin) :
  forme de `Split` et paramètres d'`Orbit` de la **première** source (l'arme passe avant les objets) ;
  `ammo_type` gardé pour une arme mixte `Mag` + `Infinite`.
- `Split.child` absent = « projectile par défaut du porteur » (Q3) : sémantique à fixer par M4-T1 (le
  contrat ne la calcule pas).

## 5. Atteignabilité

Contrat : aucune partie ordinaire ne rencontre ces mécaniques (`larme` et `lentille` sont des contenus de
laboratoire du testbed, avertissements « aucune source » du lint attendus). M4-T1 exécute les
modificateurs d'objet et les contrats `Spectral`/`Split`/`Orbit` ; M4-T0f/M4-T11a mettent `Infinite`,
`ShotSpeed`/`ShotSize` et les objets de larmes dans `isaac` (salle de trésor).
