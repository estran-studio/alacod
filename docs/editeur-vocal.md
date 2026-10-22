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
| `ALACOD_EDITOR=1` | mode éditeur (lot K5-1 : clavier jouable, partie non en pause, `check_distance` 0 forcé) |
| `ALACOD_REMOTE=1` | serveur BRP (JSON-RPC sur HTTP) `127.0.0.1:15702` ; `ALACOD_REMOTE_PORT` change le port |
| `ALACOD_LABELS=1` | étiquettes F6 au-dessus des entités (nom, prix, état) |
| feature `native` | `bevy/file_watcher` : le rechargement à chaud des assets |

Il n'y a pas d'option `--map` dans la ligne de commande des jeux : la carte vient du manifeste (`entry.start_map` de `game.ron`) ; `SEED` fait varier
l'assemblage. Le client vocal et Claude parlent au même port ; l'ordre de lancement n'importe pas.

## 2. Outils

- **`scripts/alacod-remote`** (Python, `urllib`, sans dépendance) : `state` (JSON complet), `brief` (une ligne par joueur + ennemis proches),
  `input Up,Fire --pan 100,0 [--handle 0]`, `step 30 [--no-wait]`, `pause`, `resume`, `screenshot <png>`, `save <scénario.ron>`, `discover`.
  Les sous-commandes `restart`, `modifier`, `unmodify` et `reload` arrivent avec le lot K5-1 (section 6 de ce fichier).
- **`scripts/alacod-remote discover`** : appelle `rpc.discover` (OpenRPC 1.3.2) et liste les méthodes (les sept `alacod/*`, puis les intégrées
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

## 4. Règles pour Claude

1. **Une modification à la fois.** Lire le champ, l'éditer, constater, répondre.
2. **N'écrire que sous `games/<jeu>/assets`.** Jamais de Rust, de `Makefile`, de `tests/`.
3. **Lint vert avant de dire « fait »** : le hook tourne après chaque écriture ; sinon `alacod lint games/<jeu>` à la main. Lint rouge : corriger ou restaurer le fichier.
4. **Un commit `editor: <résumé>` par changement appliqué** (`git add` du seul fichier touché), ex. `editor: vitesse joueur 150 → 135`.
5. **« Annule ça »** : `git log -1 --format=%s` ; seulement si le sujet commence par `editor:`, `git revert --no-edit HEAD`. Sinon, le dire et ne rien faire.
6. **Dire en une phrase ce qui a changé et où** : « `movement.max_speed` du joueur : 150 → 135 (player_config.ron), pris en compte à la prochaine partie. »
   Dire aussi *quand* l'effet se voit (section 5).
7. Ne jamais pousser (`git push`), jamais `git stash`, jamais de commande destructive.

## 5. Ce qui se recharge à chaud

Mesuré sur `make editor GAME=zombies HEADLESS=1` (2026-10-22) : une édition de `weapons.ron` donne au journal
`INFO game::weapon_assets: armes rechargées : WeaponRegistry mis à jour (4 armes)` en moins de 4 s.

| Donnée | Effet | Preuve (plan § 2) |
|---|---|---|
| tables de personnages (`*_config.ron`) → `CharacterConfigs` | **immédiat** pour les spawns futurs ; les entités vivantes gardent leurs `Stats` (copiées au spawn : « plus rapide » dans le RON ne change pas le joueur vivant → `alacod/set_modifier`, K5-1) | `character/config_asset.rs:44` |
| armes (`weapons.ron`) → `WeaponRegistry` et armes portées | **immédiat** (armes portées : `weapons_config_update_system`, `Update`) ; la mêlée n'est pas couverte (risque 7) | `weapon_assets.rs:55-56`, `:179-199` |
| `camera.ron`, `ui/feedback.ron`, `ui/hud.ron` | **immédiat** (présentation) | `camera/mod.rs:666-681`, `feedback.rs:71-86`, `ui/hud.rs:1750-1763` |
| `Registry` (items, salles, curses, patterns, statuts, progression, ennemis) | K5-0 : **relance du binaire** (rechargé seulement au lobby) ; K5-1 : rechargé hors lobby, appliqué **au `restart`** | `content_hot_reload.rs:34-40`, `core_sim.rs:249-262` |
| balance, power-ups, vagues (`ResolvedBalance`, `PowerUpsTable`) | **au `restart`** (résolus à `OnEnter(GameLoading)`) ; K5-0 : relance | `core.rs:166-176`, `balance.rs:5-26` |
| RON **invalide** | aucun rechargement : erreur `Failed to load asset … Could not parse RON` au journal, la table en mémoire reste l'ancienne ; la correction suivante recharge | mesuré 2026-10-22 (écriture d'un RON invalide pendant que le jeu tourne) |

### Assets : images, sons, LDtk, polices

À compléter par l'audit en cours.

## 6. Voix

- **Entrée** : la dictée du système. macOS : dictée (touche micro ou `Fn Fn`) dans le terminal de Claude Code. Rien à installer.
- **Sortie** : hook `Stop` de Claude Code, **opt-in** (`ALACOD_VOICE=1`) : lit la dernière réponse avec `say` (macOS) ou `espeak-ng` (Linux).
  Le script est `scripts/claude-say.sh` (écrit à part, branche `k5-0-voix`) ; sans `ALACOD_VOICE=1` il ne fait rien.
- **Client vocal complet** (Hermes ou autre) : décision en K5-4. Il lui suffit d'un shell et de HTTP local ; pas de pont MCP (plan D5).

## 7. Limites

- **Local seulement** : session synctest locale, un joueur. Pas de p2p (une édition à chaud divergerait entre pairs, sans canal pour la répliquer), pas de web (ni file watcher ni remote en wasm).
- Les commandes de partie du lot K5-1 sont refusées si la session est en ligne ou si `check_distance` ≠ 0 (le mode éditeur le force à 0 ; erreur JSON-RPC qui nomme la cause).
- Aucune vérification de rollback en mode éditeur : un réglage qui casserait le déterminisme ne se voit qu'aux scénarios (`make test_scenarios`).
