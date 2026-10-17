# Rapport m3-t0e-contrats-curses-pools (b3)

Conventions : `docs/conventions.md` §64. Branche `m3-t0e-contrats-curses-pools`, fusionnée avec `origin/main`
`64ec961c` (m2-full-run) ; un seul conflit (`game/src/run_state.rs`, commentaire de module), résolu en gardant
les deux listes de ressources remises à zéro.

## 1. Fait

Contrats et règles pures, rien d'exécuté en jeu (T8/T9/T10). Décisions de William (2026-10-16) appliquées :
Q5 (épuisement par run et par pool, `PoolState` rollback neutre, tirage = `loot_seed` + index d'épuisement),
Q7 (les cinq curses), Q6 (hors périmètre : T8).

| Point de la fiche | Livré |
|---|---|
| 1. Curses | `run::curse` : `CurseDef { weight, effects }`, `CurseEffect` (`HideMinimap`, `HideHealth`, `HideItemNames`, `FloorRooms`, `Modifier(stat, op, valeur)` + `to_modifier(id)` source `curse:<id>`), `CurseConfig { chance, pool }` (champ `curses` de `Floors`, `FloorsEntry::curses`), `curse_seed` = `fnv1a("curse:<graine>:<étage>")`, `draw_curse` (RNG local, deux tirages toujours consommés), `RunCurses(BTreeMap<u32, String>)` rollback **neutre** (`RunPlugin`), remise à zéro à la relance. Kind `Curse` (`curses/<id>.ron`, registre, schéma), lint `lint_curses`. |
| 2. Pools par type de salle | Aucun champ nouveau : `table` (coffre, boutique, `Floors.loot`) nomme déjà le pool ; dit au §64. |
| 3. Épuisement | `LootTableDef.exhaust` (défaut faux), `items::PoolState` rollback neutre (renommé de `PoolsSeen` de la fiche : il porte l'index d'épuisement, pas seulement « vu »), `draw_exhausting` (`draw_loot_with` la rappelle avec `seen` vide : tirages existants inchangés), `exhaust_seed` (index 0 = `loot_seed`), `PoolState::{seen,has_seen,count,record,next_seed}`. Les deux options A (tirage à la génération, proposée) / B (à l'ouverture) et leur coût sont au §64. |
| 4. Accomplissements | `meta::achievement` : `AchievementDef { when, unlocks }`, `AchievementCondition` (`Victory`, `FloorAtLeast`, `BossDefeated`, `ItemHeld`), `RunFacts`, `achieved`. Kind `Achievement`, schéma, lint. **Écart** : pas de champs ajoutés à `RunSummary` (`Copy`, haché, dans `Run`) : `RunFacts` hors rollback réunit boss vaincus et objets tenus (T10) ; justifié au §64. |
| 5. Attentes | `Curse(floor, Some(id) \| None, at_frame)`, `PoolSeen(pool, item, seen, at_frame)` ; liste de `CLAUDE.md` à jour. |

Contenu `testbed` : 5 curses, séquence `maudit`, pool `epuisable` (`exhaust: true`), accomplissement
`deux_etages` + déblocage `manche_epuisable` (un `Loot` sur `epuisable` seulement). 3 fixtures de lint
(`curse_invalid_values`, `floors_curses_invalid`, `achievement_invalid`) + `crates/content/tests/m3_t0e_fixtures.rs`.
Scénario témoin `curse_pool_unset` (séquence `maudit`, 300 frames, aucune curse, aucun objet sorti).
Test `crates/scenario/tests/curses_pools.rs` : les ressources écrites en cours de partie traversent le synctest,
les attentes les lisent (et échouent avec la valeur observée), la relance les remet à zéro (vérifié par mutation :
sans le reset de `run_state`, le test échoue).

## 2. Vérifié (sur la fusion avec `origin/main` `64ec961c`, HEAD `3f80402b`, avant le commit LIVRÉ)

- `make test_scenarios_par SHARDS=3` sans BLESS : 498 scénarios, 3 tranches ok, 226 s.
- `make test_crates` : 1 044 tests passés, 0 échec (dont unitaires `run::curse` 9, `items::loot` +10, `meta::achievement` 6).
- `make lint` ×5 : zombies, testbed, throne, gungeon, isaac « aucune erreur ».
- `make gen GAME=…` ×5 : code 0, `git status` propre après (aucun diff).
- `cargo fmt --all -- --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `check_sim_boundary`, `make test_tools` : OK.
- Fuzz `ALACOD_FUZZ=0:8` : 1 test passé, code 0.
- **p2p-stress** `testbed`, 3 000 frames, `/tmp/inputs.ron` : OK, **5 327 frames resimulées** (client 0 ; client 1 : 17),
  0 `DesyncDetected`, codes de sortie 0/0, traces identiques (2 999 lignes). Montage **temporaire non commité** (retiré) :
  un système en `FrameStart` écrivait `RunCurses` (étages 0 à 3) et `PoolState` (`epuisable`) toutes les 5 frames
  d'après la position du joueur (donc ses entrées : une prédiction fausse écrit autre chose). Docker OrbStack,
  `MATCHBOX_PORT=3576`, `COMPOSE_PROJECT_NAME=b3`. Les traces ne contiennent que des hash : je n'ai pas relu les valeurs
  des ressources dans la trace (elles sont écrites par construction, système exécuté).

## 3. Traces bougées

**Aucune.** Aucun `.trace` existant modifié (les 498 scénarios verts sans BLESS). Seule trace ajoutée : `curse_pool_unset`.
Preuve §5 du témoin : `trace-diff.py` entre `portal_next_floor` et `curse_pool_unset` (mêmes 300 frames) avec
`--ignore Run` n'a donné de différence qu'à la longueur (299 frames vs 2 600) — la chaîne `Run.mode` (`Floors { config }`)
diffère, c'est le seul écart attendu ; le comparateur s'est arrêté sur la longueur, sans ligne divergente avant.
(La comparaison directe `cmp` des `.trace` diffère dès la ligne 1 pour la même raison : le hash du mode.)

## 4. Non fait / à relire

- Rien n'écrit `RunCurses` ni `PoolState` en jeu : un scénario `.ron` ne peut poser que `None` / `seen: false`
  (T8/T9). Atteignabilité : T9 tire une curse dans `Floors` d'`isaac`, T8 active `exhaust`, T10 écrit un accomplissement.
- Relecture indépendante : non faite par moi (attendue de l'orchestrateur). Points à regarder :
  1. `draw_exhausting` : ordre des passes d'assouplissement (rareté/exclusion avant répétition d'un objet vu ; pool
     entièrement vu repart complet) — choix de ma part, §64.
  2. Écart `RunFacts` vs champs de `RunSummary`.
  3. Nom `PoolState` à la place de `PoolsSeen` (fiche) — arbitrage Q5.
  4. `Run.mode` haché dans la trace du témoin : preuve §5 relative.
- Journal `docs/taches.md` §10 : à ajouter sur `main` après le merge (règle du README §6) ; proposition :
  `| 2026-10-16 | M3-T0e (curses, pools à épuisement, accomplissements : contrats et règles pures ; RunCurses/PoolState rollback neutres, kinds Curse/Achievement, attentes Curse/PoolSeen, §64) | m3-t0e-contrats-curses-pools | b3 | mergée ; 498 scénarios sans trace bougée, test_crates 1 044, lint/gen ×5, fuzz, p2p-stress testbed montage 5 327 frames resimulées ; non fait : exécution (T8/T9/T10) |`
