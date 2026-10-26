# Plan de l'atelier de réglage (chantier K7, lots AT) — rédigé le 2026-10-26

**État au 2026-10-26 : AT-0 en cours (branche `at-0-consolidation`) ; les autres lots attendent AT-0.**

| Lot | Contenu court | Prérequis | Estimation |
|---|---|---|---|
| AT-0 | Consolidation de la séance : commit, revue, suite complète, binaire `alacod` installé, petits défauts relevés — **en cours : fonctionnalités faites, suite complète à jouer** | — | 0,5 j |
| AT-1 | Commandes de confort : `list`, triches de dev, spawn en formation, gel des ennemis, vues de debug par la CLI | AT-0 | 1 j |
| AT-2 | Mesures dans la simulation : `DevMetrics`, fenêtres de mesure, `alacod remote metrics` | AT-0 | 1,5 j |
| AT-3 | Widgets de dev à l'écran (sources HUD `dev_*`, F8, `alacod remote widget`) | AT-2 | 1,5 j |
| AT-4 | Surimpressions de réglage : cône de dispersion, portée, traînée et dash, marqueurs de coups | AT-0 | 1 j |
| AT-5 | Ralenti : `alacod remote speed 0.25` | AT-0 | 0,5 j |
| AT-6 | Banc d'essai sans fenêtre : `alacod bench weapon|enemy|move [--vs REF]` | AT-2 | 2 j |
| AT-7 | Fiches statiques : `alacod table Weapon|Character [--diff REF]` (DPS théorique, TTK calculé) | AT-0 | 0,5 j |
| AT-8 | De la séance au test : `remote save --metrics`, `alacod log` des réglages | AT-2, AT-6 | 1 j |
| AT-9 | Séance de William (voix) : armes, ennemis, déplacement | AT-1 à AT-5 | 0,5 j + William |

Total ≈ **10 j** d'agents `[INFERENCE : estimation, rien n'est mesuré]`.

## 1. Objectif et critère de fin

William joue dans le bac à sable (`alacod sandbox <jeu> --detach`, arène vide, mode éditeur) et règle par la voix les **personnages**, les
**ennemis** et les **armes** de n'importe quel jeu alacod, clones comme jeux créés par `alacod new`. L'agent qui l'écoute ne lit ni n'écrit de
fichier à la main : il passe par `alacod` (CLI) pour **mesurer, changer, re-mesurer et comparer**, et William voit les chiffres à l'écran.

Critère de fin (vérifié en AT-9) :
1. Pour chacune des trois familles, la boucle « mesure → `alacod set` → mesure → comparaison » se fait en **une phrase de William et au plus
   trois commandes `alacod`**, chiffres avant/après lus à voix haute et affichés en jeu.
2. Les mêmes chiffres sortent **sans fenêtre** (`alacod bench`), déterministes, comparables entre deux références git.
3. Rien ne dépend du `Makefile` ni de `scripts/` ; tout marche sur un jeu créé par `alacod new` sans code ajouté.
4. Aucune trace de scénario existante ne bouge (suite complète, `schedule_order_diff` expliqué).

## 2. Leçons de la séance du 2026-10-24 au 2026-10-26

Ce que la séance de réglage de `zombies` a montré, et où chaque point est traité.

| # | Constat | Cause | État | Lot |
|---|---|---|---|---|
| 1 | Le jeu « se fermait tout seul » à chaque lancement | process lancé par l'outil supervisé de l'agent, tué en fin de tour ; aucun code du jeu en cause | réglé : `alacod sandbox --detach` (SB-6, §89) | — |
| 2 | Le bac à sable ouvrait la carte roguelike (fenêtres, portes, armes murales) | `sandbox.arena` de `zombies` = `exemples/test_map.ldtk` | réglé : `alacod new-arena`, `arene_vide.ldtk` dans les cinq jeux et le gabarit | — |
| 3 | Un ennemi posé par `remote spawn` était invisible (barre de vie seule) | personnage à `z = 0`, sous les couches de tuiles LDtk | réglé : `lift_characters_to_entity_layer` (`map_ldtk/src/plugins.rs`) ; objets et power-ups posés à `z = 0` vérifiés visibles (AT-0) : l'image d'un objet au sol (`sheet_monde`, `offset_z` 3,4 dans `gungeon` et `isaac`) passe au-dessus de la couche `Entities` (z = 3), un power-up est un repère gizmo et une étiquette à z = 50 ; pas de remontée à ajouter | AT-0 |
| 4 | La partie « gelait » après un coup de zombie | un agent avait fait `step` sans `resume` ; rien à l'écran ne disait « pause » | réglé côté client (`step` rend l'état d'avant) ; **pas d'indicateur de pause à l'écran** | AT-1 |
| 5 | Chaque réglage coûtait une lecture et une édition de fichier | pas de commande d'édition | réglé : `alacod get/set/undo` (SB-7) | — |
| 6 | Chaque commande `alacod` prenait 36 à 55 s | appel par `cargo run -p content --bin alacod` (vérification et édition de liens à chaque fois) | réglé : binaire installé, seule forme documentée ; `alacod self-check` dit s'il est plus ancien que les sources (code 1) | AT-0 |
| 7 | `remote screenshot` rendait la main avant d'écrire le fichier (une image périmée a été lue) | capture asynchrone (`remote.rs`, observateur `save_to_disk`) | réglé : `screenshot` attend le fichier écrit et de taille stable (`--wait`, 5 s par défaut, code 1 au-delà) | AT-0 |
| 8 | Dans l'arène ouverte, trois zombies tuent le joueur immobile ; la défaite arrête la simulation | aucun mode invulnérable, aucun soin, aucun « tout effacer » | **ouvert** | AT-1 |
| 9 | La dictée déforme les ids ; `describe` est long à lire | pas de liste courte avec recherche approchée | **ouvert** : `alacod list <Kind> [filtre]` | AT-1 |
| 10 | « Un peu moins de dispersion » a donné 0,15 → 0,12 (−20 %, au-delà du barème « un peu » = 10 à 15 %) | calcul à la main par l'agent | réglé : valeurs relatives `alacod set … x0.9`, `+10%`/`-10%`, `+=5`/`-=5` (`+5`/`-5` restent absolus) | AT-0 |
| 11 | Rien à l'écran ne dit ce qui vient de changer | `set` n'informe que la CLI | **ouvert** : bandeau « dernier réglage » | AT-3 |
| 12 | Impossible de dire si l'arme est « meilleure » après un réglage | aucun compteur de tirs, coups, dégâts par arme ; `Bullet` ne porte pas l'id de son arme | **ouvert** | AT-2, AT-3, AT-6 |
| 13 | Les preuves des agents ont laissé deux commits `editor:` (`b1aab036` et son revert) dans l'historique de William | `set --commit` joué dans le dépôt de William | **ouvert** : règle §7 (preuves sur port et dépôt jetables) | AT-0 |
| 14 | Le travail SB-6, SB-7 et le correctif de sprite est **non commité** sur `essai-vocal` ; suite complète non jouée ; le journal dit « aucune trace modifiée » sans l'avoir prouvé | lots enchaînés en séance | **ouvert** | AT-0 |

## 3. Ce qui existe (relevé du 2026-10-26)

| Brique | État | Où |
|---|---|---|
| CLI `alacod` | `lint`, `new`, `test`, `new-content`, `new-arena`, `sandbox`, `editor`, `describe`, `get`, `set`, `undo`, `remote …` ; codes de sortie 0/1/2/3/4, `--json` | `crates/content/src/bin/alacod.rs`, `remote.rs`, `edit.rs`, `script.rs`, `launch.rs`, `arena.rs` |
| BRP `alacod/*` | 16 méthodes : `state`, `input`, `step`, `pause`, `resume`, `screenshot`, `save_recording`, `restart`, `set_modifier`, `clear_modifiers`, `reload_content`, `spawn_character`, `spawn_powerup`, `spawn_item`, `sandbox_profile`, `give` ; garde `guard_hot_edit` (local, `check_distance` 0) | `crates/game/src/remote.rs` |
| Commandes de partie rejouables | `GameCommands` (`BTreeMap<frame, Vec<GameCommand>>`), appliquées à frame+1, enregistrées et rejouées par le runner | `crates/game/src/game_commands.rs`, `map_ldtk/src/game/commands.rs` |
| Scripts | `alacod remote run <fichier>` : `wait N frames`, `sleep`, `expect <chemin> <op> <v>` sur le JSON de `state` | `games/zombies/scripts/monde_test.alacod` |
| Dégâts | `FrameEvents<DamageEvent>` (six émetteurs), `HitLanded` après défenses, `HitBy`, `Death`, `HitCount` (si `counts_hits`) ; **rien ne compte les tirs, ni les coups ou dégâts par arme** | `sim_core/src/damage.rs`, `game/src/character/health/mod.rs`, `game/src/effects_v3.rs` |
| Précédent « rollback sans checksum, jamais lu par la simulation » | `RunHistory` (`rollback_and_trace_resource_no_checksum`, conventions §10) | `game/src/profile.rs` |
| HUD | 30 sources fermées (`content::ui::HUD_SOURCES`), widgets `Bar`, `Text`, `Icons`, `Map`, `Legend`, `Hearts`, `Counter` ; `HudSnapshot` existe aussi sans fenêtre | `content/src/ui.rs`, `game/src/ui/hud.rs`, `ui/hud_model.rs` |
| Vues de debug | F3 (champ de flux + colliders + vagues), F4, F5 (portées d'aggro et d'attaque), F6 (étiquettes), F7 (légende) ; clavier seulement | `character/ai_debug.rs`, `collider/debug.rs`, `legibility.rs` |
| Retours | `FeedbackLog` (devine l'arme d'un coup par l'arme active de la source ; un tick à plusieurs frames ne voit que la dernière) | `game/src/feedback_log.rs` |
| Mesure en scénario | `Metrics` du runner, gabarit d'arme `test:` (`WeaponOnTarget` contre `target`), `alacod-gen --play`, `alacod-sim` | `scenario/src/runner.rs`, `generate.rs`, `bin/alacod-sim.rs` |
| Horloge | `RemoteClock{paused, pending_frames}` ; **aucun ralenti** ; `Time<Virtual>` jamais touché (conventions §55) | `game/src/remote.rs` (`drive_clock`) |
| Stats | `StatId` (MoveSpeed, Damage, FireRate, ReloadSpeed, Range, EnemyMoveSpeed, ShotSpeed…), `Modifiers` ; `refresh_live_stats` à la modification d'un personnage ; armes portées recopiées ; **mêlée non rechargée** | `sim_core/src/stats.rs`, `character/config_asset.rs`, `weapon_assets.rs` |
| Cibles | `target` (testbed, starter), `cible` (isaac, throne, gungeon) ; **pas de cible dans `zombies`** | contenus des jeux |

## 4. Décisions

### D1 — Les chiffres viennent de la simulation, une seule source pour le jeu et le banc
- **Décision** : une ressource `DevMetrics` (nouveau module `game::dev_metrics`) **insérée seulement sur demande** (mode éditeur, ou champ
  de scénario `dev_metrics: true` pour le banc), enregistrée par `rollback_and_trace_resource_no_checksum`, **jamais lue par `GgrsSchedule`**,
  écrite par les systèmes existants au moyen d'un `Option<ResMut<DevMetrics>>` (tir, collision de balle, mêlée, `rollback_resolve_damage_events`,
  mort, déplacement, dash). Lue par le BRP (`alacod/metrics`), les widgets (AT-3) et le runner (AT-6).
- **Pourquoi** : `FeedbackLog` perd des événements quand un tick simule plusieurs frames (connu) ; un compteur de présentation diverge donc du
  banc. Une ressource rollback reste juste en synctest `d > 0` (le banc). Sans la ressource, les systèmes ne font rien de plus : aucune trace
  existante ne bouge. Précédent : `RunHistory`.
- **Attribution à l'arme** : `Bullet` n'a pas d'id d'arme, et lui ajouter un champ sous checksum déplacerait toutes les traces (CLAUDE.md,
  « Ajouter un champ à un composant rollback »). `DevMetrics` tient donc une table `balle (GgrsNetId) → (tireur, arme, mode)` remplie au tir
  et vidée à la destruction de la balle. Mêlée, projectiles, brûlure, charge : clé lue dans les `tags` de `DamageEvent` et dans l'arme de
  mêlée tenue.
- **On ne fait pas** : de compteur dans `FeedbackLog`, de champ nouveau dans `Bullet`, de `HashMap` (BTreeMap, clés `GgrsNetId`).
- **À vérifier en AT-2** `[INFERENCE]` : qu'un type `_no_checksum` absent n'écrit rien dans la trace ; sinon le dire et proposer.

### D2 — Les widgets de dev sont le HUD existant, pas Egui
- **Décision** : sources HUD nouvelles `dev_*` et un gabarit `dev_hud.ron` **fourni par l'engine** (embarqué, surchargeable par
  `ui/dev_hud.ron` du jeu), affiché par F8 ou `alacod remote widget <id|all> on|off`. Présentation seule (`PresentationPlugin`).
- **Pourquoi** : K5 a écarté Egui (« l'éditeur est une IA », `plan-engine.md` K5) ; le HUD a déjà la liaison par sources, le lint, le
  rechargement à chaud et `HudSnapshot` sans fenêtre (attente `HudText` possible). Un jeu créé par `alacod new` l'a sans rien écrire.
- **On ne fait pas** : de fenêtre d'inspecteur, de graphique temps réel (une valeur et une mini-série en texte suffisent au départ).

### D3 — Mesurer en jouant = des fenêtres de mesure nommées
- **Décision** : `alacod remote measure start [--label A]`, `measure stop`, `measure reset`, `measure list` : `DevMetrics` découpe ses
  compteurs par fenêtre (frame de début et de fin). « Compare A et B » = tableau des deux fenêtres, écarts en %.
- **Pourquoi** : la visée de William varie d'un essai à l'autre ; comparer deux fenêtres de même durée ou même nombre de tirs est la seule
  comparaison honnête en jeu. Le chiffre exact vient du banc (D4).

### D4 — Comparer deux versions = le banc sans fenêtre, déterministe
- **Décision** : `alacod bench … --vs <ref>` joue le même banc sur la tête et sur `<ref>` (worktree git, target partagé, comme
  `make compare_video`), mêmes graines, et rend un tableau avant/après. Résultats rangés dans `target/alacod/bench/<commit>-<hash du contenu>.json`.
- **Pourquoi** : la simulation est déterministe ; deux versions du contenu donnent des chiffres exacts et reproductibles, sans visée humaine.
- **Réutilise** : `scenario::generate` (gabarits `WeaponOnTarget`, `CharacterTest`), le runner, `DevMetrics`.

### D5 — Les triches de dev passent par les commandes de partie
- **Décision** : invulnérabilité, soin, recharge des munitions, « tuer tous les ennemis », gel des ennemis sont des `GameCommand`
  (indexées par frame, enregistrées, rejouées), gardées par `guard_hot_edit`. Mort par `despawn_rollback`. Gel = modificateur `EnemyMoveSpeed Set 0`
  posé sur chaque ennemi (stat existante) plus un `Stunned` sans fin si la stat ne suffit pas pour les attaques `[INFERENCE : à vérifier]`.
- **Pourquoi** : une session sauvée (`remote save`) doit rejouer à l'identique (règle de K5-2) ; écrire l'état hors commande casse le replay.

### D6 — Le ralenti pilote `RemoteClock`, jamais `Time<Virtual>`
- **Décision** : `alacod remote speed <0.1..1>` : `drive_clock` n'avance qu'une frame tous les `1/vitesse` ticks de rendu
  (`TimeUpdateStrategy::ManualDuration`, déjà utilisé pour `step`).
- **Pourquoi** : conventions §55 (GGRS avance sur `Time<Virtual>`). Le rendu reste à 60 Hz, l'animation est lisible.

### D7 — `alacod` est la seule interface, et elle est rapide
- **Décision** : le binaire installé (`cargo install --path crates/content --bin alacod --locked`) est la forme documentée partout ;
  `alacod self-check` dit si le binaire installé est plus vieux que les sources. Chaque commande a une sortie d'**une ligne** lisible à voix
  haute et `--json` pour les scripts. Les méthodes BRP restent le contrat ; la CLI n'ajoute aucune logique de jeu.

### D8 — Rien de tout ça ne déplace une trace
- **Décision** : toute addition est inerte sans `DevMetrics`, sans commande ou sans vue ouverte. Chaque lot qui touche la simulation joue la
  suite complète sans `BLESS` et `make schedule_order_diff` ; l'enregistrement de `DevMetrics` apparaît dans la liste des types rollback :
  attendu, écrit au rapport et au §90 de `docs/conventions.md`.

## 5. Besoins par famille (ce que William doit pouvoir dire)

### Armes
| On dit | Commande | Lot |
|---|---|---|
| « Quel est le DPS du fusil ? » | `alacod table Weapon shotgun` (théorique : rafale et soutenu, chargeur et rechargement compris) ; `alacod bench weapon shotgun --target target --distance 100` (mesuré) | AT-7, AT-6 |
| « Montre-moi mon DPS / ma précision » | `alacod remote widget weapon on` (DPS glissant 3 s, tirs, coups, % de touche, dégâts par coup) | AT-3 |
| « Mesure ça, puis compare » | `alacod remote measure start --label avant` … `measure stop` ; après réglage `--label apres` ; `alacod remote measure compare avant apres` | AT-2 |
| « Un peu moins de dispersion » | `alacod set Weapon machine_gun config.firing_modes.default.spread x0.9 --commit` | AT-0 |
| « Montre la dispersion et la portée » | `alacod remote overlay spread on` (cône), `overlay range on` (cercle) | AT-4 |
| « Compare avec avant mes réglages » | `alacod bench weapon machine_gun --vs HEAD~3` | AT-6 |

### Ennemis
| On dit | Commande | Lot |
|---|---|---|
| « Mets-en dix en cercle » | `alacod remote spawn zombie_1 --count 10 --ring 150` (`--line`, `--grid`) | AT-1 |
| « Fige-les » / « relâche-les » | `alacod remote freeze` / `unfreeze` | AT-1 |
| « Combien de temps pour le tuer au fusil ? » | `alacod bench enemy zombie_1 --weapon shotgun` (TTK, coups, dégâts infligés au joueur par seconde) ; widget `enemy` (TTK de la dernière mort) | AT-6, AT-3 |
| « Montre leurs portées » | `alacod remote overlay ai on` (F5 : aggro, vue, attaque) ; `overlay flow on` (F3) | AT-1 |
| « Efface tout » | `alacod remote clear [--team Enemies]` | AT-1 |

### Personnages (joueur)
| On dit | Commande | Lot |
|---|---|---|
| « Je suis trop lent » | `alacod remote modifier MoveSpeed Mul 1.1` (essai), puis `alacod set Character player movement.max_speed x1.1 --commit` | existe, AT-0 |
| « Montre ma vitesse et mon dash » | `alacod remote widget move on` (vitesse instantanée et max, temps 0 → max, distance du dernier dash, recharge) ; `overlay trail on` | AT-3, AT-4 |
| « Mesure l'accélération et le dash » | `alacod bench move player` (inputs scriptés : course, arrêt, dash ; vitesse max réelle, temps d'accélération et de freinage, distance de dash) | AT-6 |
| « Rends-moi invincible » / « soigne-moi » | `alacod remote god on` / `heal` / `refill` | AT-1 |
| « Ralentis » | `alacod remote speed 0.25` | AT-5 |

## 6. Lots

### AT-0 — Consolidation de la séance (0,5 j)
- **Fichiers** : travail non commité de `essai-vocal` (SB-6, SB-7, `lift_characters_to_entity_layer`) ; `crates/content/src/{remote,edit}.rs` ; `docs/editeur-vocal.md` ; `docs/taches.md`.
- **Livrable** :
  1. Branche `at-0-consolidation` depuis l'état courant ; commits séparés SB-6, SB-7, correctif de sprite ; revue par un agent `reviewer` ;
     suite complète de scénarios sans `BLESS`, `make check`, `make schedule_order_diff` ; corriger les lignes du journal qui affirment
     « aucune trace modifiée » sans preuve.
  2. Binaire installé (D7) : toute la doc et le bloc de prompt du §4 de `docs/editeur-vocal.md` appellent `alacod …`, jamais `cargo run`.
  3. `remote screenshot --wait [s]` : attend que le fichier existe et soit complet ; défaut activé.
  4. `alacod set` relatif : `x0.9`, `+10%`, `-10%`, `+5` (valeur numérique ou `Fixed` seulement ; l'ancienne et la nouvelle valeur affichées).
  5. Objets et power-ups posés (`spawn-item`, `spawn-powerup`) : capture regardée ; s'ils sont sous les tuiles, même remontée que les personnages.
  6. Règle §7 écrite dans le guide : preuves d'agent sur `ALACOD_REMOTE_PORT` et dépôt jetables, jamais `--commit` dans le dépôt de William.
- **Acceptation** : `main` contient le tout ; suite complète verte, traces inchangées ; `alacod remote brief` répond en moins d'une seconde.

### AT-1 — Commandes de confort (1 j)
- **Fichiers** : `game/src/remote.rs`, `game/src/game_commands.rs`, `map_ldtk/src/game/commands.rs`, `content/src/remote.rs`, `content/src/bin/alacod.rs`.
- **Livrable** :
  - `alacod list <Kind> [filtre]` : ids et noms affichés, une ligne chacun ; un id inconnu partout dans la CLI propose les trois plus proches.
  - `GameCommand` nouvelles (D5) : `SetInvulnerable{handle, on}`, `Heal{handle, amount?}`, `RefillAmmo{handle}`, `KillAll{team?}`,
    `Freeze{on}` ; CLI `god on|off`, `heal`, `refill`, `clear`, `freeze`/`unfreeze`. Rejouées par le runner (test dans `scenario/tests/game_commands.rs`).
  - `spawn … --count N --ring R | --line L | --grid C,R` (positions calculées côté CLI, une commande par personnage, ordre stable).
  - Méthode BRP `alacod/debug_view {view: colliders|flow|ai|labels|legend, on}` ; CLI `overlay <vue> on|off` (mêmes ressources que F3 à F7).
  - Indicateur « PAUSE (remote) » à l'écran quand `RemoteClock.paused` (présentation).
- **Acceptation** : chaque commande prouvée sur une partie lancée par `alacod sandbox --detach` (capture regardée) ; une session sauvée avec
  `god`, `clear` et `spawn --ring` se rejoue à l'identique ; traces inchangées.

### AT-2 — Mesures dans la simulation (1,5 j)
- **Fichiers** : `game/src/dev_metrics.rs` (nouveau), émetteurs de tir et de dégâts (`combat/src/weapons`, `melee.rs`, `character/health/mod.rs`),
  `core_sim.rs` (enregistrement, avant les cinq de clôture), `game/src/remote.rs`, `content/src/remote.rs`, `scenario` (champ `dev_metrics`).
- **Livrable** :
  - `DevMetrics` (D1) : par (tireur, arme, mode) : tirs, projectiles, coups, dégâts, morts, première et dernière frame de coup ; par cible :
    dégâts et coups reçus, frame d'apparition et de mort (TTK) ; par joueur : distance parcourue, vitesse max, dashes, dégâts reçus.
    Fenêtres nommées (D3).
  - BRP `alacod/metrics {window?}` et `alacod/measure {start|stop|reset, label?}` ; CLI `remote metrics`, `remote measure …`, `measure compare A B`.
  - Attente de scénario `Metric(chemin, op, valeur, at_frame)` (une seule, générique) pour les tests.
- **Acceptation** : dans les scénarios `weapon_*` rejoués avec `dev_metrics: true`, les coups de `DevMetrics` égalent `HitCount` de la cible ;
  toutes les traces existantes inchangées ; `schedule_order_diff` ne montre que le type ajouté ; une partie à `check_distance 2` donne les mêmes chiffres qu'à 0.

### AT-3 — Widgets de dev (1,5 j)
- **Fichiers** : `content/src/ui.rs` (sources `dev_*`), `game/src/ui/hud_model.rs`, `game/src/ui/hud.rs`, `game/src/ui/dev_hud.rs` (nouveau), gabarit embarqué `dev_hud.ron`.
- **Livrable** : sources `dev_weapon` (DPS glissant, tirs, coups, % de touche, dégâts par coup), `dev_enemy` (TTK de la dernière mort, dégâts
  infligés au joueur par seconde), `dev_move` (vitesse, max, temps d'accélération, dernier dash), `dev_window` (fenêtre en cours, durée),
  `dev_last_edit` (bandeau 4 s du dernier `alacod set` : `alacod set` appelle `alacod/notify` si une partie répond) ; DPS flottant au-dessus
  d'une cible (`WorldLabel`) ; F8 et `alacod remote widget …`.
- **Acceptation** : capture regardée avec les panneaux ouverts ; valeurs égales à `alacod remote metrics` ; `HudText` sur une source `dev_*`
  dans un scénario ; un jeu créé par `alacod new` affiche le gabarit sans fichier ajouté.

### AT-4 — Surimpressions de réglage (1 j)
- **Fichiers** : `game/src/dev_overlays.rs` (nouveau, gizmos, présentation).
- **Livrable** : cône de dispersion et cercle de portée de l'arme active ; traînée des positions du joueur (2 s) et point d'arrivée prévu du
  dash ; marqueurs de coups (où chaque balle a touché, 1 s) ; CLI `overlay spread|range|trail|hits on|off`.
- **Acceptation** : captures regardées ; après `alacod set … spread x0.5`, le cône rétrécit sans relance.

### AT-5 — Ralenti (0,5 j)
- **Fichiers** : `game/src/remote.rs` (`RemoteClock`, `drive_clock`), `content/src/remote.rs`.
- **Livrable** : `alacod remote speed <v>` (0,1 à 1) (D6) ; l'indicateur d'AT-1 affiche la vitesse.
- **Acceptation** : à 0,25, `state.frame` avance de 15 ± 1 par seconde ; à 1, comme avant ; traces inchangées.

### AT-6 — Banc d'essai sans fenêtre (2 j)
- **Fichiers** : `content/src/bench.rs` (nouveau, côté CLI), `scenario` (lecture de `DevMetrics`, gabarits), `content/src/launch.rs` (worktree pour `--vs`).
- **Livrable** :
  - `alacod bench weapon <id> [--target <perso>] [--distance D] [--frames N] [--seeds a..b]` : DPS, % de touche, dégâts par coup, TTK de la cible.
  - `alacod bench enemy <id> [--weapon <w>]` : TTK, dégâts infligés au joueur, temps de contact.
  - `alacod bench move <perso>` : vitesse max réelle, temps 0 → max et max → 0, distance et durée du dash, par inputs scriptés.
  - `--vs <ref>` (D4) ; `--json` ; une ligne de résumé lisible à voix haute ; cache par commit et contenu.
  - Arène et cible : l'arène du bac à sable du jeu ; cible = `--target`, sinon la `generate_template.target` du jeu, sinon erreur claire.
    `zombies` reçoit une cible (`target`, copie de celle du testbed).
- **Acceptation** : `alacod bench weapon shotgun` donne deux fois le même résultat ; `--vs HEAD~1` après un `alacod set … damage x2` montre
  ≈ ×2 sur les dégâts ; marche sur un jeu créé par `alacod new` (test `make test_alacod_cli`).

### AT-7 — Fiches statiques (0,5 j)
- **Fichiers** : `content/src/table.rs` (nouveau), `bin/alacod.rs`.
- **Livrable** : `alacod table Weapon|MeleeWeapon|Character [id…] [--cols …] [--diff <ref>] [--json]` : DPS théorique en rafale et soutenu
  (chargeur, cadence, plombs, rechargement), TTK calculé contre chaque ennemi du jeu, vitesses et dash des personnages ; `--diff` montre les
  écarts depuis `<ref>` (ce qu'ont changé les commits `editor:`). Sans jeu lancé.
- **Acceptation** : chiffres du DPS théorique recoupés à la main sur deux armes au rapport ; `--diff HEAD~1` montre la dispersion 0,15 → 0,12.

### AT-8 — De la séance au test (1 j)
- **Fichiers** : `game/src/recording.rs`, `content/src/remote.rs`, `content/src/bin/alacod.rs`.
- **Livrable** : `alacod remote save <nom> --metrics` écrit un scénario rejouable et des attentes `Metric` bornées (±10 % par défaut) tirées
  des fenêtres de mesure ; `alacod log [--since <ref>]` : la liste des réglages `editor:` (quoi, ancienne → nouvelle valeur), pour la revue de William.
- **Acceptation** : une séance de réglage devient un scénario vert dans `alacod test` ; `alacod log` lisible à voix haute.

### AT-9 — Séance de William (0,5 j + William)
- Protocole dans `rapports/at-9-seance.md` : une arme (DPS, dispersion), un ennemi (TTK, vitesse), le déplacement (vitesse, dash), à la voix,
  délai de la phrase à l'effet noté ; ce qui manque devient une ligne de dette ou un lot suivant.

## 7. Règles du chantier

- CLAUDE.md en entier, checklist « Nouveau système GGRS » pour AT-1, AT-2 et AT-5 ; `docs/conventions.md` §90 « Atelier de réglage » ouvert par
  le premier lot qui touche la simulation.
- **Tout par `alacod`** : aucun lot n'ajoute de cible `make` nécessaire ni de script dans `scripts/` ; une cible `make` peut rester un alias.
- **Preuves d'agent** : partie jetable sur un autre port (`ALACOD_REMOTE_PORT`), jamais `--commit` dans le dépôt de William, jamais `kill` de
  sa partie ; une partie relancée l'est par `alacod sandbox <jeu> --detach`. Jamais laisser une partie en pause.
- **Dossier de compilation à part** pendant que William joue : `CARGO_TARGET_DIR=target/agent` devant `alacod sandbox`, `cargo test`, `cargo run` et
  `make` qui lient un binaire de jeu. Sur macOS, relier `target/debug/zombies` pendant qu'il tourne a tué la partie de William (2026-10-26, AT-0).
- Une capture d'écran prise pour preuve est **regardée**, pas seulement écrite.
- Un rapport par lot dans `docs/taches/rapports/`, ligne `LIVRÉ <branche>` au journal de `docs/taches.md` §10, une phrase au §8.3.
- Une commande cargo à la fois, `env -u CI`.

## 8. Hors périmètre et risques

- **Hors périmètre** : réglage en ligne (garde `check_distance` 0 inchangée) ; inspecteur Egui ; éditeur de cartes ; graphiques temps réel.
- **Risques** :
  1. `DevMetrics` en synctest : la table balle → arme doit être rollbackée avec le reste (D1) ; sinon les chiffres du banc divergent du jeu.
  2. Coût : une entrée par balle vivante (`bench_tears` : 432 balles) ; vider à la destruction, mesurer `save_ms` en AT-2.
  3. Attribution des dégâts indirects (brûlure, explosion, familiers) : clé « inconnue » plutôt qu'une fausse arme ; le dire dans le widget.
  4. Mêlée non rechargée à chaud (`weapons_config_update_system`) : un réglage de mêlée exige `restart` ; à lever ou à dire en AT-0.
  5. Gel des ennemis : `EnemyMoveSpeed Set 0` peut ne pas couper les attaques à distance ; à vérifier en AT-1 (D5).
  6. Tout réglage `editor:` d'une arme ou d'un personnage de départ déplace des traces et peut casser des attentes de scénario (les bots
     jouent autrement) : constaté en AT-0, `89b2679b` (dispersion de la mitraillette 0,15 → 0,12) a déplacé 51 traces `zombies` et cassé
     les attentes de `clone_quad` et `la_fonderie_demo`. Règle : `BLESS` groupé en fin de séance, en **un** commit qui cite les commits
     `editor:`, attentes corrigées revues avec William ; commande candidate `alacod bless` pour AT-8.
