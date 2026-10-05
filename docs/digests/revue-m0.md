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
| R4 | bug | Après « Rejouer », le joueur (et les balles, et les zombies) traversent les murs. Cause : `CollisionGrids::walls` (grille dérivée, non rollback) n'est reconstruite que si la signature des murs (nombre, somme des `GgrsNetId`) change ; la relance recrée les mêmes murs avec les mêmes ids, la grille garde les `Entity` détruites. `cleanup_rollback_world_system` ne la remettait pas à zéro. Aucun test ne le voyait : le test de relance (`restart_replays_identically_*`) joue `idle`, joueur immobile ; `avant_poste` n'avait qu'un scénario où le joueur fait trois cases | partie 2 et suivantes (log `game_run_XkVqOT`) | oui : test `restart_keeps_walls_solid` (joueur en (−200, 1080) après relance, au lieu de (282, 764)) | bloque | correctif + test + scénario `revue_murs_avant_poste` (murs de la salle de départ, graine du jeu) | corrigé |
| R5 | bug | La touche R (rechargement) abandonne la partie et la relance : `ui::game_over::button_system` lit `just_pressed(KeyR)` pendant tout `InGame`, pas seulement sur l'écran de fin. Six relances d'affilée dans le log, aucune frame jouée entre elles | partie 1 et suivantes | oui : test `r_ne_relance_que_sur_l_ecran_de_fin` | bloque | correctif (R ne relance que si l'écran de fin est affiché) + test | corrigé |
| R6 | bug | Touches réelles (`character/player/control.rs`) différentes de celles annoncées pour la revue : interagir = **H** (pas E), dash = **C** (pas espace), arme suivante = Tab ; les flèches haut/bas/gauche déplacent la **caméra**, mais flèche droite déplace le joueur **et** la caméra (liée deux fois). Aucune doc joueur des touches | mise en route | oui | gêne | doc des touches (présentation) ; choix des touches et des flèches : à William (sensation/design) | ouvert |
| R7 | bug ? | Partie solo 2 (log `game_run_7pV4I7`, f0-8066, fenêtre fermée normalement) : aucun `tests/scenarios/revue_solo1.ron` écrit, aucune ligne « session enregistrée » dans le log. L'enregistreur est bien dans le jeu fenêtré (`core.rs:263`) ; reste à savoir si `ALACOD_RECORD` était dans la commande | partie 2 | à vérifier | gêne | vérifier la commande de lancement ; sinon test de l'écriture à la fermeture de fenêtre | ouvert |
| R8 | design | Toute partie locale utilise la graine fixe `default_seed` 123456 (`games/zombies/assets/game.ron`) : même carte et même suite de power-ups à chaque partie. Simulation du flux `loot` (graine 123456) : DP, DP, Carpenter, DP, DP, DP, DP, DP, Insta-Kill… — exactement la partie 2 (6 Double Points sur 7 drops) | parties 1 et 2 | oui (déterministe) | gêne | à William : graine tirée au hasard par partie (hors simulation, partagée par la session) sauf `--seed`, ou garder la graine fixe | ouvert |
| R9 | détail | `RollbackRng` est un LCG mod 2³² et `next_u32_range` prend `next_u32() % n` : bits de poids faible de période courte (les 4 derniers bits ont une période de 16). Distribution à long terme correcte (simulée : DP 25,0 % pour 25 % attendus), mais suites courtes corrélées | — | — | détail | M1 : prendre les bits de poids fort (change toutes les traces qui tirent un entier : preuve requise) | ouvert |
| S1 | sensation | « Le gameplay est vraiment mauvais » (William, après la partie 2 : 38 kills, mort en vague 4 vers f8066 ≈ 2 min 15) | partie 2 | — | — | à préciser avec William, puis tâches M1 | noté |
| S2 | sensation | « La carte ne fait aucun sens » (`avant_poste`, générée par `gen_map.py` hors dépôt en m0-v8 ; 4 salles sur 9 placées avec la graine 123456) | partie 2 | — | — | à préciser avec William, puis tâche M1 (carte faite à la main dans LDtk ?) | noté |
| — | constat | Fonctionnement vérifié par William en jeu après R4/R5 : interactions (portes, achats), le reste « fonctionne correctement » | partie 2 | — | — | — | constat |

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
