# Passation M-1 « Web d'un jeu externe » (déménagement sur le Mac, 2026-11-07) — lot NON fini

Branche engine `wquintal/m-1-web-externe` (partie d'`origin/main` `d04cea3e`). Spec : `/home/debian/orca/runs/alpha/m-1-web-externe.md` + `commun.md` (VPS ; à recopier si absent).
Worktree 1837 créé sur le VPS : branche `m-1-web` (partie d'`origin/main` 1837 `57c4d6b`), **aucun changement fait** (rien à pousser de plus que la branche vide).

**Fait (engine, compilé seulement en `cargo test --lib`)** : `crates/content/src/web_build.rs` (build wasm : lint → cargo wasm32 → wasm-bindgen épinglé par `Cargo.lock`, auto-installé dans
`<target>/alacod-tools` → wasm-opt release si présent → assets → `index.html` + `build.json`, sortie `<out>/builds/<version>/<jeu>/` ; serveur HTTP local `Server`) ; sous-commande
`alacod build --web [jeu] [--out] [--release] [--features] [--version] [--serve --port]` (`bin/alacod.rs`) ; `crates/game/build.rs` (`rerun-if-env-changed=APP_VERSION`) ; gabarit
`templates/starter` (`build.rs` d'embarquement du contenu, `main.rs` avec chemin `cfg(wasm32)` : `load_embedded` + racine d'assets `concat!("/builds/", env!("APP_VERSION"), "/{{name}}/assets")`,
`getrandom` js, mention dans `CLAUDE.md`) ; test `#[ignore]` `new_game_builds_for_the_web_without_any_added_code` + cible `make test_alacod_web` (et `test_alacod_cli` le saute) ;
docs : `docs/conventions.md` § 108, `docs/jouer.md`, dettes D190–D191.
**Vérifié** : `cargo test -p content --lib web_build` : 4 tests verts (version du verrou, versions d'URL, page, serveur MIME et `..` refusé).
**Pas fait / non vérifié** : le test e2e n'a jamais fini (compilation wasm32 de bevy interrompue par mon arrêt à ≈ 33 min, pas par une erreur de code : aucune erreur de compilation vue avant le SIGTERM) ;
`wasm-bindgen` jamais installé ni exécuté ; `index.html` jamais ouvert ; `make check_rapide WASM=1` non lancé ; `cargo fmt`/clippy non lancés sur les nouveaux fichiers ; taille du `.wasm` non mesurée.
**Côté 1837 (rien commencé)** : copier `templates/starter/{build.rs,src/main.rs}` (remplacer `{{name}}` par `1837`) dans le jeu, ajouter `getrandom` à son `Cargo.toml`, `./jouer.sh --web` (installe `alacod`
du `rev` de `Cargo.toml` par `cargo install --locked --git <url> --rev <rev> --root target/outils content --bin alacod`, puis `alacod build --web --release --serve`), `README.md` en tête, `docs/tranche.md` « État »,
rev engine = SHA de ta branche poussée (puis `main` quand le coordinateur l'envoie).
**Prochaine étape exacte** : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make test_alacod_web` (≈ 20-35 min à froid) ; corriger ce qu'il révèle ; puis `cargo fmt`, `make check_rapide WASM=1`,
rapport `docs/taches/rapports/m-1-web-externe.md`, ligne `LIVRÉ` dans `docs/taches.md`, envoyer `status` « E poussé <sha> », puis le côté 1837, puis `worker_done`.
**Pièges** : un seul cargo à la fois ; le jeu d'essai est son propre workspace (réutiliser `CARGO_TARGET_DIR` de l'engine) ; `main.rs` doit contenir la chaîne d'assets en **un seul littéral** (le build la cherche dans le wasm) ;
pas de `wasm-opt` installé (release seulement, optionnel) ; asset_root absolu = origine à la racine (D190) ; aucun Chrome automatisé ; rien de 1837 dans l'engine.
**Questions ouvertes** : un `AssetPlugin.file_path` relatif marcherait-il en wasm (D190) ? l'auto-installation de `wasm-bindgen-cli` sur le Mac (durée) ; pour M-2 : le lobby (`data-matchbox`, `data-lobby`) n'est pas posé par la page.
