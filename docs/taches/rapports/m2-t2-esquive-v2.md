# M2-T2 — Esquive v2

Branche `m2-t2-esquive-v2`. Conventions : `docs/conventions.md` §43.

## Ce qui est livré

- **Charges** : `dash_charges` / `dash_recharge_frames` (`MovementConfig`) ou `charges` / `recharge_frames`
  (`DashConfig`). Le délai v1 (`cooldown_frames`) reste l'écart minimal entre deux esquives.
- **Variantes nommées** : `dash: Some((...))` dans le RON du personnage. `effect: Roll | Shove | InPlace`
  (nouvelles variantes en fin d'enum), distance, durée, frames d'invulnérabilité.
  - `Shove` : l'esquive repousse (`Velocity.knockback`) les corps qu'elle traverse (`dash_shove_system`).
  - `InPlace` : distance forcée à 0, la durée ne compte que pour l'invulnérabilité.
- **I-frames contre projectiles seuls** : `iframes_kind: Projectiles` (ou `All`, défaut). Stockées dans
  `DashExtra.projectile_iframes_until`, **séparément** de `Health.invulnerable_until_frame` : le contact
  mêlée reste actif pendant l'esquive. Lues par `combat::projectile` et par le tir historique
  (`weapons/mod.rs`).
- **Lint** (4 refus + fixtures) : charges 0, recharge 0 avec charges, durée 0 hors `InPlace`,
  i-frames > durée. Actif seulement si le personnage déclare `dash` ou `dash_charges`.
- **Gungeon** : le pilote roule avec 2 charges (recharge 60 frames) et des i-frames projectiles ; le
  mouvement (64 px, 8 frames, délai 24, 6 i-frames) est inchangé. Zombies, testbed, throne : inchangés.
- **Scénarios** : `dash_charges` (gungeon), `dash_shove`, `dash_in_place`, `dash_jump_bullets` (testbed,
  variante imposée par `PlayerScript.dash`, sans toucher au contenu du jeu).

## Décision de conception : composant séparé et neutre

`DashState` est sous checksum et porté par tous les joueurs : toute nouvelle donnée dedans aurait
déplacé les traces des quatre jeux. L'état nouveau vit dans `combat::actors::DashExtra`, enregistré par
`rollback_and_trace_neutral` (contribution 0 sans porteur) et **posé seulement** sur les personnages qui
ont des données d'esquive v2. `DashState` et son hash sont intacts. `OnDodge` (M2-T3) peut donc se
brancher sur `is_dashing`, qui passe toujours à vrai à la frame du déclenchement.

## Preuve §5 (traces gungeon)

13 traces gungeon ont changé (`bench_floor`, `gungeon_{boss,clear,start,two_floors}`,
`gungeon_floor_seed_{1,23,123}`, et les générés `enemy_bullet_kin_{moving,still}`, `weapon_{arsenal,
bare_hands,pistolet}`) : toutes parce que le pilote porte maintenant `DashExtra`.

Preuve : `ALACOD_DUMP_TRACE` sur `f13035d6` (main) et sur la branche, `scripts/trace-diff.py … --ignore
DashExtra` sur `gungeon_start` (119 frames), `weapon_pistolet` (199), `enemy_bullet_kin_moving` (599) :
**identiques**. Sans `--ignore`, la seule différence est la ligne du joueur (le composant). Aucune trace
existante hors gungeon n'a bougé (`dash_*`, `movement_melee`, `surface_ice`, zombies, testbed, throne).

## Pièges rencontrés

- `Option<DashConfig>` dans `PlayerScript` impose `PartialEq` sur `DashConfig`.
- Fixture `dash_duration_zero` : `effect` doit être un enum dans le schéma de lecture, pas une `String`.
- `dash_jump_bullets` : `coward` / `breacher` posés par `characters:` ne font aucun dégât mêlée ; le
  scénario utilise `charger` (contact) et `tourelle_salve`. Le délai d'entrée de 5 frames décale l'effet
  d'un appui.
- Preuve §5 : un artefact `combat` de la build de base a été réutilisé dans le target partagé
  (`DashExtra` introuvable) ; `touch` des sources puis relance.

## Non fait (hors périmètre)

Parade, `OnDodge` (M2-T3), présentation de l'esquive, équilibrage.
