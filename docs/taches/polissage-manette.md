# polissage-manette — manette jouable dans le navigateur, menus compris (S1-1, priorité 1)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 2, §5 décision 2),
`crates/game/src/character/control.rs` (`PlayerAction`, `get_input_map` l. 58-108), `character/input_devices.rs:23-50`,
`ui/screens/input.rs` (navigation des écrans, l. 30-60), `ui/game_over.rs`, `docs/jouer.md:71-97`,
`docs/taches/rapports/dette-d69-e4-personnage.md` §8-9, `docs/conventions.md` (bits d'input `INPUT_*`),
`website/static/loader.js`. Branche `polissage-manette` depuis `origin/main`. Estimation : 3 j. Clones : les quatre.
Machine : **Mac avec affichage, Chrome et une manette physique** ; le VPS ne peut pas.

## Contexte (inventaire §2, §13)

**La manette est cruciale** (William, 2026-10-21) : le focus est plus manette que clavier. Aujourd'hui, on ne peut **pas
tirer** à la manette (`PointerClick` = clic gauche seul, `control.rs:102`), la visée est le stick gauche et le
déplacement le D-pad ; aucune liaison pour Sprint, Dash, SwitchWeapon, SwitchWeaponMode, DropWeapon, UseActive, Blank,
Bomb, ChoiceA/B/C, Modifier. Les menus de pause et d'inventaire ont des liaisons (`ui/screens/input.rs:53-60`) ; la fin de
partie [INFERENCE] n'en a pas. `bevy_gilrs` est une feature par défaut de `bevy`, probablement active en wasm par
unification ; sa marche dans le navigateur (API Gamepad) n'est **pas prouvée**. Les jeux wasm sont construits avec
`--no-default-features --features render` (`build-web-games.mjs:28-36`).

## À faire

1. **Prouver la manette dans le navigateur d'abord** (demi-journée) : `cargo tree -e features --target
   wasm32-unknown-unknown -p zombies` pour `bevy_gilrs`, construire zombies (`make build_wasm_apps` ou le script), ouvrir
   dans Chrome avec la manette, appuyer sur un bouton (le navigateur n'expose la manette qu'après un premier appui
   [INFERENCE : comportement connu de l'API Gamepad, à constater]) ; noter ce qui marche : boutons, gâchettes analogiques,
   deux sticks, déconnexion/reconnexion. Si `bevy_gilrs` ne fonctionne pas en wasm, **rapporter à William avant de coder**
   (option : lecture de `navigator.getGamepads()` côté `loader.js` ou web-sys, injectée comme événement d'entrée).
2. **Schéma à valider par William avant d'écrire le code** (proposition, un seul tableau dans le rapport) :

   | Bouton (disposition Xbox) | Action | Remarque |
   |---|---|---|
   | Stick gauche | déplacement (`Move*`, analogique si leafwing le permet) | le D-pad reste un doublon |
   | Stick droit | visée (`Pan`) | **change la visée actuelle** (stick gauche) ; garder la dernière direction au repos |
   | Gâchette droite (RT) | **tir** (`PointerClick`) | analogique vers bouton avec seuil |
   | Gâchette gauche (LT) | actif (`UseActive`) | isaac, gungeon |
   | RB | changer d'arme (`SwitchWeapon`) | |
   | LB | dash (`Dash`) | isaac n'a pas de dash (`dash_distance` 0) : sans effet |
   | Stick gauche appuyé (L3) | sprint (`Sprint`) | |
   | Stick droit appuyé (R3) | changer de mode d'arme (`SwitchWeaponMode`) | |
   | X (Ouest) | recharger (`Reload`, existant) | |
   | Y (Nord) | interaction (`Interaction`, existant) | |
   | B (Est) | mêlée (`MeleeAttack`, existant) | |
   | A (Sud) | bombe (`Bomb`, isaac) ; **confirmer** dans les menus et le choix de mutation (`ChoiceConfirm`, existant) | conflit : à régler par contexte (menu ouvert, choix en attente) ou en déplaçant la bombe |
   | D-pad haut | blank (`Blank`) | gungeon, isaac |
   | D-pad bas | lâcher l'arme (`DropWeapon`) | |
   | D-pad gauche/droite | `ChoicePrev`/`ChoiceNext` (existant) ; `ChoiceA/B/C` pour la mutation | le D-pad ne sert plus à se déplacer |
   | Start / Select | pause / inventaire (existant) | |

   Points ouverts à signaler à William : le conflit Sud, `Modifier` (sans usage manette), le choix stick droit pour la visée.
3. Après validation : `get_input_map` reste l'unique source ; chaque action utile à un clone a une liaison manette, ou une
   exception écrite. Les clones n'utilisent que ce qu'ils ont (une bombe d'isaac ne gêne pas zombies).
4. **Menus à la manette** : pause, inventaire, fin de partie (« Rejouer », retour au salon : `RunRequest::Restart`,
   `ReturnToHostLobby`), choix de mutation : haut/bas/confirmer/retour (`ui/screens/input.rs` à étendre à `game_over.rs`).
5. **Écran d'aide** des touches **lues dans la table** (clavier et manette), ouvert depuis la pause, filtré par clone.
6. `docs/jouer.md` : section manette (navigateur compris).
7. Essai E4 avec William (`dette-d69-e4-personnage.md` §8) **dans le navigateur** : manette branchée/débranchée, premier
   appui, mutation, lâcher d'arme, deux joueurs.

## Acceptation

- Une partie de zombies et une d'un autre clone jouées **entièrement à la manette dans Chrome** (tirer, viser, dasher/recharger,
  actif ou bombe, menus, fin de partie), notes de William.
- Schéma validé par écrit ; écran d'aide capturé.
- Les bits d'input de `PlayerScript` et **toutes les traces inchangés** (`make test_scenarios` vert sans `BLESS`).
- Rapport `docs/taches/rapports/polissage-manette.md` (ce qui marche en wasm, ce qui n'est pas vérifié).
