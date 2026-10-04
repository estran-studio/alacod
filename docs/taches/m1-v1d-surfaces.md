# m1-v1d-surfaces — surfaces v1 : tags de cellules et vitesse (T1.7, chantier E4 v1)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche `m1-v1d-surfaces`
créée depuis la tête livrée de T1.6 `m1-v1d-terrain-destructible`, même target). **Prérequis** :
T1.6 mergée ou livrée (crate `world`, `CellKind`, `CellGrid` neutre, `MapGenerationMode::Cave`,
`CellState`, conventions §21). Petite tâche (2 j), voie V1d monde.

## Contexte

Plan §6 : « T1.7 Surfaces v1 (E4) — tags de cellules et modificateurs de vitesse (eau peu
profonde, sable). Acceptation : `PlayerPosition` sur deux surfaces. » Plan-engine E4 :
« Surfaces et dangers (M). Tags de cellules (boue, glace, neige, eau peu profonde, huile),
effets sur mouvement et esquive ; piques, fosses, barils, feu. » — v1 = **mouvement seulement**.
Plan-engine §4 : les surfaces agissent par **modificateurs de stats** (B2, conventions §9 :
`Modifier { stat, op, value, source }`, `Modifiers::remove_by_source`, `StatReader`), pas par
du code spécial dans le déplacement.

Ce qui existe après T1.6 : `world::CellKind { Floor, Wall, Rock }`, `CellGrid` (case de 16,
ressource rollback neutre), `Destructible`, set `RollbackSystemSet::World`, `CellState`. Les
personnages bougent dans `move_enemies` (ennemis) et `apply_inputs`/mouvement joueur (vitesse
depuis les stats : `move_speed` via `StatReader`, dash). Les cartes LDtk ordinaires laissent
`CellGrid` vide.

## Décisions (fixées ici après revue avec b1, pas à réinventer)

1. **Données** : ressource `world::SurfaceGrid`, **creuse** (`BTreeMap<(i32, i32), SurfaceId>`
   en cases de grille monde de 16 — depuis le snap de m0-v7, case LDtk = case de grille
   monde, et l'origine d'une carte LDtk n'est pas (0,0)), **rollback + checksum neutre** (vide =
   0 : traces existantes inchangées), séparée de `CellGrid`/`CellKind` ; compatible destruction
   (une case `Rock` creusée garde sa surface ou n'en a aucune). `SurfaceId` = `u8` non nul ;
   table `SurfaceId → nom` hors rollback, issue du contenu.
2. **Source v1 : LDtk seulement** — couche IntGrid optionnelle `Surfaces` dans les `.ldtk`
   (valeur IntGrid = `intgrid_value` du kind), lue au chargement comme `Walls` ; une carte sans
   la couche ne touche pas la ressource. Cavernes : champ optionnel `surfaces` de `CaveConfig`
   = **v2**, hors périmètre (mentionné au § conventions).
3. **Contenu** : kind `Surface` (`surfaces/<id>.ron`, déclaré par `game.ron`) :
   `( intgrid_value: 1, tags: ["eau"], move_speed: "0.5", acceleration: Some("1.0") )` —
   `move_speed` et `acceleration` sont des **facteurs abstraits** (`Mul`) que le système traduit
   vers la bonne stat selon le personnage (`MoveSpeed` joueurs / `EnemyMoveSpeed` ennemis ;
   accélération idem), une seule ligne à maintenir. Glace = `acceleration: "0.2"` (le joueur
   glisse). Lint : `intgrid_value` unique et > 0, facteurs > 0, tags non vides.
4. **Application** : système dans `RollbackSystemSet::Input` (avant `Movement`),
   `order_mut_iter!` par `GgrsNetId`, pour joueurs et ennemis : la case sous les pieds = centre
   du collider avec son offset ; si la surface change, `Modifiers::remove_by_source
   (Named("surface"))` puis pose des modificateurs de la nouvelle surface **sans `until`** ; rien
   n'est écrit quand elle ne change pas. Effet immédiat dans la frame ; `Modifiers` est déjà
   rollback → **aucun nouvel état** hors `SurfaceGrid`. Un personnage hors surface n'est jamais
   touché (traces intactes). `Flying` (`NavProfile`) ignore les surfaces (§ conventions).
5. **Esquive, friction, navigation** : **v2**, hors périmètre — le dash reste inchangé, pas de
   friction, pas de coût de flow field par surface (à mentionner au § conventions comme suite).
6. **Attentes** : `PlayerPosition` existante ; `CellState` de T1.6 étendue d'un champ optionnel
   `surface: Some("eau")` (pas une nouvelle attente).
7. **Contenu et scénarios testbed** : surfaces `eau` (×0,5), `sable` (×0,8), `glace`
   (accélération ×0,2) ; carte `testbed/surfaces.ldtk` (copie d'`arena` + couche `Surfaces` :
   bandes sol → eau → sable → glace en ligne) ; scénarios : `surface_walk` (le joueur marche à
   droite 300 frames à travers les bandes : `PlayerPosition` aux frames de sortie de chaque
   bande, distances mesurées, rapports de vitesse ≈ facteurs à ± 1 px), `surface_none` (même
   input hors bandes, référence), `surface_ice` (changement de direction sur la glace :
   `PlayerPosition` montre la glissade), `surface_enemy` (un `grunt`/`dummy` mobile traversant
   l'eau : `EnemyDistance`/`EnemyContactBefore` plus tard que sur sol). Quatre nouvelles traces
   (bless orchestrateur).
8. **Hors périmètre** : dangers (piques, fosses, barils, feu : E4 v2 / E5), neige/boue (contenu
   1837, M3), cavernes avec surfaces, dash/friction/navigation (v2), rendu au-delà du minimum
   (les tuiles de la couche LDtk se dessinent déjà).

## Traces attendues

Aucune trace existante ne change : `SurfaceGrid` vide (neutre) sur toute carte sans couche
`Surfaces`, aucun modificateur posé hors surface, `CellGrid` de T1.6 intacte. Critère central,
vérifiable par `make test_scenarios` vert sans bless.

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ;
aucune trace bénie par l'agent ; `docs/conventions.md` : **uniquement** §26 « Surfaces » (+ une ligne
dans ta §21 renvoyant à `SurfaceGrid`) ; `CLAUDE.md` : champ `surface` de `CellState` ; `docs/taches.md` : ne pas toucher.
Merger `origin/main` juste avant de livrer ; conflits : garder les deux.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : lecture de la couche LDtk (origine non nulle), case sous les pieds
   (collider + offset), changement de surface = modificateurs remplacés exactement (source
   `surface`), traduction `move_speed` → `MoveSpeed`/`EnemyMoveSpeed`, `Flying` ignore,
   `SurfaceGrid` neutre à vide, lint (`intgrid_value` dupliquée, facteur ≤ 0).
2. `surface_walk`, `surface_none`, `surface_ice`, `surface_enemy` verts ; **tous les scénarios
   existants verts sans trace modifiée**.
3. `bench_horde` et `bench_cave` dans leur budget (le système ne coûte qu'au changement de
   case) ; chiffres sous charge acceptés, strict si la machine est calme.
4. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` sans
   modification ; exemples racine compilés.
5. §26 écrit ; rapport honnête avec point d'état.

## Livrer

Rapport `docs/taches/rapports/m1-v1d-surfaces.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). `git push -u origin m1-v1d-surfaces`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v1d-surfaces <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
