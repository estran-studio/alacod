# Rapport m1-d43-d44-fin-de-partie — fin de partie et soft-lock (D43 + D44)

**Branche** `m1-d43-d44-fin-de-partie`, partie de la tête livrée de m1-d41-spawns-degages
`62acb13` ; `origin/main` `f242633` mergé. Consigne d'orch (dix lignes acceptées : D43 et D44
deviennent de l'outillage).

## État en cours

- **Fait** : diagnostic, outillage, scénario de preuve, vérification (§4). Livré.

## 1. D43 : diagnostic

**La règle de la simulation est correcte**, en `Waves` et en `Floors`, à 1 et 2 joueurs :
`character::health::rollback_check_defeat` (`DeathManagement`, sans condition de mode) déclare
`Defeat` dès que tous les joueurs **présents** sont à terre ou meurent cette frame (un mort est
détruit et ne compte plus) ; `rollback_apply_bleedout` pose `Death` au bout de `bleedout_frames`
(1 800 par défaut, celui du pilote de throne). Les vagues étaient déjà couvertes
(`downed_all_lose`, `downed_bleedout`, `run_lose_summary`) ; `Floors` à deux : nouveau scénario
`throne_duo_defaite` (§3).

Les « parties sans fin » vues par b1 (21 soft-locks à deux bots avec un joueur à terre, relus
dans les séries de m1-integration-scenarios et du lot navigation) sont de deux sortes :

- **(a) défaite déjà déclarée** (ex. graine 3 de la série d'intégration : `Ended Defeat` à
  f3243, dernier joueur à terre, l'autre mort) : `alacod-sim` ne s'arrêtait que lorsqu'aucun
  joueur n'existait plus ; le saignement (1 800 frames) dépassant la fenêtre du soft-lock (1 200),
  la graine finissait en **faux soft-lock**. Défaut d'outil, corrigé (§2).
- **(b) partie en cours, un joueur à terre, l'autre debout** (ex. graine 2 à deux bots, série du
  lot navigation `simN` : joueur 0 à terre, joueur 1 à 100 PV ; après D43/D44, **graine 14 à deux
  bots** : joueur 0 à terre, joueur 1 à 100 PV, tourelle et franc-tireur restants) : conforme aux règles (la partie
  continue tant qu'un joueur est debout). **Défaut de bot, pour b1** : le bot debout ne vient pas
  relever son coéquipier (il ne réanime jamais) et ne tue plus rien ; le coéquipier saigne
  jusqu'à la mort. À reprendre côté bots (« réanimer avant de chasser »).

## 2. Outillage (aucune trace ne bouge)

- **D43** : `StopEarly::stop_when_run_ended` (activé par `alacod-sim`) arrête la simulation dès
  que `Run.step` est `Ended` ; `Metrics::run_end` (issue et frame) ; `SimResult::run_end` dans le
  JSON et colonne « fin » (`Defeat f…`, `Victory f…`, `soft-lock`, `objectif`).
- **D44** : `runner::FloorsProgress { floor, enemies, enemy_health, players }` ;
  `progressed` : niveau changé, ennemi en moins, **santé totale des ennemis en baisse** (dégâts
  sans kill : combat lent contre le boss), ou **état d'un joueur changé** (à terre, relevé,
  mort). Une santé qui remonte (régénération) ne compte pas et devient la nouvelle référence.
  Unitaire `floors_progress_tests`.
- `docs/conventions.md` §24 : une ligne.

## 3. Preuve en `Floors` : `throne_duo_defaite`

Graine 28 (trouvée parmi 21..40 à deux bots : la seule où l'un tombe à terre puis l'autre meurt) :
étages f530 et f1836 ; joueur 0 à terre à f2481 ; joueur 1 mort à f2744 : **défaite la même
frame**, le joueur 0 encore à terre (saignement jusqu'à f4281). Attentes : `FloorIndex`,
`RunState(Playing)` à f2743, `PlayerDowned` 0, `PlayerAlive` 1, `Defeat` by f2744,
`RunState(Ended(Defeat))` à f2745, `PlayerDowned` 0 à f2770. Nouvelle trace (à bénir).

## 4. Vérifié

- Unitaire `floors_progress_tests` ; suite (`--include-ignored`, 15 crates) : 566 verts ;
  `scenarios` : **aucune trace existante ne bouge** (seul `throne_duo_defaite`, nouveau, sans
  trace) ; doctest `game::waves` (préexistant).
- **20 graines** (`alacod-sim --game throne --floors run --seeds 1..20 --until-floor 3
  --max-frames 15000`, bots `prudent`), avant (lot navigation, `simN`) et après D43/D44 :

| Bots | Avant | Après : finies | défaites | soft-locks |
|---|---|---|---|---|
| 1 | 0/20 (8 SL, 12 morts) | **3/20** | 12 | 5 |
| 2 | 3/20 (17 SL) | **9/20** | 0 | 11 |
| 4 | 11/20 (9 SL) | **17/20** | 0 | 3 |

0 desync. La simulation est inchangée (aucune trace ne bouge) : l'écart vient de l'outil.
D44 laisse aller à leur terme des combats lents (boss, tourelle à distance) que l'ancienne
heuristique coupait après 1 200 frames sans kill, et D43 range les défaites à part. Soft-locks
restants : ennemis que les bots ne vont pas chercher (souvent **hors de vue** d'après le relevé
D42), bot seul qui ne prend pas le portail (graines 3 et 16, déjà transmises), et le cas (b).
Résultats : `alacod_tasks/<tâche>/d40/cal/simE_{1,2,4}.json`.
- `docs/conventions.md` §24 : une ligne.
