# m2-t0d-squelette-gungeon — contenu `games/gungeon/` squelette (M2, vague 0)

Lire d'abord `docs/taches/README.md` (machine `orca`, `docs/taches.md` §7 « Exécution »),
`CLAUDE.md`, `docs/conventions.md` (§3 manifeste, §35 salles, §36 objets, §37-38 boss et profil).
Estimation : 1 à 2 j. Modèle : T1.0c de M1 (`games/throne/` squelette).

## Branche et intégration

Branche `m2-t0d-squelette-gungeon` depuis `main`, puis `git merge origin/m2-t0b-contrats-objets` (T0b, en vérification chez orch ; `git merge origin/main` quand elle y sera), puis
`git merge origin/m2-t0c-contrats-boss-profil` : c'est toi qui résous les conflits mécaniques
annoncés (littéraux `Scenario { .. }` : `items` de T0b + `profile` de T0c dans `recording.rs`,
`generate.rs`, `alacod-sim.rs`, tests de `scenario` ; numéros de section de conventions). Un
commit de fusion à part, avec la liste des conflits et leur résolution. Orch vérifie T0c et T0d
ensemble sur ta branche (une seule suite complète).

## But

Le clone `gungeon` démarre : un dossier de jeu et un binaire, sur les contrats de la vague 0.

- `games/gungeon/` : `Cargo.toml`, `src/main.rs` (boilerplate de l'engine comme `games/throne`),
  `assets/game.ron` (manifeste : personnages, armes, objets, salles, ennemis), un personnage
  (dash à i-frames de §34), une arme (pistolet à munitions infinies, style Gungeon), deux objets
  (un passif, un actif `Rooms(2)`), un consommable `key` et un `blank` (compteur seulement).
- Un ennemi tireur simple (`Shoot` d'un pattern `Aimed`) et un boss vide à deux phases
  (contrats T0c).
- Un étage de trois salles sur l'assembleur `Basic` (voie B) : `depart`, `combat` (verrouillante,
  2 ennemis), `boss` (verrouillante) ; carte LDtk dans `games/gungeon/assets/maps/` (générée par
  script comme `m2-e1-salles.make_salles.py`, script gardé dans `docs/taches/rapports/`).
- Mode de run : `Floors` à un étage pour l'instant (l'étage assemblé est un monde), défaite à la
  mort, victoire au boss tué.
- `make gungeon` et cible du `Makefile` comme `throne` ; `make lint` couvre `games/gungeon`
  (« aucune erreur ») ; `make gen GAME=gungeon` (scénarios générés par arme).
- Scénarios : `gungeon_start` (le clone démarre, joueur vivant, salle de départ), `gungeon_clear`
  (joueur scripté : entre dans la salle de combat, verrouillage, nettoie, réouverture, ramasse un
  objet), `gungeon_boss` (atteint la salle du boss, `BossPhase` 1 puis mort du boss, `RunState`
  victoire). Attentes `RoomState`, `HasItem`, `BossPhase`.

## Contraintes

Aucune trace existante ne bouge. Pas de rendu dans les crates de simulation. Sprites : réutiliser
ceux du testbed ou de throne (pas d'assets nouveaux sans `assets.yaml`).

## Vérification

README §4 (suite complète, tests des crates, lint ×4, fmt, scripts, `make gen` ×4 sans
modification inattendue) ; p2p à deux clients sur `gungeon` si possible (Docker : prévenir orch
avant d'utiliser le port 3536). Rapport `docs/taches/rapports/m2-t0d-squelette-gungeon.md`.
