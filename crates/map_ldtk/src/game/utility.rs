use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::RawLevelAccessor;
use bevy_ecs_ldtk::LevelIid;
use bevy_ecs_ldtk::{assets::LdtkProject, LevelSet};

// should be a step before the game part
pub fn load_levels_if_not_present(
    ldtk_project: Res<Assets<LdtkProject>>,
    mut level_set: Query<&mut LevelSet>,
) {
    if ldtk_project.is_empty() {
        return;
    }
    let ids: Vec<_> = ldtk_project.ids().collect();
    let id = ids.first().unwrap();

    let ldtk_project = ldtk_project.get(*id).unwrap();
    let level_iids: Vec<_> = ldtk_project
        .data()
        .iter_raw_levels()
        .map(|l| l.iid.clone())
        .collect();

    let mut level_set = level_set.iter_mut().last().unwrap();
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
