# dette-d69-e5a-ecrans : écrans, HUD, mutation (D69 étape 2, E5a)

Branche `dette-d69-e5a-ecrans`, base `8ca55686` (`origin/main` après D117 ; E3c et E6 fusionnées pendant la livraison : voir § 6). Fiche
`docs/taches/dette-d69-etape2.md` § E5a, annexe `annexe-presentation.md` §1.1. Refactoring pur : aucune trace ne bouge, aucun `BLESS`.

## 1. Coupes faites (en place, aucun fichier déplacé)

| fichier | avant | après |
|---|---|---|
| `ui/screens/mod.rs` | `read_nav_keys` (clavier `ButtonInput<KeyCode>`, manette `Gamepad`) dans `ScreensPlugin`, `(read_nav_keys, update_screens).chain()` | modèle seul : `Screens`, `UiActionQueue` (`pub struct UiActionQueue(pub Vec<UiAction>)`), `InventorySource`, `MerchantSource`, `update_screens`, `ScreensPlugin` sans clavier (`update_screens.run_if(in_state(InGame))`) |
| `ui/screens/input.rs` (**nouveau, app**) | — | `read_nav_keys` (corps identique octet pour octet à l'original, vérifié par `diff` par le relecteur) + `ScreensInputPlugin` : `read_nav_keys.before(update_screens).run_if(in_state(AppState::InGame))` |
| `ui/mutation_screen.rs` | `update_mutation_screen_view` lisait `ActionState<PlayerAction>` (`ChoiceA/B/C` tenus, `ChoicePrev/Next` pressés) | modèle seul ; lit la ressource **`MutationNavInput { handle, held: u32, step: i32 }`** (`Resource, Default`, initialisée par `MutationScreenModelPlugin`), **seulement si `nav.handle` est le joueur affiché** ; garde `scripted.input_at` (`ScriptedInputs`) |
| `ui/mutation_input.rs` (**nouveau, app**) | — | `read_mutation_nav` (même règle de choix du joueur affiché : `CameraFollowOverride`, sinon joueur local de plus petit handle ; même boucle sur `ActionState`) + `MutationInputPlugin` : `.before(update_mutation_screen_view)`, `run_if(InGame)` ; réécrit la ressource à chaque exécution |
| `ui/mod.rs` | `GameUiPlugin` = Lobby, Disconnected, mutation, hud_model, screens | `GameUiModelPlugin` (mutation, `HudModelPlugin`, `ScreensPlugin`, même ordre) ; `GameUiPlugin` (Lobby, Disconnected) ; `GameUiInputPlugin` (`MutationInputPlugin`, `ScreensInputPlugin`) |
| `core.rs` | `app.add_plugins(GameUiPlugin)` | trois lignes à la même place : `GameUiPlugin`, `GameUiModelPlugin`, `GameUiInputPlugin` (ordre d'avant : lobby, déconnexion, modèles) + import ; rien d'autre |
| `ui/hud_model.rs` | — | rien à faire : `CameraFollowOverride` (E1) y est défini et `SIM_FPS` vient de `session` (E2) ; le fichier est déjà sans fuite |

`GameUiPlugin` (Lobby, Disconnected) reste **hors de `PresentationPlugin`** : il tourne en headless p2p (`LobbyOnline`). Les entrées clavier restent, comme avant, enregistrées aussi
en headless (aucune touche à y lire). Aucun type rollback créé, retiré ni déplacé ; aucun système de `GgrsSchedule` ; `ScreensInputPlugin`/`MutationInputPlugin` sont `Update` seulement.
Rattachement à `CoreAppPlugin` : E7.

Test ajouté : `mutation_screen::tests::la_vue_suit_les_entrees_du_joueur_affiche_seulement` (deux joueurs, `CameraFollowOverride(1)` ; entrées du joueur 0 ignorées, bouton `ChoiceC` tenu du
joueur 1 → carte 2, `step: -1` → carte 1). Il rate si le garde `nav.handle == handle` disparaît.

## 2. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`8ca55686`) | après |
|---|---|---|
| fichiers du crate | 122 | 124 (`ui/screens/input.rs`, `ui/mutation_input.rs`) |
| fichiers à fuite | 49 | 49 |
| erreurs de résolution | 762 | 762 |
| **après fusion d'`origin/main` `e0061e67`** (E3c, E6, D116) : `origin/main` seul → fusion | 46 fichiers à fuite sur 130, 755 erreurs | 46 sur 132, 755 |

Les fuites ont **changé de fichier** : `ui/screens/mod.rs` (15, `KeyCode`/`Gamepad`/`ButtonInput`) et `ui/mutation_screen.rs` (1, `leafwing_input_manager`) sont libérés, `ui/screens/input.rs` (15) et `ui/mutation_input.rs` (1) les
portent (app par construction). Fichiers sim libérés : `ui/screens/mod.rs`, `ui/mutation_screen.rs` (`ui/hud_model.rs` l'était déjà).

**Cliquet inchangé : 22 fichiers.** La fiche demandait `ui/hud_model.rs`, `ui/screens/mod.rs`, `ui/mutation_screen.rs` ; la sonde les **refuse à l'essai** : « dépend de fichiers hors liste » (la fermeture interne contient encore des fichiers à fuite).

| fichier | dépend encore de |
|---|---|
| `ui/hud_model.rs` | `character/health/mod.rs`, `curses.rs`, `effects_runtime.rs`, `loot.rs`, `profile.rs`, `progression.rs` |
| `ui/screens/mod.rs` | `hub.rs`, `profile.rs`, `run_state.rs` |
| `ui/mutation_screen.rs` | `character/player/input.rs`, `progression.rs` |

Fermeture maximale des fichiers sans fuite (calcul jetable, avant la fusion de E6) : aucun des trois n'y entre ; les ajouts possibles auraient été `args/cli.rs`, `args/web.rs`, `character/enemy/ai/mod.rs`, `curses.rs`, `jjrs/browser.rs`, `lib.rs`, `light/mod.rs`, `logs.rs`,
`performance.rs`, `ui/mod.rs`, `ui/weapon_hud.rs`, `web/mod.rs` (hors fiche, non ajoutés ; E6 a depuis fait entrer `curses.rs` et `character/enemy/ai/mod.rs`). Après la fusion, la sonde refuse toujours les trois (`ui/hud_model.rs` : `character/health/mod.rs`, `effects_runtime.rs`, `loot.rs`, `profile.rs`, `progression.rs` ; `ui/screens/mod.rs` : `hub.rs`, `profile.rs`, `run_state.rs` ; `ui/mutation_screen.rs` : `character/player/input.rs`, `progression.rs`). Même constat qu'à E3b : la fermeture est la limite, à lever par E3c-E4 (`progression.rs`, `profile.rs`), E6 (`run_state.rs`) et E4 (`character/player/input.rs`). Commentaire daté dans
`scripts/d69-sim-files.txt`. `probe-game-sim.py --tests --sim-list` : sortie 0 (le test ajouté compile en ECS seul).

## 3. Preuves

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` (deux fois : première variante, puis code final) | **sortie 0** : 4 520 lignes de schedules, 645 types rollback identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS (deux fois) | 3 tranches `ok`, **802 scénarios**, **aucun `.trace` modifié** (`git status` propre hors code et docs) ; scénarios à `HudText`/`ScreenText` compris (`isaac_hud_items`, `isaac_banner_blind`, `ui_pause_menu`, `ui_inventory`, `hub_buy_unlock`, `devil_deal_isaac`…) |
| `make test_crates` (deux fois) | `exit 0` |
| `make lint` ×5, `make gen GAME=…` ×5 (deux fois) | 0 erreur ; scénarios générés tous `ok ok` ; aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` (`--ignored`) | OK / OK (3 600 frames par graine, 8 explosions par graine avec bombes) |
| `cargo fmt --check`, `check-rollback-registration.sh`, `make check_sim_boundary` | OK (13 crates SIM) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e5a cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` | OK ×3 |
| **p2p-stress zombies** (`p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3571`, `COMPOSE_PROJECT_NAME=b1`), code final | codes 0 0, 0 `DesyncDetected`, **1 947 frames resimulées** (21 rollbacks, profondeur max 11), 2 999 lignes identiques |
| **p2p-stress isaac** (idem) | codes 0 0, 0 `DesyncDetected`, **1 782 frames resimulées** (16 rollbacks), 2 999 lignes identiques |
| **lobby en headless** : `scripts/p2p-lobby.sh zombies` (`TRIALS=2`, `LobbyOnline` → partie → bouton « Lobby » → `LobbyOnline` → partie 2) | 2/2 verts (écarts réels 142 et 237 ms), 0 abandon de session ; traces des parties 1 et 2 identiques |
| captures `play_scenario --capture --every 2` (rendu réel sur le GPU du Mac), binaire de `8ca55686` (worktree jetable) contre le code final, comparées par `cmp` | `ui_pause_menu` **45/45 identiques**, `ui_inventory` **30/30**, `levelup_choice` (écran de mutation, carte 3 surlignée par `ChoiceC` scripté) **150/150**, `isaac_hud_items` **116/120** : voir § 4 |

## 4. `isaac_hud_items` : 4 images sur 120 diffèrent de quelques pixels, cause établie

Images 14, 16, 26 et 122 : des différences de **un ou quelques pixels sur le bord supérieur d'étiquettes de monde** (F6, `legibility`), jamais dans le HUD, les écrans ni la simulation (diff amplifié lu). Déterministe (trois rejeux du code après : mêmes
quatre images ; deux rejeux du code avant : identiques entre eux). Bissection (chaque ligne : build + capture, même scénario) :

| essai (sur le worktree de `8ca55686`) | images différentes |
|---|---|
| `HEAD` construit dans le worktree de travail (contrôle de chemin et de build) | 0 |
| système `Update` vide ajouté | 0 |
| coupe des écrans seule (`screens/input.rs`, sans toucher la mutation) | 0 |
| coupe de la mutation seule | 4 (14, 16, 26, 122) |
| **une ressource inutilisée `Default` ajoutée, rien d'autre** | **4 (14, 16, 26, 122)** |
| variantes d'ordre d'enregistrement (modèles avant ou après le lobby, entrées dans ou hors `GameUiPlugin`, sans les entrées) | 4, les mêmes |

Cause : `MutationNavInput` est une **ressource nouvelle** ; en bevy 0.19 une ressource est une entité, donc les `Entity` créées ensuite sont décalées d'un index, et l'ordre de dessin des étiquettes qui se chevauchent à profondeur égale (tri par
entité) change sur ces quatre images. Ce n'est ni de l'ordre de systèmes ni du déterminisme : la simulation trie par `GgrsNetId`, les traces sont identiques, et toute ressource ajoutée au jeu produit la même différence. [INFERENCE] Les captures d'E3a (autres scénarios) ne l'avaient pas montré :
leurs ressources nouvelles n'ont probablement pas décalé d'étiquettes superposées sur ces scénarios (non vérifié ; E3b n'a pas rejoué de captures). Je ne vois pas de coupe qui satisfasse la fiche (la ressource `MutationNavInput` est demandée) sans cette conséquence ; l'écart est accepté et documenté.

## 5. Vérifications non vertes sur `8ca55686` (hors périmètre, corrigées par la fusion)

Sur la base `8ca55686`, avant fusion : `scripts/check-forbidden.sh` (5 occurrences `Instant::now` sans exception : `jjrs/p2p.rs`, `jjrs/signal_watch.rs`, `ui/lobby.rs`, code de D109) et `make test_tools`
(`DeterminismGuardTests.test_current_repository_has_no_unreviewed_or_stale_exception`, même garde). Aucun de ces fichiers n'est dans mon diff. Après la fusion de `origin/main` (D116 : exceptions `Instant` du lobby) : `check-forbidden.sh` OK (47 occurrences auditées), `make test_tools` OK (§ 6).

## 6. Fusion de `origin/main` pendant la livraison

`origin/main` a avancé de `8ca55686` à `e0061e67` (E3c, E6, D116). Fusion dans la branche, deux conflits : `core.rs` (la ligne `WaveAssetsPlugin` de E3c touche la même zone : les deux côtés gardés, mes trois plugins d'interface avant `WaveAssetsPlugin`,
puis `WaveSystemPlugin`) et `scripts/d69-sim-files.txt` (commentaires : E6 et E3c d'abord, E5a ensuite ; cliquet à 29 fichiers). `docs/conventions.md` fusionné sans conflit. Preuves rejouées sur le résultat fusionné : voir le tableau suivant
(les captures n'ont **pas** été rejouées après la fusion : E3c et E6 ne touchent ni écran, ni HUD, ni étiquette).

| vérification sur le résultat fusionné (`104c4ba4`) | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` (`e0061e67`) | **sortie 0** : 4 520 lignes, 645 types rollback identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, **802 scénarios**, aucun `.trace` modifié |
| `make test_crates`, `make lint` ×5, `make gen GAME=…` ×5 | `exit 0`, 0 erreur, tous `ok ok`, aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` | OK / OK |
| `cargo fmt --check`, `check-forbidden.sh` (47 occurrences auditées), `check-rollback-registration.sh`, `make test_tools` (25 tests), `make check_sim_boundary` | OK |
| `cargo check -p game --features lighting,ui_hui,debug_ui`, build web wasm32, `cargo check --workspace --all-targets` | OK ×3 |
| p2p-stress zombies (3 000 frames) | codes 0 0, 0 `DesyncDetected`, **1 785 frames resimulées**, 2 999 lignes identiques |
| p2p-stress isaac | codes 0 0, 0 `DesyncDetected`, **1 858 frames resimulées**, 2 999 lignes identiques |
| `scripts/p2p-lobby.sh zombies` (`TRIALS=2`) | 2/2 verts (152 et 227 ms), 0 abandon de session |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 29 fichiers au cliquet ; § 2 |

## 7. Décisions et limites

- **Entrées enregistrées en headless comme avant** : `GameUiInputPlugin` n'est pas dans `PresentationPlugin` (lecture sans objet en headless : aucune touche, `ActionState` jamais pressé, `ALACOD_INPUT=neutral` coupe déjà `read_nav_keys`). Choisir `PresentationPlugin` aurait changé ce qui tourne en headless.
- **`read_mutation_nav` duplique la règle de choix du joueur affiché** (4 lignes) plutôt que de la factoriser : la factorisation aurait obligé la simulation à connaître le type de requête de l'application. Le garde `nav.handle == handle` et le test couvrent la divergence.
- `update_screens` reste `pub(crate)` : le rendre `pub` (fiche : seul `UiActionQueue` l'est) ferait fuir `HudPlayer`, `InventorySource` et `MerchantSource` privés (avertissement `private_interfaces`) ; E9 le rendra public avec ses paramètres.
- `UiActionQueue` est `pub` avec champ public : `input.rs` y pousse (E9 : crate différent).

## 8. Relecture indépendante

Agent `reviewer`, lecture seule, sur le diff non commité (sans cargo) : **aucun P1/P2**. Deux P3 corrigés : la doc de `MutationNavInput` (« sans écrivain en headless » : faux, l'écrivain tourne, rien n'est pressé) et la parenthèse doublée du lien de doc retiré
dans l'en-tête de `mutation_screen.rs`. Confirmé : équivalence ligne à ligne de `update_mutation_screen_view` (choix du joueur, `held_buttons`, `step`, script, joueur sans `ActionState`, deux joueurs locaux), corps de `read_nav_keys` identique, aucune autre référence à `UiActionQueue`/`read_nav_keys`,
`just_pressed` lu dans la même frame `Update`, aucune résurgence d'état périmé, aucun type rollback ni système `GgrsSchedule` touché. Les chiffres du § 3 sont les miens (le relecteur n'a pas lancé cargo) ; l'ordre d'enregistrement a été relu avant que le § 4 ne montre qu'il n'est pas en cause.
