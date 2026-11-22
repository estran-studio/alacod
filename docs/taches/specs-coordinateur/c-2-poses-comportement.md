# C-2 — Poses des phases de comportement (brief du worker, 22 nov 2026)

Tu es le worker du lot de correction **C-2** de l'engine alacod. Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais). Français partout.

## Pourquoi
Demande datée D2 (dim 29 nov) du coordinateur 1837 : au rev `dc734091`, `alacod describe Model` n'expose aucune pose pour les phases de comportement de X-6b (annonce de `Charge` / `Telegraph`,
charge en zigzag, `ReturnHome`, § 119) ni de X-6c (`TakeCover`, § 120) ; `crates/game/src/model3d/playback.rs` (`decide_poses`) ne lit ni `BehaviorRuntime` ni la phase de charge. Le rat de 1837 a ses
clips (grognement, recul, demi-tours) mais ne peut pas les raccorder : il est livré en idle/run, sans le signal avant la charge ni le recul. Règle de William du 16 nov : une animation qui ne joue pas doit être
attrapée par nos tests, **test d'abord, correction ensuite**. Fiche : `docs/taches/plan-1837-cli.md` § 6, ligne **C-2** (contenu et preuve écrits avant ton départ).

## À faire (générique, noms neutres, rien de 1837)
1. **Relevé** (fichier:ligne) : `decide_poses` / `ModelPlayback` (T-3, § 122-123), `animation_vivante`, lint `MissingPose`, phases de `Charge`, `ReturnHome`, `TakeCover` et où elles vivent (`BehaviorRuntime`…).
2. **Test d'abord** : scénario(s) neutre(s) du testbed avec un modèle neutre (`pantin` ou un modèle du testbed) dont le comportement passe par annonce de charge → charge → retour, et un repli à couvert ;
   attentes `Clip`/`ClipProgress` exigeant le clip de chaque phase ; rouge avant le code (échec d'attente ou de compilation : dis lequel).
3. **Correction** : champs de pose **optionnels** de `Model` pour ces phases (noms neutres, ex. `charge_windup`, `charge`, `return_home`, `take_cover`), choisis **hors rendu** dans `ModelPlayback` à partir de l'état
   de simulation (lecture seule) ; champ absent = pose actuelle (rien ne change pour les modèles existants, testbed et 1837 compris) ; `animation_vivante` couvre ces phases quand le champ est posé ; `describe` et
   `new-content` les montrent ; lint : **avertissement** (jamais une erreur, pour ne pas casser le contenu existant) quand un modèle d'un personnage dont les règles contiennent ces phases n'a pas de pose dédiée ;
   pose inconnue du manifeste = erreur comme aujourd'hui.
4. Aucun changement de la simulation : traces d'état inchangées (la présentation ne s'y lit pas).

## Preuve
Rouge puis vert ; scénarios neutres à **un et deux joueurs** (deux cibles), traces neuves seulement ; `rodeur_*`, `zigzag_*`, `repli_couvert_*`, `tireur_pantin_*`, `model_pantin` sans trace modifiée ;
capture de 20 à 60 s : `play_scenario --capture` suffit si le coordinateur ne donne pas de feu vert pour un build release (le dire) ; contrôle rapide découpé vert. Conventions § **131**, dettes **D231–D232**,
rapport `docs/taches/rapports/c-2-poses-comportement.md` (trois lignes « existait / manquait / ajouté » en tête), ligne `LIVRÉ wquintal/c-2-poses-comportement : …` à la fin de `docs/taches.md`.

## Machine : le MacBook personnel de William (règles dures)
- 8 cœurs, 16 Go de RAM partagés avec sa partie. Un seul `cargo` à la fois, jamais en arrière-plan. Préfixe : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`.
- **Jamais de profil debug** : `--profile headless` pour tout test/vérification (le Makefile le fait) ; `--release` seulement pour la capture.
- Disque : ~45 Go libres, **plancher 30**. `df -h /System/Volumes/Data` avant tout gros build ; sous 40 Go, `scripts/target-sweep.sh target` ; sous 30 Go, `ask`.
  La capture release et le contrôle rapide l'un après l'autre, jamais ensemble.
- Contrôle rapide : jamais `make check_rapide` d'un bloc. Depuis ton worktree :
  `LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, puis `scripts/target-sweep.sh target` ;
  journal copié en `$HOME/orca/runs/alacod/logs/x-6g-check-rapide-<sha>.log`.
- Suite complète jamais sur le Mac (la nuit du VPS la joue après le merge).
- Si `GITHUB_TOKEN` est dans ton environnement : `env -u GITHUB_TOKEN git push …`.
- Ne touche jamais `/Users/wq/Project/bascanada/alacod` ni `/Users/wq/Project/bascanada/1837` (lecture par `git show` seulement). `1867_lore` : lecture seule.
- `pkill`/`killall` par motif interdits ; aucun processus laissé à la fin. Si tu modifies `games/testbed/assets/game.ron` ou `camera.ron` pour la capture, rétablis-les sans les commiter.

## Règle machine (William joue sur ce Mac)
Tout cargo/make : `nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 …` ; **un seul cargo lourd sur toute la machine** (avec le coordinateur 1837) : avant ton premier build complet,
avant le contrôle rapide et avant tout build release, `status` au coordinateur et attends son feu vert. Les tests ciblés (`-p game`, un scénario) après le premier build : libres, en nice 19.

## Communication
Branche `wquintal/c-2-poses-comportement` (avance rapide, jamais `--force`). `status` à chaque étape ; question bloquante : `ask` court avec ton choix par défaut. Prêt : `status` « C-2 prêt <sha> » avec le journal ;
fin : rapport commité, **un seul** `worker_done`.
