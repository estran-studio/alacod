# C-1 — Journal du spawner trop bavard (brief du worker, 21 nov 2026, nuit)

Tu es le worker du lot de correction **C-1** de l'engine alacod. Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais). Français partout.

## Pourquoi
Verdict de William (21 nov, nuit) sur 1837 `476e9a2` (engine `8778a267`) : « … beaucoup de lag, il y a aussi extrêmement de logs qu'il faudrait sûrement trimmer ». Le coordinateur 1837 a mesuré
2 859 lignes `INFO game::character::enemy::spawning` en 55 s de partie locale (`git -C /Users/wq/Project/bascanada/1837 show <branche>:docs/preuves/porte-mac/logs/7-jeu-natif.log`,
demande-lui la branche si besoin) : un `spawner_skip` par frame, même sans spawner. Lot de correction : il passe **avant** la fin du lot X-6h en cours (REGLES § 6). Fiche : `docs/taches/plan-1837-cli.md`
§ 6, ligne **C-1** (contenu et preuve écrits avant ton départ).

## À faire
1. **Test d'abord** : partie normale du testbed (et une carte sans spawner) jouée N frames (headless, runner de scénarios ou `App` de test), journal capturé au niveau par défaut du jeu ; le test
   échoue si une ligne INFO ou WARN se répète à chaque frame (même cible et même message, aux frames près). Rouge sur `main` (journal gardé), vert après.
2. **Correction** : `crates/game/src/character/enemy/spawning.rs` (`spawner_skip` : `no_players_in_room` l. ~96, `max_enemies` l. ~115, `no_valid_spawner` l. ~243) en `trace!` en gardant le format
   `ggrs{f=… spawner_skip …}` (`CLAUDE.md` § 10), ou journalisé à la transition seulement ; tout autre `info!`/`warn!` par frame que le test trouve, corrigé de même. **Aucun changement de logique** :
   ni ordre, ni RNG, ni état ; les traces d'état ne bougent pas.
3. Chiffre : nombre de lignes INFO avant/après sur 60 s de partie du testbed (headless suffit ; natif release seulement si le coordinateur donne le feu vert).

## Preuve
Test rouge puis vert ; chiffre avant/après ; scénarios des familles touchées (zombies à spawners : `make test_scenarios SCENARIO=…` quelques-uns, testbed) sans trace modifiée ; dire à quel `RUST_LOG`
`make diff_log` doit tourner pour garder les lignes `spawner_skip` ; contrôle rapide **découpé** vert sur ta tête poussée. Rapport court `docs/taches/rapports/c-1-journal-spawner.md`, dette D230 si un reste,
ligne `LIVRÉ wquintal/c-1-journal-spawner : …` à la fin de `docs/taches.md`.

## Machine : règle immédiate (William joue sur ce Mac)
- **Tout cargo/make** : `nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 CARGO_TARGET_DIR=/Users/wq/orca/workspaces/alacod/x-6h-camera-portes-combat/target …`
  (cible **partagée** avec le worker X-6h : économise le disque et le verrou de cargo empêche deux builds en même temps). Jamais de profil debug (`--profile headless`), un seul cargo à la fois.
- Contrôle rapide découpé : `nice -n 19 env CARGO_TARGET_DIR=<la même> LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh` ;
  **avant de le lancer, `status` au coordinateur et attends son feu vert**. Journal copié en `~/orca/runs/alacod/logs/c-1-check-rapide-<sha>.log`.
- Disque ~38 Go, plancher 30 : `df -h /System/Volumes/Data` avant tout gros build ; sous 30, `ask`. Pas de `target-sweep` sur la cible partagée sans accord du coordinateur.
- Si `GITHUB_TOKEN` est dans ton environnement : `env -u GITHUB_TOKEN git push …`. Ne touche jamais `/Users/wq/Project/bascanada/alacod` ni `/Users/wq/Project/bascanada/1837`. `pkill` par motif interdit.

## Communication
Branche `wquintal/c-1-journal-spawner` (avance rapide, jamais `--force`). `status` à chaque étape ; question bloquante : `ask` court avec ton choix par défaut. Prêt : `status` « C-1 prêt <sha> » avec
le chemin du journal ; fin : rapport commité, **un seul** `worker_done`.
