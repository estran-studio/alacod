# M3-T2 : effets v3

Branche `m3-t2-effets-v3`. Détail des règles : `docs/conventions.md` §67.

## Livré
Points 1, 2, 3, 6 de la fiche et leurs scénarios : `OnHit` (`HitLanded`, émis par le résolveur de dégâts),
`OnEvent`/`Emit` (même frame, 8 tours ; **composition linéaire** : une action non `Scaled` s'exécute par
exemplaire, le récepteur `OnEvent` écoute une fois par objet), `ApplyStatus` sur la cible (`OnHit`,
`OnDamageTaken`) et `ApplyStatusSelf`, `TargetInRange`, `OnPickup` filtré (`item`), `Scaled { PerCopy }`
(actions linéaires seulement), limites par exemplaire (`ItemDef::stack`, Q16 : partagées par défaut,
`Independent` en option). Point 4 (exemplaires) : aucune trace existante ne bouge. Module `game::effects_v3`.
Lint : `ApplyStatus` hors cible (dont `OnKill`), statut inconnu, `stacks = 0`, noms vides, `item` hors
`OnPickup` ou inconnu, `PerCopy` non linéaire.
Échelle (point 7) : `effects_hundred_items`, mesure au §67 (+0,07 ms par frame, `listed_effects` non précalculé).

## Non livré
- Point 5, transformations (`requires_tag`, `transformation_three`) : conditionné à Q10, non retenu ici.
- Hors périmètre : T11b (contenu isaac), T16 (HUD de transformation).
- `OnPickup(Some(item))` de la fiche est réalisé par le champ `item` de l'effet (décision documentée au §67).

## Questions
Q10 (transformations) : non traité. Q16 : défaut `Shared`, `Independent` en option par objet.

## Frames à regarder
`effect_on_hit` : f30 tir, poison dès l'impact. `effect_per_copy` : deux piles, +2 de dégâts. `effect_on_event` : f10, dégâts 1 → 2.
`effect_pickup_filtered` : f110 (bottes), dégâts 1 → 2. `effect_target_in_range` / `effect_scale_per_copy` / `effect_apply_status` : coup vers f40-45.

## Revue du 2026-10-17 : correctifs
1. Composition `Emit` × `OnEvent` linéaire (`listed_effects` : récepteur pour le premier exemplaire seulement) ; `effect_per_copy` : +2 (trace rebénie seule) ; unitaire `un_recepteur_on_event_ecoute_une_fois_par_objet`.
2. Scénarios `effect_on_event`, `effect_target_in_range`, `effect_apply_status`, `effect_pickup_filtered`, `effect_scale_per_copy` (chacun avec `_temoin`) et `effects_hundred_items` (+ témoin) ; objets testbed `echo`, `portee`, `morsure`, `filtre`, `empile`, `cent_*`.
3. Lint `item` hors `OnPickup` et objet inconnu (fixtures `effect_item_filter_wrong_trigger`, `effect_item_filter_unknown`) ; §67.
4. Lint `PerCopy` : liste blanche `effects::actions::is_linear` (fixtures `effect_per_copy_not_linear`, témoin `effect_per_copy_linear_ok`).
5. `ApplyStatus` sur `OnKill` refusé (fixture `effect_status_on_kill`), §67.
6. Dette du `pistol` : D83 (D82 pris par `m2-t14d-gabarits`).

## Troisième passage de la revue (2026-10-17)
1. `apply_event_effects_system` : l'`EffectState` d'un porteur qui n'en a pas encore vit maintenant dans le système d'un tour à l'autre (`locals`), inséré une fois à la fin. Avant : chaque tour repartait d'un état vide (l'insertion par `Commands` n'est visible qu'après le système) et un `OnEvent` à `charges: Some(1)` qui soigne et réémet s'exécutait à chaque tour (8 soins). Test `un_effet_a_charges_ne_se_recharge_pas_a_chaque_tour` : 1 soin ; il échoue (18 au lieu de 11) quand l'état est remis à zéro à chaque tour.
2. `has_per_copy` parcourt les enveloppes `Scaled` comme le lint et `classify` : `Scaled(None, Scaled(PerCopy, Heal(1)))` avec 3 exemplaires soigne 3. Test `per_copy_sous_une_enveloppe_sans_echelle_est_multiplie` (échoue en ne regardant que l'enveloppe extérieure).
3. `effects_hundred_items` : 97 `cent_*` (charge) + 3 observateurs à une charge, un par famille, avec une conséquence visible comparée au témoin : `Tick` → `Luck` (1 contre 0), `OnHit` → `HealthRegen` (11 contre 10), `OnKill` → `Damage` (2 contre 1). Les soins des `cent_*` à pleine santé ne prouvaient rien. Les stats propres au joueur sont seules lisibles (le lint refuse une stat sans lecteur, `Armor` n'existe pas sur le joueur). Les deux morts : le `dummy` (net id 15) et le `follower` (12) de la carte `props_room`, le 3e `Enemy` ; la `target` (14) survit. `KillsAtLeast` ne compte que les `WaveEnemy` : le scénario compte donc les `Enemy` (3 à f20, 1 à f599). Trace de `effects_hundred_items` rebénie (les observateurs ont changé l'état) ; le témoin garde la sienne.
4. p2p rejoué, voir plus bas.

## Vérification (passage du 2026-10-17, après fusion avec `origin/main` 0402a071, M2-T14d)
- `make test_crates` : OK, 975 tests.
- `make test_scenarios_par SHARDS=3` (sans BLESS des traces existantes) : OK, 482 scénarios. Traces existantes bougées : **aucune** (14 traces ajoutées par la branche, 0 modifiée par rapport à `origin/main`).
- `make lint` ×5 : OK, aucune erreur. `make gen` ×5 : OK, sans diff. `cargo fmt --all -- --check` : OK. `make check_sim_boundary` : OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines, aucune divergence.
- Mesure `effects_hundred_items` : voir §67 (0,19 ms contre 0,12 ms sans objets, 0,145 ms `four_players_idle`, trois exécutions, machine partagée ; mesurée avant les observateurs, qui ne changent pas l'ordre de grandeur).

## Le `pistol` ne tirait pas (cause)
Ce n'est pas une régression : même résultat sur `origin/main` (sonde minimale, `balles_max=0`, cible intacte). `weapon_probe` : 6 balles, réserve 48, `dernier_tir f0` de f6 à f38. `WeaponState::last_fire_frame` vaut 0 au départ et le tir exige `frame - last_fire_frame >= 60 / firing_rate` ; le `pistol` (`firing_rate: 2.0`) ne peut donc pas tirer avant la frame 30. Les scénarios tirent maintenant à f30. Dette **D83**.

## p2p, deux clients (passage du 2026-10-17, avec rollbacks réels)
**Correction des passages précédents** : les « frames resimulées comptées sur les `ggrs{f=N}` en double » étaient fausses. Les lignes `effect carrier=… on=OnHit` sont écrites une fois par exemplaire (deux vipères = deux lignes identiques), et le compteur du journal (`frames resimulées après rollback : 0`) était exact : ces runs n'ont **aucun rollback** et ne prouvaient rien. À ne pas reprendre.

Recette : `docker-compose.ci.yaml` (`MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`), binaire testbed headless, `ALACOD_INPUT_DELAY=0`, 3000 frames, montage temporaire non commité (annulé par `git checkout`) : `create_player` donne `Inventory { passives: ["vipere", "vipere"] }` ; script d'entrées à deux joueurs qui change toutes les 1 à 3 frames (tir, déplacement, visée) ; le client 1 est suspendu et repris (`SIGSTOP`/`SIGCONT`, 12 ms chacun) pour que le client 0 le devance et se trompe de prédiction. Mesures par deux `eprintln!` temporaires (une par enregistrement de frame déjà vue, une par pose de statut), retirés.
- Codes de sortie 0 des deux clients, aucun `DesyncDetected`.
- Client 0 : `frames resimulées après rollback : 12374`, client 1 : 0.
- **Poses dans des frames resimulées** : 262 exécutions de `effect_status` pour 67 poses distinctes (frame, cible) ; 53 poses exécutées plusieurs fois (jusqu'à 7), par exemple `f=56 target=38 poison stacks=2` quatre fois. Les 53 poses du client 1 se retrouvent à l'identique chez le client 0 (53/53, avant f2985) ; les 14 poses de plus du client 0 sont des versions prédites, annulées.
- `cmp` des traces : identiques sur les 2991 premières lignes ; les 8 dernières frames diffèrent (fenêtre de prédiction : le client 0 s'arrête à f3000 avec des entrées prédites du pair, rien n'est confirmé). Aucun desync signalé par GGRS ; la recette sans suspension, sans rollback, donnait des traces entièrement identiques.
