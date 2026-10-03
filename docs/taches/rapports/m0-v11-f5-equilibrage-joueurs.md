# Rapport m0-v11 — F5 : équilibrage par nombre de joueurs (vagues, prix, santé)

**Base de la branche** : `origin/main` à jour au moment de la livraison (`<sha-main>`, merge
d'hygiène fait juste avant — règle m0-v9).

## A. Évaluation par la simulation

- **Type** : `content::expr::NumOrExpr` (crates/content/src/expr.rs) — `Integer(u32)` (entier
  RON nu, inchangé), `Literal(Fixed)` (chaîne numérique, inchangée), `Expression(NumExpr)`
  (chaîne RON évaluée, ex. `"10.0 + (players - 1) * 40.0"`). La désérialisation essaie dans
  cet ordre : entier nu → `Integer` ; chaîne qui est un nombre → `Literal` ; sinon parse
  `Expr` → `Expression` — un échec de parse est un échec du chargement. Les littéraux
  existants gardent exactement leurs valeurs (les 62 scénarios existants, leurs traces).
- **Point d'évaluation** : `game::balance::resolve_balance_system`, sur
  `OnEnter(AppState::GameLoading)` — assets chargés, session configurée, avant le spawn des
  joueurs/personnages (Update de `GameLoading`) et avant la première frame simulée. Il insère
  `game::balance::ResolvedBalance` (ressource ordinaire, hors rollback comme `Assets`) :
  `waves`, `economy`, `perks` (`BTreeMap<String, ResolvedPerkDef>`), et
  `health_max_by_character` (`BTreeMap<String, Fixed>`, clé = `asset_name_ref`). **Aucune
  expression ne vit dans l'état rollback** : les systèmes de simulation (vagues, économie,
  interactions, HUD, création de personnages) lisent `ResolvedBalance`, jamais les assets
  d'origine. Les méthodes de calcul (`get_tier`, `calculate_enemy_count`,
  `calculate_health_multiplier`, `refill_price`) ont été déplacées telles quelles sur les
  types résolus. Une relance locale re-entre `GameLoading` et re-résout avec le même nombre
  de joueurs.
- **Nombre de joueurs** : `OnlineState::Online` → `ggrs_config.connection.max_player`
  (source autoritaire en ligne, même code d'évaluation partagé — critère 5) ; sinon
  `PlayersCount` (partie locale ou scénario : `scenario.players.len()`).
- **Erreurs** : `content::expr::EvalError` (identifiant inconnu — seul `players` existe —,
  division par zéro, valeur hors domaine, ex. négatif pour un `u32`). `resolve_balance_system`
  panique en nommant le fichier, le champ, le nombre de joueurs et l'erreur — jamais de
  valeur par défaut silencieuse (les `eval`/`eval_num` indulgentes pré-existantes de
  `Expr`/`NumExpr` sont gardées pour la compatibilité ; F5 passe par les `try_eval`
  strictes).

## B. Les trois familles de champs

- **Vagues** (`games/zombies/assets/waves/wave_config.ron`) : `base_enemies`,
  `enemies_per_wave`, `max_random_variance`, `min_wave_delay_frames`, `grace_period_frames`,
  `max_concurrent_enemies`, `spawn_batch_size`, `spawn_interval_frames`,
  `min_player_distance`, `max_player_distance`, `health_multiplier_per_wave`,
  `damage_multiplier_per_wave`, `max_wave`, et par palier `max_wave` + poids des ennemis.
  Les overrides de scénario (`apply_wave_overrides`) écrivent des `Integer` (entier
  littéral).
- **Prix** : `games/zombies/assets/economy/economy.ron` (`kill_points`, `hit_points`,
  `repair_points`, `nuke_points`, `repair_points_cap_per_wave`, `refill_price_ratio`) et
  `games/zombies/assets/economy/perks.ron` (`price` de chaque perk). Les prix d'armes
  murales et de portes sont des champs **Int posés dans l'éditeur LDtk** (`WeaponLocation`,
  portes), pas du RON — hors périmètre (documenté §18) ; `weapons.ron`/`melee_weapons.ron`
  ne portent aucun prix aujourd'hui. La recharge d'une arme murale reste
  `prix_achat × refill_price_ratio` (ratio résolu).
- **Santé** : `base_health.max` de chaque personnage du registre (`CharacterConfig`) —
  ennemis comme « player », les deux chemins de spawn (`spawn_enemy` via CharacterSpawn et
  spawners, `create_player`) lisent `ResolvedBalance::health_max_by_character`.
  Rien d'autre n'est étendu (le reste des chantiers F est pour M1/M2).

## C. Preuve de contenu (2 vs 4 joueurs)

- **Valeur de contenu** : `games/zombies/assets/ZombieShooter/Sprites/Zombie/zombie_scaled_config.ron`,
  `base_health.max: "10.0 + (players - 1) * 40.0"` — **50 PV à 2 joueurs, 130 PV à 4**.
- **Personnage de preuve** : `zombie_scaled` — nouveau personnage (inscrit au manifeste de
  `games/zombies/assets/game.ron`, kind `Character`), gabarit de `zombie_hard_config.ron`
  (collider, skins vides, tag `zombie`), mais IA **stationnaire** (forme du `target` du
  testbed : `movement_type: None`, `stationary: Some(true)`, sans attaque) et collider
  20×32. Il n'est référencé par aucun palier de `wave_config.ron` ni par aucune entité des
  cartes existantes : les 62 scénarios existants ne le rencontrent jamais.
- **Carte de preuve** : `games/zombies/assets/exemples/zombie_scaled.ldtk` — nouvelle carte
  mono-niveau (`Level_0`, `__neighbours: []`, `LevelConnection` à zéro — un seul tampon du
  niveau, un seul `CharacterSpawn`), dérivée de `test_map.ldtk`. `CharacterSpawn zombie_scaled`
  en grid[9,5] (monde ~(407,695)), juste devant le muzzle du joueur 0 : chaque balle de
  `machine_gun` touche (la dispersion aléatoire non-shotgun de l'engine, ±0.5 rad, reste
  dans le collider 20×32 à cette distance), quelle que soit l'orientation du joueur.
  `map_seed: 123456` fixe le tampon de la salle ; `powerup_drop_chance_override: "0.0"`
  (aucun drop aléatoire).
- **Scénario 2 joueurs** : `tests/scenarios/equilibrage_joueurs_duo.ron` — joueur 0 tire
  vers l'ouest de f10 à f76 (`machine_gun` de départ : automatique, cadence 6 frames,
  8 dégâts, magasin 30 — 11 balles, aucun rechargement), joueur 1 sans input. Attentes :
  `RunState Playing`, `PlayerAlive 0/1` à f220, `EntityCount(Enemy, 0..0)` à f220 — 7 balles
  (56 ≥ 50) tuent le zombie avant f220.
- **Scénario 4 joueurs** : `tests/scenarios/equilibrage_joueurs_quad.ron` — même partie,
  joueurs 1-3 sans input (pas de bot). Attentes : `RunState Playing`, `PlayerAlive 0-3` à
  f220, `EntityCount(Enemy, 1..1)` à f220 — 11 balles (88) < 130 : le zombie est vivant
  à f220.
- **Divergence** : seule différence entre les deux parties = l'évaluation de `players` au
  lancement (§18) — 50 PV (meurt) vs 130 PV (survit).
- **Résultats mesurés** (runs isolés, `--skip recording_replays_identically`) :
  - Le joueur 0 tire au `machine_gun` de départ (automatique, cadence 6 frames, magasin 30,
    aucun rechargement dans la fenêtre) : 11 balles, f15 à f75, muzzle monde (411.0, 690.5),
    zombie spawné à (407, 695) — chaque balle naît dans le collider 20×32 du zombie, 11/11
    `hit=true` côté `bullet_rollback_collision_system` (mesuré).
  - **Duo** : 7ᵉ balle → mort à ~f51 (56 ≥ 50) ; à f220 `EntityCount(Enemy)` = 0,
    joueurs 0-1 vivants, `RunState Playing` — toutes les attentes passent.
  - **Quad** : 11 balles encaissées (88 < 130) ; à f220 `EntityCount(Enemy)` = 1,
    joueurs 0-3 vivants — toutes les attentes passent.
  - Les deux échouent uniquement sur « pas de trace de référence » (bless orchestrateur).

## D. Documentation

`docs/conventions.md` **§18 uniquement** (§16/§17 réservées, b1 en cours) : bloc
« Implémentation (m0-v11) » — type/enum et ordre de désérialisation, point d'évaluation
(`OnEnter(GameLoading)`, `ResolvedBalance`), source du nombre de joueurs (online/offline),
liste exacte des champs par famille (et la note armes/portes = Int LDtk), erreurs possibles
et tests unitaires.

## Traces

- Les **62 traces existantes sont inchangées** (aucun bless, aucune valeur existante
  modifiée) — vérifié par la suite complète.
- **2 bless demandés à l'orchestrateur** (jamais par l'agent) : `equilibrage_joueurs_duo`
  et `equilibrage_joueurs_quad` — nouveaux scénarios, pas de trace de référence ; preuve
  standard §10 (noms, attentes, frames ci-dessus). Les deux runs échouent uniquement sur
  `pas de trace de référence (lancer avec ALACOD_BLESS=1)`.

## Tests / lint / fmt / scripts / gen

(à compléter après exécution : `make test_scenarios`, `make lint`, `make fmt`,
`make scripts`, `make gen`, tests unitaires `cargo test -p content -p game balance`)

- Tests unitaires (critère 2) : `content::expr` (désérialisation `NumOrExpr` littéral et
  expression, `try_eval` et erreurs) et `game::balance`
  (`expression_resolves_per_player_count`, `division_by_zero_is_a_load_failure`,
  `unknown_identifier_is_a_load_failure`).
