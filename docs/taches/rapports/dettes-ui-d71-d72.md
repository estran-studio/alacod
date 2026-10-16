# dettes-ui-d71-d72 : écran de mutation en bevy_hui, mémoire de visite de la minicarte

Branche `dettes-ui-d71-d72` depuis `origin/main`. Présentation seule : rien dans `GgrsSchedule`, aucune trace existante modifiée.

## 1. Fait

- **D71** : gabarit `ui/mutation_screen.html` (throne, testbed) et adaptateur `game::ui::mutation_hui` (`MutationHuiPlugin`, dans `PresentationPlugin`, feature `ui_hui`). `MutationScreenView` et le protocole `ChoiceA/B/C` sont inchangés (`confirm_input` émet toujours le bit dans l'input). Retirés : `mutation_screen.ron`, `MutationScreenLayout`, `lint_mutation_screens`, `Registry::mutation_screens`, fixtures `mutation_screen_font_missing`/`_two_slots`. Conventions §30 et §56 mises à jour.
- **D72** : mémoire dérivée en présentation, sans état rollback. `hud_model::MinimapVisits` + `track_minimap_visits` (joueurs locaux, `Update`, avant `update_hud_snapshot`), remise à zéro au changement de géométrie d'étage ; `minimap_rooms` prend l'ensemble des visites. Le cas « un restart doit garder la visite » n'existe pas (une partie neuve repart de zéro) : pas de variante `Visited` rollback, donc pas de preuve §5.
- Scénario `minimap_shop_visit` (gungeon, `etages_grammaire`, graine 12, 200 frames) : la boutique est voisine ouest du départ (les graines 1 à 12 ont été sondées ; avec la graine 1 de `shop_seeded` la boutique n'est accessible qu'à travers une salle de combat verrouillée). Sa trace est celle du scénario livré ; aucune trace existante n'a été modifiée.

## 2. Vérifié

**Suite rejouée sur la fusion avec `main` `86d7657c`, sans `BLESS` et sans modification de trace.**

| Commande | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` | OK — 368 scénarios, 3 tranches, 177 s |
| `make test_crates` | OK — workspace headless, aucun échec |
| `make lint` | OK — zombies, testbed, throne, gungeon et isaac : aucune erreur |
| `cargo fmt --all -- --check` | OK — aucune sortie |
| `make check_sim_boundary` | OK — 11 crates ECS seul ; 5 dettes D69 signalées |
| Capture mutation | OK — rendu logiciel, `/tmp/ui-d71`, frame 760 : trois cartes, surbrillance or, barre de temps |
| Capture minicarte | OK — rendu logiciel, `/tmp/ui-d72`, frames 40/90/190 : entrée, salle visitée actuelle, puis salle visitée après sortie |

Les scénarios `throne_mutation_*` et `minimap_shop_visit` sont inclus dans la suite ; aucune trace
existante n'a été réécrite.

## 3. Non fait / non vérifié / échecs hors périmètre

- p2p à deux clients, restart en ligne et bench au calme : non vérifiés ici (hors périmètre demandé).
- Le lint et les compilations émettent des avertissements préexistants, sans échec.
- Limite D72 : un rollback ne défait pas une visite ; aucune variante `Visited` rollback n'a été nécessaire.

## 4. Captures et frames à regarder

- D71 : `/tmp/ui-d71/frame_00760.png` — cartes `Irradié`, `Tireur`, `Chasseur`, première carte surlignée en or et barre de délai visible ; frames 750–790 pour l'écran ouvert et le choix.
- D72 : `/tmp/ui-d72/frame_00040.png` — carte autour de la boutique avant l'entrée ; `frame_00090.png` — joueur dans la boutique ; `frame_00190.png` — boutique visitée après sortie.

## 5. Atteignabilité

`minimap_shop_visit` (gungeon, graine 12) rencontre la mécanique ; l'écran de mutation est joué par
`throne_mutation_choice` et les attentes `HudText` couvrent la minicarte.

