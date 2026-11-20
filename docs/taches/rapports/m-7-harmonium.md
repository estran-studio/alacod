# M-7 — Feature `harmonium` : la musique suit l'horloge du jeu

Jalon servi : **S2, dimanche 22 novembre 2026** (cible vendredi 20 ; `plan-1837-cli.md` §6, ligne M-7 : « la musique de la salle par harmonium »).
Branche : `wquintal/m-7-harmonium`, partie d'`origin/main` `40c1cf82`. Sources lues : `harmonium/specs/013-moteur-dans-le-jeu/pret-pour-1837.md` (§ 3 correctif minimal, § 6 décision de William : option a)
et `j6-horloge-externe.md`. harmonium épinglé : `0189c6a5f6f10990896723d81ccaaf85ed169326` (`origin/main` d'harmonium, contient J6 `6c3e4c5`).

**État : livré** (commits docs seuls après `aac08438`).

## Périmètre et résultat

| Point du lot | Issue | Preuve |
|---|---|---|
| `harmonium_bevy` épinglé par `rev` (SHA complet), URL `estran-studio/harmonium`, commentaires périmés retirés | Fait | `crates/game/Cargo.toml`, commit `ba8399da` |
| `cargo update -p harmonium_bevy` : l'arbre Bevy 0.17.3 quitte le lock, aucune version partagée avec l'engine ne change | Fait | diff du `Cargo.lock` comparé paquet par paquet : 3 ajouts (`harmonium`, `rosc`, `serde_path_to_error`), 88 noms retirés ou réduits (Bevy 0.17.3 et ses dépendances), `oxisynth`, `oxisynth-chorus`, `oxisynth-reverb`, `soundfont` passent du registre au fork `estran-studio/OxiSynth` (dépendance d'harmonium) ; aucun autre paquet ne change de version |
| API actuelle (`HarmoniumPlugin::new(HarmoniumConfig)`, `HarmoniumControl`) ; `adaptive.rs`, `setup_harmonium_music`, `HarmoniumTag` (`enemy/create.rs`), `AiDriver` (`input_devices.rs`) retirés | Fait | `crates/game/src/audio/music.rs` (neuf), `audio/mod.rs`, `git rm audio/adaptive.rs` ; `cargo check --profile headless -p game --features harmonium` natif : 0 erreur (aucun avertissement neuf dans les fichiers touchés) |
| Option a : `GameClock` posé à l'entrée en partie, `sim_frame` à chaque frame de présentation depuis `FrameCount`, rien dans `GgrsSchedule` | Fait | `anchor_music_clock` (`OnEnter(InGame)`, `ExternalClock::new(60, FrameCount.frame, num, den)`), `feed_music_clock` (`Update`, avant `HarmoniumSet`), `release_music_clock` (`OnExit(InGame)`) ; 3 tests verts avec la feature : `cargo test --profile headless -p game --features harmonium --lib audio::music` (profil inconnu ou période nulle refusés, muet = volume 0 sans pause, ancrage à la frame d'entrée puis `sim_frame` suit `FrameCount`, sortie retire l'horloge) |
| Donnée du jeu : profil et période en fraction de frames, lint, réglage neutre du testbed, rien de 1837 dans l'engine, sans réglage pas de musique | Fait | champ `music` du manifeste (`content::manifest::MusicEntry`, `PulseFrames`), `content::lint::lint_music` (`OutOfRange`), fixtures `music_ok` / `music_pulse_zero` / `music_profile_empty`, `crates/content/tests/m7_music_fixtures.rs`, test `parses_music_with_default_banks` ; testbed : `music: (profile: "gmd_pop", pulse_frames: (num: 30, den: 1))` (`games/testbed/assets/game.ron`) |
| Banques : pas commitées, convention de téléchargement | Fait | `scripts/harmonium-banks.sh` (`gh release download studio-resources-2`, cache par poste), `.gitignore` `games/*/assets/harmonium/` ; le **SF3** (7,6 Mo) sert natif et web (oxisynth lit le SF3 sur toutes les cibles : `features = ["sf3"]`), le SF2 de 32 Mo n'est pas utilisé |
| Build web avec la feature | Fait | `cargo build --locked -p zombies --target wasm32-unknown-unknown --no-default-features --features render,game/harmonium` (`RUSTFLAGS=--cfg=web_sys_unstable_apis`, `CARGO_TARGET_DIR=target_wasm`, profil dev du Makefile) : fini en 26 min 34 s, `zombies.wasm` 2,4 Go (débogage) ; `wasm-bindgen` 0.2.129 (version du `Cargo.lock`, installé dans `/tmp`, celui du poste est en 0.2.106) : `wasm_bg.wasm` 265 Mo, `wasm.js` ; les chaînes « hôte démarré en pause » (harmonium_bevy) et « musique : profil » (`audio/music.rs`) sont dans le wasm. Le build est celui de la ligne cargo de `scripts/build-web-games.mjs` avec la variable neuve `WEB_FEATURES=game/harmonium` (ajoutée à `render` ; le script lui-même n'a pas été lancé : il commence par un `cargo run` de `alacod lint` en profil dev). **Non joué dans Chrome** : zombies ne déclare pas `music` (inerte), aucun geste ni écoute ; le comportement web du plugin (geste, Worker Blob, CSP `worker-src blob:`) est celui d'harmonium, vérifié par lui dans Chrome le 6 nov (`pret-pour-1837.md` § 1) |
| `cargo check -p game --features harmonium` en `wasm32-unknown-unknown` | Fait | `APP_VERSION=m7-check RUSTFLAGS=--cfg=web_sys_unstable_apis cargo check -p game --features harmonium --target wasm32-unknown-unknown` (`target_wasm`) : `Finished`, 0 erreur ; natif : `cargo check --profile headless -p game --features harmonium` : 0 erreur |
| Testbed natif avec la feature, `ALACOD_MUTE=1` : `HarmoniumStatus::Ready`, `MusicClock` valide et `playing`, statut externe qui suit (journal) | Fait (muet, aucune écoute) | `cargo build --profile headless -p testbed --features native,game/harmonium` (13 min 36 s), puis `ALACOD_MUTE=1 ALACOD_INPUT=neutral ALACOD_STATE_TRACE=/tmp/m7-smoke.trace ALACOD_EXIT_AT_FRAME=2700 target/headless/testbed --sandbox --local-port 7000 --players localhost` (fenêtre ouverte 45 s, sortie à la frame 2700, aucun processus laissé). Journal : `musique : profil « gmd_pop », pulsation 30/1 frames, volume 0` ; `Harmonium : ressources chargées (principale=true, percussion=true, presets=0)` ; `hôte démarré en pause` 2 s après le chargement ; `f0 Res(Ready) valid=true playing=true` (mesure 1, 135 BPM, tempo d'émotion avant l'horloge) ; `f1 … horloge_jeu=true statut_externe=Waiting` (horloge posée à l'entrée en partie) ; `f32 Starting sauts=1` ; `f235 Res(Ready) valid=true playing=true mesure=3 bpm=120.0 Tracking` ; ensuite `Tracking` jusqu'à la fin : écart filtré −1,5 ms (f536), +1,3 (f837), −0,2 (f1137), −0,3 (f1437), −0,2 (f1739), −0,4 (f2039), 0,0 (f2340), −1,0 ms (f2641) ; `nudges=0 sauts=2` ; tempo **exactement 120,0 BPM** (période 30 frames) dès la mesure 3 ; une mesure toutes les 2 s (mesure 23 à f2641 = 44 s). Aucun `ERROR` ni `panic` |
| `make check_rapide` (feature coupée par défaut), `make check_sim_boundary`, aucune trace modifiée | Fait | `make check_rapide` sur **`aac08438`** (tête de code poussée), code de sortie 0, journal `~/orca/runs/alpha/logs/m-7-harmonium-check-rapide-aac08438.log` (format, `check_forbidden`, `check_rollback_registration`, `check_sim_boundary`, `alacod lint` des cinq jeux dont le testbed avec son `music`, `test_crates` : 99 blocs `test result: ok`, aucun échec) ; un premier passage avec `WASM=1` a été arrêté à l'étape facultative `check_wasm` par la consigne disque du coordinateur (journal `…-aac08438-wasm-coupe.log` : tout ce qui précède est vert), le wasm32 est prouvé à part (lignes ci-dessus). Traces : `make test_scenarios SCENARIO=model_pantin,frappe_moment,repli_couvert_temoin,four_players_idle` **sans `BLESS`** : 5 tests verts, `git status` propre, aucune trace modifiée (aucun système de simulation, aucun type rollback, aucun crate `SIM` touché ; journal `…-scenarios.log`) |
| § 121, D217, ce rapport, LIVRÉ | Fait | `docs/conventions.md` § 121, `docs/taches/dettes.md` D217, ce rapport, ligne `LIVRÉ` de `docs/taches.md` |

Aucun crate `SIM` ne dépend d'harmonium (la dépendance est dans `game`, optionnelle ; `content` ne porte que des données).

## Contrôle rapide (journal `~/orca/runs/alpha/logs/m-7-harmonium-check-rapide-aac08438.log`)

Dernières lignes : `test result: ok. 0 passed; … finished in 0.00s` (derniers doctests), puis `✓ Contrôle rapide passé (suite de scénarios : la nuit)` et `EXIT 0` ; 99 blocs `test result`, tous `ok`.
Les commits suivants (rapport, `docs/taches.md`) ne touchent que des fichiers de documentation.

## Décisions

- **SF3 partout** : le § 3 du rapport d'harmonium donnait le SF2 en natif et le SF3 sur le web ; oxisynth (fork d'harmonium, feature `sf3`) lit le SF3 sur toutes les cibles et la release `studio-resources-2` ne publie que lui. Une seule banque, un seul chemin par défaut, 7,6 Mo au lieu de 32. Le testbed natif l'a joué (`Ready`, `Tracking`) ; jamais comparé au SF2 à l'oreille (D217).
- **La donnée du jeu est un champ du manifeste** (`music`, comme `entry.map` et `sandbox`), pas un fichier de plus ni un kind de plus : il n'y a qu'un réglage par jeu, lu à l'installation du plugin (le manifeste est inséré avant `CoreSetupPlugin` par tous les jeux). Banques en champs optionnels à défaut (`harmonium/…`).
- **Sans réglage, le plugin n'est pas installé** (pas de cpal ouvert, pas d'asset chargé), même avec la feature : un jeu qui compile avec `game/harmonium` sans `music` ne change pas.
- **Muet = volume 0, musique en marche** (pas `paused`) : l'horloge doit tourner pour prouver le suivi, et sur le web un `paused` au moment du geste empêcherait la reprise de l'`AudioContext`.
- **Horloge posée à `OnEnter(InGame)`, retirée à `OnExit(InGame)`** : `FrameCount` repart de 0 à chaque partie ; une horloge gardée entre deux parties aurait une frame de simulation inférieure à son ancrage.
- **Pas de `build-web-games.mjs` joué** : il lance d'abord `cargo run` de `alacod lint` en profil dev (un second build complet) ; la ligne cargo du script a été rejouée à la main avec la variable `WEB_FEATURES` qu'il lit désormais. Ligne `Makefile` `check_wasm` inchangée (feature coupée par défaut).
- Jeu web de la preuve : `zombies` (le seul jeu de `games/` dont la ligne web existe) ; le testbed n'a pas de build web. La musique en Chrome n'a donc pas été jouée (zombies n'a pas de `music`, pas d'écoute) ; c'est le premier geste de William sur 1837.

## Ce qui reste

- **D217** : réglages du suivi et latence de sortie web à l'oreille (William), `frame_fraction`, horloge reposée par salle, étape de CI/site pour les banques, `MasterVolume`.
- **1837** (hors engine, dans son dépôt) : déclarer `music: (profile: …, pulse_frames: (num: …, den: …))` dans son `game.ron`, poser les banques par `scripts/harmonium-banks.sh <jeu>` (ou son équivalent), construire le web avec `game/harmonium` (`alacod build --web --features render,game/harmonium`), vérifier la CSP `worker-src blob:` du site, changer le `rev` de l'engine après le merge.
- Écoute de William (geste humain, carte) : seule preuve que le tempo et le raccord se lisent bien (§ 6, ligne M-7).
- La suite complète de scénarios n'a pas été rejouée ici (la nuit) ; quatre scénarios l'ont été sans `BLESS`.

## Suivi (revue du merge `00fc97f1`, VPS, 2026-11-20)

Branche `wquintal/m-7-suivi` (depuis `origin/main` `36ab2386`), jalon servi **S2** (dim 22 nov). Présentation seule : aucun système de `GgrsSchedule`, aucun type rollback, aucune trace touchée.
Cible cargo partagée `target-partage`, profil `headless` seulement. Journaux dans `~/orca/runs/alpha/logs/`.

| Point | Issue | Preuve |
|---|---|---|
| **P2** tempo hors 40–300 BPM accepté au lancement | Fait | `prepare` refuse désormais un tempo à la noire hors 40–300 BPM **pour le mètre du profil** (`profile_tempo` : `BarTiming` du `meter` du profil + `ExternalClock::tempo_for`, le calcul de `LiveHost::external_tempo` ; dépendance `harmonium_core` à la même `rev`) : `warn!` qui cite période, tempo et mètre, pas de musique, pas de panique. Test `une_periode_hors_de_40_a_300_bpm_pour_le_profil_est_refusee` (bornes 12 et 90 frames en 4/4, 11 et 91 refusées, 20 frames acceptée en 4/4 et refusée au reel 2/2) : **rouge** sur `585f3697` (`m-7-suivi-p2-rouge.log` : « 301 BPM refusé »), **vert** sur `9f2e2df4` (`m-7-suivi-p2-p3-vert.log`, 5/5) |
| **P2** `alacod lint` | Fait (il ne le signalait pas) | Choix **B** du coordinateur : `content` ne dépend pas d'harmonium ; `lint_music` refuse la pulsation **impossible pour tout mètre**, hors 20–300 par minute (`3600 × den / num`, entiers). Fixtures `music_pulse_too_fast` (11/1), `music_pulse_too_slow` (181/1) rouges, `music_pulse_bounds_ok` (180/1) verte ; test `a_pulse_impossible_for_every_meter_is_reported` **rouge** avant le lint (aucune erreur `music`), **vert** après (3/3). Écart assumé entre les deux règles : § 121, D217 (9) |
| **P3** allocations par frame dans `log_music_status` | Fait | Clé `Copy` (`MusicLogKey` : statut, `valid`, `playing`, phase) comparée à chaque frame, ligne formatée seulement quand due. Mesure (`m-7-suivi-p3-mesure.log`, corps d'avant et d'après, allocateur compteur) : 299 frames sans changement, **avant 1 794 allocations (6,0 par frame) avec suivi, 897 (3,0) sans ; après 0**. Test permanent `le_journal_n_alloue_qu_au_changement` sous la garde temps réel d'harmonium (`rt_check::enter_audio_context`, allocateur global de `harmonium_host` actif en `debug_assertions`) : mutation jetable d'une allocation par frame → `SIGABRT` « REAL-TIME VIOLATION » (`m-7-suivi-p3-mutation-rouge.log`), vert sans |
| **Docs périmées** | Fait | `docs/plan-engine.md` l. 122 (harmonium porté, la musique suit `FrameCount`), l. 364 (musique calée : M-7), l. 697 (`harmonium_bevy` épinglé, plus « non porté ») ; **D151 fermée** (l'intégration décrite n'est plus la voie ; reste repris en D217 (10) : pulsation non liée à une horloge de run, fichiers `bevy_kira_audio` sans horloge) ; D217 (9) réécrit ; § 121 (refus au lancement, lint, journal) ; doc de `MusicEntry` |

Transcripts (cinq lignes par point) :

```text
P2 (game)  $ … cargo test --profile headless -p game --no-default-features --features harmonium --lib audio::music
  585f3697 : test audio::music::tests::une_periode_hors_de_40_a_300_bpm_pour_le_profil_est_refusee ... FAILED
             panicked at crates/game/src/audio/music.rs:206:72: 301 BPM refusé
  9f2e2df4 : test audio::music::tests::une_periode_hors_de_40_a_300_bpm_pour_le_profil_est_refusee ... ok
             test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 392 filtered out; finished in 0.31s
P2 (lint)  $ … cargo test --profile headless -p content --no-default-features --test m7_music_fixtures
  avant :    test a_pulse_impossible_for_every_meter_is_reported ... FAILED
             music_pulse_too_fast : [ LintError { kind: BrokenReference, … entry.start_map … } ]
  après :    test a_pulse_impossible_for_every_meter_is_reported ... ok
             test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
P3         $ rustc -O --edition 2021 mesure.rs -o mesure && ./mesure
  suivi Tracking : 299 frames sans changement : avant 0 ligne, 1794 allocations (6.0/frame) ; après 0 ligne, 0 allocations
  sans suivi : 299 frames sans changement : avant 0 ligne, 897 allocations (3.0/frame) ; après 0 ligne, 0 allocations
  mutation : panicked at …/harmonium_audio/src/realtime/rt_check.rs:53:9 … (signal: 6, SIGABRT: process abort signal)
  9f2e2df4 : test audio::music::tests::le_journal_n_alloue_qu_au_changement ... ok
Docs       $ grep -n "harmonium" docs/plan-engine.md
  122: | Musique adaptative, … | 🟡 `harmonium` porté (M-7, §121 …) : la musique **suit** l'horloge de la simulation (`FrameCount`, période du manifeste) …
  364: | Le Diable beau danseur … | … musique calée : 🟡 M-7 (§121, la musique suit `FrameCount` ; accord avec une horloge de run : D217) |
  697: - **Dépendances forkées ou épinglées** (`bevy_ecs_ldtk`, `matchbox` ; `harmonium_bevy`, dépôt du studio épinglé par
  dettes.md 172 : | D151 | **~~Musique calée sur l'horloge de simulation~~ (fermée le 2026-11-20 par M-7 …
```

**Contrôle rapide découpé** (`m-7-suivi-check-rapide-decoupe.sh`, `m-7-suivi-test-crates-decoupe.sh`, repris de T-3b, garde disque 15 Go) sur **`9f2e2df4`**, journal
`~/orca/runs/alpha/logs/m-7-suivi-check-rapide-9f2e2df4.log` : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_integration` code 0 (584 s, lint des cinq jeux dont le
testbed et son `music` 30/1, jeu court étiqueté et gestes 4/4) ; `test_crates` découpé code 0 (1 447 s) : « binaires joués : 57 / 56 tests d'intégration + lib/bins/doc ; failed : 0, codes non nuls : 0 » ; binaires de tests
purgés après. La feature `harmonium` n'est pas dans `test_crates` (coupée par défaut) : ses tests sont joués à part (ligne P2/P3). Aucune trace modifiée, aucun `BLESS`.

Non fait : pas de testbed natif rejoué avec la feature (refus au lancement prouvé par le test de `prepare`, le chemin `warn!` d'`add_music` est inchangé) ; pas de `cargo check` wasm32 (aucun code neuf propre au web :
`harmonium_core`/`harmonium_audio` sont déjà dans l'arbre wasm d'`harmonium_bevy`). `Cargo.lock` : deux lignes (dépendances de `game`), aucune version changée.
