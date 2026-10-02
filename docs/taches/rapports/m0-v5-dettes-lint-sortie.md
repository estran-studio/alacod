Tête de branche vérifiée (code) : `598ce33cfb5eccadffe97510c71ce3ecb587f73c`
Fiche : [`T3.4-dettes-lint-sortie`](../T3.4-dettes-lint-sortie.md)
Branche : `m0-v5-dettes-lint-sortie` — agent : Codex — 2026-10-02.

Ce SHA est la tête au moment d’écrire le rapport. Le commit suivant ne contient que ce
rapport ; son SHA de livraison est transmis à William. Base intégrée : `main` à
`2c6fe0e` (les deux commits reçus pendant la tâche ne changent que la CI).

## Fait

- **Huit règles de lint, huit fixtures** : clés répétées dans `weapons.ron`,
  `melee_weapons.ron` et `items/powerups.ron` (`KeyedEntries`, `DuplicateId`) ;
  `TimedModifier` avec `op: Mul` et `value <= 0` ; durées nulles du flash et de la
  secousse ; amplitude négative ; son de feedback absent ; armes de départ dépassant
  `weapon_slots`. Chaque test a échoué avant sa règle, puis réussi après. Aucune de ces
  huit règles n’était déjà couverte par le parse.
- `registry.rs` charge un schéma typé pour les fichiers Ui nommés `feedback.ron`,
  correspondant à `game::feedback` ; les autres fichiers Ui restent validés syntaxiquement.
  `lint.rs` vérifie les plages et l’existence des sons sous `assets/`, comme pour les armes.
  Un test supplémentaire contrôle le nombre et le kind des erreurs de chaque fixture
  (deux erreurs `frames`, une par effet, dans `feedback_frames_zero`). Les tests couvrent
  aussi les bornes permises : Mul positif, autres opérations négatives ou nulles,
  amplitude zéro et nombre d’armes égal au nombre d’emplacements.
- **D7** : `games/testbed/assets/ZombieShooter/Sprites/Character/player_config.ron`
  déclare `weapon_slots: 3`. La lecture de `create_player` montre que les trois armes
  étaient déjà créées : aucune n’était perdue ; la capacité limitait seulement les
  ramassages. Aucun scénario ou trace n’a été modifié.
- **D8/D9** : impressions de debug remplacées par `debug!` dans `map/src/generation`,
  `map_ldtk/src/generation` et `player_spawn.rs` ; suppression de la ligne vide de debug.
  Le message des armes murales précise qu’il compte les emplacements dans les copies de
  gabarits de salles. Le diagnostic d’erreur de carte invalide reste visible.
- **D11** : `ScenarioOutcome.entity_hits` lit les compteurs `HitCount` en fin de partie,
  hors simulation. `alacod-gen --play` affiche leur total dans la colonne « coups »
  (une seule cible porte ce compteur dans son arène) et les frames réellement atteintes.
  Le test du gabarit volontairement faux compare ce compteur au nombre nommé par
  l’échec d’`EntityHits`.
- **D16** : `GameManifest::load` active `implicit_some`. Les deux manifestes déclarent
  explicitement `mode: Waves` et `mode: Sandbox`, sans `Some(...)` ; la fixture
  `entry_mode_waves_without_waves` utilise `mode: Waves`. Le test du chargeur a échoué
  avec « Expected option » avant la correction, puis réussi dans la suite `content`.
  Sur cette base, les manifestes des jeux omettaient auparavant le champ ; les modes
  explicites conservent ceux déjà déduits par le moteur.
- `docs/conventions.md` documente les huit règles, les doublons, les emplacements et la
  syntaxe du mode. D7/D8/D9/D11/D16 sont marquées « fait dans T3.4 » dans `dettes.md`.

## Vérifié

Toutes les commandes Cargo/make ont utilisé `source ../env.sh`,
`CARGO_BUILD_JOBS=4` et le target amorcé par `scripts/task-new.sh`, avec une seule
compilation à la fois et le profil `headless`. Aucun build avec rendu.

| Commande exécutée | Résultat réel |
|---|---|
| `cargo test -q --profile headless -p content --test lint_fixtures <fixture>` pour chacune des huit nouvelles fixtures | Huit échecs attendus avant correction (code 101), puis huit succès individuels (1 réussi / 0 échec chacun). |
| `cargo test -q --profile headless -p content --lib loads_implicit_entry_mode` avant correction | Échec attendu du chargement de `mode: Waves` ; test vert ensuite dans la suite complète. |
| `cargo test -q --profile headless -p content` | 64 unitaires + 36 tests de fixtures = 100 réussis, 0 échec, 0 ignoré. Les fixtures passent de 27 à 36 : huit règles + un contrôle d’isolation. |
| `make test_scenarios` | 61 scénarios, 44 617 frames, traces comparées aux références existantes sans bless ; test binaire : 2 réussis / 0 échec / 7 ignorés, 315,96 s. |
| `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots -p effects --no-fail-fast` | 275 réussis / 0 échec / 8 ignorés, dont 30 tests d’attentes, 36 de fixtures et le test du générateur ; les 61 scénarios sont rejoués dans cette commande. |
| `make lint` | Zéro erreur : zombies (4 personnages, 4 armes, 6 mêlées, 1 config de vagues, 2 cartes), testbed (7 personnages, 4 armes, 6 mêlées, 0 config de vagues, 4 cartes). |
| `cargo fmt --all -- --check` | Code 0, aucune sortie. |
| `./scripts/check-forbidden.sh` | Code 0 ; les 4 occurrences préexistantes : 3 `HashSet`, 1 `rand::` dans un commentaire. Aucune nouvelle occurrence. |
| `./scripts/check-rollback-registration.sh` | Code 0, « OK » ; aucun appel direct interdit. |
| `make gen GAME=zombies` | Dix armes, attentes et traces toutes `ok` ; aucune modification des `.ron` ou `.trace` générés. |
| `ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py` | 61 scénarios / 44 617 frames, zéro échec, 314,15 s. `bench_bullets` : 102,4 fps (minimum 70), 155 balles ; `bench_horde` : 66,0 fps (minimum 38), 61 ennemis. Pas d’autre compilation locale pendant cette passe. |
| `bash ../verification/p2p.sh` (détails ci-dessous) | Deux clients réels ; `cmp` réussi ; 599 lignes par trace. Service de signaling arrêté après le test. |
| `git diff --exit-code -- tests/scenarios docs/taches.md` ; `git diff --check` | Codes 0 ; aucune trace, aucun scénario ni journal modifié ; aucune erreur d’espacement. |
| `git check-ignore -v <nouveau fichier>` pour les 18 nouveaux fichiers de fixtures | Aucun ignoré. |

P2P : le script lance `docker compose -f docker-compose.ci.yaml up -d signaling`,
puis `cargo build -q -p zombies --profile headless --no-default-features`.
Il lance deux `timeout 180s cargo run -q -p zombies --profile headless --no-default-features`
avec `ALACOD_HEADLESS=1`, `ALACOD_EXIT_AT_FRAME=600`, `APP_VERSION=x`,
`--matchbox ws://127.0.0.1:3536 --lobby t3-4-<pid> --number-player 2 --players localhost remote`
et des `--cid`/`--name` distincts. `ALACOD_STATE_TRACE` vise `../verification/p2p-0.trace`
et `p2p-1.trace`. Les deux processus terminent avec succès ; `cmp` et `sha256sum` donnent
le même contenu : `a44f5b4f531e0aceb25ac3f66a9be9b02d15eaf9cea84e1c4ba679575a3b0bce`.
Le script termine par `docker compose -f docker-compose.ci.yaml down`.

Résultats observés dans la nouvelle colonne :

| Arme | Frames | Coups |
|---|---:|---:|
| machine_gun | 600 | 20 |
| pistol | 600 | 4 |
| rifle | 600 | 8 |
| shotgun | 600 | 49 |
| axe | 400 | 3 |
| bare_hands | 400 | 6 |
| club | 400 | 4 |
| knife | 400 | 9 |
| sword | 400 | 5 |
| zombie_claws | 400 | 4 |

La sortie des scénarios contient zéro occurrence de « Adding … doors », « adding room »
ou « player spawn ». Les messages normaux restent présents.
Les logs locaux sont dans `/home/wq/Project/bascanada/alacod_tasks/m0-v5-dettes-lint-sortie/verification/`
(`*-rouge.log`, `*-vert.log`, `content.log`, `scenarios.log`, `crates.log`, `lint.log`,
`fmt.log`, `forbidden.log`, `rollback.log`, `gen.log`, `bench.log`, `metrics.log`, `p2p-*`).

## Non fait / incertain / non vérifié

- Aucun dump `trace-diff` et aucun bless : aucune référence n’a changé. Les 61 traces
  ont été comparées intégralement par le runner, y compris celles du testbed après D7.
- Pas de rendu, de vidéo, de revue visuelle ni d’écoute audio. Le lint vérifie l’existence
  des sons ; il ne vérifie pas leur lecture ni leur contenu sonore.
- Aucun nouveau scénario de ramassage dans le testbed : la capacité de trois est validée
  par le lint et les scénarios existants, sans mise en scène dédiée d’un troisième ramassage.
- CI distante non vérifiée dans ce rapport.
- Aucun merge dans `main`, aucune suppression de worktree, aucune ligne de journal :
  ces actions restent à l’orchestrateur. Aucune autre fiche commencée.

## Dettes laissées

- D5/D6 et les autres dettes hors de ce lot restent ouvertes. La participation de
  `HitCount` au checksum/à la trace n’est pas modifiée par la colonne du générateur.
- L’amorçage a copié un target `headless` de 52 Go, contrairement aux 3 Go annoncés dans
  le README. Le worktree et son target sont conservés pour la revue de l’orchestrateur.
- Aucune règle de ce lot laissée inachevée ; aucun échec restant dans les vérifications locales.
