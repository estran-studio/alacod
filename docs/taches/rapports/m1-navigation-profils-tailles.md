# Rapport m1-navigation-profils-tailles — navigation par profil et par taille (D41 + D38)

**Branche** `m1-navigation-profils-tailles`, partie de la tête livrée de D42 `780029f` ;
`origin/main` `b22d4ab` (intégration de la vague 2, traces throne bénies) mergé avant la
compilation. Fiche : dix lignes validées par orch (garde-fous : profils des zombies vérifiés
avant de coder ; un commit = un lot de preuves ; bless à l'orchestrateur).

## État en cours

- **Fait** : trois lots (§1 à §3), preuve (§4), scénarios recalés (§5). Livré.
- **Reste à l'orchestrateur** : bless des traces listées au §4 ; report au journal de la
  **correction de D41** (§3).

## 1. Navigation par clé (profil, gabarit)

- `navigation::NavKey { profile, size }`, `AgentSize::{Small, Large}` : `Large` au-delà de
  `SMALL_AGENT_MAX` = 20 px de corps **en jeu** (`AgentBody` du collider, déjà mis à l'échelle
  par `create_character`).
- `FlowFieldCache.layers : BTreeMap<NavKey, FlowField>`. **`Hash` de `NavKey` écrit à la
  main** : une clé `Small` se hache comme son profil seul ; un cache au seul champ historique
  garde le même checksum (unitaire `cle_petite_hachee_comme_son_profil`, et la suite : aucune
  trace sans boss ni `Flee` ne bouge).
- **Clés construites** : `MOVEMENT_FLOW_KEY` (`GroundBreaker`, `Small`, le champ historique)
  toujours, plus la clé **canonique** de chaque ennemi **qui se déplace** (`!stationary`).
  `FlowFieldCache::canonical` : un profil qui voit exactement les mêmes obstacles que
  `GroundBreaker` sur la carte courante (pas de cellule d'obstacle que l'un passe et l'autre
  pas : ni fenêtre ni barricade pour `Ground`) partage son champ. Les ennemis `Ground` des
  cavernes de throne et du testbed suivent donc le champ historique ; seules les cartes à
  obstacles cassables construisent un champ `Ground` à part.
- **Gabarit grand** : `is_blocked_for` bloque aussi toute case voisine (8) d'un obstacle (centre
  à une case des murs, couloirs de trois cases) ; `is_too_narrow_for` ne s'applique qu'au petit.
- **Lecteurs** : `move_enemies` (direction, récupération, évitement local, échappements),
  `behavior_motion` (recul `KeepDistance`/`Flee`), le ciblage (`nearest_target` du profil de
  l'ennemi), le diagnostic `softlock` (D42 : champ de la clé de chaque ennemi, liste des champs
  construits) lisent la clé de l'ennemi. La recherche de fenêtre à casser
  (`enemy_target_selection`) reste sur le champ historique (seuls les casseurs la cherchent).
- Garde-fou (a) : zombies `zombie_config`/`_full`/`_hard` = `GroundBreaker` ;
  `zombie_scaled_config` = `Ground` mais fixe (`stationary`, `aggro_range` 0) : ne lit aucun
  champ et n'en fait pas construire. Repli `EnemyAiConfig::zombie()` = `GroundBreaker`.
- Unitaires `nav_key_tests` (4) : gabarit selon le corps, hash neutre, dégagement du grand,
  canonicalisation.

## 2. D38 : pas d'attaque au corps à corps pendant `Flee`

`combat::weapons::melee::MeleeHold` (marqueur rollback, **`rollback_and_trace_neutral`** :
type nouveau sans porteur hors fuite, sinon parité des types vides et toutes les traces
bougeaient — relevé par orch avant compilation) ; `rules::melee_hold_system` (`EnemyAI`, en
dernier) le pose tant que la règle retenue est `Flee` et le retire ensuite ;
`enemy_melee_attack_system` (set `Weapon`, frame suivante) ne lance pas d'attaque tant qu'il est
là (une attaque en cours se termine).

## 3. Boss et correction de D41

**Correction d'un diagnostic de m1-integration-scenarios** : `create_character` met le collider
à l'échelle de `scale`. Le boss « 28 px » de m1-integration-scenarios faisait **39 px en jeu**
(28 × 1,4), le boss « 20 px » 28 px. Ce qui le figeait n'était pas la limite de 20 px de la
navigation : à 39 px, son corps déborde sur la roche du bord (y = 0) depuis un point
d'apparition dégagé d'une seule case (`world::is_open`), classe D37. Avec l'ancienne
navigation, le boss de 28 px en jeu se déplaçait déjà. **D41 se réduit à** « un agent de plus de
~32 px en jeu exige des points d'apparition dégagés de deux cases » (générateur de cavernes),
hors de ce lot.

`roi_rat` garde donc un collider de 20 (28 px en jeu) : **gabarit grand**, il suit le champ
`GroundBreaker/Large` et se déplace (`throne_quad` : (910, 52) à f1400, (576, 324) à f2400).

## 4. Preuve §10

Suite (`scenarios`, 15 crates, `--include-ignored`) sur l'état final : **toutes les autres
traces identiques**, zombies et testbed compris (canonicalisation + hash neutre + `MeleeHold`
neutre). Traces qui changent :

| Trace | Première différence | Cause (prouvée) |
|---|---|---|
| `enemy_flee` (testbed) | ligne 32 (f31) | **D38** : premier `MeleeHold` sur `coward` à f31 |
| `throne_progression`, `throne_solo` | ligne 1334 (f1333) | **D38** : premier `MeleeHold` sur `pillard` à f1333 |
| `throne_three_floors` | ligne 837 (f836) | **D38** : premier `MeleeHold` sur `pillard` à f836 |
| `throne_quad` | ligne 1073 (f1072) | **D38** : premier `MeleeHold` sur `pillard` à f1072 |
| `enemy_roi_rat_still`, `enemy_roi_rat_moving` (générés) | ligne 2 (f1) | **D41** : le boss (28 px en jeu) fait construire le champ `GroundBreaker/Large` |

Méthode : dumps (`ALACOD_DUMP_TRACE`) des quatre scénarios, première frame où `MeleeHold`
apparaît = exactement la frame de la première différence de trace.

## 5. Scénarios recalés et 20 graines

| Scénario | Changement |
|---|---|
| `throne_solo` | mort et défaite à f3560 (f3640 avant : le pillard ne griffe plus en fuyant) ; `PlayerAlive` f3550, `Defeat` by f3560 |
| `throne_quad` | étage 3 à f3167 (f3313) ; boss 706 entamé à f2400 (≤ 200), nouveau boss au rechargement **NetId 1417** (1496 avant) à f3167 |
| `throne_three_floors` (duo) | **graine 4** (et non plus 123456, où le duo se bloque désormais au niveau 3 sur le boss et la tourelle, **en vue** des bots selon le diagnostic D42) : étages f761, f1949, f4869, mutations des deux joueurs, alerte f1661 |
| `throne_progression` | trace seule (attentes inchangées, test HUD vert) |

`alacod-sim --game throne --floors run --seeds 1..20 --until-floor 3 --max-frames 15000`, bots
`prudent`, après ce lot (référence avant : rapport m1-integration-scenarios §5) :

| Bots | Avant ce lot | Après ce lot |
|---|---|---|
| 1 | 0/20 (6 SL, 14 morts) | 0/20 (8 SL, 12 morts) |
| 2 | 3/20 (graines 3, 14, 15) | 3/20 (graines 4, 15, 17) |
| 4 | 11/20 | 11/20 |

0 desync. La navigation ne change pas le taux de réussite des bots (b1 s'en charge) ; elle rend
au boss la possibilité de venir au contact et retire la griffe des fuyards.

## 6. Vérifié

- Tests des crates : unitaires `game`/`combat`/`scenario` verts (dont `nav_key_tests`,
  `softlock::tests`) ; suite : 560 verts, échecs : `scenarios` (traces du §4, à bénir) et le
  doctest `game::waves` (préexistant). Scénarios recalés (§5) : attentes vertes.
- `make gen` : zombies 16/16 et testbed 36/36 sans différence ; throne 39/39 attentes vertes,
  2 traces différentes (`enemy_roi_rat_*`, §4) ; aucun fichier généré modifié.
- `make lint` des trois jeux sans erreur, `fmt`, `check_forbidden` 4 (identique),
  `check_rollback_registration` OK, exemples compilés.
- Target purgé (incremental et examples).
