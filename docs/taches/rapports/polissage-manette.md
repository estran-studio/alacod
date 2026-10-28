# polissage-manette — rapport S1-1 (voie b0, branche `polissage-manette`)

Fiche : `docs/taches/polissage-manette.md`. Périmètre réduit au 2026-10-27 : **zombies et throne** (gungeon et isaac : liaisons
gardées si elles ne coûtent rien, ni preuve ni essai). Machine : Mac, Chrome 156 (vrai Google Chrome, piloté par CDP).

> **Réserve de lecture, en tête.** Aucune manette physique n'était branchée sur la machine de la voie. Toute la preuve « dans
> Chrome » ci-dessous est faite avec une manette **simulée** : une page de test remplace `navigator.getGamepads()` par un objet à la
> disposition standard (17 boutons, 4 axes, valeurs analogiques), que le jeu lit **par le vrai chemin** `bevy_gilrs` (`gilrs-core`
> wasm) → Bevy → leafwing → `collect_device_inputs` → GGRS. Elle prouve la chaîne logicielle ; elle ne prouve ni le matériel, ni le
> comportement de Chrome face à une vraie manette (premier appui, mapping non standard, reconnexion physique). Ces points sont la
> raison de l'essai E4 (§7).

## 1. Issue par point de la fiche

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | `bevy_gilrs` en wasm | **fait** (manette simulée) ; matériel **non vérifié** | §2 |
| 2 | Schéma validé par William | **fait** (accepté le 2026-10-22), une contradiction de la fiche tranchée (§3) | §3 |
| 3 | `get_input_map` source unique, une liaison par action utile | **fait** | `control.rs`, tests `character::control::tests` |
| 4 | Menus à la manette : pause, inventaire, fin de partie, choix de mutation | pause, inventaire, fin de partie, salon local **faits et vus** ; mutation **par tests, pas vue dans le navigateur** | §4, §5 |
| 5 | Écran d'aide lu dans la table, filtré par clone | **fait** | captures, tests `ui::screens::help::tests`, scénario `ui_pause_menu` |
| 6 | `docs/jouer.md` : section manette (navigateur compris) | **fait** | `docs/jouer.md` § Manette |
| 7 | Essai E4 avec William, dans le navigateur | **à faire ven 30** : build, serveur, protocole et enregistreuse prêts (§7) | — |
| — | Traces inchangées, bits d'input inchangés | **fait** | §6 |

## 2. Ce qui marche en wasm (manette simulée)

`cargo tree -e features --target wasm32-unknown-unknown -p zombies -i bevy_gilrs` : `bevy_gilrs` 0.19.1 et `gilrs` 0.11.2 sont dans
l'arbre (feature par défaut de `bevy`, unifiée par `game` ; les jeux wasm se construisent en `--no-default-features --features
render`, ce qui ne l'ôte pas). **Aucun changement de `Cargo.toml`, de `loader.js` ni de code propre au web.** Constructions :
`scripts/build-web-games.mjs zombies throne` (dev, 8 min 35 à froid, 20 s à 3 min ensuite), servies par un serveur statique local.

Chaque ligne ci-dessous est lue dans le journal du jeu (`INFO … PROBE buttons=… fire=… pan=…`, sonde temporaire retirée avant le
commit) pendant une partie de zombies dans Chrome :

| Entrée simulée | Lu par le jeu | Verdict |
|---|---|---|
| connexion de la manette (apparue après le chargement, comme le fait Chrome au premier appui) | reconnue, entrées lues | oui |
| stick gauche haut / bas / gauche / droite (axe Y web −1 = haut) | `UP` / `DOWN` / `LEFT` / `RIGHT` | oui, **sens de Y correct** |
| stick gauche diagonal (0,7 ; −0,7) | `UP|RIGHT` | oui |
| stick gauche 0,39 puis 0,41 sur un axe | rien, puis `RIGHT` (seuil 0,4) | oui |
| stick gauche 0,2 | rien | oui |
| stick droit droite / haut / diagonale | `pan` (127,0) / (0,127) / (90,−90) | oui |
| stick droit relâché | `pan` gardé | oui |
| RT à 0,2 / 0,5 / 0,7 / 1,0 / relâché | `fire` faux / **vrai** / vrai / vrai / faux | oui (après correction, §5.2) |
| A, B, X, Y, LB, RB, LT, L3, R3 | `BOMB`, `MELEE`, `RELOAD`, `INTERACT`, `DASH`, `switch_weapon`, `ACTIVE`, `SPRINT`, `MODE` | oui |
| croix haut / bas / gauche / droite | `BLANK` / `DROP` (une seule requête) / rien / rien (pas de déplacement) | oui |
| Start, croix ↓ ×4, A | pause ouverte, « Touches » surligné, aide affichée | oui (captures §4) |
| A, B pendant la pause | aucun bit de jeu (pas de bombe, pas de coup) | oui |
| manette **débranchée** en tenant stick et RT | entrées relâchées, rien de collé | oui |
| manette rebranchée | entrées lues de nouveau | oui |
| fin de partie : A après 0,8 s | partie relancée | oui |
| fin de partie : B | écran d'attente du lobby local ; A le valide, partie relancée | oui |

**Non vérifié (aucune manette physique)** : appui physique qui « réveille » la manette dans Chrome, valeurs réelles des gâchettes
(plage, repos), dérive des sticks, manettes PlayStation/Switch, manette à mapping non standard (`gamepad.mapping != "standard"` :
`gilrs-core` la lit en codes bruts, **sans mapping** : risque réel pour une manette générique), deux manettes, Bluetooth,
reconnexion physique, mutation (§5), tir et visée au long cours sur une vraie partie (E4).

## 3. Tableau final des liaisons

Source unique : `crates/game/src/character/control.rs::get_input_map` (affichée en jeu par l'entrée « Touches » de la pause, copiée
dans `docs/jouer.md` § Manette). Disposition Xbox ; PlayStation : Croix / Rond / Carré / Triangle = A / B / X / Y.

| Bouton | Action | Remarque |
|---|---|---|
| Stick gauche | déplacement (`Move*`) | huit directions : un bouton de direction par axe au-delà de 0,4 (`BoxInput` n'a pas de déplacement analogique) |
| Stick droit | visée (`Pan` → `pan_x`, `pan_y`) | dernière direction gardée au repos ; la souris reprend la main si le curseur bouge de plus de 3 px |
| RT | tir (`PointerClick`) | tire à 0,5 de course, relâche à 0,35 |
| LT | objet actif (`UseActive`) | idem (jeux à objets seulement) |
| RB | arme suivante (`SwitchWeapon`) | |
| LB | esquive (`Dash`) | |
| L3 / R3 | sprint / mode de l'arme | L3 jugé à l'essai E4 (D160) |
| X / Y / B | recharger / interagir / mêlée | inchangés |
| A | **confirmer** dans les menus, le choix de mutation, l'écran de fin ; **bombe** ailleurs | contexte : sous un écran ou un choix de mutation, la bombe ne part pas |
| Croix haut / bas | blank / lâcher l'arme | |
| Croix gauche / droite | surbrillance du choix de mutation | |
| Start / Select | pause / inventaire | inchangés |
| (menus) croix haut/bas, A, B | choisir, valider, retour | écran « Touches » : B revient au menu, B de nouveau ferme |
| (fin de partie) A / B | rejouer / lobby | après 0,8 s d'affichage |
| — | `Modifier` (esquive arrière) | pas de liaison manette, clavier seulement (D160) |

**Contradiction de la fiche tranchée** : la ligne « stick gauche = déplacement (le D-pad reste un doublon) » contredit « D-pad
gauche/droite = `ChoicePrev`/`ChoiceNext` … le D-pad ne sert plus à se déplacer ». J'ai suivi la seconde, plus précise : la croix ne
déplace plus (elle servait de déplacement, avec les deux directions horizontales déjà liées au choix de mutation).

## 4. Captures

`docs/taches/rapports/polissage-manette/` (Chrome, manette simulée, build `e4-manette`) :

| Fichier | Contenu |
|---|---|
| `zombies-fin-de-partie.png`, `throne-fin-de-partie.png` | écran de fin : « Rejouer (R / A) », « Lobby (B) » |
| `zombies-pause.png`, `throne-pause.png` | menu de pause, cinq entrées dont « Touches » |
| `zombies-aide.png`, `throne-aide.png` | écran « Touches » : action, clavier/souris, manette, filtré par clone (throne : ligne « Mutation » en plus) |
| `zombies-inventaire.png`, `throne-inventaire.png` | inventaire (Select) |

## 5. Ce qui a été trouvé en route (et corrigé)

1. **La visée au stick n'existait pas.** `PlayerAction::Pan` était liée au stick gauche et jamais lue : `pan_x`/`pan_y` ne venaient
   que du curseur (`input_devices.rs`). Ajouté : `resolve_aim`, avec direction gardée et reprise par la souris. Le premier curseur
   vu après le stick est une position de départ, pas un mouvement (le navigateur envoie un `mousemove` à la prise de focus : sans
   cela, la souris reprenait la visée dès la première frame, constaté en wasm).
2. **RT aurait tiré à l'effleurement.** leafwing 0.21 tient une action pour pressée dès 0,02 de course
   (`ActionState::set_button_value`), alors que `Gamepad::pressed` de Bevy attend 0,75 : constaté en natif et en wasm (valeur 0,2 →
   `fire` vrai ; test `character::control::tests` + `input_devices::tests`). `trigger_pressed` lit `button_value` et pose le seuil.
3. **Zombies et throne n'avaient ni menu de pause ni inventaire visible** : `HuiHandles` n'a de gabarit que si `ui/pause_screen.html`
   existe, et seuls gungeon et isaac l'avaient. Échap ne montrait rien, sans erreur. Gabarits ajoutés (`pause_screen.html`,
   `inventory_screen.html`, `hui_row.html`) dans `games/zombies/assets/ui` et `games/throne/assets/ui` — fichiers **nouveaux** dans
   des dossiers des voies b1 et b3, dits au coordinateur.
4. **Sous un écran ouvert, la partie continue** et A (bombe), B (mêlée), croix ↓ (lâcher) auraient agi en navigant ; B qui ferme la
   pause donnait un coup de mêlée tant qu'il restait tenu. `SCREEN_OPEN_KEEPS` et le masque « tenu pendant l'écran » les arrêtent.
5. **« Rejouer » en solo web mène à l'écran d'attente du lobby local**, qui ne se validait qu'à Entrée ou au clic
   (`ui/lobby.rs::release_local_hold`) : A le valide. Ajout d'une ligne « A (manette) : nouvelle partie » à l'affichage ;
   `local_hold_text` (texte de b2) intact.
6. **`make check` n'est pas vert sur `origin/main`** : `cargo fmt --check` échoue sur `crates/content/tests/lint_fixtures.rs:1074`
   (une constante sur deux lignes, héritée de X-4a). Corrigé par `cargo fmt` (1 ligne, identique pour qui d'autre le corrige) ;
   et `make test_tools` : 3 échecs de `scripts/tests/test_claude_say.py` (le test lie `/usr/bin/python3` dans un `PATH` vide, ce que
   ce Mac ne permet pas), sans lien avec S1-1, non corrigés.

**Mutation (throne)** : A valide la carte surlignée (`confirm_input`) et ne pose pas de bombe, croix ←/→ déplacent la surbrillance
(`ChoicePrev/Next`, liaisons de `mutation_input.rs` inchangées) : prouvé par tests unitaires. **Pas vu dans le navigateur** : le choix
de throne se propose entre deux étages (`choice_timing: BetweenFloors`), après trois kills et une salle vidée, hors de portée d'un
essai scripté sans manette physique. À voir à l'E4.

## 6. Tests et vérifications

- Tests unitaires ajoutés (tous verts) : `character::control::tests` (3 : le vrai chemin leafwing, sticks à seuil, croix, épaules,
  course des gâchettes), `character::input_devices::tests` (5 : visée gardée, curseur, seuil de tir, écran ouvert et boutons tenus,
  A valide la mutation sans bombe), `ui::screens::help::tests` (3), `ui::screens::model::tests` (pause « Touches », Retour),
  `ui::game_over::tests` (délai de la manette). Scénario `tests/scenarios/ui_pause_menu.ron` étendu (« Touches » validé, ligne
  « Tirer … RT » lue dans la table).
- `make check`, décomposé (`test_tools` échoue pour la raison du §5.6 et arrête la cible entière) : voir §8 pour les transcripts.
- Bits d'input : aucun fichier de `character/player/input.rs`, `combat/src/actors.rs` ni `replay.rs` modifié (`git diff --stat`).

## 7. Protocole E4 avec William (vendredi 30, ~20 min, dans Chrome)

Prêt : build web local `e4-manette` de zombies et throne (dans `website/static/builds/`, ignoré par git), serveur statique sur le
port 8777 de ce Mac (lancé par la voie ; `cd website/static && python3 -m http.server 8777 --bind 127.0.0.1` pour le relancer).

1. Brancher la manette (USB ou Bluetooth), **puis** ouvrir Chrome sur
   `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette` (throne : `game=throne`). Le chargement dure ~30 s (build dev).
2. Cliquer dans la page, **appuyer sur un bouton de la manette** : sans cela Chrome ne l'expose pas. Noter si le premier appui est perdu.
3. Enregistrer : `screencapture -v -V 60 ~/Desktop/e4-zombies.mov` dans un terminal (60 s, tout l'écran), puis jouer 20 à 60 s.
4. Jouer : stick gauche déplace, stick droit vise et **garde** sa direction au repos, RT tire (progressif ?), X recharge, LB esquive,
   B mêlée, RB arme, L3 sprint (confortable ?), croix bas lâche l'arme.
5. Start : pause (le jeu **continue** derrière, D158) ; croix ↓ jusqu'à « Touches », A : l'aide ; B revient, B ferme. Select : inventaire.
6. Débrancher la manette en jeu, constater que rien ne reste collé ; la rebrancher, appuyer sur un bouton, reprendre.
7. Se faire tuer : A rejoue (après un instant), ou B puis A depuis l'écran d'attente.
8. Throne : monter à 3 kills et finir la salle, **choix de mutation** : croix ←/→ surligne, A valide, pas de bombe.
9. Deux joueurs : hors de portée du web local (un joueur par page) ; le noter.
10. Notes dans `docs/taches/rapports/plan-polissage-notes.md` (section « E4 manette ») : ce qui gêne, ce qui manque, le verdict.

À juger en particulier : seuil de RT (0,5), seuil du stick (0,4 par axe), L3 pour le sprint, « la pause ne met pas en pause » (D158).

## 8. Reste, dettes, estimation

- **À faire** : essai E4 (ven 30), correction de ce qu'il révèle (≤ 0,5 j), enregistrements d'écran 20-60 s des deux jeux avec une
  vraie manette (critère d'acceptation : aucun produit ici), transcripts de `make check`.
- Dettes ouvertes (`docs/taches/dettes.md`) : **D158** la pause ne fige pas la partie, **D159** une manette pilote tous les joueurs
  locaux, **D160** liaisons sans usage prouvé (`Modifier`, sprint sur L3, bombe/blank/actif).
- Estimation de la fiche : 3 j, tenue (J1-J2 faits en un jour et demi ; reste E4 et son retour).
