# C-5 — Riposte au coup

**Prêt pour la partie de William** (pas livré avant son verdict). Code testé : `3da46efe` (branche `wquintal/c-5-riposte-coup`).
- Jouer : `alacod sandbox testbed` n'a pas le scénario ; le plus direct est `make play_scenario SCENARIO=riposte_un_joueur` (puis `riposte_deux_joueurs`, témoin `riposte_temoin`, retour `riposte_fin`).
- Captures (VPS, rendu hors écran sous Xvfb, ralenties à 8 images/s : 22,5 s et 23,75 s) : `docs/taches/rapports/preuves/c-5/riposte_un_joueur.mp4`, `riposte_deux_joueurs.mp4`.

- **Existait** : `enemy_target_selection` ne choisissait que par `aggro_range` et `sight_range` ; un coup ne changeait jamais `EnemyTarget` (`DamageAccumulator.last_hit_by` vit une frame, `Hearing` réagit à la naissance d'une balle).
- **Manquait** : un champ de donnée qui fasse d'un joueur qui blesse l'ennemi sa cible, et sa preuve (aucun scénario ne tirait de loin sur un ennemi hors aggro).
- **Ajouté** : `ai.provocation: Some(frames)` (§ 133), `BehaviorRuntime::provoked` (haché seulement posé), lecture de `FrameEvents<DamageEvent>`, garde `ReturnHome` levée, marche droite sur l'attaquant si le champ de flux en désigne un autre ; 4 scénarios `riposte_*` ; D241.

## Tableau rouge → vert (journaux `~/orca/runs/alacod/logs/c-5-rouge.log`, `c-5-vert.log`)

| scénario | rouge (lecture des coups neutralisée) | vert |
|---|---|---|
| `riposte_un_joueur` | distance 216 > 180 (f130), aucun behavior retenu | poursuite, `Chase` |
| `riposte_deux_joueurs` | x = 640 > 590 (f140), distance 216 > 135 (f170) | vise le tireur (le plus loin), reste sur y = 696 |
| `riposte_fin` | x = 640 > 600 (f150), aucun behavior | poursuite puis retour au point d'apparition |
| `riposte_temoin` (sans le champ) | vert | vert (ne bouge pas) |

Premier rouge non valable : `ReturnHome` sans `Melee` ni `Charge` est une erreur de lint (ajout de `Melee("zombie_claws")`). Un tir à f20 ne touchait pas (aucune provocation, même avec la règle) : tirs à f70, cause non élucidée [INFERENCE : arme pas prête avant].

## Preuve
- Familles `zigzag_*`, `retour_*`, `rodeur_*`, `repli_couvert_*`, `tireur_*`, `enemy_*`, `decor_solide_*` : 41 scénarios verts, aucune `.trace` existante modifiée (`git diff --stat` : seules quatre `.trace` neuves `riposte_*`, `c-5-familles.log`).
- `check_sim_boundary` : OK (`c-5-boundary.log`). Aucun système ajouté : `schedule_order_diff` non joué.
- Test unitaire `provocation_is_hashed_only_when_set_and_expires`.

## Corrections de revue (verdict `revue-c-5-verdict.md`)
- **P2** : le test fige désormais le `Debug` exact (`BehaviorRuntime { selected: None, since_frame: 0, charge: Idle, charge_ready_at: 0, charge_hit: None, wander_dir: (0, 0), wander_next: 0, heard: None }`) et le hash (`12140863795936750328`) d'un `BehaviorRuntime::default()`, relevés sur origin/main au rev `725e4767` (worktree jetable VPS, test temporaire, `c-5-base-hash.log`) ; vert sur la branche (`c-5-p2.log`).
- **P3.2** : § 133 dit qu'un coup reçu pendant le sommeil de salle (`RoomDormant`) est perdu. `alacod describe` : `provocation` est dans `AiSchema` (`registry.rs`), dont le schéma est dérivé (schemars) ; non rejoué par la commande elle-même.
- **P3.1** (scénario « mur entre eux ») : non fait, D241 reste la limite.
- **Tir à f20** : non élucidé ; le journal `projectile … hits` n'a pas été lu (aucun cargo supplémentaire après la suspension du disque). Les tirs à f70 prouvent le comportement.

## Contrôles (VPS, x86-64, traces arm64 inchangées)
- 10 scénarios (`riposte_un_joueur/deux_joueurs/fin/temoin`, `zigzag_charge_retour`, `retour_apres_morsure`, `repli_couvert_temoin`, `decor_solide_ennemi`, `rodeur_garde_neutre`, `tireur_pantin_cycle`) verts, `git status` propre : `c-5-scenarios.log`.
- Contrôle rapide découpé, `3da46efe` : étapes `make` (format, test_tools, check_forbidden, check_rollback_registration, check_sim_boundary, lint) vertes dans `check-rapide-decoupe-3da46efe.log` (coupé à 22 Go libres dans `check_integration`) ; reprise : `check_integration` code 0 (87 s), puis `test_crates` découpé 61 binaires, 0 échec, 1185 tests lib/bins : `c-5-check-rapide-3da46efe.log`. Tous dans `~/orca/runs/alacod/logs/` du VPS.
- Capture : `c-5-capture.log` ; `play_scenario --capture --every 1` (feature `render`, `DISPLAY` Xvfb sans `xauth`, lavapipe), `ffmpeg -framerate 8`.

## Limites
D241 (ligne droite hors champ de flux, événement avant résolution des dégâts). Capture : voir message de fin.
