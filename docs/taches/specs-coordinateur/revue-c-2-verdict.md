# Revue C-2 — poses des phases de comportement (tête `64868f00`, contre `origin/main`)

**Verdict : merge après corrections** (deux P2 documentaires, aucun P1).

## 1. Défaut inchangé — OK
- `decide_poses` ajoute `Option<&BehaviorRuntime>` à la query (optionnelle, aucun filtre) ; `input.behavior = runtime.and_then(behavior_phase)`.
- `pose::choose` (pose.rs, bloc « C-2 ») ne retourne que si `looping_gesture(.., phase.gesture(poses), ..)` rend `Some`, donc seulement si le champ de pose est posé ; champ absent → `g.as_ref()?` rend `None` → on retombe sur trigger/aim/reload/gait comme avant. Aucun chemin où la lecture change le choix champ absent. `Poses::merged` : `or_else` sur les quatre champs, `None` si les deux absents.
- `sim_activity` : ne retourne la phase que si un `Poses` (modèle ou arme) déclare le geste ; sinon inchangé.
- Traces : `git diff --name-status` ne montre que 4 `.trace` **ajoutés** (`pose_*`), aucun modifié ni supprimé.

## 2. Correctif `ClipSpeed` (`model_expect.rs`, `clip_speed`) — juste
Avant : toute avance négative = +durée (un clip à l'envers donnait +46). Après : avance ramenée dans ]−d/2, d/2]. Pour une lecture avant sans tour, avance ∈ [0, d/2] : identique. Tour de boucle avant (avance < −d/2) : identique. Seule la plage « avance négative petite » change, qui n'était lue juste par aucun scénario.
Attentes `ClipSpeed` existantes : `tireur_pantin_cycle` (f15 : 0,99–1,01 ; f81 : ≤ 0,01 ; f100 : 1,15–1,25), `gestes_entree_reelle.rs:238,256,260` (1,15–1,25 ; 0,95–1,05) : toutes des avances avant < d/2 (ou nulles) → **verdict inchangé**. Seules les attentes neuves de `pose_charge_retour` (f75 `max: -0.3`) dépendent de la correction.
Risque résiduel : un clip très court dont l'avance par frame dépasse d/2 serait mal lu (aliasing) ; non atteint ici. [INFERENCE : lecture du code, pas rejoué.]

## 3. Frontière simulation/présentation — OK
- `behavior_phase` : `&BehaviorRuntime` en lecture, appelé depuis `decide_poses` (présentation, `model3d`) et `sim_activity` (lecture `World`). Aucun champ ni type rollback ajouté ; aucun enregistrement rollback touché ; aucune écriture de simulation.
- `content` : ajoute seulement des champs `serde` à `Poses` et un lint dans `reach.rs` ; dépendances bevy ECS seul, rien de présentation importé.

## 4. Conformité au brief — OK avec réserves
- Priorité : dash > coup reçu > mêlée > tir (`since_shot`) > **phase de behavior** > épaulement/visée > rechargement > allure. Attaque, coup reçu et mort passent donc avant la phase ; la phase l'emporte sur visée/rechargement (voulu : le repli remplace le rechargement, prouvé par `pose_repli_couvert` f150).
- Lint : `LintWarningKind::MissingBehaviorPose`, via `warn(...)` seulement ; pose inconnue du manifeste = `MissingPose` erreur inchangée ; fixtures `model_behavior_poses_{complete,missing,unknown}` testées (`p1d_model_poses_fixtures.rs`). `describe Model` : rapporté, non rejoué (aucun cargo).
- Scénarios : chaque phase exige son clip nommé (`aim` annonce, `strike-walk`/`-fr` ruée, `aim-walk` recul + `ClipSpeed ≤ −0,3` + `ModelYaw`, `idle` à l'attente, `strike-walk-l` puis `hit` pour le repli, `reload` à la reprise), à 1 et 2 joueurs. Ce ne sont pas des « non idle » génériques : `idle`/`reload` d'origine ne satisfont aucune attente de phase. Rouge crédible (idle/walk/reload à la place).
- Réserve rouge : le code était déjà compilé, le rouge = `models.ron` sans les champs, pas un test écrit avant le code. Acceptable (le test de pose unitaire et les attentes échouent bien sans données), journal `/tmp/c2-rouge.log` non conservé.

## 5. Neutralité et preuves
- Aucun nom de contenu 1837 dans l'engine ; testbed seul (`charge_pantin`, `couvert_pantin`). Seuls les fixtures de lint s'appellent `rat.ron` (animal générique).
- Vidéos : 4 mp4 dans `preuves/c-2/` (8 s chacune, 30 i/s) ; image extraite de `pose_charge_retour` à 2,4 s : arène du testbed, « Charge pantin » incliné en posture de ruée, vague 0 — cohérent. (Non vérifiées image par image.) Durée 8 s < 20–60 s du brief, déjà acceptée par le coordinateur selon le rapport.

## P1
Aucun.

## P2
1. **`docs/taches/dettes.md` ne contient pas D231 ni D232** (dernier : D229 ; D230 absent aussi). Elles ne sont que dans `docs/conventions.md:9356`. Correction : ajouter les deux lignes D231 (cap du corps pendant l'annonce = `FacingDirection`, simulation) et D232 (`new-content` ne scaffolde pas `Model`) au tableau, même format que D229.
2. **`docs/taches.md:1803` : `LIVRÉ … : (à remplir à la fin)`** et `rapports/c-2-poses-comportement.md` finit sur « Contrôle rapide : voir la fin » sans suite ; le rapport a toujours son titre « squelette ». Correction : remplir la ligne LIVRÉ (comme C-1/X-6h) et le résultat du contrôle rapide découpé (sha, journal) avant merge.

## P3
1. `crates/game/src/model3d/playback.rs` (`sim_activity`, bloc C-2) : le geste est cherché dans `model.poses` **ou n'importe quelle arme**, alors que `choose` utilise les poses fusionnées de l'arme active. Et `return_home` du testbed n'a que `moving` : à l'arrêt pendant `Retreat`, `choose` retombe sur idle mais `sim_activity` annonce « recul » (invariant permissif, jamais faux-rouge). Correction : utiliser les poses fusionnées de l'arme active, et exiger `still` ou `moving` selon la vitesse ; ou le noter en dette.
2. `reach.rs` (~l.700) : `has` considère toutes les armes du modèle ; un modèle qui ne pose la phase que sur une arme non portée n'est pas averti. Avertissement seulement : tolérable.
3. Pas d'attente de cap avant la ruée (D231, assumée et documentée).
