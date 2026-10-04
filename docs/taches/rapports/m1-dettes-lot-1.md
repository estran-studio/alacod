# Rapport m1-dettes-lot-1 — lot de dettes M1 (D31, D33, D35, D37, kiter dans le mur)

**Branche** `m1-dettes-lot-1`, partie de la tête livrée de T1.13 `a5df89d` (contient T1.9 et
T1.13 ; même worktree, même target). Fiche : `docs/taches/m1-dettes-lot-1.md` (lue dans le main
local d'orch, pas encore poussée). Base `origin/main` `e26a3e3` (inchangé à la livraison, rien à
merger).

## État en cours

- **Fait** : D31, D33, D35, D37, kiter (D39 documentée) et l'ajout `Scenario.mode` (§1),
  vérifiés (§2). Livré.
- **Target** purgé après la suite complète (16G, /home 75G libres).
- **Reste à l'orchestrateur** : bless de `portal_next_floor` (preuve §1, D37), des 18 générés de
  T1.13 et des 3 de T1.9 si la branche est vérifiée avant elles ; reporter D39 dans `dettes.md` ;
  bench, p2p, merge.

## 1. Fait (un commit par dette)

- **D31** (`D31 : …`) : `camera_control_system` et `player_indicator_system`
  (`crates/game/src/camera/mod.rs`) : `let Ok(window) = windows.single() else { return };`.
  Vérifié : `cargo build -p zombies --profile headless` (features par défaut, rendu compris)
  compile (32 min, target purgé). **La fermeture de la fenêtre n'a pas été testée à l'écran**
  (aucun affichage dans cette session).
- **D33** (`D33 : …`) : `scripts/check-forbidden.sh` sans `declare -A` (compteurs dans un tableau
  indexé parallèle à `PATTERNS`). Sortie et code de retour **identiques** à l'ancien script, en
  mode normal (0) et `--strict` (1), sous bash 5, sous `bash --posix`, et sous **bash 3.2.57**
  (image docker `bash:3.2`, GNU grep ajouté par `apk`) — où l'ancien script échoue bien sur
  « declare: -A: invalid option ».
- **D35** (`D35 : …`) — **écart à la fiche** : `--no-default-features` sur `-p content --bin
  alacod` n'aurait rien changé (`content` n'a aucune feature). `cargo tree -e features -i
  bevy_pbr` montre la vraie cause : **`bevy_fixed`** dépendait de `bevy = "0.19.1"` avec ses
  features par défaut (2d, 3d, ui, audio), activées pour toute crate qui en dépend. Correctif :
  `bevy_fixed` en `default-features = false` ; `utils` déclare `["keyboard", "bevy_camera"]`
  (`KeyCode`, `Camera2d` de `camera/tod.rs` et `web/logs.rs`, qu'il recevait jusque-là par
  `bevy_fixed`). Le `Makefile` ne change pas.
  - Arbre de `content` : plus aucun `bevy_render`, `bevy_pbr`, `bevy_sprite_render`,
    `bevy_ui_render`, `bevy_core_pipeline`.
  - Sortie du lint **identique** avant/après :
    ```
    games/zombies : aucune erreur (5 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 4 cartes)
    rc=0
    games/testbed : aucune erreur (15 personnages, 14 armes, 6 armes de corps à corps, 0 vagues, 17 cartes)
    rc=0
    ```
  - Coût du binaire du lint depuis zéro (Bevy minimal) : 224 s, RSS rustc max ~1,9 Go
    (2 jobs), contre les 6 Go et plusieurs minutes notés par D35.
  - Les 11 crates sans features Bevy par défaut (`bevy_fixed utils sim_core run stats combat
    world effects behaviors bots content`) compilent **seules** (`cargo check -p X --tests`) ;
    tests de `content` seuls verts (76 + 70). Les builds unifiés (`game`, jeux, `cargo test` de la
    CI, suite des crates) gardent les mêmes features : `game`, `map`, `scenario`… demandent Bevy
    complet eux-mêmes.
- **D37** (`D37 : …`) : `follower` de `testbed/floor_b.ldtk` de (6, 10) à (6, 9) (édition du JSON :
  `__grid`, `px`, `__worldY`). La rangée 11 est un mur de x = 3 à 8 : en (6, 10) le corps le
  chevauchait. **Preuve §10** (dumps `ALACOD_DUMP_TRACE` avant/après, même binaire, seul l'asset
  change ; `scripts/trace-diff.py` + comparaison par entité sur toutes les frames) :
  - `bench_cave`, `explode_wall` (séquence `caverne`, qui finit sur `floor_b`) : **identiques**.
  - `portal_next_floor` (et `clock_floor_reset`, T1.9, sans trace de référence) : première
    différence f229 (entrée dans `floor_b`) : `FixedTransform3D` du follower (NetId 80) et de son
    `CharacterSpawn` (NetId 58 : y 616 → 632). Puis, **en chaîne** (le follower bouge désormais) :
    breacher (81) `FixedTransform3D`/`Velocity` dès f311 (séparation entre ennemis), `Velocity`
    du joueur dès f320 (poussée), positions des hitbox de griffe du breacher, `FlowFieldCache` dès
    f550. La fiche attendait « position et mouvement du follower seulement » : les effets en chaîne
    sont la conséquence directe d'un follower qui bouge.
  - Le follower bouge : 1 position sur 2 600 frames avant, 1 425 après. Santé finale du joueur
    identique (2,5), attentes de `portal_next_floor` vertes. **Bless de `portal_next_floor` à
    orch.** `floor_d` gardé (`trois_niveaux` inchangé).
- **Kiter dans le mur** (`D39 … documentée, non corrigée`) : diagnostic par instrumentation
  temporaire (retirée). À f543, `move_enemies` (`crates/game/src/character/enemy/ai/pathing.rs`)
  trouve le déplacement diagonal bloqué par le coin d'un mur (616, 632, 48 × 16), puis glisse :
  « X seul » depuis la position de départ (libre) **et** « Y seul » depuis la position de départ
  (libre, commentaire « independent of X ») — les deux sont appliqués, ce qui donne exactement la
  position diagonale refusée : chevauchement 0,86 × 0,12 à f544, ressorti à f545. Ce n'est **pas**
  propre à `KeepDistance` : c'est le glissement générique de tous les ennemis. `move_characters`
  (joueurs, `player/input.rs:638`) a déjà le correctif (Y testé depuis le X déjà glissé). La
  correction toucherait tous les ennemis (zombies compris) : hors du critère « local au behavior
  sans trace existante changée » → **D39** ci-dessous ; commentaire de `kiter.ron` mis à jour,
  attente toujours bornée à f540.
- **Ajout d'orch — `Scenario.mode`** (commit `Scenario.mode : …`) : `mode: Option<EntryMode>`
  l'emporte sur `entry.mode` (le runner remplace `entry.mode` du manifeste avant de l'insérer) ;
  absent, rien ne change. `mode: Floors` sans `floors` : échec du runner. `RecordedSettings::mode`
  (réenregistrement). Le générateur écrit `mode: Sandbox` dans les gabarits (armes et ennemis)
  d'un jeu qui déclare `generate_template` ; testbed et zombies sans le champ (générés
  inchangés). `EntryMode` dérive `Serialize`. Tests `mode_impose_par_le_scenario` (zombies
  `Waves` → `Sandbox` imposé, sans champ → `Waves`, `Floors` sans `floors` refusé) et
  `gabarits_generate_template_en_sandbox`. Une ligne au §28.

## D39 (à reporter dans `dettes.md` par orch)

| D39 | Glissement le long des murs de `move_enemies` : « X seul » et « Y seul » sont testés chacun depuis la position de départ puis appliqués tous les deux ; deux déplacements libres séparément se combinent dans le coin d'un mur (constaté : `enemy_kiter_still`, f544, chevauchement 0,86 × 0,12, une frame) | `crates/game/src/character/enemy/ai/pathing.rs` (bloc « Try Y only (independent of X) ») | tester Y depuis le X déjà glissé, comme `move_characters` (`player/input.rs:638`) ; touche tous les ennemis : preuve §10 sur toutes les traces, bless, puis étendre l'attente de `kiter.ron` à `frames` |

## 2. Vérifié (résultats réels)

Sur la branche (base `e26a3e3`, contient T1.9 et T1.13) :

- **Scénarios** (via la suite des crates) : 127 joués, **0 attente en échec** ; **une seule
  « trace différente » : `portal_next_floor`** (D37, prouvée au §1) ; 21 sans trace (18 générés
  de T1.13, 3 de T1.9). Aucune trace bénie par moi.
- **Tests des crates** (`scenario run combat game content map_ldtk map sim_core stats bots
  effects utils behaviors world`, `--include-ignored`) : 491 verts hors `scenarios` (ci-dessus),
  dont `placement` 6/6. Seul autre échec : le doctest `rust,ignore` de `game::waves`,
  préexistant (identique dans main).
- **`make lint`** : `games/zombies` et `games/testbed` sans erreur (sortie identique, D35).
- **`make gen`** : zombies 16/16 et testbed 32/32 attentes `ok`, **aucun fichier généré
  modifié**.
- **`fmt`** propre ; **`check_forbidden`** (nouveau script) 4 occurrences, identique ;
  **`check_rollback_registration`** OK ; **exemples racine** compilés ; **`cargo build -p zombies`**
  (rendu) compilé (D31).

## 3. Écarts, non fait / incertain

- D35 : le correctif porte sur `bevy_fixed`/`utils`, pas sur la cible `lint` (le flag de la fiche
  était sans effet).
- D37 : différences en chaîne au-delà du follower (§1), attendues.
- Kiter : non corrigé (D39).
- Fermeture de fenêtre (D31) non testée à l'écran. Lint de `throne` : pas sur cette base.
- Bench strict et p2p : non faits (orch).

## 4. Dettes, questions ouvertes

- D39 (ci-dessus).
- Le follower de `floor_b` bouge mais garde `AnimationState("Idle")` (présentation, hors trace
  de jeu ; à regarder si le follower doit jouer `Run`).
