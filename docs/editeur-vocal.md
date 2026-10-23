# Éditeur par l'IA : guide pour Claude Code et pour William (chantier K5, v0 du lot K5-0)

Plan : `docs/taches/plan-editeur-vocal.md`. William joue ; Claude modifie les RON de `games/<jeu>/assets` et pilote la partie en marche ;
le rechargement à chaud montre le résultat. Aucun éditeur graphique : le client (Claude Code, ou un client vocal qui l'héberge) n'a besoin que
d'un shell.

## 1. Lancer

```bash
make editor GAME=zombies                # fenêtre, rendu, clavier jouable
make editor GAME=zombies HEADLESS=1     # sans fenêtre (VPS, CI) : profil headless
make editor GAME=zombies SEED=42        # graine de carte imposée (--seed, D105)
```

`GAME` = un dossier de `games/` (zombies, gungeon, isaac, throne, testbed). La cible lance le binaire du jeu avec :

| Variable | Rôle |
|---|---|
| `ALACOD_EDITOR=1` | mode éditeur (K5-1) : implique le remote ; la partie **tourne dès le départ** (pas de `resume`), le **clavier reste la source d'input**, `check_distance` forcé à 0 |
| `ALACOD_REMOTE=1` | serveur BRP (JSON-RPC sur HTTP) `127.0.0.1:15702` ; `ALACOD_REMOTE_PORT` change le port |
| `ALACOD_LABELS=1` | étiquettes F6 au-dessus des entités (nom, prix, état) |
| feature `native` | `bevy/file_watcher` : le rechargement à chaud des assets |

En mode éditeur la partie n'est pas en pause : on joue au clavier pendant que Claude édite. `alacod-remote input Right` **prend la main** sur le
clavier pour ce handle (le joueur avance seul) ; `alacod-remote input ""` (vide) **rend la main au clavier**. Hors mode éditeur (`make remote`),
le comportement d'avant est inchangé : partie en pause, clavier coupé, `step` pour avancer.

Il n'y a pas d'option `--map` dans la ligne de commande des jeux : la carte vient du manifeste (`entry.start_map` de `game.ron`) ; `SEED` fait varier
l'assemblage. Le client vocal et Claude parlent au même port ; l'ordre de lancement n'importe pas.

## 2. Outils

- **`scripts/alacod-remote`** (Python, `urllib`, sans dépendance) : `state` (JSON complet), `brief` (une ligne par joueur + ennemis proches),
  `input Up,Fire --pan 100,0 [--handle 0]` ; **`input ""` relâche et rend la main au clavier**, `step 30 [--no-wait]`, `pause`, `resume`,
  `screenshot <png>`, `save <scénario.ron>`, `discover`.
  Mode éditeur (K5-1) : `restart [--kind {Restart,ToLobby,EnterRun,ToHub}] [--seed N]`, `modifier [--handle 0|--net-id N] [--until FRAMES] [--source S] STAT {Add,Pct,Mul,Set} VALEUR`,
  `unmodify [--handle 0|--net-id N] [--source S]`, `reload`. `brief` commence par `editor=… online=… check_distance=… run.step=…`.
- **Méthodes `alacod/*` du mode éditeur** (JSON-RPC, `http://127.0.0.1:15702`) :
  - `alacod/restart` `{kind?: "Restart"|"ToLobby"|"EnterRun"|"ToHub", seed?: i32}` : pose un `RunRequest` ; le jeu repasse par `GameLoading`
    (tables dérivées du `Registry` recalculées, carte rechargée) puis `InGame`. `seed` écrit `MapGenerationConfig.seed` ; refusé avec `ToLobby`.
  - `alacod/set_modifier` `{handle?|net_id?, stat, op: Add|Pct|Mul|Set, value: "0.9", source?: "editor", until_frames?}` : modificateur
    **vivant** sur une entité (joueur 0 par défaut) ; `stat` = un `StatId` (`MoveSpeed`, `MaxHealth`, `Damage`, `FireRate`, `ReloadSpeed`…) ou `custom:<nom>`.
  - `alacod/clear_modifiers` `{handle?|net_id?, source?: "editor"}` : retire ceux de la source.
  - `alacod/reload_content` : relit le `Registry` tout de suite ; renvoie les compteurs et `errors` (liste vide = remplacé). Les tables dérivées
    prennent effet au prochain `GameLoading` (`restart` ou changement d'étage).
  - `alacod/state` s'enrichit : `editor`, `online`, `check_distance`, `game`, `map`, `seed`, `run.step`, `config` de chaque ennemi.
  - **Garde** : `restart`, `set_modifier`, `clear_modifiers` et `reload_content` sont refusées si la session est en ligne ou si
    `check_distance > 0` ; l'erreur JSON-RPC nomme la cause (« check_distance=2 »…).
- **`scripts/alacod-remote discover`** : appelle `rpc.discover` (OpenRPC 1.3.2) et liste les méthodes (les `alacod/*`, puis les intégrées
  `world.*`, `registry.schema`, `schedule.*`). Les `world.*` exigent des types `Reflect` : presque aucun type de simulation ne l'est ;
  **l'état de la partie se lit par `alacod/state`**, pas par `world.query`.
- **`alacod lint games/<jeu>`** (`cargo run -q -p content --bin alacod --profile headless -- lint games/<jeu>`) : valide manifeste et RON.
  `--json` pour une sortie lisible par machine. Code de sortie 1 s'il y a des erreurs.
- **`alacod describe games/<jeu> [--json] [Kind]`** : les kinds du jeu et leurs champs. **Ne pas recopier cette liste : l'appeler** quand on ne sait pas
  quel champ porte un réglage.
- **Hook de lint** (`.claude/settings.json` → `scripts/lint-edited-game.sh`) : après chaque `Edit`/`Write` sous `games/<jeu>/`, relance
  `alacod lint games/<jeu>` (asynchrone, 300 s). Un lint rouge est à corriger avant de dire « fait ».
- `git` : `git revert --no-edit HEAD` pour « annule ça ».

## 3. Vocabulaire → champs RON

Les nombres `Fixed` sont **des chaînes** (`"150.0"`), les entiers (frames, prix, tailles de chargeur) sont des nombres. Chemins relatifs à
`games/zombies/assets/` (vérifiés le 2026-10-22). Pour un autre jeu : `alacod describe games/<jeu>`.

| Ce que dit William | Fichier | Champ |
|---|---|---|
| vitesse du joueur | `ZombieShooter/Sprites/Character/player_config.ron` | `movement.max_speed` (150) |
| le joueur démarre / s'arrête plus vite | idem | `movement.acceleration` (3000), `movement.deceleration` (3000) |
| sprint | idem | `movement.sprint_multiplier` (2.0) |
| dash (portée, durée, recharge, invulnérabilité) | idem | `movement.dash_distance` (64), `dash_duration_frames` (8), `dash_cooldown_frames` (24), `dash_iframes` (6) |
| vie du joueur, régénération | idem | `base_health.max` (100), `regen_rate`, `regen_delay_frames` |
| vitesse d'un zombie | `ZombieShooter/Sprites/Zombie/zombie_config.ron` (et `zombie_hard_`, `zombie_full_`, `zombie_scaled_`, `boss_zombie_config.ron`) | `movement.max_speed` (120 pour `zombie_1`) |
| vie d'un zombie | idem | `base_health.max` (50) |
| dégâts, cadence, portée d'attaque d'un zombie | idem | `ai: Some((attack_damage, attack_cooldown_frames, attack_range, aggro_range))` |
| dégâts d'une arme | `ZombieShooter/Sprites/Character/weapons.ron` | `<arme>.config.firing_modes.<mode>.bullet_type.Standard.damage` (pistolet : 15.0) |
| cadence de tir | idem | `…firing_modes.<mode>.firing_rate` (tirs par seconde ; pistolet 2.0, mitrailleuse 10.0) |
| dispersion | idem | `…firing_modes.<mode>.spread` (pistolet 0.0005, mitrailleuse 0.15) ; fusil à pompe : `firing_mode: Shotgun(pellet_count, spread_angle)` |
| recul | idem | `…firing_modes.<mode>.recoil` |
| taille du chargeur | idem | `…firing_modes.<mode>.mag: Mag(mag_size, mag_limit)` |
| temps de recharge | idem | `…firing_modes.<mode>.reload_time_seconds` |
| arme de mêlée | `weapons/melee/melee_weapons.ron` | un bloc par arme (`alacod describe games/zombies MeleeWeapon`) |
| nombre de zombies par vague | `waves/wave_config.ron` | `base_enemies` (3), `enemies_per_wave` (2), `max_random_variance` (2) |
| zombies en même temps | idem | `max_concurrent_enemies` (4) |
| délai entre deux apparitions, entre deux vagues, préparation | idem | `spawn_interval_frames` (120), `min_wave_delay_frames` (600), `grace_period_frames` (180) |
| les vagues durcissent plus vite | idem | `health_multiplier_per_wave` (0.05), `damage_multiplier_per_wave` (0.03) |
| quels zombies sortent | idem | `wave_tiers[].enemy_probabilities` |
| points par kill, par coup, par réparation | `economy/economy.ron` | `kill_points` (60), `hit_points` (10), `repair_points` (10), `nuke_points` (400) |
| prix d'un perk | `economy/perks.ron` | `<perk>.price` (juggernog 2500) ; effet : `<perk>.modifiers` |
| prix d'une porte ou d'une arme murale | `maps/la_fonderie.ldtk` (champ `price` de l'entité) | édité dans LDtk, pas à la main (voir section « assets » ci-dessous) |
| flash d'impact | `ui/feedback.ron` | `hit_flash.frames` (4), `hit_flash.color` |
| secousse d'écran | idem | `shake.frames` (8), `shake.amplitude` (12.0) |
| hit-stop (arrêt sur image au coup) | idem | `hit_stop_frames` (2) |
| nombres de dégâts flottants | idem | `damage_numbers` (true) |
| volume d'un son | idem | `volumes.<son>` (dB) |
| caméra : suivi, zoom | `camera.ron` | `lerp_speed` (12.0), `min_zoom` (0.6), `max_zoom_out` (3.0), `default_player_zoom` (1.0), `player_padding` (100.0) |
| HUD : position, taille, couleur, police | `ui/hud.ron` | `widgets[].anchor`, `offset`, `size`, `color`, `font_size` ; sources dans `content::ui::HUD_SOURCES` |

Un réglage relatif (« un peu plus vite ») : lire la valeur, changer de 10 à 15 %, l'écrire, le dire. Valeur nulle ou ×1000 : le lint ne juge pas
l'équilibre, c'est à Claude de refuser les absurdités.

Barème : « un peu » = 10 à 15 % de la valeur actuelle (« un peu plus vite » : ×1,1 à ×1,15 ; « un peu moins » : ×0,85 à ×0,9), « beaucoup » = ×2
(ou ÷2), « pas du tout » = on refuse ou on demande. Unités : vitesses en pixels par seconde (px/s) ; durées `*_frames` en frames d'un tick à 60 Hz
(60 frames = 1 s) ; `*_seconds` en secondes ; `firing_rate` en tirs par seconde ; dispersions en radians ; santé en points de vie (PV) ; sommes en
points (zombies) ou en sous (isaac, gungeon). Arrondir à une valeur lisible. Les chemins ci-dessous sont relatifs à `games/<jeu>/assets/` ; un champ
s'écrit `chemin.pointé` à travers les blocs du RON ; la colonne « Valeur actuelle » est le texte exact du fichier le 2026-10-22. Les tableaux
suivants sont les réglages de polissage ; pour tout autre : `alacod describe games/<jeu> [Kind]`.

### gungeon

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pistolero.ron` | `movement.max_speed` | `"150.0"` | px/s |
| il démarre et s'arrête plus vite | `characters/pistolero.ron` | `movement.acceleration` | `"3000.0"` | px/s² |
| la roulade va plus loin | `characters/pistolero.ron` | `movement.dash_distance` | `"64.0"` | px |
| la roulade dure plus longtemps | `characters/pistolero.ron` | `movement.dash_duration_frames` | `8` | frames |
| moins d'attente entre deux roulades | `characters/pistolero.ron` | `movement.dash_cooldown_frames` | `24` | frames |
| invulnérable plus longtemps en roulade | `characters/pistolero.ron` | `movement.dash_iframes` | `6` | frames |
| une charge de roulade en plus | `characters/pistolero.ron` | `dash.charges` | `Some(2)` | charges |
| les charges de roulade reviennent plus vite | `characters/pistolero.ron` | `dash.recharge_frames` | `Some(60)` | frames |
| plus de vie pour le joueur | `characters/pistolero.ron` | `base_health.max` | `"100.0"` | PV |
| le joueur se soigne plus vite | `characters/pistolero.ron` | `base_health.regen_rate` | `Some("10.0")` | PV/s |
| le pistolet fait plus mal | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.bullet_type.Standard.damage` | `"12.0"` | PV |
| le pistolet tire plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.firing_rate` | `"3.0"` | tirs/s |
| le pistolet est moins précis | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.spread` | `"0.001"` | radians |
| les balles du pistolet vont plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.bullet_type.Standard.speed` | `"380.0"` | px/s |
| le chargeur du pistolet est plus grand | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.mag.mag_size` | `12` | balles |
| le pistolet recharge plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.reload_time_seconds` | `"0.5"` | s |
| le revolver fait plus mal | `weapons/weapons.ron` | `revolver.config.firing_modes.default.bullet_type.Standard.damage` | `"30.0"` | PV |
| le fusil à pompe tire plus de plombs | `weapons/weapons.ron` | `fusil_pompe.config.firing_modes.default.firing_mode.pellet_count` | `8` | plombs |
| la mitraillette tire plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.firing_rate` | `"10.0"` | tirs/s |
| la mitraillette part moins dans tous les sens | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.spread` | `"0.14"` | radians |
| les ennemis font plus mal (leurs tirs) | `weapons/weapons.ron` | `arsenal.plomb.damage` | `"4.0"` | PV |
| le bullet kin va plus vite | `characters/bullet_kin.ron` | `movement.max_speed` | `"60.0"` | px/s |
| le bullet kin a plus de vie | `characters/bullet_kin.ron` | `base_health.max` | `"30.0"` | PV |
| le bullet kin tire plus souvent | `characters/bullet_kin.ron` | `ai.behaviors.Shoot.cooldown_frames` | `150` | frames |
| le bullet kin voit plus loin | `characters/bullet_kin.ron` | `ai.aggro_range` | `Some("500.0")` | px |
| le fuyard va plus vite | `characters/fuyard_kin.ron` | `movement.max_speed` | `"75.0"` | px/s |
| le shotgun kin a plus de vie | `characters/shotgun_kin.ron` | `base_health.max` | `"55.0"` | PV |
| le boss (gatling) a plus de vie | `characters/gatling.ron` | `base_health.max` | `"120.0"` | PV |
| le boss change de phase plus tôt | `characters/gatling.ron` | `boss.phases.until` | `Some(HealthBelow("0.5"))` | fraction de vie |
| le boss tire plus souvent | `characters/gatling.ron` | `boss.phases.repeat` | `Some(150)` | frames |
| plus de salles par étage | `floor_grammars/etage.ron` | `grammar.rooms` | `(10, 14)` | salles (min, max) |
| la boutique coûte plus cher (commun) | `economy/economy.ron` | `shop_prices.Common` | `15` | sous |
| la boutique coûte plus cher (rare) | `economy/economy.ron` | `shop_prices.Rare` | `50` | sous |
| les bottes donnent plus de vitesse | `objets/bottes.ron` | `modifiers` | `(stat: MoveSpeed, op: Pct, value: "0.25")` | +25 % |
| le chargeur rapide recharge plus vite | `objets/chargeur_rapide.ron` | `modifiers` | `(stat: ReloadSpeed, op: Pct, value: "0.3")` | +30 % |
| la pierre de soin soigne plus | `objets/pierre_de_soin.ron` | `effects.do` | `Heal("20")` | PV |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| l'arrêt sur image au coup dure plus | `ui/feedback.ron` | `hit_stop_frames` | `2` | frames |
| le tir de pistolet est plus fort en volume | `ui/feedback.ron` | `volumes` | `"shot_pistol": -12.0` | dB |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |
| la vue est plus large | `camera.ron` | `min_zoom` | `0.5` | zoom (plus petit = plus large) |

### isaac

Le joueur d'isaac est `characters/pleureur.ron` (`asset_name_ref: "player"`) ; 1 PV = ½ cœur. Pas de roulade ni de sprint.

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pleureur.ron` | `movement.max_speed` | `"120.0"` | px/s |
| il démarre plus vite | `characters/pleureur.ron` | `movement.acceleration` | `"3000.0"` | px/s² |
| plus de vie pour le joueur | `characters/pleureur.ron` | `base_health.max` | `"6.0"` | PV (demi-cœurs) |
| plus de temps d'invulnérabilité après un coup | `characters/pleureur.ron` | `hit_iframes` | `Some(60)` | frames |
| la bombe explose plus vite | `characters/pleureur.ron` | `bomb.fuse_frames` | `75` | frames |
| la bombe a plus de portée | `characters/pleureur.ron` | `bomb.radius` | `"56.0"` | px |
| la bombe fait plus mal | `characters/pleureur.ron` | `bomb.damage` | `"4.0"` | PV |
| les larmes font plus mal | `weapons/weapons.ron` | `larmes.config.firing_modes.default.bullet_type.Standard.damage` | `"3.5"` | PV |
| les larmes partent plus vite (cadence) | `weapons/weapons.ron` | `larmes.config.firing_modes.default.firing_rate` | `"3.0"` | tirs/s |
| les larmes vont plus vite | `weapons/weapons.ron` | `larmes.config.firing_modes.default.bullet_type.Standard.speed` | `"260.0"` | px/s |
| les larmes vont plus loin | `weapons/weapons.ron` | `larmes.config.firing_modes.default.range` | `"300.0"` | px |
| les larmes sont moins précises | `weapons/weapons.ron` | `larmes.config.firing_modes.default.spread` | `"0.001"` | radians |
| les ennemis font plus mal (leurs tirs) | `weapons/weapons.ron` | `arsenal.plomb.damage` | `"1.0"` | PV |
| la mouche va plus vite | `characters/mouche.ron` | `movement.max_speed` | `"55.0"` | px/s |
| la mouche a plus de vie | `characters/mouche.ron` | `base_health.max` | `"4.0"` | PV |
| la mouche frappe plus fort | `characters/mouche.ron` | `ai.attack_damage` | `Some("1.0")` | PV |
| la mouche frappe plus souvent | `characters/mouche.ron` | `ai.attack_cooldown_frames` | `Some(120)` | frames |
| l'araignée va plus vite | `characters/araignee.ron` | `movement.max_speed` | `"66.0"` | px/s |
| le cracheur crache plus souvent | `characters/cracheur.ron` | `ai.behaviors.Shoot.cooldown_frames` | `300` | frames |
| le cracheur voit plus loin | `characters/cracheur.ron` | `ai.aggro_range` | `Some("130.0")` | px |
| le boss colosse a plus de vie | `characters/colosse.ron` | `base_health.max` | `"44.0"` | PV |
| le boss gardien a plus de vie | `characters/gardien.ron` | `base_health.max` | `"40.0"` | PV |
| plus de salles par étage | `floor_grammars/sous_sol.ron` | `grammar.rooms` | `(8, 12)` | salles (min, max) |
| plus de chances d'une salle du diable | `floor_grammars/sous_sol.ron` | `post_boss.chance` | `"0.33"` | probabilité |
| plus de chances d'une malédiction | `floors/sous_sol.ron` | `curses.chance` | `"0.3"` | probabilité |
| la boutique coûte plus cher (commun) | `economy/economy.ron` | `shop_prices.Common` | `15` | sous |
| la boutique coûte plus cher (rare) | `economy/economy.ron` | `shop_prices.Rare` | `50` | sous |
| le marché du diable coûte plus de cœurs | `rooms/diable.ron` | `shop.cost` | `Some(MaxHealth("2"))` | PV max |
| la pile rapide donne plus de cadence | `objets/pile_rapide.ron` | `modifiers` | `(stat: FireRate, op: Pct, value: "0.1")` | +10 % |
| le plomb fondu donne plus de dégâts | `objets/plomb_fondu.ron` | `modifiers` | `(stat: Damage, op: Pct, value: "1.0")` | +100 % |
| la lunette longue allonge la portée | `objets/lunette_longue.ron` | `modifiers` | `(stat: Range, op: Pct, value: "0.15")` | +15 % |
| l'œil vif accélère les larmes | `objets/oeil_vif.ron` | `modifiers` | `(stat: ShotSpeed, op: Pct, value: "0.2")` | +20 % |
| le réservoir vital donne plus de vie | `objets/reservoir_vital.ron` | `modifiers` | `(stat: MaxHealth, op: Add, value: "2")` | PV |
| l'élixir de vitesse dure plus longtemps | `objets/elixir_vitesse.ron` | `effects.do.frames` | `300` | frames |
| le cœur soigne plus | `objets/coeur.ron` | `effects.do` | `Heal("2")` | PV |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |
| la vue est plus large | `camera.ron` | `min_zoom` | `0.6` | zoom (plus petit = plus large) |

### throne

Le joueur de throne est `characters/pilote.ron` (`asset_name_ref: "player"`, trois armes au départ : `mitraillette`, `revolver`, `lance_lames`).

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pilote.ron` | `movement.max_speed` | `"150.0"` | px/s |
| la roulade va plus loin | `characters/pilote.ron` | `movement.dash_distance` | `"64.0"` | px |
| moins d'attente entre deux roulades | `characters/pilote.ron` | `movement.dash_cooldown_frames` | `24` | frames |
| invulnérable plus longtemps en roulade | `characters/pilote.ron` | `movement.dash_iframes` | `6` | frames |
| plus de vie pour le joueur | `characters/pilote.ron` | `base_health.max` | `"100.0"` | PV |
| le joueur se soigne plus vite | `characters/pilote.ron` | `base_health.regen_rate` | `Some("10.0")` | PV/s |
| la mitraillette fait plus mal | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.bullet_type.Standard.damage` | `"8.0"` | PV |
| la mitraillette tire plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.firing_rate` | `"10.0"` | tirs/s |
| la mitraillette part moins dans tous les sens | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.spread` | `"0.15"` | radians |
| le chargeur de la mitraillette est plus grand | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.mag.mag_size` | `30` | balles |
| la mitraillette recharge plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.reload_time_seconds` | `"1.5"` | s |
| le revolver fait plus mal | `weapons/weapons.ron` | `revolver.config.firing_modes.default.bullet_type.Standard.damage` | `"15.0"` | PV |
| le fusil à pompe tire plus de plombs | `weapons/weapons.ron` | `fusil_a_pompe.config.firing_modes.default.firing_mode.pellet_count` | `8` | plombs |
| le canon à ricochet fait plus mal | `weapons/weapons.ron` | `canon_ricochet.config.firing_modes.default.bullet_type.Standard.damage` | `"20.0"` | PV |
| le rat va plus vite | `characters/rat.ron` | `movement.max_speed` | `"60.0"` | px/s |
| le chien a plus de vie | `characters/chien.ron` | `base_health.max` | `"20.0"` | PV |
| le chien court plus vite | `characters/chien.ron` | `movement.max_speed` | `"90.0"` | px/s |
| la brute a plus de vie | `characters/brute.ron` | `base_health.max` | `"120.0"` | PV |
| le buffle frappe plus fort | `characters/buffle.ron` | `ai.attack_damage` | `Some("12.0")` | PV |
| le buffle frappe plus souvent | `characters/buffle.ron` | `ai.attack_cooldown_frames` | `Some(90)` | frames |
| le rôdeur voit plus loin | `characters/rodeur.ron` | `ai.sight_range` | `Some("900.0")` | px |
| le cracheur crache plus souvent | `characters/cracheur.ron` | `ai.behaviors.Shoot.cooldown_frames` | `60` | frames |
| la tourelle a plus de vie | `characters/tourelle.ron` | `base_health.max` | `"150.0"` | PV |
| le boss (roi rat) a plus de vie | `characters/roi_rat.ron` | `base_health.max` | `"360.0"` | PV |
| le roi rat frappe plus fort | `characters/roi_rat.ron` | `ai.attack_damage` | `Some("15.0")` | PV |
| le roi rat tire plus souvent | `characters/roi_rat.ron` | `ai.behaviors.Shoot.cooldown_frames` | `120` | frames |
| plus d'ennemis au premier niveau | `caves/niveau_1.ron` | `enemy_spawns` | `4` | ennemis |
| plus d'ennemis au troisième niveau | `caves/niveau_3.ron` | `enemy_spawns` | `9` | ennemis |
| la caverne du premier niveau est plus remplie de roche | `caves/niveau_1.ron` | `fill_ratio` | `"0.38"` | fraction |
| la partie durcit plus vite | `difficulty.ron` | `value` | `"1 + floor * 0.25 + floor_minutes * 0.25"` | formule |
| plus de chances de bonus à la mort | `items/powerups.ron` | `drop_chance` | `"0.40"` | probabilité |
| la rage dure plus longtemps | `items/powerups.ron` | `rage.frames` | `600` | frames |
| plus de chances de lâcher une arme | `progression/run.ron` | `weapon_drop_chance` | `"0.08"` | probabilité |
| plus de rads par mort | `progression/run.ron` | `per_kill` | `"1"` | rads |
| le vampire soigne plus | `mutations/vampire.ron` | `effects.do` | `Heal("4")` | PV |
| points par mort | `economy/economy.ron` | `kill_points` | `60` | points |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |

## 4. Règles pour Claude

1. **Une modification à la fois.** Lire le champ, l'éditer, constater, répondre.
2. **N'écrire que sous `games/<jeu>/assets`.** Jamais de Rust, de `Makefile`, de `tests/`.
3. **Lint vert avant de dire « fait »** : le hook tourne après chaque écriture ; sinon `alacod lint games/<jeu>` à la main. Lint rouge : corriger ou restaurer le fichier.
4. **Un commit `editor: <résumé>` par changement appliqué** (`git add` du seul fichier touché), ex. `editor: vitesse joueur 150 → 135`.
5. **« Annule ça »** : `git log -1 --format=%s` ; seulement si le sujet commence par `editor:`, `git revert --no-edit HEAD`. Sinon, le dire et ne rien faire.
6. **Dire en une phrase ce qui a changé et où** : « `movement.max_speed` du joueur : 150 → 135 (player_config.ron), pris en compte à la prochaine partie. »
   Dire aussi *quand* l'effet se voit (section 5).
7. Ne jamais pousser (`git push`), jamais `git stash`, jamais de commande destructive.

### Bloc prêt à coller

À mettre dans le `CLAUDE.md` du dépôt ou dans le prompt système du client vocal (12 lignes, lisibles à voix haute) :

```
Tu règles un jeu alacod en marche (make editor) pour William, qui parle : tes réponses sont lues à voix haute.
1. Une seule modification à la fois : lis la valeur dans le fichier (docs/editeur-vocal.md, section 3), écris-la, constate.
2. N'écris que sous games/<jeu>/assets, du jeu lancé. Jamais de Rust, de Makefile ni de tests.
3. Lint vert avant de dire « fait » : alacod lint games/<jeu>. Le hook est asynchrone : attends son verdict.
4. Un changement de fichier = un commit « editor: <résumé> » du seul fichier touché.
5. Un modificateur vivant (`alacod-remote modifier`) n'est pas un commit ; il s'annule par `alacod-remote unmodify --handle 0 --source editor`.
6. « Annule ça » : `git log -1 --format=%s` ; si le sujet commence par « editor: », `git revert --no-edit HEAD`. Si le dernier changement était un modificateur vivant, `alacod-remote unmodify --handle 0 --source editor`. Sinon dis-le et ne touche à rien.
7. Dis en une phrase quoi, où, et quand l'effet se voit : tout de suite, ou après `alacod-remote restart --kind Restart`.
8. « Un peu » = 10 à 15 %, « beaucoup » = fois deux. Refuse une valeur nulle ou absurde.
9. Phrase ambiguë ou identifiant incertain : pose une question courte, ou liste les ids (alacod describe) et propose le plus proche.
10. Réponses courtes : deux phrases, sans tableau, sans code, sans adresse.
11. Jamais git push, jamais git stash, jamais de commande destructive.
```

### Deux sortes de changement

| | Modification de fichier | Modificateur vivant |
|---|---|---|
| Commande | édition d'un RON sous `games/<jeu>/assets` + commit `editor: …` | `alacod-remote modifier --handle 0 MoveSpeed Mul 0.9 --source editor` |
| Dure | tant que le fichier ne change pas : survit au `restart` et à la relance | jusqu'au `restart` (le nouveau personnage repart sans modificateur) |
| S'annule par | `git revert --no-edit HEAD` (si HEAD commence par `editor:`) | `alacod-remote unmodify --handle 0 --source editor` |
| Usage | réglage voulu durable | essai immédiat sur le joueur vivant |

Pour « c'est un peu trop rapide » : `alacod-remote modifier --handle 0 MoveSpeed Mul 0.9 --source editor` donne l'effet tout de suite, puis l'édition du RON rend le réglage durable. Un changement qui n'existe que comme modificateur disparaît au `restart` : le dire.

## 5. Ce qui se recharge à chaud

Mesuré sur `make editor GAME=zombies HEADLESS=1` (2026-10-22) : une édition de `weapons.ron` donne au journal
`INFO game::weapon_assets: armes rechargées : WeaponRegistry mis à jour (4 armes)` en moins de 4 s.

| Donnée | Effet | Preuve (plan § 2) |
|---|---|---|
| tables de personnages (`*_config.ron`) → `CharacterConfigs` | **immédiat** pour les spawns futurs ; les entités vivantes gardent leurs `Stats` (copiées au spawn : « plus rapide » dans le RON ne change pas le joueur vivant → `alacod-remote modifier`, ou `restart`) | `character/config_asset.rs:44` |
| armes (`weapons.ron`) → `WeaponRegistry` et armes portées | **immédiat** (armes portées : `weapons_config_update_system`, `Update`) ; la mêlée n'est pas couverte (risque 7) | `weapon_assets.rs:55-56`, `:179-199` |
| `camera.ron`, `ui/feedback.ron`, `ui/hud.ron` | **immédiat** (présentation) | `camera/mod.rs:666-681`, `feedback.rs:71-86`, `ui/hud.rs:1750-1763` |
| `Registry` et `GameManifest` (items, salles, curses, patterns, statuts, progression, économie, perks, `game.ron`) | rechargé hors lobby **seulement en mode éditeur** (K5-1, session locale à `check_distance` 0 ; hors mode éditeur, en ligne ou à `check_distance` > 0, aucun remplacement hors lobby, `warn!` « registre non rechargé : <cause> ») : événement d'asset **et** scrutation des fichiers du contenu, une fois par seconde (natif ; la scrutation n'existe qu'en mode éditeur) ; les tables dérivées prennent effet au prochain `GameLoading` : `alacod-remote restart --kind Restart` ou changement d'étage (mode `Floors`). |
| balance, power-ups, vagues (`ResolvedBalance`, `PowerUpsTable`) | prennent effet au prochain `GameLoading` : `alacod-remote restart --kind Restart` ou changement d'étage (mode `Floors`) | `core.rs:166-176`, `balance.rs:5-26` |
| RON **invalide** | aucun rechargement : `ERROR contenu invalide, registre non rechargé` (scrutation) ou `Failed to load asset … Could not parse RON` (asset) au journal, la table en mémoire reste l'ancienne ; la correction suivante recharge | mesuré 2026-10-22 (écriture d'un RON invalide pendant que le jeu tourne) |

### Assets : images, sons, LDtk, polices

Audit complet : `docs/taches/rapports/k5-audit-hot-reload-assets.md`. En bref :

- **PNG des sprites** : immédiat (les sprites gardent leur handle ; Bevy recharge l'image). Garder le même chemin.
- **Grilles et animations** (`sprites/sprites.ron`, configs d'animation) : immédiat (layouts reconstruits sur `Modified`).
- **Sons** : à la prochaine lecture (prochain tir, coup…) ; le son en cours n'est pas remplacé.
- **LDtk et tilesets** (`maps/*.ldtk`, `wall_art`) : au `restart` seulement ; le PNG d'un tileset est immédiat, pas une nouvelle grille.
- **HUD, caméra, feedback** : immédiat (arbre UI reconstruit, paramètres recopiés sur `Modified`).
- **Polices** : suivies par handle, rechargées par Bevy.

## 6. Voix

- **Entrée** : la dictée du système. macOS : dictée (touche micro ou `Fn Fn`) dans le terminal de Claude Code. Rien à installer. La dictée écrit sans
  ponctuation ni majuscules et déforme les identifiants (`zombie_runner`) : Claude **liste** les ids (`alacod describe games/<jeu>`) et propose le
  plus proche au lieu de deviner. Un client vocal complet (Hermes ou autre) sert aussi d'entrée : il héberge Claude, section ci-dessous.
- **Sortie** : hook `Stop` de `.claude/settings.json` → `scripts/claude-say.sh`, **opt-in**. Sans `ALACOD_VOICE=1` il ne fait rien.
  Variables relues dans le script le 2026-10-22 (toutes présentes) :
  - `ALACOD_VOICE=1 claude` : à la fin de chaque réponse, la dernière réponse (blocs de code remplacés par « bloc de code », code en ligne, liens et
    marques Markdown retirés) est lue par `say` (macOS), sinon `espeak-ng`, sinon `espeak` (Linux, voix `fr`), sinon `piper` (avec
    `ALACOD_VOICE_PIPER_MODEL`), sinon écrite dans `$TMPDIR/alacod-voice.txt` (`/tmp` par défaut).
  - `ALACOD_VOICE_NAME=Thomas` : voix de `say` (macOS seulement) ; ignorée si la voix n'est pas dans la liste de `say -v '?'` (voix par défaut du
    système alors).
  - `ALACOD_VOICE_MAX=300` : longueur maximale lue en caractères (600 par défaut), coupée à la dernière phrase entière ; une valeur illisible redonne 600.
  - `ALACOD_VOICE=stop scripts/claude-say.sh` : coupe la lecture en cours, par `pkill say` : **macOS seulement**. Sous Linux : `pkill espeak-ng`
    (ou `pkill espeak`, `pkill piper`).
  - Tests du script : `python3 -m unittest scripts/tests/test_claude_say.py -v`.
- **Installer une voix française.**
  - macOS : Réglages Système > Accessibilité > Contenu énoncé > Voix du système > Gérer les voix, télécharger une voix française (Thomas, Amélie) ;
    vérifier : `say -v '?' | grep fr_FR`, essai : `say -v Thomas "bonjour"`, puis `ALACOD_VOICE_NAME=Thomas`.
  - Linux (Debian, Ubuntu) : `sudo apt install espeak-ng` ; essai : `espeak-ng -v fr "bonjour"`. La voix est celle de `fr`, robotique mais suffisante.
    Pour mieux : `piper` avec un modèle français, `ALACOD_VOICE_PIPER_MODEL=<fichier.onnx>`.
- **Boucle micro / voix.** La sortie lit la dernière réponse de Claude ; si le micro reste ouvert, il se transcrit lui-même et Claude répond à sa propre
  voix. Règle pour William : **couper la dictée pendant la lecture**, la rouvrir quand elle s'arrête (ou `ALACOD_VOICE=stop scripts/claude-say.sh`).
  Réponses courtes (section 4) : la lecture dure quelques secondes.
- **Client vocal complet** (Hermes ou autre) : il lui suffit d'un shell et de HTTP local ; pas de pont MCP (plan D5), sauf si William répond que son
  client n'a ni l'un ni l'autre (décision en attente : `docs/taches/rapports/k5-4-essai-vocal.md`). S'il n'a **que HTTP** : appeler `POST
  http://127.0.0.1:15702` en JSON-RPC (`rpc.discover` liste les méthodes `alacod/*`) et éditer les RON par l'outil de fichiers du client.
- **Client sur une autre machine.** Le serveur de la partie ne répond qu'à `127.0.0.1` (volontaire : on n'ouvre pas le port). Depuis la machine du
  client, tunnel SSH : `ssh -L 15702:127.0.0.1:15702 <machine>` (laisser la session ouverte) ; `alacod-remote` et `curl http://127.0.0.1:15702`
  parlent alors à la partie distante. Les fichiers RON restent sur la machine du jeu : Claude doit y tourner (ou y avoir un shell par SSH).

## 7. Limites

- **Local seulement** : session locale à `check_distance` 0 (pas de synctest), un joueur. Pas de p2p (une édition à chaud divergerait entre pairs, sans canal pour la répliquer), pas de web (ni file watcher ni remote en wasm).
- Les commandes de partie (`restart`, `set_modifier`, `clear_modifiers`, `reload_content`) sont refusées si la session est en ligne ou si `check_distance` ≠ 0 (le mode éditeur le force à 0 ; erreur JSON-RPC qui nomme la cause).
- Aucune vérification de rollback en mode éditeur : un réglage qui casserait le déterminisme ne se voit qu'aux scénarios (`make test_scenarios`).

## 8. Dépannage

Dans l'ordre : le plus fréquent d'abord. Chaque cas donne le signe, la cause, la commande.

- **Le jeu ne répond pas** (`alacod-remote` : connexion refusée, ou délai dépassé). Le jeu n'est pas lancé, ou pas par `make editor`, ou le port
  est autre. Tester : `curl -s -X POST -d '{"jsonrpc":"2.0","id":1,"method":"rpc.discover"}' http://127.0.0.1:15702`. Une réponse JSON : le jeu
  est là, le souci est ailleurs. Rien : relancer `make editor GAME=<jeu>` ; si `ALACOD_REMOTE_PORT` est posé, utiliser ce port. Le serveur n'écoute
  que `127.0.0.1` : depuis une autre machine, il faut le tunnel SSH de la section 6.
- **Commande refusée** (erreur qui nomme `online` ou `check_distance=N`). Les commandes de partie ne valent qu'en session locale à `check_distance`
  0 (section 7). Cause : jeu lancé en ligne, ou avec `ALACOD_CHECK_DISTANCE=2` dans l'environnement. Remède : relancer par `make editor`, sans
  `ALACOD_CHECK_DISTANCE`.
- **RON invalide** (journal : `Failed to load asset … Could not parse RON`). Le jeu garde l'ancienne valeur en mémoire et ne recharge rien. Lire
  l'erreur (fichier, ligne), corriger, ou revenir en arrière avec `git checkout -- <fichier>` (fichier non commité seulement). La correction suivante
  recharge d'elle-même. Les nombres `Fixed` sont des chaînes (`"150.0"`), pas des nombres nus.
- **`reload` avec `errors` non vide** (`alacod-remote reload`). Le registre n'est pas remplacé : l'ancien reste. Lire chaque erreur (un id inconnu, un champ mal écrit), corriger le fichier nommé, relancer `alacod-remote reload` jusqu'à `errors: []`, puis `alacod-remote restart --kind Restart` pour appliquer les tables dérivées. La commande échoue (code 1) si `errors` n'est pas vide.
- **Rien ne change à l'écran.** Relire la section 5 : une édition de registre ou de balance prend effet au prochain `GameLoading` (restart ou changement d'étage), tandis que les tables de personnages concernent les prochains spawns et que les armes, caméra et feedback sont immédiats. Utiliser `alacod-remote reload`, puis `alacod-remote restart --kind Restart` si nécessaire. `alacod-remote input ""` relâche l'input distant et rend la main au clavier ; il n'existe pas de sous-commande `spawn` dans K5-1. Vérifier aussi que le fichier modifié est bien sous `games/<jeu>/assets` **du jeu lancé** (`alacod-remote brief` : la première ligne donne le jeu) et que le journal montre le rechargement (`armes rechargées` pour `weapons.ron`).
- **Lint rouge.** Lire le message de `alacod lint games/<jeu>` (le hook asynchrone rend la main avant la fin : attendre son verdict ou lancer la
  commande soi-même). Corriger le champ nommé ou restaurer le fichier ; ne pas committer `editor:` tant que le lint n'est pas vert.
- **La voix ne lit rien.** Vérifier `ALACOD_VOICE=1` dans l'environnement de `claude` (le hook n'agit pas sans), puis qu'une commande de synthèse
  existe (`say`, `espeak-ng`, `espeak`, `piper` avec `ALACOD_VOICE_PIPER_MODEL`) ; sans elles, le texte est écrit dans `$TMPDIR/alacod-voice.txt`.
- **Le micro transcrit la voix de Claude.** Couper la dictée pendant la lecture (section 6) ; `ALACOD_VOICE=stop scripts/claude-say.sh` arrête la lecture.
