# polissage-perf-web — FPS dans le navigateur (S1-5)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 7), `CLAUDE.md` (« Coût du rollback »),
`tests/budgets.ron`, `docs/taches/dettes.md` (D70, D97, D118, D122), inventaire §7 et §13. Branche `polissage-perf-web`
depuis `origin/main`, après S1-2. Estimation : 1 j. Machine : **Mac avec Chrome** (au calme, charge notée).

## Contexte

Aucune mesure FPS wasm n'existe (inventaire §13). La simulation (60 Hz, rollback GGRS) tourne sur le thread principal du
navigateur [INFERENCE : aucun `atomics` trouvé]. Mesures headless du VPS (`latest.json`, commit `0d2c1b61`) : `isaac_quad` 10,1 fps
sim / 50,4 ms de pire rollback ; `isaac_duo` 20,5 fps ; `bench_floor_isaac` 22,7 fps (sous son plancher de 26) ; zombies et
throne bien au-dessus de leurs planchers.

## À faire

1. Ajouter (présentation seule, hors `GgrsSchedule`) un compteur FPS d'affichage et de simulation lisible dans la console
   ou en surcouche de debug (touche existante ou paramètre d'URL) ; `ALACOD_SNAPSHOT_PROFILE` n'existe pas en wasm.
2. Mesurer, par clone, à 1 puis 2 joueurs (deux onglets locaux) : FPS affichage, FPS sim, parties de 3 minutes de bot ou
   scénario rejoué ; tailles et temps de chargement (avec S1-2).
3. Si un clone tombe sous 60 Hz : profil, puis première dégradation à trace identique (lumière coupée pour le web,
   `crates/game/Cargo.toml` feature `lighting`) ; **ne pas** réduire le contenu sans William (décision 3).

## Acceptation

- Tableau « clone × joueurs » FPS affichage/sim, navigateur et machine nommés, charge notée ; « non mesuré » pour le reste.
- Aucune trace ne bouge. Rapport `docs/taches/rapports/polissage-perf-web.md`.
