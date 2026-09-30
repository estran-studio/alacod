# Dettes ouvertes (petites tâches indépendantes)

Lire d'abord `docs/taches/README.md`. Chaque dette est une tâche d'une demi-journée au plus,
sur sa propre branche `m0-dette-<sujet>`, vérification standard, merge, ligne de journal.
Aucune ne doit changer une trace sauf mention.

| # | Dette | Où | Acceptation |
|---|---|---|---|
| D1 | Fixture de lint pour `entry.mode: Waves` sans dossier `Wave` | T2.8 (`entry_mode_waves_without_waves`) | fait dans T2.8 |
| D2 | Sons référencés absents : `weapons.ron` (deux jeux) cite `sounds/machine-gun-reloading.ogg`, le fichier est `machine-gun-reload.ogg` ; aucun lint des chemins audio | T2.8 (`audio_missing_file`) | fait dans T2.8 |
| D3 | Sprites nommés en code (`crates/game/src/global_asset.rs`) et feuilles mélangées aux configs sous `ZombieShooter/Sprites/**` (plan §5 A5) ; le `rifle` réutilise le sprite du fusil à pompe | `global_asset.rs`, `games/*/assets` | registre de feuilles de sprites par kind `SpriteSheet` déclaré dans `game.ron`, `sprite_config.name` résolu par le registre ; sprites rangés par entité (`characters/`, `weapons/`, `enemies/`) ; traces identiques (preuve : dumps `idle`, `two_players_shooting`) |
| D4 | Prix des armes murales et des perks non affichés à l'écran | T2.12 (`prompt`) | fait dans T2.12 |
| D5 | `HitCount` et `ai.stationary` (testbed) hors checksum | `crates/game/src/character/health` (`HitCount`), config IA | décider : soit enregistrer en `rollback_and_trace_component` (bless justifié avec preuve `--ignore HitCount`), soit documenter pourquoi hors checksum (compteur de test seulement) dans `docs/conventions.md` |
| D6 | `EntityHits` : pas de test unitaire de l'attente | `crates/scenario/tests/expectations.rs` | un test qui compte des coups sur `target` dans l'arène du testbed |
| D7 | Testbed : `weapon_slots: 2` mais trois armes de départ dans certains personnages | `games/testbed/assets/characters/*.ron` | cohérent (2 armes ou `weapon_slots: 3`) ; lint : `starting_weapons.len() <= weapon_slots` avec fixture ; traces testbed re-blessées si le contenu change (preuve) |
| D8 | Bruit de sortie : `println!` de debug dans `crates/map_ldtk/src/game/entity/player_spawn.rs` (« player spawn … ») et dans la génération (`to.rs` : « Adding N doors… », « adding room … ») | `crates/map_ldtk`, `crates/map` | remplacer par `debug!`/`trace!` ; `make test_scenarios` lisible |
| D9 | `spawn_weapon_locations_when_map_loaded` : le message « Map is loaded with N weapon locations » compte les copies de gabarits (voir `games/zombies/README.md`) — le dire dans le message, ou compter par gabarit | `crates/map_ldtk/src/game/local.rs` | message exact |
| D10 | Générateur de carte : le gabarit `spawn: true` est réutilisé comme salle ordinaire (deux salles `Level_0` avec la seed des scénarios) — décision de design à prendre : le conserver (documenté) ou exclure les gabarits de départ des salles suivantes | `crates/map/src/generation/imp/basic.rs` (`get_next_room_recursize`) | décision écrite dans `docs/conventions.md` §1 ; si exclusion : toutes les traces `test_map` changent (preuve sur `idle`, `points_on_kill` : seules les entités de la salle disparue et les ids décalés) |
| D11 | `alacod-gen` : le tableau `--play` n'affiche pas le nombre de coups observés (il faut une sonde pour caler `min_hits`/`max_hits`) | `crates/scenario/src/bin/alacod-gen.rs` | colonne « coups » dans le tableau |
| D12 | Allumette (signaling avec JWT) non supporté par le client natif : la CI de nuit utilise `matchbox_server` nu | `crates/game/src/jjrs/p2p.rs`, `allumette` | flux HTTP `/auth` + `/lobbies` dans le client natif, profil `allumette` de `docker-compose.ci.yaml` activé dans `scripts/nightly.sh` |
| D13 | `RunEnd::Abandon` sans `RunSummary` ; `ToLobby` ramène aussitôt vers une nouvelle partie en local (comportement du lobby local) | `crates/game/src/run_state.rs`, `ui/lobby.rs` | résumé calculé aussi à l'abandon ; le lobby local attend une action |
| D14 | Restart p2p non supporté (redirigé vers le lobby) | `run_state.rs`, `jjrs/p2p.rs` | hors M0 (plan §9.8) ; noter dans M1 |
| D15 | Grille de collision (T2.1) : gain de performance jamais mesuré en ratio | `crates/combat` | mesure une fois avec/sans grille sur `bench_bullets`/`bench_horde`, chiffre dans le journal |
