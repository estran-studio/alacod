# Rapport m1-v1c-variantes — variantes et élites (T1.5)

**Branche** `m1-v1c-variantes`, partie de la tête livrée de T1.4 `111cd99`
(`m1-v1c-behaviors-composables`), même worktree et même target. Fiche :
`docs/taches/m1-v1c-variantes.md` (main `a3a1876`). Base à la livraison : _(merge final)_.

## État en cours

- **Fait** : tout le périmètre de la fiche (§1), scénarios calés et verts.
- **En cours** : vérification complète (suite, crates, lint, gen, exemples).
- **Prochaines étapes** : merge `origin/main`, purge, rapport final, push, LIVRÉ.
- **Chiffres connus** : grunt tiré (NetId 30) = aucune variante (graine 123456), `blinde`
  (graine 777) ; `rapide` EnemyMoveSpeed 90 (×1,5), `blinde` santé 120 (×2).

## 1. Fait

- **Déclaration** (`crates/game/src/character/variant.rs`, `CharacterConfig::variants`) :
  `variants: Some((chance: "0.5", table: { nom: (weight, modifiers, tags, skin) }))`,
  `chance` défaut 1, `weight` défaut 1, ordre `BTreeMap`. Élite = variante ordinaire.
- **Tirage** (`draw_variant`) : `RollbackRng` local, graine
  `fnv1a("variants") ^ run_seed ^ net_id` (tronqués en `u32`), deux tirages (chance, poids) ;
  `RngStreams` jamais touché. Variante imposée (champ LDtk) : aucun tirage.
  - **Identifiant** : plutôt qu'un `peek_next()` sur la factory (envisagé, puis écarté),
    `spawn_enemy` **alloue** le `GgrsNetId` d'un personnage à variantes et le passe à
    `create_character` (nouveau paramètre `net_id: Option<GgrsNetId>`) : même valeur que si
    `create_character` l'allouait (c'est sa seule allocation, rien ne s'intercale), et le tirage
    utilise l'identifiant réel — aucun risque de divergence silencieuse si l'ordre interne de
    `create_character` change. Les personnages sans variantes passent `None` (chemin d'origine).
  - **Graine** : `run_seed` = la graine de carte. Les vagues et les spawners la lisent dans
    `RngStreams::run_seed` (lecture seule) ; les `CharacterSpawn` de carte apparaissent **avant**
    que `RunSeed`/`RngStreams` soient posés (`OnEnter(GameStarting)`) et la lisent dans
    `MapGenerationConfig::seed` (même valeur). Constaté au calage : sans ça, la graine valait la
    valeur provisoire 12345 de `core.rs` pour toutes les cartes.
- **Application** (`spawn_enemy`, seulement si une variante est choisie) : composant neutre
  `Variant(nom)` ; `Modifiers` de la variante (source `Named("variant:<nom>")`, permanents) ;
  `Tags` en union ; skin passé à `create_character`. Base `MaxHealth` = santé F5 posée pour tout
  personnage **à variantes** ; santé initiale = santé résolue par les modificateurs `MaxHealth`.
  Log `ggrs{variant net_id=… name=…}`.
- **Champ LDtk `variant`** (`CharacterSpawnComponent`, `map_const::FIELD_VARIANT_NAME`) : lu au
  spawn **et** transporté par le pipeline de génération de carte (`CharacterSpawnConfig`,
  `generation::from`/`to` : avant, seuls `character`/`team` traversaient la copie des gabarits —
  constaté au calage). Réécrit seulement s'il est rempli (cartes existantes inchangées).
- **Attentes** : `EnemyVariant { entity, variant, at_frame }` ; `Stat` reçoit un champ optionnel
  `entity` (une stat d'ennemi, `EnemyMoveSpeed` ; absent = le joueur `handle`, comme avant) ;
  moment clé `variant_spawn` (`scenario::events`).
- **Lint** (`crates/content`) : `chance` hors `[0, 1]`, `weight = 0`, nom en double
  (`KeyedEntries`), `skin` inconnu, `MoveSpeed` sur un personnage à `ai` (→ `EnemyMoveSpeed`),
  stat inconnue (chargement), `variant` LDtk vers personnage/variante inconnus (lecture JSON
  minimale des `CharacterSpawn` des `.ldtk`, `serde_json` ajouté à `content`). Fixtures
  `variant_weight_zero`, `variant_chance_out_of_range`, `variant_skin_unknown`,
  `variant_move_speed_ai`, `variant_duplicate`, `variant_ldtk_unknown`.
- **Contenu testbed** : `grunt` (variantes `rapide` et `blinde`, chance 0.5), `grunt_plain` (le
  même sans table : référence de `variant_none`, amendement accepté), carte
  `testbed/arena_variantes.ldtk` (une seule arène, 4 `CharacterSpawn`, champ `variant` déclaré
  dans les définitions de l'entité).
- **Scénarios** :
  - `variant_fast` : `EnemyVariant(33) = rapide`, `Stat(EnemyMoveSpeed, entité 33) = 90`, distance
    au joueur à f80 ∈ [50, 65] pour le rapide contre [90, 105] pour `grunt_plain` (même départ).
  - `variant_none` : `EnemyVariant(31) = None`, vitesse 60, santé 60, distance f80 ∈ [90, 105].
  - `variant_elite` : `EnemyVariant(32) = blinde`, santé 120 (base 60), vitesse 60.
  - `variant_draw` (graine 777) : `EnemyVariant(30) = blinde` (valeur tirée mesurée ; aucune
    avec la graine 123456), `Event(variant_spawn, "variante blinde (30)")` avant f10.
  - **Traces** : `variant_fast`, `variant_none` et `variant_elite` jouent la **même partie** (même
    carte, même graine) et vérifient chacun leur sujet : leurs trois traces sont identiques,
    c'est voulu (accepté par l'orchestrateur). Quatre nouvelles traces à bénir.
- `docs/conventions.md` §25 ; `CLAUDE.md` : `EnemyVariant`. `games/zombies` : aucune variante.

## 2. Vérifié (résultats réels)

_(complété après la vérification finale)_

## 3. Non fait / incertain

_(complété)_

## 4. Dettes, questions ouvertes

- Le skin d'une variante n'a pas d'effet visible en headless (aucune couche dans les skins du
  testbed) : vérifié seulement par le lint et le passage à `create_character`.
- Saison/nuit/champion comme sources de tirage, variantes d'armes et points d'élite : hors
  périmètre (M3, B5 v3, D4).
