# Rapport — m1-restart-p2p (D14, §33)

Branche partie d'`origin/main` 26f1496 ; `origin/main` remergé (963f2d2, puis T1.18).

## État

Livrée : restart en ligne fonctionnel (chemin `--matchbox`), divergence du restart local
trouvée et corrigée, suite verte, aucune trace de scénario déplacée.

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
- **Correctif trouvé par la preuve** : `CollisionGrids` (donnée dérivée, hors rollback) remis à
  zéro dans `cleanup_rollback_world_system`. Sa grille de murs n'est reconstruite que si la
  signature des murs (nombre, somme des `GgrsNetId`) change ; une partie relancée recrée les
  mêmes murs avec les mêmes net ids, donc la grille gardait les `Entity` des murs détruits et la
  partie 2 n'avait plus de murs. Visible au premier essai du script : partie 2 locale différente
  de la partie 1 à f304 (un zombie glisse autrement le long d'un mur, `WallSlideTracker` 6 contre
  5, position x 405,95 contre 406,92) ; trouvé avec la trace détaillée (`ALACOD_STATE_TRACE_FULL=1`
  écrit maintenant `<fichier>.full`, l'historique détaillé des dernières frames). Le bug touchait le
  restart local depuis T2.1 ; aucun scénario ne redémarre, d'où aucune trace déplacée.

## Vérifié

`CARGO_BUILD_JOBS=2`, profil `headless`.

`./scripts/p2p-restart.sh` (sortie réelle, après le correctif et après le dernier merge) :
```
p2p partie 1 : traces identiques (599 lignes)
p2p partie 2 : traces identiques (599 lignes)
p2p : partie 2 différente de la partie 1 (graine dérivée du numéro de partie)
local : partie 2 identique à la partie 1 (599 lignes)
```
Les journaux montrent la seconde connexion dans la salle `restart-<pid>-r1`.

- Suite complète (avant le dernier merge) : `make test_scenarios` vert, **aucune trace déplacée** ;
  tests des crates verts (dont `jjrs::restart`, `run_state` : plan du restart en ligne,
  `state_trace` : `trace_step`, suffixes) ; `make lint` ; `cargo fmt --check` (après `cargo fmt`) ;
  `check-forbidden`, `check-rollback-registration` ; `make gen` testbed, zombies, throne ;
  `cargo check -p throne` ; exemples : verts.
- Après merge d'`origin/main` (T1.18) : build zombies, `cargo test -p game`, `p2p-restart.sh`,
  `make test_scenarios`, `make lint`, `cargo fmt --check` : verts.

## Écarts à la fiche

- Pas de comparaison « partie 2 p2p = partie fraîche de même graine » : la graine relancée n'est
  pas imposable à une partie fraîche sans nouvel argument ; remplacée par « partie 2 ≠ partie 1 »
  (graine dérivée) en p2p et « partie 2 = partie 1 » en local (même graine) qui prouve la remise
  à zéro.
- Délai de 30 s : constante `RESTART_TIMEOUT_SECS`, pas dans `camera.ron`/`ui`.
