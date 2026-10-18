# Rapport m3-t14-bot-isaac (b3)

Conventions : `docs/conventions.md` §79 (et renvois §47, §70, §73). Branche `m3-t14-bot-isaac`, fusionnée avec `origin/main`
`7ee084e8` (D86). Scripts de campagne et JSON de résultats à côté : `docs/taches/rapports/m3-t14-bot-isaac/`
(`campagne.sh`, `analyse.py`, `morts.py`, `diable.sh`, `s1_*`, `i1_*`, `i2_*`, `i4_*`, `diable_never.json`, `diable_above_4.json`).

## 1. Fait

| Point de la fiche | Livré |
|---|---|
| Politique en données | `ExplorerPolicy` (`crates/game/src/replay.rs`) : `shopping`, `shop` (`Nearest` / `HeartsThenCheapest(low_health)`), `shells`, `devil` (`Never` / `IfHealthAbove(n)`), `bombs` (secret, voisinage, rochers, réserve), `anti_jitter`, `combat_aware`. Défaut = comportement d'avant (aucun autre jeu ne change). `games/isaac/assets/bot.ron` (lu par `alacod-sim`), `Scenario::explorer` pour un scénario. |
| 1. Grille | l'explorateur parcourt les étages `Grid` : portes sur les quatre côtés, boucles (salles et portes triées, buts par coût de chemin). |
| 2. Bombes | pose contre le mur d'une salle secrète probable (type de salle `door_lock: Secret`, non ouverte, au moins 2 salles voisines) et devant le **premier rocher** du chemin d'un but non atteignable autrement ; **un seul poseur par mur** (plus petit handle debout, `GoalContext::peers`) ; fuite de la mèche prioritaire (`flee_step`). |
| 3. Clés et sous | porte à clé ouverte s'il a une clé (Interaction à la porte, sa jumelle s'ouvre) ; coffres ; ramassage des sous bornés à un détour de 400 de coût ; achats en douilles : un **cœur d'abord** quand la vie est au plus la moitié du maximum (et qu'il n'est pas plein), sinon **le moins cher**, puis le plus proche, chaque joueur n'achetant que ce qu'il paie ; objets refusés (`refuse_when_full`, stock au plafond) jamais but. |
| 4. Diable | `devil: Never \| IfHealthAbove(n)` : achète si l'âme couvre le prix, ou si la vie courante dépasse **strictement** `n` et le reste à payer. **Défaut `Never`** (Q14 à confirmer par William, §4). |
| 5. Campagnes | §3. |
| 6. Scénarios | `bot_isaac_floor`, `bot_isaac_secret`, `bot_isaac_two` + `bot_isaac_key`, `bot_isaac_devil`, `bot_isaac_devil_never`, `bot_isaac_rock`, `bot_isaac_rock_sans_bombe`, `bot_isaac_shop`, `bot_isaac_shop_blesse` (dix, tableau au §79). Carte testbed `bot_rocks.ldtk` (`scripts/construire-cartes-t14.py`). |

Autres ajouts, tous des **mécanismes de bot** nés de soft-locks réels des campagnes (chacun gaté par la politique, inerte par défaut) :

- **Anti-jitter** (`anti_jitter`, `ExplorerJitter`, seconde mémoire du bot hors rollback) : le bot qui tourne sur place s'attache à son but et tient son
  point de passage 60 frames au plus (graine 19 sans bombe : deux salles de coût égal, 600 frames d'oscillation avec l'input retardé).
- **Anti-blocage de combat** : empreinte de la vie des ennemis immobile 240 frames → le bot s'approche en ligne droite puis contourne l'ennemi
  (larmes arrêtées par un coin de mur : graines 63, 86, 95, 119, 178, 182 sur le contenu T11c).
- **Poursuite sur place** (`pursue`) : 300 frames sans quitter un rayon de 24 vers un but qui n'est pas une salle → but écarté pour ce joueur.
- **Réanimation à portée** (`revive_step`, `snap`) : à portée de la surface que le jeu sélectionnerait, Interaction tenue sans bouger (graine 103
  à 2 bots : le bot oscillait de ±9 autour de la portée, la réanimation, qui se tient plusieurs frames, ne finissait jamais).
- **`combat_aware`** : fuite de toute bombe allumée (champion explosif de D88 compris) et approche pour des larmes en orbite (`satellite`, graine 41).
- Diagnostic : `ALACOD_BOT_DEBUG=1`, moments clés `door_lock`, `bomb`, `shelf`, JSON `mechanics` d'`alacod-sim`, options `--policy`, `--devil`, `--bombs`,
  `--pickup-progress` (le ramassage compte comme progrès du détecteur de soft-lock, faux par défaut).

## 2. Vérifié (sur la fusion avec `origin/main` `7ee084e8`)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 659 scénarios, 3 tranches ok (199 s).
- `make test_crates` : **1 245 tests passés, 0 échec** (80 binaires ; `bots` : 90 tests, dont anti-jitter, engagement, poursuite, orbite, tenue de la réanimation, identité des buts).
- `make lint` ×5 : zombies, testbed, throne, gungeon, isaac « aucune erreur ». `make gen GAME=…` ×5 : code 0, `git status` sans diff.
- `cargo fmt --all -- --check`, `check_sim_boundary` (13 crates, 2 en dette D69), `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` : OK.
- Fuzz `ALACOD_FUZZ=0:8` (`--ignored`) : 1 test passé (43 s) ; `ALACOD_FUZZ_SCENARIO` ×3 (`bot_isaac_secret`, `bot_isaac_two`, `bot_isaac_key`, 4 graines) : passés.
- **p2p-stress isaac** (Docker OrbStack, `MATCHBOX_PORT=3576`, `COMPOSE_PROJECT_NAME=b3`), avec les **inputs de deux bots figés** (`scripts/p2p-gungeon-inputs.sh`,
  `GAME=isaac FLOORS=sous_sol UNTIL_FLOOR=5 SIM_ARGS=--pickup-progress`, graine 79, 3 000 frames), montage **temporaire non commité** (`pleureur.ron` :
  `bomb` et `starting_consumables: {"key": 0, "bomb": 1}` ; `game.ron` : `default_seed: 79`, car `--seed` n'est pas lu par `isaac` ; restaurés) :
  **3 939 frames resimulées** (client 0 ; 302 rollbacks, profondeur max 11 ; client 1 : 0), 0 `DesyncDetected`, codes 0/0, traces identiques (2 999 lignes), **passage d'étage 0 → 1 resimulé (f1802)**.
  Écueil trouvé en route : avec `--seed 79` seul, le client jouait la carte de `default_seed` (ids de salles et butin différents de ceux de l'enregistrement,
  inputs caducs : bot mort à f745) ; `EXPECT_FLOORS=1` l'a dit au lieu de l'avaler.
- Preuve que les autres jeux ne bougent pas : les 14 scénarios à explorateur existants (`bot_explorer_floor/two`, `grid_floor_seed_1/23/123`, `grid_post_boss`,
  `gungeon_boss_bot`, `gungeon_floor_seed_1/23/123`, `gungeon_full_run`, `gungeon_full_run_four`, `gungeon_two_floors`, `isaac_floor_seed_1`) ont les traces **de
  `main`** (aucune blessée par cette branche). Piège évité : la première version de la fuite de bombe divergeait de `main` sur `grid_post_boss` (f767) et
  `grid_floor_seed_23` (f2371) dès que D88 a donné des champions explosifs aux grilles ; la fuite et l'orbite sont donc sous `combat_aware`.

## 3. Campagnes (séquence par défaut d'`isaac`, `sous_sol`, deux étages ; `--pickup-progress`)

`alacod-sim --game isaac --floors sous_sol --seeds 1..200 --until-floor 5 --max-frames 20000`, binaire de la fusion, politique de `bot.ron`.
« Bombe d'essai » (`--bombs 1`) : `isaac` n'a ni `BombCarrier` ni bombe de départ avant M3-T11d (décision §69) ; la campagne « livrée » (sans bombe) montre le jeu tel
qu'il est, la campagne à bombe d'essai exerce mur secret et rochers.

| Campagne | Graines | Victoires | Morts | Soft-locks | Desyncs | Étage 2 atteint | Victoire, frames (médiane) |
|---|---|---|---|---|---|---|---|
| 1 bot, **livré** (sans bombe) `s1` | 200 | 12 | 188 | **0** | **0** | 95 | 6 106 |
| 1 bot, bombe d'essai `i1` | 200 | 18 | 182 | **0** | **0** | 108 | 6 539 |
| 2 bots, bombe d'essai `i2` | 200 | 116 | 84 | **0** | **0** | 186 | 6 484 |
| 4 bots, bombe d'essai `i4` | 20 | 18 | 2 | **0** | **0** | 20 | 6 232 |

Mécaniques rencontrées (occurrences ; graines qui la rencontrent) :

| Mécanique | `s1` | `i1` | `i2` | `i4` |
|---|---|---|---|---|
| mur secret ouvert à la bombe | — | 224 ; 195 (sans : 41, 74, 177, 178, 180) | 373 ; 200 | 40 ; 20 |
| porte à clé ouverte (étage 2) | 50 ; 50 (l'étage 2 est atteint dans 95 graines) | 62 ; 62 | 157 ; 157 | 19 ; 19 |
| coffres ouverts | 213 ; 164 | 442 ; 192 | 703 ; 200 | 77 ; 20 |
| objets ramassés | 631 ; 188 | 836 ; 194 | 1 320 ; 200 | 140 ; 20 |
| bombes (`bomb` = posée : la sienne, ou celle d'un champion explosif en `s1`) | 168 ; 112 | 397 ; 198 | 713 ; 200 | 74 ; 20 |
| phases de boss | 315 ; 150 | 361 ; 159 | 657 ; 198 | 80 ; 20 |
| diable (achats, `IfHealthAbove(4)`, 55 graines à salle du diable) | 4 achats en 3 graines (23, 28, 181) | | | |

Tueurs (rejeu avec `--log`, `morts.py`) : échantillon de 20 graines à 1 bot sans bombe (1 à 20) : tourelle en croix 3, suiveur 3, mouche 3, gardien (boss) 3,
spectre 1, rampant 1, bourdon 1, **non attribué 5** (le tueur n'est pas un ennemi nommé par le journal ; [INFERENCE] bombe de champion explosif, non vérifiée) ; 9 morts à l'étage 1, 11 au second.
2 bots (17 morts d'une campagne précédente, même contenu) : tourelle en croix 4, non attribué 4, suiveur 2, spectre 2, mouche 2, gardien 2, rampant 1 ; 15 morts sur 17 au second étage.
`s1` et `i1` n'ont pas été rejouées après la dernière correction (réanimation à portée sans marge, tenue bornée) : elle ne joue qu'avec deux joueurs ou plus
(une surface de réanimation), et trois graines à un bot (23, 48, 79) sont identiques image pour image avant et après.

### Lecture : le contenu est trop dur, pas le bot (Q14)

L'étage 1 est franchi dans **48 %** des parties à 1 bot (95/200) ; la victoire arrive dans 6 % (12/200). Les mesures qui isolent la cause :

| Contenu | Visée | Victoires à 1 bot |
|---|---|---|
| `main` + T11c (avant T10, suivis Q17, D88, D95) | libre | **188 / 200** |
| + suivis Q17 (`pleureur` à quatre directions) + D88 (champions dans 98 % des graines) + D95/D96 | quatre directions | 12 / 200 |
| même contenu, `aim: Free` pinné (40 graines, essai jetable) | libre | **10 / 40** |

La visée à quatre directions divise les victoires par cinq (2/40 contre 10/40 sur les mêmes graines) et les champions de D88 le reste. Le bot joue la visée à
quatre directions comme `main` l'a écrite (`apply_four_way`) ; les mécanismes ci-dessus n'y touchent que pour un combat bloqué. **Fourchette de Q14 (taux de franchissement
de l'étage 1) : à fixer avec William sur ces chiffres** ; la cible du jeu appartient à T11d (équilibrage), pas au bot.

Comparaison du diable (55 graines à salle du diable, binaire de la campagne précédente, même politique) : `Never` 5 victoires / 50 morts, `IfHealthAbove("4.0")` 5 / 50 : **aucun signal**, le bot n'atteint presque jamais le
présentoir vivant. Mesure antérieure sur le contenu de T11b (200 graines, 55 graines à salle du diable) : `Never` 55 victoires ; `IfHealthAbove("4.0")` 48 victoires et
7 morts (7 des 10 morts de la campagne) : défaut `Never`, à confirmer (**Q14**).

## 4. Traces bougées

**Aucune trace existante n'a été blessée par cette branche.** Les traces bénies sont celles des **dix scénarios neufs** `bot_isaac_*` (créés ici ; rebénies
après chaque fusion de `main` qui changeait le contenu : T10/Q17/D88/D95/D96, justifié par la cause, pas par le bot). Preuve §5 que le bot ne bouge rien ailleurs : au §2
(14 scénarios à explorateur et tous les scénarios des autres jeux identiques à `main`) ; le premier écart constaté (f767, `grid_post_boss`) a une cause identifiée
(fuite d'une bombe de champion explosif par le bot, `world::Bomb` lu par `ExplorerWorld::frame`) et a été supprimé par `combat_aware`, pas blessé.

## 5. Non fait / dettes

- **Q14** : défaut du diable (`Never`) et fourchette de franchissement de l'étage 1 à décider avec William (§3).
- Inférence de la salle secrète : le bot la lit par le type de salle (`door_lock: Secret`, non ouverte, ≥ 2 voisines) ; une inférence sans cette lecture (case vide voisine de ≥ 2
  salles) reste à faire.
- `devil_accepts` ne modélise pas `shop::max_health_after` (`Mul`/`Set` de `MaxHealth`) : aucun contenu `isaac` n'en a.
- Deux bots arrivés ensemble à un mur : la règle des pairs (un seul poseur) ne vaut que par la proximité des handles ; chacun peut rester sans bombe si le plus petit handle meurt.
- Rochers jouables seulement sur la carte testbed `bot_rocks` (`isaac` n'en a pas avant T11d).
- Un ennemi éveillé inaccessible derrière des rochers laisse le bot en combat perpétuel.
- Kit de bombes d'essai (`--bombs 1`) : `isaac` n'a pas de `BombCarrier` avant M3-T11d ; la boutique d'`isaac` n'existe pas (T11d) : la politique d'achat en douilles est prouvée par des
  scénarios à présentoirs posés, pas par les campagnes.
- La victoire arrive au dernier ennemi du dernier étage : le trésor derrière une porte à clé de la dernière salle de combat n'est parfois pas ouvert (pas un défaut du bot).
- `FOUR_WAY_LANE` (14) et la visée à quatre directions restent celles de `main` ; un tireur qui ne touche pas à 9 unités de la cible (marge du coin de mur) n'est corrigé que par
  l'anti-blocage (240 frames).
- p2p : l'option `--seed` du client `isaac` n'est pas lue (`LaunchSeed` sans consommateur) : il faut `default_seed` dans `game.ron` pour rejouer une graine ; à corriger ou à documenter (script `p2p-gungeon-inputs.sh` : `GAME`, `FLOORS`, `UNTIL_FLOOR`, `SIM_ARGS` ajoutés).

## 6. Relecture indépendante (sous-agent `reviewer`, lecture seule)

Trois passes. 1re (code du bot) : aucun P1 ; P2 (point de passage tenu sans délai, identité de but tronquée) et P3 corrigés ; P3 `devil_accepts` laissé en dette. 2e (delta : anti-jitter,
`pickups_are_progress`, `shells`, `Step::what`) : aucun P1 ; P2 « garde du point de passage sans borne » **corrigé** (60 frames, `JITTER_HOLD_FRAMES`, test), P3 état périmé en combat
**corrigé** (l'état est oublié hors exploration), P3 identité par hachage FNV complet **corrigé**, P3 doc corrigée. 3e (`ReviewT14Final`, mécanismes ajoutés ensuite : anti-blocage de combat,
poursuite sur place, réanimation à portée, `combat_aware`) : **aucun P1, quatre P2, tous traités** : (1) `apply_four_way` décalait le bot qui tient une réanimation → `BotView::hold_revive`
(sous `combat_aware`, test) ; (2) `pursue` comptait des pas non exécutés (réanimation, butin, esquive) et pouvait écarter un but jamais tenté → seuls les pas exécutés et non nuls comptent ;
(3) `ExplorerJitter` n'était pas remise à zéro d'une partie à l'autre (les `GgrsNetId` repartent de zéro) → remise à zéro à `OnEnter(InGame)` ; (4) `snap` mesurait la portée à la boîte
d'un collider circulaire → **marge de 5 essayée puis abandonnée** (elle recréait le blocage de réanimation : 4 soft-locks à 2 bots, graines 1, 168, 180, 187), remplacée par une tenue
**bornée** (`ExplorerJitter::snap_hold` : 240 frames de tenue puis 60 d'approche, test) : si le jeu refuse l'Interaction à la limite, l'approche reprend la main ; les quatre graines gagnent ensuite.

## 7. Journal (à ajouter sur `main` après le merge)

`| 2026-10-18 | M3-T14 (bot explorateur v3 pour isaac : politique en données (bot.ron), porte à clé, bombes (mur secret, rochers, un poseur par mur, fuite de mèche), sous bornés, achats cœurs d'abord, diable Never par défaut, anti-jitter, anti-blocage de combat, réanimation à portée, combat_aware ; §79) | m3-t14-bot-isaac | b3 | mergée ; campagnes 200 graines : 1 bot 12 victoires/188 morts, 2 bots 116/84, 4 bots (20) 18/2, 0 soft-lock, 0 desync ; suite 659, test_crates 1 245, lint/gen ×5, fuzz, p2p-stress isaac 3 939 frames resimulées ; aucune trace existante blessée ; morts = contenu (visée à quatre directions ×5, champions D88), Q14 ouverte ; non fait : inférence de salle secrète sans type, boutique isaac (T11d) |`
