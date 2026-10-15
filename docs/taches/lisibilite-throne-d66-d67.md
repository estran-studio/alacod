# lisibilite-throne-d66-d67 — tourelles lisibles, télégraphe de salve en directions de tir

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Simulation et présentation »), les lignes D56, D57,
D66 et D67 de `docs/taches/dettes.md`, le rapport `docs/taches/rapports/m1-revue-humaine.md`
(constats de la 3ᵉ run), `docs/conventions.md` §31 et §55 (feedback, télégraphes de pattern),
`crates/game/src/feedback.rs` (`pattern_shape`), `games/throne/assets/characters/tourelle.ron`,
`roi_rat.ron`. Branche `lisibilite-throne-d66-d67` depuis `origin/main`. Estimation : 1,5 j.

## Contexte

Revue M1 de William : des « zombies coincés qui ne regardent pas » (le cas diagnostiqué est une
**tourelle** stationnaire, avec une silhouette de zombie et une orientation figée) ; des « immenses
cercles rouges » (télégraphe de la salve en couronne, `Ring`, sans forme dédiée dans M2-T19).

## À faire

1. **D67, télégraphe `Ring`** : `pattern_shape` dessine pour une couronne des **rayons courts autour
   de l'émetteur**, dans les directions de tir de la salve (même calcul que les projectiles : angle
   de départ, nombre, pas). C'est de la présentation seule : ni les dégâts ni la portée ne changent.
   - Fonction pure testée.
   - Moment clé `feedback` (« télégraphe ring 220 (8 rayons) »).
   - Attente `Event` dans `enemy_roi_rat_still` ou un scénario throne qui montre la salve.
2. **D66, lisibilité des stationnaires** :
   - l'orientation visuelle de la tourelle suit sa cible (`FacingDirection` côté présentation si
     possible ; si c'est de l'état rollback, preuve §5) ;
   - un marqueur visuel de tourelle, en données dans `tourelle.ron` (teinte, échelle ou sprite de
     remplacement), sans asset nouveau obligatoire.
   - Ne pas toucher au blocage des ennemis mobiles : William doit d'abord localiser les autres cas.
3. Dire au rapport les vidéos à regarder (`make videos SCENARIO=…`, frames) pour William.

## Règles

Aucune trace ne doit bouger, sauf justification avec preuve. Les crates de simulation n'importent
rien de rendu. `cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud : ignorer les consignes `orca` du README (§1) ; pas
de GPU ni de Docker (captures en rendu logiciel). Pousser la branche régulièrement. Voie rapide du
README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ lisibilite-throne-d66-d67`, avec le rapport `docs/taches/rapports/lisibilite-throne-d66-d67.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
scripts, `make gen` ×4).
