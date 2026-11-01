# polissage-throne-complet — § S2-12 : cavernes de throne (audit, causes, corrections)

Branche `polissage-throne-map`, base `origin/polissage-integration` `0c39eb1d` (contient D176), VPS ovh-inner. Fiche : `docs/taches/polissage-throne-complet.md`
points 1 (cavernes) et 4. Les parties S2-10 (visuel) et S2-11 (sons) ont leurs rapports propres. Conventions : `docs/conventions.md` § 104.

## 1. Audit des cavernes (100 graines, fait, mesuré)

Les cavernes de throne sont générées par automate cellulaire (`crates/world/src/cave.rs`) : il n'y a **pas de salles typées**. Mesures par graine
(exemple Rust jetable `cave_audit`, non commité, D178 ; spawns avec `spawn_clearance` 1/1/2 et gabarit grand aux niveaux 2-3, approximation du registre) :

| | niveau 1 (48×32, 4 spawns) | niveau 2 (56×40, 6) | niveau 3 (64×44, 9) |
|---|---|---|---|
| sol, médiane (min–max) | 1 083 (922–1 184) = 0,70 | 1 659 (1 500–1 791) = 0,74 | 2 134 (1 869–2 268) = 0,76 |
| chambres 5×5 (composantes de cases ouvertes à 2) : médiane / dont ≥ 12 cases | 2 / 1 | 2 / 1 | 3 / 1 |
| plus grande chambre (cases) | 503 | 828 | 1 086 |
| cases de couloir (hors `is_open`) médiane | 284 | 402 | 510 |
| impasses (sol à ≤ 1 voisin) médiane (max) | 1 (3) | 1 (6) | 1 (8) |
| spawns placés / demandés | 4/4 sur 100 graines | 6/6 | 9/9 |
| paires de spawns dans la même chambre, médiane | 0 | 0 | **28** (100 graines > 0) |
| distance de chemin joueur → spawns (cases) | 26–55 | 32–54 | 30–56 |

**Défaut relevé** : une grande salle ouverte (1 chambre dominante, sol 70-76 %), aucune variété d'une graine à l'autre et, au niveau 3, les 9 spawns
dans la même chambre. Pas de couloir d'une case problématique (D48 filtre déjà les poches), spawns toujours placés.
Rythme (campagne avant, 1 bot) : étages franchis à f ~1 000 / ~2 500 / ~4 300-4 900 ; durée médiane 4 290 frames.

## 2. Les 23 soft-locks de S2-7 : une seule cause

Rejoués : solo 13, 45, 69 ; duo 7, 9 (`--log`, JSON `softlock`). Tous : portail ouvert, un joueur **sur** l'ancre, aucun ennemi, et pourtant pas de transition.
Cause (fait, vérifié sur les 5) : `floor_transition_system` n'agit pas tant qu'un choix de mutation est ouvert en `BetweenFloors`
(`crates/map_ldtk/src/game/floors.rs:505-512`) ; deux niveaux gagnés avant le portail = deux choix de `choice_frames` 600 ; les bots ne pressent jamais
`ChoiceA/B/C` ; 2 × 600 = 1 200 = `FLOORS_SOFTLOCK_FRAMES` (`runner.rs`) : le détecteur conclut à un soft-lock **à la fin** du second choix, une frame avant que
la transition ne parte. Ex. graine 13 : portail f1820, `mutation rancune` f2421 (choix 1), choix 2 ouvert, fin à f3021. Ce n'est ni la carte, ni le moteur de jeu : bot + détecteur (D177).
Les 23 sont de ce type (10 solo à l'étage 2 ; duo 3 à l'étage 2 et 10 à l'étage 3) : après correction il en reste 0 (§ 4) ; les 18 non rejoués sont couverts par la campagne, pas un à un.

## 3. Corrections (contenu seulement)

- `caves/niveau_1.ron` `fill_ratio` 0,38 → 0,42 ; `niveau_2.ron`, `niveau_3.ron` 0,38 → 0,40 (0,42 ne place plus tous les spawns du niveau 3 : 2 graines sur 100 à 8 points, hors tourelle/boss garantis ; 0,45 perd des spawns dès le niveau 2).
- `progression/run.ron` `choice_frames` 600 → 300.

Chambres médianes (5×5, ≥ 12 cases) 2/2/3 → 4/4/5 ; sol médian 0,70/0,74/0,76 → 0,61/0,70/0,72 ; paires de spawns dans la même chambre au niveau 3, médiane 28 → 18. Spawns tous placés sur 100 graines aux trois niveaux.

## 4. Campagne avant / après (mêmes commandes, graines 1..100, `--until-floor 3 --max-frames 15000`, `alacod-sim` sans dev-dylib)

Commande : `target/headless/alacod-sim --game throne --bots N --profiles prudent[,prudent] --floors run --seeds A..B --until-floor 3 --max-frames 15000 --json …`
(3 plages de graines × solo et duo, 6 processus). « Avant » = contrôle sur la base `0c39eb1d` avec D176 : **identique** aux chiffres de S2-7 (D176 ne change rien à throne).
JSON : `polissage-throne-map-avant-{1bot,2bots}.json`, `polissage-throne-map-apres-{1bot,2bots}.json` (même dossier).

| | 1 bot avant | 1 bot après | 2 bots avant | 2 bots après |
|---|---|---|---|---|
| runs complètes (étage 3 franchi) | 41 | **50** | 86 | **99** |
| morts (étage 3 / étage 2) | 49 (48 / 1) | 50 (50 / 0) | 1 (1 / 0) | 1 (1 / 0) |
| soft-locks (aucun progrès 1 200 frames) | 10 | **0** | 13 | **0** |
| plafonds (15 000 frames) | 0 | 0 | 0 | 0 |
| desync | 0 | 0 | 0 | 0 |
| durée médiane / max (frames) | 4 290 / 6 412 | 4 106 / 6 125 | 4 120 / 5 557 | 3 463 / 4 521 |
| dégâts subis, médiane | 124,5 | 123,5 | 69 | 70 |

Lecture : les soft-locks étaient un artefact (10 + 13 → 0), les runs perdues deviennent des runs complètes ; la difficulté de l'étage 3 en solo (50 % de morts) n'a pas bougé (ni armes ni ennemis touchés). Le critère « pas de kill depuis > 4 000 frames » de la fiche est plus lâche que le détecteur de 1 200 frames : 0 plafond dans les deux cas.

## 5. Traces

BLESS des **9 traces de throne** seulement (`throne_duo_defaite`, `_floor_1`, `_mutation_choice`, `_progression`, `_quad`, `_rodeur_vue_graine_4`, `_softlock_recul`, `_solo`, `_three_floors`) ;
`make test_scenarios` complet avant BLESS : seules ces 9 traces diffèrent (aucun jeu autre, aucun scénario généré). Preuve par A/B de levier (`docs/conventions.md` § 104) :
cavernes seules : 9/9 divergent à la ligne 0 ; `choice_frames` seul : 6 divergent au premier choix de mutation (lignes 573-1120), 3 intacts. Attentes recalées à la main (repères de portail, étage, mutation, mort), `throne_mutation_choice` réenregistré,
`throne_rodeur_vue_graine_4` vise le `rodeur` `NetId` 91. Après BLESS : les 9 passent sans BLESS.

## 6. Reste / non vérifié

- **Non vérifié** : le ressenti (variété à l'écran) — VPS sans rendu (D94). **Vidéo de 60 s à faire sur le Mac** : `ALACOD_MUTE=1 make videos SCENARIO=throne_three_floors,throne_progression`.
- Dettes : D177 (bots/détecteur, remède moteur ou bots), D178 (outil d'audit de caverne), D179 (scénarios liés à la géométrie).
- 18 des 23 soft-locks non rejoués un par un (même signature dans le JSON, absents après correction).

## 7. make check

Vert (`env -u CI make check`, après recalage de trois tests Rust de `crates/scenario/tests/expectations.rs` : f469, `Niv. 3` à f1880, `Étage 3` à f2460, caverne du pillard enregistré remise à 0,38). Dernières lignes :

```
games/isaac : aucune erreur (24 personnages, 3 armes, 2 armes de corps à corps, 0 vagues, 3 cartes)
✓ All checks passed
```
