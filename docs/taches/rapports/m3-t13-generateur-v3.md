# Rapport M3-T13 : générateur v3 (`m3-t13-generateur-v3`)

## Fait
- Trois gabarits, **champs ajoutés aux `test:` existants** (conventions §78) : `TearsOnTarget`
  (`tears:` d'un objet → `tears_<id>.ron`), `FamiliarHeld` (`familiar:` → `familiar_<id>.ron`),
  `ChampionForced` (`champions:` d'un personnage → `champion_<id>_<variante>.ron`). Les fichiers
  existants (`item_<id>`, `enemy_<id>_*`) sont inchangés : le gabarit a sa propre `frames`.
- Fonctions pures testées (`crates/scenario/tests/generate_v3.rs`, 6 tests) : noms de fichiers,
  attentes de chaque gabarit, cible de remplacement sur la ligne de tir, effets de mort.
- Cible à PV réglable : `tears.target` (personnage à `counts_hits` posé devant la cible de l'arène).
- Lint (`lint_item_tests`, `lint_character_tests`) + 4 fixtures ; schéma `alacod describe`
  (Item, Character) à jour (miroirs `ItemTestSchema`, `CharacterTestSchema`) ; `--list`/`--only`
  voient les nouveaux sujets (comptes `larme(s)`, `familier(s)`, `champion(s)` dans le résumé).
- Contenu testbed avec `test:` : 7 objets de larmes, 6 objets familiers, 5 personnages à champions.
- Aucun changement de simulation : pas de trace existante touchée, aucune attente nouvelle.

## Vérifié
- `make gen` ×5 sur les cinq jeux : exit 0 à chaque tour, `git status tests` vide (aucun fichier
  ni trace existant modifié).
- Fichiers générés ajoutés (tous au testbed ; zombies, throne, gungeon, isaac : 0) : 28 scénarios
  avec trace : 7 `tears_*`, 6 `familiar_*`, 6 `item_*` (ItemHeld des objets familiers), 9 `champion_*`.
  `familiar_compagnon` : 9 coups sur la cible contre 5 pour le joueur seul (`tears_guide`) ;
  `min_hits: 7`. Variantes `explosif` (moment clé `explosion`) et `divise` (`EntityCount`) tuées par le
  joueur et attentes vertes.
- `make test_scenarios_par SHARDS=3` (sans BLESS) : 3 tranches ok, 456 scénarios, 189 s.
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

## Questions
- Seuil N de l'échelle (voir ci-dessus).

## Dettes
- `generate_game` rejoue toute la sonde de tous les sujets même avec `--only` : le temps de
  `--only` n'est pas proportionnel au sujet (≈ 18 s pour le testbed).
- `EntityCount` d'un champion qui se divise est une borne basse (les ennemis de l'arène comptent).

## Atteignabilité
Tout est atteignable par `make gen GAME=testbed` ; `alacod-gen games/testbed --only tears_guide --play`.
Revue indépendante due avant merge (demandée à l'orchestrateur).
