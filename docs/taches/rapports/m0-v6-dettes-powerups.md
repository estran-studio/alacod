Tête de branche vérifiée (code) : `b7f0c9c1f33a824337f142f1c489133abd8915e3`
Fiche : [`m0-v6-dettes-powerups`](../m0-v6-dettes-powerups.md) (dettes D6, D17, D18)
Branche : `m0-v6-dettes-powerups` — agent : Claude (Claude Code, session cloud) — 2026-10-02.

Ce SHA est la tête au moment d'écrire le rapport ; le commit suivant ne contient que ce
rapport. Base : `origin/main` `f170a92`, puis `origin/main` `b0e3f99` intégré par merge
(`b7f0c9c`, sans conflit ; aucune trace modifiée sur `main` entre les deux).

## À retenir pour l'orchestrateur

**D17 change quatre traces, pas une seule.** La fiche dit « seule la trace `powerup_nuke`
change » ; c'est faux : `clone_solo`, `clone_duo` et `clone_quad` placent un Nuke (f650,
f950, f750), qui crédite maintenant 400 points à chaque joueur. Leurs attentes `Currency`
exactes échouaient toutes de +400 exactement. Je les ai relevées de 400 dans un commit
séparé (`94229eb`), que l'orchestrateur peut écarter s'il préfère une autre décision.
**Aucune trace n'est blessée** sur la branche. Les quatre traces sont à blesser par
l'orchestrateur, avec la preuve ci-dessous, que j'ai déjà faite dans le clone.

## Fait

- **D17** (`7450209`) : `KillAllWaveEnemies` (`powerups::apply_powerup_actions_system`)
  pousse un `economy::PointsCredit::Nuke { handle }` par joueur vivant (ordre `GgrsNetId`,
  joueurs portant `Death` exclus ; un joueur à terre est crédité), une fois par ramassage et
  pas par ennemi tué. `award_points_system` le résout en fin de frame comme les autres
  crédits : montant `EconomyConfig::nuke_points` (nouveau champ, défaut 400, raison
  `"nuke"`), multiplié par le multiplicateur de points du joueur (800 sous Double Points,
  comme CoD). Les ennemis tués par le nuke ne rapportent toujours pas de points de kill
  (`Death { last_hit_by: None }`, inchangé).
  - `nuke_points: 400` explicite dans `economy/economy.ron` des deux jeux ; mirroir du
    registre de contenu (`content::registry`, `EconomyFileSchema`/`EconomyEntry`) aligné,
    même défaut (pas de nouvelle règle de lint : un `u32` n'a pas de plage interdite).
  - `tests/scenarios/powerup_nuke.ron` : `Currency(handle: 0, min: 500, max: 500,
    at_frame: 10)` et `Currency(handle: 0, min: 900, max: 900, at_frame: 30)` (ramassage à
    f20 + marge) ; commentaire « À regarder » complété.
  - Test de système `nuke_credite_chaque_joueur_vivant_une_fois` (deux vivants dans l'ordre
    inverse des handles, un mort exclu, trois ennemis → deux crédits seulement).
  - `docs/conventions.md` §12 (champ) et §14 (« Nuke : points »).
- **D17, suite** (`94229eb`) : attentes `Currency` postérieures au Nuke relevées de 400
  dans `clone_solo` (4), `clone_duo` (5), `clone_quad` (4) ; commentaires « À regarder »
  complétés. Aucune autre attente touchée.
- **D6** (`9171bb2`) : deux tests dans `crates/scenario/tests/expectations.rs`, sur
  `testbed_target_hits` (seul `target` porte `HitCount` dans l'arène) :
  `entity_hits_compte_les_coups_sur_target` (compteur relu dans le monde à f200, ≥ 5 ;
  `EntityHits` passe avec 0..0 à f5 et compte..compte à f200) et
  `entity_hits_hors_bornes_ou_sans_compteur_echoue` (sous le minimum, au-dessus du maximum,
  sur le joueur sans `HitCount`, sur un net_id absent : quatre failures, chacune avec sa
  raison). Aucun scénario ni trace.
- **D18** (`903f10a`) : au ramassage, `apply_powerup_actions_system` retire d'abord sur
  chaque joueur les modificateurs de source `powerup:<id>` (`Modifiers::remove_by_source`),
  une fois pour toutes les actions du power-up (un power-up à plusieurs actions
  modificatrices les garde toutes), puis les repose : la durée recommence, l'effet ne se
  multiplie plus. Test `meme_power_up_rafraichit_sans_cumuler` (ramassages à f10 et f40 → un
  seul ×2 jusqu'à f100, le perk intact) ; il échoue sans le correctif (3 modificateurs,
  vérifié en commentant la ligne). `docs/conventions.md` §14 (« Rafraîchissement »).
- `docs/taches/dettes.md` : D6, D17, D18 marquées « fait dans m0-v6 ».

## Vérifié (sur l'état fusionné `b7f0c9c1f33a824337f142f1c489133abd8915e3`, sauf mention)

Clone cloud, Rust nightly, paquets de la CI installés, `--profile headless`, une commande
cargo à la fois. Contrairement à ce que prévoyait la consigne, les scénarios tournent ici
(≈ 11 min pour les 61).

| Commande | Résultat réel |
|---|---|
| `make test_scenarios` | 61 scénarios ; seules `powerup_nuke`, `clone_solo`, `clone_duo` et `clone_quad` échouent, uniquement sur la trace (lignes 21, 651, 951, 751), aucune attente en échec. Test binaire : 1 réussi / 1 échoué (`scenarios`) / 7 ignorés, 611,7 s. Les 57 autres traces identiques. |
| `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots -p effects --no-fail-fast` | 287 réussis / 1 échec / 8 ignorés. L'échec est le test `scenarios` (les mêmes 4 traces non blessées). `expectations` 32 (30 + 2 D6), `game` lib 22 (+2 : D17, D18), `lint_fixtures` 36, `run` 13. |
| `make lint` | zombies : aucune erreur (4 personnages, 4 armes, 6 mêlées, 1 vagues, 2 cartes) ; testbed : aucune erreur (7, 4, 6, 0, 4). |
| `cargo fmt --all -- --check` | code 0, aucune sortie. |
| `./scripts/check-forbidden.sh` | 4 avertissements (HashSet 3, rand 1), les préexistants ; aucun nouveau. |
| `./scripts/check-rollback-registration.sh` | « OK ». |
| `make gen GAME=zombies` (avant le merge, tête `903f10a`) | 10 armes, attentes ok, traces ok ; `git status` propre ensuite. |
| `cargo test -p game powerups` sans `remove_by_source` (avant le merge) | `meme_power_up_rafraichit_sans_cumuler` échoue : 3 modificateurs au lieu de 2. |

**Preuve `trace-diff` (README §5)**, faite avant le merge : dumps `ALACOD_DUMP_TRACE` de
`main` `f170a92` (worktree `../alacod-main`, même target) et de la branche avec D17 seul
(`2994a45`, même arbre que `7450209`). Le merge n'a changé aucune trace de `main`.
`--ignore "Currency,FrameEvents<run::currency::CurrencyEvent>,FrameEvents<game::economy::PointsCredit>"`,
plus `Run` pour les `clone_*` :

| Scénario | Résultat avec `--ignore` | Sans `--ignore` : première frame qui diffère, et quoi |
|---|---|---|
| `powerup_nuke` | identique (99 frames) | f20 : `Currency(500)` → `Currency(900)` du joueur ; `FrameEvents<PointsCredit>` `[]` → `[Nuke { handle: 0 }]` ; `FrameEvents<CurrencyEvent>` `[]` → `[CurrencyEvent { handle: 0, delta: 400, reason: "nuke" }]`. Ensuite seul `Currency` diffère (900). |
| `clone_solo` | identique (3046 frames, `Run` ignoré en plus) | f650 : les trois mêmes lignes (1 joueur) ; en ignorant aussi `Currency` et les files, il ne reste que `Run.summary.points_total` 4200 → 4600 à la fin (f3016). |
| `clone_duo` | identique (4667 frames, `Run` en plus) | f950 : deux crédits `Nuke` (handles 0 et 1) et leurs `CurrencyEvent` ; `points_total` 4440 → 5240 (f4637). |
| `clone_quad` | identique (4371 frames, `Run` en plus) | f750 : quatre crédits (handles 0 à 3). |

Ce que l'orchestrateur doit voir en reproduisant la preuve : dans chacune des quatre
traces, le hash change à partir de la frame du ramassage du Nuke (lignes 21, 651, 951,
751 du `.trace`), uniquement à cause des points (solde des joueurs, deux files de la frame,
puis `Run.summary.points_total` à la fin des `clone_*`). Rien d'autre ne doit bouger : le
crédit passe par `FrameEvents<PointsCredit>`, lu seulement par `award_points_system`, qui
n'écrit que `Currency` et `FrameEvents<CurrencyEvent>`. Un solde plus haut aurait pu faire
réussir un achat refusé avant : les diffs « identique » ci-dessus montrent qu'aucun achat,
aucune position ni aucun autre état ne diverge dans ces scénarios. D18 et D6 ne
changent aucune trace (passe complète ci-dessus). Les dumps ont été supprimés (1,7 Go par
`clone_*`).

Ma copie de `trace-diff.py` pour lire des dumps `.gz` (place disque) n'est pas commitée :
elle ne change que l'ouverture du fichier (`gzip.open` si `.gz`).

## Non fait / non vérifié ici

- **Aucun bless** : `powerup_nuke`, `clone_solo`, `clone_duo` et `clone_quad.trace` sont à
  réécrire par l'orchestrateur (`BLESS=1 SCENARIO=<nom> make test_scenarios`), après avoir
  refait la preuve sur sa machine.
- **Bench au calme** (`ALACOD_BENCH_STRICT=1`) : non fait. La passe non stricte affiche des
  avertissements de budget sur la machine cloud (ex. `bench_bullets` 57,6 fps < 70 ;
  `bench_horde` 42,0 fps, au-dessus de son plancher de 38), environnement non comparable ; à rejouer machine calme.
- **p2p à deux clients** : non fait (pas de Docker ici). D17/D18 changent la simulation des
  points et des modificateurs : à rejouer.

## Dettes, questions ouvertes

- Décision à confirmer : un joueur **à terre** reçoit les points du nuke (seuls les joueurs
  portant `Death` sont exclus). Je n'ai pas trouvé de règle CoD explicite ; c'est le choix
  le plus simple.
- D18 : deux power-ups *différents* qui touchent la même stat se cumulent toujours (sources
  différentes) ; c'est voulu (CoD n'en a pas), mais à garder en tête pour les contenus
  futurs.
- La fiche et `dettes.md` annonçaient « seule `powerup_nuke` change » : à corriger dans les
  critères d'acceptation (quatre traces).
