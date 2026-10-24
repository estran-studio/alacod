# Rapport sb-1-contenu (b6) : kind `Sandbox`, profils et arènes

Fiche : `docs/taches/plan-bac-a-sable.md` (§1, §3 D1/D2, §4 SB-1, §5). Branche `sb-1-contenu` depuis `origin/main` `9381b3ac`. VPS 8 cœurs partagé avec b2 (charge 4 à 60), pas d'affichage.
Doc : `docs/conventions.md` §85. Aucune trace ne bouge, aucun `BLESS`.

## Format retenu et où vivent les types (D1)

Un profil est un **sous-ensemble de `Scenario`**, pas un second format. `content` est une dépendance de `game` (l'inverse est impossible) : plutôt que de dupliquer les types de `game::replay`,
**ils ont déménagé vers le bas** et `game` les réexporte (chemins et RON inchangés, aucun appelant à migrer) :

| Type | Avant | Maintenant |
|---|---|---|
| `CharacterPlacement`, `ItemPlacement`, `ChestPlacement`, `ShopSetup`, `ShelfPlacement`, `ModifierSpec` | `game::replay` | `content::sandbox` (`pub use` dans `game::replay`) |
| `DashConfig`, `DashEffect`, `IframesKind` | `game::character::movement` | `sim_core::dash` (`pub use` dans `movement`) ; `JsonSchema` sous la feature `schema` ; `Team` aussi |
| `SandboxPlayer` | — | `content::sandbox` : les champs de `PlayerScript` **sans `inputs` ni `bot`** |

`From<SandboxPlayer> for PlayerScript` (dans `game::replay`) et le test `sandbox_player_est_un_sous_ensemble_de_player_script` (mêmes clés JSON, même RON dans les deux sens, 12 clés = 10 + `inputs` + `bot`) :
ajouter un champ à un seul type casse le test. `deny_unknown_fields` refuse `inputs`, `frames`, `expect`… (erreur `Parse`). SB-3 charge le profil par `Registry::sandbox_profiles` et convertit par `From`.

## Livré

- **Kind `Sandbox`** (`sandbox/<id>.ron`) : `KNOWN_KIND_NAMES`, chargeur (`load_sandbox_profiles`, doublons `DuplicateId`), schéma (`schema_for_kind`, couvert par `every_kind_has_a_schema`), `alacod describe Sandbox`,
  `alacod new-content Sandbox <id>` (copie du premier profil, sans code spécifique : le kind est « par fichier »).
- **`sandbox.arena`** (facultatif) dans `game.ron` (`SandboxEntry`), D2 : `map` du profil > `sandbox.arena` > `generate_template.map` (`content::sandbox::manifest_arena`, `SandboxProfile::arena`).
- **Lint** (`lint_sandbox`) : dossier `Sandbox` ⇒ arène connue **et** profil `arene` ; `sandbox.arena` connue (même sans dossier) ; par profil : `map`, `weapon` (distance ou mêlée), `items`, `mutations`, personnages posés
  (+ `variant`), objets au sol, pool de coffre, objets de présentoir connus ; stat de `modifiers` lue par un système ; `bomb` > 0 ; clés de coffre/présentoir uniques.
- **Profils** : `arene` + `lourd` pour zombies, gungeon, isaac, throne, testbed ; `arene` dans `templates/starter`. Arènes : zombies `sandbox: (arena: "exemples/test_map.ldtk")`, testbed `"testbed/arena.ldtk"` (déclarées) ;
  gungeon, isaac, throne et starter retombent sur leur `generate_template.map`. Joueur 0 en (424, 696) dans toutes ces cartes : un ennemi à (624, 696) est 200 unités à sa droite.
  `arene` : cible (`cible`/`target`, `counts_hits`) posée, solde ou bombes/clés selon le jeu ; `lourd` : arme lourde, objets/mutations, vitesse ×1,2, 2 à 3 ennemis posés.
- **Fixtures** : `sandbox_ok`, `sandbox_bad_references` (10 erreurs), `sandbox_no_arena`, `sandbox_unknown_arena`, `sandbox_no_default_profile`, `sandbox_extra_field`.

## Preuves

- `crates/content/tests/sandbox.rs` (10 tests : fixtures, résolution d'arène, jeux livrés sans erreur de lint, `describe` + schéma, `new-content` dans un dossier temporaire).
- `crates/scenario/tests/sandbox_profiles.rs` (10 tests) : chaque profil livré devient un `Scenario` en mode `Sandbox`, 60 frames, invariants verts, arme active, solde exact, objets (passifs `ItemCount`, consommables
  `Consumable`) et personnages posés présents. A trouvé que `ItemCount` ne compte pas les consommables (test corrigé, pas le contenu).
- Vérifications (VPS, `--profile headless --no-default-features`, un paquet à la fois) :

| Vérification | Résultat |
|---|---|
| `test_crates` | sim_core 46, bevy_fixed 19, utils 4, stats 0, items 52, effects 38, world 41, run 38, meta 18, combat 119, behaviors 17, animation 2, map 26, bots 99, **content 369**, game 287, map_ldtk 15, scenario 153, telemetry 0, gungeon/isaac/testbed/throne/zombies/alacod 0 : **0 échec** |
| `make test_scenarios_par SHARDS=3`, sans BLESS | 809 scénarios (suite complète, tranches 0 et 2 en 2 515 s et 2 715 s ; tranche 1 coupée par le délai d'une commande, rejouée seule : 269 scénarios, 3 496 s) : **tout ok, aucune trace modifiée, arbre propre** |
| `make lint` ×5 | zombies, testbed, throne, gungeon, isaac : aucune erreur |
| `make gen` ×5 | zombies, testbed, throne, gungeon, isaac : rc 0, tous `ok`, aucun fichier modifié |
| `cargo fmt --check` | ok |
| `scripts/check-forbidden.sh` | OK, 51 occurrences auditées |
| `make check_sim_boundary` | OK, 13 crates, 2 en dette D69 |
| `make test_tools` | 38 tests OK |

Rendu impossible ici : non vérifié à l'écran (SB-3/SB-5).

## Limites et dettes

- **D126** (`dettes.md`) : un profil ne choisit pas le personnage du joueur (`PlayerScript` n'a pas ce champ).
- Rien n'**applique** encore un profil (SB-2/SB-3) : la preuve de jeu passe par un `Scenario` construit à partir du profil (`sandbox_profiles.rs`).
- `at_frame` des placements = frame de session ; la pose d'un joueur par `players[]` est celle des `PlayerScript` (remplace, n'ajoute pas : voir la revue de SB-2).
- Un disque saturé (7 Go libres, deux targets de 63 Go) a coupé un run : binaires de test déjà vérifiés supprimés à la main, target-sweep entre deux commandes.
