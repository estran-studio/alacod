# m2-t7-boss-arene — boss à phases jouable et arène (M2, V1c, chantier D4)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §37 (contrats de boss,
`boss_phase_system`, frise), §35 (salles typées et verrouillées), §41 (patterns v2 : `Spiral`,
`Fan`, `Burst`), §40 (étages assemblés, salle `boss` au bout), §23 (horloges) et le rapport
`m2-t0c-contrats-boss-profil`. Branche `m2-t7-boss-arene` depuis `main`. Estimation : 4 j. Section
de conventions réservée : **§45**.

## Contexte

M2-T0c a posé les phases (`HealthBelow`, `AfterFrames`), la frise (`SpawnPattern`) et `BossState` ;
le boss de `gungeon` (squelette, M2-T1) tire des patterns v2. Il manque : une salle de boss qui se
verrouille et se rouvre à la mort, des règles de tir **par phase** (limite notée en §37 : `ai.ranged`
fixé à la config), une arène à tenir (horloge, vagues internes) et la preuve qu'un bot bat le boss.

## À faire

1. **Tir par phase** : une règle `Shoot` d'une phase utilise sa propre table de tir (lever la limite
   §37) ; sans règle de phase, comportement actuel exact.
2. **Salle de boss** : à l'entrée, la salle `boss` se verrouille (§35) et le boss s'active (dormant
   avant) ; à sa mort, déverrouillage, `RoomChanged` → `Cleared`, portail/sortie d'étage (§40)
   seulement après. `FrameEvents<BossDefeated>` (neutre) pour la suite (butin M2-T5, effets).
3. **Arène à tenir** : variante de salle `Arena` (donnée) : horloge (§23) de `n` frames, vagues
   internes (liste d'ennemis par vague, en donnée), victoire à la fin de l'horloge si le joueur vit.
4. **Actions de frise** : `SpawnEnemies(id, count)` (renforts, positions seedées par le flux RNG de
   la salle) et `Telegraph(frames)` avant un `SpawnPattern` (état dérivé, pas de nouveau type si
   possible). Nouvelles variantes **en fin d'enum**.
5. **Lint** : arène sans vague, horloge 0, ennemi inconnu dans `SpawnEnemies`, `count` 0 → refusés ;
   fixtures.
6. **Contenu `gungeon`** : le boss squelette en trois phases (patterns v2 différents, renforts en
   phase 2), une salle `Arena` dans la grammaire d'étage `etage` si elle reste valide sur 1 000
   graines (test de M2-T10).
7. **Scénarios** : `boss_room_lock` (verrou à l'entrée, réouverture à la mort), `boss_phase_shoot`
   (tir propre à la phase), `boss_reinforcements` (`EntityCount`), `arena_survive`, `arena_fail`,
   et un scénario `gungeon_boss_bot` où un bot (`bot:` existant) bat le boss (attente `BossPhase`
   puis mort, `RoomState` `Cleared`).

## Règles

Aucune trace existante ne doit bouger hors `gungeon` (nouveaux types en variante **neutre**,
variantes en fin d'enum, champs optionnels hors hash quand absents : voir `Effect` en §42). Les
traces `gungeon` qui bougent : preuve §5 (`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`),
cause dans le commit, avant tout bless. Pas de `f32`, `order_iter!`, `FrameEvents` (jamais
`Message`), `despawn_rollback()`. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Butin du boss (M2-T5), ralenti et feedback de fin de boss (M2-T19), HUD du boss (M2-T18).

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ
toutes les deux heures et tue les processus d'arrière-plan : lancer la suite au premier plan) et finir par un commit
`LIVRÉ m2-t7-boss-arene` qui contient le rapport. p2p : `make test_multiplayer` si docker et un
affichage sont disponibles, sinon le dire au rapport (l'orchestrateur le fera).

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ; rapport
`docs/taches/rapports/m2-t7-boss-arene.md` (ce qui est fait, choix, traces bougées et preuve, ce qui
reste), conventions §45.
