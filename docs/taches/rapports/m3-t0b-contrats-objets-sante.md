# Rapport M3-T0b — contrats objets et santé

Branche `m3-t0b-contrats-objets-sante`. La dette pose les contrats sans exécuter les points
bloqués par T2/T3/T8.

## Livré

1. `Inventory::count` compte passifs, actif et consommables ; tests unitaires.
2. `items::item_effect_key` réserve la clé historique du premier exemplaire et le suffixe `#k`
   pour les suivants ; aucun branchement dans `listed_effects`.
3. `effects::Scale::PerCopy`, enveloppe d'action `Scaled`, et règle pure `scale_value` ; lint
   réservé aux effets d'objet ; exécution en M3-T2.
4. `items::Price`, `Shelf::cost` optionnel hors hash quand absent, et règles pures `can_pay`/`pay`.
   L'interaction est M3-T8.
5. `CharacterConfig.hit_iframes` et résolveur : un dégât final strictement positif pose la durée,
   sans déclencher sur bouclier ou armure absorbante. Le scénario neuf `hit_iframes` couvre deux tirs
   espacés de dix frames.
6. Q2 non codée : proposition par défaut `ExtraHealth { soul }` absorbé point par point avant les
   PV, demi-cœur = un point, distinct de `Shield`.
7. `game::loot::BOMB_ITEM = "bomb"` réservé.
8. `ItemDef.max_stack` optionnel, absent illimité ; lint du zéro ; exécution du plafond en M3-T3.
9. Q4 non codée : documentées les variantes `Inventory` par joueur et ressource d'équipe.
10. Lint/schema/docs et attentes mises à jour ; §61 complété.

## Q16

Proposition par défaut : limites indépendantes par exemplaire pour tous les objets, avec preuve des
traces gungeon touchées ; alternative `stack: Independent | Shared` par objet. Le branchement est
reporté à M3-T2.

## Vérification et traces

Le scénario `hit_iframes` est une trace neuve (seule bénie). Les traces existantes sont
identiques : les champs sont absents du contenu livré, les types nouveaux sont neutres et les
variantes sont en fin d'enum. Preuve des i-frames : deux balles touchent la cible (`iframes_target`,
`hit_iframes: Some(30)`, 5000 PV) ; elle ne perd que 8 PV (4992). Contrôle sans i-frames
(`hit_iframes: None`) : 4984, soit deux coups.

## Questions pour William

- Q2 : valider `ExtraHealth { soul }` point par point au-dessus de `Health`, demi-cœur = 1 PV,
  sans cœurs noirs.
- Q4 : choisir partage par joueur ou ressource rollback d'équipe pour sous, bombes et clés.
- Q16 : confirmer les limites indépendantes par exemplaire pour tous les objets ou un réglage par objet.

## Vérifications effectuées après fusion

Cause du blocage précédent (erreur M3-T0b, corrigée) : `games/testbed/assets/characters/iframes_target.ron`
omettait `scale`, champ obligatoire de `CharacterConfig` côté jeu mais absent du schéma de contenu
(le lint passait). Le chargement du personnage échouait, la run se terminait, puis
`cleanup_rollback_world_system` retirait `Run` : tout scénario `testbed` paniquait au runner. Les
échecs de `make gen` et `test_crates` venaient de la même cause, pas d'un état préexistant.
`counts_hits` est aussi retiré de `iframes_target` (réservé à la cible de `generate_template`).

- `git fetch && git merge origin/main` : OK, `origin/main` est `5d9f854b`.
- `make test_scenarios SCENARIO=barrel_chain` : OK.
- `make test_scenarios SCENARIO=hit_iframes` : OK après `BLESS=1` (trace neuve seule).
- `make test_scenarios_par SHARDS=3` (sans BLESS) : OK, 369 scénarios, aucune trace existante
  modifiée.
- `make test_crates` : OK.
- `make gen GAME=zombies|testbed|throne|gungeon|isaac` : OK, sans diff.
- `make lint` : OK pour les cinq jeux.
- `cargo fmt --all -- --check` : OK.
- `cargo test -p content` : OK, 262 tests (1 ignoré) ; `make check_sim_boundary` : OK.

## Dette D79 (schéma sans `scale`)

Le schéma de contenu donne `scale` = 1.0 par défaut alors que `CharacterConfig` l'exige : un
personnage sans `scale` passe le lint et casse le chargement du jeu (cause du blocage de cette
tâche). Le rendre obligatoire demande d'ajouter `scale` à 53 fixtures de `crates/content` : essayé,
revenu en arrière, noté `docs/taches/dettes.md` D79.

## Fusion de `origin/main` (c0aceb3e, M3-T1)

Conflits résolus en gardant les deux intentions, variantes de `main` d'abord : `Action::Chance`
puis `Action::Scaled` (`actions.rs`, `describe.rs`, `effects_runtime.rs`, `powerups.rs`), champ
`ItemDef.projectile` de `main` conservé avec `max_stack` après (`items/src/lib.rs`), deux lignes de
journal (`docs/taches.md`). `cargo check --workspace --tests` (features par défaut) échoue sur
`crates/game/src/ui/screens/hui.rs` (champs `merchant*` manquants dans des tests) ; ce fichier est
identique à `origin/main`, la même compilation y échoue donc aussi.

Vérifications après la fusion de `c0aceb3e` : `cargo test -p content` OK ; `make test_scenarios_par
SHARDS=3` (sans BLESS) OK, 391 scénarios, aucune trace existante modifiée, `hit_iframes.trace`
identique à la trace bénie ; `make test_crates` OK ; `make gen` ×5 OK sans diff ; `make lint` ×5
OK ; `cargo fmt --all -- --check` OK ; `make check_sim_boundary` OK. `cargo check --workspace
--tests` : échec reproduit sur `origin/main` (`hui.rs`, champs `merchant*`), `cargo check -p game
--tests` passe sur la branche.
