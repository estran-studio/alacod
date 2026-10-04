use self::basic::BasicMapGeneration;

use super::{config::MapGenerationMode, context::MapGenerationContext, IMapGeneration};

mod basic;

pub fn get_implementation(context: MapGenerationContext) -> Box<dyn IMapGeneration> {
    match context.config.mode {
        MapGenerationMode::Basic => Box::new(BasicMapGeneration::create(context)),
        // Pas d'assemblage de salles : `map_ldtk::generation::cave` écrit le niveau
        // directement dans le gabarit, sans passer par `map_generation`
        MapGenerationMode::Cave(_) => {
            unreachable!("une caverne est construite par map_ldtk::generation::cave")
        }
    }
}
