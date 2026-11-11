# X-6c — Se replier derrière un couvert (voie m, jalon S4 mer 2 déc, lancé en avance le 10 nov)

Branche `wquintal/x-6c-repli-couvert` (partie d'`origin/main` 9e937158). Plan : `docs/taches/plan-1837-cli.md` § 6, ligne X-6c (sa preuve fait foi). Section de conventions : **§ 120**. Dette : **D216**.
Code engine contrôlé : `8e92edc5` (les commits suivants ne touchent que `docs/`). Rien côté 1837, rien de 1837 dans l'engine (testbed : `tireur_couvert`, `tireur_couvert_court`, `tireur_sans_repli`).
Jalon servi : **S4 mer 2 déc** (« les volontaires qui se replient derrière les couverts »).

## Points du lot

| # | Point | Issue | Preuve |
|---|---|---|---|
| 0a | Préalable 1 : compteur de glissement remis à zéro pendant le retour (`homing`) | **fait** | `pathing.rs` (`slide_frames_after`, remise à zéro avant le mouvement), test `slide_counter_does_not_run_while_homing` (vert) ; les scénarios à retour (`rodeur_garde_*`, `zigzag_*`, `retour_apres_morsure`) gardent leur trace |
| 0b | Préalable 2 : garde bloquée par un mur libérée après `max_frames` | **fait** | `BehaviorRuntime::guard_until` (haché seulement s'il est posé), `next_guard_until`, `guard_motion(…, now, …)`, test `a_blocked_guard_gives_up_after_max_frames` (vert) ; mêmes six scénarios, traces inchangées. Commit séparé `6552f788` |
| 1 | Relevé de dix lignes avant le code du lot | **fait** | `status` envoyé avant `Behavior::TakeCover` ; repris dans le § 120 |
| 2 | Behavior générique à nom neutre, fin d'enum : après un tir, point abrité le plus proche derrière un couvert (ligne de vue coupée), dans un rayon en donnée, s'y rendre, y rester N frames, reprendre ; sans couvert à portée : comportement actuel | **fait** | `Behavior::TakeCover { radius, hold, reload, speed, max_frames }` (`behaviors/src/lib.rs`), sélection et état `CoverPhase` (`ai/rules.rs`, `ai/state.rs`), géométrie pure `ai/shelter.rs` (`Fixed` et entiers, aucun `f32`/RNG, couverts en ordre `GgrsNetId`) ; déclenchement au début du rechargement du `Shoot` (« pendant son rechargement », `reload` prolonge l'attente jusqu'à sa fin) |
| 3 | Tie-break déterministe (distance, `GgrsNetId` du couvert, position) | **fait** | `shelter::find_shelter` : deux passes indépendantes de l'ordre (plus courte distance, puis plus petite clé parmi les points à 1/16 d'unité) ; tests unitaires (`equal_distance_picks_the_smaller_net_id`, avec dérive du tireur) et scénarios `repli_couvert_egalite` / `_inverse` |
| 4 | État rollback : champs neufs hachés seulement actifs, variante neutre, variante d'enum en fin | **fait** | `BehaviorRuntime` (déjà neutre) gagne `cover`, `cover_done`, `guard_until`, `Hash`/`Debug` à la main (actifs seulement) ; `Behavior::TakeCover` en dernier ; aucun type rollback neuf, aucun système neuf |
| 5 | Scénarios **neutres** du testbed, traces neuves seulement | **fait** | six scénarios et leurs `.trace` : `repli_couvert_apres_tir` (tir fini f53, repli f54-f72 vers (601, 656), ligne de vue **coupée** à f80 et f120 par `CoverSight`, dégagée à f20 et f60, reprise `Chase` dès f133, dégagée à f150), `repli_couvert_temoin` (sans `TakeCover` : jamais coupée), `repli_couvert_sans_couvert` et `repli_couvert_hors_portee` (mêmes positions que le témoin aux mêmes frames : comportement inchangé), `repli_couvert_egalite` et `_inverse` (deux caisses symétriques : choix par `GgrsNetId`, de l'autre côté quand l'ordre est inversé ; le runner joue en synctest) |
| 6 | Aucune trace existante modifiée | **fait, sur un échantillon** | 42 scénarios à comportements (`Shoot`, `Charge`, `ReturnHome`, `KeepDistance`, `Strafe`, `Flee`, `Wander`, `couvert_*`, `enemy_*`, `tireur_poste_*`, `rodeur_garde_*`, `zigzag_*`) rejoués sans `BLESS` sur le code avec `TakeCover`, 0 écart ; suite complète non jouée (consigne : la nuit du VPS) |
| 7 | Lint et `describe` | **fait** | erreurs `OutOfRange` (rayon, `hold`, vitesse, durée, `Shoot` requis, ordre : seul `Flee` précède), fixture `behavior_take_cover`, test `behavior_take_cover_fixture_reports_trigger_order_and_bounds` ; `describe_lists_take_cover` (`cli.rs`) |
| 8 | `make check_rapide` vert | **fait** | sur `8e92edc5` : 1663 tests passés, 0 échec ; journal `~/orca/runs/alpha/logs/x-6c-repli-couvert-check-rapide-8e92edc5.log`, dernière ligne `✓ Contrôle rapide passé (suite de scénarios : la nuit)`. Un premier passage sur `ee939073` avait échoué sur un test unitaire de mon cru (`equal_distance_picks_the_smaller_net_id` : la quantification de la distance se trompait au bord d'un quantum) ; remplacée par une tolérance indépendante de l'ordre |
| 9 | Docs : § 120, D216, rapport, LIVRÉ | **fait** | `docs/conventions.md` § 120, `docs/taches/dettes.md` (D216, et une phrase ajoutée à D214 pour le suivi), `CLAUDE.md` (liste des attentes), ce rapport, ligne `LIVRÉ` de `docs/taches.md` |

## Choix et pourquoi

- **Déclencheur = le début du rechargement** (`RangedAttackState::ready_at` dans le futur), pas un événement « tir parti » : pas d'état de détection, une seule valeur retenue (`cover_done`) évite de recommencer pour le même tir. `reload: true` étend l'attente jusqu'à la fin du rechargement (la variante « pendant son rechargement » de la spec) ; sans lui, `hold` frames.
- **Test de ligne de vue propre au couvert** : `line_of_sight` ne lit que les murs de l'IntGrid, un couvert est un personnage prop. Le segment/boîte (produits croisés en `i128`, bords compris) est partagé avec l'attente de scénario `CoverSight`, donc la preuve mesure ce que le behavior calcule.
- **Trajet droit, point choisi une fois** : même parti que `ReturnHome` (§ 119) ; un champ de flux vers un point arbitraire n'existe pas (il est multi-sources depuis les joueurs). Limites en D216.
- **Tolérance de distance et non égalité exacte** : le tireur dérive de quelques bits (`696,00023`) dès qu'il tire ; une égalité exacte ne se produisait jamais et le tie-break par `GgrsNetId` aurait été mort. Constat fait en écrivant `repli_couvert_egalite`.

## Non fait / non vérifié

- `make schedule_order_diff` **non joué** : aucun système ni type rollback ajouté (`behavior_select_system` étendu seulement).
- Pas de suite complète de scénarios (consigne : la nuit du VPS), pas de preuve p2p réelle ; la preuve de rollback est le synctest du runner sur les six scénarios.
- Limites ouvertes : **D216** (trajet droit, point choisi une fois, réessai à chaque frame du rechargement, `Shoot` à la reprise même si le joueur est caché).
- Disque : 38 Go libres en fin de lot (`target` 26 Go, `target-sweep.sh` ne libère rien de plus).
- Côté 1837 (à l'arrivée du `rev`) : le volontaire s'écrit `[TakeCover(radius: "…", hold: …, reload: true), Shoot(…), Chase(…)]` (`TakeCover` **en tête**, `Shoot` dans la liste, la caisse porte `cover: Some(Indestructible)`).

## Dernières lignes de `make check_rapide` (8e92edc5)

```
✓ Contrôle rapide passé (suite de scénarios : la nuit)
exit=0
```

## Suivi de revue (branche `wquintal/x-6c-repli-couvert`, partie de `86440e8a`, un P2 et un P3)

Code contrôlé : `bdda38f0` (`make check_rapide` vert : 1666 tests, 0 échec ; journal `~/orca/runs/alpha/logs/x-6c-repli-couvert-check-rapide-bdda38f0.log`, dernière ligne `✓ Contrôle rapide passé (suite de scénarios : la nuit)`).

| # | Constat | Issue | Preuve |
|---|---|---|---|
| P2 | Point abrité inatteignable : seule la case du centre était testée contre `wall_cells` ; corps (demi-étendues, décalage du collider) et autres caisses ignorés | **fait** | `shelter::Body`, `find_shelter` (corps à `BODY_GAP` du bord, position = centre du corps − décalage ; rejet si le corps chevauche une caisse ou si le trajet en traverse une), `rules::body_cells_free` (toutes les cases sous le corps) ; ligne de vue vers le **centre du corps** avec tolérance d'arrivée (4 unités) ; `CoverSight` lit le centre du collider. Scénarios `repli_couvert_rang` (échoue sans : `TakeCover` encore retenu à f170) et `repli_couvert_mur` (échoue sans : x = 629,7 contre le mur à f120). Tests unitaires `a_neighbouring_cover_blocks_the_points_it_overlaps`, `the_collider_offset_moves_the_point_not_the_body`, `a_wall_under_the_arrival_body_rejects_the_point` |
| P3 | Déclencheur sur `now < ready_at` : part aussi sur `first_wait`, recul de coup, attente tirée de `ranged_cycle` | **fait** | déclenchement sur la **fin réelle d'une salve** : `BehaviorRuntime::cover_shooting` (séquence de la frame d'avant) et `cover_pending` (remplacent `cover_done`), hachés seulement actifs. Scénario `repli_couvert_premiere_salve` + `tireur_couvert_cycle` (`first_wait` 40) : échoue sans (`TakeCover` dès f20, `Chase` attendu) |

- **Preuve « échoue sans »** : les trois scénarios neufs rejoués avec l'ancienne logique (corps ignoré, déclencheur sur `ready_at`, retouche temporaire non commitée) échouent comme décrit ci-dessus ; retouche retirée.
- **Traces** : 42 scénarios à comportements et tir rejoués sans `BLESS`, 0 écart. Les traces `repli_couvert_apres_tir`, `_egalite`, `_egalite_inverse`, `_hors_portee`, `_sans_couvert` (TakeCover) sont **réécrites** (voulu : champs hachés neufs, points d'arrivée décalés par le corps) ; `_temoin` inchangée ; trois traces neuves (`rang`, `mur`, `premiere_salve`).
- **Constat de scénario** : le tireur avance de quelques unités vers le joueur avant sa volée (règle `Shoot` sans mouvement propre) ; les scénarios d'égalité placent donc le tireur sur l'axe (600, 696) et les caisses en y 666 et 726, sinon l'égalité de distance se rompt (diagnostiqué par un `eprintln` temporaire, retiré).
- Non fait : `schedule_order_diff` (aucun système ni type rollback ajouté), suite complète (la nuit sur le VPS).
