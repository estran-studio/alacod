# m1-analyse-200-throne — pourquoi les bots perdent sur `throne` (200 graines)

## État en cours

- Fait : classement des 51 échecs des 200 graines à 2 bots (39 défaites, 12 soft-locks),
  rejeu des 39 défaites avec le binaire exact (39/39 identiques au JSON), 12 rejeux détaillés,
  trois scénarios figés, digest `docs/digests/m1-200-graines-throne.md`. 4 bots : run arrêté par
  la mémoire de la machine, non analysé.
- Base : origin/main `a88339a` (crates et `games/` identiques à `61ac539`, `git diff` vide),
  puis merge d'origin/main avant livraison.

## Fait

- Lecture et classement des JSON (`../m1-200-throne/throne-2-*.json`) : tableau en annexe du
  digest, une ligne par graine en échec, cause et preuve (champ JSON ou rejeu).
- `alacod-sim` n'a pas de subscriber de log (`RUST_LOG` muet) : les rejeux détaillés passent par
  le scénario sauvé (`--save-scenario`) rejoué dans `cargo test -p scenario --test scenarios`
  avec `ALACOD_SCENARIO` et `ALACOD_DUMP_TRACE`, la copie du `.ron` dans `tests/scenarios/` le
  temps du rejeu (jamais commitée). Scripts hors dépôt : `../d40/throne200/` (`rejeu.sh`,
  `rejeu_detail.sh`, `extrait.py` qui résume la trace de 1,1 Go en un extrait de 1 Mo,
  `analyse.py`, `fige.py`). Extraits : `../m1-200-throne/rejeux/extraits/`.
- Incident : deux rejeux détaillés en parallèle ont saturé la mémoire (le runner garde la trace
  détaillée en mémoire), arrêtés par le harnais ; repris un à la fois sur 12 graines (accord
  d'orch).
- Une compilation (feu vert d'orch) : `cargo test --profile headless -p scenario --test
  scenarios --no-run`, 6 min, `CARGO_BUILD_JOBS=2`.

## Vérifié

- Rejeu : 39/39 défaites reproduites à l'identique (frames, `run_end`, `floor_frames`,
  `damage_taken`).
- Scénarios figés `throne_defaite_tireurs` (103), `throne_defaite_boss` (124),
  `throne_defaite_coequipier` (131) : toutes les attentes vertes ; seule manque la trace de
  référence (`pas de trace de référence`), à bénir par l'orchestrateur. Placés dans
  `tests/scenarios/` (le seul dossier que lit la suite) et non `games/throne/assets/scenarios/`
  (inexistant). Joués sur la base de la branche, **pas** rejoués sur `main` après merge (pas de
  feu vert de compilation). De `61ac539` à `f80b82b`, ce qui touche la simulation de throne :
  `crates/bots` (sans effet, inputs enregistrés), l'assembleur `map::generation::imp::basic`
  (cartes à gabarits, pas les cavernes), `world::CaveConfig::transit` (sérialisé seulement s'il
  est vrai) et des commentaires : ils devraient rester verts ; à confirmer au bless. Durée : 30 à 37 s chacun en headless.

## Non fait

- 4 bots (run arrêté par la mémoire).
- Vidéos (aucune nécessaire : les causes se lisent dans les extraits).
- Source de la première mise à terre des 27 défaites non rejouées en détail.

## Dettes

- `alacod-sim` sans subscriber de log (proposé dans le digest).
- La trace détaillée tient entière en mémoire dans le runner (plusieurs Go pour 3 000 frames de
  `throne`) : deux rejeux à la fois saturent une machine de 15 Go.
