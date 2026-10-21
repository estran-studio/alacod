# polissage-manette — manette jouable et écran d'aide des touches (B4)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 2), `crates/game/src/character/control.rs`
(`PlayerAction`, `get_input_map` lignes 58-108), `input_devices.rs:23-50`, `docs/jouer.md:71-97`,
`docs/taches/rapports/dette-d69-e4-personnage.md` §8-9, `docs/conventions.md` (bits d'input : `INPUT_*`). Branche
`polissage-manette` depuis `origin/main`. Estimation : 3 j. Clones : les quatre. Machine : Mac avec affichage et **une
manette physique** ; l'essai E4 est de William.

## Contexte (mesuré, inventaire §2)

À la manette on ne peut **pas tirer** : `PointerClick` = clic gauche seul (`control.rs:102`), aucune gâchette. La visée est
le stick gauche alors que le déplacement est au D-pad. Aucune liaison pour Sprint, Dash, SwitchWeapon, SwitchWeaponMode,
DropWeapon, UseActive, Blank, Bomb, ChoiceA/B/C, Modifier. Aucun remappage. Essai de William jamais fait.

## À faire

1. Schéma manette (proposition à valider avant d'écrire, une table dans le rapport) : déplacement au stick gauche
   (le D-pad reste), **visée au stick droit**, tir = gâchette droite, mêlée/recharge/interaction aux boutons existants,
   Dash et Sprint aux gâchettes/épaules, SwitchWeapon/Mode, DropWeapon, UseActive, Blank, Bomb sur les boutons restants,
   ChoiceA/B/C sur D-pad + Sud. Le choix du stick droit **change la visée pour la manette** : vérifier la dérivation
   de l'angle (`Pan`) dans `input_devices.rs` et que la donnée entre toujours par l'input GGRS (jamais lue dans la
   simulation).
2. Un seul fichier de vérité : `get_input_map` ; chaque action de `PlayerAction` utile aux clones a une liaison clavier et
   une liaison manette, ou une exception écrite dans le rapport (actions de debug : P, O, F-touches).
3. Écran d'aide (présentation, `ui/screens`, ouvert depuis la pause) qui liste les touches **lues dans la table**
   (pas recopiées à la main), clavier et manette, filtrées par ce que le clone utilise (une bombe d'isaac absente de
   zombies n'apparaît pas).
4. Mettre à jour `docs/jouer.md` (§ touches et manette).
5. Essai E4 avec William : clavier/souris, manette branchée/débranchée, premier appui, mutation, lâcher d'arme, deux
   joueurs ; cocher `dette-d69-e4-personnage.md` §8-9.

## Acceptation

- Une partie de zombies et une d'isaac jouées **entièrement à la manette** (tirer, viser, dash, recharger, actif, bombe,
  choix de mutation), vidéo ou captures et note de William.
- Les bits d'input de `PlayerScript` et toutes les traces inchangés (`make test_scenarios` vert sans `BLESS`).
- Écran d'aide ouvert depuis la pause, capture dans le rapport.
- Rapport `docs/taches/rapports/polissage-manette.md`.
