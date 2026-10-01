# Contenu du jeu `zombies`

Clone type Call of Duty Zombies construit sur l'engine alacod (`docs/plan-engine.md`). Tout le
contenu (cartes, personnages, armes, économie, HUD) est chargé à l'exécution par le manifeste
`assets/game.ron` (format et kinds : `docs/conventions.md` §3, `crates/content`). Le binaire
est `games/zombies/src/main.rs` ; le plugin de partie est partagé avec `testbed`
(`map_ldtk::game::local::LdtkLocalGamePlugin`).

## Arborescence (`games/zombies/assets/`)

| Chemin | Kind | Contenu |
|---|---|---|
| `game.ron` | manifeste | dossiers de contenu et leur kind, carte de départ, seed par défaut |
| `camera.ron` | `Camera` | réglages de caméra (`CameraSettings`, suivi en ligne) |
| `ZombieShooter/Sprites/Character/player_config.ron` | `Character` | personnage joueur : `starting_weapons`, `weapon_slots`, vie, à terre |
| `ZombieShooter/Sprites/Character/weapons.ron` | `Weapon` | armes à distance : `pistol`, `machine_gun`, `shotgun`, `rifle` |
| `ZombieShooter/Sprites/Character/*_sheet.ron`, `*_animation.ron` | — | feuilles de sprites et animations du joueur et des armes (nommées dans `crates/game/src/global_asset.rs`) |
| `ZombieShooter/Sprites/Zombie/zombie*_config.ron` | `Character` | zombies : `zombie`, `zombie_hard`, `zombie_full` (+ feuilles et animations) |
| `ZombieShooter/Sprites/Obj/Weapons.png` | — | feuille des sprites d'armes (trois armes) |
| `weapons/melee/melee_weapons.ron` | `MeleeWeapon` | armes de mêlée (`bare_hands`, `knife`, `club`, `sword`, `axe`, `zombie_claws`) |
| `waves/wave_config.ron` | `Wave` | vagues : effectifs, cadence, montée en difficulté |
| `economy/economy.ron` | `Economy` | points par kill, coup et réparation ; ratio de recharge d'une arme murale déjà possédée |
| `economy/perks.ron` | `Perk` | perks : `juggernog`, `speed_cola`, `double_tap`, `stamin_up` |
| `items/powerups.ron` | `PowerUp` | table des power-ups : `insta_kill`, `double_points`, `max_ammo`, `carpenter`, `nuke` (`drop_chance`, poids, actions) |
| `exemples/test_map.ldtk` | `Map` | gabarits de salles de la carte jouable (trois niveaux LDtk, voir ci-dessous) |
| `exemples/test_map_shop.ldtk` | `Map` | copie de `test_map` avec une arme murale et un perk dans le gabarit de départ, pour les scénarios d'achat |
| `exemples/atlas/` | — | tuiles des cartes |
| `ui/hud.ron`, `ui/feedback.ron` | `Ui` | HUD (sources `health`, `wave`, `ammo`, `weapon`, `enemies`, `players`, `currency`) et retours (flash, secousse, sons) |
| `fonts/`, `sounds/` | — | police et sons |

## Power-ups (T2.5, chantier C1 v0)

Cinq power-ups de référence CoD : `insta_kill`, `double_points`, `max_ammo`, `carpenter`,
`nuke` (`items/powerups.ron`, kind `PowerUp` — voir `docs/conventions.md` §13). Un ennemi
tué (équipe `Enemies`) a `drop_chance` (15 %) de laisser tomber un power-up, choisi par
tirage pondéré (`weight`) parmi la table, dans le flux RNG nommé `loot`
(`crates/game/src/powerups.rs`). Un power-up au sol se ramasse **au passage** (pas
d'interaction à bouton, contrairement aux armes/fenêtres/perks) et s'applique à **tous les
joueurs** de la partie (sémantique CoD) ; il disparaît après `lifetime_frames` (≈ 30 s) s'il
n'a jamais été ramassé.

## La carte : des gabarits assemblés

Une partie ne joue pas `test_map.ldtk` tel quel : le chargeur (`crates/map_ldtk/src/loader`)
passe le projet au générateur (`crates/map/src/generation`, mode `Basic`), qui tire au sort avec
la seed (`default_seed` de `game.ron`, `seed` d'un scénario) une salle de départ parmi les niveaux
marqués `spawn: true`, puis accroche d'autres salles par leurs cases `LevelConnection`, jusqu'à
`max_room` (10) ou faute de connexion libre. Chaque niveau LDtk est un **gabarit** réutilisable :
avec la seed des scénarios, la partie compte quatre salles, dont deux copies de `Level_0` (la
salle de départ et une salle ordinaire), donc deux murs `pistol` et deux Juggernog. Les joueurs
apparaissent dans la salle de départ (`PlayerSpawn` du gabarit de départ seulement).

## Armes murales et perks de `test_map.ldtk`

| Gabarit | Entité LDtk | Contenu | Prix |
|---|---|---|---|
| `Level_0` (départ) | `WeaponLocation` | `pistol` | 500 |
| `Level_0` (départ) | `SodaLocation` | `juggernog` (vie max ×2) | 2500 |
| `Level_1` | `WeaponLocation` | `rifle` | 1200 |
| `Level_1` | `WeaponLocation` | `shotgun` | 1000 |
| `Level_1` | `SodaLocation` | `speed_cola` (rechargement ×0,5) | 3000 |
| `Level_2` | `WeaponLocation` | `machine_gun` | 1500 |
| `Level_2` | `SodaLocation` | `double_tap` (cadence ×1,5) | 2000 |
| `Level_2` | `SodaLocation` | `stamin_up` (vitesse +25 %) | 2000 |

Le prix d'une arme murale est un champ de l'entité (`price`), celui d'un perk vient de
`economy/perks.ron`. Le joueur démarre avec `machine_gun`, `pistol` et `shotgun`
(`player_config.ron`) : acheter une arme déjà possédée recharge ses munitions pour
`refill_price_ratio` × prix (`economy.ron`) ; le `rifle` est la seule arme murale qu'il ne
possède pas au départ. Un scénario peut restreindre l'arme de départ (`weapon:` dans
`PlayerScript`, voir `tests/scenarios/shop_tour.ron`).

## Ajouter du contenu

### Une arme à distance

1. Une entrée dans `weapons.ron` : `config` (`ammo_type`, `firing_modes` avec cadence, dispersion,
   balle, portée, chargeur ; `test` = bornes du scénario généré, voir ci-dessous), `sprite_config`
   (`name` = feuille `<name>_sheet.ron`, `index` = case de la feuille), `audio_config`.
   Les nombres `Fixed` sont des chaînes (`"4.0"`).
2. Facultatif : l'ajouter aux `starting_weapons` d'un personnage (les traces de tous les scénarios
   changent : bless justifié).
3. La poser sur un mur (section suivante).
4. `make lint`, puis `make test_scenarios` : chaque arme reçoit un scénario généré
   (`alacod-gen`, `crates/scenario/src/generate.rs`) qui vérifie `min_hits`/`max_hits` ;
   `cargo run -p scenario --profile headless --bin alacod-gen -- --play` affiche les coups
   observés pour poser les bornes.

### Un perk

1. Une entrée dans `economy/perks.ron` : `name`, `price`, `modifiers: [(stat, op, value)]`
   (`op` : `Add`, `Mul`, `Pct` ; stats de `sim_core::StatId`).
2. Une `SodaLocation` avec `perk: "<id>"` sur la carte.
3. `make lint` (référence de perk inconnue refusée).

### Une arme murale ou une machine à perk sur la carte

Dans LDtk (1.5.3), poser une entité `WeaponLocation` (champs `weapon`, `price`) ou
`SodaLocation` (champ `perk`) sur une case libre contre un mur d'un gabarit, jamais sur un spawn,
une porte ou une fenêtre (le générateur recopie l'entité dans chaque salle faite de ce gabarit). Les définitions de champs existent dans `test_map.ldtk` et `test_map_shop.ldtk`.

À la main dans le JSON : copier une instance existante du même type et garder toutes ses
clés (`__grid`, `__pivot`, `__tags`, `__tile`, `__smartColor`, `iid` unique, `width`, `height`,
`defUid`, `px`, `fieldInstances`, `__worldX`, `__worldY`) ; chaque champ porte `__identifier`,
`__type`, `__value`, `__tile`, `defUid` (uid du champ dans `defs.entities[].fieldDefs`) et
`realEditorValues`. Le chargeur refuse une instance incomplète (`missing field`). Garder le
format natif de LDtk (tabulations, tableaux de nombres sur une ligne) pour un diff lisible.

Toute entité ajoutée à `test_map.ldtk` crée des entités rollback : les traces de tous les
scénarios sur cette carte changent dès la frame 0. Vérifier avec `scripts/trace-diff.py` que
seules les nouvelles entités et la numérotation `GgrsNetId` bougent, puis
`BLESS=1 make test_scenarios` avec la justification dans le commit (`docs/conventions.md` §8).

## Vérifier

```bash
make lint                                   # alacod lint games/zombies (et testbed)
make test_scenarios SCENARIO=shop_tour      # achat d'une arme murale puis d'un perk sur test_map
make play_scenario SCENARIO=shop_tour       # le même, avec rendu
make zombies                                # jouer (fenêtre, joueur local)
```

## Limites connues

- Les feuilles de sprites sont nommées dans `crates/game/src/global_asset.rs` et restent
  mélangées aux configs sous `ZombieShooter/Sprites/**` (`docs/plan-engine.md` §5 A5) ; le
  `rifle` réutilise le sprite du fusil à pompe.
- Le prix des armes murales et des perks n'est pas affiché à l'écran.
- Power-ups : pas de sprite de pickup au sol, ni d'icône/minuteur à l'écran pour un power-up actif (T2.12 HUD v1) ; pas de
  son/flash au ramassage (T2.13) ; `KillAllWaveEnemies` (Nuke) ne crédite aucun point de kill
  (contrairement à CoD) — simplification v0, voir le rapport de la tâche T2.5.
