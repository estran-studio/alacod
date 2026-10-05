# Rapport m1-d42-softlock-diagnostic — un diagnostic de soft-lock qui lit le bon champ (D42)

**Branche** `m1-d42-softlock-diagnostic`, partie de la tête livrée de m1-integration-scenarios
`b30d980`. Pas de fiche : consigne d'orch (dix lignes de m1-integration-scenarios, D42).

## État en cours

- **Fait** : code, tests, vérification (§3). Livré. Target non purgé (petite tâche,
  compilation incrémentale sur celui de m1-integration-scenarios, purgé à sa livraison).

## 1. Le défaut

`scenario::softlock::snapshot` lisait, pour chaque ennemi, le champ de flux de son profil
déclaré (`EnemyAiConfig::nav_profile()`, `Ground` pour tous les ennemis de `throne`). Or le seul
champ construit et suivi par le déplacement est `GroundBreaker` (D38) : tout ennemi `Ground`
était signalé « sans chemin depuis sa case exacte ». Ce faux diagnostic a envoyé l'enquête de
m1-integration-scenarios sur la piste D37 (point d'apparition dans la roche) avant qu'on trouve
les vraies causes (corps de 28 px, D41 ; bots qui ne chassent pas).

## 2. Correctif (hors simulation, aucune trace ne bouge)

- `game::character::enemy::ai::navigation::MOVEMENT_FLOW_PROFILE` (= `GroundBreaker`) : la seule
  source du profil du champ suivi ; `pathing.rs` (cinq lectures), `rules.rs` (recul) et
  `update_flow_field_system` (construction) l'utilisent au lieu du littéral (même valeur, aucun
  comportement changé).
- `softlock.rs` :
  - `Snapshot::flow_field` : le champ lu (`Some("GroundBreaker")`), ou `None` = **non calculé**
    (aucun champ construit) ; l'observation dit alors « chemins non calculés » au lieu de
    compter des ennemis « sans chemin ».
  - `path_cost`, `next_cell` et le point visé se lisent dans ce champ ; observation renommée
    « N ennemis hors du champ de flux GroundBreaker depuis leur case exacte ».
  - **Relevé par ennemi restant** (`EnemySnapshot::character`, `nearest_player`) et une
    observation par ennemi : « restant : tourelle #544 case (57, 40), 1 PV, joueur 1 le plus
    proche à 176 (hors de vue) ». Ligne de vue : **ligne fine** sans `Wall` entre les centres,
    même test que les bots (`bots::navigation::walls_clear`) ; la marge de tir de b1 n'est pas
    encore dans `main`.
- Tests : `softlock::tests` (non calculé sans champ ; comptage dans le champ suivi et relevé).

## 3. Vérifié

- `softlock::tests` : 2/2.
- **Cas réel** (`alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run
  --seeds 1..1 --until-floor 3`, état de m1-integration-scenarios) : mêmes 4 893 frames qu'avant
  (le refactor ne change rien) ; observations : « 0 ennemis hors du champ de flux GroundBreaker
  depuis leur case exacte », « restant : tourelle #657 case (3, 11), 153 PV, joueur 1 le plus
  proche à 307.89304 (en vue) » — avant : « 1 ennemis sans chemin ». Le bot voit la tourelle et
  ne tire pas à cette distance : utile à b1 (m1-v3-bots-portail).
- **Suite des crates** (`--include-ignored`, 15 crates) : 555 verts ; `scenarios` : **mêmes 32
  traces throne** différentes que `b30d980` (à bénir pour m1-integration-scenarios), aucune
  autre, 0 attente en échec ; doctest de `game::waves` (préexistant) ; et
  `hud_text_sur_throne_progression` (test de T1.18 venu de `main`, écrit sur l'ancien
  `throne_progression`) : corrigé sur m1-integration-scenarios (`a66b3a4`), mergé ici.
- `make lint` des trois jeux sans erreur, `fmt` propre, `check_forbidden` 4 (identique), `check_rollback_registration` OK.
