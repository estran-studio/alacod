//! Bots v0 (T2.11) : joueurs artificiels déterministes, pour jouer des parties sans humain
//! (`alacod-sim`) et fixer des scénarios de régression (`tests/scenarios/bots_*.ron`).
//!
//! - [`view`] : [`BotView`], vue en fixed-point de l'état lisible par un joueur (sa position et
//!   sa santé, ses munitions, l'ennemi et la fenêtre les plus proches, la vague). Construite par
//!   [`input::read_bot_inputs`], consommée par [`decide::decide`].
//! - [`decide`] : la fonction pure `decide(profile, view, rng) -> BoxInput` et les profils
//!   `immobile`/`fonceur`/`prudent`. Aucune dépendance à l'ECS : testable hors Bevy.
//! - [`input`] : [`input::BotAssignments`] (ressource, `BTreeMap<handle, BotProfile>`),
//!   [`input::read_bot_inputs`] (système ajouté au schedule `ReadInputs`) et
//!   [`input::BotsPlugin`] qui les enregistre.
//!
//! `BotProfile` lui-même est défini dans `game::replay` (pas ici) : c'est le format de
//! scénario (`PlayerScript::bot`) qui en a besoin, et `game` ne doit pas dépendre de `bots`
//! (`bots` dépend de `game`, `sim_core`, `bevy_fixed`, `utils`, `map` — jamais l'inverse). Ce
//! crate le ré-exporte pour que les appelants écrivent `bots::BotProfile` uniformément.

pub mod decide;
pub mod input;
pub mod view;

pub use decide::decide;
pub use game::replay::BotProfile;
pub use input::{read_bot_inputs, BotAssignments, BotsPlugin};
pub use view::{BotView, EnemyView, WindowView};
