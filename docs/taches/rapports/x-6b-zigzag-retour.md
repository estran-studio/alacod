# X-6b — Charge en zigzag et retour au point de départ (voie m, jalon S3 mer 25 nov, lancé en avance le 10 nov)

Branche `wquintal/x-6b-zigzag-retour` (partie d'`origin/main` 18761974). Plan : `docs/taches/plan-1837-cli.md` § 6, ligne X-6b (sa preuve fait foi). Section de conventions : **§ 119**. Dettes : **D214**, **D215**.
Code engine contrôlé : `fc047bc4` (le commit du rapport ne touche que `docs/`). Rien côté 1837, rien de 1837 dans l'engine (testbed : `zigzag_charger`, `straight_charger`, `biter_returner`).
Jalon servi : **S3 mer 25 nov** (les rats : « charge en zigzag puis recul vers leur trou »).

## Points du lot

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Relevé de dix lignes avant tout code | **fait** | envoyé en `status` avant la première ligne de code ; repris dans le § 119 (`behaviors/src/lib.rs:36-38`, `:178-187` ; `ai/rules.rs:282-330`, `:461-468` ; `ai/state.rs:657-693` ; `character/mod.rs:128` ; `enemy/create.rs:296-300` ; `ai/pathing.rs:661-681`, `:921-923` ; `content/registry.rs:470`, `lint.rs:3694`) |
| 2 | Option zigzag de `Charge` en donnée (amplitude, période), `Fixed`, déterministe, absente = identique | **fait** | `Behavior::Charge { telegraph, zigzag: Option<Zigzag> }` (`#[serde(default)]`) ; onde triangulaire, vitesse latérale `4·amplitude·60/période`, phase = frame de la ruée, aucun RNG ; `zigzag_charge_retour` : y de 688 à 705 pour un axe à 696 ; témoin `zigzag_temoin_droit` : y = 696 exactement ; test `zigzag_return::zigzag_deviates_sideways_and_the_straight_charge_does_not` (> 6 avec, < 0,01 sans) |
| 3 | Behavior de retour au point de départ : recule face à la cible, attend N frames, peut recharger, ne poursuit pas | **fait** | `Behavior::ReturnHome { wait, speed, max_frames }` (fin d'enum) ; point d'apparition `BehaviorRuntime::home` ; `zigzag_charge_retour` : morsure f48, recul de f50 à f88, arrivée à 3,1 unités du point (< 4, donc < une case), attente 40 frames, seconde ruée f150 ; test `retreat_faces_the_target_stops_at_the_spawn_point_then_recharges` (orientation `Left` en reculant vers l'est, immobile f95-f125, ruée relancée) ; `retour_apres_morsure` : déclenché par un coup de mêlée, deux boucles |
| 4 | Champs neufs hors `Hash`/`Debug` quand inactifs ; types rollback neufs en variante neutre ; variantes en fin d'enum | **fait** | `Hash` et `Debug` de `BehaviorRuntime` écrits à la main (`home`/`returning` seulement si actifs) ; **aucun type rollback neuf** (le type existant est déjà neutre) ; `ReturnHome` en fin d'enum |
| 5 | Aucune trace existante modifiée (les `Charge` existants inchangés) | **fait, sur un échantillon** | 12 scénarios à `Charge` (`enemy_charge`, `enemy_charger_*`, `enemy_bandana_kin_moving`, `enemy_follet_moving`, `enemy_rampant_moving`, `enemy_buffle_moving`, `enemy_roi_rat_*`, `throne_solo`, `bot_isaac_floor`, `gungeon_classes`) rejoués sans `BLESS` : `5 passed`, `git status` sans `.trace` modifié. Le reste de la suite : la nuit du VPS |
| 6 | Deux joueurs : cible déterministe | **fait, avec un constat** | `zigzag_deux_joueurs` : joueurs à distance égale, toujours le **même** joueur visé (le 1) à chaque charge, rejoué en synctest sans écart. **Constat (D215)** : ce n'est pas le plus petit `net_id` mais l'ordre des cases du champ de flux multi-sources ; le tie-break `net_id` ne joue que dans le repli en ligne droite. Aucun code de sélection nouveau |
| 7 | Scénarios neutres, traces neuves, lint, `describe` | **fait** | 4 scénarios neufs (`zigzag_charge_retour`, `zigzag_temoin_droit`, `zigzag_deux_joueurs`, `retour_apres_morsure`) et leurs `.trace` ; attente neuve `EnemyPosition` ; fixtures `behavior_zigzag_invalid`, `behavior_return_home_orphan` (`lint_fixtures.rs`) ; `cli.rs::describe_lists_zigzag_and_return_home` ; tests unitaires `behaviors` (`charge_sans_option_et_retour_au_depart`, aller-retour RON) |
| 8 | `make check_rapide` vert | **fait** | sur `fc047bc4` : 1648 tests passés, 0 échec ; journal `~/orca/runs/alpha/logs/x-6b-zigzag-retour-check-rapide-fc047bc4.log`, dernières lignes `✓ Contrôle rapide passé (suite de scénarios : la nuit)`, `exit=0` ; sweep de `target` joué après |
| 9 | Docs : § 119, D214–D215, rapport, LIVRÉ | **fait** | `docs/conventions.md` § 119, `docs/taches/dettes.md`, `CLAUDE.md` (liste des attentes), ce rapport, ligne `LIVRÉ` de `docs/taches.md` |

## Choix et pourquoi

- **Zigzag par une vitesse latérale ajoutée**, pas par une direction tournée : l'amplitude est en unités (lisible par un designer), la phase est une fonction pure de la frame de la ruée (aucun état, aucun RNG). `MotionExtra` (`drift`, `face`) est un complément séparé de `BehaviorMotion` : tous les behaviors existants le laissent à `None`, donc zéro octet de simulation changé pour eux.
- **L'attente remplace le refroidissement** de la charge : sans cela, une attente plus courte que `attack_cooldown_frames` laissait le rôdeur sans règle, donc en poursuite par le mouvement d'origine.
- **Deux déclencheurs** (fin de charge, coup de mêlée) parce que la spec dit « après une attaque ou une charge ».
- **État dans `BehaviorRuntime`** plutôt qu'un composant de point d'apparition : pas de nouvelle parité de type vide à gérer.

## Non fait / non vérifié

- `make schedule_order_diff` **non joué** : aucun système, schedule ou type rollback ajouté (`behavior_select_system` et `move_enemies` seulement étendus).
- Pas de suite complète (consigne : la nuit du VPS), pas de preuve p2p réelle ; la preuve de rollback est le synctest du runner sur les quatre scénarios.
- Limites ouvertes : **D214** (recul en ligne droite sans évitement, zigzag qui glisse contre un mur, écart mesuré ≈ 85 % de l'amplitude nominale, `speed` > ≈ 8 peut osciller autour du point), **D215** (tie-break des joueurs à égalité).
- Côté 1837 (à l'arrivée du `rev`) : écrire le rat avec `[ReturnHome(wait: …), Charge(telegraph: …, zigzag: Some((amplitude, period)))]` et, pour un « il ne poursuit pas », ne pas lister `Chase`.
