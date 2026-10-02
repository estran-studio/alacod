# Lot de dettes — power-ups et coups (D6, D17, D18)

Lire d'abord `docs/taches/README.md` (pour cet agent : §1 « Variante cloud »). Indépendant de
T3.2 (vidéos/digest) et de T1.0a (contrats de combat/IA) : aucun fichier commun.
Branche : `m0-v6-dettes-powerups` (créée depuis `origin/main`).
Ce lot regroupe trois dettes d'un même thème : les power-ups créditent et se rafraîchissent
comme dans CoD, et l'attente `EntityHits` a son test unitaire. Chacune est au plus une
demi-journée ; toutes partagent la même branche et le même rapport.

## Dettes (`docs/taches/dettes.md`)

| Dette | Quoi | Fichiers | Correctif attendu |
|---|---|---|---|
| D17 | Le nuke (`KillAllWaveEnemies`) ne crédite aucun point (CoD : 400 points à chaque joueur) | `crates/game/src/powerups.rs` (`apply_powerup_actions_system`), `economy.rs` | un `PointsCredit` par joueur au ramassage, montant dans `economy.ron` ; attente `Currency` dans `powerup_nuke` ; **seule la trace `powerup_nuke` change** (preuve) |
| D18 | Deux power-ups identiques se cumulent (Double Points × Double Points = ×4) : pas de politique de rafraîchissement | `crates/game/src/powerups.rs` (`apply_powerup_actions_system`) | `Modifiers::remove_by_source("powerup:<id>")` avant de reposer le modificateur (rafraîchissement, comme CoD) ; test de système ; **aucune trace existante modifiée** |
| D6 | `EntityHits` : pas de test unitaire de l'attente | `crates/scenario/tests/expectations.rs` | un test qui compte des coups sur `target` dans l'arène du testbed |

## Contexte à lire d'abord

- `crates/game/src/powerups.rs` en entier (`powerup_pickup_detect_system`,
  `apply_powerup_actions_system`, `KillAllWaveEnemies`) et `crates/game/src/economy.rs`
  (`PointsCredit`, `FrameEvents<PointsCredit>` : c'est le chemin que les kills empruntent et
  que le nuke ne prend pas). Le crédit de D17 suit ce chemin existant.
- Les précédents de `Modifiers::remove_by_source` : `crates/game/src/interaction.rs` (~607,
  retrait du modificateur « downed ») et `crates/game/src/character/health/mod.rs` (~416).
- `items/powerups.ron` des deux jeux, `tests/scenarios/powerup_nuke.ron` (il n'a pas encore
  d'attente `Currency`), et `crates/scenario/tests/expectations.rs` (un test par attente :
  `health_player_in_range`, `entity_count_player`, `player_downed_true_once_fallen`, … — D6
  imite ce style ; voir aussi le test du générateur ajouté par D11 dans
  `crates/scenario/tests/generated.rs`, qui couvre `entity_hits` au niveau scénario, pas
  l'attente elle-même).
- L'attente `Currency` existe dans `crates/game/src/replay.rs` (variante
  `ScenarioExpectation::Currency`) : D17 ne crée que son usage dans le scénario.

## Règles du lot

- **Aucun bless par l'agent** (les scénarios ne tournent pas dans le cloud). Pour D17,
  l'orchestrateur réécrira la trace `powerup_nuke` à la vérification avec la preuve
  `trace-diff` du README §5 : l'agent décrit dans son rapport **exactement** ce qui doit
  changer (le hash d'état du joueur à partir de la frame du ramassage, à cause des points
  crédités) et pourquoi rien d'autre ne doit changer. Si le code change autre chose que les
  points, c'est un bug du lot.
- D17 : crédit au **ramassage** (pas par ennemi tué), un `PointsCredit` par joueur vivant,
  montant lu dans la config (`economy.ron`), 400 par défaut comme CoD. L'attente `Currency`
  ajoutée à `powerup_nuke.ron` suit la syntaxe des attentes existantes (frame du ramassage +
  marge).
- D18 : rafraîchir = retirer l'ancien modificateur de même source puis poser le nouveau (la
  durée recommence, l'effet ne se multiplie pas). Le test de système est un test Rust pur dans
  `powerups.rs` (ou ses tests), imitant les tests existants du fichier.
- D6 : aucun scénario ni trace : un test Rust pur dans `expectations.rs`, sur le même modèle
  que `entity_count_player` (charger un scénario du testbed, jouer, comparer le compteur au
  nombre de coups attendu).
- Un commit par dette (messages en français, attribution en fin de message), poussés sur la
  branche. Ne pas toucher `docs/taches.md` (journal de l'orchestrateur) ; la colonne
  « acceptation » de `docs/taches/dettes.md` peut être mise à jour (« fait dans m0-v6 »).

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

- 61 scénarios verts après le bless de la seule trace `powerup_nuke` (preuve `trace-diff` :
  le diff ne montre que l'effet des points du nuke) ; les 60 autres traces strictement
  identiques.
- Tous les tests des crates verts, y compris le test D6 et le test de système D18.
- `make lint`, `cargo fmt --all -- --check`, `check-forbidden.sh` sans nouvelle occurrence,
  `make gen GAME=zombies` sans modification.
