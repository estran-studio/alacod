# Tâches et parallélisation

> Complète `docs/plan-engine.md` (le quoi et le pourquoi) : ici, le comment, dans quel ordre et en
> parallèle. Les identifiants de chantiers (A1, B5, K0…) sont ceux du plan, §5 ; les jalons M0 à M6
> ceux du §6. Statut : proposition du 2026-09-28. Les jalons se détaillent deux à l'avance : M0 et
> M1 en fiches, M2 en tâches, M3 à M6 par voies à la sortie du jalon précédent.

## 1. Les règles du travail en parallèle

Le dépôt est un seul workspace Cargo et la simulation est déterministe : deux agents qui touchent
les mêmes fichiers ou qui changent le gameplay en même temps se marchent dessus au merge (code) et
sur les traces de référence (`.trace`). D'où les règles :

1. **Une voie = une tâche Orca = une branche.** Orca crée un worktree de chaque sous-repo sur la
   branche de la tâche (`scripts/orca-setup.sh`) ; le nom suit `<jalon>-<voie>-<sujet>` (ex.
   `m0-v1-degats`). Une PR par repo modifié, même nom de branche partout (`AGENTS.md`).
2. **Chaque voie possède des fichiers** (§2). On ne modifie pas un fichier d'une autre voie ; si on
   en a besoin, on l'écrit dans la fiche de la tâche et on attend le merge, ou on le fait passer par
   la tâche de contrats de la vague suivante.
3. **Une seule voie change les traces existantes** : la voie simulation (V1). Les autres prouvent
   qu'elles ne changent pas le gameplay : `make test_scenarios` vert **sans** `BLESS`. Un refactor
   qui garde les traces identiques est un refactor prouvé.
4. **Les déplacements de fichiers et les types partagés** ne se font qu'à la frontière des vagues,
   dans une tâche de contrats courte et sérielle (T0.x, T1.0, T2.0…). Jamais pendant une vague.
5. **Ordre de merge fixe par vague** : outillage (V3) → données (V2) → simulation (V1) → présentation
   (V4) → réseau (V5). La simulation merge après les données et l'outillage, pour blesser ses traces
   avec tout le reste en place ; la présentation ne touche pas aux traces.
6. **Rebase sur `main` chaque jour**, CI rapide verte avant merge, `main` toujours vert.
7. **Quatre voies actives au plus** en même temps : au-delà, les merges et la revue coûtent plus que
   le parallélisme rapporte, et le digest hebdomadaire devient illisible.
8. **Chaque tâche commence par ses tests** (plan §9.7) : le scénario ou le test unitaire avant le
   code, le « À regarder » avant la vidéo.

## 2. Les voies

| Voie | Rôle | Possède | Chantiers |
|---|---|---|---|
| **V1 Simulation** | tout ce qui tourne dans `GgrsSchedule` | `crates/game` (parties simulation), puis les crates de vocabulaire : `combat`, `stats`, `effects`, `behaviors`, `world`, `run` ; les traces `.trace` | B1 à B8, C1 à C5 (moteur), D1 à D5, E1 à E9, F1 à F5, G (simulation), H2, J2 |
| **V2 Données** | le contenu, son format, sa validation | `crates/content` (registre, lint, expressions), `crates/bevy_fixed` (RNG), `games/*/assets`, `docs/conventions.md` | A1 à A5, le contenu RON de chaque clone, K4 |
| **V3 Outillage et tests** | ce qui vérifie sans humain | `crates/scenario`, `crates/bots`, `games/testbed`, `tests/scenarios`, `scripts/`, `Makefile`, `.github/` | K0 (avec V1), K1, K2, K3, K5, K6, testbed |
| **V4 Présentation** | ce qui ne tourne pas dans la simulation | `PresentationPlugin` et ses modules (`camera`, `ui`, `light`, `audio`, `character/visuals`, `animation`), `games/*/assets/ui` | I1, I2, I3, H1 (présentation), H3, A5 (crate `animation`) |
| **V5 Réseau et session** | lobbies, session GGRS, p2p | `crates/game/src/jjrs`, `args`, `telemetry`, le repo `allumette` | J1, J3, J4 |
| **V6 Assets et cartes** | ce que le joueur voit et entend, et les cartes LDtk | `games/*/assets/{sprites,tilesets,audio,maps}`, `games/*/assets/assets.yaml` (registre des licences), `docs/assets.md` | recherche d'assets libres, registre des licences, cartes LDtk des clones, gabarits de salles, tilesets ; voir §11 |

Quand une voie a plusieurs agents (V1 surtout), elle se coupe par crate : V1a combat, V1b effets
et objets, V1c ennemis, V1d monde, V1e run et méta. Deux sous-voies ne partagent aucun fichier.

## 3. Le découpage en crates, condition du parallélisme

`crates/game` fait 12 800 lignes et tout le monde y écrit : `core.rs` (assemblage), `weapons/mod.rs`
(1 242 lignes), `character/mod.rs` (enregistrement des systèmes), `global_asset.rs`. Tant que c'est
le cas, une seule voie peut y travailler. Le découpage cible, fait progressivement, **un crate par
vocabulaire** :

| Crate | Contenu | Créé par |
|---|---|---|
| `sim_core` | les contrats : `Team`, `Tag`, `DamageKind`, `DamageEvent`, `StatId`, `Stats`, `Modifier`, `Gauge`, `RollbackSystemSet` complet, le trait de registre des kinds, `FrameEvents` | T0.2 |
| `content` | manifeste, registre, expressions, lint, `alacod lint` | T1.5 |
| `combat` | équipes et dégâts, santé, statuts, projectiles et patterns, mêlée, parade, grille spatiale | T1.1, puis T1.0 de M1 (déplacement de `weapons`) |
| `stats` | stats et modificateurs | T1.2 |
| `effects` | déclencheurs, conditions, actions, objets, inventaire, familiers | M1 |
| `behaviors` | perception, ciblage, behaviors, résolutions, boss | M1 |
| `world` | salles, étages, surfaces, feu, eau, destructible | M1 (E3), M2 (E1, E2) |
| `run` | modes, état de run, horloges, route, profil | M1 (F1), M2 (G1) |
| `bots` | sources d'inputs réactives, `alacod sim` | T2.11 |
| `game` | ce qui reste : assemblage (`core.rs`), présentation, session ; se vide au fil des jalons | — |

Règle : un crate se crée au moment où sa voie commence son premier chantier, et les fichiers qui y
déménagent partent **à la frontière de vague** (règle 4), avec traces identiques comme preuve.

## 4. Le rythme d'un jalon

```
vague 0  contrats (sériel, 1 agent, 3 à 5 jours) : types partagés, déplacements, squelettes
vague 1  parallèle (3 à 4 voies, 1 à 2 semaines) : chaque voie sur ses fichiers
vague 2  parallèle (3 à 4 voies, 1 à 2 semaines) : adoption, contenu, outillage
vague 3  intégration (1 à 2 agents, 1 semaine) : le clone assemblé, scénarios, vidéos, revue humaine
```

À la fin de la vague 3, l'humain joue le clone et laisse ses notes (plan §9.8) ; les notes deviennent
des tâches de la vague 0 du jalon suivant.

Tailles en **jours-agent** (un agent, le flow du §9.7 en place) ; les semaines sont calendaires avec
quatre voies. C'est une estimation, à recaler après M0.

## 5. M0 : `zombies` complet

**But** (plan §6) : le jeu est un crate et un dossier ; points, achats, perks, à terre et
réanimation, power-ups ; 4 joueurs en ligne ; le déterminisme se vérifie tout seul et la CI tourne.

### Vague 0 : contrats (sériel)

#### T0.1 Déterminisme vérifiable (K0) — V3 avec V1, 3 j
- Dépend de : rien. Bloque : tout.
- Fichiers : `crates/game/src/core.rs`, tous les sites `rollback_component_*` et
  `rollback_resource_*`, `state_trace.rs`, `crates/scenario/src/runner.rs`, `jjrs/local.rs`.
- Livrable : `app.rollback_and_trace::<T>()` (rollback + `checksum_component` + trace) et son
  équivalent ressource ; plus aucun appel direct ; un test qui compare types rollback et types
  tracés ; le runner observe `SyncTestMismatch` et échoue à la première frame divergente en
  rejouant en trace complète pour nommer le composant ; option `check_distance` du runner (2 par
  défaut, 8 en stress).
- Acceptation : les douze scénarios passent avec traces **identiques** ; un scénario volontairement
  cassé (un composant non tracé, une mutation hors rollback) échoue avec le bon message ; `make
  test_scenarios` reste sous la durée actuelle plus 20 %.

#### T0.2 Contrats `sim_core` — V1, 3 j
- Dépend de : T0.1. Bloque : toute la vague 1.
- Fichiers : nouveau `crates/sim_core`, `core.rs` (ordre des sets), `character/player/create.rs` et
  `enemy/create.rs` (insertion de `Team`), `system_set.rs`.
- Livrable : `Team` (Players, Enemies, Allies, Neutral), `Tag` (chaîne interne, ensemble ordonné),
  `DamageKind`, `DamageEvent { source, target, kind, amount, frame }` en `FrameEvents`, `StatId`
  ouvert (enum + `Custom(String)`), `Stats`, `Modifier { stat, op, value, source, until }`, `Gauge`,
  `RollbackSystemSet` complet et ordonné (Input, Interaction, Movement, Weapon, Projectiles,
  CollisionDamage, Effects, Status, DeathManagement, EnemySpawning, EnemyAI, Run, FrameCounter),
  `PlayersCount` ressource, trait `KindRegistry` (un plugin déclare ses kinds ; le lint lit la
  liste), `Team` posé sur les joueurs et les ennemis existants.
- Acceptation : compile, traces identiques, `cargo test` du crate (résolution de `Modifier`
  triviale, ordre des sets stable).

#### T0.3 Extraction de `games/zombies/` et squelette de `games/testbed/` (A2) — V2, 2 j
- Dépend de : T0.2. Bloque : V2 et V3 de la vague 1.
- Fichiers : `Cargo.toml` (membres `games/*`), `assets/` → `games/zombies/assets/`,
  `games/zombies/src/main.rs` (l'ancien `map_explorer`), `games/testbed/` (un `main.rs` et une
  salle vide), `crates/scenario` (jeu en paramètre du scénario : `game: "zombies"`), `Makefile`,
  `scripts/scenario-video`, `examples/`.
- Livrable : `cargo run -p zombies` joue la partie actuelle ; les scénarios portent leur jeu ; le
  testbed lance une arène vide avec un joueur.
- Acceptation : traces identiques ; `make videos` fonctionne ; plus rien dans `assets/` à la racine.

#### T0.4 CI rapide (K6a) — V5, 2 j, en parallèle de T0.1 à T0.3
- Dépend de : rien (jeton à réparer par l'humain).
- Fichiers : `.github/workflows/*.yaml`, `Makefile` (`make check`), `Dockerfile.builder`.
- Livrable : à chaque commit et PR, sur ubuntu : format, `cargo test`, `make test_scenarios`, grep
  des interdits (`std::collections::HashMap`, `f32` dans les composants rollback de `crates/game`
  hors présentation) ; moins de cinq minutes avec le cache ; badge dans le README.
- Acceptation : une PR volontairement rouge est bloquée ; `main` vert.

### Vague 1 (parallèle)

**V1 simulation** (sériel dans la voie)

#### T1.1 Équipes et dégâts (B1) — 3 j
- Dépend de : T0.2. Bloque : T1.2, T2.1.
- Fichiers : nouveau `crates/combat` (`team.rs`, `damage.rs`), `character/health/mod.rs`,
  `weapons/mod.rs` (collision des balles), `weapons/melee.rs`, `enemy/ai/behavior.rs` (attaque
  → `DamageEvent`).
- Livrable : toute blessure passe par `DamageEvent` ; `DamageAccumulator` ne s'écrit plus
  directement ; politique de tir ami par arme (`friendly_fire: Never | Always | Cursed`) ;
  résistances et immunités par tag (`immune_to: [Bullet]`) ; `Health.invulnerable_until_frame`
  enfin lu ; la matrice de couches remplacée par `Team` + règles.
- Acceptation : scénarios `friendly_fire_never`, `friendly_fire_cursed`, `immune_tag`, verts en
  synctest à 2 ; traces existantes inchangées sauf bless justifié (aucun attendu) ; bench inchangé.

#### T1.2 Stats branchées (B2) — 4 j
- Dépend de : T1.1, T1.4 (expressions, mergée avant). Bloque : T1.3, T2.3.
- Fichiers : nouveau `crates/stats`, `character/movement.rs`, `character/player/input.rs`
  (vitesse, sprint, dash), `weapons/mod.rs` (cadence, rechargement, dégâts), `health/mod.rs`
  (max, regen), `character/config.rs` (le RON expose `stats:`).
- Livrable : `Stats` par entité, résolution `(base + Σ flat) × (1 + Σ pct) × Π mult` en fixed-point,
  modificateurs sourcés avec fin (frame, statut, salle) ; les constantes de `pathing.rs`
  (séparation, ralentissement) deviennent des stats des ennemis.
- Acceptation : unitaires (ordre, empilement, expiration) ; scénario `stat_move_speed` (position à
  N frames avec `Mult 0.5`) ; traces identiques quand aucun modificateur n'est actif.

#### T1.3 À terre et réanimation (B6, partie « à terre ») — 3 j
- Dépend de : T1.2, T1.7. Bloque : T3.1.
- Fichiers : `crates/combat/src/downed.rs`, `health/mod.rs` (mort → à terre si coop), `interaction.rs`
  (réanimer = interaction maintenue), `character/player/input.rs`.
- Livrable : à 2 joueurs et plus, un joueur à 0 PV tombe à terre (rampe, ne tire pas, minuteur
  `bleedout_frames` dans le RON) ; un coéquipier le réanime en N frames ; seul, il meurt ; tous à
  terre = défaite.
- Acceptation : scénarios `downed_revive`, `downed_bleedout`, `downed_all_lose` à 2 en synctest ;
  attentes `PlayerDowned`, `PlayerRevived` (T1.7).

**V2 données**

#### T1.4 Expressions numériques (A3) — 2 j
- Dépend de : T0.2. Bloque : T1.2, T1.5.
- Fichiers : nouveau `crates/content/src/expr.rs`.
- Livrable : parseur (`+ - * / min max` , comparaisons, identifiants `players`, `wave`, `stat.x`,
  `gauge.x`) et évaluateur fixed-point sur un contexte fourni par l'appelant ; erreurs de parse
  avec position.
- Acceptation : unitaires (précédence, débordements saturés, identifiant inconnu = erreur au
  chargement, jamais à l'exécution).

#### T1.5 Manifeste, registre, `alacod lint` (A1) — 5 j
- Dépend de : T0.3, T1.4. Bloque : T2.6, T2.8, T2.10.
- Fichiers : `crates/content` (`manifest.rs`, `registry.rs`, `lint.rs`, `bin/alacod.rs`),
  `crates/game/src/global_asset.rs` (supprimé au profit du registre), `character/player/create.rs`,
  `enemy/create.rs`, `waves/`, `games/zombies/assets/game.ron`, `games/testbed/assets/game.ron`.
- Livrable : `game.ron` déclare les dossiers ; chargement en registre typé (`CharacterId`,
  `WeaponId`, `EnemyId`, `WaveConfigId`…) ; refus de démarrer sur référence cassée, id dupliqué,
  valeur hors plage, kind inconnu (liste fournie par `KindRegistry`), flottant dans une valeur
  `Fixed` ; `alacod lint games/zombies` en CLI ; rechargement à chaud hors partie ; les joueurs
  reçoivent les armes de leur `characters/*.ron` (`starting_weapons`) et non tout `weapons.ron`.
- Acceptation : fixtures de contenu invalide, une par erreur, avec le message attendu ; les
  scénarios existants gardent leurs traces (les armes de départ sont déclarées à l'identique) ;
  hook PostToolUse qui lance le lint sur `games/**`.

#### T1.6 RNG par flux (A4) — 1 j
- Dépend de : T0.2. Bloque : M1 (patterns, butin).
- Fichiers : `crates/bevy_fixed/src/rng.rs`, `core.rs` (graine de run).
- Livrable : `RollbackRng::stream(name)` dérivé de la graine de run et d'un nom stable ; les
  vagues utilisent `stream("waves")`.
- Acceptation : unitaires (deux flux indépendants, même graine = même suite) ; traces des scénarios
  de vagues **changent** (bless justifié : nouveau tirage), les autres non.

**V3 outillage**

#### T1.7 Attentes et invariants de M0 (K1) — 3 j
- Dépend de : T0.3. Bloque : T1.3, T2.x scénarios.
- Fichiers : `crates/game/src/replay.rs`, `crates/scenario/src/runner.rs`,
  `crates/scenario/src/events.rs`, `crates/scenario/src/invariants.rs` (nouveau).
- Livrable : attentes `Health`, `NoDamageBetween`, `Stat`, `Currency`, `PlayerDowned`,
  `PlayerRevived`, `EntityCount(tag)`, `Event(nom)`, `RunState` ; invariants de frame (santé
  bornée, entités dans la carte, joueur hors mur, net ids uniques) activables par scénario ;
  moments clés étendus (achat, à terre, réanimation, power-up).
- Acceptation : chaque attente a un scénario fixture vert et un rouge ; invariants verts sur les
  douze scénarios.

#### T1.8 Grille spatiale (B4a) — 3 j
- Dépend de : T0.2. Bloque : T2.1.
- Fichiers : `crates/combat/src/grid.rs` (structure seule, pas encore adoptée).
- Livrable : grille de cellules 32 px sur les colliders rollback, requêtes par zone et par cercle,
  reconstruite chaque frame (déterministe : ordre par net id), API `for_each_near`.
- Acceptation : test d'équivalence avec la force brute sur 1 000 configurations aléatoires ;
  scénarios `bench_horde` (200 ennemis) et `bench_bullets` (500 balles, arme fixture) ajoutés au
  testbed sans encore utiliser la grille (ils mesurent l'avant).

#### T1.9 Bench (K3) — 2 j
- Dépend de : T0.3. Bloque : T2.14.
- Fichiers : `crates/scenario` (métriques), `scripts/scenario-review.py` et `.html` (colonne
  perf), `tests/budgets.ron`.
- Livrable : chaque scénario rapporte frames simulées par seconde, taille de snapshot (via
  bevy_ggrs), entités max ; `budgets.ron` fixe des seuils par scénario `bench_*` ; échec si
  dépassé ; historique par commit sur la page de revue.
- Acceptation : `make bench` en local, chiffres visibles sur la page.

**V4 présentation**

#### T1.10 Caméra par joueur en ligne (I3) — 1 j
- Dépend de : rien. Fichiers : `camera/mod.rs`.
- Livrable : en ligne, chaque client suit son joueur local ; en local à plusieurs, le comportement
  actuel ; réglage dans `camera.ron`.
- Acceptation : `play_scenario` à 2 avec `--follow 1` montre le bon joueur ; aucune trace touchée.

#### T1.11 HUD v0 piloté par `ui/hud.ron` (I1 v0) — 4 j
- Dépend de : T0.3. Bloque : T2.12.
- Fichiers : `crates/game/src/ui/hud.rs` (nouveau), `games/zombies/assets/ui/hud.ron`.
- Livrable : un arbre `bevy_ui` construit depuis le RON (ancrages, barres, compteurs, icônes) lié
  aux données de la simulation par nom (`health`, `wave`, `ammo.mag`, `ammo.reserve`, `points`) ;
  rechargement à chaud du RON ; rien dans la simulation.
- Acceptation : capture d'écran de référence à 960×540 et 1920×1080 comparée en CI lente (T2.14) ;
  traces intactes.

**V5 réseau**

#### T1.12 Quatre joueurs (J1) — 3 j
- Dépend de : T0.3, T0.4. Bloque : T3.1.
- Fichiers : `jjrs/`, `args/`, `Makefile` (`test_multiplayer N=4`), `docker-compose.yaml`
  (allumette local), `ui/lobby.rs` (quatre entrées).
- Livrable : scénarios `four_players_idle` et `four_players_shooting` en synctest ; quatre clients
  headless par allumette en local, `diff_log` identique ; télémétrie de desync qui nomme la frame.
- Acceptation : `make test_multiplayer N=4` vert en local ; sera nocturne (T2.14).

**Ordre de merge de la vague 1** : T1.7, T1.8, T1.9 → T1.4, T1.6, T1.5 → T1.1, T1.2, T1.3 → T1.10,
T1.11 → T1.12.

### Vague 2 (parallèle)

**V1 simulation**

#### T2.1 Adoption de la grille (B4b) — 2 j
- Dépend de : T1.8, T1.1. Fichiers : `weapons/mod.rs` (balles), `melee.rs`, `player/input.rs`
  (déplacement), `enemy/ai/pathing.rs` (séparation).
- Livrable : plus aucune boucle sur tous les colliders.
- Acceptation : traces identiques (la grille ne change pas les résultats, seulement le coût) ;
  `bench_bullets` et `bench_horde` au moins trois fois plus rapides ; budgets resserrés.

#### T2.2 Munitions typées et inventaire d'armes (B7) — 3 j
- Dépend de : T1.2. Fichiers : `weapons/mod.rs` (réserves), `crates/combat/src/inventory.rs`
  (nouveau), `interaction.rs` (ramasser, lâcher), `games/zombies/assets/weapons/*.ron`.
- Livrable : réserves par type de munition (`ammo_type: Plomb | …`, cinq types), deux emplacements,
  lâcher et ramasser au sol, échange avec l'arme murale, coup de crosse pendant le rechargement.
- Acceptation : scénarios `ammo_shared_reserve`, `drop_pickup_swap` ; `Ammo` étendu par type
  (T1.7).

#### T2.3 Monnaie et achats (C5 v1) — 4 j
- Dépend de : T1.2, T1.5. Fichiers : `crates/run/src/currency.rs` (nouveau crate `run` minimal),
  `interaction.rs` (achat = interaction avec coût), `waves/` (points par kill), `map_ldtk` (entités
  `WeaponLocation`, `SodaLocation` enfin lues), `games/zombies/assets/economy/*.ron`.
- Livrable : `Currency` par joueur ; points par kill et par réparation ; portes payantes (le
  `cost` existant) ; armes murales ; perks = modificateurs de stats permanents pour la partie
  (`Juggernog` = `max_hp × 2`…) ; prix dans le RON.
- Acceptation : scénarios `buy_door`, `buy_wall_weapon`, `buy_perk` ; attentes `Currency`, `Stat`.

#### T2.4 État de run et mode `Waves` (F1) — 3 j
- Dépend de : T2.3, T1.3. Fichiers : `crates/run/src/{run,modes}.rs`, `waves/` (devient le mode
  `Waves` du run), `core.rs`, `ui/game_over.rs` (résumé lu dans `Run`).
- Livrable : ressource `Run { seed, mode, step, players, flags }` rollback ; conditions de fin
  (tous à terre → défaite) ; résumé (vague atteinte, kills, points) ; relance sans relancer le
  binaire (retour au lobby ou repartir avec la même config) en moins de dix secondes.
- Acceptation : scénario `run_lose_summary` ; `RunState` attendu ; relance chronométrée dans un test.

#### T2.5 Actions de ramassage : power-ups (C1 v0) — 2 j
- Dépend de : T2.3. Fichiers : `crates/effects/src/actions.rs` (nouveau crate, minimal),
  `waves/` (drop à la mort selon table), `games/zombies/assets/items/powerups.ron`.
- Livrable : un pickup au sol applique une liste d'actions : `TimedModifier`, `RefillAmmo`,
  `RepairAllWindows`, `KillAllWaveEnemies`, `Currency(×2 pendant N)` ; c'est la graine de C1.
- Acceptation : un scénario par power-up dans le testbed ; drop déterministe (flux `loot`).

**V2 données**

#### T2.6 Contenu `zombies` en RON — 3 j
- Dépend de : T1.5. Fichiers : `games/zombies/assets/**`.
- Livrable : quatre armes murales avec prix, quatre perks, la table des power-ups, les vagues
  (existant), les personnages avec `starting_weapons`, le `game.ron` complet ; les sprites
  actuels rangés par entité.
- Acceptation : `alacod lint games/zombies` vert ; les scénarios de T3.1 n'ont besoin d'aucun
  contenu de plus.

#### T2.7 Doc des conventions (K4) — 2 j
- Fichiers : `docs/conventions.md`, `CLAUDE.md` (renvoi).
- Livrable : LDtk (couches, entités, champs, tailles), dossier de jeu, `game.ron`, checklist d'un
  nouveau vocabulaire (kind, registre, lint, `rollback_and_trace`, scénario, attente, vidéo).

#### T2.8 Lint des nouveaux kinds — 1 j
- Dépend de : T1.5, T2.3, T2.5. Livrable : le lint connaît `ammo_type`, `friendly_fire`, les
  perks (stat ids), les actions de power-ups ; un test par règle.

**V3 outillage**

#### T2.9 Testbed — 3 j
- Dépend de : T0.3, T1.5. Fichiers : `games/testbed/assets/**`.
- Livrable : salles LDtk minimales (arène vide, couloir, deux salles et une porte, une fenêtre),
  mannequin immobile à santé réglable, cible qui compte les coups, ennemi suiveur, un allié, un
  civil ; `game.ron` ; scénarios de base.
- Acceptation : `bench_*` et les scénarios fixtures de la vague 1 y tournent.

#### T2.10 Générateur de scénarios v0 — 3 j
- Dépend de : T2.9, T1.5. Fichiers : `crates/scenario/src/generate.rs`, `bin/alacod-gen`.
- Livrable : pour chaque arme et chaque perk du jeu, un scénario instancié dans le testbed depuis
  un gabarit (le joueur apparaît avec l'objet, tire sur le mannequin N frames) ; le `test:` d'une
  définition ajoute des attentes ; sans `test:`, invariants seulement ; traces de référence gérées
  comme les autres.
- Acceptation : `alacod gen games/zombies` produit et joue les scénarios ; un objet sans `test:`
  passe, un `test:` faux échoue.

#### T2.11 Bots v0 et `alacod sim` — 4 j
- Dépend de : T1.7. Fichiers : nouveau `crates/bots`, `crates/scenario` (source d'inputs `Bot`),
  `bin/alacod-sim`.
- Livrable : `InputSource::Bot(profil)` déterministe (flux RNG `bots`) : `immobile`, `fonceur`
  (vers l'ennemi le plus proche, tire, répare la fenêtre la plus proche quand libre), `prudent`
  (garde ses distances) ; `alacod sim --game zombies --bots 4 --seeds 1..50 --until-wave 10` sort
  un JSON (vague atteinte, morts, kills, durée, desync) ; un run se sauve en scénario.
- Acceptation : 50 graines à 4 bots sans desync en synctest ; métriques sur la page de revue.

**V4 présentation**

#### T2.12 HUD v1 et écran de résumé — 3 j
- Dépend de : T1.11, T2.4. Livrable : perks en icônes, indicateur « à terre » et minuteur, power-up
  actif, prompts d'achat avec prix en icône ; écran de résumé et relance.
- Acceptation : captures de référence mises à jour ; traces intactes.

#### T2.13 Feedback minimal (I2 v0) — 2 j
- Livrable : flash blanc à l'impact, secousse courte à l'explosion, son d'achat et de power-up,
  dérivés des `FrameEvents` relus en présentation.
- Acceptation : visible sur les vidéos ; traces intactes.

**V5 réseau**

#### T2.14 CI lente (K6b) — 3 j
- Dépend de : T0.4, T1.9, T1.12. Fichiers : `.github/workflows/nightly.yaml`, scripts.
- Livrable : chaque nuit et sur `main` (self-hosted) : bench avec seuils, `alacod sim` à 4 sur 50
  graines, p2p à 2 et 4 par allumette (docker compose), vidéos et captures, page de revue publiée
  ; notes de revue écrites par le serveur dans `tests/review-notes/<commit>.md`.
- Acceptation : une nuit verte de bout en bout ; artefacts téléchargeables.

**Ordre de merge de la vague 2** : T2.9, T2.10, T2.11 → T2.6, T2.7, T2.8 → T2.1 à T2.5 → T2.12,
T2.13 → T2.14.

### Vague 3 : intégration (1 à 2 agents)

#### T3.1 Scénarios du clone — V3 avec V1, 3 j
Partie type à 1, 2 et 4 (vagues 1 à 5, achats, perk, un à terre réanimé, deux power-ups), en
synctest ; « À regarder » par scénario ; bless justifié en une ligne par scénario.

#### T3.2 Vidéos, digest — V4, 1 j
Montage, comparaison avec la dernière vidéo de M0 vague 0, digest écrit avec les métriques de bots.

#### T3.3 Revue humaine — 1 j
Jouer trente minutes à deux ; notes sur la page.

#### T3.4 Fermeture des notes — 2 j
Chaque note devient une attente, un invariant, un scénario ou une tâche de M1.

**Critères de sortie** : plan §9.8. Calendrier indicatif : vague 0 une semaine, vagues 1 et 2 deux
semaines chacune, vague 3 une semaine : **six semaines** avec quatre voies.

## 6. M1 : `throne` (Nuclear Throne)

**But** (plan §6) : cavernes destructibles, projectiles ennemis, deux armes et cinq munitions,
niveaux et mutations, portail, run seedée, horloge de difficulté.

### Vague 0 : contrats (sériel, 4 j)

#### T1.0a Contrats de combat et d'IA — V1
`crates/combat` reçoit `weapons/` (déplacement, traces identiques) ; enums squelettes enregistrés au
`KindRegistry` : `ProjectileModifier`, `Pattern`, `StatusDef`, `Behavior`, `Perception`,
`Targeting`, `Effect { on, if, do }` ; composants d'état `Statuses`, `BehaviorState` ; crates
`effects` et `behaviors` créés vides avec leurs sets.

#### T1.0b Contrats de monde et de run — V1
`crates/world` : `CellKind`, `CellGrid` (ressource rollback, source du flow field), `Destructible`
; `crates/run` : `Mode::Floors`, `FloorIndex`, `Clock`. `RollbackSystemSet::World` inséré.

#### T1.0c Contenu `games/throne/` squelette — V2
`game.ron`, un personnage, une arme, un ennemi, une caverne fixe : le clone démarre vide.

### Vague 1 (parallèle)

**V1a combat**
- T1.1 Projectiles composables (B5 v1) — 4 j : `Bounce(n)`, `Pierce(n)`, `Size`, `Lifetime`,
  `Homing(force)`, `Gravity`, `on_hit: [Action]`, `on_expire: [Spawn(pattern)]` ; explosions comme
  projectile à durée nulle. Acceptation : un scénario générable par modificateur dans le testbed ;
  `BulletCount`, `HitsAtLeast` ; `bench_bullets` à 500 dans le budget.
- T1.2 Patterns et tir ennemi (B5 v1) — 3 j : `Aimed`, `Spread`, `Ring`, `Sequence`, `Telegraph`
  ; attaque `Shoot(pattern)` pour les ennemis ; graine par émetteur (flux `patterns`).
  Acceptation : même graine = même trace à 1 et à 4 joueurs ; scénario `enemy_ring`.
- T1.3 Statuts (B3) — 3 j : `Statuses` avec `Burn`, `Slow`, `Stun`, `Freeze`, empilement, tick,
  modificateurs, visuel dérivé. Acceptation : `HasStatus`, `StatusStacks`, un scénario par statut.

**V1c ennemis**
- T1.4 Behaviors composables v1 (D1) — 5 j : `Chase`, `KeepDistance`, `Strafe`, `Charge(telegraph)`,
  `Shoot`, `Melee`, `Flee`, `Wander` ; sélection par priorité ; `Perception { sight }` ;
  `Targeting::Nearest` ; les zombies de `games/zombies` réécrits en RON avec traces identiques
  (preuve). Acceptation : `EnemyState`, `EnemyDistance` ; diagnostics de navigation en attentes.
- T1.5 Variantes (D2) — 2 j : `variants: { nom: (modifiers, tags, skin) }`, tirage par flux
  `variants`. Acceptation : scénario `variant_fast`.

**V1d monde**
- T1.6 Terrain destructible et cavernes (E3) — 5 j : générateur de cavernes (automate cellulaire,
  graine), `CellGrid` → colliders et flow field incrémental, cellule détruite par explosion.
  Acceptation : unitaires du générateur (connexité, 1 000 graines) ; scénario `explode_wall` ;
  `CellState` attendu ; `bench_cave` (recalcul du flow field après 50 destructions).
- T1.7 Surfaces v1 (E4) — 2 j : tags de cellules et modificateurs de vitesse (eau peu profonde,
  sable). Acceptation : `PlayerPosition` sur deux surfaces.

**V1e run**
- T1.8 Mode `Floors` (F1) — 4 j : séquence de niveaux, portail quand `EntityCount(enemy) == 0`,
  chargement du niveau suivant avec continuité des net ids (numérotation triée par contenu), défaite
  et résumé, boucle infinie après le dernier niveau. Acceptation : `FloorIndex` ; scénario
  `portal_next_floor` ; bots qui finissent trois niveaux.
- T1.9 Horloges (F2) — 2 j : `Clock` d'étage et de run, événements planifiés, difficulté qui
  monte avec l'index et le temps (`players` dans les expressions). Acceptation : `Clock(id, fired)`.
- T1.10 Effets v1 et mutations (C1 v1, C4 v1) — 4 j : `Effect { on, if, do }` avec `OnLevelUp`,
  `OnKill`, `OnDamageTaken`, `Tick` ; actions `Modifier`, `Heal`, `SpawnProjectile` ; rads → niveau
  → choix parmi trois mutations tirées d'un pool (flux `loot`) ; pool d'armes par niveau.
  Acceptation : scénario par déclencheur ; `Event(levelup)` ; le choix passe par un input dédié
  (scriptable).

**V2 données**
- T1.11 Contenu `throne` — 5 j : douze armes sur cinq munitions, dix ennemis (dont trois tireurs et
  un chargeur), huit mutations, trois niveaux de caverne, tables de butin, `test:` sur chaque
  définition. Acceptation : lint vert, scénarios générés verts.
- T1.12 Lint des nouveaux kinds — 2 j.

**V3 outillage**
- T1.13 Générateur v1 — 3 j : gabarits par ennemi (seul contre joueur immobile, puis mobile) et par
  statut ; `test:` sur les ennemis. 
- T1.14 Bots v1 — 3 j : profil `prudent` qui esquive les projectiles, complétion de niveau ;
  `alacod sim` avec `--until-floor` ; métriques de niveau.
- T1.15 Attentes de M1 — 2 j : `BulletCount`, `HitsAtLeast`, `HasStatus`, `StatusStacks`,
  `EnemyState`, `EnemyDistance`, `FloorIndex`, `Clock`, `CellState`.

**V4 présentation**
- T1.16 Écran de mutation et transition de niveau — 3 j : choix à trois cartes à la manette ;
  fondu et recentrage au portail.
- T1.17 Feedback v1 (I2) — 3 j : hit stop, secousse paramétrée, flash, télégraphe de charge
  (cercle au sol depuis `Telegraph`), chiffres de dégâts optionnels.
- T1.18 HUD throne — 2 j : barre de rads, munitions par type, niveau.

**Ordre de merge** : V3 → V2 → V1 (V1a, V1c, V1d, V1e dans cet ordre : chacune blesse ses traces,
V1e en dernier) → V4.

### Vague 2 : intégration (1 semaine)
Scénarios du clone (trois niveaux à 1, 2 et 4 ; un boss simple par timeline), bots sur 200 graines,
vidéos, revue humaine, fermeture des notes. Calendrier indicatif : **cinq semaines**.

## 7. M2 : `gungeon`, par voies (à détailler à la sortie de M1)

- **Vague 0** : contrats de salles (`RoomKind`, `RoomState`, `Entrance`), d'objets (`ItemKind`,
  `Inventory` à emplacements), de boss (`Phase`, `Timeline`), de profil (`Profile`) ; `crates/meta`.
- **V1a combat** : B5 v2 (blanks, rebonds sur murs, patterns en spirale et en éventail), B6 (roulade
  à i-frames, charges, variantes en données).
- **V1b effets et objets** : C1 v2 (tous les déclencheurs), C2 (genres, emplacements, actifs à
  cooldown par salles, consommables, synergies), C4 (coffres, clés, pools, rareté), C5 (boutique).
- **V1c ennemis** : D4 (phases, timelines, arènes à tenir), D1 v2 (formations simples).
- **V1d monde** : E1 (salles typées, verrouillage sur les présents, activation), E2 (grammaire
  d'étage sur gabarits LDtk, minicarte, transition), E4 v2 (tables, barils, fosses).
- **V1e run et méta** : G1 (profil et sauvegarde), G2 (hub : la Brèche), F1 (`Floors` à salles).
- **V2** : contenu `gungeon` (vingt armes, vingt objets, dix ennemis, un boss, quinze gabarits de
  salles), lint.
- **V3** : K2 v1 (bot explorateur qui finit un étage), générateur v2 (objets et salles), bench à
  500 balles en salle verrouillée, attentes `RoomState`, `Inventory`, `BossPhase`, `ProfileHas`.
- **V4** : I1 v1 (le test comparatif d'UI de deux jours, puis HUD, pause, inventaire, minicarte),
  I2 v2.
- Calendrier indicatif : **huit semaines** (XL).

## 8. M3 à M6

Par voies, à la sortie de M2 et de chaque jalon suivant, avec les chantiers du plan §6 : M3 (la
tranche verticale de 1837 : A5 en V4, B5 parade et D3 en V1a et V1c, E4 neige et E8 nuit en V1d, F2
nuit en V1e, H1 en V4, J2 en V1e, le plugin `1837` dans son dépôt) ; M4 Isaac ; M5 Hades ; M6 la
région des chantiers puis la campagne.

## 9. Suivi

- Ce fichier est la source de vérité des tâches : une ligne de statut par tâche (`à faire`, `en
  cours (branche)`, `mergée (commit)`), tenue par l'agent qui prend la tâche.
- Une tâche commence par un commentaire dans sa fiche : qui, quelle branche, quand ; elle finit par
  le commit de merge et la vidéo.
- Le digest hebdomadaire (plan §9.8) liste les tâches mergées, en cours, bloquées, et les métriques.
- Une tâche qui découvre une autre tâche l'ajoute ici, dans la vague suivante, jamais dans la sienne.

## 10. Journal

| Date | Tâche | Branche | Agent | État |
|---|---|---|---|---|
| 2026-09-28 | T0.1a (K0, première étape : extension `rollback_and_trace`, observateur `SyncTestMismatch`, `check_distance`, test du filet) | `m0-v3-determinisme` | Haiku | livrée avec trois défauts (API `Trigger`, `check_distance` non branché, test inopérant), reprise par l'orchestrateur, voir la ligne suivante |
| 2026-09-28 | T0.4 (K6a, CI rapide sans conteneur, `make check`, `scripts/check-forbidden.sh`) | `m0-v5-ci-rapide` | Haiku | mergée (`3218f5d`) ; six occurrences interdites en avertissement (pathing.rs, state.rs, map_ldtk plugin.rs, un commentaire dans bevy_fixed) |
| 2026-09-28 | T1.4 (A3, crate `content`, module `expr`) | `m0-v2-expressions` | Haiku | mergée ; 55 tests unitaires ; corrigé par l'orchestrateur : majeures alignées (`thiserror 2`, `ron 0.12`), `Cargo.lock` sans montée collatérale |
| 2026-09-28 | T1.10 (I3, caméra par joueur en ligne, `play_scenario --follow`) | `m0-v4-camera` | Haiku | mergée ; `online_follow` dans `camera.ron`, `PlayConfig` dans le runner (signature de `build_app` changée), captures vérifiées par l'agent, traces intactes |
| 2026-09-28 | T0.1a, reprise par l'orchestrateur : conflit `runner.rs` avec la caméra, `On<>` au lieu de `Trigger<>`, `check_distance` branché de bout en bout, test du filet réécrit (ressource hors rollback lue dans `GgrsSchedule`) | `m0-v3-determinisme` | Fable | mergée ; douze scénarios verts, traces intactes, deux tests du filet verts (`crates/scenario/tests/determinism.rs`), `ALACOD_CHECK_DISTANCE` (2 par défaut) |
| 2026-09-28 | T1.8a (B4, grille spatiale : structure et tests d'équivalence, nouveau crate `combat`) | `m0-v3-grille` | Haiku | mergée ; 15 tests dont quatre d'équivalence avec la force brute ; corrigé par l'orchestrateur : détour par `f32` (règle 7), troncature au lieu du plancher pour les cellules négatives, entités de test invalides (tests ignorés), features de `bevy` |
| 2026-09-28 | T2.7 (K4, `docs/conventions.md` : LDtk, sprites, dossier de jeu, checklist d'un vocabulaire) | `m0-v2-conventions` | Haiku | mergée ; une reprise (quinze erreurs factuelles corrigées par l'agent sur liste, dernières retouches par l'orchestrateur) |
| 2026-09-28 | T0.1b et T0.1c (K0 : migration des 47 enregistrements rollback vers l'extension, checksums partout, script strict, trace d'état générique = checksum GGRS) | `m0-v3-rollback-migration` | Sonnet (tâche large et piégeuse : `Entity` et `f32` à exclure du hachage, `HashMap` à convertir ; T0.1a en Haiku a demandé une reprise complète) | en cours |
| 2026-09-28 | T1.9 (K3, métriques de performance par scénario, `tests/budgets.ron`, `make bench`, section Performance de la page de revue) | `m0-v3-bench` | Haiku | mergée ; mesures de référence 95 à 218 fps (headless), budgets = moitié, bloquants avec `make bench` seulement ; corrigé par l'orchestrateur : chrono incluant le chargement, budgets bloquants en test ordinaire (aléatoire sous charge), nom de dossier différent de celui des vidéos, `ron 0.8`, scripts ignorant `CARGO_TARGET_DIR` |
| 2026-09-28 | T1.7 (K1 v1 : attentes `Health`, `EntityHealth`, `NoDamageBetween`, `EntityCount`, `Event` ; invariants de frame santé bornée, net ids uniques, joueur hors mur) | `m0-v3-attentes` | Haiku | mergée ; 16 tests ; coût des invariants négligeable (118 à 242 fps machine au repos) ; corrigé par l'orchestrateur : champ `invariants` annoncé mais absent du format, requêtes reconstruites et murs copiés à chaque frame, `NoDamageBetween` en `f32` avec état partagé, tests manquants |
| 2026-09-28 | T1.11 (I1 v0 : HUD `bevy_ui` décrit par `assets/ui/hud.ron`, sources santé, vague, munitions, arme, ennemis ; rechargement à chaud) | `m0-v4-hud` | Haiku | mergée ; HUD visible dans les captures (vague, ennemis, joueurs, barre et texte de santé, arme, munitions) ; corrigé par l'orchestrateur : préfixe relu depuis le texte affiché (« Vague 11… »), extension `.ron`, requête d'arme sur le joueur (munitions toujours « ? »), rechargement à chaud abandonné ; trouvé au passage : l'UI n'apparaissait dans **aucune** capture (caméra retargetée vers une image, plus de caméra d'UI par défaut), corrigé dans le runner. Reste : les overlays de debug (caméra, vagues, FPS) chevauchent le HUD dans les captures |
| 2026-09-28 | T1.12 (J1 : scénarios à quatre joueurs en synctest, `make test_multiplayer N=4`, lobby à quatre) | `m0-v5-quatre-joueurs` | Haiku | mergée ; deux scénarios à quatre (179 et 209 fps au calme, l'agent annonçait 25 sous charge), `make test_multiplayer N=4` ; corrigé par l'orchestrateur : un lobby différent par client (test p2p inopérant), tableaux bash sans `SHELL`, liste `--players` à deux entrées, doc avec des fps et une attente inexistante. Non exécuté : le test p2p réel demande un affichage et allumette |
| 2026-09-29 | T0.1b + T0.1c (K0 : tout l'état rollback passe par `RollbackTraceApp`, enregistrement direct interdit (`scripts/check-rollback-registration.sh` strict dans `make check`), trace d'état générique dont le hash est le `Checksum` GGRS) | `m0-v3-rollback-migration` | Sonnet, puis Fable | mergée (`e30de30`) ; livrée par Sonnet (`c149ad9`, `5c249be`, `753a6d5`), mais le filet K0 a révélé un bug de déterminisme **préexistant** dès que `Velocity` et `FixedTransform3D` sont entrés dans le checksum : `shoot_around`, `remote_first_fight` et `recording_replays_identically` échouaient en synctest. Cause trouvée par l'orchestrateur (diagnostic `ALACOD_DIAG=1`, `e2d54e8`) : une entité tuée était détruite par `despawn()` ; au rechargement d'une frame antérieure, bevy_ggrs la respawnait avec ses seuls composants rollback, sans `CharacterConfigHandles` elle devenait invisible à `move_enemies` et la séparation de ses voisins divergeait (frame 637 : quatre zombies, tout le reste identique). Écarté au passage : l'ordonnancement (`GgrsSchedule` refuse déjà toute ambiguïté, résultat identique en mono-thread). Correctif (`1a09285`) : `despawn_rollback()` de bevy_ggrs pour la mort, les balles et les hitbox de mêlée (entité désactivée, détruite à la confirmation, ressuscitée intacte) ; règle 9 dans `CLAUDE.md`. Traces : identiques jusqu'à la première mort ; `shoot_around` et `remote_first_fight` avaient été blessées par Sonnet sur des runs interrompus (639 et 427 lignes), re-blessées complètes. Test du replay corrigé (même nombre de frames que l'original). Dettes notées par Sonnet : `ObstacleAttackEvent` sans `GgrsNetId`, `PointerWorldPosition` code mort, coût du checksum de `FlowFieldCache`, trace une frame plus courte que le scénario (état final sauvegardé seulement à l'avance suivante). Leçon : un agent ne blesse jamais une trace d'un run qui n'a pas atteint sa dernière frame. Après le merge : `four_players_idle` et `four_players_shooting` (T1.12, blessées dans l'ancien format) re-blessées dans le format `Checksum` ; sur `main`, quatorze scénarios verts, filet K0, seize attentes et unitaires verts |
| 2026-09-29 | Frontière de vague : formatage global du workspace, `make format` strict, `make fmt` | `main` (sériel) | Fable | faite ; 92 fichiers reformatés sans changement de code, quatorze scénarios et filet K0 verts après coup |
| 2026-09-29 | T0.2 (contrats `sim_core` : `Team` statique, `Tag`, `DamageKind`, `DamageEvent`, `FrameEvents` et `RollbackSystemSet` déménagés et complétés, `StatId`, `Stats`, `Modifier` + résolution, `Gauge`, `PlayersCount`, `KindRegistry` ; rien n'entre dans le checksum) | `m0-v1-sim-core` | Sonnet | mergée ; livrée sans compilation (build tué par manque de mémoire, machine partagée), vérifiée et corrigée par l'orchestrateur : deux tests faux sur 28 (seuil de jauge attendu hors intervalle ; `Tags` non transparent en RON, corrigé par `#[serde(transparent)]`), `HashSet` dans les tests, `check-forbidden` étendu à `sim_core` et `combat` ; quatorze traces identiques, filet K0, 29 unitaires |
| 2026-09-29 | T0.3 (A2 : `games/zombies/` avec l'ancien `map_explorer` et les assets, `games/testbed/`, `game:` dans les scénarios, Makefile et scripts) | `m0-v2-games-zombies` | Haiku | mergée ; livrée en dix minutes (`3de65f4`), retouchée par l'orchestrateur : sections `[profile]`/`[patch]` retirées des manifestes des jeux, chemins dans docs et commentaires, TODO T1.5 (jeu en dur dans l'enregistreur) ; **le testbed n'avait jamais été lancé** : carte LDtk écrite à la main refusée par le chargeur, assets manquants (vagues, sprites, atlas) ; remplacée par une salle dérivée de `test_map.ldtk` (joueurs et portes, zéro spawn d'ennemi), vérifiée 119 frames en headless ; `scripts/scenario-video` honore `CARGO_TARGET_DIR` ; `make zombies` / `make testbed`. Vérifié sur l'état fusionné avec T0.2 : builds, format, quatorze traces identiques, filet K0, seize attentes, `zombies` joue une partie complète en headless, `make videos` produit `idle.mp4` et le montage. Leçon : un agent qui livre un binaire le lance |
| 2026-09-29 | T1.1 (B1 : toute blessure par `FrameEvents<DamageEvent>`, règles `combat::damage` (tir ami par arme, tags `cursed`, immunités et résistances par tag, invulnérabilité honorée), scénarios `friendly_fire_never`, `friendly_fire_cursed`, `immune_tag` ; preuve par trace détaillée avant bless) | `m0-v1-equipes-degats` | Sonnet | en cours |
| 2026-09-29 | T1.6 (A4 : `RunSeed` dérivée de la graine de carte, `RngStreams` par nom (`fnv1a(nom) ^ graine`), vagues sur `"waves"`, dispersion des tirs sur `"weapons"`) | `m0-v2-rng-flux` | Haiku | mergée ; l'agent a rendu les mains sur une « divergence synctest » qui était son propre bug (`RngStreams` enregistré comme composant, jamais restauré) ; corrigé par l'orchestrateur, six attentes recalées sur le nouveau tirage (dont une balle qui sort par une fenêtre : les fenêtres laissent passer les balles par conception), `event_kill_found` déplacé sur `remote_first_fight`, quatorze traces re-blessées (nouveau tirage) ; filet K0, seize attentes, quatorze unitaires RNG. Mergée avant T1.1, qui re-blessera après fusion de `main` |
| 2026-09-29 | T1.5 (A1 : manifeste `game.ron`, registre typé (`CharacterId`, `WeaponId`, `MeleeWeaponId`, `EnemyId`, `WaveConfigId`, `MapId`), lint (référence cassée, id dupliqué, hors plage, kind inconnu, flottant dans un `Fixed`), CLI `alacod lint`, `make lint`, `starting_weapons` par personnage, rechargement à chaud dans le lobby, hook Claude Code `PostToolUse` sur `games/**`) | `m0-v2-manifeste-lint` | Sonnet | mergée ; rapport complet et honnête (62 + 6 tests, lint des deux jeux à zéro erreur, quatorze traces identiques, testbed 119 frames) ; l'agent avait signalé `examples/character_tester.rs` cassé à l'exécution (pas de `Registry`) : branché par l'orchestrateur sur `games/zombies`. Le testbed ne porte plus les sprites de zombie ni les vagues. Dettes : kinds en dur de `register_kinds` pas branchés sur le registre ; sprites encore nommés en dur dans `global_asset.rs` (A5) ; rechargement à chaud et hook non observés en conditions réelles |
| 2026-09-29 | T2.11 (bots v0 : `crates/bots`, profils `immobile`/`fonceur`/`prudent`, `InputSource::Bot`, `bot:` dans les scénarios, CLI `alacod-sim` (JSON par graine, desync, sauvegarde en scénario), `make sim`, page de revue) | `m0-v3-bots` | Sonnet | en cours (lancée avant T2.9 : ses dépendances sont satisfaites et elle ne touche pas aux fichiers de T1.1 hors `replay.rs`) |

Orchestration : Fable crée les worktrees (`scripts/task-new.sh` du meta-repo), lance un agent par
tâche avec le modèle le moins cher (Haiku d'abord, Sonnet si une tâche échoue deux fois), vérifie
la branche (`make test_scenarios`, `cargo test`, lecture du diff), fusionne dans `main`
(`scripts/task-merge.sh`, merge `--no-ff`) dans l'ordre de merge de la vague, et tient ce journal.
Base de référence : `main` à `4e93269`, douze scénarios verts en 78 s.

Leçons de la vague 0 : (1) un `target` partagé entre worktrees fait la queue sur le verrou cargo et,
pire, sert des artefacts périmés (les empreintes des crates du workspace sont relatives à la racine et
datées) : un `target` par tâche, amorcé depuis `main` puis `touch` des sources (`task-new.sh`) ;
(2) les agents rapportent parfois « vérifié » sans avoir pu compiler : l'orchestrateur relance toujours
les vérifications lui-même ; (3) un agent arrêté laisse ses `cargo` en arrière-plan : les tuer ; (4) le `Makefile` forçait
`CARGO_TARGET_DIR := ./target` : chaque `make test_scenarios` d'un worktree recompilait tout à froid
dans un `target` local (corrigé en `?=` sur la branche T0.1a) ; (5) des agents ont compilé dans `/tmp`
(tmpfs de 7,7 Go, saturé à 80 %) : le nettoyer et ne jamais y compiler.

Frontière de la vague 1, faite le 2026-09-29 : commit de formatage global (`cargo fmt --all`, 92 fichiers, traces identiques), puis `make format` devient `cargo fmt --all -- --check` (la CI échoue sur le style) et `make fmt` formate. Toute branche ouverte avant ce commit doit passer `make fmt` avant son merge.

## 11. Voie V6 : assets et cartes (session à part)

Une voie sans code, qui peut avancer dans une autre session dès maintenant : elle ne touche que des
fichiers de contenu (`games/<jeu>/assets/**`) et n'entre jamais en conflit avec les autres voies.
Elle applique les règles du §1 : une branche par tâche (`m0-v6-carte-zombies`), aucune modification
de fichiers existants hors de sa propriété, aucun `.trace`.

**Règles propres à la voie**
- **Tout asset a une licence enregistrée** avant d'entrer dans le dépôt : `games/<jeu>/assets/assets.yaml`
  (id, fichier, source avec l'URL réellement consultée, auteur, licence, modifications faites). CC0
  d'abord ; CC-BY accepté avec le crédit dans le fichier ; rien d'ambigu ; rien qui vienne des jeux
  de référence eux-mêmes. Le pack `ZombieShooter` actuel doit être identifié et enregistré lui aussi.
- **Les formats de l'engine** : planches de sprites en grille régulière décrites par un
  `SpriteSheetConfig` (`crates/animation/src/lib.rs` : `path`, `tile_size`, `columns`, `rows`,
  `anchor`, `offset_*`, `animated`) et un `AnimationMapConfig` (`frame_duration`, `animations` par
  lignes ou par indices) ; l'engine rend gauche et droite par retournement (8 directions suivies,
  2 dessinées : voir A5). Audio en `.ogg`. Cartes en LDtk 1.5, lues par le fork `bevy_ecs_ldtk`.
- **Les conventions LDtk actuelles** sont celles de `assets/exemples/test_map.ldtk` : couches
  `Walls` (IntGrid), `LevelConnection` (IntGrid : les ouvertures entre niveaux), `Entities` ;
  entités `DoorHorizontal`, `DoorVertical` (champ `cost`, porte appariée), `WindowHorizontal`,
  `WindowVertical`, `PlayerSpawn` (champ `index` 0..3), `ZombieSpawn`, `CrateLocation`,
  `WeaponLocation`, `SodaLocation` ; champ de niveau `spawn` (le niveau de départ). Un niveau LDtk =
  une salle ; le générateur (`crates/map/src/generation`) assemble les salles par leurs connexions
  (côtés N, S, E, W, position et taille). `make map_preview` affiche une carte, `make
  map_generation` la génère. Ces conventions seront écrites dans `docs/conventions.md` (T2.7) ; en
  attendant, la voie les documente elle-même dans `docs/assets.md` à mesure qu'elle les découvre.
- **Nouveaux fichiers seulement, au futur emplacement** `games/<jeu>/assets/...` (T0.3 déplace
  l'existant vers `games/zombies/assets/`) : pas de modification des fichiers de `assets/`.
- **Poids** : ne garder que les planches utilisées, pas les packs entiers ; pas de fichier de plus
  de quelques mégaoctets sans en parler (git LFS est une décision à prendre).
- **Vérification sans humain** : chaque carte a une capture (`make map_preview` ou la vidéo d'un
  scénario `idle` posé dessus par V3) et une ligne « À regarder » ; le registre est validé par un
  petit lint (`scripts/check-assets.py` : chaque fichier binaire du dossier a une entrée, chaque
  entrée a une licence permise).

**Tâches**

| Tâche | Quoi | Sert à | Taille |
|---|---|---|---|
| V6.1 Registre et lint des assets | `assets.yaml` par jeu, `scripts/check-assets.py`, `docs/assets.md` (formats, conventions, sources retenues) ; identifier et enregistrer le pack `ZombieShooter` | tout | S |
| V6.2 Recherche d'assets par clone | une liste courte de packs libres par clone (tilesets, personnages, ennemis, projectiles, objets, icônes, sons), avec licence vérifiée, et le téléchargement des seuls fichiers utiles ; pistes à vérifier : Kenney (CC0 : Top-down Shooter, Tiny Dungeon, Roguelike packs, sons), 0x72 « 16x16 Dungeon Tileset II » (CC0), OpenGameArt en filtrant CC0 | M1, M2, M4, M5 | M |
| V6.3 Import de planches | `scripts/assets/import-sheet.py` : à partir d'une planche et de ses paramètres, écrit les `.ron` de `SpriteSheetConfig` et `AnimationMapConfig` ; exemple sur un ennemi de `throne` | V2 (contenu) | S |
| V6.4 Carte `zombies` v1 | une vraie carte à la CoD Zombies pour M0 : six à huit salles, portes payantes (dont appariées), fenêtres sur les murs extérieurs, spawners dehors, quatre points de départ, emplacements d'armes murales et de perks (`WeaponLocation`, `SodaLocation`), avec le tileset actuel ; capture et « À regarder » | M0 (T3.1 la jouera) | M |
| V6.5 Salles du testbed (LDtk) | arène vide, couloir, deux salles et une porte, une fenêtre, de l'eau, du bois : les `.ldtk` du plan §9.5 ; V3 (T2.9) y branche les entités et le RON | tests de vocabulaire | S |
| V6.6 Tileset de cavernes `throne` | murs, sols, bords, débris, compatible avec un rendu à partir de cellules (E3 dessinera depuis `CellGrid` : convenir avec V1d du format des règles de tuiles) | M1 | M |
| V6.7 Gabarits de salles `gungeon` | quinze salles typées (combat petite, moyenne, grande, boutique, coffre, boss, secrète, entrée, sortie, couloirs) avec entrées N, S, E, W ; commence quand les métadonnées de salle (E1, M2 vague 0) sont fixées ; la recherche du tileset et des props se fait avant | M2 | L |
| V6.8 Audio | sons CC0 par clone (tir, impact, mort, achat, porte, power-up), deux boucles de musique ; réglages de `audio/adaptive.rs` | M0, M1 | S |

Ordre proposé : V6.1, puis V6.4 et V6.5 (utiles à M0), puis V6.2 et V6.3 (M1), V6.6, V6.8, et V6.7
quand M2 commence. Les sprites de 1837 ne passent pas par cette voie : ils viennent du pipeline
`assetgen` de `1867_lore` (A5).
