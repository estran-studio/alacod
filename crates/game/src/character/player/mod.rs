pub mod control;
pub mod create;
pub mod input;
pub mod jjrs;

use bevy::prelude::*;
use ggrs::PlayerHandle;

#[derive(Component, Reflect, Default, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct LocalPlayer {}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Player {
    pub handle: PlayerHandle,
    pub color: Color,
    pub name: String,
    pub pubkey: String,
}

/// Hash manuel : exclut `color` (présentation, `Color` est `f32` et n'implémente pas
/// `Hash`) ; `handle`/`name`/`pubkey` identifient le joueur et contribuent au checksum.
impl std::hash::Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
        self.name.hash(state);
        self.pubkey.hash(state);
    }
}
