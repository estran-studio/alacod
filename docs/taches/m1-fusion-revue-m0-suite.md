# m1-fusion-revue-m0-suite — fusionner `revue-m0-suite` (movement-feel + correctifs de la revue M0) dans `main`

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche `m1-fusion-revue-m0-suite` créée
depuis `origin/revue-m0-suite`). 1 à 2 j. Compilations : `CARGO_BUILD_JOBS=2`, `df -h /home` ≥ 20 Go avant chaque
compilation. Décision de William (2026-10-07) : on fusionne, puis il fait une grande session de test manuelle
enregistrée sur `zombies` et `throne` à partir de `main`.

## Contexte

`origin/revue-m0-suite` (`91d0f4e`, base `main` `26f1496`) contient :
- `6156d22` **movement-feel** : course nerveuse (3000/3000 px/s²), dash à i-frames (64 px en 8 frames, cooldown 24,
  6 i-frames, appui gardé 8 frames), caméra indépendante du framerate, profil `dev` optimisé (D30) ; `6d3fd8f` bless
  de 182 traces + 2 neuves. **Jamais joué par un humain.**
- **R4** (murs traversés après « Rejouer » : `main` a le même correctif, `f9f0d8e`), test `restart_keeps_walls_solid`,
  scénario `revue_murs_avant_poste`.
- **R5** (la touche R relançait la partie à tout moment, `game_over.rs`), test `r_ne_relance_que_sur_l_ecran_de_fin` :
  **absent de `main`**.
- Le carnet `docs/digests/revue-m0.md` (R1–R9, S1–S2), `docs/taches/REPRISE-revue-m0.md`, le rapport provisoire
  `docs/taches/rapports/fix-revue-m0.md`, `docs/taches/rapports/movement-feel.md`.

`main` a avancé de ~200 commits depuis `26f1496` (M1 entier : D41, D48, D51, D53, bots, graine 100…). Essai à blanc
de William : **~93 traces et 8 fichiers en conflit** (`crates/bots/src/decide.rs`, `crates/game/src/run_state.rs`,
`docs/conventions.md`, `docs/taches/dettes.md`, `tests/scenarios/{bot_floors_three,throne_floor_1,
throne_progression,throne_three_floors}.ron`) — plus à présent, `main` ayant encore bougé.

## Décisions (fixées ici)

1. **Fusion de `origin/main` dans la branche** (pas l'inverse), résolution par fichier :
   - `run_state.rs` : **la version de `main`** (R4 + restart p2p D14), en gardant le test `restart_keeps_walls_solid`
     et le scénario `revue_murs_avant_poste` de la branche, et **R5** (`game_over.rs`).
   - Code de simulation (`decide.rs`, mouvement, dash) : garder **les deux** comportements ; si movement-feel et un
     correctif de M1 touchent la même fonction, le dire dans le rapport avec la résolution.
   - `docs/conventions.md`, `dettes.md` : garder les deux (numérotation de `main`, §1–33 ; la section movement-feel
     prend le numéro suivant libre). Les dettes de la branche (D41 « movement-feel » si elle existe) **sont renumérotées**
     après D53 si leur numéro est déjà pris sur `main`.
   - Scénarios `.ron` en conflit : partir de la version de `main` et y réappliquer ce que movement-feel recalait
     (positions, frames), puis **remesurer les attentes** sur l'état fusionné (scénarios scriptés : preuve par inputs ;
     scénarios à bots : comme m1-v3-bots-softlocks).
   - **Traces** : ne pas résoudre à la main. Après fusion du code, **toutes** les traces se rebénissent sur l'état
     fusionné, avec preuve du §10 par **catégorie** : movement-feel change l'accélération et le dash dès la frame 0
     (`DashState`/`Acceleration`) ; montrer sur un scénario par jeu que la première ligne différente est celle du
     changement attendu, et lister les scénarios **qui ne bougent pas** (s'il y en a).
2. **Attentes M0 et M1 à tenir** : `clone_quad` (vague 5, 13 kills, quatre debout), `clone_solo`, `clone_duo` ;
   `throne_quad`, `throne_three_floors` (objectifs d'étage). Si une attente casse, remesurer avec preuve ; si un
   objectif n'est plus atteint (ex. `clone_quad` n'atteint plus la vague 5), **s'arrêter et le dire à orch**.
3. **Mesures** avant/après (base `main` actuel contre l'état fusionné), une sim à la fois :
   - `zombies` 20 graines `avant_poste`, 4 acheteurs, `--until-wave 5` ;
   - `throne` 20 graines, 2 et 4 bots, `--until-floor 3` ;
   - bench strict au calme (`bench_horde`, `bench_bullets`, `bench_cave`, `bots_four_mixed`) : demander la fenêtre à orch.
   Critère : aucun soft-lock ni desync ajouté ; une baisse nette de réussite des bots se signale (movement-feel change
   la façon dont ils bougent) avant de livrer.
4. **Rapport** : compléter `docs/taches/rapports/fix-revue-m0.md` avec une section « Fusion dans main » (conflits et
   résolutions, traces, mesures) ; mettre à jour `docs/digests/revue-m0.md` (R4/R5 « corrigé sur main ») ; ne pas
   toucher `docs/taches.md`.
5. **Hors périmètre** : R2, R3, R6–R9, S1–S2 (décisions de William pendant sa session de test), rééquilibrage.

## Livrer

Suite complète (bless sur l'état fusionné), crates, lint ×3, fmt, scripts, `make gen` ×3 (bless des générés si
movement-feel les touche, avec preuve), exemples, p2p ; `git push -u origin m1-fusion-revue-m0-suite` ; `SendMessage`
à `orch [f1df5b]` : `LIVRÉ m1-fusion-revue-m0-suite <sha> : <conflits, traces rebénies, mesures avant → après>`.
