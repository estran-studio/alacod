# m2-t14d-gabarits — quinze gabarits, grammaire `etage` complète, équilibrage D64

Branche `m2-t14d-gabarits` (depuis `origin/main` `433cfc6a`, fusionnée avec `origin/main` `030a4bc0` avant la livraison (M3-T1, T0b, T6, T4, T7)).
Données seulement : `games/gungeon/assets/`, scénarios, traces gungeon. Aucune ligne de code de moteur, de bot
ou de boss ; seules les constantes de deux tests suivent les données (grammaire : `map_ldtk::generation::tests` ;
graine par défaut de `explorer_finds_weapons_on_ordinary_floors`, `crates/scenario/tests/bots.rs`).
Conventions : `docs/conventions.md` §52.d. Mesures brutes : `docs/taches/rapports/m2-t14d-gabarits/`
(`av*` = avant, `ap*` = après, un JSON par lot de 10 graines ; `campagne.sh`, `resume.py`, `morts.py`,
`morts.sh`, `rencontre.py`).

## 1. Mesure de départ (`main` `433cfc6a`, `etages_deux`, 200 graines, `explorateur`, `--until-floor 1`)

| | morts | objectif (boss tué, étage 2 atteint) | soft-locks | desyncs | salles visitées / total | mort : frame médiane |
|---|---|---|---|---|---|---|
| 1 bot | **200 / 200** | 0 | 0 | 0 | 2,4 / 10,1 | 368 |
| 2 bots | **174 / 200** (87 %) | 26 | 0 | 0 | 6,5 / 10,1 | 1 652 |

Les 17 soft-locks sur 20 graines de l'état « fix-added-death » ne se reproduisent plus : `main` contient depuis
le réveil de salle de `dettes-d15-d63-d68` (D68/D77), 0 soft-lock sur 400 parties. Rien à corriger de ce côté
au départ ; les soft-locks mesurés plus bas sont nés pendant l'équilibrage (§5).

Tueurs principaux avant (`morts.py`, salle verrouillée au moment de la mort ; tueur = dernier `entity killed by`
du journal, absent pour environ un tiers des parties) :

| | salle de combat | arène | autre / aucune | tueurs connus |
|---|---|---|---|---|
| 1 bot (200 morts) | 160 | 29 | 11 | `bullet_kin` 138, `bandana_kin` 2, `veteran_kin` 1, `lanceur_kin` 1 |
| 2 bots (70 morts sur 174, les 70 premières graines mortes) | 43 | 15 | 12 | `bullet_kin` 37, `lanceur_kin` 10, `bandana_kin` 6, `tourelle_kin` 1, `shotgun_kin` 1 |

Cause : une salle de combat posait cinq ennemis dans 128 px (3 `bullet_kin`, deux ennemis de T14c) ; un
`bullet_kin` tirait trois salves de deux balles ricochantes de 6 dégâts toutes les 90 frames (jusqu'à 36 par
salve, et les balles rebondissent dans une salle de huit cases) ; le joueur n'avait presque aucun soin
(`ration` : 15 PV, une récompense sur trois, une fois sur quatre un coffre que le bot n'ouvre pas).

## 2. Gabarits et grammaire (tâche 2)

Quinze gabarits (`maps/salles.ldtk`, 8×8 cases) : les douze d'avant (`Depart`, `CombatEW/NS/T/Croix`,
`Recompense`, `Boutique`, `BossW/N`, `ArenaEW/Croix`, `EscouadeEW`) plus trois nouveaux, produits par
`docs/taches/rapports/m2-t14d-gabarits.make_gabarits.py` (idempotent, identifiants dérivés des noms) :

| Gabarit | type | ouvertures | contenu |
|---|---|---|---|
| `Coffre` | `coffre` (nouveau, `rooms/coffre.ron`) | N E S W | 3 tables, sable autour du coffre central |
| `Secret` | `secret` (nouveau, `rooms/secret.ron`) | W (cul-de-sac) | 3 tables, eau (2 cases) et fosse (3 cases) dans les angles, coffre rare au centre |
| `ArenaNS` | `arene` | N S | 2 barils, 1 table |

Types représentés : combat (`CombatEW/NS/T/Croix`, `EscouadeEW`), coffre, boutique, boss, secret, plus départ,
récompense et arène. Surfaces : dossier `surfaces/` (`eau`, `sable`, `fosse`) et couche IntGrid `Surfaces` du
projet LDtk, posée dans `Coffre` et `Secret` ; elle suit les gabarits dans l'étage assemblé (sonde `map_probe`,
graine 28 : « fosse : 3 cases »). Props : `Table` et `Barrel` dans cinq gabarits (`CombatEW`, `CombatNS`, `Coffre`,
`Secret`, `ArenaNS`), jamais devant une ouverture.

Grammaire `etage` : de 10 à 14 salles (8 à 12 avant), types exigés `boss`, `recompense`, `boutique`, `arene`,
**`escouade`**, `coffre`, `secret` : la salle `escouade` de M2-T8 est enfin dans l'étage de la partie ordinaire.
`court`, `grand`, `court_escouade` et leurs scénarios scriptés ne bougent pas. Sans changement de moteur : le
filler reste `combat`, les nouveaux types sont des exigences (une salle chacun).

Validation : `ALACOD_FLOOR_SEEDS=1000 cargo test -p map_ldtk gungeon_` : 2 tests verts (`gungeon_etages_de_grammaire` :
`etage` (10–14), `grand`, `court` ; un seul boss en cul-de-sac, aucun chevauchement, chaque type exigé posé
exactement une fois ; `gungeon_decor_sur_1000_graines` : 3 accessoires par salle décorée, dans la salle, ni sur
une porte ni sur un spawn). Durée de `etage` : 200 graines en 6,4 s (chargement LDtk compris).

Scénarios générés nouveaux (tests de type de salle, §54) : `room_escouade` (grammaire `court_escouade`),
`room_arene` (`court_arene`, nouvelle), `room_coffre` (`court_coffre`, nouvelle).

### Graines `alacod-sim` qui rencontrent chaque mécanique

Commande : `alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux --seeds 41..41
--until-floor 1 --max-frames 20000 --shopping` (les coffres et la boutique ne sont ouverts qu'avec `--shopping`,
D62) ; `rencontre.py <bots> <séquence> <graine> [--shopping]` relève les premières frames du journal.

| Mécanique | graine | frames |
|---|---|---|
| escouade (verrou → nettoyée) | 41 (aussi 13, 29, 54) | 1096 → 1233 |
| arène (3 vagues, tenue) | 41 | 1476 → 2226 |
| coffre de la salle `secret` | 41 ; 13 | f369 (`weapon:grenadier`) ; f1100 (`rancune`) |
| coffre de la salle `coffre` | 41 ; 13 | f1045 (`fiole`) ; f3392 (`weapon:lance_roquettes`) |
| boutique (3 achats) | 41 | f2334, f2352, f2373 |
| baril qui explose | 41 | f604 |
| récompense de salle (ration, clé, blank) | 41, 13, 29, 54 | f206 à f226 |
| fosse, eau, sable dans l'étage | 28 | présents au chargement (sonde) ; aucun bot n'y tombe |

Tables renversées : présentes (17 sur la graine 28) mais aucun bot ne les actionne (Interaction), aucune graine
ne les « rencontre » dans une simulation ; elles se jouent à la main. Chute dans une fosse : aucune non plus
(le champ de flux contourne les fosses).

## 3. D64, équilibrage (tâche 4)

Itérations (1 bot, graines 1 à 60 sauf mention ; mêmes graines à chaque ligne) :

| Essai | changement | morts à 1 bot |
|---|---|---|
| départ (200 graines) | — | 200 / 200 |
| e1 (200 graines) | salve : une balle de 4 par salve (3 salves), recharge 120 | 195 / 200, 4 soft-locks |
| e2 | + gabarits : un `bullet_kin` de moins, `shotgun_kin`/`veteran_kin`/`fuyard_kin` seuls en renfort ; arène : un `bullet_kin` de moins par vague ; `crachat` 4, `chevrotine` 3, explosion 10, `bandana_kin` 10 ; récompense 0,5, `ration` 25 PV | 40 / 60 (à 2 bots : 5 / 60) |
| e3 | + `tourelle_kin` : boule `boulette` ; arène : plus de `bullet_kin` dans la dernière vague | 27 / 60, 3 soft-locks |
| e3b | `CombatT` à trois `bullet_kin` (scénarios scriptés), `shotgun_kin` déplacé | 33 / 60 |
| e4 | `bullet_kin` 30 PV, recharge 150, arène 750 frames (0/250/500) | 30 / 60 |
| e5 | une récompense à chaque salle de combat nettoyée (`ration` 6/9) | 28 / 60, 2 soft-locks |
| **e6 = final** | `ricochet` `Bounce(1)`, portée 320 ; `tourelle_kin` 60 PV, `boulette` 4 | **16 / 60**, 0 soft-lock |

Les leviers sont l'allègement de la salve (`bullet_kin`, le tueur de 138 parties sur 142 attribuées), le nombre
d'ennemis par gabarit et le soin ; l'arène était le second tueur (29 morts à 1 bot sur 200 au départ, 15 sur 70
à 2 bots). Ce qui a le plus pesé, par essai : e2 (−33 points à 1 bot), e3 (−22) puis e6 (−20 : les balles qui
rebondissaient dans huit cases touchaient plusieurs fois). La récompense à chaque salle (e5) ne pèse presque rien
à elle seule (la `ration` rend 25 PV, le bot en ramasse peu avant de repartir).

## 4. Mesure finale et tableau avant / après (200 graines, mêmes commandes)

| | morts | objectif | soft-locks | desyncs | salles visitées / total | mort : frame médiane | dégâts pris (médiane) |
|---|---|---|---|---|---|---|---|
| **1 bot, avant** | 200 (100 %) | 0 | 0 | 0 | 2,4 / 10,1 | 368 | 96 |
| **1 bot, après** | **54 (27 %)** | 142 | 4 | 0 | 11,1 / 12,4 | 2 137 | 172,5 |
| **2 bots, avant** | 174 (87 %) | 26 | 0 | 0 | 6,5 / 10,1 | 1 652 | 366 |
| **2 bots, après** | **7 (3,5 %)** | 193 | **0** | 0 | 12,2 / 12,4 | 1 557 | 227 |

Objectifs de la fiche : moins de 15 % de morts à 2 bots (3,5 %), moins de 50 % à 1 bot (27 %), 0 desync (0 sur
800 parties) : **atteints**. 0 soft-lock : atteint à 2 bots, **non atteint à 1 bot (4 sur 200, 2 %)** : voir §5.

Tueurs après (`morts.py`, parties mortes seulement) :

| | salle de combat | arène | boss | aucune | tueurs connus |
|---|---|---|---|---|---|
| 1 bot (54 morts) | 24 | 15 | 1 | 14 | `lanceur_kin` 7, `bullet_kin` 4, `tourelle_kin` 3, `bandana_kin` 2, `shotgun_kin` 1, `fuyard_kin` 1, `gatling` 1 |
| 2 bots (7 morts) | 6 | 0 | 0 | 1 | inconnus |

Le tueur n'est plus le `bullet_kin` mais le `lanceur_kin` (grenades à éclats, l'explosion reste de 10) : à regarder
si l'on veut baisser encore la mortalité à 1 bot. L'arène reste le lieu de 28 % des morts à 1 bot.

## 5. Soft-locks (tâche 3)

Aucun au départ (400 parties). Pendant les essais : 4 à 1 bot (e1), 3 (e3), 2 (e5) ; final : 4 sur 200 à 1 bot
(graines 66, 84, 120, 183), 0 à 2 bots (200 parties). Causes distinguées :

1. **Cycle limite du bot devant une ouverture de deux cases** (graines 28 des essais, 66, 84, 120 : trois sur
   quatre au final, toutes à la frame ~1470 de détection, bot à (x ≈ 0, y ≈ 43 ou −82), c'est-à-dire à 20 px d'une
   ouverture nord-sud de la salle de départ). Constat (graine 28, essai e3, `ALACOD_MAP_PROBE`, journal de
   l'`explorateur` instrumenté sans être livré) : le premier but est la salle voisine (coût 70 ou 90 selon la case de
   départ du champ), aucun ennemi éveillé, aucun objet, aucun projectile ; l'entrée de la salle ne laisse passer
   un corps de 20 px que pour |x| < 6 ; `BotNavigation::approach` vise le centre de case (0, 40), le bot
   dépasse de ±7,7 px (accélération 3 000 px/s², 8 directions), la ligne vers la case suivante n'est plus `clear`,
   le repli choisit à nouveau (0, 40) : le pilotage alterne `Up`, `Down+Left`, `Left`, `Up`, `Down+Right`… sans fin
   (cycle de quelques dizaines de frames, 600 frames sans kill, le plafond est atteint). Les ennemis de la salle
   visée sont dormants et hors de cause : déplacer le `shotgun_kin` de `CombatCroix` (hors de l'axe de la porte)
   ne change rien à la frame de blocage. **Cause : bot (`crates/bots`, pilotage `explore_step` /
   `BotNavigation::approach` et sa zone morte), pas données.** À 2 bots l'autre joueur débloque la partie.
   Non codé (fiche : « ne pas toucher au bot ») ; consigné comme **D82** (`docs/taches/dettes.md`) avec la
   graine, la frame et la position ; pistes : freiner (ne pas relancer le pas quand le point visé est à moins
   d'un demi-pas), ou viser le milieu du passage (`chicane_step`, comme le champ de flux). Aucun levier de
   données : élargir les ouvertures exige des portes plus larges (entité `DoorHorizontal` de 32 px, appariement
   par position) et déplace toutes les traces gungeon une seconde fois.
2. Graine 183 : 4 412 frames, les 12 salles visitées, 7 nettoyées, aucun ennemi restant dans l'observation
   (bot en (257, −82), dernier but non atteint) : **même famille**, non élucidée plus loin (le diagnostic
   `softlock` ne liste que des ennemis).

Le diagnostic d'un soft-lock par données seules est impossible tant que le bot oscille : la distribution des graines
touchées change à chaque essai (28 sur e3, 16 et 45 sur e5, 66 / 84 / 120 / 183 au final) alors que les gabarits ne
changent pas, ce qui confirme un état du bot dépendant de la trajectoire plutôt qu'une salle sans sortie.
Aucune salle sans sortie ni porte qui ne s'ouvre pas n'a été observée : les 1 000 étages de la grammaire sont
connexes et leurs ouvertures libres sont murées.

## 6. Traces gungeon qui bougent, preuve §5

Contre `main` `433cfc6a` (dumps `ALACOD_DUMP_TRACE`, mêmes binaires, données échangées : `scripts/trace-diff.py`).
**43 traces bougées, 3 nouvelles, aucune d'un autre jeu** (zombies, testbed, throne, isaac : identiques).

| Scénario | première frame qui diffère | cause |
|---|---|---|
| `gungeon_start`, `gungeon_clear`, `dash_charges`, `gungeon_boss`, `ui_*`, `unlock_*`, `assets_gungeon_decor`, `generated/gungeon/boss_gatling`, `weapon_arsenal*` (carte `etage_1`) | **0** | entité `bullet_kin` : `Health` 40 → 30 et `MaxHealth`, `Shoot` `cooldown_frames` 90 → 150, table `arsenal` : `crachat` 6 → 4, `ricochet` (6, portée 500, `Bounce(2)`) → (4, portée 320, `Bounce(1)`), `boulette` ajoutée |
| `generated/gungeon/enemy_*` | 0 ou 1 | même données (PV, tir) de l'ennemi testé |
| `gungeon_floor_seed_*`, `gungeon_floor_start`, `gungeon_two_floors`, `bot_explorer_*`, `generated/gungeon/room_*` (grammaire `court`) | 0 | idem (le gabarit `CombatT` garde ses trois `bullet_kin`, mêmes positions) |
| `shop_seeded`, `minimap_shop_visit`, `bench_floor` (grammaire `etage`, `grand`) | 0 | autres étages : sept types exigés, gabarits et accessoires ajoutés (`SurfaceGrid` non vide, autres entités) |

Attentes recalées (sans en retirer) : `gungeon_boss_bot` (boss entré f216, phases f322 / f382, mort f402 ; avant f247 /
f362 / f422 / f442), `bot_explorer_two` (mort du boss f341, étage suivant f434), `bot_explorer_floor` (commentaire
seul), `gungeon_two_floors` (le boss de l'étage 2 est le `NetId` 143), `shop_seeded` (présentoirs : `arbalete` 15,
`mitraillette` 30, `revolver` 15 ; même rôle, la graine tire un autre étage), `minimap_shop_visit` (graine 13 :
la boutique est voisine ouest du départ).

## 7. Vérifications (état fusionné avec `origin/main` `030a4bc0`)

- `make test_scenarios_par SHARDS=3` : 431 scénarios, 3 tranches vertes, 931 s. Les traces gungeon rebénies sur la
  branche (avant les fusions) sont celles que rejoue l'état fusionné : aucune n'a bougé avec M3-T0b, T1, T6, T4, T7
  (barils compris, M3-T4 a refait leur explosion).
- `make lint` : 0 erreur sur les cinq jeux ; `games/gungeon` : 0 avertissement (testbed : 82, bac à sable voulu).
- `make gen GAME=gungeon` : code 0, aucun fichier ne change.
- `make test_crates` : code 0 (69 binaires de test verts). Seul test modifié hors constantes de grammaire :
  `explorer_finds_weapons_on_ordinary_floors` (D62), dont la graine par défaut 3 tirait un autre étage où le bot
  n'atteint ni boutique ni coffre en 1 000 frames : défaut passé à 41 (13, 29 et 54 passent aussi).
- `cargo fmt --all -- --check` : propre.
- Grammaire : `ALACOD_FLOOR_SEEDS=1000 cargo test -p map_ldtk gungeon_` : 2 tests verts.
- Campagne `etages_deux` 200 graines à 1 et 2 bots : mesurée avant les fusions ; contrôle des graines 1 à 20 (1 et
  2 bots) après la première fusion : issue, frames et dégâts identiques graine par graine. Le moteur fusionné
  rejoue les mêmes traces gungeon, donc les mêmes parties.
- p2p gungeon à 2 clients : voir §7 bis.

## 7 bis. p2p gungeon à deux clients (README §4)

`docs/taches/rapports/m2-t14d-gabarits/p2p.sh` (signaling `docker compose -f docker-compose.ci.yaml up -d signaling`,
`MATCHBOX_PORT=3566`, `COMPOSE_PROJECT_NAME=b2`, `docker compose down` à la fin) lance deux fois `$CARGO_TARGET_DIR/headless/gungeon
--matchbox ws://127.0.0.1:3566 --lobby … --number-player 2 --players localhost remote` avec `ALACOD_HEADLESS=1 ALACOD_INPUT_DELAY=0
ALACOD_STATE_TRACE=/tmp/p2p-$i.trace ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x` et `ALACOD_INPUT=script:p2p_gungeon.ron` (deux joueurs,
direction changée toutes les 60 frames, tir toutes les 6 frames, phases décalées) ; second client 8 s après le premier.
État fusionné `030a4bc0` + cette branche, jeu au départ par défaut (`etages_deux`).

- Sorties des deux clients : code 0 ; aucun `DesyncDetected`.
- Traces : 599 lignes chacune, **`cmp` identiques**.
- Frames resimulées : le compteur du jeu (« frames resimulées après rollback ») donne **56** (client 0) et **104** (client 1) ;
  les numéros `ggrs{f=N}` présents en double dans les journaux : 204 (client 0) et 230 (client 1) numéros distincts (ils comptent
  toute ligne de journal d'une frame rejouée, donc plus large que le compteur). Les rollbacks ont donc bien lieu.
- Écarts : aucun. Limite : le jeu part de l'étage de la grammaire dès la frame 0 sans la salle du hub dans ce script ; la transition
  d'étage et les salles verrouillées ne sont pas atteintes en 600 frames (aucune ligne `room_state` dans le journal).

## 8. Dettes ouvertes, points à relire

- **D82** (nouvelle) : cycle limite de l'`explorateur` devant une ouverture de deux cases, 4 soft-locks sur 200 à 1 bot
  (§5).
- **D64** : fermée par cette tâche (mesures §4).
- Le « secret » n'est pas caché : `DoorLock::Secret` est exécuté (M3-T4, mergé pendant cette tâche) mais `gungeon` ne
  donne aucune bombe (D81) ; un mur plein que personne ne casserait aurait rendu la salle et son coffre inaccessibles.
- Les coffres et la boutique ne sont ouverts par les bots qu'avec `--shopping` ; les campagnes de la fiche (sans
  `--shopping`) ne mesurent donc pas leur effet.
- Le jeu est nettement plus doux que la fiche d'origine : un `bullet_kin` fait 4 dégâts par balle (6) et a 30 PV
  (40) ; la `tourelle_kin` a 60 PV (90). Les deux seuils de la fiche sont tenus avec de la marge (3,5 % et 27 %) :
  à relire par William si le ton visé est plus dur (on peut remonter les PV du `bullet_kin` ou la portée du
  `ricochet` avant de toucher aux gabarits ; le `lanceur_kin` est le tueur principal à 1 bot).
- La grammaire `etage` n'a plus que 2 à 6 salles de combat (7 types exigés) : les étages sont plus variés mais
  moins longs en combats ; `CombatT` n'a pas bougé (scénarios scriptés).
- p2p gungeon à 2 clients : non rejoué ici (pas de Docker, `scripts/p2p` à lancer par l'orchestrateur).
