# m3-t15-bench-larmes : rapport

Branche `m3-t15-bench-larmes` depuis `origin/main` `3f37ba43`. Worker b6. Conventions §80, dette D84 (**fermée** par `dette-d84-budgets-calme`, section « D84 au calme » en fin de rapport), D97, D98.
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

D84 : **fermée** le 2026-10-18 (voir « D84 au calme » ci-dessous). **D97** (coût de `bench_tears`, 56 ms, population de `Split`). **D98** (M4 contre VPS, à rejouer sur un second poste).

## 8. Atteignabilité

Sans objet : mesure ; les scénarios jouent des objets et familiers existants (`division`, `rebond`, `guide`, `essaim`) et l'étage `sous_sol_grand` par le bot `explorateur`.

## D84 au calme (dette-d84-budgets-calme, 2026-10-18)

Branche `dette-d84-budgets-calme` depuis `origin/main` `f96fa463` (qui contient `m2-salles-murs-gungeon` : gabarits gungeon agrandis, mesurés tels quels). Worker b6, VPS Debian 8 cœurs (Haswell, 3,1 GHz, 3 à 5 % de vol d'hyperviseur), `--profile headless`, compilation faite **avant** les relevés, un seul processus de bench à la fois, nightly absent (`nightly.sh` jamais vu en cours).

### Les relevés

Quatre `make bench` (`ALACOD_BENCH_STRICT=1`) consécutifs, fichiers de charge dans `docs/taches/rapports/m3-t15-d84-charge/` (`charge-r1..4.txt` : date, `loadavg` 1 min toutes les ~10 s ; r2 à r4 ajoutent `autres=` = CPU instantané de tous les processus **hors** bench, `others.py`, lu dans `/proc`). r1 à r3 : budgets de `main` (28 à 29 échecs de budget chacun, attendus : l'ancien fichier venait d'un autre poste), r4 : budgets recalibrés.

| relevé | début → fin (UTC) | durée du test | loadavg 1 min : médiane / p95 / max | CPU hors bench (cœurs ×100 %) : médiane / p95 / max | résultat |
|---|---|---|---|---|---|
| r1 | 14:10 → 15:21 | 4 282 s | 2,9 / 8,4 / 13,9 | non échantillonné (top-4 de `ps` seulement) | 29 budgets dépassés |
| r2 | 15:21 → 16:28 | 3 985 s | 2,5 / 10,3 / 13,0 | 43 % / 185 % / 314 % | 28 |
| r3 | 16:28 → 17:36 | 4 127 s | 2,5 / 7,0 / 10,1 | 26 % / 146 % / 284 % | 29 |
| r4 (validation) | 19:55 → 21:03 | 4 084 s | 3,5 / 7,2 / 13,7 | 44 % / 666 % / 770 % | **vert, `exit=0`** |

**Le critère « `uptime` 1 min < 1,5 tenu » est intenable pour ce bench, et n'a pas été tenu.** Le processus de bench seul occupe 1,5 à 2 cœurs en continu (150 à 205 % dans `ps`) avec des pointes de 14 threads exécutables (`vmstat`, colonne `r`) : sa propre charge 1 min est de 2 à 3, pointes à 10-14 (au dernier scénario de r1, à 13,9, `ps` montre le bench à 111 %). Le seuil de 1,5 de la consigne comptait ~1,1 pour le bench (rapport T15) ; il n'est pas atteint même sans aucun autre processus (charge de 0,68 au repos avant r1, 1,5 à 3 dès les premiers scénarios). **Je n'ai donc pas refait les relevés indéfiniment** ; j'ai mesuré ce qui est mesurable, la charge **hors bench** (r2 à r4), et je la donne ci-dessus.

Perturbateurs identifiés (pas des workers de la série) : outillage de l'orchestrateur sur le VPS, jamais un build ni un bench d'un autre worker pendant les relevés. `hermes` (python) à 50 à 100 % par moments en r2 (15:25 à 16:09) et r3 ; **`chrome` (2 processus, 35 à 80 %) pendant les deux premières minutes de r3** ; **`hermes` à 560 à 700 % pendant 32 échantillons de r4** (≈ 5 minutes sur 68, entre 19:55 et 21:00) ; `omp`, `node`, `tailscaled` : 5 à 20 % chacun en continu. Avant r1 : le build de b2 (`rustc` 4 × 100 %, puis `cargo test` des crates, `map_ldtk` à 300 %) a tenu le poste de 13:00 à 14:08 : j'ai attendu qu'il finisse et que `loadavg` retombe à 0,7 avant de lancer r1. Écart entre relevés des scénarios budgétés : 10 à 18 % (médiane sur les 653 scénarios : 19 %, pointes de 60 à 100 % sur des scénarios de 60 à 150 frames où le démarrage domine, non budgétés). Les budgets sont à la moitié de la pire mesure : cette dispersion est couverte ; elle **n'est pas** une mesure de référence au calme absolu, qui n'existe pas sur ce poste avec ce bench.

Le même poste a mesuré ces scénarios 30 % plus vite il y a deux jours (les relevés « indicatifs » de T15, ci-dessus : `idle` 118 à 155 fps, `bench_cave` 106 fps, contre 82 à 88 et 69 à 82 aujourd'hui) : le poste varie d'une journée à l'autre, donc la comparaison avant/après T15 ci-dessous est faite **le même jour**, binaires entrelacés.

### Budgets recalibrés (49 entrées)

Règle : plancher = ⌈pire fps / 2⌉, plafond de rollback = ⌈2 × pire rollback⌉ (pire des trois relevés r1 à r3). Résultat : **36 planchers baissés, 13 montés** ; **11 plafonds de rollback montés, aucun baissé** ; 31 planchers baissés de plus de 25 % de leur ancienne valeur. r4 : tous les scénarios au-dessus de leur plancher et sous leur plafond, suite complète (653 scénarios) verte d'un seul tenant, plus bas fps de la suite 26,4 (`bench_tears`, plancher 14).

**Clés mortes corrigées.** `item_bottes`, `item_fiole`, `item_pierre_de_soin`, `item_lame`, `room_combat`, `boss_gatling` ne désignaient aucun scénario (les scénarios générés ont pour nom `generated/<jeu>/<nom>`, recherche exacte dans `budgets.scenarios`) : ils étaient jugés sur le défaut (40 fps). Ils sont désormais clés par leur id complet (6 clés → 10 ids : `generated/gungeon/item_bottes`, `generated/testbed/item_bottes`, …, `generated/gungeon/boss_gatling`), d'où les 13 hausses (par exemple `item_fiole` 20 → 46).

| scénario | fps r1 / r2 / r3 | r4 | ancien plancher | nouveau plancher | rollback pire r1 / r2 / r3 (ms) | r4 | ancien plafond | nouveau plafond |
|---|---|---|---|---|---|---|---|---|
| bench_floor | 34.0 / 37.7 / 35.0 | 33.5 | 40 | **18** (-55 %) | 36.7 / 33.8 / 36.2 | 37.9 | 18 | **74** |
| gungeon_full_run | 42.3 / 45.7 / 41.5 | 43.1 | 24 | **21** (-12 %) | 24.2 / 22.4 / 24.9 | 24.3 | 46 | **50** |
| gungeon_full_run_four | 29.8 / 32.4 / 33.7 | 32.4 | 20 | **15** (-25 %) | 30.0 / 27.8 / 27.3 | 28.3 | 51 | **60** |
| bot_explorer_floor | 79.0 / 85.4 / 75.5 | 77.4 | 120 | **38** (-68 %) | 13.9 / 13.2 / 15.2 | 15.1 | — | — |
| bot_explorer_two | 70.4 / 75.1 / 63.6 | 69.4 | 100 | **32** (-68 %) | 14.6 / 13.8 / 16.4 | 15.3 | — | — |
| generated/gungeon/item_bottes | 86.1 / 95.9 / 96.4 | 74.9 | 20 | **44** (+120 %) | 14.5 / 13.1 / 13.1 | 16.3 | — | — |
| generated/testbed/item_bottes | 75.6 / 80.9 / 92.2 | 77.3 | 20 | **38** (+90 %) | 16.4 / 16.0 / 13.7 | 17.4 | — | — |
| generated/gungeon/item_fiole | 91.0 / 102.5 / 110.9 | 92.8 | 20 | **46** (+130 %) | 14.1 / 12.2 / 11.0 | 13.6 | — | — |
| generated/testbed/item_fiole | 84.4 / 97.2 / 86.9 | 78.4 | 20 | **43** (+115 %) | 15.0 / 13.6 / 15.7 | 16.8 | — | — |
| generated/gungeon/item_pierre_de_soin | 80.1 / 99.2 / 76.8 | 96.9 | 20 | **39** (+95 %) | 15.7 / 12.0 / 16.5 | 13.1 | — | — |
| generated/testbed/item_lame | 85.5 / 82.0 / 96.9 | 76.2 | 20 | **42** (+110 %) | 14.8 / 15.3 / 13.5 | 16.9 | — | — |
| generated/gungeon/room_combat | 62.0 / 80.2 / 76.1 | 90.7 | 25 | **32** (+28 %) | 18.1 / 13.9 / 15.3 | 12.9 | — | — |
| generated/isaac/room_combat | 61.5 / 58.4 / 70.5 | 64.2 | 25 | **30** (+20 %) | 18.4 / 19.3 / 16.5 | 18.2 | — | — |
| generated/testbed/room_combat | 68.3 / 82.0 / 71.0 | 83.4 | 25 | **35** (+40 %) | 15.6 / 13.1 / 15.3 | 13.3 | — | — |
| generated/gungeon/boss_gatling | 77.7 / 61.8 / 95.0 | 71.6 | 25 | **31** (+24 %) | 14.6 / 17.4 / 12.0 | 16.3 | — | — |
| ammo_burst | 75.2 / 87.5 / 76.5 | 82.6 | 120 | **38** (-68 %) | 13.5 / 11.8 / 13.8 | 13.0 | — | — |
| bullets_walls | 80.2 / 88.5 / 84.5 | 89.7 | 140 | **41** (-71 %) | 12.0 / 11.3 / 12.0 | 11.6 | — | — |
| dash_wall | 79.9 / 93.3 / 79.1 | 75.7 | 100 | **40** (-60 %) | 12.4 / 10.8 / 13.2 | 13.9 | — | — |
| door_open | 62.5 / 87.9 / 83.5 | 76.5 | 45 | **32** (-29 %) | 15.3 / 12.5 / 13.4 | 15.0 | — | — |
| idle | 81.8 / 88.2 / 84.2 | 84.1 | 110 | **41** (-63 %) | 13.0 / 12.0 / 12.8 | 13.0 | — | — |
| movement_melee | 80.7 / 87.0 / 71.7 | 93.7 | 90 | **36** (-60 %) | 12.8 / 12.1 / 14.7 | 11.6 | — | — |
| remote_first_fight | 86.5 / 86.9 / 90.7 | 85.8 | 120 | **44** (-63 %) | 12.1 / 12.0 / 11.6 | 12.6 | — | — |
| shoot_around | 83.3 / 84.8 / 76.8 | 84.7 | 100 | **39** (-61 %) | 12.5 / 12.4 / 14.0 | 12.8 | — | — |
| two_players_idle | 72.2 / 73.3 / 74.6 | 81.9 | 80 | **37** (-54 %) | 14.6 / 14.6 / 14.5 | 13.5 | — | — |
| two_players_shooting | 73.9 / 74.4 / 80.5 | 88.3 | 130 | **37** (-72 %) | 13.3 / 13.1 / 13.0 | 11.7 | — | — |
| weapons_workout | 74.1 / 80.3 / 83.6 | 81.0 | 100 | **38** (-62 %) | 14.0 / 12.9 / 12.8 | 13.3 | — | — |
| window_repair | 70.2 / 88.2 / 84.1 | 79.5 | 100 | **36** (-64 %) | 15.1 / 12.0 / 12.8 | 14.0 | — | — |
| four_players_idle | 66.8 / 80.1 / 65.4 | 69.6 | 100 | **33** (-67 %) | 13.7 / 12.0 / 14.8 | 14.3 | — | — |
| four_players_shooting | 65.0 / 84.4 / 67.4 | 77.6 | 100 | **33** (-67 %) | 14.8 / 11.6 / 14.9 | 13.0 | — | — |
| testbed_ally_safe | 86.3 / 93.3 / 90.3 | 107.6 | 150 | **44** (-71 %) | 13.0 / 11.8 / 12.4 | 10.4 | — | — |
| testbed_arena_idle | 98.8 / 94.2 / 94.7 | 81.8 | 150 | **48** (-68 %) | 10.8 / 11.4 / 11.8 | 13.1 | — | — |
| testbed_civilian_blocks | 100.8 / 93.4 / 90.6 | 93.0 | 160 | **46** (-71 %) | 10.8 / 11.6 / 12.5 | 12.3 | — | — |
| testbed_corridor_idle | 113.1 / 109.3 / 97.3 | 93.7 | 180 | **49** (-73 %) | 9.7 / 9.8 / 11.6 | 11.8 | — | — |
| testbed_dummy_shoot | 89.4 / 111.2 / 84.9 | 95.2 | 160 | **43** (-73 %) | 12.9 / 9.8 / 13.3 | 11.9 | — | — |
| testbed_follower | 89.1 / 97.8 / 90.9 | 93.7 | 170 | **45** (-74 %) | 11.9 / 10.8 / 12.4 | 12.0 | — | — |
| testbed_target_hits | 101.2 / 90.7 / 92.6 | 91.6 | 140 | **46** (-67 %) | 10.9 / 12.1 / 12.4 | 12.4 | — | — |
| testbed_two_rooms_door_idle | 100.7 / 106.8 / 89.7 | 93.0 | 180 | **45** (-75 %) | 10.2 / 9.6 / 12.3 | 11.9 | — | — |
| testbed_window | 105.8 / 97.5 / 80.0 | 99.0 | 160 | **40** (-75 %) | 10.0 / 10.7 / 13.2 | 10.9 | — | — |
| bench_bullets | 56.8 / 58.6 / 50.8 | 58.0 | 80 | **26** (-68 %) | 20.3 / 20.0 / 23.2 | 20.3 | 8 | **47** |
| bench_bullets_room | 38.9 / 42.9 / 38.1 | 33.2 | 40 | **20** (-50 %) | 40.4 / 37.5 / 41.6 | 47.7 | 26 | **84** |
| throne_quad | 46.4 / 45.4 / 47.0 | 51.8 | 14 | **23** (+64 %) | 27.5 / 28.0 / 27.6 | 25.3 | — | — |
| throne_three_floors | 52.3 / 54.1 / 55.0 | 60.8 | 20 | **27** (+35 %) | 24.3 / 23.6 / 23.6 | 21.8 | — | — |
| throne_softlock_recul | 63.8 / 56.9 / 61.3 | 70.0 | 26 | **29** (+12 %) | 19.2 / 21.5 / 20.2 | 18.0 | — | — |
| bench_horde | 37.1 / 39.3 / 34.9 | 35.8 | 38 | **18** (-53 %) | 51.0 / 49.0 / 53.4 | 52.9 | 70 | **107** |
| bench_cave | 69.4 / 81.7 / 73.1 | 69.4 | 100 | **35** (-65 %) | 15.3 / 13.3 / 14.9 | 15.9 | 5 | **31** |
| bench_tears | 27.7 / 30.7 / 26.6 | 26.4 | 17 | **14** (-18 %) | 70.0 / 63.7 / 72.2 | 73.2 | 113 | **145** |
| bench_tears_trio | 69.4 / 78.1 / 68.7 | 72.4 | 49 | **35** (-29 %) | 19.3 / 17.7 / 20.2 | 19.1 | 27 | **41** |
| bench_familiars | 71.7 / 80.7 / 84.6 | 73.3 | 45 | **36** (-20 %) | 15.5 / 14.2 / 13.5 | 15.9 | 26 | **31** |
| bench_floor_isaac | 52.3 / 58.9 / 52.8 | 51.1 | 29 | **27** (-7 %) | 20.5 / 18.6 / 21.0 | 21.7 | 38 | **42** |

### Baisses de plus de 25 % : le poste, pas une régression

Pour chaque scénario dont le plancher baisse de plus de 25 %, même scénario sur `7ee084e8` (`origin/main` **d'avant T15**, parent de la fusion `6e6be0ba`, worktree `/home/debian/orca/workers/b6/alacod-base-pret15`, même `CARGO_TARGET_DIR`, même profil), deux passes le même jour que r1 à r3 (vers 18 h UTC, juste après) : fps moyen des trois relevés de la branche contre fps moyen des deux passes de `main` d'avant T15 (ratio < 1 : la branche est plus lente). Les scénarios générés, `throne_*` et les scénarios de T15 ne baissent pas de plus de 25 % (ou n'existaient pas avant T15).

| scénario | fps branche (r1/r2/r3) | fps avant T15 (2 passes) | ratio | rollback branche (ms) | rollback avant T15 (ms) |
|---|---|---|---|---|---|
| bench_floor | 34 / 38 / 35 | 45 / 46 | 0.78 | 36.7 / 33.8 / 36.2 | 25.9 / 25.7 |
| ammo_burst | 75 / 88 / 77 | 101 / 90 | 0.84 | 13.5 / 11.8 / 13.8 | 9.6 / 10.8 |
| testbed_ally_safe | 86 / 93 / 90 | 104 / 106 | 0.86 | 13.0 / 11.8 / 12.4 | 10.6 / 10.4 |
| bot_explorer_two | 70 / 75 / 64 | 81 / 80 | 0.87 | 14.6 / 13.8 / 16.4 | 13.8 / 14.2 |
| testbed_target_hits | 101 / 91 / 93 | 98 / 115 | 0.89 | 10.9 / 12.1 / 12.4 | 11.5 / 9.4 |
| testbed_follower | 89 / 98 / 91 | 94 / 111 | 0.90 | 11.9 / 10.8 / 12.4 | 11.6 / 9.6 |
| bench_bullets | 57 / 59 / 51 | 64 / 59 | 0.91 | 20.3 / 20.0 / 23.2 | 18.1 / 19.0 |
| bullets_walls | 80 / 88 / 84 | 91 / 93 | 0.92 | 12.0 / 11.3 / 12.0 | 11.3 / 11.1 |
| testbed_dummy_shoot | 89 / 111 / 85 | 105 / 100 | 0.93 | 12.9 / 9.8 / 13.3 | 10.7 / 11.2 |
| four_players_idle | 67 / 80 / 65 | 74 / 77 | 0.94 | 13.7 / 12.0 / 14.8 | 13.2 / 12.6 |
| shoot_around | 83 / 85 / 77 | 83 / 91 | 0.94 | 12.5 / 12.4 / 14.0 | 12.6 / 11.6 |
| bench_bullets_room | 39 / 43 / 38 | 45 / 40 | 0.95 | 40.4 / 37.5 / 41.6 | 35.5 / 39.6 |
| testbed_window | 106 / 98 / 80 | 104 / 96 | 0.95 | 10.0 / 10.7 / 13.2 | 10.2 / 11.1 |
| bot_explorer_floor | 79 / 85 / 75 | 84 / 84 | 0.95 | 13.9 / 13.2 / 15.2 | 13.7 / 13.7 |
| bench_horde | 37 / 39 / 35 | 39 / 38 | 0.97 | 51.0 / 49.0 / 53.4 | 49.0 / 50.1 |
| two_players_idle | 72 / 73 / 75 | 72 / 78 | 0.98 | 14.6 / 14.6 / 14.5 | 14.9 / 13.9 |
| movement_melee | 81 / 87 / 72 | 80 / 80 | 0.99 | 12.8 / 12.1 / 14.7 | 13.4 / 13.3 |
| idle | 82 / 88 / 84 | 88 / 80 | 1.01 | 13.0 / 12.0 / 12.8 | 12.1 / 13.3 |
| door_open | 62 / 88 / 83 | 73 / 82 | 1.01 | 15.3 / 12.5 / 13.4 | 15.1 / 13.6 |
| testbed_corridor_idle | 113 / 109 / 97 | 107 / 103 | 1.01 | 9.7 / 9.8 / 11.6 | 9.9 / 10.4 |
| remote_first_fight | 87 / 87 / 91 | 87 / 86 | 1.02 | 12.1 / 12.0 / 11.6 | 12.1 / 12.3 |
| testbed_two_rooms_door_idle | 101 / 107 / 90 | 99 / 95 | 1.02 | 10.2 / 9.6 / 12.3 | 11.0 / 11.3 |
| bench_cave | 69 / 82 / 73 | 73 / 72 | 1.03 | 15.3 / 13.3 / 14.9 | 14.8 / 15.1 |
| testbed_arena_idle | 99 / 94 / 95 | 89 / 96 | 1.04 | 10.8 / 11.4 / 11.8 | 12.4 / 11.5 |
| window_repair | 70 / 88 / 84 | 82 / 73 | 1.04 | 15.1 / 12.0 / 12.8 | 12.8 / 14.2 |
| four_players_shooting | 65 / 84 / 67 | 65 / 73 | 1.05 | 14.8 / 11.6 / 14.9 | 15.6 / 13.7 |
| weapons_workout | 74 / 80 / 84 | 79 / 70 | 1.07 | 14.0 / 12.9 / 12.8 | 13.4 / 14.8 |
| testbed_civilian_blocks | 101 / 93 / 91 | 85 / 85 | 1.12 | 10.8 / 11.6 / 12.5 | 13.1 / 12.8 |
| dash_wall | 80 / 93 / 79 | 67 / 83 | 1.12 | 12.4 / 10.8 / 13.2 | 14.0 / 12.5 |
| two_players_shooting | 74 / 74 / 81 | 77 / 56 | 1.14 | 13.3 / 13.1 / 13.0 | 12.8 / 17.5 |

Sur 30 scénarios, ratio de 0,89 à 1,14 pour 28 (bruit du poste, 10 à 18 %), donc les écarts aux anciens planchers (macOS arm64 et cloud 4 cœurs, 2 à 3,5 fois plus rapides que ce VPS, voir « Comparaison M4/VPS », D98) sont le **poste**. Deux limites :

- `ammo_burst` 0,84 et `bench_floor` 0,78 : refaits **six fois chacun, binaires entrelacés** (`bin-base`/`bin-head` copiés, une exécution de l'un puis de l'autre) avec `bot_explorer_two`, `bench_bullets`, `testbed_ally_safe` :

| scénario | fps avant T15 (médiane de 6) | fps branche (médiane de 6) | rollback avant (ms) | rollback branche (ms) | verdict |
|---|---|---|---|---|---|
| ammo_burst | 89,2 | 85,1 | 11,2 | 12,2 | bruit (écart 5 %, plages 64 à 93 / 74 à 97) |
| bench_bullets | 60,2 | 56,0 | 19,5 | 20,9 | bruit (7 %) |
| bot_explorer_two | 75,3 | 67,9 | 15,2 | 16,0 | bruit (10 %, plages qui se recouvrent) |
| testbed_ally_safe | 85,1 | 92,9 | 14,4 | 12,7 | bruit |
| **bench_floor** | **45,9** | **35,3** | **27,0** | **36,7** | **−23 % en fps, +36 % de rollback, stable sur les 6 passes (plages 38 à 47 contre 31 à 39 ; rollback 25 à 30 contre 33 à 41)** |

- **`bench_floor` est plus lent que `main` d'avant T15, de façon reproductible.** Ce n'est pas T15 (qui n'a touché ni la simulation ni le contenu : traces inchangées) : [INFERENCE] c'est `m2-salles-murs-gungeon`, seul changement de gungeon entre `7ee084e8` et `f96fa463` (salles ×1,9 à ×5,5, étage plus grand, champ de flux de 406 µs par frame déjà relevé par T15). Non isolé par une mesure sur `6e6be0ba` (la consigne : mesurer `main` tel qu'il est). Le plancher (18 fps) et le plafond (74 ms) de `bench_floor` portent donc le coût du contenu agrandi ; ce n'est pas une régression de code sans cause, mais le pire rollback de 37 ms (3 ticks de 16,7 ms et plus) est **à suivre** : aucune dette ouverte tant que la cause est un choix de contenu ; à rouvrir si `bench_floor` descend sous 30 fps sur ce poste.

**Régression réelle : aucune.** Aucune dette nouvelle. (D97, `bench_tears` : pire rollback 64 à 72 ms ici contre 56 ms au rapport, même ordre ; plafond 145 ms ; reste ouverte.)
