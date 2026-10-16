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

## Vérification (passage du 2026-10-17, après fusion avec `origin/main` 12a83423, M3-T13)
- `make test_crates` : OK, 973 tests.
- `make test_scenarios_par SHARDS=3` (sans BLESS des traces existantes) : OK, 478 scénarios. Traces existantes bougées : **aucune** (14 traces ajoutées par la branche, 0 modifiée par rapport à `origin/main`).
- `make lint` ×5 : OK, aucune erreur. `make gen` ×5 : OK, sans diff. `cargo fmt --all -- --check` : OK. `make check_sim_boundary` : OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines, aucune divergence.
- Mesure `effects_hundred_items` : voir §67 (0,19 ms contre 0,12 ms sans objets, 0,145 ms `four_players_idle`, trois exécutions, machine partagée).

## Le `pistol` ne tirait pas (cause)
Ce n'est pas une régression : même résultat sur `origin/main` (sonde minimale, `balles_max=0`, cible intacte). `weapon_probe` : 6 balles, réserve 48, `dernier_tir f0` de f6 à f38. `WeaponState::last_fire_frame` vaut 0 au départ et le tir exige `frame - last_fire_frame >= 60 / firing_rate` ; le `pistol` (`firing_rate: 2.0`) ne peut donc pas tirer avant la frame 30. Les scénarios tirent maintenant à f30. Dette **D83**.

## p2p, deux clients (passage précédent, avant la revue)
Recette : `docker-compose.ci.yaml` (`MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`), binaire testbed headless, `ALACOD_INPUT_DELAY=0`, 600 frames, second client lancé 8 s après le premier. Montage : `create_player` donnait `Inventory { passives: ["vipere", "vipere"] }` à chaque joueur (annulé par `git checkout`), script d'entrées à deux joueurs qui change toutes les 6 frames.
- Codes de sortie 0 des deux clients ; `cmp` des traces : **identiques**, 599 lignes chacune.
- Frames resimulées (lignes `ggrs{f=N}` en double) : run 1 : client 0, 133 lignes sur 54 frames ; client 1, 100 lignes sur 25 frames. Run 2 (tir constant) : client 0, 150 lignes sur 71 frames ; client 1, 100 lignes sur 25 frames. (Le compteur final du journal affiche 0, il est faux.)
- Mécanisme exercé : 23 `ggrs{f=N effect_status ...}` (poison posé par `OnHit`/`PerCopy`) identiques des deux côtés.
- **Limite** : aucun `effect_status` ne tombe dans une frame resimulée (0 sur 23) ; l'égalité des traces prouve la cohérence du résultat, pas un rejeu de la pose elle-même.
- Non rejoué après la revue : les correctifs ne changent aucun état rollback (aucun composant ni ressource en plus), seulement le lint, la liste des récepteurs `OnEvent` et du contenu testbed.
