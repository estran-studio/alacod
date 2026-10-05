# Rapport — m1-restart-p2p (D14, §33)

Branche partie d'`origin/main` 26f1496.

## État en cours

Code écrit, **pas encore compilé** (attente du signal d'orch). Reste : compilation, tests,
`make test_scenarios` (aucune trace ne doit bouger), `scripts/p2p-restart.sh` (sorties réelles
ici), suite complète.

## Fait

- `game::jjrs::restart` : `OnlineGames` (parties en ligne, remis à 0 à l'entrée normale de
  `LobbyOnline`), `OnlineRestart { game, since }`, `restart_room` (`{lobby}-r{n}`),
  `restart_seed`, `restart_timed_out` (30 s), tests purs ; socket matchbox fermé à
  `OnExit(InGame)`.
- `run_state` : `plan_run_request` reçoit `restart_online` (chemin `--matchbox`) ; « Rejouer » en
  ligne → `OnlineRestart` + `LobbyOnline`, sans abandon ; allumette : redirect conservé ; tests.
- `jjrs::p2p` : salle de restart, délai → salle d'origine, graine dérivée dans
  `system_after_map_loaded`.
- `ui::disconnected` : pas d'overlay si la partie est terminée ; overlay retiré à la sortie de
  partie.
- `state_trace` : `ALACOD_RESTART_AT_FRAME` (traces `-g1`/`-g2`, restart 60 frames après `-g1`),
  `trace_step` pur testé.
- `scripts/p2p-restart.sh` ; README §4 ; `docs/conventions.md` §33.

## Écarts à la fiche

- Pas de comparaison « partie 2 p2p = partie fraîche de même graine » : la graine relancée n'est
  pas imposable à une partie fraîche sans nouvel argument ; remplacée par « partie 2 ≠ partie 1 »
  (graine dérivée) en p2p et « partie 2 = partie 1 » en local (même graine) qui prouve la remise
  à zéro.
- Délai de 30 s : constante `RESTART_TIMEOUT_SECS`, pas dans `camera.ron`/`ui`.
