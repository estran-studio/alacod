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

- `cargo check -p content --bin alacod` : vert (avertissements préexistants seuls). `cargo metadata` tourne dans le dossier du jeu (sinon le patch `.cargo/config.toml` d'`engine local` n'est pas chargé quand le jeu est passé en argument depuis ailleurs) et le manifeste est canonicalisé.
- `cargo test -p content --bin alacod --test cli` (second passage, après la revue) : **12 tests unitaires + 20 tests CLI passés, 2 ignorés** (`#[ignore]` : `alacod test` et le smoke complet).
- `cargo test -p scenario --no-default-features --test gifts` : **2 passés** (mode headless : sans `--no-default-features` le test sort tout de suite sur `RENDER_ENABLED`). Il a aussi révélé que la première livraison **ne compilait pas** `scenario` (`Scenario` sans `gifts` dans `generate.rs`) : corrigé. `cargo check -p scenario --no-default-features --tests` vert.
- `cargo test -p game --no-default-features --lib -- grant:: recording:: replay::` : **7 passés**.
- e2e `new_game_runs_its_sandbox_and_answers_remote_through_alacod_alone` : **passé sur le Mac b0** (1 254 s) sur `a854ec4` (relevé de Main) ; non rejoué sur ce VPS.
- Aucune trace de scénario existante n'a été bénie ni réécrite (`BLESS` jamais passé).

## Corrections de la revue indépendante (bloquants)

1. `grant.rs` : le rejeu d'un don tourne dans `GgrsSchedule` ; les deux destructions d'arme (emplacement plein, mêlée remplacée) passent par `despawn_rollback()` (import `RollbackDespawnCommandExtension`), plus par `despawn()`.
2. `gifts.rs::recorded()` : `to_scenario` déduit les joueurs des inputs enregistrés (aucun ici) et rendait `players: []` : le test rétablit les joueurs du scénario de base (assertion non vide) et l'exécute réellement (headless).
3. `alacod engine` lit et réécrit `Cargo.toml` par `toml_edit` (nouvelle dépendance de `content`, déjà dans `Cargo.lock` en 0.25.15) : `[dependencies.content]`, table en ligne multiligne (TOML 1.1) ou tableau `features` multiligne, `dev-/build-dependencies`, `[target.'cfg()'.dependencies]` ; commentaires et ordre des clés conservés quand tag→tag ou rev→rev. Tests : `every_valid_toml_form_of_the_dependency_is_understood`, `a_table_form_dependency_can_switch_from_tag_to_revision`, `a_missing_or_malformed_dependency_is_reported`, `the_local_override_path_is_read_back_from_the_config_block` (unitaires) et `engine_commands_accept_table_form_and_multiline_dependencies` (CLI).
4. `alacod remote reload` : le JSON va sur stdout et la commande sort en code 1 si `errors` n'est pas vide, comme `scripts/alacod-remote` (tests `remote_speaks_the_alacod_methods_of_the_game` et `remote_reload_succeeds_with_the_json_on_stdout_when_the_content_is_valid`).

## Risques connus

- Le premier build d’un jeu dépendant d’un tag git peut nécessiter réseau et téléchargement du checkout Cargo; `--path` reste le repli hors ligne explicite.
- `alacod test` continue de lancer `alacod-gen` de l’engine (compatibilité X-2a); l’exécution du runner dans le binaire du jeu et l’écriture complète dans le dépôt externe relèvent de X-2b/X-1.
- Le client Rust est volontairement local (`127.0.0.1`); pour une partie distante, conserver le tunnel SSH décrit dans `docs/editeur-vocal.md`.
- Les dettes D144–D148 ci-dessus sont des limites connues, non masquées par le rapport.

Aucun nom, contenu ou réglage du jeu externe n’a été ajouté à l’engine.
