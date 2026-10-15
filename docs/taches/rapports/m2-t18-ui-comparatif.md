# M2-T18 partie 1 — test comparatif d'UI (menu de pause et inventaire gungeon)

Branche `m2-t18-ui-comparatif`. Conventions : `docs/conventions.md` §56. Plan : `docs/plan-engine.md` §7.1.
Présentation seule, aucune trace existante n'a bougé. **William choisit** ; ce rapport donne les pièces et
une recommandation argumentée, pas une décision.

## 1. `bevy_hui` et Bevy 0.19.1

`bevy_hui` **0.7.0** (2026-06-20) dépend de `bevy ^0.19` : compatible avec le Bevy 0.19.1 du dépôt, il
compile et passe les tests. La 0.8.0 (2026-10-11) exige `bevy ^0.20` : inutilisable tant que le dépôt
reste en 0.19. Le crate suit Bevy une version mineure à la fois (table de son README) : chaque montée de
Bevy attend une release du crate. Pas de fork. Ajouté derrière la feature optionnelle `ui_hui` de `game` et
de `scenario` et `gungeon` ; absent par défaut, jamais dans une crate de simulation.

## 2. Ce qui est livré

| Pièce | Fichier |
|---|---|
| Modèle commun, pur (état, actions, vues, focus, ordre du focus) | `crates/game/src/ui/screens/model.rs` |
| Entrées (Échap/Start, I/Select, ↑↓/D-pad, Entrée/A, Retour/B), état, script de scénario | `crates/game/src/ui/screens/mod.rs` |
| Option A, couche maison étendue | `screens/custom.rs`, `games/gungeon/assets/ui/{pause,inventory}_screen.ron` |
| Option B, `bevy_hui` | `screens/hui.rs`, `games/gungeon/assets/ui/{pause_screen,inventory_screen,hui_row}.html` |
| Réglage | `ALACOD_SCREENS=custom\|hui` (défaut `custom`) ; `hui` exige `--features ui_hui` |
| Scénarios | `tests/scenarios/ui_pause_menu.ron`, `ui_inventory.ron` (+ `.trace`) |
| Attente et script | `ScreenText(screen, section, contains, closed, at_frame)` ; champ de scénario `ui: [(at_frame, action)]` |

Le menu a quatre entrées (Reprendre, Recommencer = `RunRequest::Restart`, Lobby = `RunRequest::ToLobby`,
Réglages vide). L'inventaire montre armes (chargeur | réserve), objet actif et sa charge (texte et jauge),
passifs, consommables (blanks, clés), lus dans `WeaponInventory`, `AmmoReserves`, `items::Inventory`,
`ItemTable`. Le focus (marqueur `>`, couleur, fond) est visible et identique dans les deux options.

**Aucun état de jeu touché** : tout vit dans `Update`, hors rollback ; le script `ui` du scénario et les
touches n'entrent jamais dans l'input GGRS. Conséquence assumée, comme l'écran de mutation (§30) : la
simulation continue pendant qu'un écran est ouvert (GGRS ne peut pas s'arrêter), et les touches de jeu
restent actives. Les bloquer est une décision de la partie 2. La souris n'est pas câblée.

## 3. Tableaux des critères

Mesures faites ici (machine sans GPU : pas de fenêtre ni de manette). Les nombres de lignes sont des lignes
non vides hors commentaires et hors tests.

### Option A : couche maison étendue (`custom.rs` + `.ron`)

| Critère | Résultat |
|---|---|
| wasm | `cargo check -p game --target wasm32-unknown-unknown` : **vert** (16 s incrémental). Pas lancé dans un navigateur. |
| Rechargement à chaud | **Oui, mesuré.** Chemin de rechargement (`AssetServer::reload`) : arbre reconstruit en 2 updates, 2 à 7 ms. Watcher de fichiers réel (`bevy/file_watcher`, test ignoré `watcher_de_fichiers_detecte_la_modification`) : **375 ms** entre l'écriture du fichier et l'arbre reconstruit (la latence est celle du watcher de Bevy, identique à l'option B). |
| Manette | Le focus est celui du modèle commun (D-pad, A, B, Start, Select lus sur `Gamepad`). La table touches → actions est pure et testée ; **la lecture d'une vraie manette n'a pas pu être essayée** (aucun périphérique). Focus visible : oui (captures). |
| Lignes de code | `custom.rs` 357 (interpréteur **générique**, une fois pour tous les écrans futurs) ; données : pause 17, inventaire 24 lignes RON. |
| Ajouter un écran | Un fichier de données si les sections existent déjà (`Text`, `List`, `Bar` liées par nom) ; sinon une section dans le modèle. Pas de code de rendu. |
| Temps d'écriture | Non chronométré. Écrit d'un jet, compilé et passé à la première relecture ; aucun défaut trouvé au rendu. |
| Dépendances ajoutées | **Aucune** (`bevy_common_assets` et `RonAssetPlugin` déjà là pour le HUD). |
| À jeter à l'arrivée de `.bsn` | `custom.rs` (357 lignes) et les `.ron` d'écrans (~41 lignes) : remplacés par des fichiers `.bsn`. Le modèle (`model.rs`, `mod.rs`, 569 lignes) reste. |
| Limites du langage | Boîtes `Column`/`Row`, texte, listes, jauge. Pas d'image, de grille, d'état au survol, de transition : à écrire si on les veut. |

### Option B : `bevy_hui` 0.7 (`hui.rs` + `.html`)

| Critère | Résultat |
|---|---|
| wasm | `cargo check -p game --target wasm32-unknown-unknown --features ui_hui` : **vert** (6 min 19 s à froid avec Bevy). Pas lancé dans un navigateur. |
| Rechargement à chaud | **Oui, mesuré.** `reload` : reconstruit en 3 updates, 4,5 ms (pause) et 9 à 11 ms pour l'inventaire (les lignes créées par le code sont recréées). Watcher réel (`watcher_de_fichiers_detecte_la_modification_hui`) : **314 ms**. Le rechargement détruit les enfants du gabarit : tout ce que le code y a ajouté doit être recréé (fait, testé). |
| Manette | `bevy_hui` n'a **pas** de navigation au focus (ni clavier, ni manette) : on réutilise le modèle commun ; les `{propriétés}` portent le texte et la couleur du focus. Même réserve que A sur la manette réelle. Focus visible : oui (captures). |
| Lignes de code | `hui.rs` 227 (colle propres à **chaque** écran : `view_properties` nomme `e0`…`e3`, `charge_width`…) ; données : pause 29, inventaire 24, ligne 11 lignes HTML. |
| Ajouter un écran | Un `.html`, **plus** du code : les propriétés de l'écran (`view_properties`), et pour toute liste variable un conteneur `id` rempli par le code. Pas de boucle ni de condition dans le langage. Généraliser cette colle revient à écrire la couche de A. |
| Temps d'écriture | Non chronométré. Plus long : lecture des sources du crate (pas de documentation dans le paquet publié), et **deux défauts invisibles aux tests sans rendu**, trouvés aux captures logicielles (voir §4). |
| Dépendances ajoutées | `bevy_hui` 0.7.0 et `owo-colors` 4.4.0 dans `Cargo.lock` (`nom` 7 et `thiserror` 1 y étaient déjà). Optionnelles (`ui_hui`). |
| À jeter à l'arrivée de `.bsn` | `hui.rs` (227), les 5 `.html` (~100 lignes) et la dépendance ; la syntaxe pseudo-HTML ne se convertit pas mécaniquement en `.bsn`. Même modèle commun conservé. |
| Capacités en plus | Survol et transitions (`hover:`, `delay`, `ease`), images, grille, composants à emplacements (`<slot>`), `on_press` (souris). **Aucune n'est utilisée** par ces deux écrans. |

### Ce que les deux partagent (569 lignes + 171 de tests)

`model.rs` et `mod.rs` : état, actions, `reduce`, vues, ordre du focus, lecture de l'inventaire, entrées,
script de scénario. C'est l'essentiel du travail, et il ne dépend d'aucune option. Le choix ne porte que
sur ~350 lignes de rendu et ~100 lignes de données par option.

## 4. Défauts trouvés par les captures logicielles (option B)

Les tests sans rendu (arbre `Text` dans une `App` minimale) passaient. Un rendu réel (Xvfb + lavapipe :
`xvfb-run`, `VK_ICD_FILENAMES=…/lvp_icd.json`, `play_scenario --capture`) a montré :

1. **Panique de `bevy_hui`** (`spawn_ui`) quand l'inventaire change pendant qu'il est ouvert : le code
   détruisait des lignes que `bevy_hui` construisait dans la même frame (commande sur une entité détruite).
   Corrigé : `sync_screen.before(HuiSystems::Build)`.
2. **Fond de la ligne focalisée absent** : une propriété lue par le nœud **racine** d'un gabarit n'est pas
   recompilée. Corrigé : un nœud enveloppe dans `hui_row.html`.

L'option A n'a eu aucun défaut au rendu. Une fois corrigées, les deux options rendent la pause **au pixel
près** quand la mise en page est la même (`pause_f32_identique_maison_et_hui.png`, même somme md5).

## 5. Recommandation (William décide)

1. Le plus gros du travail (modèle, focus, entrées, scénarios) est commun : l'option ne se joue que sur le rendu.
2. Pause et inventaire sont des **listes avec un focus et des jauges** ; le HUD gungeon (partie 2) aussi.
   La couche maison les décrit en données sans code de rendu par écran ; `bevy_hui` demande de la colle Rust
   par écran, faute de boucle.
3. Rechargement à chaud : équivalent (≈ 350 ms de watcher, puis 2 à 3 updates), wasm vert pour les deux.
4. `bevy_hui` coûte une dépendance qui suit Bevy version par version et qu'il faudra jeter avec `.bsn` ;
   la couche maison aussi se jette, mais elle ne s'ajoute pas au graphe de dépendances.
5. En faveur de `bevy_hui` : survol, transitions, images, grille, composants à emplacements, que A ne sait pas faire.
   Si l'habillage des menus (animations, panneaux en images) compte plus que la maintenance, c'est lui.
6. Si les écrans restent des listes et des jauges, la couche maison suffit et coûte moins.
7. Un point à trancher quel que soit le choix : les touches de jeu restent actives sous un écran ouvert
   (partie 2 : un « verrou d'input » local, hors simulation, ou rien).
8. Aucun des deux n'a été essayé avec une vraie manette ni une souris.

## 6. Preuve

- **Tests purs** (`model.rs`, 8) : boucle du focus, ordre du focus (armes, actif, passifs, consommables), un
  seul élément focalisé, textes, actif prêt, navigation et commandes de la pause, touches → actions.
  `custom.rs` (3 dont 1 ignoré) et `hui.rs` (6 dont 1 ignoré) : construction de l'arbre, rechargement à chaud,
  changement de vue sans commande orpheline, fichiers des jeux lisibles.
- **Scénarios** `ui_pause_menu` (90 frames) et `ui_inventory` (60 frames) : le script `ui` ouvre l'écran et
  navigue ; 14 et 13 attentes `ScreenText` ; l'inventaire suit la simulation (la fiole utilisée passe de
  « 2 / 2 · prêt » à « 0 / 2 »). Traces enregistrées seulement pour ces deux scénarios.
- **Captures** (ici, rendu logiciel lavapipe, 960×540) : `docs/taches/rapports/m2-t18-ui-comparatif/`.
  Pour l'orchestrateur (GPU), les commandes et frames sont dans `docs/conventions.md` §56 :
  `play_scenario … --capture /tmp/ui/<nom> --every 2`, option B avec `ALACOD_SCREENS=hui` et
  `--features render,ui_hui` ; frames pause 10, 20–22, 31, 40, 51, 62, 64 ; inventaire 12, 20, 21–24, 31.

## 6bis. Suite rejouée sur la fusion avec main

**Suite verte sur la fusion avec main `b527bdb`** (merge `9f262c9`, `origin/main` contenu dans la branche à la
livraison), sans `BLESS` :

| Commande | Résultat |
|---|---|
| `make test_scenarios_par` (4 tranches, 409 s) | **299 scénarios verts** (297 existants + `ui_pause_menu`, `ui_inventory`), aucune trace réécrite, `git status` propre |
| `make test_crates` | **718 réussis, 0 échec, 11 ignorés** (42 binaires de test) |
| `make lint` ×4 | zombies, testbed, throne, gungeon : « aucune erreur » (gungeon : 20 avertissements d'armes sans source, préexistants) |
| `cargo fmt --all -- --check` | rien à signaler |
| `./scripts/check-forbidden.sh` | 9 occurrences, toutes préexistantes (aucune dans `ui/screens/`) |
| `./scripts/check-rollback-registration.sh` | OK |
| `cargo check` wasm32 (`-p game`, avec et sans `ui_hui`) | vert |
| tests de `game` avec `ui_hui` (`screens`) | 14 réussis ; 2 ignorés (watcher) passés à part : 375 ms / 314 ms |

Aucune trace n'a bougé : pas de preuve §5 nécessaire (aucun type rollback ni système de `GgrsSchedule`
ajouté). `make gen`, p2p (Docker) et bench non lancés : rien d'armes, de session ni de simulation changé.

## 7. Non fait, non vérifié, dettes

- **Non vérifié** : manette réelle, souris, navigateur (wasm vérifié par `cargo check` seulement), rendu sur
  GPU (captures logicielles seulement), p2p à deux clients (présentation seule : rien dans `GgrsSchedule`,
  `rollback_*` inchangé), bench au calme.
- **Pas de lint typé** pour les `.ron` d'écrans (kind `Ui`, syntaxe RON seulement ; un test de `game` lit les
  fichiers des jeux). Les `.html` ne sont pas lus par le lint.
- **Les écrans n'existent que dans gungeon** ; zombies, testbed et throne n'ont ni pause ni inventaire.
- La liste vide (« (aucune) ») n'est faite que dans l'option A.
- **Atteignabilité (M2, `docs/taches.md` §7)** : le scénario `ui_inventory` (objets tenus dès la frame 1) et
  `ui_pause_menu` jouent les écrans dans gungeon ; une partie ordinaire ouvre ces écrans par Échap et I. Le
  HUD gungeon (blanks, clés, actif, minicarte) est la partie 2, après la décision.
