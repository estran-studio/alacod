# m1-v2-contenu-throne — `games/throne/` : squelette puis contenu du clone (T1.0c + T1.11, voie V2)

Lire d'abord `docs/taches/README.md` (agent **local**, branche `m1-v2-contenu-throne` créée depuis
`origin/main` ou la tête livrée de la tâche précédente, même target). **Prérequis** : T1.2, T1.4,
T1.5, T1.6, T1.7, T1.8, T1.14 (mergées). T1.9 horloges et T1.10 effets/mutations sont **en cours**
(b0, b1) : la **phase 1** de cette tâche ne les utilise pas ; la **phase 2** (mutations, horloge)
commence quand elles sont dans main. Tâche moyenne (5 j au total), **données seulement** : aucun
code d'engine. Un manque de l'engine = `BLOQUÉ` ou dette, jamais un contournement.

## Contexte

Plan §6 : « T1.0c Contenu `games/throne/` squelette : `game.ron`, un personnage, une arme, un ennemi,
une caverne fixe : le clone démarre vide. » « T1.11 Contenu `throne` — douze armes sur cinq
munitions, dix ennemis (dont trois tireurs et un chargeur), huit mutations, trois niveaux de
caverne, tables de butin, `test:` sur chaque définition. Acceptation : lint vert, scénarios générés
verts. » Plan-engine M1 : « Cavernes destructibles, projectiles ennemis, deux armes et cinq
munitions, niveaux et mutations, portail, run seedée, horloge de difficulté. » `games/throne/`
**n'existe pas**. Modèles : `games/testbed/assets/game.ron` (kinds `Character`, `Weapon`,
`MeleeWeapon`, `Map`, `Ui`, `Camera`, `SpriteSheet`, `Economy`, `PowerUp`, `Floors`, `Cave`,
`Surface`, `Pattern`) et `games/zombies/` (jeu complet). Cartes de caverne : `cave:<id>` comme chemin
(§21), séquence `Floors` (`floors/<id>.ron`, §17), `alacod-sim --game throne --floors <id>
--until-floor 3`.

## Décisions (proposées par l'orchestrateur ; l'agent confirme ou amende en dix lignes avant de coder)

1. **Phase 1 — squelette (T1.0c, 1 j)** : `games/throne/{Cargo.toml, src/main.rs}` copiés de
   `zombies` (nom, `CARGO_MANIFEST_DIR`), membre du workspace, cible `make throne` ; `assets/game.ron`
   avec `entry: (start_map: "cave:niveau_1", default_seed: 123456, mode: Floors)` et `floors/run.ron`
   = `["cave:niveau_1", "cave:niveau_2", "cave:niveau_3"]` ; un personnage joueur (`pilote`, copie de
   `player` du testbed), une arme (`revolver`, munition `balles`), un ennemi (`rat`, `[Chase, Melee]`),
   une caverne `niveau_1` (`CaveConfig` avec `characters: ["rat"]`), `economy.ron` minimal, `ui`,
   `camera.ron`, sprites **réutilisés** du testbed/zombies (placeholders, V6 fournira les vrais :
   `assets.yaml` à mettre à jour). `make lint GAME=throne` vert, `alacod-sim --game throne --bots 1
   --profiles prudent --floors run --seeds 1..3 --until-floor 2 --max-frames 6000` finit.
2. **Phase 1 — contenu (T1.11 sans mutations, 3 j)** : **cinq munitions** (`balles`, `obus`,
   `explosifs`, `energie`, `lames`) ; **douze armes** réparties (≥ 2 par munition) en RON avec
   projectiles composables T1.1 (`Bounce`, `Pierce`, `Size`, `Lifetime`, `Homing`, `Gravity`,
   `on_expire: Spawn`) et patterns T1.2 ; chacune avec `test:` (`frames`, `min_hits`) ; **dix ennemis**
   en behaviors T1.4 : trois tireurs (`Shoot` avec patterns `Aimed`, `Spread`, `Ring`), un chargeur
   (`Charge`), un kiter (`KeepDistance`+`Shoot`), un fuyard (`Flee`), quatre mêlée (`Chase`/`Wander`),
   variantes T1.5 sur trois d'entre eux (`rapide`, `blinde`), `test:` sur chacun (si T1.13 est dans
   main ; sinon noter) ; **trois niveaux de caverne** (`niveau_1..3`, taille et `enemy_spawns`
   croissants, `characters` par niveau) ; **tables de butin** = `items/powerups.ron` (drop à la mort,
   §14) avec `max_ammo`-like par munition, et `weapon_pool` par niveau **seulement en phase 2**
   (vient de T1.10) ; surfaces : une couche `Surfaces` est impossible dans une caverne générée (v2,
   §26) → aucune surface en phase 1, dit dans le rapport.
3. **Phase 2 (après T1.9 et T1.10 dans main, 1 j)** : huit mutations (`mutations/*.ron`, effets v1),
   `progression.ron` (rads par kill, niveaux, `weapon_pool` par niveau), `entry.clocks`/
   `entry.difficulty` (santé et dégâts ennemis qui montent avec l'étage), tests et scénarios de T1.10/
   T1.9 réutilisés sur `throne`.
4. **Scénarios** : `throne_floor_1` (bot `prudent` finit le niveau 1 : `FloorIndex(1)`), `throne_three_
   floors` (deux bots `prudent`, `FloorIndex(3)` avant f9000, `--until-floor 3` sur 20 graines :
   chiffres dans le rapport), plus les scénarios générés (`make gen GAME=throne`). Traces : bless
   orchestrateur. **Zombies et testbed : rien ne change.**
5. **Lint** : `make lint GAME=throne` vert ; toute règle de lint manquante pour un nouveau cas (arme
   sans munition déclarée, caverne sans ennemi en `Floors`, D36) devient une dette, pas un
   contournement.
6. **Hors périmètre** : code d'engine, sprites définitifs (V6), écran de mutation (T1.16), HUD
   throne (T1.18), feedback (T1.17), boss.

## Traces attendues

Aucune trace existante ne change (nouveau jeu seulement). Vérifiable par `make test_scenarios` vert
sans bless hors des nouveaux scénarios `throne_*` et générés.

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ; aucune
trace bénie par l'agent ; `docs/conventions.md` : **uniquement** §29 « Le jeu `throne` » ;
`docs/taches.md` : ne pas toucher ; `assets.yaml` (V6) : lister les placeholders. Merger `origin/main`
juste avant de livrer ; conflits : garder les deux. Livrer la **phase 1** seule si T1.9/T1.10 ne sont
pas encore dans main (LIVRÉ phase 1), puis la phase 2 sur une branche `m1-v2-contenu-throne-p2`.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. `make lint GAME=throne` vert ; `make gen GAME=throne` sans modification ; scénarios `throne_*`
   et générés verts ; **tous les scénarios existants verts sans trace modifiée**.
2. `alacod-sim --game throne --bots 2 --profiles prudent --floors run --seeds 1..20 --until-floor 3
   --max-frames 12000` : 20/20, 0 desync, 0 softlock (ou écarts expliqués graine par graine).
3. Suite des crates verte, `make lint` des trois jeux, `make fmt`, scripts ; exemples compilés.
4. §29 écrit ; rapport honnête avec point d'état et la liste des placeholders d'assets.

## Livrer

Rapport `docs/taches/rapports/m1-v2-contenu-throne.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). `git push -u origin m1-v2-contenu-throne`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v2-contenu-throne <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
