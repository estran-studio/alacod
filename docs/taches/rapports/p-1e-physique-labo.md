# P-1e — Physique du labo : le Frère réagit à la frame qui suit l'appui

Jalon servi : **S2, dim 22 nov** (plan `docs/taches/plan-1837-cli.md` § 6, ligne P-1e : « la physique du Frère et des balles au chiffre du labo »). Branche `wquintal/p-1e-physique`.
Convention : § 125 de `docs/conventions.md` ; dette D221. Journaux et captures : `~/orca/runs/alpha/logs/` (VPS : `/home/debian/orca/runs/alpha/logs/`), préfixe `p-1e-`.
Verdict de William (ven 20, 1837 `f2ac3bf`, natif, dans la salle) : « On est assez proche du labo, notre physique est juste plus lente et lourde. »

## 1. Mesure d'abord (avant toute correction)

Tableau complet : `logs/p-1e-tableau-avant.md` ; avant/après : `logs/p-1e-tableau.md`.

- **Banc** : `1867_lore/site/labo/armes/` (identique à `origin/main:tools/site/armes.js`, `cf27aaa`), défauts, Frère, fusil. Copie instrumentée dans `/tmp` (relevé de `J.pos`, `t`,
  `J.action`, balles à chaque image ; `dt` fixé à 1/60 s, ce que donne un écran à 60 Hz ; dessin coupé pour la mesure seulement). Entrées réelles par CDP `Input.dispatchKeyEvent` /
  `dispatchMouseEvent`, chromium Playwright 1248 en `--headless=new` (sans `DISPLAY` : sous Xvfb, l'ANGLE de ce chromium échoue sur `xcb_connect` ; `xauth` absent pour `xvfb-run`).
  Pilote `p-1e-captures/banc-mesure.mjs`, analyse `banc-analyse.py`.
- **Jeu** : 1837 `origin/main` `1a3e639` (= `f2ac3bf` + `docs/tranche.md`), release, `[patch]` vers ce worktree (engine `8f093d2f` = `36ab2386` + musique), Xvfb + lavapipe, partie
  en pause, une frame par `alacod/step`, entrées par `alacod/input` (input tenu, lu dans `ReadInputs` par `read_local_inputs` et soumis à la session GGRS du jeu : le chemin du
  clavier). Pilote `p-1e-captures/jeu-mesure.py`. `alacod/state` liste maintenant les balles (`bullets` : id, position, vitesse par frame, joueur ; lecture seule) pour mesurer leur vitesse.
- **Vérifiés** : pas de temps (banc : `requestAnimationFrame`, plafond 0,05 s, pas de sous-pas ; jeu : 60 Hz fixe, accumulateur de `bevy_ggrs`), échelle (24 u/m ; 4,4 m/s = 105,6 u/s,
  16 m/s = 384 u/s, 2 m = 48 u), intégration (banc : vitesse pleine dès l'image ; jeu : `acceleration`/`deceleration` 7200 u/s² = pleine vitesse en une frame).

**Cause** : tous les nombres de la simulation sont ceux du banc (vitesse, accélération, balles, épaulé 9 / tir 18 / rechargement 72, marche × 0,6 et × 0,5, recul nul). L'écart est
le **délai d'entrée** : `jjrs/local.rs` construit la session de la partie locale avec `args::input_delay_from_env()`, défaut **5 frames pour tous les modes** (posé pour le smoke p2p,
`1e8f0203`). Mesuré : départ 100 ms après l'appui (banc 16,7 ms), glisse de 36,7 cm après chaque relâché (banc 0), demi-tour 100 ms et 36,7 cm de dépassement (banc 1 image, 0),
balle 250 ms après l'appui (banc 183 ms). Cause d'engine, aucune donnée de 1837 à changer.

Écarts de présentation (P-1d § 4, plus un) : tir du Frère joué à la durée du clip (34 f au lieu de 18), rechargement entré à 23,6 % de son clip, **épaulé** montré à 36 % seulement
(même cause, non relevé par P-1d), premier pas de gigue sur `gigue-02`.

## 2. Correction (engine)

| Point | Fait | Où |
|---|---|---|
| Délai d'entrée par mode | 0 hors ligne (partie locale native et web, bac à sable, éditeur), `ONLINE_INPUT_DELAY` 5 en ligne **inchangé**, `ALACOD_INPUT_DELAY` explicite toujours prioritaire | `game::args::{input_delay_override, input_delay_from_env(online)}`, `GameArgs::input_delay`, `GameArgsPlugin` |
| Scénarios et traces inchangés | `Scenario::input_delay` (absent : `SCENARIO_INPUT_DELAY` 5, celui des références) ; runner : variable, sinon champ, sinon 5 | `game::replay::Scenario`, `scenario::runner::game_args` |
| Enregistrement fidèle | un enregistrement du client écrit le délai de sa session (`RecordedSettings::input_delay`) ; celui du runner garde celui du scénario | `game::recording` |
| Gestes d'arme à l'écran | poses `shoulder` et `shoot` étirées sur `raise_frames` / `fire_frames` d'une arme à gestes (`duration_s` de `models.ron` prioritaire) ; le tir tient sa dernière image la frame où le rechargement n'est pas encore visible | `game::model3d::playback::decide_poses` |
| Ordre des esquives | la première esquive joue `dash[0]` (`PoseInput::dash` = rang depuis 0) | `game::model3d::playback`, doc `pose::PoseInput` |

Checklist GGRS : aucun système de `GgrsSchedule`, aucun type rollback, aucun hash touché ; le délai d'entrée est un réglage de session (`SessionBuilder::with_input_delay`), les deux
corrections de pose sont en `PostUpdate` (présentation). `make schedule_order_diff` non joué : l'ordre ne peut pas bouger (aucun système ajouté ni déplacé).

## 3. Contrôle (rouge puis vert)

`crates/scenario/tests/physique_labo.rs` :
- `la_partie_locale_reagit_a_la_frame_qui_suit_l_input` (testbed, pantin, session du client hors ligne `input_delay_from_env(false)`) : départ, arrêt, départ à gauche, demi-tour changent le pas à la frame N + 1 ;
- `les_gestes_d_une_arme_a_gestes_durent_ceux_de_l_arme` (pantin, `arme_a_gestes`, clips allongés au double) : épaulé ≥ 80 % de son clip, tir 18 ± 1 frames, rechargement entré à la frame qui suit, à ≤ 5 % ;
- `la_premiere_esquive_joue_le_premier_clip_de_dash` (deux clips d'esquive) : ordre `roll`, `strike` ;
- `real_room_matches_the_bench` (ignoré, jeu externe, `ALACOD_P1E_*`) : vitesse ± 1 %, réaction ≤ N + 1, balle ± 1 % et partie après épaulé + 1, durée du clip de tir, clip de la première esquive.

| Contrôle | Rouge | Vert |
|---|---|---|
| Testbed, trois tests | mutation des trois points (défaut hors ligne remis à 5, rang d'esquive depuis 1, durées de geste non posées) : réaction à f36 pour un input f30 ; épaulé jusqu'à 0,133 s sur 0,3 ; ordre `strike, roll` | 3/3 |
| Salle réelle de 1837 (`f2ac3bf` + `physique_labo.ron`) | même mutation : réactions f36/66/86/106 (banc 31/61/81/101), balle f165 pour un appui f150 | réactions f31/61/81/101, 105,57 u/s, balle 384 u/s à f160, `tir` 18 f, `gigue-01` |
| Forme jouable de 1837 (`alacod test 1837 --tag rapide`) : `tests/scenarios/physique_labo.ron` | `ALACOD_INPUT_DELAY=5` (l'ancien défaut ; la variable prime sur `input_delay: Some(0)`) : 6 attentes en échec (`PlayerPosition` f31/61/62/101/102, `BulletCount` f160) | 6/6 scénarios `rapide` verts, traces comparées ; les 5 traces existantes de 1837 inchangées |

Journaux : `logs/p-1e-controle-rouge-f2ac3bf.log`, `logs/p-1e-controle-vert-98211f6c.log`. Sur l'engine de `f2ac3bf` lui-même (`36ab2386`), `physique_labo.ron` ne se lit pas (champ
`input_delay` inconnu) : le rouge « de f2ac3bf » est donc montré par la mutation qui remet exactement l'ancien comportement.

## 4. Consignes du coordinateur (20 nov, `logs/p-1e-consigne-mesure.md`)

1. **En ligne inchangé et mesuré** : deux clients 1837 release, matchbox local, entrées scriptées, traces d'état des deux clients identiques ; délai 5 : départ 83 ms, glisse 29 cm,
   demi-tour 83 ms ; la trace en ligne est **identique** avant (engine `8f093d2f`) et après (`98211f6c`) le correctif : sha256 `d49719b2…` pour les deux clients des deux runs, et
   pour le run netem à délai 5 (commandes et sommes : `logs/p-1e-p2p/sommes-traces.txt`, scripts `run.sh`, `netem.sh`, `analyse.py` dans le même dossier). Sous RTT 80 ms (netem
   `delay 40ms 5ms`, ping mesuré 79,9 et 77,4 ms) : délai 5 → 0/3 rollbacks ; délai 2 (`1100caaa…`, un autre déroulé puisque les inputs agissent plus tôt) → 33 ms, 7 cm, 2/0 rollbacks.
   Proposition « délai en ligne 2 » dans `propositions.md` (dette D221), non appliquée.
2. **Priorité de la variable et comportements gardés** : `ALACOD_INPUT_DELAY` passe avant tout défaut, dans le client (`input_delay_override`) comme dans le runner (avant le champ du
   scénario) — prouvé par le rouge de `physique_labo.ron` sous `ALACOD_INPUT_DELAY=5`. Scénarios : défaut 5 et `check_distance` inchangés (`check_distance_from_env`, non touché) ; les
   5 scénarios `rapide` de 1837 rejoués avec traces identiques. Preuve p2p (`scripts/p2p-*.sh`) : toutes en ligne, donc `ONLINE_INPUT_DELAY` 5 ou leur `ALACOD_INPUT_DELAY` explicite
   (`p2p-stress.sh` : 0), comme avant.
3. **Présentation alignée** : tir, épaulé et rechargement à l'écran, premier pas de gigue — rouge/vert chacun (§ 3), observés au vrai jeu après (`logs/p-1e-captures/jeu-apres-clips.txt` :
   `epauler` 9 f, `tir` 18 f, `rechargement` dès la frame suivante à 1,4 %, `gigue-01`).

## 5. Preuve au vrai jeu (après) et capture

`logs/p-1e-tableau.md` : jeu après = banc sur chaque ligne (latence 1 frame, glisse 0, demi-tour 1 frame, balle 10 frames après l'appui). Capture côte à côte banc / jeu après,
21 s à 30 i/s, mêmes gestes aux mêmes instants (`p-1e-captures/gestes-capture.json` : marche et arrêt, demi-tour, tir à l'arrêt et en marchant, gigue, marche haut/bas, second
demi-tour, tir, gigue à gauche ; le Frère n'a pas de recul au banc) : `logs/p-1e-captures/cote-a-cote-banc-jeu-apres-98211f6c.mp4` (images `cote-a-cote-t*.png`). Banc : temps à
pas fixe 1/60 s, une image toutes les 2 frames (`banc-capture.mjs`) ; jeu : release, Xvfb + lavapipe, `alacod/screenshot` toutes les 2 frames (`jeu-mesure.py`, `P1E_GESTES`).

## 6. Données de 1837 (au coordinateur 1837, rien de commité)

Aucune valeur de RON à changer pour la physique. À ajouter après le réépinglage sur l'engine qui contient P-1e (le champ `input_delay` n'existe pas avant) : `tests/scenarios/physique_labo.ron`
et sa trace (copies : `logs/p-1e-1837/`, `tags: ["rapide"]`, 6 attentes). `run_above_m_s: 4.0` du Frère (banc : `seuilCourse` 3,6) ne change rien à 4,4 m/s.

## 7. Ce qui reste

- D221 : le délai en ligne (5 frames, 83 ms) ; proposition à 2 écrite, à décider par William avec une nuit p2p à 2.
- La gigue du banc armes (0,36 s) n'est pas reprise : 1837 suit le banc gigue (un temps, 28 f), choix de `parite-labo.md` § 2.

## 8. Contrôle rapide (découpé, feu vert du coordinateur)

`logs/p-1e-physique-check-rapide-d28d0ef0.log` (scripts `logs/p-1e-check-rapide-decoupe.sh`, `p-1e-test-crates-decoupe.sh`, garde disque 15 Go) : `make format test_tools check_forbidden
check_rollback_registration check_sim_boundary lint check_integration` code 0 (1485 s ; jeu court `rapide` et `gestes_entree_reelle` verts, traces identiques) ; `test_crates` découpé :
`--lib --bins` 1166 passed, `--doc` 0, 57 tests d'intégration un par un (dont `physique_labo` 3/3, `schedule_order` 1/1, `sandbox_mode`), 0 échec, codes non nuls 0 (2196 s) ;
binaires de tests supprimés après chacun, sweep (target 12 → 10 Go), fin 00:03 UTC, `/` 30 Go libres. Revue de `9135a98b` (P1 : `input_delay` manquant dans le littéral de test de
`recording`, E0063) corrigée dans `d28d0ef0`, avant ce contrôle.
