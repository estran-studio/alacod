# M-6 — Les `moments` du manifeste

Jalon servi : **S2, dimanche 22 novembre 2026** (cible vendredi 20 ; `plan-1837-cli.md` §6, ligne M-6). Elle sert la fenêtre `frappe` de P-1 (§110), jusque-là recopiée à la main en frames dans le RON de 1837.
Branche : `wquintal/m-6-moments`, partie d'`origin/main` `926b025c`. Tête de code poussée et vérifiée : **`670e2cfb`** (`b071a9b7` + la revue P3 du coordinateur, voir plus bas). Aucune note de passation (lot neuf).

## Périmètre et résultat

| Point du lot | Issue | Preuve |
|---|---|---|
| Lire `moments` (`frappe`, `depart`, `deplacement`) et les résoudre en **images entières** au chargement, arrondi écrit | Fait | `content::models::AnimeManifest::clip_moments` : fraction × durée du clip (`(fin − début) / fps`) × 60 images/s, **image la plus proche, demi-image vers le haut**, bornée à la dernière image du clip ; nombres lus en décimal. Tests `moments_resolve_to_whole_frames_rounded_to_nearest`, `moment_ties_round_up_whatever_the_binary_representation`, `a_window_to_the_end_of_the_clip_stops_at_its_last_frame` |
| `frappe` d'un clip → `strike_frames` d'une arme de mêlée liée, **quand le RON ne le donne pas** | Fait | lien explicite `ModelDef::melee_clips` (arme → clip) ; `ModelsTable::strike_windows` ; `game::weapon_assets::apply_moment_strikes` dans `sync_weapon_registry` (chargement et rechargement de l'asset). Registre lu par la simulation : test `moment_strike_window_reaches_the_registry` (`frappe_modele` = `(9, 14)`, `renvoi_secteur` = `(10, 13)` du RON, `axe` = `None`) |
| Le RON prime ; avertissement de lint s'il diffère | Fait | `apply_moment_strikes` ne touche jamais un `Some` (test `the_ron_strike_window_wins_over_the_clip_window`) ; `LintWarningKind::MomentMismatch` (`content::warnings`, n'échoue pas `alacod lint`) : fixture `moments_ron_differs` (RON `(3, 5)` contre clip `(9, 14)` : avertissement « le RON prime », registre inchangé) ; second avertissement quand ni le RON ni aucun clip lié de l'arme (tous modèles) ne donnent de fenêtre (revue P3 : pas d'avertissement si un autre modèle la donne ; test `a_ron_window_that_differs_is_a_warning_and_a_missing_one_is_too`) |
| Clip inconnu | Fait | `BrokenReference` (fixture `moments_clip_unknown`, test `a_bound_weapon_takes_the_window_of_its_clip`) ; rien n'est dérivé |
| `depart` et `deplacement` lus, exposés, **non câblés** | Fait (non câblés : D206) | `ClipMoments::start` / `::movement` (images entières, `distance_m` en donnée de contenu), lint de plage (`invalid_moments_are_lint_errors`) ; aucune lecture par la simulation |
| Lint complémentaire | Fait | fenêtre dérivée qui sort du coup (`fin >= attack_duration_frames`) : fixture `moments_derived_outside_attack` ; deux modèles qui divergent pour une même arme : `two_models_binding_one_weapon_to_different_windows_are_refused` ; arme de mêlée inconnue / clip absent d'`actions` |
| Scénario neutre du testbed dont la frappe tombe dans la fenêtre dérivée, trace neuve seulement | Fait | `tests/scenarios/frappe_moment.ron` + `.trace` (seule référence neuve) : pantin, arme `frappe_modele` sans `strike_frames`, coup de 21 images lancé à f15, cible touchée à l'image 24 (début de la fenêtre `(9, 14)`), compteur 0 jusqu'à f24, 1 dès f25. **Témoin négatif** : sans la ligne `melee_clips` de `models.ron`, la cible est touchée dès f16 et les attentes `EntityHits` des frames 16 et 24 échouent |
| Aucun f32 dans la simulation, aucune trace existante ne bouge | Fait | seul `strike_frames: Option<(u32, u32)>` (champ P-1, hors hash absent) change de valeur ; arme non liée ou clip sans `frappe` : inchangé. Rejoués **sans `BLESS`** : `model_pantin`, `coup_en_file`, `renvoi_secteur`, `renvoi_secteur_visee`, `parade_projectiles`, `parade_projectiles_expiree`, `parade_projectiles_instantanee`, `hit_iframes`, `four_players_idle` (traces identiques). Suite complète non rejouée (la nuit) |
| Contrôle rapide | Fait | `make check_rapide` vert sur `670e2cfb` poussé, code de sortie 0 (un premier passage vert sur `b071a9b7`, journal `…-b071a9b7.log`, avant la revue P3) ; sweep joué ensuite (`scripts/target-sweep.sh target`, pas de `target_wasm`) |
| Docs | Fait | `docs/conventions.md` §116, dette D206 (`docs/taches/dettes.md`), ce rapport, ligne LIVRÉ de `docs/taches.md` |

Aucun crate `SIM` modifié (la dérivation vit dans `content` et `game::weapon_assets`, côté application), aucun système de `GgrsSchedule`, aucun type rollback ; `make check_sim_boundary` vert dans le contrôle rapide. Pas de `make schedule_order_diff` (ordre des systèmes inchangé).

## Décisions

- **Lien explicite `melee_clips`** plutôt que déduit de `poses.melee` : la présentation ne connaît pas l'id d'une arme de mêlée (`MeleeWeapon::config.name` est un nom d'affichage, la clé de `melee_weapons.ron` n'est portée par aucun composant) ; un lien déduit aurait dérivé une fenêtre pour toutes les armes de mêlée d'un jeu et déplacé leurs traces.
- **Fraction × durée du clip à 60 Hz** : c'est la lecture de la présentation (`pose::choose` lit le geste à `elapsed / durée du clip`) ; elle suppose un coup joué sur la durée du clip (limite notée au §116 et à D206 ; le lint garantit seulement que la fenêtre tient dans `attack_duration_frames`).
- **Arrondi** : demi-image vers le haut, nombres du manifeste lus comme leur plus court décimal (0,35 × 20 = 7, pas 6,99999…) ; la borne `fin` est écrêtée à la dernière image du clip pour respecter la règle `fin < attack_duration_frames` de P-1.
- **Testbed** : `scripts/gen-pantin.py` écrit `moments.strike.frappe`, le `.glb` régénéré est identique octet pour octet (seul `anime.json` change).

## Contrôle rapide (journal `~/orca/runs/alpha/logs/m-6-moments-check-rapide-670e2cfb.log`)

Dernières lignes : `test result: ok. 0 passed; … finished in 0.00s` (derniers doctests), puis `✓ Contrôle rapide passé (suite de scénarios : la nuit)` et `EXIT 0` ; 95 blocs `test result` tous sans échec ; `alacod lint` vert pour zombies, testbed, throne, gungeon, isaac.

## Ce qui reste

- **D206** : câbler `depart` (départ de la balle calé sur le clip) et `deplacement` (pas d'une attaque) à la simulation, chacun avec son scénario du testbed.
- La fenêtre est en temps de clip : si 1837 veut un coup plus long ou plus court que son clip, il règle `attack_duration_frames` (ou garde son `strike_frames` en RON, qui prime) ; un `alacod/reload_content` qui ne touche que `models.ron`/`anime.json` n'est relu qu'au rechargement suivant de `melee_weapons.ron`.
- 1837 : lier `hache` à son clip par `melee_clips` et retirer son `strike_frames` recopié quand `moments.frappe` de son manifeste fait foi (hors engine, dans son dépôt).
- La suite complète de scénarios n'a pas été rejouée ici (la nuit).
