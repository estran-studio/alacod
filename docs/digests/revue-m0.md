# Revue humaine de M0 (T3.3) — notes de jeu

Ouvert le 2026-10-04. Branche `fix-revue-m0` (depuis `main` `1a39283`). William joue au clone `zombies` sur son
MacBook (Apple Silicon, macOS), seul puis à deux par `docs/jouer-a-deux.md`. Une ligne par
observation ; les dettes D28–D40 sont connues et ne sont pas re-signalées.

Légende — **Type** : bug (comportement faux) ou sensation (rythme, prix, dégâts, feedback : se
note, ne se corrige pas ici). **Gravité** : bloque / gêne / détail. **Suite** : attente (sur un
scénario existant), scénario (nouveau, souvent un enregistrement `revue_<sujet>`), correctif (dans
cette branche), M1 (tâche à ouvrir par l'orchestrateur).

## Mise en route (macOS)

| # | Type | Quoi | Quand | Repro | Gravité | Suite | État |
|---|---|---|---|---|---|---|---|
| R1 | — | `cargo build -p zombies --profile headless` compile sur macOS arm64 (nightly 1.101, 2026-09-29) : 1 min 18 s avec les dépendances déjà en cache. Seul avertissement propre au Mac : `ld: __eh_frame section too large (max 16MB)` (unwind compact), sans effet observé | mise en route | oui | détail | aucune | constat |
| R2 | bug | `make record_session` passe par `ldtk_map_explorer` : profil `dev` + `--features native`, donc une compilation complète de plus et ~15 fps (D30) pendant la session à enregistrer | mise en route | oui | gêne | correctif proposé (Makefile : profil `headless` par défaut pour `record_session`), à décider par William ; contournement : `ALACOD_RECORD=… cargo run -p zombies --profile headless` | ouvert |
| R3 | bug | `ALACOD_HEADLESS=1` sur un binaire compilé avec `render` (défaut) panique au démarrage : `No sub-app with label 'RenderApp' exists` (bevy_app), sans dire qu'il faut `--no-default-features` | mise en route | oui | détail | correctif proposé : message clair (ou refus explicite) au démarrage | ouvert |

## Partie solo

| # | Type | Quoi | Quand (vague, frame) | Repro | Gravité | Suite | État |
|---|---|---|---|---|---|---|---|
| R4 | bug | Après « Rejouer », le joueur (et les balles, et les zombies) traversent les murs. Cause : `CollisionGrids::walls` (grille dérivée, non rollback) n'est reconstruite que si la signature des murs (nombre, somme des `GgrsNetId`) change ; la relance recrée les mêmes murs avec les mêmes ids, la grille garde les `Entity` détruites. `cleanup_rollback_world_system` ne la remettait pas à zéro. Aucun test ne le voyait : le test de relance (`restart_replays_identically_*`) joue `idle`, joueur immobile ; `avant_poste` n'avait qu'un scénario où le joueur fait trois cases | partie 2 et suivantes (log `game_run_XkVqOT`) | oui : test `restart_keeps_walls_solid` (joueur en (−200, 1080) après relance, au lieu de (282, 764)) | bloque | correctif + test + scénario `revue_murs_avant_poste` (murs de la salle de départ, graine du jeu) | **corrigé sur `main`** (même correctif sur `main` par `f9f0d8e` ; test et scénario arrivés par m1-fusion-revue-m0-suite) |
| R5 | bug | La touche R (rechargement) abandonne la partie et la relance : `ui::game_over::button_system` lit `just_pressed(KeyR)` pendant tout `InGame`, pas seulement sur l'écran de fin. Six relances d'affilée dans le log, aucune frame jouée entre elles | partie 1 et suivantes | oui : test `r_ne_relance_que_sur_l_ecran_de_fin` | bloque | correctif (R ne relance que si l'écran de fin est affiché) + test | **corrigé sur `main`** (m1-fusion-revue-m0-suite) |
| R6 | bug | Touches réelles (`character/player/control.rs`) différentes de celles annoncées pour la revue : interagir = **H** (pas E), dash = **C** (pas espace), arme suivante = Tab ; les flèches haut/bas/gauche déplacent la **caméra**, mais flèche droite déplace le joueur **et** la caméra (liée deux fois). Aucune doc joueur des touches | mise en route | oui | gêne | doc des touches (présentation) ; choix des touches et des flèches : à William (sensation/design) | ouvert |
| R7 | bug ? | Partie solo 2 (log `game_run_7pV4I7`, f0-8066, fenêtre fermée normalement) : aucun `tests/scenarios/revue_solo1.ron` écrit, aucune ligne « session enregistrée » dans le log. L'enregistreur est bien dans le jeu fenêtré (`core.rs:263`) ; reste à savoir si `ALACOD_RECORD` était dans la commande | partie 2 | à vérifier | gêne | Reproduit le 2026-10-08 (lancement avec `ALACOD_RECORD`, fenêtre fermée : rien d'écrit). Cause : `write_recording_on_exit` (`Last`) n'était ordonné qu'après `state_trace::ExitRequests`, pas après `bevy::window::ExitSystems`, où `exit_on_all_closed` émet `AppExit` à la fermeture de la fenêtre ; selon l'ordre, le message n'était pas encore là et le jeu s'arrêtait sans écrire. Correctif : `.after(bevy::window::ExitSystems)`. Vérifié en jeu le 2026-10-09 : `revue_mouvement.ron` écrit à la fermeture (5304 frames) | corrigé (non commité) |
| R8 | design | Toute partie locale utilise la graine fixe `default_seed` 123456 (`games/zombies/assets/game.ron`) : même carte et même suite de power-ups à chaque partie. Simulation du flux `loot` (graine 123456) : DP, DP, Carpenter, DP, DP, DP, DP, DP, Insta-Kill… — exactement la partie 2 (6 Double Points sur 7 drops) | parties 1 et 2 | oui (déterministe) | gêne | à William : graine tirée au hasard par partie (hors simulation, partagée par la session) sauf `--seed`, ou garder la graine fixe | ouvert |
| R9 | détail | `RollbackRng` est un LCG mod 2³² et `next_u32_range` prend `next_u32() % n` : bits de poids faible de période courte (les 4 derniers bits ont une période de 16). Distribution à long terme correcte (simulée : DP 25,0 % pour 25 % attendus), mais suites courtes corrélées | — | — | détail | M1 : prendre les bits de poids fort (change toutes les traces qui tirent un entier : preuve requise) | ouvert |
| S1 | sensation | « Le gameplay est vraiment mauvais » (William, après la partie 2 : 38 kills, mort en vague 4 vers f8066 ≈ 2 min 15) | partie 2 | — | — | à préciser avec William, puis tâches M1 | noté |
| S2 | sensation | « La carte ne fait aucun sens » (`avant_poste`, générée par `gen_map.py` hors dépôt en m0-v8 ; 4 salles sur 9 placées avec la graine 123456) | partie 2 | — | — | à préciser avec William, puis tâche M1 (carte faite à la main dans LDtk ?) | noté |
| S3 | sensation | movement-feel : « les mouvements marchent assez bien » (course, dash). Premier retour humain sur movement-feel ; i-frames non commentées | `revue_mouvement` (2026-10-09, vidéo OBS 00:01:14) | — | — | aucune (validé) ; à confirmer sur une partie plus longue | noté |
| S4 | sensation | Équilibrage des armes (confirmé par William le 2026-10-09) : shotgun « super fort », pistolet inutile, mitraillette forte mais « tellement imprécise » (trop de spread) | `revue_mouvement`, vidéo 00:00:37–00:01:01, vagues 1-2 | — | — | M1 : passe d'équilibrage des armes (`weapons/`, spread, dégâts, cadence) | noté |
| S5 | sensation | Équilibrage des vagues « bien trop [dur] en partant » (confirmé par William le 2026-10-09 ; « sonores » était une erreur de transcription). Mesure : vague 1 (7 zombies) vidée en ~22 s, vague 2 (12) en ~30 s | `revue_mouvement`, vidéo 00:00:43 | — | — | Passe de contenu S5 : référence 4 acheteurs × 20 graines mesurée ; options A/B proposées, détails dans `docs/taches/rapports/m0-revue-suite-contenu.md` | choix de William attendu ; aucun réglage appliqué |
| S6 | sensation | Feedback : « pas trop d'indications sonores par rapport aux vagues » (début et fin de vague silencieux) | `revue_mouvement`, vidéo 00:00:32 | — | — | M1 : son de début et de fin de vague (présentation) | noté |
| S7 | sensation | « Son des armes trop fort » (phrase de 00:00:15, mal transcrite, précisée par William le 2026-10-09) | `revue_mouvement` | — | — | M1 : mixage (volume des tirs par rapport au reste) | noté |
| R10 | bug ? | « Impossible d'acheter un soda, il chevauche les fusils » (précision de William, 2026-10-09). Transcription : « Je sais pas comment acheter les trucs de pack-à-punch, ils ont tous l'air d'être en dessous des armes, on peut juste en sélectionner un. » Il n'y a pas de pack-a-punch dans M0 ; dans `avant_poste.ldtk`, les deux `SodaLocation` (Jug, Cellier) ne sont pas sous des armes. À la même minute, le prompt affiché est « [H] Munitions mitrailleuse — $750 » avec le shotgun en main. **Contre-constat** : dans cette partie, aucune machine à soda n'existe : le log n'a pas de ligne « Map is loaded with N soda locations », et `map_probe` (`ALACOD_MAP_PROBE=revue_mouvement:3900`, sonde étendue aux `Interactable`) ne trouve que 2 armes murales, (376, 760) et (872, 760), et aucun `Perk`. Les 4 salles placées par la graine 123456 ne sont pas Jug ni Cellier, les seules qui ont une `SodaLocation`. Reste à savoir ce qui a été pris pour un soda (sprite d'arme murale ?) ou dans quelle autre partie | `revue_mouvement`, vidéo 00:01:05 | non reproduit | gêne | demander à William où (capture) ; lié à S2/R8 (salles tirées par la graine) | ouvert |
| R11 | ? | « Il y a un bug justement » (vidéo 00:00:21, fin de la vague 1, rien de visible sur l'image) ; William : remarque générale, « il faudrait peaufiner un peu » | `revue_mouvement` | à préciser | — | à préciser avec William | ouvert |
| S8 | sensation / contenu | « Une meilleure carte » ; « il n'y a pas de spawn de zombies dans les autres salles ». Constat dans `avant_poste.ldtk` : les 5 `ZombieSpawn` sont tous dans `Depart`, aucune des 8 autres salles n'en a ; ouvrir une porte n'apporte donc aucune menace nouvelle. Précise S2 | session `revue_relance` (2026-10-09) | oui (données) | gêne | M1 : refaire la carte (spawners par salle, fenêtres sur les salles ouvertes, disposition) ; lié à S2 et R8 | noté |
| — | constat | **R4 et R5 validés par William en jeu** (2026-10-09, session `revue_relance`, non filmée) : R rechargé en pleine partie sans relance (log : `sound reload` f578, aucune relance avant la mort) ; mort f1158 (`outcome=Defeat`, vague 1), « Rejouer », murs solides après relance ; « pas de problème » | `revue_relance` | — | — | — | constat |
| — | constat | Fonctionnement vérifié par William en jeu après R4/R5 : interactions (portes, achats), le reste « fonctionne correctement » | partie 2 | — | — | — | constat |

## Bilan de la revue solo (William, 2026-10-09)

Validation de son côté terminée : R4, R5, R7 corrigés et vérifiés en jeu ; movement-feel validé
(S3). Les points à faire en M1 sont, dans ses mots : équilibrage des vagues (S5), des armes
(S4), son des armes trop fort (S7), indications sonores des vagues (S6), une meilleure carte avec
des spawns dans les autres salles (S2, S8), le soda « impossible à acheter » (R10, non reproduit).
Restent ouverts sans décision : R2, R3, R6 (touches), R8 (graine fixe), R9.

Les deux enregistrements sont rangés dans `docs/captures/revue-m0/` (hors `tests/scenarios/` :
le test des scénarios exige une trace pour chaque `.ron` du dossier). Les vidéos OBS restent sur
le MacBook de William (`~/Movies/2026-10-09 00-33-22.mov`).

## Partie à deux

| # | Type | Quoi | Quand (vague, frame) | Repro | Gravité | Suite | État |
|---|---|---|---|---|---|---|---|

## Points à regarder exprès

- [ ] HUD : points, munitions, vague
- [ ] Achats au mur et perks (prix, prompt, disparition après achat)
- [ ] Portes
- [ ] Fenêtres et réparation
- [ ] Power-ups : Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke
- [ ] À terre et réanimation (à deux)
- [ ] Fin de partie et retour au lobby (restart p2p absent : D14, connu)
- [ ] Sons
- [ ] Fermeture de la fenêtre
- [ ] fps

## Sessions enregistrées

| Fichier | Sujet | Frames | Joueurs | Usage |
|---|---|---|---|---|
| `docs/captures/revue-m0/revue_mouvement.ron` | movement-feel, vagues 1-3 (vidéo OBS `2026-10-09 00-33-22.mov`, 86 s) | 5304 | 1 | sans attentes, non béni ; source de S3–S7, R10, R11 |
| `docs/captures/revue-m0/revue_relance.ron` | R5 puis mort et « Rejouer » (R4) ; l'enregistreur ne garde que la partie relancée | 1293 | 1 | sans attentes, non béni |
