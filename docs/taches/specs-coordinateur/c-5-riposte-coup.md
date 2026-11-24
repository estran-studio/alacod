# Brief C-5 — riposte au coup (suite du verdict de William du 23 nov, point 1)

Objectif studio : O2 au service d'O3 ; jalon **D2 dim 29 nov**. Lot de correction, ordre validé par William le 23 nov (C-3 → **C-5** → C-6 → U-1 ? → C-4 → X-6e). Simulation : `anthropic/claude-sonnet-5-5`, `--agent omp`. Français partout.

## Le défaut
William : « Les ennemis attaqués de loin ne font rien, comme le rat ; il devrait être aggro vers moi. » Relevé (`~/orca/runs/alacod/releve-arme-aggro-2026-11-23.md`, B) :
- `enemy_target_selection` (`crates/game/src/character/enemy/ai/behavior.rs:29-263`) ne choisit que par distance (`aggro_range`) et vue (`sight_range`, D66) ; un coup ne change jamais `EnemyTarget` ;
- `DamageAccumulator.last_hit_by` (`health/mod.rs:58-62`) vit une frame ; `DamageEvent { source, target }` (`sim_core/src/damage.rs:75-76`) est lu par `ranged_cycle_system` (`ai/cycle.rs`) pour la stupeur seulement ;
- `Hearing` (`ai/rules.rs:196-240`) réagit à la naissance d'une balle près du tireur, pas au coup.

## La correction attendue
1. **Test d'abord** : scénarios testbed neutres rouges sur le code sans la règle, puis verts : 1 joueur (tir hors aggro → l'ennemi poursuit le tireur : `EnemyState`, `EnemyDistance` qui diminue), 2 joueurs (l'ennemi vise le **tireur**, pas le plus proche), fin de provocation (retour au point d'apparition avec `ReturnHome`, ou perte de cible), **témoin** sans le champ (rien ne bouge).
2. **Opt-in** : champ d'IA (ex. `provocation: Some(frames)`), absent par défaut ; `BehaviorRuntime` (sous checksum) mémorise l'attaquant (`GgrsNetId`, échéance) depuis `FrameEvents<DamageEvent>` ; **Hash et `Debug` manuels**, champ haché seulement s'il est actif (règle « ajouter un champ à un composant rollback » du `CLAUDE.md`, motif `return_active()`) ; tie-break `net_id` si plusieurs coups la même frame ; `enemy_target_selection` cible l'attaquant pendant la durée, prioritaire sur le plus proche ; la garde de `ReturnHome` est levée pendant la provocation. **Aucune trace existante ne bouge** (aucun BLESS).
3. Déterminisme : `Fixed`, `order_iter!`, `BTreeMap`, pas de `f32`, logs par `GgrsNetId`, `FrameEvents` (jamais `Message`).
4. Docs : § 133 de `docs/conventions.md` (C-4 prendra le suivant libre), ligne `CLAUDE.md` (une ligne), `describe` à jour ; dettes à partir de **D241**.

## Critère d'acceptation côté 1837 (lecture seule, ne pas copier)
Dépôt `estran-studio/1837`, branche `wquintal/v-e-arme-aggro` @ `70805d1` : `tests/scenarios/aggro_au_coup.ron`, `aggro_au_coup_deux.ron` (rat réel touché à > 428,57 u, attendu `EnemyState Chase` vers le tireur ; sorties `docs/preuves/v-e/rouge-aggro.txt`). 1837 ajoutera `Chase` et `sight_range` à son rat au réépinglage (le rat n'a pas de règle pour avancer à distance).

## Preuve (avant le merge)
- Tableau rouge → vert ; familles `zigzag_*`, `retour_*`, `rodeur_*`, `repli_couvert_*`, `tireur_*`, `enemy_*`, `decor_solide_*` sans trace modifiée (la preuve est le `git diff --stat` : aucune `.trace` existante).
- `check_sim_boundary` ; `make schedule_order_diff BASE=725e4767` si un système est ajouté.
- Capture de 20 à 60 s (`play_scenario --capture` accepté, le dire ; un ennemi touché de loin qui se retourne et poursuit).
- Contrôle rapide découpé (`specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`), journal `~/orca/runs/alacod/logs/c-5-check-rapide-<sha>.log`.
- Rapport `docs/taches/rapports/c-5-riposte-coup.md`, trois lignes en tête.

## Machine
- Tout cargo : `nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 CARGO_TARGET_DIR=$HOME/orca/targets/alacod-engine` (cible **vide** : build à froid), jamais de profil debug.
- **Feu du coordinateur avant tout cargo** (demande par `ask`, termine ton tour) ; code et scénarios d'abord sans cargo.
- Disque : arrêt et message sous 31 Go ; `~/orca/targets/1837` intouchable ; `scripts/target-sweep.sh` après le contrôle.
- **Pousse ta branche** (`git push origin HEAD:wquintal/c-5-riposte-coup`) avant `worker_done` ; tu ne merges pas.
