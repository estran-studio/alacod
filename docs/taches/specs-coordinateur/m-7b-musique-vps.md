# M-7b « Musique qui se tait » — sur le VPS (coordinateur alacod, run `run_9effecfcae39`, ven 20 nov 2026)

Objectif studio O2 au service d'O3. Jalon **S2 dim 22 nov**. Présentation, audio temps réel. Tu ne merges pas.
Preuve qui fait foi : ligne **M-7b** du § 6 de `docs/taches/plan-1837-cli.md` (`4901e3b3`). § 124, D220, rapport `docs/taches/rapports/m-7b-musique-continue.md`.

## Le verdict (William, ven 20, 1837 `f2ac3bf` = engine `36ab2386`, natif sur son Mac, dans la salle)
« Animation ennemie est bonne » (fantassins validés). Musique : « fonctionne mais **stoppe un peu au début**, et **parfois ça l'a arrêtée pendant le bruit de
collision de balle** ». Deux défauts. Règle du 16 nov : **le test d'abord, la correction ensuite, aucune question de diagnostic à William**.

## Ce qu'on sait (à vérifier, pas à croire)
- Effets sonores : `crates/game/src/feedback.rs` (`play_path_with_fallback`, ~l. 1378) → `bevy_kira_audio` (`self.audio.play(asset_server.load(path))` à **chaque** son :
  premier chargement/décodage d'un fichier au premier tir ou à la première collision ?). Musique : feature `harmonium` (`crates/game/src/audio/music.rs`, harmonium épinglé
  `0189c6a`, `harmonium_bevy`/`harmonium_audio`/`harmonium_core`) : son propre flux de sortie ?
- **Piège** : `ALACOD_MUTE=1` coupe **tous** les effets (`feedback::muted()`) et met la musique à volume 0 : aucune preuve passée n'a jamais joué effets + musique ensemble.
- Pistes du studio, toutes à départager par l'observation : (a) voix/canaux ou périphérique de sortie partagés entre kira et harmonium (deux flux cpal sur le même
  périphérique, l'un qui prend la main) ; (b) sous-alimentation du tampon au chargement des banques (SF2/SF3) ou d'un son d'effet (chargement/décodage sur un fil qui
  affame le fil audio, ou verrou partagé) ; (c) horloge au premier ancrage (`GameClock`/`ExternalClock`, ancrée à `OnEnter(InGame)` : saut ou attente au début).
- **Si la cause est dans le dépôt harmonium** : n'y touche pas (lecture seule, `/home/debian/workspace/harmonium`) ; rapport précis + proposition, `status` au coordinateur ;
  corrige dans l'engine ce qui est de l'engine (ex. ordre de chargement, fil, réglage de tampon exposé par harmonium).

## Ce que tu livres, dans l'ordre
1. **Contrôle qui échoue d'abord** : un test sans carte son qui capte la **sortie audio réelle** (mixée ou par flux : musique et effets) et échoue quand la musique se tait
   ou saute (trou d'énergie, discontinuité, tampon vide/« underrun » signalé, horloge qui recule ou s'arrête) **pendant que des effets jouent** et **au démarrage**.
   Générique (testbed, noms neutres ; un réglage `music` neutre existe au manifeste du testbed). Journal rouge `logs/m-7b-controle-rouge-<sha>.log` sur le code actuel,
   vert après correction `logs/m-7b-controle-vert-<sha>.log`. Dans `check_rapide` si sa durée le permet (budget écrit), sinon la nuit (dis lequel et pourquoi).
2. **Cause établie** par la mesure (journal, compteurs d'underrun, temps du fil audio), `status` « cause musique : … » dès qu'elle l'est.
3. **Correction**, test rouge → vert ; aucune trace modifiée ; frontière simulation/présentation intacte (rien dans `GgrsSchedule`).
4. **Enregistrement audio de 30 s de la salle de 1837 avec tirs**, avant et après (fichier WAV ou équivalent, sans carte son) : copie jetable détachée de 1837
   (`git -C /home/debian/orca/1837 worktree add --detach /home/debian/orca/workspaces/1837/m-7b-jetable origin/main`, `[patch]` vers ton worktree, build **release** dans la cible
   partagée), `xvfb-run -a`, entrées qui tirent (script de scénario `ALACOD_INPUT=script:` ou remote). Fichiers sous `logs/m-7b-audio/` (`avant.wav`, `apres.wav`) + une
   image de la forme d'onde ou du spectrogramme (ffmpeg) qui montre les trous avant / leur absence après. Copie jetable retirée à la fin.
   Sortie sans carte son : ALSA est là (libasound, pas de PulseAudio/PipeWire). Préfère une configuration **par processus** (`ALSA_CONFIG_PATH` ou équivalent vers un fichier
   à toi, plugin `file`/`null` de libasound) plutôt qu'un `~/.asoundrc` global (d'autres processus du poste l'auraient). Deux flux (kira + harmonium) vers un même fichier
   ne se mélangent pas : capte-les séparément ou par un tap dans le processus. **Aucune installation de paquet système sans mon accord** (`ask` avec la commande exacte).

## Où, règles
- Worktree `/home/debian/orca/workspaces/alacod/m-7b-musique`, branche `wquintal/m-7b-musique` (depuis `main` `4901e3b3`). Cible partagée chaude :
  `CARGO_TARGET_DIR=/home/debian/orca/workspaces/alacod/target-partage` sur toutes tes commandes (aucun autre dossier cible ; pas de wasm sauf nécessité, `status` avant).
- Préfixe `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0` (et **pas** `ALACOD_MUTE=1` pour les mesures audio, puisque le défaut ne se voit qu'avec les effets ; ailleurs, oui).
  Un seul cargo ; jamais de profil debug ; cargo seulement si `df -h /` ≥ 20 Go libres, arrêt de ton cargo sous 15 Go ; `scripts/target-sweep.sh $CARGO_TARGET_DIR`
  après chaque cargo ; rien entre 03:45 et 05:00 UTC.
- Contrôle rapide **découpé** (décision du coordinateur, 20 nov) : `~/orca/runs/alpha/logs/p-1d-check-rapide-decoupe.sh` et `p-1d-test-crates-decoupe.sh` sous ton nom
  (journal `logs/m-7b-musique-check-rapide-<sha>.log`), puis purge des binaires de tests d'intégration. Si ton test exige la feature `harmonium`, dis comment il tourne en CI/nuit.
- Lis `CLAUDE.md`, `~/orca/runs/alpha/commun.md` (écrit pour le Mac ; ce fichier-ci prime), `docs/taches/rapports/m-7-harmonium.md` (§ 121, D217, § Suivi).

## Communication
Commite et pousse à chaque étape. Carte `orca worktree set --worktree active --comment …`. `status` à chaque étape (cause, rouge, vert, E prêt) ; relève ta boîte après chaque
build ou test. 429 : `status` tout de suite. Question bloquante : `ask` avec ton choix par défaut, et avance. Fin : `status` « E prêt <sha> », rapport, ligne LIVRÉ,
aucun processus restant, un seul `worker_done`.
