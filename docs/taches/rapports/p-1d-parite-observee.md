# P-1d — Parité observée : visée au curseur et cycle des fantassins

Jalon servi : **S1, dim 15 nov** (plan `docs/taches/plan-1837-cli.md` § 6, ligne P-1d et « Changement d'ordre du 16 nov »). Branche `wquintal/p-1d-parite-observee`.
Conventions : § 122 de `docs/conventions.md` ; dette D218. Journaux et captures : `~/orca/runs/alpha/logs/` (VPS, `/home/debian/orca/runs/alpha/logs/`).

## 1. Tranche 1 — visée au curseur : livrée

Mergée dans `main` `85c86745` (puis `00fc97f1` avec M-7) ; 1837 `9e7c6df` l'embarque. Verdict de William (lun 16, sur 1837 `9e7c6df`) : « mon personnage a les animations
correctes et complètes, mais pas les ennemis ». Le Frère n'a pas changé depuis (`Fades::default()`, cap exact, esquive et armes inchangées).

## 2. Tranche des fantassins

### Cause (observée au vrai jeu, pas supposée)

1837 `9e7c6df`, engine `85c86745`, release, Xvfb + lavapipe, `bevy_remote` frame par frame (`alacod/step`) : les quatre fantassins de la salle jouent **un seul nœud
d'animation** (`fusil-idle`, boucle de 2 s) pendant tout le cycle que la simulation exécute (épauler, cône, tir, rechargement), sauf une dizaine de frames de marche au recul et
au retour. Relevé : `p-1d-captures/observation-jeu.md` (section `jeu-avant-85c86745`). `animate_models` ne remplissait `PoseInput` (détente, tir, rechargement) qu'à partir
d'un `WeaponInventory` ; un fantassin n'en a pas (`Shoot` + `Emitter` + `RangedAttackState` + `ranged_cycle`). `model3d` est inchangé entre `9e937158` et `85c86745` : pas de
régression, les fantassins de la salle n'ont jamais animé (les captures de P-1/R-1 montraient un fantassin isolé). Aucun test ne pouvait le voir : la pose n'existait qu'au rendu.

### Correction (engine, générique)

- `game::model3d::playback` (`ModelPlaybackPlugin`, aussi en headless) : `decide_poses` décide hors rendu le clip, sa position, le fondu, le cap, l'objet en main, et les range
  dans le composant **`game::model3d::playback::ModelPlayback`** (nom et module stables pour T-3b ; `clip()`, `time()`, `duration()`, `looping()`, `held`, `moving`,
  `unresolved`, `yaw`) ; `animate_models` ne fait plus qu'appliquer. `alacod/state` expose `anim` (clip joué) de chaque joueur et ennemi à modèle.
- Tireur posté (`model3d::shooter::read`) : épaulé = attente avant le télégraphe, en joue = télégraphe, tir = étapes suivantes, rechargement = `cooldown_frames` (clip de 2 s
  joué une fois, étiré sur 3 s : banc `armes.js` A:695-696), coup reçu = `ranged_cycle.stagger_frames` (pose `hit`). Le geste du tir tient sa dernière image jusqu'à la fin de
  la séquence (`held`) ; la frame où la séquence finit compte déjà pour le rechargement (`shooter::reload_end`). Les poses d'un tireur sont celles de l'arme de sa règle `Shoot`
  par-dessus celles du modèle, comme pour l'arme active d'un joueur.
- Invariant **`animation_vivante`** du runner (`crates/scenario/src/invariants.rs`, actif par défaut, désactivable par scénario) et lint **`MissingPose`** (`alacod lint`) :
  détail au § 122. Les deux tournent dans `alacod test` (le runner lint le contenu puis joue chaque scénario avec les invariants).

### Preuves

| Point | Issue | Preuve |
|---|---|---|
| Compilation de `7f7ad107`…`5d7a1aec` | fait | `cd7ff778` (seule erreur : `FixedTransform3D::default`) ; tests `model3d`, `models`, `p1d`, `animation_invariant` verts |
| `animation_vivante` rouge sur 1837 `9e7c6df` | fait | `logs/p-1d-animation-vivante-rouge-9e7c6df.log` : lecture du tir désactivée (`if false`), les 4 fantassins « séquence de tir » / « rechargement » sur `fusil-idle` |
| `animation_vivante` vert avec la correction | fait | `logs/p-1d-animation-vivante-vert-335969b8.log` : `shooter_poses` (2 tests ignorés, salle réelle de 1837) verts |
| Lint `MissingPose` rouge/vert | fait | `logs/p-1d-missing-pose-lint-335969b8.log` : fixture `model_poses_missing` 5 `MissingPose` ; `model_poses_complete` aucune ; 1837 `9e7c6df` tel quel : 2 (`fantassin-de-ligne`, `garde-cle-poste` sans `poses.hit`) ; avec `poses.hit` : aucune erreur. Rouge du runner : `logs/p-1d-missing-pose-rouge-9e7c6df.log` |
| Dans `alacod test` | fait | `logs/p-1d-alacod-test-335969b8.log` : 1837 `9e7c6df` tel quel → « 2 erreur(s) de contenu », code 1 ; avec `poses.hit` → les 11 scénarios écrits verts (traces comparées), 4 générés en échec (§ 4) |
| Dans `make check_rapide` | fait | cible `check_animation` (`test_scenarios SCENARIO=model_pantin,frappe_moment`, invariant actif) ; lint des 5 jeux de l'engine sans erreur ; tests `animation_invariant` et fixtures dans `test_crates` |
| `make check_rapide` (forme équivalente, décision du coordinateur du 20 nov : le bloc unique monte à ~47 Go de target, au-delà du disque) | fait, vert | `logs/p-1d-parite-observee-check-rapide-cf5f9b59.log` : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_animation` code 0 (209 s) ; `test_crates` découpé, mêmes options (`--workspace --profile headless --no-default-features --no-fail-fast --features game/dev-dylib -- --exact --skip scenarios`) : `--lib --bins` 1166 passed, `--doc` 0 test, puis les 54 tests d'intégration de `cargo metadata` un par un (binaire supprimé après chacun) : 54/54 joués, 0 échec (script `logs/p-1d-test-crates-decoupe.sh`). Passe précédente sur `280d0105` (bloc unique) arrêtée par la garde disque à 10 Go : `logs/p-1d-parite-observee-check-rapide-280d0105.log` |
| Traces existantes | inchangées | aucun type rollback touché ; `model_pantin`, `frappe_moment` verts (traces identiques) ; `schedule_order_diff` non joué : aucun système de `GgrsSchedule` ajouté ni déplacé (`decide_poses` est en `PostUpdate`) |
| Observé au vrai jeu | fait | `logs/p-1d-captures/observation-jeu.md` : après (`280d0105`), chaque fantassin joue `epauler` 19 f → `viser` 65 f → `tir` 35 f → `rechargement` 179 f (clip 0 → 1,95 s, une fois) → `fusil-marche` (retour) → `fusil-idle`, sur les `AnimationPlayer` réels et dans `alacod/state.anim` |
| Capture côte à côte banc / jeu, cycle complet | fait | `logs/p-1d-captures/cote-a-cote/cycle-complet.png` (et une image par état : épauler, cône, tir, recul, rechargement, retour) : banc (`1867_lore` `cf27aaa`) / jeu avant / jeu après |

Méthode d'observation (VPS, sans écran) : `xauth` absent, donc `Xvfb :96` lancé directement (même effet que `xvfb-run -a`), jeu en pause (`ALACOD_REMOTE=1`,
`ALACOD_INPUT=neutral`, `ALACOD_MUTE=1`), une frame par `alacod/step`, `world.query` sur `bevy_animation::AnimationPlayer`, `alacod/screenshot` toutes les 3 frames. Banc :
copie de `site/labo/armes/` dans `/tmp` (le dépôt `1867_lore` n'est pas touché) avec deux lignes ajoutées (`window.ENNEMIS`, `window.R`) pour figer le temps (`R.ralenti = 0`)
au milieu de chaque état et faire **une** capture par état, chromium de Playwright en `--headless=new` (le mode fenêtré n'ouvre pas son port de débogage sous ce Xvfb).

### Revue de `39090cc0` (agent reviewer) : six points

| Point | Correction | Rouge avant / vert après |
|---|---|---|
| 1. `stuck_for` sanctionnait la dernière image du tir tenue exprès | `ModelPlayback::held`, posé par `decide_poses` quand il tient le tir ; l'invariant ne compte pas une image tenue | `a_last_frame_held_on_purpose_is_not_a_stuck_clip` : rouge avec `invariants.rs` de `280d0105` (« figé à 0.500 s depuis 30 frames »), vert après |
| 2. Second seuil de déplacement (0,15 u/frame) contre `pose::STILL_BELOW` | `ModelPlayback::moving` (vitesse au sol lue par la pose) remplace la mesure de position | `a_walk_slower_than_a_position_threshold_is_still_a_walk` : rouge avant (« le clip joué `walk` est celui de l'attente »), vert après |
| 3. `MissingPose` ignorait les poses de l'arme | lint et `decide_poses` résolvent les poses de l'arme de la règle `Shoot` par-dessus celles du modèle | fixture `model_poses_on_weapon` : rouge avec `lint.rs` de `280d0105` (4 `MissingPose`), vert après |
| 4. `check_rapide` ne jouait l'invariant sur aucun vrai scénario | cible `check_animation` dans `check_rapide` (binaire `scenarios` seul) | `logs/p-1d-check-animation-499f48d1.log` : les deux scénarios en 26 s (vert, invariant actif) ; 465 s ici avec la compilation du binaire `scenarios` (dans `check_rapide`, il est déjà construit par `test_crates`) |
| 5. Tests verts filtrés | fixtures vertes : échouent sur toute erreur sauf celles d'un jeu minimal sans carte ni armes (nommées) ; salle réelle : échoue sur toute failure du runner | `logs/p-1d-revue-vert.log` |
| 6. Note de passation, § 122 | note supprimée (son contenu est ici) ; § 122 à jour (`held`, `moving`, arme de la règle, `check_animation`) | — |

Journaux : `logs/p-1d-revue-rouge.log`, `logs/p-1d-revue-vert.log`.

## 3. Données de 1837 (à transmettre au coordinateur 1837, rien de commité)

1. `assets/models/models.ron` : `hit: Some((still: Some("touche"))),` dans les `poses` de `fantassin-de-ligne` **et** de `garde-cle-poste` (les deux ont
   `ranged_cycle.stagger_frames: Some(37)`) ; sans cette ligne, `alacod lint` et `alacod test` échouent (`MissingPose`) avec l'engine de ce lot.
2. ~~`salve_fantassin.ron` `Wait(35)` trop long~~ : **corrigé, c'était un écart d'engine** (relevé par le coordinateur 1837, vérifié dans les `.glb` du banc et de 1837 :
   les clés du clip `tir` vont de 1/24 à 14/24 s). three.js (banc) et bevy prennent pour durée le temps de la **dernière clé** (`fin / fps` : `tir` 0,583 s = 35 frames,
   `epauler` 0,375 s = 22,5, `rechargement` 2,0 s, `touche` 0,542 s) ; `AnimeManifest::duration` prenait la plage (`(fin − début) / fps`), une image de moins : le
   tir finissait 2,5 frames avant la séquence et le rechargement ne montrait jamais sa dernière image (seek max 1,95 s au jeu). `Wait(35)` et `Wait(23)` de 1837 sont justes.
   Correction : `duration = fin / fps` (§ 5). `exact_frames` (repères de simulation, M-6 § 116) garde la plage : décision du coordinateur du 20 nov, inchangé dans ce lot (l'aligner
   déplacerait les fenêtres de frappe, donc des traces).
3. `alacod test` sur 1837 : les scénarios générés `weapon_machine_gun`, `weapon_pistol`, `weapon_rifle`, `weapon_shotgun` échouent sur `animation_vivante` (le Frère recharge
   ces armes en jouant `idle` : `models.ron` n'a de poses d'arme que pour `fusil_chasse`). Ces quatre armes (`ZombieShooter/Sprites/Character/weapons.ron`) ne sont données par
   personne (avertissement du lint) : les retirer des `content_folders`, ou leur déclarer des poses.

## 4. Ce qui reste

- `shooter_poses` (salle réelle de 1837) non rejoué sur la tête corrigée (disque) : les corrections de la revue n'y changent ni clip ni temps (1837 n'a pas de `weapons` pour `brown_bess_fantassin` ; `held` n'est qu'un drapeau) ; à rejouer au prochain build release de 1837.
- Après le merge (consigne du coordinateur), dans le même lot : revue observée des autres états (spec, « Ordre » point 2 : joueur et fantassin, attente, marche, viser, coup de hache, gigue, touché, mort) et nouveau texte des lignes
  de `1837/docs/parite-labo.md` : non faits ; demandent un nouveau build release de 1837 (≈ 8 Go) et des entrées jouées par état.
