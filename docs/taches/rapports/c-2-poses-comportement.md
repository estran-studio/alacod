# C-2 — Poses des phases de comportement (rapport, squelette)

- **Existait** : `decide_poses` / `ModelPlayback` (`crates/game/src/model3d/playback.rs`), `sim_activity` (animation vivante), lint `MissingPose` (`crates/content/src/lint.rs:1690`), phases dans `BehaviorRuntime` (`crates/game/src/character/enemy/ai/state.rs` : `ChargePhase` l.669, `ReturnPhase` l.686, `CoverPhase` l.697, `BehaviorRuntime` l.716) ; aucune pose pour ces phases.
- **Manquait** : champs de pose, lecture de `BehaviorRuntime` par `decide_poses`, couverture par `animation_vivante`, avertissement de lint, scénarios.
- **Ajouté** : voir `docs/conventions.md` § 131 (4 champs `Poses`, `behavior_phase`, `BehaviorPhase` dans `pose.rs`, lint `reach.rs`, 4 scénarios `pose_*`, 3 fixtures).

## Preuve (à remplir)
- Rouge : (modèles du testbed sans les champs, code déjà écrit) — à lancer.
- Vert, traces neuves, traces existantes inchangées, capture, contrôle rapide : à venir.
