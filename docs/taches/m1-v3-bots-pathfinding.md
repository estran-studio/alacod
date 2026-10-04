# m1-v3-bots-pathfinding — `prudent` et `fonceur` naviguent par le flow field (suite T1.14, voie V3)

Lire d'abord `docs/taches/README.md` (agent **local**, branche `m1-v3-bots-pathfinding` depuis la tête
livrée précédente de l'agent). Petite tâche (2 j). Motif : critère throne 17/20 (graines 5, 12, 20 :
bots coincés dans un recoin ou derrière un mur en allant au portail ou vers un ennemi caché), dette
« pathfinding prudent » de T1.14.

## Décisions (fixées ; l'agent amende en dix lignes s'il voit un problème)

1. **Réutiliser `crates/bots/src/navigation.rs`** (Dijkstra multi-source 8 px de `chasseur`/
   `acheteur`, cache par clés) pour `prudent` et `fonceur` : deux buts possibles, **portail ouvert**
   (`BotView.portal`) et **ennemi hors de vue le plus proche** ; le champ donne la direction du pas
   suivant ; la ligne droite reste le repli quand aucun chemin n'existe (dit dans le rapport).
   Esquive (T1.14) et distance de combat de `prudent` restent prioritaires sur la navigation.
2. **Dérivé hors rollback**, comme aujourd'hui (`ReadInputs`, aucun RNG, aucune ressource de jeu) ;
   `bots.rs`/`hunter_doors.rs` restent verts ; aucun scénario existant ne change (bots hors trace),
   sauf `bot_floors_three`/`bot_prudent_*` si leurs inputs changent → preuve §10, bless orchestrateur.
3. **Critères** : `alacod-sim --game throne --bots 2 --profiles prudent --floors run --seeds 1..20
   --until-floor 3 --max-frames 12000` : **20/20** (ou écarts expliqués graine par graine, avec le
   `SoftlockDump`) ; testbed `trois_niveaux` 20/20 conservé ; `bench_horde` inchangé (les bots ne
   tournent pas dans les benchs).
4. **Hors périmètre** : esquive des zombies au contact, bots dans le jeu fenêtré, nouveaux profils.

## Règles

Deux compilations au plus ; purge + point d'état ; `docs/conventions.md` : un paragraphe au §24 v1 ;
`docs/taches.md` et `dettes.md` : ne pas toucher. Merger `origin/main` avant de livrer.

## Livrer

Rapport `docs/taches/rapports/m1-v3-bots-pathfinding.md` ; `git push -u origin m1-v3-bots-pathfinding` ;
`SendMessage` à `orch` : `LIVRÉ m1-v3-bots-pathfinding <sha> : <une ligne>`. Ne merge pas, ne bénis pas.
