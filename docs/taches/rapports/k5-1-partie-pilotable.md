# k5-1-partie-pilotable : rapport

Branche `k5-1-partie-pilotable` (depuis `k5-1-wip`, fusionnée avec `k5-0-1-editeur` pour récupérer la fiche). Aucun système dans `GgrsSchedule`, aucune ressource rollback : **0 trace modifiée**.

## Fait

- `editor_mode` : `EditorMode`, `hot_edit_allowed`, `session_hot_edit_allowed` (`online` testé avant la lecture de `check_distance`).
- `remote.rs` : mode éditeur (partie en marche, clavier conservé, `alacod/input` prend la main par handle, `input` vide la rend) ; `alacod/restart`, `set_modifier`, `clear_modifiers`, `reload_content` ; `alacod/state` enrichi. Toutes les écritures passent par `guard_hot_edit` (refus `INVALID_REQUEST` avec la cause).
- `content_hot_reload` : `Registry` et `GameManifest` rechargés hors lobby (événements d'assets) et scrutation des mtime (1 s, natif, **mode éditeur seulement**). Hors lobby, remplacement automatique seulement si `session_hot_edit_allowed` est `Ok` (sinon `warn!` une fois « registre non rechargé : <cause> »). `reload_registry_now` relève l'empreinte avant la lecture.
- `scripts/alacod-remote` : `restart`, `modifier`, `unmodify`, `reload`, `brief` ; correction : `input ""` n'envoyait jamais « relâcher » (le script ajoutait toujours `pan 0,0`) ; `--pan` n'est plus envoyé par défaut.
- Revue (commit `0b9db79a`) : P2-1 à P2-4, P3-1 à P3-3 traités ; exceptions exactes dans `scripts/determinism-allowlist.json` pour les `Instant` de la scrutation et de ses tests.
- Docs : `docs/editeur-vocal.md`, `CLAUDE.md`, `docs/conventions.md` §84, docstring de `alacod-remote`.

## Preuves (sorties réelles)

- Tests unitaires `game --lib` (`editor_mode`, `content_hot_reload`, `remote::`) : 12 passés, 1 ignoré (coût).
- `cargo test -p scenario --test editor_reload` : `registry_reloads_outside_lobby_and_restart_rederives_tables`, `set_modifier_slows_the_player_and_rejects_bad_params`, `registry_is_not_polled_without_editor_mode` : **3 passés**.
- `make test_crates` : sortie 0, aucun échec.
- `make test_scenarios` : **la suite complète** a été jouée (le `ALACOD_SCENARIO=…` en variable d'environnement n'est pas lu par le Makefile, qui attend `SCENARIO=`) : 809 lignes de scénario, sortie 0, `test result: ok. 5 passed; 0 failed; 12 ignored`, 7485 s ; `git status` : aucun `.trace` modifié (inclut `idle`, `four_players_shooting`, `clone_solo`, `remote_first_fight`, `throne_solo`, `gungeon_full_run`, `isaac_full_run`).
- Preuve headless (`make editor GAME=zombies HEADLESS=1`, 2026-10-23) :
  - `discover` : `alacod/clear_modifiers`, `input`, `pause`, `reload_content`, `restart`, `resume`, `save_recording`, `screenshot`, `set_modifier`, `state`, `step`.
  - `brief` : `editor=True online=False check_distance=0 run.step=Playing { since_frame: 0 } jeu=zombies carte=floor:la_fonderie graine=991772655`, `pause=False` sans `resume`.
  - `input Right` : joueur de x=-72 à x=213,8 en quelques secondes ; `input ""` → `released`, le joueur reste à x=214.
  - `restart --seed 7` → `{"request": "Restart", "seed": 7}` ; `state` : `seed 7`, `frame` repartie de 0 (`sortie d'InGame : 330 entité(s) rollback détruite(s), session et état de run remis à zéro`).
  - Vitesse (frames/px mesurés sur deux relevés `state`) : 2,5 px/frame sans modificateur, 2,248 avec `modifier --handle 0 MoveSpeed Mul 0.9` (−10 %) ; `unmodify` → `{"modifiers": 0, "removed": 1}`.
  - `reload` après `drop_chance: "0.15"` → `"0.16"` dans `items/powerups.ron` : `{"characters": 6, "errors": [], "melee": 6, "reloaded": true, "waves": 1, "weapons": 4}` ; fichier restauré par `git checkout --`.
  - RON invalide (`(((` dans `waves/wave_config.ron`) : `ERROR bevy_asset::server: Failed to load asset 'waves/wave_config.ron' … Could not parse RON: 1:2: Expected identifier`, puis `ERROR game::content_hot_reload: contenu invalide, registre non rechargé : waves/wave_config.ron: erreur RON : 1:2: Expected identifier` (et `game.ron: champ entry.mode = Waves : aucun dossier de contenu « Wave » déclaré`) ; partie toujours `Playing` ; après `git checkout --` : `contenu rechargé (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues)`.
  - Refus avec `ALACOD_CHECK_DISTANCE=2` : `modifier`, `restart --seed 7`, `reload` et `unmodify` échouent avec `{'code': -32600, 'message': "commande d'édition refusée : check_distance=2 (le mode éditeur exige 0 : …)"}`.
- `cargo fmt --all -- --check` : 0 ; `scripts/check-forbidden.sh` : OK (51 occurrences auditées) ; `scripts/check-rollback-registration.sh` : OK ; `alacod lint` : zombies, gungeon (1 avertissement), isaac, testbed (115 avertissements), throne, sans erreur.

## Coût mesuré de la scrutation

`content_fingerprint` sur `games/isaac` : **1,92 ms par relevé** (198 fichiers suivis, moyenne de 100 appels, test ignoré `fingerprint_cost_on_isaac`), un relevé par seconde (≈ 0,2 % d'un cœur). Elle n'existe qu'en mode éditeur : un binaire normal ne paie rien.

## Incident disque

Le disque a été saturé deux fois : cause = liaison dynamique de Bevy coupée par l'environnement (`CI=true` dans le shell de l'agent fait renvoyer une chaîne vide à `scripts/dev-features.sh features`) combinée à des `cargo test -p <crate>` lancés sans `$(./scripts/dev-features.sh features)` (plusieurs variantes de Bevy statique, binaires de test d'environ 1 Go) ; correction : target `headless` reconstruit à neuf, `CI` retiré (`env -u CI`) pour toutes les commandes, `make` seul, binaires d'environ 0,46 Go.

## Non vérifié

- Rendu, clavier réel, Mac, p2p (hors périmètre, session locale seulement).
- `make schedule_order_diff` non lancé (aucun système ni type rollback ajouté ; la suite de scénarios complète ne bouge aucune trace).
- Le clavier n'a pas été exercé (headless) : seule la prise de main par `alacod/input` et sa libération sont prouvées.

## Décisions

- `seed` de `alacod/restart` écrit `MapGenerationConfig.seed` seulement (i32) ; refusé avec `ToLobby`.
- Limite assumée : un personnage ou une arme **ajouté** exige une relance du binaire (`GlobalAsset` lit le registre au `Startup`).
- Le registre neuf prend effet au prochain `GameLoading` : `restart`, mais aussi changement d'étage (mode `Floors`) ; les écrans de mutation et de fin peuvent montrer transitoirement le contenu neuf.
