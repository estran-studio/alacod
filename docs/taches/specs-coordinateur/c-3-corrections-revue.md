# C-3 : corrections de revue (verdict : merge après corrections, `specs-coordinateur/revue-c-3-verdict.md`)

Même branche `wquintal/c-3-collision-props`, même règles machine (nice 19, 4 jobs, cible `~/orca/targets/alacod-engine` **vide** : build à froid ; **aucun cargo avant le feu du coordinateur**, à demander par `ask`).

1. **P2-1** : `crates/game/src/character/enemy/ai/cycle.rs` (~310-331, recul et `ReturnHome`) teste le pas contre `SolidBody` comme `pathing.rs:826-836` ; scénario neuf `decor_solide_retour` (ennemi qui rentre à son point de départ derrière une caisse solide : il la contourne ou s'arrête, jamais à travers), rouge (clause retirée) puis vert ; à deux joueurs si le scénario le permet sans le compliquer, sinon le dire.
2. Même famille, si le coût est faible : `recovery_direction` (`pathing.rs:431-465`) écarte aussi les cases d'un `SolidBody`.
3. **P3-2** : scénario `decor_solide_dash` (dash à `dash_distance 48` contre une caisse solide : bloqué).
4. **P3-1, P3-3** : § 132 et D240 corrigés (le dash est bloqué ; reste éventuel ; une caisse solide coupe la vue à `sight_range` ; `Phasing` bloqué, voulu, à écrire).
5. **P3-4, P3-5** : rapport : rouge joué avec des attentes intermédiaires (le dire) ; la preuve « aucune trace existante modifiée » est le `git diff --stat`.
6. Rejouer : les scénarios `decor_solide_*` et les familles touchées par `cycle.rs` (`zigzag_*`, `retour_*`, `rodeur_*`, `repli_couvert_*`, `tireur_*`, `pose_*`) **sans trace existante modifiée**, puis le contrôle rapide découpé ; pas de nouvelle capture (sauf si le code de rendu change).
Fin : branche poussée, `worker_done` avec SHA et journaux.
