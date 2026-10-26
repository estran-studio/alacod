# Plan de l'éditeur vocal (chantier K5) — rédigé le 2026-10-22

**État au 2026-10-26 : K5-0 à K5-3 livrés et sur `origin/main` ; l'app de test est désormais le bac à sable (`make sandbox GAME=<jeu> [PROFIL=]`, `docs/taches/plan-bac-a-sable.md`, SB-1 à SB-4 mergés), `character_tester` est retiré (`make character_tester` = alias). L'essai vocal K5-4 est refait dans le bac à sable (SB-5, `docs/taches/rapports/sb-5-essai-vocal.md`) ; décision du pont MCP en attente de William.** Remplace le chantier K5 « Éditeur de tuning (Egui) » de `docs/plan-engine.md:530`.
**Révision du 2026-10-22 (William)** : pas de serveur MCP en K5-0. BRP suffit à un client doté d'un shell ; le pont MCP n'est envisagé qu'en K5-4 si le client vocal l'exige.
Idée de William (2026-10-22) : pas d'éditeur graphique à la Unity ; l'éditeur, c'est une IA à la voix.
Analyse technique source : rapport « rechargement à chaud / commandes de partie / apps de test » du 2026-10-22 (agent `analyse-hot-reload-partie`).

Convention : chaque fait cite sa source `fichier:ligne` ; `[INFERENCE]` = déduit, non observé ; `[hypothèse]` = non lu. Les estimations
(jours-agent) sont toutes des `[INFERENCE]`. Aucun chiffre inventé : « non mesuré » dit ce qui manque.

## 1. Objectif

William parle à Claude (depuis Hermes ou tout client vocal) pendant que le jeu tourne : « un petit peu trop rapide »,
« spawne-moi tel ennemi », « reset », « annule ça ». Claude modifie les RON et pilote la partie en marche ; le rechargement à chaud
montre le résultat tout de suite. Les apps de test (Map Tester, Character Tester, plus tard Music Tester pour Harmonium) démarrent avec
le même mode local synctest à 1 joueur et `check_distance = 0`.

> Boucle : parole → Claude → (Edit RON + hook lint + commit `editor:` | `alacod-remote` → BRP `alacod/*`) → jeu en marche → rechargement à chaud / `alacod/state` → réponse.

**À quoi ça ressemble**

William : « C'est un peu trop rapide. »
Claude : `alacod-remote modifier --handle 0 MoveSpeed Mul 0.9` **[K5-1]**
Claude : réponse — validé ; Edit de `movement.max_speed` 150 → 135 dans `games/zombies/assets/.../player_config.ron` **[K5-0]**
Claude : réponse — hook lint vert, commit `editor: vitesse joueur 150 → 135` **[K5-0]**
William : « Spawne un zombie_runner devant moi. »
Claude : `alacod-remote spawn zombie_runner --at 420,300` **[K5-2]**
Claude : réponse — état relu par `alacod/state` **[K5-1]**
William : « Reset. »
Claude : `alacod-remote restart` **[K5-1]**
William : « Annule ça. »
Claude : `git revert --no-edit HEAD` (après vérification que HEAD est `editor:`) **[K5-0]**

**Fini quand** : William, en jouant à `zombies` lancé par `make editor GAME=zombies`, dit à la voix les phrases du § 1, et voit chacune prendre effet
**sans relancer le binaire ni toucher au clavier** ; chaque écriture est un commit `editor:` relisible ; `make test_scenarios` sans `BLESS` reste vert
(aucune trace ne bouge).

## 2. Ce qui existe (vérifié) / ce qui manque

| Point | État |
|---|---|
| Remote | `RemotePlugin::default()` (`remote.rs:79`) : les 7 méthodes `alacod/state`, `input`, `step`, `pause`, `resume`, `screenshot`, `save_recording` (`crates/game/src/remote.rs:76-87`) et les 23 méthodes intégrées `world.*`/`registry.schema`/`schedule.*` ; **`rpc.discover`** (OpenRPC 1.3.2, `bevy_remote-0.19.1/src/builtin_methods.rs:110-111`, `:1078-1113`) liste les méthodes intégrées et les `alacod/*` (`lib.rs:805-840`, `schemas/open_rpc.rs:111-121`). Transport : HTTP seul, `127.0.0.1:15702` (`http.rs:103-131`, `with_address` possible), serveur sur `IoTaskPool` (`http.rs:234-249`). Actif seulement si `ALACOD_REMOTE=1` (`remote.rs:65-67`) ; démarre en pause (`remote.rs:89-91`) ; entrées par `RemoteInputs` (`remote.rs:266-274`) ; handlers traités dans `RemoteLast`, après les ticks GGRS (`bevy_remote-0.19.1/src/lib.rs:842-849`). Client : `scripts/alacod-remote` (`urllib`). |
| Réflexion BRP | `world.query`/`world.get_components` exigent `Reflect` + `register_type` : 29 `derive(Reflect)` et **2 `register_type`** dans `crates/` (`waves/mod.rs:69-70`) ; `Health` (`combat/src/actors.rs:219-223`), `FixedTransform3D` (`bevy_fixed/src/fixed_math.rs:838-843`), `Velocity`, `Weapon` (`combat/src/weapons/mod.rs:348-353`), `Stats` (`sim_core/src/stats.rs:119-120`) sans `Reflect` ; `Fixed` = alias `FixedI32<U16>` sans `Reflect` (`fixed_math.rs:12-16`). **L'état de jeu passe donc par `alacod/state` (résumé compact typé), pas par la réflexion** ; rendre les types rollback `Reflect` est hors périmètre. |
| Hook de lint | `.claude/settings.json:2-12` : `PostToolUse` sur `Edit|Write` → `scripts/lint-edited-game.sh` (`alacod lint games/<jeu>` par `cargo run -p content --bin alacod --profile headless`, `:37-42`), asynchrone, 300 s. Pas de `.claude/skills/` ni `.claude/commands/`. |
| Rechargement à chaud **effectif en partie** | `CharacterConfigs` : `sync_character_configs`, `PreUpdate.before(RunGgrsSystems)`, sans condition d'état (`character/config_asset.rs:44`). `WeaponRegistry` : `sync_weapon_registry` (`weapon_assets.rs:55`). `Weapon.config` des armes portées : `weapons_config_update_system`, `Update` (`weapon_assets.rs:56`, `:179-199`), pour `WeaponsConfig` seulement. Aussi `camera.ron` (`camera/mod.rs:666-681`), `ui/feedback.ron` (`feedback.rs:71-86`), `ui/hud.ron` (`ui/hud.rs:1750-1763`). Condition : feature `native` = `bevy/file_watcher` (`Cargo.toml:16-19`). |
| Rechargement **absent en partie** | `Registry` : `content_hot_reload.rs:34-40`, `Update.run_if(in_lobby)`. Donc items, salles, curses, patterns, statuts, progression (tables dérivées à `OnEnter(GameLoading)` : `core_sim.rs:249-262`, `map_ldtk/src/loader/runtime.rs:366-381`) jamais rechargés ; `Restart` saute le lobby (`run_state.rs:285-286`). Balance, power-ups, vagues : résolus une fois à `OnEnter(GameLoading)` (`core.rs:166-176`, `balance.rs:5-26`). |
| Session locale | `SyncTestSession` toujours (`jjrs/local.rs:100-105`) ; `check_distance` **0 par défaut** en jeu joué (`args/mod.rs:56-62`) : ni snapshot ni resimulation, donc une modification à chaud y est sûre. Elle casse en synctest d>0 (scénarios : défaut 2, `args/mod.rs:49-51` ; `ALACOD_STATE_TRACE`) et en p2p (desync, aucun canal entre pairs). Un mismatch local ne fait qu'un `warn!` (`bevy_ggrs-0.22.0/src/schedule_systems.rs:85-110`) ; `log_ggrs_events` ignore tout sauf P2P (`jjrs/mod.rs:163-165`). |
| Valeurs copiées au spawn | `Stats` (MoveSpeed, Acceleration, Sprint, MaxHealth…), collider, tags, dash, effets (`character/create.rs:126-233`) ; `EnemyAiConfig`, behaviors (`character/enemy/create.rs:238-263`). La vitesse lit `Stats` d'abord, la config n'est qu'un repli (`character/player/input.rs:397`, `stats/src/reader.rs:44-49`). **« Plus rapide » dans le RON ne change pas le joueur vivant.** |
| Placements à une frame exacte | Mécanisme du runner : ressource **indexée par frame**, jamais dépilée (`scenario/src/runner.rs:395-407`, `:825-885`) ; id par `GgrsNetIdFactory`, rollback (`core_sim.rs:143`). Privé à `crates/scenario`. `spawn_character` est dans `map_ldtk` (`map_ldtk/src/game/local.rs:311-351`), `game` ne dépend pas de `map_ldtk` (`crates/game/Cargo.toml`). |
| Reset | `RunRequest::Restart`, ressource posable depuis un handler (`run_state.rs:202-216`, consommée `:473-560`). |
| Levier valeur vivante | `Modifier` sur `Stats` (`sim_core/src/modifier.rs:11-40`), lu par `StatReader::get` (`stats/src/reader.rs:44-49`). |
| CLI `alacod` | `lint --json`, `describe --json`, `new`, `new-content`, `test --json` (`crates/content/src/bin/alacod.rs:19-82`). Pas de sous-commande `mcp`. |
| Apps de test | `character_tester` passe par `CoreSetupPlugin` (`examples/character_tester.rs:43-50`) : remote seulement si `ALACOD_REMOTE=1` (`core.rs:196-199`), et `make character_tester` ne le pose pas (`Makefile:319-320`). `map_preview` n'a pas de `CoreSetupPlugin` (`examples/map_preview.rs:31-50`), recharge la carte à la touche R (`:80-96`). `map_generation` est une CLI sans `App` (`examples/map_generation.rs:10-62`). |
| Voix / BRP | Aucun client vocal ni voix intégré dans les `.rs`, `.sh`, `.toml`, `Makefile` (inventaire du 2026-10-22). Pas de cible `make editor`. |
| Web | Ni file watcher ni remote en wasm (`core.rs:196`, `#[cfg(not(target_arch = "wasm32"))]`). |

## 3. Décisions

### D1 — Rechargement en partie : session locale à `check_distance = 0` seulement
- **Décision** : le mode éditeur (`make editor`, `ALACOD_EDITOR=1`, qui implique `ALACOD_REMOTE=1`) force `check_distance = 0`. Les commandes `alacod/*` d'édition refusent `OnlineState::Online` (`session.rs:44-49`) et d>0 par une erreur JSON-RPC explicite.
- **Pourquoi** : d=0 est déjà le défaut du jeu joué (`args/mod.rs:56-62`) ; les syncs écrivent dans des ressources non rollback lues par `GgrsSchedule`, ce qui diverge en synctest d>0 et en p2p.
- **On ne fait pas** : de versionnement des données par frame (ressource `ContentEpoch` rollback + historique), ni d'application par `PendingContentReload` à `FrameStart` : les frames N−d..N−1 resimulées liraient déjà la nouvelle donnée. Coût : des dizaines de lectures `Res<CharacterConfigs>` à changer ; bénéfice : seulement en synctest.
- Les syncs existants ne changent pas ; leurs tests les exercent déjà en synctest (`scenario/tests/character_configs.rs:136`, `weapon_registry.rs:235`).

### D2 — Effet d'une édition RON
- Immédiat : ce qui est relu à chaque frame et les spawns futurs (`CharacterConfigs`, armes).
- `Stats` de base des entités vivantes : réappliquées par un système dédié (K5-2), sans toucher `Modifiers`.
- Tout le reste (`Registry`, balance, vagues) : au `restart` suivant. K5-1 recharge donc `Registry` hors lobby.
- « Annule ça » = git (`undo`), pas un historique en mémoire.

### D3 — Commandes de partie : ressource non rollback indexée par frame
- `GameCommands: BTreeMap<u32, Vec<Command>>`, appliquée dans `GgrsSchedule` à `FrameCount + 1`, dans les mêmes sets que le runner (`EnemySpawning`, `Effects` : `runner.rs:356-361`, `:399-406`).
- **Jamais une file dépilée** : à la resimulation d'une frame, un spawn dépilé disparaît et le checksum diffère.
- Copiée dans `RecordedSettings` (`recording.rs:40-53`, `:98-108`) pour que `save_recording` rejoue la session avec ses spawns.
- **Pas par l'input GGRS** (un `u32` plus la visée : `recording.rs:286-289`).

### D4 — « Un peu trop rapide » et « dégâts de l'arme X »
- Vivant : `set_modifier` (Mul ou Pct sur `MoveSpeed`, source `editor:<n>`), réversible. Durable : RON (`movement.max_speed`).
- Arme : RON (`weapons.ron`), déjà réappliqué aux armes portées. `StatId::Damage` est un multiplicateur par joueur, sans notion d'arme : on ne l'utilise pas pour « l'arme X ».

### D5 — Pas de pont MCP : BRP + instructions

BRP est le format d'API du jeu (JSON-RPC/HTTP, auto-décrit par `rpc.discover`) ; MCP n'est qu'une façon pour un client IA de découvrir des outils.
Un client doté d'un shell (Claude Code) n'en a pas besoin : `scripts/alacod-remote` (ou `curl`) pour la partie, `Edit`/`Write` pour les RON,
le hook de lint, `git` pour « annule ça ». Les garde-fous deviennent des **instructions** (`docs/editeur-vocal.md`, § vocabulaire et règles :
un changement à la fois, un commit `editor:` par changement appliqué, écriture limitée à `games/<jeu>/assets`, `undo` = `git revert` du dernier
`editor:`) et des **réglages du client** (`permissions` de Claude Code limitées à `games/<jeu>/assets/**` `[INFERENCE : à vérifier dans la doc des règles
`Edit(...)`]`).

Pont MCP : seulement si le client vocal n'a ni shell ni HTTP, décidé en K5-4 ; candidats `bevy_brp_mcp` 0.23.0 (compatibilité avec bevy 0.19.1
non vérifiée) ou ≈ 150 lignes `rmcp` 3.6.0 (absent du `Cargo.lock`, `tokio` 1.53.1 présent). On ne fait pas : de Python, de serveur en processus
dans le jeu (le pont doit survivre au jeu pour le relancer).

### D6 — Apps de test
- `make editor GAME=<jeu> [MAP=…]` : app de test principale (le jeu lui-même : `native` = file watcher, remote, labels F6, mode éditeur).
- `character_tester` reçoit remote et mode éditeur par défaut (`CoreSetupConfig` ou cible Makefile).
- `map_preview` reste tel quel (aperçu statique, touche R). Map Tester = `make editor` + `restart` avec graine.
- Music Tester (Harmonium) : **hors périmètre** tant qu'Harmonium n'existe pas ; même mode local et même script remote.

### D7 — Voix
On ne code pas le vocal. Le client (Hermes ou autre) héberge Claude et parle au remote BRP par `scripts/alacod-remote` (HTTP local).
Livrables : doc d'installation, liste des méthodes BRP, **prompt de vocabulaire** (`docs/editeur-vocal.md` : « plus rapide » → `movement.max_speed`,
unités, une modification à la fois, dire ce qui a changé, lint toujours), séance d'essai avec William.

## 4. Lots

Branches `k5-<n>-<sujet>`. Estimations `[INFERENCE]`. V1 = simulation, V3 = outillage (`docs/taches.md:8-34`).

| # | Lot | Voie | Dépend de | Est. `[INFERENCE]` | Fiche |
|---|---|---|---|---|---|
| K5-0 | Mode éditeur et instructions | V3 | rien | 0,5 j | livré (`k5-0-1-editeur`) |
| K5-1 | Partie pilotable, lot 1 | V3 avec V1 | K5-0 | 1 j | livré (`k5-1-partie-pilotable`) ; fiche `docs/taches/k5-1-partie-pilotable.md` |
| K5-2 | Partie pilotable, lot 2 | V1 | K5-1 | 3 j | livré (`k5-2-commandes-spawn`) ; fiche `docs/taches/k5-2-commandes-spawn.md` |
| K5-3 | Apps de test | V3 | K5-1 (K5-2 pour les spawns) | 1 j | livré (`k5-3-apps-de-test`) ; fiche `docs/taches/k5-3-apps-de-test.md` |
| K5-4 | Branchement vocal et décision MCP | V3 + William | K5-1 | 1 j (+1 j si pont) | `docs/taches/k5-4-branchement-vocal.md` : guide et protocole écrits ; essai William à faire |

### **K5-0 Mode éditeur et instructions** (V3, 0,5 j)
- `Makefile` cible `editor` (`ALACOD_EDITOR=1` ⇒ `ALACOD_REMOTE=1`, features `native`, `ALACOD_LABELS=1`).
- `scripts/alacod-remote` (`discover` : affiche `rpc.discover`), `docs/editeur-vocal.md` v0 (outils = méthodes BRP + script, vocabulaire → champs RON, règles).
- **Acceptation** : depuis Claude Code, sur `make editor GAME=zombies`, lire l'état, avancer de 60 frames, éditer `movement.max_speed` d'un ennemi, voir le prochain spawn plus rapide (`alacod/state`), commit `editor:`, `git revert` ; aucun fichier Rust, aucune trace.

### K5-1 — Partie pilotable, lot 1
- **Fichiers** : `crates/game/src/editor_mode.rs` (nouveau : `EditorMode`, `hot_edit_allowed`), `args/mod.rs` (d=0 forcé), `remote.rs`, `content_hot_reload.rs`, `scripts/alacod-remote`.
- **Livrable** :
  - `alacod/restart {kind?, seed?}`, `alacod/set_modifier {handle|net_id, stat, op, value, source, until?}`, `alacod/reload_content` ;
  - `alacod/state` enrichi : `online`, `check_distance`, `editor`, `game`, `map`, `seed`, `run.step`, clé de config par ennemi ;
  - `alacod/state` renvoie aussi `editor: false` hors mode éditeur, et l'erreur des commandes refusées nomme la cause (`online`, `check_distance=2`) ;
  - `Registry` rechargé hors lobby, appliqué au `GameLoading` suivant ;
  - sous-commandes `restart`, `modifier`, `reload` de `scripts/alacod-remote`.
- **Acceptation** (scénario dans `crates/scenario/tests/`) :
  - `Registry` rechargé en partie, puis `Restart` voit la nouvelle table ;
  - `set_modifier` change la distance parcourue ;
  - refus en `Online` et en d>0 (test unitaire de la garde) ;
  - **`make test_scenarios` sans BLESS : 0 trace bougée.**

### K5-2 — Partie pilotable, lot 2
- **Fichiers** : `crates/game/src/game_commands.rs` (nouveau), `core_sim.rs` (enregistrement avant les cinq de clôture), `remote.rs`, `recording.rs`, `spawn_character` et `LevelSpawnAssets` déplacés de `map_ldtk::game::local` vers `game` (ou systèmes posés dans `map_ldtk`), système « réappliquer la base de `Stats` » près de `character/config_asset.rs`.
- **Livrable** : `alacod-remote spawn` (`alacod/spawn_character`, `spawn_powerup`, `spawn_item`) ; `Stats` des entités vivantes mises à jour sur `AssetEvent::Modified` de `CharacterConfigAsset`.
- **Acceptation** :
  - une session avec un spawn commandé, sauvée par `save_recording` puis rejouée par le runner → trace identique ;
  - `max_speed` édité → `Stats.MoveSpeed` du joueur vivant mis à jour (test à la frame 30, comme `character_configs.rs:136`) ;
  - **0 trace existante bougée** (systèmes inertes sans commande) ;
  - `make schedule_order_diff` : ordre rollback inchangé hors ajout.

### K5-3 — Apps de test
- **Fichiers** : `CoreSetupConfig` et `core.rs:196-199`, `Makefile` (`character_tester`, `editor`), `examples/character_tester.rs` ; `map_preview` inchangé.
- **Livrable** : remote et mode éditeur sans variable d'environnement.
- **Acceptation** : `make character_tester` et `make editor` répondent sur le port remote sans variable ; `docs/jouer.md` à jour.

### K5-4 — Branchement vocal et décision du pont MCP
- **Fichiers** : `docs/editeur-vocal.md` (nouveau), `scripts/alacod-remote`.
- **Livrable** : `docs/editeur-vocal.md` complété ; **décision du pont MCP** selon le client (shell/HTTP → rien ; sinon `bevy_brp_mcp` ou `rmcp`, +1 j) ; séance d'essai des cinq phrases du § 1, journal.
- **Acceptation** : les cinq phrases jouées par William ; « spawne » seulement après K5-2.

Total ≈ 6,5 j `[INFERENCE]` (+1 j si pont MCP). Ce qui n'est pas mesuré : le coût réel de déplacer `spawn_character` hors de `map_ldtk` (non lu en détail au-delà de `local.rs:311-351`) et celui de l'audit de `sync_health_from_stats` ; K5-2 commence par ces deux audits.

Preuve d'un lot de simulation (K5-2) : chaque système ajouté à `GgrsSchedule` est inerte sans commande ; `make test_scenarios` sans `BLESS`
et `make schedule_order_diff` en font la preuve (`docs/taches.md` §10).

## 5. Quand

Faits : la semaine de polissage n'est **pas lancée** (`plan-polissage.md:3`) ; son total B est ≈ 47 j pour 35 j nominaux
(`plan-polissage.md:187-190`) ; règle 7 de `docs/taches.md:32` (quatre voies actives au plus), déjà dépassée par décision de William
(sept voies) ; ordre de merge outillage → données → simulation → présentation (`docs/taches.md:28`).
Conflits prévisibles avec la semaine : `Makefile` (S1-2, S1-3) et `core_sim.rs` (S1-6, interpolation) (`plan-polissage.md:262-264`). Aucun autre.

| Option | Contenu | Coût |
|---|---|---|
| **A (proposée)** | K5-0 et K5-1 **maintenant**, une voie, ≈ 1,5 j `[INFERENCE]`, pendant que William relit et choisit les packs ; mergés en premier (outillage, 0 trace). Puis K5-4 dès K5-1 mergé (une demi-journée d'agent + une séance de William). K5-2 après le merge de S1-6 (`core_sim.rs`) ou après la semaine, pas pendant. K5-3 avec K5-2 ou après. | La semaine démarre avec l'éditeur : la boucle « je joue, je dis, ça change » sert aux tâches de réglage S1-6 (feel par clone en données), S1-10, S2-3, S2-6, S2-7. Coût : une voie de plus avant le départ de la semaine. |
| B | Tout après la semaine, dans l'ordre K5-0 → K5-1 → K5-4 → K5-2 → K5-3. | La semaine se fait sans éditeur : réglages à l'éditeur de texte, relance et recompilation comme aujourd'hui. Aucun conflit de fichiers. |

`[INFERENCE]` L'option A suppose que K5-0 et K5-1 ne touchent pas aux fichiers de la semaine, sauf `Makefile` (ajout d'une cible, conflit trivial).

## 6. Hors périmètre et risques

**Hors périmètre** : p2p et web (ni file watcher ni remote en wasm) ; rollback vérifié en mode éditeur (à d=0, un bug de déterminisme introduit par un réglage ne se voit qu'aux scénarios) ; éditeur Egui (abandonné) ; Music Tester ; port Rust du pont ; synthèse et reconnaissance vocale.

| # | Risque | Parade |
|---|---|---|
| 1 | Lecture d'un RON à moitié écrit par le file watcher | Écriture atomique (fichier temporaire + rename) dans `write_ron`. |
| 2 | `AssetEvent::Modified` sur un RON invalide `[hypothèse]` : pas d'événement si le chargeur échoue | À mesurer en K5-0 (écrire un RON invalide, relire l'état). Le lint suit l'écriture (D5) : un RON invalide est restauré aussitôt, le file watcher peut voir deux modifications ; mesurer si la première produit un `Modified`. |
| 3 | `set_modifier` écrit un composant rollback hors `GgrsSchedule` | Licite à d=0 seulement ; garde `hot_edit_allowed` (D1). |
| 4 | Client vocal sans shell | Pont MCP, +1 j, décidé en K5-4. |
| 5 | `Registry` rechargé hors lobby : `GameLoading` relit `Registry` et les `Assets` ; une copie oubliée resterait ancienne | Vérifier `PatternLibrary`, `StatusLibrary`, tables de `setup_generated_map`, `CurseTable` (`core_sim.rs:249-262`, `runtime.rs:366-381`, `curses.rs:47-70`) ; test « Restart voit la nouvelle table » (K5-1). |
| 6 | Réappliquer `Stats` écrase des modificateurs ? | Non attendu : `Modifiers` est séparé (`stats/src/reader.rs:44-49`) ; à prouver par test (K5-2). |
| 7 | `weapons_config_update_system` ne couvre que `WeaponsConfig`, pas `MeleeWeaponsConfig` `[hypothèse]` (`weapon_assets.rs:179-199`) | Mesurer en K5-1 ; si confirmé, dette ou extension en K5-2. |
| 8 | Un mismatch synctest fige la simulation `[hypothèse]` (`ggrs-0.13.0/src/sessions/sync_test_session.rs`, `advance_frame` retourne `Err` avant d'avancer) | Le mode éditeur force d=0 ; le message d'erreur des commandes dit pourquoi. |
| 9 | Un RON valide au lint mais absurde (vitesse 0, dégâts ×1000) | Le lint ne juge pas l'équilibre ; `undo` et `git history` sont la réponse. |

## 7. Règles du chantier

- Celles de `CLAUDE.md` (déterminisme, frontière simulation/présentation, `FrameEvents`, jamais de `Message` bevy dans la simulation).
- Tout système ajouté à `GgrsSchedule` est inerte sans commande et prouvé par `make test_scenarios` sans `BLESS`.
- Logs : `ggrs{f=… editor_command …}` avec `GgrsNetId`, jamais `Entity`.
- Pas de pont MCP tant qu'un client ne l'exige ; `scripts/alacod-remote` reste sans dépendance (`urllib`).
- Nouveau crate : à classer dans `scripts/check-sim-boundary.sh` (aucun n'est prévu ; `editor_mode.rs` et `game_commands.rs` vont dans `game`).
- Dettes : D126 à D133 ont été prises par le bac à sable (`plan-bac-a-sable.md`) ; les réservations K5 d'origine (D131, D132) ne valent plus : un risque K5 confirmé prend le prochain numéro libre de `dettes.md` (réserve du polissage : D134–D138).
- Chaque lot finit par `LIVRÉ <branche>` dans le journal (`docs/taches.md` §10) et un rapport dans `docs/taches/rapports/`.
- Documentation à jour au merge de K5-0 : `docs/conventions.md` (section éditeur vocal) et `CLAUDE.md` (variables `ALACOD_EDITOR`, outils BRP). `docs/taches.md` §8.2 et `docs/plan-engine.md` K5 renvoient déjà à ce plan.
