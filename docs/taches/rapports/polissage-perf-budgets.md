# Rapport — polissage-perf-budgets (S2-9, voie b0, zombies et throne seulement ; isaac retiré)

Fiche : `docs/taches/polissage-perf-budgets.md`, réduite à son § 0 par la consigne de run (`b0-e4-s2-9.md`). Branche `polissage-perf-budgets`
depuis `origin/polissage-integration` (`4cea5046`). Aucun code de simulation touché, aucune trace ne bouge (un seul fichier modifié :
`tests/budgets.ron`).

## 0. Verdict

- **Le Mac est très loin de tous les budgets.** Sur les 115 scénarios de zombies (63) et de throne (52), le plus lent est
  `la_fonderie_demo` à **119 fps de simulation au pire de quatre passes** (2 fois le temps réel de 60 Hz ; défaut du fichier
  de budgets = 40 fps) ; le pire rollback (restauration + 12 frames resimulées) est de **12,7 ms** (`la_fonderie_demo`), soit **0,76 tick**
  de 16,7 ms, sous le seuil de 2 ticks (33 ms) qui déclenche une optimisation. Throne : plus lent `throne_quad` 207 fps, pire rollback 6,1 ms.
- **Aucune optimisation n'est faite** : le seuil de la consigne (pire rollback > 33 ms) n'est atteint par aucun scénario de zombies ni de throne.
  Le champ de flux (D70) pèse **250 µs par frame** dans `p2p_smoke` (quatre joueurs qui bougent) contre 9,1 ms le 2026-10-15 (M4 avant
  les grilles denses) : il reste le premier poste de ce scénario, mais le scénario entier tient en 2,7 à 6,1 ms de pire rollback (§ 3).
- **`make bench` strict passe sur le Mac** (RC 0, quatre fois, § 1). Le Mac est 5 à 20 fois plus rapide que le relevé VPS du 2026-10-20 sur les mêmes
  scénarios (§ 4) : ce passage ne prouve rien pour le VPS.
- **Budgets** : décision du coordinateur (réponse C à la question du 2026-10-29) : `tests/budgets.ron` reste calibré sur le **VPS**, un seul fichier,
  pas de valeur tirée du Mac. Quatre plafonds de rollback ajoutés, **provisoires**, à partir du seul relevé VPS existant (§ 4). Les **96 scénarios
  de zombies et de throne sans budget propre** attendent un relevé au calme sur le VPS : liste et commande exacte en § 4.

## 1. Machine, charge, passes

MacBook Pro **Apple M4 Max**, 14 cœurs, 36 Go, macOS (Darwin 25.6), build `--profile headless` (`--features game/dev-dylib`),
`CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac` (partagé avec les autres voies), `ALACOD_MUTE=1`, aucun son, un seul processus de bench.
**Ce n'est pas un Mac au calme** : les autres voies (b1, b2, b3, intégration) compilaient et jouaient des scénarios, le 1 min de `uptime`
va de 3,6 à 60 selon la passe. Les mêmes scénarios varient d'un facteur 3 à 5 entre la passe la plus chargée et la plus calme
(`immune_tag` 303 → 1 603 fps, `throne_quad` 207 → 477 fps) : toutes les valeurs sont des **planchers de capacité**, pas des performances de référence.

| passe | commande | périmètre | `uptime` 1 min au départ → fin | issue |
|---|---|---|---|---|
| p1 | `make bench` (`ALACOD_BENCH_STRICT=1`, 810 scénarios, 20 min) | suite entière | 14,4 → 37,1 | RC 0 |
| p2 | `ALACOD_BENCH_STRICT=1 make test_scenarios SCENARIO=<115 noms>` | zombies + throne | 26,7 → 37,5 | RC 0 |
| p3 | idem | zombies + throne | 37,5 → 16,9 | RC 0 |
| p4 | `make bench` (après l'ajout des plafonds) | suite entière | 13,0 → 3,6 | RC 0 |

Détail par scénario, quatre passes : `docs/taches/rapports/polissage-perf-budgets/mac-m4max-4-passes.csv` (fps, pire rollback, budget lu).

## 2. Tableau Mac par clone

Valeurs « pire des quatre passes » pour le fps, « pire des quatre » pour le rollback (`rollback_ms_worst`).

| clone | scénarios | joueurs | fps simulés : plus lent / médian / plus rapide | pire rollback | scénario le plus lourd |
|---|---|---|---|---|---|
| zombies | 63 | 1 à 4 | 119 / 503 / 809 | **12,7 ms** (0,76 tick) | `la_fonderie_demo` (6 800 frames, 309 entités) |
| throne | 52 | 1 à 4 | 207 / 560 / 979 | **6,1 ms** (0,37 tick) | `throne_quad` (3 400 frames, 4 joueurs, 200 entités) |

Les cinq plus lourds de chaque clone (fps des quatre passes, du plus chargé au plus calme ; pire rollback en ms, même ordre) :

| clone | scénario | frames | joueurs | entités max | fps p1 / p2 / p3 / p4 | rollback p1 / p2 / p3 / p4 | budget lu |
|---|---|---|---|---|---|---|---|
| zombies | `la_fonderie_demo` | 6 800 | 1 | 309 | 184 / 119 / 197 / 277 | 7,9 / 12,7 / 7,9 / 6,0 | défaut 40 fps |
| zombies | `bench_horde` | 900 | 4 | 255 | 465 / 240 / 232 / 486 | 4,2 / 7,1 / 7,2 / 4,1 | 26 fps / 61 ms |
| zombies | `p2p_smoke` | 600 | 4 | 121 | 343 / 258 / 448 / 673 | 4,1 / 6,1 / 3,5 / 2,7 | défaut 40 fps |
| zombies | `bench_bullets` | 600 | 4 | 338 | 874 / 293 / 486 / 668 | 1,1 / 3,3 / 1,9 / 1,4 | 41 fps / 24 ms |
| zombies | `immune_tag` | 150 | 2 | 94 | 1 178 / 303 / 723 / 1 603 | 0,7 / 2,7 / 1,1 / 0,6 | défaut 40 fps |
| throne | `throne_quad` | 3 400 | 4 | 200 | 314 / 207 / 411 / 477 | 4,0 / 6,1 / 3,2 / 2,9 | 23 fps / 59 ms |
| throne | `generated/throne/weapon_plasma` | 200 | 1 | 29 | 1 014 / 869 / 215 / 2 053 | 0,9 / 1,1 / 2,7 / 0,5 | défaut 40 fps |
| throne | `generated/throne/weapon_revolver` | 200 | 1 | 29 | 1 090 / 841 / 232 / 1 134 | 0,8 / 1,1 / 2,5 / 0,8 | défaut 40 fps |
| throne | `generated/throne/enemy_chien_moving` | 600 | 1 | 34 | 713 / 265 / 1 182 / 1 259 | 1,3 / 3,4 / 0,9 / 0,8 | défaut 40 fps |
| throne | `throne_three_floors` | 5 000 | 2 | 168 | 484 / 272 / 511 / 715 | 2,7 / 4,6 / 2,5 / 2,0 | 27 fps / 48 ms |

Le fps d'un scénario court de throne (200 frames, 29 entités) est très bruité d'une passe à l'autre (`weapon_plasma` 215 à 2 053 fps) ; la cause n'est pas isolée.

## 3. Profil du champ de flux (D70)

`ALACOD_SYSTEM_PROFILE=1 ALACOD_SNAPSHOT_PROFILE=1 make test_scenarios SCENARIO=<nom>` (la sortie `profil sets :` du runner est un temps par
`RollbackSystemSet`, µs par frame de simulation, resimulation du synctest comprise) ; logs `/tmp` non gardés, valeurs ci-dessous recopiées.

| scénario | `FlowField` µs / frame | premier set | deuxième set | lecture |
|---|---|---|---|---|
| `la_fonderie_demo` (le plus lourd) | 11,5 | `EnemyAI` 18,8 | `FlowField` 11,5, `Weapon` 10,6 | le champ de flux n'est plus le premier poste |
| `bench_horde` | 11,9 | `EnemyAI` 16,8 | `FlowField` 11,9, `Weapon` 10,2 | idem |
| `throne_quad` | 12,3 | `EnemyAI` 19,8 | `FlowField` 12,3, `Weapon` 11,7 | idem |
| `p2p_smoke` (quatre joueurs qui bougent, le cas de D70) | **250,4** | `FlowField` 250,4 | `Weapon` 15,2, `EnemyAI` 14,0 | reconstruction à chaque changement de case : 8 fois le suivant, ≈ 80 % de la frame simulée (0,32 ms) |

D70 (mesures du 2026-10-15) : 9,1 ms par frame sur `p2p_smoke` avant les grilles denses, 7,06 ms sur M4 ; ici **0,25 ms** pour le même coût de
reconstruction. Le pire rollback du scénario est de **2,7 à 6,1 ms** selon la charge (24,8 ms sur VPS le 2026-10-15, 85,1 ms avant la sortie (a)). **Le seuil de 33 ms est
loin** : pas d'optimisation, et en particulier pas la sortie (b) de D70 (cadence de 30 frames : changement de gameplay, bless de toute la suite).
`docs/taches/dettes.md` n'est pas modifié (D70 reste ouverte pour la cible VPS ; ce rapport n'est pas une mesure VPS).

## 4. Budgets : ce qui est ajouté, ce qui reste

`tests/budgets.ron` : règle du fichier = plancher moitié et plafond double de la **pire de trois** mesures **au calme sur le VPS Debian 8 cœurs**
(D84). Le Mac donne 5 à 20 fois plus de fps que le relevé VPS du 2026-10-20 sur les mêmes scénarios (`bench_horde` 232 à 486 fps contre 42,0 ; `throne_quad`
207 à 477 contre 44,5 ; `four_players_idle` 501 à 1 416 contre 70,5) et bien plus encore que leurs planchers (26, 23 et 33 fps) : un plancher dérivé d'une passe Mac ferait échouer `make bench` sur le VPS. Décision du coordinateur : **pas de second fichier**, pas de valeur Mac.

**Ajoutés (provisoires)** : plafond de pire rollback, au double arrondi au-dessus du pire rollback de l'**unique** relevé VPS disponible
(`latest.json` du commit `0d2c1b61`, VPS partagé, 2026-10-20, `docs/taches/rapports/plan-polissage.inventaire.md` § 7, ces scénarios n'avaient pas de plafond) :

| scénario | relevé VPS (fps / rollback ms) | plancher (inchangé) | plafond ajouté |
|---|---|---|---|
| `four_players_idle` | 70,5 / 13,5 | 33 | **28** |
| `four_players_shooting` | 71,3 / 13,2 | 33 | **27** |
| `throne_quad` | 44,5 / 29,0 | 23 | **59** |
| `throne_three_floors` | 53,3 / 23,9 | 27 | **48** |

Un seul relevé, sur un poste partagé : ce n'est pas la règle D84 (pire de trois au calme), d'où le mot « provisoires » dans le commentaire du fichier. Les quatre passent sur le Mac
(rollbacks de 0,6 à 6,1 ms) ; leur validité sur le VPS se constate à son prochain `make bench` strict.

**Non ajoutés, faute de relevé VPS** : les **96 scénarios de zombies (47) et de throne (49) qui n'ont aucune entrée propre** (ils lisent le défaut de 40 fps, sans plafond de rollback),
plus `throne_softlock_recul` (plancher 29 fps, aucun relevé VPS de rollback). Aucune valeur n'est inventée. La liste exacte est dans
`docs/taches/rapports/polissage-perf-budgets/sans-budget-vps.txt` ; à jouer **trois fois au calme sur le VPS** (`uptime` 1 min < 1,5, relever la charge), en copiant `target/metrics/latest.json` entre deux passes :

```
cd <worktree VPS> && for i in 1 2 3; do ALACOD_BENCH_STRICT=1 ALACOD_SCENARIO="$(cat docs/taches/rapports/polissage-perf-budgets/sans-budget-vps.txt)" \
  make test_scenarios && cp target/metrics/latest.json /tmp/budgets-vps-pass$i.json; done
```

puis, par scénario, plancher = moitié (arrondie au-dessus) du pire `sim_fps` des trois et plafond = double (arrondi au-dessus) du pire `rollback_ms_worst` ; ajouter les entrées dans `tests/budgets.ron`. Les 96
noms sont les 115 scénarios moins les 19 qui ont déjà une entrée. Estimation : 0,25 j sur le VPS (trois passes de ≈ 10 min pour ces 96 scénarios) plus 0,1 j d'édition.
Pas de dette ouverte : la voie b0 n'a plus de numéro libre (D158 à D160 pris par S1-1), la question au coordinateur est restée sans réponse à la date de ce rapport ; la ligne `LIVRÉ` porte le reste.

## 5. Optimisation à trace identique

Aucune : pire rollback de zombies 12,7 ms et de throne 6,1 ms, contre un seuil de déclenchement de 33 ms. `make schedule_order_diff` non lancé :
aucun fichier de simulation ni de schedule n'a changé (`git diff origin/polissage-integration --stat` : `tests/budgets.ron`, ce rapport, un CSV et une liste).
Aucune trace ne bouge : la suite entière passe sans `BLESS` (p1 et p4).

## 6. Seuils navigateur de S1-5 reportés (`polissage-perf-web.md`)

S1-5 n'a pas écrit de seuil chiffré ; ses mesures (Chrome for Testing 150, M4 Max, écran 60 Hz, 1 puis 2 joueurs **locaux**, charge du Mac de 13 à 52) servent de **référence à ne pas dégrader** :

| clone | sim médiane / p5 | pas de simulation moyen / pire | fil principal occupé | entités | chargement | `wasm_bg.wasm` |
|---|---|---|---|---|---|---|
| zombies | 60,0 / 59,7 fps | 0,89 à 1,00 ms / 7,8 à 10,9 ms | 24 à 27 % | 282 à 343 | 1,9 à 2,0 s | 48,2 Mo (11,8 gzip, 7,7 brotli) |
| throne | 60,0 / 59,8 à 60,0 fps | 0,53 à 0,58 ms / 1,9 à 5,2 ms | 18 à 21 % | 74 à 104 | 1,8 à 2,0 s | 41,4 Mo (11,5 gzip, 7,6 brotli) |

- Limite à surveiller : **25 Mio par fichier de Cloudflare Pages**, dépassée aujourd'hui par les deux `wasm_bg.wasm` non optimisés (le script de build avertit ; `wasm-opt` est la piste, S1-5 § 8).
- Non mesuré, repris tel quel : deux onglets en ligne (WebRTC impossible sur ce Mac, VPN `utun1`), 120 Hz (lecture à l'E4 avec `?perf=1`), charge pire cas, Firefox, Safari.
- Lecture croisée : le pas de simulation moyen du navigateur (≈ 0,9 ms zombies) est de l'ordre de la simulation native du Mac (0,3 à 0,6 ms pour les scénarios les plus lourds, § 3) ; le navigateur est donc de l'ordre de 1,5 à 3 fois plus lent que le natif sur cette machine [INFÉRENCE : scénarios et charges différents, pas de mesure sur une même partie].

## 7. Vérifications (tête de branche `polissage-perf-budgets`)

- `make bench` (strict) : RC 0, deux fois (p1 avant, p4 après l'ajout des plafonds). Les 810 scénarios passent sans `BLESS` : **aucune trace ne bouge**.
- `make check` : **RC 2 à `test_tools`**, trois échecs de `scripts/tests/test_claude_say.py` (`test_reads_last_assistant_and_cleans_markdown`, `test_say_ignores_unlisted_voice`, `test_say_uses_system_voice_when_name_absent` :
  le test lie `/usr/bin/python3` dans un `PATH` vide) : échec d'environnement du Mac déjà rapporté par S1-1, S1-5 et S1-9, sans rapport avec ce lot, dans un fichier qui n'est pas à cette voie. Le reste de `make check` a été joué
  cible par cible : `make -k format check_forbidden check_rollback_registration check_sim_boundary test_crates lint` : voir le bloc ci-dessous ; la partie `test_scenarios` de `make test` est la suite du `make bench` (même test, sans le mode strict).

```
$ make -k format check_forbidden check_rollback_registration check_sim_boundary test_crates lint     (RC 0, ALACOD_MUTE=1, CARGO_TARGET_DIR=.target-mac)
OK: aucun nouveau motif interdit ; 52 occurrences auditées dans determinism-allowlist.json
OK: aucun appel direct à rollback_component_with*/rollback_resource_with* hors de crates/utils/src/rollback.rs
OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69
test_crates : 94 blocs « test result: ok », aucun échec
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)
games/testbed : aucune erreur (41 personnages, 25 armes, 6 armes de corps à corps, 0 vagues, 46 cartes)
games/throne : aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes)
games/gungeon : aucune erreur (11 personnages, 21 armes, 1 armes de corps à corps, 0 vagues, 5 cartes)
games/isaac : aucune erreur (24 personnages, 3 armes, 2 armes de corps à corps, 0 vagues, 3 cartes)
```

## 8. Reste

| Reste | Estimation |
|---|---|
| Trois passes au calme sur le VPS des 96 scénarios sans budget (§ 4) puis édition de `tests/budgets.ron` | 0,25 j (b3, qui a le VPS) + 0,1 j |
| Remesurer au calme les quatre plafonds provisoires (même lot) | inclus |
| Dette D70 : reste ouverte sur la cible VPS ; rien à faire sur le Mac | — |
| Seuils navigateur : deux onglets en ligne, 120 Hz (E4 du 30), `wasm-opt` | voir S1-5 § 8 |
