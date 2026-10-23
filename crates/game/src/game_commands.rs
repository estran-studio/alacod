//! Commandes d'édition appliquées dans la simulation à une frame déterminée.
use std::collections::BTreeMap;

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::Fixed;
use sim_core::team::Team;

#[derive(Debug, Clone, PartialEq)]
pub enum GameCommand {
    SpawnCharacter {
        character: String,
        x: Fixed,
        y: Fixed,
        team: Option<Team>,
        variant: Option<String>,
    },
    SpawnPowerUp {
        id: String,
        x: Fixed,
        y: Fixed,
    },
    SpawnItem {
        id: String,
        x: Fixed,
        y: Fixed,
    },
}

/// File non rollback : elle est indexée, jamais dépilée, afin de rester identique en resimulation.
#[derive(Resource, Default, Clone, Debug)]
pub struct GameCommands(pub BTreeMap<u32, Vec<GameCommand>>);
