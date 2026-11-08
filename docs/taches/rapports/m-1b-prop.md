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

Un prop sans `Enemy` sort de : la séparation ennemi-ennemi de `move_enemies` (`pathing.rs`, seule collision molle entre ennemis : un prop ne repousse plus un ennemi voisin ; murs et fenêtres restent `Wall`/`Window`),
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
- Côté 1837 (coordinateur 1837, à l'arrivée du `rev`) : 17 `decor-*.ron` à `prop: true`, retirer `ai`/`team`/`base_health` devenus inutiles.
