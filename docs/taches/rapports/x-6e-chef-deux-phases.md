# X-6e — Chef en deux phases (reprise du 22 nov)

| Existait | Manquait | Ajouté |
|---|---|---|
| Phases de boss (§ 37), ruée `Charge` (§ 119), cycle de tir posté et `moments` (§ 116), escouades (`squad`), apparitions, poses de phase de comportement (C-2, § 131) | Un chef qui ordonne des salves à son groupe, une ouverture après salve, une transition et une arme par phase, une décharge calée sur le clip, un témoin d'arme portée | `Behavior::Command` et `Opening`, `Phase::transition`/`drop_weapon`, `SquadOrders`, `timed_on_departs`, poses `order`/`transition`, attentes `EnemyWeapon` et `EnemyState none`, neuf scénarios `chef_*`, contenu testbed |

**État : squelette écrit avant le feu (22 nov), preuves à compléter.** Conventions : § 127. Dette : D223.

## Relevé de ce que la branche ajoute
Voir `docs/conventions.md` § 127 et la note `passation-x-6e-chef.md`. Fusion de `origin/main` (X-6f/g/h, C-1, C-2) : `351e0ae2` ; conflits dans `models.rs`, `p1d_model_poses_fixtures.rs`, `pose.rs`, `playback.rs`, `models.ron` (les deux côtés gardés ; `BehaviorRuntime` lu par le tuple `Staged`).

## Preuves (à remplir)
- 9 scénarios `chef_*` : _
- Tests unitaires (`behaviors`, `content` `model_poses_chef_*`, `game` `patterns::tests`, `model3d`) : _
- `alacod lint` du testbed, `make gen GAME=testbed` : _
- Traces existantes avant/après (aucune modifiée) : _
- Régressions provoquées (bloc retiré → attente rouge) : `squad_orders.may_start` → `chef_salve_ordre` ; `drop_weapon` → `chef_phase_arme` ; `transition` → `chef_phase_arme` ; `Opening` → `chef_ouverture_melee` ; timeline en phase 0 → `chef_sans_renfort` ; départs du clip → `chef_ruee_decharge` : _
- `check_sim_boundary`, `schedule_order_diff` : _
- Capture : `docs/taches/rapports/preuves/x-6e/` : _
- Contrôle rapide découpé : _
