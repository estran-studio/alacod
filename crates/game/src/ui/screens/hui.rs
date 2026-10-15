//! Option B du test comparatif : `bevy_hui` 0.7 (M2-T18, `docs/conventions.md` §56), feature
//! `ui_hui`.
//!
//! `ui/pause_screen.html` et `ui/inventory_screen.html` sont des gabarits pseudo-HTML ; leurs
//! `{propriétés}` reçoivent les textes et couleurs de la [`ScreenView`]. `bevy_hui` n'a ni boucle
//! ni liste : les lignes de l'inventaire sont des instances de `ui/hui_row.html` que ce module
//! crée sous les conteneurs `id="weapons"`, `"active"`, `"passives"`, `"consumables"`. Un
//! rechargement à chaud reconstruit le gabarit (les lignes disparaissent) : elles sont recréées
//! dès que les conteneurs neufs existent.
//!
//! Le focus (flèches, D-pad) est celui du modèle commun : `bevy_hui` n'a pas de navigation.

use bevy::prelude::*;
use bevy_hui::prelude::*;
use content::registry::Registry;
use std::collections::BTreeMap;

use super::custom::layout_file;
use super::model::{ListItem, ScreenId, ScreenView, Section};
use super::{ScreenBackend, Screens};
use crate::core::AppState;

const ROW_TEMPLATE: &str = "ui/hui_row.html";
const TEXT: &str = "#ffffff";
const MUTED: &str = "#ffffff99";
const FOCUS: &str = "#f1c40f";
const FOCUS_BACKGROUND: &str = "#ffffff22";
const NO_BACKGROUND: &str = "#00000000";

/// Rendu `bevy_hui` (`PresentationPlugin`, feature `ui_hui`).
pub struct HuiScreensPlugin;

impl Plugin for HuiScreensPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HuiPlugin>() {
            app.add_plugins(HuiPlugin);
        }
        app.init_resource::<HuiHandles>()
            .add_systems(OnEnter(AppState::InGame), load_templates)
            .add_systems(
                Update,
                sync_screen
                    .after(super::update_screens)
                    .run_if(in_state(AppState::InGame))
                    .run_if(is_hui_backend),
            )
            .add_systems(OnExit(AppState::InGame), despawn_screen);
    }
}

fn is_hui_backend(backend: Option<Res<ScreenBackend>>) -> bool {
    backend.is_some_and(|backend| *backend == ScreenBackend::Hui)
}

#[derive(Resource, Default)]
struct HuiHandles {
    pause: Option<Handle<HtmlTemplate>>,
    inventory: Option<Handle<HtmlTemplate>>,
    row: Option<Handle<HtmlTemplate>>,
}

/// Fichier de gabarit d'un écran.
pub fn template_file(screen: ScreenId) -> String {
    format!("ui/{}_screen.html", screen.name())
}

fn load_templates(
    asset_server: Res<AssetServer>,
    registry: Res<Registry>,
    mut handles: ResMut<HuiHandles>,
) {
    // Les gabarits n'existent que là où les écrans de la couche maison existent (gungeon).
    let present = |screen: ScreenId| {
        let file = layout_file(screen);
        registry
            .ui_files
            .iter()
            .any(|path| path.to_string_lossy().replace('\\', "/").ends_with(&file))
    };
    let load = |screen: ScreenId| {
        present(screen).then(|| asset_server.load::<HtmlTemplate>(template_file(screen)))
    };
    handles.pause = load(ScreenId::Pause);
    handles.inventory = load(ScreenId::Inventory);
    handles.row = (present(ScreenId::Pause) || present(ScreenId::Inventory))
        .then(|| asset_server.load::<HtmlTemplate>(ROW_TEMPLATE));
}

/// Racine d'un écran `bevy_hui` ; `shown` : la vue qui a servi aux propriétés.
#[derive(Component)]
struct HuiScreenRoot {
    screen: ScreenId,
    shown: ScreenView,
}

/// Conteneur dont les lignes ont été créées pour la vue courante.
#[derive(Component)]
struct Populated;

fn despawn_screen(mut commands: Commands, roots: Query<Entity, With<HuiScreenRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn list_text(view: &ScreenView, name: &str) -> String {
    match view.sections.get(name) {
        Some(Section::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

fn item_line(item: &ListItem) -> String {
    let mark = if item.focused { "> " } else { "  " };
    if item.detail.is_empty() {
        format!("{mark}{}", item.label)
    } else {
        format!("{mark}{}  {}", item.label, item.detail)
    }
}

/// Les propriétés des gabarits pour une vue (pur).
pub fn view_properties(view: &ScreenView) -> BTreeMap<String, String> {
    let mut props = BTreeMap::new();
    props.insert("title".into(), list_text(view, "title"));
    props.insert("hint".into(), list_text(view, "hint"));
    props.insert("detail".into(), list_text(view, "detail"));
    if let Some(Section::List(entries)) = view.sections.get("entries") {
        for (index, item) in entries.iter().enumerate() {
            props.insert(format!("e{index}"), item_line(item));
            props.insert(
                format!("c{index}"),
                (if item.focused { FOCUS } else { TEXT }).into(),
            );
            props.insert(
                format!("b{index}"),
                (if item.focused {
                    FOCUS_BACKGROUND
                } else {
                    NO_BACKGROUND
                })
                .into(),
            );
        }
    }
    if let Some(Section::Bar { label, fraction }) = view.sections.get("charge") {
        props.insert("charge_label".into(), label.clone());
        props.insert(
            "charge_width".into(),
            format!("{}%", (fraction * 100.0).round() as i32),
        );
    }
    props
}

fn row_properties(item: &ListItem) -> TemplateProperties {
    TemplateProperties::default()
        .with("label", &item_line(item))
        .with("color", if item.focused { FOCUS } else { MUTED })
        .with(
            "bg",
            if item.focused {
                FOCUS_BACKGROUND
            } else {
                NO_BACKGROUND
            },
        )
}

#[allow(clippy::too_many_arguments)]
fn sync_screen(
    mut commands: Commands,
    screens: Res<Screens>,
    handles: Res<HuiHandles>,
    mut roots: Query<(Entity, &mut HuiScreenRoot, &mut TemplateProperties)>,
    containers: Query<(Entity, &UiId, Option<&Children>, Has<Populated>)>,
    parents: Query<&ChildOf>,
) {
    let current = roots.iter_mut().next();
    let Some(view) = &screens.view else {
        if let Some((entity, ..)) = current {
            commands.entity(entity).despawn();
        }
        return;
    };
    let (root_entity, repopulate) = match current {
        Some((entity, mut root, mut props)) if root.screen == view.screen => {
            let changed = root.shown != *view;
            if changed {
                for (key, value) in view_properties(view) {
                    props.set(&key, &value);
                }
                root.shown = view.clone();
                commands.trigger(CompileContextEvent { entity });
            }
            (entity, changed)
        }
        other => {
            if let Some((entity, ..)) = other {
                commands.entity(entity).despawn();
            }
            let template = match view.screen {
                ScreenId::Pause => &handles.pause,
                ScreenId::Inventory => &handles.inventory,
            };
            let Some(template) = template.clone() else {
                return;
            };
            let mut props = TemplateProperties::default();
            for (key, value) in view_properties(view) {
                props.set(&key, &value);
            }
            commands.spawn((
                HtmlNode(template),
                props,
                HuiScreenRoot {
                    screen: view.screen,
                    shown: view.clone(),
                },
            ));
            return;
        }
    };
    // Lignes des listes : sous chaque conteneur `id` de ce gabarit.
    let Some(row) = handles.row.clone() else {
        return;
    };
    for (container, id, children, populated) in &containers {
        if !is_under(container, root_entity, &parents) {
            continue;
        }
        let Some(Section::List(items)) = view.sections.get(id.id().as_str()) else {
            continue;
        };
        if populated && !repopulate {
            continue;
        }
        for child in children.into_iter().flat_map(|c| c.iter()) {
            commands.entity(child).despawn();
        }
        for item in items {
            commands.spawn((
                HtmlNode(row.clone()),
                row_properties(item),
                ChildOf(container),
            ));
        }
        commands.entity(container).insert(Populated);
    }
}

fn is_under(mut entity: Entity, root: Entity, parents: &Query<&ChildOf>) -> bool {
    loop {
        if entity == root {
            return true;
        }
        match parents.get(entity) {
            Ok(parent) => entity = parent.parent(),
            Err(_) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::{inventory_view, pause_view, InventoryInput, ScreensState};
    use super::*;

    #[test]
    fn proprietes_du_menu_de_pause() {
        let props = view_properties(&pause_view(&ScreensState::default()));
        assert_eq!(props["title"], "Pause");
        assert_eq!(props["e0"], "> Reprendre");
        assert_eq!(props["c0"], FOCUS);
        assert_eq!(props["e1"], "  Recommencer");
        assert_eq!(props["c1"], TEXT);
        assert_eq!(props["e3"], "  Réglages");
    }

    fn copy_templates(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("alacod-{name}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("ui")).unwrap();
        let source =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/gungeon/assets/ui");
        for file in ["pause_screen.html", "inventory_screen.html", "hui_row.html"] {
            std::fs::copy(source.join(file), dir.join("ui").join(file)).unwrap();
        }
        dir
    }

    fn texts(app: &mut App) -> Vec<String> {
        let mut query = app.world_mut().query::<&Text>();
        query.iter(app.world()).map(|text| text.0.clone()).collect()
    }

    fn app_on(dir: &std::path::Path, view: ScreenView) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: dir.display().to_string(),
                ..default()
            },
            HuiPlugin,
        ));
        app.init_asset::<Font>()
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>();
        let server = app.world().resource::<AssetServer>().clone();
        app.insert_resource(HuiHandles {
            pause: Some(server.load("ui/pause_screen.html")),
            inventory: Some(server.load("ui/inventory_screen.html")),
            row: Some(server.load("ui/hui_row.html")),
        });
        app.insert_resource(Screens {
            state: ScreensState::default(),
            view: Some(view),
        });
        app.add_systems(Update, sync_screen);
        app
    }

    fn wait_for(app: &mut App, wanted: &str) -> (u32, std::time::Duration) {
        let start = std::time::Instant::now();
        for updates in 1..=2000 {
            app.update();
            if texts(app).iter().any(|t| t.contains(wanted)) {
                return (updates, start.elapsed());
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        panic!("« {wanted} » jamais affiché : {:?}", texts(app));
    }

    /// Le gabarit du menu de pause se construit, les propriétés remplissent les textes, un
    /// changement de focus recompile les propriétés, et un rechargement du fichier
    /// (`AssetServer::reload`, ce que fait `bevy/file_watcher`) reconstruit l'arbre.
    #[test]
    fn menu_de_pause_et_rechargement_a_chaud() {
        let dir = copy_templates("hui-pause");
        let mut app = app_on(&dir, pause_view(&ScreensState::default()));
        let (updates, took) = wait_for(&mut app, "> Reprendre");
        eprintln!("hui : premier affichage en {updates} updates, {took:?}");
        assert!(texts(&mut app).contains(&"  Réglages".to_string()));

        let moved = ScreensState {
            pause_focus: 1,
            ..ScreensState::default()
        };
        app.world_mut().resource_mut::<Screens>().view = Some(pause_view(&moved));
        wait_for(&mut app, "> Recommencer");

        let file = dir.join("ui/pause_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace("{hint}", "version B {hint}");
        std::fs::write(&file, edited).unwrap();
        let start = std::time::Instant::now();
        app.world().resource::<AssetServer>().reload("ui/pause_screen.html");
        let (updates, _) = wait_for(&mut app, "version B");
        eprintln!(
            "hui : rechargement visible en {updates} updates, {:?}",
            start.elapsed()
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    /// L'inventaire : les lignes sont créées sous les conteneurs `id` (pas de boucle dans
    /// `bevy_hui`), et recréées après un rechargement qui reconstruit les conteneurs.
    #[test]
    fn inventaire_lignes_creees_par_le_code() {
        use super::super::model::{ActiveLine, WeaponLine};
        let dir = copy_templates("hui-inventory");
        let input = InventoryInput {
            weapons: vec![WeaponLine {
                name: "pistolet".into(),
                mag: Some(8),
                reserve: 40,
                active: true,
            }],
            active: Some(ActiveLine {
                name: "Fiole".into(),
                charge: 1,
                required: 2,
            }),
            passives: vec!["Amulette".into()],
            consumables: vec![("Blank".into(), 2)],
        };
        let view = inventory_view(&input, &ScreensState::default());
        let mut app = app_on(&dir, view);
        let (updates, took) = wait_for(&mut app, "Blank  ×2");
        eprintln!("hui : inventaire affiché en {updates} updates, {took:?}");
        let shown = texts(&mut app);
        assert!(shown.contains(&"> pistolet (en main)  8 | 40".to_string()), "{shown:?}");
        assert!(shown.contains(&"1 / 2".to_string()), "{shown:?}");

        let file = dir.join("ui/inventory_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace(">Armes<", ">Armes équipées<");
        std::fs::write(&file, edited).unwrap();
        let start = std::time::Instant::now();
        app.world().resource::<AssetServer>().reload("ui/inventory_screen.html");
        wait_for(&mut app, "Armes équipées");
        let (updates, _) = wait_for(&mut app, "Blank  ×2");
        eprintln!(
            "hui : inventaire rechargé, lignes recréées en {updates} updates, {:?}",
            start.elapsed()
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn proprietes_de_la_jauge() {
        let view = inventory_view(&InventoryInput::default(), &ScreensState::default());
        let props = view_properties(&view);
        assert_eq!(props["charge_width"], "0%");
    }
}
