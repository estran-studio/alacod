# T2.8 — lint des nouveaux kinds, livraison

SHA de la tête de branche vérifiée (code, contenu, traces) :
`0ed87dec52d5a54ed0df9bfd5f326af573c6d9ea`.
Fiche : `docs/taches/T2.8-lint-nouveaux-kinds.md`.
Base : main `213947318edc6148bb079d97f73f7d4d2354db13`.
Date : 2026-10-02. Agent : Claude (Claude Code, session cloud).
Environnement : variante cloud (README §1), clone `estran-studio/alacod` seul, 4 cœurs,
compilation à froid. La branche de travail créée par l'environnement
(`claude/intelligent-cerf-3n9cv7`) est poussée sous le nom `m0-v2-lint-kinds`.
Le SHA livré inclut ensuite le commit de ce rapport, purement documentaire.

**Vérification standard verte, hors p2p et bench (non faisables ici).** Aucun merge dans
main, aucune modification de `docs/taches.md`, aucune autre fiche commencée.

## 1. Fait

Deux commits de travail :

1. `271353f` contenu : sons d'arme vers des fichiers existants (dette D2).
   - `weapons.ron` des deux jeux : `reloading: "sounds/machine-gun-reloading.ogg"` devient
     `sounds/machine-gun-reload.ogg` (fichier réel).
   - **Décision** : le testbed n'avait **aucun** dossier `sounds/`, alors que ses armes et
     son `ui/feedback.ron` y font référence. Le simple renommage ne suffisait pas à rendre
     `make lint` vert. J'ai copié `machine-gun.ogg` et `machine-gun-reload.ogg` depuis
     `games/zombies/assets/sounds/` (environ 270 Ko). Alternative écartée : retirer
     `audio_config` des armes du testbed. À trancher par l'orchestrateur si la copie
     binaire gêne.
   - `audio_config` n'est lu par aucun type de `crates/game` : aucun effet sur la
     simulation.
2. `0ed87de` T2.8 : les règles, fixtures, tests et la doc.

Règles (une fixture sous `crates/content/tests/fixtures/` et un test dans
`tests/lint_fixtures.rs` chacune) :

| Fixture | Règle | Kind |
|---|---|---|
| `entry_mode_waves_without_waves` | règle T2.4 existante, fixture ajoutée | `BrokenReference` |
| `ammo_type_empty_custom` | `Custom("")` ou blanc, dans `lint_weapons` | `OutOfRange` |
| `ammo_type_unknown` | variante inconnue, parse | `Parse` |
| `friendly_fire_unknown` | valeur inconnue, parse | `Parse` |
| `perk_no_modifiers` | `modifiers: []` | `OutOfRange` |
| `perk_mul_non_positive` | `op: Mul` et `value <= 0` | `OutOfRange` |
| `perk_unknown_stat` | stat inconnue, parse | `Parse` |
| `perk_duplicate_id` | clé répétée dans `perks.ron` | `DuplicateId` |
| `powerup_frames_zero` | `frames: 0` pour `TimedModifier` et `CurrencyMultiplier` | `OutOfRange` |
| `powerup_factor_non_positive` | `factor <= 0` | `OutOfRange` |
| `powerup_weight_zero` | scindée de `powerup_out_of_range` | `OutOfRange` |
| `powerup_drop_chance_out_of_range` | scindée de `powerup_out_of_range` | `OutOfRange` |
| `powerup_no_actions` | `actions: []` | `OutOfRange` |
| `powerup_lifetime_zero` | `lifetime_frames: 0` | `OutOfRange` |
| `powerup_pickup_range_non_positive` | `pickup_range <= 0` | `OutOfRange` |
| `economy_ratio_out_of_range` | `refill_price_ratio` hors [0, 1] | `OutOfRange` |
| `audio_missing_file` | son de `audio_config` absent de `assets/` | `BrokenReference` |

Décisions et écarts à la fiche :

- **`friendly_fire` n'était pas mirroité** dans les schémas du lint : une valeur inconnue
  passait sans erreur. Ajouté à `WeaponConfigSchema` et `MeleeWeaponConfigSchema`
  (`#[serde(default)]` = `Never`, comme le type réel). `EnemyAiConfig` n'est pas en RON.
- **Messages de parse lisibles** : `ammo_type`, `friendly_fire` et `stat` (perks) passent par
  une aide `with_field_name` qui préfixe l'erreur RON du nom du champ. Exemple réel :
  `weapons.ron: erreur RON : 7:24-7:29: champ ammo_type : Unexpected variant named `Laser`
  in enum `AmmoType`, expected one of ...`.
- **`stat: Luck` de la fiche est valide** (`StatId::Luck` existe) : la fixture utilise
  `Bonheur`.
- **Doublon de perk** : RON (comme serde pour une `BTreeMap`) garde en silence la dernière
  valeur d'une clé répétée. `perks.ron` est désormais lu en liste (`KeyedEntries`, ordre du
  fichier), et le contrôle de doublon existant de `load_perks` le rapporte.
- **`pickup_range`/`lifetime_frames` obligatoires** dans le mirroir des power-ups, comme
  dans `game::powerups::PowerUpDef`. Les fixtures T2.5 `powerup_duplicate_id` et
  `powerup_unknown_stat` ont été complétées (ces champs, et une action non vide) pour ne
  porter qu'un problème.
- **`powerup_lifetime_zero`** couvrait deux règles dans la fiche : scindée en
  `powerup_lifetime_zero` et `powerup_pickup_range_non_positive`.
- **Un seul problème par fixture** : vérifié par un test,
  `t2_8_fixtures_have_a_single_problem`. Il exige que chaque fixture T2.8 ne produise que
  des erreurs du kind attendu, en plus de `entry.start_map` (`"unused"`, commun à toutes
  les fixtures, existantes comprises).
- Le lint touche le disque pour une seule chose : l'existence des sons (doc de `lint::run`
  mise à jour). `audio_config` et ses champs ne sont pas des `Option` dans le mirroir, car
  RON exigerait `Some(...)` ; un champ absent vaut « pas de son ».
- Aucun nouveau `LintErrorKind`.
- Doc : tableau complet des règles de lint dans `docs/conventions.md` §3 ; §12 (économie,
  perks) et §14 (power-ups) mis à jour.

## 2. Vérifié (tout a tourné dans ce clone, sur `0ed87de`)

| Commande | Résultat |
|---|---|
| `cargo test -q --profile headless -p content --test lint_fixtures` avec `lint.rs`/`registry.rs` de main | 15 échecs, 12 réussis : toutes les nouvelles règles rouges ; les 3 nouvelles fixtures vertes portent des règles existantes |
| `cargo test -q --profile headless -p content` | 63 unitaires et 27 `lint_fixtures` réussis, 0 échec |
| `make test_scenarios` | 58 scénarios joués (48 + 10 générés), `2 passed; 0 failed; 7 ignored` ; aucune trace modifiée (`git status tests/` vide) |
| tests des dix crates (§4) | 257 réussis, 0 échec, 8 ignorés (240 de la référence + 17 nouveaux `lint_fixtures`) |
| `make lint` | zombies et testbed : « aucune erreur » |
| `cargo fmt --all -- --check` | rien à afficher |
| `./scripts/check-forbidden.sh` | 4 avertissements, les mêmes que la référence (3 `HashSet`, 1 `rand::`) |
| `./scripts/check-rollback-registration.sh` | OK |
| `make gen GAME=zombies` | 10 armes `ok ok`, aucun fichier modifié sous `tests/` |

Lancé par précaution : `make gen`, parce que `weapons.ron` a changé, mais seulement dans
`audio_config`.

## 3. Non fait, non vérifié

- **p2p à deux clients** : non vérifié ici (Docker et signaling indisponibles dans le
  cloud). Ce n'est pas requis par la fiche, car le lint ne touche ni la simulation ni la
  session. À rejouer par l'orchestrateur s'il le souhaite.
- **Bench strict** (`ALACOD_BENCH_STRICT=1`) : non vérifié ici. En mode normal,
  `make test_scenarios` affiche 30 alertes « budget » (ex. `idle` 40.5 fps < 80, `bench_horde`
  25.2 fps). C'est attendu sur cette machine cloud à 4 cœurs, plus lente que celle de
  William : aucune conclusion de performance à en tirer.
- Le jeu n'a pas été lancé avec rendu ; les sons du testbed n'ont pas été écoutés.

## 4. Dettes et questions ouvertes

- Clés répétées dans une même table RON : seul `perks.ron` les détecte. `weapons.ron`,
  `melee_weapons.ron` et `powerups.ron` gardent encore la dernière en silence. Le correctif
  tient en une ligne par schéma (`KeyedEntries`), avec une fixture chacun.
- `op: Mul` avec `value <= 0` dans une action `TimedModifier` de power-up n'est pas
  vérifié : la fiche ne le demandait que pour les perks. C'est la même règle, à étendre
  si voulu.
- `ui/feedback.ron` (sons `shot`/`reload`) n'est pas linté : kind `Ui` générique. Le
  testbed y référençait des sons absents, corrigés de fait par la copie.
- Copie binaire des deux sons dans le testbed : à confirmer (voir §1).
