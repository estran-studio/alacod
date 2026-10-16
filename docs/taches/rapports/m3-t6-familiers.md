# M3-T6 : familiers et orbitaux en jeu

Branche `m3-t6-familiers` (depuis `origin/main` `c0aceb3e`, qui contient M3-T0d et M3-T1 ; `origin/main` n'a pas
bougé pendant la tâche, la fusion finale est « déjà à jour »). Conventions : `docs/conventions.md` §71. Session b0
(pas de Docker : p2p non rejoué).

## 1. Fait

- **Apparition et destruction, dérivées de l'état** (`game::familiars::familiar_sync_system`) : chaque frame, les
  familiers voulus d'un joueur (`desired_familiars` : objets tenus, passifs dans l'ordre de ramassage puis actif,
  `per_copy` par exemplaire) sont comparés aux existants (`FamiliarOf`) ; `reconcile` garde le plus long préfixe commun
  et détruit (`despawn_rollback()`) / crée le reste. Un objet ramassé en crée, l'actif remplacé détruit les siens. Aucun
  événement à rejouer. `GgrsNetId` : porteurs par net id croissant puis rang croissant.
- **Un familier est un personnage** (`create_character`), équipe de sa fiche sinon celle du porteur, **sans `Enemy` ni
  `Player`** (non compté par salles/vagues/HUD, non ciblé), couche `environment` (aucun obstacle, ne gêne aucun joueur),
  **immunisé** par `Health.invulnerable_until_frame = u32::MAX` (traversé par balles, projectiles et mêlée).
- **`Follow`** (file : le premier vise le porteur, chacun des suivants le précédent), **`Orbit`** (position déduite du
  numéro de frame, phase = rang parmi les orbitaux × 2π / leur nombre), **`MirrorShot`** (quand l'arme active du porteur
  a tiré ; arme du familier ; une fois par `every` frames, `FamiliarGun` neutre), **`BlockBullets`** (balle hostile dont
  l'AABB balayée touche le familier : `despawn_rollback`). Fonctions pures et testées dans `behaviors::familiar`.
- **Attente** `FamiliarDistance { handle, rank, to_rank, min, max, at_frame }` ; `FamiliarCount` (T0d) compte de vrais
  familiers.
- **Lint** : un personnage cité par un objet `familiar` n'est plus signalé « n'apparaît dans aucune salle ».
- **Testbed** : personnages `orbital`, `rempart` (+ `compagnon` de T0d), objets `compagnon_actif`, `orbital`, `essaim`
  (`per_copy: 2`), `trio` (`per_copy: 3`), `rempart`.
- **Scénarios** (8 traces neuves, bénies ; commentaires « À regarder » ; attentes exactes) : `familiar_follow`,
  `familiar_mirror_shot` (+ témoin `_sans_objet`), `familiar_two_copies`, `familiar_lost_with_active`,
  `orbital_blocks_bullet` (+ témoin), `familiar_quad` (4 joueurs, synctest).
- **Unitaires** : `orbit_phase`, `orbit_position` (rayon constant sur 5 effectifs × 120 frames, périodicité, orbitaux
  opposés, tour de plus de 32 767 frames), `follow_step` (laisse, convergence, file de trois), `FamiliarGun`,
  `desired_familiars`, `reconcile` (cinq cas), lecture des règles d'un familier.
- Doc : §71, `CLAUDE.md` (liste des attentes), journal `docs/taches.md`.

## 2. Vérifié (résultats réels, sur `origin/main` `c0aceb3e` + la branche)

- `make test_scenarios_par SHARDS=3` **sans BLESS des traces existantes** : 398 scénarios (390 + 8 neufs), 3 tranches
  « ok » (121 s) ; **aucune trace existante modifiée** (`git status` propre hors fichiers neufs).
- `make test_crates` : 909 tests passés, 0 échec, 13 ignorés (dont `every_kind_has_a_schema`, `shipped_games_lint_without_error`).
- `make lint` ×5 : « aucune erreur » (zombies, testbed, throne, gungeon, isaac) ; avertissements « aucune source » attendus
  pour `orbital` et `rempart` (contenu de laboratoire).
- `make gen` ×5 : testbed (62 ok), zombies (16), throne (39), gungeon (55), isaac (10), aucun diff git.
- `cargo fmt --all -- --check` propre ; `check-forbidden.sh` OK (39 occurrences auditées) ;
  `check-rollback-registration.sh` OK ; `make check_sim_boundary` : 11 crates, 5 en dette D69 ; `make test_tools` OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines de 3 600 frames sans divergence (graine 4 : vague 2, 7 kills, comme avant).
- **Mesure `familiar_quad`** (4 joueurs, 8 familiers, suite en 3 tranches donc machine partagée, indicatif) :
  `advance_ms_avg` 0,098 ms (`four_players_idle` : 0,076), `save_ms_avg` 0,21 ms, `rollback_ms_worst` 1,34 ms
  (`four_players_idle` : 1,13), 55 entités max, snapshot 18 Kio.

## 3. Non fait, incertain, non vérifié

- **p2p à deux clients** (Docker) : non rejoué. Les nouveaux états rollback (`FamiliarOf`, `FamiliarGun`) sont
  neutres ; `familiar_quad` passe le synctest (resimulation et comparaison des checksums à chaque frame) mais pas
  un vrai rollback par prédiction.
- **Bench** : non fait (T15). Aucune vidéo produite.
- `familiar_motion_system` cherche porteur et suiveur par balayage (O(F²) par frame) : négligeable à 8 familiers, à
  réécrire avec une table si T11b en met des dizaines.
- Un porteur mort : ses familiers restent en place, immobiles, jusqu'à la fin de la partie (proposition de la fiche) ;
  aucun test de ce cas (le scénario de mort du porteur n'a pas de familier).
- Les balles hostiles à tête chercheuse (`Homing`, `Aimed`) visent un familier invulnérable (règle de §16 : « visent
  toujours » un personnage invulnérable).

## 4. Décisions et questions pour William

- **Q13** : la fiche renvoie à la réponse de William ; faute de réponse, **la proposition par défaut du plan
  (`plan-m4-isaac.md`, Q13) et de T0d** : `MirrorShot` tire l'**arme du familier**, sans les modificateurs du porteur.
  Pas de larmes composées du porteur. Passer à « larmes du porteur » : appeler `Projectile::composed` avec
  `items::projectile_mods` du porteur dans `familiar_mirror_shot_system` (changement de ~10 lignes, M3-T1 l'a prévu).
- **Apparition dérivée plutôt qu'événementielle** : un objet perdu au milieu de la liste refait les rangs suivants
  (nouveaux net ids). Choisi pour la robustesse au rollback ; à confirmer.
- **Familier sans `Enemy`** (donc ni `spawn_enemy` ni l'IA ennemie) : évite de fausser les salles, vagues et le HUD.
- **Immunité par `invulnerable_until_frame = u32::MAX`** plutôt qu'un nouveau composant d'immunité : les trois chemins
  de collision le respectent déjà. Si un chantier futur (explosions M3-T4) ne le lit pas, il faudra une règle dédiée.
- **Balle de `MirrorShot` créditée au porteur** (net id et équipe) : kills et points lui reviennent ; elle ne le touche
  jamais. À confirmer.
- Le familier traverse les murs (vol) ; aucune collision avec les joueurs, ennemis ni balles hors `BlockBullets`.

## 5. Dettes et points à relire

- `familiar_mirror_shot_system` détecte le tir du porteur par `WeaponState::last_fire_frame == frame` de l'arme
  **active** : le tir d'une arme de mêlée ne déclenche rien ; au frame 0, `input.fire` est exigé (`last_fire_frame`
  vaut 0 par défaut).
- `familiar_block_bullets_system` s'exécute entre `bullet_rollback_system` et `bullet_rollback_collision_system` ; les
  balles que les émetteurs ennemis font naître dans `Weapon` après lui ne sont testées qu'à la frame suivante.
- Les logs (`familiar_spawn`, `familiar_lost`, `familiar_shot`, `familiar_block`) suivent le format `ggrs{f=…}` et ne
  nomment que des `GgrsNetId`.

## 6. Atteignabilité

Aucune partie ordinaire ne rencontre encore un familier. **Dette nommée, à fermer par M3-T11b** : au moins deux
familiers et un orbital dans le pool trésor d'`isaac` (graine à nommer à ce moment).
