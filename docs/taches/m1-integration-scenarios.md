# m1-integration-scenarios — vague 2 de M1 : scénarios du clone à 1, 2 et 4, boss simple, vidéos

Lire d'abord `docs/taches/README.md` (agent **local**, `origin/main` mergé). Tâche moyenne (3 j),
**données et scénarios** ; aucun code d'engine sauf manque avéré (alors `BLOQUÉ` ou dette). Plan §6
vague 2 : « Scénarios du clone (trois niveaux à 1, 2 et 4 ; un boss simple par timeline), bots sur 200
graines, vidéos, revue humaine, fermeture des notes. » Les 200 graines et la revue humaine sont à
l'orchestrateur et à William.

## Décisions (fixées ; l'agent amende en dix lignes s'il voit un problème)

1. **Scénarios du clone** : `throne_solo` (1 bot `prudent`), `throne_duo` (2, = `throne_three_floors`
   actuel renommé ou gardé tel quel : garder, dire lequel), `throne_quad` (4 bots `prudent`), tous sur
   `floors/run.ron`, graine fixée, `FloorIndex(3)` avant f9000 (solo : f12000), `PlayerAlive` pour chacun,
   `RunSummary(floor_reached_min: 3)`, `Event(levelup)` ≥ 1, `Clock alerte fired`. La suite les joue en
   synctest (preuve de déterminisme à 1, 2 et 4). Trois traces (bless orchestrateur).
2. **Boss simple, données seulement** : personnage `roi_rat` (santé ≈ 12× un rat, `[Shoot(couronne),
   Charge, Chase]`, variante `champion` imposée impossible sans LDtk → tag `champion` direct dans
   `tags`), placé **une fois** au niveau 3 : `CaveConfig.characters` de `niveau_3` reçoit `"roi_rat"` et
   `enemy_spawns` monte d'un ; si la caverne ne permet pas « exactement un exemplaire », noter la dette
   et le dire. Le portail du niveau 3 exige sa mort (`EntityCount(enemy) == 0`) : la boucle infinie
   recharge le niveau 3 avec un nouveau boss (comportement `Floors`, accepté). Attente `EntityHealth`
   du boss dans `throne_quad` ; `test:` sur `roi_rat` (gabarits).
3. **Vidéos** : `make videos SCENARIO=throne_solo,throne_duo,throne_quad` et `make views
   SCENARIO=throne_quad` ; copier les mp4 ≤ 5 Mo et leurs `events.json` dans `docs/digests/videos/` (comme
   T3.2) ; une image de chaque dans le rapport.
4. **Digest brouillon** : `docs/digests/m1-fin-de-vague-2.md` sur le modèle de `m0-fin-de-vague-2.md`,
   avec les tableaux « Critères de sortie » et « Ce qui manque », les métriques 20 graines ; la ligne
   200 graines reste « à remplir par l'orchestrateur ».
5. **Hors périmètre** : phases de boss (D4, M2), code d'engine, 200 graines, revue humaine.

## Règles

Deux compilations au plus ; purge + point d'état ; aucune trace bénie ; `docs/conventions.md` : une ligne
au §29 (boss) ; `taches.md`/`dettes.md` : ne pas toucher. Traces existantes intactes (`niveau_3` change →
`throne_three_floors`/`throne_progression` peuvent bouger : preuve §10, dire la cause).

## Livrer

Rapport `docs/taches/rapports/m1-integration-scenarios.md` ; `git push -u origin m1-integration-scenarios` ;
`SendMessage` à `orch` : `LIVRÉ m1-integration-scenarios <sha> : <une ligne>`. Ne merge pas, ne bénis pas.
