# m1-dettes-lot-1 — lot de dettes M1 : D31, D33, D35, D37, kiter dans le mur

Lire d'abord `docs/taches/README.md` (agent **local**, branche `m1-dettes-lot-1` créée depuis la tête
livrée de la tâche précédente de l'agent ou `origin/main`, même target). Petite tâche (1 à 2 j),
hors voie : quatre dettes indépendantes et un constat de T1.13. **Un commit par dette**, message
`Dxx : …`.

## Décisions (fixées ; l'agent amende en dix lignes s'il voit un problème)

1. **D31 — panic à la fermeture de la fenêtre** : `camera_control_system` et `player_indicator_system`
   (`crates/game/src/camera/mod.rs`) : remplacer `.single().unwrap()` par `let Ok(window) = q.single()
   else { return };`. Présentation seule, aucune trace. Vérification : `cargo build -p zombies`
   (profil headless, features par défaut) compile ; dire dans le rapport que la fermeture n'a pas
   été testée à l'écran si aucun affichage.
2. **D33 — `scripts/check-forbidden.sh` sous bash 3.2** : réécrire sans `declare -A` (tableaux
   indexés parallèles ou boucle sur une liste `motif|message`), même sortie, mêmes codes de retour,
   `--strict` conservé ; tester avec `bash --posix` si bash 3.2 n'est pas disponible, et le dire.
3. **D35 — `make lint` compile le rendu** : cible `lint` du Makefile (et `check`) avec
   `--no-default-features` sur `-p content --bin alacod` **si et seulement si** la sortie du lint est
   identique sur les deux jeux (comparer les sorties avant/après, les coller dans le rapport) ;
   sinon, expliquer pourquoi et laisser.
4. **D37 — follower de `floor_b` dans le mur** : remonter le `follower` de (6, 10) à (6, 9) dans
   `games/testbed/assets/testbed/floor_b.ldtk` (édition du JSON, pas de régénération). La trace de
   `portal_next_floor` change : **preuve §10** (dump main contre branche, `scripts/trace-diff.py`,
   différence attendue = position et mouvement du follower seulement), le bless est à
   l'orchestrateur. `trois_niveaux` garde `floor_d` ; dire si `floor_d` peut alors disparaître (non :
   le garder, deux niveaux valides ne nuisent pas).
5. **Kiter immobile recule dans un mur** (T1.13 : `enemy_kiter_still`, f544, `EnemyNeverInWall` borné
   à f540) : diagnostiquer `KeepDistance` (recul sans test de collision ou de flow field ?). Si la
   correction est locale au behavior et ne change aucune trace existante (aucun ennemi `KeepDistance`
   dans zombies ; `enemy_keep_distance` du testbed changera → preuve §10 et bless orchestrateur),
   la faire et étendre l'attente à `frames` ; sinon, dette D39 décrite précisément.
6. **Hors périmètre** : D28, D29, D30, D34, D36, D38 (décisions de William), restart p2p (D14).

## Traces attendues

Changent : `portal_next_floor` (D37) et peut-être `enemy_keep_distance`/`enemy_kiter_still` (5),
chacune avec sa preuve. Toutes les autres identiques (`make test_scenarios` vert sans bless ailleurs).

## Règles

Deux compilations au plus ; purge + point d'état après chaque suite (README §1) ; aucune trace bénie
par l'agent ; `docs/taches/dettes.md` : **ne pas toucher** (l'orchestrateur ferme les dettes) ;
`docs/conventions.md` : rien sauf une ligne au §24 si le kiter est corrigé. Merger `origin/main`
juste avant de livrer.

## Critères d'acceptation

1. Un commit par dette ; D31, D33, D35 vérifiées comme décrit ; D37 avec preuve ; point 5 corrigé
   ou D39 rédigée.
2. Tous les scénarios verts, traces modifiées = celles annoncées, avec preuve.
3. Suite des crates verte, `make lint` (trois jeux), `make fmt`, scripts (le nouveau
   `check-forbidden.sh` compris), `make gen` sans modification ; exemples compilés.
4. Rapport honnête avec point d'état.

## Livrer

Rapport `docs/taches/rapports/m1-dettes-lot-1.md`. `git push -u origin m1-dettes-lot-1`, puis
`SendMessage` à `orch` : `LIVRÉ m1-dettes-lot-1 <sha> : <une ligne>`. Ne merge pas, ne bénis pas.
