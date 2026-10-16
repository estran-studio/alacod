# M3-T3 : cœurs, i-frames, sous, bombes et clés au sol

Branche `m3-t3-coeurs-pickups` (depuis `origin/main` a9432ed9, fusionnée avec `0a719425`), worker b1, 2026-10-16.
Spécification : `docs/conventions.md` §68. Réponses de William appliquées : **Q2** (demi-cœur = 1 PV, `ExtraHealth { soul }`
neutre absorbé avant les PV point par point, cœurs d'âme en présentation), **Q4** (sous, bombes, clés **par joueur**), **Q6**
(`ExtraHealth` lisible par `can_pay`, rien décidé pour T8).

## 1. Fait

1. **Couche de vie** : `combat::actors::ExtraHealth { soul: Fixed }` (rollback neutre), consommable « cœur d'âme » par la nouvelle action
   `GainSoul(Fixed)` (variante en fin d'enum), attente `ExtraHealth(handle, n, at_frame)`. **Ordre retenu des couches, testé** :
   bouclier → armure → `ExtraHealth` → PV, dans la fonction pure `combat::damage::apply_layers` (appelée par
   `rollback_resolve_damage_events`) ; `DamageKind::True` ignore bouclier et armure mais **pas** l'âme ; un coup absorbé par l'âme
   démarre les `hit_iframes`, compte pour `HitLanded`/`HitCount`/crédit de points, mais n'accumule rien aux PV (donc pas d'`OnDamageTaken`).
   Unitaires : `combat::damage::layer_tests` (6) ; dans le vrai résolveur : `crates/scenario/tests/extra_health.rs` (4 : âme point par point,
   ordre bouclier/armure/âme/PV sur trois coups, coup absorbé qui démarre les i-frames, dégâts vrais).
2. **`hit_iframes: Some(60)`** sur `pleureur` (le joueur d'`isaac`), en données. `hit_iframes_isaac` : 4 PV à f130 et 3 à f260 ; sans la
   donnée 3 et 1 (témoin permanent : `extra_health.rs::isaac_without_hit_iframes_loses_more`, qui efface la donnée dans les assets).
3. **Sous au sol** : `LootPick::Currency(n)` en fin d'enum ; `currency:<n>` ; posé en `ShellPickup` par le coffre, la récompense de salle, le
   butin de champion et le butin de rocher ; refusé dans un pool de boutique (lint, y compris par déblocage). La mort d'un ennemi d'`isaac`
   lâche déjà des sous par `shells_drop` (M2-T6) ; un champion peut en lâcher par son pool.
4. **Bombes et clés au sol, plafonds exécutés** : `ItemDef::accepts` (pure) ; au contact, le premier joueur à portée **qui peut prendre l'objet** (un
   joueur au plafond laisse l'objet à l'autre, sinon il reste au sol) ; en boutique, refus sans dépense (`shop_full`). `isaac` : `key` et `bomb` à 99,
   pool de récompense de salle élargi ; `testbed` : `bomb` à 3. Lint : `starting_consumables` > `max_stack`.
5. **Cœur au sol à pleine vie** : champ `refuse_when_full: true` (déclaré par `coeur`), **comme la fiche le proposait** : seuls les consommables qui le
   déclarent restent au sol ; la `ration` garde son comportement (ramassée sans effet). Lint : exige un consommable avec `Heal`.
6. **Coop par joueur** (Q4) : rien de nouveau à coder (l'`Inventory` et le `Currency` sont déjà par joueur) ; le HUD gagne la source `bombs` (joueur affiché),
   `isaac/ui/hud.ron` l'affiche. Scénario à deux joueurs `coop_pickups_per_player`.
7. **Scénarios** (chacun avec `// À regarder :`) : `soul_heart_absorbs` (+ `_temoin`), `hit_iframes_isaac`, `coin_pickup`, `bomb_pickup_cap`,
   `heart_full_stays` (+ `_temoin`), `coop_pickups_per_player` : **8 traces neuves**, bénies une à une. Mutations jouées à la main (données seules) : sans
   `max_stack` ou sans `refuse_when_full`, `bomb_pickup_cap`, `coop_pickups_per_player` et `heart_full_stays` échouent (4 bombes au lieu de 3 ; santé 92 au lieu de 96).
8. **Lint** : `GainSoul = 0`, `Currency(0)` (pool et déblocage), `Currency` dans un pool de boutique (pool, étage, déblocage), `refuse_when_full` sans `Heal`,
   `starting_consumables` > `max_stack` : 7 fixtures (`crates/content/tests/m3_t3_fixtures.rs`).

## 2. Traces bougées (preuve §5)

**Six traces `isaac`**, aucune autre : `isaac_floor_seed_1`, `generated/isaac/{boss_gardien, enemy_mouche_moving, enemy_mouche_still, enemy_suiveur_moving,
enemy_suiveur_still}`. Cause : `hit_iframes: Some(60)` de `pleureur` (le pool de récompense d'isaac a changé aussi : il ne se lit que si une récompense tombe).
Preuve : dump détaillé (`ALACOD_DUMP_TRACE`) des six scénarios sur `origin/main` a9432ed9 (worktree à part) et sur la branche, `scripts/trace-diff.py` sans `--ignore` :
la première frame qui diffère est, pour les six, **la frame du premier coup reçu par le joueur** (f555, f92, f73, f202, f77, f264), avec **une seule ligne différente**, celle du joueur, et un seul champ :
`Health { current: 5, max: 6, invulnerable_until_frame: None }` → `Some(f+60)` (615, 152, 133, 262, 137, 324). `ExtraHealth` n'a aucun porteur dans ces parties (type neutre : aucune ligne).
Toutes les autres traces (testbed, zombies, gungeon, throne, générées des quatre autres jeux, `isaac_boots`, `generated/isaac/room_combat`) sont **identiques**.

## 3. Vérifié (sur la fusion avec `origin/main` 0a719425)

- `make test_scenarios_par SHARDS=3` sans BLESS des traces existantes : **490 scénarios, trois tranches ok** (227 s) ; 482 d'avant + 8 neufs, **6 traces `isaac` rebénies** (voir § 2), aucune autre bougée.
- `make test_crates` : **997 tests, 0 échec** (991 avant les deux fixtures de déblocage et la fusion). `make test_tools` : 18 ok.
- `make lint` ×5 (`zombies`, `testbed`, `throne`, `gungeon`, `isaac`) : aucune erreur. `make gen` ×5 : tous les scénarios générés ok, **aucun diff** de `tests/scenarios/generated`.
- `cargo fmt --all -- --check` : ok. `make check_sim_boundary` : ok (11 crates, 5 en dette D69). `check-forbidden.sh` : ok (39 occurrences auditées). `check-rollback-registration.sh` : ok.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines, 3600 frames, aucune divergence ; `ALACOD_FUZZ_BOMBS=1` : ok.
- `cargo test -p scenario --test extra_health` : 5 ok ; `-p content --test m3_t3_fixtures` : 7 ok.
- Docker : OrbStack (`PATH=$HOME/.orbstack/bin:$PATH`), `MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`.

## 4. p2p, deux clients, rollbacks forcés (`scripts/p2p-stress.sh`)

Montage temporaire **non commité** (annulé par `git checkout`), jeu `testbed` : un système `GgrsSchedule` qui, toutes les 45 frames dès f60 et pour chaque joueur (ordre `GgrsNetId`),
pose un `coeur_ame` à ses pieds (f%45 = 0), un tas de 3 sous (10), une `bomb` (20, plafonnée à 3, donc refusée ensuite) et lui inflige 5 dégâts « ennemis » (30) : absorption partielle (4 d'âme, 1 PV).
Entrées : `scripts/p2p-stress-inputs.py 3000` (changent toutes les 1 à 3 frames), `ALACOD_INPUT_DELAY=0`, client 1 suspendu 12 ms toutes les 30 ms, `MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`, 3000 frames.
- **Run 1** : codes 0/0, `DesyncDetected` 0, **10 862 / 1** frames resimulées, `cmp` : traces **identiques (2999 lignes)**. Le script l'a pourtant déclaré en échec : 3 `NetworkInterrupted` (timeout 1500 ms) chacun suivi de `NetworkResumed`
  (une suspension plus longue que prévu : machine partagée, compilation du binaire par le script juste avant). Ce n'est ni un desync ni une déconnexion définitive ; relancé aussitôt.
- **Run 2** : « p2p-stress : OK », codes 0/0, `DesyncDetected` 0, déconnexions 0, **4906 / 25** frames resimulées, traces **identiques, 2999 lignes** (= FRAMES-1).
  Journal : 327 `soul_absorb` exécutés au client 0 pour 108 (frame, joueur) distincts (94 rejoués, jusqu'à 6 fois), 103 au client 1 ; 392 ramassages de cœur d'âme, 386 de sous, 22 de bombes (6 au client 1, plafond atteint) ; sur les 99 absorptions que les deux clients ont en commun avant f2985, la valeur finale est identique (99/99).
  Huit absorptions du client 0 n'existent pas au client 1 : exécutions prédites puis annulées (le client 1, derrière, ne voit que des entrées confirmées ; [INFERENCE] le joueur y est en dash, donc invulnérable, la valeur finale ne logue rien).

## 5. Non fait, incertain

- **Pas de bench** (machine partagée, consigne).
- `Price::can_pay`/`pay` ne reçoivent pas `ExtraHealth` : le paiement par cœurs d'âme (Q6) est T8 ; `ExtraHealth::soul` est public, rien d'autre n'a été prévu.
- Aucun plafond de vie d'âme (Isaac : 12 cœurs au total) ; HUD des cœurs : T16.
- `OnDamageTaken` ne se déclenche pas sur un coup que l'âme absorbe en entier ; à reprendre si un objet doit y réagir.
- La pose de bombe d'`isaac` (`BombCarrier`) reste à M3-T11 : les bombes d'`isaac` se ramassent et se comptent, rien ne les pose encore.

## 6. Dettes, questions

- **Relecture indépendante** (agent `reviewer`, lecture seule) : 3 constats. (1) `state_trace.rs` « supprimé » : faux positif, la comparaison se faisait contre un `origin/main` déjà avancé (le correctif `outillage-p2p-stress` n'est pas à moi, il disparaît à la fusion). (2) `Scaled` en `OnUse` : réel, antérieur, voir dette T3-1. (3) `Currency(0)` ajouté par un déblocage : corrigé (lint + fixture `unlock_currency_zero`).
- **Dette T3-1 (antérieure à la tâche)** : `items::apply_instant_effects` n'exécute pas l'enveloppe `Scaled` d'un `OnUse` (consommable, actif) alors que le lint la déballe et valide l'action intérieure (T0b) :
  `OnUse: [Scaled(GainSoul(..))]`, comme `Scaled(Heal(..))` et `Scaled(GainShield(..))`, est accepté puis sans effet. Découvert à la relecture ; documenté au §68, pas corrigé (hors fiche, le sens de `PerCopy` pour un consommable
  consommé au ramassage est à décider).
- **Observation** : le test `hit_iframes.rs::shield_absorb_uses_its_own_short_iframes_not_the_hit_ones` (M3-T0b) est vacuous : ses trois coups (f12, f30, f40) donnent 4992 avec ou sans bouclier à cause des 30 frames de `hit_iframes`. Non corrigé (hors fiche).
  Piège rencontré : la cible placée à f5 n'existe qu'à partir de f6 ; poser un composant « à f5 » ne fait rien.
- **Piège d'outillage** : un `git worktree` de `main` compilé avec le même `CARGO_TARGET_DIR` écrase les artefacts des crates du workspace (le hash ne dépend pas du chemin) ; `find crates games -name '*.rs' -exec touch {} +` après coup.
- Question pour William : le pool de récompense d'`isaac` (poids : clé 3, blank 2, ration 2, sou 4, cœur 3, cœur d'âme 1, bombe 2) est un premier jet ; équilibrage à M3-T11d.

## 7. Atteignabilité

Récompense de salle `combat` d'`isaac` (`loot/salle.ron` : sous, cœurs, bombes, clés) et mort d'ennemi d'`isaac` (`shells_drop`) ; scénario d'une partie ordinaire : `isaac_floor_seed_1` (le bot `explorateur` bat l'étage, victoire, avec `hit_iframes`). Boutique : un présentoir refuse un consommable plein
(`shop_full`) ; la grammaire d'étage d'`isaac` n'a pas de boutique (tables de T11d).

## 8. Frames à regarder

`soul_heart_absorbs` : f5 ramassage, f65 volée (100 → 96), f155 (88). `heart_full_stays` : le cœur reste jusqu'au premier coup (f65), santé 96 à f110. `coin_pickup` : f10 coffre, tas de 7 sous, solde 507. `bomb_pickup_cap` : f5–f30 au plafond, f30 pose, f40 ramassée, explosion f50.
`coop_pickups_per_player` : f5 clés/bombe du milieu (P0 à 3), f31 seconde bombe prise par P1, f50 coffre de P1 (507 contre 500). `hit_iframes_isaac` : f7 premier coup, ~f125 et ~f250 deux coups presque simultanés, un seul passe.
