# T3.1 — scénarios du clone zombies

SHA de la tête de branche vérifiée (code et traces) : `0c6ba3165b35d35fd3d1320c47b3808df8bd4c1c`.
Fiche : `docs/taches/T3.1-scenarios-du-clone.md`.
Base : `origin/main` à `80123e971b38407c05ba9497d6eab9df8422f8c9`.
Branche : `m0-v3-scenarios-clone`. Agent : Codex. Date : 2026-10-01.
Le SHA livré inclut ensuite le commit documentaire de ce rapport ; ce dernier ne change
ni code, ni contenu, ni trace. Le SHA final est transmis dans la ligne de livraison.

## 1. Fait

Branche créée depuis `origin/main` par la commande demandée, checkout direct dans
`alacod/` du workspace fourni (variante cloud demandée, sans nouveau worktree ni env.sh).
Le target headless préexistait : première compilation annoncée en 0,35 s. Une seule
commande Cargo à la fois, toujours profil headless ; seul build avec rendu :
`play_scenario --features render` pour les vidéos autorisées. Seul le repo alacod est modifié.

Trois scénarios RON et leurs traces sous `tests/scenarios/`, sur `zombies`,
`exemples/test_map.ldtk`, graine 123456. Ils désactivent les drops aléatoires et
placent Max Ammo et Nuke à des frames précises. Les invariants restent tous actifs.

| Scénario | Joueurs | Frames | Lignes de trace | Vague / kills | État final |
|---|---:|---:|---:|---|---|
| `clone_solo` | 1 scripté | 3047 | 3046 | 5 / 13 | Victory, vivant |
| `clone_duo` | 2 scriptés | 4668 | 4667 | 5 / 13 | Victory, deux debout |
| `clone_quad` | 2 fonceurs, 1 prudent, 1 scripté | 4372 | 4371 | 5 / 13 | Playing, quatre debout |

- Solo : pistolet acheté à f116 (500), Juggernog à f166 (2500, MaxHealth 200),
  Max Ammo posé à f400 (25 → 30 balles, réserve Balle 240), Nuke à f650
  (3 ennemis avant, 0 après), soldes après achats 3000 → 3520 → 3910 → 4200.
- Duo : une balle du pistolet Never est effectivement tirée vers le joueur 0 à f40
  (`Ammo` 6 → 5, `NoDamageBetween`, santé 100) ; joueur 1 à terre à f149,
  réanimé à f355 ; achats f516/f566, Max Ammo f800 (deux joueurs), Nuke f950.
  Le combat utilise ensuite deux pistolets Never. Soldes finaux 3400/1040, santé 200/90.
- Quad : rifle Always réservé à la mise en scène, joueur 0 à terre f90 et réanimé f355 ;
  le joueur 3 achète le pistolet f526 et Juggernog f586, puis utilise le pistolet Never.
  Max Ammo f700, Nuke f750. Les bots restent pilotés par leur profil ; leur solde monte
  de 500 à 1040/1170/890. Le joueur scripté garde 3000 après ses achats.
  Les quatre santés restent positives à la fin ; `RunState(Playing)` exclut toute
  défaite préalable puisque la fin de Run est irréversible.
- 28/41/37 attentes respectivement : vagues 1 à 5 datées, achats, perk, munitions,
  Nuke avant/après, points, états des joueurs et du run. « À regarder » avec les frames
  observées via `ALACOD_EVENTS=1`, sans inspection d'images.
- `crates/game/src/replay.rs` / `crates/scenario/src/runner.rs` : ajout des champs
  optionnels `WaveOverride.max_wave` et `min_wave_delay_frames`, appliqués à l'asset
  chargé avant simulation. Absents, ils conservent la configuration du jeu. C'était
  nécessaire pour obtenir un résumé de victoire sans changer les assets existants et
  pour raccourcir les pauses. Les effectifs sont 2 + variance 0..2, sans croissance,
  délai entre vagues 120 frames ; préparation solo 300, duo/quad 600.
- `crates/scenario/tests/run.rs` : test de victoire avec résumé à `max_wave: 1`, puis
  sérialisation/rejeu de l'enregistrement et égalité des traces.
- `docs/conventions.md` : les trois parties sont documentées comme références du clone.
  Aucun ajout à `tests/budgets.ron` : les trois rejeux restent nettement sous 90 s.

Les inputs de combat ont été enregistrés en headless à partir de visées réévaluées
par tranches de 15 frames, puis figés en RON. L'outil temporaire d'écriture a été retiré.
Les premières étapes solo/duo/quad ont été jouées à 600/1200 frames avec bless ciblé ;
les essais longs qui faisaient tomber des joueurs n'ont pas été retenus.

## 2. Vérifié

- `ALACOD_EVENTS=1 make test_scenarios` sans bless : **61 scénarios verts**
  (51 manuscrits + 10 générés), 0 divergence de trace, 0 mismatch de synctest, 313,41 s.
  Les 2 tests de cette commande passent, 7 diagnostics ignorés.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots -p effects --no-fail-fast` :
  **258 tests réussis, 0 échec, 8 ignorés**, code de sortie 0. Dont 30 tests d'attentes,
  3 tests de run (`crates/scenario/tests/run.rs`) et le second passage des 61 scénarios.
- `cargo test -q --profile headless -p scenario --test determinism` : **2 réussis,
  0 échec**, 9,82 s. Les trois nouveaux scénarios passent également le synctest dans
  les deux rejeux globaux.
- `make lint` : **2 jeux sans erreur** (`zombies`, `testbed`), sortie 0.
- `cargo fmt --all -- --check` : sortie 0, aucun diff.
- `./scripts/check-forbidden.sh` : sortie 0, **4 occurrences préexistantes**
  (3 HashSet, 1 commentaire rand), aucune dans les fichiers Rust modifiés.
- `./scripts/check-rollback-registration.sh` : sortie 0, **OK**.
- `git diff --check` et vérification des nouveaux fichiers par `git check-ignore -v` :
  aucun problème, aucun nouveau fichier ignoré.

Mesures du premier rejeu global normal (durée murale affichée par le runner) :

| Scénario | Durée | FPS simulés | Joueurs vivants |
|---|---:|---:|---:|
| `clone_solo` | 20,2 s | 152,3 | 1 |
| `clone_duo` | 31,4 s | 149,2 | 2 |
| `clone_quad` | 37,8 s | 116,0 | 4 |

`make videos SCENARIO=clone_solo`, puis `clone_duo`, puis `clone_quad` : les trois
commandes passent, avec montage et page de revue. Métadonnées contrôlées par ffprobe :
H.264, **960×540, 30 images/s**, respectivement **1524 / 2334 / 2186 images**,
durées **50,8 / 77,8 / 72,866667 s**. Les fichiers `.events.json` sont comparés
intégralement aux événements du rejeu headless : **127 / 128 / 240 événements
identiques** (frames, kinds et labels), soit 495. Aucune image n'a été visionnée.

Journaux locaux : `/tmp/m0-v3-standard-scenarios.log`,
`/tmp/m0-v3-standard-crates.log`, `/tmp/m0-v3-determinism.log`,
`/tmp/m0-v3-lint.log`, `/tmp/m0-v3-fmt.log`, `/tmp/m0-v3-forbidden.log`,
`/tmp/m0-v3-rollback-registration.log`, `/tmp/m0-v3-video-*.log`.

Bless final ciblé avec `ALACOD_EVENTS=1 make test_scenarios SCENARIO=<nom> BLESS=1` :
solo 3047 frames / 13 kills (24,1 s), duo 4668 / 13 (36,3 s), quad 4372 / 13 (40,9 s),
chaque commande verte (2 tests réussis, 0 échec, 7 ignorés). Jamais de bless global.
Les 58 traces héritées ont été comparées par leurs hashes Git à `origin/main` :
**58 identiques, 0 modifiée**. Les seules traces ajoutées sont les trois nouvelles.
Pas de `trace-diff` : aucune trace existante n'est réécrite, exemption des nouveaux
scénarios prévue au README §5 et dans le prompt. Le commit porte chaque justification :
« clone_solo : nouveau scénario, trace créée », idem pour clone_duo et clone_quad.

## 3. Non fait / non vérifié ici

**Revue visuelle non faite**, conformément à la consigne « tu ne peux pas voir
d'images ». Vidéos produites mais **non visionnées** :
`target/videos/0c6ba31/clone_solo.mp4`, `clone_duo.mp4`, `clone_quad.mp4`,
avec les trois `.events.json` et `montage.mp4` dans le même dossier.
Page de revue : `target/videos/index.html`. Ces artefacts locaux ne sont pas versionnés ;
l'orchestrateur peut les regarder ici ou les régénérer sur la branche livrée.

- P2P à deux clients : **non vérifié ici**, conformément à la variante cloud demandée.
- Bench au calme (`ALACOD_BENCH_STRICT=1`) : **non vérifié ici** ; les fps ci-dessus
  sont ceux d'un test normal, pas une validation de benchmark. À rejouer par l'orchestrateur.
- `make gen GAME=zombies` : non lancé, aucune arme ni définition d'arme modifiée.
- Pas de merge dans main, ni de modification de `docs/taches.md`, ni de fiche suivante.

## 4. Dettes / points de revue

- Validation visuelle des trois parties par l'orchestrateur. Utiliser les commentaires
  « À regarder » et les événements datés ; la santé et les achats ont été vérifiés par
  attentes, pas par observation du HUD.
- Le mode Waves gagne à l'**entrée** en vague 5 : solo et duo ne combattent pas cette
  cinquième vague. Le quad s'arrête également dans sa préparation, mais reste Playing.
  Ce comportement existant est explicite dans les scénarios et la documentation.
- Nuke ne crédite pas de points (dette préexistante D17) ; les attentes de progression
  monétaire portent sur le combat ultérieur. Aucune modification de simulation hors
  réglages de scénario, aucun nouvel état rollback, aucune dette nouvelle identifiée.
