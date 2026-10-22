# polissage-zombies-visuel — zombies visuel complet (S1-8)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axes 3 et 9, S-0),
`docs/taches/plan-assets.md`, `docs/conventions.md` §83 (langage visuel), `crates/game/src/legibility.rs`,
`games/gungeon/assets/ui/hud.ron` (`wall_art`, `player_colors`, `legend` : le modèle), `docs/taches/rapports/m2-salles-murs-gungeon.md`,
`games/zombies/assets/assets.yaml`, `character/visuals.rs:183-200,331-380`. Branche `polissage-zombies-visuel` depuis
`origin/main`. Estimation : 2 j. **Dépend de S-0** : pas d'intégration avant le choix du pack (J2) ; l'audit se fait en attendant. Clone : zombies. Machine : **Mac** (captures).

## Contexte (inventaire §3, §9, §10)

Zombies n'a ni `wall_art`, ni `player_colors`, ni `legend` dans `hud.ron` ; ses tilesets sont SunnyLand/Dungeon, partagés
avec les trois autres jeux ; le statut propre/emprunté de ses sprites n'est pas recensé ; le nombre d'animations idle/marche/
attaque/mort par entité n'est pas trouvé.

## À faire

1. **Audit chiffré** (0,5 j) : tableau « classe d'entité × état (propre / emprunté / absent) × animations présentes »
   pour joueur, chaque famille d'ennemi, boss, projectiles, armes, ramassables, portes, fenêtres, murs, sol, props, portail,
   HUD. Premier livrable du lot ; il fixe la suite.
2. Sol et murs propres : `wall_art` de zombies (même format que gungeon) et tilesets propres au clone ; fenêtres et portes.
3. Joueur et ennemis : sprites propres au jeu, animés idle/marche/attaque/mort (**issus du pack choisi par William en S-0**, `polissage-pack-assets.md` ; licences dans `assets.yaml`) ; seulement ce que l'audit montre manquant.
4. Lisibilité coop reprise de gungeon : `player_colors`, `legend` (F7), halos des ramassables (armes murales, power-ups).
5. Captures à 1, 2 et 4 joueurs et vidéo de `play_scenario --capture`.

## Acceptation

- Tableau d'audit avant/après dans le rapport, aucune case « emprunté » sur les classes principales ; captures et vidéo.
- Présentation seule : `make test_scenarios` sans `BLESS`.
- Rapport `docs/taches/rapports/polissage-zombies-visuel.md` avec ce qui reste (et son estimation).
