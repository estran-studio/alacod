# m1-analyse-200-throne — pourquoi les bots perdent sur `throne` (critère §9.8, 200 graines)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-analyse-200-throne` créée depuis `origin/main`). Tâche d'**analyse** (1 j) : aucun changement
de code ni de contenu, pas de trace bénie ; des scénarios sauvegardés peuvent être ajoutés
(décision 4). Calcul : seulement après le **feu vert** d'orch (la machine est prise par les
200 graines puis par une vérification), `CARGO_BUILD_JOBS=2`.

## Contexte

Le critère de sortie de M1 (`docs/plan-engine.md` §9.8) : « les bots finissent le clone sur 200
graines sans soft-lock ni desync ». L'orchestrateur joue les 200 graines de `throne` sur `main`
`61ac539` (bots de m1-v3-bots-portail inclus, réanimation de b1 pas encore mergée) :

```
alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run --seeds a..b \
  --until-floor 3 --max-frames 15000 --progress --json throne-2-a-b.json
```

Résultat provisoire à 118 graines : 84 finissent les trois étages, **0 desync**, 7 soft-locks,
le reste en **défaite** (presque toujours au niveau 2, un mort sur deux, entre 2 700 et 4 400
frames). Le critère n'exige ni 200/200 ni l'absence de défaite, mais les défaites et les
soft-locks disent où sont les limites : des bots, du contenu (ennemis, cavernes, réserves), ou de
l'engine. C'est ce que cette tâche doit établir, chiffres à l'appui, pour décider les prochaines
tâches de bots (b1) et de contenu.

## Données

Déposées par l'orchestrateur dans `/home/wq/Project/bascanada/alacod_tasks/m1-200-throne/`
(hors dépôt, hors `/tmp`) : `throne-2-*.json` et `.log` (2 bots), puis `throne-4-*` (4 bots,
lancés ensuite), `alacod-sim-61ac539` (le binaire exact), `main_sha.txt`. Chaque graine a dans le
JSON : `seed`, `floor`, `frames`, `deaths`, `desync`, `softlock`, `run_end`, `floor_frames`,
`dodges`, le relevé D42 des ennemis restants (soft-lock) et `players_end` (D36 : solde, à
terre, position). Lire `crates/scenario/src/bin/alacod-sim.rs` pour les noms exacts.

## Décisions (fixées ici)

1. **Classer chaque échec** (défaite ou soft-lock) dans une table : graine, étage, frames,
   `floor_frames`, cause présumée parmi un vocabulaire court que tu fixes (par exemple
   `tireur à distance`, `chargeur`, `boss`, `munitions à zéro`, `coéquipier non relevé`,
   `ennemi hors de vue`, `portail non pris`, `caverne fermée`…), et la preuve (champ JSON ou
   ligne de log). Les 200 graines à 2 bots d'abord ; 4 bots quand elles arrivent.
2. **Rejouer** au plus **12 graines** représentatives (les causes les plus fréquentes, 2 à 3
   par cause) avec le binaire fourni et `--save-scenario <dossier>` pour obtenir le scénario,
   puis les examiner avec les attentes qui existent (`EntityHealth`, `EnemyDistance`,
   `FloorIndex`, `Ammo`, `AmmoReserve`, `PlayerDowned`, `HudText`… liste dans `CLAUDE.md`) sur `make play_scenario` /
   la suite, **après feu vert**. Pas plus de **3 vidéos** (`make videos SCENARIO=…`), et
   seulement si une cause reste illisible dans les chiffres.
3. **Comparer** aux 20 graines du journal (`docs/taches.md` §9 : 2 bots 9/20 avec l'ancien
   outil, puis 7/20 et 18/20 à 4 bots après bots-portail) : les taux sur 200 confirment-ils ?
   Quel est le taux par étage (combien meurent à l'étage 1, 2, 3) ?
4. **Scénarios figés** : pour chaque cause majeure (au plus 3), un scénario sauvegardé dans
   `games/throne/assets/scenarios/throne_defaite_<cause>.ron` (joueurs **scriptés** ou bots, au
   choix de la reproductibilité : m1-d43-defaite-scriptee a montré qu'un scénario à bots casse à
   chaque correctif de bots), avec une attente qui fixe l'état mesuré (comme `throne_solo`).
   Traces à bénir par l'orchestrateur (nouvelles traces : pas de preuve de changement à fournir,
   seulement le relevé mesuré dans le rapport).
5. **Livrable principal** : `docs/digests/m1-200-graines-throne.md` : commande, sha, machine ;
   tableau des causes (effectif, part, étage) ; taux par étage ; les trois correctifs proposés,
   chacun avec sa voie (bots b1 / contenu / engine), sa preuve, son effet attendu, son coût
   estimé ; ce que les 200 graines disent du critère (atteint : 0 desync ; soft-locks : nombre et
   causes ; défaites : hors critère mais mesurées). **Ne pas** conclure sur des hypothèses non
   mesurées : chaque cause doit pointer une graine et un champ.
6. **Hors périmètre** : corriger les bots (b1), changer le contenu, toucher `docs/taches.md` ou
   `dettes.md` (l'orchestrateur reporte), relancer 200 graines (l'orchestrateur le fait).

## Critères d'acceptation

1. Toutes les graines en échec classées, aucune « inconnue » sans un mot d'explication.
2. Digest écrit avec les trois correctifs ; scénarios figés verts localement (`make test_scenarios
   SCENARIO=throne_defaite_*` après feu vert) ; rapport `docs/taches/rapports/m1-analyse-200-throne.md`
   (README §7, point d'état).
3. Rien de béni ; aucun `.rs` ni RON de contenu modifié hors les nouveaux scénarios.

## Livrer

Merger `origin/main` juste avant de livrer. `git push -u origin m1-analyse-200-throne`, puis
`SendMessage` à `orch` : `LIVRÉ m1-analyse-200-throne <sha> : <causes principales en une ligne>`.
