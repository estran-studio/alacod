//! Équipe d'une entité (joueur, ennemi, allié, neutre).
//!
//! # Composant statique, hors rollback
//!
//! `Team` est posé une fois à la création de l'entité (`character/player/create.rs`,
//! `character/enemy/create.rs`) et n'est modifié par aucun système en T0.2. Il n'est donc
//! **pas** enregistré via `utils::rollback::RollbackTraceApp` : l'ajouter au rollback (et
//! donc au checksum GGRS comparé par le synctest et les traces de référence) changerait
//! `tests/scenarios/*.trace` sans aucune raison de gameplay.
//!
//! Un composant hors rollback n'est jamais restauré par un rollback GGRS (il n'est pas
//! dans l'instantané), mais ça ne le rend pas dangereux ici : tant que l'entité qui le
//! porte n'est jamais détruite avec `despawn()` (interdit dans `GgrsSchedule`, CLAUDE.md
//! règle 9) mais toujours avec `despawn_rollback()`, l'entité — et donc `Team` avec elle —
//! survit intacte à une éventuelle résurrection après rollback (`RollbackDespawnPlugin`
//! ne détruit réellement l'entité qu'une fois la frame confirmée). `Team` reste donc
//! cohérent sur tous les clients sans être dans l'instantané.
//!
//! Le jour où `Team` devient mutable (capture, trahison, chantier D3 « résolutions
//! alternatives »), il faudra l'enregistrer avec `app.rollback_and_trace::<Team>()` et
//! blesser (`BLESS=1`) toutes les traces de `tests/scenarios/*.trace`, puisque sa valeur
//! commencera à apparaître dans le checksum comparé entre clients.

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum Team {
    Players,
    Enemies,
    Allies,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_ron() {
        for team in [Team::Players, Team::Enemies, Team::Allies, Team::Neutral] {
            let ron = ron::to_string(&team).expect("sérialisation RON");
            let back: Team = ron::from_str(&ron).expect("désérialisation RON");
            assert_eq!(team, back);
        }
    }
}
