# Rapport assets-lot-c — sprites de `gungeon` (docs/plan-assets.md, lot C)

Présentation seule : aucune trace existante ne bouge (voir Vérifications).

## Choix du pack
**Tiny Dungeon** de Kenney (https://kenney.nl/assets/tiny-dungeon), **CC0** (page lue le 2026-10-15 :
« License Creative Commons CC0 » ; `License.txt` de l'archive copié dans le dépôt), cases de 16 px comme les
cases LDtk. Projectiles : **Particle Pack** de Kenney (CC0, même vérification). Archives et sha256 dans
`games/gungeon/assets/assets.yaml` (entrées `sprites-tiny-dungeon-kenney`, `sprites-particle-pack-kenney`).
Aucun fichier CC BY : pas d'attribution obligatoire, la licence CC0 est quand même copiée à côté des images.

## Ce qui est dessiné
| Quoi | Id (`sprites.ron`) | Case de `tiny_dungeon.png` |
|---|---|---|
| Héros (pistolero) | `player` | 85 |
| 10 ennemis | `bullet_kin` 98, `bullet_kin_squad` 100, `shotgun_kin` 87, `veteran_kin` 96, `bandana_kin` 112, `tourelle_kin` 124, `fuyard_kin` 120, `lanceur_kin` 84, `cible` 97 | cases ci-contre |
| Boss `gatling` | `gatling` | 110, affiché x3,5 (56 px) |
| Coffre (fermé 89, ouvert 91), présentoir 75, douille 101, objet au sol 114, tonneau 82, table 72 | `chest` `shelf` `shell` `item` `barrel` `table` | |
| Projectiles | `bullet_player` (circle_05, jaune), `bullet_enemy` (magic_05, rouge) | images Particle Pack 64 px |

Pas d'animation dessinée : Kenney ne fournit qu'une image par personnage. `anim_<id>.ron` donne la même case à
`Idle`, `Run` et `Dying_1` ; la planche reste prête pour des images de marche (changer `start`/`end`).
Calque `shadow` du héros conservé (blackHUNTERdev) ; `shirt` retiré de l'entrée `player` (les skins qui le citent l'ignorent).

## Code (présentation, hors simulation)
`crates/game/src/world_sprites.rs` (`WorldSpritePlugin`, schedule `Update`) : pose un enfant `Sprite` sur
`Chest`, `Shelf`, `ShellPickup`, `ItemPickup`, `BarrelState`, `TableState` (case `Idle` de l'animation de l'entrée ;
coffre ouvert : `Open` ; présentoir vendu : caché) et remplace le carré de couleur des `Bullet` par l'image
de leur équipe. Un jeu sans l'entrée garde le rendu d'avant (zombies, throne, testbed inchangés). Rien dans `GgrsSchedule`,
aucun composant rollback ajouté. Les `characters/*.ron` de gungeon déclarent le calque `body` (`layers: {"body": ""}`).

## Preuve visuelle (rendu logiciel, pas de GPU)
Xvfb + lavapipe (`VK_ICD_FILENAMES=…/lvp_icd.json`), `play_scenario --capture`. Xvfb n'étant pas installé sur
orca, ses `.deb` (xvfb, xkbcomp, libs) ont été extraits sous `dl/` et lancés dans un espace de noms utilisateur
(`unshare -Urm`, overlay de `/usr/bin` pour `xkbcomp`) : aucun root, rien installé sur la machine.
Captures dans `docs/captures/assets-lot-c/` (960 x 540). **À regarder, William** :
| Capture | Scénario, frame | Ce qu'on voit |
|---|---|---|
| `decor_f120.png` | `assets_gungeon_decor` (nouveau), f120 | héros, coffre fermé, deux présentoirs (étagère brune), deux objets au sol (potion), deux `bullet_kin`, boss |
| `squad_f60.png` | `gungeon_squad`, f60 | escouade de `bullet_kin_squad`, barils, boss |
| `shotgun_kin_f240.png`, `veteran_f240`, `bandana_f240`, `tourelle_f240`, `fuyard_f240`, `lanceur_f240` | `enemy_<id>_moving`/`_still` générés, f240 | chaque ennemi seul face au héros |
| `boss_f450.png` | `gungeon_boss`, f450 | boss `gatling` (x3,5) |
| `explorateur_f100.png`, `explorateur_f200.png` | `bot_explorer_floor`, f100 et f200 | salle de combat : douilles au sol (carré bronze), coffre de récompense, tirs |
Pour rejouer : `play_scenario tests/scenarios/<nom>.ron --capture /tmp/x --every 30` sous GPU.

## Limites et réserves à juger à l'œil
- **Style** : pixel art 16 px sombre, plus petit que les anciens calques 32 px (affiché x2). Les ennemis se distinguent peu
  entre eux à distance (cases proches : chevalier, viking, bandana) ; une couleur de teinte par ennemi serait un réglage de données à ajouter si besoin.
- **Tonneau** (case 82) : montre des cerceaux sur fond sombre, lisible mais pas parfait ; case 66 (dessus rond) en alternative, une ligne dans `anim_barrel.ron`.
- **Coffre ouvert** : Tiny Dungeon n'a pas de coffre ouvert dessiné ; la case 91 (couvercle foncé) en tient lieu.
- **Pas d'animation** (voir plus haut), pas de sprite de mort ni de télégraphe.
- **Murs** : les piliers noirs des salles sont les murs sans tileset de gungeon, hors périmètre (lot E).
- Les armes tenues par le héros gardent les sprites blackHUNTERdev (lot suivant).
- Un joueur « 2 » (skin 2 de `pistolero`) n'a plus de teinte propre.

## Vérifications
Résultats ci-dessous, rejouées après fusion de `origin/main` (62f9aaf3).
