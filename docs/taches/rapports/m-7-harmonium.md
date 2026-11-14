# M-7 — Feature `harmonium` : la musique suit l'horloge du jeu

Jalon servi : **S2, dimanche 22 novembre 2026** (cible vendredi 20 ; `plan-1837-cli.md` §6, ligne M-7 : « la musique de la salle par harmonium »).
Branche : `wquintal/m-7-harmonium`, partie d'`origin/main` `40c1cf82`. Sources lues : `harmonium/specs/013-moteur-dans-le-jeu/pret-pour-1837.md` (§ 3 correctif minimal, § 6 décision de William : option a)
et `j6-horloge-externe.md`. harmonium épinglé : `0189c6a5f6f10990896723d81ccaaf85ed169326` (`origin/main` d'harmonium, contient J6 `6c3e4c5`).

**État : en cours (reprise après les pauses cargo du 14 nov).** Ce rapport est complété à chaque preuve obtenue ; les lignes « pas fait » sont à jour au dernier commit.

## Périmètre et résultat

| Point du lot | Issue | Preuve |
|---|---|---|
| `harmonium_bevy` épinglé par `rev` (SHA complet), URL `estran-studio/harmonium`, commentaires périmés retirés | Fait | `crates/game/Cargo.toml`, commit `ba8399da` |
| `cargo update -p harmonium_bevy` : l'arbre Bevy 0.17.3 quitte le lock, aucune version partagée avec l'engine ne change | Fait | diff du `Cargo.lock` comparé paquet par paquet : 3 ajouts (`harmonium`, `rosc`, `serde_path_to_error`), 88 noms retirés ou réduits (Bevy 0.17.3 et ses dépendances), `oxisynth`, `oxisynth-chorus`, `oxisynth-reverb`, `soundfont` passent du registre au fork `estran-studio/OxiSynth` (dépendance d'harmonium) ; aucun autre paquet ne change de version |
| API actuelle (`HarmoniumPlugin::new(HarmoniumConfig)`, `HarmoniumControl`) ; `adaptive.rs`, `setup_harmonium_music`, `HarmoniumTag` (`enemy/create.rs`), `AiDriver` (`input_devices.rs`) retirés | Fait | `crates/game/src/audio/music.rs` (neuf), `audio/mod.rs`, `git rm audio/adaptive.rs` ; `cargo check --profile headless -p game --features harmonium` natif : 0 erreur (aucun avertissement neuf dans les fichiers touchés) |
| Option a : `GameClock` posé à l'entrée en partie, `sim_frame` à chaque frame de présentation depuis `FrameCount`, rien dans `GgrsSchedule` | Fait | `anchor_music_clock` (`OnEnter(InGame)`, `ExternalClock::new(60, FrameCount.frame, num, den)`), `feed_music_clock` (`Update`, avant `HarmoniumSet`), `release_music_clock` (`OnExit(InGame)`) ; 3 tests verts avec la feature : `cargo test --profile headless -p game --features harmonium --lib audio::music` (profil inconnu ou période nulle refusés, muet = volume 0 sans pause, ancrage à la frame d'entrée puis `sim_frame` suit `FrameCount`, sortie retire l'horloge) |
| Donnée du jeu : profil et période en fraction de frames, lint, réglage neutre du testbed, rien de 1837 dans l'engine, sans réglage pas de musique | Fait | champ `music` du manifeste (`content::manifest::MusicEntry`, `PulseFrames`), `content::lint::lint_music` (`OutOfRange`), fixtures `music_ok` / `music_pulse_zero` / `music_profile_empty`, `crates/content/tests/m7_music_fixtures.rs`, test `parses_music_with_default_banks` ; testbed : `music: (profile: "gmd_pop", pulse_frames: (num: 30, den: 1))` (`games/testbed/assets/game.ron`) |
| Banques : pas commitées, convention de téléchargement | Fait | `scripts/harmonium-banks.sh` (`gh release download studio-resources-2`, cache par poste), `.gitignore` `games/*/assets/harmonium/` ; le **SF3** (7,6 Mo) sert natif et web (oxisynth lit le SF3 sur toutes les cibles : `features = ["sf3"]`), le SF2 de 32 Mo n'est pas utilisé |
| Build web avec la feature | Fait pour le script, **pas encore prouvé** | `scripts/build-web-games.mjs` : `WEB_FEATURES=game/harmonium` s'ajoute à `render` ; build joué : pas fait |
| `cargo check -p game --features harmonium` en `wasm32-unknown-unknown` | **Pas fait** (arrêté par la pause batterie du 14 nov) | |
| Testbed natif avec la feature, `ALACOD_MUTE=1` : `HarmoniumStatus::Ready`, `MusicClock` valide et `playing`, statut externe qui suit (journal) | **Pas fait** | |
| `make check_rapide`, `make check_sim_boundary`, aucune trace modifiée | **Pas fait** | |
| § 121, D217, ce rapport, LIVRÉ | § 121 et D217 faits (`docs/conventions.md`, `docs/taches/dettes.md`) ; rapport en cours ; LIVRÉ pas encore | |

Aucun crate `SIM` ne dépend d'harmonium (la dépendance est dans `game`, optionnelle ; `content` ne porte que des données).
