SHA de tête vérifiée avant le commit de ce rapport : `ce99807` (code).
Fiche : aucune. Correctif interactif demandé par William, session Claude Code sur son Mac
(aarch64-apple-darwin), **hors protocole Orca** : pas de `task-new.sh`, pas de worktree ; la
branche a été créée dans le checkout du Mac.
Branche : `fix-local-launch-sounds`. Base d'origine `b6a58a0` ; `origin/main` (`d877a26`) fusionné
dans la branche (merge `2970a54`, sans conflit) avant la vérification finale.
Date : 2026-10-03 ; agent : Claude Code (Sonnet 5.5).

Le commit de livraison ajoute ce rapport au SHA vérifié ci-dessus ; son SHA est donné dans la
ligne LIVRÉ.

## Fait

Cinq commits de code, plus le merge de `main` :

1. `32a21c2` **args** (`crates/game/src/args/mod.rs`) : `cargo run -p zombies` sans `--players` ni
   `--number-player` paniquait (`InvalidRequest "Number of players must be at least 1."` dans
   `system_after_map_loaded_local`), car `max_player` valait 0. Hors ligne (ni `--matchbox` ni
   `--allumette`), le build natif démarre maintenant avec **un** joueur local, comme le repli du canvas
   web. Les modes en ligne sont inchangés. Les scénarios passent par `GameArgsPlugin`, pas par
   `get_args()`.
2. `c322cf3` **feedback** (`crates/game/src/feedback.rs`, `docs/conventions.md` §9) : trois défauts
   de sons, présentation seulement (`PostUpdate`, absent en headless).
   - Reload : le son repartait à chaque image rendue tant que `reloading_ending_frame` était `Some`
     (22 départs en 1,4 s dans une session mesurée). Il part maintenant au passage `None` → `Some`,
     ce que `conventions.md` §9 documentait déjà.
   - Tir : le rollback de la session locale (synctest) recrée les balles des dernières frames, qui
     redevenaient `Added<Bullet>`. Un son par (tireur, frame de création).
   - `sounds/machine-gun.ogg` dure **17,7 s** (enregistrement de tir soutenu, mesuré à `ffprobe`) et
     était joué en entier à chaque balle. Palliatif : coupé en fondu après `SHOT_SOUND_MAX` = 250 ms
     (`Time<Real>`, `AudioInstance::stop`). Voir dettes.
3. `d79a868` puis `ce99807` **desync du dash** (`crates/game/src/character/player/input.rs`,
   `crates/combat/src/actors.rs`, `crates/game/src/character/mod.rs`,
   `tests/scenarios/dash_aim_change.{ron,trace}`) :
   - Symptôme : en jeu local, un dash avec la souris qui bouge gèle la partie. Le synctest trouve une
     frame rejouée différemment (`Detected checksum mismatch during rollback`), GGRS n'avance plus et
     l'avertissement se répète à chaque image.
   - Cause (nommée par `ALACOD_DIAG=1`) : seule la rotation de l'arme (`FixedTransform3D` de
     l'entité arme) diverge. `combat::weapons::system_weapon_position` (dans `GgrsSchedule`) la calcule
     à partir de `CursorPosition`, que `apply_inputs` n'écrivait qu'**après** les deux `continue` du
     dash. Pendant un dash le composant restait donc figé, et comme il n'est pas rollbacké, une frame
     resimulée lisait la visée d'une frame plus récente. Aucun scénario ne le voyait : leur visée est
     constante pendant un dash.
   - Correctif final (`ce99807`) : `apply_inputs` écrit `CursorPosition` à chaque frame, **avant**
     tout `continue`. Donnée dérivée de l'input, lue dans la même frame, donc non rollbackée, aucun
     nouveau type rollback. Seul effet de jeu : l'arme suit la visée aussi pendant les 15 frames d'un
     dash (aucun tir n'est possible pendant un dash, `weapon_rollback_system`).
   - Historique : `d79a868` avait d'abord enregistré `CursorPosition` en
     `rollback_and_trace_no_checksum`. Le merge de `main` a fait apparaître la règle de `CLAUDE.md`
     « ne jamais passer en `no_checksum` pour "sauver" les traces » ; William a choisi de recalculer la
     visée plutôt que de garder ou de passer sous checksum (83 traces à blesser). `ce99807` retire
     l'enregistrement et change l'ordre d'écriture ; `d79a868` reste dans l'historique.
   - Scénario de régression `tests/scenarios/dash_aim_change.ron` : la visée change pendant un dash.
     Sa trace est **nouvelle** (créée dans cette branche, re-blessée une fois entre `d79a868` et
     `ce99807` : quatre frames, f15-17 et f29, là où la visée diffère de (0,0) pendant le dash).
     Aucune trace existante n'est modifiée.
4. `dd47229` **fuzz d'inputs** (`crates/scenario/tests/fuzz_inputs.rs`, test `#[ignore]`) : parties
   aléatoires d'un joueur sur `maps/avant_poste.ldtk` (déplacements, tir, dash, sprint, mêlée,
   rechargement, interaction, visée) ; signale toute frame que le synctest rejoue différemment
   (`ALACOD_DIAG=1` nomme le composant). Variables : `ALACOD_FUZZ=<de>:<à>`, `ALACOD_FUZZ_FRAMES`,
   `ALACOD_FUZZ_MAP_SEED`, `ALACOD_FUZZ_SPARSE=1` (peu de zombies, pour que le joueur survive).
   `cargo test -p scenario --profile headless --test fuzz_inputs -- --ignored --nocapture`.

## Vérifié (état final : merge de `main` + `ce99807`, machine : Mac aarch64, `--profile headless`)

- `make test_scenarios` : **83 scénarios verts**, traces identiques, `test result: ok. 3 passed; 0
  failed; 7 ignored`, 253 s, code de sortie 0. `git diff origin/main --name-status -- tests/scenarios` :
  uniquement `A dash_aim_change.ron` et `A dash_aim_change.trace`.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p
  sim_core -p stats -p bots -p effects --no-fail-fast` : **351 réussis / 0 échec / 9 ignorés**, code de
  sortie 0 (8 ignorés préexistants + `fuzz_inputs`).
- `make lint` : `games/zombies : aucune erreur (5 personnages, 4 armes, 6 armes de corps à corps, 1
  vagues, 4 cartes)` ; `games/testbed : aucune erreur (7 personnages, 11 armes, 6 armes de corps à
  corps, 0 vagues, 7 cartes)`.
- `cargo fmt --all -- --check` : rien. `./scripts/check-rollback-registration.sh` : OK.
- Preuve de régression : avec l'ancienne place de l'écriture (`CursorPosition` après les `continue`),
  `ALACOD_SCENARIO=dash_aim_change` échoue : `frame 32: synctest mismatch (frames [30])`, « la
  simulation n'a atteint que la frame 32 sur 120 ». Avec `ce99807` : 120 frames, verte. Le même
  résultat avait été obtenu sur `d79a868` (sans son enregistrement rollback).
- Fuzz, sur le code **avant** le correctif : 30 graines sur 30 divergent entre les frames 20 et 532
  (3600 frames demandées). Sur l'état final : graines 0-29 à 3600 frames, **30 parties, 0
  divergence** ; graines 100-203 à 1500 frames, **104 parties, 0 divergence** ; graines 300-331 à 3600
  frames avec `ALACOD_FUZZ_SPARSE=1`, **32 parties, 0 divergence** (10 avec le joueur encore vivant
  à la fin, vague 2 atteinte). Total 166 parties, 0 divergence ni panique.
- À la main, par William, sur la version `d79a868` (seul l'ordre d'écriture de la visée change
  ensuite) : lancement sans argument OK ; William rapporte que le dash ne pose plus de problème.
  Journal de cette session : 2783 frames, 0 mismatch, 0 panic, fermeture propre (le journal ne trace
  pas les dashs) ; avant le correctif, un dash avait gelé la partie à la frame 3186. 9 sons de reload
  sur la session, écart minimum entre deux sons 68 frames (le défaut d'origine en produisait un
  toutes les 3 à 8 frames).
- Mesures de contexte (lag signalé par William) : `cargo run` (profil `dev`, opt-level 0) → environ
  **15,2 FPS** rendus (66 ms par image, 4,1 frames de simulation par image, pire image 143 ms) ;
  `cargo run --profile headless --features native` → **59,9 FPS** affichés par le compteur du jeu.
  Sons : session de test, 65 sons de tir pour 65 balles tirées (30 + 30 + 5) et 2 sons de reload
  pour 2 rechargements.

## Non fait / non vérifié

- **p2p à deux clients** (README §4, obligatoire quand la simulation change) : non exécuté ici (pas
  de Docker ni de serveur de signalisation sur ce Mac). `apply_inputs` est touché : à rejouer par
  l'orchestrateur, traces attendues identiques entre clients (599 lignes).
- **Bench** (`ALACOD_BENCH_STRICT=1`) : non exécuté. Le changement ne devrait pas peser (deux
  écritures de `i32` déplacées), mais rien n'a été mesuré.
- `./scripts/check-forbidden.sh` : **non exécutable sur ce Mac** (`declare -A` : bash 3.2). Contrôle
  manuel des lignes ajoutées par la branche pour `std::collections::HashMap`, `HashSet`, `rand::`,
  `Instant::now` : aucune occurrence. À relancer sur la machine Linux.
- `ce99807` (visée recalculée) n'a pas été rejoué **à la main** en jeu : il l'est par le scénario de
  régression et par 166 parties de fuzz. La version précédente (`d79a868`) l'a été.
- Le réglage `SHOT_SOUND_MAX` (250 ms) n'a pas été jugé à l'oreille par l'agent (aucune écoute
  possible) ; William n'a pas confirmé le rendu sonore du tir.
- Build web (wasm) : non compilé. `feedback.rs` n'utilise que des API déjà présentes sur toutes les
  cibles (`Time<Real>`, `Assets<AudioInstance>`), mais rien ne le prouve.

## Dettes laissées, décisions ouvertes

- **`[profile.dev]`** : `cargo run` et `make zombies` compilent sans optimisation (environ 15 FPS en
  jeu). `make zombies ARGS="--profile headless"` donne 60 FPS. Régler `opt-level = 1` (crates du
  workspace) et `3` (dépendances) dans `[profile.dev]` est la pratique Bevy, mais rallonge les builds
  de tout le monde : décision non prise.
- **Panic à la fermeture de la fenêtre** : `crates/game/src/camera/mod.rs:208` (`camera_control_system`)
  et `:419` (`player_indicator_system`) font `.single().unwrap()` sur `Query<&Window>` ; vu une fois
  sur quatre fermetures (code de sortie 101). Non corrigé.
- **Sons de tir** : `SHOT_SOUND_MAX` est un palliatif. Il faut un échantillon de coup unique (voie
  V6, avec une entrée dans `assets.yaml`) ; `games/testbed/assets/ui/feedback.ron` pointe vers le même
  fichier.
- **`scripts/check-forbidden.sh`** incompatible bash 3.2 (macOS) : `make check` ne passe pas en local
  sur Mac.
- **Proposition pour `CLAUDE.md`** (non appliquée, le fichier est tenu par l'orchestrateur) : un
  composant lu par la simulation mais non rollbacké doit être réécrit à chaque frame **avant** tout
  `continue`/retour anticipé du système qui l'écrit ; sinon c'est un état caché que le rollback ne
  restaure pas (`dash_aim_change`). Les scénarios à visée constante ne voient pas ce défaut, le fuzz
  d'inputs si : à envisager dans la CI de nuit (`scripts/nightly.sh`), par exemple `ALACOD_FUZZ=0:8`.
- `FacingDirection` n'est pas non plus mis à jour pendant un dash, mais il est rollbacké
  (`rollback_and_trace`) : pas de desync, laissé tel quel.
