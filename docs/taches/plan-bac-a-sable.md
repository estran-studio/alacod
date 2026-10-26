# Plan du bac à sable de l'engine (chantier K5, lots SB) — rédigé le 2026-10-23

**État au 2026-10-26 : SB-1 à SB-4 livrés et mergés dans `main` (`0fe303f5`) ; reste SB-5 (essai de William sur son Mac).**

| Lot | Merge | Rapport |
|---|---|---|
| SB-1 (+ suites de revue) | `6aa63588`, `b224c896` | `rapports/sb-1-contenu.md` |
| SB-2 | `46047401` | `rapports/sb-2-mise-en-place.md` |
| SB-3 | `fb93c818` | `rapports/sb-3-mode-bac-a-sable.md` |
| SB-4 | `0fe303f5` | `rapports/sb-4-jeu-externe-docs.md` |
| SB-5 | à faire (William) | protocole et fiche : `rapports/sb-5-essai-vocal.md` |

Preuves : 809 scénarios sans `BLESS`, `schedule_order_diff` vide, `make test_crates` paquet par paquet vert, essais headless `--sandbox` sur les cinq jeux, test de bout en bout
`alacod new` (`make test_alacod_cli`) ; revue indépendante de chaque lot. Jamais vu avec rendu (VPS sans affichage) : c'est l'objet de SB-5. Docs : `docs/conventions.md` §85–§88,
`docs/jouer.md`, `docs/editeur-vocal.md`. Dettes ouvertes : D130 (`give` non enregistré), D131 (`player` par défaut en dur), D132 (rendu, `ToLobby`), D133 (`schedule_order_diff`) ; D126 fermée.
Suite après SB-5 : remplir la fiche, décider du pont MCP (`rapports/k5-4-essai-vocal.md` §3), puis polissage des clones (`docs/taches/plan-polissage.md`).

Remplace la fiche `k5-5-bac-a-sable` (spécifique aux clones, retirée). Décision de William (2026-10-23) : « une solution générique d'alacod pour
tester la configuration de nos personnages et ennemis d'un jeu, qui marche pour mes clones internes et pour les jeux créés avec l'engine, comme
mon futur 1837 ». Point de départ : relevé du code du 2026-10-23 (scout, `main` `19e62e42`) ; chaque fait cite `fichier:ligne`.

## 1. But et critère de fin

William lance **n'importe quel jeu construit avec l'engine** (un clone de `games/*`, ou un jeu créé par `alacod new`) en **bac à sable** :
arène calme, sans vagues ni étages, un **profil** de départ (personnage, armes, objets, argent, modificateurs, ennemis posés), clavier ou manette,
**mode éditeur** (remote BRP `alacod/*`, rechargement à chaud des RON, `check_distance = 0`). À la voix (K5) : « spawne-moi tel ennemi devant moi »,
« donne-moi le fusil », « mets-moi 500 sous », « charge le profil lourd », « reset », « un peu plus rapide ».

**Fini quand** :
1. `cargo run -p zombies -- --sandbox` (et gungeon, isaac, throne, testbed) ouvre l'arène du jeu avec son profil `arene` ; `--sandbox <profil>` en choisit un autre.
2. Un jeu créé par `alacod new demo` (gabarit `templates/starter`), **sans une ligne de code ajoutée**, fait de même avec le profil livré par le gabarit.
3. Les phrases ci-dessus marchent par `scripts/alacod-remote` sans relancer le binaire (sauf ce qui est dit comme limite).
4. Aucune trace de scénario ne bouge (`BLESS` interdit), `schedule_order_diff` vide.
5. `examples/character_tester.rs` est retiré (le bac à sable le remplace ; `make character_tester` = alias de `make sandbox GAME=zombies`).

## 2. État relevé (2026-10-23)

| Brique | État | Où |
|---|---|---|
| Mode éditeur, remote BRP, `GameCommands`, `DeviceInputs` | dans `game`, donc offerts à tout jeu | `game/src/core.rs` (natif), `editor_mode.rs`, `remote.rs:454,579,626,680-760`, `game_commands.rs`, `character/player/input.rs:96-290` |
| Types de profil (`Scenario`, `PlayerScript`, placements) | dans `game::replay` | `game/src/replay.rs:40-472` |
| **Application** d'un profil (armes, objets, argent, modificateurs, mutations, dash, bombe, visée ; placements de personnages, objets, coffres, présentoirs) | **dans le crate `scenario` (APP)**, inaccessible à un jeu | `scenario/src/runner.rs:810-1303` |
| Pose d'un personnage à la demande | `map_ldtk` (déjà dépendance du gabarit) | `map_ldtk/src/game/commands.rs:21-127`, `game/local.rs:38-55, 311` |
| Arène de test par jeu | `generate_template.map` dans gungeon, isaac, throne, starter ; **absent** de zombies et testbed (testbed : `start_map: "testbed/arena.ldtk"`, `mode: Sandbox`) | `content/src/manifest.rs:138,152-160` |
| `character_tester` | exemple codé en dur sur `games/zombies`, vagues, murs de test, sans `LdtkLocalGamePlugin` | `examples/character_tester.rs` |
| Jeu externe | `templates/starter` dépend de `game` et `map_ldtk`, lance `CoreSetupPlugin` + `LdtkLocalGamePlugin` ; `alacod new` copie le gabarit | `templates/starter/src/main.rs:1-64`, `content/src/bin/alacod.rs` |
| `RunMode::Sandbox` | existe : aucune condition de victoire | `run/src/modes.rs:66`, `game/src/run_state.rs:125` |

## 3. Décisions (par défaut, sans avis contraire de William)

- **D1 — Un profil est un fichier de contenu** `sandbox/<nom>.ron` du jeu, kind **`Sandbox`** (déclaré dans `content_folders`), lint
  (ids de personnages, armes, objets, ennemis connus du registre), schéma, `alacod describe`, `alacod new-content Sandbox <nom>`. Format : le
  **sous-ensemble utile de `Scenario`** (`players[]` avec les champs de `PlayerScript` sans `inputs`/`bot`, `characters`, `items`, `chests`, `shop`,
  `map` facultatif) ; on réutilise les types de `game::replay`, pas un second format.
- **D2 — Arène** : `map` du profil, sinon `sandbox.arena` du manifeste (champ nouveau facultatif), sinon `generate_template.map`. zombies et testbed
  reçoivent une arène déclarée (lint : un jeu qui a un dossier `Sandbox` doit avoir une arène).
- **D3 — Activation** : `--sandbox [profil]` sur la ligne de commande de **tout** jeu, lu par `game::args` (`BaseArgsPlugin`), sans code dans
  `main.rs` ; il allume le mode éditeur (comme `ALACOD_EDITOR=1`), `RunMode::Sandbox`, coupe les vagues (`ResolvedBalance.waves_enabled`), force la
  carte de l'arène. Session locale, 1 joueur par défaut (`--players` reste possible).
- **D4 — Une seule mise en place** : le code d'application de `scenario/runner.rs:810-1303` passe dans **`map_ldtk::game::profile`** (il a besoin de
  `spawn_character`) ; le runner de scénarios l'appelle ; le bac à sable aussi. Preuve : aucune trace ne bouge.
- **D5 — Remote** : `alacod/sandbox_profile` (liste ; charge un profil = `restart` avec lui), `alacod/give` (arme, objet, argent au joueur vivant,
  par les mêmes fonctions que D4), gardés par `hot_edit_allowed`. `scripts/alacod-remote profile|give`. Pas de MCP (décision K5).
- **D6 — `character_tester` retiré** (cutover propre : exemple, cibles Makefile `character_tester*`, build web) ; alias `make character_tester`.
  Le « Map Tester » reste `make editor` + `restart --seed`.

## 4. Lots (deux voies au plus en parallèle)

| Lot | Contenu | Prérequis | Voie | Estimation |
|---|---|---|---|---|
| **SB-1 Contenu** | kind `Sandbox`, schéma, lint, describe, scaffold, `sandbox.arena` du manifeste et son lint ; fixtures ; profils `arene` + un chargé pour zombies, gungeon, isaac, throne, testbed, et `arene` dans `templates/starter` ; arènes de zombies et testbed déclarées | — | A | 1 j |
| **SB-2 Mise en place commune** | `runner.rs:810-1303` → `map_ldtk::game::profile` (fonctions et systèmes, sans changer un résultat) ; le runner l'appelle | — | B | 1 j |
| **SB-3 Mode bac à sable** | `--sandbox [profil]` (D3), plugin dans `map_ldtk::game` posé par `LdtkLocalGamePlugin` quand le drapeau est là, `restart` rejoue le profil, remote D5, `make sandbox GAME= PROFIL=`, retrait de `character_tester` (D6) | SB-1, SB-2 | A | 1,5 j |
| **SB-4 Jeu externe et docs** | test de bout en bout : `alacod new demo` dans un dossier temporaire, compilation, `--sandbox` en headless N frames (`ALACOD_HEADLESS=1 ALACOD_EXIT_AT_FRAME`), joueur posé avec le profil ; `docs/editeur-vocal.md` (§1 apps, §2.1 phrases « donne-moi », « charge le profil »), `docs/jouer.md`, `docs/conventions.md` (§ nouveau), `CLAUDE.md`, gabarit `templates/starter` (README/CLAUDE.md du jeu créé) | SB-3 | B | 1 j |
| **SB-5 Essai William** | séance K5-4 refaite dans le bac à sable sur son Mac (protocole `docs/taches/rapports/k5-4-essai-vocal.md` adapté) | SB-3 (SB-4 pour la doc) | William | 45 min |

Total ≈ 4,5 j-agent `[INFERENCE]`. Chemin critique : SB-2 → SB-3 → SB-4.

## 5. Règles communes

- Aucune trace existante ne bouge (`BLESS` interdit) ; `make schedule_order_diff BASE=origin/main` vide pour SB-2 et SB-3 ; rien dans
  `GgrsSchedule` sauf nécessité prouvée (et dite) ; garde `hot_edit_allowed` sur toute méthode qui écrit.
- Générique : **aucun nom de jeu dans le code** (ni `zombies`, ni `player` codé en dur : `examples/character_tester.rs` lisait
  `health_max_by_character["player"]`) ; tout passe par le manifeste et les profils.
- Vérification par lot : `make test_crates` (un paquet à la fois sur le VPS : disque partagé), `make test_scenarios_par SHARDS=3` sans BLESS,
  `make lint` ×5, `make gen` ×5, fmt, `check-forbidden`, `check_sim_boundary`, `make test_tools`. Rendu impossible sur le VPS (D94) : commandes
  d'essai données à William.
- Revue indépendante avant chaque merge ; message de merge = quoi + preuves + verdict. Dettes à partir de **D126**.
