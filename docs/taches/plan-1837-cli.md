# Plan : préparer l'engine pour 1837, premier jeu externe géré par la CLI (lots X) — rédigé le 2026-10-26

**État au 2026-10-26 : plan écrit, rien lancé.**

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
  cases » et réduit ses planches par `scale` (X-4a le documente, X-4b le propose à l'import). Pivot : aux pieds (design de 1837), à vérifier sur une
  planche réelle en X-4b (le manifeste le place à mi-hauteur).
- **Ennemis de D1 (ex-Q-2, tranché par William le 2026-10-26)** : **petit rôdeur** (charge en zigzag, recul) et **tireur qui recule** (tir annoncé,
  couvert pendant le rechargement), inspirés des adversaires de Gungeon (`1867_lore/design/archetypes-ennemis.md`). Sprites : `fantassin-de-ligne`
  ou `volontaire-loyaliste` pour le tireur ; `loup-hiver-1837` provisoire pour le rôdeur.

À trancher par William (bloque seulement le lot indiqué) :
- **Q-3** (X-6) : la liste « À fermer pour le code » de `prologue-loge.md`.

## 4. Lots (quatre voies au plus en parallèle depuis le 2026-10-26)

| Lot | Contenu | Prérequis | Voie | Effort |
|---|---|---|---|---|
| **X-1 Points d'extension** | entrée unique (D-e) ; runner de scénarios appelable depuis le binaire du jeu ; point d'insertion des enregistrements rollback d'un plugin ; variantes `Custom` ouvertes ; kinds de contenu et règles de lint déclarés par un plugin ; D133 | — | A | high |
| **X-2a CLI : dépendance et lancement** | D-b (`new` par tag, `engine local/pin/status`), racine engine par `cargo metadata` au lieu de `BUILT_FROM_ENGINE` ; `alacod run [--sandbox [profil]] [--headless] [--seed] [--players]` ; `alacod remote …` (remplace `scripts/alacod-remote` pour un jeu externe) ; D126, D130, D131 | — | B | medium |
| **X-2b CLI : test, vidéo, build** | `alacod test` joue les scénarios par le binaire du jeu (plugin compris) et écrit dans le jeu ; `alacod video` (rendu, Mac) ; `alacod build --release` et `--web` (version du jeu dans `APP_VERSION`) ; test de bout en bout du §1.1 | X-1, X-2a | B | medium |
| **X-3 Verbes D1** | fenêtre de jugement (repère : temps d'horloge, début d'action, coup qui arrive ; notes parfait/bien/raté en `FrameEvents`) ; action de renvoi de projectiles (changement d'équipe, visée) et parade de mêlée ; poids des armes (vitesse en portant, si l'existant ne suffit pas) ; hit stop « cible seule » (présentation) ; musique calée sur l'horloge (présentation) | — (verbes natifs de l'engine ; conflits possibles avec les `Custom` de X-1 dans les mêmes enums, résolus au merge) | C (Mac) | high |
| **X-4a Sprites : directions** | nombre de directions par planche (D-c), choix de ligne par `FacingDirection` (`crates/animation`), schéma et lint de `sprites.ron` ; preuve à l'écran (`play_scenario --capture`) | — | D (Mac) | medium |
| **X-4b Sprites : import** | `alacod import-sprite <manifeste.sprite.json>` → entrée `sprites.ron` et `SpriteSheetConfig` ; ancres et mains : seulement si `design/sprites-composables.md` est tranché | X-4a, X-2a (`alacod.rs`) | D | medium |
| **X-5 Création de 1837 et palier D1** | `alacod new 1837`, assets du kit Frontière importés, une pièce, rôdeur et tireur qui recule, hache, fusil, gigue rythmée, scénarios et traces dans le dépôt du jeu ; échelle réglée en jouant ; tout par `alacod` | X-1 à X-4, D-a | A ou B | medium |
| **X-6 Paliers D2 à D5** | verbes `Lure`/`Grab`, contrainte de branche de la clé du boss (`map::generation::grid`), graine de la sauvegarde (`meta::Profile`), extérieur dessiné → intérieur généré ; contenu des paliers | X-5, Q-3 | — | à planifier |

Chemin critique : X-1 → X-2b → X-5. Quatre voies depuis le 2026-10-26 (Mac disponible) : A = X-1 puis X-2b (VPS), B = X-2a (VPS),
C = X-3 (Mac), D = X-4a puis X-4b (Mac ; le rendu y est possible).

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
- `CLAUDE.md` (déterminisme, frontière simulation/présentation), `docs/conventions.md` §4 (point 0 compris).
- Aucun nom de jeu dans le code de l'engine ; aucune trace de clone ne bouge (`BLESS` interdit) ; `make schedule_order_diff` vide pour X-1 et X-3.
- VPS : `make test_crates` un paquet à la fois, `scripts/target-sweep.sh` sous 30 Go libres.
- Dettes : **D139 à D155** réservées à ce plan (D134–D138 : polissage) : X-1 D139–D143, X-2a D144–D148, X-3 D149–D152, X-4a D153–D155 ;
  annoncer le numéro pris dans le journal.
- Docs au merge : `docs/conventions.md` (un § par lot), `docs/jouer.md`, `templates/starter/CLAUDE.md`, `CLAUDE.md` (CLI et entrée unique).
- Chaque lot finit par `LIVRÉ <branche>` dans le journal (`docs/taches.md` §10) et un rapport dans `docs/taches/rapports/x-<n>-*.md`.
