# T2.5 — reprise des power-ups, livraison finale

SHA de la tête de branche vérifiée (code et traces) :
`c6211eccf9bd8bb53f0c066f4d2251fbe5097aeb`.
Fiche : `docs/taches/T2.5-reprise-power-ups.md`.
Base de preuve : main `cc2a7a82ad088addb38bbc4547a39eefc3914799`, après T2.4.
Date : 2026-10-01. Agent : Codex.
Le SHA livré inclut ensuite le commit documentaire de ce rapport ; aucun code ni
contenu de simulation n'est changé par ce dernier commit.

**Vérification finale entièrement verte.** T2.5 est livrée pour revue et merge local
par l'orchestrateur. Aucun merge dans main, changement propre au journal
`docs/taches.md` ou retrait de worktree n'a été effectué. Aucune fiche suivante
n'a été commencée.

## 1. Fait

Reprise du WIP `9866a6a` dans le worktree existant
`/home/wq/Project/bascanada/alacod_tasks/m0-v1-power-ups/alacod`, branche
`m0-v1-power-ups`, avec son target amorcé. Lecture du préambule, de la fiche,
de CLAUDE.md, des conventions et du diff WIP. Une compilation à la fois,
`source ../env.sh`, `CARGO_BUILD_JOBS=4`, profil headless ; binaires de jeu sans
features par défaut. Aucun build avec rendu.

Corrigé par le commit de reprise :

- `PowerUpPickup` est enregistré avec rollback, **checksum GGRS** et trace, plutôt que
  la variante sans checksum. L'identité et l'expiration participent à la détection des
  désynchronisations.
- Le ramassage est fermé dès `frame >= expires_at_frame`, avant l'application d'une
  éventuelle action. Les placements scriptés utilisent la durée de vie de la table,
  au lieu des 100 000 frames arbitraires du WIP.
- `PowerUpPlacement.x/y` sont des `Fixed`, écrits en chaînes RON. Le placement ne
  reconvertit plus des `f32` dans `GgrsSchedule`.
- Max Ammo additionne les capacités des armes portées par type de munition, remplit
  leurs modes et annule le rechargement en cours. Une réserve déjà supérieure à la
  capacité calculée n'est pas diminuée. Le scénario prouve notamment 288 balles
  (mitrailleuse 240 + pistolet 48), sans débit ultérieur de rechargement.
- Les modificateurs utilisent `ModifierSource::Named("powerup:<id>")`, sur chaque
  joueur. Le runner conserve le jeu, les overrides d'armes/vagues, les placements et
  la chance de drop dans son scénario réenregistré. Le replay de Max Ammo est testé.
- Les 57 scénarios autres que `powerup_drop_on_kill` désactivent explicitement les
  drops aléatoires ; le générateur d'essais d'armes produit aussi cette isolation.
  Le scénario de drop garde sa chance forcée à 1, les tables des jeux restent à 15 %.
  À chance nulle le système ne crée ni ne consomme le flux `loot`.
- Six tests de systèmes couvrent le ramasseur unique de plus petit net_id, la frontière
  d'expiration, le buff de toute l'équipe, les réserves partagées et la recharge, la
  chance nulle, les morts simultanées ordonnées et l'indépendance du flux `waves`.
  Deux tests d'intégration supplémentaires couvrent l'expiration selon la table et le
  replay de Max Ammo. Les attentes des scénarios Max Ammo, Carpenter et Insta-Kill
  ont été renforcées.
- Les affirmations de preuve déjà faite dans le WIP ont été retirées : un diagnostic
  les contredit sur `points_on_kill`. La doc décrit le checksum, les coordonnées,
  l'expiration, les réserves et l'isolation du loot ; les commentaires « À regarder »
  reconnaissent l'absence de sprite de pickup. Formatage des passages WIP nécessaire.

Gardé du WIP : vocabulaire `effects::Action` (cinq variantes) et applicateur pur ;
tables des cinq power-ups ; kind/registre/lint PowerUp et ses trois fixtures ;
ramassage automatique en distance Fixed, tri par net_id et `despawn_rollback` ;
application à toute l'équipe ; Carpenter sur toutes les fenêtres ; Nuke et drops sur
l'équipe Enemies, y compris les personnages de laboratoire ; tirages pondérés dans
`loot`, morts triées ; plugin et ordres de systèmes ; attente `PowerUpPickups`.
Les changements d'actions/économie/lint supplémentaires dans ce commit sont du
formatage uniquement, hormis les corrections énumérées ci-dessus.


Finalisation après le merge de T2.4 :

- `git fetch origin` puis `git merge --no-commit origin/main`, merge enregistré
  dans `0916514`. Les 51 conflits de traces reprennent provisoirement main ; les
  conflits d'import dans `runner.rs` et de sections dans les conventions conservent
  les deux fonctionnalités. Run, RunRequest, RunState/RunSummary et les tests de
  run restent présents. Le nouveau scénario `run_lose_summary` isole aussi le loot.
- Quatre preuves contre main après T2.4, **avant tout bless** : 3 996 frames identiques.
  Le commit `c6211ec` porte la justification précise du bless. Les 58 scénarios ont
  été blessés, mais seules les six traces `powerup_*` ont effectivement changé ;
  les 52 références héritées de main restent identiques.
- Les dix scénarios d'armes ont été régénérés et explicitement blessés. La chance
  nulle figure dans le RON produit par le générateur ; ses traces restent identiques.
- CLAUDE.md liste aussi RunState et RunSummary ; la doc d'actions décrit la source
  `powerup:<id>` ; les renvois Power-ups pointent vers le §14, après Run au §13.

## 2. Vérifié

Les commandes ci-dessous ont été exécutées dans cette session. Les logs persistants
sont hors git dans `../verification/`. Le rapport intermédiaire et ses résultats
avant fusion restent consultables au commit `20fd2c9` ; ils ne décrivent plus
l'état courant.

### Preuve README §5

Sur main, sans `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=4` ; dans la branche,
`source ../env.sh` et `CARGO_BUILD_JOBS=4`. Un scénario par appel, dossiers vides :

```bash
ALACOD_SCENARIO=<nom> ALACOD_DUMP_TRACE=<dossier-vide> APP_VERSION=x \
  cargo test -p scenario --profile headless --test scenarios scenarios -- --nocapture
python3 scripts/trace-diff.py <dump-main>/<nom>.full <dump-branche>/<nom>.full \
  --ignore 'PowerUpPickup,FrameEvents<game::powerups::PowerUpPickedUp>'
```

| Scénario | Frames comparées | Résultat trace-diff |
|---|---:|---|
| idle | 1 499 | exit 0, identique |
| two_players_shooting | 299 | exit 0, identique |
| points_on_kill | 599 | exit 0, identique |
| downed_all_lose | 1 599 | exit 0, identique |
| Total | **3 996** | **aucune autre différence** |

Les huit appels de dump sont eux-mêmes verts : chacun 1 test réussi, 0 échec,
8 filtrés. Types ignorés : exactement les deux nouveaux types rollback T2.5.
Run, RngStreams, stats, économie et modificateurs ne sont pas ignorés. Les drops
sont désactivés dans ces scénarios : aucun flux loot créé ou consommé.

Preuve effectuée sur le merge `0916514`, dont la simulation est identique à celle
vérifiée dans `c6211ec` (les changements suivants sont docs, goldens et sérialisation
équivalente des RON générés). Script `final-proof.sh`, logs `19-final-proof.log`
et `final-proof/{main,branche,diff}-<nom>.log`. Les huit `.full` ont été supprimés
après leurs comparaisons réussies ; les résultats et traces courtes sont conservés.

### Bless et génération

```bash
source ../env.sh
export CARGO_BUILD_JOBS=4
BLESS=1 make test_scenarios
make gen GAME=zombies
make gen GAME=zombies GEN_BLESS=1
```

- Bless : **58 scénarios, 32 530 frames**, 48 manuscrits et 10 générés ; harness
  **2 réussis, 0 échec, 7 ignorés**, 220,31 s. Six power-ups verts avec leurs attentes.
  Exit 0, log `20-final-bless.log`.
- Génération avec comparaison : **10/10 armes**, 4 à distance et 6 de corps à corps,
  **4 800 frames**, attentes et traces vertes ; exit 0, `21-final-gen.log`.
- Génération avec bless explicite : mêmes **10/10**, attentes vertes, exit 0,
  `22-final-gen-bless.log`. Le changement de sérialisation RON ne change aucune trace
  générée. Justification commise dans `c6211ec`.

### Vérification standard, sans bless, sur c6211ec

```bash
make test_scenarios
cargo test -q --profile headless -p scenario -p run -p combat -p game -p content \
  -p map_ldtk -p sim_core -p stats -p bots -p effects --no-fail-fast
make lint
cargo fmt --all -- --check
./scripts/check-forbidden.sh
./scripts/check-rollback-registration.sh
```

- `make test_scenarios` : **58 scénarios, 32 530 frames**, toutes les attentes et
  traces vertes ; **2 tests réussis, 0 échec, 7 ignorés**, 225,33 s, exit 0.
  Log `23-final-scenarios.log`.
- Tests des crates : **240 réussis, 0 échec, 8 ignorés**, exit 0,
  `24-final-crates.log`. Dont **effects 6/6**, **game 13/13**,
  **content 63/63 + lint_fixtures 10/10** (les trois fixtures PowerUp incluses),
  **expectations 30/30**, **run 13/13**. Le test de relance locale et celui de
  retour au lobby passent : cible d'intégration `run`, **2/2**, 9,91 s.
  La cible scenarios repasse : **2/2, 7 ignorés**, 227,96 s. Le huitième test
  ignoré est un doctest. Les chiffres sont ceux observés, pas ceux du README ancien.
- `make lint` : exit 0, **aucune erreur** dans les deux jeux. Zombies : 4 personnages,
  4 armes, 6 armes de corps à corps, 1 configuration de vagues, 2 cartes ; testbed :
  7 personnages, 4 armes, 6 armes de corps à corps, 0 vagues, 4 cartes.
  `25-final-lint.log`.
- Format : exit 0, sortie vide, `25-final-fmt.log`.
- Interdits : exit 0, **4 avertissements préexistants**, aucun nouveau : 3 HashSet
  (`character/enemy/ai/state.rs`, `map_ldtk/game/plugin.rs`) et 1 commentaire rand
  (`bevy_fixed/math.rs`). `26-final-forbidden.log`.
- Enregistrement rollback : exit 0, **OK**, aucun appel direct hors utils.
  `26-final-registration.log`. Tous les exits sont consignés dans
  `23-final-standard.status`.

### P2P à deux clients

```bash
bash ../verification/final-p2p.sh
```

Build unique `APP_VERSION=x cargo build -q -p zombies --profile headless
--no-default-features`, puis deux exécutions parallèles du binaire compilé avec
`ALACOD_HEADLESS=1`, `ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME=600`, `APP_VERSION=x`,
`--matchbox ws://127.0.0.1:3536 --number-player 2 --players localhost remote`, lobby unique.
Signaling Docker depuis l'image locale existante, projet dédié `alacod-t25-codex`.

**Deux exits 0, 599 lignes chacun, cmp exit 0**, aucune occurrence Desync/ERROR/panicked.
Le conteneur et le réseau dédiés sont arrêtés et retirés. Logs `27-final-p2p.log`,
`27-p2p-build.log`, `27-p2p-cleanup.log`, traces et logs `final-p2p-0/1.*`.
Inputs neutres ; drops et effets exercés par les scénarios synctest et les tests de systèmes.

### Bench au calme

```bash
ALACOD_BENCH_STRICT=1 make test_scenarios && ./scripts/scenario-metrics.py
```

Avant exécution : `vmstat 1 3`, **88 % CPU idle** sur les deux intervalles instantanés,
aucun autre cargo/rustc. Charge et processus dans `28-final-charge.log` et
`28-final-processes.log`. Les deux commandes ont été exécutées successivement,
la seconde seulement après le succès de la première.

**Bench exit 0, métriques exit 0 : 58 scénarios, 32 530 frames, 58 budgets respectés,
0 sous le plancher.** Harness 2 réussis, 0 échec, 7 ignorés, 224,45 s.
Logs `28-final-bench.log`, `28-final-bench.status`, `29-final-metrics.log`.
Métriques du commit sous le target de tâche `metrics/c6211ec/metrics.json`,
copie conservée dans `../verification/final-bench-metrics.json`.

| Scénario | fps mesurées | Plancher |
|---|---:|---:|
| bench_bullets | 108,3 | 70 |
| bench_horde | 68,5 | 38 |
| movement_melee | 133,8 | 80 |
| remote_first_fight | 156,2 | 90 |
| shoot_around | 142,7 | 90 |

### Historique des corrections

Le WIP passait ses 57 goldens hérités, mais une comparaison détaillée sur
points_on_kill révélait une divergence RNG loot à la frame 469. L'isolation du loot,
le checksum des pickups et les autres corrections sont dans `86c5283`.
Les premières relances ont aussi révélé des erreurs de montage des nouveaux tests
(RollbackOrdered absent, attente d'expiration décalée) et un replay perdant les
réglages de loot ; elles ont été corrigées. La suite complète ci-dessus a depuis
été rejouée avec succès sur l'état fusionné. Les résultats intermédiaires rouges
restent consignés dans les logs 01 à 18 et le rapport historique `20fd2c9`.

## 3. Non fait / non vérifié

- Les **8 tests ignorés** n'ont pas été exécutés.
- Vidéos, validation visuelle humaine et compilation avec rendu : non effectuées.
  Aucun sprite de pickup au sol ni indicateur d'effet actif ajouté.
- Le p2p ne teste pas la relance en ligne, toujours non supportée par T2.4 ; il utilise
  les inputs neutres du protocole standard.
- Merge dans main, ligne de journal et retrait du worktree : réservés à l'orchestrateur.
- T2.8 et les fiches suivantes : non commencées.

## 4. Dettes et décisions ouvertes

- Actions sur toute l'équipe, ramassage automatique unique départagé par net_id,
  durée de vie au sol de 1 800 frames dans les tables. Nuke/drop portent sur
  Team::Enemies, y compris les ennemis de laboratoire ; Nuke ne crédite pas de
  points de kill (simplification v0 documentée).
- Les modificateurs de même power-up peuvent se cumuler : Double Points peut
  multiplier davantage. Pas de politique de rafraîchissement ou de non-cumul.
- Les trois règles/fixtures du lint WIP sont conservées. Les validations supplémentaires
  des actions, durées, portées, poids et bornes restent pour T2.8. Le multiplicateur
  de monnaie conserve la conversion Fixed du WIP ; aucune nouvelle garantie pour
  les montants hors de la plage Fixed.
- L'enregistrement général n'a pas été étendu à tous les overrides de joueur.
  Les paramètres nécessaires aux power-ups, au jeu, aux armes et aux vagues sont
  préservés ; replay de Max Ammo et de shoot_around vérifié.
