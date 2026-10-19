# Rapport m3-t18-integration-sortie (b1), partie 1 : scénarios, campagnes, p2p, dossier de sortie

Fiche : `docs/taches/m3-t18-integration-sortie.md`. Branche `m3-t18-integration-sortie` depuis `origin/main` `a1edf161` : toute la vague 1 de M3,
T11d1, T11d2, T14, T15, D84, D69 E3c à E6. Machine : Mac M1 Pro partagé, 8 cœurs ; d'autres workers y tournaient, la charge allait de 17 à 60
pendant les mesures. Docker par OrbStack. Scripts et résultats bruts : `docs/taches/rapports/m3-t18-integration-sortie/`.

Ce que contient cette partie : les points 1, 2, 4 et 6 de la fiche. Le bench (point 3) se fera plus tard sur le VPS, au calme. Les vidéos et la
revue humaine (point 5) se feront sur le Mac avec William : le § 6 donne la liste.

**Code ajouté.** Outillage seulement, hors simulation ; aucune trace existante ne bouge (§ 7) :
- le moment clé `variant_death` (`crates/scenario/src/events.rs`) : une variante qui disparaît sans changement d'étage, donc un champion tué.
  Le changement d'étage retire aussi les ennemis de l'étage quitté, ce n'est pas une mort. Avant, aucun moment clé ne disait « champion tué »,
  alors que le critère 3 l'exige ;
- `alacod-sim --events <dossier>`, qui écrit les moments clés de chaque graine en JSON. `mechanics` compte aussi `champion_tue`, `rocher`, `curse`
  et `portail`.

## 1. Scénarios du clone (critères 3 et 5)

Les cinq scénarios sont nouveaux. Ce sont les seules traces bénies (`BLESS=1 SCENARIO=<nom>`, un par un).

Ce qu'ils ont en commun :
- **aucun `Armor` et aucun `PlayerScript::modifiers`** (D93) ;
- **aucun `floors:`** : la séquence par défaut du manifeste (`sous_sol` puis `sous_sol_profond`), sans `FloorsOverride` ;
- la politique d'`explorateur` de `games/isaac/assets/bot.ron`, écrite en ligne dans le scénario ;
- la visée à quatre directions de `pleureur` (aucun `aim:`).

Méthode de choix de la graine : la campagne `alacod-sim` du § 2 écrit les moments clés de chaque graine (`--events`). Le script prend la plus
petite graine qui réunit tous les moments demandés. Rejouée en scénario, une graine refait la même partie que dans `alacod-sim` : vérifié sur
`isaac_full_run`, `isaac_curse_run` et `isaac_devil_run`, dont les frames des attentes sont celles de la campagne. `isaac_duo` et `isaac_quad`
donnent des objets de départ (`items`) : leur partie n'est plus celle de la campagne, leurs frames viennent d'un premier passage du scénario.

| Scénario | Graine, joueurs | Frames | Moments clés (attentes) | Manque |
|---|---|---|---|---|
| `isaac_full_run` | 120, 1 bot | 5 700 | voir le détail ci-dessous | rien |
| `isaac_devil_run` | 103, 2 bots, `devil: IfHealthAbove("0.0")` | 6 870 | salle du `gardien` nettoyée (f1305) ; joueur 0 : `Stat(MaxHealth)` 6 → 4 → 2 (présentoirs du diable `cornes_noires` f1590, `queue_fourchue` f1612, `HasItem`) ; joueur 1 inchangé (7, son `pot_de_miel`) ; portail ; étage 2 avec `MaxHealth` 2 ; victoire f6864 | rien ; la politique du diable n'est pas celle par défaut, voir plus bas |
| `isaac_curse_run` | 2, 1 bot | 4 150 | curse `darkness` à l'étage 1 (moment `curse`, `Curse`, `HudText(curses)`) ; **effet** : `HudText(minimap)` vide à f30, f2000 et f4000, alors que la vie est affichée (« 6/6 ») ; mur secret, boss et phases, portail ; à l'étage 2 (f4097) : aucune curse, `curses` vide, la minicarte revient (« depart · visitée · actuelle ») | rien |
| `isaac_duo` | 2, 2 bots, `items` par joueur | 3 600 | familiers des deux joueurs à f10 et après la transition (`FamiliarCount`) ; larmes qui se divisent et larmes guidées (indices `feedback`) ; salles, boss et phases, portail, étage 2 ; champion `explosif` tué (f2964), joueur 0 à terre (f2970), puis réanimé (f3544) | — |
| `isaac_quad` | 1, 4 bots, `items` par joueur | 3 300 | familiers des quatre joueurs à f10 (1, 1, 1, 2), le joueur 3 achète `cousin` (3 familiers à f1500) ; larmes qui se divisent, guidées et perforantes ; boss et phases, portail, étage 2 avec 1, 1, 1, 3 familiers ; joueur 2 à terre (f2930) puis réanimé (f3181) ; les quatre vivants à f3299 | — |

**`isaac_full_run` en détail.** C'est le critère 3 complet, joué par le bot nu, avec victoire :
- salle verrouillée (f60), puis nettoyée (f225) ;
- trois achats en sous à la boutique : `glu` (larmes, f383), `pain` (f412), `encre` (larmes, f439) ;
- cœur ramassé (f1188) ;
- bombe de départ posée (f1247) : elle explose à f1321, ouvre le mur secret (f1322) et **casse trois rochers à la même frame** (`EventAfter(feedback, "rocher cassé")`). `Consumable(bomb)` passe de 1 à 0 ;
- `gardien` : verrouillé, phases 0 → 1 (f2122) et 1 → 2 (f2163), portail (f2328) ;
- étage 2 avec deux champions. L'`explosif` est tué (f3270) et l'`essaim` aussi (f3753). `EnemyVariant` vérifie `explosif` sur le net id 455 ;
- porte à clé du trésor ouverte (f4287), `Consumable(key)` = 0 ;
- second mur secret (f4436) ;
- familier `frere` acheté (f5032), `FamiliarCount` = 1 ;
- `colosse` : phases, puis victoire (f5680).

**Graines.** Sur les 200 graines à un bot, `isaac_full_run` est la plus petite qui réunit les treize moments et la victoire. Seule la graine 1
s'en approche : il ne lui manque que le rocher cassé par une bombe.

**Pourquoi deux bots pour le diable.** À un bot, `--devil above:2` sur les graines 1 à 20 ne donne aucun achat : le bot meurt avant le présentoir,
ou passe le portail sans entrer dans la salle. À deux bots, `--devil above:0` sur les onze graines dont le premier étage a une salle du diable
(`slot_assign pool=diable` du journal des campagnes) : trois graines achètent (103, 116, 119), les huit autres non
(`m3-t18-integration-sortie/diable-2bots.tsv`). Le défaut `devil: Never` de `bot.ron` reste celui des campagnes (Q14, non tranchée). Le scénario
déclare sa politique en ligne.

**Pourquoi `items` dans les scénarios à 2 et à 4.** Le critère 5 demande les familiers et les larmes de quatre joueurs. Une partie ordinaire ne
le garantit pas : dans la campagne à quatre bots, un familier est ramassé dans 10 graines sur 20, et rarement par plusieurs joueurs.
`items` donne à chaque joueur un objet de larmes et un familier dès la frame 1 :
- joueur 0 : `bifurque`, `frere` ;
- joueur 1 : `lynx`, `orbe` ;
- joueur 2 : `billard`, `cousin` ;
- joueur 3 : `aiguille`, `anneau`.

C'est la seule mise en place. Ce n'est pas un modificateur de confort : aucune stat de survie n'est touchée.

Le rebond de `billard` n'a pas d'indice `feedback` (§82). Aucune attente ne le prouve dans `isaac_quad` ; le commentaire « À regarder » le dit.

## 2. Campagnes (critère 4)

**Commande.** `alacod-sim --game isaac --bots N --profiles explorateur… --floors sous_sol --seeds A..B --until-floor 5 --max-frames 20000
--pickup-progress --log --events …`.
- `--floors sous_sol` désigne la séquence par défaut du manifeste (`entry.floors`) ;
- binaire de l'arbre de `main` `a1edf161` : seul `alacod-sim` diffère, par ses sorties ;
- politique de `bot.ron` (`devil: Never`) ;
- bombe de départ réelle, sans `--bombs` ;
- lots de 20 graines, `campagne.sh`.

Le journal `--log` est lu au vol par `flux.py`. Il donne le tueur de chaque joueur mort et les étages qui ont une salle du diable ; il n'est pas
gardé. Une partie perdue sans tueur dans le journal (piques, sa propre bombe, joueur à terre sans personne pour le relever) est rejouée par
`chutes.py`, qui relève la dernière chute de chaque joueur. Toutes les parties perdues à 2 et à 4 bots sont rejouées ainsi.

| Campagne | Graines | Victoires | Parties perdues | Soft-locks | Desyncs | Étage 1 franchi | Frames de victoire (médiane) |
|---|---|---|---|---|---|---|---|
| s1 : 1 bot | 1 à 200 | 24 (12,0 %) | 175 | **1** (graine 63, D121) | **0** | 79 (39,5 %) | 6 147 |
| d2 : 2 bots | 1 à 200 | 95 (47,5 %) | 105 | **0** | **0** | 160 (80,0 %) | 6 565 |
| q4 : 4 bots | 1 à 20 | 15 (75 %) | 5 | **0** | **0** | 20 (100 %) | 6 076 |

Les chiffres de s1 et de d2 sont **identiques graine pour graine** à ceux de T11d2 (même script, contenu de T11d2). T11d2 donnait : s1 24 / 175 /
graine 63 / 79, i2 95 / 105 / 0 / 160. D69 E3a à E6 et E5a, mergées depuis, n'ont rien changé à ces parties.

**La graine 63 est un vrai soft-lock (D121, nouvelle).** T11d2 l'avait classée « long trajet » en la rejouant avec une autre politique
([INFERENCE] d'après le texte de son protocole). Rejouée ici avec la politique de `bot.ron` et sans détecteur jusqu'à f4800 : aucun moment clé
après f918, et le joueur reste vivant. `ALACOD_BOT_DEBUG=1` montre le bot qui oscille sur place vers un tas de sous qu'il ne ramasse jamais. Son pas
devient nul toutes les 120 frames, ce qui remet sa poursuite à zéro (`input.rs:876`) : le but n'est jamais écarté.

**Causes des parties perdues.** On compte la dernière chute de chaque joueur. À 1 bot, c'est la chute du joueur ; une partie sans tueur dans le
journal (25 graines) a été rejouée par `chutes.py`.

| Campagne | Étage de la défaite | Causes |
|---|---|---|
| s1 (175) | premier 120, second 55 | `suiveur` 22, `araignee` 22, `tourelle_croix` 22, **piques 21**, `gardien` 19, `bourdon` 13, `cracheur` 12, `arroseur` 11, `mouche` 7, `follet` 7, `rampant` 7, `colosse` 4, sa propre bombe 3, pattern `couronne` 3, `spectre` 1 |
| d2 (105 parties, 210 chutes) | premier 40, second 65 | **piques 51**, `suiveur` 36, `araignee` 27, `follet` 19, `gardien` 18, `tourelle_croix` 16, `bourdon` 12, `arroseur` 7, `mouche` 5, `colosse` 4, `cracheur` 4, pattern `couronne` 4, `rampant` 3, bombe d'un coéquipier 1, `spectre` 1 |
| q4 (5 parties, 20 chutes) | second 5 | **piques 11**, `suiveur` 3, `araignee` 3, `arroseur` 2, `colosse` 1 |

**Les piques sont la première cause de chute à 2 et à 4 bots : le bot les ignore (D119, nouvelle).** `crates/bots` ne lit aucune surface de
dégâts. À la graine 118 à un bot, le joueur tire immobile sous les piques de f522 à f582, puis meurt. À plusieurs, un joueur réanimé reste
sur les piques et retombe. Les taux ci-dessus en sont biaisés vers le bas.

**Q14 n'est pas tranchée.** Ce dossier publie les taux, sans conclure sur la fourchette. William décide si la couverture des mécaniques
(ci-dessous) remplace la fourchette de victoires, et D119 pèse sur ces taux.

### 2.1 Couverture : le tableau « Atteignabilité » du §8.1, mesuré

On donne la part des graines où la mécanique est rencontrée. Elle est lue dans les moments clés de chaque graine. Les objets sont classés par
leur RON : tag `larmes`, champ `familiar`, `effects` hors consommables. Le détail graine par graine est dans `s1.tsv`, `d2.tsv` et `q4.tsv`.

| Ligne du §8.1 | Mesure | s1 (200) | d2 (200) | q4 (20) |
|---|---|---|---|---|
| Larmes modifiées, empilement | objet de larmes ramassé | 66 % | 88 % | 19 |
| | deux objets de larmes sur un même joueur | 48 % | 67 % | 13 |
| | deux exemplaires du même objet sur un joueur | 30 % | 45 % | 10 |
| Effets v3, transformations | objet à effets ramassé | 57 % | 86 % | 20 |
| | transformation | 0 % | 0 % | 0 : **le contenu `isaac` n'en déclare aucune** (Q10 ; `synergies/` n'a que `garde_rapprochee`) |
| Familiers, orbitaux | familier ou orbital ramassé | 39 % | 58 % | 10 |
| Bombes, rochers, piques | bombe d'un joueur qui explose (rayon 56) | 91 % | 98 % | 20 |
| | rocher cassé (par une explosion) | 30 % | 35 % | 6 |
| | piques | aucun moment clé : rencontrées au moins dans les chutes ci-dessus (21 parties à 1 bot) | | |
| Salle secrète | mur secret ouvert | 91 % | 98 % | 20 |
| Clés, trésor fermé | porte à clé ouverte (étage 2) | 22 % | 64 % | 18 |
| Cœurs, sous, boutique | cœur ramassé (`coeur`, `coeur_ame`, `pain`, `festin`, `eclat_ame`) | 60 % | 84 % | 18 |
| | sous ramassés | 100 % | 100 % | 20 |
| | achat en boutique (sous) | 65 % | 90 % | 20 |
| Pools à épuisement | objet tiré d'un pool et pris (coffre, présentoir) | 86 % | 94 % | 20 |
| Marché du diable | salle du diable sur un étage atteint | 42 % | 48 % | 9 |
| | achat au diable | 0 (`devil: Never`) | 0 | 0 ; avec `--devil above:0` à 2 bots : 3 graines sur 11 (§ 1) |
| Champions | champion apparu | 75 % | 90 % | 20 |
| | champion tué (`variant_death`) | 62 % | 84 % | 18 |
| Curses | curse tirée | 36 % | 44 % | 10 |
| Boss | boss et phases | 56 % | 87 % | 20 |
| | portail ouvert (premier boss tué) | 41 % | 80 % | 20 |
| Accomplissements | hors campagne | deux runs enchaînées : `crates/scenario/tests/achievements.rs` (vert dans `test_crates`), `achievement_unlocks`, `achievement_not_met` | | |

## 3. p2p réel (critère 7)

La recette est celle de T14 : les inputs des bots sont figés par `scripts/p2p-gungeon-inputs.sh` (`GAME=isaac FLOORS=sous_sol UNTIL_FLOOR=5
SIM_ARGS=--pickup-progress`), puis rejoués par `scripts/p2p-stress.sh` avec `EXPECT_FLOORS=1` :
- délai d'input 0 ;
- un client suspendu en boucle (SIGSTOP/SIGCONT) ;
- `MATCHBOX_PORT=3571`, `COMPOSE_PROJECT_NAME=b1` ;
- arbre de la branche, simulation de `main`.

Charge du Mac au lancement : 60 puis 51. Aucun script n'a eu besoin d'une « version isaac » : `GAME=isaac` suffit.

| Partie | Résultat | Mécaniques dans les journaux des clients |
|---|---|---|
| **2 clients**, graine 103 de d2, 3 000 frames | `p2p-stress : OK` ; codes 0 0 ; **0 `DesyncDetected`** ; 1 653 et 12 frames resimulées (client 0 : 247 rollbacks, profondeur moyenne 6,7, maximale 11) ; **transition d'étage f2608** sur les deux ; 2 999 lignes contiguës et identiques | `bomb_place` f235 et `secret_wall_open` f310 (bombe de départ) ; familier `escorte` f1378 (`familiar_spawn`, puis `familiar_shot` de f2088 à f2825, et réapparu au second étage f2609) ; objet de larmes `epine` acheté f2438 (`shop_buy`), et `talisman` f2460 |
| **4 clients** (smoke), graine 4 de q4, 3 200 frames | `p2p-stress : OK` ; codes 0 0 0 0 ; **0 `DesyncDetected`** ; 1 541, 364, 1 453 et 1 478 frames resimulées (profondeur maximale 11) ; **transition f2891** sur les quatre ; 3 199 lignes identiques | bombes f309 et f2930, murs secrets f384 et f3005 ; `glu` (larmes) f1364 ; `escorte` (deux familiers) f2604 ; `ombre` f3164 |

**Deux machines (D73).** Ce n'est pas faisable depuis le Mac seul : les clients ci-dessus tournent sur une même machine.
- Ce qu'il faut : un client sur le VPS, x86-64, avec un `matchbox_server` sur l'une des deux machines. La recette est dans
  `m2-p2p-deux-machines.md` § 3.4.
- Ce que je n'ai pas fait : je n'ai pas touché le VPS, qui est le poste d'un autre worker.
- Ce qui existe : la dernière preuve à deux machines pour `isaac` date du 2026-10-18 (`7ee084e8`), avec une transition d'étage à 2 et à 4
  clients. Mais c'était le contenu d'avant T11d2 : 49 objets, sans `colosse`.
- Sur le contenu actuel, c'est donc **non vérifié**. Les deux fichiers d'inputs se régénèrent avec les commandes ci-dessus : graine 103 à 2
  bots, graine 4 à 4 bots.

## 4. Dossier de sortie : critères du §8.1

| # | Critère | État | Preuve |
|---|---|---|---|
| 1 | Contenu (Q1), `test:` et scénario généré par définition, lint sans avertissement | **partiel** | Volume : 100 objets, dont 37 de larmes, 9 familiers et 11 actifs ; 10 ennemis, dont 3 à champions ; 2 boss ; 23 gabarits (`m3-t11d2-volume.md` § 1). `make lint` : `games/isaac` aucune erreur, **0 avertissement**. `make gen GAME=isaac` : rc 0, 213 scénarios générés « ok ok », aucun fichier modifié. **Manque** : 9 objets sans `test:`, tous des consommables (`blank`, `bomb`, `coeur`, `coeur_ame`, `eclat_ame`, `festin`, `key`, `pain`, `ration`). `ItemHeld` refuse un consommable (D114). |
| 2 | Empilement prouvé | **fait** | `combat::projectile` teste les 72 paires ordonnées de variantes différentes et les 9 paires identiques (M3-T1) : vert dans `make test_crates`. `tears_stack_two_copies`, `tears_stack_three` et `effect_per_copy` sont verts dans la suite. |
| 3 | `isaac_full_run`, `isaac_devil_run`, `isaac_curse_run` | **fait** | § 1 : tous les moments clés, bot nu, séquence par défaut. Réserve : le diable est joué à 2 bots avec la politique `IfHealthAbove("0.0")`. |
| 4 | Bots : 200 graines à 1 et 2 bots, 0 soft-lock, 0 desync, taux de l'étage 1 dans la fourchette Q14 ; 20 graines à 4 bots sans desync | **partiel** | § 2 : 0 desync sur 420 graines. **1 soft-lock** à 1 bot (graine 63, D121). Taux publiés : étage 1 franchi 39,5 %, 80 % et 100 %. **Fourchette Q14 non fixée**, couverture publiée (§ 2.1). |
| 5 | Synctest à 2 et 4 : `isaac_duo`, `isaac_quad` | **fait** | § 1 : verts dans la suite (synctest), familiers et larmes des 2 et 4 joueurs. |
| 6 | Performance : `bench_tears`, `bench_floor_isaac`, machine nommée au calme | **non vérifié** | Ce sera sur le VPS, au calme, plus tard. Dernières valeurs connues, poste chargé, ni ce commit ni ce protocole : pire rollback de `bench_tears` 31,3 / 32,5 / 31,4 ms sur le VPS (revue de dette-d84-budgets-calme, 2026-10-19 ; budget `max_rollback_ms` 65). Les métriques de la suite sur ce Mac chargé ne valent pas mesure. |
| 7 | p2p réel : 2 clients à inputs actifs, transition avec bombe, larmes et familier ; smoke à 4 ; deux machines si D73 le permet | **partiel** | § 3 : 2 clients et 4 clients faits sur une machine, traces identiques, 0 `DesyncDetected`. **Deux machines : non vérifié** sur le contenu actuel. |
| 8 | Humain : trente minutes en coop, notes fermées | **non vérifié** | Ce sera sur le Mac avec William : § 6. |

Le contrat de `docs/etat-engine.md`, en bref :
- (1) boucle accessible : critères 3 et 4 ;
- (2) régression : § 7 ;
- (3) P2P intégré : critère 7, sans boss ni restart en p2p dans cette partie ;
- (4) réseau dégradé : non refait pour `isaac`, aucun `NETEM` ;
- (5) plateformes : non refait (dépend des deux machines) ;
- (6) performance : critère 6 ;
- (7) humain : critère 8.

## 5. Porte des dettes (§4 de `docs/taches.md`) : dettes ouvertes par M3

Une proposition pour la sortie, dette par dette. Le sort (corrigée, planifiée, tranchée) appartient à l'orchestrateur et à William. Les lignes
`D78`, `D89` et `D93` de `dettes.md` reçoivent le constat de T18. D119 à D121 sont nouvelles.

| Dette | Ouverte par | État | Pour la sortie de M3 | Raison |
|---|---|---|---|---|
| D74 | plan-m4-isaac | tranchée (corriger le contenu gungeon) | non bloquant | gungeon, pas `isaac` |
| D75 | plan-m4-isaac | corrigée de fait par M3-T2 (`ItemDef::stack`, `effect_per_copy` vert) ; la ligne n'a pas de sort écrit | non bloquant | le critère 2 passe ; à fermer dans la ligne |
| D76, D79, D80, D90, D91 | plan / T0b / T4 / Q12 / Q10 | fermées | — | — |
| D78 | M3-T0f | réduite (contenu) ; `isaac_duo` et `isaac_quad` verts (T18) | non bloquant | restent le repli sans panique et le lint |
| D81, D88, D95, D96, D97, D100, D105, D106, D117 | M3 | fermées | — | — |
| D83 | M3-T2 | ouverte, sans sort | non bloquant | arme lente au départ, aucun critère touché |
| D89 | suivi Q17 | réduite (T18 : cinq scénarios en `FourWay`) | non bloquant | restent cinq scénarios épinglés `Free` ; leurs traces existantes ne bougent pas ici |
| D92 | suivi Q12 | ouverte, question à William | non bloquant | crédit des points d'un charmé |
| D93 | dette-d88 | critère 3 tenu sans armure (T18) ; cinq anciens scénarios restent sous armure | non bloquant | le risque nommé, un critère 3 sous armure, ne s'est pas produit |
| D98 | M3-T15 | ouverte | non bloquant | le poste de référence est le VPS (critère 6, à mesurer) |
| D99 | M3-T11d1 | ouverte, faible | non bloquant | aucun cas dans les 420 graines (0 soft-lock à 2 bots) |
| D101, D102, D104 | M3-T14 | ouvertes | non bloquant | bot seulement, aucun contenu `isaac` concerné (type de salle déclaré, aucun `Mul`/`Set` au diable) |
| D103 | M3-T14 | ouverte | non bloquant | p2p à 2 et 4 avec bombes : traces identiques (§ 3) ; une bombe en trop n'est pas un desync |
| D107 | M3-T11d1 | ouverte (garde à 2 700 évaluations depuis D117) | non bloquant | coût de génération, hors bench |
| D110 | réconciliation T14 / gungeon | ouverte | non bloquant | bot seulement |
| D111 | dette-d97 | ouverte, sans sort | non bloquant | déterministe ; arrondi par `f32` à trancher (règle 7 de `CLAUDE.md`) |
| D112 | M3-T11d2 | ouverte | non bloquant | `nova` et `hydre` existent sous une forme réduite |
| D113 | M3-T11d2 | ouverte | **bloquant pour la revue coop (critère 8)** | `nova` (10 dégâts, rayon 100) tue un coéquipier à 6 PV : William doit trancher (tir ami voulu, Q4, ou correctif) avant les trente minutes à deux |
| D114 | M3-T11d2 | ouverte | **bloquant pour le critère 1 tel qu'écrit** | 9 consommables sans `test:` ; sinon William accepte que les consommables soient prouvés par scénario (`isaac_consommables_lot`) |
| D115 | M3-T11d2 | ouverte, mineure | non bloquant | lint seulement |
| D116 | dettes-d105-d109 | corrigée, reste l'arrivée tardive | non bloquant | blocage borné, jamais un desync |
| **D119** | **M3-T18** | ouverte | non bloquant pour les critères ; **à corriger avant de fixer Q14** | le bot ignore les piques : première cause de chute à 2 et 4 bots, taux biaisés |
| **D120** | **M3-T18** | ouverte, mineure | non bloquant | échange d'actifs en boucle au ramassage (charge perdue), sans effet sur le déterminisme |
| **D121** | **M3-T18** | ouverte | **bloquant pour le critère 4** (0 soft-lock) | soft-lock de la graine 63 à 1 bot, cause dans le bot (poursuite remise à zéro) |

Les dettes d'avant M3 (M2, D73 à D77, D82, D84 à D87, D94, D108, D109, D118) ne sont pas dans ce tableau. D94 (nightly sans allumette ni vidéos)
reste une condition de la CI, pas d'un critère.

## 6. Vidéos et revue humaine (critères 5 et 8) : à faire sur le Mac

Commandes : `make videos SCENARIO="isaac_full_run isaac_devil_run isaac_curse_run isaac_duo isaac_quad"`, `make views SCENARIO=isaac_quad` et
`make views SCENARIO=isaac_duo` (une vue par joueur), puis `make review_videos`.

Ce que William regarde (frames du « À regarder » de chaque scénario) :

| Vidéo | Ce qu'il faut juger |
|---|---|
| `isaac_full_run` | f1247 à f1322 : la mèche, l'anneau d'explosion, les trois rochers cassés et le mur secret ouvert se lisent-ils ? f383 à f439 : achats, bandeau du nom de l'objet. f1188 : le cœur ramassé se voit-il dans les cœurs du HUD ? Étage 2 (f2780 et suivantes) : la teinte des champions `explosif` et `essaim` se reconnaît-elle, et leur mort (f3270, f3753) se comprend-elle ? f5032 : l'apparition du familier `frere` ; les phases du `colosse`. |
| `isaac_devil_run` | f1590 et f1612 : le paiement en vie maximale est-il lisible dans les cœurs du HUD (6 → 4 → 2 PV) ? Le prompt du présentoir annonce-t-il le prix en cœurs ? |
| `isaac_curse_run` | Étage 1 : l'absence de minicarte et l'indication « Malédiction : darkness » ; f4097 : son retour à l'étage 2. |
| `isaac_duo`, `isaac_quad` (et leurs `views`) | Lisibilité des larmes par modificateur (division, guidage, perforation ; le rebond n'a pas de signe) ; à qui appartient chaque familier quand quatre joueurs en ont six ; la mise à terre et la réanimation (`isaac_duo` f2970 à f3544, `isaac_quad` f2930 à f3181). |

**Trente minutes en coop (critère 8).**
- En local : `make isaac` (solo) ; `APP_VERSION=dev cargo run -p isaac --features native -- --local-port 7000 --players localhost localhost` (deux héros, mêmes touches : voir `docs/jouer.md`).
- En ligne, la vraie coop : `docs/jouer.md`, « En ligne à deux », avec `-p isaac` / `target/debug/isaac` à la place de `gungeon`, et `--seed <n>` identique des deux côtés.

Graines proposées :
- 120 : la partie de `isaac_full_run` ;
- 103 : la salle du diable au premier étage ;
- 2 : la curse `darkness`.

Notes attendues : lisibilité des larmes, des cœurs (cœurs d'âme compris), des familiers et des champions ; le tir ami de `nova` (D113). Chaque
note devient une attente, une dette ou une tâche (fermeture des notes).

## 7. Vérifié (résultats réels, Mac chargé)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 3 tranches ok, **807 scénarios** (802 de `main` + 5 nouveaux), 677 s. `git status` :
  aucune trace modifiée, seules les cinq `.trace` nouvelles s'ajoutent.
- `make test_crates` : **1 303 passés, 0 échec**.
- `make lint` (les cinq jeux) : aucune erreur. isaac 0 avertissement, gungeon 1 et testbed 115 comme avant.
- `make gen GAME=zombies|testbed|throne|gungeon|isaac` : rc 0 pour chacun, aucun fichier modifié ensuite.
- `cargo fmt --all -- --check` : propre. `scripts/check-forbidden.sh` : OK (47 occurrences auditées). `make test_tools` : OK.
- Les cinq scénarios, joués seuls avec `BLESS=1` à leur création, puis sans BLESS dans la suite : toutes leurs attentes sont vertes.
- Campagnes : § 2. p2p : § 3, deux exécutions de `p2p-stress.sh`, « OK ».
- Non fait : le bench (critère 6), les vidéos et la revue humaine (critères 5 et 8), les deux machines (critère 7), le réseau dégradé pour
  `isaac`.
