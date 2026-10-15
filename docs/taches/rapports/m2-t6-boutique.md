# Rapport m2-t6-boutique — boutique et monnaie `shells` (M2, chantier C5)

Branche `m2-t6-boutique` depuis `main` (fe79f9a), fusionnée avec `origin/main` `b527bdb` avant livraison.
Conventions : §50.

## Fait

- **Douilles au sol** : `economy.ron` `shells_drop: Some((min, max))` ; `game::shop::shell_drop_on_death_system`
  tire dans un `RollbackRng` **local** (`loot_seed(graine, étage, "shells:<GgrsNetId du mort>")`), aucun flux de
  `RngStreams`. `ShellPickup` (rollback neutre), ramassé au contact, `Currency::earn` + `CurrencyEvent` (`pickup`),
  `despawn_rollback()`. Ordonné après les autres drops (tous consomment `GgrsNetIdFactory`).
- **Boutique** : `rooms/<type>.ron` `shop: Some((table, shelves))` ; `room_shop_system` (map_ldtk) pose les
  présentoirs ; objets tirés par `shop::draw_shelves` (graine `shop:<salle>`, indépendante du moment) ; prix par
  rareté dans `economy.ron` `shop_prices`. Salle jamais verrouillée.
- **Achat** : `items::Shelf` (neutre), `InteractionType::Shop` (en fin d'enum), `shelf_interaction_system` :
  `spend` puis objet donné comme un ramassage (passif/actif/consommable), refus = `CurrencyEvent{delta:0}` +
  anti-rebond de 30 frames, présentoir vendu perd son `Interactable`. Prompt HUD « Acheter Trèfle — 30 ».
- **Lint** : 5 cas (prix 0, `shells_drop` min>max, boutique sans présentoir, pool vide, pool inconnu) + 5 fixtures.
- **Contenu gungeon** : `rooms/boutique.ron` (3 présentoirs, remplace le coffre de T5), `loot/boutique.ron`,
  `economy.ron` (`shells_drop: (2, 5)`, prix 15/30/50).
- **Scénarios** : `shells_drop`, `shop_buy`, `shop_refused` (testbed, présentoirs et drop imposés par
  `Scenario::shop`), `shop_seeded` (gungeon, grammaire `etage`, graine 1) ; attentes `Shelf`, `ShelfCount`,
  `ShellPiles`. Test unitaire du tirage (`shop::tests`).

## Vérifié (résultats réels)

- `make test_scenarios_par` (4 tranches) **sur la fusion avec main b527bdb** : 301 scénarios fusionnés, 4 tranches ok, 338 s.
- `make test_crates` : 716 réussis, 0 échec, 10 ignorés.
- `make lint` ×4 : aucune erreur (gungeon : 20 avertissements, tous préexistants : armes sans source, `cible`).
- `cargo fmt --all -- --check` : ok ; `check-forbidden` : 9 occurrences préexistantes ; `check-rollback-registration` : OK.
- `make gen GAME=…` ×4 : aucun fichier modifié.

## Traces et preuve §5

Seules des traces **gungeon** ont bougé (14 blessées : `gungeon_boss`, `_boss_bot`, `_clear`, `_floor_seed_{1,23,123}`,
`_floor_start`, `_two_floors`, `bench_floor`, `bot_explorer_floor`, `bot_explorer_two`, générés `gungeon/room_combat` et
`gungeon/boss_gatling`) ; aucune trace zombies/testbed/throne.
- `gungeon_floor_seed_1` (dump main / branche, `trace-diff.py`) : première différence frame 149 (premier kill) =
  l'entité `shells_34` (`ShellPickup`) et le compteur `GgrsNetIdFactory` ; rien d'autre.
- `bench_floor` : différence frame 0 seulement = deux coffres de salle `boutique` (T5) remplacés par trois présentoirs.
- `gungeon_two_floors` : `NetId` du boss 159 → 163 (ids décalés par les douilles et présentoirs).

## Non fait / non vérifié

- **p2p à deux clients** (Docker) et bench strict : non rejoués ici (cloud), à rejouer par l'orchestrateur.
- Pas de vidéo ni de rendu : le prompt HUD n'est vérifié que par test unitaire de `Prompt::text`.
- Le bot explorateur meurt avant d'atteindre la boutique (alacod-sim ci-dessous) : l'achat n'est pas joué dans une
  partie de bot.

## Dettes

- Pas de sprite de présentoir ni de douille (placeholders absents, entités sans rendu).
- Doublons évités par 8 essais (peut rendre moins de présentoirs que demandé si le pool est petit).
- `ShellPickup` ignore les multiplicateurs de monnaie (Double Points) et les joueurs à terre.

## Atteignabilité

Grammaire `etage` (`floors: etages_grammaire`), qui exige une `boutique` : scénario `shop_seeded` (`map_seed: 1`)
voit les trois présentoirs dès la frame 10 (clé, trèfle, blank). `alacod-sim --game gungeon --bots 1 --profiles
explorateur --floors etages_grammaire --until-floor 1 --seeds 1..3` charge le même étage (graine 1 : boutique présente) ;
les trois graines finissent `mort` vers la frame 720, avant la boutique. Les douilles tombent dès le premier kill
(`gungeon_floor_seed_1`, frame 149).
