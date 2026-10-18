# dette-d88-tirage-variantes — D88 : tirage des variantes indépendant (b1)

Branche `dette-d88-tirage-variantes`, depuis `origin/main` f18f5945. Conventions : `docs/conventions.md` §25 (tirage), §72 (chance par étage), §76.3 (isaac).

## 1. Correctif

`crates/game/src/character/variant.rs`, `variant_seed(run_seed, net_id)` :

```
hash = fnv1a("variants:<run_seed>:<net_id>")      // idiome de loot_seed (§48) et curse_seed (§64)
seed = fmix32((hash ^ hash >> 32) as u32)          // repli 32 bits, puis finaliseur MurmurHash3
```

puis le `RollbackRng` local et le tirage existant, inchangés (`chance_at`, table pondérée, deux tirages : chance, puis poids).

**Écart à la demande, mesuré** : le repli FNV seul (l'idiome exact de `loot_seed`) ne suffit pas. Le dernier octet haché n'entre que par un produit avec un nombre premier à faible poids : sur 20 000 graines × net ids 100 à 115, la corrélation de Pearson entre net ids voisins montait à **0,069** (net ids 101/102 ; 0,047 et 0,056 ailleurs, toujours positive), contre un bruit d'échantillon de 0,007. Le test statistique ci-dessous l'a refusé (seuil 0,03). Un `fmix32` après le repli la ramène à zéro. `loot_seed` et `curse_seed` gardent le repli seul : **même exposition**, mesurée en §6 (dette D95).

### Tests (`variant::tests`, graines fixes, aucun hasard)

- `tirage_independant_par_graine_et_net_id` : graines 1 à 20 000 × 16 net ids (100 à 115), chance 0,1, table à une entrée.
  - graines sans aucun champion : **18,4 %** (attendu 0,9^16 = 18,5 % ; avant : 89 %) ; borne du test [15 %, 25 %] ;
  - chance mesurée par net id dans [0,08 ; 0,12] ;
  - corrélation de Pearson entre net ids voisins, 15 paires : de **−0,0129 à +0,0087** ; borne du test |r| < 0,03 (avec le repli seul : 0,069, le test échoue).
- `graines_voisines_tirages_differents` : graines 123 456 à 123 656, chance 0,5, table à deux entrées, 32 net ids : le vecteur de tirages de la graine `s` diffère toujours de celui de `s + 1`.

Les autres tests de `variant` (même graine ⇒ même tirage, chance 0 et 1, poids nul, variante imposée) passent sans changement.

## 2. Traces

### 2.1 Les 46 traces rebénies (liste complète, via `ALACOD_SCENARIO=<liste> ALACOD_BLESS=1`)

Seules celles-ci ont échoué sur la suite sans BLESS (première passe : 46 « trace différente »). Les 579 autres traces de la suite (625 scénarios) n'ont pas bougé.

- **isaac (24)** : `curse_bigger_floor`, `curse_blind`, `curse_hides_health`, `curse_hides_minimap`, `devil_deal_isaac`, `devil_room_absent`, `devil_room_chance`, `generated/isaac/boss_gardien`, `generated/isaac/room_combat`, `grid_floor_seed_1`, `grid_floor_seed_123`, `grid_floor_seed_23`, `grid_post_boss`, `grid_secret_bomb`, `grid_secret_no_bomb`, `grid_secret_room`, `grid_treasure_floor0`, `hit_iframes_isaac`, `isaac_banner_blind`, `isaac_boots`, `isaac_floor_seed_1`, `isaac_hud_items`, `isaac_reliquaire`, `isaac_synergy_garde`.
- **gungeon (10)** : `bench_floor`, `generated/gungeon/enemy_fuyard_kin_moving`, `generated/gungeon/enemy_fuyard_kin_still`, `generated/gungeon/enemy_veteran_kin_moving`, `generated/gungeon/enemy_veteran_kin_still`, `generated/gungeon/room_arene`, `gungeon_classes`, `gungeon_full_run`, `gungeon_full_run_four`, `minimap_shop_visit`.
- **throne (9)** : `throne_duo_defaite`, `throne_floor_1`, `throne_mutation_choice`, `throne_progression`, `throne_quad`, `throne_rodeur_vue_graine_4`, `throne_softlock_recul`, `throne_solo`, `throne_three_floors`.
- **testbed (3)** : `champion_chance_floor`, `champion_chance_floor_0`, `variant_draw` (les deux premiers et `variant_draw` ont aussi changé de graine de carte, §3).

### 2.2 Preuve §5 par jeu (dumps `ALACOD_DUMP_TRACE`, `main` f18f5945 contre branche, `scripts/trace-diff.py` + comparaison composant par composant)

| jeu | scénario | première frame qui diffère | première apparition d'un ennemi à variantes | ce qui diffère à cette frame |
|---|---|---|---|---|
| gungeon | `generated/gungeon/enemy_fuyard_kin_still` | 1 | 1 | `fuyard_kin` (net id 26) : `Variant` (aucune → `rapide`), `Modifiers` (`EnemyMoveSpeed` ×1,5, source `variant:rapide`), `Velocity`, `FixedTransform3D`, `CharacterAppearance` (teinte) : tout dérive de la variante tirée |
| throne | `throne_solo` | 0 | 0 | `chien` (net id 84) : `Variant`, `Modifiers`, `Health` (variante `blinde` : `MaxHealth` ×2) |
| testbed | `champion_chance_floor_0` (dump pris avant le changement de graine, §3) | 1 | 1 | sept `elite_chance` (net ids 14, 15, 22, 24, 25, 42, 50) : `Variant` seul ; 2 champions avant, 5 après |
| isaac | `generated/isaac/room_combat` | 0 | 0 | `tourelle_croix` 121 et 143, `mouche` 131 : `Variant` seul |

Cause unique = le tirage : le composant `Variant` (et ce qu'il entraîne) est la seule différence à la première frame, partout, et la première frame qui diffère est celle de la première apparition d'un ennemi à variantes.

**Traces neutres, sans champion avant ni après** : `generated/gungeon/enemy_shotgun_kin_still` (`shotgun_kin` a des variantes) et `generated/throne/enemy_rat_still` (`rat` a des variantes) : « identique (599 frames comparées) » sur dump complet. Les 579 traces non listées ne bougent pas non plus (variantes à checksum neutre ; suite verte sans BLESS hors ces 46).

## 3. Attentes recalées (sans affaiblissement)

Les attentes gardent leur type et leur sens ; les frames sont re-mesurées (`ALACOD_EVENTS=1`). Chaque `.ron` modifié porte une ligne `D88 (2026-10-17, dette-d88-tirage-variantes)` en en-tête.

**testbed**
- `champion_chance_floor_0` / `champion_chance_floor` : sur l'ancienne graine 1942 (choisie pour contourner la corrélation) le nouveau tirage donnait **5 champions sur 40 à l'étage 0 (10 %) et 8 sur 40 à l'étage 1 (28 %)** : la différence tenait encore (8 > 5), mais de justesse (3 d'écart, pour 11,2 ± 2,8 attendus). Graine changée en **110** (balayage 1 à 2 999, graine la plus petite qui donne 4 et 11, soit 10 % et 27,5 %) : le contraste 10 % / 28 % de la chance par étage se lit directement. Les 40 + 40 `EnemyVariant` sont réécrites par net id (4 et 11 champions). Le modèle Python du tirage utilisé pour choisir la graine reproduit exactement les 5 et 8 de la graine 1942 et les indices du dump.
- `variant_draw` : le grunt tiré (net id 30) donnait `rapide` avec la graine 777 (le scénario affirme `blinde`, distinct du `rapide` imposé au net id 33). Graine **799** (premier `blinde` après 777) ; `variant_fast` / `variant_none` / `variant_elite` (graine par défaut 123456) tirent toujours « aucune variante » pour le net id 30 : traces inchangées.

**isaac**
- `grid_floor_seed_123`, `grid_floor_seed_23` : le bot est plus lent (davantage de champions) ; `frames` 1950 → 2100 et 2530 → 2750, frames du boss, de la victoire et de `PlayerAlive` re-mesurées.
- `grid_floor_seed_1`, `isaac_floor_seed_1` : **adaptation** : le bot `explorateur` arrivait au boss à 1 PV sur 6 et mourait (défaite f2111) ; re-chronométrer ne suffisait pas. Le joueur reçoit `modifiers: [(stat: Armor, op: Add, value: "50")]` (`PlayerScript::modifiers`), dit en en-tête ; mêmes attentes (victoire, boss vaincu, joueur vivant). Un `MaxHealth` ajouté a été écarté : il ne remonte pas la santé courante. `isaac_floor_seed_1` : le bot meurt à l'étage 2 à f3475 (f3039 avant), `frames` 3000 → 3400. Les graines de carte sont inchangées (le nom des scénarios est leur graine).

**throne** (aucun changement de graine ni de sens) : `throne_progression`, `throne_solo`, `throne_quad`, `throne_floor_1`, `throne_three_floors`, `throne_softlock_recul`, `throne_duo_defaite` : frames d'étage, de niveau, d'alerte, de mutation re-mesurées. Écarts de sens, dits en en-tête : `throne_three_floors` : mutations du joueur 0 `[sang_froid, irradie]` (avant `[adrenaline, rancune]`, toujours deux) ; `throne_duo_defaite` : la défaite et l'ordre (joueur 1 à terre, joueur 0 mort) tiennent, plus tard (f972 contre f579) ; `throne_softlock_recul` : `frames` 2000 → 2200 (plus lent, ni mort ni soft-lock).

**gungeon** : `gungeon_full_run` (`frames` 7700 → 7900, toutes les attentes re-mesurées : victoire f7730 contre f7545) et `gungeon_full_run_four` (victoire f6455 contre f6876). Deux écarts de sens, dits en en-tête : `rancune` est maintenant ramassée par le **joueur 1** (`HasItem` du handle 0 → 1) ; l'événement du coffre `arc` est devenu un `EventAfter(after_frame: 3000)` (un autre coffre `arc` s'ouvre dès f1630), plus strict. Dans `gungeon_full_run_four` deux `BossPhase` sont relus avant la mort du boss (« entité absente » à la frame d'origine).

**Tests de crate** (`crates/scenario/tests/expectations.rs`, 3 échecs de `make test_crates` sur la première passe) :
- `hud_text_sur_throne_progression` : `Niv. 1` à f499 (f515), `Niv. 2` à f1880 (f1890) ;
- `throne_deferred_choice_trace_starts_at_the_earned_level` : première divergence 505 → 489 (premier niveau gagné plus tôt) ;
- `throne_recorded_pillard_leaves_the_wall_corner` : monde enregistré rejoué avec des variantes qui sortent maintenant (drops, net ids de plus : le pillard 221 n'existait plus à la frame 2325). Le crochet qui retire déjà la vue directe (D66) retire aussi `variants` des fiches : l'attente porte sur la navigation du pillard, pas sur le tirage.

## 4. Atteignabilité (`alacod-sim`, graine par défaut comprise)

`alacod-sim` écrit maintenant, par graine, la colonne `variantes` (moments clés `variant_spawn`, champ JSON `variants`) et une ligne de synthèse (« variantes rencontrées : n/N graine(s) »).

- **isaac** `--game isaac --bots 1 --profiles explorateur --seeds 1..50 --floors sous_sol --until-floor 2 --max-frames 12000` (1 141 s) : variantes sur **47 graines sur 50 (94 %)**, 259 au total, 11 au plus sur une graine ; **absentes** des graines 3, 29 et 42 ; réparties 0 : 3 graines, 1 : 2, 2 : 6, 3 : 2, 4 : 11, 5 : 4, 6 : 7, 7 : 3, 8 : 3, 9 : 4, 10 : 3, 11 : 2 (une partie courte ou une mort précoce en voit moins). Au-dessus des ~80 % attendus. **Graine par défaut 123456** : 8 variantes (avant : aucune au premier étage). Issues : 10 victoires, 37 morts, 3 soft-locks (graines 9, 42, 43) contre 17 victoires, 30 morts, 3 soft-locks avant D88 (rapport m3-t11c) : les champions pèsent sur le bot (rééquilibrage : T11d).
- **gungeon** `--seeds 1..20` : `--floors etages_court_deux --until-floor 1 --max-frames 20000` (carte courte, ~700 frames) : variantes sur **11 graines sur 20 (55 %)**, 13 au total, 2 au plus sur une graine ; `--floors etages_deux --until-floor 2` (la run complète, 453 s) : **20 graines sur 20**, 63 au total, 6 au plus ; 11 victoires, 9 morts.

## 5. Vérifié (résultats réels)

- `cargo fmt --all -- --check` : propre.
- `make test_scenarios_par SHARDS=3` **sans BLESS** (après le bless des seules 46 traces listées) : 625 scénarios verts, 297 s.
- `make test_crates` : 1 174 réussis, 0 échec, 14 ignorés (première passe : 1 171 réussis, 3 échecs, corrigés §3).
- `make lint` ×5 : 0 erreur (isaac 25 avertissements, comme avant). `make gen GAME=…` ×5 : zombies 16, testbed 98, throne 39, gungeon 59, isaac 95 scénarios, tous « ok ok », code 0.
- `make check_sim_boundary` : 11 crates respectent la frontière, 5 en dette D69 (inchangé). `check_forbidden` et `check_rollback_registration` : OK.
- Fuzz `ALACOD_FUZZ=0:8` (3 600 frames par graine) : aucune divergence ; `ALACOD_FUZZ=0:4 ALACOD_FUZZ_SCENARIO=isaac_floor_seed_1` (isaac, avec champions) : aucune divergence.
- `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (`MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`) : **1 533 frames resimulées** (client 0), 0 desync, 2 999 lignes identiques. Une première exécution a donné 211 resimulées (< 1 000, refusée par le script) : machine chargée (charge 20 à 30), le planificateur laisse moins d'écart entre les deux clients ; rejouée telle quelle, 1 533.

## 6. Dettes et remarques

- D88 fermée (`docs/taches/dettes.md`).
- **D95 ouverte** : `loot_seed` / `curse_seed` ont la même exposition (leurs clés finissent par un nombre qui varie : la première version de ce rapport disait à tort le contraire). Mesure (réplique Python bit à bit, même statistique que le test de `variant` : 20 000 graines × 16 clés voisines, chance 0,1) : `champion:<n>` max **0,053** (n = 100) et **0,063** (n = 1000) ; `shells:<n>` **0,055** (n = 100) et **0,058** (n = 37) ; `rock:3:10:<y>` **0,046**, `rock:3:<x>:5` 0,017 ; `curse:<graine>:<étage>` (6 étages) **0,039**. Seuil 0,03 dépassé : D95 « appliquer `fmix32` à `loot_seed`/`curse_seed` » (coût : traces de butin), hors périmètre ici.
- Équilibrage : les champions d'isaac font chuter l'`explorateur` (10 victoires sur 50 contre 17) ; les deux scénarios `grid_floor_seed_1` et `isaac_floor_seed_1` reposent sur `Armor +50` du joueur (dette **D93** ouverte : rééquilibrer en T11d, puis retirer l'armure).

## 7. Relecture indépendante

Verdict « mergeable », aucun P0/P1. P2 corrigé : `conventions.md` §72 disait que les scénarios de chance gardaient la graine 1942 (elle est 110). P2 suivi : `Armor +50` devient la dette numérotée D93 (critère de sortie : retirer l'armure). P3 corrigés : `hud_text_sur_throne_progression` « Étage 3 » à f3096 (et non plus 3490), commentaire périmé de `gungeon_full_run` (f3892). Notes sans correctif : `graines_voisines_tirages_differents` ne détecte pas à lui seul l'ancien défaut (le détecteur est le test statistique) ; la marge de `PlayerAlive` d'`isaac_floor_seed_1` (f3399 contre une mort à f3475) est de même nature que l'originale.

## 8. Intégration de `origin/main` 803cc9f0 (suivis Q13/Q17, T10) — seconde livraison

**Conflits (7)** : `docs/taches.md` (deux lignes de journal gardées), `docs/taches/dettes.md` (D88 fermée gardée, D89 à D92 de main gardées, ma dette « Armor +50 » renumérotée **D93**, D95 ajoutée ; D94 laissé à `dette-d86`), `grid_floor_seed_1.ron`, `grid_floor_seed_123.ron`, `isaac_floor_seed_1.ron` (les épinglages `aim: Some(Free)` de main **et** mon `Armor +50` : `(bot: explorateur, aim: Some(Free), modifiers: [(stat: Armor, op: Add, value: "50")])` ; `grid_floor_seed_123` garde la note Q17 de main et mes frames recalées), `generated/isaac/boss_gardien.trace` et `room_combat.trace` (rebénies sur le résultat fusionné). Les en-têtes de `grid_floor_seed_1` et `isaac_floor_seed_1` disent « gagne avec `Armor +50` (D93) ».

**Rejeu après fusion** (`make test_scenarios_par SHARDS=3`, sans BLESS) : sur les 46 traces déjà rebénies, seules **deux** bougent (les générés isaac `boss_gardien` et `room_combat`, dont main a changé les scénarios : épinglage `aim`, d'où une trace différente dès la frame 5), plus **deux traces de T10** qui n'existaient pas sur ma branche :
- `achievement_not_met` : cause = tirage des variantes (même partie isaac, graine 8) ; trace seule.
- `achievement_unlocks` : cause = tirage ; **en plus** le bot meurt à f1376 (défaite) avec les champions : la victoire, les accomplissements `gardien_vaincu` / `run_finie` et le HUD `achievements` n'arrivaient plus. Adapté comme `grid_floor_seed_1` : `Armor +50` (D93), mêmes attentes, aucune retirée ; gardien en phase 1 à f1110, phase 2 à f1270, victoire f1960 (avant : ~f2110) ; `frames` 2190 inchangé.
Total rebéni après fusion : **48** traces (46 + ces deux de T10), les 4 ci-dessus bougent par rapport à main + ma première livraison. Les autres scénarios de Q13/Q17/T10 (`aim_four_way*`, `familiar_mirror_*`, `status_charm*`, `transformation_*`…) n'ont aucune trace différente : aucun n'a d'ennemi à variantes qui tire un champion.

**Preuve §5 refaite sur l'arbre fusionné** (dumps `ALACOD_DUMP_TRACE` ; « avant » = le même arbre avec l'ancienne `variant_seed`, pour isoler le tirage ; `scripts/trace-diff.py` et comparaison composant par composant) :

| jeu | scénario | 1re frame qui diffère | ce qui diffère |
|---|---|---|---|
| gungeon | `generated/gungeon/enemy_fuyard_kin_still` | 1 (= 1re apparition) | `fuyard_kin` 26 : `Variant`, `Modifiers`, `Velocity`, `FixedTransform3D`, teinte (tout dérive de la variante) |
| throne | `throne_solo` | 0 (= 1re apparition) | `chien` 84 : `Variant`, `Modifiers`, `Health` |
| testbed | `champion_chance_floor_0` (graine 110) | 1 (= 1re apparition) | quatre `elite_chance` (28, 33, 36, 51) : `Variant` seul |
| isaac | `generated/isaac/room_combat`, `achievement_not_met` | 0 (= 1re apparition) | `tourelle_croix` 121 et 143, `mouche` 131 : `Variant` seul |

Témoins à variantes sans champion avant ni après, dump complet : `generated/gungeon/enemy_shotgun_kin_still`, `generated/throne/enemy_rat_still`, `generated/isaac/enemy_mouche_still`, `generated/isaac/enemy_suiveur_still` : « identique (599 frames comparées) ».

## 9. Points de la revue
- **Phrase fausse corrigée** (§1) : les clés de `loot_seed` / `curse_seed` finissent par un nombre qui varie ; mesure §6 → **D95** ouverte (|r| 0,039 à 0,063 > 0,03).
- **Pillard** : `throne_recorded_pillard_leaves_the_wall_corner` vérifie maintenant que l'acteur au net id 221 est à (575,96 ; 65) à f2325 (±0,05) avant l'attente de navigation.
- **`variant_seed` sans allocation** : `bevy_fixed::rng::{fnv1a_extend, fnv1a_extend_decimal, FNV1A_OFFSET}` (`fnv1a` s'appuie dessus) ; mêmes octets que la clé `format!` : test `graine_sans_allocation_egale_a_la_cle_textuelle` (graines et net ids aux bornes, dont `u64::MAX`). Toutes les traces identiques à la version avec `format!`.

## 10. Résultats après fusion (réels)
- `make test_scenarios_par SHARDS=3` sans BLESS sur l'arbre fusionné : seuls échecs `achievement_not_met`, `achievement_unlocks`, `generated/isaac/boss_gardien`, `generated/isaac/room_combat` (§8) ; rebénis, rejoués avec `grid_floor_seed_1/123`, `isaac_floor_seed_1` : verts.
- `make test_crates` : 1 206 réussis, **1 échec qui n'est pas de D88** : `achievements::les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante` (`emplacement 1 : reservoir_vital n'est pas un objet débloqué`). Même échec, même message, sur `origin/main` 803cc9f0 **intact** (worktree propre), avec l'ancienne `variant_seed` sans `Armor`, et avec la nouvelle : cause probable (non corrigée, hors périmètre) : `draw_exhausting` tire `pick % total` ; débloquer `medaille` fait passer le poids total du pool `boss` de 10 à 12 et remappe les autres emplacements, alors que le test suppose que seul l'objet débloqué change. À traiter par T10.
- `make lint` ×5 : 0 erreur ; `make gen` ×5 : zombies 16, testbed 102, throne 39, gungeon 59, isaac 95, tous ok ; `cargo fmt --all -- --check` propre ; `check_sim_boundary`, `check_forbidden`, `check_rollback_registration` OK.
- `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` : **1 878 frames resimulées**, 0 desync, 2 999 lignes identiques. (Un premier lancement a échoué à la compilation sur `fnv1a_extend` introuvable : empreinte `cargo` périmée de `bevy_fixed` en mode dylib ; `touch crates/bevy_fixed/src/rng.rs` l'a résolu, sans changement de source.)
