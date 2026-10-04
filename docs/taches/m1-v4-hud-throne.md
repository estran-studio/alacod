# m1-v4-hud-throne — HUD throne : rads, niveau, munitions par type, statuts (T1.18, voie V4)

Lire d'abord `docs/taches/README.md` (agent **local**, `origin/main` mergé). Petite tâche (2 j),
**présentation seule** : aucune trace ne change. Prérequis : HUD data-driven (`ui/hud.ron`, conventions
§15), T1.10 (`Gauges`, `Level`), T1.3 (`Statuses`), munitions `Custom` de throne.

## Décisions (fixées après proposition de b1)

1. **Nouvelles sources** du HUD : `rads` (fraction vers le prochain seuil, depuis `Gauges` et la
   `Progression`), `level`, `ammo_by_type` (réserve de chaque munition `Custom`), `statuses` (icônes ou
   texte), `floor` ; liste fermée, lint (source inconnue = `UnknownKind`).
2. **Fonction pure** `hud_values(joueur) -> HudPlayerValues` (extraite d'`update_hud_values`), tests
   unitaires sans rendu ; ressource `HudSnapshot` mise à jour aussi en headless.
3. **Attente optionnelle** `HudText { source, contains, at_frame }` lisant `HudSnapshot` (hors
   simulation, hors trace) ; à défaut tests purs seuls.
4. **Contenu** : `games/throne/assets/ui/hud.ron` (barre de rads sous la vie, niveau, munitions par
   type en bas à droite, statuts près de la vie) ; zombies et testbed inchangés.
5. **Preuve** : captures hors écran de `throne_progression` (rads qui montent, niveau 1 puis 2,
   munitions) si le GPU le permet ; sinon tests et `HudText`.

## Règles

Deux compilations au plus ; purge + point d'état ; `docs/conventions.md` : **uniquement** §32 « HUD
throne » (+ une ligne au §15) ; traces : `make test_scenarios` vert sans bless.

## Livrer

Rapport `docs/taches/rapports/m1-v4-hud-throne.md` ; `git push -u origin m1-v4-hud-throne` ;
`SendMessage` à `orch` : `LIVRÉ m1-v4-hud-throne <sha> : <une ligne>`. Ne merge pas.
