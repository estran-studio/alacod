# Rapport sb-4-jeu-externe-docs (b6) : jeu créé par `alacod new`, docs, protocole SB-5

Fiche : `docs/taches/plan-bac-a-sable.md` (§1 critère 2, §4 SB-4, §5). Branche `sb-4-jeu-externe-docs` depuis `origin/main` `fb93c818` (SB-1 à SB-3 mergés). VPS sans affichage. **Aucun code de l'engine touché** (un test, des docs, le `CLAUDE.md` du gabarit) :
aucune trace ne bouge, pas de BLESS, `schedule_order_diff` sans objet.

## 1. Test de bout en bout (critère 2)

`crates/content/tests/cli.rs::new_game_opens_its_sandbox_without_any_added_code` (`#[ignore]`, `make test_alacod_cli`). Étapes, toutes vérifiées :

1. `alacod new demo --engine <engine>` dans un dossier temporaire ; `src/` ne contient que `main.rs`, qui ne dit pas « sandbox » (aucune ligne ajoutée).
2. `cargo build --no-default-features --features native --profile headless` dans `demo`, le profil `headless` redéclaré par `--config profile.headless.*` (le jeu est son propre workspace) ; `CARGO_TARGET_DIR` partagé avec l'engine.
3. `demo --sandbox --local-port 0 --players localhost`, `ALACOD_HEADLESS=1`, port libre, `ALACOD_STATE_TRACE` + `ALACOD_EXIT_AT_FRAME=1200` (la seconde variable exige la première), lecture par `scripts/alacod-remote state` dès la frame 300, puis attente de la sortie spontanée.

Sortie réelle (cargo test, VPS) : `test new_game_opens_its_sandbox_without_any_added_code ... ok`, `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 591.04s`.
Essai à la main du même binaire (premier jeu `demo`, frame 351) : `editor=True online=False check_distance=0 run.step=Playing { since_frame: 0 } jeu=demo carte=maps/arena.ldtk graine=123456 sandbox=arene`,
`vague={'killed': 0, 'number': 0, 'phase': 'NotStarted', 'to_spawn': 0}`, `joueur 0 (424.0,696.0) pv 100.0/100.0 arme machine_gun 30x240`, cible `target` en (624.0, 696.0) pv 5000 ; sortie spontanée code 0 à la frame 899 (« frames resimulées après rollback : 0 »).

Durée : 591 s (≈ 9,5 min de compilation à froid du jeu, puis ≈ 20 s de jeu). Trop long pour `make test_crates` et la CI : `#[ignore]`, joué par `make test_alacod_cli` (qui joue déjà `alacod test`, 426 s) ; la commande à la main est dans `docs/conventions.md` §88.
Le test tue son propre enfant (`Child::kill`, PID propre) en cas d'échec ; jamais par motif.

Observé, pas une anomalie : la carte du gabarit porte cinq `character_spawn` (civilian, follower, dummy, ally, target) qui apparaissent en plus de la cible du profil (six « ennemis » dans `brief`, dont un `follower` qui poursuit le joueur) ;
le journal porte une trace `spawner_skip reason=no_valid_spawner` par frame (trace voulue du spawner, format `ggrs{…}`).

## 2. Docs touchées

| Fichier | Changement |
|---|---|
| `docs/editeur-vocal.md` | §1 : `make sandbox` en tête, jeu créé (`cargo run -- --sandbox`) ; §2.1 : renvoi et ids ; §2.2 : tableau des phrases (« donne-moi le fusil », « mets-moi 500 sous », « charge le profil lourd », « spawne X devant moi », « reset », « un peu plus rapide » → commandes exactes, `unmodify` ; profil du jeu créé) ; §4 : bloc « prêt à coller » mis à jour (12 points, mode bac à sable) ; §8 : phrase périmée sur `spawn` (« n'existe pas dans K5-1 ») corrigée |
| `docs/jouer.md` | ligne du jeu créé, renvoi à `make test_alacod_cli` à la place de « la preuve est le lot SB-4 » |
| `docs/conventions.md` | §88 (jeu créé : mode d'emploi, preuve, observations) |
| `CLAUDE.md` | bac à sable d'un jeu créé sans code, test de preuve, `ALACOD_EXIT_AT_FRAME` exige `ALACOD_STATE_TRACE` |
| `templates/starter/CLAUDE.md` | section « Bac à sable » (profils `sandbox/*.ron`, commandes `alacod-remote`, `restart`), ligne « Lancer » ; pas de `README.md` créé : le gabarit n'en a pas, `CLAUDE.md` est le fichier lu par l'agent du jeu |

## 3. SB-5 préparé

`docs/taches/rapports/sb-5-essai-vocal.md` : préparation sur le Mac (compilation, lint, `make sandbox GAME=zombies`, terminal B, voix), 13 phrases (profils, fusil, 500 sous, spawn, un peu plus rapide, charge `lourd`, reset, retour à `arene`, édition + annulation,
ambiguë, refus par `ALACOD_CHECK_DISTANCE=2`, id inconnu) avec la commande attendue et l'effet à voir, bonus jeu créé, grille de résultats **vide** (« non joué »), ce que la séance décide (rendu D94/D132, délai de la boucle, pont MCP de K5-4).
Commandes `alacod-remote` de chaque ligne déjà exécutées en headless par SB-3 ; rien n'a été vu à l'écran.

## 4. Vérifications

| Vérification | Résultat |
|---|---|
| test de bout en bout (`make test_alacod_cli`, filtre `new_game_opens`) | 1 passé, 591 s |
| `cargo test -p content --profile headless --no-default-features -- --skip scenarios` | 108 + 11 (1 ignoré) + 2 + 171 + 6 + 5 + 40 + 8 passés, **0 échec** |
| `make test_tools` | 38 tests OK |
| `make lint` (zombies, testbed, throne, gungeon, isaac) | 0 erreur ×5 |
| `cargo fmt --all -- --check` | ok |
| `scripts/check-forbidden.sh` | OK, 51 occurrences auditées |

Seul `content` est touché (un test) : les autres paquets n'ont pas de changement de code. Pas de dette ouverte (D133 inutilisée).

## 5. Limites

- Rendu jamais vu (D94, D132) : SB-5. Le test ne fait pas jouer d'input : il prouve l'ouverture, le profil, le remote et la sortie.
- Hors CI : la compilation d'un jeu complet (≈ 10 min à froid) reste dans `make test_alacod_cli`.
- Un seul profil (`arene`) dans le gabarit : `lourd` se crée par `alacod new-content Sandbox lourd`.
