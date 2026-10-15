//! Écran de mutation en `bevy_hui` (D71, `docs/conventions.md` §56) : adaptateur entre
//! [`MutationScreenView`] et le gabarit `ui/mutation_screen.html`.
//!
//! Présentation seule. Le choix reste celui de la simulation : valider émet le bit
//! `ChoiceA/B/C` dans l'input (`mutation_screen::confirm_input`), ce module ne lit que la vue.
//! L'écran est dérivé de l'état : un rollback qui annule le choix le fait disparaître.
#![cfg(feature = "ui_hui")]

use bevy::prelude::*;
use bevy_hui::prelude::*;
use content::registry::Registry;

use super::mutation_screen::{compact_description, MutationScreenView};
use crate::core::AppState;

const TEMPLATE: &str = "ui/mutation_screen.html";
const TEXT: &str = "#ffffff";
const FOCUS: &str = "#f1c40f";
const IDLE_BORDER: &str = "#ffffff4d";

pub struct MutationHuiPlugin;

impl Plugin for MutationHuiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HuiPlugin>() {
            app.add_plugins(HuiPlugin);
        }
        app.init_resource::<MutationTemplate>()
            .add_systems(OnEnter(AppState::InGame), load_template)
            .add_systems(
                Update,
                sync_mutation_screen
                    .after(super::mutation_screen::update_mutation_screen_view)
                    .before(bevy_hui::HuiSystems::Build)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_mutation_screen);
    }
}

#[derive(Resource, Default)]
struct MutationTemplate(Option<Handle<HtmlTemplate>>);

/// Racine de l'écran ; `shown` : les propriétés déjà posées (recompilation au changement).
#[derive(Component)]
struct MutationRoot {
    shown: Vec<(String, String)>,
}

fn load_template(
    asset_server: Res<AssetServer>,
    registry: Res<Registry>,
    mut template: ResMut<MutationTemplate>,
) {
    // Un jeu sans écran de mutation (zombies, gungeon) ne demande pas d'asset absent.
    let present = registry
        .ui_files
        .iter()
        .any(|path| path.to_string_lossy().replace('\\', "/").ends_with(TEMPLATE));
    template.0 = present.then(|| asset_server.load(TEMPLATE));
}

fn despawn_mutation_screen(mut commands: Commands, roots: Query<Entity, With<MutationRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Propriétés du gabarit pour une vue ouverte (pur) : nom, description, couleur et bordure de
/// chaque carte (la surlignée en or), largeur de la barre de temps.
pub fn view_properties(view: &MutationScreenView) -> Vec<(String, String)> {
    let mut props = Vec::new();
    for index in 0..crate::ui::mutation_screen::CARD_SLOTS {
        let card = view.options.get(index);
        let selected = card.is_some() && index == view.highlighted;
        props.push((
            format!("name{index}"),
            card.map(|card| format!("{}  {}", index + 1, card.name))
                .unwrap_or_default(),
        ));
        props.push((
            format!("desc{index}"),
            card.map(|card| compact_description(&card.description))
                .unwrap_or_default(),
        ));
        props.push((
            format!("tint{index}"),
            (if selected { FOCUS } else { TEXT }).to_string(),
        ));
        props.push((
            format!("border{index}"),
            (if selected { FOCUS } else { IDLE_BORDER }).to_string(),
        ));
    }
    props.push((
        "bar_width".to_string(),
        format!("{}%", (view.time_fraction() * 100.0).round() as i32),
    ));
    props
}

fn sync_mutation_screen(
    mut commands: Commands,
    view: Res<MutationScreenView>,
    template: Res<MutationTemplate>,
    mut roots: Query<(Entity, &mut MutationRoot, &mut TemplateProperties)>,
) {
    let current = roots.iter_mut().next();
    if !view.open {
        if let Some((entity, ..)) = current {
            commands.entity(entity).despawn();
        }
        return;
    }
    let props = view_properties(&view);
    match current {
        Some((entity, mut root, mut properties)) => {
            if root.shown != props {
                for (key, value) in &props {
                    properties.set(key, value);
                }
                root.shown = props;
                commands.trigger(CompileContextEvent { entity });
            }
        }
        None => {
            let Some(handle) = template.0.clone() else {
                return;
            };
            let mut properties = TemplateProperties::default();
            for (key, value) in &props {
                properties.set(key, value);
            }
            commands.spawn((
                HtmlNode(handle),
                properties,
                MutationRoot { shown: props },
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::mutation_screen::MutationCard;

    fn view(highlighted: usize, frames_left: u32) -> MutationScreenView {
        MutationScreenView {
            open: true,
            handle: Some(0),
            options: ["a", "b"]
                .iter()
                .map(|id| MutationCard {
                    id: id.to_string(),
                    name: format!("Nom {id}"),
                    description: format!("Effet {id}."),
                })
                .collect(),
            frames_left,
            total_frames: 600,
            highlighted,
            since_frame: 10,
        }
    }

    fn get<'a>(props: &'a [(String, String)], key: &str) -> &'a str {
        &props.iter().find(|(k, _)| k == key).unwrap().1
    }

    #[test]
    fn proprietes_des_cartes_et_de_la_barre() {
        let props = view_properties(&view(1, 150));
        assert_eq!(get(&props, "name0"), "1  Nom a");
        assert_eq!(get(&props, "desc1"), "Effet b.");
        assert_eq!(get(&props, "border1"), FOCUS);
        assert_eq!(get(&props, "border0"), IDLE_BORDER);
        assert_eq!(get(&props, "tint1"), FOCUS);
        assert_eq!(get(&props, "bar_width"), "25%");
        assert_eq!(get(&props, "name2"), "", "troisième carte absente");
        assert_eq!(get(&props, "border2"), IDLE_BORDER);
    }
}
