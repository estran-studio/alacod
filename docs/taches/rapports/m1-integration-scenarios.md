# Rapport m1-integration-scenarios — vague 2 de M1 : scénarios du clone à 1, 2 et 4, boss simple, vidéos

**Branche** `m1-integration-scenarios`, partie de la tête livrée de m1-v4-feedback-v1 `fcd0c91` ;
`origin/main` `963f2d2` (T1.16 + T1.17) mergé avant la compilation. Fiche :
`docs/taches/m1-integration-scenarios.md` (main local d'orch).

## État en cours

- **Fait** : décisions confirmées (dix lignes, amendements acceptés) ; boss `roi_rat` ;
  **correctif d'engine** « une caverne = un asset » (décision (a) d'orch, commit à part `38c9a29`)
  et test de cohérence murs ⇔ terrain ; preuve du correctif seul (§3).
- **En cours** : jouabilité des étages 2 et 3 enfin chargés (calibrage), scénarios, 20 graines,
  vidéos, digest.

## 1. Correction d'un résultat antérieur

**Les trois étages de la run `throne` chargeaient tous `niveau_1`**, depuis T1.11 (phases 1 et
2) jusqu'à cette tâche. Les « **20/20 graines `--until-floor 3`** » du journal pour
m1-v2-contenu-throne (p1 19/20, p2 17/20) et m1-v3-bots-pathfinding (20/20) **portaient sur trois fois
`niveau_1`** (48 × 32, quatre ennemis de mêlée et rôdeur) : le contenu de `niveau_2` (tireurs,
chargeur, 56 × 40) et de `niveau_3` (tous les profils, 64 × 44) n'avait jamais été joué, et
`throne_three_floors`/`throne_progression` jouaient trois fois la même caverne.

Cause : `map_ldtk::loader::resolve_map_config` donnait à toute caverne le chemin d'asset de
son gabarit (`caves/gabarit.ldtk`, un seul par dossier) ; en mode `Floors`,
`setup_generated_map` charge un monde par étage avec ce même chemin, et l'`AssetServer`, qui
dédoublonne par chemin, rendait aux trois le premier chargement (`niveau_1`, avec ses
réglages). De plus l'iid du niveau (`cave-{seed}`) était le même pour les trois cavernes de
même graine.

**Second bug caché** (mesuré) : au passage d'étage, `CellGrid` (terrain : destruction,
`CellState`) était régénérée depuis la **vraie** config (`niveau_2`, `niveau_3`) alors que les
murs venaient du LDtk de `niveau_1`. Test `crates/scenario/tests/cave_floors.rs` (un mur LDtk ⇔
une cellule solide, dimensions de la grille par étage), avant le correctif :

| Étage | Grille | Murs hors grille | Cases solides sans mur |
|---|---|---:|---:|
| 0 (`niveau_1`) | 48 × 32 | 0 | 0 |
| 1 (`niveau_2`) | 56 × 40 | 264 | 431 |
| 2 (`niveau_3`) | 64 × 44 | 269 | 479 |

Une destruction de terrain aux étages 2 et 3 reconstruisait donc les murs (`rebuild_cave_walls`)
depuis une autre carte que celle affichée. Après le correctif : 0 / 0 aux trois étages, aux
bonnes dimensions.

## 2. Correctif d'engine « une caverne = un asset » (`38c9a29`)

- `game::cave_assets` : source d'assets `cave://`, enregistrée **avant** `AssetPlugin`
  (`CoreSetupPlugin::get_default_plugin`, `add_before::<AssetPlugin>`), même racine
  (`AssetPlugin.file_path`). Une caverne se charge depuis `cave://<dossier>/<id>.ldtk` ; le
  lecteur sert tout `.ldtk` de cette source avec `<dossier>/gabarit.ldtk` (lecteur par défaut de
  la plateforme, fichiers ou HTTP en wasm) et tout autre fichier tel quel (tilesets). Pas de
  copie de gabarit.
- `resolve_map_config` : `map_path = cave_asset_path(template, id)` ; le callback du loader
  retrouve l'id depuis ce chemin. `cave_level_iid(id, seed)` = `cave-{id}-{seed}` (entités :
  `<iid>-player-<n>`…).
- Tests : `cave_assets::tests` (chemins), `generation::cave` (deux cavernes de même graine =
  deux iid), `cave_floors` (cohérence par étage), et les tests existants `cave`, `floors`.

## 3. Preuve §10 du correctif seul

Suite `scenarios` avec le correctif et le `niveau_3` de `main` (sans boss) : **seules les traces
`throne` multi-étages changent**, et toutes à la **première entrée dans le deuxième étage**,
désormais `niveau_2` (107 entités au lieu de 85) :

| Scénario | Première différence | Cause |
|---|---|---|
| `throne_floor_1` | ligne 808 (f807) | entrée dans l'étage 1 : `niveau_2` au lieu de `niveau_1` |
| `throne_mutation_choice` | ligne 925 (f924) | idem |
| `throne_progression` | ligne 808 (f807) | idem ; attentes de niveau 2 et d'étage 2 à recaler |
| `throne_three_floors` | ligne 517 (f516) | idem ; attentes d'étages et de mutations à recaler |

Tout le reste est identique, **zombies et testbed compris** (`explode_wall`, `bench_cave`,
`clock_floor_reset`, `portal_next_floor`, `difficulty_scales`, `bot_floors_three`, la séquence
`caverne` du testbed : l'iid `cave-{id}-{seed}` ne change aucune trace).

## 4. Boss `roi_rat`

`characters/roi_rat.ron` : 360 PV (12 rats), tag `champion` posé directement, `[Shoot(weapon:
"arsenal", pattern: "couronne", range: "260.0", cooldown_frames: 120), Charge(telegraph: 30),
Chase]`, `attack_range` 40, dégâts 15, `scale` 1.4, `test: (frames: 600)` (gabarits
`enemy_roi_rat_still`/`_moving` verts aux attentes par défaut).

- **Placement** : **premier** de `niveau_3.characters`, `enemy_spawns` 8 → 9. Mis en dernier
  (première idée, acceptée puis amendée), il n'apparaissait jamais : la graine 123456 ne trouve
  que **8** points espacés (`ZOMBIE_SPAWN_SPACING`), le 9ᵉ n'existe pas. En tête, il prend le
  point 0 (le plus éloigné du départ, toujours présent) : **exactement un boss sur toute
  graine** (au plus 9 points). Contrepartie : les autres ennemis se décalent d'un point, et le
  dernier de la liste (`rat`) manque quand la caverne n'a que 8 points.
- **Corps de 20 × 20** (et non 28 × 28) : avec 28 px, le boss sélectionnait bien `Chase` avec
  une cible (dump de `throne_quad`, f1380 à f2200) mais **restait figé** sur son point
  d'apparition (920, 40) ; avec 20 px il se déplace aussitôt (dump : (909, 51) à f1400, (653, 338)
  à f2000). La navigation suppose des agents d'au plus 20 px : dette D41 (§7).

## 5. Jouabilité des étages 2 et 3, calibrage

`alacod-sim --game throne --bots N --profiles prudent,… --floors run --seeds 1..20
--until-floor 3 --max-frames 15000` (« fini » = étage 3 atteint ; SL = soft-lock détecté :
1 200 frames sans kill ni étage). **Les anciens 20/20 portaient sur trois fois `niveau_1`.**

| Réglage | 1 bot | 2 bots | 4 bots |
|---|---|---|---|
| avant le correctif (journal, trois fois `niveau_1`) | — | 20/20 | — |
| correctif + boss 20 × 20 + butin relevé (`drop_chance` 0,15 → 0,25, `munitions_balles` 12 → 36, `munitions_lames` 12 → 24) | 0/20 (7 SL, 13 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + réserves de départ ×2 (`mag_limit` des trois armes du pilote) | 0/20 (10 SL, 10 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + tourelle en dernier dans `niveau_3` (**final**) | 0/20 (6 SL, 14 morts) | **3/20** (16 SL, 1 défaite) | **11/20** (9 SL) |

**0 desync** dans toutes les séries (synctest). Résultats finaux complets (par graine, avec les
ennemis restants des soft-locks) : `docs/digests/m1-fin-de-vague-2.sim.json`.

- **Munitions : ce n'était pas la cause.** Avec les réserves doublées, mêmes étages graine par
  graine qu'avant, alors que les bots ont des centaines de balles au moment du blocage (le
  butin relevé et les réserves ×2 sont gardés : réalistes pour trois vraies cavernes).
- **La cause est la limite des bots « cible hors de vue »** : presque toujours 1 ou 2 ennemis
  restants sur les points les plus éloignés du départ — d'abord la tourelle (seul ennemi fixe,
  mise en dernier pour cela), puis des ennemis lents ou à distance (brute, tireurs) que le bot
  `prudent` ne va pas chercher dans une caverne de 64 × 44. Corrigé côté bots par b1
  (m1-v3-bots-portail), qui mesure sur cette branche ; ces chiffres sont sa référence.
- `fill_ratio` : 0,34 contre 0,38 sur `niveau_3` ne change pas la tendance (premières graines :
  blocages au même étage) ; non modifié.
- **Solo** : 0/20 (Nuclear Throne seul est dur par nature, et les deux soft-locks au portail de
  l'étage 0, graines 3 et 16, sont un défaut de bots : dumps dans
  `docs/taches/rapports/m1-integration-scenarios/`).

## 7. Dettes et limites trouvées (à reporter par l'orchestrateur)

| # | Dette | Où | Données en attendant |
|---|---|---|---|
| D41 | La navigation est limitée aux agents d'au plus 20 px (`FlowFieldCache::is_too_narrow` : « zombies 20 px, cellules 16 px ») : un boss de 28 px choisit `Chase` mais reste figé sur son point d'apparition dans une caverne | `crates/game/src/character/enemy/ai/navigation.rs` | `roi_rat` en 20 × 20, `scale` 1.4 pour le visuel |
| D42 | `scenario::softlock` lit le champ de flux du profil `Ground`, or seul `GroundBreaker` est construit (D38) : tout ennemi `Ground` est signalé « sans chemin depuis sa case exacte », faux diagnostic (il a envoyé l'orchestrateur sur la piste D37) | `crates/scenario/src/softlock.rs` | lire le diagnostic `path_cost` avec ce biais |

**Limite des bots** (pas une dette d'engine) : le bot `prudent` ne finit pas une cible fixe hors
de vue — tourelle laissée à 1 PV dans un recoin de `niveau_3` (graine 123456, 2 bots : un bot
mort, l'autre à 11 cases avec son lance-lames, sans tirer) ; le détecteur de soft-lock arrête la
partie après 1 200 frames sans kill ni étage.
