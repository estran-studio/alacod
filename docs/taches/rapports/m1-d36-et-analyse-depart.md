# m1-d36-et-analyse-depart — D36 (lint caverne sans ennemi, `DestroyTerrain` en `on_hit`) et analyse « acheteurs dans Depart »

## État en cours

- Fait : lint D36 + fixture, décision `on_hit` documentée (§21), instrumentation `alacod-sim`
  (portes, état des joueurs à l'arrêt), analyse sur 20 graines à 4 acheteurs + 5 graines du
  mélange. Branche mergée avec origin/main `a2486fe`. Rien de béni (aucune trace ne change :
  champ `transit` sérialisé seulement s'il est vrai, `explode_wall` vert).
- **Hypothèse réfutée** pour le critère M0 (4 acheteurs) : les acheteurs ouvrent des portes et
  quittent Depart. Elle ne vaut que pour le mélange sans acheteur. Note ajoutée au digest m0.

## D36 — lint « caverne sans ennemi dans une séquence `Floors` »

- `CaveConfig.transit: bool` (`crates/world/src/cave.rs`, `#[serde(default)]`, omis si faux) :
  caverne voulue sans ennemi (son portail s'ouvre aussitôt).
- `lint_floors` (`crates/content/src/lint.rs`) : un étage `cave:<id>` dont la caverne n'a pas de
  `characters` ou a `enemy_spawns: 0`, sans `transit: true`, est une erreur (`OutOfRange`) qui
  nomme le fichier et la marche à suivre.
- `games/testbed/assets/caves/petite.ron` : `transit: true` (caverne de démonstration sans ennemi).
- Fixture `crates/content/tests/fixtures/floors_cave_without_enemies` (cavernes `vide`,
  `sans_points`, `transit`, `peuplee`) et test `floors_cave_without_enemies_fixture` : exactement
  deux erreurs, `cave:vide` et `cave:sans_points`.
- `docs/conventions.md` : ligne du tableau des lints, paragraphe D36 au Lint de §21.

## D36 — `DestroyTerrain` en `on_hit` d'un personnage (§21)

Décision documentée dans `docs/conventions.md` §21 : `on_hit: [DestroyTerrain]` ne creuse que
sur un mur (`ProjectileWallHit`) ; sur un personnage, `apply_projectile_on_hit_system` n'applique
que `ApplyStatus`. Pour creuser à tout impact, mettre `DestroyTerrain` en `on_expire` (le
projectile se termine au coup sans percée restante, `on_expire` part alors). Pas de changement de code.

## Analyse « les acheteurs restent dans Depart » (avant_poste)

`alacod-sim` relève désormais `doors_opened` (événements `door` : une entité porte par côté de
passage, d'où des comptes pairs) et, à l'arrêt (vague 5), `players_end` : solde, à terre,
position, `in_spawn_room` (salle `spawn` du `RoomBounds` qui contient le joueur).

Commande : `alacod-sim --game zombies --bots 4 --seeds 1..20 --until-wave 5 --max-frames 20000`
(quatre `acheteur`, profil du critère M0), binaire sur `748e374` (D45) + instrumentation.

| Graine | Vague | Frames | Événements porte | Hors Depart à la vague 5 | Soldes |
|---:|---:|---:|---:|---:|---|
| 1 | 5 | 6522 | 6 | 0/4 | 1430, 1500, 1490, 1280 |
| 2 | 5 | 6620 | 10 | 3/4 | 660, 1680, 2480, 850 |
| 3 | 5 | 7583 | 12 | 1/4 | 990, 260, 830, 830 |
| 4 | 5 | 6647 | 10 | 4/4 | 1960, 1290, 1890, 1080 |
| 5 | 5 | 6701 | 8 | 3/4 | 170, 360, 1240, 1810 |

Sur les 20 graines : portes ouvertes à **toutes** (6 à 12 événements), **49/80** joueurs hors de
Depart à la vague 5 (tous dans Depart seulement aux graines 1 et 9, après avoir ouvert des
portes), soldes de 60 à 4300 (médiane 1100), 0 mort, 0 bloquée (relevé D42 : aucun softlock).
Résultats : `../d40/sim_acheteurs_apres.{log,json}`.

**Mélange `fonceur,fonceur,prudent,immobile`** (graines 1..5, `../d40/sim_melange_apres.json`) :
**0 porte**, les survivants (toujours le `prudent`, parfois l'`immobile`) sont dans Depart ;
2 à 4 morts, vague 5 à 4 graines sur 5 (graine 5 : 4 morts à la vague 4). Seul `acheteur` ouvre
des portes (`crates/bots/src/hunter.rs`).

Conclusion : l'hypothèse est **fausse pour le critère M0**, qui joue avec 4 acheteurs : la carte
est parcourue. Elle est vraie pour le mélange de `bots_four_mixed`, qui ne mesure donc que Depart.
La limite réelle est ailleurs (rapport m1-assembleur-d45-d47) : avant_poste ne donne que
**5 cartes** sur les graines 1..20, et l'assembleur n'en change aucune ; la graine fait surtout
varier le nombre de salles (4 à 9). Proposition (non faite ici) : gabarits alternatifs par
connexion dans `avant_poste.ldtk` pour que la graine change la carte ; et un acheteur dans le
mélange si on veut qu'il mesure autre chose que Depart.
