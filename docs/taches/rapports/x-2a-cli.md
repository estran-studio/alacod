# X-2a — CLI : dépendance et lancement

Branche : `x-2a-cli-dependance-lancement`  
Base de reprise : `25af5b5f` (`origin/main`)  
Journal de reprise : `journaux/b2-20261026-192250.md` (travail interrompu sans réponse finale)

## Périmètre livré

- `alacod new` copie le gabarit depuis l’engine choisi et écrit par défaut les dépendances des crates de l’engine avec l’URL git et le dernier tag (`git tag --sort=-v:refname`). `--tag`, `--rev`, `--url` et `--path` permettent les cas explicites, dont le test hors ligne par chemin.
- `alacod engine status`, `pin` et `local` lisent/réécrivent les dépendances engine. `local` écrit un patch Cargo local dans `.cargo/config.toml`, ajouté au `.gitignore`; `pin` retire cette surcharge. La racine réellement résolue est obtenue par `cargo metadata` (avec le dossier du jeu comme répertoire courant afin de charger sa configuration).
- `alacod run` lance `cargo run` dans le dossier du jeu, avec `--sandbox [profil]`, `--headless`, `--seed`, `--players` et les arguments après `--`. Le profil headless est redéclaré pour le workspace autonome du jeu et `ALACOD_HEADLESS=1` est hérité.
- `alacod remote` est un client Rust JSON-RPC/HTTP des méthodes BRP existantes `alacod/*`. Il reprend les commandes de `scripts/alacod-remote` (état, résumé, input, avance, pause/reprise, capture, sauvegarde, découverte, restart, modifier, reload, profils, dons et spawns) sans second protocole. Le script Python reste conservé pour piloter les jeux de l’engine pendant la coexistence.
- D126 : le personnage est porté par `PlayerScript`, `SandboxPlayer` et `PlayerProfile`, puis choisi à la création du joueur.
- D130 : `alacod/give` est enregistré par `InputRecorder`, sérialisé dans `Scenario::gifts` et rejoué dans la simulation via une commande différée; preuve unitaire/synctest dans `crates/scenario/tests/gifts.rs`.
- D131 : `entry.player` et `GameManifest::player_character()` remplacent le personnage par défaut codé en dur dans la création, le lint et la lisibilité.

## Conventions et documentation

- Contrat CLI : `docs/conventions.md` §89.
- Mode d’emploi : `docs/jouer.md`, `templates/starter/CLAUDE.md`, et section CLI de `CLAUDE.md`.
- Journal : ligne X-2a dans `docs/taches.md`.
- Dettes réservées au plan : D144 (`set_modifier`/`clear_modifiers` non enregistrés), D145 (don rejoué dans `Effects`, donc ordre intra-frame différent d’un don à chaud), D146 (gabarit encore lu depuis le checkout de compilation), D147 (`Cargo.lock` non mis à jour par `pin`/`local`) et D148 (résolution metadata susceptible de tenter le réseau sans mode/délai explicite). Elles restent ouvertes; elles ne sont pas présentées comme fermées.

## Vérifications

- `cargo check -p content --bin alacod` : vert sur cette reprise (2,6 s, cache chaud ; seuls des avertissements préexistants : `effects::runtime` pattern unreachable, champs de schémas non lus). Inclut le correctif de reprise : `cargo metadata` tourne dans le dossier du jeu (sinon le patch `.cargo/config.toml` d'`engine local` n'est pas chargé quand le jeu est passé en argument depuis ailleurs) et le manifeste est canonicalisé.
- `cargo test -p content --test cli` : **18 passés, 2 ignorés** (6,08 s, sur cette reprise). Couvre `new` par tag, `--tag`/`--rev`/`--url`, refus sans tag, `engine pin`/`local`/`status`, racine par `cargo metadata`, `run` contre un faux `cargo`, `remote` contre un faux serveur BRP.
- `cargo test -p scenario --test gifts` : **non exécuté sur cette reprise** (compilation du paquet `scenario` > 3 600 s sur le VPS partagé, délai dépassé ; relancé, pas de résultat). Le fichier `crates/scenario/tests/gifts.rs` est livré ; la preuve synctest du rejeu D130 reste à rejouer par la vérification standard au merge.
- Aucune trace de scénario existante n'a été bénie ni réécrite (`BLESS` jamais passé).
- Smoke coûteux non rejoué ici (`new` + `engine local` + `run --sandbox --headless` + `remote` + `engine pin` via `make test_alacod_cli` / test ignoré dédié, ~10 min à froid) : listé comme non vérifié, aucun résultat non observé revendiqué.

## Risques connus

- Le premier build d’un jeu dépendant d’un tag git peut nécessiter réseau et téléchargement du checkout Cargo; `--path` reste le repli hors ligne explicite.
- `alacod test` continue de lancer `alacod-gen` de l’engine (compatibilité X-2a); l’exécution du runner dans le binaire du jeu et l’écriture complète dans le dépôt externe relèvent de X-2b/X-1.
- Le client Rust est volontairement local (`127.0.0.1`); pour une partie distante, conserver le tunnel SSH décrit dans `docs/editeur-vocal.md`.
- Les dettes D144–D148 ci-dessus sont des limites connues, non masquées par le rapport.

Aucun nom, contenu ou réglage du jeu externe n’a été ajouté à l’engine.
