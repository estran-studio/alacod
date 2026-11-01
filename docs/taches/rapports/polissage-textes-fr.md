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

## Captures faites (2026-10-30, Mac, voie b1, repli public)

`ALACOD_MUTE=1`, `CARGO_TARGET_DIR=…/.target-mac`, surcharges locales (`*.local.ron`, `sprites/postapo/`, `sounds/sonniss/` de `games/zombies` et `games/throne`) déplacées hors de `games/` pendant les captures puis remises. Aucun code modifié. PNG sous `docs/captures/polissage-textes-fr/`.

| Point | Capture | On y lit |
|---|---|---|
| 1 | `pause-f12-reprendre.png` | Menu « Pause » : Reprendre (surligné), Recommencer, Salon, Réglages, Touches ; pied « ↑ ↓ choisir · Entrée / A valider · Échap / B reprendre » |
| 1 | `pause-f22-reglages.png` | Focus sur « Réglages », entrée « Salon » dans la liste |
| 1 | `pause-f31-volume-100.png` | « Réglages » validé : « Volume : 100 % » |
| 1 | `pause-f33-volume-80.png` | Curseur « Volume : 80 % [########--] (← → pour régler) » |
| 1 | `pause-f35-volume-90.png` | « Volume : 90 % » |
| 1 | `pause-f41-salon.png` | Focus sur « Salon », le détail « Volume » a disparu |
| 2 | `inventaire-f12-armes-actif-passifs.png` | « Inventaire » : Armes (pistolet (en main) 12 \| 119988), Objet actif « Fiole de rage » 2 / 2 · prêt, Passifs (Amulette, Bottes), Consommables (Blank ×2, Clé ×1) |
| 2 | `inventaire-f22-passifs.png` | Focus sur le passif « Amulette » |
| 2 | `inventaire-f24-consommables.png` | Jauge de l'actif « 0 / 2 », focus sur « Blank ×2 » |
| 3 | `fin-de-partie-zombies-defaite.png` | `zombies` (`run_lose_summary`, f1690) : « DÉFAITE », « vague 1 - 0 kills - 500 points / 1402 frames », boutons « Rejouer (R / A) » et « Salon (B) » |
| 3 | `fin-de-partie-throne-defaite.png` | `throne` (`throne_duo_defaite`, f990) : « DÉFAITE », « Étage 1 · 0 éliminés 0:16 », mêmes boutons |
| 4 | `salon-en-ligne-alice-attente-pair.png` | Salon en ligne (`zombies`, 1 pair sur 2) : « En attente des joueurs… », « alice: Prêt », « Joueur 2: En attente… » (cases vides aux accents, voir plus bas) |
| 5 | `partie-interrompue-alice-apres-depart-de-bob.png` | Pair parti (processus tué, `NetworkInterrupted` puis `Disconnected` à 4,5 s) : « PARTIE INTERROMPUE », « Joueur 2 s'est déconnecté », bouton « Retour au salon » |

Méthode : points 1 à 3 par `play_scenario … --capture <dossier> --every 1` (`every 10` pour 3), images copiées telles quelles. Points 4 et 5 par `make ldtk_map_explorer_matchbox` (alice, puis bob une fois la fenêtre d'alice ouverte), fenêtre de `zombies` photographiée par `screencapture -l<id>` (sans le bureau), journaux dans `/tmp/s14-logs/`.

Serveur de signaling : `wss://allumette.estran.studio` a répondu en HTTP mais le jeu n'a pas pu s'y connecter (`connection to signaling server failed, 2 attempt(s) remain`, puis `socket dropped`, deux clients, salon `b1-s14-21173`) ; Docker n'est pas lancé sur ce Mac. Les points 4 et 5 ont été joués contre un `matchbox_server 127.0.0.1:3536` local (`MATCHBOX_URL=ws://127.0.0.1:3536`), deux clients sur la même machine ; la connexion est faite en moins de 3 s.

### Non capturé

- Ligne lente « Connexion au pair… N s » du salon : elle n'apparaît que si le signaling annonce un pair qui ne se connecte pas en 3 s (`connect_status_text`, `CONNECT_SLOW_SECS`) ; en local le pair se connecte avant, et simuler la lenteur demande un réseau dégradé (`NETEM`, `scripts/p2p-*.sh`, root). À faire par S1-3 ou sur deux réseaux.
- Salon à deux joueurs affichés « Joueur 1 / Joueur 2 » : le salon de 2 pairs s'efface dès la connexion (partie lancée), la capture montre l'état à un pair.
- Point 6 (« PARTIE COMMUNE PERDUE ») : pas de déclencheur, inchangé.

### Textes encore en anglais ou mal rendus sur les captures (listés, non corrigés)

| Écran | Texte | Fichier probable |
|---|---|---|
| Salon en ligne, « PARTIE INTERROMPUE » | **Cases vides à la place de « ê » (« Prêt »), de « é » (« déconnecté ») et de « … » (« joueurs… », « attente… »)** : la police du salon n'a pas ces glyphes ; l'écran de déconnexion n'a pas de police posée (police par défaut) | `crates/game/src/ui/lobby.rs` (`LOBBY_FONT`), `crates/game/src/ui/disconnected.rs`, `assets/fonts/` |
| « PARTIE INTERROMPUE » | Le bouton « Retour au salon » passe sur deux lignes | `crates/game/src/ui/disconnected.rs` |
| Fin de partie `zombies` | « kills », « frames » (résumé : « vague 1 - 0 kills - 500 points\n1402 frames ») | `crates/game/src/ui/game_over.rs` l. 113 |
| HUD `zombies` (en jeu, fin de partie) | Arme « machine_gun » (id brut) alors que `ui/hud.ron` porte `"machine_gun": "mitrailleuse"` dans `names` | `games/zombies/assets/ui/hud.ron` l. 147, lecteur du widget `weapon` |
| Pause, inventaire (`gungeon`, HUD) | « Blanks 2 » (HUD), « Blank ×2 » (inventaire) | `games/gungeon/assets/ui/hud.ron` l. 90, `games/gungeon/assets/objets/blank.ron` |
| Pause, inventaire (`gungeon`) | « Bullet kin » (étiquette F6 des ennemis, forcée en capture) : nom dérivé de l'id `bullet_kin` par `pretty()` quand aucun nom n'est donné | `crates/game/src/legibility.rs` (`pretty`), contenu `gungeon` |

Les cases vides sont le seul défaut qui rend un texte français illisible ; à traiter par S1-3 ou une dette (D161–D163, voie b1, numéros réservés ; non créée ici).

## Suite : correctifs des constats des captures (2026-10-30, branche `polissage-textes-fr-suite`)

| Constat | Cause (une ligne) | Correctif | Capture après (repli public, `ALACOD_MUTE=1`) |
|---|---|---|---|
| Cases vides sur « Prêt », « En attente… », « déconnecté » | Le salon en ligne et l'écran de perte de session posaient un `TextFont` sans `font` : police par défaut de Bevy, sans accents ni « … » (seul l'écran d'attente local posait `LOBBY_FONT`) | `texts::UI_FONT` posée sur tous les textes de `ui/lobby.rs` et `ui/disconnected.rs` ; valable pour tous les jeux (même fichier `fonts/FiraMono-Medium.ttf` dans chacun et dans `templates/starter`) | `salon-en-ligne-alice-attente-pair-apres.png`, `partie-interrompue-alice-apres-depart-de-bob-apres.png` |
| « Retour au salon » sur deux lignes | Bouton de largeur fixe 200 px, texte de 15 caractères à 30 px | `min_width` 200 px + marge intérieure, la largeur suit le texte | `partie-interrompue-alice-apres-depart-de-bob-apres.png` |
| « kills » / « frames » | Littéraux dans `ui/game_over.rs` et `ui/lobby.rs` | `texts::run_summary_waves` (« vague 1 · 0 éliminés · 500 points ») et `texts::run_duration` (« 0:23 », comme le mode `Floors`) | `fin-de-partie-zombies-defaite-apres.png`, `fin-de-partie-throne-defaite-apres.png` |
| « machine_gun » dans le HUD de `zombies` | `update_hud_values` écrivait l'id de l'arme ; `names` n'était lu que par le prompt et les étiquettes | le widget `weapon` lit `hud.ron` `names` (id si absent) : « mitrailleuse » | `fin-de-partie-zombies-defaite-apres.png` (coin bas droit) |

Les captures de pause et d'inventaire sont rejouées à l'identique (`pause-*-apres.png`, `inventaire-*-apres.png`) : mêmes textes qu'avant. `gungeon` (« Blank(s) », « Bullet kin ») reste hors périmètre. Points 4 et 5 rejoués sur un `matchbox_server` local (signaling public inutilisable, voir plus haut).

## Preuves

- Tests `game` (texts, p2p_socket, screens, lobby, disconnected, game_over, jjrs) : `37 passed; 0 failed`.
- `make check` : `✓ All checks passed`, sortie 0.
- `make test_scenarios` sans `BLESS` : `test result: ok. 5 passed; 0 failed; 12 ignored` (9177 s), sortie 0 : `ui_pause_menu` vert avec « Salon », aucune trace ne bouge.
- Grep des chaînes anglaises de la fiche dans les écrans : 0 (hors journaux et diagnostics listés plus haut).
- Suite (2026-10-30, Mac) : `cargo test -p game --lib ui::` `56 passed; 0 failed` ; `make format check_forbidden check_rollback_registration check_sim_boundary test_crates lint` sortie 0 (`test_tools` non joué : Mac) ; `make test_scenarios` sans `BLESS` en quatre tranches (`ALACOD_SCENARIO_SLICE=0/4` à `3/4`, `ALACOD_MUTE=1`) : `5 passed; 0 failed` chacune, aucune trace ne bouge, surcharges locales déplacées pendant les tests.
