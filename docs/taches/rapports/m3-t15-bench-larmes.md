# m3-t15-bench-larmes : rapport

Branche `m3-t15-bench-larmes` depuis `origin/main` `3f37ba43`. Worker b6. Conventions §80, dette D84 (reste ouverte), D97, D98.
Poste : **VPS Debian 8 cœurs, partagé** (un autre worker, b2, joue la même suite sur la même machine), build à froid
(`CARGO_BUILD_JOBS=4`, 45 min), profil `headless`, suite complète en un processus. **Cloud/partage : la mesure de référence n'a pas pu
être faite à `uptime` < 1,5 en continu** (voir §2) ; le dire est la règle, la pire valeur de trois relevés les moins chargés fait foi.

## 1. Fait

- Quatre scénarios neufs (`tests/scenarios/bench_*.ron` + `.trace`, traces de scénarios neufs seulement) :
  `bench_tears_trio` (1 joueur, `division` + `rebond` + `guide`, 30 `target`), `bench_tears` (4 joueurs × 10 objets de larmes dont
  `division` ×2, `guide`, `rebond` ×2, `perforant`, `spectre`, `venin`, `lentille`, `trefle`, + 2 familiers `essaim`, 30 cibles, cadence de la `larme`
  à 20 coups/s : **432 balles au pic**, fiche : ≥ 400), `bench_familiars` (4 joueurs × 2 familiers), `bench_floor_isaac` (`sous_sol_grand`,
  12-15 salles, 2 bots `explorateur`, 6 champions `explosif` posés). Aucune trace existante ne change (suite sans `BLESS`, §5).
- `crates/scenario/src/runner.rs` : 6 lignes, `ALACOD_SYSTEM_PROFILE=1` imprime `profil sets : …` dans le runner (le profil par set n'existait
  que dans le jeu). Diagnostic seul, absent sans la variable.
- `tests/budgets.ron` : **budgets des 29 scénarios existants remis à leur valeur de `main`** (revue) ; en-tête et 4 entrées **provisoires** pour les scénarios neufs (poste, date, charge, « à recalibrer au calme, D84 »).
- `docs/conventions.md` §80, `docs/etat-engine.md` (mesures VPS), `CLAUDE.md` (une phrase), `docs/taches/dettes.md` (D84 ouverte avec son protocole de sortie, D97, D98), `docs/taches.md` §10.

## 2. Relevés : machine et charge

`make test_scenarios` avec `ALACOD_BENCH_STRICT=1` (`make bench` = ce test puis `scenario-metrics.py`), un processus, ~40 à 52 minutes.
Le processus de bench pèse lui-même ~1,1 de charge ; b2 en jouait un autre en parallèle à plusieurs moments.

| relevé | début | `uptime` 1 min au départ → à la fin | usage |
|---|---|---|---|
| r1 | 2026-10-17 23:06 | 2,73 → 2,59 | chargé (mes builds juste avant) : vérifie la tenue sous charge |
| r2 | 23:59 | 2,25 → 1,78 | chargé |
| r3 | 2026-10-18 00:44 | 1,78 → 1,28 | indicatif (load 1,3 à 1,8, mon processus compris) |
| r4 | 01:25 | **0,14** → 5,16 (build d'un autre worker en fin de suite) | indicatif, le plus calme au départ |
| r5 | 02:04 | 0,77 → 2,82 | indicatif |
| r6 | 03:53 | 1,60 → 5,24 | indicatif, **sous charge** : exit 0 avec les planchers d'alors (arrondis au plus proche) ; **ne valide pas** un fichier de budgets final (rejoué contre les valeurs arrondies au-dessus : `testbed_arena_idle` 87,2 < 88) |

Écarts : les mêmes scénarios vont jusqu'à 2 fois plus vite en r4 qu'en r1 (`bench_floor` 45 → 77 fps, `idle` 99 → 129). Le gabarit « load < 1,5 »
n'a été tenu qu'au départ de r4 et en fin de r3 ; j'ai gardé les trois relevés les moins chargés (r3, r4, r5) et pris la **pire valeur** par
scénario ; **ces relevés sont chargés (jusqu'à 5,2) : ils sont indicatifs, pas une référence au calme** (`bench_floor` varie de 49,6 à 77,4 fps entre relevés, `testbed_two_rooms_door_idle` de 156 à 298).

## 3. Benchs de larmes, de familiers et d'étage

Mesures (pire de r3 à r5 ; ms par frame simulée ; rollback = restauration + 12 frames, `rollback_ms_worst`) :

| scénario | fps r1..r5 | entités | balles | save | load | simulation | pire rollback | snapshot max | plus lourd | plancher | plafond |
|---|---|---|---|---|---|---|---|---|---|---|---|
| bench_tears | 31 / 32 / 34 / 41 / 40 | 506 | 432 | 4.46 | 3.57 | 4.39 | 56.3 | 218 KiB | Projectile | 17 | 113 |
| bench_tears_trio | 89 / 81 / 97 / 127 / 111 | 86 | 24 | 1.69 | 1.08 | 1.02 | 13.3 | 42 KiB | EnemyAiConfig | 49 | 27 |
| bench_familiars | 67 / 98 / 89 / 133 / 131 | 63 | 17 | 1.79 | 1.07 | 0.97 | 12.7 | 20 KiB | FixedTransform3D | 45 | 26 |
| bench_floor_isaac | 60 / 55 / 57 / 91 / 86 | 225 | 14 | 2.63 | 1.80 | 1.43 | 18.9 | 53 KiB | FixedTransform3D | 29 | 38 |

Planchers : moitié du pire fps ; plafonds : double du pire rollback (règle du fichier). Charge : r3 à r5 ci-dessus.

Profil par set (`ALACOD_SYSTEM_PROFILE=1`, µs par frame de simulation, resimulation du synctest comprise), poste VPS, charge ~2 :

| scénario | premiers sets |
|---|---|
| bench_tears | **Projectiles 2 292**, Weapon 367, CollisionDamage 207, EnemyAI 194, DeathManagement 184, Interaction 180 |
| bench_tears_trio | EnemyAI 146, Projectiles 116, Weapon 85, DeathManagement 80 |
| bench_familiars | Weapon 124, DeathManagement 86, EnemyAI 86, Interaction 60, Effects 52 |
| bench_floor_isaac | FlowField 173, DeathManagement 112, Run 107, Weapon 96, EnemyAI 68 |
| bench_floor (gungeon, pour comparer) | FlowField 406, DeathManagement 151, Run 143, Weapon 127 |

Attribution de `bench_tears` (variantes temporaires de `bench_tears`, supprimées) : sans `division` : 39 balles, simulation 1,46 ms, rollback
18,9 ms ; sans `guide` : 1 080 balles, 6,06 ms, 78,9 ms (le guidage fait toucher plus tôt, il ne coûte pas de population) ; sans `rebond` :
inchangé (432 balles, 53 ms) ; sans aucun objet : 97 balles, 15,9 ms. `Projectiles` est linéaire en balles (~5 µs par balle et par frame),
pas quadratique : `Homing` n'est pas en cause. La population vient de `Split` (deux exemplaires de `division`, génération maximale 8).
**Le pire rollback de `bench_tears` (56 ms) dépasse 3 ticks de 16,7 ms** : dette **D97** chiffrée, seuil non relevé pour passer (le plafond 113 est le
double de la mesure, règle du fichier). `bench_floor_isaac` (18,9 ms) et `bench_familiars` (12,7 ms) tiennent dans ~1 tick. Familiers : coût propre
faible (`bench_familiars` : simulation 0,97 ms contre 0,66 pour `effects_hundred_items` à 1 joueur ; `familiar_quad` 0,6 sur 300 frames).
Champ de flux : `FlowField` 173 µs par frame sur l'étage isaac, 406 sur `bench_floor` de gungeon (plus de salles verrouillées) : D70 ne mord plus.

## 4. Comparaison avec les mesures du 2026-10-15 (`etat-engine.md`, poste M4)

| scénario | M4 2026-10-15 : save / load / sim / rollback / snapshot | VPS (pire r3-5) : save / load / sim / rollback / snapshot | rollback VPS/M4 |
|---|---|---|---|
| bench_bullets_room | 0.6 / 0.46 / 1.05 / 13 / 198 KiB | 3.02 / 2.16 / 2.21 / 28.7 / 223 | ×2.2 |
| bench_horde | 0.46 / 0.45 / 2.91 / 35.3 / 88 KiB | 3.63 / 2.68 / 3.66 / 46.6 / 88 | ×1.3 |
| four_players_idle | 0.2 / 0.17 / 0.11 / 1.5 / 29 KiB | 1.53 / 1.05 / 0.61 / 8.4 / 26 | ×5.6 |
| p2p_smoke | 0.51 / 0.38 / 7.06 / 85.1 / 30 KiB | 1.77 / 1.15 / 1.25 / 16.2 / 27 | ×0.2 |
| bench_bullets | — / — / — / 3.9 / — KiB | 2.18 / 1.64 / 1.10 / 14.9 / 74 | ×3.8 |
| bench_cave | — / — / — / 2.1 / — KiB | 1.54 / 1.02 / 0.72 / 9.6 / 28 | ×4.6 |
| bench_floor | — / — / — / 8.8 / — KiB | 3.63 / 2.77 / 1.78 / 24.1 / 88 | ×2.7 |

Lecture (le poste n'est pas le même, donc **aucune conclusion de régression ne peut être tirée de ces chiffres seuls**, D98) :
- `p2p_smoke` : simulation 7,06 → 1,25 ms et rollback 85,1 → 16,2 ms **sur un poste plus lent** : la dette D70 (champ de flux à chaque case) ne se
  reproduit plus ; je n'ai pas vérifié ici quelle tâche l'a corrigée.
- `bench_horde` ×1,3, `bench_bullets_room` ×2,2 : le rapport attendu entre le M4 et un cœur de VPS.
- `bench_cave` ×4,6, `bench_bullets` ×3,8, `four_players_idle` ×5,6 : plus haut ; la sauvegarde (`save`) y est 7 à 8 fois celle du M4 alors que la
  simulation légère n'en coûte que 5 à 6 : sur ce poste le snapshot pèse autant que la simulation en scénario léger. [INFERENCE] différence de mémoire/cache
  du poste, pas un défaut localisé : aucun système ne ressort au profil par set ; à confirmer en rejouant sur le M4 (D98).
- Fait établi par les relevés : les **anciennes mesures** documentées dans `budgets.ron` (ex. `testbed_*` du 2026-09-29 : 214 à 282 fps) valent ici
  0,56 à 0,77 fois leur ancienne valeur (164,8 / 214 = 0,77 ; 156,1 / 281 = 0,56), régulièrement : effet de poste, pas d'un scénario. Les anciens
  **planchers** étaient eux inégaux : 29 sur 29 au-dessus de la moitié de la mesure de ce poste, mais certains tenaient quand même au calme
  (`dash_wall` 134,7 fps contre 100, `bench_floor` 49,6 contre 40) ; ils échouaient sous charge, d'où D84.

### D84 : mesures indicatives pour les 27 scénarios (+ `bench_floor`, `bench_horde`) — **budgets non modifiés**

Revue : les relevés chargés ne servent pas de référence ; **les 29 budgets existants sont remis à leur valeur de `main`**. Le tableau ci-dessous est conservé comme **mesure indicative** ; les colonnes « plancher après » / « plafond après » étaient une **proposition de calcul** (⌈moitié de la pire valeur de r3 à r5⌉, double), **non appliquée**. D84 reste ouverte (protocole de sortie dans `dettes.md`).



| scénario | fps r1 | r2 | r3 | r4 | r5 | référence (pire 3-5) | plancher avant | plancher après | rollback pire r3-5 (ms) | plafond avant → après | décision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ammo_burst | 74.8 | 77.2 | 111.6 | 157.4 | 143.5 | 111.6 | 120 | 56 | 9.0 | — | plancher abaissé à la moitié |
| bench_bullets | 49.8 | 60.4 | 89.8 | 90.2 | 79.2 | 79.2 | 80 | 40 | 14.9 | 8 → 30 | plancher abaissé à la moitié ; plafond relevé au double |
| bench_bullets_room | 37.5 | 39.0 | 60.4 | 61.1 | 56.3 | 56.3 | 40 | 29 | 28.7 | 26 → 58 | plancher abaissé à la moitié ; plafond relevé au double |
| bench_cave | 67.2 | 76.4 | 119.1 | 128.5 | 105.7 | 105.7 | 100 | 53 | 9.6 | 5 → 20 | plancher abaissé à la moitié ; plafond relevé au double |
| bench_floor | 45.3 | 46.5 | 49.6 | 77.4 | 70.1 | 49.6 | 40 | 25 | 24.1 | 18 → 49 | plancher abaissé à la moitié ; plafond relevé au double |
| bench_horde | 40.0 | 35.4 | 39.4 | 53.3 | 55.0 | 39.4 | 38 | 20 | 46.6 | 70 → 94 | plancher abaissé à la moitié ; plafond relevé au double |
| bot_explorer_floor | 102.6 | 99.0 | 109.0 | 158.4 | 121.7 | 109.0 | 120 | 55 | 9.9 | — | plancher abaissé à la moitié |
| bot_explorer_two | 104.2 | 92.7 | 116.3 | 142.8 | 114.4 | 114.4 | 100 | 58 | 9.4 | — | plancher abaissé à la moitié |
| bullets_walls | 127.0 | 96.1 | 104.5 | 162.0 | 143.7 | 104.5 | 140 | 53 | 9.2 | — | plancher abaissé à la moitié |
| dash_wall | 146.0 | 112.0 | 137.9 | 178.9 | 134.7 | 134.7 | 100 | 68 | 7.0 | — | plancher abaissé à la moitié |
| four_players_idle | 120.3 | 108.3 | 108.3 | 139.1 | 121.3 | 108.3 | 100 | 55 | 8.4 | — | plancher abaissé à la moitié |
| four_players_shooting | 112.2 | 81.8 | 98.9 | 141.1 | 123.7 | 98.9 | 100 | 50 | 9.9 | — | plancher abaissé à la moitié |
| idle | 99.2 | 126.6 | 154.8 | 128.7 | 118.2 | 118.2 | 110 | 60 | 8.1 | — | plancher abaissé à la moitié |
| movement_melee | 93.5 | 128.3 | 153.2 | 158.7 | 130.4 | 130.4 | 90 | 66 | 7.4 | — | plancher abaissé à la moitié |
| remote_first_fight | 97.4 | 137.1 | 147.0 | 139.6 | 133.8 | 133.8 | 120 | 67 | 7.2 | — | plancher abaissé à la moitié |
| shoot_around | 96.5 | 130.2 | 160.0 | 140.3 | 130.9 | 130.9 | 100 | 66 | 7.4 | — | plancher abaissé à la moitié |
| testbed_ally_safe | 106.2 | 168.9 | 190.2 | 164.8 | 183.9 | 164.8 | 150 | 83 | 6.0 | — | plancher abaissé à la moitié |
| testbed_arena_idle | 110.1 | 168.2 | 180.7 | 174.6 | 177.7 | 174.6 | 150 | 88 | 5.4 | — | plancher abaissé à la moitié |
| testbed_civilian_blocks | 121.2 | 156.3 | 143.9 | 187.1 | 191.9 | 143.9 | 160 | 72 | 7.1 | — | plancher abaissé à la moitié |
| testbed_corridor_idle | 142.0 | 176.4 | 195.0 | 204.1 | 238.5 | 195.0 | 180 | 98 | 4.8 | — | plancher abaissé à la moitié |
| testbed_dummy_shoot | 97.6 | 142.1 | 156.9 | 186.0 | 224.1 | 156.9 | 160 | 79 | 6.6 | — | plancher abaissé à la moitié |
| testbed_follower | 89.9 | 150.6 | 172.4 | 183.7 | 197.8 | 172.4 | 170 | 87 | 5.6 | — | plancher abaissé à la moitié |
| testbed_target_hits | 100.7 | 152.4 | 155.2 | 209.5 | 204.3 | 155.2 | 140 | 78 | 6.6 | — | plancher abaissé à la moitié |
| testbed_two_rooms_door_idle | 132.6 | 134.0 | 156.1 | 298.1 | 224.0 | 156.1 | 180 | 79 | 6.3 | — | plancher abaissé à la moitié |
| testbed_window | 114.2 | 156.9 | 176.2 | 234.1 | 220.4 | 176.2 | 160 | 89 | 5.3 | — | plancher abaissé à la moitié |
| two_players_idle | 92.0 | 128.2 | 122.2 | 130.6 | 111.0 | 111.0 | 80 | 56 | 9.3 | — | plancher abaissé à la moitié |
| two_players_shooting | 113.9 | 132.5 | 140.8 | 150.0 | 131.1 | 131.1 | 130 | 66 | 7.0 | — | plancher abaissé à la moitié |
| weapons_workout | 93.5 | 119.7 | 152.7 | 112.4 | 130.9 | 112.4 | 100 | 57 | 8.9 | — | plancher abaissé à la moitié |
| window_repair | 95.9 | 126.8 | 153.3 | 129.5 | 139.1 | 129.5 | 100 | 65 | 7.7 | — | plancher abaissé à la moitié |

Remarques : tous les planchers « avant » étaient **au-dessus de la moitié** de la mesure de ce poste (ils venaient du Mac M4 ou du cloud 4 cœurs), d'où
des abaissements uniformes ; `bench_horde` : plafond 70 → 94 (double de 46,6), c'est la seule remontée de plafond qui n'était pas cassée au calme.
Régression réelle nommée : **aucune** pour D84 (aucune mesure de régression établie, mais aucun budget confirmé au calme) ; D97 (coût de `bench_tears`) et D98 (comparaison M4 à confirmer) sont ouvertes.

## 5. Vérifié (résultats réels)

Fusion avec `origin/main` (`4630291a`, D88 tirage des variantes, D95/D96 butin, D69 combat en SIM) faite avant le commit `LIVRÉ` ; dettes
renumérotées **D97/D98** (D89 à D96 existaient sur `main`). Seule trace qui bouge sur la fusion : **`bench_floor_isaac.trace`, scénario neuf de cette
branche** (le tirage des variantes et du butin de `main` change l'étage de la graine 1) ; rebénie, aucune trace de `main` touchée.

| commande | résultat réel |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans `BLESS`), sur la branche seule | 3 tranches `ok`, 631 scénarios fusionnés, 1 061 s |
| idem sur la fusion avec `main` (après le bless de `bench_floor_isaac`) | 3 tranches `ok`, **646 scénarios**, 1 146 s, **aucune trace différente** |
| `cargo fmt --all -- --check` | rustfmt a demandé un reformatage de mes 6 lignes du runner (appliqué) ; ensuite sans sortie, code 0 |
| `make test_tools` | code 0 |
| `make test_crates` (sur la fusion) | **1 214 tests, 0 échec**. Lancé en morceaux (hors `scenario`, puis chaque `--test` de `scenario`, binaires effacés entre deux) parce que le **disque partagé** a été plein (0 octet libre, `ld` : Bus error) pendant la commande d'un bloc ; mêmes commandes et mêmes filtres que `make test_crates` (`--exact --skip scenarios`). Sur la branche seule, `achievements` échouait (« reservoir_vital n'est pas un objet débloqué ») : c'est D96 de `main`, corrigée par la fusion ; il passe sur la fusion |
| `make bench` strict (`ALACOD_BENCH_STRICT=1`) | **aucun passage d'un seul tenant au calme.** r6 (branche, charge 1,6 → 5,2) : exit 0 avec les anciens planchers calculés (`testbed_arena_idle` 87,2 < 88 avec les finaux). r7 (fusion, charge 3 → 14) : `p2p_smoke` 36,5 et `item_calice_ardent` 37,3 fps contre le plancher par défaut 40 ; rejoués seuls à charge 5 à 7, dans leurs budgets. Les budgets actuels sont ceux de `main` + 4 provisoires. |
| budgets proposés contre les relevés | indicatif seulement : les relevés r1 à r5 respectaient les valeurs proposées (non appliquées) |


## 6. Non fait / incertain

- La mesure de référence n'a pas été faite à `uptime` < 1,5 **continu** : voir §2 ; relevés r3 à r5 retenus, valeur la pire.
- Pas de `make perf_review` contre une référence copiée avant : il n'existe aucune référence d'avant sur ce poste et les scénarios sont neufs ; `perf-review`
  contre r3 (1 dixième près) n'aurait comparé que le bruit du poste. À faire sur un second poste (D98).
- Pas de p2p (la tâche ne touche pas la simulation, une ligne de diagnostic dans le runner exceptée).
- Aucune optimisation du coût des larmes (hors fiche : « mesure seulement ») ; D97 chiffrée.
- Relecture indépendante avant merge : à lancer par l'orchestrateur (non faite par moi).

## 7. Dettes

D84 **reste ouverte** (protocole : trois `make bench` au calme sur un poste nommé, `uptime` < 1,5 tenu). **D97** (coût de `bench_tears`, 56 ms, population de `Split`). **D98** (M4 contre VPS, à rejouer sur un second poste).

## 8. Atteignabilité

Sans objet : mesure ; les scénarios jouent des objets et familiers existants (`division`, `rebond`, `guide`, `essaim`) et l'étage `sous_sol_grand` par le bot `explorateur`.
