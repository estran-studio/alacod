# Rapport — m0-v7 : les bots finissent le clone

**SHA de la tête avant commit du rapport : `e1aaf18fddc5339ed0c860aed11c0ac7ac829d03`.**
Fiche : [m0-v7-bots-finisent-le-clone](../m0-v7-bots-finisent-le-clone.md).
Branche : `m0-v7-bots-finisent-le-clone` ; agent : Codex ; date : 2026-10-02.

Livraison de **phase 1 uniquement**.
Les huit rejeux longs représentent 160 000 frames en 2 722,30 s (45 min 22,3 s),
0 failure, 0 desync. Le code du diagnostic initial est `3b5c6bb` (graines
4/11/12/13/16), enrichi par `e63c72b` (17/18/19). Les outils de sondage sont
complétés par `05ec58a`. `main` à `05fd7f9` est intégré par `554e99f`,
sans conflit ; les vérifications et sondes suivantes se font sur cet état fusionné.
La mise à jour documentaire de `main` à `b0e3f99` est ensuite intégrée par
`e1aaf18` : seul `docs/taches.md` diffère de la tête de code testée `554e99f`.
Le commit de livraison qui suit ne contient que le rapport, les dumps et D20.

## Fait

Dump permanent `softlock` du JSON `alacod-sim` quand le plafond est atteint sans
objectif ni mort de tous les joueurs. Le runner prend deux instantanés hors
simulation : fin et 600 frames auparavant. Ils contiennent la phase et les
compteurs de vague, les ennemis (positions, santé, cible, état, coût et prochaine
case du flow field), les joueurs encore présents (à terre signalé, munitions et
solde), les portes fermées (coût, interaction), les fenêtres et la grille ASCII
navigation/physique. Les observations restent factuelles : aucune cause arbitraire
n'est attribuée à une porte. Aucun système ni état rollback ajouté.

Profils v0 et fichiers `.trace` inchangés. Aucun bless.

## Diagnostic

Dumps complets : [huit rejeux longs](m0-v7-bots-finisent-le-clone.diagnostic.json)
et [huit sondes à f5000](m0-v7-bots-finisent-le-clone.probes.json).
Les enregistrements RON restent dans `../recordings/seed_n.ron` du worktree.

Commande pour chaque graine `n` dans `4 11 12 13 16 17 18 19` :

```bash
source ../env.sh
export CARGO_BUILD_JOBS=4
cargo run -q -p scenario --profile headless --bin alacod-sim -- \
  --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile \
  --seeds "${n}..${n}" --until-wave 5 --max-frames 20000 \
  --save-scenario ../recordings --json "../diagnostic/seed_${n}.json"
```

Sonde du scénario enregistré, avec les chemins absolus car Cargo lance les tests
depuis `crates/scenario` (premier essai avec chemins relatifs : fichiers introuvables,
relance corrigée ci-dessous). Couples `(n, id)` : `(4, 301)`, `(11, 792)`, `(12, 297)`,
`(13, 670)`, `(16, 320)`, `(17, 403)`, `(18, 247)`, `(19, 493)`.

```bash
ALACOD_PROBE_FILE="$PWD/../recordings/seed_${n}.ron" \
ALACOD_PROBE_JSON="$PWD/../diagnostic/probe_${n}.json" \
ALACOD_PROBE="${id}:5000" \
cargo test -q -p scenario --profile headless --test scenarios nav_probe -- --ignored --nocapture
```

### Cas observés

- Graine 4 : six zombies, cinq vers `(465, -735)` à la fenêtre 66 `(484, -730)`,
  un vers `(726, -479)` à la fenêtre 69 `(692, -474)` ; toutes deux cassées.
  Les six ont un chemin GroundBreaker (coûts 220/880). Entre f19400 et f20000,
  déplacement net maximal 1,28 px ; joueurs 2 et 3 strictement aux mêmes positions.
  Le prudent a un chargeur et une réserve Balle vides, mais 48 Plomb inutilisés.
  L'immobile a encore 30 + 240 Balle et ne tire jamais. Les deux fonceurs sont morts.
  La sonde f5000 montre les six points physiquement bloqués : `(488, -736)` par
  le mur 135 (`x = 476..492`, `y = −826..−746`), et `(712, −479)` par
  le mur 153 (`x = 684..716`, `y = −506..−490`).
- Graine 11 : un zombie vers `(-246, -527)` à la fenêtre 50 `(-265, -524)`,
  cassée, chemin de coût 644. Déplacement net 1,13 px, dernier kill f3236.
  Les fonceurs restent vers `(-491, -236)` et le prudent vers `(-427, -172)`,
  contre les murs en direction du zombie distant. Leurs entrées sauvegardées à
  f19999 sont `Down, Right`, sans réparation ; la fenêtre la plus proche est pleine.
  Quatre bots en vie avec des munitions.
  Les trois chasseurs ont le solde pour la porte 6 à 750, qui reste fermée.
  À f5000, le point `(−264, −528)` chevauche le mur 83, bornes
  `x = −273..−257`, `y = −572..−540`.
- Graine 12 : six zombies vers `(-504, 240)` à la fenêtre 60 `(-486, 247)`,
  cassée, tous avec un chemin de coût 214. Déplacement net maximal 1,32 px,
  dernier kill f698. Prudent sans Balle (48 Plomb inutilisés), immobile encore
  approvisionné, deux fonceurs morts. Positions des survivants inchangées à f19400/f20000.
  À f5000, les six points `(−488, 241)` sont bloqués par le mur 84,
  `x = −494..−478`, `y = 151..231`.

- Graine 13 : un zombie `(304, 288)` à la fenêtre 54 `(285, 282)`, cassée,
  chemin de coût 672. Même position à f19400/f20000, dernier kill f3211.
  Fonceurs `(59, 570)` et prudent `(123, 634)` contre les murs, avec des munitions ;
  quatre joueurs vivants. Les trois chasseurs gardent `Down, Right` jusqu’au
  plafond, sans réparation (entrées sauvegardées). À f5000, le zombie vise
  `(304, 288)` depuis `(304.02643, 287.99298)` : vitesse principale et knockback
  nulles, aucun mur chevauché au point visé. La prochaine case est `(18, 18)`,
  mais le point visé tombe dans `(19, 18)` ; il ne fait pas progresser jusqu’à
  cette prochaine case. Le même zombie reste exactement à cette position à
  f19400/f20000.
- Graine 16 : deux zombies vers `(712–713, 400)` à la fenêtre 46 `(694, 402)`,
  cassée, chemins de coût 608. Déplacements nets 0,54 et 1,08 px, dernier kill f954.
  Les trois combattants ont un chargeur et une réserve Balle vides (48 Plomb chacun
  encore disponibles). Les fonceurs bougent encore (déplacements nets 25–27 px)
  mais n'obtiennent ni kill ni contact avec les zombies ; prudent et immobile sont
  aux mêmes positions. Quatre joueurs vivants. À f5000, les deux points
  `(696, 401)` chevauchent le mur 101 (`x = 686..702`, `y = 354..386`).

- Graine 17 : deux zombies `(−680, 678)` au-dessus du mur 143, bornes
  `x = −704..−560`, `y = 645..661`, qui sépare les fonceurs `(−678, 641)`.
  Tous deux ont un chemin de coût 60 ; le point visé `(−680, 673)` ferait
  chevaucher leur corps avec ce mur (même résultat à f19400 et f20000).
  Déplacement net 0,85 px, dernier kill f1067. Trois combattants sans Balle,
  avec 48 Plomb inutilisés chacun ; quatre joueurs vivants.

- Graine 18 : deux zombies vers `(−323, −496)` à la fenêtre 50 `(−341, −489)`,
  cassée. Chemins de coûts 440/444, mais point visé `(−344, −495)` bloqué par
  le mur 175 : `x = −349..−333`, `y = −537..−505` ; le corps au point visé
  descend à −511, soit 6 px dans le mur. Même blocage aux deux instantanés.
  Déplacements nets 1,13 et 1,77 px, dernier kill f1173. Trois combattants sans
  Balle, 48 Plomb chacun encore disponibles, quatre joueurs vivants.

- Graine 19 : un zombie `(−83, 933)` au-dessus du mur 149, bornes
  `x = −154..−10`, `y = 901..917`, qui sépare les fonceurs vers `(−70..−77, 897)`.
  Chemin de coût 64 ; point visé `(−72, 929)` bloqué par ce mur (corps descendant
  à 913, 4 px dans le mur). Dernier kill f2533. Le zombie et les fonceurs se
  déplacent le long du mur (déplacements nets 21 et 31–43 px), sans résoudre
  le combat. Trois combattants sans Balle, 48 Plomb chacun inutilisés, quatre
  joueurs vivants (joueur 1 à 97,5 de santé).

À f5000, 20 des 21 zombies restants visent un point qui chevaucherait un mur ;
le cas 13 vise un point presque confondu avec sa position, hors de la prochaine case.
Les compteurs observés sont cohérents : phase `InProgress`, aucun ennemi à générer,
les ennemis de vague restants expliquent l'attente. Aucun cas observé de vague qui
resterait bloquée avec zéro zombie. Les fenêtres ont des ouvertures IntGrid dans
les gabarits LDtk. Des points de steering physiquement bloqués sont démontrés pour 17/18/19 ;
les sondes f5000 démontrent aussi ces chevauchements pour 4/11/12/16.
Le mauvais trajet des bots et la mitrailleuse
épuisée sont des faits distincts. Deux instantanés ne prouvent pas une immobilité
à chaque frame intermédiaire : les déplacements ci-dessus sont des déplacements nets.

La construction du [point de steering](../../../crates/game/src/character/enemy/ai/navigation.rs)
additionne les poussées cardinales et diagonales sans vérifier la collision du
point final ni sa présence dans la case suivante. Les dumps montrent deux
signatures de cette limite : chevauchement physique pour sept graines, point
presque confondu avec la position du zombie et situé hors de la case suivante
pour 13. Le repli vers la case d’après ne s’applique que lorsque la direction
calculée est exactement nulle. La sonde 13 constate une vitesse finale nulle ;
le détail de l’évitement à chaque frame n’est pas instrumenté ici.

## Tableau des 20 graines de validation

À cette livraison de phase 1, seules les huit graines bloquées sont rejouées avec
le line-up v0. **Le nouveau line-up n’existe pas encore : les 20 validations de
phase 2 sont toutes non faites.** Les douze autres graines v0 ne sont pas rejouées ici.
Les huit caps ont les mêmes vague/frames/morts/kills/desync que le JSON du digest.
Six caps ont quatre bots vivants (les tableaux du digest donnent six, malgré
le chiffre cinq de sa prose et de la fiche).

| Graine | Vague v0 rejouée ici | Frames | Morts / 4 | Kills | sim fps | Validation line-up final (phase 2) |
|---:|---:|---:|---:|---:|---:|---|
| 1 | Non rejouée | — | — | — | — | Non jouée |
| 2 | Non rejouée | — | — | — | — | Non jouée |
| 3 | Non rejouée | — | — | — | — | Non jouée |
| 4 | 1 | 20000 | 2 | 0 | 55.5 | Non jouée |
| 5 | Non rejouée | — | — | — | — | Non jouée |
| 6 | Non rejouée | — | — | — | — | Non jouée |
| 7 | Non rejouée | — | — | — | — | Non jouée |
| 8 | Non rejouée | — | — | — | — | Non jouée |
| 9 | Non rejouée | — | — | — | — | Non jouée |
| 10 | Non rejouée | — | — | — | — | Non jouée |
| 11 | 2 | 20000 | 0 | 18 | 60.2 | Non jouée |
| 12 | 1 | 20000 | 2 | 1 | 60.1 | Non jouée |
| 13 | 2 | 20000 | 0 | 19 | 60.8 | Non jouée |
| 14 | Non rejouée | — | — | — | — | Non jouée |
| 15 | Non rejouée | — | — | — | — | Non jouée |
| 16 | 1 | 20000 | 0 | 5 | 56.4 | Non jouée |
| 17 | 1 | 20000 | 0 | 6 | 61.7 | Non jouée |
| 18 | 1 | 20000 | 0 | 6 | 62.4 | Non jouée |
| 19 | 2 | 20000 | 0 | 16 | 54.6 | Non jouée |
| 20 | Non rejouée | — | — | — | — | Non jouée |

## Vérifié

Toutes les commandes de compilation ci-dessous sont précédées de `source ../env.sh`
et de `export CARGO_BUILD_JOBS=4`, dans le worktree de la tâche.

- `make test_scenarios` : 61 scénarios verts, traces comparées, 2 tests réussis,
  0 échec et 7 ignorés ; 416,44 s. `clone_quad` finit à f4372 en vague 5,
  13 kills et quatre survivants, avec la trace v0 inchangée.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content
  -p map_ldtk -p sim_core -p stats -p bots -p effects -p behaviors --no-fail-fast` :
  290 réussis, 0 échec, 8 ignorés (sommes des 34 blocs de résultats).
- `make lint` : `zombies` et `testbed` sans erreur.
- `cargo fmt --all -- --check` : sortie vide, code 0.
- `./scripts/check-forbidden.sh` : code 0, quatre occurrences préexistantes.
- `./scripts/check-rollback-registration.sh` : OK, aucun appel direct interdit.
- `make gen GAME=zombies` : code 0, dix scénarios d’armes comparés à leurs traces,
  aucun fichier généré modifié.
- `ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py` :
  deux codes 0, 61 scénarios verts, 2 tests réussis / 0 échec / 7 ignorés,
  318,99 s ; `bench_bullets` 102,3 fps ≥ 70, `bench_horde` 65,6 fps ≥ 38.
  Aucun autre `cargo`, `rustc`, rejeu ou `ffmpeg` détecté au départ ; les contrôles
  de processus pendant le bench ne montrent que ce rejeu comme charge lourde.
- `cargo test -q --profile headless -p scenario --test softlock` : 2 réussis,
  0 échec, 0 ignoré. Plafond artificiel à f601, état précédent à f1, objectif atteint
  exactement au plafond à f1 sans dump, sérialisation JSON, trace exactement égale
  au même run sans collecte (deux exécutions de 601 frames).
- Huit sondes explicites : 1 test réussi et 0 échec par graine, 40 000 frames
  en 841,89 s. Huit dumps JSON produits, tous exactement à f5000 ; 21 zombies
  restants, dont 20 avec un point visé physiquement bloqué.
- Huit graines rejouées : métriques exactement égales au digest
  pour vague/frames/morts/kills/desync, 0 failure, 0 desync.

## Non fait / non vérifié

- Phase 2 : pathfinding, nouveaux profils, achats et validation 20 graines.
- Critère 200 graines réservé à l'orchestrateur après merge.
- Aucune revue visuelle ni test p2p : aucun changement de simulation ou de session
  dans cette phase (diagnostic hors rollback).
- Aucun merge dans `main`, aucune modification manuelle de `docs/taches.md`.

## Dettes / questions ouvertes

D20 reste ouverte. Ne pas marquer « fait dans m0-v7 » avant la phase 2.
Le passage physique, la progression du steering, les chemins des bots et la
gestion de leur armement devront être traités en conservant les profils v0.
Attente du retour de l’orchestrateur avant de commencer la phase 2.
