# dette-d69-e5b-feedback : journal de feedback en simulation (D69 étape 2, E5b)

Branche `dette-d69-e5b-feedback` (worker b2), base `e0061e67` (E1, E2, E3a, E3b, E3c, E6), fusionnée avec `origin/main` `a1edf161` (E3d, E4, E5a, D84) avant la
livraison : **toutes les vérifications ci-dessous sont celles de l'arbre fusionné**. Fiche `docs/taches/dette-d69-etape2.md` § E5b ; annexe
`docs/taches/rapports/dette-d69-plan-etape2/annexe-presentation.md` § 1.3. Refactoring pur : **aucune trace ne bouge, aucun `BLESS`**.

## 1. Ce qui change

`feedback.rs` (2 348 lignes, mixte) est coupé en deux, sans déplacer de type rollback (le journal n'en contient aucun).

| | simulation : `crates/game/src/feedback_log.rs` (nouveau, 991 lignes) | application : `crates/game/src/feedback.rs` (1 399 lignes) |
|---|---|---|
| types | `FeedbackKind`, `FeedbackCue`, `FeedbackLog` (+ `FEEDBACK_HISTORY_LEN`), `PatternKind`, `PatternShape`, `TelegraphShape`, `FeedbackConfigLoaded`, `pub use content::feedback::{…}` | `FeedbackConfig` (enveloppe `Asset`, `#[serde(transparent)]`), `FeedbackFx`, composants de rendu |
| fonctions | `fan_rays`, `pattern_shape`, `pattern_label`, `telegraph_shape`, `ranged_range`, `weapon_of_hit`, `blank_ring_radius`, `flash_alpha`, `boss_zoom_weight`/`boss_zoom_factor`, `record_feedback_cues`, `record_t19_cues` | rendu, gizmos, sons (`bevy_kira_audio`), `apply_feedback_cues`, `tint_champions`, `draw_telegraphs`… |
| plugins | `FeedbackLogPlugin` : `init_resource::<FeedbackLog>()` et les deux systèmes `PostUpdate` (`record_feedback_cues`, `feedback_v3::record_v3_cues.after(…)`), **sans `RonAssetPlugin`** | `FeedbackAssetsPlugin` (nouveau : `RonAssetPlugin::<FeedbackConfig>`, `load_feedback_config` en `Startup`, `publish_loaded_feedback_config` en `Update`), `FeedbackPlugin` (rendu, inchangé) |
| tests | `telegraph_tests` (formes, anneau du blank, zoom, télégraphes : 9 tests purs, déplacés tels quels) | `shot_sound_tests` |

Les systèmes sont les mêmes, avec les mêmes ordres relatifs (`record_v3_cues.after(record_feedback_cues)` ; `apply_feedback_cues.after(record_v3_cues)` côté rendu).
`FeedbackConfigLoaded` (ressource, absente tant que l'asset n'est pas chargé) passe dans le journal : la simulation la lit en `Option<Res<…>>` comme avant ; c'est
`feedback::publish_loaded_feedback_config` (application) qui l'écrit, au même moment qu'avant (`Update`, premier tour après le chargement, puis sur `AssetEvent::Modified`).

### `core.rs` (diff minimal)

```
-        app.add_plugins(crate::feedback::FeedbackLogPlugin);
+        app.add_plugins(crate::feedback::FeedbackAssetsPlugin);
+        app.add_plugins(crate::feedback_log::FeedbackLogPlugin);
```

Les deux plugins sont à l'ancienne place de `FeedbackLogPlugin`, dans cet ordre : le chargeur RON, `Startup` et `Update` sont enregistrés avant le journal, comme dans l'ancien
`build`. `FeedbackPlugin` (rendu) n'a pas bougé.

### Lecteurs migrés (sans `pub use` de compatibilité)

| lecteur | avant | après |
|---|---|---|
| `scenario/src/events.rs` (`feedback_events`, trois sites) | `game::feedback::{FeedbackLog, FeedbackKind, pattern_label}` | `game::feedback_log::{…}` ; `game::feedback_v3::cue_label` inchangé |
| `game/src/items.rs` (`draw_blank_wave_system`, application) | `crate::feedback::FeedbackConfigLoaded` | `crate::feedback_log::FeedbackConfigLoaded` |
| `game/src/feedback_v3.rs` | `crate::feedback::{FeedbackCue, FeedbackKind, FeedbackLog}` | `crate::feedback_log::{…}` |
| `game/src/feedback.rs` (rendu) | types et fonctions du journal locales | `use crate::feedback_log::{…}` |
| `scenario/src/runner.rs` | **aucun** : la fiche cite trois lectures ; `grep -rn 'feedback' crates/scenario crates/bots games examples templates --include='*.rs'` n'en trouve aucune hors `scenario/src/events.rs` | – |

### Écart à la fiche : `VariantTints` n'est pas créée

La fiche (point 2) décrit `variant_tint` comme lisant `GlobalAsset` et `Assets<CharacterConfig>` (`feedback_v3.rs:29-37,320-335,388-389,507`) et demande une ressource
`VariantTints` publiée par l'application. **C'est déjà fait par E3a** (`dette-d69-e3a-character-configs`, § 1 et § 5 : « `variant_tint(configs, nom, variante)` perd `GlobalAsset` ») :
`feedback_v3::variant_tint(&CharacterConfigs, &str, &str)` lit la ressource de simulation `CharacterConfigs` (`BTreeMap`, publiée par l'application, `sync_character_configs`), et
`feedback_v3.rs` est sans fuite depuis E3a (`grep -n 'GlobalAsset\|Assets<\|Handle<\|AssetServer' crates/game/src/feedback_v3.rs` : 0). Créer `VariantTints` ajouterait une seconde
table dérivée de `CharacterConfigs` à tenir synchrone (rechargement à chaud compris) pour ne rien gagner ; je ne l'ai donc pas créée. Si la ressource dédiée est voulue pour une
autre raison (précalcul `parse_tint`), elle s'ajoute en un fichier sans toucher à la coupe.

## 2. Sonde `make probe_game_sim` (avant/après)

Avant : worktree `origin/main` `a1edf161` ; après : l'arbre fusionné de la branche.

| | avant | après |
|---|---|---|
| fichiers du crate | 135 | 136 (`feedback_log.rs`) |
| fichiers à fuite | **44** | **44** |
| erreurs de résolution | **754** | **754** |
| `feedback.rs` | 37 (asset 11, ui 11, sprite/caméra 7, audio 3, 5 autres : la sonde n'affiche que les quatre premières catégories) | 37 (idem) |

Avant la fusion de E3d/E4/E5a (base `e0061e67`) : 46 fichiers à fuite, 757 erreurs, 130 → 131 fichiers ; mêmes constats. **Aucun fichier n'est libéré et aucun n'est ajouté à la
liste des fuites** : le journal était déjà sans fuite *en tant que code* (il ne sortait de la sonde qu'à cause du fichier qui le contenait) ; la coupe lui donne un fichier à lui,
dont la sonde dit qu'il compile en ECS seul, et il emmène avec lui les 37 erreurs restantes de `feedback.rs` côté application (rendu, sons, gizmos, caméra).

### Cliquet (`scripts/d69-sim-files.txt`) : **refusé par la sonde**

La fiche demande d'ajouter `feedback_log.rs` et `feedback_v3.rs`. Essayé (35 fichiers + les deux = 37) ; la sonde échoue (« `feedback_log.rs` dépend de fichiers hors liste : `boss.rs`,
`character/health/mod.rs` ; `feedback_v3.rs` dépend de fichiers hors liste : `bombs.rs`, `character/config.rs`, `familiars.rs` » ; ajouter ces cinq fichiers, c'est ouvrir la fermeture), comme pour les
fichiers libérés de E3a, E3b, E3c, E4 et E5a. Leur fermeture interne compte 79 fichiers dont 49 hors liste et **16 à fuite** (`camera/mod.rs`, `character/{config_asset,control,health_bar,visuals}.rs`,
`economy_assets.rs`, `global_asset.rs`, `interaction.rs`, `items.rs`, `legibility.rs`, `powerups_assets.rs`, `room_fog.rs`, `ui/hud.rs`, `waves/assets.rs`, `weapon_assets.rs`,
`world_sprites.rs`) ; dépendances directes de la liste à 35 fichiers : `feedback_log.rs` → `boss.rs`, `character/health/mod.rs` ; `feedback_v3.rs` → `bombs.rs`, `familiars.rs`, `character/config.rs`
(fermeture calculée par `internal_deps` de `scripts/probe-game-sim.py`, le calcul du cliquet ; `feedback_log.rs` ↔ `feedback_v3.rs` se référencent l'un l'autre).
Les deux fichiers n'entrent donc pas au cliquet ; le commentaire daté de la liste le dit, avec le calcul à relancer après E5c. Je n'ai pas forcé la liste (la sonde échouerait, et le
cliquet n'a de sens que fermé).

## 3. Vérifications (arbre fusionné, résultats réels)

Mesures sur l'arbre fusionné (`1aca73e9`, base `origin/main` `a1edf161`), machine : VPS Debian 8 cœurs, partagée (b6 y mesure ; charge 1,5 à 9 selon les essais).

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **sortie 0** : ordre des systèmes et des types rollback identiques, 4 520 lignes de schedules, 645 types |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, **802 scénarios**, 1 630 s ; **aucun `.trace` modifié** (`git status` : seuls ce rapport et `scripts/d69-sim-files.txt`) ; dont les 18 scénarios à `Event(kind: "feedback")` |
| `make test_crates` | `exit 0` : 84 suites, **1 303 tests passés, 0 échec**, 18 ignorés (les 9 tests de `telegraph_tests`, déplacés dans `feedback_log.rs`, en font partie des passés) |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`, `scripts/check-rollback-registration.sh`, `make test_tools` (25 tests), `make check_sim_boundary` | OK ×5 (13 crates SIM, 2 en dette D69) |
| `make lint` ×5 | 0 erreur (zombies, testbed, throne, gungeon, isaac) |
| `make gen GAME=…` ×5 | **431 scénarios générés joués, tous `ok ok`** (zombies 16, testbed 104, throne 39, gungeon 59, isaac 213), aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` (`--test fuzz_inputs -- --ignored`) | OK (378,7 s) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e5b cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` | OK ×3 (la cible `wasm32-unknown-unknown` a dû être installée sur le VPS : `rustup target add`) |
| **p2p-stress zombies** (`tests/scenarios/p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3566`, `COMPOSE_PROJECT_NAME=b2`) | **1 800 frames resimulées** (client 0 : 21 rollbacks, profondeur max 11 ; client 1 : 2), codes de sortie 0 0, 0 `DesyncDetected`, 2 999 lignes identiques ; même essai sur la branche avant fusion : 1 637 resimulées, mêmes critères |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 35 fichiers au cliquet ; § 2 |
| captures `cmp` | **non faites** (§ 4) |

Avant la fusion de `a1edf161` (base `e0061e67`), la même porte avait été passée sur la branche seule : `schedule_order_diff` identique, 802 scénarios et aucune trace modifiée, 1 299 tests de crates,
`make gen` ×5 tous `ok ok`, `p2p-stress` zombies 1 637 frames resimulées.

## 4. Captures `cmp` (non faites ici)

`play_scenario --capture` ouvre un contexte `bevy_winit` : sans affichage ni GPU il échoue (D94 : « Failed to build event loop », ni `WAYLAND_DISPLAY` ni `DISPLAY`, `xvfb-run` absent sur ce
VPS). **Non tenté ici, non contourné.** Commandes à rejouer sur le Mac (même comparaison que E3a : `cmp` image par image entre `origin/main` et la branche). Les scénarios couvrent
les genres du journal (flash et chiffres, secousse, hit stop, télégraphe de charge et de pattern, blank et son anneau, explosion, champion et sa teinte, transformation, larmes,
familiers, bombes) et les scénarios de la porte commune de l'étape 1 :

```bash
# Sur le Mac, depuis le dépôt : BASE = a1edf161 (origin/main), E5B = la branche dette-d69-e5b-feedback
git worktree add --detach ../alacod-base a1edf161
git worktree add ../alacod-e5b dette-d69-e5b-feedback
SCENARIOS="two_players_shooting champion_splits champion_explodes enemy_charge tourelle_couronne_telegraphe \
  blank_clears_bullets transformation_three tears_orbit status_charm gungeon_boss isaac_hud_items tears_shot_speed"
for tree in alacod-base alacod-e5b; do
  ( cd ../$tree && for s in $SCENARIOS; do
      cargo run -q -p scenario --features render --bin play_scenario -- tests/scenarios/$s.ron \
        --capture /tmp/e5b/$tree/$s --every 2 ; done )
done
for s in $SCENARIOS; do
  for f in /tmp/e5b/alacod-base/$s/*.png; do cmp -s "$f" "/tmp/e5b/alacod-e5b/$s/$(basename "$f")" || echo "DIFFÈRE $s $(basename "$f")"; done
done
```

Le brouillard filtre le rendu du feedback (`apply_feedback_cues`, chiffres, télégraphes lisent `FogView`) : `two_players_shooting` joue la carte par défaut de `zombies`
(`maps/le_relais_modules.ldtk`, `ui/fog.ron` `enabled: true`), donc la liste couvre un scénario à brouillard actif.

## 5. Limites

- La preuve « rendu identique » n'est pas faite ici (§ 4). Ce qui la remplace : la simulation lit le journal aux mêmes frames (la suite entière passe sans BLESS, dont les 18 scénarios qui ont une attente
  `Event(kind: "feedback")` ; `status_charm*`, `transformation_two` et `tears_shot_speed` n'en ont aucune et ne prouvent ici que l'absence de trace différente) et le rendu lit le même
  `FeedbackLog` ; le code de rendu n'a pas été modifié (seuls ses imports et sa position dans le fichier changent ; relecture : blocs déplacés identiques au caractère près).
- `feedback.rs` garde 37 erreurs de sonde : rendu, sons, gizmos, caméra (E9 le déplace en application).
- `Assets<FeedbackConfig>` et `AssetServer` restent dans `feedback.rs` (application) par construction (E5b ne touche pas au rendu).

## 6. Relecture indépendante

Deux agents `reviewer` en lecture seule (diff `origin/main...HEAD`, sans cargo) : **aucun bloquant, aucun majeur**.

- **Code** : blocs déplacés de `feedback.rs` vers `feedback_log.rs` identiques au caractère près (`diff` contre `origin/main:crates/game/src/feedback.rs` : seuls l'en-tête, les imports, `pub fn ranged_range`, le commentaire
  de `FeedbackConfigLoaded` et le `build` du plugin changent) ; `FeedbackAssetsPlugin` + `FeedbackLogPlugin` reproduisent l'ancien `build` (mêmes systèmes, mêmes schedules et ordres) à l'ancienne place de
  `core.rs` (headless compris) ; aucun callsite sur un ancien chemin, aucun `pub use` de compatibilité ; `feedback_log.rs` sans `Asset`, `Handle`, `Sprite`, `Camera`, audio, gizmos ; aucune fonction perdue ;
  aucun risque de déterminisme ajouté. L'écart `VariantTints` jugé justifié.
- **Rapport et docs** : sept mineurs, **tous corrigés** : citation de la sonde du cliquet (la liste d'essai compte 37 fichiers, la sortie est celle de la liste réellement essayée) ; l'énoncé « 802 scénarios à moments
  clés `feedback` » (802 est le total de la suite, 18 scénarios ont une attente `Event(kind: "feedback")`, `status_charm*`, `transformation_two` et `tears_shot_speed` n'en ont aucune) ; le brouillard filtre le
  rendu du feedback (`FogView`), il est couvert par `two_players_shooting` (carte du Relais) ; le script des captures ne créait pas le worktree de la branche ; la ventilation de la sonde (37 erreurs, quatre catégories
  affichées sur neuf) ; la section citée du rapport E3a (§ 5, pas § 3) ; la portée du `grep` (fichiers Rust).
- Fusion de `origin/main` pendant le travail (E3d, E4, E5a, D84 arrivés) : conflits dans `core.rs` (deux lignes) et `scripts/d69-sim-files.txt` (commentaires) résolus ; toute la porte rejouée sur l'arbre fusionné (§ 3).
