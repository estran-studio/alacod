# M3-T12 : lint des kinds M3 et atteignabilité des mécaniques

Branche `m3-t12-lint-atteignabilite` (depuis `origin/main` 1600b43b, fusionnée avec `origin/main` b18458cf = M3-T9 avant la
livraison). Conventions : `docs/conventions.md` §77. Session b6 (OMP) ; une relecture indépendante (agent `reviewer`) en deux
passes. **Lint seulement** : crate `content`, la déclaration `mechanics` de `games/isaac/assets/game.ron`, et une ligne dans
un test de `crates/game/src/run_state.rs` (le littéral `GameManifest` gagne un champ). Aucune trace ne bouge.

## 1. Fait

| Volet | Où | Contenu |
|---|---|---|
| Audit des règles M3 | `docs/conventions.md` §77 | une ligne par mécanique (munitions infinies, larmes, stats, bombes, rochers, piques, portes, grille, familiers, champions, curses, pools à épuisement, accomplissements, `Price`, `ExtraHealth`, `hit_iframes`, plafonds, `Scaled`) : règle, niveau, fixture. Les T0a à T7 avaient presque tout ; **trois trous** comblés (ci-dessous) |
| Déclaration `mechanics` | `content::manifest` (`Mechanic`, `GameManifest::mechanics`), `games/isaac` | douze variantes, une par ligne du tableau « Atteignabilité » de `docs/taches.md` §8.1 ; défaut vide : aucun jeu existant ne change |
| Atteignabilité | `crates/content/src/mechanics.rs` (nouveau), `reach.rs` | kind `MechanicUnmet` ; 9 règles toujours actives, 12 règles déclarées, sur la **partie ordinaire** (séquence `entry.floors`) |
| Sources | `reach.rs` | `pool_items`, `loot_pool_items`, `loot_pool_weapons` (déjà branché depuis D62 : le constat de la fiche était périmé), `unlock_grant_sources` (les entrées qu'un déblocage ajoute à un pool comptent), `item_sources` partagé |
| Stats | `reach::stat_uses`, test | `every_named_stat_has_a_reader` ; `stat_uses` lit aussi les actions d'entrée et de frise des phases de boss (curses : T9 l'avait fait en parallèle, fusionné) ; `ShotSpeed`, `ShotSize`, `Luck` avaient déjà un lecteur |
| Messages M2 | `reach::warnings` | « type de salle … ; mécanique : salle secrète (murée…) », « objet … aucune source … ; mécanique : familiers, effets » |
| `MapLevelInfo` | `registry.rs` | `rocks`, `loot_rocks`, `surface_values`, `rock_other_values` (lus dans les cartes LDtk) |
| `alacod lint --json` | inchangé | rend `"kind": "MechanicUnmet"` (test `lint_json_reports_mechanic_warnings`) ; `alacod describe` : aucun changement (le manifeste n'est pas un kind de contenu) |

### Règles ajoutées → fixture

Fixture de base **`mechanics_ok`** (`crates/content/tests/fixtures/mechanics_ok/`) : un jeu en grille (départ, combat avec un rocher, un
rocher à butin, une pique, boss, trésor à clé, secrète, boutique, diable) qui porte les douze mécaniques ; `base_is_clean` prouve
zéro erreur et zéro avertissement. Chaque test de `crates/content/tests/m3_t12_fixtures.rs` lui retire ou change un élément
dans un dossier temporaire (une règle = une mutation = un avertissement attendu).

| Règle | Niveau | Test (mutation de `mechanics_ok`) |
|---|---|---|
| salle secrète `required` d'une grille sans `door_lock: Secret` | erreur `OutOfRange` | `secret_room_not_required_or_not_walled` |
| pool `exhaust` qui n'a que des `Currency` | erreur `OutOfRange` | `exhaust_pool_of_coins_only_is_an_error` |
| couche `Rocks` : valeur autre que 1/2 ; couche `Surfaces` : valeur sans surface | erreur `OutOfRange` / `BrokenReference` | `rocks_and_surfaces_layers_are_checked` |
| stat sans lecteur dans une curse / une phase de boss | erreur `UnreadStat` | `unread_stats_are_found_in_curses_and_boss_phases` |
| pool jamais tiré (coffre, boutique `floor_tables`, récompense, rocher à butin, champion) | avertissement | `pool_that_nothing_draws`, `pool_of_a_default_chest_is_drawn`, `shop_table_is_not_drawn_when_floor_tables_exist`, `reward_pool_of_the_floor_replaces_the_default`, `reward_chest_draws_the_default_chest_pool` |
| familier sans objet atteignable | avertissement | `familiar_whose_items_have_no_source`, `test_only_familiar_is_not_warned` |
| synergie à objet sans source | avertissement | `synergy_with_an_item_that_has_no_source` |
| salle spéciale vide (secrète, post-boss, exigée) | avertissement | `special_room_that_carries_nothing` |
| mur secret sans bombe | avertissement | `secret_wall_without_a_bomb`, `test_only_characters_give_no_bomb_and_wrapped_explosions_count` |
| curse jamais tirée ; variante à chance nulle | avertissement | `curse_that_no_sequence_draws`, `variant_with_a_zero_chance` |
| accomplissement inaccomplissable ; déblocage jamais accordé | avertissement | `achievement_with_an_impossible_condition`, `unlock_that_nothing_grants` |
| `Tears` (trésor, boutique, deux exemplaires) | déclarée | `tears_missing_from_the_*`, `tears_that_cannot_be_held_twice`, `tears_in_a_single_exhausting_pool_cannot_stack` |
| `Effects`, `Familiars` | déclarée | `effects_missing_from_the_boss_pool_and_no_transformation`, `familiars_missing_from_the_treasure_and_shop_pools` |
| `Bombs` (bombe, rochers, piques) | déclarée | `bombs_rocks_and_spikes` |
| `SecretRoom`, `Keys`, `Hearts` | déclarée | `secret_room_*`, `keys_in_rewards_shops_and_locked_treasure`, `hearts_coins_and_shop` |
| `ExhaustPools`, `DevilMarket`, `Champions` | déclarée | `exhausting_pool_drawn_once`, `devil_market_without_a_post_boss_room`, `champions_on_no_reachable_enemy` |
| `Curses`, `Achievements` | déclarée | `curses_only_count_in_the_ordinary_sequence`, `achievements_without_a_visible_unlock`, `achievement_with_an_impossible_condition_is_not_visible` |
| sans `mechanics`, aucune règle déclarée ne parle | — | `undeclared_mechanics_are_not_checked` |

## 2. Décisions à relire

- **`mechanics` dans le manifeste** (écart à la fiche, qui ne le nomme pas). Sans déclaration, le lint ne peut pas savoir qu'`isaac` doit
  avoir un familier : il serait muet, et le critère de sortie M3 « `make lint GAME=isaac` sans avertissement d'atteignabilité » vrai par
  vacuité. Coût : un champ optionnel au manifeste, une ligne dans un test de `game` (le littéral du manifeste). Retrait possible
  sans effet ailleurs : supprimer le champ et `declared()`.
- **Partie ordinaire** = la séquence `entry.floors`. Les règles « toujours actives » gardent, comme §53, toute séquence comme racine.
- **Boutique à `floor_tables`** : `table` n'est pas tiré (`RoomShopDef::table_for_floor`) ; **`salle`** n'est tiré en récompense que
  pour une séquence sans `loot` ou à pool d'étage sans `reward` (`room_reward_system`) : deux écarts à ma première version,
  trouvés par la relecture.
- **`Price`** : aucune surface de contenu avant T8 ; `Price::problems` est testée en unitaire dans `items`. Rien à linter encore.
- **Nombre de tirages par partie** : un minimum prudent (salles exigées × étages ; trois pour le remplissage). `ExhaustPools`
  avertit seulement quand aucun pool à épuisement n'atteint deux tirages.
- **`every_named_stat_has_a_reader`** : la liste est tenue à la main ; son `match` sans `_` fait échouer la compilation à l'ajout d'une
  variante, il ne la fait pas entrer seul dans la liste.

## 3. Vérifié (résultats réels)

| Commande | Résultat réel |
|---|---|
| `cargo test -p content --profile headless` | 108 + 11 (1 ignoré) + 2 + 163 + 6 + 5 + **37 (`m3_t12_fixtures`)** + 8 + 1, 0 échec (sur l'arbre final) |
| `make lint` ×5 | **aucune erreur** sur `zombies`, `testbed`, `throne`, `gungeon`, `isaac` ; avertissements : 0, 106 (97 + 9), 0, 1, 20 |
| `make test_crates` | **1 126 passés, 0 échec, 14 ignorés** (sur la fusion avec `main` b18458cf ; avant fusion : 1 111) |
| `make test_scenarios_par SHARDS=3` (sans BLESS, hors créneau nightly, lancé à 06:27) | **519 scénarios, 3 tranches ok, 1 099 s**, `git status` propre (aucune trace ne bouge) |
| `make check_sim_boundary` | OK : 11 crates, 5 en dette D69 |
| `scripts/check-forbidden.sh`, `check-rollback-registration.sh` | OK (38 occurrences auditées) ; OK |
| `make test_tools` | 19 tests, OK |
| `cargo fmt --all -- --check` | rien à afficher |
| `alacod lint --json games/isaac` | 20 avertissements `MechanicUnmet`, 0 erreur |

Suite sur la fusion avec `main` `b18458cf` : la dernière correction de code (`SecretRoom` sur les sources ordinaires, une ligne) est postérieure
au `test_crates` complet ; elle a été rejouée par `cargo test -p content` et `make lint` ×5 (ci-dessus), pas par un second `test_crates`
(`content::warnings`, donc `mechanics`, n'est appelé que par `alacod lint` et les tests de `content` : aucune autre crate ne l'appelle). Une première suite de scénarios lancée à
06:02 est sortie verte (519 scénarios) mais elle avait un second processus concurrent dans le même `target` (échec de fusion des métriques, sans
rapport avec une trace) : elle a été rejouée seule, résultat ci-dessus.

## 4. Avertissements d'atteignabilité par jeu

| Jeu | Avant | Après | Nouveaux |
|---|---|---|---|
| `zombies` | 0 | 0 | — |
| `throne` | 0 | 0 | — |
| `gungeon` | 0 | 1 | **réel** : `loot/boss.ron`, pool « boss » (11 objets) tiré par personne. Le commentaire du pool le dit : « prévu pour M2-T7, qui le branchera sur la mort du boss ; aucun système ne le tire encore ». Aucune tâche M3 ne le ferme : **dette M2** (le brancher sur le coffre de la salle `boss` ou le retirer) |
| `testbed` | 97 | 106 | 9, tous voulus (bac à sable) : pools d'essai `commun`, `epuisable`, `etage2`, `pieces` non tirés par une partie ; familiers d'essai `compagnon`, `orbital`, `rempart` sans objet atteignable ; synergie `lame_braise` (objet `braise` sans source) ; mur secret sans bombe. Les 97 d'avant gagnent le suffixe « ; mécanique : … » |
| `isaac` | 0 | 20 | voir ci-dessous |

`make lint` : **aucune erreur** sur les cinq jeux.

### `isaac` : 20 avertissements, la tâche qui ferme chacun

| # | Avertissement | Ferme |
|---|---|---|
| 1 | `rooms/diable.ron` : salle post-boss sans coffre ni boutique (marché du diable vide) | **T8** (pool du diable, présentoir payé en vie max) |
| 2 | `rooms/secrete.ron` : mur secret sans bombe jouable (`bomb` : aucun personnage ; l'objet `bomb` est sourcé par `salle`) | **T11a** (le personnage `pleureur` reçoit `bomb` : « La pose exige un `BombCarrier` que le personnage n'a pas encore (M3-T11) », `objets/bomb.ron`) |
| 3, 4 | larmes : aucun objet de larmes dans le pool trésor (`salle`), ni en boutique | **T11a** (objets de larmes) + **T11d** (pool du trésor) ; la boutique : **T8** + T11d |
| 5, 6, 7 | effets : aucun objet à effets dans le pool trésor, boutique, boss | **T11b** (objets) + **T11d** (pools) ; boutique : T8 ; pool du boss : **T11c** + T8 (« pool par type de salle ») |
| 8 | aucune synergie atteignable | **T11b** (transformations, si Q10) |
| 9, 10 | familiers : 0 sur 2 dans le pool trésor, 0 en boutique | **T11b** (familiers) + T11d (pools), boutique : T8 |
| 11 | aucun personnage jouable à `bomb` | **T11a** |
| 12, 13 | aucun rocher, aucune pique dans un gabarit `combat` | **T11d** (gabarits avec rochers, rochers à butin, piques) |
| 14 | salle secrète : aucune bombe atteignable | **T11a** (idem 2 et 11) |
| 15 | aucune clé dans une boutique | **T8** + T11d |
| 16 | aucune boutique exigée par la grammaire | **T11d** (`required: {"boutique": 1}`) + T8 |
| 17 | aucun pool `exhaust` | **T8** (épuisement sur le pool du trésor) + T11d |
| 18 | aucun ennemi à `variants` | **T11c** (champions) |
| 19 | la séquence ordinaire n'a pas de `curses` | **T11d** (curses dans la séquence par défaut : « la séquence par défaut d'`isaac` les tirera à M3-T11d », §74) |
| 20 | aucun accomplissement dont le déblocage entre dans un pool tiré | **T10** + T11d |

Rejouer ce lint « après T11 » (une passe courte) : `alacod lint games/isaac` doit tomber à 0.

## 5. Relecture indépendante

Agent `reviewer`, deux passes. Première passe : trois défauts certains (boutique à `floor_tables`, `salle` toujours compté,
coffre de récompense absent des sites de tirage), cinq faux positifs ou négatifs probables (exemplaires et `exhaust`, bombes de
personnages `test_only`, `Scaled`/`Chance` non lus, familier `test_only`, butin de champion à chance nulle), des claims du §77
inexacts : tous corrigés (commit 6adfb003), avec un test chacun. Seconde passe : « RAS » sur les huit corrections, un point P3 relevé et
corrigé (la règle déclarée `SecretRoom` lisait les sources globales : elle lit maintenant celles de la partie ordinaire) et une limite
documentée au §77 (une entrée ajoutée par un déblocage compte comme source sans vérifier que le déblocage est accordé). Comptes du §77
(isaac 20, testbed +9) rejoués après les corrections et la fusion de T9 : inchangés.

## 6. Non fait / dette

- Pas de lint de `Price` (pas de surface avant T8). Pas de lecture des scénarios (comme §53).
- `lock_from_floor` contre la longueur d'une séquence n'est vérifié que pour les clés (`lint_key_sources`).
- Dette M2 hors tâche : pool `boss` de `gungeon` jamais tiré (§4).
- Les avertissements sont statiques : une salle posée par la grammaire n'est pas une salle *rencontrée* ; les campagnes (T14, T18) le prouvent.

## 7. Atteignabilité

Le lint est lui-même la mesure d'atteignabilité statique de M3 : `alacod lint games/isaac` (§4 ci-dessus), à rejouer après T11.
