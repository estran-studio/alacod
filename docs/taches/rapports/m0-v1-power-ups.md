# T2.5 — reprise des power-ups, livraison intermédiaire bloquée

SHA de la tête de code vérifiée : `86c5283d07b10e2ce9df9c8416f8580870a14df1`.
Fiche : `docs/taches/T2.5-reprise-power-ups.md` du checkout principal.
Date : 2026-10-01. Agent : Codex.
Ce SHA précède le commit de ce rapport ; le SHA livré inclut ensuite le rapport.

**Statut : incomplet, bloqué sur le merge de T2.4 par l'orchestrateur.** La fiche exige
« Faire T2.4 (merge) avant » puis une fusion de main dans cette branche. Le checkout
principal et `origin/main` sont toujours à `adbd06059eff293368f86e45ffdee3c2dc52deba` ;
ils ne contiennent pas la livraison T2.4 `d246a205d7df8fa4df46b60691c74ecb6ebd186d`
(PR #54). Une demande de transmission a été adressée à William pendant le travail.
L'attente dépasse 30 minutes. Aucun merge dans main, journal ou retrait de worktree
n'a été effectué. Aucune fiche suivante n'a été commencée.

## 1. Fait

Reprise du WIP `9866a6a`, sur son worktree existant
`/home/wq/Project/bascanada/alacod_tasks/m0-v1-power-ups/alacod`, avec son target
amorcé existant (33 Go au départ). Lecture du préambule, de la fiche, de CLAUDE.md,
des conventions pertinentes et du diff de code/contenu du WIP. Une compilation à la
fois, `source ../env.sh`, `CARGO_BUILD_JOBS=4`, profil headless ; build zombies sans
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
- Les 56 scénarios autres que `powerup_drop_on_kill` désactivent explicitement les
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

**Aucune trace n'a été réécrite pendant cette reprise.** Les 57 traces du WIP restent
présentes mais ne constituent pas des références finales validées. Le checkout
principal reste propre, sur main.

## 2. Vérifié

Les logs persistants sont hors git dans `../verification/` du worktree. Les trois dumps
complets de diagnostic (environ 324 Mo chacun) ont été supprimés après comparaison ;
leurs commandes, résultats et logs sont conservés. Aucun résultat ci-dessous ne
prétend valider l'état après fusion avec T2.4, qui n'existe pas encore.

### Scénarios sans bless

- `source ../env.sh; export CARGO_BUILD_JOBS=4; make test_scenarios` sur le WIP initial :
  **57 scénarios, 30 930 frames**, 47 manuscrits + 10 générés ; 2 tests de harness verts,
  0 échec, 7 ignorés, 210,34 s. Log `01-wip-scenarios.log`. Cela vérifie uniquement la
  cohérence du WIP avec ses propres goldens.
- Même commande après les corrections de simulation : 57 scénarios atteignent leur
  dernière frame et satisfont leurs attentes, dont les six power-ups. Les **57 traces
  divergent dès la première ligne**. Harness : 0 passé, 2 échoués, 7 ignorés, 213,49 s
  (`06-corrected-scenarios.log`). Le second échec, replay perdant la chance de drop,
  a ensuite été corrigé et la vérification standard confirme sa résolution.

Liste exacte des traces actuellement rouges :

```text
ammo_burst, ammo_shared_reserve, bench_bullets, bench_horde,
bots_four_mixed, bots_two_fonceurs, bullets_walls, buy_door, buy_perk,
buy_wall_weapon, dash_wall, door_open, downed_all_lose, downed_bleedout,
downed_revive, drop_pickup_swap, four_players_idle, four_players_shooting,
friendly_fire_cursed, friendly_fire_never, idle, immune_tag, movement_melee,
points_on_kill, powerup_carpenter, powerup_double_points, powerup_drop_on_kill,
powerup_insta_kill, powerup_max_ammo, powerup_nuke, remote_first_fight,
shoot_around, shop_tour, stat_move_speed, testbed_ally_safe, testbed_arena_idle,
testbed_civilian_blocks, testbed_corridor_idle, testbed_dummy_shoot,
testbed_follower, testbed_target_hits, testbed_two_rooms_door_idle,
testbed_window, two_players_idle, two_players_shooting, weapons_workout,
window_repair, weapon_axe, weapon_bare_hands, weapon_club, weapon_knife,
weapon_machine_gun, weapon_pistol, weapon_rifle, weapon_shotgun, weapon_sword,
weapon_zombie_claws
```

### Tests des crates et relances nécessaires

```bash
cargo test -q --profile headless -p scenario -p run -p combat -p game -p content \
  -p map_ldtk -p sim_core -p stats -p bots -p effects --no-fail-fast
```

Première exécution : **221 passés, 6 échoués, 8 ignorés** (`07-standard-tests.log`).
Les échecs sont les quatre tests de systèmes dont le montage manquait la ressource
GGRS `RollbackOrdered`, l'attente d'expiration du placement observant le compteur du
runner une frame trop tôt, et la cible scenarios pour ses 57 traces.
Les deux erreurs de montage/attente ont été corrigées puis relancées :

- `cargo test -q --profile headless -p game -p effects` : **game 9/9, effects 6/6**, 0
  échec, 1 doctest ignoré (`08-unit-recheck.log`).
- `cargo test -q --profile headless -p scenario --test expectations powerup_` :
  **4 passés, 0 échoué, 26 filtrés**, 13,55 s (`09-powerup-expectations-recheck.log`).
  Le replay de Max Ammo, la disparition à la durée configurée et les deux attentes
  WIP de comptage sont verts.

Bilan après relances ciblées, **sans nouvelle exécution intégrale** : 226 tests
réussis, 1 cible encore rouge (scenarios), 8 ignorés, en comptant chaque test une seule
fois. Dans cette branche avant fusion, content compte 62 tests unitaires et 10 tests
`lint_fixtures`, tous verts ; les trois fixtures power-ups font partie des 10.
`expectations` compte 30 tests : les 29 autres étaient verts lors de la première
exécution, puis le test d'expiration a passé la relance. Les nombres du README
pour le main plus récent ne décrivent pas encore cette ancienne base de branche.
Le harness `recording_replays_identically` est vert dans la suite standard : la
cible scenarios compte 1 passé, 1 échoué, 7 ignorés, 212,27 s.

### Lint, formatage, scripts

- `make lint` : exit 0 ; **aucune erreur** sur zombies (4 personnages, 4 armes, 6 armes
  de corps à corps, 1 config de vagues, 2 cartes) et testbed (7 personnages, 4 armes,
  6 armes de corps à corps, 0 config de vagues, 4 cartes). Log `10-lint.log`.
- `cargo fmt --all -- --check` : exit 0, sortie vide (`11-fmt.log`).
- `./scripts/check-forbidden.sh` : exit 0, **4 avertissements préexistants**, identiques
  au checkout principal : 3 HashSet (state.rs et plugin.rs), 1 commentaire rand dans
  bevy_fixed/math.rs. Aucun nouvel avertissement (`12-forbidden.log`).
- `./scripts/check-rollback-registration.sh` : exit 0, **OK**, aucun enregistrement
  direct hors utils (`13-registration.log`).
- `make gen GAME=zombies` non lancé : aucune définition d'arme du jeu n'a changé.
  Les dix scénarios générés ont été exécutés dans les suites ; leurs goldens restent
  rouges avec les autres. Régénérer leurs fichiers lors de la finalisation si nécessaire.

### Diagnostics trace-diff, provisoires

Dumps séparés, une seule compilation à la fois, via :

```bash
ALACOD_SCENARIO=points_on_kill ALACOD_DUMP_TRACE=<dossier-vide> APP_VERSION=x \
  cargo test -p scenario --profile headless --test scenarios scenarios -- --nocapture
python3 scripts/trace-diff.py <dump-main>/points_on_kill.full \
  <dump-branche>/points_on_kill.full \
  --ignore 'PowerUpPickup,FrameEvents<game::powerups::PowerUpPickedUp>'
```

Sur le principal, `CARGO_TARGET_DIR` était retiré ; dans la tâche, env.sh était sourcé.
Types ignorés : uniquement les deux nouveaux types rollback de T2.5.

- WIP `9866a6a` contre main **avant T2.4**, `adbd060` : les deux dumps sont produits par
  un test passé (1/1, 8 filtrés), mais comparaison **exit 1**, première divergence
  à la **frame 469**, `RngStreams` ajoute `loot`. Les graines `waves`/`weapons` restent
  égales. Logs `02-diagnostic-wip`, `03-diagnostic-main`, `04-diagnostic-diff`.
- Code corrigé `86c5283` contre ce même main : dump produit malgré l'échec attendu
  sur son golden (0 passé, 1 échoué, 8 filtrés, 7,70 s). Comparaison **exit 0,
  599 frames identiques**, mêmes deux types ignorés. Logs `14-diagnostic-corrected`,
  `15-diagnostic-corrected-diff`.

Ce second diagnostic valide l'isolation corrigée du loot sur ce scénario ; **il
n'autorise aucun bless à ce stade**. Les quatre preuves contre main contenant T2.4
restent à faire, comme prescrit par la fiche.

### P2P

`bash ../verification/p2p.sh` : build `APP_VERSION=x cargo build -q -p zombies
--profile headless --no-default-features`, puis serveur signaling Docker utilisant
l'image locale existante (aucune compilation Docker). Projet dédié
`alacod-t25-codex`, lobby unique. Deux exécutions parallèles du binaire déjà compilé,
avec `ALACOD_HEADLESS=1`, `ALACOD_STATE_TRACE`, `ALACOD_EXIT_AT_FRAME=600`, `APP_VERSION=x`,
`--matchbox ws://127.0.0.1:3536 --number-player 2 --players localhost remote`.

**Deux exits 0, 599 lignes chacun, cmp exit 0**, aucune occurrence Desync/ERROR/panicked
dans les deux logs. Le conteneur et le réseau dédiés ont été retirés. Logs et traces
`16-p2p*`, `p2p-0/1.*`. Ce test utilise des inputs neutres ; les drops et effets sont
exercés par les scénarios en synctest et les tests de systèmes.

### Bench au calme

Après fin de toute compilation et du p2p, `vmstat 1 2` mesure 87 % de CPU idle sur
l'intervalle instantané (79 % sur la première ligne cumulée). Aucun autre cargo/rustc.

```bash
ALACOD_BENCH_STRICT=1 make test_scenarios
./scripts/scenario-metrics.py
```

Bench **exit 2**, dû exclusivement aux **57 traces non reblessées** ; harness 1 passé,
1 échoué, 7 ignorés, 209,21 s. Les **57 scénarios**, 30 930 frames, satisfont leurs attentes
et **tous les 57 budgets**, aucun plancher non respecté. Ceci est un relevé de
performance réussi, pas une commande de bench verte. Le script de métriques est
lancé séparément après cet échec : **exit 0**, tableau de 57 lignes. Logs
`17-charge-avant-bench`, `17-bench-strict`, `17-bench-status`, `18-bench-metrics` ;
copie JSON `bench-metrics.json`, métriques sous `target/metrics/86c5283/`.

| Scénario | fps mesurées | Plancher |
|---|---:|---:|
| bench_bullets | 109.9 | 70 |
| bench_horde | 69.2 | 38 |
| movement_melee | 134.2 | 80 |
| remote_first_fight | 157.3 | 90 |
| shoot_around | 148.8 | 90 |

## 3. Non fait / non vérifié

- Merge T2.4 dans main (orchestrateur), puis fusion de ce main dans la branche T2.5.
- Preuves finales sur idle, two_players_shooting, points_on_kill, downed_all_lose
  contre le **main après T2.4**. Le diagnostic 599 frames ci-dessus est provisoire.
- Bless justifié, nouvelle vérification standard entièrement verte, puis p2p/bench
  de l'état fusionné. T2.5 **n'est pas terminée** ; les traces restent rouges.
- Vidéos, validation visuelle humaine, sprite de pickup au sol et indicateur d'effet
  actif. Les commentaires ne prétendent plus montrer un sprite inexistant.
- Merge final, ligne de journal et suppression du worktree : réservés à l'orchestrateur.

## 4. Dettes et décisions ouvertes

- Décisions WIP conservées : actions à toute l'équipe ; ramassage automatique et
  unique ; départage par net_id ; durée de vie au sol 1 800 frames dans les tables ;
  Nuke/drop sur Team::Enemies et non uniquement WaveEnemy ; Nuke ne crédite pas de
  points de kill, simplification v0 documentée.
- Des modificateurs de même power-up peuvent s'empiler lors de ramassages répétés
  (Double Points peut se multiplier davantage). Une politique de rafraîchissement
  ou de non-cumul n'est pas implémentée dans cette reprise.
- Le lint WIP conserve ses trois règles/fixtures ; les validations supplémentaires
  des actions, durées, portées, poids et bornes relèvent de la reprise T2.8. La
  conversion Fixed du multiplicateur de points reste celle du WIP, sans nouvelle
  garantie pour des montants hors plage Fixed.
- L'enregistrement général n'a pas été étendu à tous les réglages de joueur ; seules
  les configurations nécessaires aux power-ups/armes/vagues et au jeu sont préservées
  ici. Le replay du scénario Max Ammo et celui de shoot_around ont été exécutés.
- Les 57 goldens hérités du WIP n'ont toujours pas la preuve finale requise. Ne pas
  merger cette branche avant sa finalisation et sa nouvelle livraison.
