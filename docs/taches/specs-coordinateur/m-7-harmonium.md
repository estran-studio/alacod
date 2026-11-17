# Tâche : M-7 « Feature `harmonium` » (Anthropic Sonnet ; présentation, hors simulation) — jalon **S2 dim 22 nov, cible ven 20**

Lis `~/orca/runs/alpha/commun.md`, `CLAUDE.md` (frontière simulation/présentation, `PresentationPlugin`), la ligne M-7 du § 6 de `docs/taches/plan-1837-cli.md`
(**sa preuve fait foi**). Branche `wquintal/m-7-harmonium`, worktree `~/orca/workspaces/alacod/m-7-harmonium` (partie d'`origin/main` `40c1cf82`).

## Sources (lecture seule)
- **Spec** : `~/Project/bascanada/harmonium/specs/013-moteur-dans-le-jeu/pret-pour-1837.md`, **§ 3** (correctif minimal en 4 fichiers) et **§ 6** (décision de William :
  option a, le jeu est la source du temps, harmonium suit une horloge externe ; unité = la pulsation dansée du profil ; période fractionnaire en frames permise).
- **API de l'horloge externe** : `~/Project/bascanada/harmonium/specs/013-moteur-dans-le-jeu/j6-horloge-externe.md` (`GameClock`, `ExternalClock::new(sim_hz, anchor_frame, num, den)`,
  `game_clock.sim_frame` à chaque frame de présentation, avant `HarmoniumSet`).
- harmonium `origin/main` = **`0189c6a`** (contient J6 `6c3e4c5`) : c'est la `rev` à épingler.

## Ce que tu livres
1. `crates/game/Cargo.toml` : `harmonium_bevy = { git = "https://github.com/estran-studio/harmonium", rev = "0189c6a…" (SHA complet), optional = true }` ; commentaires périmés
   retirés ; `cargo update -p harmonium_bevy` (l'arbre Bevy 0.17.3 doit quitter `Cargo.lock` ; aucune version partagée avec l'engine ne doit changer : vérifie le diff du lock).
2. `crates/game/src/audio/` : API actuelle (`HarmoniumPlugin::new(HarmoniumConfig { main_bank, percussion_bank, .. })`, `HarmoniumControl`) ; `adaptive.rs` et
   `setup_harmonium_music` supprimés ; blocs `HarmoniumTag` (`enemy/create.rs`) et `AiDriver` (`input_devices.rs`) retirés.
3. **Horloge du jeu (option a)** : à l'entrée en partie (une salle), poser `GameClock.clock = ExternalClock::new(60, frame_d'ancrage, num, den)` ; à chaque frame de présentation,
   `GameClock.sim_frame` = frame de simulation courante (`FrameCount`), avant `HarmoniumSet`. **Rien dans `GgrsSchedule`**, aucun type rollback, aucune lecture de `MusicClock` par la simulation.
4. **Donnée du jeu** : profil harmonium et période de la pulsation (num/den en frames) dans un réglage du jeu (fichier RON des assets du jeu, ou champ du manifeste : suis la
   convention existante), lint si le champ est mal formé ; le testbed porte un réglage neutre ; aucun nom de 1837 dans l'engine. Sans réglage : pas de musique.
5. Assets : banques `GeneralUser-GS.sf3`/`.sf2` + licence (release `studio-resources-2` d'`estran-studio/harmonium`) et `harmonium_trad.sf2` (`harmonium/assets/`) ; s'ils sont lourds,
   ne pas les commiter dans l'engine si une convention de téléchargement existe (`make dep_web` / scripts) : relève-la et suis-la ; le build web ajoute `game/harmonium`.
6. Preuves (ligne M-7 du § 6) : `cargo check -p game --features harmonium` natif et `--target wasm32-unknown-unknown` ; testbed lancé en natif avec la feature, **`ALACOD_MUTE=1`
   (volume 0, jamais d'écoute)** : `HarmoniumStatus::Ready`, `MusicClock` valide et `playing`, statut externe qui suit (journal) ; build web avec la feature ; `make check_rapide`
   (feature coupée par défaut) ; `make check_sim_boundary` ; aucune trace modifiée. § 121, D217, rapport `docs/taches/rapports/m-7-harmonium.md`, LIVRÉ.

Règles : jamais de profil debug (`--profile headless` pour les tests, release pour le jeu, wasm par make) ; un seul cargo ; sweep `target` et `target_wasm` après ;
**commite et pousse à chaque étape** (Mac sur batterie) ; ne touche ni `~/Project/bascanada/alacod`, ni `~/Project/bascanada/1837`, ni `~/Project/bascanada/harmonium` (lecture seule) ;
`status` « E prêt <sha> » par tranche verte, un seul `worker_done`.
