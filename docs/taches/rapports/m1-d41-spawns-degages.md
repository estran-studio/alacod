# Rapport m1-d41-spawns-degages — points d'apparition dégagés selon le corps (D41 reformulée)

**Branche** `m1-d41-spawns-degages`, partie de la tête livrée de m1-navigation-profils-tailles
`8c89a3c`. Consigne d'orch : D41 reformulée (« un agent de plus de ~32 px en jeu exige des points
d'apparition dégagés de 2 cases », générateur de cavernes).

## État en cours

- **Fait** : code, tests, vérification (§3). Livré. Aucune trace ne bouge.

## 1. Changement

- `world::CaveConfig::spawn_clearance` (serde, défaut 1, non sérialisé à 1) ;
  `world::is_open_within(grid, x, y, r)` (Tchebychev ; `is_open` = r 1) ;
  `points_of_interest(grid, players, enemy_spawns, enemy_clearance)` : les `ZombieSpawn`
  candidats doivent être dégagés de `enemy_clearance` cases ; les `PlayerSpawn` gardent 1 ; avec
  1, exactement les points d'avant.
- `content::registry` : le personnage lit son `collider` (écrit nu, `collider: (...)`) et son
  `scale` ; `CharacterEntry::body_extent` = max(demi-largeur + |offset x|, demi-hauteur +
  |offset y|) × `scale`, la taille **en jeu** (`create_character` met le collider à l'échelle).
  Après le chargement de tous les kinds, chaque caverne reçoit `spawn_clearance` = le besoin du
  plus grand corps de ses `characters` (`spawn_clearance_for` : 1 jusqu'à 24 px du centre de la
  case, soit 8 + 16 ; une case de plus par 16 px). Plus juste que « > 32 px » : l'offset vers les
  pieds compte (le boss de m1-integration-scenarios, 39 px en jeu, a une étendue de 28 px vers le
  bas : 2 cases ; l'actuel, 28 px en jeu, 22,4 px : 1).
- `docs/conventions.md` §21 : une ligne.

## 2. Effet sur le contenu

Tout le contenu actuel garde un dégagement de 1 (test `degagement_des_cavernes_du_contenu_reel`
sur throne, testbed et zombies ; `roi_rat` : étendue 22,4) : **aucun point d'apparition ne
bouge**, `petite`, `bench`, `niveau_1..3` compris.

## 3. Vérifié

- Unitaires : `world::cave::tests` (dégagement 2 : points entourés de 5 × 5 cases de sol,
  joueurs inchangés ; défaut RON 1), `content::registry::spawn_clearance_tests` (étendue avec
  offset et échelle ; seuils), `lint_fixtures::degagement_des_cavernes_du_contenu_reel`.
- Suite (`--include-ignored`, 15 crates) : 565 verts ; `scenarios` : **exactement les 7 traces
  du lot navigation encore non bénies sur cette base** (`enemy_flee`, `throne_progression`,
  `throne_solo`, `throne_quad`, `throne_three_floors`, `enemy_roi_rat_*`), aux mêmes lignes ;
  aucune autre, 0 attente en échec ; doctest `game::waves` (préexistant).
- `make lint` des trois jeux sans erreur (un premier essai lisait `collider` comme `Option` :
  tous les personnages échouaient au parse, attrapé par le test sur le contenu réel, corrigé) ;
  `make gen` zombies 16/16, testbed 36/36 sans différence, throne 39/39 (les 2 `roi_rat` du lot
  navigation) ; `fmt` ; `check_forbidden` 4 ; `check_rollback_registration` OK.
