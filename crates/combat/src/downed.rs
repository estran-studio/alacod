//! État « à terre » et réanimation (T1.3, chantier B6 « Esquive et capacités », partie
//! « à terre »). Contrats seulement (composants, constantes) : les systèmes qui les
//! posent/consomment vivent dans `game` (`character::health`, `interaction`,
//! `character::player::input`), qui a accès à `Player`/`GgrsNetId`/`CharacterConfig` — ce
//! crate ne dépend pas de `game` (voir la doc du crate parent).
//!
//! **Décision (à documenter aussi dans le rapport de la tâche)** : à terre, un joueur reste
//! *vulnérable* en théorie (aucun champ `invulnerable_until_frame` posé), mais en pratique
//! `Health` n'est plus jamais modifiée : `character::health::rollback_resolve_damage_events`
//! ignore tout `DamageEvent` dont la cible porte [`Downed`], et les ennemis ne le ciblent
//! plus tant qu'un autre joueur est debout (`character::enemy::ai::{navigation, pathing,
//! behavior}`). C'est l'option la plus simple des deux envisagées (l'autre : remonter
//! `Health.current` à 1 et poser une invulnérabilité temporaire) et elle ne demande pas de
//! nouvelle exception dans `resolve_damage`.
//!
//! **T2.4, chantier F1** : l'ancienne ressource `RunOutcome` (issue de la partie couplée à
//! l'état « à terre ») a disparu au profit de `run::run::Run::step`
//! (`RunStep::Ended { outcome: RunEnd::Defeat, .. }`, posé par
//! `character::health::rollback_check_defeat`, toujours dans `game` : ce crate ne dépend
//! toujours pas de `run`) — voir `docs/conventions.md` section « Run ».

use bevy::prelude::Component;
use bevy_fixed::fixed_math::{self, Fixed};
use serde::{Deserialize, Serialize};
use sim_core::modifier::ModifierSource;
use sim_core::wisp::{WispDef, WispLure};
use utils::net_id::GgrsNetId;

/// Un joueur à terre. Posé par `character::health::rollback_apply_accumulated_damage`
/// quand la santé d'un **joueur** tombe à 0 alors qu'au moins un autre joueur est encore
/// debout (ni mort, ni à terre) — sinon il meurt comme avant T1.3 (`Death`). Retiré par la
/// réanimation complète (`interaction::handle_revive_interaction`) ou remplacé par `Death`
/// au bout de `bleedout_at_frame` sans réanimation (`character::health::rollback_apply_bleedout`,
/// « saigne à mort »).
///
/// Rollback + trace (`RollbackTraceApp::rollback_and_trace::<Downed>()`, `game::character::mod`) :
/// entre dans le `Checksum` GGRS et les traces d'état, comme tout composant qui influence la
/// simulation.
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Downed {
    /// Frame à laquelle ce joueur est tombé à terre.
    pub since_frame: u32,
    /// Frame à laquelle il meurt de saignement (`Death`) s'il n'est pas réanimé avant
    /// (`since_frame + CharacterConfig::bleedout_frames`, résolu une fois à la frame où il
    /// tombe : un changement de config en cours de partie ne le déplace pas).
    pub bleedout_at_frame: u32,
}

/// Réanimation en cours : un joueur debout maintient l'interaction sur un joueur à terre.
/// Ajouté au premier `InteractionEvent { interaction_type: Revive }` reçu pour ce joueur à
/// terre, retiré dès qu'une frame passe sans un tel événement (bouton relâché ou hors de
/// portée — `progress_frames` retombe donc à 0 : revenir réanimer recommence de zéro) — voir
/// `interaction::handle_revive_interaction`.
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Reviving {
    /// Joueur qui réanime (peut changer d'une frame à l'autre si un autre joueur prend le
    /// relais ; la progression n'est jamais perdue pour autant, seulement gelée pendant
    /// une frame sans interacteur).
    pub by: GgrsNetId,
    /// Frames d'interaction maintenue accumulées. À `CharacterConfig::revive_frames`
    /// (résolu sur le joueur à terre, pas sur celui qui réanime) : réanimation complète.
    pub progress_frames: u32,
}

/// Feu follet (M-5, `docs/conventions.md` §113) : posé **avec** `Downed` quand le personnage tombé
/// porte `wisp: Some(..)` (`sim_core::wisp::WispDef`), retiré avec lui. **Aucun champ n'est ajouté à
/// `Downed`** : son hash, donc les traces de tout jeu sans feu follet, ne bouge pas. Rollback +
/// trace en variante **neutre** (`RollbackTraceApp::rollback_and_trace_neutral`) : sans porteur,
/// contribution nulle au checksum.
///
/// Copie résolue de la donnée à la chute (un rechargement du contenu en cours de partie ne change
/// pas un feu follet déjà tombé), pour que les systèmes de ciblage des ennemis n'aient pas besoin
/// de `CharacterConfigs`.
#[derive(Component, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wisp {
    /// Leurre des ennemis (`None` : ignoré d'eux).
    pub lure: Option<WispLure>,
    /// Frames à passer dans la portée d'un point de reprise pour se relever (`None` : pas de reprise).
    pub respawn_frames: Option<u32>,
    /// Fraction de `Health.max` rendue à la reprise.
    pub respawn_health: Fixed,
    /// Frames consécutives passées dans la portée d'un point de reprise (retombe à 0 hors portée).
    pub respawn_progress: u32,
}

impl Wisp {
    /// L'état initial d'un feu follet qui vient de tomber.
    pub fn from_def(def: &WispDef) -> Self {
        Self {
            lure: def.lure,
            respawn_frames: def.respawn_frames,
            respawn_health: def.respawn_health,
            respawn_progress: 0,
        }
    }
}

/// Un feu follet leurre-t-il les ennemis ? Prend l'`Option<&Wisp>` d'un joueur (les requêtes des
/// systèmes de ciblage lisent `Has<Downed>` et `Option<&Wisp>`) ; `false` sans `Wisp`.
pub fn is_decoy(wisp: Option<&Wisp>) -> bool {
    wisp.is_some_and(|wisp| wisp.lure.is_some())
}

/// Point de reprise (M-5, §113) : un feu follet qui y reste `Wisp::respawn_frames` frames se relève
/// (« le prochain camp, le prochain feu »). Posé par la carte (entité LDtk `RespawnPoint`) ou par un
/// placement de contenu (`content::sandbox::RespawnPlacement`). Immobile : rollback neutre, comme
/// les autres entités de carte (`game::hub::HubStation`).
#[derive(Component, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RespawnPoint {
    /// Distance (unités monde) sous laquelle un feu follet « est au point ».
    pub range: Fixed,
}

/// Source du modificateur de vitesse posé pendant qu'un joueur est à terre (`StatId::MoveSpeed`,
/// `ModifierOp::Mul`, valeur `CharacterConfig::downed_speed_mult`) : `Modifiers::push_from` à la
/// pose, `Modifiers::remove_by_source` à la réanimation ou à la mort (T1.2, `sim_core::modifier`).
/// Fonction plutôt que constante : `ModifierSource::Named` porte une `String`, pas de `const`
/// `String` possible en Rust stable.
pub fn downed_modifier_source() -> ModifierSource {
    ModifierSource::Named("downed".to_string())
}

/// Portée de réanimation : distance sous laquelle un joueur debout peut réanimer un
/// coéquipier à terre (`interaction::Interactable` posé sur le joueur à terre avec cette
/// portée, `InteractionType::Revive`).
///
/// **Décision** (constante plutôt qu'une stat, documentée dans le rapport de la tâche) :
/// aucun levier de gameplay identifié dans ce chantier pour la faire varier par personnage
/// ou par objet — une future stat `ReviveRange` resterait un changement compatible (défaut
/// = cette constante).
pub fn revive_range() -> Fixed {
    fixed_math::new(40.0)
}

/// Fraction de `Health.max` restaurée à la fin d'une réanimation.
///
/// **Décision** (constante plutôt qu'une stat, documentée dans le rapport de la tâche) :
/// simple multiplicateur fixe, même raisonnement que [`revive_range`].
pub fn revive_health_fraction() -> Fixed {
    fixed_math::new(0.3)
}
