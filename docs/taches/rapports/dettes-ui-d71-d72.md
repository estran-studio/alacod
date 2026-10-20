# dettes-ui-d71-d72 : écran de mutation en bevy_hui, mémoire de visite de la minicarte

Branche `dettes-ui-d71-d72` depuis `origin/main`. Présentation seule : rien dans `GgrsSchedule`, aucune trace existante modifiée.

## 1. Fait

- **D71** : gabarit `ui/mutation_screen.html` (throne, testbed) et adaptateur `game::ui::mutation_hui` (`MutationHuiPlugin`, dans `PresentationPlugin`, feature `ui_hui`). `MutationScreenView` et le protocole `ChoiceA/B/C` sont inchangés (`confirm_input` émet toujours le bit dans l'input). Retirés : `mutation_screen.ron`, `MutationScreenLayout`, `lint_mutation_screens`, `Registry::mutation_screens`, fixtures `mutation_screen_font_missing`/`_two_slots`. Conventions §30 et §56 mises à jour.
- **D72** : mémoire dérivée en présentation, sans état rollback. `hud_model::MinimapVisits` + `track_minimap_visits` (joueurs locaux, `Update`, avant `update_hud_snapshot`), remise à zéro au changement de géométrie d'étage ; `minimap_rooms` prend l'ensemble des visites. Le cas « un restart doit garder la visite » n'existe pas (une partie neuve repart de zéro) : pas de variante `Visited` rollback, donc pas de preuve §5.
- Scénario `minimap_shop_visit` (gungeon, `etages_grammaire`, graine 12, 200 frames) : la boutique est voisine ouest du départ (les graines 1 à 12 ont été sondées ; avec la graine 1 de `shop_seeded` la boutique n'est accessible qu'à travers une salle de combat verrouillée). Trace **nouvelle** (`.trace` créée) ; aucune autre trace modifiée (`git diff origin/main -- '*.trace'` : seule cette création).

## 2. Vérifié (suite rejouée sur la fusion avec main `f81caa7`, sans BLESS sur l'existant)

| Commande | Résultat |
|---|---|
| `make test_scenarios` (complet) | `5 passed; 0 failed; 8 ignored`, 927 s, aucune trace réécrite |
| `make test_crates` | 792 réussis, 0 échec, 10 ignorés |
| `make lint` ×4 | zombies, testbed, throne, gungeon : « aucune erreur » |
| `cargo fmt --all -- --check` | propre |
| `./scripts/check-rollback-registration.sh` | OK |
| `make check_sim_boundary` | OK, 11 crates, 5 en dette D69 |
| `cargo test -p game --features ui_hui --lib ui::` | 46 réussis, dont `gabarit_des_jeux_construit_et_recompile` (vrais gabarits throne et testbed) |

## 3. Non fait / non vérifié / échecs hors périmètre

- `./scripts/check-forbidden.sh` **échoue** : `crates/game/src/performance.rs:37,62` (`Instant::now()`, 2 occurrences, aucune exception). `make test_tools` échoue pour la même raison (`test_current_repository_has_no_unreviewed_or_stale_exception`). Ce fichier vient de `origin/main` (commit 41d3859, perf du flow field), je ne l'ai pas modifié : échec déjà présent sur main, à traiter par son auteur (exception justifiée dans `determinism-allowlist.json`, c'est de la présentation).
- p2p à deux clients, restart en ligne, bench au calme : non vérifiés ici (cloud, pas de Docker ; rien ne touche la simulation).
- **Captures** : non faites (pas de GPU). À regarder avec `make play_scenario SCENARIO=throne_mutation_choice` : la carte surlignée est bordée et titrée en or, la barre de temps se vide ; pour la minicarte, `minimap_shop_visit` f40-80 (entrée dans la boutique), f90 (« boutique · visitée · actuelle »), f199 (boutique visitée, départ actuel).
- Limite D72 : un rollback ne défait pas une visite.

## 4. Atteignabilité

`minimap_shop_visit` (gungeon, graine 12) rencontre la mécanique ; l'écran de mutation est joué par `throne_mutation_*`.
