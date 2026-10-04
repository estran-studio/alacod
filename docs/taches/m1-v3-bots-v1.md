# m1-v3-bots-v1 — bots v1 : esquive des projectiles, niveaux, `--until-floor` (T1.14, voie V3)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de l'agent qui la prend, branche
`m1-v3-bots-v1` créée depuis `origin/main`, même target que sa tâche précédente). **Prérequis** :
T1.8 `Floors` (mergée), T1.2 tir ennemi (mergée : les ennemis tirent, `arena_tir`, scénario
`enemy_ring`), m0-v7 phase 2 (bots `chasseur`/`acheteur`, conventions §24). Petite tâche (3 j).

## Contexte

Plan §6 : « T1.14 Bots v1 — profil `prudent` qui esquive les projectiles, complétion de niveau ;
`alacod sim` avec `--until-floor` ; métriques de niveau. » Conventions §17 : « Pas d'arrêt
`--until-floor` dans `alacod-sim` (T1.14) » ; `BotView::portal` existe (un bot `fonceur`/`prudent`
sans ennemi visible marche vers le portail ouvert, ligne droite).

Ce qui existe : `crates/bots` — `decide(profile, view, rng) -> BoxInput` pure en `Fixed`
(`decide.rs`, `prudent` = tient 180–320 px et tire), `BotView { position, health, health_max,
ammo, wave, nearest_enemy, nearest_window, hunter, portal }` (`view.rs`, dérivée dans
`ReadInputs`, hors rollback), `hunter.rs`/`navigation.rs` (flow field 8 px, zombies) ;
`alacod-sim` (`--bots`, `--profiles`, `--seeds`, `--until-wave`, `--max-frames`, `--map`,
`--floors`, `--json`, `--progress`), `SimResult { seed, wave, floor, frames, deaths, kills,
wall_seconds, desync, sim_fps, failures, softlock }`, `scripts/scenario-metrics.py` ; projectiles
`combat::Projectile` + `Velocity` (`actors.rs` l.39), équipe via `Team` ; `floors/trois_niveaux.ron`
du testbed ; **aucun scénario de `tests/scenarios/` n'utilise `bot: Some(..)`** : changer les
décisions des profils ne déplace aucune trace (les tests `crates/scenario/tests/bots.rs` et
`hunter_doors.rs` vérifient le rejeu, à garder verts).

## Décisions (proposées par l'orchestrateur ; l'agent confirme ou amende en dix lignes avant de coder)

1. **Vue** : `BotView.projectiles: Vec<ProjectileView { position, velocity, size, distance }>` —
   projectiles d'une équipe **autre** que celle du bot, à moins de 320 px, triés par `GgrsNetId`
   (`nearest_by_net_id`), au plus 16 (les plus proches). Dérivée comme le reste de la vue : aucun
   état caché, aucun RNG.
2. **Esquive** (`prudent` v1, fonction pure `dodge(view) -> Option<FixedVec2>` dans
   `crates/bots/src/dodge.rs`, testée sans Bevy) : pour chaque projectile, point d'approche
   minimale sur sa trajectoire linéaire dans les `k = 30` prochaines frames ; menace si cette
   distance < rayon du corps + `size` + marge 8 px ; direction d'esquive = perpendiculaire à la
   vitesse du projectile, du côté qui éloigne, somme sur les menaces, normalisée (`Fixed`). Quand
   une esquive existe, elle **remplace** le déplacement de `prudent` (tir conservé : `pan` vers
   l'ennemi le plus proche) ; sinon `prudent` v0 inchangé. `fonceur`, `immobile`, `chasseur`,
   `acheteur` : **inchangés**.
3. **Complétion de niveau** : `prudent` et `fonceur` marchent déjà vers le portail sans ennemi
   visible ; v1 ajoute : ennemis **restants hors de vue** (`EntityCount(enemy) > 0` et
   `nearest_enemy == None`) → marcher vers l'ennemi le plus proche en ligne droite (`BotView.
   nearest_enemy_any: Option<EnemyView>`, sans limite de portée) ; les niveaux du testbed sont des
   salles ouvertes, **pas de pathfinding** (le flow field 8 px des zombies reste réservé à
   `chasseur`/`acheteur`, à mentionner comme suite).
4. **`alacod-sim --until-floor <n>`** : arrêt quand `FloorState::index ≥ n` (exige `--floors`,
   erreur sinon) ; `--until-wave` reste obligatoire pour `Waves` (l'un ou l'autre selon le mode,
   usage mis à jour). `SimResult` gagne `floor_frames: Vec<u32>` (frame de chaque passage de
   portail, vide hors `Floors`, `skip_serializing_if`), `damage_taken: u32` (total joueurs),
   `dodges: u32` (frames où une esquive a remplacé le déplacement d'au moins un bot ; compté dans
   le bot, hors simulation). `scripts/scenario-metrics.py` : colonnes `floor` et `frames/niveau`
   quand une graine a `floor > 0`, `dodges` quand non nul ; résumé « niveaux finis / graines ».
5. **Soft-lock en `Floors`** : le `SoftlockDump` existant se déclenche aussi quand aucun passage de
   portail ni kill n'a eu lieu depuis 1 200 frames (même champ `softlock`).
6. **Contenu et scénarios testbed** : `floors/trois_niveaux.ron` existe ; vérifier que chacun de ses
   niveaux a au moins un ennemi atteignable, sinon ajouter `testbed/floor_c.ldtk` ; scénarios :
   `bot_prudent_dodge` (`arena_tir`, tireur `Ring` de T1.2, un joueur `bot: Some(Prudent)` : `Health`
   à f600 ≥ celle du scénario jumeau `bot_prudent_nododge` enregistré avec les inputs v0
   figés en `Scripted`, et `NoDamageBetween` sur une fenêtre de 300 frames à préciser d'après la
   mesure), `bot_floors_three` (`floors: Some("trois_niveaux")`, deux bots `prudent` :
   `FloorIndex(2)` avant f6000, `PlayerAlive` ×2). Deux à trois nouvelles traces (bless
   orchestrateur). **Critère du plan** : `alacod-sim --game testbed --bots 2 --profiles prudent
   --floors trois_niveaux --seeds 1..20 --until-floor 3 --max-frames 12000` : 20/20 finissent
   trois niveaux, 0 desync, 0 softlock — chiffres dans le rapport.
7. **Hors périmètre** : esquive des zombies (contact), pathfinding pour `prudent`, profils
   nouveaux, bots dans le jeu fenêtré, métriques vidéo, T1.13 générateur.

## Traces attendues

Aucune trace existante ne change (bots hors rollback, aucun scénario existant avec bot,
`SimResult` hors simulation). Vérifiable par `make test_scenarios` vert sans bless.

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ; aucune
trace bénie par l'agent ; `docs/conventions.md` : **uniquement** un sous-titre « v1 (T1.14) » à la
fin du §24 et la ligne « Pas d'arrêt `--until-floor` » du §17 à mettre à jour ; `CLAUDE.md` : rien
sauf si une attente est ajoutée ; `docs/taches.md` : ne pas toucher. Merger `origin/main` juste
avant de livrer ; conflits : garder les deux.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : `dodge` (projectile frontal → pas de côté ; projectile qui passe à côté →
   `None` ; deux menaces opposées → somme), `ProjectileView` filtrée par équipe et triée,
   `nearest_enemy_any`, `--until-floor` sans `--floors` refusé, `floor_frames`.
2. Scénarios `bot_prudent_dodge`, `bot_floors_three` verts ; **tous les scénarios existants verts
   sans trace modifiée** ; `bots.rs` et `hunter_doors.rs` verts.
3. Critère 20 graines du point 6 atteint ou écart expliqué graine par graine.
4. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` sans
   modification ; exemples racine compilés.
5. §24 v1 écrit ; rapport honnête avec point d'état.

## Livrer

Rapport `docs/taches/rapports/m1-v3-bots-v1.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). `git push -u origin m1-v3-bots-v1`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v3-bots-v1 <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
