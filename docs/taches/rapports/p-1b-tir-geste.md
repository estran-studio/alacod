# P-1b — Tir d'une arme à geste sans visée au départ (et P-1c : prop point de reprise)

Jalon servi : **S1, dimanche 15 novembre 2026** (`plan-1837-cli.md` § 6, ligne P-1b ; suite de P-1, § 110).
Branche : `wquintal/p-1b-tir-geste`, partie de `origin/main` **`dac9c528`** puis fusionné avec **`cc6ec15e`** (M-5, P-1c). Code P-1b : `d47e8f90` ; code P-1c (commit séparé) : **`29b61238`**, tête de code poussée et vérifiée.

## Constat

Une arme à geste (`gesture: Some`, épaulé de 9 f) tire à la fin de l'épaulé ; si l'input de cette frame n'a pas de visée (`pan (0, 0)`), `aim_direction`
rend `ZERO` et la balle partait à vitesse nulle (0 impact ; preuve 1837 J-2). Reproduit avant correction sur les deux scénarios neufs (0 coup reçu, code d'`origin/main`).

## Résultat par point

| Point | Issue | Preuve |
|---|---|---|
| Jamais de balle à direction nulle | Fait | `crates/combat/src/weapons/mod.rs` : `gesture_fallback_aim` (pure, `Fixed`) ; appliquée seulement si `gesture.is_some()` et la visée de la frame est nulle |
| Ordre du repli | Fait | visée de la frame du départ → dernière visée non nulle de l'épaulé (`WeaponState::raise_aim`) → regard (`melee::facing_to_fixed`, huit directions, la droite sans touche) |
| Champ neuf haché et affiché seulement s'il est posé | Fait | `Hash` (`3u8`, `x`, `y`) et `Debug` manuels ; test `raise_aim_absent_ne_change_ni_hash_ni_debug` (un état sans le champ a le hash d'avant, au bit près) |
| Scénario neutre, trace neuve | Fait | `tests/scenarios/geste_visee_relachee.ron` (visée à gauche, relâchée avant la fin : la balle part à gauche, `cible_zone` touchée) et `geste_sans_visee_regard.ron` (jamais de visée : part à droite, touche) ; traces `.trace` neuves ; sans le correctif : « 0 coups reçus < min 1 » |
| `geste_*` et `weapon_arme_a_gestes` sans BLESS | **Pas tenu, décision du coordinateur** | voir ci-dessous |
| `make schedule_order_diff` | Non joué | aucun système ajouté, déplacé ni réordonné, aucun type rollback ajouté (un champ de `WeaponState`) |
| `make check_rapide` | Fait | sur `d47e8f90` : `~/orca/runs/alpha/logs/p-1b-tir-geste-check-rapide-d47e8f90.log`, 1604 tests ; **sur la tête de code `29b61238`** (P-1b + P-1c, après fusion de `cc6ec15e`) : `~/orca/runs/alpha/logs/p-1b-tir-geste-check-rapide-29b61238.log`, sortie 0, **1614 tests** passent, 0 échec, sweep joué |
| § 110 | Fait | `docs/conventions.md` § 110, puce « Tir d'une arme à geste sans visée » |

## Ce que fait le banc (`1867_lore/site/labo/armes/armes.js`, lecture seule)

`partir()` tire vers le point visé par la souris **à l'instant du départ** ; si ce point est à moins de 0,6 m de la bouche, vers `J.visee`, le dernier cap visé
(mis à jour à chaque frame). La visée ne se « relâche » donc jamais au banc : sa visée est toujours la **dernière**. D'où, plutôt que la visée du premier instant de l'épaulé
(spec), la **dernière visée non nulle vue pendant l'épaulé** (posée au début, rafraîchie à chaque frame d'épaulé visée). Quand la visée ne bouge pas, c'est la même chose.
Aucun `DEFAUTS` ne règle ce repli (`regard: "visee"` ne règle que le cap du corps).

## Conflit de la spec : traces existantes (option A, décidée par le coordinateur)

Le champ est posé à chaque épaulé visé, donc haché pendant les neuf frames d'épaulé de **tous** les scénarios à geste à visée constante : leurs traces bougent. Aucun hash conditionnel
à l'état ne l'évite (la visée retenue n'est pas déductible de l'état haché). Option B (champ non haché) refusée par le coordinateur (CLAUDE.md : état lu par la simulation sous checksum) ;
option C (regard seul, sans état neuf) divergerait du banc (il garde la dernière visée). Option A retenue, BLESS limité aux scénarios à geste :

| Trace rebénie | Lignes (frames) qui diffèrent |
|---|---|
| `geste_epaule_tir` | 20–28, 200–208 |
| `geste_clic_perdu` | 20–28, 206–214 |
| `geste_dash_epaule` | 20–23 (épaulé annulé par le dash : champ effacé), 80–88 |
| `geste_dash_recharge` | 20–28 |
| `geste_etourdi_epaule` | 30–35 (épaulé annulé par l'étourdissement) |
| `zone_touchee_bord`, `_centre`, `_temoin` | 30–38 |
| `generated/testbed/weapon_arme_a_gestes` | 43–51, 155–163 |

Preuve : comparaison ligne à ligne avec `HEAD` avant le commit, même nombre de lignes, seules les fenêtres d'épaulé (neuf frames, moins si annulé) diffèrent, tout le reste
est identique à l'octet, nombre d'entités compris. Aucun autre scénario ne porte `arme_a_gestes` ni `gesture:` (grep `games`, `templates`, `tests`).

## Détail de la correction

- Dans le repli, la rotation de l'arme (déduite du `pan` nul par `system_weapon_position`, donc sans sens) décalait la bouche du canon : la balle est spawnée avec la rotation de sa propre direction
  et le côté du regard de cette direction (`FacingDirection::from_fixed_vector`). Sans cela le scénario sans visée manquait la cible (bouche à 14 px de l'axe).
- Retenue effacée au départ de la balle et à toute annulation de l'épaulé (dash, course, mêlée, étourdissement, gel, mise à terre).
- `FourWay` : la visée retenue passe par `aim_direction` avec le mode du porteur ; le regard de repli reste à huit directions (un jeu à quatre directions ne pose pas de geste d'arme à ce jour).
- Non couvert : une arme à geste `Burst` (ignore l'épaulé) n'a pas de visée retenue : repli direct sur le regard.

## P-1c — un prop qui porte `respawn_range` reste un point de reprise (ajout du coordinateur, 8 nov ; jalon dim 22 nov, sans régresser dim 15)

| Point | Issue | Preuve |
|---|---|---|
| `RespawnPoint` sur la branche prop | Fait | `crates/game/src/character/enemy/create.rs` : la branche `prop` de `spawn_enemy` sortait avant la pose ; `insert_respawn_point` (neuve, partagée avec la branche des ennemis, même condition `respawn_range`) y est appelée |
| Le lint l'accepte | Fait, sans changement | `lint_props`/le bloc `prop` de `lint.rs` ne refuse que `counts_hits`, `starting_weapons`, `boss`, `variants`, `ai.behaviors` ; `alacod lint games/testbed` (dans `check_rapide`) valide le nouveau `respawn_prop` |
| Scénario neutre qui échoue sans le correctif | Fait | `games/testbed/assets/characters/respawn_prop.ron` (copie de `respawn_beacon` avec `prop: true`) et `tests/scenarios/wisp_respawn_prop.ron` (celui de `wisp_respawn_point` avec le prop) ; sans le correctif : « HUD wisp = FEU FOLLET (attendu Reprise) », « joueur 0 pas réanimé avant la frame 280 », santé 0/100 au lieu de 50/100 ; avec : tout vert ; trace neuve seulement |
| `prop_*` et `wisp_*` sans BLESS | Fait | 15 scénarios existants (`prop_no_enemy`, `_cap`, `prop_repels_enemy` et son témoin, `prop_stops_bullet` et son témoin, les neuf `wisp_*`) rejoués : traces identiques |
| § 113 | Fait | `docs/conventions.md` § 113, fin de la puce « Reprise à un point » |

`RespawnPoint` est un composant rollback neutre : aucun porteur existant, donc aucune trace ne bouge ; seul le scénario neuf en porte un.

## Reste

Rien dans le périmètre. Gabarit généré : T-2 (déjà livré). Rien de 1837 dans l'engine. Le « E prêt 0b1e8610 » envoyé plus tôt est remplacé par celui de la tête finale (la demande P-1c est arrivée juste après).
