# M3-T2 : effets v3

Branche `m3-t2-effets-v3`. Détail des règles : `docs/conventions.md` §67.

## Livré
`OnHit` (via `HitLanded`, émis par le résolveur de dégâts), `OnEvent`/`Emit` (même frame, 8 tours),
`ApplyStatus` sur la cible et `ApplyStatusSelf`, `Scaled { PerCopy }` exécuté, `TargetInRange`, limites
par exemplaire (`ItemDef::stack`, Q16 : indépendantes sur demande, partagées par défaut). Module
`game::effects_v3`. Lint : `ApplyStatus` hors cibles, statut inconnu, `stacks = 0`, noms vides.

## Questions
Q10 (transformations) : non traité ici. Q16 : défaut `Shared`, `Independent` en option par objet.

## Frames à regarder
`effect_on_hit` : f10 tir, poison à f~20-45. `effect_per_copy` : même tir, deux piles.

## Vérification (passage du 2026-10-17, sur la fusion avec `origin/main` 030a4bc0)
- Fusion `origin/main` : sans conflit de texte (M3-T7 touche aussi `effects_runtime.rs`, les deux intentions coexistent) ; compile.
- `make test_crates` : OK, 964 tests.
- `make test_scenarios_par SHARDS=3` (sans BLESS des traces existantes) : OK, 430 scénarios. Traces existantes bougées : **aucune**. Neuves : `effect_on_hit`, `effect_per_copy` (rebénies : le tir est passé de `machine_gun` à `pistol`, f30).
- `make lint` ×5 : OK. `make gen` ×5 : OK, sans diff. `cargo fmt --all -- --check` : OK. `make check_sim_boundary` : OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines, aucune divergence.
- Unitaires : `game::effects_v3::tests` (chaîne de 3 tours dans une frame, plafond `MAX_EVENT_ROUNDS`, `ApplyStatus` sur la cible et `ApplyStatusSelf` sur le porteur, `PerCopy` une exécution × `count`), `game::effects_runtime::tests::cles_de_limites_partagees_ou_independantes`, `crates/scenario/tests/effects_v3.rs` (`HitLanded` dans le vrai résolveur : dégât réel émis, 0 et auto-infligé non, coup perdu dans les i-frames non). Fixtures de lint : `effect_status_no_target`, `effect_status_unknown`, `effect_status_stacks_zero`, `effect_name_empty` (`Emit` et `OnEvent` vides).

## Le `pistol` ne tirait pas (cause)
Ce n'est pas une régression : même résultat sur `origin/main` (sonde minimale, `balles_max=0`, cible intacte). `weapon_probe` : 6 balles, réserve 48, `dernier_tir f0` de f6 à f38. `WeaponState::last_fire_frame` vaut 0 au départ et le tir exige `frame - last_fire_frame >= 60 / firing_rate` ; le `pistol` (`firing_rate: 2.0`) ne peut donc pas tirer avant la frame 30. Les scénarios tirent maintenant à f30. Dette **D83**.

## p2p, deux clients (montage temporaire non commité)
Recette : `docker-compose.ci.yaml` (`MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`), binaire testbed headless, `ALACOD_INPUT_DELAY=0`, 600 frames, second client lancé 8 s après le premier. Montage : `create_player` donnait `Inventory { passives: ["vipere", "vipere"] }` à chaque joueur (annulé par `git checkout`), script d'entrées à deux joueurs qui change toutes les 6 frames.
- Codes de sortie 0 des deux clients ; `cmp` des traces : **identiques**, 599 lignes chacune.
- Frames resimulées (lignes `ggrs{f=N}` en double) : run 1 : client 0, 133 lignes sur 54 frames ; client 1, 100 lignes sur 25 frames. Run 2 (tir constant) : client 0, 150 lignes sur 71 frames ; client 1, 100 lignes sur 25 frames. (Le compteur final du journal affiche 0, il est faux.)
- Mécanisme exercé : 23 `ggrs{f=N effect_status ...}` (poison posé par `OnHit`/`PerCopy`) identiques des deux côtés.
- **Limite** : aucun `effect_status` ne tombe dans une frame resimulée (0 sur 23) ; l'égalité des traces prouve la cohérence du résultat, pas un rejeu de la pose elle-même.
