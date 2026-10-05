SHA de tête vérifiée : voir la ligne `LIVRÉ` (rapport **provisoire**, session interrompue le
2026-10-05 pour reprendre sur l'autre Mac de William ; reprise : `docs/taches/REPRISE-revue-m0.md`).
Fiche : brief de revue humaine M0 (T3.3, `docs/taches/T3.3-revue-humaine.md`), MacBook de William.
Branches : `fix-revue-m0` (correctifs de la revue, base `main` `1a39283`) et `revue-m0-suite`
(`movement-feel` + `fix-revue-m0`, base `26f1496`) — c'est `revue-m0-suite` qui se reprend.
Date : 2026-10-04/05 ; agent : Claude Code (Opus 5.5).

## Fait

- **Mise en route macOS** (notes R1–R3, `docs/digests/revue-m0.md`) : `cargo build -p zombies
  --profile headless` compile sur macOS arm64 (1 min 18 s, dépendances en cache) ;
  `check-forbidden.sh` et `check-rollback-registration.sh` passent sous bash 3.2.57 (D33 bien
  fermée) ; `make record_session` compile en `dev` (R2) ; `ALACOD_HEADLESS=1` sur un binaire
  avec `render` panique sans message utile (R3).
- **R4 — murs traversés après « Rejouer »** (`crates/game/src/run_state.rs`) :
  `CollisionGrids` remise à zéro dans `cleanup_rollback_world_system`. Cause : la grille des
  murs n'est reconstruite que si la signature (nombre, somme des `GgrsNetId`) change ; la
  relance recrée les mêmes murs avec les mêmes ids. **`main` a fait le même correctif en
  parallèle** (`f9f0d8e`, D14) : à la fusion, garder la version de `main`. Ce qui reste propre
  à cette branche : le test `restart_keeps_walls_solid` (`crates/scenario/tests/run.rs`) et le
  scénario `revue_murs_avant_poste` (carte `avant_poste`, graine 123456 du jeu, joueur contre
  chaque mur).
- **R5 — la touche R (rechargement) relançait la partie** (`crates/game/src/ui/game_over.rs`) :
  `button_system` tourne pendant tout `InGame` ; R ne relance plus que si `GameOverUiRoot`
  existe. Test `r_ne_relance_que_sur_l_ecran_de_fin`. **Pas corrigé sur `main`** (`ddb9789`).
- **Carnet de revue** `docs/digests/revue-m0.md` : R1–R9, S1–S2.
- **`movement-feel`** (autre session, voir `docs/taches/rapports/movement-feel.md`) commité avec
  le feu vert de William : `6156d22` (code) et `6d3fd8f` (bless de 182 traces + 2 neuves),
  puis fusionné avec `fix-revue-m0` dans `revue-m0-suite` ; `revue_murs_avant_poste` rebénie
  sur l'état fusionné (`8ac2c0b`, même cause que les 182 : frame 0, `DashState`/`Acceleration` ;
  les 4 attentes de position passent sans retouche).

## Vérifié (chiffres réels)

Sur `fix-revue-m0` (`4593fef`, Mac) :
- `make test_scenarios` : `test result: ok`, 129 scénarios, aucune trace modifiée, 442,6 s.
- Avant correctif, `restart_keeps_walls_solid` échoue : joueur en (282.0, 763.97) à f400 dans
  la première partie, (−200.83, 1080.49) après relance. Après : 4/4 tests de `run.rs` verts.
- `r_ne_relance_que_sur_l_ecran_de_fin` : échoue sans le correctif (`Some(Restart)` au lieu de
  `None`), passe avec.
- `cargo fmt --all -- --check` : propre.

Sur `revue-m0-suite` (`8ac2c0b`) :
- `revue_murs_avant_poste` : attentes vertes avant et après bless ; `run.rs` 4/4 ;
  `r_ne_relance_que_sur_l_ecran_de_fin` vert.
- Suite complète (`make test_scenarios`, Mac, sur `8ac2c0b`) : `test result: ok`, **185
  scénarios** joués (184 de movement-feel + `revue_murs_avant_poste`), aucune trace différente,
  543,3 s.

## Non fait / non vérifié

- Tests des crates (README §4, deuxième commande), `make lint`, `make gen`, `make check` :
  **non lancés** dans cette session.
- p2p à deux clients : non fait (prévu ce soir avec William, puis interrompu).
- Partie à deux, réanimation, fin de partie en ligne : non joués.
- R7 (enregistrement non écrit) : cause non établie.
- Fusion de `origin/main` (`ddb9789`, 163 commits de plus que `26f1496`) : **non faite**.

## Dettes, questions ouvertes

Voir `docs/digests/revue-m0.md` : R2, R3, R6 (touches), R7, R8 (graine fixe : décision de
William), R9 (LCG, bits faibles), S1/S2 (sensations à préciser).
