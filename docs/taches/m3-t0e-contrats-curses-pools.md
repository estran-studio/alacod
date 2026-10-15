# m3-t0e-contrats-curses-pools — curses et pools d'objets à épuisement (M3, vague 0)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §4, §13 et
§17 (run, `Floors`), §23 (horloges, difficulté), §40 (grammaire, graine par emplacement), §48 (butin : tirage
indépendant du moment, `loot_seed`), §50 (boutique), §51 (déblocages de M2-T12, une fois mergée), et le code
`crates/run/src/` (`Run`, `Run.flags`, `FloorState`), `crates/items/src/loot.rs` (`LootTableDef`,
`draw_loot`, `loot_seed`), `crates/game/src/loot.rs`, `crates/game/src/shop.rs`. Branche
`m3-t0e-contrats-curses-pools` depuis `origin/main`, **une fois M2-T12 mergée** (`git log origin/main`
contient `merge m2-t12-profil-deblocages` ; sinon attendre : même fichier `items::loot`). Estimation : 2 j.
Section de conventions réservée : **§64**.

## But

Poser les contrats dont dépendent M3-T8 (pools et marché du diable), M3-T9 (curses) et M3-T10
(accomplissements). **Aucune trace existante ne bouge.**

## Contexte (état de `main` au 2026-10-15)

- Rien n'existe pour les modificateurs de run (F4) : `Run.flags: BTreeSet<String>` n'est lu ni écrit.
- Un pool (`LootTable`) tire un objet pondéré, seedé par `loot_seed(graine, étage, clé de contenu)` : un
  coffre ouvert tôt ou tard donne le même objet. `exclude_held` (opt-in) retire les objets tenus, au prix
  d'un tirage qui dépend de l'inventaire de l'ouvreur.
- Isaac retire de son pool tout objet **vu** (épuisement). C'est contraire à l'indépendance du moment
  (l'ordre d'ouverture des salles change ce qui reste) : il faut une règle qui garde le déterminisme et,
  si possible, l'indépendance du moment. **Q5** (au rapport) demande si William veut l'épuisement et à
  quelle portée.

## À faire

1. **Curses (contrat)** : kind de contenu `Curse` (`curses/<id>.ron`, id = nom de fichier ; `game.ron` :
   `(path: "curses", kind: "Curse")`), `run::CurseDef { weight, effects: [CurseEffect] }` avec
   `CurseEffect` en liste fermée, extensible en fin d'enum. Variantes proposées (la liste du clone est
   **Q7**, ne pas en coder plus que le contrat) : `HideMinimap`, `HideHealth`, `HideItemNames` (présentation
   seule : lues par le HUD), `FloorRooms(mult)` (lu par la grammaire, M3-T5/T9), `Modifier(stat, op, value)`
   (posé sur chaque joueur pour l'étage, source `curse:<id>`). Tirage : chance par étage dans la séquence
   `Floors` (`curses: Some((chance, pool: [ids]))`, absent = aucune curse), graine
   `fnv1a("curse:<graine>:<étage>")` (RNG local, aucun flux de `RngStreams`), donc indépendante du moment.
   Ressource rollback **neutre** `RunCurses(BTreeMap<u32 étage, String>)`. Contrat : types, lint (curse
   inconnue, chance hors [0, 1], poids 0), règle pure du tirage et ses unitaires ; exécution en T9.
2. **Pools par type de salle** : champ optionnel `pool: Option<String>` d'un type de salle (`RoomKindDef`),
   d'un coffre et d'une boutique (déjà `table`) : rien de nouveau si `table` suffit ; le dire au §64.
3. **Épuisement (contrat, règle bloquée par Q5)** : champ optionnel `exhaust: bool` de `LootTableDef`
   (défaut faux, comportement actuel) et ressource rollback neutre `PoolsSeen(BTreeMap<String pool,
   BTreeSet<String objet>>)`. Proposition technique par défaut, à présenter et non à imposer : les objets
   d'un pool à épuisement sont **tirés à la génération de l'étage** (salles dans l'ordre du plan, graine de
   l'emplacement), ce qui garde l'indépendance du moment pour l'ouverture ; un objet tiré sort du pool pour
   le reste de la run. Écrire au §64 les deux options (tirage à la génération / tirage à l'ouverture avec
   dépendance au moment) et leur coût. Contrat : types, lint, règle pure `draw_exhausting(pool, seen, rng)`
   et ses unitaires (le flux avance de la même façon quel que soit `seen`, comme `draw_loot`).
4. **Accomplissements (contrat)** : kind `Achievement` (`achievements/<id>.ron`) : `when: [Condition sur
   RunSummary]` (fin de run en victoire, étage atteint ≥ n, boss vaincu `id`, objet tenu à la fin) et
   `unlocks: [id d'Unlock de M2-T12]`. Évalué **hors simulation** en fin de run, comme l'écriture du profil.
   Contrat : types, lint (déblocage inconnu), règle pure `achieved(summary, def)` ; exécution en T10.
   `RunSummary` gagne au besoin les champs nécessaires (boss vaincus, objets tenus) en champs optionnels hors
   hash quand vides (son `impl Hash` est déjà manuel).
5. **Attentes** : `Curse(floor, Some(id) | None, at_frame)`, `PoolSeen(pool, item, bool)` ; mettre à jour la
   liste de `CLAUDE.md`.

## Règles

Aucune trace existante ne bouge (ressources neutres, champs optionnels). Les RNG de curse et de pool sont
**locaux** (comme `variant.rs` et `loot_seed`) : créer un flux dans `RngStreams` changerait le hash de la
ressource et toutes les traces. Le profil et les accomplissements restent hors `GgrsSchedule`. Pas de `f32`,
`BTreeMap`.

## Hors périmètre

Curses en jeu et leur effet sur la grammaire et le HUD (M3-T9, T16) ; épuisement en jeu, salle du diable
(M3-T8) ; accomplissements écrits dans le profil (M3-T10).

## Vérification

README §4 complet ; unitaires des trois règles pures ; rapport
`docs/taches/rapports/m3-t0e-contrats-curses-pools.md`, conventions §64.

## Atteignabilité

Contrats : rien en partie ordinaire. T9 tire une curse dans la séquence `Floors` de `isaac` (graine nommée),
T8 active l'épuisement sur le pool trésor, T10 écrit un accomplissement à la fin d'une partie ordinaire.

## Session cloud

Tâche prévue pour une session Claude Code cloud, **après le merge de M2-T12**. Ignorer les consignes `orca`
du README (§1). Pas de Docker : le dire au rapport. Pousser la branche régulièrement. Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ m3-t0e-contrats-curses-pools`,
qui contient le rapport.
