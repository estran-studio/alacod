# Rapport — m1-v2-contenu-throne : le jeu `throne`, phase 1 (T1.0c + T1.11 sans mutations, voie V2)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport) ; branche partie de
`4f772d8` (LIVRÉ T1.10, contient donc T1.10, pas encore dans main), base `origin/main` `e26a3e3`.
Fiche : [m1-v2-contenu-throne](../m1-v2-contenu-throne.md). Worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date :
2026-10-04.

## État en cours

- **Fait** : phase 1 (squelette et contenu sans mutations), deux scénarios, §29, `assets.yaml`.
- **Chiffres** : 19/20 graines finissent la run (3 étages), 0 desync, 0 mort ; traces existantes
  identiques ; 2 traces nouvelles `throne_*` à bénir (plus les 7 de T1.10, sur la branche).
- **Prochaine étape** : LIVRÉ phase 1 ; phase 2 (mutations, progression, horloge) sur
  `m1-v2-contenu-throne-p2` quand T1.9 et T1.10 seront dans main.

## Fait

- **Squelette** : `games/throne/{Cargo.toml, src/main.rs}` (copies du testbed renommées), membre
  du workspace (`games/*`), `make throne`, `make lint` linte les trois jeux. Manifeste : `entry:
  (start_map: "cave:niveau_1", default_seed: 123456, mode: Floors)`, `floors/run.ron` = trois
  cavernes.
- **Joueur** `characters/pilote.ron` (id `player`, copie du testbed) ; armes de départ
  `mitraillette`, `revolver`, `lance_lames`.
- **Douze armes** sur cinq munitions `Custom` (`balles`, `obus`, `explosifs`, `energie`, `lames`,
  deux ou trois armes chacune), projectiles composables (`Bounce`, `Pierce`, `Size`, `Lifetime`,
  `Homing`, `Gravity`, `on_expire: Spawn` : explosion + couronne, souffle, éventail) ; plus
  `arsenal` (arme des ennemis, table `projectiles` de leurs patterns) ; mêlée `bare_hands`,
  `griffes`, `crocs`, `massue`. `test:` sur les treize armes à distance.
- **Dix ennemis** en behaviors T1.4 : mêlée `rat`, `chien`, `brute`, `rodeur` (`Wander`) ; tireurs
  `cracheur` (`Aimed`), `arroseur` (`Spread`), `tourelle` (`Ring`) ; chargeur `buffle` ; kiter
  `franc_tireur` ; fuyard `pillard`. Variantes T1.5 sur `rat`, `chien`, `brute`. Quatre patterns.
- **Trois cavernes** `niveau_1..3` (48 × 32 → 64 × 44, 4 → 8 ennemis, `characters` par niveau),
  gabarit et tilesets copiés du testbed.
- **Butin** `items/powerups.ron` : `munitions` (`RefillAmmo`), `rage`, `vitesse`.
- **Assets** : placeholders copiés du testbed, `games/throne/assets/assets.yaml` (7 entrées,
  licences reprises de zombies, `statut: placeholder`) : sprites du joueur (pack blackHUNTERdev),
  `Weapons.png`, `Slash_strip3.png` (source inconnue, dette V6 existante), sons `machine-gun*.ogg`,
  police Fira Mono, tilesets SunnyLand et Dungeon. Ennemis sans sprite.
- **Scénarios** `throne_floor_1`, `throne_three_floors` ; **docs** `docs/conventions.md` §29.

## Réglages trouvés par la mesure

- **Cavernes plus ouvertes** (`fill_ratio` 0,45 → 0,38) et `rodeur` qui voit à 300 (au lieu de
  120) : avec 0,45, 3 graines sur 3 bloquées à l'étage 1. Le bot `prudent` va en ligne droite vers
  l'ennemi ou le portail et reste contre la roche (dette T1.14 : pas de pathfinding pour
  `prudent`).
- **Munitions** : avec deux armes à balles seulement, 17/20, et les 3 softlocks sont des bots à
  sec à côté du dernier ennemi. Troisième arme de départ sur une autre munition (`lance_lames`),
  réserves du revolver ×2 et de la mitraillette ×1,5, butin 0,15 (munitions 60 %) : 19/20.
- **Jauges `test:`** calées sur la mesure (voir « Vérifié »).

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=2`, profil headless.

- **Critère des 20 graines** : `alacod-sim --game throne --bots 2 --profiles prudent,prudent
  --floors run --seeds 1..20 --until-floor 3 --max-frames 12000` : **19/20 au 3ᵉ étage**, 0 desync,
  0 mort, 78 points de dégâts au total, de 1 545 à 3 631 frames. **Graine 5** : softlock à l'étage
  2 (`niveau_2`) : le `pillard` à 16/40 fuit (`Flee` sous 50 %) et reste derrière un mur ; les
  bots, sans pathfinding, ne le contournent pas (ils ont encore 6 balles et 48 lames).
- `throne_floor_1` (un bot, graine 123456) : portail à f925 ; `throne_three_floors` (deux bots) :
  f655, f2166, f3399 (f3385 dans `alacod-sim`, qui garde le butin), aucun dégât.
- **Armes** : les scénarios générés, joués avec les armes de throne copiées **temporairement**
  dans le testbed (non commité), sont tous verts (17 : 13 à distance, 4 de mêlée). Coups mesurés :
  revolver 1, mitraillette 7, fusil à pompe 13, canon à ricochet 3, lance-grenades 31, roquette 4,
  mortier 2, laser 3, plasma 1, traqueur 3, lance-lames 4, disque 1, arsenal 2.
- `make test_scenarios` (sans bless) : **toutes les traces existantes identiques** ; échecs
  uniquement « pas de trace de référence » (`throne_floor_1`, `throne_three_floors`, et les 7 de
  T1.10). `bench_horde` 32,8 fps sous charge (plancher 38, rien de throne ne s'y joue).
- `cargo test` des quatorze crates : **471 réussis, 1 échec, 9 ignorés** ; l'échec est
  `scenarios`, uniquement sur les traces nouvelles.
- `make lint` sans erreur (**trois jeux** ; throne : 11 personnages, 13 armes, 4 de mêlée) ;
  `cargo fmt --all -- --check` vide ; `check-forbidden.sh` 4 occurrences préexistantes ;
  `check-rollback-registration.sh` OK ; `make gen GAME=testbed` et `GAME=zombies` code 0, aucun
  fichier modifié ; `cargo check --profile headless -p throne` et `cargo build --profile headless
  --examples` code 0.

## Non fait / non vérifié

- **`make gen GAME=throne` rouge** (décision (a) de l'orchestrateur) : le générateur joue tout dans
  l'arène du testbed, où les armes de throne n'existent pas (panique sur `arsenal`, premier id
  trié). Armes propres à throne, donc toutes : `arsenal`, `canon_ricochet`, `disque`,
  `fusil_a_pompe`, `lance_grenades`, `lance_lames`, `laser`, `mitraillette`, `mortier`, `plasma`,
  `revolver`, `roquette`, `traqueur`, et en mêlée `crocs`, `griffes`, `massue`. Les scénarios
  générés de throne ne sont pas versionnés : ils casseraient `make test_scenarios`. Attend le
  gabarit par jeu de T1.13.
- `test:` des ennemis (T1.13 pas dans main).
- Jeu fenêtré (`make throne`) compilé, pas lancé ; pas de p2p.
- Phase 2 (mutations, progression, `weapon_pool`, horloge).

## Dettes / questions ouvertes

- **Butin par munition** : `RefillAmmo` recharge toutes les munitions ; un butin « une munition »
  demande une action d'engine.
- **Pathfinding de `prudent`** (T1.14) : seule cause des softlocks restants ; les cavernes de throne
  sont réglées plus ouvertes en attendant.
- **Lint manquant** : rien ne vérifie qu'une caverne d'une séquence `Floors` a des ennemis (D36),
  ni qu'une munition `Custom` est fournie par au moins deux armes.
- `Slash_strip3.png` reste de source inconnue (dette V6 existante, recopiée).
