# Revue C-7 — Manette : diagnostic et aide (`78315428`, code `49e0f869`)

**Verdict : merge** (aucun P1, aucun P2 ; P3 à noter, aucun bloquant). Relecture en lecture seule, aucun cargo lancé ; les résultats de tests cités viennent du rapport du worker, non rejoués.

## 1. Frontière : OK
- Diff Rust : `gamepad_diagnostics.rs` (nouveau), `app_plugin.rs` (+3, dans `if !self.headless`), `mod.rs` (+1), `control.rs` (**uniquement** dans `mod tests`, +82). Ni `control.rs` hors tests, ni `input_devices.rs` ne bougent : aucune liaison ni seuil changé.
- Systèmes en `Update`, ressource `GamepadDiagnostics` sans rollback, aucun `GgrsSchedule`, crate `game` (APP, pas SIM). `BTreeMap`/`BTreeSet` sur `Entity` : lus seulement par le diagnostic, pas par la simulation. Aucune `.trace` touchée. [INFERENCE] aucune trace ne peut bouger.

## 2. Justesse du cas Parsec : plausible côté Linux
- `gilrs-core-0.6.8/src/platform/linux/gamepad.rs:569` ouvre le périphérique en `O_RDWR|O_NONBLOCK` sans `EVIOCGRAB`. Un `EVIOCGRAB` exclusif posé par un autre processus n'empêche donc pas l'ouverture. gilrs émet `Connected` (l. 325/333) et `bevy_gilrs` (`gilrs_system.rs:51-62`) crée l'entité `Gamepad`. Un grab exclusif ne fait donc que taire les événements : une `Gamepad` connectée et muette, ce que l'avertissement cible. C'est lu dans le code ; je ne l'ai pas observé avec Parsec. [INFERENCE] sur le mécanisme exact de Parsec (grab `EVIOCGRAB` ou périphérique uinput virtuel). Un pad uinput virtuel muet produirait aussi l'avertissement, à bon escient.
- Faux positifs (manette posée) : acceptable. Le message dit « Si personne ne touche la manette, ignorez ce message », il est unique par manette, à 1 800 frames de jeu avec un joueur local.

## 3. Web (wasm) : OK, un libellé à nuancer
- `std::env::var` sur wasm32 renvoie `Err` : `from_env` donne `debug=false`, rien de faux. `log_gamepad_connections` est inerte sans `debug`. `RawGamepadEvent` existe côté web (backend wasm de gilrs). L'avertissement reste actif dans Chrome, ce qui est utile.
- `cfg!(target_arch = "wasm32")` est juste, mais n'est atteint que par `mapping_source`, appelée seulement quand `debug` est vrai : jamais sur le web, donc code mort là-bas. Inoffensif.
- P3 `gamepad_diagnostics.rs:82` : « mapping *standard* du navigateur » est faux pour une manette dont `p.mapping` vaut `""`. `gilrs-core/platform/wasm/gamepad.rs:197` distingue `Standard` et `NoMapping`. Le message devrait dire « mapping standard ou, sinon, brut du navigateur ». Sans effet, car non atteint sur le web.

## 4. Tests : prouvent le comportement
- `control.rs` (`manette_muette_avertie_une_seule_fois`, `manette_active_jamais_avertie`, `sans_joueur_local_pas_d_avertissement`) : un test exerce la frontière du délai (9 frames : rien, 10 : un avertissement, +30 : toujours un) ; il teste l'état `warned`, pas le `warn!`, ce qui est acceptable. Le test « active » envoie un vrai `RawGamepadEvent` (`pad.axis`, l. 203). Le test « sans joueur » prouve la garde `LocalPlayer`.
- P3 `control.rs` (`watch`) : le test réinstalle les systèmes à la main ; si `GamepadDiagnosticsPlugin` perdait un système ou son `.after`, rien ne casserait. Le câblage réel n'est pas testé.
- P3 `gamepad_diagnostics.rs:210` : `la_source_du_mapping_nomme_la_variable` teste du libellé (`contains("embarquée")`). Selon la règle « wording tests » : à supprimer ou à laisser, sans conséquence. `le_motif_du_guid_suit_le_format_sdl` est un vrai test de format.
- P3 `gamepad_diagnostics.rs:127,159` : `spoke` est définitif. Une manette qui parle puis devient muette (prise par Parsec en cours de partie) n'est jamais avertie. Cohérent avec « jamais de faux positif », mais le cas inverse n'est pas couvert.
- P3 : le garde-fou « événement de connexion `continue` » (l. ~121) n'est pas testé. Les doublons à la reconnexion ne sont pas testés non plus.

## 5. Docs : OK
- § 134 est numéroté à la suite (après § 133), conforme au code. Une seule ligne `CLAUDE.md`, au bon endroit (liste des variables d'environnement). `templates/starter/README.md` est un nouveau fichier avec `{{name}}`. Le gabarit est un répertoire copié, donc `alacod new` le prend ; je n'ai pas vérifié la substitution de `{{name}}` dans les `.md`.
- Aucun contenu de jeu externe (noms neutres, `Xbox Wireless Controller 045e:0b13` n'est que le matériel).
- P3 `docs/conventions.md` § 134 : « Aide : une ligne dans le README » : le README cite des commandes, vérifier qu'il n'est pas écrasé quand `alacod new` génère son propre README (chemin `README.md`).

## 6. Preuve en direct : suffisante, une imprécision
- Natif Linux : `ALACOD_INPUT_DEBUG=1 alacod sandbox <jeu> 2>&1 | grep -i manette` est exact et suffit en deux minutes (connexion, stick droit, RT, puis contre-épreuve Parsec ~30 s). Le journal passe par `info!` ; le filtre par défaut du jeu est `info` (`crates/game/src/logs.rs:41`), donc rien à ajouter, sauf si `RUST_LOG` est posé dans l'environnement de William.
- La variante `cargo run --features native -- --sandbox` n'a pas été vérifiée.
- Chrome : `navigator.getGamepads()[0]` puis `[p.id, p.mapping, p.buttons[7].value, p.axes[2], p.axes[3]]` est exact pour un mapping `"standard"` (RT = bouton 7, stick droit = axes 2 et 3). Si `mapping` vaut `""`, les indices sont autres : le rapport le suppose standard sans le dire en cas de non-standard. Chrome ne donne les valeurs qu'après un premier appui, ce que le rapport dit.
- P3 rapport : ajouter le cas `mapping == ""` côté Chrome.

## Synthèse
- P1 : aucun. P2 : aucun.
- P3 : `gamepad_diagnostics.rs:82` (libellé web), `:210` (test de libellé), `:127/159` (`spoke` définitif), `control.rs` `watch` (câblage non testé), README `alacod new` à vérifier.
