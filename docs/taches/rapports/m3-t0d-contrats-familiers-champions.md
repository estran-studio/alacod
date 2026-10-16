# M3-T0d — Contrats des familiers, orbitaux et champions v2

Branche `m3-t0d-contrats-familiers-champions` (depuis `origin/main` 3bab4764). Conventions :
`docs/conventions.md` §63. Contrats seuls : M3-T6 exécute les familiers, M3-T7 les champions.

## Ce qui est livré

- **Behaviors de familier** (`behaviors::Behavior`, en fin d'enum) : `Follow { gap }`,
  `Orbit { radius, frames_per_turn }`, `MirrorShot { weapon, every }`, `BlockBullets` ; `name`,
  `is_familiar`, kinds déclarés, `select` les ignore (un familier n'a pas d'IA de poursuite). Miroirs de
  schéma et de lint dans `content` (`BehaviorEntry`).
- **`FamiliarOf { owner: GgrsNetId, rank }`** : composant rollback **neutre**, enregistré par
  `BehaviorsPlugin`.
- **`ItemDef.familiar: Option<FamiliarRef { character, per_copy }>`** : hors hash (définition),
  `per_copy` = 1 par défaut ; lint (personnage connu, avec un behavior de familier, `per_copy > 0`) et
  règle réciproque (un behavior de familier exige un objet porteur).
- **Champions v2** : `VariantDef.{on_death, behaviors, loot, tint}` optionnels (le composant
  `Variant(nom)` ne change pas), `chance` en nombre **ou expression** (`players`, `floor`), nouvelle
  action `SpawnCharacters { id, count }` (en fin d'enum), lint complet des champs.
- **Attente** `FamiliarCount { handle, count, at_frame }` ; **testbed** : personnage et objet `compagnon`
  (`Follow` + `MirrorShot` de `pistol`), acceptés par le lint ; neuf fixtures de lint.

## Décisions

- **`SpawnCharacters` plutôt que `SpawnEnemies`** pour la division : `SpawnEnemies` tire ses positions
  dans le flux RNG de la salle (`room:<niveau>`), la mort d'un champion ne doit pas déplacer ce tirage ;
  les positions de `SpawnCharacters` seront des décalages fixes autour du point.
- **Q13** (proposition par défaut) : `MirrorShot` tire l'arme du familier, sans les modificateurs du
  porteur.
- **Actions admises à la mort d'un champion** : `Explode`, `SpawnCharacters`, `SpawnPattern` ; le reste
  est refusé (`Unsupported`). `SpawnCharacters` est refusée dans un effet, un power-up et une frise de
  boss.
- **Chance en expression** : le tirage (`draw_variant`) résout l'expression avec 1 joueur à l'étage 0 tant
  que M3-T7 n'a pas branché les vraies valeurs (nombre de joueurs, index d'étage) ; un nombre est exact,
  donc le tirage de §25 est inchangé. Le lint vérifie [0, 1] pour 1 et 4 joueurs aux étages 0 et 5.
- **Un personnage = un identifiant `asset_name_ref`** (pas le nom du fichier) : `compagnon` a le sien.

## Preuves

Voir la section « Vérification » ci-dessous (suite des scénarios, crates, lint, gen, frontière sim).
Aucune trace existante n'est censée bouger : variantes en fin d'enum, champs optionnels absents des
contenus existants, `FamiliarOf` neutre sans porteur.

## Non fait (hors périmètre)

Création et comportement des familiers (M3-T6), champions en jeu (M3-T7), visuels (M3-T17), alliés qui
combattent (D5).

## Note de reprise (session fermée avant la fin)

**État** : code et contenu écrits ; **non vérifié par la suite complète**. Vérifié seulement :
`cargo test -p behaviors -p effects -p items -p content` (verts, dont les neuf fixtures,
`every_kind_has_a_schema`, `shipped_games_lint_without_error` avec le `compagnon` du testbed),
`cargo check` de `game`, `map_ldtk`, `scenario` (tests compris), `cargo fmt --check`.

**Reste** :
1. `make test_crates` (jamais terminé : la chaîne a été interrompue), puis `make test_scenarios_par
   SHARDS=3` : aucune trace existante ne doit bouger (variantes en fin d'enum, champs optionnels,
   `FamiliarOf` neutre ; si une trace bouge, preuve §5 avant tout bless).
2. `make lint` ×4 (+ `isaac`), `make gen` ×4 (+ `isaac`), `make check_sim_boundary`, fuzz
   `ALACOD_FUZZ=0:8`, p2p si la session ou l'état rollback change (`FamiliarOf` est neutre : le p2p de
   l'orchestrateur suffit).
3. Fusionner `origin/main` (T13 y est, 85b8b2dd) avant `LIVRÉ m3-t0d-contrats-familiers-champions <sha>` ;
   remplacer la section « Preuves » ci-dessus par les résultats réels.

**Points à relire** : `draw_variant` résout `chance` avec 1 joueur à l'étage 0 (voir « Décisions ») ;
`lint_variants` prend maintenant `registry` ; le personnage `compagnon` du testbed a son propre
`asset_name_ref` (l'id d'un personnage), il n'a pas de sprite dédié (réutilise les couches vides de
`skins`, comme `ally`).
