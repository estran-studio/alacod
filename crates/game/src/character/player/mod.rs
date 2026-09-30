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

/// Hash manuel : seul `handle` contribue au checksum GGRS. `color` est de la présentation
/// (`f32`, pas de `Hash`) ; `name` et `pubkey` sont des données d'identité **propres à
/// chaque client** en p2p (un client ne connaît que son propre nom, les autres joueurs
/// reçoivent un nom de repli) : les hacher faisait diverger les checksums de tous les
/// clients dès la frame 0 (test p2p headless de la CI de nuit, T2.14) sans que la
/// simulation diffère. Elles ne pilotent rien dans la simulation.
impl std::hash::Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}
