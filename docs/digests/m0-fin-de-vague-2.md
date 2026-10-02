# M0 `zombies` — digest de fin de vague 2 (et de la vague 3)

`main` à **4f0d550** (2026-10-02, après T3.1 et T3.4). Écrit par T3.2 (agent, session cloud).
Chiffres mesurés dans cette session sauf mention « journal » (mesures de l'orchestrateur sur
la machine de William, `docs/taches.md` §10). Notes de revue des vidéos :
`tests/review-notes/4f0d550.md`.

## En bref

- Le clone joue une partie scriptée complète : achats au mur et perks, à terre et réanimation,
  power-ups, cinq vagues, jusqu'à l'écran de victoire en solo et à deux (`clone_solo`,
  `clone_duo`), jusqu'à l'entrée en vague 5 à quatre (`clone_quad`, partie encore en cours).
- Le déterminisme tient : 61 scénarios bit-identiques à leur trace, 20 parties à quatre bots
  sans un desync.
- Ce qui bloque la sortie de M0 : les bots ne finissent pas le clone (aucune graine sur 20 n'atteint
  la vague 5, 8 tournent jusqu'au plafond sans finir leur vague), la CI de nuit n'a jamais été
  verte, F5 (équilibrage par nombre de joueurs) n'a pas été fait, et la revue humaine (T3.3)
  reste à faire.

## Ce que le clone sait faire

Une ligne par chantier du plan (`docs/plan-engine.md` §5) livré pendant M0, avec sa tâche.

| Chantier | Tâche(s) | Ce que ça donne |
|---|---|---|
| K0 déterminisme vérifiable | T0.1a, T0.1b/c | tout l'état rollback passe par `RollbackTraceApp` (checksum GGRS + trace d'état), enregistrement direct interdit par script ; filet `SyncTestMismatch` |
| K6 CI | T0.4, T2.14 | étage rapide (fmt, tests, scénarios, scripts) vert sur `main` ; étage de nuit écrit (`scripts/nightly.sh`, runner `alacod-builder`) mais jamais vert (voir plus bas) |
| A1 manifeste, registre, lint | T1.5, T2.8, T3.4 | `game.ron`, registre typé, `alacod lint` (36 fixtures), `make lint` sans erreur sur les deux jeux |
| A2 un jeu = un crate + un dossier | T0.3 | `games/zombies`, `games/testbed` |
| A3 expressions | T1.4 | module `content::expr` (parse, `players`, `wave.count`) ; **pas encore évalué par la simulation** (voir F5) |
| A4 RNG par flux | T1.6 | `RunSeed`, `RngStreams` nommés (`waves`, `weapons`, `loot`) |
| B1 équipes et dégâts | T1.1 | `FrameEvents<DamageEvent>`, tir ami `Never`/`Always`/`Cursed` par arme, immunités par tag |
| B2 stats | T1.2 | crate `stats`, modificateurs sourcés, `ALACOD_DUMP_TRACE` + `trace-diff.py` |
| B4 grille spatiale | T1.8a, T2.1 | `CollisionGrids` hors rollback ; `bench_bullets` (155 balles), `bench_horde` (61 ennemis) |
| B6 à terre et réanimation | T1.3 | saignement, réanimation maintenue, défaite quand tous à terre |
| B7 munitions et inventaire | T2.2 | `AmmoType`, réserves par joueur, trois emplacements, lâcher/ramasser |
| C1 v0 power-ups | T2.5 | Insta-Kill, Double Points, Max Ammo, Carpenter, Nuke ; drop à la mort (flux `loot`) |
| C5 v1 monnaie et achats | T2.3, T2.6 | points par kill/coup/réparation, portes payantes, quatre armes murales, quatre perks dans la carte |
| F1 état de run, mode `Waves` | T2.4 | `Run`, `RunSummary`, victoire/défaite, relance sans quitter le binaire (local) |
| I1 HUD | T1.11, T2.12 | `ui/hud.ron` : vie, vague, munitions, monnaie, perks, à terre, power-ups, prompt d'achat avec prix ; écran de fin avec « Lobby » |
| I2 feedback | T2.13 | flash à l'impact, secousse de caméra, sons de tir et de rechargement |
| I3 caméra par joueur en ligne | T1.10 | `online_follow`, `play_scenario --follow` |
| J1 coop à quatre | T1.12 | quatre joueurs en synctest et en p2p (`make test_multiplayer N=4`) |
| K1 attentes et invariants | T1.7 (+ chaque tâche) | 27 sortes d’attentes (`game::replay::Expectation`), trois invariants vérifiés à chaque frame |
| K3 bench | T1.9 | `tests/budgets.ron`, `make bench`, métriques par commit |
| K4 conventions | T2.7 (+ chaque tâche) | `docs/conventions.md` |
| Testbed | T2.9 | quatre salles de laboratoire, six personnages de test, neuf scénarios |
| Générateur de scénarios | T2.10 | `alacod-gen`, un scénario par arme (`tests/scenarios/generated/`) |
| Bots et `alacod sim` | T2.11 | bots `immobile`/`fonceur`/`prudent`, `alacod-sim` (JSON, desync) |
| Scénarios du clone | T3.1 | `clone_solo`, `clone_duo`, `clone_quad` (vagues 1 à 5, achats, perk, à terre, deux power-ups) |
| Dettes de lint et de sortie | T3.4 | D7, D8, D9, D11, D16 fermées |

## Les chiffres

### Tests (cette session)

| Vérification | Résultat |
|---|---|
| `make test_scenarios` | **61 scénarios verts** (51 + 10 générés), aucune trace modifiée (sur `7df8a28` puis sur `4f0d550` + T3.2, 13 min) |
| tests des dix crates (README §4) | **275 réussis, 0 échec, 8 ignorés** (`4f0d550` + T3.2) |
| `make lint` | zombies et testbed : aucune erreur |
| CI rapide sur `main` (GitHub, `7df8a28`) | « Tests & Format » vert (fmt, tests, scénarios, interdits, enregistrement rollback) |

### Bench

Au calme, **journal** (orchestrateur, T3.4, `1af3780`) : `bench_bullets` 104,7 fps (plancher
70), `bench_horde` 66,7 fps (plancher 38), `bots_four_mixed` 109,5 fps. Dans le cloud (cette
session, scénarios en parallèle sur quatre cœurs) : `bench_bullets` 49,3, `bench_horde` 37,9,
24 scénarios sur 61 sous leur plancher : mesure non représentative, sans valeur de bench.

### `alacod-sim` : 20 graines, quatre bots, jusqu'à la vague 5

```
cargo run -q -p scenario --profile headless --bin alacod-sim -- --game zombies --bots 4 \
  --profiles fonceur,fonceur,prudent,immobile --seeds 1..20 --until-wave 5 --max-frames 20000 \
  --json docs/digests/m0-fin-de-vague-2.sim.json
```

Joué sur `7df8a28` (1 h 55, code de sortie 0). Entre `7df8a28` et `4f0d550`, T3.4 n'a changé
que des messages de log et le lint (les 61 traces sont restées identiques) : la simulation
jouée est la même.

| Graine | Vague atteinte | Frames | Morts | Kills | Fin | sim fps | Desync |
|---:|---:|---:|---:|---:|---|---:|---|
| 1 | 4 | 8 960 | 4/4 | 37 | 4 bots morts | 26,8 | non |
| 2 | 2 | 5 901 | 4/4 | 12 | 4 bots morts | 22,6 | non |
| 3 | 2 | 6 417 | 4/4 | 9 | 4 bots morts | 36,5 | non |
| 4 | 1 | 20 000 | 2/4 | 0 | plafond (bloquée) | 40,5 | non |
| 5 | 2 | 6 302 | 4/4 | 16 | 4 bots morts | 37,5 | non |
| 6 | 3 | 7 265 | 4/4 | 21 | 4 bots morts | 31,6 | non |
| 7 | 2 | 6 238 | 4/4 | 14 | 4 bots morts | 28,4 | non |
| 8 | 3 | 8 798 | 4/4 | 21 | 4 bots morts | 35,6 | non |
| 9 | 2 | 5 536 | 4/4 | 14 | 4 bots morts | 51,7 | non |
| 10 | 2 | 6 551 | 4/4 | 15 | 4 bots morts | 24,6 | non |
| 11 | 2 | 20 000 | 0/4 | 18 | plafond (bloquée) | 41,0 | non |
| 12 | 1 | 20 000 | 2/4 | 1 | plafond (bloquée) | 39,2 | non |
| 13 | 2 | 20 000 | 0/4 | 19 | plafond (bloquée) | 35,3 | non |
| 14 | 3 | 6 750 | 4/4 | 18 | 4 bots morts | 28,0 | non |
| 15 | 2 | 7 162 | 4/4 | 13 | 4 bots morts | 36,0 | non |
| 16 | 1 | 20 000 | 0/4 | 5 | plafond (bloquée) | 33,5 | non |
| 17 | 1 | 20 000 | 0/4 | 6 | plafond (bloquée) | 36,5 | non |
| 18 | 1 | 20 000 | 0/4 | 6 | plafond (bloquée) | 37,5 | non |
| 19 | 2 | 20 000 | 0/4 | 16 | plafond (bloquée) | 35,7 | non |
| 20 | 2 | 5 551 | 4/4 | 10 | 4 bots morts | 33,3 | non |

**Desync : 0 sur 20.** Vague atteinte : 1 (5 graines), 2 (11), 3 (3), 4 (1) ; aucune vague 5.
271 kills au total. Les **8 graines bloquées** (4, 11, 12, 13, 16, 17, 18, 19) atteignent les
20 000 frames sans que leur vague se termine, dont 5 avec les quatre bots en vie : soit des
zombies restent hors d'atteinte des bots (les bots v0 n'ont pas de pathfinding, journal T2.11),
soit la vague ne peut plus se terminer. C'est un softlock au sens du critère de sortie de M0,
et la cause n'est pas encore diagnostiquée. Le fps de simulation (22 à 52) est celui du synctest
dans le cloud.

## Vidéos

Rendues sur `4f0d550` (`make videos SCENARIO="clone_solo clone_duo clone_quad"`, 960×540, une
image toutes les deux frames, Xvfb + Vulkan logiciel lavapipe), regardées image par image aux
moments clés (notes : `tests/review-notes/4f0d550.md`). Toutes font moins de 5 Mo et sont
copiées dans `docs/digests/videos/` :

| Vidéo | Durée | Taille | Moments clés | Source |
|---|---:|---:|---:|---|
| [`clone_solo.mp4`](videos/clone_solo.mp4) | 50,8 s | 1,8 Mo | 127 | `target/videos/4f0d550/clone_solo.mp4` |
| [`clone_duo.mp4`](videos/clone_duo.mp4) | 77,8 s | 3,0 Mo | 128 | `target/videos/4f0d550/clone_duo.mp4` |
| [`clone_quad.mp4`](videos/clone_quad.mp4) | 72,9 s | 4,6 Mo | 240 | `target/videos/4f0d550/clone_quad.mp4` |
| [`montage.mp4`](videos/montage.mp4) | 77,8 s | 3,3 Mo | — | `target/videos/4f0d550/montage.mp4` |
| `clone_quad.vues.mp4` (quatre vues `--follow`) | 72,9 s | 15,6 Mo | 240 | **lien seulement** (> 5 Mo) : `target/videos/4f0d550/clone_quad.vues.mp4`, `make views SCENARIO=clone_quad` |
| [`idle_4e93269_vs_4f0d550.mp4`](videos/idle_4e93269_vs_4f0d550.mp4) (comparaison) | 25,0 s | 3,5 Mo | 8 / 14 | `target/videos/compare/idle_4e93269_vs_4f0d550.mp4` |
Les moments clés (`*.events.json`) sont copiés à côté de chaque vidéo. Page de revue :
`make review_videos`, puis http://localhost:8766 (révision `4f0d550`).

À voir en priorité : la vue du joueur 3 de `clone_quad`, puisque la vue principale suit le
joueur 0 (un bot) et ne montre pas les achats, le Max Ammo ni la Nuke ; l'écran « VICTOIRE »
de `clone_solo` (f3020) ; « Réanimation 47 % » de `clone_duo` (f260).

### Comparaison avec la vague 0 (`4e93269`)

`make compare_video SCENARIO=idle BASE=4e93269` : le même `idle.ron` (celui de `main`, sans ses
attentes) joué à gauche par le code de la vague 0, à droite par `4f0d550`. Regardée aux frames
200, 336, 390, 470, 880 et 1120 ; les deux moitiés diffèrent de 9 000 à 63 000 pixels selon le
moment (sur 480 000).

| | Vague 0 (`4e93269`) | `main` (`4f0d550`) |
|---|---|---|
| Zombies hors de la pièce | f181 | f181 |
| Première fenêtre cassée | f329 | f381 |
| Premier coup reçu | f399 | f465 |
| Deuxième fenêtre cassée | f775 | f873 |
| Mort du joueur | f1052 | f1117, écran « DÉFAITE » (vague 1, 0 kills, 0 points) |
| Moments clés | 8 | 14 (dont 5 armes murales à f0 et la défaite) |

Ce qui a changé à l'écran : le HUD (vague, solde, vie, munitions, ennemis), les armes murales et
la machine à perk dans la pièce de départ (T2.6), la ligne de debug du bas, l'écran de fin avec
résumé et boutons (T2.4, T2.12). Même carte, même trajet de la horde autour du bâtiment ; les
dates décalées viennent du tirage par flux de T1.6 (attentes recalées à l'époque, commentaire
« À regarder » d'`idle.ron` resté sur les dates de la vague 0, corrigé par T3.2). Anomalie vue
sur `main` à f1120 : après la mort, le HUD affiche « $ » sans montant et « ? | ? » pour les
munitions.

Le premier essai a produit une comparaison fausse (deux fois le code de la vague 0) :
`scripts/scenario-video compare` partage le `target` entre les deux arbres, cargo y juge à jour
les crates du workspace compilées pour l'autre côté, et l'échec de la compilation de `main`
n'arrêtait pas le script. Corrigé dans T3.2 (`inherit_errexit` et `touch` des sources des deux
arbres avant chaque compilation) ; la comparaison ci-dessus est celle du script corrigé.

## Ce qui manque pour M0

Critères de sortie du plan (§9.8), état au `4f0d550` :

| Critère | État |
|---|---|
| Lint et tests verts sur `main` | **oui** : CI rapide verte sur `7df8a28`, vérification locale verte sur `4f0d550` |
| Tous les scénarios du clone en synctest à 2 et à 4 | **oui** : `clone_duo` et `clone_quad` sont joués en synctest et sont verts, ainsi que `four_players_*` |
| Les bots finissent le clone sur 200 graines sans softlock ni desync | **non** : 20 graines jouées ici, 0 desync, mais aucune ne finit (vague 5 jamais atteinte) et 8 sont bloquées ; 200 graines non tentées. Il faut des bots qui survivent et chassent les zombies isolés (pathfinding, profil qui achète), puis le diagnostic des 8 blocages |
| Bench dans les budgets | **oui au calme** (journal, T3.4) ; non vérifiable dans le cloud |
| Vidéos publiées | **en partie** : générées et regardées, copiées dans le dépôt ; la publication automatique (étage de nuit, page de revue en ligne) ne tourne pas |
| Doc des conventions à jour | à jour pour chaque tâche livrée (T2.12, T3.4…), sans relecture d'ensemble |
| Notes du jalon précédent fermées | sans objet pour M0 (premier jalon) ; les notes de ce digest restent ouvertes pour T3.3 et la fermeture des notes |

Autres manques :
- **Restart p2p non supporté** : en ligne, « Rejouer » redirige vers le lobby
  (`crates/game/src/run_state.rs`, `RunRequest::Restart`, « Local seulement »). **Reporté à
  M1** : relancer une partie en ligne demande de recréer la session GGRS entre les pairs, ce qui
  sort de M0. Dette D14 fermée par ce digest (`docs/taches/dettes.md`).
- **F5, équilibrage par nombre de joueurs** : M0 le liste (plan §6) mais aucune tâche ne l'a
  porté. `content::expr` sait lire `players`, mais aucun RON de la simulation n'est évalué par
  `Expr` : vagues, prix et santé ne dépendent pas du nombre de joueurs.
- **CI de nuit** (plan §9.9, étage lent) : 15 exécutions sur `main`, **aucune verte** (10
  annulées, 5 en échec ; la n° 14 sur `7df8a28` échoue à l'étape bench en moins d'une seconde,
  sortie alors masquée). `main` a reçu depuis plusieurs corrections du runner (make, ffmpeg,
  build-essential, bench sans budgets absolus, sortie visible) : à revérifier à la prochaine nuit.
  Déploiement WASM (« Build & Deploy ») en échec sur le runner auto-hébergé pour `7df8a28`,
  migration de l'action Cloudflare en cours (`a455e95`).
- **p2p par allumette** (D12) : le client natif ne sait pas s'authentifier ; la CI utilise
  `matchbox_server` nu.
- **Revue humaine** : jouer trente minutes à deux (T3.3), puis transformer chaque note en
  attente, invariant, scénario ou tâche de M1. Attention : le plan appelle ça T3.4 (« fermeture
  des notes »), mais la fiche `docs/taches/T3.4-dettes-lint-sortie.md` a servi à un lot de dettes.
- Présentation : pas de sprite de pickup pour les power-ups ni de retour visuel des effets
  instantanés, pas d'état du coéquipier à l'écran, sprites encore nommés en code (D3).

## Dettes ouvertes

D'après `docs/taches/dettes.md` (après T3.4 et ce digest) :

| # | Dette |
|---|---|
| D3 | sprites nommés en code, feuilles rangées sous `ZombieShooter/Sprites/**`, le `rifle` sans sprite propre |
| D5 | `HitCount` et `ai.stationary` (testbed) hors checksum : décision à prendre |
| D6 | pas de test unitaire de l'attente `EntityHits` |
| D10 | le gabarit de départ réutilisé comme salle ordinaire : décision de design |
| D12 | allumette (JWT) non supporté par le client natif |
| D13 | pas de résumé à l'abandon ; `ToLobby` relance aussitôt une partie en local |
| D15 | gain de la grille de collision jamais mesuré en ratio |
| D17 | la Nuke ne crédite pas de points |
| D18 | deux power-ups identiques se cumulent (×4) |
| D19 | `InputRecorder::to_scenario` ne reporte pas les réglages de scénario |

Trouvailles de ce digest, à ajouter par l'orchestrateur : les 8 graines bloquées d'`alacod-sim` ;
aucun moment clé pour la victoire ni pour le ramassage d'un power-up, et les armes murales
étiquetées « tombée au sol » à la frame 0 (`crates/scenario/src/events.rs`) ;
le HUD affiche « $ » sans montant et « ? | ? » une fois le joueur mort (comparaison, f1120) ;
`make videos SCENARIO=a,b` (écrit avec des virgules dans la fiche T3.2) ne marche pas : le script
attend des noms séparés par des espaces ; les métriques de `make test_scenarios` vont dans
`crates/scenario/target/metrics/` quand `CARGO_TARGET_DIR` n'est pas défini (session cloud), et
`scripts/scenario-metrics.py` ne les trouve pas.
