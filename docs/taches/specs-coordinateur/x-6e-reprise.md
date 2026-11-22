# X-6e — Chef en deux phases, REPRISE (brief du worker, 22 nov 2026)

Tu es le worker du lot **X-6e** de l'engine alacod (lot (4) de l'ordre validé par William). Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais). Français partout.

## Ce que tu reprends
- Branche `wquintal/x-6e-chef` @ `dae6c685` (base `8778a267`), écrite sur le VPS le 20-21 nov, **lot non fini**. Lis d'abord sa note `docs/taches/rapports/passation-x-6e-chef.md` : état, pièges,
  régressions provoquées à journaliser, traces existantes à prouver, ce qui reste. **Reprise, pas réécriture.**
- Fiche et preuve : `docs/taches/plan-1837-cli.md` § 6, ligne d'origine **X-6e Chef en deux phases** (contenu) et ligne **X-6e reprise du 22 nov** (fusion, preuve exacte). Lis-les sur `origin/main`.
- Correction à la note : **X-6d n'est pas mergé** (garé). `main` a reçu depuis ta base : X-6f (§ 128), X-6g (§ 129), X-6h (§ 130), C-1, **C-2** (§ 131, poses `charge_windup`/`charge`/`return_home`/`take_cover`
  dans `Model`, `decide_poses` lit `BehaviorRuntime`).

## À faire
1. **Inventaire sans cargo** : note de passation, `git log`, `git diff origin/main...HEAD --stat`. Puis `git merge origin/main` (jamais rebase, jamais `--force`) ; résous les conflits
   (`behaviors/src/lib.rs`, `BehaviorRuntime`, `rules.rs`, `pathing.rs`, `model3d/{playback,pose}.rs`, `content/src/models.rs`, `p1d_model_poses_fixtures.rs`, `models.ron`, `runner.rs`, `rooms.rs`…).
   Les poses de la ruée du chef : par les champs de C-2 s'ils s'appliquent, aucun second mécanisme. Commite la fusion, pousse, `status` « X-6e fusion » (conflits et choix).
2. Au **feu** du coordinateur : compilation, 9 `chef_*` (dont `chef_coop` à deux joueurs), tests unitaires (`behaviors`, `content` fixtures `model_poses_chef_*`, `game` `patterns::tests`, `model3d`), `alacod lint` du testbed,
   `make gen GAME=testbed` (seuls `generated/testbed/weapon_chef_cone|chef_paire.{ron,trace}` ajoutés), traces existantes de la fiche **sans trace modifiée** (tableau avant/après ; BLESS seulement justifié d'une ligne),
   régressions provoquées (tableau bloc retiré → attente rouge), `check_sim_boundary`, `schedule_order`.
3. Capture de 20 à 60 s du chef en deux phases (`play_scenario --capture`, scénarios `chef_*` enchaînés ; un build release seulement sur feu du coordinateur ; supprime ensuite les artefacts render de la cible,
   le contrôle rapide ne compile jamais `render`) ; vidéos dans `docs/taches/rapports/preuves/x-6e/`.
4. Docs : rapport `docs/taches/rapports/x-6e-chef-deux-phases.md` (trois lignes « existait / manquait / ajouté » en tête), conventions § 127, D223 (et D234+ si besoin) dans `docs/taches/dettes.md`,
   lignes `CLAUDE.md` (attentes `EnemyWeapon`, `EnemyState none`), ligne `LIVRÉ wquintal/x-6e-chef : …` à la fin de `docs/taches.md`.
5. Contrôle rapide **découpé** sur feu du coordinateur, puis « X-6e prêt <sha> ».

## Machine : le MacBook personnel de William (règles dures ; il joue sur ce Mac)
- Tout cargo/make : `nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 CARGO_TARGET_DIR=/Users/wq/orca/targets/alacod-engine …` ; jamais en arrière-plan ; jamais de profil debug
  (`--profile headless`, le Makefile le fait) ; `--release` seulement pour une capture sur feu. **Un seul cargo lourd sur toute la machine** (avec le coordinateur 1837) : **aucun cargo avant « feu X-6e »** ;
  avant le contrôle rapide et tout build release, redemande le feu. Les tests ciblés après le premier build : libres en nice 19 tant que tu as le feu.
- **Disque : ~29 Go, sous le plancher de 30** (le studio pose la question de fond à William). `df -h /System/Volumes/Data` avant chaque gros build ; `scripts/target-sweep.sh /Users/wq/orca/targets/alacod-engine`
  entre deux commandes cargo (jamais pendant) ; **arrêt dur sous 25 Go** et `ask`. Ne supprime rien hors de la cible `alacod-engine` ; `~/orca/targets/1837` est la cible de William : intouchable.
- Contrôle rapide : jamais `make check_rapide` d'un bloc. Depuis ton worktree :
  `nice -n 19 env CARGO_TARGET_DIR=/Users/wq/orca/targets/alacod-engine LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh` ;
  journal copié en `$HOME/orca/runs/alacod/logs/x-6e-check-rapide-<sha>.log`. Attention au budget du jeu court (`tags: ["rapide"]`, 4 800 frames, déjà à 4 615) : n'étiquette pas tes scénarios `rapide`.
- Suite complète jamais sur le Mac (la nuit du VPS la joue après le merge). Pas de `target/` dans ton worktree.
- `env -u GITHUB_TOKEN git push …` si la variable existe. Ne touche jamais `/Users/wq/Project/bascanada/alacod` ni `/Users/wq/Project/bascanada/1837` (lecture par `git show`). `1867_lore` : lecture seule.
- `pkill`/`killall` par motif interdits ; aucun processus laissé à la fin. Si tu modifies `games/testbed/assets/game.ron` ou `camera.ron` pour la capture, rétablis-les sans les commiter.

## Communication
Branche `wquintal/x-6e-chef` (avance rapide). `status` à chaque étape ; question bloquante : `ask` court avec ton choix par défaut. Prêt : `status` « X-6e prêt <sha> » avec le journal ;
fin : rapport commité, **un seul** `worker_done`.
