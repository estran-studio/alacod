# m0-200-graines-test-map — critère M0 §9.8 rejoué sur `test_map` (calcul long, session cloud)

Tâche de **calcul seulement** : aucun changement de code, aucune trace bénie. Repo
`estran-studio/alacod`, branche `main` (tête `eab022f` ou plus récente). Lire `CLAUDE.md` et
`docs/taches/README.md` §1 (compilation) avant de commencer.

## Contexte

Le critère de sortie de M0 (`docs/plan-engine.md` §9.8) demande que les bots finissent le clone
`zombies` sur 200 graines sans soft-lock ni desync. Il a été mesuré la nuit du 2026-10-04 sur la
carte par défaut `avant_poste` (200/200 vague 5, 0 desync, 0 softlock, 3 graines avec un mort ;
`docs/taches.md` §9). La mesure historique portait sur `exemples/test_map.ldtk` : elle reste à
rejouer sur cette carte avec l'outil actuel. C'est tout l'objet de cette tâche.

## À faire

1. Dépendances système d'une compilation Bevy headless sous Linux (Debian/Ubuntu) :
   `pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev` (ajouter ce que le
   lien réclame). Rust stable via `rustup` (la `rust-toolchain` du dépôt fait foi s'il y en a une).
2. Compiler l'outil, **une seule compilation** :
   `cargo build -q -p scenario --profile headless --bin alacod-sim` → `target/headless/alacod-sim`.
3. Jouer 200 graines, quatre bots `acheteur`, en quatre lots parallèles (ou séquentiels si la
   machine a moins de 16 Go) :
   ```
   ./target/headless/alacod-sim --game zombies --bots 4 --profiles acheteur,acheteur,acheteur,acheteur \
     --map exemples/test_map.ldtk --seeds 1..50 --until-wave 5 --max-frames 20000 --progress \
     --json sim-1-50.json > sim-1-50.log 2>&1
   ```
   puis `51..100`, `101..150`, `151..200`. Les JSON et les logs vont dans `docs/digests/m0-200-test-map/`.
4. Résumer : graines jouées, graines atteignant la vague 5, desync, softlock, graines avec au
   moins un mort, frames min/max ; la liste des graines qui n'atteignent pas la vague 5, avec leur
   `fin` et leurs frames. Le JSON d'une graine porte `seed`, `wave`, `frames`, `deaths`,
   `desync`, `softlock` (lire `crates/scenario/src/bin/alacod-sim.rs` si les noms diffèrent).
5. Écrire `docs/digests/m0-200-graines-test-map.md` : commande exacte, sha, machine (cœurs, Go),
   tableau du résumé, liste des graines en échec, et la phrase « critère atteint / non atteint ».
   Ne pas toucher `docs/taches.md` ni les traces.
6. Commiter sur la branche `m0-200-graines-test-map` (digest + JSON ; pas les logs s'ils dépassent
   1 Mo au total), `git push -u origin m0-200-graines-test-map`. Message final : une ligne
   `LIVRÉ m0-200-graines-test-map <sha> : <n>/200 vague 5, <d> desync, <s> softlock, <m> avec un mort`.

## Si ça bloque

Si la compilation échoue sur une dépendance système introuvable, installer ce qui manque et
réessayer une fois ; si une graine fait planter l'outil, la noter (graine, message) et continuer
le lot sans elle. Si rien ne compile, livrer `BLOQUÉ m0-200-graines-test-map : <cause exacte>`
avec les 30 dernières lignes de l'erreur dans le digest, sur la même branche.
