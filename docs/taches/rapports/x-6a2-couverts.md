# X-6a2 — Couverts qui arrêtent les balles de toutes les équipes (voie m, jalon S3 dim 29 nov, lancé en avance le 8 nov)

Branche `wquintal/x-6a2-couverts` (partie d'`origin/main` 58126c8a). Plan : `docs/taches/plan-1837-cli.md` § 6, ligne X-6a2 (sa preuve fait foi). Section de conventions : **§ 118**. Dette : **D213**.
Code engine contrôlé : `72eb3a98` (le commit du rapport ne touche que `docs/`). Rien côté 1837, rien de 1837 dans l'engine (testbed : `couvert`, `couvert_fragile`).
Jalon servi : **S3 dim 29 nov** (l'extérieur du prologue : « apprendre à bouger, frapper et tirer à couvert »). Hors lot, non fait : l'IA qui se met à couvert (D3, semaine du 30 nov).

## Points du lot

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Relevé avant tout code (dix lignes `fichier:ligne`) | **fait** | envoyé au coordinateur en `status` avant la première ligne de code ; repris dans le § 118 : `weapons/mod.rs:2117-2129` et `:2202-2213`, `team.rs:44-56`, `damage.rs:58-64`, `projectile.rs:1222-1243` (+ `nearest_target` :886), `weapons/mod.rs:2169-2186` (murs), `collision_grid.rs:131-147`, `map_ldtk/src/game/local.rs:347`, `enemy/create.rs:91-112`, `health/mod.rs:209-248`. Constat : balle ennemie contre un prop `Enemies` = **traverse** ; contre un prop `Neutral` = arrêtée sans dégât ; prop `Players`/`Allies` = traversé par le joueur ; personnage sans `Team` = jamais candidat |
| 2 | Un couvert, marqué en donnée, arrête les balles de toutes les équipes | **fait** | `CharacterConfig::cover: Option<sim_core::cover::Cover>` (`Indestructible` \| `Destructible`), composant posé par `spawn_enemy` ; `BulletTarget::Cover` (`bullet_rollback_collision_system`) et `projectile_collision_system` : toujours candidat, sans règle d'équipe ni invulnérabilité ; scénarios `couvert_toutes_equipes` (est) et `couvert_toutes_equipes_miroir` (ouest) |
| 3 | Bloque les corps comme M-1b | **fait, par construction** | `cover` exige `prop: true` (lint) : même chemin de spawn (couche `enemy_layer`, grille de séparation de `move_enemies`) ; pas de scénario neuf sur le corps (inchangé : `prop_repels_enemy` rejoué dans la suite complète) |
| 4 | Destructible en donnée (santé existante) | **fait** | `Destructible` : `rollback_resolve_damage_events` appelle `combat::damage::resolve_defenses` (`resolve_damage` moins la règle d'équipe) ; `couvert_fragile` : 24 PV → 16 (f30) → 8 (f32) → détruit (f37), le `dummy` derrière à 200 à f40 puis 184 ; `couvert_fragile_ennemi` : les flèches ennemies le blessent (24 → 19 → 14) |
| 5 | Balles perforantes et projectiles à `Pierce` | **fait** | un couvert les arrête (`Piercing` : `BulletTarget::Cover` ; `Pierce` : `projectile.ended`) ; `couvert_arrete_perforante` (mode `piercing` du `shotgun` : `dummy` derrière à 200) et `couvert_arrete_projectile_perforant` (`proj_pierce` : 200), chacun avec son témoin à `prop-test` (180 et 190) |
| 6 | Aucune trace existante modifiée ; type rollback neuf seulement si nécessaire | **fait** | `Cover` est un composant statique hors rollback, absent par défaut (comme `Team`) : **aucun type rollback**. Suite **complète** de scénarios (`cargo test -p scenario --profile headless --test scenarios`, 434 `.ron` + générés) rejouée sans `BLESS` : `test result: ok. 5 passed`, aucune trace existante changée (`git status` : seuls des fichiers neufs) |
| 7 | Scénarios neutres, lint, `describe` | **fait** | 10 scénarios neufs `couvert_*` (traces neuves) ; fixture `cover_requires_prop` (`lint_fixtures.rs::cover_fixture_refuses_a_cover_that_is_not_a_prop`) ; `describe Character` liste `cover` (`cli.rs`) ; test `combat::damage::tests::resolve_defenses_skips_the_team_rule_but_keeps_the_defenses` ; `alacod lint games/testbed` vert (dans `make check_rapide`) |
| 8 | Docs : § 118, D213, rapport, LIVRÉ | **fait** | `docs/conventions.md` § 118 (+ renvoi dans § 112, « constat non corrigé » devenu « corrigé par X-6a2 »), `docs/taches/dettes.md` D213, `CLAUDE.md` (liste des attentes : `EntityCount(kind: Cover)`), ce rapport, ligne `LIVRÉ` de `docs/taches.md` |

## Choix et pourquoi

- **Marquage explicite, pas une règle « tout prop »** : une règle générale aurait fait arrêter les balles ennemies par tous les props existants (testbed `prop-test`, décors de 1837) et déplacé leurs traces (le brief le demande en dernier recours). Un champ `cover` absent ne change rien.
- **`cover` sur un personnage `prop: true`**, pas sur le kind `Prop` (barils, tables) ni une entité LDtk : c'est le chemin que 1837 utilise déjà pour ses décors (M-1b). Un couvert de carte reste possible par `CharacterSpawn` + personnage ; l'entité dédiée est notée en D213 (comme D201).
- **Deux valeurs (`Indestructible`, `Destructible`) plutôt qu'un booléen** : sans elles, « indestructible » n'existe que par une grosse `base_health` (§ 112) ou `team: Neutral`, et `Neutral` ne dit pas « toutes équipes la blessent ».
- **Dégâts : `resolve_defenses`**, extraction pure de la fin de `resolve_damage` (même résultat pour tout le reste, prouvé par la suite complète) : un `DamageEvent` visant un couvert destructible ignore la règle d'équipe (sinon la balle ennemie s'arrête sans le blesser) mais pas les défenses de la cible.

## Vérifié

- Suite complète de scénarios sans `BLESS` : verte, traces existantes identiques. Les neuf `couvert_*` rejoués sans `BLESS` après écriture des traces : `5 passed` (le filtre compte les tests du binaire, pas les scénarios).
- Témoins : sans `cover`, mêmes positions, les flèches traversent le prop (joueur 100 → 90 après deux flèches, `couvert_temoin_prop`) et les perforantes touchent le `dummy` derrière (`couvert_arrete_*_temoin`) : les scénarios à couvert **ne sont pas creux**.
- Rollback (demande du coordinateur) : les neuf `couvert_*` rejoués en synctest à `ALACOD_CHECK_DISTANCE=6` (le défaut est 2 ; 8 est refusé par GGRS, `start_synctest_session` panique) : `5 passed`, mêmes traces, aucun `synctest mismatch`. `couvert_fragile` détruit le couvert à f37 (`despawn_rollback`) puis les balles suivantes le traversent : la résurrection par rollback garde `Cover` (le composant vit sur l'entité désactivée, pas dans l'instantané).
- `make check_rapide` vert sur `72eb3a98` : 1630 tests passés, 0 échec ; journal `~/orca/runs/alpha/logs/x-6a2-couverts-check-rapide-72eb3a98.log`. Dernières lignes : `test result: ok. 0 passed; 0 failed …` puis `✓ Contrôle rapide passé (suite de scénarios : la nuit)`, `exit=0`. Sweep de `target` joué après le contrôle (18 Go).

## Non fait / non vérifié

- `make schedule_order_diff` **non joué** : aucun système, aucun schedule, aucun type rollback ajouté ; seuls changent deux filtres de collision et un `match` du résolveur de dégâts.
- Pas de preuve p2p (`make test_multiplayer`, `p2p-*.sh`) : le composant n'est pas dans le checksum, la simulation n'a pas de nouvelle source de non-déterminisme (pas de RNG, `BTreeMap`/tri par `GgrsNetId` inchangés) ; non joué.
- D213 : contact à la position d'arrivée (pas de segment), ordre des `GgrsNetId` pour les balles perforantes, points au coup au but sur un couvert destructible, pas de couvert de carte dédié. Aucun n'est atteint par les valeurs du testbed.
- Côté 1837 (coordinateur 1837, à l'arrivée du `rev`) : poser `prop: true, cover: Some(Indestructible)` (ou `Destructible`) sur les décors qui doivent arrêter les balles ; `team: Some(Neutral)` n'est plus nécessaire pour cela.

## Suivi de revue (P2, P3)

- **P2** : un projectile composable à `Split` traversait un couvert (les enfants naissaient dans le couvert et héritaient de `projectile.hits`, qui le contenait). Correctif : pas de `Split` quand la cible est un couvert (`projectile_collision_system`, `filter(|_| !is_cover)`). Scénario `couvert_split` (trace neuve) : **échoue sans le correctif** (3 larmes vivantes à f46, trois cibles touchées), passe avec. 62 scénarios (`tears_*`, `item_*`, `champion_*`, `bench_tears*`, `prop_*`, `couvert_*`, `familiar_*`, `isaac_duo`, `isaac_quad`) rejoués sans `BLESS` : traces identiques. Dix scénarios `couvert_*` au total.
- **P3** : § 118 corrigé : la mêlée garde son filtre d'équipe ; les explosions (`bombs.rs::explosion_system`) et les ticks de statut (`status_tick_system`) atteignent un couvert `Destructible` de toute équipe (branche `Cover` de `rollback_resolve_damage_events`). Noté en D213 (3 bis).
- **Contrôle rapide du suivi** : `make check_rapide` vert sur `c6fd9bf0` (1630 tests, 0 échec), journal `~/orca/runs/alpha/logs/x-6a2-couverts-check-rapide-c6fd9bf0.log` ; sweep de `target` joué. Le commit suivant ne touche que ce rapport.
