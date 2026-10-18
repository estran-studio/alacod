# dette-d69-combat-sim : `combat` (et `map`) passent de PENDING à SIM

Branche `dette-d69-combat-sim`, base `4cd1b245`. Étape (1) de D69 (`docs/taches/dettes.md`) : `make check_sim_boundary`
prouve par la compilation que `combat` est de la simulation (bevy ECS seul). Refactoring pur : **aucune trace de
scénario ne bouge**, aucun `BLESS`.

## 1. Fuites trouvées (`cargo tree -p combat -e normal --no-default-features`) et fermeture

Avant : 505 crates dans le graphe, dont 15 de présentation (`bevy_asset`, `bevy_audio`, `bevy_camera`, `bevy_gizmos`,
`bevy_matchbox`, `bevy_pbr`, `bevy_picking`, `bevy_render`, `bevy_sprite`, `bevy_text`, `bevy_ui`, `bevy_window`,
`bevy_winit`, `wgpu`, `winit`). Après : **147 crates, aucun de présentation**. Trois dépendances en étaient la source
(`cargo tree -i bevy_asset` : `animation` avec les features complètes de bevy, `bevy_common_assets`,
`bevy_matchbox`), plus du code qui n'avait pas besoin d'elles pour fuir.

| Fuite | Fermeture |
|---|---|
| `animation` (features complètes de bevy, tire tout le rendu) pour `FacingDirection`, `AnimationState`, `ActiveLayers`, `AnimationStateBundle` | Les quatre types vont dans `sim_core::animation_state` (SIM : `sim_core` est le crate de contrats que `combat`, `game` et `animation` connaissent déjà ; un crate de plus n'apportait rien). Octet pour octet les mêmes derives, `Hash`, `Debug`. Leur enregistrement rollback (`AnimationState`, `FacingDirection`, `ActiveLayers`, dans cet ordre) passe de `D2AnimationPlugin` à `sim_core::animation_state::AnimationStatePlugin`, ajouté dans `game::core` **juste avant** `D2AnimationPlugin` : même position avant `GgrsPlugin`, même ordre d'enregistrement, aucun type nouveau (parité des types vides inchangée). `animation` ne garde que la présentation et dépend de `sim_core`. |
| `bevy_matchbox` pour `PeerConfig = GgrsConfig<BoxInput, PeerId>` | `PeerConfig` reste à côté de `BoxInput` (`combat::actors`) ; `PeerId` vient de `matchbox_protocol` (crate de données : `cfg-if`, `derive_more`, `serde`, `uuid`, **sans** socket ni WebRTC), le même type que celui de `bevy_matchbox` (même dépôt git, même `rev`, `Cargo.lock` : une seule entrée). Le déplacer vers `sim_core` aurait demandé d'y déplacer aussi `BoxInput` pour rien. |
| `bevy_common_assets` + `#[derive(Asset)]` : `WeaponsConfig`, `MeleeWeaponsConfig`, `RonAssetPlugin`, `weapons_config_update_system` (`AssetServer`, `Assets`, `AssetEvent`) | `combat` ne voit plus que les types désérialisés (`WeaponAsset`, `MeleeWeaponAsset`). Les deux enveloppes `Asset`, les deux chargeurs RON et le rechargement à chaud vivent dans le nouveau `game::weapons` (`WeaponAssetsPlugin`, ajouté après `BaseWeaponGamePlugin`) ; `game::weapons` réexporte `combat::weapons::*` et masque `WeaponsConfig`/`melee::MeleeWeaponsConfig` : les ~45 sites d'appel `Assets<WeaponsConfig>` n'ont pas changé. |
| `Visibility` (`bevy_camera`) posée par la simulation sur les armes (`spawn_weapon_for_player`, `spawn_melee_weapon_for_character`) et lue/écrite par `weapon_inventory_system` | La simulation ne pose plus que `ActiveWeapon`. `weapon_inventory_system` (déménagé dans `game::weapons`) recalcule la visibilité de chaque arme portée à chaque `Update` et l'insère si elle manque ; `hide_new_melee_weapons` (PreUpdate, après `RunGgrsSystems`) cache l'arme de mêlée comme avant. Fuite que l'énoncé ne listait pas. |
| `Sprite` posé par `spawn_bullet` depuis `GgrsSchedule` | `spawn_bullet` insère `BulletLook { projectile_size, shot_size }` (non rollback, non haché, que des `Fixed`) ; `game::world_sprites::attach_bullet_visuals` (PreUpdate, après `RunGgrsSystems`, donc avant tout système d'`Update` comme avant) pose le `Sprite` : mêmes couleurs, même formule `f32` dans le même ordre (`to_f32(Size) × BASE_SPRITE`, × `to_f32(ShotSize)` si ≠ 1 ; 3,5 sinon). `ShotSize` n'est pas dans `Bullet` (son `Hash` est sous checksum) : d'où `BulletLook`. Filtre `With<Bullet>, Without<Sprite>` et non `Added<Bullet>` (voir §4). |
| Features de bevy | `combat` : `bevy` en `default-features = false` + `serialize` (`Entity` (dé)sérialisable, composants qui en portent une) + `bevy_color` (`Color` de `Player`) : deux features de données, sans crate de rendu (vérifié par le script). |

Aussi (« sans travail supplémentaire ») :
- **`map` passe en SIM** : sa dépendance `bevy_ecs_ldtk` n'était utilisée par aucun fichier (0 occurrence dans `crates/map/src`) et
  ses features `render`/`debug_ui` aussi ; bevy avec features par défaut. Retirés (`map_ldtk` garde son propre `render`).
- **`animation` passe en `APP`** : plus aucun état de simulation, que de la présentation ; aucun crate SIM n'en dépend (son
  graphe tirerait `bevy_render`, le script l'aurait signalé).
- `FORBIDDEN` du script gagne `bevy_common_assets` et `matchbox_socket` (le socket WebRTC) : une régression par ces deux chemins
  serait nommée.

## 2. `check_sim_boundary` avant / après

| | Avant | Après |
|---|---|---|
| SIM (graphe seul + `cargo check --no-default-features` ensemble) | 11 crates | **13** : + `combat`, `map` |
| PENDING (dette D69) | `combat`, `animation`, `map`, `map_ldtk`, `game` | `map_ldtk`, `game` |
| APP | `bots`, `scenario`, `telemetry` | + `animation` |
| graphe de `combat` | 505 crates, 15 de présentation | 147 crates, 0 |

## 3. Preuve « refactoring pur »

- `make test_scenarios_par SHARDS=3`, sans BLESS : voir §6 ; aucun `.trace` modifié (`git status` propre hors code et docs).
- Pas de nouveau type rollback, pas de changement de `Hash`, pas de système déplacé dans `GgrsSchedule` (les systèmes déménagés
  sont tous en `Update`/`PreUpdate`) ; le rapport de revue indépendante confirme les trois types déménagés identiques au `diff -w`
  près (hors doc), l'ordre d'enregistrement conservé, et qu'aucune requête de simulation ne lit `Visibility` ou `Sprite` sur
  armes et balles. Seule différence observable : le nom de type (`sim_core::animation_state::…` au lieu de `animation::…`) dans le
  dump détaillé `ALACOD_STATE_TRACE_FULL`, jamais dans le hash ni dans les `.trace`.
- **Captures avant/après** (`play_scenario --capture --at`, rendu réel sur le GPU du Mac, binaire de `4cd1b245` construit dans un
  worktree à part contre le binaire de la branche, `ALACOD_MUTE=1`), comparées octet à octet (`cmp`) : `tears_shot_speed`
  (`ShotSize` ×1,5) f18–f52, `two_players_shooting` f20–f70, `gungeon_boss` (balles `bullet_enemy`/`bullet_player` habillées
  par `skin_bullets`) f85–f135, et des images isolées de `tears_split`, `tears_orbit` : **137 images consécutives
  identiques, plus 7 isolées (`tears_split` f24 à f56, `tears_orbit` f40 et f80), 0 différence**. Dans les captures, la balle est visible à chaque image (aucun clignotement). Deux images
  de `gungeon_boss` (f200, f330) varient d'une exécution à l'autre **sur le binaire de `4cd1b245` lui-même** (mêmes empreintes que
  celles de la branche selon les exécutions : source de bruit du rendu, sans rapport avec le changement).
  Aucun scénario ne tire de balle `Explosive` (aucun contenu n'en déclare) : le test
  `world_sprites::le_sprite_d_une_balle_reprend_couleur_et_taille_de_l_apparition` fixe couleur (noir, blanc explosif), côté 3,5,
  `Size × BASE_SPRITE`, `ShotSize`, et le cas d'une balle qui perd son sprite.
- Sécurité rollback du visuel : le sprite et `BulletLook` ne sont pas rollback ; une balle recréée par resimulation est une entité
  neuve et reçoit son sprite au `PreUpdate` suivant la simulation ; une balle ressuscitée par `despawn_rollback` garde ses composants
  (désactivée, pas détruite) et son sprite.

## 4. Revue indépendante (agent `reviewer`, lecture seule, avant merge)

- **P2 corrigé** : première version avec `Added<Bullet>`. En p2p, un lot de frames dans un même `PreUpdate` peut créer une balle puis la
  détruire par `despawn_rollback` avant que le système ne la voie ; un rollback qui la ressuscite ne la rend pas « ajoutée »
  (`RollbackDespawned` retiré, `Bullet` inchangé) : elle aurait vécu sans sprite (invisible). Seulement en p2p avec prédiction
  fausse, invisible au synctest et aux captures. Corrigé par `With<Bullet>, Without<Sprite>` (couvre aussi le repli sans
  `BulletLook`), même changement pour `hide_new_melee_weapons` (`With<MeleeWeapon>, Without<Visibility>`), et le test fixe le cas.
- P3 corrigés : commentaire faux sur la visibilité des enfants dans `game::weapons`, doc de `spawn_bullet`, commentaire de
  `scenario::runner` qui plaçait `weapon_inventory_system` dans `BaseWeaponGamePlugin`.
- Axes déterminisme/rollback et frontière : rien à signaler.

## 5. Ce qui reste de D69 (étape 2, chiffré, aussi dans `docs/taches/dettes.md`)

Mesuré le 2026-10-18 sur la branche :

| Crate | Fichiers / lignes | Crates interdits dans le graphe seul | Détail |
|---|---|---|---|
| `game` | 114 / 47 286 | 18 (`bevy_asset`, `bevy_audio`, `bevy_camera`, `bevy_common_assets`, `bevy_gizmos`, `bevy_kira_audio`, `bevy_matchbox`, `bevy_pbr`, `bevy_picking`, `bevy_render`, `bevy_sprite`, `bevy_text`, `bevy_ui`, `bevy_window`, `bevy_winit`, `matchbox_socket`, `wgpu`, `winit`) | 68 fichiers touchent un type de présentation, dont 45 assets/`Handle`/`AssetServer`/`AssetEvent`, 8 `bevy_matchbox`/`matchbox_socket`, 6 `leafwing`, 4 `bevy_kira_audio`, 2 `egui` |
| `map_ldtk` | 31 / 8 595 | 19 (les mêmes + `bevy_ecs_ldtk`) | 23 fichiers importent `bevy_ecs_ldtk`, 26 touchent un type de présentation |
| `map` | — | **0** | fermé ici |
| `animation` | — | (présentation) | fermé ici : classé `APP` |

`bots` et `scenario` (APP) lisent `game::` 262 fois (`character` 60, `replay` 54, `weapons` 20, `entity` 17, `hub` 12, `loot` 8…) :
ils ne dépendront de `game_sim` qu'après le tri module par module. Plan : `game_sim` (simulation) et `game_app` (UI, caméra, audio,
réseau, `world_sprites`, `feedback`, `room_fog`, chargeurs RON ; `map_ldtk` de ce côté ou découpé de même), `game_sim` passe en SIM.
La recette de ce rapport est le modèle : donnée de simulation inchangée, enveloppe `Asset` et visuels dérivés de l'état côté
application, chemins d'appel conservés par réexport quand le module est coupé en deux.

## 6. Vérification

| Contrôle | Résultat |
|---|---|
| `make check_sim_boundary` | OK : 13 crates de simulation, 2 en dette D69 ; `cargo check --no-default-features` ECS seul des 13 |
| `make test_scenarios_par SHARDS=3` (sans BLESS) | 642 scénarios, 3 tranches ok, aucun `.trace` réécrit (deux exécutions : 221 s sur le code du WIP, 419 s sur le code final après la correction de revue, mêmes 642 verts) |
| `make test_crates` | 1 218 tests réussis, 0 échec |
| `make lint` ×5 | zombies, testbed, throne, gungeon, isaac : « aucune erreur » |
| `make gen GAME=…` ×5 | zombies 16, testbed 102, throne 39, gungeon 59, isaac 95 scénarios, tous « ok ok », aucune trace différente |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh` (38 occurrences auditées, exceptions `HashMap` des enveloppes d'asset déplacées de `combat` vers `game/src/weapons.rs`), `scripts/check-rollback-registration.sh`, `make test_tools` (25) | OK |
| fuzz `ALACOD_FUZZ=0:8` (`fuzz_inputs`, 3 600 frames) | 8 graines, aucune divergence |
| `scripts/p2p-stress.sh gungeon tests/scenarios/p2p_smoke.ron 3000` (`MATCHBOX_PORT=3571`, `COMPOSE_PROJECT_NAME=b1`) | **2 178 frames resimulées** (client 0), 0 desync, codes de sortie 0 0, 2 999 lignes identiques |
| build web : `cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` (le Makefile n'a que des cibles de build complètes avec `wasm-bindgen`/`wasm-opt`) | sortie 0 |

## 7. Non fait / limites

- Le build web est un `cargo check`, pas un `make build_…` complet ni une exécution dans un navigateur.
- Pas de capture p2p avec rollback réel : l'absence de clignotement est prouvée par les captures (synctest, images consécutives
  identiques), par le raisonnement ci-dessus et par la relecture indépendante ; le p2p réel est prouvé pour la simulation (traces
  identiques, 2 178 frames resimulées), pas pour le rendu.
- Le dump `ALACOD_STATE_TRACE_FULL` nomme les types avec leur nouveau chemin de module.
