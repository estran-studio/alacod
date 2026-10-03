use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkProjectHandle;
use bevy_ecs_ldtk::prelude::RawLevelAccessor;
use bevy_ecs_ldtk::LevelIid;
use bevy_ecs_ldtk::{assets::LdtkProject, LevelSet};

// should be a step before the game part
//
// T1.8 : un `LevelSet` par monde LDtk (mode `Floors` : un monde par niveau de la séquence),
// rempli avec les niveaux du projet de *ce* monde. Avec un seul monde, identique à l'ancien
// comportement (premier projet chargé, dernier `LevelSet`).
pub fn load_levels_if_not_present(
    ldtk_project: Res<Assets<LdtkProject>>,
    mut worlds: Query<(&LdtkProjectHandle, &mut LevelSet)>,
) {
    if ldtk_project.is_empty() {
        return;
    }
    for (handle, mut level_set) in worlds.iter_mut() {
        let Some(ldtk_project) = ldtk_project.get(handle) else {
            continue;
        };
        let level_iids: Vec<_> = ldtk_project
            .data()
            .iter_raw_levels()
            .map(|l| l.iid.clone())
            .collect();

        if !level_set.iids.is_empty() {
            let mut clear = false;
            for iid in level_set.iids.iter() {
                if !level_iids.iter().any(|x| iid.to_string() == *x) {
                    clear = true;
                    break;
                }
            }
            if clear {
                level_set.iids.clear();
            }
        }

        level_iids.iter().for_each(|id| {
            level_set.iids.insert(LevelIid::new(id));
        });
    }
}
