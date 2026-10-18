# dettes-moteur-d76-d79-d80-d90-d91 : cinq dettes moteur fermées

Branche `dettes-moteur-d76-d79-d80-d90-d91`, base `4630291a` (= `origin/main`, qui n'a pas bougé pendant la tâche : pas de fusion).
Un commit par dette, WIP poussé après chacune. **Seules les traces de D90 bougent** (deux), plus les traces des scénarios nouveaux.

| Dette | Commit | Remède | Preuve |
|---|---|---|---|
| D79 | `6a1e753b` | `CharacterFileSchema.scale` obligatoire (`default_scale` retiré) | 64 fixtures `characters/**` + 3 `player.ron` + 5 RON en ligne de `m3_t12_fixtures` reçoivent `scale: "1.0"` (le chiffre de 53 de la dette était périmé) ; fixture `character_scale_missing`, test `character_without_scale_is_refused_with_a_clear_message` (`Parse`, message « `scale` est obligatoire pour un personnage, ex. `scale: "1.0"` ») ; lint ×5 sans erreur |
| D80 | `5d85d9c8` | `apply_instant_effects` passe par `apply_action_sourced` (`health` devient `Option<&mut Health>`) : une seule liste d'actions ; `Explode` acceptée en `OnUse` | actif testbed `bombe_active`, scénarios `item_active_bomb_rock` (3 rochers cassés à la frame suivant l'usage) et `item_active_bomb_enemy` (dummy proche 200 → 190, porteur 100 → 90, dummy hors rayon intact) ; fixtures `explode_onuse_ok`, `explode_onuse_radius_zero` ; 72 scénarios d'objets, boutiques, bombes, boss rejoués sans trace qui bouge |
| D91 | `d20b32a3` | `held_with_tag` par balayage (passif compté à sa première occurrence, actif s'il n'est pas déjà passif) : plus de `Vec` | un seul appelant (`SynergyDef::is_active`, dont héritent `synergy_modifiers_system`, `projectile_mods`, les effets) ; test `held_with_tag_compte_les_objets_distincts` ; 18 scénarios de synergies/transformations/larmes inchangés |
| D90 | `d0c7a77a` | `melee_hold_system` voit aussi `With<MeleeHold>` (`Statuses` retiré dès qu'il est vide) | **BLESS** de `status_fear` et `generated/isaac/tears_rancune_vive`, preuve §5 ci-dessous ; test `la_retenue_d_une_peur_finie_est_levee_sans_regles_de_comportement` |
| D76 | `3474650b` | docs, `Explosive` branchée, `Run.flags` retiré | voir §3 |

## 1. D90 : preuve §5 (dumps `ALACOD_DUMP_TRACE`, avant/après le correctif, même branche, `scripts/trace-diff.py`)

- `status_fear` : première frame qui diffère **301**. Dans le dump « avant », le `follower` (net id 12) perd `Statuses` à f301
  (fin de la peur) et garde `MeleeHold` pour toujours ; « après », `MeleeHold` est retirée à f301, **avec** `Statuses`.
  `trace-diff --ignore MeleeHold` : « identique (399 frames comparées) ». Rien d'autre ne bouge.
- `generated/isaac/tears_rancune_vive` : première frame qui diffère **152** (la peur finit, `Statuses` retiré à f152 ; la
  retenue est levée avec elle). `--ignore MeleeHold` : la première différence restante est f153 : la `cible_mordante`,
  libérée, frappe le joueur (`DamageEvent` mêlée, hitbox : 27 → 28 entités), la peur est reposée (`Statuses` revient à
  f153) et la retenue revient à f154. Rien n'a bougé avant f152. Les écarts de f153 en aval sont la conséquence de l'attaque
  permise par la retenue levée, pas d'un autre effet.
- Les deux frames (f301, f152) sont celles que la dette annonçait.

## 2. D80 : décisions

- `apply_action_sourced` est l'exécution commune (effets de personnage, objets tenus, frise de boss, `OnUse`). Les actions que
  le lint refuse à `OnUse` (`GaugeAdd`, `SpawnPattern`, `Scaled`, `CurrencyMultiplier`…) restent refusées ; l'exécution commune
  les signalerait par un `warn!` (action non exécutée) au lieu de les ignorer en silence. L'`EffectState` passé est un
  `default()` local : aucune action `OnUse` autorisée ne l'écrit.
- `Explode` d'un actif blesse son porteur (tir ami `Always`, comme la bombe de joueur) : documenté dans `bombe_active.ron` et §69.
- Appelants migrés : `item_consumable_pickup_system`, `item_use_active_system`, `shop::shelf_interaction_system` (tous reçoivent
  `EffectAssets`, déjà requis par le jeu complet).

## 3. D76 : décisions

- **Docs** : §3 annonce maintenant 31 kinds (`KNOWN_KIND_NAMES` fait foi ; la dette en comptait 26, il en est venu depuis) ;
  §40 : le doc disait déjà « quinze gabarits » (M2-T14d) ; compté dans les niveaux de `salles.ldtk` (1 départ, 4 combat,
  1 récompense, 1 boutique, 2 boss, 3 arène, 1 escouade, 1 coffre, 1 secret = 15), détail ajouté ; sommaire régénéré depuis
  les `## ` (80 entrées, dans l'ordre du fichier, jusqu'au §83 ; §40/§41, §49/§45, §51/§50 y sont dans l'ordre où ils ont été écrits).
- **`blast_radius` : branché, pas retiré.** Raison : retirer la variante `Explosive` décale le discriminant de `Piercing` dans le
  `derive(Hash)` de `BulletType` (donc `Bullet`, sous checksum) et déplacerait les traces des balles perforantes ; retirer seulement
  les champs laisserait une variante qui ne fait rien. `Bullet::blast` + `spawn_blast` (`combat::weapons`) posent **la bombe à
  mèche d'une frame de `Action::Explode`** (aucun second chemin) quand une balle explosive se termine : arrêt contre un
  personnage/mur, fin de portée (balle ordinaire), fin de projectile composable (`projectile_expire_system`). Rayon
  `blast_radius`, dégâts `damage × explosive_damage_multiplier`, tireur crédité, équipe et tir ami de l'arme, `destroys: false`.
  **Aucune trace existante ne bouge** : aucun contenu ne déclarait `Explosive` (`grep` sur `games/`, `templates/`, `tests/`,
  `crates/*/tests` : seules les deux armes ajoutées) et les `GgrsNetId` ne sont alloués qu'à la fin d'une balle explosive. Testbed :
  armes `balle_explosive` (sans `projectile:`, portée 100) et `balle_explosive_composee` (`Lifetime(20)`) ; scénarios
  `explosive_bullet_hit` (dummy 1 : 200 → 185 balle + souffle ; dummy 2 derrière : 190 souffle seul ; dummy 3 hors rayon et joueur
  intacts), `explosive_bullet_range`, `explosive_bullet_composed` (balle éteinte sans toucher : dummy voisin 200 → 190), plus
  les scénarios générés `weapon_balle_explosive[_composee]`. La relecture a relevé qu'une balle annulée (blank, bouclier de
  familier, effacement hostile) disparaît sans exploser : documenté comme choix (§69). Pas de lint propre à `Explosive` (aucun
  lint de `bullet_type` n'existe).
- **`Run.flags` : retiré** (ni lu ni écrit ; un lecteur sans écrivain serait un lecteur de rien). Le retrait seul aurait changé le
  hash de `Run` (préfixe de longueur de l'ensemble vide) et déplacé toutes les traces : `impl Hash for Run` écrit à sa place le
  hash d'un `BTreeSet` vide ; test `hash_de_run_inchange_apres_le_retrait_de_flags` (égal au hash du tuple d'avant). Un futur champ
  de drapeaux reprend cette place (vide : même hash). Le `Debug` de `Run` perd `flags: {}` (dump détaillé seulement).

## 4. Vérifié (résultats réels)

| Commande | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (HEAD de D76, avant ajout des deux scénarios générés) | 647 scénarios, 3 tranches ok, 285 s, aucun BLESS sauf les traces de D90 et des scénarios nouveaux |
| `make test_scenarios_par SHARDS=3` (arbre final, avec `weapon_balle_explosive*`) | **649 scénarios**, 3 tranches ok, 500 s (machine chargée), aucun BLESS dans ce passage ; suite verte sur l'état final, `origin/main` = `4630291a` inchangé |
| `make test_crates` | exit 0 ; 80 binaires, **1 224 passés, 0 échec**, 14 ignorés (1 218 avant : +6 tests : D79, D80 ×2, D91, D90, D76) |
| `make lint` ×5 | zombies, testbed (25 armes), throne, gungeon, isaac : « aucune erreur » |
| `make gen GAME=…` ×5 | zombies, throne, gungeon, isaac : tout `ok` ; testbed : tout `ok` sauf les deux nouvelles armes (`trace absente`, normal) puis `GEN_BLESS=1` : seuls 4 fichiers neufs (aucune trace existante réécrite) |
| `cargo fmt --all -- --check` | ok |
| `./scripts/check-sim-boundary.sh` | OK : 13 crates SIM, 2 en dette D69 |
| `./scripts/check-forbidden.sh` | OK, 38 occurrences auditées |
| `./scripts/check-rollback-registration.sh` | OK |
| `make test_tools` | 25 tests OK |
| fuzz `ALACOD_FUZZ=0:8` | ok (8 graines × 3 600 frames, synctest sans divergence) |
| fuzz `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` | ok (8 explosions par graine, sans divergence) |
| p2p `p2p-stress.sh zombies` (3 000 frames, 2 clients, charge machine 10 pour un seuil de 6) | OK : codes 0 0, 0 `DesyncDetected`, 6 972 frames resimulées, 2 999 lignes de trace identiques |
| Relecture indépendante (agent `reviewer`, lecture seule) | aucun P0/P1 ; P2 repris : balle annulée sans explosion documentée, tir ami d'`Explode` en `OnUse` documenté |

## 5. Non fait / incertain

- Le p2p ne joue pas les nouveaux contenus (`bombe_active`, balles explosives : aucun montage p2p dédié) ; leur déterminisme est
  prouvé par le synctest des scénarios (rollback à chaque frame) et le fuzz, pas par un p2p réel.
- Pas de lint de `blast_radius`/`explosive_damage_multiplier` (aucun lint de `bullet_type` n'existe dans `content`).
- Dette ouverte inchangée : D69 étape 2.
