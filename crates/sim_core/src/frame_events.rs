//! Événements internes à une frame de simulation GGRS.
//!
//! Les `Message` bevy ne doivent pas servir dans le `GgrsSchedule` : ils ne sont pas
//! sauvegardés dans les snapshots et chaque lecteur garde un curseur hors rollback.
//! Un événement écrit dans une timeline annulée par un rollback peut donc être perdu
//! ou appliqué deux fois.
//!
//! [`FrameEvents`] est une simple file, enregistrée pour le rollback et vidée au début
//! de chaque frame de simulation ([`RollbackSystemSet::FrameStart`]). Un événement n'est
//! visible que dans la frame où il a été émis, par les systèmes ordonnés après
//! son émetteur.
//!
//! Déménagé de `crates/game/src/frame_events.rs` en T0.2 (`docs/taches.md`) pour que les
//! futurs crates de vocabulaire (`combat`, `effects`, `behaviors`...) puissent l'utiliser
//! sans dépendre de `game`. `crates/game/src/frame_events.rs` réexporte ce module.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use utils::rollback::RollbackTraceApp;

use crate::system_set::RollbackSystemSet;

/// File d'événements produite et consommée dans la même frame GGRS.
///
/// Les `Entity` contenues ne sont valides que pendant la frame d'émission : un type `T`
/// qui en porte doit implémenter `Hash` manuellement en les excluant (voir
/// `InteractionEvent` dans `crates/game/src/interaction.rs`). `Vec<T>::hash` hache déjà
/// la longueur puis chaque élément : le `#[derive(Hash)]` ci-dessous fait exactement ce
/// qu'un hash manuel ferait.
#[derive(Resource, Clone, Debug, Hash, PartialEq)]
pub struct FrameEvents<T: Clone + Send + Sync + 'static>(Vec<T>);

impl<T: Clone + Send + Sync + 'static> Default for FrameEvents<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: Clone + Send + Sync + 'static> FrameEvents<T> {
    pub fn send(&mut self, event: T) {
        self.0.push(event);
    }

    /// Événements dans leur ordre d'émission.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub trait FrameEventsAppExt {
    /// Enregistre une file [`FrameEvents<T>`] pour le rollback et son nettoyage par frame.
    fn add_frame_events<T>(&mut self) -> &mut Self
    where
        T: Clone + Send + Sync + std::hash::Hash + std::fmt::Debug + 'static;

    /// Comme [`Self::add_frame_events`], mais la file **vide** contribue `0` au checksum GGRS
    /// (`rollback_and_trace_resource_neutral`) : une file nouvelle qui reste vide dans toutes
    /// les parties existantes ne déplace aucune trace (T1.6 : demandes de destruction de
    /// terrain, émises seulement dans une caverne).
    fn add_frame_events_neutral<T>(&mut self) -> &mut Self
    where
        T: Clone + Send + Sync + PartialEq + std::hash::Hash + std::fmt::Debug + 'static;
}

impl FrameEventsAppExt for App {
    fn add_frame_events<T>(&mut self) -> &mut Self
    where
        T: Clone + Send + Sync + std::hash::Hash + std::fmt::Debug + 'static,
    {
        self.init_resource::<FrameEvents<T>>()
            .rollback_and_trace_resource::<FrameEvents<T>>()
            .add_systems(
                GgrsSchedule,
                clear_frame_events::<T>.in_set(RollbackSystemSet::FrameStart),
            )
    }

    fn add_frame_events_neutral<T>(&mut self) -> &mut Self
    where
        T: Clone + Send + Sync + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    {
        self.init_resource::<FrameEvents<T>>()
            .rollback_and_trace_resource_neutral::<FrameEvents<T>>()
            .add_systems(
                GgrsSchedule,
                clear_frame_events::<T>.in_set(RollbackSystemSet::FrameStart),
            )
    }
}

fn clear_frame_events<T: Clone + Send + Sync + 'static>(mut events: ResMut<FrameEvents<T>>) {
    events.0.clear();
}
