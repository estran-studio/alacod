# Rapport sb-3-mode-bac-a-sable (b6) : `--sandbox [profil]`

Fiche : `docs/taches/plan-bac-a-sable.md` (§1 critères 1, 3, 4, 5 ; §3 D2, D3, D5, D6 ; §4 SB-3 ; §5). Branche `sb-3-mode-bac-a-sable` depuis `origin/main` `6aa63588` (SB-1 et SB-2 mergés). VPS 8 cœurs partagé, pas d'affichage.
Doc : `docs/conventions.md` §87, `docs/jouer.md` (« Bac à sable »), `docs/editeur-vocal.md` §2.2. Aucune trace ne bouge, aucun `BLESS`.

## Livré

| Brique | Où |
|---|---|
| `--sandbox [profil]` (profil facultatif, `arene` par défaut ; n'avale jamais l'option suivante) | `game/src/args/cli.rs` → `GameArgs::sandbox` → ressource `game::sandbox::SandboxRequest` posée par `GameArgsPlugin` |
| Mode éditeur implicite (remote, `check_distance` 0) | `CoreSetupPlugin` (`core.rs`) lit `SandboxRequest` ; `play_check_distance_from_env(sandbox)` |
| Plugin du bac à sable, **posé par `LdtkLocalGamePlugin` quand `SandboxRequest` existe** : aucune ligne dans un `main.rs` | `map_ldtk/src/game/sandbox.rs` (`SandboxPlugin`) |
| Validation au lancement : jeu sans kind `Sandbox`, profil inconnu (profils listés), **arène absente**, session en ligne → message + code 1 | `game::sandbox::resolve`, `launch_sandbox` |
| Arène (D2), `RunMode::Sandbox`, graine fixe `ARENA_SEED` (123456, `--seed` la remplace), vagues coupées (`WaveModeEnabled`, `ResolvedBalance::waves_enabled` après `resolve_balance_system`), `HubSession` retirée | `launch_sandbox`, `disable_waves` |
| **`restart` rejoue le profil** : `Profile::install` à chaque `OnEnter(GameLoading)` ; `ProfileApplied` remplace le `Local<bool>` | `profile.rs`, `sandbox.rs` |
| Remote : `alacod/sandbox_profile {id?}` (liste, ou charge = arène + `SandboxSession` + `Restart`), `alacod/give {handle?, weapon?, item?, currency?, set_currency?}`, `alacod/state.sandbox` ; tous gardés par `hot_edit_allowed` (la liste est en lecture) | `game/src/remote.rs` |
| `scripts/alacod-remote` : sous-commandes `profile [ID]` et `give` ; `brief` affiche `sandbox=<profil>` | `scripts/alacod-remote` |
| Pose partagée **et** mode d'ajout : `grant_item`, `equip_weapon` (appelées par `apply_player_profile` et par `give`) ; `give` (arme à distance, mêlée, objet, argent) | `game/src/grant.rs` |
| D126 : `character` dans `SandboxPlayer`, `PlayerScript`, `PlayerProfile`, posé **à la création** (`spawn_players_when_map_loaded` lit `PlayerOverrides`, `create_player(character, …)`, santé de ce personnage) ; lint : personnage connu et skins `1`/`2` (D78) | `content/src/{sandbox,lint}.rs`, `replay.rs`, `create.rs`, `local.rs` |
| `PlayerProfile` se construit **directement** de `SandboxPlayer` (`From`), `from_script` déstructure `PlayerScript` : une troisième structure qui diverge ne compile plus | `profile.rs` |
| `shop.shells_drop_override` d'un profil appliqué en bac à sable (`ShopOverrides`) | `install_session_profile` |
| `apply_player_overrides` parcourt les joueurs **par handle** (les armes consomment `GgrsNetIdFactory`) | `profile.rs` |
| `sandbox_profiles.rs` compte les `ScriptedPlacement` (index par index), plus de `>=` | `scenario/tests/sandbox_profiles.rs` |
| Retrait de `character_tester` : exemple, `[[example]]`, cibles `character_tester_matchbox` et `build_character_tester_web`, entrée `applications.json`, `.gitignore` du site ; `make character_tester` = `make sandbox GAME=zombies` ; `make sandbox GAME= PROFIL= [SEED=] [HEADLESS=1]` | `Makefile`, `Cargo.toml`, `website/` |

Pas de système dans `GgrsSchedule` : `ProfilePlugin` garde ses cinq systèmes dans le même ordre (`schedule_order_diff` vide) ; le bac à sable n'ajoute que `Startup`, `OnEnter`, `Update`. `PlayerProfile`/`Profile` restent hors rollback.

## Ce que fait `restart` en bac à sable

`RunRequest::Restart` → `GameLoading` : `FrameCount` à 0, la partie (joueurs, ennemis, objets, `GameCommands`) disparaît ; le joueur est recréé avec le `character` du profil, le profil actif est relu **dans le registre** (donc `reload` puis `restart` prend un RON modifié)
et reposé (réglages du joueur, placements `at_frame`, `ShopOverrides`), les vagues restent coupées. Perdu : ce que `give`, `modifier` et `spawn` ont posé. `profile <id>` = `SandboxSession` changée + arène du profil dans `MapGenerationConfig` + `Restart`.
`restart --kind ToLobby` : non essayé (D132).

## Essais headless, cinq jeux (sorties réelles)

Binaires `--profile headless --no-default-features --features native`, `ALACOD_HEADLESS=1 ALACOD_REMOTE_PORT=<port> ALACOD_EXIT_AT_FRAME=100000`, PID propre tué en fin. Ligne lancée : `<jeu> --sandbox --local-port 0 --players localhost`, puis `scripts/alacod-remote` :
`step 300`, `profile`, `give --weapon W --currency 25`, `give --item I`, `profile lourd`, `spawn E --ahead 100`, `modifier MoveSpeed Mul 0.5` / `unmodify`, `restart`, puis trois erreurs (`give --weapon nope`, `profile nope`, `give --item nope`). Journal du jeu : 0 `ERROR`/`panicked` dans les cinq.

| Jeu | Première ligne de `brief` (frame ≈ 600, après `step 300`) | `give` | `profile lourd` (frame 20) | `restart` (frame 5) |
|---|---|---|---|---|
| zombies | `editor=True online=False check_distance=0 … carte=exemples/test_map.ldtk graine=123456 sandbox=arene`, `joueur 0 (424.0,696.0) … arme machine_gun`, `ennemis 0`, `vague NotStarted` | `shotgun` déjà tenu : réserve complétée, active (`weapon_already_held: true`), solde 10 025 | `machine_gun 30x240`, 3 zombies posés (`zombie_1/2/full`) à droite | `machine_gun`, `zombie_1` reposé |
| gungeon | `carte=maps/gabarit_armes.ldtk … sandbox=arene`, `pistolet`, `cible` posée | `fusil_pompe` ajouté, `pistolet` retiré (emplacements pleins : `weapon_replaced`), `bottes` passif | `mitrailleuse_lourde 100x999900`, `bullet_kin`, `bandana_kin` | `mitrailleuse_lourde` |
| isaac | `carte=maps/gabarit_armes.ldtk`, `pv 6.0/6.0 arme larmes`, `cible` | `larmes_courtes` (remplace `larmes`), solde 525, `loupe` passif | `larmes`, `mouche`, `araignee` | idem |
| throne | `carte=gabarit_armes.ldtk`, `mitraillette 30x912`, `mannequin` | `canon_ricochet` (remplace `mitraillette`), solde 525 | `roquette 3x12`, `rat` | `roquette` |
| testbed | `carte=testbed/arena.ldtk`, `machine_gun 30x240` | `machine_gun` déjà tenu, solde 1 025, `bottes` passif | `grunt`, `archer`, `cible` posés | `grunt` reposé |

Joueur 0 en (424.0, 696.0) dans les cinq arènes (zombies : seulement grâce à `ARENA_SEED` ; la première version, avec la graine tirée par le jeu, le posait en (−808, −632) ou (168, 824) et les placements du profil auraient été hors arène).
`spawn E --ahead 100` : l'ennemi apparaît à 92–99 unités devant le joueur à la frame suivante. Témoin des vagues : zombies **sans** `--sandbox` (`--seed 123456`), frame 600 : `vague {'number': 1, 'phase': 'Spawning'}`, un `zombie_full` attaque le joueur ; avec `--sandbox`, frame 585 : `vague {'number': 0, 'phase': 'NotStarted'}`, `ennemis 0`.
Refus au lancement (code 1) : `--sandbox nope` (« profil « nope » inconnu du jeu « zombies » (profils : arene, lourd) »), `--sandbox --matchbox …` (« session locale seulement »), un profil dont `character: "zombie_full"` n'a pas le skin `2` (« players[0].character = « zombie_full » : skin « 2 » absent … » via le lint au lancement).

## Tests ajoutés

- `crates/scenario/tests/sandbox_mode.rs` (7) : l'assemblage d'un `main.rs` avec `sandbox: Some(..)` (`GameArgsPlugin`), sans code de bac à sable. zombies `lourd` 720 frames : arène du manifeste, `Sandbox`, `waves_enabled` faux, aucun ennemi de vague, joueur en (424, 696), profil posé (solde 50 000, `machine_gun`, 3 `ScriptedPlacement`) ;
  `give` ajoute une arme (`["machine_gun", "shotgun"]`, active) ; **`restart` rejoue le profil** (solde et arme d'avant le don, 3 zombies, frame < 700) ; changer `SandboxSession` puis `restart` pose `arene` ; les cinq jeux s'ouvrent sur `arene` ; `le_profil_choisit_le_personnage_du_joueur` (D126 : `zombie_full` choisi,
  santé max différente de celle de `player`, sans champ : `player`).
- `crates/content/tests/sandbox_character.rs` + fixture `sandbox_character` (personnage inconnu, skin `2` absent, profil valide).
- Unitaires : `game::grant` (3 : argent ajouté/retiré/imposé, objet comme un ramassage et qui s'ajoute, don refusé sans effet), `game::sandbox` (4 : jeu sans profil, profil inconnu listé, arène absente, arène du profil > manifeste, graine = `Scenario::map_seed`), `remote` (gardes des 6 méthodes d'édition, chargement de profil, don vide), `args::cli` (`--sandbox` à profil facultatif),
  `replay::sandbox_subset` (13 clés).
- Corrigé : `sandbox_profiles.rs` (voir plus haut).

## Vérifications

Machine partagée (charge 4 à 8), tout joué au premier plan ou en tâche attendue, **sans BLESS**, `git status` propre hors code et docs après chaque étape. Les cinq premières lignes sont sur `f91a0d63` (code du lot) ; après la fusion de `origin/main` `b224c896`
(suites de SB-1 par b2 : lint, fixtures, docs ; aucun fichier de simulation) seules les lignes marquées « après fusion » ont été rejouées.

| Vérification | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **809 scénarios**, 3 tranches `ok`, 2 549 s ; **aucune trace bougée**, arbre propre |
| `make schedule_order_diff BASE=origin/main` | **identique** : 4 550 lignes de schedules, 645 types rollback, 5 jeux (base = `origin/main` `b224c896`, tête = le lot ; les deux sorties comparées par `diff -r`). Le script a d'abord échoué sur la tête (voir « Piège de méthode ») : la tête a été rejouée seule après `touch` des fichiers modifiés, la base n'avait pas échoué |
| `test_crates` par paquet (`-p`, mêmes options que la cible : `--profile headless --no-default-features --no-fail-fast -- --exact --skip scenarios`) | animation 2, sim_core 46, bevy_fixed 19, utils 4, behaviors 17, effects 38, bots 99, combat 119, items 52, run 38, stats 0, world 41, **game 298**, **content 370**, meta 18, map 26, telemetry 0, **scenario 160**, gungeon/isaac/testbed/throne/zombies/alacod 0, `map_ldtk` (`--features runtime`) 33 : **0 échec** |
| après fusion : content 370, game 298, scenario 160 | **0 échec** |
| `make lint` ×5 (avant et après fusion) | zombies, testbed, throne, gungeon, isaac : « aucune erreur » ; avertissements préexistants : gungeon 1 (`loot/boss.ron`), testbed 115 (contenu de laboratoire non tiré), les trois autres 0 |
| `make gen` ×5 (avant et après fusion) | rc 0 ×5, tous `ok`, **aucun fichier modifié** (`character: None` ne se sérialise pas) |
| `cargo fmt --all -- --check` (avant et après fusion) | ok |
| `scripts/check-forbidden.sh` (avant et après fusion) | OK, 51 occurrences auditées |
| `make check_sim_boundary` | OK, 13 crates SIM, 2 en dette D69 |
| `make test_tools` | 38 tests OK |
| après fusion : `make test_alacod_cli` | 1 test (ignoré par défaut, 426 s) : ok |
| Essais headless `--sandbox` ×5 jeux | voir plus haut : 0 `ERROR`/`panicked` |

## Correctifs de la revue (BLOQUANT ×2, confiance 0,99)

1. **Le mode `Sandbox` ne survivait pas à `reload`.** `reload_registry_now` et le rechargement automatique remplacent `GameManifest` par le disque ; `entry.mode = Sandbox` n'était posé qu'au `Startup`. `isaac --sandbox` → `reload` → `restart` rétablissait `Floors` :
   `compute_floor_plan` construisait un `FloorPlan`, `setup_generated_map` chargeait les niveaux de la run. Correctif (`map_ldtk/src/game/sandbox.rs`) : `pin_mode` (`Update`, `run_if(resource_changed::<GameManifest>)`) et `pin_session` (`OnEnter(GameLoading)`,
   **`.before(compute_floor_plan)`**) rétablissent le mode à chaque relecture et à chaque (re)chargement de partie. L'écriture est gardée par un test de valeur : `force_sandbox_mode` lit par `Deref` (un `&mut *manifest` marquerait la ressource modifiée et `pin_mode`
   se redéclencherait chaque frame ; relevé par le test `pin_mode_retablit_sandbox_sans_marquer_une_ressource_deja_juste`).
2. **Un `map` modifié dans le profil actif était ignoré au `restart`.** `Restart` va droit en `GameLoading`, `apply_arena` ne tourne qu'à l'entrée de `LobbyLocal`. `pin_session` pose maintenant la carte du profil actif (relu du registre, `resolve(..).arena`) dans `LdtkGameMap` et
   `MapGenerationConfig` à chaque `GameLoading`, avant `setup_generated_map`, **sans toucher aux graines** (`restart --seed`).

**Autres lecteurs vérifiés après `reload` en bac à sable** (`grep GameManifest` dans `game`, `map_ldtk`, `map`, `run`) : `compute_floor_plan` (`GameLoading`, désormais ordonné après `pin_session`) ; `jjrs::local::system_after_map_loaded_local` (mode de run, `GameStarting`, donc après) ; `jjrs::p2p::start_online_run`
(en ligne : refusé au lancement) ; `recording::start_native_recording` (lit le mode épinglé) ; `resolve_progression_system` (`entry.progression`), `resolve_clocks_system` (`entry.clocks`, `entry.difficulty`) : lus au disque, **voulu** (réglages de jeu, éditables à chaud, identiques hors bac à sable,
et inchangés par un `restart` sans `reload`) ; `entry.hub` (lu par le `main` seulement ; `HubSession` retirée au lancement, jamais reposée) ; `entry.map`, `start_map` (copiés dans `LdtkGameMap` par le `main`, jamais relus) ; vagues (`ResolvedBalance::waves_enabled` recalculé à chaque `GameLoading`
puis remis à faux par `disable_waves`, ordonné après `resolve_balance_system` ; `WaveModeEnabled` jamais relue du disque). Aucun autre lecteur du mode ne retombe sur le disque.

**Tests ajoutés** : `scenario/tests/sandbox_mode.rs` (maintenant 9) — `reload_puis_restart_gardent_le_mode_bac_a_sable` (copie d'isaac **avec `GameRoot`**, profil édité sur le disque, `reload_registry_now` réel qui remet `Floors`, puis `restart` : mode `Sandbox`, pas de `FloorPlan`, arène, profil relu : solde 777, 1 placement, joueur
en (424, 696)) et `reload_puis_restart_prennent_l_arene_modifiee_du_profil` (deux arènes distinctes : la seconde a ses `PlayerSpawn` décalés de 160 ; `map` ajouté au profil, `reload`, `restart` : `map_path` nouveau, graine inchangée, `FloorPlan` absent, joueur 160 plus loin, placement posé) ;
`map_ldtk::game::sandbox::tests` (3) — `pin_session` (mode, carte du profil ou du manifeste dans les deux ressources, graines 11 et 7 intactes ; sans session ou profil introuvable, la carte ne bouge pas) et `pin_mode`. **Échec avant correctif prouvé** : `pin_session` et `pin_mode` désactivés, les deux tests d'intégration échouent
(`Some(Floors)` au lieu de `Some(Sandbox)` ; `maps/gabarit_armes.ldtk` au lieu de `maps/arene_decalee.ldtk`).

**Essai headless `isaac --sandbox`** (port 15741, PID propre) : `brief` initial `carte=maps/gabarit_armes.ldtk graine=123456 sandbox=arene` ; `reload` → `{"reloaded": true, "errors": []}` ; `restart` → `carte=maps/gabarit_armes.ldtk … sandbox=arene`, `joueur 0 (424.0,696.0)`, `cible` posée (frame 30) ;
`profile lourd` après reload → `carte=maps/gabarit_armes.ldtk … sandbox=lourd`, `mouche` posée ; `reload` + `restart --seed 99` → `graine=99`, même carte. 0 `ERROR`/`panicked`.

**Vérifications après correctifs** (sans BLESS, arbre propre) : `make test_scenarios_par SHARDS=3` **809 scénarios**, 3 tranches `ok`, 2 305 s, aucune trace bougée ; `make schedule_order_diff BASE=origin/main` **identique** (4 550 lignes, 645 types ; le script a de nouveau échoué sur le piège du target partagé,
tête rejouée seule après `touch` des fichiers du diff et comparée par `diff -r` à la sortie de la base `b224c896`) ; `test_crates` (`-p`, options de la cible) : game 298, content 370, scenario 162, `map_ldtk --features runtime` 36 : 0 échec ; fmt, `check-forbidden` (51 occurrences), `check_sim_boundary` (13 crates) OK.
`pin_session`/`pin_mode` ne sont ajoutés que par `SandboxPlugin` : les applications de scénario n'ont pas ce plugin.

## Limites et dettes

- **Rendu non vu** (D94, D132) : pas de fenêtre ici ; les commandes que William jouera sont dans le Summary et `docs/jouer.md`. `restart --kind ToLobby` non essayé (D132).
- **D130** : `give` (comme `set_modifier`) n'est pas enregistré par `save` : un scénario enregistré ne le rejoue pas.
- **D131** : le personnage par défaut reste `player` en dur (`DEFAULT_PLAYER_CHARACTER`, lint D78, `legibility.rs`).
- **D126 fermée.** Aucun jeu n'a de second personnage jouable (skins `1`/`2`) : la preuve de la sélection est un scénario sur un personnage de PNJ (hors lint) ; le lint refuse un tel profil au lancement.
- `give` pose l'inventaire et les modificateurs de l'objet (la même fonction que la pose d'un profil), pas un ramassage : aucun événement `ItemPicked`, donc pas de bandeau d'objet ; un consommable à effets `OnUse` est compté, pas appliqué.
- Piège de méthode : le `schedule_order_diff` partage le dossier `target` entre l'arbre de base et l'arbre de travail ; les fichiers de la base, plus récents, faisaient passer des crates modifiés pour « à jour » (échec de compilation absurde). Remède : `touch` des fichiers modifiés avant de relancer.
