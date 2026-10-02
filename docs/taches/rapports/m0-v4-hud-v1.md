# T2.12 — HUD v1 et écran de résumé, livraison

SHA de la tête de branche vérifiée (code, contenu, captures) :
`6185e41` (voir `git log` de `m0-v4-hud-v1`).
Fiche : `docs/taches/T2.12-hud-v1-resume.md`.
Base : main `80123e9` (après le merge de T2.8).
Date : 2026-10-02. Agent : Claude (Claude Code, session cloud).
Environnement : variante cloud (README §1), clone `estran-studio/alacod` seul, 4 cœurs.
Le SHA livré inclut ensuite le commit de ce rapport, purement documentaire.

**Vérification standard verte, hors p2p et bench (non faisables ici). Aucune trace
modifiée.** Aucun merge dans main, aucune modification de `docs/taches.md`, aucune autre
fiche commencée.

## 1. Fait

Deux commits de travail :
- `7ae4c3e` : code et contenu (`hud.rs`, `game_over.rs`, `interaction.rs`, `hud.ron` des
  deux jeux).
- `6185e41` : doc (`CLAUDE.md` § HUD, `docs/conventions.md` §15) et captures
  (`docs/captures/hud-v1/`).

Livrables de la fiche :

1. **Sources** (`crates/game/src/ui/hud.rs`, système `update_hud_v1_values`, `Update`
   seulement) :
   - `perks` : nouveau widget `Icons(source)`, un carré coloré avec étiquette par perk
     (table `icons` de `hud.ron`, repli gris + initiale) ; avec `Text`, « J S ».
   - `downed` : « À TERRE 12 s » et « Réanimation 40 % » ; le réanimateur voit aussi la
     progression du joueur qu'il relève.
   - `powerups` : une ligne par power-up actif (modificateurs `powerup:<id>` avec
     `frame <= until`), nom tiré de `items/powerups.ron`.
   - `prompt` : l'interactable que l'appui sur Interaction toucherait, **avec la même
     règle** que `interaction_detection_system` (distance à la surface du collider, portée,
     ordre `GgrsNetId`). Textes : « Ouvrir — $750 », « Acheter fusil à pompe — $1000 »,
     « Munitions pistolet — $250 » (arme déjà possédée, `refill_price`), « Ramasser … »,
     « Juggernog — $2500 » / « Juggernog — possédé », « Réanimer », « Réparer » (fenêtre
     abîmée seulement). Noms d'armes : table `names` de `hud.ron`.
   - Les textes sont produits par des fonctions pures, couvertes par 7 tests unitaires
     dans `hud.rs`.
2. **`hud.ron`** de zombies : perks en bas à gauche au-dessus de la vie, `downed` au centre,
   `powerups` en haut à droite, `prompt` en bas au centre. Testbed : `downed`, `powerups`
   et `prompt`, car il perd l'ancien texte (voir décisions).
3. **Écran de fin** : bouton « Lobby » (`RunRequest::ToLobby`) à côté de « Rejouer (R) »,
   résumé inchangé.
4. **Doc** : `CLAUDE.md` § HUD et `docs/conventions.md` §15.
5. **Captures** dans `docs/captures/hud-v1/`, chacune < 100 Ko, avec un `README.md` qui
   les décrit et donne la commande :

| Fichier | Scénario et frame |
|---|---|
| `prompt_achat.png` | `shop_tour` f100 |
| `perk_powerup.png` | `perk_powerup.ron` f260 |
| `a_terre.png` | `downed_revive --follow 1` f200 |
| `ecran_fin.png` (en plus) | `downed_all_lose` f1506 |

Décisions, prises d'après ce que montraient les captures :
- **Police** : la police par défaut de Bevy n'a ni accents ni tiret long. « possédé »,
  « À TERRE » et le « — » des prix s'affichaient en carrés, et c'était déjà le cas de
  « DÉFAITE » sur l'écran de fin. Nouveau champ `font` à la racine de `hud.ron`
  (`fonts/FiraMono-Medium.ttf`, présente dans les deux jeux). L'écran de fin utilise la
  même police.
- **Contraste** : nouveau champ `background` par widget (`#rrggbbaa`), masqué quand le
  texte est vide, avec une hauteur qui suit le nombre de lignes. Appliqué à `downed`,
  `powerups` et `prompt`.
- **Placement** : la ligne de debug « Wave 1 | PREP … » occupe le bas de l'écran. Les perks
  (offset 80) et le prompt (offset 110) sont placés au-dessus. `downed` est remonté de
  150 px pour ne pas couvrir les personnages.
- **Ancre `Center`** ajoutée : la fiche place `downed` « au centre ». Le texte est centré
  pour les ancres centrées, aligné à droite pour celles de droite.
- **Joueur du HUD** : celui de `CameraFollowOverride` (`play_scenario --follow`), sinon le
  joueur local de plus petit handle. Avant, c'était un `LocalPlayer` arbitraire quand
  plusieurs joueurs sont locaux. Ce choix s'applique aussi aux sources existantes (vie,
  munitions, monnaie).
- **Ancien prompt retiré** d'`interaction.rs` : il était en anglais et ne couvrait ni
  armes ni perks. Le garder aurait affiché deux prompts. Les cercles de portée
  (gizmos) restent.
- **Fin de run** : les sources T2.12 sont vidées quand `Run.step` vaut `Ended`. Sinon,
  « À TERRE » d'un joueur qui saigne encore recouvrait le titre « DÉFAITE ».
- **Recette `perk_powerup.ron`** : aucun scénario ne montre à la fois un perk et un
  power-up. C'est `shop_tour` plus deux power-ups posés au point d'apparition. Le fichier
  est rangé dans `docs/captures/hud-v1/`, hors de `tests/scenarios`, donc sans trace et
  jamais joué par les tests.

## 2. Vérifié (dans ce clone)

| Commande | Résultat |
|---|---|
| `make test_scenarios` sur main `80123e9`, avant de coder | 58 scénarios, `2 passed; 0 failed; 7 ignored` |
| `make test_scenarios` sur la branche | 58 scénarios (48 + 10 générés), `2 passed; 0 failed; 7 ignored` ; `git status` : aucun `.trace` modifié |
| tests des dix crates (§4) | 264 réussis, 0 échec, 8 ignorés (257 de la référence + 7 nouveaux tests unitaires du HUD) |
| `make lint` | zombies et testbed : « aucune erreur » |
| `cargo fmt --all -- --check` | rien à afficher |
| `./scripts/check-forbidden.sh` | 4 avertissements, les mêmes que la référence |
| `./scripts/check-rollback-registration.sh` | OK |
| captures `play_scenario --capture` (profil headless, `--features render`, Xvfb + lavapipe) | 4 captures regardées une à une ; trois itérations de réglage d'après ce qu'elles montraient |

`make gen` n'a pas été lancé : aucune arme n'a changé.

Ce que les captures montrent, à la dernière itération :
- **Prompt** de `shop_tour` : « Acheter pistolet — $500 » (f100), « Munitions pistolet —
  $250 » (f120-150), « Juggernog — $2500 » (f160), « Juggernog — possédé » (f170).
- **Power-ups** : « Double Points 26 s » et « Insta-Kill 26 s », ainsi que l'icône « J ».
- **À terre** : « À TERRE 14 s / Réanimation 47 % ».
- **Écran de fin** : « DÉFAITE » avec son accent, et les deux boutons.

## 3. Non fait, non vérifié

- **p2p à deux clients** et **bench strict** : non faisables dans le cloud. Le HUD est
  absent en headless, et aucune trace n'a changé.
- **Le clic sur « Lobby » n'a pas été testé** (pas de fenêtre ni de souris) ; il pose
  `RunRequest::ToLobby` par le même chemin que « Rejouer ». Le bouton est seulement vu en
  capture.
- **Pas de partie jouée à la main** (`make zombies`, fenêtre impossible).
- Prompts **non vus à l'écran** : porte, fenêtre, réanimation, arme au sol. Leurs textes
  sont couverts par les tests unitaires, et la sélection de l'interactable est celle qui
  produit les prompts vus en capture.
- HUD du testbed non capturé ; rechargement à chaud de `hud.ron` non testé.
- Le choix du « joueur du HUD » en ligne : un seul joueur local, donc lui. Non vérifié en
  p2p.

## 4. Dettes et questions ouvertes

- Les textes de debug (caméra en haut à gauche, « Wave 1 | PREP … ») chevauchent
  « Joueurs N » en haut à gauche ; c'était déjà le cas avant, je n'y ai pas touché.
- Pas de bouton ni de raccourci clavier « L » pour Lobby : seul le bouton existe, comme le
  demande la fiche.
- Session cloud : la place disque est juste (build headless et rendu ≈ 36 Go) ; j'ai dû
  supprimer des binaires de test périmés en cours de route, sans effet sur les résultats.
