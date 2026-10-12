# M2-T10 — Grammaire d'étage

Branche `m2-t10-grammaire-etage`. Conventions : `docs/conventions.md` §40.

## Étape 1 (livrée, mergée c3c32183)

- `MapGenerationMode::Floor(FloorGrammar)` ; `Basic` intact.
- `world::FloorGrammar` (données), `map::generation::floor` (planificateur, `validate_plan`),
  `FloorMapGeneration` (rejoue le plan par `get_next_room`).
- Champ `room_kind` dans `AvailableLevel` (lu depuis LDtk).
- Kind de contenu `FloorGrammar`, désignation `floor:<id>`, lint + 2 fixtures.
- 1 000 graines : moyenne 183,5 µs, max 11,8 ms, 1,05 tentative en moyenne ; tailles 8:189, 9:204,
  10:202, 11:198, 12:207. Chaque plan passe `validate_plan`.

## Étape 2

- **Gabarits** : 9 gabarits gungeon (`maps/salles.ldtk`), script `m2-t10-grammaire-etage.make_salles.py`.
  Test `gungeon_etages_de_grammaire` : 200 graines × grammaires `etage` (8–12) et `grand` (15–20), plus
  `court` (3 salles, 400 graines dont les graines dérivées du 2ᵉ emplacement) : bornes, un boss, aucun
  chevauchement.
- **`Floors` d'étages assemblés** : `floor://` (un chemin d'asset par étage), graine dérivée par
  emplacement, ouvertures libres murées, séquences `etages_*`.
- **Deux défauts de `Floors` trouvés par `gungeon_two_floors`** (invisibles tant qu'un seul étage avait
  des salles typées) : (1) les salles de l'étage 2, déjà chargées et superposées à celles de l'étage 1, se
  verrouillaient et se vidaient en même temps ; (2) les portes de l'étage 2 naissaient fermées et le
  joueur restait bloqué. Corrigés : `rooms::CurrentRooms` (salles de l'emplacement courant) et
  `RoomStates::doors_opened` remis à faux à la transition.
- **Piège** : `entry.mode: Floors` prend la première séquence par ordre d'id. Mes premiers noms
  (`deux_etages`, `grammaire`) passaient avant `etage_1` et changeaient tous les scénarios gungeon
  (22 ennemis au lieu de 3). Renommées `etages_*`.
- **Scénarios** : `gungeon_floor_seed_{1,23,123}` (étage `court`, boss atteint par script, victoire à
  f645), `gungeon_two_floors` (portail f645, étage 2 à f752, victoire f1425, `FloorIndex`),
  `bench_floor` (étage `grand`, deux bots `fonceur`).
- **Atteinte du boss scriptée.** Sur l'étage `etage` complet, les bots `prudent` nettoient une salle puis
  s'arrêtent ; les `fonceur` nettoient cinq salles en 1 168 frames mais ne vont jamais vers le boss
  (dormant) en 6 000 frames. Sur `grand`, ils y arrivent (boss tué à f2549). Le bot explorateur est T16,
  hors périmètre.

## Mesures (machine partagée, profil headless)

- `bench_floor` : 18,5 sim fps, 3 000 frames en 163 s, 282 entités et 37 ennemis au plus ; 12
  verrouillages et ouvertures de salle (reconstruction du champ de flux à chaque porte) sans à-coup
  visible dans les fps ; plancher `tests/budgets.ron` : 9.
- `gungeon_floor_seed_1` (4 ennemis) : 30 sim fps sur la même machine.

## Vérifications

- `make test_scenarios` complet (profil headless) : vert, **aucune trace existante modifiée**
  (zombies, testbed, throne, `gungeon_start/clear/boss` inchangés) ; nouvelles traces bénies :
  `gungeon_floor_seed_{1,23,123}`, `gungeon_two_floors`, `bench_floor`.
- `cargo test --profile headless` sur run, combat, game, content, map_ldtk, sim_core, stats, bots,
  effects, world, map : vert (dont `gungeon_etages_de_grammaire`, 1 000 graines).
- `make lint` ×4 jeux (zombies, testbed, throne, gungeon) : sans erreur ; `cargo fmt --all -- --check` propre.
- p2p à deux clients sur `gungeon` (signaling local, port 3556), 900 frames : traces identiques
  (899 lignes), aucun desync. Le chemin `Floors` + grammaire n'a pas d'option de ligne de commande
  (la séquence vient du scénario, `FloorsOverride`) : il est vérifié en synctest (rollback) par les
  scénarios `gungeon_floor_seed_*`/`gungeon_two_floors`/`bench_floor`, pas en p2p réel.
- Piège rappelé : `entry.mode: Floors` prend la **première** séquence par ordre d'id ; les séquences
  de la grammaire sont nommées `etages_*` pour que `etage_1` (T0d) reste la défaut. Le bot
  explorateur (atteindre le boss) est T16, hors périmètre.
