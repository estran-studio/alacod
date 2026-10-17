# Suivis M3 : Q13 `MirrorShot` larmes, Q17 visée à quatre directions, Q12 charme, Q10 transformations

Branche `m3-suivis-q10-q12-q13-q17` depuis `origin/main` `2d30a1c1`, fusionnée avec `origin/main` `f18f5945` (M3-T11c) en fin de
tâche. Quatre commits (un par suivi, dans l'ordre Q13, Q17, Q12, Q10) puis la fusion et les corrections de relecture. Conventions :
§71 (Q13), §66 (Q17), §19 (Q12), §44 (Q10 : la consigne citait §41, qui traite des projectiles v2 et du blank ; les
transformations sont une extension des synergies, §44). Session b0.

## 1. Fait

### Q13 : `MirrorShot` tire les larmes du porteur (§71)
- Contrat : `Behavior::MirrorShot { every }` ; le champ `weapon` disparaît (migration de `BehaviorEntry`, du lint « arme inconnue »
  et des personnages `compagnon` du testbed et `frere` d'`isaac`). Un familier n'a plus d'arme propre.
- Exécution (`game::familiars::familiar_mirror_shot_system`) : l'arme **active** du porteur dans son **mode actif**, composée avec
  ses objets et synergies par `Projectile::composed` et `items::projectile_mods`, ses stats `Damage`, `Range`, `ShotSpeed`,
  `ShotSize`, `Luck`, sa direction de visée ; crédit (net id, équipe, handle) au porteur ; tags du familier ; un tir par
  déclenchement, sans dispersion ni RNG ; porteur sans arme : pas de tir.
- Trois aides **extraites** de `weapon_rollback_system` sans changer une valeur (`combat::weapons::{tear_motion,
  carried_projectile_mods, shot_projectile}`) : le tir du joueur et le tir miroir partagent le même code.

### Q17 : `aim: FourWay` (§66)
- `aim: Free | FourWay` (donnée de personnage, `sim_core::aim::AimMode`, défaut `Free`). **Composant statique non rollback, posé
  seulement pour `FourWay`** : un autre jeu ne porte rien, aucune trace n'y bouge.
- Quantification dans `weapon_rollback_system` **et** dans le tir miroir des familiers (`sim_core::aim::aim_direction`) : axe
  dominant du `pan` entier (comparaison de `|pan_x|` et `|pan_y|`, aucun arrondi), signe de sa composante ; **égalité exacte :
  l'horizontale l'emporte** ; `pan` nul : pas de tir.
- `PlayerScript::aim` (`Some(FourWay)` pose, `Some(Free)` retire) ; `pleureur` (`isaac`) l'active.
- Bot (`crates/bots`) : `BotView::aim`, `decide::apply_four_way` appliqué à la sortie de `decide` : direction quantifiée écrite,
  tir seulement dans la file (écart perpendiculaire ≤ 14), décalage vers la cible sinon, route gardée sans ligne dégagée.
- Scénarios neufs `aim_four_way` (trois tirs en biais dont la diagonale parfaite : tous partent à l'ouest et touchent) et
  `aim_four_way_libre` (témoin : aucun coup).

### Q12 : statut `Charm` (§19)
- `StatusDef::Charm` (fin d'enum), `statuses/charme.ron` (`frames: 200`), arme `status_charm`. `TargetType::Enemy` et
  `AttackTarget::Enemy` en fin d'enum (le `derive(Hash)` hache l'index : les variantes d'avant gardent le leur).
- Un ennemi charmé cible l'ennemi hostile le plus proche (non charmé, vivant, équipe `Enemies`, hors salle dormante ; égalité : plus
  petit `GgrsNetId`, `ai::charm::nearest_target`), marche droit sur lui (`charm_motion`), le frappe au corps à corps (dégât de
  l'équipe `Players`), ne touche aucun joueur (salve interrompue, hitbox de mêlée retenue par `MeleeHold`, aucune règle de
  comportement, pas d'ouïe) et redevient hostile à l'expiration sans état à nettoyer.
- Scénarios neufs `status_charm` (+ témoin `status_charm_temoin`) et `status_charm_contact` (le charmé collé au joueur) ; généré
  `weapon_status_charm`.

### Q10 : transformations (§44)
- `SynergyDef.requires_tag: Option<TagRequirement { tag, count }>`, `tint: Option<String>`, `requires` facultatif ;
  `SynergyDef::is_active` (règle unique) ; `SynergyTable::active` prend l'`ItemTable` (quatre appelants migrés). Compte : objets
  **distincts** tenus (passifs et actif) portant le tag.
- Lint : `count = 0`, tag que personne ne porte, `count` supérieur au nombre de porteurs, `tint` invalide, doublon (clé =
  ensemble d'objets **et** condition de tag) ; lint des mécaniques : une transformation compte comme atteignable seulement avec
  assez d'objets à source.
- Présentation : `TransformationTint` (calques du joueur, derrière un champion, devant le marqueur) et moment clé `feedback`
  « transformation 26 (champignons) ».
- Testbed : `amanite`, `bolet`, `morille` (tag `champignon`), synergie `champignons` (`MoveSpeed` `Pct` +0,3, teinte `#c07040`),
  scénarios `transformation_three` (150 → 195,00046 à la troisième prise, f75) et témoin `transformation_two` ; générés
  `item_amanite`, `item_bolet`, `item_morille`.

## 2. Traces bougées (preuve §5)

Outil : `ALACOD_DUMP_TRACE` (dump complet de toutes les frames) + `scripts/trace-diff.py`, binaire de test de `main` `2d30a1c1` contre
celui de la branche, mêmes scénarios.

| Suivi | Trace | 1re frame qui diffère | Cause |
|---|---|---|---|
| Q13 | `familiar_mirror_cadence` | 20 | premier tir du porteur, donc premier tir miroir ; seule ligne en désaccord : la `Bullet` du familier, `pistol` (dégât 15, vitesse 400) devenue `larme` (3,5 ; 200) |
| Q13 | `familiar_mirror_shot` | 35 | idem (tir demandé à f30, balles parties à f35) |
| Q13 | `familiar_quad` | 105 | premiers tirs miroirs des joueurs 1 et 3 |
| Q13 | `champion_explodes`, `champion_explodes_sans_variante` | 25 | idem ; le champion meurt à f35 au lieu de f29 (attentes décalées) |
| Q13 | `generated/testbed/familiar_compagnon` | 21 | idem ; `min_hits` 9 → 8, témoin `max` 8 → 7 |
| Q17 | `generated/isaac/boss_gardien`, `generated/isaac/room_combat` | 5 | première frame où le bot a un ennemi en vue : son `pan` est quantifié, donc `CursorPosition` et la rotation du sprite d'arme changent (`FixedTransform3D` de `larmes`, `FacingDirection` `DownLeft` → `Down`) ; attentes tenues (boss à terre f499) |
| Q12, Q10 | aucune | | variantes en fin d'enum, champs optionnels hors hash, aucun contenu existant ne les utilise : suite complète sans `BLESS` |

Identiques au bit près (preuve par la suite) : `familiar_frere` d'`isaac` (le frère tirait déjà `larmes`, le porteur n'a pas
d'objet de larmes), tous les scénarios générés d'`isaac` (épinglés `aim: Free`, traces inchangées), les cinq scénarios de grille
(`grid_floor_seed_*`, `grid_post_boss`, `isaac_floor_seed_1`, épinglés).

## 3. Vérifié (résultats réels, sur le code final)

RÉSULTATS

## 4. Décisions et points à relire

- **Q13 : `weapon` retiré du contrat** plutôt que gardé en repli : « l'arme du porteur » est la seule sémantique ; une arme propre au
  familier serait du code mort. À confirmer. Les tags de la balle restent ceux du familier, `friendly_fire` celui de l'arme du
  porteur ; un fusil à pompe ne donne qu'une balle (pas de plombs).
- **Q17 : deux épinglages `aim: Free`**, à relire. (a) Les scénarios **générés** d'un jeu dont le joueur est `FourWay`
  (`isaac`) : la cible de l'arène est en biais (+128, −48, `AIM_AT_TARGET`) et ne peut pas être touchée à quatre directions ; la
  déplacer (carte `gabarit_armes.ldtk`) aurait déplacé tous les `.trace` générés d'`isaac` et obligé à recaler une vingtaine de
  `min_hits`. (b) Les cinq scénarios de grille menés par `explorateur` : le bot `FourWay` fonctionne mais est plus lent (sur
  `grid_post_boss`, avant T11c : boss verrouillé f712 au lieu de f628, mort f945 au lieu de f862), ce qui décalerait toutes leurs
  attentes ; leur objet est la grammaire de la grille. La visée à quatre directions du bot est prouvée par `boss_gardien`,
  `room_combat` et `aim_four_way*`.
- **Q17 : la visée affichée reste libre** : `system_weapon_position` tourne le sprite d'arme (donc le point de bouche de
  `larmes`, offset (−5, 2)) vers le curseur brut ; seule la direction des balles est quantifiée. Pour un humain en `FourWay`, la
  bouche peut être décalée. À traiter en présentation si William le veut.
- **Q17 : constantes du bot** (file de tir 14, engagement 320) choisies d'après le rayon d'une larme (5) et le demi-corps d'un
  ennemi (~10) ; `crates/bots` est aussi touché par `m3-t14-bot-isaac` (b3) : conflit probable dans `decide.rs`, `view.rs`,
  `input.rs` (petits hunks : `decide` enveloppe `decide_profile`, `BotView::aim`, une ligne dans `input.rs`).
- **Q12 : trois limites assumées.** Déplacement en ligne droite (pas de champ de flux : il ne mène qu'aux joueurs), donc un
  charmé séparé de sa cible par un mur glisse dessus et reste bloqué ; aucun tir pendant le charme ; un charmé ne vise jamais un
  autre charmé. Les autres ennemis ne le visent pas (l'IA ne cible que les joueurs) ; le coup d'un charmé est crédité à lui, pas
  aux joueurs (aucun point).
- **Q10 : « tient `count` objets » lu comme `count` objets distincts** (un exemplaire en double compte une fois, comme les
  transformations d'Isaac) ; une lecture « exemplaires » change une ligne (`held_with_tag`). La teinte est une donnée de la
  synergie (`tint`), pas du joueur.
- Le journal de `docs/taches.md` et la liste des attentes de `CLAUDE.md` sont à jour.

## 5. Relecture indépendante

`ReviewQ10Q12Q13Q17` (agent `reviewer`, lecture seule, sur les quatre commits) : **un P1, deux P2, trois P3 corrigés dans la
fusion.** P1 : un charmé collé au joueur continuait de le griffer (`enemy_melee_attack_system` ne regarde pas l'IA ; `MeleeHold`
n'était posé que pour `Fear`/`Flee`) ; scénario `status_charm_contact` (échoue sans le correctif : le joueur perd 2,5 PV à f46,
f103 et f160 ; passe avec). P2 : un charmé pouvait viser un `Allies` ou un `Neutral` qu'un coup `Players` ne blesse pas (filtre
`Team::Enemies`) ; arrêt hors de portée pour `attack_range` < 30 (`min(OptimalAttackDistance, attack_range)`). P3 : peur + charme
(le charmé apeuré ne frappe plus), ouïe et `Charge` sous charme (aucune règle retenue), lint des mécaniques ignorant `requires_tag`.
Correction du P1 : `enemy_melee_attack_system` retient la hitbox de tout ennemi charmé (`held || charmed(statuses)`), état dérivé
du statut. Première tentative abandonnée : poser `MeleeHold` dans `rules::melee_hold_system` ; elle a révélé un défaut
**préexistant** (`Statuses` est retiré dès qu'il est vide, la requête `BehaviorRuntime`/`Statuses` ne retrouve plus l'ennemi, sa
`MeleeHold` ne se lève jamais : le `breacher` perdait ses coups de mêlée après le charme) dont la correction aurait déplacé
`status_fear` (f301, expiration de la peur) et `generated/isaac/tears_rancune_vive` (f152). **Non corrigé** (hors suivi, à
décider) : pour `Fear`, un ennemi sans règles de comportement garde sa `MeleeHold` après la peur.
Non corrigé : la visée affichée du joueur humain (ci-dessus).
