# M3-T1 : larmes v3 (tir composé, `Spectral`, `Split`, `Orbit`, `Poison`, `Fear`, chance d'effet)

Branche `m3-t1-larmes-v3` (depuis `origin/main` `5d9f854b`, qui contient M3-T0a sous le nom
`m4-t0a-contrats-larmes`, T0c, T0d, T0f ; `origin/main` n'a pas bougé pendant la tâche, la fusion finale est
« déjà à jour »). Conventions : `docs/conventions.md` §66. Session b0 (pas de Docker : p2p non rejoué).

## 1. Fait

- **Tir composé** : `weapon_rollback_system` lit `items::Inventory` du tireur, `ItemTable`, `SynergyTable`
  (`items::projectile_mods`, nouvelle, rend les `ProjectileMods` entiers ; `projectile_sources` en dérive) et
  compose, par `Projectile::composed`, le `projectile:` de l'arme avec ceux des objets non vides (règles de
  `compose`, §60). `on_hit` et `on_expire` des objets s'ajoutent à ceux de l'arme dans l'ordre de §60. Sans
  objet qui ajoute quelque chose : chemin d'origine (`Projectile::new` ou balle ordinaire), au bit près. Une
  balle ordinaire devient un `Projectile` dès qu'un objet ajoute quelque chose. `combat` dépend de `items`.
- **`Projectile`** (rollback) gagne `spectral`, `split`, `orbit`, `luck` avec un **`Hash` manuel** : ces champs
  ne sont hachés que s'ils sont actifs (le hash d'un projectile d'avant est inchangé, `Debug` idem). Règle
  ajoutée à la checklist de `CLAUDE.md`.
- **`Spectral`** : la collision saute la branche des murs. **`Split`** : à chaque impact sur un personnage,
  `spawn_split_children` (éventail de `spread` centré sur la direction de vol ; `child: Some(id)` = entrée de
  la table de l'arme, `None` = copie du parent sans `Split`/`Orbit`/`on_expire`) ; l'enfant hérite de la liste
  des cibles déjà touchées, `MAX_GENERATION` respectée. **`Orbit`** : `projectile_steering_system` +
  `orbit_position` (pure). Lint `Unsupported` retiré pour les trois.
- **Statuts v2** : `StatusDef::{Poison, Fear}` en fin d'enum ; `Poison` = règles de `Burn`, dégâts de genre
  `DamageKind::Poison` ; `Fear` : `move_enemies` impose le recul de `Flee` (`rules::flee_motion`, extrait de
  `behavior_motion`), plus d'attaque au corps à corps (`enemy_attack_system`, `MeleeHold`). Contenu
  `statuses/poison.ron`, `peur.ron` ; lint (`Poison` : `damage > 0`, `period > 0` ; `frames > 0`).
- **Chance d'effet** : `Action::Chance { base, per_luck, then }` (fin d'enum), `effects::chance::roll_chance` /
  `chance_seed` (RNG local, graine `(graine de run, net id du projectile, rang)` mélangée par splitmix64),
  `Luck` lue au tir (gardée seulement si le projectile porte une `Chance`), résolue dans
  `projectile_collision_system`. Aucun `RngStreams` consommé (la graine de run est lue seule).
- **Lint** : `Spectral`/`Split`/`Orbit`/objets ne sont plus refusés ; `Orbit` + `Homing`/`Gravity` refusé ;
  cycle de `Split` dans la table refusé ; `Chance` (base `[0, 1]`, `then` autorisé, statut vérifié à travers) ;
  objet : `Split` sans `child`, `on_expire` sans `Spawn` (pas de table `projectiles` à lui) ; `Chance` refusée
  hors `on_hit` de projectile (effets, power-ups, boss). Six fixtures nouvelles ou renommées
  (`projectile_contract_ok`, `item_projectile_ok`, `item_projectile_no_table`, `projectile_chance_out_of_range`,
  `projectile_split_cycle`, `status_poison_fear_out_of_range`) ; `item_projectile_out_of_range` et
  `projectile_split_out_of_range` gardées.
- **Testbed** : carte `props_room`, objets `guide`, `perforant`, `rebond`, `spectre`, `division`, `orbite`,
  `venin`, `trefle`, armes `status_poison`/`status_fear`, statuts `poison`/`peur`.
- **Scénarios** (traces neuves, bénies une à une) : `tears_item_homing` (+ témoin `_sans_objet`),
  `tears_stack_two_copies`, `tears_stack_three`, `tears_spectral_wall` (+ témoin), `tears_split`, `tears_orbit`,
  `status_poison`, `status_fear`, `tears_luck_chance`, `tears_luck_stat` (12) ; générés (`make gen GAME=testbed`) :
  `weapon_status_poison`, `weapon_status_fear`, `item_{guide,perforant,rebond,spectre,division,orbite,venin,trefle}`.
  Chaque scénario a son commentaire « À regarder » et des attentes exactes (frames d'impact, santé, statuts).
- **Unitaires** : `compose` : **toutes les paires ordonnées de variantes différentes** (81 − 9 = 72 paires, aucune
  interaction : `compose([a], [[b]]) == resolve([a, b]) == compose([b], [[a]])`), les neuf paires identiques (une
  règle chacune), trois sources ; `Projectile::composed` (ordre, identité sans objet) ; hash du projectile ;
  `resolve_chances` ; `orbit_position` ; `Statuses` (`Poison`, `Fear`) ; `roll_chance` (déterminisme, bornes,
  fréquence sur 4 000 net ids) ; `items::projectile_mods`.
- **Diagnostic** : `projectile_probe` (`ALACOD_PROJECTILES=<scénario>:<de>:<à>`), `map_probe` liste les personnages
  non joueurs. `sim_core::stats::default_base` donne 0 à `Luck` (attente `Stat`, générateur).
- Doc : conventions §66, `CLAUDE.md` (liste des attentes : aucune nouvelle ; checklist : champ ajouté à un composant
  sous checksum), journal `docs/taches.md`.

## 2. Vérifié (résultats réels, sur `origin/main` `5d9f854b` + la branche, avant le dernier commit)

- `make test_scenarios_par SHARDS=3` **sans BLESS des traces existantes** : 390 scénarios fusionnés, 3 tranches
  « ok » (151 s) ; **aucune trace existante modifiée** (`git status` propre hors fichiers neufs), `tears_*` de
  M3-T0a comprises. Avant tout contenu neuf (code seul sur les 368 scénarios d'avant) : même résultat, 145 s.
- `make test_crates` : 898 tests passés, 0 échec, 13 ignorés (avant le test `items::projectile_mods` ajouté
  ensuite : `cargo test -p items` : 17 passés).
- `make lint` ×5 : « aucune erreur » pour zombies, testbed, throne, gungeon, isaac ; avertissements
  d'atteignabilité attendus pour le contenu de laboratoire neuf (`guide`, `perforant`... « aucune source »,
  `status_fear`/`status_poison` « aucun personnage... ne la donne »).
- `make gen` ×5 : testbed (bénie : 10 traces neuves, les 52 autres réécrites à l'identique, aucun diff git),
  zombies, throne, gungeon, isaac sans diff.
- `cargo fmt --all -- --check` : propre. `./scripts/check-forbidden.sh` : OK (39 occurrences auditées).
  `./scripts/check-rollback-registration.sh` : OK. `make check_sim_boundary` : 11 crates, 5 en dette D69.
  `make test_tools` : 17 tests OK.
- Fuzz `ALACOD_FUZZ=0:8` (`fuzz_inputs`, 3 600 frames) : 8 graines sans divergence (graine 4 : vague 2, 7 kills,
  comme avant la tâche).
- `content::schema::every_kind_has_a_schema` : vert (`Action::Chance` récursive comprise).

## 3. Non fait, incertain, non vérifié

- **p2p à deux clients** (Docker) : non rejoué (à faire par l'orchestrateur). Aucun état rollback nouveau :
  `Projectile` change de forme mais pas de hash tant que rien de neuf n'est actif ; les nouveaux contenus
  (objets de larmes) n'existent que dans le testbed.
- **Bench** : non mesuré (consigne : T15). `bench_bullets_room` dans les métriques de la suite (machine
  partagée, donc bruitées) : 180 sim fps (final) contre 165 (code seul, avant le contenu), sans valeur de
  comparaison avec `main`. Coût attendu : une requête `Inventory` et une allocation vide par tir ; une
  recherche du tireur par frame et par projectile en orbite seulement.
- Scénarios jamais vus à l'écran : aucune vidéo (`make videos`) produite ; les « À regarder » décrivent ce
  que les sondes `projectile_probe` ont mesuré.
- Visuels des larmes, HUD, bots : hors tâche (T17, T16, T14).

## 4. Questions pour William et décisions prises

- **Q12 (charme)** : non codé. Le contrat est prêt pour l'accueillir : un `StatusDef::Charm` en fin d'enum et un
  équivalent de `flee_motion` qui poursuit un autre ennemi ; il faut décider qui il attaque (l'ennemi le plus
  proche de l'équipe adverse ? un seul coup avant de se calmer ?) et son effet sur les ennemis qui visent un
  joueur (changement d'équipe ou seulement de cible).
- **Q17 (visée à 4 directions)** : non codé. Les 360° actuels ne changent pas. Une option de personnage
  (`CharacterConfig::aim: FourWay`) quantifierait `aim_dir` dans `weapon_rollback_system` ; le piège : le
  curseur/pan est une donnée d'input rollbackée, la quantification doit rester déterministe (comparaison sur
  `Fixed`, pas d'angle flottant).
- **Q13 (familier qui hérite des larmes)** : non codé (T6 n'existe pas). `Projectile::composed` prend ses
  ajouts en paramètre : T6 y passera `items::projectile_mods` du porteur si l'héritage est retenu.
- **Représentation de la chance** : la fiche dit `chance: Some(base, per_luck)` sur un `on_hit` ; j'ai fait une
  variante d'action `Action::Chance { base, per_luck, then }` (fin d'enum), qui garde `on_hit: Vec<Action>`
  (aucun changement de hash des armes existantes) et se lit aussi dans une arme. À confirmer.
- **`Split` sans `child`** (Q3, « projectile par défaut du porteur ») : choisi = copie du parent (dégâts, vitesse,
  portée pleins, modificateurs restants hors `Split`/`Orbit`/`on_expire`). Trois enfants de dégâts pleins
  triplent le dégât total : un contenu qui veut moins déclare un `child` de table.
- **Un objet n'a pas de table `projectiles`** : `Split` avec `child` et `on_expire: Spawn` refusés par le lint
  pour un objet (le runtime les ignorerait avec un `warn!`). Si T11a veut des éclats d'objet, il faudra une table
  d'objet (ou l'id d'une arme).
- **Orbite** : sens trigonométrique, rayon qui croît de 0 au rayon en 10 frames, centre = position courante du
  tireur ; la longueur d'arc compte dans la portée (un tour de rayon 40 en 60 frames fait ~4 u/frame : la larme
  d'`orbite` tient environ 170 frames à portée 700, `Lifetime` 240 y est donc plus long que la portée).
- **`Fear`** ne dépend pas des règles de l'ennemi (il fuit même sans `Flee`), et `Poison` = `Burn` à la
  nature de dégâts près (pas de règle propre : à discuter si le poison ne doit pas s'empiler).
- **`ReloadSpeed`** : dette D74 inchangée (hors périmètre).

## 5. Dettes et points à relire

- `projectile_collision_system` et `weapon_rollback_system` prennent des paramètres de plus
  (`Commands`, `GgrsNetIdFactory`, `RngStreams` en lecture ; `Inventory`, `ItemTable`, `SynergyTable`) : à
  relire pour l'ordre des allocations de `GgrsNetId` (collision → expiration, ordre `GgrsNetId` du parent).
- `melee_hold_system` itère désormais aussi les ennemis qui portent des `Statuses` (filtre
  `Or<(With<BehaviorRuntime>, With<Statuses>)>`).
- `Projectile::luck` n'est lue que si l'`on_hit` porte une `Chance` (sinon le hash d'un porteur de `Luck`
  changerait sans raison) ; une `Chance` posée sur un `ProjectileDef` de table hérite de la chance du tireur.

## 6. Atteignabilité

Rencontré quand M3-T11a met les objets de larmes dans le pool trésor d'`isaac` (une salle de trésor par étage) ;
d'ici là, scénarios testbed seulement : **dette nommée, à fermer par M3-T11a** (objets de larmes, `Split` d'objet).
