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

Le scénario `hit_iframes` est une trace neuve. Les traces existantes doivent rester identiques :
les champs sont absents du contenu livré, les types nouveaux sont neutres et les variantes sont en
fin d'enum. Frames à regarder dans la capture : f10 (premier impact), f20 (second impact ignoré),
f40 (un seul coup compté).

## Questions pour William

- Q2 : valider `ExtraHealth { soul }` point par point au-dessus de `Health`, demi-cœur = 1 PV,
  sans cœurs noirs.
- Q4 : choisir partage par joueur ou ressource rollback d'équipe pour sous, bombes et clés.
- Q16 : confirmer les limites indépendantes par exemplaire pour tous les objets ou un réglage par objet.

## Vérifications effectuées après fusion

- `git fetch && git merge origin/main` : déjà à jour (`origin/main` à `2897e3cb`).
- `cargo check -p content -p game --tests` : OK.
- `cargo test -p content` : OK, 262 tests passés (1 ignoré).
- `make lint` : OK pour zombies, testbed, throne, gungeon et isaac ; avertissements de contenu
  préexistants seulement.
- `make gen GAME=throne`, `make gen GAME=gungeon`, `make gen GAME=isaac` : OK, sans diff.
- `make gen GAME=zombies` et `make gen GAME=testbed` : échec préexistant du générateur :
  cible `target` introuvable dans l'exécution testbed ; aucune génération n'a été bénie.
- `cargo fmt --all -- --check` : OK.
- `make check_sim_boundary` : OK ; 11 crates simulation ECS seul, 5 dettes D69 signalées.
- `make test_crates` : échec hors dette, plusieurs tests scénario paniquent sur la ressource
  `run::Run` absente (`crates/scenario/src/runner.rs:1593`).
- `make test_scenarios_par SHARDS=3` : échec identique sur les trois tranches, avant comparaison
  de traces, sur `run::Run` absente. Les traces existantes n'ont pas bougé.
