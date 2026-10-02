//! Contrats B3 : état des statuts, rempli et exécuté par T1.3 de M1.
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum StatusDef {
    Burn,
    Slow,
    Stun,
    Freeze,
}

/// Une entrée posée : nombre de piles et frame absolue d'expiration.
/// La politique d'empilement et la borne exacte d'expiration seront fixées par T1.3.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct StatusEntry {
    pub status: StatusDef,
    pub stacks: u32,
    pub expires_at_frame: u32,
}

/// Ordre de pose explicite, sans état caché. Non posé sur les personnages en vague 0.
#[derive(Component, Default, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Statuses(pub Vec<StatusEntry>);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_def_ron_round_trip() {
        let values = vec![
            StatusDef::Burn,
            StatusDef::Slow,
            StatusDef::Stun,
            StatusDef::Freeze,
        ];
        assert_eq!(
            ron::from_str::<Vec<StatusDef>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
}
