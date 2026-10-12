use self::{basic::BasicMapGeneration, floor::FloorMapGeneration};

use super::{config::MapGenerationMode, context::MapGenerationContext, IMapGeneration};

mod basic;
mod floor;

pub fn get_implementation(context: MapGenerationContext) -> Box<dyn IMapGeneration> {
    match context.config.mode {
        MapGenerationMode::Basic => Box::new(BasicMapGeneration::create(context)),
        // M2-T10 : étage assemblé par une grammaire (les autres éléments — portes, fenêtres,
        // apparitions — viennent de la même logique que `Basic`).
        MapGenerationMode::Floor(_) => Box::new(FloorMapGeneration::create(context)),
        // Pas d'assemblage de salles : `map_ldtk::generation::cave` écrit le niveau
        // directement dans le gabarit, sans passer par `map_generation`
        MapGenerationMode::Cave(_) => {
            unreachable!("une caverne est construite par map_ldtk::generation::cave")
        }
    }
}
