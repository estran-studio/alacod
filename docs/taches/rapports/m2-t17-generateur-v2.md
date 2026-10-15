# Rapport m2-t17-generateur-v2 — générateur v2 : gabarits par objet, par salle et par boss (M2-T17, V3)

Branche `m2-t17-generateur-v2` (depuis `origin/main` 2936b72), session cloud. Conventions : §54.

## 1. Fait

- **`PlayerScript::items`** (nouveau, vide par défaut, absent du RON d'un scénario existant) : le joueur
  tient les objets listés **avant la première frame**, posés comme un ramassage (`scenario::runner::
  apply_player_overrides` : passif → `Inventory` + modificateurs `item:<id>` ; actif → emplacement actif
  **chargé** ; consommable → compteur). C'est le « chemin des objets de départ » de la fiche : il n'en
  existait pas (seul `items:` du scénario, qui pose un objet au sol à ramasser), je l'ai créé sur le modèle
  de `mutations`. Les effets `OnPickup` ne sont pas rejoués.
- **Trois gabarits** (`crates/scenario/src/generate.rs`) : `ItemHeld` (`item_<id>.ron`), `RoomClear`
  (`room_<id>.ron`), `BossFight` (`boss_<id>.ron`). Données : `game::gen_test` (`ItemTest`, `RoomTest`,
  `BossFightTest`) ; les fichiers `objets/*.ron` et `rooms/*.ron` restent lus par leurs crates de contrat
  (champ `test` ignoré) et relus par le générateur pour ce seul champ. Le `test:` d'un personnage gagne
  `boss: Some(())` (le boss est un personnage) ; ce gabarit est un troisième gabarit actif à côté de
  `still`/`moving`.
- **Attentes par défaut** (§54), toutes dérivées du contenu : objet → `HasItem` f1, une `Stat` par stat
  modifiée (attendu calculé par `resolve` sur la valeur de base lue par une sonde **sans** l'objet),
  actif : `ItemCharge(requis)` f5 puis `ItemCharge(0)` f16 ; salle → moments clés `room` « verrouillée » puis
  « nettoyée » + `PlayerAlive` (seulement `PlayerAlive` si elle ne verrouille pas) ; boss → un moment clé
  `boss_phase` par phase + « salle boss nettoyée ». `test.expect` s'**ajoute** aux défauts (comme les armes).
- **Lint** : `lint_item_tests`, `lint_room_tests`, `lint_character_tests` étendu ; 6 fixtures
  (`item_test_frames_zero`, `item_test_consumable`, `room_test_no_template`, `room_test_both_scenes`,
  `room_test_frames_zero`, `boss_fight_without_boss`), toutes dans `lint_fixtures.rs`.
- **Contenu** : `test:` sur **tous** les objets non consommables de `gungeon` (7) et `testbed` (16), les types
  de salle `combat` (+ `depart` ; + `recompense` au testbed) et le boss `gatling`. Chacun joue en moins de
  41 s (mesuré, ≤ 700 frames). Fichier ajouté : `games/testbed/assets/floors/salle_combat.ron` (étage d'une carte, voir § 4).
- **Budgets** (`tests/budgets.ron`) : planchers cloud pour les nouveaux scénarios (avertissement seulement).
- `docs/conventions.md` §54.

## 2. Scénarios générés et frame où chaque attente passe

Toutes les attentes ont été jouées **avant** le bless (`alacod-gen --play`, état « ok » / trace « absente »),
puis `alacod-gen --bless` ; frames relevées par `ALACOD_EVENTS=1`.

| Scénario | Frames | Attentes → frame où elles passent |
|---|---|---|
| `gungeon/item_bottes`, `testbed/item_bottes` | 60 | `HasItem` f1 ; `Stat MoveSpeed 187.5` f1 (150 × 1,25) |
| `gungeon/item_fiole` (Rooms 2), `item_pierre_de_soin` (Rooms 1), `testbed/item_fiole` (Rooms 1) | 60 | `HasItem` f1 ; `ItemCharge` requis f5 ; `ItemCharge(0)` f16 (voir § 4) |
| `testbed/item_braise`, `item_eclat`, `item_canon` (`Frames(5)`) | 60 | `HasItem` f1 ; `ItemCharge(5)` f5 (pas de retombée vérifiée : `Frames` se recharge, `canon` a un `cost`) |
| 12 passifs à effets seuls (gungeon : amulette, cartouchière, étendard, trèfle ; testbed : anneau_salle, bandoulier, bandoulier_comptee, boussole, lame, manche, porte_bonheur, sceau, semelles, semelles_lentes, talisman) | 60 | `HasItem` f1 seulement (aucun `modifiers`) |
| `gungeon/room_combat` (`etages_court`, graine 1) | 400 | `combat` verrouillée f59, nettoyée f329 ; `PlayerAlive` f399 |
| `gungeon/room_depart` | 120 | `PlayerAlive` f119 (salle qui ne verrouille pas) |
| `testbed/room_combat` (`salle_combat`, graine 1) | 700 | verrouillée f65, nettoyée f534 ; `PlayerAlive` f699 |
| `testbed/room_depart`, `room_recompense` | 120 | `PlayerAlive` f119 |
| `gungeon/boss_gatling` (`fonceur`, partie du manifeste) | 600 | phase 0 → 1 f322, 1 → 2 f382, « salle boss nettoyée » f402 (salle de combat verrouillée f63, nettoyée f181, boss verrouillée f218 : non assertés) |

Total créé : 29 scénarios — `gungeon` 10 (7 objets, 2 salles, 1 boss), `testbed` 19 (16 objets, 3 salles). `zombies` et
`throne` : aucun changement (aucun `test:` M2).

## 3. Vérifié (résultats réels)

- `make lint` : 4 jeux « aucune erreur ».
- `cargo test -p content --test lint_fixtures` : 102 réussis (dont les 6 nouvelles fixtures).
- `make test_crates` : 666 réussis, 0 échec, 10 ignorés (toutes les crates demandées).
- Suite de scénarios **sans `BLESS`**, `scripts/test-scenarios-sharded.sh 4` (mêmes options que
  `make test_scenarios`, 4 tranches) : 4 tranches « ok », 159 scénarios manuscrits + 120 générés, 809 s.
  Aucun `.trace` existant n'a bougé (`git status` : seules `room_combat` créées par cette tâche ont été
  réécrites, après un changement de durée, avant le commit de la tâche).
- `make gen GAME=zombies|testbed|throne|gungeon` : aucune ligne d'échec, aucune trace « différente »,
  `git status` propre après régénération (×4).
- `cargo fmt --all -- --check` : propre. `./scripts/check-rollback-registration.sh` : OK.
  `./scripts/check-forbidden.sh` : 6 avertissements, **identiques sur `origin/main`** (mesuré dans un worktree) ;
  aucun ajouté.

## 4. Décisions, pièges, écarts à la fiche

- **Délai d'entrée de 6 frames.** `UseActive` scripté en `from: 10, to: 11` n'agit qu'à la frame 16 :
  `ItemCharge` est encore pleine à f12, f13, f14 et f15, à 0 à f16 (mesuré, `gungeon/item_fiole`). La fiche
  disait « UseActive à f10 puis ItemCharge qui retombe » : l'attente est donc à f16. Je n'ai pas cherché
  l'origine du décalage (question ouverte ci-dessous).
- **Pas d'attente `RoomState` ponctuelle** pour la salle : elle dépendrait de la frame exacte du bot ; les
  moments clés `room` (cumulatifs, `by_frame`) disent la même chose sans fragilité. Idem `BossPhase` pour le
  boss (elle échoue une fois le boss mort, or la phase 2 précède la mort de 20 frames).
- **Testbed, `salles.ldtk` en `map:` ne marche pas** : le bot `explorateur` n'ouvre pas la salle de combat hors
  mode `Floors`. Je n'ai pas touché au bot : j'ai ajouté une séquence `Floors` d'un seul niveau
  (`floors/salle_combat.ron`, `levels: ["testbed/salles.ldtk"]`), le bot y verrouille f65 et nettoie f534
  (500 frames ne suffisaient pas : 700). Le mode du testbed reste `Sandbox` (la séquence n'est jouée que quand
  un scénario la nomme).
- Les stats attendues viennent de `sim_core::modifier::resolve` sur une valeur de base sondée sans objet ; le
  générateur échoue (`MissingStat`) si le joueur n'a pas la stat plutôt que d'émettre une attente vide.
- `CharacterTestEntry` (registre du lint) perd `Copy` (il porte maintenant des `String`).
- Le `frames` d'un actif doit dépasser 16 (assertion explicite du gabarit).
- Deux scénarios générés de deux jeux peuvent porter le même nom (`item_bottes`, `item_fiole`, `room_combat`,
  `room_depart`) : l'export de métriques les fusionne (268 entrées pour 279 scénarios), précédent déjà
  présent pour `weapon_*`. Aucun effet sur les traces.

## 5. Couverture

Couvert : tous les objets **non consommables** de `gungeon` et `testbed` (leur présence ; leurs stats et leur
charge là où le contrat les expose), les types de salle `combat` (deux jeux), `depart` (deux jeux) et
`recompense` (testbed), le boss `gatling`.

**Non couvert** :
- Les effets d'objet (`effects:` : `OnPickup`, `OnRoomClear`, `OnReload`, `OnFloorStart`, soin à l'usage…) :
  le gabarit ne les déclenche pas et n'attend rien d'eux ; seul `test.expect` explicite peut les viser.
- Les consommables (`blank`, `key`, `ration`, `obus`) : refusés par le lint (`HasItem` ne les voit pas).
- Salles : `gungeon` `arene`, `boutique`, `recompense`, `boss` ; `testbed` `boss`, `arene`, `arene_mortelle`.
  `arene` (900 frames à tenir après le trajet) dépasserait la minute ; non essayée.
- Boss : seul `gatling` ; les `gardien*` du testbed n'ont pas de `test:` (cartes `salles_boss*.ldtk`).
- p2p à deux clients, restart en ligne, bench au calme (`ALACOD_BENCH_STRICT=1`) : **non vérifiés ici** (aucun
  changement de simulation : le seul code de simulation touché est la pose initiale de `Inventory`/`Modifiers`
  par le runner de scénario, hors jeu).
- `make test_scenarios` lui-même n'a pas été lancé : j'ai joué la même suite par `scripts/test-scenarios-sharded.sh`.

## 6. Dettes et questions ouvertes

- Pourquoi 6 frames entre un appui scripté et son effet sur `UseActive` (alors que `Fire` pulsé répond sans
  délai observé) ? À éclaircir ; l'attente `ItemCharge(0)` est calée sur la mesure.
- `explorateur` ne verrouille pas de salle sans le mode `Floors` : à noter pour un futur gabarit de salle par
  carte seule (`map:` du `RoomTest` reste supporté, mais aucun contenu ne s'en sert).
- Gabarits par statut, synergie et arène : hors périmètre.

## 7. Atteignabilité (M2, `docs/taches.md` §7)

Le livrable est le générateur lui-même : chaque scénario ci-dessus **est** une partie de `gungeon` ou du
testbed qui rencontre la mécanique (objet tenu, salle verrouillée puis nettoyée, boss à trois phases).
`boss_gatling` rejoue le chemin de `gungeon_boss_bot` ; `room_combat` de `gungeon` celui de
`bot_explorer_floor`. Aucune dette T14 : les gabarits couvrent le contenu existant.
