# Suivi M-7 (harmonium) — sur le VPS (coordinateur alacod, run `run_9effecfcae39`)

Objectif studio O2 au service d'O3. Jalon **S2 dim 22 nov**. Présentation seule (aucun système de `GgrsSchedule`, aucun type rollback). Tu ne merges pas.
Preuve attendue (brief du studio, 19 nov) : **cinq lignes de transcript** par point (commande et sortie), + tests rouge avant / vert après pour P2.

## À lire
`CLAUDE.md` ; `~/orca/runs/alpha/commun.md` (écrit pour le Mac, **ce fichier-ci prime**) ; la spec du lot livré `~/orca/runs/alpha/m-7-harmonium.md` ; son rapport
`docs/taches/rapports/m-7-harmonium.md` (§ 121, D217) ; la revue du merge `00fc97f1` (points P2/P3 cités ci-dessous).

## Les trois points
1. **P2** : une période de pulsation hors 40–300 BPM n'est pas refusée au lancement (`crates/game/src/audio/music.rs`, `prepare`) : la refuser (erreur claire, pas de musique,
   pas de panique), et que `alacod lint` la signale déjà sur le manifeste (vérifier ; sinon l'ajouter, fixture rouge/verte). Test rouge avant, vert après.
2. **P3** : allocations à chaque frame dans `log_music_status` : n'allouer qu'au changement d'état (ou derrière un niveau de journal), mesure ou test qui le montre.
3. **Docs périmées** : `docs/plan-engine.md` (l. ~122 et ~697) et la dette D151 décrivent l'ancienne intégration harmonium : les mettre à jour (D151 fermée ou réécrite selon l'état réel).

## Où, sur le VPS
- Worktree `/home/debian/orca/workspaces/alacod/m-7-suivi`, branche `wquintal/m-7-suivi` (depuis `origin/main`).
- **Cible cargo partagée** : `CARGO_TARGET_DIR=/home/debian/orca/workspaces/alacod/target-partage` sur toutes tes commandes (aucun autre dossier cible ; pas de `target_wasm` :
  `cargo check --target wasm32-unknown-unknown` seulement si un point touche du code compilé en wasm, et alors dis-le avant par `status`).
- Banques harmonium déjà en cache : `~/orca/workspaces/alacod/packs/harmonium-banks`. Pas de son : `ALACOD_MUTE=1`, pas de carte son sur le VPS (ALSA null).

## Règles de la machine
Préfixe `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1` ; un seul cargo ; jamais de profil debug ; cargo seulement si `df -h /` ≥ 20 Go libres, arrêt
de ton cargo sous 12 Go ; `scripts/target-sweep.sh $CARGO_TARGET_DIR` après chaque cargo ; rien entre 03:45 et 05:00 UTC.
Contrôle rapide **découpé** : reprends `~/orca/runs/alpha/logs/p-1d-check-rapide-decoupe.sh` et `p-1d-test-crates-decoupe.sh` sous ton nom (journal
`logs/m-7-suivi-check-rapide-<sha>.log`), puis purge les binaires de tests d'intégration. Aucune trace modifiée.

## Fin
`status` « E prêt <sha> » avec le journal ; complète `docs/taches/rapports/m-7-harmonium.md` (section « Suivi ») ; un seul `worker_done` (`--report-path`).
Communication : commite et pousse à chaque étape ; carte `orca worktree set --worktree active --comment …` ; 429 : `status` tout de suite ; question : `ask` avec ton choix par défaut.
