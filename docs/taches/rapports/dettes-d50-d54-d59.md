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

**Refait sur la base D34** (main e355be53 : D34 a rebéni les traces `throne` à cause d'un écart de ~1e-4 sur la vitesse
des balles) : la preuve §5 est rejouée sur ce socle (dump à 0,40 contre dump à 0,25, `scripts/trace-diff.py`), mêmes
causes : un `powerup_munitions_balles` de plus (`munitions_obus` pour `throne_three_floors`) à la mort d'un ennemi, aux
mêmes frames (f303, f532, f215, f416, f3239) ; les attentes recalées ci-dessus passent telles quelles sur la nouvelle
base (seules les traces ont été rebénies de nouveau).

Aucune attente n'a perdu son sens. `throne_floor_1` et `throne_softlock_recul` : trace seulement.

## D59 — la grammaire devient l'étage par défaut de `gungeon`

- **Champ explicite du manifeste** : `entry.floors` (`content::manifest::EntryPoint`, optionnel) désigne la
  séquence du dossier `Floors` que `mode: Floors` joue par défaut ; absent, la première par ordre d'id (comportement
  d'avant, gardé pour les jeux à une séquence). `game::run_state::resolve_run_mode_with_floors` le lit (l'override d'un
  scénario gagne) ; test unitaire. Lint : id inconnu → `BrokenReference`, champ sans `mode: Floors` → `Unsupported`
  (deux fixtures, test dans `lint_fixtures.rs`). Commentaire de `content::reach` mis à jour (toute séquence reste une
  racine d'atteignabilité).
- **`gungeon`** : `floors: "etages_deux"` dans `game.ron` (`floor:etage` ×2 : étage assemblé, portail, second étage).
  Les scénarios sans `floors:` ne sont pas en mode `Floors` (ils chargent une carte) ; le bot explorateur (M2-T16) impose
  sa séquence : aucun scénario de T16 touché, aucune trace n'a bougé (suite verte, voir plus bas).
- **p2p 2 clients sur étage assemblé et transition** : il fallait un moyen de vider un niveau sans clavier. Diagnostic
  `ALACOD_CLEAR_ENEMIES_AT=<frame>` (`map_ldtk::game::floors::debug_clear_enemies_system`, `despawn_rollback`, sans
  effet sans la variable, comme `ALACOD_PRESS_BLANK_AT`). Mesure : `gungeon`, `--matchbox`, 2 clients, `ALACOD_INPUT=neutral`
  `ALACOD_CLEAR_ENEMIES_AT=200`, `ALACOD_EXIT_AT_FRAME=900` : « mode Floors : séquence « etages_deux », 2 niveau(x) :
  [floor:etage, floor:etage] » ; à f200 `floor_portal_open floor=0` puis `floor_transition from=0 to=1 slot=1` sur
  **les deux clients** ; `cmp` des traces : **identiques, 899 frames**. Couvre `floor://`, la graine dérivée par emplacement
  et la transition (D59).
- **Effet de bord corrigé** : `entry.floors` change « la partie du manifeste » de `gungeon` (avant : la première
  séquence par ordre d'id, `etage_1`). Six scénarios qui la jouaient sans `floors:` démarraient sur `etages_deux`
  (201 entités à f0, salles et positions différentes) : `dash_charges`, `gungeon_boss`, `gungeon_boss_bot`,
  `gungeon_clear`, `gungeon_start` prennent `floors: Some("etage_1")` ; pour les scénarios de scène générés
  (`alacod-gen`, `SceneContext::start_floors`), le générateur pose la séquence dont l'unique niveau est
  `entry.start_map` quand le manifeste désigne `entry.floors` (`boss_gatling` régénéré). Leurs traces n'ont pas bougé
  (vérifiées une à une, `make gen GAME=gungeon` ne touche que `boss_gatling.ron`). Aucun scénario de M2-T16 touché.
- **Limite** : la mort réelle du boss n'est pas jouée en p2p (les ennemis sont retirés par le diagnostic) ; la victoire
  de fin de séquence n'est pas atteinte dans ces 900 frames.

## Vérifications

Sur la fusion avec main e355be53 (perf-navigation, dettes-outillage D34, M2-T14c), `drop_chance` = "0.40" :
- `make test_scenarios_par SHARDS=4` : **310 scénarios verts** (1060 s) après bless des 7 traces `throne` (preuve §5 ci-dessus) ;
- `cargo fmt --all -- --check`, `make lint`, `make gen GAME=gungeon` verts ; `make test_crates` vert (**712 tests**) ;
- p2p `gungeon` 2 clients (D59) : séquence `etages_deux`, transition d'étage 0 → 1 à f200 sur les deux clients, traces identiques sur 899 frames.

**Deux tests de `crates/scenario/tests/expectations.rs` dépendaient aussi de throne** (trouvés par `make test_crates`, pas par
la suite de scénarios) : `hud_text_sur_throne_progression` (attente `floor` « Étage 3 » de f2815 à f3490, aucune retirée) et
`throne_recorded_pillard_leaves_the_wall_corner` (le pillard enregistré passe de l'id 220 à 221 : un drop de plus avant son
apparition ; reconnu à sa position dans le coin des murs, constante `PILLARD_NET_ID`).

**Leçons** : (1) changer le défaut d'un manifeste (D59) touche tous les scénarios qui jouent « la partie du manifeste » ;
(2) relever une chance de butin décale les ids, les tirages de mutations et les frames d'étage : lancer `make test_crates`
avant de bénir, la suite de scénarios ne couvre pas `crates/scenario/tests/expectations.rs` ; (3) `/tmp` (tmpfs 12 Go) ne
tient pas les dumps `.full` (200 Mo chacun) : les écrire sur le disque.
