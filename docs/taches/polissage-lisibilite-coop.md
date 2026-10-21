# polissage-lisibilite-coop — couleur par joueur, halo et légende (B5)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 3), `crates/game/src/legibility.rs`
(couleur par handle `player_colors`, halo des ramassables, légende), `docs/conventions.md` §83, `games/gungeon/assets/ui/hud.ron`
(le seul clone avec `player_colors` et `legend` : 1 et 3 occurrences, grep du 2026-10-21),
`docs/taches/rapports/m3-salles-murs-isaac.md` (D123), `docs/taches/plan-assets.md`. Branche `polissage-lisibilite-coop`
depuis `origin/main`. Estimation : 2 j. Clones : zombies, throne, isaac (gungeon : contrôle). Machine : Mac avec
affichage ; **pas de VPS** (captures).

## À faire

1. Reprendre dans `hud.ron` de zombies, throne et isaac les entrées `player_colors`, `legend` et les halos de ramassables
   que gungeon a déjà ; adapter les catégories à chaque clone (isaac : cœurs, clés, bombes, objets ; zombies : armes
   murales, power-ups). Données seulement : un jeu qui n'ajoute pas l'entrée garde son rendu.
2. Si `legibility.rs` suppose des classes propres à gungeon, généraliser **sans changer le rendu de gungeon** (capture
   avant/après identique).
3. Capture à 2 puis 4 joueurs par clone (`play_scenario --capture`, `--follow`), l'ennemi qui tire distinct de l'ennemi qui
   marche (D66 pour throne : seul le contrôle, la correction est B7), pointeur de visée là où il manque (isaac, D123).
4. Pas de nouveau sprite ici (P5) : couleurs, teintes, halos, légende.

## Acceptation

- Pour chaque clone, une capture à 4 joueurs où chaque héros se distingue par la couleur de son handle ; F7 affiche une
  légende lisible.
- Présentation seule : aucune trace ne bouge (`make test_scenarios` sans `BLESS`).
- Les captures dans `docs/captures/polissage-lisibilite-coop/`, rapport `docs/taches/rapports/polissage-lisibilite-coop.md`
  avec les limites restantes (D123).
