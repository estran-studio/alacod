# Rapport m1-throne-gen-et-d40 — `make gen GAME=throne` vert et butin par munition (D40)

**Branche** `m1-throne-gen-et-d40`, partie de la tête livrée de D39 `82c503b` ; `origin/main`
(D39 + statuts de b1) mergé avant la compilation (`cfbd14e`, un conflit additif dans
`crates/effects/src/actions.rs` : `ApplyStatus` puis `RefillAmmoOf`, les deux gardées). Fiche :
`docs/taches/m1-throne-gen-et-d40.md`.

## État en cours

- **Fait** : les deux sujets, preuve des traces changées, vérifications (§4). Livré.
- **Target** purgé après la suite (incremental et examples supprimés, une génération par crate ;
  target 38G, /home 93G libres) ; dumps supprimés.
- **Reste à l'orchestrateur** : bénir les **38 traces neuves** (37 générées de throne et
  `throne_ammo_pickup`, retirées du dépôt par le dernier commit, copie dans
  `alacod_tasks/<tâche>/d40/traces_neuves/`) et les **2 traces changées** (`throne_progression`,
  `throne_three_floors`, preuve §3) ; bench, p2p, merge.

## 1. Gabarits de throne (`make gen GAME=throne` vert)

- `games/throne/assets/game.ron` : `generate_template: (map: "gabarit_armes.ldtk", target:
  "cible")` (amendement confirmé à orch : chemin relatif à `games/throne/assets/`, pas
  `throne/...`).
- `test: Some((frames: 600))` (attentes par défaut T1.13) sur les **10 ennemis** : arroseur,
  brute, buffle, chien, cracheur, franc_tireur, pillard, rat, rodeur, tourelle. Les 20 scénarios
  (immobile, mobile) passent **sans calibrage** : `Event hit` + `EnemyNeverInWall` (immobile),
  `PlayerAlive` + `EnemyNeverInWall` (mobile).
- **Armes** : 13 à distance + 4 de mêlée (la fiche en comptait 12 + 3 : `arsenal`, l'arme des
  ennemis, et `bare_hands` en plus). Toutes les jauges de b1 tiennent, **sauf `traqueur`**
  (`Homing`) : 0 coup sur la cible. Cause (dump) : la tête chercheuse vise le personnage
  touchable le plus proche ; le mannequin (552, 648, `GgrsNetId` 34) est à la même distance du
  joueur que la cible (552, 744), mais plus près du canon (sortie à y ≈ 691) : il prend 5 coups de
  10 en 200 frames. `test:` du traqueur : `min_hits: 0` + `expect: [EntityHealth(net_id: 34,
  max: 160.0, at_frame: 200)]` (au moins 4 coups), commenté dans `weapons.ron`.
- **37 scénarios** versionnés sous `tests/scenarios/generated/throne/` (`.ron` seuls ; traces à
  bénir).

## 2. D40, butin par munition

- `effects::Action::RefillAmmoOf(AmmoType)`, **dernière variante** (après `ApplyStatus` de
  main) : les hashs des variantes existantes ne bougent pas. Ajoutée aux bras de
  `as_modifier` (`None`), `effects_runtime` et `content::lint` (refusée dans un effet, comme
  `RefillAmmo`).
- `game::powerups` : le bras `RefillAmmo | RefillAmmoOf(_)` filtre par munition ; chargeur et
  réserve des seules armes de cette munition ; `clear_reloading` seulement si l'arme active a
  été remplie (ou `RefillAmmo`). Unitaire `refill_ammo_of_ne_remplit_qu_une_munition`.
- Lint `lint_powerups` : munition déclarée par aucune arme → `BrokenReference` ; fixture
  `powerup_refill_ammo_unknown`.
- `games/throne/assets/items/powerups.ron` : `munitions` remplacé par `munitions_balles`,
  `_obus`, `_explosifs`, `_energie`, `_lames` (poids 12 chacun : même chance totale, 60, que
  l'ancien `munitions`).
- Scénario **`throne_ammo_pickup`** (carte gabarit, `Sandbox`, 240 frames) : 5 tirs (25),
  rechargement à f50 (fin à f145) ; `munitions_lames` à f100 : chargeur 25, réserve de balles 456
  (pleine) **et le rechargement n'est pas annulé** (chargeur 30, réserve 426 à f150) ; nouveaux
  tirs (25) ; `munitions_balles` à f210 : chargeur 30, réserve 456. (Premier brouillon : les
  ramassages tombaient pendant le rechargement sans rien vérifier de D40, redessiné.)
- `docs/conventions.md` : §14 (`RefillAmmoOf`), §29 (gabarits et butin).

## 3. Preuve §10 (traces changées)

Suite complète : **2 traces throne changent**, aucune autre (zombies, testbed, autres throne
identiques). Méthode : binaire de la branche, joué avec le `powerups.ron` de `origin/main` (seul
contenu changé qui touche la simulation hors gabarits) : **les deux traces bénies sont
reproduites exactement** ; le code de D40 seul ne change rien. Puis dumps ancien contenu contre
nouveau (`trace-diff.py` + comparaison par entité sur toutes les frames) :

- **`throne_progression`** : première différence **f1574**, `GgrsNetId(300)` :
  `powerup_munitions` → `powerup_munitions_lames`, **même tirage, même position, même
  expiration** (l'entrée de même rang du tirage pondéré). Ramassé à f1587 : seules autres
  différences jusqu'à la fin (f1950), la réserve du joueur 73 et l'état de rechargement de sa
  mitraillette (l'ancien `munitions` remplissait les balles et annulait le rechargement). Toutes
  les attentes restent vertes.
- **`throne_three_floors`** : première différence **f434**, même mécanisme (`GgrsNetId(170)`,
  `munitions` → `munitions_lames`), ramassé à f459 par les deux bots (réserves et rechargement
  de la mitraillette) ; la divergence gagne les tirs à f650 puis le reste de la run : troisième
  étage à **f2330** au lieu de f1759 (`alacod-sim --game throne --bots 2 --profiles
  prudent,prudent --floors run --seeds 123456..123456 --until-floor 3` : étages f517, f1190,
  f2330, 0 mort). **Attentes recalées** sur cette mesure (`frames: 2350`, étage 2 à f1195 et
  f2325, étage 3 à f2335, mutations : joueur 0 `rancune`, joueur 1 `irradie` + `chasseur`,
  joueurs vivants à f2349), commentaire du scénario mis à jour.

Effet de jeu à noter : avec le butin par munition, les bots `prudent` finissent la run de
référence 571 frames plus tard (moins de balles) ; c'est la conséquence attendue de D40, pas un
changement de tirage.

## 4. Vérifié

- **Tests des crates** (`scenario run combat game content map_ldtk map sim_core stats bots
  effects utils behaviors world`, `--include-ignored`) : **516 verts**, 2 échecs : `scenarios`
  (les 2 traces du §3) et le doctest `rust,ignore` de `game::waves` (préexistant).
- **`make lint`** : zombies, testbed, throne sans erreur.
- **`make gen`** : throne 37/37 `ok` ; zombies 16/16 et testbed 36/36 `ok`, aucun fichier généré
  modifié.
- **`fmt`** propre (commit dédié) ; **`check_forbidden`** 4 occurrences (identique) ;
  **`check_rollback_registration`** OK.
- **Exemples racine** compilés (`cargo build --profile headless --examples`).

## 5. Écarts, non fait / incertain

- **Traces** : aucune bénie dans le dépôt (règle de la fiche) ; les 38 traces neuves sont dans
  `d40/traces_neuves/` à titre de comparaison. Tant qu'elles ne sont pas bénies, `scenarios`
  échoue aussi sur ces 38 scénarios (« pas de trace de référence »).
- Commits : la fiche demande un commit par sujet ; le code des deux sujets était dans un WIP
  commun (`dd766b1`, écrit avant le feu vert), suivi de commits séparés pour la génération
  throne, le scénario D40, le recalage et fmt.
- `test:` du traqueur : attente sur le `GgrsNetId` 34 du mannequin, stable tant que le gabarit et
  l'ordre de spawn ne changent pas (même hypothèse que `net_id` de la cible dans les générés).
- Bench strict et p2p : non faits (orch).
