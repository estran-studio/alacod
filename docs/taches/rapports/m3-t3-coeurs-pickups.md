# M3-T3 : cœurs, i-frames, sous, bombes et clés au sol

Branche `m3-t3-coeurs-pickups` (depuis `origin/main` a9432ed9, fusionnée avec `0a719425`, puis à la revue avec `64ec961c` et `b2da3508`), worker b1 (réalisation), b6 (revue indépendante et correctifs), 2026-10-16.
Spécification : `docs/conventions.md` §68. Réponses de William appliquées : **Q2** (demi-cœur = 1 PV, `ExtraHealth { soul }`
neutre absorbé avant les PV point par point, cœurs d'âme en présentation), **Q4** (sous, bombes, clés **par joueur**), **Q6**
(`ExtraHealth` lisible par `can_pay`, rien décidé pour T8).

## 1. Fait

1. **Couche de vie** : `combat::actors::ExtraHealth { soul: Fixed }` (rollback neutre), consommable « cœur d'âme » par la nouvelle action
   `GainSoul(Fixed)` (variante en fin d'enum), attente `ExtraHealth(handle, n, at_frame)`. **Ordre retenu des couches, testé** :
   bouclier → armure → `ExtraHealth` → PV, dans la fonction pure `combat::damage::apply_layers` (appelée par
   `rollback_resolve_damage_events`) ; `DamageKind::True` ignore bouclier et armure mais **pas** l'âme ; un coup absorbé par l'âme
   démarre les `hit_iframes`, compte pour `HitLanded`/`HitCount`/crédit de points, mais n'accumule rien aux PV (ni mort, ni `last_hit_by`, ni délai de régénération). **`OnDamageTaken`** (revue) : avec `HealthRegen` il se lit sur la perte de PV réelle (un coup absorbé en entier par l'âme ne le déclenche pas) ; **sans `HealthRegen` — `pleureur` — `effects_runtime` le déclenche sur tout `DamageEvent` visant le porteur** (§25), donc aussi sur un coup absorbé par l'âme. **Règle inchangée, documentation corrigée** (§68).
   Unitaires : `combat::damage::layer_tests` (6) ; dans le vrai résolveur : `crates/scenario/tests/extra_health.rs` (5 : âme point par point,
   ordre bouclier/armure/âme/PV sur trois coups, coup absorbé par l'âme qui démarre les i-frames, coup absorbé par le bouclier qui ne démarre **pas** les `hit_iframes`, dégâts vrais).
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
   `heart_full_stays` (+ `_temoin`), `coop_pickups_per_player` : **8 traces neuves**, bénies une à une. Ajoutés à la revue : `shop_full` (+ `_temoin`), `room_reward_coins`, `champion_drops_coins` (**4 traces neuves**). Mutations jouées à la main (données seules) : sans
   `max_stack` ou sans `refuse_when_full`, `bomb_pickup_cap`, `coop_pickups_per_player` et `heart_full_stays` échouent (4 bombes au lieu de 3 ; santé 92 au lieu de 96).
8. **Lint** : `GainSoul = 0`, `Currency(0)` (pool et déblocage), `Currency` dans un pool de boutique (pool, étage, déblocage), `refuse_when_full` sans `Heal`,
   `starting_consumables` > `max_stack`, et (revue) **`Scaled` à `OnUse`** (consommable ou actif, `Unsupported`) : 8 fixtures (`crates/content/tests/m3_t3_fixtures.rs`).

## 2. Traces bougées (preuve §5, refaite à la revue sur l'état fusionné avec `origin/main` `64ec961c`)

**Onze traces `isaac`** bougent, plus `hit_iframes_isaac` (trace neuve) ; aucune autre. Méthode : dumps détaillés (`ALACOD_DUMP_TRACE`) des onze scénarios sur trois états — (1) `main` (worktree), (2) la
branche **sans** `hit_iframes` de `pleureur` (reste : pool `salle`, code), (3) la branche complète — comparés par `scripts/trace-diff.py` sans `--ignore`. Deux causes, jamais confondues :
`hit_iframes` (2 → 3) : première frame = premier coup reçu, **une ligne** (le joueur), **un champ** : `Health.invulnerable_until_frame` `None` → `Some(f+60)` ; pool `salle` (1 → 2) : première frame = tirage de la
récompense, **une ligne** : l'objet au sol change (même position).

| Trace | 1 → 3 : première frame, cause | autre cause, plus tard |
|---|---|---|
| `grid_floor_seed_1` | f250, `hit_iframes` (→ 310) | pool `salle` f529 (clé → bombe) |
| `grid_floor_seed_123` | f365, `hit_iframes` (→ 425) | aucune (pool seul : identique) |
| `grid_floor_seed_23` | f254, `hit_iframes` (→ 314) | pool `salle` f357 (blank → ration) |
| `grid_post_boss` | f352, **pool `salle`** (clé → bombe) | `hit_iframes` f845 (→ 905) |
| `isaac_floor_seed_1` | f250, `hit_iframes` (→ 310) | pool `salle` f529 (clé → bombe) |
| `generated/isaac/boss_gardien` | f160, `hit_iframes` (→ 220) | pool `salle` f163 (ration → bombe) |
| `generated/isaac/enemy_mouche_moving` | f73, `hit_iframes` (→ 133) | aucune |
| `generated/isaac/enemy_mouche_still` | f202, `hit_iframes` (→ 262) | aucune |
| `generated/isaac/enemy_suiveur_moving` | f77, `hit_iframes` (→ 137) | aucune |
| `generated/isaac/enemy_suiveur_still` | f264, `hit_iframes` (→ 324) | aucune |
| `generated/isaac/room_combat` | f250, `hit_iframes` (→ 310) | aucune (pool seul : identique) |

**Correction de l'ancienne preuve** : elle attribuait tout à `hit_iframes` et écartait le pool (« ne se lit que si une récompense tombe »). Sur la grille d'`isaac` des récompenses tombent : le pool `salle` **fait bouger**
`grid_floor_seed_1`, `grid_floor_seed_23`, `grid_post_boss`, `isaac_floor_seed_1` et `boss_gardien`, et c'est la seule cause de la première divergence de `grid_post_boss`. Ces traces sont bénies pour les deux causes.
`hit_iframes_isaac` : n'existe pas sur `main`, trace neuve (la carte `map_seed: 1` est la grille) ; ses ennemis (20, −8) et (−36, −8) sont toujours à 28 px du joueur (départ (−8, −8), ancre de l'étage inchangée),
les attentes (6, 5, 4, 4, 3 PV) passent sans changement : **rien à recaler**. Identiques : `grid_secret_*`, `grid_treasure_*`, `isaac_boots`, générées `weapon_*`, et tous les autres jeux.

## 3. Vérifié (revue b6)

Preuve §5, p2p et mutation : sur la fusion avec `64ec961c`. Tout le reste ci-dessous : **rejoué sur la fusion finale avec `origin/main` `b2da3508`** (D66 vue directe, traces throne de main, fusion sans conflit ; aucune trace `isaac` n'a bougé du fait de cette seconde fusion).

- `make test_scenarios_par SHARDS=3` **sans BLESS hors `isaac`** : **512 scénarios, trois tranches ok** (1 056 s sur la machine partagée). Bénis (liste exacte) : les **onze** traces `isaac` du § 2 et `hit_iframes_isaac` (neuve) ;
  nouvelles traces : `shop_full`, `shop_full_temoin`, `room_reward_coins`, `champion_drops_coins`. Aucune autre trace (testbed, zombies, gungeon, throne, `grid_secret_*`, `grid_treasure_*`, `isaac_boots`, générées `weapon_*`) n'a bougé.
- `make test_crates` : **1 038 tests, 0 échec**, 14 ignorés (somme des lignes `test result`). `make test_tools` : 18 ok.
- `make lint` ×5 (`zombies`, `testbed`, `throne`, `gungeon`, `isaac`) : aucune erreur. `make gen` ×5 : tous les scénarios générés ok, aucun diff hors des six traces `generated/isaac` du § 2.
- `cargo fmt --all -- --check` : ok. `make check_sim_boundary` : ok (11 crates, 5 en dette D69). `check-forbidden.sh` : ok (38 occurrences auditées). `check-rollback-registration.sh` : ok.
- Fuzz `ALACOD_FUZZ=0:8` (`-- --ignored`) : 8 graines, 3600 frames, aucune divergence (267 s sur la fusion finale) ; `ALACOD_FUZZ_BOMBS=1` : 8 graines, 8 explosions par partie, aucune divergence (314 s).
- `extra_health.rs` : 6 tests ok ; **mutation jouée** : avec `SHIELD_IFRAMES` porté de 15 à 30 (le coup absorbé par le bouclier bloquerait le coup de f30, comme s'il posait les 30 frames de `hit_iframes`), le nouveau test échoue avec `(5000.0, 0.0, 0)` au lieu de `(4992.0, 0.0, 0)` ; rétabli à 15.
  `hit_iframes.rs` : 2 tests ok (le test vacuous est supprimé). `-p content --test m3_t3_fixtures` : 8 ok ; `m3_t0b_fixtures` : 6 ok.
- Docker : `MATCHBOX_PORT=3596`, `COMPOSE_PROJECT_NAME=b6`.

## 4. p2p, deux clients, rollbacks forcés (`scripts/p2p-stress.sh`)

Montage temporaire **non commité** (reconstruit à la revue, annulé par `git checkout`), jeu `testbed` : un système `GgrsSchedule` (`CollisionDamage`, après les traducteurs de dégâts, avant `rollback_resolve_damage_events`) qui, toutes
les 45 frames dès f60 et pour chaque joueur (ordre `GgrsNetId`), pose à ses pieds un `coeur_ame`, un tas de 3 sous et une `bomb` (plafonnée à 3, donc refusée ensuite) et lui inflige 5 dégâts « ennemis » (absorption partielle : 4 d'âme, 1 PV),
avec la trace `ggrs{f=… montage player=<handle> net_id=<id>}` (joueur 0 = net id 26, joueur 1 = net id 31).
Entrées : `scripts/p2p-stress-inputs.py 3000`, `ALACOD_INPUT_DELAY=0`, client 1 suspendu 12 ms toutes les 30 ms, 3000 frames. Les journaux de la première livraison (autre machine) n'existent plus : tout a été rejoué.
- **Run 1** : échec de construction du montage (conflit d'ordre sur `FrameEvents<DamageEvent>` entre mon système et les traducteurs), corrigé par des `.after(...)`.
- **Run 2** : « p2p-stress : OK », codes 0/0, `DesyncDetected` 0, déconnexions 0, **7 907 / 20** frames resimulées (client 0 / client 1), `cmp` : traces **identiques, 2 999 lignes** (= FRAMES-1).
- **Point 6 (absorptions vues au seul client 0)** : 412 exécutions de `soul_absorb` au client 0 pour 99 (frame, cible) distincts, 94 au client 1 pour 94 distincts ; 94 communes. **Cinq** (frame, cible) n'existent qu'au client 0 (f990, f1305, f1710, f1890, f2925 ;
  la première livraison en comptait huit sur un autre run), **toutes sur le net id 31 = joueur 1, aucune sur le joueur 0**, aucune seulement au client 1. L'explication « dash prédit du joueur 1 » n'est pas réfutée : le script d'entrées du joueur 1 tient `Dash`
  à ces frames ou juste avant (f990 : 990-992 ; f1305 : 1303-1306 ; f1710 : 1698-1701 et 1704-1706 ; f1890 : 1887-1890 ; f2925 : 2925-2926). Le mécanisme lui-même (le dash rend invulnérable au client 1, pas au client 0 qui prédit) reste une
  **[INFÉRENCE]** : aucun journal ne l'observe directement.

## 5. Non fait, incertain

- **Pas de bench** (machine partagée, consigne).
- `Price::can_pay`/`pay` ne reçoivent pas `ExtraHealth` : le paiement par cœurs d'âme (Q6) est T8 ; `ExtraHealth::soul` est public, rien d'autre n'a été prévu.
- Aucun plafond de vie d'âme (Isaac : 12 cœurs au total) ; HUD des cœurs : T16.
- `OnDamageTaken` : voir § 1 (règle du §25 inchangée : sans `HealthRegen`, un coup absorbé par l'âme le déclenche) ; une règle unique est à décider si un objet d'`isaac` doit réagir aux seules pertes de PV.
- La pose de bombe d'`isaac` (`BombCarrier`) reste à M3-T11 : les bombes d'`isaac` se ramassent et se comptent, rien ne les pose encore.
- **Sous du rocher à butin : aucun scénario**. Même appel (`items::loot_currency` puis `shop::spawn_shell_pickup`) que la récompense de salle et le champion, observés par `room_reward_coins` et `champion_drops_coins`, mais un essai de scénario
  sur un étage dont le pool est `pieces_salle` et la carte `rocks_static` laissait le joueur immobile (cause non élucidée, hors fiche) : retiré plutôt que livré faux.

## 6. Dettes, questions

- **Relecture indépendante** (agent `reviewer`, lecture seule, première livraison) : 3 constats. (1) `state_trace.rs` « supprimé » : faux positif. (2) `Scaled` en `OnUse` : **corrigé à la revue** (ci-dessous). (3) `Currency(0)` ajouté par un déblocage : corrigé (lint + fixture `unlock_currency_zero`).
- **Dette T3-1 retirée** : `Scaled` à `OnUse` (consommable ou actif) est refusé au lint (`Unsupported`, « Scaled : pas exécuté par OnUse d'un objet »), fixture `scaled_in_onuse` (un consommable et un actif) ; aucun contenu ne l'utilisait
  (les `Scaled` du testbed sont des effets passifs) ; l'ancienne assertion de `m3_t0b_fixtures` sur un `Scaled(Heal)` de consommable est remplacée par la nouvelle fixture. Le sens de `PerCopy` pour un consommable consommé au ramassage reste à décider si un besoin apparaît.
- **Test vacuous remplacé** : `hit_iframes.rs::shield_absorb_uses_its_own_short_iframes_not_the_hit_ones` (bouclier posé à f5, cible présente à f6 seulement) est supprimé, avec le paramètre `shield_at_5` qu'il était seul à utiliser ;
  `extra_health.rs::a_hit_absorbed_by_the_shield_does_not_start_the_hit_iframes` le remplace (bouclier d'un point posé à f6, coups à f12 et f30, 40 frames, attendu `(4992, 0, 0)`), mutation jouée (§ 3).
  Piège rencontré : la cible placée à f5 n'existe qu'à partir de f6 ; poser un composant « à f5 » ne fait rien.
- **Piège d'outillage** : un `git worktree` de `main` compilé avec le même `CARGO_TARGET_DIR` écrase les artefacts des crates du workspace (le hash ne dépend pas du chemin) ; `find crates games -name '*.rs' -exec touch {} +` après coup (fait).
- Question pour William : le pool de récompense d'`isaac` (poids : clé 3, blank 2, ration 2, sou 4, cœur 3, cœur d'âme 1, bombe 2) est un premier jet ; équilibrage à M3-T11d.
- Fusion : `origin/main` a avancé de `89a394ef` à `64ec961c` puis `b2da3508` pendant la revue (m2-full-run, revue-sons-visuels-m1, dette-d66-vue-directe) ; les deux sont fusionnés (le second sans conflit). Conflits : `bomb.ron` (isaac : commentaire de M3-T5 gardé, `max_stack: Some(99)` gardé), `docs/taches.md` (deux lignes), six traces `isaac` (reprises de `main`, puis re-bénies).

## 7. Atteignabilité

Récompense de salle `combat` d'`isaac` (`loot/salle.ron` : sous, cœurs, bombes, clés) et mort d'ennemi d'`isaac` (`shells_drop`) ; scénario d'une partie ordinaire : `isaac_floor_seed_1` (le bot `explorateur` bat l'étage, victoire, avec `hit_iframes`). Boutique : un présentoir refuse un consommable plein
(`shop_full`, scénario `shop_full`) ; la grammaire d'étage d'`isaac` n'a pas de boutique (tables de T11d).

## 8. Frames à regarder

`soul_heart_absorbs` : f5 ramassage, f65 volée (100 → 96), f155 (88). `heart_full_stays` : le cœur reste jusqu'au premier coup (f65), santé 96 à f110. `coin_pickup` : f10 coffre, tas de 7 sous, solde 507. `bomb_pickup_cap` : f5–f30 au plafond, f30 pose, f40 ramassée, explosion f50.
`coop_pickups_per_player` : f5 clés/bombe du milieu (P0 à 3), f31 seconde bombe prise par P1, f50 coffre de P1 (507 contre 500). `hit_iframes_isaac` : f7 premier coup, ~f125 et ~f250 deux coups presque simultanés, un seul passe.
