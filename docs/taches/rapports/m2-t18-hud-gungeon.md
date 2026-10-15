# M2-T18 seconde partie : écrans en `bevy_hui`, HUD gungeon

Branche `m2-t18-hud-gungeon`, créée depuis `origin/main`. Conventions : `docs/conventions.md` §15, §32, §56.
Décision de William (2026-10-15) : `bevy_hui` pour les écrans ; le HUD reste la couche maison `ui/hud.ron`.
Présentation seule : rien dans `GgrsSchedule`, aucun type rollback nouveau, **aucune trace modifiée**
(`git diff origin/main -- '*.trace'` vide).

## 1. Fait

### Écrans : `bevy_hui` seul (point 1)

- Supprimés : `crates/game/src/ui/screens/custom.rs`, `games/gungeon/assets/ui/{pause,inventory}_screen.ron`,
  `ScreenBackend` et la variable `ALACOD_SCREENS`.
- Feature : `game/ui_hui` est activée par la feature `render` des quatre jeux (défaut natif et wasm) et de
  `scenario` (rendu, `play_scenario`) ; elle n'est jamais activée par les builds headless. Les alias
  `gungeon/ui_hui` et `scenario/ui_hui` du comparatif ont disparu (inutiles).
- Modèle commun conservé (`screens/model.rs`, `screens/mod.rs`) avec ses tests.
- Colle de `hui.rs` réduite à une liaison **générique** des sections de la `ScreenView` : `Text` → propriété
  `{nom}`, `Bar` → `{nom_label}` et `{nom_width}`, `List` → conteneur `id="nom"` rempli de lignes
  `ui/hui_row.html` (marqueur, couleur, fond du focus). Plus de `e0`/`c0`/`b0` par écran : la pause utilise la
  même liste que l'inventaire. Pas de fork de `bevy_hui`.
- Fichiers HTML découverts directement dans les dossiers `Ui` (plus besoin d'un `.ron` compagnon).
- La correction de la panique (inventaire modifié pendant qu'il est ouvert : `sync_screen` avant
  `HuiSystems::Build`) reste couverte par le test `inventaire_lignes_creees_par_le_code` ; rechargement à chaud
  couvert par `menu_de_pause_et_rechargement_a_chaud` et le test du watcher.

### Écrans existants (point 2)

- **Écran de fin** (`ui/game_over.rs`) : porté en `bevy_hui`, gabarit `ui/game_over_screen.html` dans les
  quatre jeux. Résumé toujours dérivé de `Run` (mis à jour en place, l'écran disparaît si un rollback annule
  l'issue) ; boutons « Rejouer (R) » et « Lobby » repérés par `id`, mêmes `RunRequest` ; la touche R ne relance
  toujours que depuis l'écran affiché (test conservé). Le plugin est passé de `GameUiPlugin` à
  `PresentationPlugin` (rendu seul).
- **Écran de mutation : non porté, dette.** Le modèle commun des écrans ne s'y prête pas : `MutationScreenView`
  décrit trois cartes chronométrées, et valider émet un bit `ChoiceA/B/C` dans l'**input rollback** du joueur
  (`confirm_input`), alors que `ScreensState` ne produit que des commandes locales. Le porter demande un adaptateur
  propre (cartes en grille, barre de temps, protocole d'input inchangé) : dette **D-T18a** ci-dessous.

### HUD gungeon (point 3)

- Nouvelles sources (`content::ui::HUD_SOURCES`, calculées par `hud_model::hud_values`, pures, dans
  `HudSnapshot` aussi en headless) : `blanks`, `keys`, `shells`, `passives`, `active_item`, `active_charge`
  (texte « 2 / 2 · prêt » et barre), `minimap`.
- `shells` lit `Currency` : les douilles ramassées créditent cette monnaie (M2-T6), pas un consommable.
- Nouveau widget `Map(source)` (rendu maison, schéma partagé avec le lint) : rectangles des salles de l'étage
  courant à l'échelle, cadrage stable ; courante bordée d'or, verrouillée rouge avec « × », nettoyée verte,
  boss violet avec « B », dormantes en silhouette. Passifs en `Icons` (ordre de ramassage, doublons gardés).
- Minicarte dérivée de `RoomStates` et de la géométrie de `CurrentRooms` (pont `map_ldtk` → ressource de
  présentation `HudRoomGeometry`, en `Update` avant `HudModelSet`) : aucun état nouveau dans la simulation,
  aucun cache de visites.
- `games/gungeon/assets/ui/hud.ron` : étage et compteurs en haut à gauche, passifs dessous, carte en haut à
  droite, actif et sa charge au-dessus de la vie en bas à gauche, arme en bas à droite.
- Attentes `HudText` ajoutées à des scénarios gungeon existants : `ui_inventory`, `gungeon_start`,
  `gungeon_clear`, `gungeon_boss`, `gungeon_two_floors` (traces inchangées : le HUD est hors trace).

### Lint des `.html` (point 4)

- `Registry` découvre les `.html` des dossiers `Ui` et vérifie qu'ils se lisent (UTF-8 : sinon `Parse`) ; un
  écran pause/inventaire exige `ui/hui_row.html` lisible (sinon `BrokenReference`).
- Fixtures `html_screen_missing_row` (échec attendu) et `html_screens_valid` ; l'inventaire de contenu embarqué
  pour wasm (`scripts/embed-game-content.rs`, test `embedded`) inclut les `.html`.
- Limite : la grammaire de `bevy_hui` n'est pas validée par le lint.

## 2. Vérifié

**Suite verte sur la fusion avec main `c82d7ad7`** (merge `a191a334`, sans `BLESS`) :

| Commande | Résultat |
|---|---|
| `make test_scenarios` (toute la suite, sans `BLESS`) | `scenarios ... ok`, 330 lignes de scénario jouées, `3 passed; 0 failed; 8 ignored`, aucune trace réécrite |
| `make test_crates` | 42 binaires de test, **735 réussis, 0 échec, 10 ignorés** |
| `make lint` ×4 | zombies, testbed, throne, gungeon : « aucune erreur » (avertissements préexistants : 1, 53, 1, 20) |
| `cargo fmt --all -- --check` | rien à signaler |
| `./scripts/check-forbidden.sh` | 9 occurrences, toutes préexistantes (aucune dans `ui/`) |
| `./scripts/check-rollback-registration.sh` | OK |
| `cargo test -p game --features ui_hui,bevy/file_watcher --lib ui:: -- --include-ignored` | 44 réussis, 0 échec ; watcher réel : modification visible après 331 ms |
| `APP_VERSION=x cargo check -p gungeon --target wasm32-unknown-unknown` (défaut, donc `ui_hui`) | vert |

Deux corrections après la fusion, toutes deux limitées aux tests de `hui.rs` (rien dans un binaire de jeu ni
dans la simulation, d'où la suite non rejouée) : la couleur du focus se compare en octets sRGB (la valeur passe
maintenant par une propriété texte), et le dossier temporaire est canonique (sur macOS `/var` → `/private/var`
faisait paniquer le watcher de Bevy).

**Captures** (machine de développement macOS, rendu `play_scenario --capture`, 960×540), dans
`docs/taches/rapports/m2-t18-hud-gungeon/` :

| Fichier | Scénario, frame | Ce qu'on voit |
|---|---|---|
| `hud-actif-pret.png` | `ui_inventory` f9 | blanks 2, clés 1, douilles 500, passifs A/B, « Fiole de rage 2 / 2 · prêt », carte (départ courant, boss B) |
| `inventaire-actif-vide.png` | `ui_inventory` f20 | inventaire Hui ouvert, focus sur l'arme, actif « 0 / 2 » après usage |
| `pause-focus.png` | `ui_pause_menu` f20 | focus « > Recommencer » (couleur et fond) |
| `carte-verrouillee.png` | `gungeon_clear` f100 | salle courante verrouillée (rouge, ×), clé ramassée |
| `carte-nettoyee.png` | `gungeon_clear` f650 | même salle nettoyée (verte), douilles 700 |
| `victoire.png` | `gungeon_boss_bot` f550 | écran de fin Hui : « VICTOIRE », étage 1, 3 éliminés, 0:07, deux boutons |

Frames à regarder pour l'orchestrateur : pause 10, 20–22, 31, 40, 51, 62, 64 ; inventaire et HUD 9, 12, 20,
21–24, 31 ; `gungeon_clear` 20, 60, 100, 650 ; `gungeon_boss` 320, 450, 480, 520 (boss et blanks consommés) ;
`gungeon_two_floors` 800 ; fin `gungeon_boss_bot` 450 (fondu) et 550.

## 3. Non fait, non vérifié

- **Non vérifié** : p2p à deux clients (Docker) et bench au calme (hors de portée ici ; rien dans la simulation
  ni la session n'a changé) ; manette et souris réelles (les boutons de l'écran de fin n'ont pas été cliqués) ;
  navigateur (wasm vérifié par `cargo check` seulement) ; captures de `gungeon_boss` et `gungeon_two_floors`
  (leurs attentes `HudText` passent dans la suite, images non regardées).
- `make gen` non lancé : aucune arme changée.
- Les touches de jeu restent actives sous un menu ouvert (aucun verrou d'input ajouté, décision non prise).

## 4. Dettes

- **D-T18a — écran de mutation en `bevy_hui`** : garder `MutationScreenView` et le protocole `ChoiceA/B/C`,
  écrire un gabarit de cartes et un adaptateur ; retirer ensuite `mutation_screen.ron` et son schéma de lint.
- **D-T18b — mémoire de visite de la minicarte** : `RoomStates` ne garde pas la trace d'une salle non
  combattante (boutique, récompense) visitée puis quittée : elle reste `Dormant` et la carte la dessine en
  silhouette. Corriger demande un état nouveau dans la simulation (traces à rebénir, preuve §5).
- Lint `.html` limité à la lecture et à `hui_row.html` (pas de validation de la grammaire `bevy_hui`).

## 5. Atteignabilité

Une partie ordinaire de `gungeon` affiche le HUD à chaque frame (`games/gungeon/assets/ui/hud.ron`, chargé par le
jeu) ; pause et inventaire s'ouvrent par Échap et I, l'écran de fin à la fin de la run. Scénarios qui les
jouent : `gungeon_start`, `gungeon_clear`, `gungeon_boss`, `gungeon_two_floors` (HUD, `HudText`),
`ui_pause_menu`, `ui_inventory` (écrans, `ScreenText`), `gungeon_boss_bot` (victoire, écran de fin).
