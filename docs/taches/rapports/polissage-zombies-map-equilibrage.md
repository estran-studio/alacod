# polissage-zombies-map-equilibrage (S1-10) — rapport, points 1 et 2 (b3, préparation)

Branche `polissage-zombies-map-equilibrage` (depuis `origin/polissage-integration`), clone `zombies`, fiche
`docs/taches/polissage-zombies-map-equilibrage.md`. **Points 1 et 2 seulement** (audit et campagne « avant ») : le réglage des
cartes et des armes, la campagne « après » et les `BLESS` sont à b2 (S1-10, lun 2 nov). Aucune carte, arme, scénario ni trace
modifiés, aucun `BLESS`. Commit de référence de la campagne : celui de `origin/polissage-integration` au départ (`facb7c92`).

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

## 3. Dettes (D173–D175)

Ajoutées à `docs/taches/dettes.md` (D173–D175 ; D167–D169 sont pris par ailleurs).

## 4. Reste à faire

- Point 3 (placer des spawners et des fenêtres dans les 8 autres salles d'`avant_poste`, régler les armes), point 4 (campagne « après », `BLESS` avec
  preuve §5) : b2, lun 2 nov, sur la même branche.
- Cible pour b2 : les `plafond` à faible vague (§ 2.3), la mitraillette de départ qui écrase les autres armes, le fusil jamais acheté.
- William : 15 minutes de jeu et notes sur les armes (hors de ce lot).

## 5. Vérifications

`env -u CI make format check_forbidden lint` : code 0 (`cargo fmt --check` propre, `check-forbidden.sh` sans occurrence nouvelle, `alacod lint`
des quatre jeux sans erreur ; seul avertissement : `AiSchema::sight_range` jamais lu, déjà présent sur `origin/polissage-integration`).
Aucun fichier de carte, d'arme, de scénario ni de trace modifié (`git status` : rapport, `docs/taches.md`, `docs/taches/dettes.md`).
`make check` non lancé (rien de code), comme demandé.
