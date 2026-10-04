# Rapport m1-v1c-behaviors — behaviors composables v1 (T1.4)

**Branche** `m1-v1c-behaviors-composables`, partie de la tête T1.2 `5e84fef` (prérequis, dans
le même worktree sur instruction de l'orchestrateur) + `origin/main` `a56d97b` (fiche).
Base à la livraison : _(complété au merge final d'`origin/main`)_.

## 1. Fait

### Contrat et sélection (`crates/behaviors`)

- `Behavior::Shoot { weapon, pattern, range, cooldown_frames }` remplace `Shoot(String)` de
  T1.0a (décision Q3) ; `Behavior::name()`.
- Sélection **pure** : `SelectionContext` (faits d'une frame, calculés par `game`),
  `applicable(index, behavior, ctx)` et `select(rules, ctx)` — première règle applicable,
  ordre RON = priorité, applicabilité implicite (tableau §22). Test « table de cas » (17 cas :
  zombie, kiter, lâche, rôdeur, chargeur).
- `BehaviorState` (T1.0a) : **enregistrement inchangé**, posé sur personne (doc mise à jour).

### Un seul chemin de code, sans état nouveau pour l'existant (`crates/game`)

- `EnemyBehaviors` : règles résolues au spawn (liste du RON, sinon `default_behaviors`
  dérivée de la config), perception (`hearing`) et ciblage (`ignore`). Composant **statique,
  hors rollback** (comme `Team`) : un ennemi existant ne gagne ni ne perd aucun composant
  rollback.
- Compilation vers l'état existant : `Shoot` → `EnemyAiConfig::ranged` (même hash qu'en T1.2,
  `turret`/`archer` migrés sans changer de trace), `Sight(r)` → `aggro_range`, `Melee(arme)` →
  arme équipée par `spawn_enemy` (`zombie_claws` n'est plus codé en dur ; repli `bare_hands`).
- `current_rule` : règle retenue **dérivée** pour un ennemi sans état nouveau (séquence de tir →
  `Shoot`, `Attacking` → `Melee`, cible connue → `Chase`) — lue par l'attente `EnemyState`.
- Behaviors nouveaux (`KeepDistance`, `Flee`, `Strafe`, `Charge`, `Wander`) :
  `BehaviorRuntime` (règle retenue, frame d'entrée, phase de charge, prochaine errance, ouïe),
  **checksum neutre**, posé seulement sur les ennemis qui en listent un.
  - `behavior_select_system` (`EnemyAI`, après `enemy_target_selection`, `order_mut_iter!`) :
    faits, sélection, phases de charge (télégraphe → ruée → contact/fin → refroidissement),
    errance (flux RNG `behaviors`, toutes les 60 frames), ouïe (tir de joueur né à portée →
    tireur connu 120 frames) ; log `ggrs{f=… behavior net_id=… rule=…}`.
  - `behavior_motion`, appelé par `move_enemies` : direction et vitesse imposées (recul par
    `FlowField::retreat_direction`, perpendiculaire alternée, errance demi-vitesse, ruée 3 ×,
    immobile au télégraphe) ; même résolution de collision. Pour les ennemis sans
    `BehaviorRuntime`, le calcul d'origine est intact (`None`).
  - `charge_damage_translate_system` (`CollisionDamage`) : dégât de contact d'une ruée, une
    frame après la décision (comme `enemy_attack_damage_translate_system`).
  - `enemy_attack_system` : un ennemi à `BehaviorRuntime` n'exécute que sa règle retenue
    (`Shoot` ou `Melee`) ; les autres suivent le chemin d'origine.
  - `enemy_target_selection` : filtre `Targeting::Nearest { ignore }` (liste vide : aucun
    filtre).
- **Code mort retiré** : `enemy_movement_system`, `enemy_stun_recovery_system`, `apply_stun`,
  `MonsterState::{Stunned, Breaching, Fleeing}`. Vérifié avant : le `derive(Hash)` hache l'index
  de variante, seules `Dead` (index déplacé) et les variantes retirées changent, et aucune n'est
  jamais posée ; la sérialisation (`remote.rs`, debug) nomme les variantes, pas les index.
  `enemy_target_selection` et `update_enemy_targets` **ne font pas doublon** (cible d'IA d'un
  côté, point de chemin `EnemyPath` de l'autre, dont le repli « joueur le plus proche ») :
  gardés tous les deux, et le dire était la consigne si les deux produisent l'état actuel.

### Contenu et lint

- Zombies (`zombie_config`, `zombie_full_config`, `zombie_hard_config`) : `ai` complet
  (valeurs du préréglage `zombie()`) et `behaviors: [Melee("zombie_claws"),
  Chase(profile: "GroundBreaker")]`.
- `turret`/`archer` : `ai.ranged` → `behaviors: [Shoot(...)]` (+ `Chase` pour l'archer).
- Lint : `Shoot` (règles de T1.2), `Melee` inconnue, `Chase` profil inconnu, `KeepDistance`
  `min >= max`, `Charge` télégraphe 0, liste vide, tag d'`ignore` inconnu. Fixtures
  `behavior_melee_unknown`, `behavior_keep_distance_inverted`, `behavior_charge_zero`,
  `behavior_unknown_profile`, `targeting_unknown_tag` ; `ranged_*` migrées vers `Shoot`.

### Attentes (`crates/scenario`, ré-export `game::replay`)

`EnemyState`, `EnemyDistance` (ponctuelles), `EnemyContactBefore`, `EnemyNeverInWall`
(continues) ; liste de `CLAUDE.md` mise à jour.

### Testbed et scénarios

- `kiter`, `charger`, `coward`, `drifter` (`games/testbed/assets/characters/`).
- **Écart avec la fiche** : quatre cartes `testbed/arena_ia_keep|charge|flee|wander.ldtk` au
  lieu d'une `arena_ia.ldtk`. Un scénario ne peut pas faire apparaître de personnage (pas de
  champ de placement) : dans une arène commune, les quatre ennemis tireraient, chargeraient et
  poursuivraient le même joueur dans chaque scénario (dégâts non attribuables). `arena.ldtk` et
  `arena_tir.ldtk` intacts.
- Scénarios (ennemi `NetId(27)`, valeurs mesurées) :
  - `enemy_keep_distance` : `Shoot` f20 ; `KeepDistance` f81 (recul 47 → 124 à f141) ;
    `Shoot` f171 ; `KeepDistance` f221 ; `EnemyDistance` ∈ [110, 210] à f131, f171, f221 ;
    santé du joueur ≤ 99 à f100 (volée) ; `EnemyNeverInWall` 1-260.
  - `enemy_charge` : `Chase` f20 ; `Charge` f61 ; distance figée ∈ [118, 121] à f61 et f71
    (télégraphe immobile) ; `EnemyContactBefore(120)` (contact vers f98) ; santé ≤ 90 à f110
    (100 → 85) ; `Chase` f130.
  - `enemy_flee` : le joueur tire f10-30 ; `Chase` f20 ; `Flee` f61 et f151 ; distance
    ∈ [65, 82] à f61 puis ∈ [140, 175] à f151 (elle croît) ; `EnemyNeverInWall` 1-220.
  - `enemy_wander` : `Wander` f50 ; distance ∈ [185, 200] à f51 puis ∈ [215, 230] à f121
    (il bouge) ; le joueur marche vers lui f120-220 → `Strafe` f235, retour `Wander` f280 ;
    `EnemyNeverInWall` 1-330.

## 2. Vérifié (résultats réels)

_(complété après les exécutions finales)_

## 3. Non fait / incertain

_(complété)_

## 4. Dettes, questions ouvertes

- D28 ajoutée (dégât direct `attack_damage` presque jamais appliqué, préservé).
- `Chase { profile }` : seul le champ de flux `GroundBreaker` est construit aujourd'hui ; tous
  les profils l'utilisent (comme avant T1.4). Construire un champ par profil est hors périmètre
  (réécriture de la navigation).
- Pendant `Flee`, un lâche qui liste `Melee` garde son arme équipée : `enemy_melee_attack_system`
  attaque dès qu'un joueur entre dans la portée de l'arme, indépendamment de l'IA (comportement
  d'origine de la griffe).
- `EnemyNeverInWall` : le cas d'échec n'a pas de scénario (aucune carte ne place un ennemi dans
  un mur).
