# P-1e — tableau chiffré banc / jeu, **avant** correction (20 nov 2026)

- **Banc** : `1867_lore/site/labo/armes/` (identique à `origin/main:tools/site/armes.js` `cf27aaa`, vérifié par `diff`), défauts (`DEFAUTS`, aucun réglage stocké), Frère chasseur,
  fusil de chasse. Copie instrumentée dans `/tmp/p1e-banc` (relevé de `J.pos`, `t`, `J.action`, balles à chaque image ; dessin coupé pour la mesure, logique inchangée ;
  `dt` fixé à 1/60 s, ce que donne un écran à 60 Hz, voir « Pas de temps »). Entrées **réelles** : clavier et souris par CDP `Input.dispatchKeyEvent` /
  `dispatchMouseEvent` (chromium Playwright 1248, `--headless=new`). Pilote `p-1e-captures/banc-mesure.mjs`, relevé `banc-releve.json`, analyse `banc-analyse.py` → `banc-grandeurs.txt`.
  Gigue au tempo : banc `site/labo/gigue/` (`gigue.js`, `DEFAUTS` + `defautsDesPas`), lu.
- **Jeu** : 1837 `origin/main` `1a3e639` (= `f2ac3bf` + docs), release, engine du worktree `8f093d2f` (= `36ab2386` du `rev` de 1837 + musique seule ; plus la liste
  `bullets` de `alacod/state`, lecture seule), Xvfb + lavapipe, partie en pause (`ALACOD_REMOTE=1`), une frame par `alacod/step`. Entrées par `alacod/input` : input tenu du
  joueur 0, lu dans `ReadInputs` et soumis à la **session GGRS du jeu** (même chemin que le clavier, `read_local_inputs`). Mêmes gestes aux mêmes instants que le banc
  (frame = 60 × t). Pilote `p-1e-captures/jeu-mesure.py`, relevé `jeu-avant.jsonl`, analyse `jeu-analyse.py` → `jeu-avant-grandeurs.txt`, clips `jeu-avant-clips.txt`.
  Positions de `alacod/state` arrondies à 0,1 u.

## Vérifiés, pas supposés

| Point | Banc | Jeu | Verdict |
|---|---|---|---|
| Pas de temps | `dt` = temps réel de `requestAnimationFrame`, plafond 0,05 s (`armes.js:754`), **pas de sous-pas** ; à 60 Hz : 1/60 s | 60 Hz fixe : `bevy_ggrs` accumule le temps réel (`schedule_systems.rs`, période 1e9/60 ns), `FIXED_TIMESTEP` 1/60 (`movement.rs:7`) | identique à 60 Hz |
| Échelle | mètres (`ARENE` 16 × 12 m) | 24 u/m (`models.ron` `units_per_meter`, commentaires de `player.ron` et `weapons.ron`) | conversions des RON justes : 4,4 m/s ↔ 105,6 u/s, 16 m/s ↔ 384 u/s, 2 m ↔ 48 u |
| Intégration | `pos += voulu × v × dt` : vitesse pleine dès l'image, arrêt net (`armes.js:774-775`) | `run_velocity` vers la cible avec `acceleration`/`deceleration` 7200 u/s² × 1/60 = 120 u/s par frame (pleine vitesse en une frame), puis `pos += v / 60` (`move_characters`) | identique (une frame) |
| Délai d'entrée | l'image qui suit l'événement clavier (`touches`, lu par `directionVoulue()` dans `pas()`) | `with_input_delay(5)` sur la session **locale** (`jjrs/local.rs:84` ← `args::input_delay_from_env()`, défaut 5 pour tous les modes, `1e8f0203`) : l'input d'une frame est appliqué 5 frames plus tard | **écart d'engine** |

## Grandeurs du Frère et des balles

| Grandeur | Banc (mesuré) | Jeu avant (mesuré) | Écart |
|---|---|---|---|
| Latence appui → premier déplacement | 1 image = **16,7 ms** | 6 frames = **100 ms** (5 de délai GGRS + 1) | **+83 ms** |
| Vitesse de marche, fusil porté | **4,400 m/s** (0,0733 m/image) | **4,396 m/s** (1,758 u/frame, 105,5 u/s) | 0,1 % (arrondi du relevé) |
| Vitesse de course | pas de course distincte au banc (fusil : `fPorter` 4,4 ; `seuilCourse` 3,6 m/s choisit le clip `fusil-run`) | pas de sprint (`sprint_multiplier` 1,0) ; clip `fusil-run` au-delà de `run_above_m_s` 4,0 | sans objet |
| Accélération | pleine vitesse dès la première image (échelon) | 7200 u/s² = 300 m/s² : pleine vitesse dès la première frame **après** le délai | identique une fois l'input appliqué |
| Temps pour atteindre la vitesse (depuis l'appui) | **16,7 ms** | **100 ms** | **+83 ms** |
| Freinage : distance après le relâché | **0 cm**, arrêt à l'image suivante | **36,7 cm** (8,8 u) en 5 frames | **+36,7 cm** |
| Freinage : temps d'arrêt | **16,7 ms** | **83 ms** de glisse puis arrêt net | **+67 ms** |
| Demi-tour : temps pour inverser | **16,7 ms** (1 image) | **100 ms** (6 frames) | **+83 ms** |
| Demi-tour : distance perdue (dépassement) | **0 cm** | **36,7 cm** | **+36,7 cm** |
| Marche en joue (épaulé + tir), `enJoue` | **2,640 m/s** (× 0,6) | **2,641 m/s** (63,4 u/s) | 0 |
| Marche en rechargeant, `fPendant` | **2,200 m/s** (× 0,5) | **2,199 m/s** | 0 |
| Marche hache en main, `hMarche` | **5,000 m/s** | sans objet : la hache est le coup de mêlée, le fusil reste porté ; pendant le coup 0,6 × 120 = 72 u/s = 3 m/s, comme `hRalenti` × `hMarche` | sans objet |
| Vitesse des balles (`fVitesse`) | **16,00 m/s** | **16,00 m/s** (384 u/s, 6,4 u/frame) | 0 |
| Appui → départ de la balle | **183 ms** (11 images : 1 + épaulé 0,15 s + 1) | **250 ms** (15 frames : 6 + épaulé 9) | **+67 ms** (le délai d'entrée) |
| Épaulé, geste de tir, rechargement (simulation) | 0,15 s ; 0,3 s ; 1,2 s à la fin du tir | 9 f ; 18 f ; 72 f (munition 0 → rechargement à +0,55 s, plein à +1,75 s) | 0 |
| Recul du Frère (poussée, durée) | **aucun** : le clip `tir` du Frère n'a pas de `moments.tir.deplacement` (seule la Brown Bess en a) | **aucun** (0,00 cm pendant le geste) | 0 |
| Gigue : distance, durée | banc armes : 2 m en 0,36 s (22 f), courbe 1 − (1 − u)² ; banc gigue : 2 m en un temps (tempo 129 → 0,465 s ≈ 28 f), +30 %/+15 % parfait/bien | 2,30 m (= 55,2 u, jugé « bien ») en 28 f, même courbe | aligné sur le banc gigue (choix de `parite-labo.md` § 2) ; 0,36 s du banc armes non repris |
| Gigue : latence | 1 image | 6 frames | **+83 ms** (même délai) |
| Geste du tir à l'écran (P-1d a) | clip `tir` étiré sur `fTir` 0,3 s (18 f) | clip `tir` joué à sa durée : 34 f | **écart d'engine** (présentation) |
| Épaulé à l'écran | clip `epauler` entier étiré sur `fEpauler` 0,15 s | 9 f, mais lu à la durée du clip (0,375 s) : 36 % du clip montré | **écart d'engine** (même cause, non relevé par P-1d) |
| Rechargement à l'écran (P-1d a) | `rechargement` étiré sur 1,2 s dès la fin du tir, entré à 0 % | entré à **23,6 %** (0,472 s du clip) 0,82 s après l'appui, 55 f montrées sur 72 | **écart d'engine** (même cause) |
| Premier pas de gigue (P-1d b) | `gigue-01` (`J.pas` part de −1, `armes.js:428`) | `gigue-02` | **écart d'engine** (présentation) |

## Cause

« Plus lente et lourde » : chaque geste du Frère (départ, arrêt, demi-tour, tir, gigue) part **5 frames (83 ms) plus tard** qu'au banc, et il **glisse de 37 cm** après chaque
relâché. Les nombres de 1837 (vitesses, accélération, balles, gestes) sont justes ; la cause est l'engine : la partie locale construit sa session GGRS avec le délai d'entrée
du jeu en ligne (`ALACOD_INPUT_DELAY`, défaut 5, posé pour le smoke p2p). Les trois écarts de présentation (tir, épaulé, rechargement à l'écran ; premier pas de gigue) sont
des écarts d'engine aussi.
