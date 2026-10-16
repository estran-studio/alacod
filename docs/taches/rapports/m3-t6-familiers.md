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
- **Corrections de revue** (verdict bloquant, trois points de code et un de scénario) :
  1. `MirrorShot` se déclenche sur un **tir réellement parti** : événement transitoire `sim_core::triggers::WeaponFired`
     émis par `weapon_rollback_system` (hors checksum et snapshot), lu par `owner_fired` (pure). Plus de lecture de
     `WeaponState::last_fire_frame` (0 par défaut), plus de garde-fou `input.fire`. Test unitaire du cas frame 0 /
     tir refusé par la cadence ; scénario `familiar_mirror_cadence` (gâchette tenue dès f0, la cadence refuse jusqu'à f20).
  2. `BlockBullets` : l'AABB balayée ne présélectionne plus que ; contact décidé par `swept_hits_collider` (segment
     balayé contre disque ou rectangle, `FixedWide`). Tests unitaires : passage en coin et disque en diagonale ne
     bloquent pas, traversée, extrémité, parallèle, segment nul, centre décalé.
  3. `orbital_blocks_bullet` : attentes `BulletCount` (`fireball`) : au bord sans contact (f125, f185) même compte que
     le témoin, au contact (f126, f186) un de moins ; témoin `_sans_objet` aux mêmes frames.
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

## 2. Vérifié (résultats réels, sur `origin/main` `803bf788` (M3-T0b incluse) + la branche, après les corrections)

- `make test_scenarios_par SHARDS=3` **sans BLESS des traces existantes** : 401 scénarios, 3 tranches « ok » (116 s) ;
  `git status` propre hors fichiers neufs. (Une première exécution a été tuée par un signal extérieur,
  « Terminated: 15 », sans résultat ; rejouée seule : verte.)
- `make test_crates` : 929 tests passés, 0 échec, 13 ignorés.
- `make lint` ×5 : « aucune erreur » (zombies, testbed, throne, gungeon, isaac).
- `make gen` ×5 : testbed (62 ok), zombies (16), throne (39), gungeon (55), isaac (10), aucun diff git.
- `cargo fmt --all -- --check` propre ; `check-forbidden.sh` OK (39 occurrences auditées) ;
  `check-rollback-registration.sh` OK ; `make check_sim_boundary` : 11 crates, 5 en dette D69 ; `make test_tools` OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines de 3 600 frames sans divergence (graine 4 : vague 2, 7 kills, comme avant).
- **Traces bougées par les corrections : une seule, `orbital_blocks_bullet.trace`** (rebénie seule). Preuve : à
  f246 (ligne 247) l'ancienne trace compte 55 entités, la nouvelle 56. En rétablissant temporairement l'ancien test
  (AABB seule) avec le code corrigé, l'ancienne trace redevient exacte : la cause est donc le seul correctif 2 (une
  flèche qui frôlait un coin de l'orbital était détruite à tort). Les attentes ajoutées ne bougent pas une trace ;
  `orbital_blocks_bullet_sans_objet` (sans orbital) et les 7 autres traces de familiers sont inchangées ;
  `familiar_mirror_cadence` est neuve (bénie).
- **p2p à deux clients avec prédiction : non rejoué, Docker indisponible sur cette machine** (`docker` absent). Reste à
  faire sur l'autre machine ; le synctest de `familiar_quad` (resimulation et checksums à chaque frame) passe, mais ne
  couvre pas un ramassage ou un remplacement prédit puis annulé.
- **Mesure `familiar_quad`** (précédente, indicative) : `advance_ms_avg` 0,098 ms (`four_players_idle` : 0,076),
  `rollback_ms_worst` 1,34 ms.

## 3. Non fait, incertain, non vérifié

- **p2p à deux clients** (Docker) : non rejoué (Docker indisponible, voir §2). Les nouveaux états rollback (`FamiliarOf`, `FamiliarGun`) sont
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

- `familiar_mirror_shot_system` se déclenche sur `WeaponFired` (tir à distance parti) : le coup d'une arme de mêlée ne
  déclenche rien.
- `familiar_block_bullets_system` s'exécute entre `bullet_rollback_system` et `bullet_rollback_collision_system` ; les
  balles que les émetteurs ennemis font naître dans `Weapon` après lui ne sont testées qu'à la frame suivante.
- Les logs (`familiar_spawn`, `familiar_lost`, `familiar_shot`, `familiar_block`) suivent le format `ggrs{f=…}` et ne
  nomment que des `GgrsNetId`.

## 6. Atteignabilité

Aucune partie ordinaire ne rencontre encore un familier. **Dette nommée, à fermer par M3-T11b** : au moins deux
familiers et un orbital dans le pool trésor d'`isaac` (graine à nommer à ce moment).

## 7. p2p à deux clients sous prédiction (testbed, Docker OrbStack)

Binaire `testbed` (`cargo build -p testbed --profile headless --no-default-features`), signaling matchbox du
`docker-compose.ci.yaml` (`MATCHBOX_PORT=3536`, `COMPOSE_PROJECT_NAME=b0`), script bash. Le testbed n'a aucun objet à ramasser
dans une partie : un **système temporaire, non commité**, ajouté à `games/testbed/src/main.rs` (puis annulé par `git checkout`),
posait à des frames fixes des objets aux pieds des joueurs (`compagnon_actif` f20 et `eclat` f200 pour le joueur 0 ;
`essaim` f20 et `orbital` f260 pour le joueur 1). Inputs scriptés (`ALACOD_INPUT=script:` : ramassages par `Interaction` à
f40, f210, f270 ; visée et tir qui changent toutes les 6 frames ; marche f300-340), `ALACOD_INPUT_DELAY=0`, second client lancé 8 s plus tard :

```
ALACOD_INPUT_DELAY=0 ALACOD_HEADLESS=1 ALACOD_INPUT=script:/tmp/p2p_script.ron ALACOD_STATE_TRACE=/tmp/p2p-$i.trace \
ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x ../target/headless/testbed --matchbox ws://127.0.0.1:3536 --lobby test-$$ \
  --number-player 2 --players localhost remote --cid client_$i --name client_$i
```

- `cmp /tmp/p2p-0.trace /tmp/p2p-1.trace` : **identiques**, 599 lignes chacune.
- Frames resimulées : le compteur du log final (`frames resimulées après rollback`) affiche 0 (1 sur un essai précédent) ;
  il ne reflète pas ce qui s'est passé : en comptant dans les logs les frames dont `ggrs{f=N …}` apparaît plusieurs fois,
  **client_0 : 66 exécutions en double, client_1 : 63** (sur 600 frames), donc de vrais rollbacks.
- Création sous prédiction : à f40 le client 0 a joué deux fois la frame (spawns de `compagnon`, 2 `orbital` du joueur 31
  rang 0 et 1), à f210 `eclat` remplace `compagnon_actif` (`familiar_lost owner=26 rank=0`), à f270 `orbital` ajoute le rang 2.
  Cas de prédiction annulée : à f211 le client 0 a d'abord rejoué un ramassage de `compagnon_actif` et le spawn d'un familier
  prédits, puis la resimulation de f211 n'a plus ni l'un ni l'autre ; le client 1 n'a jamais vu ces lignes. Les états finaux
  concordent (traces identiques).
- Logs filtrés : `make diff_log` ne dédoublonne pas les frames resimulées ; comparaison des lignes `familiar_*` et
  `item_pickup` dédoublonnées (`sort -u`) : 21 lignes (client 0) contre 19 (client 1), la seule différence étant les deux
  lignes de la frame 211 prédite puis annulée ci-dessus.
- `familiar_shot` : 10 tirs de familier de f105 à f213 sur les deux clients, aux mêmes frames.
