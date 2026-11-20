# P-1d — Parité observée : visée au curseur et cycle des fantassins

Jalon servi : **S1, dim 15 nov** (plan `docs/taches/plan-1837-cli.md` § 6, ligne P-1d et « Changement d'ordre du 16 nov »). Branche `wquintal/p-1d-parite-observee`.
Conventions : § 122 de `docs/conventions.md` ; dette D218. Journaux et captures : `~/orca/runs/alpha/logs/` (VPS, `/home/debian/orca/runs/alpha/logs/`).

## 1. Tranche 1 — visée au curseur : livrée

Mergée dans `main` `85c86745` (puis `00fc97f1` avec M-7) ; 1837 `9e7c6df` l'embarque. Verdict de William (lun 16, sur 1837 `9e7c6df`) : « mon personnage a les animations
correctes et complètes, mais pas les ennemis ». Les choix de comportement du Frère n'ont pas changé depuis (`Fades::default()`, cap exact, esquive et armes inchangées) ; ses **durées de présentation**, si :
la correction de la durée des clips (`ede10218`) allonge chacun de ses clips d'une image, vers le banc (à dire à William pour sa partie).

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
| `make check_rapide` (forme équivalente, décision du coordinateur du 20 nov : le bloc unique monte à ~47 Go de target, au-delà du disque) | fait, vert | `logs/p-1d-parite-observee-check-rapide-cf5f9b59.log` : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_animation` code 0 (209 s) ; `test_crates` découpé, mêmes options (`--workspace --profile headless --no-default-features --no-fail-fast --features game/dev-dylib -- --exact --skip scenarios`) : `--lib --bins` 1166 passed, `--doc` 0 test, puis les 54 tests d'intégration de `cargo metadata` un par un (binaire supprimé après chacun) : 54/54 joués, 0 échec (script `logs/p-1d-test-crates-decoupe.sh`). Rejoué de même sur la tête finale `ede10218` : `logs/p-1d-parite-observee-check-rapide-ede10218.log`, make sans `test_crates` code 0 (186 s), `--lib --bins` 1166/0, 54/54 tests d'intégration, 0 échec (le décompte « 55 / 54 » du script compte les deux lignes `workspace`). Passe précédente sur `280d0105` (bloc unique) arrêtée par la garde disque à 10 Go : `logs/p-1d-parite-observee-check-rapide-280d0105.log` |
| Traces existantes | inchangées | aucun type rollback touché ; `model_pantin`, `frappe_moment` verts (traces identiques) ; `schedule_order_diff` non joué : aucun système de `GgrsSchedule` ajouté ni déplacé (`decide_poses` est en `PostUpdate`) |
| Durée d'un clip = dernière clé (`ede10218`) | fait | `logs/p-1d-duree-clips-ede10218.log` : `content` `models` 15/0 (l'assertion 8/24 échoue avec l'ancien calcul 7/24), `game` `model3d` 14/0, salle réelle `shooter_poses` 2/0 ; au jeu : fantassin `tir` 35 f, `epauler` 22 f, `rechargement` jusqu'à 2,0 s (`p-1d-captures/observation-jeu-etats.md`) |
| Revue de `12abeae7` (P2) : manifeste du pantin | fait | `scripts/gen-pantin.py` écrit les bornes `[0, d × fps]` (temps des clés), manifeste régénéré, `.glb` identique octet pour octet, `fin − début` inchangé (repères M-6 inchangés) ; garde `crates/content/tests/manifest_clip_keys.rs` (durée du manifeste = dernière clé du `.glb`) : rouge sur le manifeste de `12abeae7`, vert après, `logs/p-1d-pantin-manifeste.log` |
| Salle réelle sur la tête de la revue | fait | `logs/p-1d-salle-reelle-120fd943.log` : 2/0 ; sur `36946e6c` contre 1837 `main` `3940825` (poses `hit` posées par 1837) : `logs/p-1d-salle-reelle-36946e6c.log`, 2/0 ; `check_animation` sur `36946e6c` (traces de `frappe_moment` et `model_pantin` identiques) : `logs/p-1d-check-animation-36946e6c.log` |
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
   Correction : `duration = fin / fps` (tableau des preuves, § 2). `exact_frames` (repères de simulation, M-6 § 116) garde la plage : décision du coordinateur du 20 nov, inchangé dans ce lot (l'aligner
   déplacerait les fenêtres de frappe, donc des traces).
3. `alacod test` sur 1837 : les scénarios générés `weapon_machine_gun`, `weapon_pistol`, `weapon_rifle`, `weapon_shotgun` échouent sur `animation_vivante` (le Frère recharge
   ces armes en jouant `idle` : `models.ron` n'a de poses d'arme que pour `fusil_chasse`). Ces quatre armes (`ZombieShooter/Sprites/Character/weapons.ron`) ne sont données par
   personne (avertissement du lint) : les retirer des `content_folders`, ou leur déclarer des poses.

## 4. Revue observée des autres états (spec, « Ordre » point 2), 20 nov

Jeu : 1837 `9e7c6df` + `poses.hit`, engine `ede10218` (durée des clips corrigée), release, Xvfb, une frame par `alacod/step`, entrées du Frère par `alacod/input`
(chronologie `p-1d-captures/jeu-etats-ede10218/` : attente, marche à droite, un tir visé à gauche, un second tir, coup de hache, gigue vers le haut) ; clip lu dans
`alacod/state.anim`. Banc : `site/labo/armes/#frere-chasseur` (copie `/tmp`, temps figé à chaque état, une capture par état). Captures côte à côte :
`p-1d-captures/cote-a-cote/joueur-etats.png` (Frère) et `cycle-complet.png` (fantassin).

| Personnage, état | Banc (`armes.js`) | Jeu (frames observées) | Verdict |
|---|---|---|---|
| Fantassin, attente | `fusil-idle` en boucle, fondu 0,15 s | `fusil-idle` | observé, identique |
| Fantassin, épauler | `epauler` une fois, 0,375 s (22,5 f), fondu 0,08 | `epauler` 22 f (19 f avant la correction de durée) | observé, identique |
| Fantassin, en joue (cône) | `viser` en boucle pendant `annonce` 1 s, cône vers le joueur | `viser` 62 f, cône au sol | observé, identique |
| Fantassin, tir et recul | `tir` une fois 0,583 s (35 f), fondu 0,04, recul 0,6 m | `tir` 35 f (32 f avant) | observé, identique |
| Fantassin, rechargement | `rechargement` 2 s étiré sur 3 s, une fois, fondu 0,1 | `rechargement` 179 f, position 0 → 2,0 s (1,95 s avant : dernière image jamais montrée) | observé, identique après correction |
| Fantassin, retour au poste | allure `fusil-marche`, face au joueur | `fusil-marche` 7 à 24 f puis `fusil-idle` | observé, identique |
| Fantassin, touché | `touche` 0,542 s, cap gelé | non observé au jeu (les deux tirs du Frère ont manqué) ; couvert en headless (`sim_activity` « coup reçu », `MissingPose` exige `poses.hit`) | non observé |
| Fantassin, mort | aucune au banc | non observée | sans objet au banc |
| Frère, attente | `fusil-idle` | `fusil-idle` | observé, identique |
| Frère, marche | allure selon la direction relative au corps (corps tourné vers la visée) ; 4,4 m/s → `fusil-run-*` | `fusil-run` à 105,6 u/s = 4,4 m/s (corps dans le sens de la marche : pas de visée pendant la marche jouée) | observé, identique (vitesse, clip de course) |
| Frère, épauler | `epauler` étiré sur `fEpauler` 0,15 s (9 f) | `epauler` 9 f | observé, identique |
| Frère, tir | `tir` étiré sur `fTir` 0,3 s (18 f), puis le rechargement | `tir` 34 f : 18 f de tir puis 16 f de `tir` pendant le rechargement | **écart d'engine** (présentation) : le geste du tir du joueur suit la durée du clip, pas celle du geste d'arme |
| Frère, rechargement | `rechargement` étiré sur 1,2 s à partir de la fin du tir | simulation : 72 f à partir de la fin du tir (f123-194) ; clip `rechargement` affiché f139-193, entré à 22 % | **écart d'engine** (même cause) |
| Frère, coup de hache | `coup` étiré sur `hDuree` 0,35 s (21 f) | `coup` 21 f | observé, identique |
| Frère, gigue | premier pas : `gigue-01` | premier pas : `gigue-02`, 28 f (un temps), 55,2 u (pas « bien »), courbe de freinage | **écart d'engine** (présentation) : l'ordre des pas commence au deuxième clip |
| Frère, touché, mort | aucun au banc | PV 100 → 80 → 60 sans changement de clip ; pas de mort avant f560 | sans objet au banc |

Écarts d'engine relevés et **non corrigés dans ce lot** (le Frère est validé tel quel par William le 16 nov ; à décider avant de le toucher) : (a) geste du tir du
joueur joué à la durée du clip au lieu du geste d'arme (`fTir`), ce qui décale le rechargement ; (b) premier pas de gigue sur `gigue-02` (`pose::choose` :
`model.dash[n % len]` avec `n` compté à partir de 1). Proposés comme tranche suivante (T-3b ou P-1e).

## 5. Texte proposé pour `1837/docs/parite-labo.md` (à appliquer par le coordinateur 1837)

Chaque « identique » de la colonne Écart devient :

| Ligne (`9e7c6df`) | Règle | Nouveau texte de la colonne Écart |
|---|---|---|
| 29 | Vitesse hache en main | non observé (lecture du 6 nov) ; la marche observée le 2026-11-20 était fusil en main (ligne 30) |
| 30 | Vitesse fusil porté | observé le 2026-11-20 : 105,6 u/s = 4,4 m/s (`p-1d-captures/jeu-etats-ede10218`, f40-60) |
| 32 | Directions de marche | non observé (une seule direction jouée) |
| 39 | Recul qui pousse le tireur | joueur : observé le 2026-11-20, aucun déplacement du Frère pendant l'épaulé, le tir et le rechargement (f96-138) ; ennemi : observé, recul puis retour (§ 7) |
| 48 | Distance d'un pas | observé le 2026-11-20 : pas jugé « bien » = 55,2 u (48 × 1,15, ligne 62) ; pas hors du temps non observé |
| 51 | Ce que protège l'invulnérabilité | non observé |
| 52 | Direction fixée au départ | observé le 2026-11-20 : pas vers le haut, x constant pendant les 28 frames |
| 63 | Hors du temps | non observé |
| 73 | Angle du tir | non observé (trajectoire de la balle non mesurée) |
| 74 | Le corps regarde la visée | observé le 2026-11-14 (curseur réel, tranche 1 de P-1d) et le 2026-11-20 (visée à gauche : `FacingDirection` Left, `joueur-etats.png`) |
| 75 | Directions du corps | observé le 2026-11-20 : fantassin arrondi aux 8 directions ; Frère au cap exact (écart assumé, décision du 6 nov, présentation) |
| 83 | Tir au clic | observé le 2026-11-20 : un appui → épaulé 9 f puis tir (f96-122) |
| 86 | Rechargement du fusil | observé le 2026-11-20 : 72 f à partir de la fin du tir (f123-194) ; à l'écran le clip n'entre qu'à 22 % (écart d'engine, rapport P-1d § 4) |
| 90, 91, 93 | Vitesse, dispersion, portée de la balle | non observé |
| 107 | Durée du coup | observé le 2026-11-20 : `coup` 21 f |
| 115 | Renvoi parfait | non observé |
| 131 | En joue : cône au sol | observé le 2026-11-20 : `viser` et cône 60 f (clip 62 f avec les fondus) |
| 135 | Une balle par salve | non observé (compte des balles non relevé) |
| 138 | Durée du tir (fantassin) | observé le 2026-11-20 : `tir` 35 f |
| 141 | Rechargement du fantassin | observé le 2026-11-20 : 180 f, clip `rechargement` joué une fois de 0 à 2,0 s, étiré sur 3 s |
| 165 | Tampon d'appui du dash | non observé |

Les lignes « verbe », « RON », « présentation » et « à trancher » du 6 nov décrivent l'état d'avant P-1 : elles ne sont pas reprises ici (hors de ce lot).

## 6. Ce qui reste

- Écarts d'engine du Frère (§ 4, a et b) : décision de William avant d'y toucher.
- Fantassin touché non observé au jeu (couvert en headless).
