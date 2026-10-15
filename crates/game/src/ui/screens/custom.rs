//! Option A du test comparatif : la couche maison étendue (M2-T18, `docs/conventions.md` §56).
//!
//! `ui/pause_screen.ron` et `ui/inventory_screen.ron` décrivent un arbre de boîtes
//! ([`LayoutNode`]) dont les feuilles sont liées par nom aux sections de la [`ScreenView`]
//! (`Text`, `List`, `Bar`). Le rendu reconstruit l'arbre `bevy_ui` quand la vue change ou quand
//! le fichier est rechargé (`bevy/file_watcher`, feature `native`).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use content::registry::Registry;
use serde::Deserialize;

use super::model::{ListItem, ScreenId, ScreenView, Section};
use super::{ScreenBackend, Screens};
use crate::core::AppState;
use crate::ui::hud::{hud_text_font, parse_color};

fn default_size() -> f32 {
    18.0
}
fn default_color() -> String {
    "#ffffff".into()
}
fn default_focus_color() -> String {
    "#f1c40f".into()
}
fn default_focus_background() -> String {
    "#ffffff22".into()
}
fn default_back() -> String {
    "#00000066".into()
}

/// Boîte qui contient d'autres nœuds.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Container {
    pub gap: f32,
    pub padding: f32,
    pub width: Option<f32>,
    /// Couleur de fond `#rrggbb[aa]`.
    pub background: Option<String>,
    /// `(couleur, épaisseur)`.
    pub border: Option<(String, f32)>,
    pub children: Vec<LayoutNode>,
}

/// Un nœud de mise en page.
#[derive(Debug, Clone, Deserialize)]
pub enum LayoutNode {
    Column(Container),
    Row(Container),
    /// Texte fixe (en-tête de section).
    Label {
        text: String,
        #[serde(default = "default_size")]
        size: f32,
        #[serde(default = "default_color")]
        color: String,
    },
    /// Section `Text` de la vue.
    Text {
        bind: String,
        #[serde(default = "default_size")]
        size: f32,
        #[serde(default = "default_color")]
        color: String,
    },
    /// Section `List` de la vue : une ligne par élément, le focusé coloré et sur fond.
    List {
        bind: String,
        #[serde(default = "default_size")]
        size: f32,
        #[serde(default = "default_color")]
        color: String,
        #[serde(default = "default_focus_color")]
        focus_color: String,
        #[serde(default = "default_focus_background")]
        focus_background: String,
        /// Texte quand la liste est vide.
        #[serde(default)]
        empty: Option<String>,
    },
    /// Section `Bar` de la vue : jauge et son texte.
    Bar {
        bind: String,
        size: (f32, f32),
        #[serde(default = "default_focus_color")]
        color: String,
        #[serde(default = "default_back")]
        back: String,
        #[serde(default = "default_size")]
        text_size: f32,
    },
}

/// Fichier `ui/<écran>_screen.ron`.
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct ScreenLayoutAsset {
    /// Police de tous les textes, relative à `assets/`.
    #[serde(default)]
    pub font: Option<String>,
    /// Voile plein écran derrière l'écran.
    #[serde(default = "default_back")]
    pub veil: String,
    pub root: LayoutNode,
}

#[derive(Resource, Default)]
struct LayoutHandles {
    pause: Option<Handle<ScreenLayoutAsset>>,
    inventory: Option<Handle<ScreenLayoutAsset>>,
}

/// Racine de l'écran dessiné ; `shown` : la vue dessinée, pour ne reconstruire qu'au changement.
#[derive(Component)]
struct CustomScreenRoot {
    shown: ScreenView,
}

/// Fichier de mise en page d'un écran.
pub fn layout_file(screen: ScreenId) -> String {
    format!("ui/{}_screen.ron", screen.name())
}

/// Rendu de la couche maison (`PresentationPlugin`).
pub struct CustomScreensPlugin;

impl Plugin for CustomScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ScreenLayoutAsset>::new(&["ron"]))
            .init_resource::<LayoutHandles>()
            .add_systems(OnEnter(AppState::InGame), load_layouts)
            .add_systems(
                Update,
                draw_screen
                    .after(super::update_screens)
                    .run_if(in_state(AppState::InGame))
                    .run_if(is_custom_backend),
            )
            .add_systems(OnExit(AppState::InGame), despawn_screen);
    }
}

fn is_custom_backend(backend: Option<Res<ScreenBackend>>) -> bool {
    backend.is_none_or(|backend| *backend == ScreenBackend::Custom)
}

fn load_layouts(
    asset_server: Res<AssetServer>,
    registry: Res<Registry>,
    mut handles: ResMut<LayoutHandles>,
) {
    // Un jeu sans ces écrans (zombies) ne doit pas demander un fichier absent.
    let load = |screen: ScreenId| {
        let file = layout_file(screen);
        registry
            .ui_files
            .iter()
            .any(|path| path.to_string_lossy().replace('\\', "/").ends_with(&file))
            .then(|| asset_server.load::<ScreenLayoutAsset>(file))
    };
    handles.pause = load(ScreenId::Pause);
    handles.inventory = load(ScreenId::Inventory);
}

fn despawn_screen(mut commands: Commands, roots: Query<Entity, With<CustomScreenRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn draw_screen(
    mut commands: Commands,
    screens: Res<Screens>,
    handles: Res<LayoutHandles>,
    layouts: Res<Assets<ScreenLayoutAsset>>,
    asset_server: Res<AssetServer>,
    roots: Query<(Entity, &CustomScreenRoot)>,
    mut events: MessageReader<AssetEvent<ScreenLayoutAsset>>,
) {
    let reloaded = events.read().count() > 0;
    let current = roots.iter().next();
    let up_to_date = current.is_some_and(|(_, root)| Some(&root.shown) == screens.view.as_ref());
    if up_to_date && !reloaded {
        return;
    }
    if let Some((entity, _)) = current {
        commands.entity(entity).despawn();
    }
    let Some(view) = &screens.view else {
        return;
    };
    let handle = match view.screen {
        ScreenId::Pause => &handles.pause,
        ScreenId::Inventory => &handles.inventory,
    };
    let Some(layout) = handle.as_ref().and_then(|h| layouts.get(h)) else {
        return;
    };
    let font = layout
        .font
        .as_ref()
        .map(|path| asset_server.load::<Font>(path.clone()));
    spawn_screen(&mut commands, view, layout, &font);
}

fn spawn_screen(
    commands: &mut Commands,
    view: &ScreenView,
    layout: &ScreenLayoutAsset,
    font: &Option<Handle<Font>>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(20),
            BackgroundColor(parse_color(&layout.veil)),
            CustomScreenRoot {
                shown: view.clone(),
            },
        ))
        .with_children(|parent| spawn_node(parent, &layout.root, view, font));
}

fn container_node(container: &Container, direction: FlexDirection) -> (Node, BackgroundColor) {
    let border = container
        .border
        .as_ref()
        .map_or(UiRect::ZERO, |(_, width)| UiRect::all(Val::Px(*width)));
    (
        Node {
            flex_direction: direction,
            row_gap: Val::Px(container.gap),
            column_gap: Val::Px(container.gap),
            padding: UiRect::all(Val::Px(container.padding)),
            width: container.width.map_or(Val::Auto, Val::Px),
            border,
            ..default()
        },
        BackgroundColor(
            container
                .background
                .as_deref()
                .map_or(Color::NONE, parse_color),
        ),
    )
}

fn spawn_node(
    parent: &mut ChildSpawnerCommands,
    node: &LayoutNode,
    view: &ScreenView,
    font: &Option<Handle<Font>>,
) {
    match node {
        LayoutNode::Column(container) | LayoutNode::Row(container) => {
            let direction = if matches!(node, LayoutNode::Column(_)) {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            let (layout, background) = container_node(container, direction);
            let mut entity = parent.spawn((layout, background));
            if let Some((color, _)) = &container.border {
                entity.insert(BorderColor::all(parse_color(color)));
            }
            entity.with_children(|inner| {
                for child in &container.children {
                    spawn_node(inner, child, view, font);
                }
            });
        }
        LayoutNode::Label { text, size, color } => {
            spawn_text(parent, text.clone(), *size, parse_color(color), font);
        }
        LayoutNode::Text { bind, size, color } => {
            if let Some(Section::Text(text)) = view.sections.get(bind) {
                if !text.is_empty() {
                    spawn_text(parent, text.clone(), *size, parse_color(color), font);
                }
            }
        }
        LayoutNode::List {
            bind,
            size,
            color,
            focus_color,
            focus_background,
            empty,
        } => {
            let Some(Section::List(items)) = view.sections.get(bind) else {
                return;
            };
            if items.is_empty() {
                if let Some(text) = empty {
                    spawn_text(parent, text.clone(), *size, parse_color(color).with_alpha(0.5), font);
                }
            }
            for item in items {
                spawn_list_item(
                    parent,
                    item,
                    *size,
                    parse_color(color),
                    parse_color(focus_color),
                    parse_color(focus_background),
                    font,
                );
            }
        }
        LayoutNode::Bar {
            bind,
            size,
            color,
            back,
            text_size,
        } => {
            let Some(Section::Bar { label, fraction }) = view.sections.get(bind) else {
                return;
            };
            if label.is_empty() {
                return;
            }
            parent
                .spawn((
                    Node {
                        width: Val::Px(size.0),
                        height: Val::Px(size.1),
                        ..default()
                    },
                    BackgroundColor(parse_color(back)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Px(size.0 * fraction),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(parse_color(color)),
                    ));
                });
            spawn_text(parent, label.clone(), *text_size, Color::WHITE, font);
        }
    }
}

fn spawn_list_item(
    parent: &mut ChildSpawnerCommands,
    item: &ListItem,
    size: f32,
    color: Color,
    focus_color: Color,
    focus_background: Color,
    font: &Option<Handle<Font>>,
) {
    let (mark, text_color, background) = if item.focused {
        ("> ", focus_color, focus_background)
    } else {
        ("  ", color, Color::NONE)
    };
    let line = if item.detail.is_empty() {
        format!("{mark}{}", item.label)
    } else {
        format!("{mark}{}  {}", item.label, item.detail)
    };
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|row| spawn_text(row, line, size, text_color, font));
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    text: String,
    size: f32,
    color: Color,
    font: &Option<Handle<Font>>,
) {
    parent.spawn((Text::new(text), hud_text_font(font, size), TextColor(color)));
}

#[cfg(test)]
mod tests {
    use super::super::model::{pause_view, ScreensState};
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    fn layout_text(label: &str) -> String {
        format!(
            "(root: Column((children: [Label(text: \"{label}\"), List(bind: \"entries\")])))"
        )
    }

    fn texts(app: &mut App) -> Vec<String> {
        let mut query = app.world_mut().query::<&Text>();
        query.iter(app.world()).map(|text| text.0.clone()).collect()
    }

    fn temp_assets(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("alacod-{name}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("ui")).unwrap();
        dir
    }

    /// Le rendu construit l'arbre de la vue (étiquette fixe, une ligne par entrée, « > » sur le
    /// focus), et un rechargement du fichier (`AssetServer::reload`, ce que fait
    /// `bevy/file_watcher` à la modification) le reconstruit avec le nouveau contenu, sans
    /// redémarrer. Mesure le délai en updates et en millisecondes.
    #[test]
    fn rechargement_a_chaud_de_la_mise_en_page() {
        let dir = temp_assets("custom");
        let file = dir.join("ui/pause_screen.ron");
        std::fs::write(&file, layout_text("version A")).unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: dir.display().to_string(),
                ..default()
            },
            RonAssetPlugin::<ScreenLayoutAsset>::new(&["ron"]),
        ));
        app.init_asset::<Font>();
        let handle = app
            .world()
            .resource::<AssetServer>()
            .load::<ScreenLayoutAsset>("ui/pause_screen.ron");
        app.insert_resource(LayoutHandles {
            pause: Some(handle),
            inventory: None,
        });
        app.insert_resource(Screens {
            state: ScreensState::default(),
            view: Some(pause_view(&ScreensState::default())),
        });
        app.add_systems(Update, draw_screen);

        let wait_for = |app: &mut App, wanted: &str| -> (u32, Duration) {
            let start = Instant::now();
            for updates in 1..=2000 {
                app.update();
                if texts(app).iter().any(|t| t.contains(wanted)) {
                    return (updates, start.elapsed());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            panic!("« {wanted} » jamais affiché : {:?}", texts(app));
        };
        let (updates, took) = wait_for(&mut app, "version A");
        eprintln!("maison : premier affichage en {updates} updates, {took:?}");
        let shown = texts(&mut app);
        assert!(shown.contains(&"> Reprendre".to_string()), "{shown:?}");
        assert!(shown.contains(&"  Réglages".to_string()), "{shown:?}");

        std::fs::write(&file, layout_text("version B")).unwrap();
        let reload_start = Instant::now();
        app.world().resource::<AssetServer>().reload("ui/pause_screen.ron");
        let (updates, _) = wait_for(&mut app, "version B");
        eprintln!(
            "maison : rechargement visible en {updates} updates, {:?}",
            reload_start.elapsed()
        );
        assert!(!texts(&mut app).iter().any(|t| t.contains("version A")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn les_fichiers_des_jeux_se_lisent() {
        for text in [
            include_str!("../../../../../games/gungeon/assets/ui/pause_screen.ron"),
            include_str!("../../../../../games/gungeon/assets/ui/inventory_screen.ron"),
        ] {
            ron::from_str::<ScreenLayoutAsset>(text).expect("mise en page d'écran");
        }
    }
}
