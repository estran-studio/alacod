# Rapport — m1-bots-apres-movement-feel (graine 19, approche du portail)

Branche partie de `origin/m1-fusion-revue-m0-suite` (d8fb814), rebasée sur `main` 7347075 après
le merge de la fusion.

Entre d8fb814 et 7347075, `crates` et `games` n'ont pas changé : les binaires de mesure valent
pour `main`. Contrôle fait : les 117 traces bénies localement avec le binaire « avant » sont
identiques à celles bénies sur `main`.

## État en cours

Vérifié, livré. Code dans `crates/bots` seulement. Suite complète verte hors les 7 traces citées ; target purgé.

## Graine 19 (throne, 2 bots) : soft-lock à l'étage 1

Rejeu de la graine 19 sur l'état fusionné : soft-lock à f3050. Relevé D42 et inputs sauvegardés
(`--save-scenario`) :

- **Le rat** (#247, 29,5 PV) est en (166, 601).
  - Son point visé, (192, 601), est à la même hauteur que lui. Il est au ras du haut du mur 185
    (x 176–208, y 512–592).
  - Son collider accroche le coin du mur : il pousse vers l'est et ne glisse jamais.
  - Il reste immobile de f2450 à f3050, à 0,15 px près.
  - C'est un défaut moteur (navigation des ennemis), sans correctif ici : **D55**, voir plus bas.
- **Les deux `prudent`** bouclent toutes les 18 frames pendant 600 frames, sans infliger un seul
  dégât :
  1. Ils avancent par le chemin : le rat est caché.
  2. Ils trouvent la ligne de tir à ≈ 172 px, sous `PRUDENT_MIN_DISTANCE` (180).
  3. Ils reculent en tirant 3 à 4 frames.
  4. Ils reperdent la ligne et recommencent.
- **Pourquoi le recul.** La règle « ennemi immobile » (`BotView::enemy_still` : pas de recul,
  approche à 120 px) ne s'applique pas. Elle se fondait sur `Velocity::main < 1`, or c'est la
  vitesse **voulue** par l'IA : un rat qui pousse contre la roche la garde non nulle.
- **Pourquoi seulement maintenant.** Avec l'ancienne course glissante, l'élan sortait les bots de
  ce cycle. La course de movement-feel répond en 3 frames, et le cycle se ferme.

**Correctif (bots)** : nouveau module `stuck.rs`, avec la mémoire `EnemyMoves`.

- Elle est hors rollback, comme `WeaponChoices`. Elle est déterministe (frame `FrameCount`, ordre
  `GgrsNetId`) et vidée à chaque `OnEnter(AppState::InGame)`.
- Un ennemi resté **60 frames dans un rayon de 2 px** compte comme immobile, même avec une vitesse
  voulue non nulle.
- La règle existante s'applique alors : pas de recul, approche à 120 px, et tir sur la ligne brute
  sous 120 px.
- Tests : `coince_apres_une_seconde_sur_place` (tremblement de la graine 19),
  `un_ennemi_qui_marche_n_est_pas_coince`, `repart_puis_oublie_les_absents`.

Résultat : la graine 19 atteint l'étage 3 à f3694, sans soft-lock.

**D55 (ouverte par orch au merge)** : un ennemi dont le point visé est au ras d'un coin de mur
accroche ce coin et ne glisse jamais. Repro : throne, 2 bots, graine 19, état fusionné avant ce
correctif ; rat #247 en (166, 601), mur 185. Le correctif des bots ne fait que le tuer plus
vite : l'ennemi reste coincé.

## Approche du portail : 120 à 190 frames de retard

`steer` pilotait en vitesse : croisière 120, zone morte 12, écart corrigé axe par axe. Ce
pilotage avait été réglé pour l'ancienne course, qui gardait son élan.

La course de movement-feel gagne ou perd 50 px/s par frame. Le pilote alternait donc l'appui et
le bouton opposé presque à chaque frame : le bot zigzaguait. Sur `throne_floor_1`, la direction
change toutes les 1 à 7 frames, et il faut 173 frames entre l'ouverture du portail (f338) et
l'étage (f511).

**Correctif (bots)** : `steer` presse la direction voulue, quantifiée en 8 secteurs. Un axe n'est
pressé que si sa part dépasse sin 22,5° (`STEER_AXIS_SHARE`).

- Il n'y a plus de vitesse de croisière ni de freinage : la course s'arrête en 3 frames, soit
  ≈ 5 px à pleine vitesse, pour un rayon de portail de 24.
- Cela vaut pour l'approche du portail (`approach_portal`) et pour la route vers un ennemi caché.
- Tests remplacés ou mis à jour : `prudent_va_droit_au_portail`,
  `prudent_suit_la_route_en_huit_directions`, `vers_un_ennemi_cache_la_route_est_suivie`,
  `portail_par_le_chemin`. Les 61 tests de `bots` sont verts.

Résultat : 109 frames de l'ouverture à l'étage sur `throne_floor_1`. Sur `bot_floors_three`, le
portail est franchi 33 à 60 frames après son ouverture, contre 121 à 188 avant.

## Mesures (une sim à la fois, mêmes commandes que b0)

| | main d'avant la fusion (836d3cf) | fusion (b0) | après |
|---|---|---|---|
| throne 2 bots, 1..20 : étage 3 | 20/20 | 19/20 | **20/20** |
| soft-locks / desync | 0 / 0 | 1 (graine 19) / 0 | **0 / 0** |
| graines avec un mort | 0 | 3 (5, 8, 10) | **0** |
| frames moyennes | 3502 | 3613 | **2892** |
| dégâts moyens encaissés | 119 | 96 | 94 |
| throne 4 bots, 1..20 : étage 3 / soft-locks / desync / morts | 20/20 / 0 / 0 / 0 | 20/20 / 0 / 0 / 0 | 20/20 / 0 / 0 / 0 |
| 4 bots : frames moyennes | 2359 | 2744 | **2119** |
| zombies 4 acheteurs, 1..20 | 20/20 vague 5 | 20/20 vague 5 | **identique à la fusion, graine par graine** (frames, kills, dégâts) |

- Tous les critères de la fiche sont tenus : 0 soft-lock, défaites non augmentées (0 à 2 bots),
  zombies 20/20.
- Zombies : `acheteur` ne passe ni par `steer` ni par `enemy_still`. Les inputs sont identiques et
  aucune trace zombies ne bouge.
- Plus lent que sur main d'avant la fusion : à 2 bots, la graine 9 seulement (3113 → 3280) ; à 4
  bots, les graines 5, 18, 19 et 20 (2379 → 2451, 2534 → 2727, 2697 → 2840, 2020 → 2224). Toutes
  finissent, sans mort.

## Scénarios et traces

Ne bougent pas :

- `clone_*`, `equilibrage_*`, `bots_four_mixed`, zombies et vagues ;
- testbed hors `bot_floors_three` ;
- `bot_prudent_dodge` et `bot_prudent_nododge`, dont les inputs sont figés.

Les attentes M0 de `clone_quad` sont intactes. Sept scénarios à bots bougent. Preuve par inputs,
`alacod-sim --save-scenario` avant contre après, même graine, même nombre de bots :

| scénario(s) | premier input différent | cause | trace diverge à |
|---|---|---|---|
| `throne_floor_1`, `throne_progression`, `throne_solo` (1 bot, 123456), `throne_quad` (4 bots, 123456), `throne_three_floors` (2 bots, graine 4) | f7, joueur 0 : `Up, Right` → `Down, Left` | ennemi caché à gauche (visée −329, 23) : l'ancien pilote pressait l'opposé pour freiner, le nouveau suit la route | ligne 13 (f12) |
| `throne_softlock_recul` (2 bots, graine 139) | f206, joueur 1 : `Up` → `Up, Left` | quantification en 8 secteurs | ligne 212 (f211) |
| `bot_floors_three` (testbed, 2 bots) | f57, joueur 0 : relâché → `Up, Left` tenu | approche du portail : l'ancien pilote relâchait à sa vitesse de croisière, puis pressait l'opposé | ligne 63 (f62) |

Comme d'habitude, les traces divergent 5 frames après le premier input différent.

Attentes remesurées (`ALACOD_EVENTS=1`) :

- **`throne_floor_1`** : étage à f616 (600 → 640 frames).
  - Le portail s'ouvre plus tard, à f507 au lieu de f338 : la trajectoire change dès f7 et le
    dernier ennemi meurt plus tard.
  - L'approche elle-même est plus courte : 109 frames au lieu de 173.
- **`throne_progression`** (1950 → 2400 frames, pour garder l'alerte) :
  - étage 1 à f616 et `sang_froid` à f904 ;
  - niveau 2 à f1162 et étage 2 à f1491 ;
  - `coriace` à f1762 et `alerte` à f2391.
- **`throne_solo`** :
  - étages à f616 et f1491 ;
  - `coriace` à f1762, alerte à f2391 ;
  - niveau 3 à f2721, vivant à f3699.
- **`throne_quad`** :
  - étages à f262, f944 et f2260 ;
  - portail du niveau 2 à f2120 ;
  - alerte à f1844 ;
  - quatre vivants.
- **`throne_three_floors`** :
  - étages à f288, f1022 et f2873, quatrième caverne à f4473 ;
  - `irradie` du joueur 0 à f3014 ;
  - personne à terre.
- **`bot_floors_three`** : étages à f81, f234 et f444 (800 → 660 frames).
- **`throne_softlock_recul`** : étages à f473 et f1413. Ses attentes tiennent telles quelles.

Toutes ces traces sont à bénir par orch.

## Suite

`make test_scenarios` : attentes vertes ; seules les 7 traces du tableau diffèrent. Le reste est
vert :

- tests des crates (dont `bots` 61/61, hors scénarios) ;
- `make lint`, fmt, scripts interdits et enregistrement rollback ;
- `make gen` ×3 (aucun fichier modifié) ;
- `cargo check -p throne` ;
- exemples.
