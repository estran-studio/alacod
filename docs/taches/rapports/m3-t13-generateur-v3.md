# Rapport M3-T13 : générateur v3 (`m3-t13-generateur-v3`)

## Fait (état final)
- Trois gabarits, **champs ajoutés aux `test:` existants** (conventions §78) : `TearsOnTarget`
  (`tears:` d'un objet → `tears_<id>.ron` + témoin `tears_<id>_sans_objet.ron`), `FamiliarHeld`
  (`familiar:` → `familiar_<id>.ron`, + `familiar_<id>_sans_objet.ron` si `min_hits > 0`),
  `ChampionForced` (`champions:` d'un personnage → `champion_<id>_<variante>.ron`). Les fichiers
  existants (`item_<id>`, `enemy_<id>_*`) sont inchangés : chaque gabarit a sa propre `frames`.
- Schéma `tears:` : `frames`, `weapon`, `aim`, `targets: [(character, dx, dy)]` (cibles à
  `counts_hits` posées relativement à la cible de l'arène, PV réglables), `hits: [(target, with_item,
  without_item)]`, `expect` (avec l'objet) / `control` (son contraire, sans l'objet).
- Champion `kill: true` : une sonde mesure la mort ; attentes `EntityCount(Enemy)` **exactes** à
  f−1 et f+1, puis chaque effet traduit (`Explode` → `explosion`, `SpawnCharacters`, `SpawnPattern` →
  `BulletCount` équipe ennemie, `loot` → marche + Interaction + `HasItem`).
- Fonctions pures testées (`crates/scenario/tests/generate_v3.rs`, 7 tests).
- Lint (`lint_item_tests`, `lint_character_tests`) + **16 fixtures** ; schéma `alacod describe`
  (Item, Character) à jour ; `--list`/`--only` voient les nouveaux sujets (comptes `larme(s)`,
  `familier(s)`, `champion(s)`, `témoin(s)` dans le résumé).
- Contenu testbed avec `test:` : 7 objets de larmes, 6 objets familiers, 5 personnages à champions.
- Aucun changement de simulation : pas de trace préexistante touchée, aucune attente nouvelle.

## Vérifié
- `make gen` ×5 sur les cinq jeux : exit 0 à chaque tour, `git status tests` vide.
- Fichiers générés ajoutés (tous au testbed ; zombies, throne, gungeon, isaac : 0) : 36 scénarios
  avec trace : 7 `tears_*` + 7 `tears_*_sans_objet`, 6 `familiar_*` + 1 `familiar_compagnon_sans_objet`,
  6 `item_*` (ItemHeld des objets familiers), 9 `champion_*`. Mesures objet/témoin : tableau de la
  section « Revue » ; `familiar_compagnon` 9 coups contre 4 (témoin), `min_hits: 9`.
- `make test_scenarios_par SHARDS=3` (sans BLESS) : 3 tranches ok, 464 scénarios, 189 s (après la revue).
- `make test_crates` ok ; `cargo test -p content` (96 + 159 fixtures) ok ; `make test_tools` ok ;
  `make check_sim_boundary` ok ; `check-forbidden`, `check-rollback-registration` ok ;
  `cargo fmt --all -- --check` ok ; `alacod lint` ×5 sans erreur.
- **Échelle** (machine partagée, profil `headless`, build à chaud) : 100 objets copiés de `guide`
  (`echelle000`…, `test:` avec `tears:`, supprimés ensuite, rien commité) = 200 scénarios
  (`item_*` + `tears_*`) : génération seule (tout le testbed + les 100) 65 s contre 18 s sans eux, soit
  ~0,47 s par objet (deux sondes par objet) ; `make gen`-équivalent avec lecture, jeu et
  écriture des traces des 200 : 2 min 20 s au total. Zéro `.ron` écrit à la main pour ces objets.
  Seuil « N s » absent de la fiche : à fixer par l'orchestrateur (100 objets < 2 min 30 mesuré).

## Non fait
- `tears` pour `orbite` (le projectile orbite ne touche pas la cible lointaine : `Orbit` se
  prouve déjà par `tears_item_orbit`) et `trefle` (chance, non déterministe à compter).
- Aucun `test:` ajouté à isaac : il n'a ni objet de larmes ni familier (T11a).

## Revue (historique)

### Premier passage : verdict bloquant, corrigé
1. **`tears:` ne prouvait pas l'effet de l'objet** : `min_hits: 1` passait sans l'objet. Corrigé : le
   gabarit écrit `tears_<id>` et le témoin `tears_<id>_sans_objet` (même tir, même placement, sans
   l'objet) ; `hits: [(target, with_item, without_item)]` + `expect`/`control` ; le lint refuse un seuil
   que le témoin passerait (`with_item <= without_item`). Même principe pour `familiar_<id>_sans_objet`.
2. **Effets de mort ignorés en silence** : corrigé. Une sonde (`probe_champion_death`) relève la mort ;
   `kill: true` attend la mort (`EntityCount(Enemy)` exact avant/après) puis `Explode`, `SpawnCharacters`,
   `SpawnPattern` (`BulletCount` équipe ennemie) et `loot` (marche, Interaction, `HasItem`). Sans effet
   traduisible : lint (`kill: true`) ; effet sans trace à la sonde : la génération échoue.
3. **Fixtures** : 16 cas (champion : variante inconnue/doublon/arme inconnue/kill non traduisible/arme sans
   kill ; tears : with_item 0, seuil ≤ témoin, rien à prouver, expect sans control, frames 0, cible hors
   index, arme inconnue, cible inconnue, cible sans `counts_hits` ; familiar : sans `familiar:`, frames 0).

### Mesures témoin / objet (coups sur la cible mesurés par `alacod-gen --play`, 150 frames sauf mention)
| objet | preuve avec l'objet | témoin sans objet | seuil posé |
|---|---|---|---|
| guide | cible hors ligne : 6 coups | 0 | `with_item 6, without_item 0` |
| perforant | cible de l'arène derrière une cible proche : 5 coups ; 2 balles en vol | 0 ; 0 balle | `with 5 / without 0` + `BulletCount 2 / 0` |
| rebond | 1 balle en vol à la frame 150 (6 coups, comme le témoin) | 0 balle | `BulletCount 1 / 0` |
| spectre | 1 balle en vol (6 coups, comme le témoin) | 0 balle | `BulletCount 1 / 0` |
| division | éclats sur la cible de l'arène : 5 coups ; 6 balles en vol | 0 ; 0 balle | `with 5 / without 0` + `BulletCount 6 / 0` |
| venin | 9 coups sur la cible posée, statut `poison` | 6 coups, pas de statut | `with 9 / without 6` + `HasStatus` / `present: false` |
| lentille | 40 frames : 1 coup (ShotSpeed) | 0 | `with 1 / without 0` |
| familiar_compagnon | 9 coups (120 frames) | 4 coups, aucun familier | `min_hits 9` (témoin ≤ 8) |

Mort des champions (testbed, mort frame 99) : `explosif` 6 → 5 ennemis + `explosion` ; `salve` 6 → 5 + 8
balles ennemies (0 avant) ; `divise` 6 → 8 ; `riche` 6 → 5 + `HasItem boussole` ramassé.

Traces blessées (testbed seulement, nouveaux ou modifiés par ce gabarit, jamais une trace préexistante) :
nouveaux `tears_{division,guide,lentille,perforant,rebond,spectre,venin}_sans_objet` et
`familiar_compagnon_sans_objet` (8 `.ron` + 8 `.trace`) ; modifiés (nés au premier jet de cette tâche,
changés par la revue) `tears_{division,guide,lentille,perforant,rebond,spectre,venin}` et
`champion_elite_{divise_divise,explosif_explosif,explosif_salve,riche_riche,riche_nu}` ;
`familiar_compagnon.ron` change (`min_hits`), sa trace est identique. Aucune trace préexistante
(avant M3-T13) n'a bougé : `make gen` ×5 puis `git status tests` vide.

## Questions
- Seuil N de l'échelle (voir ci-dessus).

## Dettes
- Le lint vérifie les seuils déclarés, pas leur adéquation aux mesures : seul `make gen` (qui joue) le prouve.
- `generate_game` rejoue toute la sonde de tous les sujets même avec `--only` : le temps de
  `--only` n'est pas proportionnel au sujet (≈ 18 s pour le testbed).
- `EntityCount` d'un champion qui se divise est une borne basse (les ennemis de l'arène comptent).

## Atteignabilité
Tout est atteignable par `make gen GAME=testbed` ; `alacod-gen games/testbed --only tears_guide --play`.
Revue indépendante due avant merge (demandée à l'orchestrateur).
