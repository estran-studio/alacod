# M-1b — Personnage « prop » (voie m, jalon S1 dim 15 : écran propre de la salle des bancs)

Branche `wquintal/m-1b-prop` (partie d'`origin/main` b6bbe24d, `origin/main` fusionné jusqu'à e6b5ce68). Demande : `1837:docs/demandes/prop-image.md`. Section de conventions : **§ 112**. Dette : **D200**.
Code engine contrôlé : `2085d52a` (le commit du rapport ne touche que `docs/`). Rien côté 1837.

## Points du lot

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Champ `prop: bool` (défaut `false`) sur `Character` : pas de barre de vie, pas d'`Enemy`, collisions et dégâts inchangés, `ai` non requis | **fait** | `CharacterConfig::prop` (`crates/game/src/character/config.rs`), chemin séparé au début de `spawn_enemy` (`enemy/create.rs`), `attach_character_visuals` (`visuals.rs`) ; scénarios ci-dessous |
| 2 | Lint : `prop: true` refusé avec `counts_hits`, `starting_weapons`, `boss` ; `describe` ; `new-content --from` | **fait** | `lint_characters` (`crates/content/src/lint.rs`, `Unsupported`) ; `lint_fixtures.rs::prop_fixture_refuses_counts_hits_weapons_and_boss_but_not_a_bare_prop` ; `cli.rs::describe_lists_prop_and_new_content_accepts_a_prop_model` |
| 3 | Simulation : champ statique lu au spawn, aucun composant rollback neuf, aucune trace existante ne bouge | **fait** | cinq scénarios existants rejoués **sans `BLESS`**, traces identiques : `explosive_bullet_hit`, `testbed_window`, `four_players_idle` (vagues, zombies), `rock_flow_field`, `rock_loot` (`make test_scenarios SCENARIO=…`, 5 passed) |
| 4 | Test et scénarios neutres (`prop-test`, `wall-test`) | **fait** | `prop_no_enemy` (5 props + 1 `dummy`, `Enemy` = 0 avant f5 puis 1), `prop_no_enemy_cap` (11 props + 1 `dummy` : ennemis = 1 < 10), `prop_stops_bullet` + témoin `prop_stops_bullet_dummy` (même impact : 200 → 192, la cible derrière reste à 200), test `visuals::prop_tests::a_prop_gets_no_health_bar_but_its_twin_does` ; **traces neuves** seulement (4 `.trace`) |
| 5 | Capture sans barre rouge | **fait, autre outil que prévu** | `docs/taches/rapports/m-1b-prop.png` (image 95 de `play_scenario prop_stops_bullet --capture`, `ALACOD_LABELS=0`) : le prop (rocher) sans barre, HUD « Ennemis 1 ». `alacod sandbox games/testbed --detach` + `alacod remote spawn prop-test --ahead` marche (`brief` : seul le `dummy` est un ennemi) mais `alacod remote screenshot` a rendu une image **toute noire** (1600×1200) deux fois : cause non cherchée |
| 6 | Docs | **fait** | § 112 de `docs/conventions.md`, D200 de `docs/taches/dettes.md`, ligne `LIVRÉ` de `docs/taches.md` ; `CLAUDE.md` inchangé (aucune règle ne change) |

## Constats (lecteurs de `Enemy`, lus dans le code ; seul le compte est prouvé par scénario)

Un prop sans `Enemy` sort de (la séparation ennemi-ennemi **le garde** depuis le suivi P3a, voir plus bas) :
du ciblage (`enemy_target_selection`, charme), du plafond du spawner, des vagues, des salles (`rooms.rs`), du HUD `enemies`, des bots (`hunter`, `explorer`), de `remote brief`, des fantômes et du son de mort. Inchangés : collisions du
joueur (par couche `enemy_layer`), balles (`Team` + `Health` + `Collider`), dégâts, mort.
- **D200** : le plafond du spawner est le littéral `global_max_enemies = 10` de `spawning.rs` ; le champ `EnemySpawnerComponent::max_enemies` n'est lu par aucun système. La demande parle de « `max_enemies: 1` » : la preuve est donc le compte `Enemy`
  (même marqueur que le plafond). Le spawner de vagues ne tourne pas en scénario (`WaveModeEnabled(true)` forcé) : « le spawner continue » n'est **pas** joué, il est déduit du compte.
- **Nom** : `prop: bool` gardé (forme de la demande). Deux champs (`health_bar`, `counts_as_enemy`) n'ont pas de cas d'emploi séparé pour une image, et `prop` voisine le kind `Prop` (barils, tables) : un autre objet, noté au § 112.

## Contrôle rapide

`make check_rapide` vert sur `2085d52a` : 1599 tests passés, 0 échec ; journal `~/orca/runs/alpha/logs/m-1b-prop-check-rapide-2085d52a.log`. Dernières lignes : `test result: ok. 0 passed; 0 failed …` puis
`✓ Contrôle rapide passé (suite de scénarios : la nuit)`. Premier essai (`89c06750`) rouge : `check-forbidden` refusait deux `HashMap` du test de barre de vie ; corrigé sans exception ajoutée. Sweep de `target` joué après le contrôle.

## Non fait / non vérifié

- `make schedule_order_diff` **non joué** : aucun système, aucun schedule, aucun type rollback ajouté (un `if` au spawn et un `if` en présentation).
- Aucune trace existante modifiée ; suite complète de scénarios : la nuit (VPS).
- Capture : un prop **touché** puis vu en jeu par `alacod remote screenshot` reste à refaire quand la capture de fenêtre rend autre chose que du noir ; 1837 recapturera la salle des bancs (`docs/preuves/`).
- Côté 1837 (coordinateur 1837, à l'arrivée du `rev`) : 17 `decor-*.ron` à `prop: true`, retirer `ai`/`base_health` devenus inutiles ; **garder `team: Some(Neutral)`** là où le décor doit arrêter les tirs ennemis (voir le suivi ci-dessous).

## Suivi de revue (P3a, P3b, P3c, équipe) — code `9db8e59c`

Fusion d'`origin/main` (643c4c4e, M-1b mergé) puis un commit de correction. Chaque point est prouvé par un test qui échoue sans le correctif.

| # | Point | Issue | Preuve |
|---|---|---|---|
| P3a | Les ennemis traversaient les props (la séparation de `move_enemies` ne portait que sur les `Enemy`) | **corrigé** | `pathing.rs` : les personnages ni `Enemy` ni `Player` à `CharacterConfig::prop` entrent dans `separation_grid` (nouvelle requête `prop_query`). Scénarios neutres `prop_repels_enemy` (quatre `prop-test` sur la route d'un `follower`) et son témoin `prop_repels_enemy_dummy` (quatre `dummy`) : mêmes distances au joueur, 145,746 à f150 et 96,695 à f199. **Sans le correctif** (`git stash` de `pathing.rs`) : `distance 145.62872 < min 145.72` (f150) et `96.64091 < min 96.68` (f199), trace différente ; ce sont les valeurs d'un couloir vide. L'effet est faible par construction (répulsion ≤ 2 pour une vitesse de 60) : fenêtres de ±0,03, valeurs en point fixe, déterministes |
| P3b | Lint : refuser aussi `variants`, `ai.behaviors` (et `boss`, déjà refusé) sur un prop | **fait** | `lint_characters` ; fixtures `prop_variants` et `prop_behaviors` dans `prop_incompatible` ; `lint_fixtures.rs::prop_fixture_refuses_counts_hits_weapons_boss_variants_and_behaviors_but_not_a_bare_prop` (cinq refus, `prop_ok` sans erreur) : échoue sans les deux `push` (aucune erreur sur les deux fixtures) |
| P3c | `prop_no_enemy_cap` ne fait pas tourner le spawner | **fait par test unitaire** | `spawning::tests::props_beyond_the_cap_do_not_stop_the_spawner` : `enemy_spawn_from_spawners_system` tourne vraiment (une salle, un joueur, un spawner sans délai, onze props posés par `spawn_enemy`) et fait apparaître un `grunt` à chaque frame (1, 2, 3 `Enemy`). Témoin `the_same_count_of_ordinary_enemies_stops_the_spawner` : onze `Enemy` ordinaires, le plafond de dix est atteint, aucune apparition en trois frames. Un scénario est impossible sans changer le runner (`WaveModeEnabled(true)` forcé) |
| Équipe | Un prop sans `team` reçoit `Team::Enemies` : que font les projectiles ennemis ? | **mesuré, non corrigé** | Voir ci-dessous |

**Équipe d'un prop et projectiles ennemis.** Sans `team`, un prop est de l'équipe `Enemies` (repli de `map_ldtk::game::local`, ligne `team.or(character_config.team).unwrap_or(Team::Enemies)`). Les balles du joueur l'arrêtent et le blessent ; les projectiles **ennemis** (même camp, tir ami `Never` par défaut, `team_allows_hit`) le **traversent** sans le toucher.
Sondes jetées (testbed `arena.ldtk`, `tireur_poste` à y 616, joueur à y 696, `prop-test` à y 656, 300 frames) : sans prop le joueur perd 5 PV à f150 et 10 à f299 ; avec le prop en équipe par défaut : **identique** (95 / 90, les flèches passent) ; avec `team: Some(Neutral)` : 100 / 100 (les flèches s'arrêtent, le prop n'est jamais blessé, `Team::Neutral` arrête sans blesser).
Jugement : acceptable pour du décor que seuls les tirs du joueur arrêtent, **mais** la demande 1837 propose de retirer `team` des 17 `decor-*.ron` : à ce moment le décor cesse d'arrêter les tirs ennemis. **À dire au coordinateur 1837 : garder `team: Some(Neutral)` dans les décors qui doivent arrêter les tirs ennemis** (le prop neutre arrête tout et ne meurt jamais). Choix de l'engine possible plus tard : défaut `Neutral` pour un prop sans équipe (ce qui changerait « dégâts inchangés » : un prop neutre n'est jamais blessé) ; non fait, décision de William.

**Contrôle rapide du suivi** : `make check_rapide` vert sur `9db8e59c`, 1601 tests, 0 échec ; journal `~/orca/runs/alpha/logs/m-1b-prop-check-rapide-9db8e59c.log`. Scénarios rejoués sans `BLESS`, traces inchangées : `prop_no_enemy`, `prop_no_enemy_cap`, `prop_stops_bullet`, `prop_stops_bullet_dummy` (déjà livrés), les cinq existants du premier lot, plus `enemy_telegraph` et `tireur_poste_touche_attente` (ennemis et tireur) ; traces **neuves** : `prop_repels_enemy`, `prop_repels_enemy_dummy`. `make schedule_order_diff` non joué : le correctif ajoute un paramètre de requête à `move_enemies`, aucun ordre de système ni type rollback. Sweep de `target` joué (17 Go).
