# m4-t0f-squelette-isaac — contenu `games/isaac/` squelette (M4, vague 0, V2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §3
(dossier de jeu, manifeste), §17 et §40 (`Floors`, grammaire d'étage), §35 (salles), §39 (le jeu `gungeon` :
modèle de squelette), et `games/gungeon/` (manifeste, `src/main.rs`, `assets/`), le rapport
`docs/taches/rapports/m2-t0d-squelette-gungeon.md` et son script `m2-t0d-squelette-gungeon.make_etage.py`.
Branche `m4-t0f-squelette-isaac` depuis `origin/main`. Estimation : 1 à 2 j. Section de conventions
réservée : **§65**.

## But

Le clone `isaac` démarre : un crate `games/isaac` (membre du workspace par `games/*`), un manifeste, un
personnage, une arme de larmes, un ennemi, trois gabarits de salle (départ, combat, boss), un étage par la
grammaire en arbre **existante**. Rien de nouveau côté moteur.

## À faire

1. **Crate** `games/isaac` : `Cargo.toml` et `src/main.rs` sur le modèle de `games/gungeon` (mêmes plugins,
   mêmes features) ; classer le crate dans `scripts/check-sim-boundary.sh` comme les autres jeux (`APP`).
2. **Manifeste** `assets/game.ron` : `name: "isaac"`, dossiers `characters`, `weapons`, `objets`, `loot`,
   `rooms`, `floors`, `floor_grammars`, `maps`, `ui`, `camera.ron`, `sprites` ; `entry` en `Floors` sur une
   séquence `sous_sol` (nom provisoire, **Q9**) d'un étage.
3. **Personnage** : un joueur à vie basse (valeurs en données ; modèle des cœurs : **Q2**, prendre 6 PV en
   attendant), une arme de larmes. Tant que M4-T0a n'est pas mergée : `Automatic` avec un chargeur très grand
   et sans rechargement visible (le dire en commentaire RON) ; passer à `Infinite` dans une fiche suivante
   (T11a), pas ici.
4. **Ennemi** : un suiveur simple (`Chase`) et une mouche volante (`Flying`) si le profil existe pour le
   contenu ; un boss à deux phases réutilisant le vocabulaire de §37/§45.
5. **Gabarits** : `maps/salles.ldtk` produit par un script Python versionné
   (`docs/taches/rapports/m4-t0f-squelette-isaac.make_salles.py`, sur le modèle de M2-T10) : trois salles de
   **même taille**, une ouverture centrée sur chacun des quatre côtés (préparation de la grille de M4-T5 ;
   taille proposée 15×9 cases, à confirmer : **Q8**). Types `depart`, `combat`, `boss`.
6. **Grammaire** `floor_grammars/sous_sol.ron` : `rooms: (5, 7)`, boss requis, en `Graph` (le planificateur
   actuel). Doit être valide sur 1 000 graines (test de M2-T10 étendu au jeu `isaac`).
7. **Sprites, sons, UI** : copiés de `gungeon` (placeholders, `assets.yaml` à jour) ; aucune ressource sous
   licence non libre (voir `docs/plan-assets.md`), aucun nom ni visuel tiré du jeu commercial (**Q9**).
8. **Scénarios** : `isaac_boots` (le clone démarre, le joueur tire, `PlayerAlive` à f300),
   `isaac_floor_seed_1` (un étage complet par un bot `explorateur` : boss vaincu, portail ; ou, si le bot ne
   le finit pas, attente de salle nettoyée et la raison au rapport).
9. **Make** : `make lint GAME=isaac`, `make gen GAME=isaac` ; ajouter `isaac` aux boucles « ×4 » du README et
   de la CI (`.github/`, `Makefile`) si elles listent les jeux en dur : ce sont des fichiers V3, le faire dans
   cette tâche avec un commit séparé et le dire au rapport.

## Règles

Données et un `main.rs` seulement. Les traces des autres jeux ne bougent pas. `make lint GAME=isaac` vert ;
l'avertissement d'atteignabilité éventuel est listé au rapport.

## Hors périmètre

Grille (M4-T5), larmes infinies (T0a/T1), rochers, bombes (T4), objets (T11), HUD propre (T16).

## Vérification

README §4 (`make test_scenarios` complet, `make test_crates`, `make lint` ×5, fmt, scripts, `make gen` ×5,
`make check_sim_boundary`) ; rapport `docs/taches/rapports/m4-t0f-squelette-isaac.md`, conventions §65.

## Atteignabilité

`isaac_floor_seed_1` est la première partie ordinaire du clone ; toutes les fiches suivantes s'y réfèrent.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : le p2p `isaac` à deux clients est rejoué par l'orchestrateur, le dire au rapport. Pousser la branche
régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ m4-t0f-squelette-isaac`, qui contient le rapport.
