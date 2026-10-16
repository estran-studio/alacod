# M3-T7 : champions et élites v2

Branche `m3-t7-champions` (depuis `origin/main` `af46f192` ; `origin/main` n'a pas bougé pendant la tâche, la fusion
finale est « déjà à jour »). Conventions : `docs/conventions.md` §72. Session b0.

## 1. Fait

- **`chance` en expression** : `draw_variant(…, VariantContext { players, floor })` ; `DifficultyReader` porte le contexte
  (`variant_context_at`), les cinq sites d'apparition le passent (`spawn_enemy`, `spawn_character`,
  `spawn_level_characters`, vagues, spawners, renforts de salle, placements de scénario ; `floors.rs` : étage suivant).
  Un littéral est exact (même tirage).
- **`behaviors` d'une variante** : remplacent ceux du personnage à la création (`ai.ranged` suit la première règle `Shoot`).
- **`on_death`** (`game::champions`, `champion_death_system`, `DeathManagement`) : `SpawnCharacters` (anneau de 14 unités,
  décalages fixes, équipe du mort, **sans variante** : pas de cascade), `Explode` (file `PendingBlasts`, rollback neutre,
  `DamageEvent` à la frame suivante, tir ami, **respecte `is_invulnerable_at`**), `SpawnPattern` (émetteur fantôme
  `PatternGhost`, détruit à la fin de sa séquence), `loot` (tirage local `champion:<net id>`, objet ou arme au sol).
- **`tint`** : champ disponible, aucune lecture dans la simulation (T17).
- **Lint** : un `SpawnPattern` à la mort refuse un pattern infini (`Ring` avec `every > 0`) ; fixture
  `champion_death_infinite_pattern`.
- **Testbed** : `elite_explosif` (`explosif`, `salve`, `nu`), `elite_divise`, `elite_riche`, `elite_comportement`,
  `elite_chance` ; pattern `couronne_unique` ; séquence `deux_vides` (+ cartes `floor_vide_a/b`).
- **Scénarios** (11 traces neuves bénies, « À regarder », variante forcée + témoin `_sans_variante`) : `champion_explodes`,
  `champion_splits`, `champion_drops`, `champion_behaviors` (chacun avec témoin), `champion_pattern`,
  `champion_chance_floor_0`, `champion_chance_floor`.
- **Unitaires** : `death_plan`, `split_offsets`, `champion_key`, `chance_en_expression_suit_joueurs_et_etage` (monotonie
  par net id, littéral indépendant du contexte), lint de pattern infini.
- Doc : §72, `CLAUDE.md` (liste des attentes : aucune nouvelle), journal.

## 2. Vérifié (résultats réels, sur `origin/main` `af46f192` + la branche)

- `make test_scenarios_par SHARDS=3` **sans BLESS des traces existantes** : 412 scénarios (401 + 11), 3 tranches « ok »
  (130 s) ; `git status` propre hors fichiers neufs : **aucune trace existante modifiée** (aucune preuve §5 nécessaire).
- `make test_crates` : 935 tests passés, 0 échec, 13 ignorés. Deux corrections de tests existants, dues à la nouvelle
  ressource d'événements : `hit_iframes.rs` et `expectations.rs` ordonnent maintenant leurs systèmes après
  `champion_blast_system` (qui est après `barrel_explosion_system`) ; la première exécution avait 5 échecs
  (ambiguïté d'ordre du `GgrsSchedule` sur `FrameEvents<DamageEvent>` ; un test unitaire de chance mal calculé).
- `make lint` ×5 : « aucune erreur » ; `make gen` ×5 (testbed 62 ok, zombies 16, throne 39, gungeon 55, isaac 10) sans diff.
- `cargo fmt --all -- --check` propre ; `check-forbidden.sh` OK (39) ; `check-rollback-registration.sh` OK ;
  `make check_sim_boundary` OK (11 crates, 5 en dette D69) ; `make test_tools` OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines de 3 600 frames sans divergence.
- **p2p deux clients** (Docker OrbStack, `MATCHBOX_PORT=3536`, `COMPOSE_PROJECT_NAME=b0`, binaire `testbed`,
  `ALACOD_INPUT=script:` visée tournante, `ALACOD_INPUT_DELAY=0`, client 1 lancé 8 s plus tard) : le nouvel état rollback
  (`PendingBlasts`, `PatternGhost`) change l'enregistrement, donc rejoué. Carte `arena.ldtk` **modifiée temporairement**
  (non commitée, annulée) : `dummy`, `target`, `follower` remplacés par `elite_explosif`, `elite_divise`, `elite_riche`
  (variante tirée, chance 1). `cmp` des traces : **identiques, 599 lignes** ; exécutions de frame en double (frames
  resimulées) : **79 (client 0) et 83 (client 1)** sur 600 frames (le compteur « frames resimulées » du log final affiche 2
  et 3 : il sous-compte) ; un champion `explosif` est mort à f50, son explosion est partie à f51, aux mêmes frames des deux
  côtés. Un seul champion est mort dans ce run (visée tournante) : `elite_divise` et `elite_riche` n'ont pas été
  exercés en p2p.

## 3. Non fait, incertain, non vérifié

- **`destroys` de `Explode`** (rochers, murs secrets) n'est pas exécuté : M3-T4 (en cours) est le propriétaire de
  l'exécution d'`Explode` ; la fonction partageable est `champion_blast_system` / `PendingBlasts` (voir §72). À
  l'intégration : un seul émetteur de `world::Exploded` par explosion.
- **Teinte** : aucune lecture (T17).
- **Relecture indépendante** : à faire avant merge (demandée par la consigne).
- Bench non mesuré ; aucune vidéo.
- p2p : divise et riche non exercés (voir §2).

## 4. Questions pour William et décisions

- **Délai d'une frame de l'explosion** (comme les barils) : la file de dégâts de la frame de la mort est déjà résolue ;
  alternative : résoudre l'explosion dans la même frame (demande un second passage du résolveur). Choisi : le délai.
- **Division sans variante** (`NO_VARIANT`) : évite la cascade infinie quand la chance est de 1 ; un contenu qui veut des
  petits champions doit un jour le demander explicitement (non prévu).
- **Équipe de la division = équipe du mort** ; santé résolue et difficulté de l'étage courant ; pas de comptage de vague
  (`WaveEnemy`) : une division en mode vagues n'allonge pas la vague. À confirmer pour T11c.
- **Explosion sans équipe** (`Neutral`, tir ami `Always`) : blesse aussi les alliés du champion et les autres ennemis,
  comme un baril.
- **SpawnPattern infini refusé au lint** plutôt que borné au runtime.

## 5. Dettes et points à relire

- **Dette (§25, non corrigée)** : le tirage de variante donne des valeurs **voisines à des net ids consécutifs**
  (`fnv ^ graine ^ net id` dans un générateur congruentiel : quarante ennemis posés d'un coup tirent presque le même
  nombre). Le corriger déplacerait les traces de tout contenu à variantes (gungeon, throne) : à numéroter par
  l'orchestrateur. Les scénarios de chance choisissent la graine 1942 pour que les tirages encadrent les seuils.
- `champion_death_system` est ordonné après `shell_drop_on_death_system` (même ressource d'allocation de net ids) : l'ordre
  d'allocation des net ids d'une mort est « douilles, puis division, motifs, butin ».
- L'étage des apparitions est `run::FloorState::index` (le `Clock::floor_index` ne bouge que si une horloge est
  configurée) ; ajouter `FloorState` à `DifficultyReader` provoquait un conflit d'accès avec
  `spawn_characters_when_map_loaded` : l'étage est donc un paramètre explicite des appelants.
- `PendingBlast` ne garde pas d'équipe source : toujours `Neutral` (voir §4).

## 6. Atteignabilité

Aucune partie ordinaire ne rencontre encore ces champions. **Dette nommée, à fermer par M3-T11c** : champions à au moins
trois ennemis des salles `combat` d'`isaac` (graine à nommer).
