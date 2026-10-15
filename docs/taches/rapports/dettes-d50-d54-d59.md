# Rapport — dettes-d50-d54-d59 (décisions de William du 2026-10-14)

Branche `dettes-d50-d54-d59`. Un commit par dette. **D59 : voir plus bas (à faire en dernier).**

## D54 — le tir ami `Always` exclut le tireur (adb87813, poussé)

- `bullet_rollback_collision_system` (`crates/combat/src/weapons/mod.rs`), la collision des projectiles
  composables et `nearest_target` (`crates/combat/src/projectile.rs`) ignorent `bullet.source` : le tir ami
  (`Always`, `Cursed`) ne touche jamais son propre tireur. `docs/conventions.md` §8 : paragraphe « D54 ».
- `dash_stun_buffer` redessiné (3 joueurs) : P1 et P2 portent `status_stun`, P0 `proj_pierce` ; P0 touche P1 à f42
  (99 → 89), plus d'auto-étourdissement. Trace réenregistrée avec preuve §5.

## D50 — chance de butin de munitions de `throne`

**Décision** : relever `drop_chance` de `games/throne/assets/items/powerups.ron` de 0,25 à **0,40**
(deux ennemis sur cinq laissent un objet ; poids total 146 inchangé). Pas de pondération par les réserves
(décision de William). `docs/conventions.md` §29 à jour.

**Mesure** (`alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run --until-floor 3
--max-frames 15000`) : sur **10 graines (61..70) plus la graine 76**, et non 20 — réduit à la demande de
l'orchestrateur à cause de la charge de la machine.

| | fin | morts | kills | frames |
|---|---|---|---|---|
| avant (0,25) | 11/11 « fin objectif » | 0/2 | 0 | 2292 à 3321 (graine 76 : 2461) |
| après (0,40) | 11/11 « fin objectif » | 0/2 | 0 | 2477 à 3602 (graine 76 : 2477) |

**Limite, dite franchement** : la mesure est **peu discriminante**. Avec `--until-floor 3`, les bots descendent
sans combattre (0 kill) : la partie s'achève avant la pénurie d'origine (graine 76, à sec dès f11280 face à la
tourelle et au boss). Elle ne reproduit donc pas le soft-lock ; elle montre seulement que le changement ne
dégrade pas la descente. La pénurie d'origine n'est pas re-mesurée de bout en bout.

**Traces** : sur le socle main 8f3fd270, sept traces `throne` bougent à 0,40 : `throne_floor_1`,
`throne_mutation_choice`, `throne_progression`, `throne_quad`, `throne_softlock_recul`, `throne_solo`,
`throne_three_floors` (`throne_duo_defaite` et `throne_ammo_pickup` restent identiques). Preuve §5
(`ALACOD_DUMP_TRACE` à 0,40 puis à 0,25 sur le même code, `scripts/trace-diff.py`) : la première frame
divergente est un `powerup_munitions_balles` de plus (`GgrsNetId`, compteur +1) à la mort d'un ennemi
(f303 `throne_floor_1`/`progression`/`solo`, f215 `quad`, f416 `softlock_recul`, f532 `mutation_choice`) ; les
traces sont rebénies, cause au message du commit.

**Ré-étalonnage des attentes** : la cause des changements est la même partout — un drop de munitions de plus à la
mort d'un ennemi (preuve §5, première divergence de chaque trace ci-dessus) change ensuite la partie (bots,
tirage des mutations). Cinq scénarios `throne` ont vu leurs *attentes* échouer, pas seulement leur trace ; elles
sont recalées sur les valeurs mesurées, **sans en retirer aucune** (accord de l'orchestrateur) :

| scénario | attente | avant | après |
|---|---|---|---|
| `throne_solo` (bot) | étage 1 / 2 | f2800 / f2815 | f3475 / f3490 (étage 2 à f3483, f2804 avant) |
| | `Mutations` | [chasseur, vampire] (2) | [irradie, sang_froid, rancune] (3) à f3490 |
| | `Clock alerte` fired | f3715 | f4400 ; `frames` 3800 → 4410 pour qu'elle sonne |
| | `PlayerAlive` | f3799 | f4409 |
| `throne_progression` (bot) | étage 1 / 2 | f2800 / f2815 | f3475 / f3490 |
| | `Clock alerte` non / oui | f3700 / f3715 | f4380 / f4400 ; `frames` 3720 → 4410 |
| | `PlayerAlive` | f3719 | f4409 |
| `throne_quad` | `FloorIndex` 2 / 3, `Event floor niveau 3` | f3155 / f3170 / f3170 | f3105 / f3120 / f3120 (étage 3 à f3110, f3161 avant) |
| `throne_mutation_choice` | `Mutations` et `Event mutation` à f786 | `vampire` | `tireur` (le `ChoiceB` applique toujours un choix) |
| `throne_three_floors` | `Mutations` à f4430 | [adrenaline, coriace] | [adrenaline, rancune] |

Aucune attente n'a perdu son sens. `throne_floor_1` et `throne_softlock_recul` : trace seulement.

## Vérifications

- D50, sur la fusion avec main 8f3fd270 : `make test_scenarios_par SHARDS=4` 297 scénarios verts (957 s), `cargo fmt --all -- --check` et `make lint` verts ; `drop_chance` = "0.40".
