# Rapport m3-t11a-larmes (b0)

Conventions : `docs/conventions.md` §76.1. Branche `m3-t11a-larmes` depuis `origin/main` `ad2be1c2`, fusionnée avec `origin/main` `2ffb95b5` (ALACOD_MUTE). Données seulement : aucun code moteur.

## 1. Fait
- `larmes` d'`isaac` en `mag: Infinite` (plus d'`ammo_type`, plus de chargeur 9999) ; stats `Damage`/`FireRate`/`Range`/`ShotSpeed`/`ShotSize`/`Luck` lisibles par les objets.
- 19 objets de larmes et de stats (tableau objet → effet → pool → graine → scénario au §76.1), pool `loot/tresor.ron` (entrées `Item`, poids 4/2/1), `rooms/tresor.ron` y pointe. Statuts `poison`, `brulure`, `gel`, `peur`, `lenteur` déclarés dans `game.ron`.
- `test: tears:` avec témoin pour 16 objets (`bottines`, `trefle_noir`, `plomb_fondu` : `item_*` seul, stats sans effet de coups mesurable par le gabarit). Seuils mesurés (commentaire de chaque `.ron`) : `ressort` 8 coups/5, `vent_arriere` touche avant f48/0, `epine` 11/6, `eclat` 5 balles/0, `braise` statut, etc.
- Lab : `cible_fragile` (15 PV, `test_only`), arme `larmes_courtes` (portée 100, `test_only`) pour `Range` (arène plus petite que la portée de `larmes`), cible posée à 22 unités pour `ShotSize`.
- Lisibilité : halo `tear.png` de M3-T17 copié, `bullet_player`/`bullet_enemy` d'isaac, `assets.yaml` (CC0 projet). Pas de sprite d'objet (aucun champ d'image dans `ItemDef`) ; rien touché de la lisibilité gungeon.

## 2. Traces bougées (preuve §5, dumps `ALACOD_DUMP_TRACE` main vs branche, `trace-diff.py`)
23 traces `isaac`, toutes sur la frame 0. Dumps comparés : `isaac_boots`, `grid_treasure_key`, `generated/isaac/boss_gardien`. Deux causes, seules lignes différentes :
1. `WeaponModesState.mag_ammo` de l'arme `larmes` : 9999 (main) → 0 (`Infinite`), dès f0 ;
2. `Chest.table` de la salle de trésor : `salle` → `tresor` (là où il y a un coffre de trésor).
Bénies : 16 hors générées (`curse_*` ×4, `grid_*` ×9, `hit_iframes_isaac`, `isaac_boots`, `isaac_floor_seed_1`) + générées `enemy_*`, `boss_gardien`, `room_combat`, `weapon_larmes` ; nouvelles : `item_*` ×19, `tears_*` ×32 (62 bénies par `make gen GAME=isaac`). Aucune trace d'un autre jeu n'a bougé.

## 3. Vérifié (sur la fusion avec `2ffb95b5`)
- `make test_scenarios_par SHARDS=3` : 3 tranches ok, 280 s (avant bless : 23 traces isaac seulement différentes, aucune attente en échec).
- `make test_crates` : 1 097 passés, 0 échec.
- `make lint` ×5 : aucune erreur, 0 avertissement d'atteignabilité sur isaac (`--json`).
- `make gen` ×5 : code 0, `git status` propre ensuite.
- fmt : **échoue sur `crates/game/src/feedback.rs:2100`, commit de `main` `2ffb95b5` (muted())**, pas de mon fait (aucun Rust modifié) ; à corriger sur main.
- p2p `scripts/p2p-stress.sh isaac` (3 000 frames, MATCHBOX_PORT=3536, projet b0) : **5 571 frames resimulées**, 0 desync, 2 999 lignes identiques.

## 4. Atteignabilité
`alacod-sim --game isaac --bots 1 --profiles explorateur --shopping --seeds N..N --floors sous_sol --until-floor 2 --max-frames 12000 --log`, balayage 1..125 : première graine dont le coffre de trésor donne l'objet = tableau §76.1 (ex. `epine` 117, `eclat` 24, `fantome` 10). Le bot ne rouvre pas le trésor fermé du 2e étage ; seules les graines 1..125 ont été balayées, les 19 objets sont tous sortis.

## 5. Dettes
- Pas de salle boutique ni de pool boutique dans isaac (T11d) : la fiche demande trésor/boutique, seul le trésor est couvert.
- Les seuils `Chance` (`venin`, `givre`, `epouvante`) dépendent du net id des larmes : un changement de cadence déplace les tirages.
- `bottines`, `trefle_noir`, `plomb_fondu` sans `tears:` (stat sans effet de coups propre) ; `Luck` n'a pas de preuve par témoin dédiée.
- Pas de sprite d'objet (moteur) ; pool trésor sans `exhaust`.
- Revue indépendante : à faire avant merge. Bench non lancé (machine partagée).
