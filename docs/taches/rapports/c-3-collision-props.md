# C-3 — Un décor solide bloque les corps

- **Existait** : prop (`prop: true`, § 112) sur `enemy_layer` : séparation douce (ennemis), ralentissement (joueur), ignoré du champ de flux ; `cover` (§ 118) pour les balles seules.
- **Manquait** : un champ de donnée rendant le collider d'un prop dur pour les corps, et sa preuve (aucun test ne posait un corps contre un prop).
- **Ajouté** : `solid: true` (§ 132), composant statique `SolidBody`, lecture par `move_characters`, `move_enemies`, `rebuild_blocked_cells` ; lint ; 6 scénarios `decor_solide_*`.

## Preuve
Journaux dans `~/orca/runs/alacod/logs/` (`c-3-*.log`).

| Scénario (testbed) | Rouge (blocage retiré : blocs `move_characters`, `check_wall_collision`, `rebuild_blocked_cells` ôtés, `c-3-rouge.log`) | Vert |
|---|---|---|
| `decor_solide_joueur` | traverse : x 479,5 (f40), 546,5 (f69) | butte à x 459,8 (bord 470 − demi-corps) ; glisse au nord (466,9 ; 721,5 à f90) puis repart (512,1 ; 764 à f159) |
| `decor_solide_deux_joueurs` | se croisent : x 546,5 / 365,5 | x 436 / 476, chacun de son côté |
| `decor_solide_destructible` | traverse | butte à 459,8 ; `Cover` 0 à f84 ; passe ensuite (592,3 à f144) |
| `decor_solide_ennemi` | [INFÉRENCE] passage en ligne droite (non rejoué avec les attentes finales : le premier rouge ne les violait pas) | s'écarte de la ligne (y 672 f30, 632 f110), x 490-505 à y 625-640 à f110, rejoint le joueur (distance ≤ 40 à f299) |
| `decor_solide_joueur_temoin`, `decor_solide_ennemi_temoin` (`prop-test`, sans `solid`) | = vert | rien ne bloque (joueur 479,5 / 546,5 ; ennemi en ligne droite y 696) |

- **Traces** : 35 scénarios des familles `prop_*`, `couvert_*`, `repli_couvert_*`, `zigzag_*`, `wisp_respawn_prop` et les six `decor_solide_*` : verts, **aucune trace existante modifiée**, aucun BLESS (seules les six traces neuves sont écrites) : `c-3-familles.log`.
- `cargo test -p content` (lint `solid`, fixture `solid_requires_prop`) : 163 + autres, verts (`c-3-content.log`). `make check_sim_boundary` : OK (`c-3-boundary.log`). `schedule_order_diff` : non lancé (aucun système ni type rollback ajouté ; `SolidBody` est hors rollback).
- **Contrôle rapide découpé** @ `b9d9d957` : 61 binaires, 0 échec, 0 code non nul : `c-3-check-rapide-b9d9d957.log`.
- **Capture** : `~/orca/runs/alacod/captures/c-3-decor-solide.mp4` (30 s, quatre scénarios enchaînés au ralenti ×2,5). C'est `play_scenario --capture` (scénarios rendus hors écran), **pas une partie interactive**. Limite : les props du testbed n'ont pas de sprite (`skins` vides), la caisse est donc invisible : on voit le joueur buter et glisser dans le vide, les deux joueurs arrêtés face à face et l'ennemi qui s'écarte. Un décor visible relève du contenu du jeu (1837).
- Preuve côté 1837 (`collision_decor*`) : au réépinglage, hors de ce lot.
