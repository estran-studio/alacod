# polissage-nav-fenetres (b4, D176) — guidage des zombies autour des fenêtres

Branche `polissage-nav-fenetres` depuis `origin/polissage-integration` `a2ade254`. Spec : `runs/alacod-polissage/b4-d176-nav-fenetres.md`.
Convention : `docs/conventions.md` § 103. Dette : D176 fermée (`docs/taches/dettes.md`).

**Verdict** : les zombies ne gèlent plus devant le point visé de leur case suivante ; `avant_poste` (carte de b2) graines 2, 3, 45, 76 ne
bloquent plus (vague 2–3 figée avant, vague 9 et kills continus jusqu'à f20 000 après). 9 traces de zombies bougent, chacune exactement à la
première frame où un zombie gelait (tableau § 3) ; aucune trace d'un autre jeu. Le jeu devient un peu plus dur (§ 4) : voulu (coordinateur),
S1-10 rééquilibrera.

## 1. Cause (reproduction de D176)

Commande (Mac, binaire `alacod-sim` **statique**, `ALACOD_DYLIB=0`, carte et `weapons.ron` de b2 `c84ef0f1` copiés le temps de la mesure,
jamais commités) :

```
ALACOD_MUTE=1 ALACOD_CHECK_DISTANCE=0 alacod-sim --game zombies --bots 1 --profiles acheteur --seeds 2..2 --until-wave 10 \
  --max-frames 20000 --map maps/avant_poste.ldtk --save-scenario … --json …
```

Graine 2 : zombie 385 à `(143,99646 ; 31,99878)`, case (8, 1), case suivante (9, 2) en diagonale, `steering_point` `(144, 32)`. Ce point est le
**coin** de la case (9, 2) : le centre (152, 40) est poussé d'une demi-case en x et en y par le coin de mur (10, 3) (bord droit de l'ouverture de
la fenêtre 54, `x128..160` sur `y48..64`). Le zombie s'arrête à 0,004 px du point sans entrer dans la case : `normalize_or_zero` rend **zéro**
sous 1/16 d'unité (`len² ≤ 2⁻⁸`), mais `flow_direction` ne testait que `length_squared() > 0` et rendait cette direction nulle (`Some(ZERO)`)
au lieu de viser la case d'après. Vitesse nulle pour toujours (la récupération de `move_enemies` ne s'en mêle pas : le point est dans sa case
et libre). Graines 3, 45, 76 : même motif (zombies 288, 368, 223, chacun à < 0,05 px d'un point visé au coin d'une case diagonale).

Cas de la fenêtre intacte (contrefactuel reverté de b2, ouvertures élargies à quatre cases) : le champ `GroundBreaker` passe par la case libre
à côté de la fenêtre ((10, 3), point visé (160, 56)) ; le corps (20 × 20, décalage y −6) y chevauche la fenêtre de 10 px, mais aucune case du
chemin n'est sous son collider : `enemy_target_selection` ne la ciblait pas, le zombie poussait contre son bord. Aucune carte livrée (ni celle
de b2) n'a d'ouverture plus large que sa fenêtre (relevé des 60 fenêtres des cartes `games/*/assets`) : cas seulement contrefactuel.

## 2. Correctif (`crates/game/src/character/enemy/ai/`)

| Point de la spec | Issue | Où |
|---|---|---|
| `steering_point` dégénéré ≠ vitesse nulle | fait | `navigation.rs` `flow_direction` : direction nulle après `normalize_or_zero` = point atteint, vise la case d'après (comme le point exact). Un écart plus grand garde la direction d'avant au bit près. |
| Fenêtre intacte sur le passage frappée | fait | `behavior.rs` `window_on_the_way` : règle des cases d'abord (inchangée, même ordre `GgrsNetId`), puis le corps posé au point visé de la case suivante qui chevauche une fenêtre à portée. `Option<&Collider>` ajouté à la requête. |
| Aucune mécanique nouvelle, aucun réglage | fait | aucun fichier de contenu touché. |
| Déterminisme | fait | `Fixed` seulement, `breakables` en `order_iter!`, premier `GgrsNetId` gagne ; aucun état nouveau. |

## 3. Preuves

**Tests avant/après.**

| Test | Avant | Après |
|---|---|---|
| scénario `avant_poste_fenetre_coin` (neuf, zombie posé au point figé de la graine 2, fenêtre 17 de `Depart`) | échec : `EnemyDistance … distance 171.2063 > max 60` (f300), `hit` « joueur 0 touché » absent ; zombie à `(527,99646 ; 559,9988)` de f5 à f590 (`map_probe`) | vert : fenêtre cassée avant f200, zombie à 30 px du joueur à f300, joueur touché |
| `behavior::tests::fenetre_au_bord_du_point_vise_est_a_casser` (géométrie du contrefactuel) | échec (`left: None, right: Some(54)`, règle des cases seule) | vert |

**Traces (BLESS, zombies seulement).** Diagnostic jetable (retiré) : une ligne par zombie et par frame où l'ancien `flow_direction` rendait
`Some(ZERO)` avec un écart non nul, sur le code d'avant. Chaque trace diverge exactement à la première de ces frames.

| Trace | 1re frame qui bouge | Cause (code d'avant) |
|---|---|---|
| `bench_horde` | f646 | zombie 196 à (287,988 ; 688,025), case suivante (18, 43), écart (0,012 ; −0,025) : figé f646–658 (zombies 136, 216 : f650–657) |
| `clone_solo` | f1169 | zombie 108 à (528,024 ; 559,990), case (32, 35) : figé f1169–1173 (coin de la fenêtre 17 de `test_map`, même point que le scénario neuf) |
| `downed_all_lose` | f959 | zombie 98 à (287,999 ; 688,000), case (18, 42) : figé f959–1141 |
| `downed_bleedout` | f1079 | zombie 113 à (287,998 ; 688,003), case (18, 43) : figé f1079–1099 |
| `idle` | f959 | zombie 93 à (287,997 ; 688,004), case (18, 43) : figé f959–1184 (zombies 95 et 91 : 187 et 134 frames) |
| `la_fonderie_demo` | f319 (graine 4) | zombie 296 à (128,005 ; −159,998), case (7, −10) : figé f319–421 ; la graine passe ensuite de 4 à 1 (ci-dessous), trace réécrite dès f0 |
| `movement_melee` | f334 | zombie 89 à (463,977 ; 559,990), case (29, 35) : figé f334–899, **jusqu'à la fin** |
| `run_lose_summary` | f959 | zombie 98, comme `downed_all_lose` (même partie) : figé f959–1141 |
| `two_players_idle` | f959 | zombie 98, même partie : figé f959–1141 |

`git diff --stat origin/polissage-integration -- '*.trace'` : 9 traces ci-dessus + `avant_poste_fenetre_coin.trace` (neuve), toutes de
`game: "zombies"` ; **aucune trace d'un autre jeu**. Le critère « fenêtre au bord du point visé » seul (navigation d'avant) laisse les 9 traces
identiques (suite jouée sur les 9 : `test result: ok`) : seul le premier point les bouge.

Attentes recalées (décision du coordinateur, option A) :
- `two_players_idle` : joueur 1 à terre à f1072 (avant f1296), joueur 0 mort à f1346 (avant f1570) : `PlayerDowned`/`PlayerAlive` à f1300
  (avant f1400).
- `la_fonderie_demo` : la graine 4 ne gagne plus en 20 000 frames (vague 9, 89 kills ; les zombies ne gèlent plus). Graine 1 : victoire à
  f7131 (vague 5) ; `frames` 7600, `RunState`/`RunSummary` à f7600, `KillsAtLeast(25)` à f5750 (24 kills à f5700) ; commentaire
  « À regarder » réécrit.

**Ordre du rollback** : `make schedule_order_diff BASE=origin/polissage-integration` → `OK : ordre des systèmes et des types rollback
identiques (4550 lignes de schedules, 645 types)`.

**Déterminisme** : toute la suite de scénarios en synctest (distance 2) ; `fuzz_inputs` graines 0:8 et `ALACOD_FUZZ_SPARSE=1` 0:8 : `test
result: ok` (aucune frame rejouée différemment) ; `alacod-sim` la_fonderie graine 4 à distance 2 : même issue qu'à distance 0 (victoire
f12187), sans desync.

## 4. Campagne courte

Binaire statique (`otool -L` : aucune `libbevy_dylib`), avant = `a2ade254`, après = ce correctif. Tableau complet :
`docs/taches/rapports/polissage-nav-fenetres/campagnes.tsv`.

| `avant_poste` (carte b2), 1 `acheteur`, `--until-wave 10 --max-frames 20000` | avant | après |
|---|---|---|
| graine 2 | vague 3, 18 kills, dernier kill f3989, zombie 385 figé | vague 9, 100 kills, dernier kill f19830 |
| graine 3 | vague 2, 10 kills, dernier kill f2280, zombie 288 figé | vague 9, 97 kills, dernier kill f19981 |
| graine 45 | vague 3, 17 kills, dernier kill f4159, zombie 368 figé | vague 9, 87 kills, dernier kill f19362 (attente entre vagues) |
| graine 76 | vague 2, 11 kills, dernier kill f2322, zombie 223 figé | vague 9, 107 kills, vague terminée à f19671 |

Les quatre restent « plafond » après, mais en jouant : la vague 10 n'est pas atteinte en 20 000 frames (rythme du bot), aucun zombie n'est figé
(positions et `steering_point` du JSON distincts à f19400 et f20000).

| `la_fonderie` (carte du jeu), 1 `acheteur`, graines 1–10 | victoires | morts | plafonds |
|---|---|---|---|
| avant | 9 | 0 | 1 (graine 8) |
| après | 7 | 3 (graines 2, 7, 8) | 0 |

Les zombies qui gelaient (jusqu'à plusieurs secondes) chassent maintenant : partie plus dure, voulu ; les chiffres de la campagne S1-10 de b2
(faits sur le code d'avant) bougeront après ce merge.

## 5. `make check`

Toutes les cibles de `make check` jouées sur le Mac (`CARGO_TARGET_DIR=.target-mac`, `ALACOD_MUTE=1`), sauf `test_tools` (échec connu du Mac :
3 tests `test_claude_say`, sans rapport) :

```
make format lint                    → exit 0 ; « games/zombies : aucune erreur … games/isaac : aucune erreur »
make test_crates check_forbidden check_rollback_registration check_sim_boundary → exit 0, 94 « test result: ok »
OK: aucun nouveau motif interdit ; 52 occurrences auditées dans determinism-allowlist.json
OK: aucun appel direct à rollback_component_with*/rollback_resource_with* hors de crates/utils/src/rollback.rs
OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69
make test_scenarios_par             → exit 0 ; tranches 0–3/4 : ok ; 813 scénarios ; suite en 4 tranches : 447 s
```

## 6. Reste

- Ciblage de fenêtre toujours soumis à un joueur perçu (`aggro_range` 500, D66) : un zombie contre une fenêtre intacte, joueur à plus de
  500 px hors de vue, attend que le joueur s'approche (pas de blocage permanent dans les campagnes ; hors du périmètre « aucune mécanique
  nouvelle »).
- Conflit de merge attendu avec `polissage-zombies-map-equilibrage` (b2) : ligne D176 de `docs/taches/dettes.md` (b2 l'ajoute ouverte, ici
  fermée) et traces de zombies (b2 les réécrit pour ses cartes : à rejouer après ce merge).
