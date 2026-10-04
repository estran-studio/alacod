# m1-d39-glissement-ennemis — `move_enemies` : glissement X puis Y depuis la position obtenue (D39)

Lire d'abord `docs/taches/README.md` (agent **local**, branche `m1-d39-glissement-ennemis` depuis la
tête livrée précédente). Petite tâche (1 j), **déplace des traces** : la preuve §10 est l'essentiel.

## Contexte

Trouvé par T1.13 (`enemy_kiter_still`, f544) et diagnostiqué dans m1-dettes-lot-1 : `move_enemies`
teste « X seul » puis « Y seul » **chacun depuis la position de départ** ; deux mouvements libres
séparément se combinent dans un coin de mur. `move_characters` (joueurs) a déjà le correctif (§24 :
« glisser X, puis vérifier Y à la position X obtenue »).

## Décisions (fixées)

1. **Correctif** : même règle que `move_characters` dans `move_enemies` (et tout autre chemin de
   déplacement d'ennemi : charge, recul de `KeepDistance`, `Flee`, steering de `pathing.rs` si
   séparé) : X d'abord, puis Y **depuis la position X obtenue**, puis repli Y seul si X bloqué.
   Fonction partagée (`combat`/`sim_core`) utilisée par les deux chemins, testée unitairement
   (coin de mur : plus aucune pénétration ; couloir droit : résultat identique à avant).
2. **Preuve §10** : dumps main contre branche pour **tous** les scénarios dont la trace change,
   `scripts/trace-diff.py` ; pour chacun, la première frame qui diffère et la cause en une ligne
   (un ennemi qui touchait un coin). Les scénarios sans ennemi près d'un coin doivent rester
   **identiques** (liste). `enemy_kiter_still` : `EnemyNeverInWall` étendu à `frames`.
3. **Critère M0** : `alacod-sim --game zombies --bots 4 --seeds 1..20 --until-wave 5 --max-frames
   20000` avant/après : vague 5 atteinte sur les mêmes graines, 0 desync, 0 softlock (tableau).
4. **Hors périmètre** : navigation, pathfinding, autres dettes.

## Règles

Deux compilations au plus ; purge + point d'état ; aucune trace bénie par l'agent ; `docs/conventions.md` :
une ligne au §24 (« les ennemis glissent comme les joueurs ») ; `dettes.md` : ne pas toucher.
Merger `origin/main` avant de livrer.

## Livrer

Rapport `docs/taches/rapports/m1-d39-glissement-ennemis.md` avec la preuve ; `git push -u origin
m1-d39-glissement-ennemis` ; `SendMessage` à `orch` : `LIVRÉ m1-d39-glissement-ennemis <sha> : <une
ligne>`. Ne merge pas, ne bénis pas.
