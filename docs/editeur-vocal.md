# Éditeur par l'IA : guide pour Claude Code et pour William (chantier K5, v0 du lot K5-0)

Plan : `docs/taches/plan-editeur-vocal.md`. William joue ; Claude modifie les RON de `games/<jeu>/assets` et pilote la partie en marche ;
le rechargement à chaud montre le résultat. Aucun éditeur graphique : le client (Claude Code, ou un client vocal qui l'héberge) n'a besoin que
d'un shell.

## 1. Lancer

Tout passe par la commande **`alacod`** (tableau des commandes au §2) : aucun script, aucun `make` n'est requis (SB-6, `docs/conventions.md` §89). Les cibles
`make sandbox` et `make editor` restent des alias.
Installer : `cargo install --path crates/content --bin alacod --locked` depuis la racine de l'engine (dans `~/.cargo/bin`) ; **la relancer après chaque pull de l'engine**, sinon `alacod` garde l'ancienne version.
`alacod self-check` le dit en une ligne : binaire plus ancien que le fichier le plus récent de `crates/content/src` ou `crates/game/src` (engine d'où il a été compilé) → « PÉRIMÉ » et code 1, sinon code 0.

```bash
alacod sandbox games/zombies --detach     # le bac à sable (§2.2) : arène VIDE + profil de départ, mode éditeur compris ; rend la main quand le remote répond
alacod sandbox games/zombies --profile lourd --seed 42 --headless
alacod sandbox                            # dans le dossier d'un jeu (ou d'un jeu créé par `alacod new`) : celui-là
alacod remote brief                       # puis on pilote la partie (§2) ; `alacod remote discover` liste les méthodes
make editor GAME=zombies                  # le jeu complet (cartes, vagues) en mode éditeur : fenêtre, rendu, clavier jouable
make editor GAME=zombies HEADLESS=1       # sans fenêtre (VPS, CI) : profil headless
```

`alacod sandbox [jeu] [--profile P] [--seed N] [--headless] [--detach] [--wait SECONDES] [-- args du jeu]` construit le jeu (`cargo run`, mêmes features
et même `APP_VERSION` que `make` : pas de recompilation d'une commande à l'autre) et le lance avec `--sandbox [profil]`. `jeu` = un dossier de `games/` ou un jeu créé par
`alacod new` (qui est son propre workspace : `cargo run` dans son dossier, SB-4, `docs/conventions.md` §88). Sans `--detach` le jeu reste au premier plan ; **avec
`--detach`** (le mode des agents) il tourne dans son propre groupe de processus, journal dans `$TMPDIR/alacod-sandbox-<paquet>.log`, et la commande rend la main quand le remote
répond (échec si le jeu meurt avant, ou après `--wait`, 900 s par défaut : compilation à froid comprise) ; elle affiche le pid et `kill -- -<pid>` pour l'arrêter. Elle refuse de
lancer un second jeu si le port répond déjà. Le jeu est lancé avec :

| Variable | Rôle |
|---|---|
| `ALACOD_EDITOR=1` | mode éditeur (K5-1) : implique le remote ; la partie **tourne dès le départ** (pas de `resume`), le **clavier reste la source d'input**, `check_distance` forcé à 0 (posé par `--sandbox` lui-même) |
| `ALACOD_REMOTE=1` | serveur BRP (JSON-RPC sur HTTP) `127.0.0.1:15702` ; `ALACOD_REMOTE_PORT` change le port (`alacod remote` le lit aussi) |
| `ALACOD_LABELS=1` | étiquettes F6 au-dessus des entités (nom, prix, état) |
| feature `native` | `bevy/file_watcher` : le rechargement à chaud des assets |

### Apps de test

| App | Commande | Ce qu'elle apporte |
|---|---|---|
| Bac à sable | `alacod sandbox <jeu> [--profile lourd] [--detach]` (alias `make sandbox GAME=<jeu> [PROFIL=lourd]`) | L'**arène vide** du jeu (murs et joueur, rien d'autre : `alacod new-arena`, §2.3), sans vagues ni étages, avec un profil de départ (`sandbox/<profil>.ron`) ; mode éditeur et remote allumés par `--sandbox` lui-même, sans variable à poser ; `check_distance` à 0. Remplace `character_tester`. Voir §2.2 |
| Jeu complet en mode éditeur | `alacod editor <jeu> [--seed N] [--detach]` (alias `make editor GAME=<jeu>`) | Les cinq jeux, carte du manifeste, `--seed` ; `ALACOD_EDITOR`/`ALACOD_REMOTE`/`ALACOD_LABELS` posées par la commande |
| Map Tester | `alacod editor <jeu>` + `alacod remote restart --seed N` | Pas d'app dédiée : la carte se régénère par graine |
| `map_preview` | `cargo run --example map_preview` | Aperçu statique rechargé à la touche R ; **sans** session GGRS, donc sans remote (`alacod/state` n'y a rien à lire) |
| `map_generation` | `make map_generation_test` | Ligne de commande sans `App` |
| Music Tester | hors périmètre | Attend Harmonium ; même mode local et même `alacod remote` |

Un jeu lancé à la main (`cargo run -p zombies`) reste un jeu : seuls `alacod editor` (`make editor`) et `--sandbox` allument le mode éditeur.
Deux apps de test ne peuvent pas tourner ensemble sur le port 15702 sans `ALACOD_REMOTE_PORT`.

En mode éditeur la partie n'est pas en pause : on joue au clavier pendant que Claude édite. `alacod remote input Right` **prend la main** sur le
clavier pour ce handle (le joueur avance seul) ; `alacod remote input ""` (vide) **rend la main au clavier**. Hors mode éditeur (`make remote`),
le comportement d'avant est inchangé : partie en pause, clavier coupé, `step` pour avancer.

Il n'y a pas d'option `--map` dans la ligne de commande des jeux : la carte vient du manifeste (`entry.start_map` de `game.ron`) ; `--seed` fait varier
l'assemblage. Le client vocal et Claude parlent au même port ; l'ordre de lancement n'importe pas.

## 2. Outils

Tout est la commande **`alacod`** ; `alacod --help` liste les commandes (une ligne chacune), `alacod <commande> --help` le détail.

| Commande | Ce qu'elle fait | Section |
|---|---|---|
| `alacod sandbox [jeu] [--detach]` | construit et lance le bac à sable (arène vide + profil) | §1, §2.2 |
| `alacod editor [jeu] [--detach]` | construit et lance le jeu complet en mode éditeur | §1 |
| `alacod remote <commande>` | pilote la partie en cours : `brief`, `state`, `spawn`, `give`, `step`, `restart`, `modifier`, `run <fichier>`… | §2, §2.5 |
| `alacod get [jeu] <Kind> <id> <champ>` | lit une valeur d'un RON de contenu | §2.4 |
| `alacod set [jeu] <Kind> <id> <champ> <valeur> [--commit]` | règle une valeur (lint, restauration si rouge, commit `editor:`) | §2.4 |
| `alacod undo [jeu]` | annule le dernier commit `editor:` | §2.4 |
| `alacod describe [jeu] [Kind]` | kinds, ids et champs d'un jeu | §2 |
| `alacod lint [jeu]` | valide le contenu | §2 |
| `alacod new-content [jeu] <Kind> <id>` | crée une entrée à partir d'un modèle | §2.2 |
| `alacod new-arena [jeu]` | écrit l'arène vide du bac à sable | §2.3 |
| `alacod new <nom>` | crée un jeu hors de l'engine | `docs/conventions.md` §88 |
| `alacod test [jeu] [id…]` | joue les scénarios générés d'un jeu | `docs/conventions.md` §54 |

Installer la commande : `cargo install --path crates/content --bin alacod --locked` (depuis la racine de l'engine ; installe dans `~/.cargo/bin`, à mettre dans le `PATH`). Mettre à jour : relancer la même commande après chaque pull de l'engine (§1).
Sans installation, `cargo build -p content --bin alacod` puis `target/debug/alacod`.


- **`alacod remote <commande>`** (Rust, sans dépendance : un `TcpStream` vers le remote ; remplace l'ancien script Python `scripts/alacod-remote`, supprimé) : `state` (JSON complet), `brief` (joueurs, ennemis proches et commandes en attente),
  `input Up,Fire --pan 100,0 [--handle 0]` ; **`input ""` relâche et rend la main au clavier**, `step 30 [--no-wait] [--keep-paused]` (rend la partie comme elle était : §2.5), `pause`, `resume`,
  `screenshot <png> [--wait S]` (attend jusqu'à `S` secondes, 5 par défaut, que le fichier soit écrit et complet ; code 1 sinon ; `--wait 0` rend la main sans attendre), `save <scénario.ron>`, `discover`, `run <fichier>` (scripts : §2.5).
  Mode éditeur (K5-1/K5-2) : `restart [--kind {Restart,ToLobby,EnterRun,ToHub}] [--seed N]`, `modifier [--handle 0|--net-id N] [--until FRAMES] [--source S] STAT {Add,Pct,Mul,Set} VALEUR`,
  `unmodify [--handle 0|--net-id N] [--source S]`, `reload`, `spawn <personnage> (--at X,Y | --ahead [PX]) [--team T] [--variant V]`,
  `spawn-powerup <id> (--at X,Y | --ahead [PX])`, `spawn-item <id> (--at X,Y | --ahead [PX])`. Bac à sable : `profile [id]`, `give`.
  `brief` commence par `editor=… online=… check_distance=… run.step=…`. Une erreur du jeu (garde, id inconnu) sort sur stderr avec le code 1 ; un jeu éteint est dit
  (« le jeu ne répond pas sur 127.0.0.1:15702 », code 3). `--json` donne une enveloppe JSON (§2.5). `alacod remote --help` et `alacod remote <commande> --help` listent tout.
- **Méthodes `alacod/*` du mode éditeur** (JSON-RPC, `http://127.0.0.1:15702`) :
  - `alacod/restart` `{kind?: "Restart"|"ToLobby"|"EnterRun"|"ToHub", seed?: i32}` : pose un `RunRequest` ; le jeu repasse par `GameLoading`
    (tables dérivées du `Registry` recalculées, carte rechargée) puis `InGame`. `seed` écrit `MapGenerationConfig.seed` ; refusé avec `ToLobby`.
  - `alacod/set_modifier` `{handle?|net_id?, stat, op: Add|Pct|Mul|Set, value: "0.9", source?: "editor", until_frames?}` : modificateur
    **vivant** sur une entité (joueur 0 par défaut) ; `stat` = un `StatId` (`MoveSpeed`, `MaxHealth`, `Damage`, `FireRate`, `ReloadSpeed`…) ou `custom:<nom>`.
  - `alacod/clear_modifiers` `{handle?|net_id?, source?: "editor"}` : retire ceux de la source.
  - `alacod/spawn_character` `{character, x, y, team?: "players"|"enemies"|"neutral", variant?}`, `alacod/spawn_powerup` `{id, x, y}`, `alacod/spawn_item` `{id, x, y}` :
    l'id est validé (erreur `INVALID_PARAMS` sinon), la commande est posée dans `GameCommands` à la frame **suivante** (réponse `{frame}`), appliquée par la simulation
    (net id attribué dans l'ordre de la file) et ajoutée comme placement (`characters`/`powerups`/`items`) à l'enregistreur : `save` produit un scénario que le runner rejoue
    à l'identique. Visibles dans `state.commands` tant que leur frame n'est pas passée. Même garde que les autres écritures (d=0, hors ligne).
  - `alacod/reload_content` : relit le `Registry` tout de suite ; renvoie les compteurs et `errors` (liste vide = remplacé). Les tables dérivées
    prennent effet au prochain `GameLoading` (`restart` ou changement d'étage).
  - `alacod/state` s'enrichit : `editor`, `online`, `check_distance`, `game`, `map`, `seed`, `run.step`, `config` de chaque ennemi.
  - **Garde** : `restart`, `set_modifier`, `clear_modifiers` et `reload_content` sont refusées si la session est en ligne ou si
    `check_distance > 0` ; l'erreur JSON-RPC nomme la cause (« check_distance=2 »…).
- **`alacod remote discover`** : appelle `rpc.discover` (OpenRPC 1.3.2) et liste les méthodes (les `alacod/*`, puis les intégrées
  `world.*`, `registry.schema`, `schedule.*`). Les `world.*` exigent des types `Reflect` : presque aucun type de simulation ne l'est ;
  **l'état de la partie se lit par `alacod/state`**, pas par `world.query`.
- **`alacod lint games/<jeu>`** : valide manifeste et RON. `--json` pour une sortie lisible par machine. Code de sortie 1 s'il y a des erreurs.
- **`alacod describe games/<jeu> [--json] [Kind]`** : les kinds du jeu et leurs champs. **Ne pas recopier cette liste : l'appeler** quand on ne sait pas
  quel champ porte un réglage.
- **Hook de lint** (`.claude/settings.json` → `scripts/lint-edited-game.sh`) : après chaque `Edit`/`Write` sous `games/<jeu>/`, relance
  `alacod lint games/<jeu>` (asynchrone, 300 s). Un lint rouge est à corriger avant de dire « fait ».
- `git` : `alacod undo` pour « annule ça » (§2.4) ; le reste de `git` est inchangé.

## 2.1 Commandes vocales de placement

« Spawne un zombie_runner devant moi » se traduit par `alacod remote spawn zombie_runner --at X,Y`.
« Devant moi » : `alacod remote spawn zombie_runner --ahead` (60 px, ou `--ahead 100`, `--handle N`). La commande lit `alacod/state`
(`players[].x`, `y` et `facing`, l'une des huit directions du personnage : `Right` = +x, `Up` = +y) et envoie la position explicite
(diagonales normalisées) ; le moteur ne connaît que des coordonnées, jamais « devant ». Sans `facing`, la commande prend `Right`.
Le tableau des phrases du bac à sable (« donne-moi », « mets-moi 500 sous », « charge le profil », « reset », « un peu plus rapide ») est au §2.2.
L'id d'un ennemi ou d'une arme vient de `alacod describe games/<jeu> Character` / `Weapon` (la dictée déforme les ids : lister, proposer le plus proche).

## 2.2 Bac à sable : « donne-moi », « charge le profil », « reset »

`--sandbox [profil]` (§1) ouvre l'**arène vide** du jeu (§2.3 : un niveau, des murs, le joueur, rien d'autre) avec un **profil** : le sous-ensemble de `Scenario` que décrit `alacod describe Sandbox` (`players[]` avec `character`, `weapon`, `items`, `currency`, `modifiers`, `mutations`, `dash`, `bomb`, `aim`, `tags`,
`immune_to` ; `characters[]` posés ; `items[]`, `chests[]`, `shop`). Le profil `arene` ne pose **aucun ennemi** (il donne seulement de quoi jouer : solde, objets de départ) ; `lourd` en pose quelques-uns. Un profil se crée par `alacod new-content Sandbox <nom>`, se lint comme le reste. Commandes (tout est gardé : local, `check_distance` 0) :

| On dit | On lance | Effet |
|---|---|---|
| « Quels profils ? » | `alacod remote profile` | liste (l'actif marqué `*`, arène, joueur, ennemis posés) |
| « Charge le profil lourd » | `alacod remote profile lourd` | arène du profil, puis `restart` : joueur recréé (avec son `character`), profil reposé. Id inconnu : erreur qui liste les profils |
| « Donne-moi le fusil » | `alacod remote give --weapon <id de l'arme>` (`shotgun` dans zombies, `fusil_pompe` dans gungeon) | **ajoute** l'arme (réserve complétée si déjà tenue ; l'arme active cède sa place si les emplacements sont pleins ; mêlée : remplace la mêlée tenue) |
| « Donne-moi la bague » | `alacod remote give --item <id>` | objet comme un ramassage (passif + ses modificateurs, actif chargé, consommable +1) |
| « Mets-moi 500 sous » | `alacod remote give --set-currency 500` | impose le solde ; « ajoute 100 » : `--currency 100`, « retire 100 » : `--currency -100` |
| « Spawne-moi un X devant moi » | `alacod remote spawn X --ahead` (`--ahead 100` : 100 px) | §2.1 ; ennemi à 60 px devant le joueur à la frame suivante |
| « Un peu plus rapide » | `alacod remote modifier --handle 0 --source editor MoveSpeed Mul 1.1` | modificateur vivant ×1,1 (barème §4 : « un peu » = 10 à 15 %) ; « annule » : `alacod remote unmodify --handle 0 --source editor` |
| « Reset » | `alacod remote restart` | rejoue le profil actif : tout ce qui a été donné, modifié ou spawné disparaît |

Un jeu créé par `alacod new` a un seul profil, `arene` (`assets/sandbox/arene.ron` : 1 000 de solde, aucun ennemi ; `alacod remote spawn target --ahead` fait apparaître la cible) ; en ajouter un : `alacod new-content Sandbox lourd`.

Pour changer un profil : éditer `games/<jeu>/assets/sandbox/<nom>.ron`, puis `alacod remote reload` et `alacod remote restart` (le profil se relit du registre à chaque entrée en partie). `give` n'est pas enregistré par `save`.

## 2.3 L'arène vide : `alacod new-arena`

Le bac à sable s'ouvre sur **une salle de murs avec le joueur, rien d'autre** : ni fenêtre, ni porte, ni arme murale, ni soda, ni point d'apparition de zombies, ni personnage. Cette arène est
une carte LDtk à un niveau (26×19 cases = 416×304 px, le joueur 0 apparaît en (424, 696)) **générée à partir des cartes du jeu lui-même** : `alacod new-arena [jeu] [--name arene_vide]
[--size LxH] [--players 1..4] [--from <carte.ldtk>] [--out <chemin dans assets/>] [--use]`.

- Les définitions LDtk (`defs` : tilesets, couches, entités) sont **recopiées d'une carte du jeu** (donneur : `--from`, sinon l'arène du bac à sable de `game.ron`, sinon `generate_template.map`,
  sinon la première carte) : les images se résolvent et les entités correspondent à celles du jeu (un gabarit d'un autre jeu casserait les tuiles et ferait refuser des `CharacterSpawn` inconnus
  par le lint). Les chemins de tileset sont recalculés pour le dossier d'écriture.
- Le niveau n'a que des `PlayerSpawn` ; la couronne de murs est écrite dans la couche IntGrid `Walls` **avec ses tuiles d'auto-règles** (calculées par les règles de la couche, validées contre
  celles que LDtk écrit) : les murs se voient, sans ouvrir LDtk.
- Écrit par défaut `maps/<nom>.ldtk` (ou le dossier du donneur si le jeu n'a pas de `maps/`) ; l'id de carte est le nom de fichier, unique dans le jeu. `--use` pose
  `sandbox: (arena: "…")` dans `game.ron` ; si le fichier n'est pas dans un dossier `Map` déclaré, la commande dit la ligne à ajouter à `content_folders`. Déterministe : la relancer ne change pas un octet.
- Autre taille : `--size 40x30` (le joueur apparaît alors ailleurs : la position d'une salle est tirée par la graine selon sa taille ; relire `alacod remote brief`).
- Un jeu qui a besoin d'une arène particulière (obstacles, ennemis fixes) en écrit une autre et la désigne par `sandbox.arena` ou par le champ `map` d'un profil (§86).

## 2.4 Lire et régler une valeur : `alacod get`, `alacod set`, `alacod undo`

Régler un jeu = changer **une** valeur d'un fichier RON de contenu. La commande le fait à la place de l'éditeur de texte et de `git` : texte exact conservé, lint, commit, annulation.

| Commande | Effet |
|---|---|
| `alacod get [jeu] <Kind> <id> <champ.pointé>` | affiche la valeur actuelle telle qu'elle est écrite (`"150.0"`) ; un bloc (struct, liste) est affiché avec la liste des réglages qu'il contient ; `--json` |
| `alacod set [jeu] <Kind> <id> <champ.pointé> <valeur> [--commit] [--raw] [--json]` | remplace la valeur, lance le lint, restaure le fichier s'il est rouge, commite si `--commit` |
| `alacod undo [jeu]` | annule le dernier commit `editor:` (rien d'autre), puis relance le lint |

`jeu` est un dossier de jeu (`games/zombies`) ; sans lui, le jeu du dossier courant. `Kind` est un kind de `game.ron` (`Character`, `Weapon`, `Item`, `Economy`, `Camera`, `Ui`…, `alacod describe`), `id` la clé de la table
(`pistolet`), le nom du fichier (`bottes`) ou, pour un personnage, son `asset_name_ref` (`zombie_1`). Pour un id inconnu, l'erreur donne le plus proche (la dictée déforme les ids) et la liste.

```bash
alacod get games/zombies Character zombie_1 movement.max_speed                  # "120.0"
alacod set games/zombies Character zombie_1 movement.max_speed 100 --commit
# Character zombie_1 movement.max_speed : 120.0 -> 100.0 (ZombieShooter/Sprites/Zombie/zombie_config.ron) ; effet : au prochain spawn (…) ; commit 1a2b3c4
alacod remote spawn zombie_1 --ahead && alacod remote step 60                    # constater (le jeu repart tout seul après le step)
alacod undo                                                                       # « annule ça » : revient à 120.0
```

- **Textuel, pas resérialisé.** La valeur est remplacée à sa place exacte : commentaires, ordre, indentation, lignes vides du fichier ne bougent pas.
- **Le type de l'ancienne valeur est gardé** : un `Fixed` reste une chaîne (`100` → `"100.0"`, la décimale d'origine est reprise), un entier reste un entier (`30.5` pour un nombre de frames est refusé), un booléen un booléen, une variante un nom
  (`Common` → `Rare`). `Some("500.0")` se règle par sa valeur (`500`) ; une valeur négative s'écrit telle quelle (`-5`).
- **Valeurs relatives** (nombre ou `Fixed` seulement) : `x0.9` ou `*0.9` (facteur), `+10%` / `-10%` (pourcentage, **signe obligatoire**), `+=5` / `-=5` (ajout). `+5` et `-5` restent des valeurs
  **absolues** (5 et −5) : seul `%` ou `=` après le signe rend la valeur relative. Le calcul part de la valeur écrite dans le fichier : `"0.15"` `x0.9` → `"0.135"` (au plus deux décimales de plus que
  l'ancienne, zéros de fin retirés, la virgule d'origine gardée : `"150.0"` `+10%` → `"165.0"`). Un entier reste un entier : un résultat fractionnaire est refusé avec les deux entiers voisins
  (`« x0.9 » sur l'entier 36 donne 32.4 : pas un entier ; donner 32 ou 33`). La ligne imprimée montre ancien -> nouveau. Barème « un peu » = 10 à 15 % : `x0.9`, `+10%`.
  `--commit`, `--raw` et `--json` peuvent suivre une valeur qui commence par `-` (`alacod set … -10% --commit`).
- **`--raw`** écrit la valeur telle quelle **à la place du nœud désigné**, bloc ou `Some(..)` compris : `alacod set … base_health.regen_rate None --raw`, `… grammar.rooms '(12, 16)' --raw`. Un RON mal formé est refusé avant d'écrire.
- **Chemins** : `movement.max_speed`, `config.firing_modes.default.bullet_type.Standard.damage` (clé de table sans guillemets, nom de variante facultatif : `…bullet_type.damage`), `modifiers.0.value` (liste par indice, ou par le nom
  d'une variante si elle est unique), `grammar.rooms.1` (tuple). Un chemin inconnu ou ambigu est refusé avec les chemins possibles ; un chemin qui désigne un bloc est refusé avec la liste des réglages qu'il contient.
- **Lint** : le lint de `alacod lint` tourne après l'écriture. Une erreur **nouvelle** restaure le fichier octet pour octet (« lint rouge … ; <fichier> restauré », code 1). Les erreurs qui existaient avant ne bloquent pas.
- **`--commit`** : `editor: <Kind> <id> <champ> <ancien> -> <nouveau>`, du **seul** fichier touché (`git commit --only` : le reste du travail en cours, modifié ou indexé, n'est pas pris). Refusé si ce fichier a déjà des changements non commités.
- **`undo`** : refuse tout dernier commit dont le sujet ne commence pas par `editor:` (y compris le `Revert "editor: …"` qu'il écrit : un second `undo` ne défait pas le premier) ; ne touche que les fichiers du commit, et refuse s'ils ont des changements non commités.
- **La ligne imprimée** (une seule) dit quoi, ancien -> nouveau, le fichier et **quand l'effet se voit** (§5) : `immédiat` (armes, caméra, interface), `au prochain spawn` (personnages : les entités vivantes gardent leurs valeurs),
  `après alacod remote restart` (le reste). Rien d'autre à lancer : le jeu relit les fichiers tout seul.
- Codes de sortie : 0 réussi (ou valeur déjà posée : « rien changé »), 1 sinon ; avec `--json`, une enveloppe sur stdout (`{"ok": false, "error": …}`).

## 2.5 Scripts et sorties machine : `alacod remote run`, `--json`, codes de sortie

- **Codes de sortie de `alacod remote`** (stables) : `0` réussi ; `1` erreur du jeu ou de la commande (garde `check_distance`, id inconnu… : message sur stderr) ; `2` usage (arguments mal formés, script mal écrit) ; `3` le jeu ne répond pas ; `4` assertion `expect`
  fausse ou attente `wait` expirée.
- **`--json`** (`alacod remote --json brief`) : un seul document sur stdout, `{"ok": true, "result": …}` ou `{"ok": false, "error": "…", "code": N}` (rien sur stderr). Le résultat est la réponse JSON du jeu ; pour `brief`, `state`, `discover` et `profile`, l'état complet.
- **`step N`** avance de N frames **puis rend la partie comme elle était** : relancée si elle tournait (mode éditeur et bac à sable : c'est le cas normal), en pause si elle l'était. Avant, le jeu restait figé après chaque `step` si on oubliait
  `resume`, et William ne pouvait plus jouer. `--keep-paused` la laisse en pause ; `--no-wait` rend la main tout de suite et la laisse en pause (`resume` ensuite). **Ne pas s'en servir en cours de partie.**
- **`alacod remote run <fichier> [--keep-going] [--expect-timeout S]`** joue un fichier de commandes, une par ligne, dans la syntaxe de `alacod remote` sans le préfixe (`alacod remote …` devant la ligne est toléré) : tout un monde de test
  (restart, give, spawn, modifier, step…) écrit une fois et rejoué. `#` et `//` commencent un commentaire. Trois verbes propres aux scripts :
  - `wait N [frames]` : attend N frames de jeu (partie en marche : on les regarde passer, un restart au milieu est toléré ; en pause : un `step`) ;
  - `sleep S` (`0.5`, `2s`, `500ms`) ;
  - `expect <mesure> <op> <valeur> [within S]` : assertion sur `alacod/state`, réévaluée jusqu'à `--expect-timeout` (2 s par défaut : le jeu applique un spawn à la frame suivante). Opérateurs `==`, `!=`, `<`, `<=`, `>`, `>=`. La mesure est un chemin du JSON
    de l'état (`frame`, `wave`, `clock.paused`) avec trois commodités : une liste vaut sa longueur (`enemies`, `players`, `windows`) ; `enemies[zombie_1]` compte les ennemis de ce nom (ou de cet état : `enemies[Chasing]`) ; `player.health`,
    `player.weapon.ammo`, `player.x` (ou `player[N].…`, N = handle) lisent un joueur.
  Tout le fichier est vérifié **avant** la première commande (`ligne 7 : …`, code 2) : une ligne mal formée ne laisse pas un monde à moitié posé. L'exécution s'arrête à la première ligne en échec (code de cette ligne, `--keep-going` pour continuer, le code
  reste celui du premier échec). Texte : une ligne `[n] commande` par commande jouée ; `--json` : un document avec une entrée par ligne.

  Exemple fourni, `games/zombies/scripts/monde_test.alacod` : repart de l'arène vide, donne le fusil, pose trois zombies devant le joueur et vérifie qu'ils sont là.

  ```bash
  alacod sandbox games/zombies --detach
  alacod remote run games/zombies/scripts/monde_test.alacod        # code 0 : le monde est comme prévu ; 4 : une assertion est fausse
  ```

## 3. Vocabulaire → champs RON

Les nombres `Fixed` sont **des chaînes** (`"150.0"`), les entiers (frames, prix, tailles de chargeur) sont des nombres : `alacod set` les écrit dans ce type (voir §2.4). Chemins relatifs à
`games/zombies/assets/` (vérifiés le 2026-10-22). Pour un autre jeu : `alacod describe games/<jeu>`.

**Du tableau à `alacod set`** : la colonne « Champ » est le chemin à donner (`.` à travers structs, tables, listes par indice, `Some(..)` et variantes sans les nommer). Le kind et l'id se lisent sur le fichier :
`characters/*.ron` (et les `*_config.ron` de zombies) → `Character <asset_name_ref ou nom du fichier>` ; `weapons.ron` → `Weapon <arme>` (le premier mot du champ est l'id : `Weapon pistolet config.firing_modes…`) ;
`objets/x.ron` → `Item x` ; `economy/economy.ron` → `Economy economy` ; `camera.ron` → `Camera camera` ; `ui/feedback.ron` → `Ui feedback` ; `floor_grammars/x.ron` → `FloorGrammar x` ; `items/powerups.ron` → `PowerUp powerups` ;
`mutations/x.ron` → `Mutation x`. Dans le doute : `alacod describe games/<jeu>` (kinds, fichiers, ids). Exemple : `alacod set games/gungeon Weapon pistolet config.firing_modes.default.firing_rate 4 --commit`.

| Ce que dit William | Fichier | Champ |
|---|---|---|
| vitesse du joueur | `ZombieShooter/Sprites/Character/player_config.ron` | `movement.max_speed` (150) |
| le joueur démarre / s'arrête plus vite | idem | `movement.acceleration` (3000), `movement.deceleration` (3000) |
| sprint | idem | `movement.sprint_multiplier` (2.0) |
| dash (portée, durée, recharge, invulnérabilité) | idem | `movement.dash_distance` (64), `dash_duration_frames` (8), `dash_cooldown_frames` (24), `dash_iframes` (6) |
| vie du joueur, régénération | idem | `base_health.max` (100), `regen_rate`, `regen_delay_frames` |
| vitesse d'un zombie | `ZombieShooter/Sprites/Zombie/zombie_config.ron` (et `zombie_hard_`, `zombie_full_`, `zombie_scaled_`, `boss_zombie_config.ron`) | `movement.max_speed` (120 pour `zombie_1`) |
| vie d'un zombie | idem | `base_health.max` (50) |
| dégâts, cadence, portée d'attaque d'un zombie | idem | `ai: Some((attack_damage, attack_cooldown_frames, attack_range, aggro_range))` |
| dégâts d'une arme | `ZombieShooter/Sprites/Character/weapons.ron` | `<arme>.config.firing_modes.<mode>.bullet_type.Standard.damage` (pistolet : 15.0) |
| cadence de tir | idem | `…firing_modes.<mode>.firing_rate` (tirs par seconde ; pistolet 2.0, mitrailleuse 10.0) |
| dispersion | idem | `…firing_modes.<mode>.spread` (pistolet 0.0005, mitrailleuse 0.15) ; fusil à pompe : `firing_mode: Shotgun(pellet_count, spread_angle)` |
| recul | idem | `…firing_modes.<mode>.recoil` |
| taille du chargeur | idem | `…firing_modes.<mode>.mag: Mag(mag_size, mag_limit)` |
| temps de recharge | idem | `…firing_modes.<mode>.reload_time_seconds` |
| arme de mêlée | `weapons/melee/melee_weapons.ron` | un bloc par arme (`alacod describe games/zombies MeleeWeapon`) |
| nombre de zombies par vague | `waves/wave_config.ron` | `base_enemies` (3), `enemies_per_wave` (2), `max_random_variance` (2) |
| zombies en même temps | idem | `max_concurrent_enemies` (4) |
| délai entre deux apparitions, entre deux vagues, préparation | idem | `spawn_interval_frames` (120), `min_wave_delay_frames` (600), `grace_period_frames` (180) |
| les vagues durcissent plus vite | idem | `health_multiplier_per_wave` (0.05), `damage_multiplier_per_wave` (0.03) |
| quels zombies sortent | idem | `wave_tiers[].enemy_probabilities` |
| points par kill, par coup, par réparation | `economy/economy.ron` | `kill_points` (60), `hit_points` (10), `repair_points` (10), `nuke_points` (400) |
| prix d'un perk | `economy/perks.ron` | `<perk>.price` (juggernog 2500) ; effet : `<perk>.modifiers` |
| prix d'une porte ou d'une arme murale | `maps/la_fonderie.ldtk` (champ `price` de l'entité) | édité dans LDtk, pas à la main (voir section « assets » ci-dessous) |
| flash d'impact | `ui/feedback.ron` | `hit_flash.frames` (4), `hit_flash.color` |
| secousse d'écran | idem | `shake.frames` (8), `shake.amplitude` (12.0) |
| hit-stop (arrêt sur image au coup) | idem | `hit_stop_frames` (2) |
| nombres de dégâts flottants | idem | `damage_numbers` (true) |
| volume d'un son | idem | `volumes.<son>` (dB) |
| caméra : suivi, zoom | `camera.ron` | `lerp_speed` (12.0), `min_zoom` (0.6), `max_zoom_out` (3.0), `default_player_zoom` (1.0), `player_padding` (100.0) |
| HUD : position, taille, couleur, police | `ui/hud.ron` | `widgets.<n>.anchor`, `offset`, `size`, `color`, `font_size` ; sources dans `content::ui::HUD_SOURCES` |

Un réglage relatif (« un peu plus vite ») : lire la valeur, changer de 10 à 15 %, l'écrire, le dire. Valeur nulle ou ×1000 : le lint ne juge pas
l'équilibre, c'est à Claude de refuser les absurdités.

Barème : « un peu » = 10 à 15 % de la valeur actuelle (« un peu plus vite » : ×1,1 à ×1,15 ; « un peu moins » : ×0,85 à ×0,9), « beaucoup » = ×2
(ou ÷2), « pas du tout » = on refuse ou on demande. Unités : vitesses en pixels par seconde (px/s) ; durées `*_frames` en frames d'un tick à 60 Hz
(60 frames = 1 s) ; `*_seconds` en secondes ; `firing_rate` en tirs par seconde ; dispersions en radians ; santé en points de vie (PV) ; sommes en
points (zombies) ou en sous (isaac, gungeon). Arrondir à une valeur lisible. Les chemins ci-dessous sont relatifs à `games/<jeu>/assets/` ; un champ
s'écrit `chemin.pointé` à travers les blocs du RON ; la colonne « Valeur actuelle » est le texte exact du fichier le 2026-10-22. Les tableaux
suivants sont les réglages de polissage ; pour tout autre : `alacod describe games/<jeu> [Kind]`.

### gungeon

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pistolero.ron` | `movement.max_speed` | `"150.0"` | px/s |
| il démarre et s'arrête plus vite | `characters/pistolero.ron` | `movement.acceleration` | `"3000.0"` | px/s² |
| la roulade va plus loin | `characters/pistolero.ron` | `movement.dash_distance` | `"64.0"` | px |
| la roulade dure plus longtemps | `characters/pistolero.ron` | `movement.dash_duration_frames` | `8` | frames |
| moins d'attente entre deux roulades | `characters/pistolero.ron` | `movement.dash_cooldown_frames` | `24` | frames |
| invulnérable plus longtemps en roulade | `characters/pistolero.ron` | `movement.dash_iframes` | `6` | frames |
| une charge de roulade en plus | `characters/pistolero.ron` | `dash.charges` | `Some(2)` | charges |
| les charges de roulade reviennent plus vite | `characters/pistolero.ron` | `dash.recharge_frames` | `Some(60)` | frames |
| plus de vie pour le joueur | `characters/pistolero.ron` | `base_health.max` | `"100.0"` | PV |
| le joueur se soigne plus vite | `characters/pistolero.ron` | `base_health.regen_rate` | `Some("10.0")` | PV/s |
| le pistolet fait plus mal | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.bullet_type.Standard.damage` | `"12.0"` | PV |
| le pistolet tire plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.firing_rate` | `"3.0"` | tirs/s |
| le pistolet est moins précis | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.spread` | `"0.001"` | radians |
| les balles du pistolet vont plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.bullet_type.Standard.speed` | `"380.0"` | px/s |
| le chargeur du pistolet est plus grand | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.mag.mag_size` | `12` | balles |
| le pistolet recharge plus vite | `weapons/weapons.ron` | `pistolet.config.firing_modes.default.reload_time_seconds` | `"0.5"` | s |
| le revolver fait plus mal | `weapons/weapons.ron` | `revolver.config.firing_modes.default.bullet_type.Standard.damage` | `"30.0"` | PV |
| le fusil à pompe tire plus de plombs | `weapons/weapons.ron` | `fusil_pompe.config.firing_modes.default.firing_mode.pellet_count` | `8` | plombs |
| la mitraillette tire plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.firing_rate` | `"10.0"` | tirs/s |
| la mitraillette part moins dans tous les sens | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.spread` | `"0.14"` | radians |
| les ennemis font plus mal (leurs tirs) | `weapons/weapons.ron` | `arsenal.config.projectiles.plomb.damage` | `"4.0"` | PV |
| le bullet kin va plus vite | `characters/bullet_kin.ron` | `movement.max_speed` | `"60.0"` | px/s |
| le bullet kin a plus de vie | `characters/bullet_kin.ron` | `base_health.max` | `"30.0"` | PV |
| le bullet kin tire plus souvent | `characters/bullet_kin.ron` | `ai.behaviors.Shoot.cooldown_frames` | `150` | frames |
| le bullet kin voit plus loin | `characters/bullet_kin.ron` | `ai.aggro_range` | `Some("500.0")` | px |
| le fuyard va plus vite | `characters/fuyard_kin.ron` | `movement.max_speed` | `"75.0"` | px/s |
| le shotgun kin a plus de vie | `characters/shotgun_kin.ron` | `base_health.max` | `"55.0"` | PV |
| le boss (gatling) a plus de vie | `characters/gatling.ron` | `base_health.max` | `"120.0"` | PV |
| le boss change de phase plus tôt | `characters/gatling.ron` | `boss.phases.0.until` | `"0.5"` (dans `Some(HealthBelow("0.5"))`) | fraction de vie |
| le boss tire plus souvent | `characters/gatling.ron` | `boss.phases.0.timeline.repeat` | `150` | frames |
| plus de salles par étage | `floor_grammars/etage.ron` | `grammar.rooms.0` (min), `grammar.rooms.1` (max) | `10`, `14` | salles |
| la boutique coûte plus cher (commun) | `economy/economy.ron` | `shop_prices.Common` | `15` | sous |
| la boutique coûte plus cher (rare) | `economy/economy.ron` | `shop_prices.Rare` | `50` | sous |
| les bottes donnent plus de vitesse | `objets/bottes.ron` | `modifiers.0.value` | `"0.25"` (`stat: MoveSpeed, op: Pct`) | +25 % |
| le chargeur rapide recharge plus vite | `objets/chargeur_rapide.ron` | `modifiers.0.value` | `"0.3"` (`stat: ReloadSpeed, op: Pct`) | +30 % |
| la pierre de soin soigne plus | `objets/pierre_de_soin.ron` | `effects.0.do.0` | `"20"` (dans `Heal("20")`) | PV |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| l'arrêt sur image au coup dure plus | `ui/feedback.ron` | `hit_stop_frames` | `2` | frames |
| le tir de pistolet est plus fort en volume | `ui/feedback.ron` | `volumes.shot_pistol` | `-12.0` | dB |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |
| la vue est plus large | `camera.ron` | `min_zoom` | `0.5` | zoom (plus petit = plus large) |

### isaac

Le joueur d'isaac est `characters/pleureur.ron` (`asset_name_ref: "player"`) ; 1 PV = ½ cœur. Pas de roulade ni de sprint.

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pleureur.ron` | `movement.max_speed` | `"120.0"` | px/s |
| il démarre plus vite | `characters/pleureur.ron` | `movement.acceleration` | `"3000.0"` | px/s² |
| plus de vie pour le joueur | `characters/pleureur.ron` | `base_health.max` | `"6.0"` | PV (demi-cœurs) |
| plus de temps d'invulnérabilité après un coup | `characters/pleureur.ron` | `hit_iframes` | `Some(60)` | frames |
| la bombe explose plus vite | `characters/pleureur.ron` | `bomb.fuse_frames` | `75` | frames |
| la bombe a plus de portée | `characters/pleureur.ron` | `bomb.radius` | `"56.0"` | px |
| la bombe fait plus mal | `characters/pleureur.ron` | `bomb.damage` | `"4.0"` | PV |
| les larmes font plus mal | `weapons/weapons.ron` | `larmes.config.firing_modes.default.bullet_type.Standard.damage` | `"3.5"` | PV |
| les larmes partent plus vite (cadence) | `weapons/weapons.ron` | `larmes.config.firing_modes.default.firing_rate` | `"3.0"` | tirs/s |
| les larmes vont plus vite | `weapons/weapons.ron` | `larmes.config.firing_modes.default.bullet_type.Standard.speed` | `"260.0"` | px/s |
| les larmes vont plus loin | `weapons/weapons.ron` | `larmes.config.firing_modes.default.range` | `"300.0"` | px |
| les larmes sont moins précises | `weapons/weapons.ron` | `larmes.config.firing_modes.default.spread` | `"0.001"` | radians |
| les ennemis font plus mal (leurs tirs) | `weapons/weapons.ron` | `arsenal.config.projectiles.plomb.damage` | `"1.0"` | PV |
| la mouche va plus vite | `characters/mouche.ron` | `movement.max_speed` | `"55.0"` | px/s |
| la mouche a plus de vie | `characters/mouche.ron` | `base_health.max` | `"4.0"` | PV |
| la mouche frappe plus fort | `characters/mouche.ron` | `ai.attack_damage` | `Some("1.0")` | PV |
| la mouche frappe plus souvent | `characters/mouche.ron` | `ai.attack_cooldown_frames` | `Some(120)` | frames |
| l'araignée va plus vite | `characters/araignee.ron` | `movement.max_speed` | `"66.0"` | px/s |
| le cracheur crache plus souvent | `characters/cracheur.ron` | `ai.behaviors.Shoot.cooldown_frames` | `300` | frames |
| le cracheur voit plus loin | `characters/cracheur.ron` | `ai.aggro_range` | `Some("130.0")` | px |
| le boss colosse a plus de vie | `characters/colosse.ron` | `base_health.max` | `"44.0"` | PV |
| le boss gardien a plus de vie | `characters/gardien.ron` | `base_health.max` | `"40.0"` | PV |
| plus de salles par étage | `floor_grammars/sous_sol.ron` | `grammar.rooms.0` (min), `grammar.rooms.1` (max) | `8`, `12` | salles |
| plus de chances d'une salle du diable | `floor_grammars/sous_sol.ron` | `grammar.layout.post_boss.chance` | `"0.33"` | probabilité |
| plus de chances d'une malédiction | `floors/sous_sol.ron` | `curses.chance` | `"0.3"` | probabilité |
| la boutique coûte plus cher (commun) | `economy/economy.ron` | `shop_prices.Common` | `15` | sous |
| la boutique coûte plus cher (rare) | `economy/economy.ron` | `shop_prices.Rare` | `50` | sous |
| le marché du diable coûte plus de cœurs | `rooms/diable.ron` | `shop.cost` | `Some(MaxHealth("2"))` | PV max |
| la pile rapide donne plus de cadence | `objets/pile_rapide.ron` | `modifiers.0.value` | `"0.1"` (`stat: FireRate, op: Pct`) | +10 % |
| le plomb fondu donne plus de dégâts | `objets/plomb_fondu.ron` | `modifiers.0.value` | `"1.0"` (`stat: Damage, op: Pct`) | +100 % |
| la lunette longue allonge la portée | `objets/lunette_longue.ron` | `modifiers.0.value` | `"0.15"` (`stat: Range, op: Pct`) | +15 % |
| l'œil vif accélère les larmes | `objets/oeil_vif.ron` | `modifiers.0.value` | `"0.2"` (`stat: ShotSpeed, op: Pct`) | +20 % |
| le réservoir vital donne plus de vie | `objets/reservoir_vital.ron` | `modifiers.0.value` | `"2"` (`stat: MaxHealth, op: Add`) | PV |
| l'élixir de vitesse dure plus longtemps | `objets/elixir_vitesse.ron` | `effects.0.do.0.frames` | `300` | frames |
| le cœur soigne plus | `objets/coeur.ron` | `effects.0.do.0` | `"2"` (dans `Heal("2")`) | PV |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |
| la vue est plus large | `camera.ron` | `min_zoom` | `0.6` | zoom (plus petit = plus large) |

### throne

Le joueur de throne est `characters/pilote.ron` (`asset_name_ref: "player"`, trois armes au départ : `mitraillette`, `revolver`, `lance_lames`).

| Ce que dit William | Fichier | Champ | Valeur actuelle | Unité |
|---|---|---|---|---|
| le joueur va plus vite | `characters/pilote.ron` | `movement.max_speed` | `"150.0"` | px/s |
| la roulade va plus loin | `characters/pilote.ron` | `movement.dash_distance` | `"64.0"` | px |
| moins d'attente entre deux roulades | `characters/pilote.ron` | `movement.dash_cooldown_frames` | `24` | frames |
| invulnérable plus longtemps en roulade | `characters/pilote.ron` | `movement.dash_iframes` | `6` | frames |
| plus de vie pour le joueur | `characters/pilote.ron` | `base_health.max` | `"100.0"` | PV |
| le joueur se soigne plus vite | `characters/pilote.ron` | `base_health.regen_rate` | `Some("10.0")` | PV/s |
| la mitraillette fait plus mal | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.bullet_type.Standard.damage` | `"8.0"` | PV |
| la mitraillette tire plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.firing_rate` | `"10.0"` | tirs/s |
| la mitraillette part moins dans tous les sens | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.spread` | `"0.15"` | radians |
| le chargeur de la mitraillette est plus grand | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.mag.mag_size` | `30` | balles |
| la mitraillette recharge plus vite | `weapons/weapons.ron` | `mitraillette.config.firing_modes.default.reload_time_seconds` | `"1.5"` | s |
| le revolver fait plus mal | `weapons/weapons.ron` | `revolver.config.firing_modes.default.bullet_type.Standard.damage` | `"15.0"` | PV |
| le fusil à pompe tire plus de plombs | `weapons/weapons.ron` | `fusil_a_pompe.config.firing_modes.default.firing_mode.pellet_count` | `8` | plombs |
| le canon à ricochet fait plus mal | `weapons/weapons.ron` | `canon_ricochet.config.firing_modes.default.bullet_type.Standard.damage` | `"20.0"` | PV |
| le rat va plus vite | `characters/rat.ron` | `movement.max_speed` | `"60.0"` | px/s |
| le chien a plus de vie | `characters/chien.ron` | `base_health.max` | `"20.0"` | PV |
| le chien court plus vite | `characters/chien.ron` | `movement.max_speed` | `"90.0"` | px/s |
| la brute a plus de vie | `characters/brute.ron` | `base_health.max` | `"120.0"` | PV |
| le buffle frappe plus fort | `characters/buffle.ron` | `ai.attack_damage` | `Some("12.0")` | PV |
| le buffle frappe plus souvent | `characters/buffle.ron` | `ai.attack_cooldown_frames` | `Some(90)` | frames |
| le rôdeur voit plus loin | `characters/rodeur.ron` | `ai.sight_range` | `Some("900.0")` | px |
| le cracheur crache plus souvent | `characters/cracheur.ron` | `ai.behaviors.Shoot.cooldown_frames` | `60` | frames |
| la tourelle a plus de vie | `characters/tourelle.ron` | `base_health.max` | `"150.0"` | PV |
| le boss (roi rat) a plus de vie | `characters/roi_rat.ron` | `base_health.max` | `"360.0"` | PV |
| le roi rat frappe plus fort | `characters/roi_rat.ron` | `ai.attack_damage` | `Some("15.0")` | PV |
| le roi rat tire plus souvent | `characters/roi_rat.ron` | `ai.behaviors.Shoot.cooldown_frames` | `120` | frames |
| plus d'ennemis au premier niveau | `caves/niveau_1.ron` | `enemy_spawns` | `4` | ennemis |
| plus d'ennemis au troisième niveau | `caves/niveau_3.ron` | `enemy_spawns` | `9` | ennemis |
| la caverne du premier niveau est plus remplie de roche | `caves/niveau_1.ron` | `fill_ratio` | `"0.38"` | fraction |
| la partie durcit plus vite | `difficulty.ron` | `value` | `"1 + floor * 0.25 + floor_minutes * 0.25"` | formule |
| plus de chances de bonus à la mort | `items/powerups.ron` | `drop_chance` | `"0.40"` | probabilité |
| la rage dure plus longtemps | `items/powerups.ron` | `powerups.rage.actions.0.frames` | `600` | frames |
| plus de chances de lâcher une arme | `progression/run.ron` | `weapon_drop_chance` | `"0.08"` | probabilité |
| plus de rads par mort | `progression/run.ron` | `per_kill` | `"1"` | rads |
| le vampire soigne plus | `mutations/vampire.ron` | `effects.0.do.0` | `"4"` (dans `Heal("4")`) | PV |
| points par mort | `economy/economy.ron` | `kill_points` | `60` | points |
| le flash d'impact dure plus longtemps | `ui/feedback.ron` | `hit_flash.frames` | `4` | frames |
| la secousse d'écran est plus forte | `ui/feedback.ron` | `shake.amplitude` | `12.0` | px |
| la caméra suit de plus près | `camera.ron` | `lerp_speed` | `12.0` | par seconde |

## 4. Règles pour Claude

1. **Une modification à la fois.** Lire le champ (`alacod get`), le régler (`alacod set`), constater, répondre.
2. **N'écrire que sous `games/<jeu>/assets`, par `alacod set`.** Jamais de Rust, de `Makefile`, de `tests/`, ni d'édition à la main d'un RON de contenu : `set` garde la mise en forme et le lint.
3. **Lint vert avant de dire « fait »** : `alacod set` lance le lint de `alacod lint` et **restaure le fichier** s'il apparaît une erreur nouvelle (il le dit, code 1). Une erreur qui existait déjà n'est pas imputée au changement (`set` la signale).
4. **Un commit `editor: <Kind> <id> <champ> <ancien> -> <nouveau>` par changement appliqué** : `alacod set … --commit` (seul le fichier touché est commité ; refusé si ce fichier avait déjà des changements non commités).
5. **« Annule ça »** : `alacod undo` annule le dernier commit **seulement si** son sujet commence par `editor:` (et ne touche que ses fichiers), sinon il le dit et ne fait rien.
6. **Dire en une phrase ce qui a changé et où** : la ligne de `set` le donne (`Character player movement.max_speed : 150.0 -> 135 (… player_config.ron) ; effet : …`). Dire aussi *quand* l'effet se voit (section 5).
7. Ne jamais pousser (`git push`), jamais `git stash`, jamais de commande destructive.
8. **Ne jamais laisser la partie en pause** : William joue. `alacod remote step` la relance seul quand elle tournait (§2.5) ; après un `pause`, `alacod remote resume` ; avant de répondre, `alacod remote brief` doit montrer `pause=False`.
9. **Les preuves d'un agent tournent sur un jeu à part**, lancé sur un autre port (`ALACOD_REMOTE_PORT=15703 alacod sandbox <jeu> --detach`, puis `ALACOD_REMOTE_PORT=15703 alacod remote …`), jamais sur la partie de William ; et jamais `alacod set --commit` ni `alacod undo` dans le dépôt de William (ils commitent ou annulent dans sa branche).
10. **Toujours regarder la capture prise comme preuve** (`alacod remote screenshot <png>`) : ouvrir l'image avant de conclure, un fichier écrit ne prouve rien.
11. **Toujours les commandes `alacod`** (`get`, `set`, `undo`, `describe`, `lint`, `remote …`) : jamais d'édition à la main d'un RON que `alacod set` sait régler, ni d'appel direct au remote (`curl`).

### Bloc prêt à coller

À mettre dans le `CLAUDE.md` du dépôt ou dans le prompt système du client vocal (16 points, lisibles à voix haute ; ils décrivent toute la boucle : lire, régler, constater, annuler, scripter) :

```
Tu règles un jeu alacod en marche (alacod sandbox ou alacod editor) pour William, qui parle : tes réponses sont lues à voix haute. Tout passe par la commande alacod (alacod --help liste tout), jamais par un script ni make.
1. Une seule modification à la fois : alacod get games/<jeu> <Kind> <id> <champ> lit la valeur (docs/editeur-vocal.md, section 3 : fichier, kind et champ ; alacod describe games/<jeu> liste kinds et ids), alacod set games/<jeu> <Kind> <id> <champ> <valeur> --commit l'écrit.
2. N'écris que par alacod set : jamais à la main, jamais de Rust, de Makefile ni de tests. Il garde la mise en forme du fichier, lance le lint, restaure le fichier si le lint devient rouge, et commite « editor: … » du seul fichier touché. Si set refuse (chemin inconnu ou ambigu), il liste les chemins possibles : choisis, ou demande.
3. La ligne que set imprime dit quoi, ancien -> nouveau, et quand l'effet se voit (tout de suite, au prochain spawn, ou après alacod remote restart). Répète-le en une phrase, et vérifie par alacod remote brief quand c'est visible.
4. Bac à sable (docs/editeur-vocal.md, 2.2), sans commit ni fichier : « donne-moi X » = alacod remote give --weapon X ou --item X ; « mets-moi 500 sous » = give --set-currency 500 ; « charge le profil P » = profile P ; « spawne X devant moi » = spawn X --ahead ; « reset » = restart.
5. Un modificateur vivant (alacod remote modifier --handle 0 --source editor STAT Mul 1.1) n'est pas un commit ; il s'annule par alacod remote unmodify --handle 0 --source editor. Il disparaît au restart : dis-le.
6. « Annule ça » : alacod undo (il refuse tout dernier commit qui n'est pas « editor: » : dis-le alors, ne touche à rien). Si le dernier changement était un modificateur vivant, unmodify.
7. Ne laisse JAMAIS la partie en pause : William joue. alacod remote step relance la partie seul après coup ; n'utilise ni --keep-paused ni --no-wait en cours de partie ; après un pause, fais alacod remote resume. Avant de répondre, alacod remote brief doit afficher pause=False.
8. Un essai en plusieurs étapes (monde, spawns, vérifications) s'écrit dans un fichier de commandes et se joue par alacod remote run <fichier> (docs/editeur-vocal.md, 2.5 ; exemple games/zombies/scripts/monde_test.alacod). Codes de sortie : 0 réussi, 1 erreur, 3 jeu éteint, 4 assertion fausse ; --json pour lire le résultat.
9. « Un peu » = 10 à 15 %, « beaucoup » = fois deux : écris la valeur relative (alacod set … x0.9, +10%, -10%, +=5), ne calcule pas à la main. Refuse une valeur nulle ou absurde.
10. Phrase ambiguë ou identifiant incertain : pose une question courte, ou liste les ids (alacod describe) et propose le plus proche.
11. Réponses courtes : deux phrases, sans tableau, sans code, sans adresse.
12. Jamais git push, jamais git stash, jamais de commande destructive.
13. Jeu éteint (alacod remote : « le jeu ne répond pas », code 3) : relance alacod sandbox games/<jeu> --detach, ne tente pas autre chose.
14. Tes preuves d'agent tournent sur un jeu à part, sur un autre port (ALACOD_REMOTE_PORT=15703 devant alacod sandbox et alacod remote), jamais sur la partie de William ; jamais alacod set --commit ni alacod undo dans le dépôt de William.
15. Une capture prise comme preuve (alacod remote screenshot) se regarde toujours avant de conclure.
16. Toujours les commandes alacod : jamais d'édition à la main d'un RON que alacod set sait régler.
```

### Deux sortes de changement

| | Modification de fichier | Modificateur vivant |
|---|---|---|
| Commande | `alacod set games/<jeu> <Kind> <id> <champ> <valeur> --commit` (RON sous `games/<jeu>/assets`, commit `editor: …`) | `alacod remote modifier --handle 0 MoveSpeed Mul 0.9 --source editor` |
| Dure | tant que le fichier ne change pas : survit au `restart` et à la relance | jusqu'au `restart` (le nouveau personnage repart sans modificateur) |
| S'annule par | `alacod undo` (si le dernier commit est un `editor:`) | `alacod remote unmodify --handle 0 --source editor` |
| Usage | réglage voulu durable | essai immédiat sur le joueur vivant |

Pour « c'est un peu trop rapide » : `alacod remote modifier --handle 0 MoveSpeed Mul 0.9 --source editor` donne l'effet tout de suite, puis l'édition du RON rend le réglage durable. Un changement qui n'existe que comme modificateur disparaît au `restart` : le dire.

## 5. Ce qui se recharge à chaud

Mesuré sur `make editor GAME=zombies HEADLESS=1` (2026-10-22) : une édition de `weapons.ron` donne au journal
`INFO game::weapon_assets: armes rechargées : WeaponRegistry mis à jour (4 armes)` en moins de 4 s.

| Donnée | Effet | Preuve (plan § 2) |
|---|---|---|
| tables de personnages (`*_config.ron`) → `CharacterConfigs` | **immédiat** pour les spawns futurs ; les entités vivantes gardent leurs `Stats` (copiées au spawn : « plus rapide » dans le RON ne change pas le joueur vivant → `alacod remote modifier`, ou `restart`) | `character/config_asset.rs:44` |
| armes (`weapons.ron`) → `WeaponRegistry` et armes portées | **immédiat** (armes portées : `weapons_config_update_system`, `Update`) ; la mêlée n'est pas couverte (risque 7) | `weapon_assets.rs:55-56`, `:179-199` |
| `camera.ron`, `ui/feedback.ron`, `ui/hud.ron` | **immédiat** (présentation) | `camera/mod.rs:666-681`, `feedback.rs:71-86`, `ui/hud.rs:1750-1763` |
| `Registry` et `GameManifest` (items, salles, curses, patterns, statuts, progression, économie, perks, `game.ron`) | rechargé hors lobby **seulement en mode éditeur** (K5-1, session locale à `check_distance` 0 ; hors mode éditeur, en ligne ou à `check_distance` > 0, aucun remplacement hors lobby, `warn!` « registre non rechargé : <cause> ») : événement d'asset **et** scrutation des fichiers du contenu, une fois par seconde (natif ; la scrutation n'existe qu'en mode éditeur) ; les tables dérivées prennent effet au prochain `GameLoading` : `alacod remote restart --kind Restart` ou changement d'étage (mode `Floors`). |
| balance, power-ups, vagues (`ResolvedBalance`, `PowerUpsTable`) | prennent effet au prochain `GameLoading` : `alacod remote restart --kind Restart` ou changement d'étage (mode `Floors`) | `core.rs:166-176`, `balance.rs:5-26` |
| RON **invalide** | aucun rechargement : `ERROR contenu invalide, registre non rechargé` (scrutation) ou `Failed to load asset … Could not parse RON` (asset) au journal, la table en mémoire reste l'ancienne ; la correction suivante recharge | mesuré 2026-10-22 (écriture d'un RON invalide pendant que le jeu tourne) |

### Assets : images, sons, LDtk, polices

Audit complet : `docs/taches/rapports/k5-audit-hot-reload-assets.md`. En bref :

- **PNG des sprites** : immédiat (les sprites gardent leur handle ; Bevy recharge l'image). Garder le même chemin.
- **Grilles et animations** (`sprites/sprites.ron`, configs d'animation) : immédiat (layouts reconstruits sur `Modified`).
- **Sons** : à la prochaine lecture (prochain tir, coup…) ; le son en cours n'est pas remplacé.
- **LDtk et tilesets** (`maps/*.ldtk`, `wall_art`) : au `restart` seulement ; le PNG d'un tileset est immédiat, pas une nouvelle grille.
- **HUD, caméra, feedback** : immédiat (arbre UI reconstruit, paramètres recopiés sur `Modified`).
- **Polices** : suivies par handle, rechargées par Bevy.

## 6. Voix

- **Entrée** : la dictée du système. macOS : dictée (touche micro ou `Fn Fn`) dans le terminal de Claude Code. Rien à installer. La dictée écrit sans
  ponctuation ni majuscules et déforme les identifiants (`zombie_runner`) : Claude **liste** les ids (`alacod describe games/<jeu>`) et propose le
  plus proche au lieu de deviner. Un client vocal complet (Hermes ou autre) sert aussi d'entrée : il héberge Claude, section ci-dessous.
- **Sortie** : hook `Stop` de `.claude/settings.json` → `scripts/claude-say.sh`, **opt-in**. Sans `ALACOD_VOICE=1` il ne fait rien.
  Variables relues dans le script le 2026-10-22 (toutes présentes) :
  - `ALACOD_VOICE=1 claude` : à la fin de chaque réponse, la dernière réponse (blocs de code remplacés par « bloc de code », code en ligne, liens et
    marques Markdown retirés) est lue par `say` (macOS), sinon `espeak-ng`, sinon `espeak` (Linux, voix `fr`), sinon `piper` (avec
    `ALACOD_VOICE_PIPER_MODEL`), sinon écrite dans `$TMPDIR/alacod-voice.txt` (`/tmp` par défaut).
  - `ALACOD_VOICE_NAME=Thomas` : voix de `say` (macOS seulement) ; ignorée si la voix n'est pas dans la liste de `say -v '?'` (voix par défaut du
    système alors).
  - `ALACOD_VOICE_MAX=300` : longueur maximale lue en caractères (600 par défaut), coupée à la dernière phrase entière ; une valeur illisible redonne 600.
  - `ALACOD_VOICE=stop scripts/claude-say.sh` : coupe la lecture en cours, par `pkill say` : **macOS seulement**. Sous Linux : `pkill espeak-ng`
    (ou `pkill espeak`, `pkill piper`).
  - Tests du script : `python3 -m unittest scripts/tests/test_claude_say.py -v`.
- **Installer une voix française.**
  - macOS : Réglages Système > Accessibilité > Contenu énoncé > Voix du système > Gérer les voix, télécharger une voix française (Thomas, Amélie) ;
    vérifier : `say -v '?' | grep fr_FR`, essai : `say -v Thomas "bonjour"`, puis `ALACOD_VOICE_NAME=Thomas`.
  - Linux (Debian, Ubuntu) : `sudo apt install espeak-ng` ; essai : `espeak-ng -v fr "bonjour"`. La voix est celle de `fr`, robotique mais suffisante.
    Pour mieux : `piper` avec un modèle français, `ALACOD_VOICE_PIPER_MODEL=<fichier.onnx>`.
- **Boucle micro / voix.** La sortie lit la dernière réponse de Claude ; si le micro reste ouvert, il se transcrit lui-même et Claude répond à sa propre
  voix. Règle pour William : **couper la dictée pendant la lecture**, la rouvrir quand elle s'arrête (ou `ALACOD_VOICE=stop scripts/claude-say.sh`).
  Réponses courtes (section 4) : la lecture dure quelques secondes.
- **Client vocal complet** (Hermes ou autre) : il lui suffit d'un shell et de HTTP local ; pas de pont MCP (plan D5), sauf si William répond que son
  client n'a ni l'un ni l'autre (décision en attente : `docs/taches/rapports/k5-4-essai-vocal.md`). S'il n'a **que HTTP** : appeler `POST
  http://127.0.0.1:15702` en JSON-RPC (`rpc.discover` liste les méthodes `alacod/*`) et éditer les RON par l'outil de fichiers du client.
- **Client sur une autre machine.** Le serveur de la partie ne répond qu'à `127.0.0.1` (volontaire : on n'ouvre pas le port). Depuis la machine du
  client, tunnel SSH : `ssh -L 15702:127.0.0.1:15702 <machine>` (laisser la session ouverte) ; `alacod remote` et `curl http://127.0.0.1:15702`
  parlent alors à la partie distante. Les fichiers RON restent sur la machine du jeu : Claude doit y tourner (ou y avoir un shell par SSH).

## 7. Limites

- **Local seulement** : session locale à `check_distance` 0 (pas de synctest), un joueur. Pas de p2p (une édition à chaud divergerait entre pairs, sans canal pour la répliquer), pas de web (ni file watcher ni remote en wasm).
- Les commandes de partie (`restart`, `set_modifier`, `clear_modifiers`, `reload_content`) sont refusées si la session est en ligne ou si `check_distance` ≠ 0 (le mode éditeur le force à 0 ; erreur JSON-RPC qui nomme la cause).
- Aucune vérification de rollback en mode éditeur : un réglage qui casserait le déterminisme ne se voit qu'aux scénarios (`make test_scenarios`).

## 8. Dépannage

Dans l'ordre : le plus fréquent d'abord. Chaque cas donne le signe, la cause, la commande.

- **Le jeu ne répond pas** (`alacod remote` : « le jeu ne répond pas sur 127.0.0.1:15702 », connexion refusée, ou délai dépassé). Le jeu n'est pas lancé, ou pas par `alacod sandbox`/`alacod editor`, ou le port
  est autre. Tester : `curl -s -X POST -d '{"jsonrpc":"2.0","id":1,"method":"rpc.discover"}' http://127.0.0.1:15702`. Une réponse JSON : le jeu
  est là, le souci est ailleurs. Rien : relancer `alacod sandbox games/<jeu> --detach` (ou `alacod editor games/<jeu> --detach`) ; si `ALACOD_REMOTE_PORT` est posé, utiliser ce port. Le serveur n'écoute
  que `127.0.0.1` : depuis une autre machine, il faut le tunnel SSH de la section 6.
- **Commande refusée** (erreur qui nomme `online` ou `check_distance=N`). Les commandes de partie ne valent qu'en session locale à `check_distance`
  0 (section 7). Cause : jeu lancé en ligne, ou avec `ALACOD_CHECK_DISTANCE=2` dans l'environnement. Remède : relancer par `alacod sandbox`/`alacod editor`, sans
  `ALACOD_CHECK_DISTANCE`.
- **La partie est figée** (`alacod remote brief` : `pause=True`, plus rien ne bouge alors que William joue). Un `step` ou un `pause` l'a laissée en pause. `alacod remote resume` la relance. `alacod remote step` la relance
  seul quand elle tournait avant (§2.5) ; seuls `--no-wait` et `--keep-paused` la laissent en pause : ne pas s'en servir en cours de partie.
- **RON invalide** (journal : `Failed to load asset … Could not parse RON`). Le jeu garde l'ancienne valeur en mémoire et ne recharge rien. Lire
  l'erreur (fichier, ligne), corriger, ou revenir en arrière avec `git checkout -- <fichier>` (fichier non commité seulement). La correction suivante
  recharge d'elle-même. Les nombres `Fixed` sont des chaînes (`"150.0"`), pas des nombres nus.
- **`reload` avec `errors` non vide** (`alacod remote reload`). Le registre n'est pas remplacé : l'ancien reste. Lire chaque erreur (un id inconnu, un champ mal écrit), corriger le fichier nommé, relancer `alacod remote reload` jusqu'à `errors: []`, puis `alacod remote restart --kind Restart` pour appliquer les tables dérivées. La commande échoue (code 1) si `errors` n'est pas vide.
- **Rien ne change à l'écran.** Relire la section 5 : une édition de registre ou de balance prend effet au prochain `GameLoading` (restart ou changement d'étage), tandis que les tables de personnages concernent les prochains spawns et que les armes, caméra et feedback sont immédiats. Utiliser `alacod remote reload`, puis `alacod remote restart --kind Restart` si nécessaire. `alacod remote input ""` relâche l'input distant et rend la main au clavier ; `alacod remote spawn` pose un personnage, `give` un objet ou une arme (§2.2). Vérifier aussi que le fichier modifié est bien sous `games/<jeu>/assets` **du jeu lancé** (`alacod remote brief` : la première ligne donne le jeu) et que le journal montre le rechargement (`armes rechargées` pour `weapons.ron`).
- **Lint rouge.** Lire le message de `alacod lint games/<jeu>` (le hook asynchrone rend la main avant la fin : attendre son verdict ou lancer la
  commande soi-même). Corriger le champ nommé ou restaurer le fichier ; ne pas committer `editor:` tant que le lint n'est pas vert.
- **La voix ne lit rien.** Vérifier `ALACOD_VOICE=1` dans l'environnement de `claude` (le hook n'agit pas sans), puis qu'une commande de synthèse
  existe (`say`, `espeak-ng`, `espeak`, `piper` avec `ALACOD_VOICE_PIPER_MODEL`) ; sans elles, le texte est écrit dans `$TMPDIR/alacod-voice.txt`.
- **Le micro transcrit la voix de Claude.** Couper la dictée pendant la lecture (section 6) ; `ALACOD_VOICE=stop scripts/claude-say.sh` arrête la lecture.
