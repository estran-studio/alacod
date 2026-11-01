# polissage-zombies-map-equilibrage (S1-10) — audit, cartes, armes, campagnes

Branche `polissage-zombies-map-equilibrage` (travail b2 sur `s1-10-local`), clone `zombies`, fiche
`docs/taches/polissage-zombies-map-equilibrage.md`. Les § 1–2 sont la campagne « avant » de b3
(`facb7c92`) ; les § 3–5 portent les changements et mesures de b2 (`c84ef0f1`, `88041423`, puis la reprise du 1er novembre sur le Mac).
Les campagnes utilisent le binaire headless et ne remplacent pas un essai humain des armes.

## 1. Audit des cartes (`avant_poste`, `la_fonderie`)

Sources : `games/zombies/assets/maps/{avant_poste,la_fonderie}.ldtk` (JSON LDtk lu avec un script Python jetable, calque `Walls`,
cellules de 16 px), `floor_grammars/la_fonderie.ron`, `game.ron` (`start_map: "floor:la_fonderie"`), `spawning.rs`.
Définitions : *couverture* = cases de mur **intérieures** (hors anneau de bord de la salle) ; *couloir d'une case* = case libre
murée des deux côtés opposés (`CLAUDE.md`, « cases trop étroites ») ; *fenêtres* = `WindowHorizontal` + `WindowVertical`.

### 1.1 `avant_poste` (9 salles de 416 × 304, une grille 26 × 19 = 494 cases chacune)

| Salle | Spawners | Fenêtres | Portes (prix) | Couverture | Libres | Couloirs 1 case | Contenu |
|---|---|---|---|---|---|---|---|
| Depart | **5** | 2 | 3 (750, 750, 750) | 30 | 384 | 0 | pistolet 500, 4 `PlayerSpawn` |
| Couloir | 0 | 0 | 4 (1000 ×4, électrifiées) | 11 | 405 | 0 | mitraillette 1500 |
| Cuisine | 0 | 0 | 1 (1000) | 13 | 397 | 0 | caisse |
| Chambre | 0 | 0 | 1 (1000) | 9 | 401 | 0 | caisse |
| Poste | 0 | **4** | 2 (1250, 1250) | 24 | 388 | 0 | caisse |
| Jug | 0 | 0 | 1 (1500) | 6 | 404 | 0 | Juggernog |
| Armurerie | 0 | 0 | 1 (1250) | 8 | 402 | 0 | fusil à pompe 1000, fusil 2000 |
| Cave | 0 | 0 | 2 (1250, 1250) | 12 | 400 | 0 | caisse |
| Cellier | 0 | 0 | 1 (1500) | 18 | 392 | 0 | Speed Cola |
| **Total** | **5** | **6** | 16 portes (3 + 4 + 1 + 1 + 2 + 1 + 1 + 2 + 1) | 131 | 3 573 | 0 | 4 armes murales, 4 caisses, 2 perks |

**Vérifié : les 5 `ZombieSpawn` d'`avant_poste` sont tous dans `Depart`** (5/5), et les huit autres salles n'en ont aucun.
Conséquence directe, lue dans `enemy_spawn_from_spawners_system` : un spawner ne joue que si un joueur est **dans la même salle**
(`RoomBounds`/`LevelId`) ; dès que les joueurs quittent `Depart`, plus aucun zombie n'apparaît (la vague reste en phase
`Spawning` avec des zombies « à générer »), et le plafond global est de 10 ennemis vivants. Les fenêtres ne sont que dans `Depart`
(2) et `Poste` (4) ; les six autres salles n'ont ni fenêtre ni spawner (c'est la matière de S1-10, point 3, à b2). Aucune salle
n'a de couloir d'une case : les salles sont des boîtes ouvertes avec des îlots de couverture (6 à 30 cases).

### 1.2 `la_fonderie` (16 sections-gabarits, assemblées par `floor_grammars/la_fonderie.ron` : 10 à 12 salles par graine, `loops: 2`)

| Section | Taille (cases) | Spawners | Fenêtres | Portes (prix du gabarit) | Couverture | Libres | Couloirs 1 case |
|---|---|---|---|---|---|---|---|
| Accueil_Comptoir | 32 × 32 | 2 | 2 | 2 (0) | 288 | 650 | 0 |
| Accueil_Consigne | 32 × 32 | 2 | 2 | 2 (0) | 335 | 607 | 0 |
| Atelier_Etablis | 32 × 32 | 2 | 2 | 4 (0) | 161 | 785 | 0 |
| Stockage_Serpentin | 32 × 32 | 2 | 2 | 4 (0) | 187 | 759 | 2 |
| Galerie_Transversale | 32 × 32 | 2 | 2 | 2 (0) | 395 | 545 | 2 |
| Vestiaires_Coude | 32 × 32 | 2 | 2 | 3 (0) | 329 | 613 | 0 |
| Poste_Aiguillage | 32 × 32 | 2 | 2 | 4 (0) | 195 | 751 | 0 |
| Reserve_Traversante | 32 × 32 | 3 | 3 | 4 (0) | 110 | 851 | 0 |
| Cour_Chargement | 64 × 32 | 4 | 4 | 4 (0) | 384 | 1 552 | 0 |
| Halle_Convoyeur | 32 × 64 | 4 | 4 | 4 (0) | 390 | 1 546 | 0 |
| Armurerie_Guichet | 32 × 32 | 2 | 2 | 2 (0) | 252 | 686 | 0 |
| Armurerie_Magasin | 32 × 32 | 2 | 2 | 2 (0) | 172 | 766 | 0 |
| Infirmerie_Annexe | 32 × 32 | 2 | 2 | 1 (0) | 315 | 620 | 0 |
| Infirmerie_Dispensaire | 32 × 32 | 2 | 2 | 1 (0) | 257 | 677 | 0 |
| Four_Finale | 64 × 32 | 2 | 2 | 1 (0) | 583 | 1 327 | 0 |
| Presse_Finale | 32 × 64 | 2 | 2 | 1 (0) | 510 | 1 400 | 2 |
| **Total (16 gabarits)** | | **37** | **37** (25 H + 12 V) | 41 | 4 863 | 14 135 | 6 (3 gabarits) |

Contenu des gabarits : 13 armes murales (pistolet 300 ×2 ; mitraillette 750 ×3, 1000 ×2 ; fusil à pompe 600 ×2, 800 ×2 ; fusil 1000 ×2),
4 perks (Stamin-Up, Speed Cola, Juggernog ×2), 2 `CharacterSpawn` de boss (`boss_zombie`, `Four_Finale` et `Presse_Finale`).
Les prix de porte des gabarits valent tous 0 (valeur d'auteur ; le prix effectif à l'exécution n'est pas relu ici : **non vérifié**,
les achats de porte observés au § 2.4 coûtent 750 à 1 250, ce sont ceux d'`avant_poste`).

**Verdict d'audit** : la concentration des spawners dans `Depart` est **démentie pour `la_fonderie`** (chaque section en a 2 à 4, ratio
fenêtres/spawners de 1:1) et **confirmée pour `avant_poste`** (5/5 ; aucune fenêtre ni spawner dans 7 des 9 salles). Couloirs d'une case : aucun sur `avant_poste` ; 6 cases sur `la_fonderie` (Stockage_Serpentin 2, Galerie_Transversale 2,
Presse_Finale 2), isolées, sans blocage observé en campagne (les soft-locks de `la_fonderie` ne sont pas des couloirs, § 2.3).

## 2. Campagne « avant » (`alacod-sim`, aucun changement de contenu)

### 2.1 Commandes exactes

Binaire : `env -u CI CARGO_BUILD_JOBS=4 cargo build -q -p scenario --bin alacod-sim --profile headless --no-default-features`
→ `target/headless/alacod-sim` (commit `facb7c92`). Quatre processus en parallèle (un par carte × nombre de bots), puis, pour
`la_fonderie` à 2 bots, trois processus (graines 1..79, 80..89, 90..100, voir « Remarque » plus bas) :

```
ALACOD_CHECK_DISTANCE=0 target/headless/alacod-sim --game zombies --bots 1 --profiles acheteur        --seeds 1..100 --until-wave 10 --max-frames 20000 --map maps/avant_poste.ldtk --save-scenario target/metrics/polissage-zombies-map-equilibrage/avant_poste-1bot-scen --json target/metrics/polissage-zombies-map-equilibrage/avant-avant_poste-1bot.json
ALACOD_CHECK_DISTANCE=0 target/headless/alacod-sim --game zombies --bots 2 --profiles acheteur,acheteur --seeds 1..100 --until-wave 10 --max-frames 20000 --map maps/avant_poste.ldtk --save-scenario …/avant_poste-2bot-scen --json …/avant-avant_poste-2bot.json
ALACOD_CHECK_DISTANCE=0 target/headless/alacod-sim --game zombies --bots 1 --profiles acheteur        --seeds 1..100 --until-wave 10 --max-frames 20000 --save-scenario …/la_fonderie-1bot-scen --json …/avant-la_fonderie-1bot.json
ALACOD_CHECK_DISTANCE=0 target/headless/alacod-sim --game zombies --bots 2 --profiles acheteur,acheteur --seeds 1..100 --until-wave 10 --max-frames 20000 --save-scenario …/la_fonderie-2bot-scen --json …/avant-la_fonderie-2bot.json   (tranche 1..79 gardée via le journal)
```

(`la_fonderie` = `start_map` du jeu `floor:la_fonderie`, grammaire, `map_seed` = graine ; `avant_poste` par `--map`.) Profil `acheteur`
(le défaut d'`alacod-sim`, achète portes et armes ; la campagne throne de S2-7 utilisait `prudent`, qui n'achète rien).
Métriques sous `target/metrics/polissage-zombies-map-equilibrage/` (dossier ignoré par git, comme tout `target/`) :
`avant-<carte>-<n>bot.json` (+ `.log`, une ligne par graine), `avant-la_fonderie-2bot-seeds{80-100,90-100}.{json,log}`,
`avant-synthese-par-graine.json` (une entrée par carte × bots × graine, tirée des journaux), `*-scen/seed_N.ron` (inputs rejouables),
`events-<carte>-<n>bot/seed_N.events.json` (moments clés, § 2.4).

**Pourquoi `ALACOD_CHECK_DISTANCE=0`** : le synctest de distance 2 (défaut du runner) fait ~29 fps sur ces cartes (un essai : graine 1,
`la_fonderie`, 1 bot, vague 10 : 550 s) contre ~100 fps à distance 0 ; 400 parties auraient pris ~10 h au lieu de ~3,5 h. La preuve de
desync est donc faite à part (§ 2.5), avec la vérification de rollback par défaut. Mesure : même graine, vague 4, distance 0 : 47,6 s ;
distance 1 : 98 s ; distance 2 : environ le double.

### 2.2 Résultats (100 graines par ligne ; vague = vague atteinte à la fin de la partie)

| Carte | Bots | Graines | Vague médiane | p10 | p90 | Vague 10 atteinte | Boss tué (`Victory`) | Équipe morte (`Defeat`) | Plafond 20 000 f (« soft-lock » du journal) | Frames médiane | Kills médian |
|---|---|---|---|---|---|---|---|---|---|---|---|
| avant_poste | 1 | 100 | 9 | 5 | 9 | 2 | 0 | 31 | 67 | 20 000 | 90 |
| avant_poste | 2 | 100 | 9 | 9 | 10 | 37 | 0 | 1 | 62 | 20 000 | 107 |
| la_fonderie | 1 | 100 | 6 | 3 | 9 | 2 | 67 | 19 | 12 | 10 656 | 47 |
| la_fonderie | 2 | 100 | 6 | 2 | 9 | 7 | 88 | 0 | 5 | 10 662 | 53 |

Lecture : `avant_poste` n'a pas de boss, la partie se joue jusqu'à la vague 10 ou au plafond ; `la_fonderie` finit sur la mort du boss
(`Victory`, vagues 1 à 9 selon la graine : la grammaire place le boss plus ou moins loin). Le tableau vient des journaux (une ligne par graine) ; ils sont
identiques aux JSON là où les deux existent (vérifié : mêmes issues sur les 3 JSON complets).

Issues du JSON (champ `outcome`, nom du classement d'`alacod-sim`) : `avant_poste` 1 bot : plafond 67, mort 31, objectif 2 ;
2 bots : plafond 62, objectif 37, mort 1 ; `la_fonderie` 1 bot : victoire 67, mort 19, plafond 12, objectif 2. 2 bots : victoire 88, objectif 7,
plafond 5 (tiré des journaux : le JSON complet n'existe pas, voir § 2.6).

Temps par vague (frames entre le début d'une vague et sa fin, médiane des graines qui l'ont finie ; JSON `waves[]`) :

| Carte, bots | v1 | v2 | v3 | v4 | v5 | v6 | v7 | v8 | v9 |
|---|---|---|---|---|---|---|---|---|---|
| avant_poste, 1 | 692 | 967 | 1 212 | 1 452 | 1 785 | 2 028 | 2 436 | 2 619 | 2 462 |
| avant_poste, 2 | 652 | 917 | 1 132 | 1 366 | 1 604 | 1 922 | 2 154 | 2 391 | 2 544 |
| la_fonderie, 1 | 692 | 940 | 1 146 | 1 384 | 1 734 | 2 051 | 2 232 | 2 338 | 2 487 |
| la_fonderie, 2 (graines 90–100 seulement) | 554 | 966 | 1 119 | 1 382 | 1 583 | 1 946 | 2 183 | | |

Dégâts subis (médiane, `damage_taken` de l'équipe) : avant_poste 1 bot 103,5 ; 2 bots 29,5 ; la_fonderie 1 bot 44,5 ; 2 bots (graines 90–100) 0.
Portes ouvertes (médiane) : 10 / 14 / 12 / 14. fps de simulation (médiane, distance 0) : 198 / 189 / 116 / 108.

### 2.3 Morts et soft-locks

**Morts par cause** : `alacod-sim` classe la mort mais **ne donne pas la cause** (ni ennemi, ni arme, ni salle ; le JSON n'a que `deaths`) →
non vérifié, ouvert en D173. Ce qui se lit : les 31 morts d'`avant_poste` à 1 bot tombent aux vagues 5 à 9 (frames 7 893 à 19 647) ;
les 19 morts de `la_fonderie` à 1 bot aux vagues 5 à 7 (frames 7 943 à 13 771). Avec 2 bots : 1 mort en tout sur `avant_poste` (graine 27,
frame 18 859, vague 9) et aucune sur `la_fonderie`. Un bot de plus divise donc les morts par 30 (avant_poste) et les annule (la_fonderie).

**Soft-locks** : le journal nomme « soft-lock » toute partie arrêtée au plafond de 20 000 frames sans objectif ni mort (`plafond`, voir
`softlock` du JSON : instantanés à la fin et 600 frames avant). Beaucoup sont une partie lente, pas un blocage : les parties qui ont
passé plus de 4 000 frames sur la **dernière vague** sont de vrais blocages. Liste (graine : vague bloquée) :

- avant_poste, 1 bot (14) : 7 (v1), 50 (v1), 23 (v3), 45 (v3), 92 (v4), 100 (v4), 28 (v5), 20 (v6), 67 (v7), 89 (v7), 26 (v8), 39 (v8), 81 (v8), 82 (v8) ;
- avant_poste, 2 bots (5) : 23 (v3), 38 (v4), 33 (v6), 22 (v7), 92 (v8) ;
- la_fonderie, 1 bot (0) : les 12 plafonds (graines 8, 26, 43, 49, 54, 55, 57, 63, 70, 74, 88, 91) sont tous en vague 9, ≤ 4 000 frames sur la vague : parties lentes ;
- la_fonderie, 2 bots (5 plafonds : graines 21, 38, 56, 59, 63, tous vague 9) : âge de la dernière vague non relu (JSON de la tranche 1..79 non gardé),
  vague 9 donc très probablement lents aussi.

Cause lue dans les `softlock` des graines 7, 50 (v1) et 23 (v3) : un zombie, `Chasing`, ses `next_cell` et `steering_point` confondus avec sa
case, à ~110 unités du joueur « hors de vue », 0 kill depuis 600 frames, le joueur (bot `acheteur`) n'ayant plus rien à viser ; aucun mur
physique dans le chemin. Dans `avant_poste` la phase de fin est `Spawning` dans 47 parties à 1 bot (26 avec un joueur encore dans `Depart`, 21 hors de
`Depart`) : 21 parties à 1 bot et 3 à 2 bots où les joueurs ont quitté la salle des spawners alors que des zombies restent « à générer », conforme à
l'audit (§ 1.1). Ce sont des défauts de bot et de carte, pas de moteur, **pas de desync** ; ouverts en D174 (cause à confirmer au rejeu).

### 2.4 Armes et achats (échantillon de 12 graines par ligne, `--events`)

Commande : celle du § 2.1 avec `--seeds 1..12 --events target/metrics/polissage-zombies-map-equilibrage/events-<carte>-<n>bot --json …/avant-events-<carte>-<n>bot.json`.
Le moment clé `weapon` ne donne que « chargeur vide (arme) » ; le nombre de chargeurs vides par arme sert d'indice d'usage, `purchase` et « prend » donnent les achats.

| Ligne | Chargeurs vides : mitraillette / pistolet / fusil à pompe | Achats : portes / recharges murales / Juggernog | Armes prises (pistolet / fusil à pompe / mitraillette) | Fusil (`rifle`) |
|---|---|---|---|---|
| avant_poste, 1 bot | 140 / 104 / 2 | 66 / 13 / 6 | 11 / 8 / 2 | jamais |
| avant_poste, 2 bots | 298 / 234 / 0 | 86 / 22 / 17 | 18 / 14 / 0 | jamais |
| la_fonderie, 1 bot | 133 / 71 / 0 | 74 / 13 / 2 | 11 / 6 / 0 | jamais |
| la_fonderie, 2 bots | 209 / 108 / 0 | 80 / 18 / 2 | 12 / 10 / 0 | jamais |

Constat : le joueur démarre à la **mitraillette** (`active_weapon: machine_gun`) et c'est l'arme la plus vidée ; le pistolet est la deuxième (bot qui le
rachète à 300–500 : « pistolet inutile » se vérifie dans le sens où il sert surtout de secours) ; le fusil à pompe n'est presque jamais vidé (2
chargeurs en 12 parties, `avant_poste` 1 bot) ; **le fusil (`rifle`) n'est jamais acheté**. L'impression de William sur le fusil à pompe « super
fort » ne se lit pas dans ces chiffres (usage négligeable) : le bot `acheteur` ne le choisit presque pas, c'est une limite du bot, à reprendre
au réglage (b2).
Issues de l'échantillon : avant_poste 1 bot : plafond 9, mort 3 ; 2 bots : plafond 9, objectif 3 ; la_fonderie 1 bot : victoire 11, plafond 1 ; 2 bots : victoire 11, objectif 1. 0 desync.

### 2.5 Desync

0 desync sur les 400 parties à distance 0 (le champ `desync` du JSON est faux partout, code de sortie 0), mais cette distance ne
vérifie pas le rollback : **preuve séparée** à la distance par défaut du runner (2), 20 graines par ligne, vague 4 :
```
target/headless/alacod-sim --game zombies --bots <1|2> --profiles acheteur[,acheteur] [--map maps/avant_poste.ldtk] --seeds 1..20 --until-wave 4 --max-frames 8000 --json target/metrics/polissage-zombies-map-equilibrage/avant-desync-<carte>-<n>bot.json
```

| Ligne (20 graines) | desync | issues |
|---|---|---|
| avant_poste, 1 bot | 0 | objectif 19, plafond 1 |
| avant_poste, 2 bots | 0 | objectif 20 |
| la_fonderie, 1 bot | 0 | objectif 18, victoire 2 |
| la_fonderie, 2 bots | 0 | objectif 18, victoire 2 |

**0 desync sur 80 parties au synctest de distance 2** (code de sortie 0) ; le champ `desync` du JSON est faux partout. L'échantillon
`--events` du § 2.4 (48 parties, distance 0) n'a pas non plus de desync. La passe à distance 2 ne couvre que 20 graines et la vague 4 :
les parties longues (vagues 5 à 10) n'ont pas de preuve de rollback dans ce lot.

### 2.6 Remarque sur la tranche 1..100 de `la_fonderie` à 2 bots

Pour tenir la durée de la campagne, ce processus (le plus lent, ~3 min par graine) a été coupé au bout de 79 graines et ses
graines 80 à 100 relancées dans deux autres processus (80..100, coupé à 89 ; 90..100). Les graines 80 à 84 ont tourné deux fois,
avec la **même issue** à chaque fois (vague, frames, kills identiques ligne à ligne : une preuve gratuite de reproductibilité).
Pour cette ligne, seuls les journaux couvrent 1..100 ; le JSON complet n'existe que pour les graines 90 à 100.

## 3. Cartes et armes (point 3)

Branche `polissage-zombies-map-equilibrage` (travail b2 sur `s1-10-local`). Aucune arme, effet, mécanique ni son nouveaux :
une carte LDtk et des valeurs de `weapons.ron`.

### 3.1 `avant_poste` : spawners et fenêtres dans toutes les salles

`games/zombies/assets/maps/avant_poste.ldtk`, recomptage des entités de chaque niveau (`ZombieSpawn`, `WindowHorizontal` +
`WindowVertical`, script jetable sur le JSON LDtk) :

| Salle | Spawners avant → après | Fenêtres avant → après |
|---|---:|---:|
| Depart | 5 → 5 | 2 → 2 |
| Couloir | 0 → **2** | 0 → **1** |
| Cuisine | 0 → **3** | 0 → **2** |
| Chambre | 0 → **3** | 0 → **2** |
| Poste | 0 → **4** | 4 → 4 |
| Jug | 0 → **3** | 0 → **2** |
| Armurerie | 0 → **3** | 0 → **2** |
| Cave | 0 → **3** | 0 → **2** |
| Cellier | 0 → **3** | 0 → **2** |
| **Total** | **5 → 29** | **6 → 19** |

Chaque nouveau spawner porte une zone d'activation (`active_x/y/width/height`, commit `c84ef0f1`) : les vagues choisissent le point
d'apparition par distance sans regarder la salle ; sans zone, des zombies naissaient dans des salles encore fermées par une porte.
Un spawner ne joue que si un joueur est dans la même salle (`enemy_spawn_from_spawners_system`, § 1.1) : avec 29 spawners répartis, les
vagues continuent d'apparaître quand les joueurs quittent `Depart`.

**Blocages du § 2.3.** La campagne a montré que la cause n'était **pas la carte** : un goulet de deux cases dans `Depart` (fenêtre 54,
`(144,56)`, murs `x32..128` et `x160..272`) existait déjà, les nouveaux spawners changent le trajet et le font ressortir. Contrefactuel
mesuré par b2 (carte restaurée ensuite) : élargir les 19 fenêtres de 2 à 4 cases laisse le blocage sur les graines 2, 3, 45 et 76. Cause
réelle : le moteur (`flow_direction` : direction nulle quand le point visé est confondu avec la case, zombie figé ; `enemy_target_selection` :
fenêtre intacte non frappée), corrigé par b4 (D176, `79538914`, mergé dans cette branche). Pas de changement de carte pour cela. Ce
qui reste de plafonds est un défaut de rythme du bot et des vagues (§ 4.3), D174.

`la_fonderie` n'est pas modifiée : 2 à 4 spawners par gabarit, ratio fenêtres/spawners de 1:1, six cases étroites isolées qui ne figurent
dans aucun blocage observé (§ 1.2).

### 3.2 Armes (`games/zombies/assets/ZombieShooter/Sprites/Character/weapons.ron`)

Mesure de départ (§ 2.4) : la mitraillette de départ est l'arme la plus vidée (140 chargeurs vides en 12 graines solo), le pistolet sert de
secours, le fusil à pompe est à peine vidé (2) et le fusil jamais acheté. Rôle visé par arme : **pistolet** = précis, secours ;
**mitraillette** = cadence, milieu de portée, sans rendement exagéré ; **fusil à pompe** = proximité, puissant mais court ;
**fusil** = longue portée, coup lourd, cadence basse.

| Arme (prix du mur) | Paramètre | Avant | Après | Dégâts par seconde, une cible (avant → après) |
|---|---|---:|---:|---:|
| Pistolet (500) | dégâts / cadence / chargeur | 15 / 2 par s / 6 | **22 / 3 par s / 9** | 30 → 66 |
| Mitraillette (1 500, arme de départ) | dispersion / dégâts (deux modes) | 0,12 / 8 | **0,07 / 7** | 80 → 70, plus précise |
| Fusil à pompe (1 000) | dégâts par plomb (8 plombs) / portée (modes `default` et `piercing`) | 30 / 400 | **20 / 320** | 240 → 160 à bout portant |
| Fusil (2 000) | dégâts / cadence / chargeur | 35 / 4 par s / 10 | **55 / 3 par s / 8** | 140 → 165 |

Les prix des armes murales (carte) sont **inchangés** : le fusil (2 000, `Armurerie`, derrière une porte à 1 250) n'est jamais acheté par le bot
`acheteur`, avant comme après (§ 4.4) ; cause non établie (trajet du bot ou prix), à juger par William (§ 4.5) plutôt que par un prix posé à l'aveugle.
Le pistolet change de rôle (secours précis à 66 par seconde, deux fois plus de coups par chargeur) ; la mitraillette perd de la puissance
brute (−12 %) contre de la précision ; le fusil à pompe perd un tiers de sa puissance et de la portée (constat de William : « super fort »).

`make gen GAME=zombies` : les six scénarios générés (`enemy_zombie_*`) dépendent du hash des armes (§ 5).

## 4. Campagne « après » (point 4)

### 4.1 Commandes

Mêmes quatre commandes et mêmes réglages que le § 2.1 (`ALACOD_CHECK_DISTANCE=0`, profil `acheteur`, graines `1..100`, `--until-wave 10`,
`--max-frames 20000`, `--map maps/avant_poste.ldtk` pour `avant_poste`, `start_map` du jeu pour `la_fonderie`), binaire
`alacod-sim` sans dev-dylib. Deux séries sous `target/metrics/polissage-zombies-map-equilibrage/` (VPS, dossier ignoré par git) :

- `apres-<carte>-<n>bot.{json,log}` : cartes et armes de `c84ef0f1`, **sans** la correction D176 (première campagne après) ;
- `apres-d176-<carte>-<n>bot.{json,log}` : même contenu **avec** la correction D176 de b4 (tête `905b38ea`, mergée), 01:39–01:54 UTC le 31.
  C'est la série finale ; les scénarios rejouables sont dans `apres-d176-*-scen/seed_N.ron`.

L'avant (§ 2.2) a été joué au commit `facb7c92`, avant D176 : la colonne du milieu isole donc l'effet des cartes et des armes, la
dernière y ajoute le moteur. Chaque ligne est la même série de 100 graines.

### 4.2 Tableau comparé (100 graines par ligne)

| Carte | Bots | Indicateur | Avant (`facb7c92`) | Après, cartes + armes (sans D176) | **Après final** (avec D176) |
|---|---:|---|---:|---:|---:|
| avant_poste | 1 | Vague médiane (p10–p90) | 9 (5–9) | 9 (7–9) | **9 (9–9)** |
|  |  | Vague 10 atteinte | 2 | 0 | **2** |
|  |  | Boss tué (`Victoire`) | 0 | 0 | **0** |
|  |  | Équipe morte | 31 | 6 | **5** |
|  |  | Plafond 20 000 f | 67 | 94 | **93** |
|  |  | dont bloquées (> 4 000 f sur la dernière vague) | 14 | 15 | **1** |
|  |  | Frames médianes | 20 000 | 20 000 | **20 000** |
|  |  | Kills médians | 90 | 98,5 | **101** |
|  |  | Dégâts subis médians (équipe) | 103,5 | 53 | **59,5** |
|  |  | Portes ouvertes médianes | 10 | 14 | **14** |
|  |  | Desync | 0 | 0 | **0** |
| avant_poste | 2 | Vague médiane (p10–p90) | 9 (9–10) | 9 (9–10) | **9 (9–10)** |
|  |  | Vague 10 atteinte | 37 | 46 | **41** |
|  |  | Boss tué (`Victoire`) | 0 | 0 | **0** |
|  |  | Équipe morte | 1 | 0 | **0** |
|  |  | Plafond 20 000 f | 62 | 54 | **59** |
|  |  | dont bloquées (> 4 000 f sur la dernière vague) | 5 | 7 | **0** |
|  |  | Frames médianes | 20 000 | 20 000 | **20 000** |
|  |  | Kills médians | 107 | 107 | **108** |
|  |  | Dégâts subis médians (équipe) | 29,5 | 12 | **11,5** |
|  |  | Portes ouvertes médianes | 14 | 14 | **14** |
|  |  | Desync | 0 | 0 | **0** |
| la_fonderie | 1 | Vague médiane (p10–p90) | 6 (3–9) | 7 (3–9) | **7 (3–9)** |
|  |  | Vague 10 atteinte | 2 | 2 | **0** |
|  |  | Boss tué (`Victoire`) | 67 | 74 | **79** |
|  |  | Équipe morte | 19 | 11 | **8** |
|  |  | Plafond 20 000 f | 12 | 13 | **13** |
|  |  | dont bloquées (> 4 000 f sur la dernière vague) | 0 | 0 | **0** |
|  |  | Frames médianes | 10 656 | 12 802 | **12 890** |
|  |  | Kills médians | 47 | 60 | **60,5** |
|  |  | Dégâts subis médians (équipe) | 44,5 | 8,5 | **4,5** |
|  |  | Portes ouvertes médianes | 12 | 14 | **14** |
|  |  | Desync | 0 | 0 | **0** |
| la_fonderie | 2 | Vague médiane (p10–p90) | 6 (2–9) | 7 (2–9) | **6 (2–9)** |
|  |  | Vague 10 atteinte | 7 | 7 | **8** |
|  |  | Boss tué (`Victoire`) | 88 | 87 | **87** |
|  |  | Équipe morte | 0 | 0 | **0** |
|  |  | Plafond 20 000 f | 5 | 6 | **5** |
|  |  | dont bloquées (> 4 000 f sur la dernière vague) | 0 | 0 | **0** |
|  |  | Frames médianes | 10 662 | 11 354 | **10 216** |
|  |  | Kills médians | 53 | 54 | **51** |
|  |  | Dégâts subis médians (équipe) | 0* | 0 | **0** |
|  |  | Portes ouvertes médianes | 14 | 16 | **14** |
|  |  | Desync | 0 | 0 | **0** |

### 4.3 Lecture

Lecture (colonne finale contre l'avant ; « bloquée » = plafond avec plus de 4 000 frames passées sur la dernière vague, définition du § 2.3) :

- **Parties bloquées : 14 → 1 en solo et 5 → 0 en duo sur `avant_poste`** (`la_fonderie` : 0 avant et après). La première campagne après (colonne du
  milieu, sans D176) en gardait 15 et 7 : les spawners et fenêtres ajoutés ne corrigent pas les blocages, ils les révèlent (§ 3.1) ; la
  correction moteur D176 (zombie figé devant une fenêtre) les supprime. L'unique partie restante (solo, graine 29, vague 8, 4 468 frames)
  a ses trois zombies en `Chasing`, `next_cell` et `steering_point` distincts de leur case : lente, pas figée.
- **Morts : 31 → 5** (`avant_poste`, 1 bot), 19 → 8 (`la_fonderie`, 1 bot) ; dégâts subis médians 103,5 → 59,5 et 44,5 → 4,5. Les cinq morts
  d'`avant_poste` tombent aux vagues 8 et 9 (graines 4, 27, 37, 89, 91), contre les vagues 5 à 9 avant. Le gain se partage entre les armes
  (pistolet à 66 par seconde au lieu de 30, pompe moins sûre) et les spawners répartis ; la campagne ne les sépare pas.
- **Victoires de `la_fonderie` en solo : 67 → 79**, duo 88 → 87 (stable). `avant_poste` n'a pas de boss.
- **Vague atteinte : inchangée** (médiane 9 sur `avant_poste`, 6 à 7 sur `la_fonderie`), vague 10 toujours rare (2 → 2 en solo, 37 → 41 en duo).
  Une vague prend environ 2 500 frames aux vagues 7 à 9 pour un plafond de 20 000 : le plafond tronque la partie, il ne mesure plus un blocage
  (70 des 93 plafonds solo sont dans la phase `Spawning` de la vague 9, zombies en route). Durée médiane d'une vague (frames, `avant_poste`) :
  solo v1 701, v5 1 664, v9 2 597 (avant 692, 1 785, 2 462) ; duo v1 652, v5 1 637, v9 2 590 (avant 652, 1 604, 2 544). **Les vagues ne sont pas plus courtes.**
- **« Les vagues viennent de partout » n'est pas mesuré** : `alacod-sim` ne journalise pas la salle d'apparition. Les 29 spawners et leurs
  zones d'activation (§ 3.1) en sont la seule preuve ; D174 (rythme du bot) et D173 (cause de mort) restent ouvertes.

### 4.4 Armes utilisées (échantillon de 12 graines par ligne, `--events`, même méthode que le § 2.4)

Commande : celle du § 2.1 avec `--seeds 1..12 --events <dossier> --json <fichier>` (Mac, binaire `alacod-sim` du commit de tête, `ALACOD_CHECK_DISTANCE=0`).
Chargeurs vides = moments clés `weapon` « chargeur vide (arme) » ; achats = `paie N (door | wall_weapon_refill | perk:juggernog)` ; prises = « prend arme ».

| Ligne | Chargeurs vides mitraillette / pistolet / pompe (avant → après) | Achats portes / recharges murales / Juggernog (avant → après) | Armes prises pistolet / pompe / mitraillette (avant → après) | Fusil |
|---|---|---|---|---|
| avant_poste, 1 bot | 140 / 104 / 2 → **136 / 156 / 0** | 66 / 13 / 6 → **82 / 15 / 7** | 11 / 8 / 2 → **12 / 9 / 0** | jamais → jamais |
| avant_poste, 2 bots | 298 / 234 / 0 → **359 / 140 / 0** | 86 / 22 / 17 → **83 / 19 / 11** | 18 / 14 / 0 → **15 / 6 / 0** | jamais → jamais |
| la_fonderie, 1 bot | 133 / 71 / 0 → **114 / 104 / 0** | 74 / 13 / 2 → **77 / 14 / 2** | 11 / 6 / 0 → **11 / 5 / 0** | jamais → jamais |
| la_fonderie, 2 bots | 209 / 108 / 0 → **215 / 66 / 0** | 80 / 18 / 2 → **82 / 13 / 4** | 12 / 10 / 0 → **12 / 4 / 0** | jamais → jamais |

Issues de l'échantillon : avant_poste 1 bot plafond 11 / mort 1 ; 2 bots plafond 7 / objectif 5 ; la_fonderie 1 bot victoire 9 / plafond 2 / mort 1 ;
2 bots victoire 10 / plafond 1 / objectif 1. Verdict : la mitraillette reste l'arme la plus vidée en duo (359 et 215) ; en solo le pistolet la rattrape
(104 → 156 sur `avant_poste`, 71 → 104 sur `la_fonderie`), ce qui va dans le sens de « pistolet utile » ; la pompe est prise 4 à 9 fois mais jamais vidée (arme sans chargeur) ; **le fusil n'est toujours pas acheté** (aucun
moment « prend rifle » ni « paie … rifle », dans les deux séries). Un bot ne juge pas un rôle d'arme : le verdict « distinct et utile » est celui de William (§ 4.5).

### 4.5 Protocole des 15 minutes de William (à faire sur le Mac, sans écoute)

Prérequis : branche `polissage-zombies-map-equilibrage` (tête citée par la ligne `LIVRÉ`) dans un worktree, `ALACOD_MUTE=1` partout (aucun son).
1. `la_fonderie` (carte par défaut, 3 armes + fenêtres partout) : `ALACOD_MUTE=1 make zombies GARGS="--seed 42"` ; jouer une partie entière.
2. `avant_poste` (la carte aux 29 spawners) : changer pour la durée de l'essai `start_map: "floor:la_fonderie"` en `start_map: "maps/avant_poste.ldtk"` dans
   `games/zombies/assets/game.ron` (ne pas commiter), puis la même commande. **Non vérifié** : le lancement natif avec rendu n'a pas été exécuté ici (la carte l'est par `alacod-sim --map`).
3. 5 min pistolet contre mitraillette (précision à mi-portée) ; 5 min fusil à pompe (puissance et portée) ; 5 min fusil (prix 2 000, accès, cadence face à la mitraillette).
4. Noter par arme : utile ou non, portée ressentie, dégâts/cadence, prix acceptable, vague à laquelle elle a été achetée ; noter toute salle où un
   zombie reste introuvable ou une vague ne se termine pas (c'est le cas de blocage D176/D174).

**Verdict humain en attente** : aucune note n'est attribuée à William sans cet essai.

### 4.6 Desync

0 desync : les 400 JSON de la série finale (`apres-d176-*`) ont `desync: false` à `ALACOD_CHECK_DISTANCE=0`, qui n'exerce pas la vérification de rollback.
Preuve séparée à la distance par défaut du runner (2), mêmes 20 graines et vague 4 que le § 2.5 (`--until-wave 4 --max-frames 8000`, pas de `ALACOD_CHECK_DISTANCE`) :

| Ligne (20 graines) | desync | issues |
|---|---:|---|
| avant_poste, 1 bot (VPS, `apres-d176-desync-avant_poste-1bot.json`) | 0 | objectif 20 |
| avant_poste, 2 bots (VPS) | 0 | objectif 20 |
| la_fonderie, 1 bot (Mac, `captures/s1-10-metrics/mac/apres-desync-la_fonderie-1bot.json`) | 0 | objectif 17, victoire 3 |
| la_fonderie, 2 bots (Mac) | 0 | objectif 17, victoire 3 |

**0 desync sur 80 parties au synctest de distance 2.** Limite inchangée : les vagues 5 à 10 n'ont pas de preuve de rollback dans ce lot.

Les journaux et JSON de ces séries : VPS `b2/target/metrics/polissage-zombies-map-equilibrage/`, copie `~/orca/workspaces/alacod/captures/s1-10-metrics/` (Mac) ;
événements et desync `la_fonderie` du Mac : `captures/s1-10-metrics/mac/`.



## 5. Traces et vérifications

### 5.1 Méthode de la preuve § 5 (`docs/conventions.md` §5, `scripts/trace-diff.py`)

Ce lot est autorisé à `BLESS` les traces de zombies seulement. Le commit `88041423` (reprise du coordinateur) a réécrit **54 traces** : 48 scénarios
`game: "zombies"` de `tests/scenarios/` et les 6 `generated/zombies/enemy_zombie_*`. Aucune trace d'un autre jeu n'a bougé :
`git diff --name-only origin/polissage-integration HEAD -- tests` liste 64 fichiers, soit ces 54 traces et 10 scénarios `.ron` (§ 5.3), et rien d'autre
sous `tests/` (les 328 autres traces, throne, gungeon, isaac, testbed et le reste de zombies, sont identiques).

Deux contrôles d'attribution, rejoués sur le Mac le 1er novembre (les 54 scénarios, **références et `.ron` d'avant** (`origin/polissage-integration`), `ALACOD_SCENARIO=<liste>`
sans `BLESS` ; le message d'échec donne la ligne de la première différence, ligne n = frame n−1) :

- **Contrôle B : armes nouvelles seules** (carte d'avant, `weapons.ron` nouveau) : **54 traces sur 54 diffèrent à la ligne 1 (frame 0)**, aucune ne tient.
- **Contrôle A : carte nouvelle seule** (`weapons.ron` d'avant, `avant_poste.ldtk` nouvelle) : seules **3 traces** diffèrent, à la ligne 1 :
  `avant_poste_demo`, `avant_poste_fenetre_coin`, `revue_murs_avant_poste` (les trois jouent `avant_poste`) ; **les 51 autres sont identiques au bit près
  à leur référence d'avant**, y compris les 10 scénarios dont les `.ron` ont changé (joués ici avec leurs `.ron` d'avant). La carte n'explique donc rien en dehors
  de ces trois, et les armes expliquent tout le reste. Journal : `captures/s1-10-metrics/mac/ctlA.log`, `ctlA.txt` (A), `ctlB.txt` (B).

Cause nommée par `scripts/trace-diff.py` sur les dumps complets (`ALACOD_DUMP_TRACE`, trois états : S0 = `origin/polissage-integration`, S1 = carte nouvelle +
armes d'avant, S2 = tête) :

- `idle`, S0 → S2 : première frame qui diffère **0**. Seules les lignes de `WeaponInventory` du joueur (net id 76) et des entités d'arme (`machine_gun`, `pistol`, `shotgun`,
  `weapon_pickup_*`) changent : mitraillette `spread 0.12 → 0.07` et `damage 8 → 7`, pistolet `firing_rate 2 → 3`, `damage 15 → 22`, `mag_size 6 → 9` (`mag_ammo: 6 → 9`),
  pompe `damage 30 → 20` et `range 400 → 320`, fusil `damage 35 → 55`, `firing_rate 4 → 3`, `mag_size 10 → 8`. `WeaponConfig::hash` inclut `firing_modes`
  (`crates/combat/src/weapons/mod.rs`, lignes 254–260) et `WeaponInventory` hache ses `Weapon` (lignes 515–528) : la valeur de départ des armes est dans le checksum avant le premier tir.
- `avant_poste_demo`, S0 → S1 (carte seule) : première frame **0**. Apparaissent les spawners ajoutés (`EnemySpawnerComponent` avec `activation_area`), leurs `GgrsNetId`, le
  compteur `GgrsNetIdFactory` (118 → 175), le net id du joueur (107 → 164), le `FlowFieldCache` (cases des nouvelles salles) et le nombre d'entités rollback (114 → 171).
- `avant_poste_demo`, S1 → S2 (armes seules) : première frame **0**, mêmes lignes d'armes que pour `idle`.

### 5.2 Une ligne par trace

Toutes les lignes ont la même première frame (0) ; la colonne « Cause » dit ce qui l'explique, la dernière dit si le `.ron` du scénario a dû changer (§ 5.3).

| Trace | 1re frame qui bouge (réf. d'avant → réf. nouvelle) | Cause | `.ron` modifié |
|---|---:|---|---|
| `ammo_burst` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | attentes de réserve (pistolet 9 coups : 2 puis 5 au lieu de 6 puis 0) |
| `ammo_shared_reserve` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `avant_poste_demo` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) **et** carte `avant_poste` (29 spawners au lieu de 5 : `GgrsNetIdFactory`, ids des joueurs, `FlowFieldCache`) | — |
| `avant_poste_fenetre_coin` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) **et** carte `avant_poste` (29 spawners au lieu de 5 : `GgrsNetIdFactory`, ids des joueurs, `FlowFieldCache`) | — |
| `bench_bullets` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `bench_horde` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `bots_four_mixed` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `bots_two_fonceurs` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `bullets_walls` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `buy_door` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `buy_perk` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `buy_wall_weapon` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `clone_duo` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | soldes et munitions recalés ; chute du joueur 1 f149 → f161 (attente `PlayerDowned` f165) |
| `clone_quad` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | `frames` 4674 → 6500 : la vague 5 arrive plus tard (mitraillette 7 dégâts) ; vagues 3/4/5 recalées à f3623/f4674/f6500 ; soldes remesurés |
| `clone_solo` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | soldes recalés (3960/4380/4690) |
| `dash_aim_change` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `dash_iframes` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | PV du joueur 1 recalés (balles de 7 : 65/58/30 au lieu de 60/52/20) |
| `dash_wall` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `door_open` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `downed_all_lose` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `downed_bleedout` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | rafale f10–100 → f10–160, chute avant f150, mort avant f1100 (7 dégâts) |
| `downed_revive` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | rafale f10–160, interaction f160–430, chute à f101 (f89 avant) |
| `drop_pickup_swap` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `equilibrage_joueurs_duo` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `equilibrage_joueurs_quad` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `feel_zombies` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | 17 → 11 zombies posés et séquence d'inputs réécrite (esquive, pistolet) : la mitraillette (7) et la pompe (20 par plomb) tuent moins vite |
| `four_players_idle` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `four_players_shooting` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `friendly_fire_cursed` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `friendly_fire_never` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_1_moving` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_1_still` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_2_moving` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_2_still` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_full_moving` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `generated/zombies/enemy_zombie_full_still` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `idle` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `immune_tag` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `la_fonderie_demo` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | `frames` 7600 → 14500 : la victoire de la graine 1 arrive vers f14 300 avec les nouvelles armes |
| `movement_melee` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `p2p_smoke` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `points_on_kill` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `profile_run_end` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `remote_first_fight` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `revue_murs_avant_poste` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) **et** carte `avant_poste` (29 spawners au lieu de 5 : `GgrsNetIdFactory`, ids des joueurs, `FlowFieldCache`) | — |
| `run_lose_summary` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `shoot_around` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `shop_tour` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `stat_move_speed` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `test_map_fenetre_graine_100` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `two_players_idle` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `two_players_shooting` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |
| `weapons_workout` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | attentes de munitions et d'arme active recalées (pistolet à chargeur de 9) |
| `window_repair` | 0 | f0 : `WeaponInventory` (hash des `firing_modes` de `weapons.ron`) | — |

### 5.3 Scénarios `.ron` modifiés (10) et tests recalés (2)

Un `.ron` ne change que si une attente ne tenait plus après les armes (valeurs d'arme, pas mécanique). Les attentes ne sont pas retirées : elles sont recalées sur la
valeur mesurée, sauf les cas suivants à lire en revue : `feel_zombies` (17 → 11 zombies posés et séquence d'inputs réécrite pour garder trois armes visibles en 60 s),
`clone_quad` (`frames` 4 674 → 6 500, vagues 3/4/5 à f3623/f4674/f6500 : la mitraillette à 7 dégâts retarde la vague 5 de 1 800 frames) et `la_fonderie_demo` (`frames`
7 600 → 14 500, victoire de la graine 1 vers f14 300 au lieu de f7 131). `clone_duo` : l'attente `PlayerDowned` avait été retirée par la reprise ; elle est **rétablie** à f165
(chute du joueur 1 mesurée à f161, `ALACOD_EVENTS=1`, f149 avant).

Tests Rust recalés (`crates/scenario/tests/expectations.rs`, nombre d'arme, une ligne chacun) :
- `ammo_reserve_matches_starting_weapons_at_creation` : réserve de Plomb 48 → **72** (pistolet : `mag_size 9 × mag_limit 8`, `6 × 8` avant).
- `player_downed_true_once_fallen` : frame 100 → **101** (mitraillette à 7 dégâts au lieu de 8 : le joueur 1 tombe à f101, f89 avant ; mesuré par `ALACOD_EVENTS=1 ALACOD_SCENARIO=downed_revive`).

### 5.4 Vérifications

- `make test_scenarios` sans `BLESS` (Mac, `.target-mac`, `ALACOD_MUTE=1`) : exit 0, `5 passed`, 1 133 s.
- `crates/scenario/tests/expectations.rs` : 50 passed.
- `make gen GAME=zombies` : voir les 6 traces `generated/zombies/enemy_zombie_*` (ligne 0, § 5.2).
- `make check` : sur le Mac, `make -k check` (`make check` s'arrête à `test_tools`) : format, `check_forbidden` (52 occurrences auditées), `check_rollback_registration`,
  `check_sim_boundary`, `test_crates`, `test_scenarios` (5 passed, 1 248 s) et `lint` des cinq jeux **verts** ; `test_tools` rouge sur le Mac seulement : 3 échecs de
  `test_claude_say` (hook vocal macOS, code de sortie 2), reproduits **sans** mes changements (`git stash`, même échec) donc étrangers au lot. Sur le VPS (`make check` du 31,
  `captures/s1-10-metrics/make-check-final.log`) `test_tools` passait (48 tests OK) puis `test_crates` échouait sur les deux tests recalés au § 5.3, depuis corrigés. Pas de `make check` unique
  entièrement vert d'un seul tenant : preuve en deux machines, à relire.
- Dettes : D173 (cause de mort absente d'`alacod-sim`), D174 (rythme du bot, plafonds), D176 (fermée par b4) — `docs/taches/dettes.md`.

### 5.5 Non vérifié

- Le lancement natif avec rendu d'`avant_poste` par William (§ 4.5, point 2) ; son verdict sur les armes.
- Les vagues 5 à 10 au synctest de distance 2 (§ 4.6).
- La répartition réelle des apparitions par salle (§ 4.3).
