# Plan : préparer l'engine pour 1837, premier jeu externe géré par la CLI (lots X) — rédigé le 2026-10-26

**État au 2026-11-06 : X-4a et X-4b livrés ; X-5 avancé au 6 nov (preuve dim 8) ; voie m découpée (§ 6). Objectif studio : O3, alpha coop du prologue, dim 20 déc.**

Demande de William (2026-10-26) : « préparer le engine pour 1837 et la création du premier jeu externe avec la CLI, qui serait 100 % managé via
notre CLI ». Règle de découpage validée le même jour : `docs/plan-engine.md` §4.8 (engine, contenu ou plugin de jeu) et `docs/conventions.md` §4
point 0. Design de 1837 : dépôt `1867_lore` (`~/workspace/1867_lore` sur le VPS) ; seule démo = le prologue « libérer la loge »
(`design/prologue-loge.md`, `design/plan/feuille-de-route.md` étape D : D1 une pièce, D2 extérieur et clé, D3 hall et grille 3×3, D4 clé du boss,
D5 mini-boss). Relevés du 2026-10-26 (scouts, `main` `5bdb201a`) : chaque fait cite `fichier:ligne`.

## 1. But et critère de fin

1837 vit dans **son propre dépôt**, créé par `alacod new`, et toute sa vie de développement passe par `alacod` : créer du contenu, le valider,
le lancer (partie, bac à sable, mode éditeur), le piloter à distance, le tester (scénarios et traces, **avec son propre code Rust**), filmer,
construire une release ou une version web, changer de version d'engine. Aucun `make`, aucun script de l'engine, aucun chemin vers une copie de
l'engine n'est nécessaire.

**Fini quand** :
1. Un test de bout en bout crée un jeu dans un dossier temporaire et le mène **uniquement par `alacod`** : `new`, `new-content`, `lint`, `test`
   (scénario rejoué avec trace), `run --sandbox --headless`, une commande `remote`, `build --release`. Le jeu porte un **plugin Rust** qui ajoute
   un behavior et une action `Custom`, un type de contenu et une règle de lint ; le scénario du test les exerce.
2. Le jeu dépend de l'engine par un tag git ; `alacod engine local <chemin>` bascule sur une copie locale et `alacod engine pin <tag>` revient.
   Le même jeu marche sur le Mac et le VPS sans retouche.
3. Les verbes D1 (fenêtre de jugement, renvoi de projectiles, poids des armes) existent dans l'engine, avec lint, schéma, scénarios et §
   de conventions ; les planches à 8 directions de la pipeline de `1867_lore` s'affichent.
4. Aucune trace de scénario des clones ne bouge (`BLESS` interdit), `make schedule_order_diff` vide.
5. Le dépôt 1837 existe et joue le palier D1 du prologue (une pièce, deux ennemis, hache, fusil, gigue) **en contenu seulement**.

## 2. État relevé (2026-10-26)

| Brique | État | Où |
|---|---|---|
| CLI | `lint`, `new`, `test`, `new-content`, `describe` ; ni `run`, ni `sandbox`, ni `remote`, ni `video`, ni `build` | `crates/content/src/bin/alacod.rs:20-133` |
| Dépendance du jeu créé | chemin **absolu** de l'engine par défaut ; `--git [--rev]` existe (épingle `HEAD` sans `--rev`) | `alacod.rs:296-361`, `templates/starter/Cargo.toml:15-21` |
| Racine engine des outils | `BUILT_FROM_ENGINE` figé à la compilation de `alacod`, puis heuristiques | `alacod.rs:291,422-470` |
| `alacod test` | lance `alacod-gen` **de l'engine** (`cargo run -p scenario --manifest-path <engine>`) : le code Rust du jeu n'est pas dans le binaire ; écrit les scénarios générés dans `<engine>/tests/scenarios/generated` | `alacod.rs:470-500`, `crates/scenario/src/bin/alacod-gen.rs:267-268` |
| Vocabulaire | kinds de contenu fermés (`KNOWN_KIND_NAMES`), `Behavior`, `Action`, `Condition`, `On` en enums fermés ; seuls `StatId`, `AmmoType`, `DamageKind` ont `Custom(..)` | `crates/content/src/registry.rs:1741`, `crates/effects/src/contracts.rs:17-51`, `actions.rs:65` |
| Enregistrements rollback | tous dans `CoreSimPlugin`, les cinq de clôture en dernier ; un plugin ajouté après `CoreSetupPlugin` enregistre **après** eux (effet réel non mesuré) | `crates/game/src/core_sim.rs:6-12,129-147` |
| Outils hors CLI | `make sandbox/videos/gen`, `scripts/alacod-remote` (Python), `scenario-video`, `p2p-*.sh`, `build-web-games.mjs` (zombies et throne seulement) : écrits pour `games/` | `Makefile:203-205,280-291`, `scripts/scenario-video:30,84`, `scripts/build-web-games.mjs:6-8` |
| Versions | tags `v0.0.1`, `v0.0.2` (anciens) ; `APP_VERSION` par défaut `v0.0.0` pour un jeu externe | `.git/packed-refs`, `crates/game/build.rs:1-5` |
| Sprites | l'engine lit une orientation et la retourne (`flip_x`) ; la pipeline de `1867_lore` sort **8 lignes** (une par direction), un manifeste `.sprite.json`, des planches `_mains_` et `ancres.json` | `crates/animation/src/lib.rs:221-233,342-353`, `1867_lore/tools/assetgen/README.md` |
| Prologue en contenu | salles verrouillées, grille, clé et porte à clé, boss à phases, surfaces, archétypes de tir/charge/mêlée, formation en arc : **existent** | `docs/conventions.md` §35, §37, §48, §70, §26 ; `crates/behaviors/src/lib.rs:31-115` |
| Prologue, manques | parade et renvoi, fenêtre de jugement (gigue), poids des armes (D1) ; `Lure`/`Grab` (D2) ; clé du boss dans une autre branche (D3) ; graine de la sauvegarde (D3) ; extérieur dessiné → intérieur généré (D2, à vérifier) | relevé `EcartPrologue` du 2026-10-26 |

## 3. Décisions

Par défaut (prises le 2026-10-26 en l'absence de réponse ; William peut les renverser avant le lot concerné) :
- **D-a Dépôt** : nouveau dépôt `estran-studio/1837` (`plan-engine.md:207`), créé par `alacod new 1837` au lot X-5 ; les assets y sont exportés
  depuis `1867_lore` (la bible reste la source). Création du dépôt GitHub : William.
- **D-b Dépendance** : tag git épinglé (`alacod new` prend le dernier tag par défaut) ; `alacod engine local <chemin>` pose une surcharge locale
  (patch Cargo hors `Cargo.toml` versionné) pour développer engine et jeu ensemble ; `alacod engine pin <tag>` / `status`. **Ne pas taguer
  X-2a seul** : `v0.1.0` est posé après les merges de X-1 **et** X-2a et la preuve qu'un jeu généré compile avec ce tag (le dernier tag
  `v0.0.2` précède le bac à sable ; le gabarit et le code de l'engine doivent rester compatibles). Pendant les lots, épingler un SHA poussé
  (`--rev`) ou utiliser `--path` explicitement ; X-2b/X-3/X-4b donneront lieu à un tag ultérieur après vérification.
- **D-c Sprites (confirmé par William le 2026-10-26)** : 1837 a besoin des **8 directions**, mais l'engine reste **flexible** : une planche déclare
  combien de directions elle dessine (1 avec miroir : le défaut actuel des clones ; 4 ; 5 avec miroir pour l'autre côté ; 8) et l'engine choisit la
  ligne de la direction la plus proche dessinée, en miroir si la planche le permet. Aucune planche existante ne change.
- **D-d Petites dettes** : D126 (personnage dans `PlayerScript`), D130 (`give` enregistré par `save`), D131 (`player` en dur) vont dans X-2a ;
  D133 (`schedule_order_diff`) dans X-1, qui s'en sert.
- **D-e Une seule entrée de jeu** : le `main.rs` d'un jeu ne fait plus que déclarer ses plugins ; l'engine fournit le point d'entrée qui sert la
  partie, `--sandbox`, le runner de scénarios, le lint du jeu et les bots. C'est ce qui rend le plugin visible partout.

- **Échelle (ex-Q-1, réglage du jeu, William 2026-10-26)** : la taille des personnages à l'écran est un **réglage propre à chaque jeu**, pas une
  décision d'engine. 1837 la règle en jouant D1 (`scale` des planches, `camera.ron`). Ce que l'engine doit garantir : qu'un jeu choisisse librement son
  nombre d'unités par mètre. Contrainte actuelle : les cases du monde et du champ de flux font 16 unités en dur (`world/src/grid.rs:9`,
  `character/enemy/ai/navigation.rs:30`, alignées sur les tuiles LDtk) ; un jeu à grands sprites garde donc des unités de monde « à la taille des
  cases » et réduit ses planches par `scale` (X-4a le documente, X-4b le propose à l'import). Pivot : aux pieds (design de 1837) ; **vérifié en X-4b le 2026-10-28** :
  le pivot du manifeste est au point de contact au sol, pas à mi-hauteur, et `alacod import-sprite --pivot-at X,Y` le pose sur l'entité (rapport `x-4b-import-sprite`).
- **Ennemis de D1 (tranché par William le 2026-10-26, soir)** : **deux fantassins** pour le cœur 30 s, dans le dépôt de 1837 ; le petit rôdeur et le
  tireur qui recule envisagés plus tôt le même jour sont remplacés. Aucun de ces contenus n'entre dans l'engine.

Décision de William (2026-10-27) : ordre jeu-d'abord X-4b → X-5 (9 nov) → X-3 → X-6 ; X-1 et X-2 gelés faute de besoin daté ; chemin critique X-4b → X-5.
Aucune fusion ni tag `v0.1.0` pour X-1/X-2 ; X-4b part de `main` (X-4a livré) sans attendre X-2a (`alacod.rs` de `main` suffit, `rapports/x-4b-arret.md`).
Notes d'arrêt : `rapports/x-1-arret.md`, `x-2a-arret.md`, `x-2b-arret.md`, `x-3-arret.md`, `x-4b-arret.md`.

**Décisions de William (5 et 6 nov 2026, brief du studio `runs/x-5/brief-studio.md`) : O3, alpha coop du prologue complet de 1837, dans Chrome à la manette, à deux en ligne, dim 20 déc.**
Les clones (throne, isaac, zombies, gungeon) restent des jeux de test de l'engine, sans aucun lot (décision du 5 nov) ; le run du polissage est fermé. Ce plan (O2) sert O3.
- Décision de William (2026-11-06) : **prologue complet** : extérieur, poste de garde et clé, hall et grille 3×3 par graine de sauvegarde, clé du boss dans une autre branche,
  sergent en deux phases, fin. Porte du **dim 22 nov** : William valide le cœur de 30 s ; sinon l'alpha se limite au cœur et à l'extérieur.
- Décision de William (2026-11-06) : **coop de première classe** : chaque palier se joue à deux en ligne dès D1, rien n'est d'abord solo. Repères : `1867_lore/design/jeu.md`
  265-273 (tranche à deux), 248 (feu follet à la mort), 225-233 (fenêtre de rythme élargie en coop).
- Décision de William (2026-11-06) : **planches 2D en 8 directions**, pas de 3D en direct.
- Décision de William (2026-11-06) : **harmonium dans le jeu**. `harmonium_bevy` est déjà en Bevy =0.19.1 ; la feature `harmonium` de `crates/game` est coupée (commentaire
  périmé « encore sur bevy 0.17 »). Le coordinateur harmonium prépare le plugin natif et wasm (rapport attendu d'ici le 13 nov :
  `harmonium/specs/013-moteur-dans-le-jeu/pret-pour-1837.md`) ; l'engine réactive la feature après ce rapport (M-8). Pas de port vers Bevy 0.20 avant Noël. Le principe II
  (plan de temps partagé, `horloge-etude.md` § 10) reste une décision de William.
- Décision de William (2026-11-06) : **Chrome** : le build de 1837 se joue par URL.
- Décision de William (2026-11-06) : **dépôt `estran-studio/1837` privé** (remplace « création du dépôt GitHub : William » de D-a : le coordinateur le crée au lot X-5).
- Décision de William (2026-11-06) : **William écrit lui-même la définition du niveau** dans `1867_lore` (`design/prologue-loge.md`, les six « À fermer ») ; les agents la
  lisent et ne l'écrivent jamais. Si la définition d'un palier manque, la question va à William une semaine avant, par la carte `--unread`, sans bloquer le reste.
- Économie (inchangée) : workers en Sonnet, deux voies validées par William (m : engine, VPS ; j : dépôt 1837, coordinateur sur le Mac) ; ni Chrome automatisé ni
  campagne de bots ; `make check` seul avant merge ; une trace qui bouge : stop et rapport ; la partie du soir de William fait foi.

À trancher par William (bloque seulement le lot indiqué) :
- **Q-3** (X-6) : la liste « À fermer pour le code » de `prologue-loge.md` ; William l'écrit lui-même (décision du 6 nov), question posée une semaine avant le palier.

## 4. Lots (état au 2026-11-06)

| Lot | Contenu | Prérequis | État (2026-11-06) | Effort |
|---|---|---|---|---|
| **X-1 Points d'extension** | entrée unique (D-e) ; runner de scénarios appelable depuis le binaire du jeu ; point d'insertion des enregistrements rollback d'un plugin ; variantes `Custom` ouvertes ; kinds de contenu et règles de lint déclarés par un plugin ; D133 | — | **gelé**, branche conservée (`x-1-points-extension`, `integration-1837`) | high |
| **X-2a CLI : dépendance et lancement** | D-b (`new` par tag, `engine local/pin/status`), racine engine par `cargo metadata` au lieu de `BUILT_FROM_ENGINE` ; `alacod run [--sandbox [profil]] [--headless] [--seed] [--players]` ; `alacod remote …` (remplace `scripts/alacod-remote` pour un jeu externe) ; D126, D130, D131 | — | **gelé**, branche conservée (`x-2a-cli-dependance-lancement`, `integration-x2a`) | medium |
| **X-2b CLI : test, vidéo, build** | `alacod test` joue les scénarios par le binaire du jeu (plugin compris) et écrit dans le jeu ; `alacod video` (rendu, Mac) ; `alacod build --release` et `--web` (version du jeu dans `APP_VERSION`) ; test de bout en bout du §1.1 | X-1, X-2a | **gelé**, brouillon conservé (`x-2b-brouillon-sur-x2a`) | medium |
| **X-3 Verbes D1** | fenêtre de jugement (repère : temps d'horloge, début d'action, coup qui arrive ; notes parfait/bien/raté en `FrameEvents`) ; action de renvoi de projectiles (changement d'équipe, visée) et parade de mêlée ; poids des armes (vitesse en portant, si l'existant ne suffit pas) ; hit stop « cible seule » (présentation) ; musique calée sur l'horloge (présentation) | X-5 (besoins montrés par 1837) | **repris par la voie m** (M-3, § 6), branche conservée (`x-3-verbes-d1` @ `0e57f6f3`) | high |
| **X-4a Sprites : directions** | nombre de directions par planche (D-c), choix de ligne par `FacingDirection` (`crates/animation`), schéma et lint de `sprites.ron` ; preuve à l'écran (`play_scenario --capture`) | — | **livré** | medium |
| **X-4b Sprites : import** | `alacod import-sprite <manifeste.sprite.json>` → entrée `sprites.ron` et `SpriteSheetConfig` ; ancres et mains : seulement si `design/sprites-composables.md` est tranché | X-4a (depuis `main`) | **livré** le 2026-10-29 (`wquintal/x-4b-import-sprite`, rapport `x-4b-import-sprite.md`) ; mains et ancres : D156, plus tard | medium |
| **X-5 Création de 1837 et kit de la cour** | dépôt privé `estran-studio/1837` par `alacod new 1837 --git --rev <main poussé>` ; kit importé par `alacod import-sprite` depuis `1867_lore/assets/sprites/` (le Frère avec hache et fusil, le fantassin, la loge, le décor de la cour) ; retrait de l'exception fantassin du testbed (§ 5) ; `1837/docs/tranche.md` ; commande du build du soir (natif) ; preuve dim 8 : la commande + une capture du Frère et du fantassin dans la cour, à l'échelle du jeu | X-4b, décision du 6 nov | **en cours** (avancé au 6 nov) | medium |
| **X-6 Paliers D2 à D5** | portes et clé portée par un ennemi, couverts qui arrêtent les balles, grille 3×3 par graine, `Lure`/`Grab`, boss en deux phases (§ 6, 23 nov → 13 déc) | X-5, M-3, Q-3 | voie m, lots écrits avant leur départ | à planifier |

Chemin critique (2026-11-06) : X-5 (6 → 8 nov), puis la voie m dans l'ordre du calendrier de `1837/docs/tranche.md` (§ 6). X-1, X-2a et X-2b restent gelés (aucune fusion, aucun tag).
Avant le 2026-11-06 : X-4b → X-5 (9 nov) → X-3 → X-6 ; avant le 2026-10-27 : X-1 → X-2b → X-5, sur quatre voies.

### Espace de solution par lot

- **X-1** : la forme de l'entrée unique (fonction `run(GameSpec)`, trait, macro) et celle des `Custom` (paramètres RON bruts décodés par le plugin
  au chargement, ou schéma déclaré) sont ouvertes. Contraintes : un `Custom` inconnu est une erreur de lint ; `alacod lint` (binaire sans le code du
  jeu) doit connaître les noms déclarés par un plugin (déclaration en données dans `game.ron`, ou lint délégué au binaire du jeu : choisir et
  justifier) ; l'ordre des enregistrements rollback des clones ne change pas (`schedule_order_diff` vide) ; zéro trace qui bouge ; mesurer si
  l'ordre d'enregistrement d'un plugin après la clôture a un effet, et le documenter.
- **X-2a** : le remote en Rust dans `alacod` (sans Python) est le défaut ; garder `scripts/alacod-remote` pour l'engine tant que les deux
  coexistent, sans second protocole (mêmes méthodes `alacod/*`).
- **X-3** : un verbe par besoin, jamais un « système de rythme » (§4.8) ; valeurs en frames entières (60 ms ≈ 4 frames à 60 Hz : arrondi à
  documenter) ; tout en `Fixed`, `FrameEvents`, checksum neutre pour les types nouveaux.
- **X-4a** : la sélection de ligne est de la présentation ; la simulation ne change pas (`FacingDirection` existe déjà, 8 valeurs). Une planche
  sans déclaration garde le comportement actuel (une orientation et le miroir) : aucune trace, aucune capture des clones ne change. Documenter
  comment un jeu choisit son échelle (unités par mètre face aux cases de 16, `scale` des couches, `camera.ron`) : c'est un réglage du jeu.

## 5. Règles communes

- **Rien de 1837 dans `alacod` (William, 2026-10-26)** : le dépôt de l'engine ne reçoit que des **blocs de logique génériques** (verbes, behaviors,
  composants, outillage). Aucun personnage, ennemi, arme, sprite, carte, valeur de réglage ni nom de 1837 (pas de « gigue », « rôdeur », « loge »,
  120 bpm…) dans le code, le contenu, les fixtures ou les scénarios de l'engine ; ceux-ci utilisent le contenu générique du testbed et des noms
  neutres (`dash`, `parade`, `renvoi`). Le contenu de 1837 (ennemis de D1, sprites, réglages) vit dans son dépôt, créé au lot X-5.
  - **Exception datée (décision de William du 2026-10-28, Q-X4b-1), levée au lot X-5** : la planche `fantassin-de-ligne` (pipeline `1867_lore`), son personnage d'essai et le scénario
    `sprite_import_fantassin` étaient dans `games/testbed` et `tests/scenarios` pour tester X-4b. Ils sont **retirés** (revert de `a878966c`, `7b5d9d20` et `a81523fe`,
    `docs/conventions.md` § 107) ; le dépôt de 1837 importe la planche lui-même. Aucune autre exception : l'outil, les tests synthétiques et la doc restent neutres.
- `CLAUDE.md` (déterminisme, frontière simulation/présentation), `docs/conventions.md` §4 (point 0 compris).
- Aucun nom de jeu dans le code de l'engine ; aucune trace de clone ne bouge (`BLESS` interdit) ; `make schedule_order_diff` vide pour X-1 et X-3.
- VPS : `make test_crates` un paquet à la fois, `scripts/target-sweep.sh` sous 30 Go libres.
- Dettes : **D139 à D155** réservées à ce plan (D134–D138 : polissage) : X-1 D139–D143, X-2a D144–D148, X-3 D149–D152, X-4a D153–D155 ; X-4b D156–D157 ; X-5 **D186–D189**,
  voie m **D190–D219** (dix par semaine du calendrier : 9 → 15 nov D190–D199, 16 → 22 nov D200–D209, X-6 D210–D219). Sections de `docs/conventions.md` : X-5 § 107,
  voie m § 108 et suivants, une par lot, demandées au coordinateur. Annoncer le numéro pris dans le journal.
- Docs au merge : `docs/conventions.md` (un § par lot), `docs/jouer.md`, `templates/starter/CLAUDE.md`, `CLAUDE.md` (CLI et entrée unique).
- Chaque lot finit par `LIVRÉ <branche>` dans le journal (`docs/taches.md` §10) et un rapport dans `docs/taches/rapports/x-<n>-*.md`.

## 6. Voie m (engine, VPS) : lots et preuves (écrits le 2026-11-06, avant leur départ)

Dans l'ordre du calendrier de `1837/docs/tranche.md`. Tout verbe nouveau de 1837 passe par l'engine (X-1 gelé), avec le contenu générique du testbed et des noms neutres (§ 5).
`make check` prend 1 à 3 h par merge : les lots d'engine passent avant le contenu, et un lot ne part qu'avec sa preuve écrite ici. Un lot = une branche, un worker Sonnet,
`make check` vert avant le merge (par le coordinateur), aucune trace des clones qui bouge sans stop et rapport. Les dates sont celles où l'engine fournit le bloc à la voie j.

| Lot | Contenu | Preuve (avant le merge) | Fourni à la voie j |
|---|---|---|---|
| **M-1 Web d'un jeu externe** | le build Chrome du dépôt 1837 : une commande qui produit le wasm et les assets d'un jeu hors de `games/` (aujourd'hui `scripts/build-web-games.mjs` et le `Makefile` ne connaissent que les jeux de l'engine), servie par URL ; version dans `APP_VERSION` | transcript de la commande sur un jeu créé par `alacod new` dans un dossier temporaire (wasm et assets produits, `cargo check --target wasm32-unknown-unknown` vert) ; partie du soir de William sur le build de 1837 par URL (aucun Chrome automatisé) | mer 11 nov |
| **M-2 À deux en ligne dans Chrome** | relevé de l'état (matchbox et lobby existent sur le web ; allumette absent du web, `crates/game/src/args/mod.rs:203` ; `polissage-reseau-amis` @ `8297b2ea`, S1-3 préparé, non recetté, non fusionné) puis le minimum pour que deux Chrome sur deux réseaux jouent 1837 à deux : fusion de `polissage-reseau-amis` si sa recette passe, allumette sur le web seulement si le lobby matchbox ne suffit pas | relevé en dix lignes (fichier:ligne) avant tout code ; `make check` ; protocole de recette en dix lignes pour William (deux machines, URL, lobby), la partie du soir fait foi ; aucun desync à l'écran | ven 13 nov |
| **M-3 Horloge, jugement, gigue, renvoi** | reprise de `x-3-verbes-d1` (`0e57f6f3`, `x-3-arret.md`) : vérifier puis rebaser sur `main` ; horloge de rythme dans la sim, condition `Timing` et notes `TimingJudged` en `FrameEvents`, dash calé (gigue), fenêtre de rythme réglable en données et élargie en coop ; renvoi des balles par une hitbox de mêlée **sur la seule fenêtre `frappe`** (changement d'équipe, visée) ; on ne garde que ce que D1 utilise | unitaires `combat`, `effects`, `content`, `game` ; scénarios neutres du testbed (renvoi dans la fenêtre et hors fenêtre, note parfait/bien/raté, fenêtre coop plus large) avec traces neuves seulement ; `make schedule_order_diff` (seuls les types neufs, neutres, avant la clôture) ; aucune trace existante modifiée ; revue indépendante (un P1 et trois P2 de `35d94c5e` fermés) | dim 15 nov |
| **M-4 Une bouche de canon par direction** | le point de sortie des balles par direction (8) lu dans les ancres du manifeste (`ancres.json`, part de D156) par `alacod import-sprite`, posé sur la feuille, lu par le tir | test d'import sur un manifeste synthétique (8 bouches) ; scénario neutre : position de naissance de la balle par direction ; capture (Xvfb + lavapipe sur le VPS) d'un tir dans les 8 directions | dim 15 nov |
| **M-5 Le cœur à deux : PV, feu follet, relève** | mort d'un joueur en coop → état « feu follet » (ne frappe pas, se déplace, attire les ennemis) ; relève par le coéquipier ou au point de reprise (`jeu.md` 248) ; PV et munitions lus par le HUD | scénarios neutres à deux joueurs (mort → feu follet → relève ; les deux tombés → défaite) ; trace neuve seulement ; HUD par `HudText` | jeu 19 nov |
| **M-6 Tri en y et `moments` du manifeste** | ordre d'affichage par y (présentation, `PostUpdate`) ; lecture des `moments` du manifeste (images-clés : `frappe`, tir) par `import-sprite`, exposés aux verbes qui en ont besoin (fenêtre `frappe` de M-3) | test d'import (moments) ; capture de deux personnages qui se croisent ; aucune trace modifiée | ven 20 nov |
| **M-7 Feature `harmonium`** | réactiver la feature de `crates/game` après le rapport du coordinateur harmonium (`pret-pour-1837.md`, attendu d'ici le 13 nov), natif et wasm, Bevy =0.19.1 ; musique de la salle (présentation, hors simulation) | `cargo check -p game --features harmonium` natif et `wasm32-unknown-unknown` ; aucune trace modifiée ; écoute de William (geste humain, carte) | ven 20 nov |
| **X-6 (23 nov → 13 déc)** | portes et clé portée par un ennemi ; couverts qui arrêtent les balles ; grille 3×3 par graine de sauvegarde ; `Lure` et `Grab` ; boss en deux phases | preuves écrites ici avant le départ de chaque lot, d'après la définition du niveau de William | 29 nov, 6 déc, 13 déc |

**Jalon servi par chaque lot** (plan par jour validé par William le 6 nov : `1837/docs/tranche.md`, copie `runs/x-5/plan-jour.md`) :
- X-5 → S0, **dim 8** : « la commande pour lancer le jeu, plus une capture » (cible sam 7 : kit importé).
- M-1 → S1, **dim 15**, cible mer 11 : « 1837 jouable dans Chrome par URL ».
- M-2 → S1, **dim 15**, cible ven 13 : « 1837 à deux en ligne dans Chrome ».
- M-3 → S1, **dim 15** : « la gigue au tempo ; la hache qui renvoie les balles ».
- M-4 → S1, **dim 15** : « une bouche de canon par direction ».
- M-5 → S2, **dim 22 (la porte)**, cible jeu 19 : « le joueur mort devient un feu follet et son coéquipier peut le relever » ; et lun 16 « la vie, les dégâts, les munitions et leur affichage ».
- M-6 → S2, **dim 22**, cible ven 20 : « le tri en profondeur » ; les `moments` servent la fenêtre `frappe` de M-3 (dim 15), en données provisoires d'ici là.
- M-7 → S2, **dim 22**, cible ven 20 : « la musique de la salle par harmonium ».
- X-6 → S3 **dim 29** (portes, clé portée par un ennemi, couverts qui arrêtent les balles), S4 **dim 6 déc** (grille 3×3 par graine ; lutins qui s'agrippent : `Grab`),
  S5 **dim 13 déc** (clé du boss dans une autre branche, sergent en deux phases) ; `Lure` selon la définition de William. Le plan par jour la fait **commencer lun 16** (« la génération des portes, de la clé et
  de l'intérieur de la loge commence ») : à lancer dès que M-1 à M-4 sont mergés, sans attendre le 23.
- S1 jeu 12 (« 4 fantassins… un cône au sol annonce le tir 1 s avant… rechargent 3 s ») : contenu de la voie j, pas un lot d'engine : l'engine a `Pattern::Telegraph(n)`
  et le dessin de cône `pattern_telegraph` (`crates/combat/src/emitter.rs:18`, `crates/game/src/feedback.rs:557`) ; à confirmer par la voie j, sinon proposition.

**Règles de William (2026-11-06) pour ce plan** : (1) les jalons du dimanche sont l'engagement ; les jours sont des cibles ; on vise l'avance : un lot fini tôt lance le
suivant sans attendre sa date, toujours dans l'ordre ; (2) chaque lot cite le jalon qu'il sert ; (3) à chaque point de contrôle, le coordinateur vérifie que les tâches en
cours servent le jalon de la semaine : une tâche qui n'en sert aucun ne part pas et va dans `propositions.md` (racine du dépôt).

Coupes si ça glisse (ordre du studio) : la grille tirée devient une disposition fixe ; puis le lutin sort de l'alpha.
