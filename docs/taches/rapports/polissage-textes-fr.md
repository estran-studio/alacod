# polissage-textes-fr — S1-4 : textes du moteur et des clones en français

Branche `polissage-textes-fr` (sur `origin/polissage-integration`), voie b2, VPS ovh-inner. Présentation seule : aucune trace ne bouge.
Conventions : `docs/conventions.md` § 100.

## Points de la fiche

| Point | Issue | Preuve |
|---|---|---|
| 1. Lister et centraliser | fait | `crates/game/src/ui/texts.rs` ; sources remplacées : `ui/lobby.rs`, `ui/disconnected.rs`, `ui/screens/model.rs`, `jjrs/mod.rs`, `jjrs/p2p.rs`, `jjrs/p2p_socket.rs`, `args/mod.rs` ; gabarits `games/*/assets/ui/game_over_screen.html` |
| 2. Traduire (DESYNC, déconnexion) | fait pour les mots ; le retour au salon reste à S1-3 | « PARTIE COMMUNE PERDUE » + phrase + bouton « Retour au salon » ; « PARTIE INTERROMPUE » / « … s'est déconnecté » ; l'action du bouton n'a pas changé |
| 3. Attentes de scénario | fait | `ui_pause_menu.ron` : « Lobby » → « Salon » ; aucun autre scénario ne lit ces chaînes |
| 4. Textes des clones | fait | `zombies` : noms des power-ups (`items/powerups.ron`) ; `throne` : `names` des armes dans `ui/hud.ron` |

## Avant / après (chaînes de la fiche)

| Avant | Après |
|---|---|
| Waiting for Players... | En attente des joueurs… |
| Ready / Connecting... / Connected / Waiting... | Prêt / Connexion… / Connecté / En attente… |
| Player N / Spectator N | Joueur N / Spectateur N |
| Connecting to peer... 7 s | Connexion au pair… 7 s |
| Peer connection failed: new attempt (2/2) | Connexion au pair échouée : nouvelle tentative (2/2) |
| …after 2 attempt(s): back to the lobby | …après 2 tentatives : retour au salon |
| Room full (3 peers for 2 places): … | Salle pleine (3 pairs pour 2 places) : … |
| GAME DISCONNECTED / « X disconnected » | PARTIE INTERROMPUE / « X s'est déconnecté » |
| DESYNC / « Frame N: état différent de X » | PARTIE COMMUNE PERDUE / « La partie commune est perdue : … (image N). » |
| Back to Menu | Retour au salon |
| Lobby (pause, fin de partie) | Salon |
| Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke (zombies) | Mort instantanée, Points doublés, Munitions max, Charpentier, Bombe atomique |
| « fusil_a_pompe », « lance_grenades »… dans le prompt (throne) | fusil à pompe, lance-grenades… |

## Reste en anglais, exprès

Journaux (`info!`/`error!`, lus par `scripts/p2p-*.sh`), niveau de télémétrie `DESYNC`, diagnostics `ALACOD_DEBUG_UI=1` (`frame.rs`, `waves/debug.rs`,
`camera/ui.rs`, `weapon_hud.rs`), noms de perks (Juggernog, Speed Cola, Double Tap, Stamin-Up), noms des armes de mêlée de zombies
(`melee_weapons.ron`, non affichés par le HUD). Dette D164 si William veut aussi les perks.

## Captures à faire sur le Mac (impossible sur le VPS, D94)

Toujours `ALACOD_MUTE=1`, cible `CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac`.

1. Pause, Réglages, Salon (scénario `ui:`) :
   `ALACOD_MUTE=1 cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/ui_pause_menu.ron --capture /tmp/s14-pause --every 2`
   (images 12, 22, 31, 33, 35, 41 : entrées « Salon », curseur « Volume »).
2. Inventaire : même commande avec `tests/scenarios/ui_inventory.ron --capture /tmp/s14-inventaire`.
3. Fin de partie (zombies, throne) : `make videos SCENARIO=<un scénario qui finit en défaite>` avec `ALACOD_MUTE=1`, ou `alacod sandbox games/zombies` puis `alacod remote screenshot` après une mort ; vérifier « Rejouer » et « Salon (B) ».
4. Salon en ligne (états réseau, joueurs « Joueur N », « En attente des joueurs… ») : `ALACOD_MUTE=1 make test_multiplayer TARGET=ldtk_map_explorer N=2`, capturer les deux fenêtres pendant l'attente ; la ligne d'état lente (« Connexion au pair… ») apparaît après 3 s sans second pair.
5. Écran « PARTIE INTERROMPUE » : même commande à N=2, fermer la fenêtre d'un des deux ; l'autre affiche le message et « Retour au salon ».
6. « PARTIE COMMUNE PERDUE » : pas de déclencheur manuel ; vérifié par `ui::texts::tests` seulement (à voir en vrai par S1-3 si un desync arrive).

## Preuves

- Tests `game` (texts, p2p_socket, screens, lobby, disconnected, game_over, jjrs) : `37 passed; 0 failed`.
- `make check` : `✓ All checks passed`, sortie 0.
- `make test_scenarios` sans `BLESS` : `test result: ok. 5 passed; 0 failed; 12 ignored` (9177 s), sortie 0 : `ui_pause_menu` vert avec « Salon », aucune trace ne bouge.
- Grep des chaînes anglaises de la fiche dans les écrans : 0 (hors journaux et diagnostics listés plus haut).
