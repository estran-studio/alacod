# m2-t0c-contrats-boss-profil — contrats de boss et de profil (M2, vague 0)

Lire d'abord `docs/taches/README.md` (machine `orca`, `docs/taches.md` §7 « Exécution »),
`CLAUDE.md`, `docs/conventions.md` §4 (checklist d'un vocabulaire) et les sections citées.
Branche `m2-t0c-contrats-boss-profil` depuis `main`. Estimation : 2 à 3 j. Plan :
`docs/plan-engine.md` §5 D4 (boss), G1 (profil) ; `docs/taches.md` §7 ; décision §7 n° 8 (format
de sauvegarde : RON versionné, un profil par joueur, pubkey allumette, écrit hors simulation).

## But

Poser les contrats dont dépendent M2-T7 (boss à phases et arène) et M2-T12 (profil et
déblocages), avec un chemin minimal de bout en bout. **Aucune trace existante ne bouge.**

## Décisions (fixées)

1. **Boss (`crates/behaviors` ou nouveau module, simulation seule)** :
   - `BossDef` en RON, dans la définition d'ennemi (champ optionnel `boss`) : `phases: [Phase]`.
   - `Phase { until: HealthBelow(ratio) | AfterFrames(n), behaviors: [Behavior], on_enter: [Action] }`
     : à l'entrée d'une phase, les behaviors de l'ennemi sont remplacés (vocabulaire T1.4
     existant : `Shoot(weapon, pattern…)`, `Chase`, `KeepDistance`…), `on_enter` réutilise les
     actions d'`effects` (ex. invoquer, poser un statut).
   - `Timeline` : liste d'actions datées en frames depuis l'entrée de la phase (`at`, `do`),
     rejouée en boucle ou non (`repeat`).
   - `BossState` (composant rollback, **neutre**) : phase courante, frame d'entrée, curseur de
     timeline. Transitions déterministes (une seule par frame, ordre des phases du RON).
   - `FrameEvents<BossPhaseChanged>` ; moment clé `boss_phase` ; trace
     `ggrs{f=… boss_phase net_id=… phase=…}`.
   - Attente de scénario `BossPhase(entity, n)`.
   - Lint : phase vide, seuil hors ]0, 1], behavior ou action inconnus ; fixtures.
2. **Profil (`crates/meta`, hors simulation)** :
   - `Profile { version, id, currencies: BTreeMap<String, u64>, unlocks: BTreeSet<String>,
     stats: BTreeMap<String, u64> }`, sérialisé en RON versionné (migration par `version`).
   - Écrit **hors simulation** à la fin d'une run à partir de `RunSummary` (système dans
     `Update`/`Last` qui lit l'état, jamais dans `GgrsSchedule`) ; un fichier par joueur local
     (identifiant : pubkey allumette si connue, sinon `local-<handle>`), dossier configurable
     (`ALACOD_PROFILE_DIR`, défaut dans le dossier de données de l'utilisateur ; les scénarios
     n'écrivent rien sauf si demandé).
   - Lecture au lancement (ressource `Profiles`, hors rollback, **jamais** lue par la
     simulation dans cette tâche : M2-T12 décidera comment un déblocage entre dans une run de
     façon déterministe et partagée entre pairs).
   - Attente `ProfileHas(player, unlock)` évaluée après la fin de run du scénario.
3. **Bout en bout minimal (testbed)** : un boss à deux phases (tir `Aimed` puis `Ring` sous 50 %
   de PV) ; scénario où le joueur le fait passer en phase 2 (`BossPhase`), puis le tue ;
   profil écrit en fin de run avec une monnaie méta incrémentée (`ProfileHas` ou lecture du
   fichier dans un test de crate).
4. **Hors périmètre** : arène à tenir, hub, déblocages qui modifient les pools, HUD.

## Vérification

README §4 complet ; p2p à deux clients si la machine le permet (Docker disponible) ; rapport
`docs/taches/rapports/m2-t0c-contrats-boss-profil.md` (§7) ; conventions : sections « Boss »
et « Profil » ; liste des attentes de `CLAUDE.md` mise à jour (ligne M2).
