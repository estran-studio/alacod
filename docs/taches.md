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
| `combat` | armes, équipes et dégâts, santé, statuts, projectiles et patterns, mêlée, parade, grille spatiale | T1.1, puis T1.0 de M1 (déplacement de `weapons`) |
| `stats` | stats et modificateurs | T1.2 |
| `effects` | actions des power-ups, contrats de déclencheurs et conditions ; objets, inventaire, familiers à venir | T2.5, puis T1.0a de M1 |
| `behaviors` | contrats de perception, ciblage, behaviors et état ; résolutions, boss à venir | T1.0a de M1 |
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

**Reporté de M0** : restart p2p (dette D14, fermée dans T3.2) — relancer une partie en ligne
demande de recréer la session GGRS entre les pairs ; une tâche de M1 devra le porter.

### Vague 0 : contrats (sériel, 4 j)

#### T1.0a Contrats de combat et d'IA — V1
`crates/combat` reçoit `weapons/` (déplacement, traces identiques) ; enums squelettes enregistrés au
`KindRegistry` : `ProjectileModifier`, `Pattern`, `StatusDef`, `Behavior`, `Perception`,
`Targeting`, `Effect { on, if, do }` ; composants d'état `Statuses`, `BehaviorState` ; crate
`behaviors` créé, `effects` étendu ; futurs systèmes dans les sets existants.

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

- T1.0a : mergée (`51d70b6`).
- m0-v7 : phase 1 (diagnostic) mergée (`5f977c9`) ; **phase 2 mergée le 2026-10-03** (`716df9e`,
  Codex puis b1) — bots `chasseur`/`acheteur`, secours de spawn, guidage de récupération, snap de
  la salle de spawn sur la grille (graines 16/17) ; **20/20 graines en victoire vague 5, 0 desync,
  0 plafond** ; 83 traces bénies avec preuve (toutes changent par le snap), 7 scénarios
  ré-étalonnés (visée de clone_duo ré-enregistrée) ; D20 fermée ; **nouvelle référence p2p
  sha256 `39654b07…`** ; critère 200 graines lancé par l'orchestrateur (overnight).
- m0-v6 : mergée (`52046ab`) ; quatre traces bénies (`b8c9cae`).
- m0-v8 : mergée (`d45431f`) ; trace `avant_poste_demo` bénie (`8fe873e`) ; `--map` (`cd6445a`) ;
  `start_map` basculé (`0942abb`) ; validation 20 graines : 0 desync, les softlocks relèvent de D20
  (m0-v7 phase 2 en cours, re-validation prévue).
- m0-dettes (lot D3, D13, D19, D21-D24) : mergée (`8a92d10`) ; aucune trace changée.
- m0-v9 : mergée (`2cb0897`) ; aucune trace changée ; recette allumette et recette `--matchbox`
  rejouées par l'orchestrateur : traces identiques, même sha256.
- m0-v10 : mergée (`7650f86`) ; aucune trace changée (zéro ligne Rust) ; recette du profil docker
  allumette rejouée par l'orchestrateur en N=2 et N=4 : N=2 sha256 `55ec099d…` identique à m0-v9,
  N=4 quatre traces identiques. Fix-up `m0-v10b-down-profile` (`77d652c`) : `--profile allumette`
  sur les trois `down` de nightly.sh (le conteneur profilé survivait sinon à l'arrêt du nightly —
  piège signalé par b0 elle-même après CLOS).
- m1-v1a : **mergée le 2026-10-03** (`ea1b222`) — projectiles composables (T1.1) + attentes
  `BulletCount`/`HitsAtLeast` ; 17 traces testbed bénies par l'orchestrateur (`6a4c9f`) ;
  bundle git (push 403 du proxy cloud) ; bench + p2p rejoués sur l'état fusionné (journal §10).
- m1-v1e : **mergée le 2026-10-03** (`9e85e79`) — mode `Floors` (T1.8) + attente `FloorIndex` ;
  bundle c2 récupéré via Syncthing, re-merge de main (conflits prévus CLAUDE.md attentes +
  conventions §16/§17 résolus en `51f5cb0`) ; 80 scénarios verts, 338 tests, sim Floors 5
  graines sans desync ; bench + p2p rejoués sur l'état fusionné (journal §10) ; réserve
  `joueur_hors_mur` entre bots à suivre avec m0-v7 p2.
- m0-v11 : **mergée le 2026-10-03** (`562729c`) — F5 équilibrage par joueurs (vagues/prix/santé
  en `NumOrExpr`, résolus une fois au lancement avec `players`, D25 fermée) ; 80 traces
  existantes inchangées, 2 traces de preuve bénies (`215e70a`) ; p2p N=2 sha256 `55ec099d…`
  identique à la référence ; régression testbed (vagues sans dossier `Wave`) trouvée et
  corrigée par b0 avant livraison.
- fix-local-launch-sounds (hors tâche, Mac de William) : **mergée le 2026-10-03** (`4ec8de0`) —
  lancement local sans `--players`, sons de feedback, desync du dash (CursorPosition réécrite
  avant les `continue`), scénario `dash_aim_change`, fuzz d'inputs ; vérifiée : 83 verts, p2p,
  preuve de régression rejouée, fuzz 16 parties sans divergence ; dettes D30-D33.
- m1-v1a-patterns : **mergée le 2026-10-04** (`T1.2`, b0) — émetteurs, patterns `Scatter`/`Named`,
  tir ennemi `ai.ranged`, kind `Pattern`, checksum neutre pour composants ; 83 traces inchangées,
  4 bénies (`43b0ffd`) ; p2p sha256 `39654b07…` identique à la référence m0-v7.
- m1-v1d-terrain-destructible : **mergée le 2026-10-04** (`T1.0b` + `T1.6`, b1) — crate `world`
  (`CellKind`, `CellGrid` neutre, `Destructible`, set `World`), cavernes par automate cellulaire
  (kind `Cave`, `MapGenerationMode::Cave`), `Action::DestroyTerrain`, `FrameEvents<ProjectileWallHit>`,
  attente `CellState`, conventions §21 ; vérifiée sur l'état fusionné : 87 traces inchangées, 4 bénies
  (`2d6fca7`), 426 tests de crates, lint/fmt/scripts/gen/exemples OK, p2p sha256 `39654b07…`
  identique à la référence ; bench strict au calme non fait (62,7 fps sous charge, budget tenu).
- m1-v1d-surfaces : **mergée le 2026-10-04** (`T1.7`, b1) — `world::SurfaceGrid` creuse et neutre lue depuis
  la couche LDtk `Surfaces`, kind `Surface` (facteurs `move_speed`/`acceleration` traduits en `MoveSpeed`/
  `EnemyMoveSpeed`/`Acceleration`), système dans `Input` avant `apply_inputs`, `Flying` ignoré, `CellState.surface`,
  conventions §26 ; vérifiée sur l'état fusionné : 91 traces inchangées, 4 bénies, 432 tests de crates,
  lint/fmt/scripts/gen/exemples OK, p2p sha256 `39654b07…` ; bench strict au calme non fait.
- m1-v1c-behaviors-composables : **mergée le 2026-10-04** (`T1.4`, b0) — behaviors composables v1
  (`Chase`, `KeepDistance`, `Strafe`, `Charge`, `Shoot`, `Melee`, `Flee`, `Wander`, sélection par priorité,
  `Perception`, `Targeting`), zombies réécrits en RON avec traces identiques, `Shoot` remplace `ai.ranged`,
  attentes `EnemyState`/`EnemyDistance`/`EnemyContactBefore`/`EnemyNeverInWall`, code mort retiré, §22 ;
  vérifiée groupée avec T1.5 et T1.14 (ci-dessous).
- m1-v1c-variantes : **mergée le 2026-10-04** (`T1.5`, b0) — `variants` dans `CharacterConfig`, RNG local
  `fnv1a("variants") ^ run_seed ^ net_id` (net id alloué dans `spawn_enemy` et passé à `create_character`),
  composant neutre `Variant`, champ LDtk `variant` (transite par le pipeline de génération), `EnemyVariant`,
  événement `variant_spawn`, attente `Stat` avec `entity` optionnel, lint, §25 ; vérifiée groupée.
- m1-v3-bots-v1 : **mergée le 2026-10-04** (`T1.14`, b1) — `prudent` v1 (esquive des projectiles, gestion
  d'arme, freinage au portail), `--until-floor`, soft-lock `Floors`, `floor_frames`/`damage_taken`/`dodges`,
  `floor_d` et `trois_niveaux` = a, d, c ; 20/20 graines finissent trois niveaux ; §24 v1. **Vérification
  groupée** sur l'état fusionné `5d8426e` : 106 scénarios, 95 traces inchangées, 11 bénies ; 472 tests de
  crates ; lint/fmt/scripts/gen/exemples OK ; p2p sha256 `39654b07…` identique à la référence ; bench strict
  au calme non fait pour les trois.
- **Critère M0 §9.8, 200 graines sur `avant_poste`** (nuit du 2026-10-04, `alacod-sim` de `7e8f541`, 4
  `acheteur`, carte par défaut du manifeste `maps/avant_poste.ldtk` — **pas `test_map`** : le critère
  historique sur `test_map` reste à rejouer avec `--map exemples/test_map.ldtk`), 4 lots parallèles sous
  charge 12-25 : 183 graines jouées avant l'arrêt des sims par le harnais (pression mémoire), **183/183
  atteignent la vague 5, 0 desync, 0 softlock**, 2 graines avec un mort (69, 139), 6 386 à 8 259 frames ;
  les 17 manquantes (49, 50, 94-100, 147-150, 197-200) rejouées le matin : 17/17 vague 5, 0 desync, 0 softlock,
  1 mort (graine 200). **Total : 200/200 atteignent la vague 5 sur `avant_poste`, 0 desync, 0 softlock, 3 graines
  avec un mort.**
  Lot séparé de 20 graines `avant_poste` : 20/20 vague 5, 0 desync, 0 softlock, 0 mort, 6 522 à 7 583 frames.
- m1-v1a-statuts : **LIVRÉE le 2026-10-03** (tête `a5effb7`) — statuts Burn/Slow/Stun/Freeze
  (T1.3) + attentes `HasStatus`/`StatusStacks` ; 79 traces existantes inchangées (trace-diff
  identique sans `--ignore`), 8 scénarios testbed à bénir ; bundle c3 en attente de relais
  Syncthing (`1867_lore/tmp/`), vérification de l'orchestrateur ensuite.

- Ce fichier est la source de vérité des tâches : une ligne de statut par tâche (`à faire`, `en
  cours (branche)`, `mergée (commit)`), tenue par l'agent qui prend la tâche.
- Une tâche commence par un commentaire dans sa fiche : qui, quelle branche, quand ; elle finit par
  le commit de merge et la vidéo.
- Le digest hebdomadaire (plan §9.8) liste les tâches mergées, en cours, bloquées, et les métriques.
- Une tâche qui découvre une autre tâche l'ajoute ici, dans la vague suivante, jamais dans la sienne.

**Fiches détaillées** (2026-09-30) : `docs/taches/` contient un préambule commun (`README.md` :
où travailler, règles de compilation, vérification standard, protocole de preuve, merge, journal,
rapport) et une fiche autonome par tâche restante (T2.4 vérification/merge, T2.5 reprise, T2.8,
T2.12, T3.1, T3.2, `dettes.md`), écrites pour un agent sans contexte (DeepSeek, Qwen, Haiku) :
coller le préambule puis la fiche dans son prompt.

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
| 2026-09-29 | T1.1 (B1 : toute blessure par `FrameEvents<DamageEvent>`, résolveur unique dans `CollisionDamage`, règles pures `combat::team`/`combat::damage` (tir ami `Never`/`Always`/`Cursed` par arme, tag `cursed`, immunités et résistances par tag, invulnérabilité honorée, `DamageKind::True`), `Tags`/`Defenses` sur les personnages, `CollisionLayer` réduit aux collisions physiques, scénarios `friendly_fire_never`, `friendly_fire_cursed`, `immune_tag`, `PlayerScript::{tags, immune_to}`, `WeaponOverride::friendly_fire`) | `m0-v1-equipes-degats` | Sonnet | mergée ; rapport exemplaire : preuve par traces détaillées (idle et movement_melee identiques ; shoot_around et remote_first_fight diffèrent parce que les balles ne sont plus absorbées sans dégât par les hitbox de griffe des zombies, conséquence voulue de la fin de la matrice de couches pour les dégâts) ; fusion de `main` (T0.3, T1.5, T1.6) par l'orchestrateur, conflits sur manifestes, `config.rs`, lock et traces, re-bless des dix-sept traces ; 37 unitaires combat, filet K0, seize attentes, bench strict. Dettes : `EnemyAiConfig.friendly_fire` pas encore en RON ; silence historique des dégâts d'ennemi sans `DamageAccumulator` reproduit tel quel (à trancher en B2/B3) ; attaque d'ennemi traduite en événement à la frame suivante |
| 2026-09-29 | T1.2 (B2 : crate `stats` (`StatsPlugin`, `StatReader`), `ModifierOp::Pct`, résolution `(base + ΣAdd) × (1 + ΣPct) × ΠMul` après `Set`, `Stats` sur chaque personnage depuis les champs existants et `stats:` du RON (mouvement, sprint, armes, santé, constantes de `pathing.rs` par ennemi), `Health.max` synchronisé, scénario `stat_move_speed`, `PlayerScript::modifiers`, outil de preuve `ALACOD_DUMP_TRACE` + `scripts/trace-diff.py` (docs/conventions.md §7-8), lint des stats) | `m0-v1-stats` | Sonnet | mergée ; preuve : cinq scénarios identiques hors `Stats`/`Modifiers` avant bless ; 37 unitaires sim_core, bench strict ; fusion de `main` (T2.11) par l'orchestrateur (un conflit dans le runner), traces des bots re-blessées. Non branché (hors liste) : friction, dash, rampe de sprint, mêlée ; `Armor`/`Luck` non lus. Leçon : après une fusion, `touch` des sources avant de faire confiance à un binaire de test (un binaire périmé a fait échouer une fixture) |
| 2026-09-29 | T1.6 (A4 : `RunSeed` dérivée de la graine de carte, `RngStreams` par nom (`fnv1a(nom) ^ graine`), vagues sur `"waves"`, dispersion des tirs sur `"weapons"`) | `m0-v2-rng-flux` | Haiku | mergée ; l'agent a rendu les mains sur une « divergence synctest » qui était son propre bug (`RngStreams` enregistré comme composant, jamais restauré) ; corrigé par l'orchestrateur, six attentes recalées sur le nouveau tirage (dont une balle qui sort par une fenêtre : les fenêtres laissent passer les balles par conception), `event_kill_found` déplacé sur `remote_first_fight`, quatorze traces re-blessées (nouveau tirage) ; filet K0, seize attentes, quatorze unitaires RNG. Mergée avant T1.1, qui re-blessera après fusion de `main` |
| 2026-09-29 | T1.5 (A1 : manifeste `game.ron`, registre typé (`CharacterId`, `WeaponId`, `MeleeWeaponId`, `EnemyId`, `WaveConfigId`, `MapId`), lint (référence cassée, id dupliqué, hors plage, kind inconnu, flottant dans un `Fixed`), CLI `alacod lint`, `make lint`, `starting_weapons` par personnage, rechargement à chaud dans le lobby, hook Claude Code `PostToolUse` sur `games/**`) | `m0-v2-manifeste-lint` | Sonnet | mergée ; rapport complet et honnête (62 + 6 tests, lint des deux jeux à zéro erreur, quatorze traces identiques, testbed 119 frames) ; l'agent avait signalé `examples/character_tester.rs` cassé à l'exécution (pas de `Registry`) : branché par l'orchestrateur sur `games/zombies`. Le testbed ne porte plus les sprites de zombie ni les vagues. Dettes : kinds en dur de `register_kinds` pas branchés sur le registre ; sprites encore nommés en dur dans `global_asset.rs` (A5) ; rechargement à chaud et hook non observés en conditions réelles |
| 2026-09-29 | T2.11 (bots v0 : `crates/bots` (`BotView`, `decide()` pur, profils `immobile`/`fonceur`/`prudent`), `InputSource::Bot`, `bot:` dans `PlayerScript`, `StopEarly` + `run_with_options` dans le runner, CLI `alacod-sim` (JSON par graine, desync, `--save-scenario`), `make sim`, page de revue, scénarios `bots_two_fonceurs` et `bots_four_mixed`, test de rejouabilité) | `m0-v3-bots` | Sonnet | mergée ; bon rapport ; l'agent a trouvé et corrigé par son propre test un bug (création d'un flux RNG jamais tiré, qui faisait diverger le replay) ; fusion de `main` (T1.1) par l'orchestrateur : conflits additifs et deux littéraux `PlayerScript` sans les nouveaux champs, deux traces de bots re-blessées, dix-sept autres identiques ; `alacod-sim` à quatre bots sans desync (5 graines avant fusion, 2 après). Non vérifié : le seuil des cinq minutes pour 5 graines (13 min sous contention) ; graine 4 sans kill en 20 000 frames (bots sans pathfinding, limite v0) ; `docs/conventions.md` pas à jour |
| 2026-09-29 | T2.9 (testbed : salles `arena`, `corridor`, `two_rooms_door`, `window` dérivées par script, entité LDtk `CharacterSpawn` (personnage + équipe) propagée dans le générateur de salles, `CharacterConfig::{team, ai, counts_hits}`, `EnemyAiConfig::stationary`, `HitCount` + attente `EntityHits`, personnages `dummy`, `target`, `follower`, `ally`, `civilian`, `breacher`, neuf scénarios `testbed_*` et budgets) | `m0-v2-testbed` | Sonnet | mergée ; bon rapport ; dix-neuf traces zombies intactes ; fusion de `main` (T1.2) par l'orchestrateur, neuf traces testbed re-blessées, deux tests unitaires de `map_ldtk` réparés (chemin `assets/` d'avant T0.3). Dettes : `HitCount` enregistré sans checksum (`rollback_and_trace_no_checksum`) et `stationary` exclu du hash d'`EnemyAiConfig` pour ne pas toucher aux traces : à remettre au checksum au prochain bless global justifié ; personnages de laboratoire sans sprite ; pas d'unitaire pour `EntityHits` |
| 2026-09-29 | T2.10 (générateur de scénarios v0 : `PlayerScript::weapon` (arme unique au spawn), gabarit « arme sur la cible » dans l'arène du testbed (tir pulsé, marche en L pour la mêlée), `test:` sur `WeaponConfig`/`MeleeWeaponConfig` (hors checksum, validé par le lint), `EntityHits::max`, `tests/scenarios/generated/<jeu>/` joués par le test existant, CLI `alacod-gen --play --bless` (suppression des scénarios d'armes disparues), `make gen`, test d'acceptation « un `test:` faux échoue ») | `m0-v3-generateur` | Sonnet | mergée ; neuf scénarios générés pour `zombies` (cinq avec `test:`), bon rapport ; l'agent a confirmé indépendamment la reconstruction de `target.ron` (trace identique) ; fusion de `main` (T1.3) par l'orchestrateur, neuf traces générées re-blessées ; quarante et un scénarios verts. Non exercé : `WeaponTest::expect` libre ; constantes de marche empiriques |
| 2026-09-29 | T2.13 (I2 v0 : `ui/feedback.ron`, flash blanc à l'impact (`HitFlash` hors rollback), secousse de caméra déterministe quand un joueur local est touché, sons de tir et de rechargement via bevy_kira_audio, lignes de preuve `feedback f...`) | `m0-v4-feedback` | Haiku | mergée ; l'agent a livré un module qui **ne faisait rien** : la config chargée n'était jamais publiée (`FeedbackConfigLoaded` absent), la secousse visait n'importe quelle caméra et s'appliquait après la propagation des transforms ; il n'avait produit aucune capture et avait pris l'animation de coup pour son flash. Corrigé par l'orchestrateur, prouvé par captures de `remote_first_fight` (secousse : 9 à 15 % des pixels changent d'une frame à l'autre contre 0,4 % au repos) ; quarante-trois traces intactes. Les sons ne sont pas vérifiables sans écoute. Leçon : pour la présentation, exiger la capture et la regarder soi-même |
| 2026-09-30 | T2.14 (K6b : `scripts/nightly.sh` (bench strict, `alacod-sim`, p2p headless avec comparaison des traces entre clients, vidéos, page de revue, notes `tests/review-notes/<commit>.md`), `.github/workflows/nightly.yaml` sur le runner `alacod-builder`, `docker-compose.ci.yaml` (service `signaling` = matchbox_server nu ; allumette en profil optionnel), `make nightly` / `make nightly_quick`) | `m0-v5-ci-nuit` | Haiku | mergée ; l'agent avait sauté le p2p (« problème docker »). Réparé par l'orchestrateur : contexte de build introuvable depuis un worktree, image allumette cassée (base Rust trop vieille, OpenSSL : corrigée et mergée dans le `main` d'allumette), allumette exige un jeton JWT dans le chemin WebSocket que le jeu natif n'obtient pas (API HTTP `/auth`, `/lobbies`) → matchbox_server nu pour le test de desync ; assets explicites des binaires de jeu hors `cargo run` ; vidéos par scénario avec sortie journalisée. **Trouvaille** : `Player` hachait `name` et `pubkey`, propres à chaque client en p2p → checksums différents dès la frame 0 ; seul `handle` est haché désormais (preuve trace-diff sans `--ignore` sur quatre scénarios, quarante-cinq traces re-blessées). Résultat : traces identiques entre 2 et entre 4 clients p2p réels sur 600 frames ; `make nightly_quick` vert de bout en bout en six minutes. Non vérifiable d'ici : cron et artefacts du runner. Dette : intégrer le flux d'authentification allumette dans le jeu natif |
| 2026-09-30 | T2.2 (B7 : `AmmoType` ouvert, `ammo_type` requis par arme, `AmmoReserves` par joueur (rechargement depuis la réserve, `mag_quantity` supprimé), `weapon_slots` (3 pour le joueur zombies : traces équivalentes), lâcher (`INPUT_DROP_WEAPON`, touche G, anti-rebond) et ramasser (`WeaponPickup`, `InteractionType::Weapon`, `spawn_weapon_pickup` avec prix pour T2.3), mêlée pendant le rechargement prouvée par test, attentes `AmmoReserve`/`WeaponPickups`, scénarios `ammo_shared_reserve` et `drop_pickup_swap`) | `m0-v1-munitions` | Sonnet | mergée ; rapport honnête (écart documenté sur les composants à ignorer dans la preuve : `Weapon` et `WeaponModesState` changent de forme, pas de valeur) ; preuve identique sur six scénarios ; fusion de `main` (T2.13, T2.14) par l'orchestrateur, quarante-cinq traces re-blessées, p2p à deux clients identique. Non fait : prompt de ramassage à l'écran ; testbed à deux emplacements avec trois armes de départ |
| 2026-09-30 | T2.3 (C5 v1 : crate `run` (`Currency`, `CurrencyEvent`, `Perks`), points par kill/coup/réparation (`economy.rs`), portes payantes, armes murales (`WeaponLocation` enfin lue, recharge au ratio, anti-rebond), perks (`SodaLocation`, `perks.ron`, kind `Perk`, modificateurs permanents, soin immédiat sur `MaxHealth`), `economy.ron` (kind `Economy`), source HUD `currency`, attentes `Currency`/`Stat`, `test_map_shop.ldtk`, scénarios `buy_door`, `buy_wall_weapon`, `buy_perk`, `points_on_kill`) | `m0-v1-monnaie` | Sonnet | mergée ; 2 h 12, rapport complet ; deux bugs trouvés par la preuve (recharge facturée à chaque frame ; `MapRollbackMarker` qui aurait décalé les `GgrsNetId` de tous les scénarios) ; `trace-diff.py` réparé pour les noms génériques (`--ignore` ignorait silencieusement `FrameEvents<T>`) ; quarante-neuf scénarios verts, 26 attentes, bench strict, p2p à deux clients identique. Non fait : prix à l'écran pour armes murales et perks ; bornes de lint sur `economy.ron` ; la carte jouable reçoit ses murs d'armes en T2.6 |
| 2026-09-30 | T2.4 (F1 : ressource rollback `Run { seed, mode, step, players, flags }`, `RunOutcome` fondu dans `Run.step`, trait `RunModeRules` (`Waves`), `RunSummary`, écran de fin qui relance sans quitter le binaire (`RunRequest::{Restart, ToLobby}`, session et entités détruites, même configuration), attentes `RunState`/`RunSummary`, scénario `run_lose_summary`, test de relance chronométré avec trace identique) | `m0-v1-etat-de-run` | Sonnet | mergée le 2026-10-01 (`1803bf0`) ; vérification indépendante par Codex (agent externe, fiche `docs/taches/T2.4-verifier-merger.md`, rapport `docs/taches/rapports/m0-v1-etat-de-run.md`) : fusion de `main` adbd060, preuve `trace-diff` sur `idle`, `points_on_kill`, `downed_all_lose`, `two_players_shooting` (3 996 frames, identiques hors `Run`/`RunOutcome`), 52 scénarios, 221 tests, lint, fmt, scripts, p2p deux clients identiques (599 lignes), bench strict vert au calme (bench_bullets 108,6 fps, bench_horde 68,4 fps) ; deux corrections de Codex (somme du résumé par `order_iter!`, test RON `mode: Some(Sandbox)`, doc §13 : les vagues ne lisent pas `Run.step`) ; rejoué par l'orchestrateur : scénarios en bench strict, tests des neuf crates, lint, fmt, scripts, verts. Dettes : fixture `entry.mode` (T2.8) ; `game.ron` exige `mode: Some(Waves)` faute d'`implicit_some` (D16) ; les vagues continuent après `Ended` ; pas de résumé à l'abandon ; bouton Lobby absent (T2.12) |
| 2026-09-30 | T2.6 (contenu `zombies` : quatre armes murales avec prix et quatre machines à perks dans les gabarits de `test_map.ldtk` — définitions des champs `weapon`/`price`/`perk` ajoutées au projet LDtk —, quatrième arme à distance `rifle` (semi-automatique, `Balle` partagée avec la mitraillette, sprite du fusil à pompe), scénario `shop_tour`, README du jeu (arborescence, gabarits assemblés par le générateur, tableau des machines, marche à suivre LDtk), `docs/conventions.md` §1/§8, `Expectation::WeaponPickups` hors armes murales, scénario généré `weapon_rifle` (8 coups observés, bornes 4..40)) | `m0-v2-contenu-zombies` | Haiku, reprise par l'orchestrateur | mergée ; l'agent a livré une carte qui ne chargeait pas (instances sans `defUid`/`width`/`height`/`__worldX`/`__worldY`, champs sans `defUid`/`realEditorValues`, définitions de champs absentes, fichier reformaté sur 31 000 lignes) et un scénario qu'il n'avait jamais vu passer (rachat de munitions à 250 au lieu de l'achat du pistolet, que le joueur possède déjà), rapport « 100 % complet » ; carte régénérée depuis le texte natif de `main` (163 lignes de diff) ; preuve `trace-diff` sur `idle` et `four_players_idle` : seules les dix entités machines (ids 108 à 117) et `GgrsNetIdFactory` diffèrent ; le générateur réutilise les gabarits (deux salles `Level_0` avec la seed des scénarios, donc deux murs `pistol` et deux Juggernog, documenté dans le README) ; 28 traces `test_map` re-blessées, `test_map_shop` et testbed inchangées ; 51 scénarios verts, unitaires de neuf crates, lint des deux jeux, bench strict (bench_bullets 92,9 fps, bench_horde 60,7 fps, T2.4 en compilation à côté), p2p à deux clients identique (599 frames, 137 entités). Non fait : table des power-ups (format défini en T2.5), sprites rangés par entité (A5, le `rifle` n'a pas de sprite propre), prix affichés à l'écran. Incident : la vérification de T2.4 (Sonnet) recompilait tout Bevy depuis un target vide (2 Go libres), tuée et redirigée vers le checkout `main` |
| 2026-09-30 | T2.5 (C1 v0 : crate `effects` (`Action` : `TimedModifier`, `RefillAmmo`, `RepairAllWindows`, `KillAllWaveEnemies`, `CurrencyMultiplier`, applicateur pur `as_modifier`), `game::powerups` (power-up au sol `PowerUpPickup` ramassé au passage et appliqué à tous les joueurs — sémantique CoD —, expiration, drop à la mort d'un ennemi par le flux RNG `loot` dans l'ordre des `GgrsNetId`), kind `PowerUp` (`items/powerups.ron` des deux jeux, registre, lint `drop_chance`/`weight`, trois fixtures), Double Points par la stat `powerup_currency_multiplier` lue par `award_points_system`, réglages de scénario `powerups` (placement scripté en coordonnées `Fixed`, dans `GgrsSchedule`) et `powerup_drop_chance_override`, attente `PowerUpPickups`, six scénarios `powerup_*` (testbed), `docs/conventions.md` §14) | `m0-v1-power-ups` | Sonnet (WIP interrompu), reprise par Codex | mergée le 2026-10-01 (`a2cb4ef`) ; rapport `docs/taches/rapports/m0-v1-power-ups.md` ; Codex a corrigé le WIP (`PowerUpPickup` hors checksum, ramassage encore possible à la frame d'expiration, `f32` dans le placement, Max Ammo par type de munition avec annulation du rechargement, réglages perdus au réenregistrement, drops aléatoires qui consommaient le flux `loot` dans tous les scénarios — divergence à la frame 469 de `points_on_kill` contre l'affirmation de preuve du WIP) : 57 scénarios et le générateur déclarent `powerup_drop_chance_override: "0.0"` ; preuve `trace-diff` contre `main` cc2a7a8 sur `idle`, `two_players_shooting`, `points_on_kill`, `downed_all_lose` (3 996 frames, identiques hors `PowerUpPickup`/`FrameEvents<PowerUpPickedUp>`), les 52 traces héritées sont restées identiques au bless ; six tests de systèmes, deux tests d'intégration (expiration selon la table, replay de Max Ammo) ; vérification de Codex puis rejouée par l'orchestrateur : 58 scénarios en bench strict au calme (bench_bullets 109,0 fps, bench_horde 68,2 fps), 240 tests des dix crates, lint des deux jeux, fmt, scripts, p2p deux clients identiques (599 lignes). Non fait : sprite de pickup et indicateur d'effet actif (T2.12), son/flash au ramassage (T2.13). Dettes : Nuke ne crédite pas de points (D17) ; deux power-ups identiques se cumulent (D18) ; `InputRecorder::to_scenario` ne reporte pas les réglages de scénario (D19) |
| 2026-09-29 | T2.1 (B4b : `CollisionGrids` dérivées hors rollback (murs statiques, personnages reconstruits avant et après `Movement`), balles, mêlée, déplacement et séparation sans boucle sur tous les colliders, tie-breaking conservé, `pathing.rs` sans `HashMap`, `Scenario::wave_overrides` et `WeaponOverride::firing_rate`, scénarios `bench_bullets` (155 balles) et `bench_horde` (61 ennemis)) | `m0-v1-grille-adoption` | Sonnet | mergée ; correction prouvée : trente-deux traces bit-identiques et `trace-diff.py` sans `--ignore` identique sur quatre scénarios ; l'agent a trouvé par vérification croisée que les hitbox de mêlée comptaient dans le ralentissement des joueurs (conservé). Gain non mesuré en ratio (la référence force brute ne sait pas jouer les scénarios de bench) ; machine calme : bench_bullets 145 fps, bench_horde 78 fps, planchers à 70 et 38. Vérification de l'orchestrateur d'abord tuée par la mémoire (calculs CFD externes), refaite au calme : quarante-trois scénarios, bench strict, suites, lint verts |
| 2026-09-29 | T1.3 (B6 « à terre » : `combat::downed` (`Downed`, `Reviving`, `RunOutcome`), `bleedout_frames`/`revive_frames`/`downed_speed_mult` en RON avec lint, décision à terre ou mort à 0 PV selon les coéquipiers debout, saignement, réanimation par interaction maintenue (`InteractionType::Revive`), ennemis et flow field qui ignorent un joueur à terre, défaite quand tous à terre, attentes `PlayerDowned`/`PlayerRevived`/`Defeat`, scénarios `downed_revive`, `downed_bleedout`, `downed_all_lose`, écran de fin lit `RunOutcome`) | `m0-v1-a-terre` | Sonnet | mergée ; preuve : cinq scénarios identiques hors `RunOutcome`, `two_players_idle` diverge à la frame 1173 (le joueur 1 tombe au lieu de mourir) ; 22 attentes, bench strict. Fusion de `main` (T2.9) par l'orchestrateur ; **incident** : `target.ron` de T2.9 n'avait jamais été commité (motif `**/target*` du `.gitignore`), `main` était rouge sur deux scénarios testbed entre les merges de T2.9 et de T1.3 ; fichier recréé, motif corrigé (`**/target/`), neuf traces testbed re-blessées. Leçon : un fichier livré peut être avalé par le `.gitignore` sans que `git status` le montre ; vérifier `git check-ignore` sur les nouveaux fichiers d'un agent |
| 2026-10-01 | T2.8 (lint des nouveaux kinds : 17 règles — `ammo_type` Custom vide (lint) ou inconnu (parse), `friendly_fire` inconnu (parse, miroir ajouté aux schémas), perks (modifiers vide, `op: Mul` avec `value` ≤ 0, id dupliqué détecté par `KeyedEntries` au lieu de l'écrasement silencieux de la `BTreeMap`), power-ups (frames/factor > 0, actions non vides, lifetime/pickup_range > 0, weight/drop_chance scindés en deux fixtures), `refill_price_ratio` dans [0, 1] (dette T2.3), son d'arme absent de `assets/` (dette D2 : `machine-gun-reloading.ogg` → `machine-gun-reload.ogg` dans les deux jeux, les deux sons copiés dans le testbed qui n'avait pas de dossier `sounds/`) ; une fixture + un test par règle, `t2_8_fixtures_have_a_single_problem`, erreurs de parse préfixées du nom du champ, docs `conventions.md` §3/§12/§14 | `m0-v2-lint-kinds` | Claude (cloud) | mergée ; vérifié localement sur l'état fusionné : 58 scénarios verts (aucune trace modifiée), 257 tests des dix crates, lint des deux jeux sans erreur, fmt, scripts (4 avertissements préexistants), `make gen` sans modification, bench strict au calme (bench_bullets 106,9 fps, bench_horde 67,9 fps), p2p deux clients identiques (599 lignes). Non fait : clés répétées encore muettes dans `weapons.ron`/`melee_weapons.ron`/`powerups.ron` (une ligne par schéma, une fixture chacun), `Mul` ≤ 0 non vérifié sur les `TimedModifier` de power-ups, `ui/feedback.ron` non linté |
| 2026-10-02 | T2.12 (HUD v1 : caisse `crates/game/src/ui/hud.rs` (+890 lignes, `HudPlayer`/`HudPlayerState`/`HudInteractables`), sources perks/à terre/power-ups/prompt dérivées en `Update` de l'état rollback (jamais d'événements, vidées quand `Run.step == Ended`), `Prompt` (Ouvrir — $750, Acheter fusil à pompe — $1000, recharge — $500, Ramasser, possédé, Réanimer, Réparer), `seconds_left` (`div_ceil`), icônes grises avec initiale, `parse_color` étendu à `#rrggbbaa`, 7 tests unitaires ; `closest_interactable` réplique exactement `interaction_detection_system` (distance de surface, `<=` portée, tri par `net_id`, plus proche strict) ; `InteractionPromptText` retiré (`display_interaction_prompts` garde les cercles de portée) ; bouton Lobby (`RunRequest::ToLobby`) sur l'écran de fin (FiraMono pour « DÉFAITE ») ; `hud.ron` des deux jeux ; 4 captures `docs/captures/hud-v1/`) | `m0-v4-hud-v1` | Claude (cloud) | mergée le 2026-10-02 (`d2ef155`) ; rapport honnête (clic du bouton Lobby non testé, `make gen` sauté, p2p et bench impossibles dans le cloud) ; vérification rejouée par l'orchestrateur sur l'état fusionné avec T3.1 : 61 scénarios verts sans trace modifiée, 265 tests des dix crates (257 + 7 HUD + 1 T3.1), lint des deux jeux, fmt, scripts (4 avertissements préexistants), enregistrement rollback complet, `make gen` sans modification, bench strict au calme (bench_bullets 107,9 fps, bench_horde 67,2 fps), p2p deux clients identiques (599 lignes) ; captures vérifiées par analyse de pixels (couleurs et positions attendues : perks, à terre, prompt d'achat jaune `#f1c40f` centré, power-up) et glyphes de la police confirmés (DÉFAITE). Incident d'orchestration : le premier merge de la tâche a été posé par erreur sur `m0-v3-scenarios-clone` au lieu de `main` ; défait et refait sur `main` (arbres identiques au premier merge, preuve `git diff`). Non fait : clic du bouton Lobby (pas d'infrastructure de test d'interface), revue visuelle des captures par William |
| 2026-10-02 | T3.1 (scénarios du clone : `WaveOverride.max_wave`/`min_wave_delay_frames` (`replay.rs`, `runner.rs`), test de rejet `max_wave < min_wave` dans `run.rs`, scénarios `clone_solo` (3 047 frames, 13 kills, victoire), `clone_duo` (4 668 frames) et `clone_quad` (4 372 frames, `Playing`), 61 scénarios verts, 58 traces héritées hash-identiques, déterminisme 2/2, vidéos `target/videos/0c6ba31/` (clone_solo 50,8 s, clone_duo 77,8 s, clone_quad 72,9 s, montage, `events.json` 127/128/240)) | `m0-v3-scenarios-clone` | Codex | mergée le 2026-10-02 (`1e97292`) ; rapport honnête (revue visuelle, p2p et bench délégués à l'orchestrateur) ; vérification rejouée sur l'état fusionné : 61 scénarios verts sans trace modifiée (315 s), 265 tests des dix crates, lint des deux jeux, fmt, scripts, bench strict au calme (bench_bullets 107,9 fps, bench_horde 67,2 fps), p2p deux clients identiques (599 lignes) ; vidéos revues mécaniquement par sondes (frames non vides, évolutives, durées exactes). Non fait : revue visuelle humaine des trois vidéos (page de revue, reprise par T3.2/T3.3) |
| 2026-10-02 | T3.4 (dettes de lint et de sortie : huit règles — clés répétées dans `weapons.ron`/`melee_weapons.ron`/`items/powerups.ron` (`KeyedEntries`, `DuplicateId`), `TimedModifier` `op: Mul` avec `value` ≤ 0, durées nulles du flash et de la secousse, amplitude négative, son de feedback absent, armes de départ au-delà de `weapon_slots` ; `registry.rs` charge `ui/feedback.ron` avec un schéma typé (plages et existence des sons), les autres fichiers Ui restent validés syntaxiquement ; huit fixtures + un contrôle d'isolation, 36 tests de fixtures ; D7 : `weapon_slots: 3` dans le `player_config.ron` du testbed (les trois armes étaient déjà créées, la capacité limitait les ramassages) ; D8/D9 : impressions de debug → `debug!` dans map/map_ldtk, message des armes murales précisé ; D11 : `ScenarioOutcome.entity_hits` lu en fin de partie hors simulation (compteurs `HitCount`), colonne « coups » du générateur, test du gabarit faux comparé à `EntityHits` ; D16 : `implicit_some` de `GameManifest`, les deux manifestes déclarent `mode: Waves`/`mode: Sandbox` explicites) | `m0-v5-dettes-lint-sortie` | Codex | mergée le 2026-10-02 (`1af3780`) ; vérifiée par l'orchestrateur sur l'état fusionné : 100 tests de `content` (64 unitaires + 36 fixtures), 275 tests des dix crates, 61 scénarios verts sans trace modifiée (316 s), lint des deux jeux, fmt, scripts (4 avertissements préexistants), enregistrement rollback complet, `make gen` sans modification, bench strict au calme (bench_bullets 104,7 fps, bench_horde 66,7 fps, bots_four_mixed 109,5 fps). p2p non rejoué par l'orchestrateur : aucune ligne de simulation touchée, les 61 traces identiques en sont la preuve ; le p2p à deux clients de l'agent (599 lignes identiques) figure dans son rapport. Non fait : aucune réécriture de trace (aucune référence changée), CI distante non vérifiée dans le rapport |
| 2026-10-02 | T3.2 (vidéos du clone sur `main` (`make videos`/`views`, 5 vidéos ≤ 5 Mo copiées dans `docs/digests/videos/` avec leurs `events.json`), comparaison vague 0 (`make compare_video BASE=4e93269`, la dérive du clone est visible : 9 000 à 63 000 pixels de différence, 8 moments clés contre 14), `alacod-sim` 20 graines à 4 bots dans le cloud (0 desync, mais **aucune graine n'atteint la vague 5** : 12 finies par la mort des bots, 8 bloquées au plafond de 20 000 frames dont 5 avec les quatre bots en vie — dette D20), digest `docs/digests/m0-fin-de-vague-2.md` (chantier par chantier, critères §9.8 un par un : le critère des bots est non, F5 jamais porté, CI de nuit jamais verte — 15 runs), notes de revue `tests/review-notes/4f0d550.md`, D14 fermée (restart p2p reporté à M1) ; correctif en cours de route : `compare_video` jouait le code de la base des deux côtés (target partagé + hash identiques des crates du workspace → artefacts périmés) — `inherit_errexit` et `touch` des sources des deux arbres dans `scripts/scenario-video`) | `m0-v4-videos-digest` | Claude (cloud) | mergée le 2026-10-02 (`90486a7`) ; vérifiée par l'orchestrateur sur l'état fusionné : 277 tests des dix crates (la suite des 61 scénarios est jouée deux fois — par le `cargo test` des crates puis par `make test_scenarios` : 316,5 s puis 323,9 s — verts sans trace modifiée), lint des deux jeux, fmt, scripts (4 avertissements préexistants), `make gen` sans modification ; bench et p2p non rejoués : aucune crate touchée (seuls `scripts/scenario-video`, le commentaire d'`idle.ron` et des docs/vidéos), les 61 traces identiques en sont la preuve ; MP4 vérifiés valides (ISO Media), `sim.json` parse. Trouvailles du digest converties en dettes D20-D25 et fiche `m0-v7-bots-finisent-le-clone` lancée (le critère M0 des 200 graines). Non fait par l'agent : diagnostic des 8 graines bloquées (m0-v7), revue humaine (T3.3), page de revue non ouverte dans un navigateur |
| 2026-10-02 | T1.0a (M1 v0 : contrats de combat et d'IA — déplacement des armes game→combat (acteurs, colliders, grille, attentes, HUD d'arme) avec réexports (`game::weapons`, `game::collider`, `game::collision_grid`, chemins personnages), `sim_core::interaction` ; crate `behaviors` (`Behavior` ×8, `Perception` ×2, `Targeting`, `BehaviorState`), crate `effects` (`Effect{on,if,do}`, `On` ×11, `Condition` ×7, `GaugeThreshold`), `combat::projectile/status` (`ProjectileModifier` ×6, `Pattern` ×6, `StatusDef` ×4, `Statuses`) ; 45 kinds en 8 catégories via `KindRegistry`, plugins montés ensemble dans `CoreSetupPlugin`, aucun système ni set rollback ; §4.3 de `docs/conventions.md`) | `m1-v1-contrats-combat-ia` | Codex | mergée le 2026-10-02 (`51d70b6`) ; vérifiée par l'orchestrateur sur l'état fusionné : 290 tests / 0 échec / 15 ignorés des onze crates (13 nouveaux de la branche ; l'écart avec les 288/0/8 du rapport vient de l'union avec le code de T3.2), 61 scénarios joués sans bless (traces intactes, `clone_quad` compris), lint des deux jeux, fmt, scripts (4 avertissements préexistants), `make gen` sans modification ; p2p à deux clients rejoué : 599 lignes, traces identiques, **sha256 `a44f5b4f…` identique à celui du rapport** (la simulation fusionnée reproduit la branche au bit près) ; bench strict vert : `bench_bullets` 87,0 fps, `bench_horde` 54,2, `bots_four_mixed` 81,4, rejoués machine calme — pas thermique, donc l'écart avec les 106/67/102 du rapport est environnemental (machine de l'agent différente) ; les traces identiques garantissent que la simulation n'a pas changé. Caveat du rapport, confirmée : `Statuses` et `BehaviorState` sont enregistrés ensemble et s'annulent au checksum tant qu'ils sont vides — ne jamais monter l'un sans l'autre. Dettes : ids de contenu en `String` (résolution par la vague 1, validation T1.12) ; `combat` porte des dépendances lourdes (animation, stats, run, ggrs) pour des données pures ; l'agent a géré le disque plein de son worktree avec un wrapper `rustc-compact.py` local non commité (sans conséquence sur le code) |

| 2026-10-02 | m0-v7 « les bots finissent le clone », **phase 1** (diagnostic des huit graines plafonnées de M0 : ressource `SoftlockDump` — instantané en lecture seule, trié par `net_id`, relu entre deux updates —, capture du dump au plafond de frames et en fin de run dans le runner (`--json` avec champ `softlock`), sondes `nav_probe` (`ALACOD_PROBE_FILE`/`ALACOD_PROBE_JSON`), huit dumps JSON commités (graines 4, 11, 12, 13, 16, 17, 18, 19), test « le dump ne change pas la trace » (`crates/scenario/tests/softlock.rs`), ligne D20 enrichie du diagnostic : steering physiquement bloqué sur fenêtres/murs, bots en ligne droite, portes payantes jamais ouvertes, 48 Plomb inutilisés) | `m0-v7-bots-finisent-le-clone` | Codex | mergée le 2026-10-02 (`5f977c9`) ; vérifiée par l'orchestrateur sur l'état fusionné : 290 tests / 0 échec des onze crates, les 61 scénarios joués sans trace modifiée — la suite a seulement échoué sur le plancher strict de `bench_bullets` en toute fin (64,9 fps < 70 sous contention : rsync du seed de la phase 2 et compilation de la voie V6 en parallèle) ; rejoué machine calme : bench strict vert (`bench_bullets` 75,9 fps, `bench_horde` 56,8, `bots_four_mixed` 69,6), lint des deux jeux, fmt, scripts (4 avertissements préexistants), enregistrement rollback complet, `make gen` sans modification ; p2p non rejoué : phase de diagnostic pur, aucune ligne de simulation touchée (runner et sondes seulement), les 61 traces identiques en sont la preuve. **Phase 2** (correctif : pathfinding des bots, profils `chasseur`/`acheteur`, portes payantes et achats, fin de vague garantie, 20 graines de validation) lancée dans la foulée chez Codex (branche `m0-v7-phase2-bots-finisent-le-clone`, worktree et target amorcés par l'orchestrateur) ; les 200 graines restent le run de validation de l'orchestrateur après merge |
| 2026-10-03 | m0-v6 « lot de dettes power-ups et coups » (D6, D17, D18 : `PointsCredit::Nuke` crédité à chaque joueur vivant au ramassage d'un Nuke — résolu par `award_points_system`, `nuke_points` 400 × multiplicateur de monnaie, raison `"nuke"` —, `Modifiers::remove_by_source("powerup:<id>")` avant réapplication, deux tests sur `testbed_target_hits`) | `m0-v6-dettes-powerups` | Claude (cloud) | mergée le 2026-10-02 (`52046ab`) ; vérifiée par l'orchestrateur sur l'état fusionné : **la fiche disait « une trace » mais D17 en change QUATRE** (`powerup_nuke` + les trois `clone_*` qui placent un Nuke : premier diff aux lignes 21/651/951/751 des `.trace`, frames 20/650/950/750+1) — c'est la seule chose qui échouait, pré-bless exactement conforme à la prédiction ; preuve rejouée indépendamment : dumps de l'ancien code (`706fb8e`, worktree détaché + target amorcé dédié) vs nouveau, `scripts/trace-diff.py --ignore "Currency,FrameEvents<run::currency::CurrencyEvent>,FrameEvents<game::economy::PointsCredit>,Run"` → 4/4 IDENTIQUES, sans ignore les diffs bruts ne montrent que les crédits Nuke et `CurrencyEvent` à partir de la frame du ramassage ; quatre traces bénies (`b8c9cae`, hashes vérifiés) ; re-vérif verte sur l'état béni (tests des onze crates, suite stricte avec budgets au calme : `bench_bullets` 105,0 fps, `bench_horde` 68,2, `bots_four_mixed` 108,5, `idle` 153,5 — sous contention les planchers avaient trempé, rejoués machine calme), lint des deux jeux, fmt, scripts, `make gen` sans modification ; **p2p N=2 rejoué** (crates de simulation touchées) via la recette README §4 (headless + matchbox Docker) : 599 lignes par trace, `cmp` identique — la cible `make test_multiplayer` est inopérante pour zombies (pas de règle `_matchbox`, préexistant, hors lot) ; commit `94229eb` conservé (attentes Currency +400 des `clone_*`). Dette ouverte du lot : le joueur **à terre** est crédité par le nuke (choix assumé par l'agent, à confirmer un jour) |
| 2026-10-03 | m0-v8 « carte avant_poste » (voie V6, assets seulement : registre `assets.yaml` de 11 licences, `(path: "maps", kind: "Map")` dans `game.ron`, `maps/avant_poste.ldtk` LDtk 9 salles (Depart→Couloir/Cuisine/Chambre→Poste 4 fenêtres→Armurerie/Cave→Cellier/Jug, portes 1000→1500, 4 `PlayerSpawn`, 5 `ZombieSpawn`, 4 `WeaponLocation`, 1 `SodaLocation`, 3 `CrateLocation`), scénario de démo `avant_poste_demo` (3600 frames, machine_gun, victoire à l'entrée en vague 5, livré sans trace) | `m0-v8-carte-zombies` | Claude (session locale, voie V6) | mergée le 2026-10-03 (`d45431f`) ; trace `avant_poste_demo` bénie par l'orchestrateur (`8fe873e`, nouvelle trace : pas de preuve requise) ; vérifiée sur l'état fusionné : tests des onze crates verts, suite stricte des 62 scénarios verte sans trace existante modifiée, benchs au calme (`bench_bullets` 127,0 fps, `bench_horde` 74,0, `bots_four_mixed` 108,6, `idle` 154,2), lint des deux jeux (zombies voit les 3 cartes), fmt, scripts, `make gen` sans modification ; acceptation : `--map` ajouté à `alacod-sim` (`cd6445a`), `start_map` basculé sur `maps/avant_poste.ldtk` (`0942abb`) — sans effet sur les traces (le runner lit `scenario.map`, pas `start_map` ; re-check `avant_poste_demo` vert après bascule) ; **validation 20 graines à 4 bots** sur la nouvelle carte contre la baseline test_map (mêmes graines, même lineup) : **0 desync des deux côtés**, **5 softlocks vs 8** — la carte ne régresse pas ; sur avant_poste, 3 softlocks d'une **signature nouvelle** (gel de spawn : `select_valid_spawners` vide → la vague reste en phase `Spawning` indéfiniment — les spawners ne s'activent qu'entre 150 et 600 unités d'un joueur, les salles éloignées jamais tant que les bots restent à Depart) et 2 classiques D20 (zombies coincés) ; le critère « sans softlock » hérite de D20 : non bloquant, **m0-v7 phase 2** (en cours chez Codex) doit couvrir aussi le gel de spawn — re-validation avant_poste prévue après ce merge ; dettes V6 signalées à William : `map_preview` cassé (préexistant, engine), licences à trancher (Pixel_Poem, Zombie2.png, Slash_strip3.png), `gen_map.py` non versionné, portes/perks non exercés en partie |
| 2026-10-03 | m0-dettes-overnight « lot de sept dettes » (D23 : `make videos SCENARIO=a,b` accepte les virgules + `DRY_RUN=1` ; D24 : `active_target_dir` — `CACHEDIR.TAG` ancêtre de l'exécutable de test, repli `CARGO_TARGET_DIR` relatif au workspace, même règle dans `scenario-metrics.py` ; D21 : `outcome_event` victoire/défaite/abandon à `RunStep::Ended`, `powerup_pickup` via `FrameEvents<PowerUpPickedUp>`, armes murales exclues du « tombé au sol » (`price.is_none()`) ; D22 : `player_source_text` pur — joueur mort → sources vides ; D19 : `RecordedSettings` construit avec l'`InputRecorder` (le runner l'insère, plus de patch de `recorded`) ; D13 : `plan_run_request` pur + `run_summary` partagé, `LocalLobbyHold` bloque `setup_ggrs_local`, l'écran de fin libère et montre le résumé, première partie inchangée ; D3 : kind `SpriteSheet` (`sprites/sprites.ron` des deux jeux, 8 id = ancienne table en dur), `GlobalAsset` piloté par le registre, lint `lint_sprite_sheets` (4 règles, 4 fixtures), sprites rangés par entité sous `sprites/{characters,weapons,enemies,effects}/`, `Weapons.png` identique au blob près (`1253024…`), configs de contenu conservées sous `ZombieShooter/Sprites/**`, `rifle` garde le sprite du fusil à pompe — dette documentée, le hash de `Weapon` inclut `sprite_config`) | `m0-dettes-overnight` | Claude (cloud) | mergée le 2026-10-03 (`8a92d10`) ; vérifiée par l'orchestrateur sur l'état fusionné : 62 scénarios verts sans trace modifiée, 304 tests des onze crates, lint des deux jeux, fmt, scripts (4 avertissements préexistants), `make gen` sans modification, bench strict vert (bench_bullets 86,7 fps, bench_horde 49,4, bots_four_mixed 72,6, idle 145,1 — planchers à la moitié de la référence m0-v8 passés ; machine partagée : fleet SITL, orca, daemon codex, `idle` à 94 % de la référence), p2p deux clients identiques (599 lignes, sha256 `55ec099d…`). Non fait : D13 jamais joué avec une fenêtre (couvert par les tests unitaires et le test d'intégration `run.rs`) ; page de revue non ouverte dans un navigateur ; bench/p2p impossibles dans le cloud (rejoués ici) |
| 2026-10-03 | m0-v9 « client allumette dans le jeu natif » (D12 : nouveau module `jjrs/allumette.rs` hors simulation — paire Ed25519 éphémère (`getrandom` 32 octets), `POST /auth/challenge` → signature `verify_strict` de la chaîne challenge telle quelle → `POST /auth/login` → JWT, lobby `zombies` (`--lobby <uuid>` = join direct, sinon découverte du premier `Waiting` puis join, sinon création ; le créateur sonde `GET /lobbies` jusqu'au complet, 60 s max, car la topologie marque `InProgress` dès que le propriétaire connecte), `GET /ice-servers` (première entrée, repli STUN par défaut), WebSocket `ws(s)://hôte/<JWT>` ; `--allumette <url>` (`-a`, `conflicts_with` `--matchbox`), `start_matchbox_socket` consomme `Option<Res<AllumetteConfig>>` (natif, cfg-gated), chemin `--matchbox` historique inchangé ; deps natif uniquement : `ureq` 2.12.1, `ed25519-dalek` 2.2.0, `base64` 0.21.7 (même version que le serveur), `getrandom` 0.2.17 (pas de `rand::` : interdit par check-forbidden) ; 4 tests unitaires) | `m0-v9-allumette-client` | Claude (session locale, voie b0) | mergée le 2026-10-03 (`2cb0897`) ; vérifiée par l'orchestrateur sur l'état fusionné : 62 scénarios verts **sans trace modifiée** (378,6 s), suite des onze crates 0 échec (262 tests hors scénarios), `game --lib` 33 tests (dont les 4 allumette), lint des deux jeux, fmt, scripts (4 occurrences préexistantes), `make gen` sans modification ; **recette `--matchbox` rejouée** (matchbox Docker + 2 clients) : traces identiques (599 lignes, 23 850 octets) ; **recette réelle allumette rejouée** (serveur frais + 2 clients décalés de 6 s) : traces identiques, **même sha256 `55ec099d…` que la recette matchbox** — le chemin allumette ne change rien au déterminisme ; log serveur « `Owner connected — lobby marked InProgress` ». Note d'hygiène : branche livrée basée sur `a39956a` (avant le lot m0-dettes de la nuit) — merge 3-way, seul conflit `dettes.md` (D12/D13, résolu en gardant les deux statuts) ; règle rappelée à l'agent : merger `origin/main` avant de livrer. Dettes laissées (rapport) : flux HTTP bloquant dans `OnEnter(LobbyOnline)` (à rendre asynchrone pour une UI de lobby), course de création documentée (limite v1), commentaire de `docker-compose.ci.yaml` devenu faux (profil allumette pas encore activé en CI — reste de D12), lobbies `Waiting` orphelins côté serveur (hors périmètre) |
| 2026-10-03 | m0-v10 « dettes légères » (reste de D12 : en-tête de `docker-compose.ci.yaml` corrigé (`signaling` = recette README §4, `allumette` = CI de nuit), `scripts/nightly.sh` STEP C basculé sur le profil allumette (`--allumette http://127.0.0.1:3537`, `ALLUMETTE_DIR` explicite ou `../allumette` sinon SKIPPED propre, restart du conteneur entre deux comptes de joueurs — lobbies frais —, créateur d'abord puis rejoigneurs ~6 s, comparaison des traces conservée) ; D5 : `HitCount` (`rollback_and_trace_no_checksum`) et `EnemyAiConfig::stationary` (exclu du `Hash` manuel) documentés comme exclusions assumées du checksum dans `conventions.md` §10, ligne D5 fermée) | `m0-v10-dettes-legeres` | Claude (session locale, voie b0) | mergée le 2026-10-03 (`7650f86`) ; **zéro ligne Rust : aucune trace ne peut changer, aucun bless** ; base à jour `2db0b23` (règle d'hygiène respectée) ; recette rejouée par l'orchestrateur via le profil docker allumette : **N=2** traces identiques sha256 `55ec099d…` (référence m0-v9/m0-dettes) et **N=4** (chemin CI jamais exercé) quatre traces identiques (`2cb8c7e8…`) — créateur + 3 rejoigneurs dos à dos passent ; `bash -n`, `make format`, `check_forbidden` (4 occurrences préexistantes), `check_rollback_registration`, `make gen GAME=zombies` sans modification ; D12 entièrement fermée (client m0-v9, CI m0-v10) |
| 2026-10-03 | m1-v1a « projectiles composables » (T1.1, voie V1a : les six modificateurs `Bounce`/`Pierce`/`Size`/`Lifetime`/`Homing`/`Gravity` implémentés sur le cycle de vie du projectile (`crates/combat/src/projectile.rs` : `ProjectileSpec`, `ExpireAction::Spawn(Pattern)`, composant rollback `Projectile`, `FrameEvents<ProjectileHit>`), `on_hit`/`on_expire` par les actions de la crate `effects` (explosion = projectile à durée nulle + pattern), armes de démonstration testbed + grenade, attentes `BulletCount`/`HitsAtLeast` revendiquées (T1.15), conventions §16) | `m1-v1a-projectiles-composables` | Claude (cloud, c1) | mergée le 2026-10-03 (`ea1b222`) ; branche récupérée en **bundle git** (49 kB : push 403 du proxy cloud — leçon : les sessions cloud livrent en bundle via Syncthing) ; vérifiée par l'orchestrateur sur l'état fusionné : 79 scénarios verts — 62 existants à traces inchangées (le bless les réécrit à l'identique), 17 traces testbed bénies (`6a4c9f`, scénarios générés nouveaux : pas de preuve requise) ; **323 tests / 0 échec** des onze crates (34 cibles), lint des deux jeux, fmt, scripts (4 avertissements préexistants), `make gen` sans modification ; **bench strict vert** sur machine partagée (`bench_bullets` 84,3 fps, `bench_horde` 46,7, `bots_four_mixed` 68,6, `idle` 161,9 — planchers à la moitié de la référence m0-v8 passés ; `bench_bullets` nettement sous sa référence calme de m0-v8 (127,0) : coût des projectiles et/ou contention, à revoir machine calme) ; **p2p N=2 rejoué** : 599 lignes par trace, `cmp` identique, sha256 `55ec099d…` identique à la référence m0-v9 — sans input personne ne tire, les nouveaux composants ne changent pas le checksum par défaut ; dettes D26 (10 scénarios testbed en doublon, ~80 s) et D27 (dispersion ±0,5 rad ignore `spread` du RON, préexistant) ; questions ouvertes du rapport conservées pour T1.2 (Homing sans portée, table projectile/arme pour `Shoot(pattern)`) |
| 2026-10-03 | m1-v1e « mode Floors » (T1.8, voie V1e : étages dans le `GgrsSchedule` — `run::floors` (`FloorState`, `RunMode::Floors` ajouté en dernier variant, `FloorWorld(emplacement)`, override `floors` du scénario), mondes LDtk chargés d'avance et superposés (`map_ldtk::game::floors`, 626 lignes), ressource `rollback_and_trace_resource_neutral` (valeur par défaut = contribution 0 au checksum : traces existantes inchangées par construction), portail au barycentre des `PlayerSpawn` quand il ne reste aucun ennemi, correctif de la grille de murs (signature de reconstruction = nombre + somme des `GgrsNetId`), attente `FloorIndex` (T1.15), conventions §17, séquence `trois_niveaux` (3 cartes testbed), scénario `portal_next_floor` (2 600 frames)) | `m1-v1e-mode-floors` | Claude (cloud, c2) | mergée le 2026-10-03 (`9e85e79`) ; branche récupérée en bundle ; re-merge de `origin/main` par l'orchestrateur (conflits prévus résolus en `51f5cb0` : CLAUDE.md liste des attentes, conventions §16/§17) ; vérifiée sur l'état fusionné : **80 scénarios verts** — 79 existants à traces inchangées, `portal_next_floor` re-béni à l'identique (la trace écrite par l'agent est canonique) ; **338 tests / 0 échec / 7 ignorés** des crates ; lint des deux jeux, fmt, scripts (4 avertissements préexistants), `make gen` sans modification ; **`alacod-sim --floors` 5 graines : 0 desync, 0 mort**, niveaux 8 à 15 atteints — l'invariant `joueur_hors_mur` (déclaré par le scénario, mécanisme préexistant d'`invariants.rs`) rapporte des chevauchements de murs entre bots qui se poussent dans la géométrie étroite des étages : réserve honnête du rapport, à couvrir avec la séparation de b1 (m0-v7 p2) ; limite documentée : mondes superposés = pas de `ZombieSpawn` de vagues en mode Floors ; **bench strict vert machine calme** (`bench_bullets` 152,0 fps, `bench_horde` 81,5, `bots_four_mixed` 124,0, `idle` 120,4 — le 84,3 de m1-v1a était bien de la contention, les projectiles coûtent peu) ; **p2p N=2 rejoué** : 599 lignes, `cmp` identique, **sha256 `55ec099d…` identique à la référence m0-v9** — preuve au bit près que le mode Floors ne change pas le checksum d'une partie Waves classique |
| 2026-10-03 | m0-v11 « F5 : équilibrage par nombre de joueurs » (D25, conventions §18 : `content::expr::NumOrExpr` (`Integer` / `Literal` / `Expression`, un échec de parse = échec du chargement) sur les vagues (`wave_config.ron`, tous les champs numériques), les prix (`economy.ron`, `perks.ron` ; prix d'armes murales et de portes = Int LDtk, hors périmètre documenté) et la santé (`base_health.max` de tout personnage) ; résolution **une fois** sur `OnEnter(GameLoading)` par `game::balance::resolve_balance_system` → ressource `ResolvedBalance` hors rollback, nombre de joueurs = `max_player` ggrs en ligne, `PlayersCount` sinon ; aucune expression dans l'état rollback ; preuve `zombie_scaled` (`"10.0 + (players - 1) * 40.0"` : 50 PV à 2, 130 à 4, carte `exemples/zombie_scaled.ldtk`) par `equilibrage_joueurs_duo` (7 balles tuent, `EntityCount(Enemy)=0` à f220) et `_quad` (11 balles, 88 < 130, `EntityCount=1`) ; régression trouvée et corrigée par b0 : la résolution réveillait les vagues du testbed sans dossier `Wave` (25 traces testbed + `make gen` rouges) → garde `wave_config.is_none()` restaurée dans `waves/systems.rs`) | `m0-v11-f5-equilibrage-joueurs` | Claude (session locale, voie b0) | mergée le 2026-10-03 (`562729c`) ; base `5d49d9e` à jour (règle d'hygiène respectée, un conflit `map_ldtk/game/local.rs` avec m1-v1e résolu en gardant les deux) ; vérifiée par l'orchestrateur sur l'état fusionné dans le checkout principal : **82 scénarios verts, 80 traces existantes inchangées**, 2 nouvelles bénies (`215e70a`), 369 tests des quatorze crates (hors suite) / 0 échec, lint des deux jeux, fmt, `check-forbidden` 4 occurrences préexistantes, `check-rollback-registration` OK, `make gen GAME=zombies` sans modification ; **p2p N=2 : traces identiques, sha256 `55ec099d…` identique à la référence m0-v9** (la résolution au chargement ne change rien au checksum) ; bench non strict (sims de b1 en cours : `bench_bullets` 100,7 / `bench_horde` 55,8 sous charge, au-dessus des planchers 70/38) ; worktree + target purgés (34 Go), branche distante supprimée |
| 2026-10-03 | hors tâche (correctif interactif sur le Mac de William : `cargo run -p zombies` sans `--players` démarre à un joueur local (`args/mod.rs`, `max_player` valait 0) ; sons de feedback (`feedback.rs`, `PostUpdate` : un son de reload au passage `None → Some`, un son par (tireur, frame de création) dédoublonné après rollback du synctest, tir coupé en fondu à `SHOT_SOUND_MAX` 250 ms car `machine-gun.ogg` dure 17,7 s) ; **desync du dash** : `CursorPosition` (visée, non rollbackée, lue la même frame par `system_weapon_position`) était écrite après les deux `continue` du dash dans `apply_inputs`, donc figée pendant un dash et relue d'une frame plus récente par une frame resimulée → rotation de l'arme au checksum divergente, synctest gelé ; correctif `ce99807` : réécrite à chaque frame avant tout `continue`, aucun nouveau type rollback (le `no_checksum` intermédiaire `d79a868` est annulé) ; scénario de régression `dash_aim_change` ; `fuzz_inputs.rs` (test `#[ignore]`, `ALACOD_FUZZ`, `ALACOD_FUZZ_SPARSE`)) | `fix-local-launch-sounds` | Claude Sonnet 5.5 (interactif, hors protocole) | mergée le 2026-10-03 (`4ec8de0`) ; vérifiée par l'orchestrateur sur l'état fusionné (main `a56d97b`) : **83 scénarios verts, 0 trace existante modifiée** (seule `dash_aim_change` ajoutée), 369 tests des crates / 0 échec / 9 ignorés (fuzz compris), lint des deux jeux, fmt, `check-forbidden` 4 préexistantes (non exécutable sur Mac, bash 3.2), `check-rollback-registration` OK, exemples racine compilés ; **p2p N=2 traces identiques, sha256 `55ec099d…` inchangé** ; **preuve de régression rejouée** : visée remise après les `continue` → `dash_aim_change` s'arrête à la frame 32 « synctest mismatch (frames [30]) », rétabli → vert ; **fuzz** 8 graines denses + 8 clairsemées (3600 frames) : 0 divergence ; bench sous charge 107,7/62,5 (planchers 70/38), bench strict au calme non fait ; gameplay voulu : l'arme suit la visée pendant un dash ; dettes D30 `[profile.dev]` (15 fps en `cargo run`), D31 panic à la fermeture de fenêtre (`camera/mod.rs` `.single().unwrap()` sur `Window`), D32 échantillon de tir unique (V6), D33 `check-forbidden.sh` sous bash 3.2 ; règle ajoutée à la checklist CLAUDE.md (composant dérivé non rollbacké réécrit chaque frame avant tout retour anticipé) |
| 2026-10-03 | m0-v7 **phase 2** « les bots finissent le clone » (D20, critère M0 §9.8 : profils `chasseur`/`acheteur` (`crates/bots/src/hunter.rs`, `navigation.rs` : flow field 8 px vers postes de tir accessibles, portes abordables, réparations, réanimation, relâchement du tir Manual/Shotgun/Burst via `WeaponState.is_firing`), caches dérivés dans `ReadInputs` (pas d'état rollback), `alacod-sim` : quatre acheteurs par défaut, `--progress`, dumps de plafond ; **secours de spawn** après 600 frames sans spawner dans la plage (`WaveState.spawn_fallback`, hash conditionnel) ; **guidage de récupération** des zombies au waypoint bloqué/hors case après 600 frames sans kill ni spawn (`pathing.rs`) ; ouverture des portes pour un zombie éveillé inaccessible ; glissement diagonal du joueur dans un coin de mur corrigé (`input.rs`) ; **snap de la salle de spawn sur la grille de tuiles** (`basic.rs` `snap_to_tile_grid` : le résidu pixel mod 16 hérité par toute la carte laissait 367 cellules physiques absentes de la nav, graines 16/17) ; 7 scénarios ré-étalonnés en un commit `07c2862` (downed_all_lose, run_lose_summary, two_players_idle, shoot_around, movement_melee +1/+1, clone_quad soldes, **clone_duo : visée des deux joueurs ré-enregistrée depuis f1000** — zombie le plus proche ≤ 400 px, 2 frames de tir / 15 — victoire vague 5 f4668 conservée) ; tests `hunter_doors`, `spawn_stall`, bots ; conventions §24) | `m0-v7-phase2-bots-finisent-le-clone` | Codex puis Claude (session locale b1) | mergée le 2026-10-03 (`716df9e`, base `23a43fc`) ; **20/20 graines en victoire vague 5 (f6877-f8624), 0 mort, 0 desync, 0 plafond** sur la tête fusionnée ; **bless de 83 traces** (toutes changent dès la frame 0 par le snap) avec preuve acceptée : main+snap+7 .ron = 77/82 identiques à la tête, les 5 écarts (idle, bench_horde, downed_all_lose, run_lose_summary, two_players_idle, à partir de f1141 = dernier spawn + 600) disparaissent octet pour octet avec le seuil de récupération à `u32::MAX` ; vérifiée par l'orchestrateur sur l'état fusionné (avec fix-local-launch-sounds) : 83 verts après bless, **82/82 traces identiques (sha256) à celles de b1**, 396 tests des crates / 0 échec / 9 ignorés, lint, fmt, `check-forbidden` 4, rollback-registration OK, exemples racine, `make gen` sans modification ; **p2p N=2 traces identiques, nouvelle référence sha256 `39654b07…`** (l'ancienne `55ec099d…` est caduque : le monde a bougé) ; bench non strict (charge) ; §19 de b1 renuméroté §24 ; critère **200 graines** lancé après merge (résultat au journal suivant) ; réserve : `joueur_hors_mur` entre bots (m1-v1e) à revoir avec les bots v1 |
| 2026-10-04 | m1-v1a-patterns « patterns, émetteurs et tir ennemi » (T1.2, voie V1a, B5 v1 : composant rollback `Emitter` (`crates/combat/src/emitter.rs`, avancée pure `tick` testée : `Ring.every` en tâche de fond dans une `Sequence`, `Telegraph`/`Wait`, `Scatter` via le flux RNG `patterns`, `Named` résolu par `PatternLibrary` hors rollback), helper d'apparition unique `weapons::spawn_bullet` (joueurs, enfants de pattern, émetteurs), **`rollback_and_trace_neutral::<C>`** (utils : `ChecksumPart(0)` sans porteur — **piège de parité mesuré** : un type vide ajoute `hash(0u64)` en XOR, un seul déplace toutes les traces, deux les laissent intactes ; explique `HitCount` hors checksum et `Projectile` de T1.1), tir ennemi `EnemyAiConfig.ranged { weapon, pattern, range, cooldown_frames }` (`RangedAttackState` neutre, arme dans `WeaponInventory`, immobile pendant le tir, arrêt si cible morte/à terre/hors portée), kind `Pattern` (`patterns/<nom>.ron`) + lint (4 fixtures), correctif latent T1.1 : `direction_of` ramène dans ±2π (le CORDIC paniquait au 8e rayon d'une couronne vers le haut), testbed `fireball_gun`/`turret`/`archer`/`arena_tir.ldtk`, scénarios `enemy_ring`, `enemy_ring_quad` (mêmes `BulletCount` aux mêmes frames à 1 et 4 joueurs), `enemy_telegraph`, conventions §20) | `m1-v1a-patterns-tir-ennemi` | Claude (session locale b0) | mergée le 2026-10-04 (merge local sur `7e8f541`, conflits `pathing.rs` et `conventions.md` résolus en gardant les deux) ; vérifiée par l'orchestrateur sur l'état fusionné : **87 scénarios verts, 83 traces existantes inchangées**, 4 nouvelles bénies (`43b0ffd`, dont le généré `weapon_fireball_gun`), 410 tests des crates / 0 échec / 9 ignorés, lint (testbed : 9 personnages, 12 armes, 8 cartes), fmt, `check-forbidden` 4, rollback-registration OK, exemples racine, `make gen` zombies 10/10 et testbed 18/18 sans modification ; **p2p N=2 traces identiques, sha256 `39654b07…` identique à la référence m0-v7** ; bench strict non fait (200 graines en cours) ; dette D34 (CORDIC imprécis près de 2π) ; `ai.ranged` sera remplacé par `Shoot{…}` dans T1.4 |
| 2026-10-04 | m1-v1d-terrain-destructible « cavernes et terrain destructible » (T1.0b + T1.6, voie V1d, chantier E3 : crate `world` (`CellKind { Floor, Wall, Rock }`, `CellGrid` ressource rollback neutre en cases de 16, `Destructible`), `RollbackSystemSet::World` entre `Projectiles` et `CollisionDamage`, `sim_core::add_frame_events_neutral` ; kind `Cave` (`caves/<id>.ron` + gabarit LDtk) et `MapGenerationMode::Cave` (automate cellulaire seedé, connexité testée sur 1 000 graines) ; `effects::Action::DestroyTerrain { radius }` en `on_hit`/`on_expire` d'un projectile sur un mur, `FrameEvents<ProjectileWallHit>`, colliders et flow field reconstruits (`FlowFieldCache::reload_walls`) sans desync en synctest ; attente `CellState(x, y, kind, at_frame)` ; piège trouvé et documenté : `#[derive(Hash)]` d'un enum à une variante n'écrit pas le discriminant, `ExpireAction` a un `Hash` manuel qui préserve la trace de `weapon_grenade` ; scénarios `explode_wall`, `bench_cave` (86 destructions, 62,7 fps sous charge), armes testbed `foreuse` et `grenade_creuse`, caverne `Floors` du testbed ; conventions §21, checklist CLAUDE.md) | vérifiée sur l'état fusionné `4cd0b46` : 91 scénarios, 87 traces inchangées, 4 bénies `2d6fca7` ; 426 tests de crates ; lint, fmt, scripts, `make gen` des deux jeux sans modification, exemples ; p2p N=2 sha256 `39654b07…` identique à la référence ; bench strict au calme non fait ; dettes D35, D36. Merge `4cd0b46` (b1). |
| 2026-10-04 | m1-v1d-surfaces « surfaces v1 : tags de cellules et vitesse » (T1.7, voie V1d, chantier E4 v1 : ressource `world::SurfaceGrid` creuse (`BTreeMap<(i32, i32), u8>`, cases de 16, rollback neutre : vide = 0), `SurfaceId` = valeur IntGrid, `SurfaceTable` hors rollback depuis le registre ; kind `Surface` (`intgrid_value`, `tags`, facteurs abstraits `move_speed`/`acceleration` traduits selon le personnage) et lint ; couche LDtk optionnelle `Surfaces` lue dans `spawn_level_walls` (chargement et `Floors`) ; système dans `RollbackSystemSet::Input` avant `apply_inputs` qui compare les modificateurs de source `surface` présents aux voulus et ne touche à rien hors surface (facteur 1 = rien), `Flying` ignoré ; `CellState` étendue (`surface`, `kind` optionnel) ; contenu testbed `eau` ×0,5, `sable` ×0,8, `glace` accélération ×0,2, carte `surfaces.ldtk` ; scénarios `surface_walk` (eau 1,25 px/frame, sable 2,0, sol 2,5), `surface_none`, `surface_ice` (+26 px de glissade contre 7,5), `surface_enemy` (contact f543 dans l'eau contre f338) ; conventions §26) | vérifiée sur l'état fusionné `c13c217` : 95 scénarios, 91 traces inchangées, 4 bénies ; 432 tests de crates ; lint, fmt, scripts, `make gen` des deux jeux sans modification, exemples ; p2p N=2 sha256 `39654b07…` identique à la référence ; bench strict au calme non fait (A/B de b1 : le système ne pèse pas sur `bench_horde`). Merge `c13c217` (b1). |
| 2026-10-04 | m1-v1c-behaviors-composables « behaviors composables v1 » (T1.4, voie V1c, chantier D1 : kind `Behavior` (`Chase { profile }`, `KeepDistance`, `Strafe`, `Charge { telegraph }`, `Shoot { weapon, pattern, range, cooldown_frames }`, `Melee`, `Flee`, `Wander`), sélection par priorité dans `BehaviorRuntime` (composant neutre : les zombies n'en ont pas), `Perception { sight }`, `Targeting::Nearest { ignore }`, `ai.ranged` remplacé par `Shoot`, zombies de `games/zombies` réécrits en RON avec traces identiques (preuve : 0 trace différente), code mort retiré (`enemy_movement_system`, `enemy_stun_recovery_system`, `MonsterState::{Stunned, Breaching, Fleeing}`), attentes `EnemyState`, `EnemyDistance`, continues `EnemyContactBefore`, `EnemyNeverInWall`, une arène par behavior (`arena_ia_keep/charge/flee/wander`), scénarios `enemy_keep_distance`, `enemy_charge`, `enemy_flee`, `enemy_wander` ; conventions §22, D28) | vérifiée groupée avec T1.5 et T1.14 sur l'état fusionné `5d8426e` (voir m1-v3-bots-v1) ; alacod-sim 20 graines identiques base/branche (b0, avant le snap). Merge `12f050c` (b0). |
| 2026-10-04 | m1-v1c-variantes « variantes et élites » (T1.5, voie V1c, chantier D2 : `variants: Some((chance, table { weight, modifiers, tags, skin }))` dans `CharacterConfig`, `RollbackRng` local de graine `fnv1a("variants") ^ run_seed ^ net_id` (le `GgrsNetId` est alloué dans `spawn_enemy` et passé à `create_character`, nouveau paramètre), composant rollback neutre `Variant(nom)`, modificateurs `Named("variant:<nom>")`, tags en union, base `MaxHealth` = santé F5 posée seulement pour les personnages à variantes, champ LDtk `variant` (transite par `CharacterSpawnConfig`/`from.rs`/`to.rs`), `RunSeed` lu depuis `MapGenerationConfig.seed` pour les personnages de carte, attente `EnemyVariant`, `Stat` avec `entity` optionnel, événement `variant_spawn`, lint (`MoveSpeed` sur IA refusé, `serde_json` dans `content` pour les `.ldtk`), testbed `grunt`/`grunt_plain`, `arena_variantes`, scénarios `variant_fast` (vitesse 90 = ×1,5, à f80 distance 58 contre 98), `variant_none`, `variant_elite` (santé 120 = ×2), `variant_draw` (graine 123456 : aucune ; 777 : `blinde`) — fast/none/elite ont la même trace, voulu ; conventions §25 ; zombies sans variante) | vérifiée groupée sur `5d8426e`. Merge `e419b96` (b0). |
| 2026-10-04 | m1-v3-bots-v1 « bots v1 » (T1.14, voie V3 : `BotView.projectiles` (toute `Bullet` d'une autre équipe, ≤ 320 px, ≤ 16, triées par `GgrsNetId`), `bots::dodge` pur en `Fixed` (point d'approche minimale sur 30 frames, marge 8 px, somme des perpendiculaires), `prudent` v1 esquive, gère son arme (recharge, changement, détente relâchée) et freine à moins de 48 px du portail tant que sa vitesse dépasse 30 (sans quoi il orbitait autour du rayon de 24), `fonceur`/`chasseur`/`acheteur` inchangés ; `alacod-sim --until-floor <n>` (exige `--floors`), soft-lock `Floors` (1 200 frames sans portail ni kill), `SimResult.floor_frames`/`damage_taken`/`dodges`, `scenario-metrics.py` ; `floor_d` (copie de `floor_b` avec le follower hors du mur) et `trois_niveaux` = a, d, c ; scénarios `bot_prudent_dodge` (8 dégâts, santé pleine à f599) contre `bot_prudent_nododge` (56 dégâts, santé 44 ; jumeau enregistré avec l'esquive désactivée localement, inputs figés en `Scripted`, diff et commande dans le rapport), `bot_floors_three` (portails f135/f388/f796) ; critère : 20/20 graines finissent trois niveaux (420 à 1 452 frames), 0 desync, 0 softlock, 0 mort ; `bots_four_mixed` et `clone_quad` utilisent `prudent` et gardent leurs traces (la fiche disait le contraire) ; conventions §24 v1, §17 mis à jour) | **vérification groupée T1.4 + T1.5 + T1.14** sur l'état fusionné `5d8426e` : 106 scénarios, 95 traces inchangées, 11 bénies ; 472 tests de crates ; lint des deux jeux, fmt, scripts, `make gen` des deux jeux sans modification, exemples ; p2p N=2 sha256 `39654b07…` identique à la référence ; bench strict au calme non fait ; dettes D37, D38. Merge `5d8426e` (b1). |
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
