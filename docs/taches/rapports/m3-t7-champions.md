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
  décalages fixes, équipe du mort, **sans variante** : pas de cascade), `Explode` (**mécanisme de M3-T4** : bombe à mèche d'une frame de l'équipe du mort, `destroys` transmis ; voir §6 bis), `SpawnPattern` (émetteur fantôme
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

Après intégration de M3-T4 (`origin/m3-t4-bombes-rochers` `566f6dba`, qui contient M3-T6 ; `origin/main` n'avait pas bougé) :

- `make test_scenarios_par SHARDS=3` sans BLESS des traces existantes : **426 scénarios** (dont les 15 de T4 et les 11 de T7),
  3 tranches « ok » (138 s) ; aucune trace existante ni de T4 modifiée.
- `make test_crates` : 946 tests passés, 0 échec, 13 ignorés.
- `make lint` ×5 « aucune erreur » ; `make gen` ×5 (testbed 62, zombies 16, throne 39, gungeon 55, isaac 10) sans diff ;
  `cargo fmt --all -- --check`, `check-forbidden` (39), `check-rollback-registration`, `make check_sim_boundary` (11 crates),
  `make test_tools` OK.
- Fuzz `ALACOD_FUZZ=0:8` : 8 graines sans divergence (le fuzz de T4 affiche « 0 explosion(s) » sans `ALACOD_FUZZ_BOMBS`).
- **p2p deux clients** (Docker, `MATCHBOX_PORT=3536`, `COMPOSE_PROJECT_NAME=b0`, même recette, `arena.ldtk` modifiée
  temporairement puis annulée) : `cmp` **identiques, 599 lignes** ; 73 (client 0) et 65 (client 1) exécutions de frame en
  double ; le champion `explosif` meurt à f50, l'explosion de T4 part à f51 (`explosion id=45`), aux mêmes frames des deux côtés.

## 2 bis. Intégration avec M3-T4

Décision d'orchestration : le mécanisme d'explosion de T4 fait foi. Conflits : `CLAUDE.md` et `docs/taches.md` (deux lignes
conservées), `hit_iframes.rs` et `expectations.rs` (ancre d'ordre `.after(game::bombs::explosion_system)`). `champions.rs` n'a pas
conflit mais a été réécrit : `PendingBlast(s)` et `champion_blast_system` supprimés ; `champion_death_system` pose une `world::Bomb`
(`fuse_until = frame + 1`, équipe du mort, tir ami `Always`, source créditée au dernier joueur, **`destroys` transmis**, que `death_plan`
jetait), exécutée par `bomb_fuse_system` / `explosion_system` : une seule émission de `DamageEvent` et d'`Exploded` par explosion,
exclusions `Death` et invulnérables de T4. Observable de `champion_explodes` inchangé (mort f29 → dégâts f30 ; attentes
passées sans modification). **Trace bougée : `champion_explodes.trace` seule** (changement de mécanisme : une entité `Bomb`
(+1 entité, un `GgrsNetId` alloué) existe pendant la frame de mort, ligne 29 : 13 → 14 entités) ; rebénie seule, les autres
traces de T7 sont inchangées. `PendingBlasts` n'existe plus : un état rollback neutre de moins.
Lint : le pattern infini d'un `SpawnPattern` à la mort suit désormais la chaîne `Named` (profondeur bornée, cycle ou nom
inconnu refusés) ; fixture `champion_death_infinite_pattern_named`.

## 3. Non fait, incertain, non vérifié

- **`destroys` de `Explode`** : transmis à la bombe de T4 (rochers, caverne, murs secrets) ; aucun scénario de T7 ne le
  prouve (testé par T4 pour ses propres bombes).
- **Teinte** : aucune lecture (T17).
- **Relecture indépendante** : faite (verdict bloquant : intégration T4, lint `Named`), corrigée ici.
- Bench non mesuré ; aucune vidéo.
- p2p : divise et riche non exercés (voir §2).

## 4. Questions pour William et décisions

- **Délai d'une frame de l'explosion** (comme les barils) : la file de dégâts de la frame de la mort est déjà résolue ;
  alternative : résoudre l'explosion dans la même frame (demande un second passage du résolveur). Choisi : le délai.
- **Division sans variante** (`NO_VARIANT`) : évite la cascade infinie quand la chance est de 1 ; un contenu qui veut des
  petits champions doit un jour le demander explicitement (non prévu).
- **Équipe de la division = équipe du mort** ; santé résolue et difficulté de l'étage courant ; pas de comptage de vague
  (`WaveEnemy`) : une division en mode vagues n'allonge pas la vague. À confirmer pour T11c.
- **Explosion de l'équipe du mort** (`Enemies`, tir ami `Always`) : blesse aussi les alliés du champion et les autres ennemis ;
  la difficulté d'étage multiplie ses dégâts.
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

## 6. Atteignabilité

Aucune partie ordinaire ne rencontre encore ces champions. **Dette nommée, à fermer par M3-T11c** : champions à au moins
trois ennemis des salles `combat` d'`isaac` (graine à nommer).
