# C-5 — Riposte au coup

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

## Limites
D241 (ligne droite hors champ de flux, événement avant résolution des dégâts). Capture : voir message de fin.
