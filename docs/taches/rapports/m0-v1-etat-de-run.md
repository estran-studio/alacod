SHA de tête vérifiée avant le commit de ce rapport : `695a7e8a41fbac3484cc94b9f04db5ff708c9fb8`
Fiche : `docs/taches/T2.4-verifier-merger.md` — T2.4, vérification avant merge.
Branche : `m0-v1-etat-de-run` ; référence `main` : `adbd06059eff293368f86e45ffdee3c2dc52deba`.
Date : 2026-10-01 ; agent : Codex.

Le commit de livraison ajoute uniquement ce rapport au SHA vérifié ci-dessus ; son SHA est donné dans la ligne LIVRÉ. Les logs locaux sont dans `/home/wq/Project/bascanada/alacod_tasks/m0-v1-etat-de-run/verification/`.

## Fait

- Repris le worktree existant à `2aee3ab`, propre. Intégré `main` `adbd060` sans conflit dans la branche de tâche (`fe96d3c`), sans modification du checkout principal ni du journal `docs/taches.md`.
- Relu le diff des contrats `Run`/`RunModeRules`, du branchement local/p2p, des systèmes de fin et de relance, du nettoyage LDtk, des attentes et des tests. `Run` passe par `rollback_and_trace_resource`, le choix de config utilise le `BTreeMap` du registre, les handles sont triés. `RunRequest` est consommée en `Update`, jamais dans `GgrsSchedule`. L’abandon écrit hors rollback juste avant la sortie et la destruction de la session, conformément à la limite locale signalée dans la fiche.
- `b012a9d` : ordonné la lecture des soldes du résumé par `GgrsNetId` avec `order_iter!`, conformément à `CLAUDE.md`. La somme saturante reste identique. Corrigé deux affirmations de `docs/conventions.md` §13 : les vagues ne lisent pas `Run.step`, et le résumé est calculé dans la même frame que la fin.
- `695a7e8` : corrigé un échec découvert pendant la suite standard. Le test `parses_explicit_entry_mode` utilisait `mode: Sandbox` sans extension RON ; un `Option<EntryMode>` exige `Some(Sandbox)` dans ce cas. Corrigé le test, documenté la syntaxe et ajouté l’affichage de la durée dans le test de relance. Le chargeur et la simulation ne changent pas dans ce commit.
- Aucune trace `.trace` réécrite, aucun bless supplémentaire, aucun fichier d’arme modifié. Le re-bless de la livraison initiale est maintenant couvert par la preuve indépendante ci-dessous.

## Vérifié

Toutes les commandes Cargo/Make du worktree ont été précédées de :

```bash
cd /home/wq/Project/bascanada/alacod_tasks/m0-v1-etat-de-run/alacod
source ../env.sh
export CARGO_BUILD_JOBS=4
```

Target existant amorcé par `scripts/task-new.sh`, profil `headless`, une invocation Cargo à la fois. Aucun build avec rendu ni target vide. Pour les dumps de référence uniquement, utilisation du checkout principal avec `CARGO_TARGET_DIR` retiré et `CARGO_BUILD_JOBS=4`.

### Preuve trace-diff — README §5

Exécuté un scénario par appel, sans filtrer la sortie (logs complets conservés), des deux côtés :

```bash
# main : cd /home/wq/Project/bascanada/alacod_root/alacod ; unset CARGO_TARGET_DIR
# branche : cd dans le worktree ; source ../env.sh
ALACOD_SCENARIO=<nom> ALACOD_DUMP_TRACE=<dossier-vide> APP_VERSION=x \
  cargo test -p scenario --profile headless --test scenarios scenarios -- --nocapture
python3 scripts/trace-diff.py <dump-main>/<nom>.full <dump-branche>/<nom>.full --ignore Run,RunOutcome
```

Dossiers utilisés : `/tmp/alacod-t24-proof-main-codex` et `/tmp/alacod-t24-proof-branche-codex`. Chaque paire a été comparée avant sa suppression. Les huit appels Cargo ont chacun passé 1 test, avec 0 échec et 8 filtrés. Aucun dump vide ni panique d’écriture.

| Scénario | Frames simulées par exécution | Frames complètes comparées | Résultat, code 0 |
|---|---:|---:|---|
| idle | 1500 | 1499 | identique hors Run/RunOutcome |
| points_on_kill | 600 | 599 | identique hors Run/RunOutcome |
| downed_all_lose | 1600 | 1599 | identique hors Run/RunOutcome |
| two_players_shooting | 300 | 299 | identique hors Run/RunOutcome |

Total : 3996 frames comparées. Référence `main` `adbd060`, branche avec le code de `b012a9d`. Les modifications suivantes ne touchent que les tests et la documentation. La comparaison conserve tous les autres types et neutralise les `Entity` brutes comme prévu par l’outil. Les dumps `.full` (jusqu’à 898 Mo chacun) et leurs dossiers temporaires sont supprimés ; les sorties `diff-*.log` restent dans `verification/`.

### Vérification standard — README §4

| Commande | Résultat réel |
|---|---|
| `make test_scenarios` | code 0 ; 52 scénarios (42 manuscrits + 10 générés), 28880 frames ; 2 tests du harness passés, 7 diagnostics ignorés ; 208,95 s ; aucune divergence, aucun avertissement de budget |
| `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots --no-fail-fast` | premier passage code 101 : 220 passés, 1 échec, 8 ignorés ; seul échec : test RON de mode explicite, corrigé en `695a7e8` |
| même commande, ajout de `-- --nocapture` | relance code 0 : 221 passés, 0 échec, 8 ignorés ; les 52 scénarios rejoués sans divergence |
| `make lint` | code 0 ; aucune erreur pour zombies (4 personnages, 4 armes, 6 mêlées, 1 config de vagues, 2 cartes) et testbed (7 personnages, 4 armes, 6 mêlées, 0 config de vagues, 4 cartes) |
| `cargo fmt --all -- --check` | code 0 ; sortie vide |
| `./scripts/check-forbidden.sh` | code 0 ; 4 avertissements préexistants, aucun nouveau : 3 occurrences HashSet et 1 commentaire rand:: ; contrôlés dans les mêmes fichiers de main avec `git show` |
| `./scripts/check-rollback-registration.sh` | code 0 ; OK, aucun enregistrement direct hors extension |
| `make gen GAME=zombies` | non applicable : aucune arme modifiée ; les 10 scénarios générés existants ont été joués |
| `ALACOD_BENCH_STRICT=1 make test_scenarios` | code 0 ; 52 scénarios, 28880 frames, aucun budget dépassé ; 2 tests passés, 7 ignorés ; 207,39 s |
| `./scripts/scenario-metrics.py` | code 0 ; tableau des 52 scénarios, chiffres ci-dessous |

Détails de la suite verte : `content` 63 unitaires, fixtures de lint 39, `combat` 40, `run` 13, attentes 26, tests de relance 2. Le test `restart_replays_identically_under_ten_seconds` a mesuré **0,045 s** pour revenir en jeu et comparé **300 lignes de trace identiques** après relance. `to_lobby_changes_app_state` passe aussi. Les 8 ignorés sont les 7 outils de diagnostic du harness de scénarios et 1 doctest ; ils n’ont pas été exercés.

La relance des neuf crates a émis trois avertissements de budget non bloquants : `movement_melee` 75,1 < 80 fps, `remote_first_fight` 89,4 < 90, `shoot_around` 76,2 < 90. Le bench strict séparé les a tous passés, respectivement à 133,4, 155,8 et 144,9 fps. Les dumps eux-mêmes ont aussi émis des avertissements de budget ; ces mesures avec dump ne servent pas de bench.

### P2P — deux clients

Build préalable :

```bash
APP_VERSION=x cargo build -q -p zombies --profile headless --no-default-features
```

Code 0. Réutilisé l’image de signaling déjà construite, sous un nom de projet Docker propre au test :

```bash
docker tag alacod-signaling:latest alacod-t24-codex-signaling:latest
docker compose -p alacod-t24-codex -f docker-compose.ci.yaml up -d --no-build signaling
```

Après 3 secondes, lancé les deux processus du binaire ainsi construit, sans deuxième invocation de compilation, dans le même lobby `test-t24-codex-<pid>` :

```bash
ALACOD_HEADLESS=1 ALACOD_STATE_TRACE=<verification>/p2p-<i>.trace \
  ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x \
  timeout 180 "$CARGO_TARGET_DIR/headless/zombies" \
    --matchbox ws://127.0.0.1:3536 --lobby <lobby-commun> --number-player 2 \
    --players localhost remote --cid client_<i> --name client_<i>
cmp <verification>/p2p-0.trace <verification>/p2p-1.trace
wc -l <verification>/p2p-0.trace <verification>/p2p-1.trace
docker compose -p alacod-t24-codex -f docker-compose.ci.yaml down
```

Deux codes de sortie 0 ; `cmp` code 0 ; **599 lignes par client**, 1198 au total ; « p2p : traces identiques ». Recherche dans les deux logs : aucune ligne `Desync`, `desync`, `ERROR` ou `panicked`. Les traces et logs sont conservés dans `verification/`. Conteneur et réseau du test arrêtés et retirés.

### Bench strict — mesures

Aucune autre compilation observée. `vmstat 1 3` avant lancement : deux échantillons instantanés à **87 % et 88 % de CPU libre**, sans swap sortant. Tous les planchers de `tests/budgets.ron` passent, sans modification des budgets. Ceci valide les budgets actuels ; aucun bench complet de main n’a été rejoué pour calculer un delta de performance entre commits.

| Scénario | Frames | FPS | Plancher FPS | Entités max | Balles max | Ennemis max |
|---|---:|---:|---:|---:|---:|---:|
| ammo_burst | 900 | 127.5 | 70.0 | 137 | 5 | 7 |
| ammo_shared_reserve | 700 | 130.6 | 40.0 | 133 | 3 | 7 |
| bench_bullets | 600 | 108.6 | 70.0 | 291 | 155 | 5 |
| bench_horde | 900 | 68.4 | 38.0 | 270 | 0 | 61 |
| bots_four_mixed | 600 | 107.9 | 40.0 | 153 | 11 | 7 |
| bots_two_fonceurs | 900 | 110.7 | 40.0 | 142 | 14 | 7 |
| bullets_walls | 400 | 156.7 | 80.0 | 133 | 8 | 4 |
| buy_door | 220 | 107.6 | 40.0 | 113 | 0 | 1 |
| buy_perk | 160 | 135.6 | 40.0 | 111 | 0 | 0 |
| buy_wall_weapon | 160 | 121.8 | 40.0 | 109 | 0 | 0 |
| dash_wall | 320 | 138.8 | 70.0 | 123 | 0 | 3 |
| door_open | 330 | 103.4 | 45.0 | 123 | 0 | 3 |
| downed_all_lose | 1600 | 136.4 | 40.0 | 140 | 0 | 7 |
| downed_bleedout | 1100 | 141.9 | 40.0 | 138 | 1 | 7 |
| downed_revive | 500 | 148.0 | 40.0 | 135 | 1 | 6 |
| drop_pickup_swap | 300 | 141.6 | 40.0 | 121 | 0 | 2 |
| four_players_idle | 600 | 144.2 | 80.0 | 148 | 0 | 7 |
| four_players_shooting | 300 | 148.9 | 100.0 | 158 | 24 | 2 |
| friendly_fire_cursed | 150 | 165.6 | 40.0 | 123 | 1 | 0 |
| friendly_fire_never | 150 | 164.5 | 40.0 | 126 | 4 | 0 |
| idle | 1500 | 148.1 | 80.0 | 135 | 0 | 7 |
| immune_tag | 150 | 155.8 | 40.0 | 123 | 1 | 0 |
| movement_melee | 900 | 133.4 | 80.0 | 133 | 0 | 7 |
| points_on_kill | 600 | 154.9 | 40.0 | 132 | 5 | 5 |
| remote_first_fight | 600 | 155.8 | 90.0 | 132 | 5 | 5 |
| run_lose_summary | 1600 | 133.3 | 40.0 | 140 | 0 | 7 |
| shoot_around | 1500 | 144.9 | 90.0 | 138 | 9 | 7 |
| shop_tour | 300 | 133.6 | 40.0 | 119 | 0 | 2 |
| stat_move_speed | 150 | 94.5 | 40.0 | 122 | 0 | 0 |
| testbed_ally_safe | 200 | 237.0 | 100.0 | 43 | 8 | 5 |
| testbed_arena_idle | 300 | 234.8 | 110.0 | 35 | 0 | 5 |
| testbed_civilian_blocks | 200 | 236.9 | 100.0 | 38 | 4 | 5 |
| testbed_corridor_idle | 120 | 295.0 | 140.0 | 18 | 0 | 0 |
| testbed_dummy_shoot | 180 | 232.8 | 110.0 | 38 | 3 | 5 |
| testbed_follower | 250 | 239.1 | 110.0 | 35 | 0 | 5 |
| testbed_target_hits | 220 | 233.4 | 110.0 | 42 | 7 | 5 |
| testbed_two_rooms_door_idle | 120 | 284.9 | 140.0 | 18 | 0 | 0 |
| testbed_window | 450 | 252.2 | 130.0 | 18 | 0 | 1 |
| two_players_idle | 1500 | 137.0 | 80.0 | 140 | 0 | 7 |
| two_players_shooting | 300 | 155.9 | 100.0 | 133 | 11 | 2 |
| weapon_axe | 400 | 218.6 | 40.0 | 33 | 0 | 5 |
| weapon_bare_hands | 400 | 223.9 | 40.0 | 33 | 0 | 5 |
| weapon_club | 400 | 218.6 | 40.0 | 33 | 0 | 5 |
| weapon_knife | 400 | 226.8 | 40.0 | 33 | 0 | 5 |
| weapon_machine_gun | 600 | 226.9 | 40.0 | 39 | 7 | 5 |
| weapon_pistol | 600 | 234.4 | 40.0 | 34 | 2 | 5 |
| weapon_rifle | 600 | 230.9 | 40.0 | 34 | 2 | 5 |
| weapon_shotgun | 600 | 233.0 | 40.0 | 43 | 11 | 5 |
| weapon_sword | 400 | 222.9 | 40.0 | 33 | 0 | 5 |
| weapon_zombie_claws | 400 | 221.9 | 40.0 | 33 | 0 | 5 |
| weapons_workout | 950 | 146.4 | 100.0 | 138 | 8 | 7 |
| window_repair | 1100 | 137.0 | 100.0 | 133 | 0 | 7 |

## Non fait / incertain / non vérifié

- Merge dans main, vérification après merge, mise à jour du journal et suppression du worktree/target : réservés à l’orchestrateur. Checkout principal resté propre sur main.
- Smoke visuel de relance et retour au lobby : non fait (facultatif dans la fiche). Aucun rendu, capture ni vidéo produit pour cette vérification.
- Restart synchronisé en p2p : non supporté par ce chantier ; la commande renvoie au lobby. Le p2p exécuté vérifie la session courante, pas une relance entre pairs.
- Les quatre preuves comparent la simulation existante ; pas de preuve sans `--ignore` supplémentaire ni de nouveau bless.

## Dettes laissées / questions ouvertes

- Fixture pour `entry.mode: Some(Waves)` sans contenu Wave toujours manquante : prévue en T2.8, conformément à la fiche ; règle de lint présente.
- Les vagues continuent après `Run.step: Ended` tant que les conditions actuelles de spawn le permettent. Limitation documentée ; la corriger changerait le gameplay et exigerait une autre preuve/bless.
- L’abandon via `ToLobby` ne calcule pas de résumé conservé avant la destruction de Run. Limitation déjà décrite dans `run_state.rs`.
- L’UI possède le bouton Rejouer et la touche R, mais aucun bouton Lobby visible ; `ToLobby` est disponible et testé comme commande. Présentation à compléter dans T2.12.
- Bornes de scénario `RunSummary` limitées à vague/kills : pas d’attente exacte pour points/durée/outcome. L’unitaire du mode vérifie leur copie ; aucune nouvelle attente ajoutée ici.
