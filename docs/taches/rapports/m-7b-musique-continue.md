# M-7b — La musique ne se tait plus (au démarrage, sous les bruits de balles)

Jalon servi : **S2, dimanche 22 novembre 2026** (`plan-1837-cli.md` § 6, ligne M-7b : « la musique ne se tait plus au démarrage ni sous les bruits de balles »,
verdict de William du 20 nov sur 1837 `f2ac3bf` = engine `36ab2386`). Branche `wquintal/m-7b-musique` (depuis `main` `4901e3b3`), VPS, cible `target-partage`.
**Présentation seule** : `crates/game/src/audio/music.rs` (feature `game/harmonium`) ; rien dans `GgrsSchedule`, aucun type rollback, aucune trace touchée.
harmonium n'est pas modifié (lecture seule) : ce qui relève de lui est au § 5. Journaux : `~/orca/runs/alpha/logs/` (noté `logs/`). Conventions § 124, dette D220.

## 1. Bilan

| Point du lot | Issue | Preuve |
|---|---|---|
| Contrôle sans carte son qui échoue quand la musique se tait ou saute, au démarrage et pendant les à-coups qui accompagnent les effets | Fait | `audio::music::continuite::la_musique_ne_se_tait_pas_au_demarrage_ni_sous_les_a_coups`, `make test_music` ; **rouge** sur `0fbc6ff1` (`logs/m-7b-controle-rouge-0fbc6ff1.log` : silence 1 973 ms au démarrage, 181 ms après une frame de 450 ms), **vert** sur `69a90d3d` et `4e1b2f3c` (`logs/m-7b-controle-vert-69a90d3d.log`, `logs/m-7b-controle-vert-4e1b2f3c.log`) ; mutations (§ 3) |
| Cause établie par la mesure | Fait | § 2 ; `status` « cause musique » envoyé avant la correction |
| Lien avec « le bruit de collision » (demande du coordinateur) | Fait : la première lecture d'un son ne fait pas de frame longue | `logs/m-7b-mesure-premier-son.log` (§ 2.3) |
| Correction dans l'engine | Fait | `gate_music` (pause **et** volume 0 tant qu'aucune horloge n'est posée), `follow_config` (micro-ajustement 5 % sur 16 pulsations) |
| Enregistrements de 30 s de la salle de 1837 avec tirs, avant / après | Fait, avec une limite du VPS | `logs/m-7b-audio/` : jeu réel sur le VPS (`vps-avant.wav`, `vps-apres.wav`, `.png`) et banc à 60 Hz avec la musique de 1837 et ses sons de tir (`avant.wav`, `apres.wav`, `.png`) (§ 4) |
| Contrôle rapide découpé | Fait, vert | `logs/m-7b-musique-check-rapide-d9c1727d.log` (§ 6) |
| Aucune trace modifiée, frontière intacte | Fait | seul `crates/game/src/audio/music.rs` change dans le code du jeu (feature coupée par défaut) ; aucun système de `GgrsSchedule` |

## 2. Cause

### 2.1 Mesure sans carte son (`scripts/audio-capture.sh`, `tools/alsa-pacer/pacer.c`)

Le VPS n'a pas de carte son (pas de `/dev/snd`, noyau cloud sans `snd-aloop`) ; le plugin `null` d'ALSA ne cadence pas (5 s de son écrites en 0,16 s). Le plugin
« ioplug » écrit pour ce lot cadence chaque flux sur l'horloge murale (un flux en retard voit un vrai underrun), écrit un WAV **par flux** et note underruns,
marge du tampon et plus long écart entre deux écritures. Chargé par processus (`ALSA_CONFIG_PATH`), aucun paquet installé (gcc et `libasound2-dev` étaient là).

1837 release (engine `4901e3b3` en `[patch]`, copie jetable `528d8ed`), salle, un tir toutes les 100 frames (`logs/m-7b-audio/tirs-salle.ron`, `ALACOD_INPUT=script:`), Xvfb :

| Flux | Underruns | Marge minimale | Plus long écart entre écritures |
|---|---|---|---|
| kira (effets) | 0 | 1 698 frames | 49,3 ms |
| kira (second gestionnaire) | 0 | 1 482 frames | 53,6 ms |
| harmonium (musique) | 0 | 2 801 frames | 41,6 ms |

Pistes (a) voix/périphérique partagés et (b) tampon affamé : **écartées** (flux séparés, aucun underrun). Les trous de la musique (`scripts/audio-gaps.py`) suivent
exactement l'état du suivi de l'horloge publié dans le journal (`musique f… statut_externe=…`, `logs/m-7b-audio/avant1-jeu.log`) : piste **(c)**.

1. **Au démarrage** : l'hôte d'harmonium démarre dès ses banques chargées (`Ready`, f0) et joue au tempo des émotions (170,9 BPM) ; à `OnEnter(InGame)` (f14),
   `set_external_clock` met le playback **en attente** (Hold) jusqu'à 30 observations stables, la garde de départ et le début d'une mesure de l'horloge.
   Mesuré : musique de 4,05 s à 4,53 s, puis silence. C'est « ça stoppe un peu au début ».
2. **En partie** : le suivi (défauts d'harmonium : zone morte 10 ms, 5 ‰ sur 8 pulsations) n'absorbe qu'environ **20 ms** d'écart simulation/mur à ce tempo ;
   au-delà il saute à la fin de la mesure en cours vers une mesure de l'horloge, et l'attente est **silencieuse** jusqu'à une mesure. Mesuré : trou de 890 ms à
   29,45 s, au saut n° 2 (`Starting`, `valid=false`).

Un tel écart naît d'une frame de plus de 250 ms (`Time<Virtual>` borné : la simulation perd le reste, `bevy_ggrs` ne le rattrape pas) ou du ralenti `run_slow`
de `bevy_ggrs` en ligne (frames 10 % plus longues quand un pair est en avance).

### 2.2 Ce que fait le VPS

Le VPS rend 1837 à 2-5 images par seconde (lavapipe, frames de 200 à 650 ms, `logs/m-7b-audio/avant2-temps-de-frame.log`) : la simulation perd du temps à presque
chaque frame. Le suivi n'y quitte presque jamais l'attente (23 s d'attente avant le premier départ dans `avant1`, jamais dans les captures du soir, VPS chargé
par un second worker). Cela grossit le défaut, sans le créer : sur le Mac de William (60 images par seconde) il faut une frame longue isolée ou le ralenti.

### 2.3 Le lien avec les collisions (demande du coordinateur)

Mesure (`logs/m-7b-mesure-premier-son.log`, App bevy + `AudioPlugin` de kira + sons de 1837 sur la carte virtuelle, test jetable retiré) : à chaque lecture
(tir, impact, rechargement, impact, mort, impact), update du fil principal ≤ 0,95 ms, `play` 0,06 à 0,20 ms, son chargé et décodé en moins d'une frame
(`Assets<AudioSource>` 0 → 1). **Pas de frame longue au premier son** : un préchargement ne corrigerait rien (non fait). kira relit et redécode un son à chaque
lecture (la poignée meurt après le play ; D220 (4)), sans à-coup. Le lien entendu : le saut tombe **à la fin de la mesure qui suit l'écart** (jusqu'à ~0,9 s
plus tard à 193 BPM), donc en plein échange de tirs (un tir toutes les ~1,7 s dans la salle).

## 3. Correction et contrôle

- **`gate_music`** (`Update`, après `feed_music_clock`, avant `HarmoniumSet`) : sans `GameClock`, `HarmoniumControl.paused = true` **et** `volume = 0` ; avec, le volume
  d'origine (`MusicVolume`, 0 si `ALACOD_MUTE=1`). Les deux, mesuré : sur ALSA, la pause demandée par l'hôte juste après l'ouverture du flux échoue en silence
  (`snd_pcm_pause` avant `RUNNING`, erreur ignorée par cpal) — avec la pause seule (build de `e1a0b954`), la musique jouait encore 1,5 s avant l'ancrage
  (`logs/m-7b-audio/pause-seule-*` : « début à 7,24 s, trou à 8,74 s ») ; le volume seul laisse passer la rampe de 10 ms du gain initial (1) du playback. Web : l'`AudioContext` reprend au premier
  geste après l'ancrage. Hors partie, plus de musique (lobby, après `OnExit(InGame)`).
- **`follow_config`** (`HarmoniumConfig::follow`) : micro-ajustement **5 %** (plafond du playback, `MAX_NUDGE_PPM`) sur **16** pulsations, le reste des réglages
  d'harmonium inchangé : ~400 ms rattrapés au tempo au lieu d'un saut silencieux.
- **Contrôle** (`audio::music::continuite`, `make test_music`) : la vraie sortie d'harmonium (hôte détaché, chaque tampon rendu par `PlaybackEngine::process_buffer`
  à son instant), le jeu (`FrameCount` par l'accumulateur de `bevy_ggrs`, delta borné à 250 ms ; une frame Bevy avec `HarmoniumPlugin` et les systèmes de la
  musique) sur une horloge murale simulée, réglage `music` neutre du testbed. À-coups : frames de 300 et 450 ms, 1 s de `run_slow`. Joué deux fois : pause qui
  arrête le flux (CoreAudio, web) et pause sans effet (ALSA). Échoue sur un silence ou une attente ≥ 150 ms après la première note, une mesure qui recule ou en
  saute une, une musique qui part plus de 3 s après l'entrée en partie, ou (pause efficace) un son avant l'ancrage. Banques : énergie jugée ; sans elles
  (CI) : attente et mesures seulement (vert aussi, `logs/m-7b-controle-vert-e1a0b954-sans-banques.log`). 5 s de jeu au calme, 26 s sur le VPS chargé.
- **Mutations** (`logs/m-7b-mutations-69a90d3d.log`) : sans le volume 0 → rouge (ALSA : silence 1 973 ms au démarrage) ; sans la pause → rouge (un tampon avant
  l'ancrage malgré la pause) ; suivi par défaut → rouge (silence 213 ms / 181 ms après la frame de 450 ms). Chaque correction est nécessaire.
- **Où il tourne** : la nuit (`make test_music`), pas `check_rapide` : la feature compile un second binaire de tests de `game` (4 à 14 min selon la charge du VPS,
  1 Go) pour quelques secondes de jeu. À ajouter à `runs/nuit/nuit.sh` (une ligne, fichier du coordinateur). Les banques sont dans le cache du VPS.

Transcript (cinq lignes) :

```text
$ make test_music   (0fbc6ff1, banc à un mode)  continuité : silence de 0.55 s à 2.53 s (1973 ms) ; silence de 10.59 s à 10.77 s (181 ms) → FAILED
$ make test_music   (4e1b2f3c)                  [pause qui arrête le flux] première note à 2.53 s (entrée en partie à 0.53 s)
                                                [pause sans effet (ALSA)] 1 tampon isolé avant l'ancrage (rampe du gain initial) ; première note à 2.53 s
                                                continuité : incidents : 0 ; suivi : resyncs 1 (le départ), nudges 2 → ok
$ banc, musique de 1837 (gigue_quebecoise 3600/129), 32 s : avant 14 incidents (trous 906, 885, 191 ms ; saut de la mesure 3 à 5) ; après 0
```

## 4. Enregistrements (`logs/m-7b-audio/`)

- **Jeu réel, VPS** : `vps-avant.wav`, `vps-apres.wav` (30 s, musique + effets mixés à partir des flux séparés, alignés par la fin du processus), flux seuls
  `vps-*-musique.wav` / `vps-*-effets.wav`, images `vps-avant.png` / `vps-apres.png` (bleu : musique, orange : effets). Avant : 0,9 s de musique au tempo des
  émotions, puis silence ; après : un seul tampon (la rampe d'ALSA), puis l'attente. Dans les deux cas le suivi ne part pas : le VPS rend 2 à 5 images par
  seconde (§ 2.2). Journaux `vps-*-jeu.log`, `vps-*-flux.log` (0 underrun).
- **Banc à 60 Hz, musique de 1837** (`ALACOD_CONTINUITE_MUSIQUE=gigue_quebecoise:3600/129 ALACOD_CONTINUITE_SECONDES=32 ALACOD_CONTINUITE_WAV=…`) : la vraie sortie
  d'harmonium avec le profil et la pulsation de 1837, les mêmes à-coups que le contrôle (frames de 300 et 450 ms à 4 et 8 s, `run_slow` de 11 à 12 s), mixée avec
  les sons de 1837 (`shot_shotgun`, `reload` aux tirs du script, `hit` aux à-coups) : `avant.wav` / `avant.png` (code d'avant : trous de 900, 890 et 200 ms,
  le début qui s'arrête puis repart deux fois), `apres.wav` / `apres.png` (aucun trou). C'est la vue représentative du Mac ; elle n'est pas le jeu.

## 5. Ce qui relève d'harmonium (lecture seule, proposition au studio)

1. **Saut silencieux** (`harmonium_host/src/external.rs`, `ExternalFollower::plan_jump`) : le saut vise la première mesure de l'horloge qui commence après la fin de
   la mesure en cours ; quand l'audio est un peu **en retard**, l'attente vaut presque une mesure (890 ms mesurés). Proposition : sous une demi-mesure d'écart,
   entrer dans la mesure visée à la bonne position (couper sans attendre) ou rattraper au tempo ; garder l'attente silencieuse pour les grands écarts.
2. **Politique par défaut** (`harmonium_core::time_plan::correction::CorrectionPolicy`, 5 ‰ sur 8 pulsations) : ~20 ms absorbés, alors que le playback accepte 5 %.
   Proposition : un défaut proche du plafond, ou un réglage exprimé en millisecondes absorbées.
3. **Gain initial à 1** (`playback.rs`, `master_volume`/`master_gain` à 1.0) : une commande de volume 0 à l'initialisation fuit 10 ms de rampe. Proposition :
   gain initial = premier volume commandé, sans rampe.
4. **Pause initiale sur ALSA** (`LiveHost::start_paused`) : `stream.pause()` juste après `play()` échoue (pcm pas encore `RUNNING`, erreur ignorée par cpal 0.15) ;
   le flux joue. Proposition : ne pas jouer le flux à la création, ou vérifier l'état. Option voisine : un `HarmoniumConfig` « attendre l'horloge » qui garde
   l'hôte muet tant qu'aucune `GameClock` n'est posée (ce que fait `gate_music` côté engine).
5. **Suivi sur une machine lente** : à 2-5 images par seconde (frames > 250 ms), le filtre (60 observations) et le seuil de discontinuité (100 ms, 3 de suite)
   remettent le départ à zéro sans fin : aucune musique. Un départ « au mieux » (jouer même mal calé) serait préférable au silence.

## 6. Contrôle rapide découpé

Feu vert du coordinateur (20 nov, 20:00 UTC), `logs/m-7b-check-rapide-decoupe.sh` et `logs/m-7b-test-crates-decoupe.sh` (repris de M-7 suivi, garde 15 Go),
cible `target-partage`, sur **`d9c1727d`** (tête de code ; les commits suivants sont des docs), journal `logs/m-7b-musique-check-rapide-d9c1727d.log` :
`make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_integration` code 0 (684 s) ; `test_crates` découpé code 0 (1 709 s) :
« binaires joués : 57 / 56 tests d'intégration + lib/bins/doc ; failed : 0, codes non nuls : 0 » ; total 2 395 s ; binaires de tests purgés, `target-partage/release` retiré,
sweep (15 Go), `/` à 42 Go. Un premier passage sur `2b399790` s'est arrêté à `test_tools` (`logs/m-7b-musique-check-rapide-2b399790-echec-forbidden.log`) :
`Instant::now()` du banc manquait à `scripts/determinism-allowlist.json` ; exception justifiée ajoutée (`d9c1727d`). `make test_music` (feature `harmonium`,
hors `check_rapide`) vert sur `4e1b2f3c` (`logs/m-7b-controle-vert-4e1b2f3c.log`, 6 tests) ; `d9c1727d` ne change que l'allowlist.

## 7. Ce qui reste

- **Écoute de William** sur le Mac (seule preuve à l'oreille) : la musique doit partir une à deux mesures après l'entrée en partie et ne plus s'arrêter ; un
  rattrapage à 5 % s'entend comme un tempo un peu plus vif ou plus lent pendant quelques secondes (D217 (1)).
- `make test_music` dans la nuit (`runs/nuit/nuit.sh`, coordinateur).
- Harmonium (§ 5, D220) ; une frame de plus de ~650 ms fait encore sauter la musique (saut d'harmonium).
- 1837 : changer le `rev` de l'engine après le merge (coordinateur 1837) ; rien à changer dans ses RON.
