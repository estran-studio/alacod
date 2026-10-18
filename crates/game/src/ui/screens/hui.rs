//! Rendu des écrans en `bevy_hui` 0.7 (M2-T18, `docs/conventions.md` §56).
//!
//! Les gabarits `ui/*_screen.html` sont découverts directement dans [`Registry::ui_files`].
//! Chaque section texte devient une propriété du même nom ; une jauge fournit `{nom_label}`
//! et `{nom_width}`. Chaque liste remplit le conteneur dont l'`id` porte son nom, avec des
//! instances de `ui/hui_row.html`. Aucun branchement de rendu ne dépend du contenu d'un écran.
//! Un rechargement à chaud reconstruit le gabarit : les lignes sont recréées dès que les
//! conteneurs neufs existent.
//!
//! Le focus (flèches, D-pad) est celui du modèle commun : `bevy_hui` n'a pas de navigation.

use bevy::prelude::*;
use bevy_hui::prelude::*;
use content::registry::Registry;
use std::collections::BTreeMap;

use super::model::{ListItem, ScreenId, ScreenView, Section};
use super::Screens;
use crate::session::AppState;

const ROW_TEMPLATE: &str = "ui/hui_row.html";
const TEXT: &str = "#ffffff";
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
                // Avant la construction des gabarits : une ligne détruite ici ne doit plus être
                // construite par `spawn_ui` dans la même frame (commande sur une entité
                // détruite : panique de `bevy_hui`, vue au rendu de `ui_inventory`).
                sync_screen
                    .after(super::update_screens)
                    .before(bevy_hui::HuiSystems::Build)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_screen);
    }
}

#[derive(Resource, Default)]
struct HuiHandles {
    pause: Option<Handle<HtmlTemplate>>,
    inventory: Option<Handle<HtmlTemplate>>,
    merchant: Option<Handle<HtmlTemplate>>,
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
    let present = |file: &str| {
        registry
            .ui_files
            .iter()
            .any(|path| path.to_string_lossy().replace('\\', "/").ends_with(file))
    };
    let load = |screen: ScreenId| {
        let file = template_file(screen);
        present(&file).then(|| asset_server.load::<HtmlTemplate>(file))
    };
    handles.pause = load(ScreenId::Pause);
    handles.inventory = load(ScreenId::Inventory);
    handles.merchant = load(ScreenId::Merchant);
    handles.row =
        (handles.pause.is_some() || handles.inventory.is_some() || handles.merchant.is_some())
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

fn item_line(item: &ListItem) -> String {
    let mark = if item.focused { "> " } else { "  " };
    if item.detail.is_empty() {
        format!("{mark}{}", item.label)
    } else {
        format!("{mark}{}  {}", item.label, item.detail)
    }
}

/// Propriétés des sections, sans connaissance des écrans (pur).
pub fn view_properties(view: &ScreenView) -> BTreeMap<String, String> {
    let mut props = BTreeMap::new();
    for (name, section) in &view.sections {
        match section {
            Section::Text(text) => {
                props.insert(name.clone(), text.clone());
            }
            Section::Bar { label, fraction } => {
                props.insert(format!("{name}_label"), label.clone());
                props.insert(
                    format!("{name}_width"),
                    format!("{}%", (fraction * 100.0).round() as i32),
                );
            }
            // Les listes sont des lignes enfants, pas des propriétés à indices fixes.
            Section::List(_) => {}
        }
    }
    props
}

fn row_properties(item: &ListItem) -> TemplateProperties {
    TemplateProperties::default()
        .with("label", &item_line(item))
        .with("color", if item.focused { FOCUS } else { TEXT })
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
                ScreenId::Merchant => &handles.merchant,
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

    fn copy_templates(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("alacod-{name}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("ui")).unwrap();
        let source =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/gungeon/assets/ui");
        for file in ["pause_screen.html", "inventory_screen.html", "hui_row.html"] {
            std::fs::copy(source.join(file), dir.join("ui").join(file)).unwrap();
        }
        // Chemin canonique : sur macOS `/var` est un lien vers `/private/var`, et le watcher de
        // Bevy panique quand l'événement ne commence pas par la racine des assets.
        dir.canonicalize().unwrap()
    }

    /// Horloge réelle de Bevy (`Time<Real>`) : le test mesure sans lire l'horloge système.
    fn real_elapsed(app: &App) -> std::time::Duration {
        app.world().resource::<Time<Real>>().elapsed()
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
            ..Default::default()
        });
        app.insert_resource(Screens {
            state: ScreensState::default(),
            view: Some(view),
            ..Default::default()
        });
        app.add_systems(Update, sync_screen.before(bevy_hui::HuiSystems::Build));
        app
    }

    fn wait_for(app: &mut App, wanted: &str) -> (u32, std::time::Duration) {
        let start = real_elapsed(app);
        for updates in 1..=2000 {
            app.update();
            if texts(app).iter().any(|t| t.contains(wanted)) {
                return (updates, real_elapsed(app) - start);
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        panic!("« {wanted} » jamais affiché : {:?}", texts(app));
    }

    #[test]
    fn decouverte_html_sans_ron_et_liste_sans_nom_reserve() {
        let dir = copy_templates("hui-discovery");
        let file = dir.join("ui/pause_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace("id=\"entries\"", "id=\"options\"");
        std::fs::write(&file, edited).unwrap();
        let mut view = pause_view(&ScreensState::default());
        let entries = view.sections.remove("entries").unwrap();
        view.sections.insert("options".into(), entries);
        let mut app = app_on(&dir, view);
        app.insert_resource(HuiHandles::default())
            .insert_resource(Registry {
                ui_files: vec![file, dir.join(ROW_TEMPLATE)],
                ..default()
            })
            .add_systems(Startup, load_templates);
        wait_for(&mut app, "> Reprendre");
        let handles = app.world().resource::<HuiHandles>();
        assert!(handles.pause.is_some());
        assert!(handles.inventory.is_none());
        assert!(handles.row.is_some());
        let _ = std::fs::remove_dir_all(dir);
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
        let mut query = app.world_mut().query::<(&Text, &TextColor)>();
        let focused: Vec<_> = query
            .iter(app.world())
            .filter(|(text, _)| text.0.starts_with("> "))
            .collect();
        assert_eq!(focused.len(), 1);
        let (text, color) = focused[0];
        assert_eq!(text.0, "> Recommencer");
        assert_eq!(color.0.to_srgba().to_u8_array(), [241, 196, 15, 255]);

        let file = dir.join("ui/pause_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace("{hint}", "version B {hint}");
        std::fs::write(&file, edited).unwrap();
        let start = real_elapsed(&app);
        app.world()
            .resource::<AssetServer>()
            .reload("ui/pause_screen.html");
        let (updates, _) = wait_for(&mut app, "version B");
        eprintln!(
            "hui : rechargement visible en {updates} updates, {:?}",
            real_elapsed(&app) - start
        );
        wait_for(&mut app, "> Recommencer");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Modification sur disque détectée par le watcher, sans appel à `reload`.
    /// `cargo test -p game --features ui_hui,bevy/file_watcher -- --ignored watcher --nocapture`.
    #[test]
    #[ignore = "demande bevy/file_watcher"]
    fn watcher_de_fichiers_detecte_la_modification_hui() {
        let dir = copy_templates("hui-watcher");
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: dir.display().to_string(),
                watch_for_changes_override: Some(true),
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
            inventory: None,
            row: Some(server.load("ui/hui_row.html")),
            ..Default::default()
        });
        app.insert_resource(Screens {
            state: ScreensState::default(),
            view: Some(pause_view(&ScreensState::default())),
            ..Default::default()
        });
        app.add_systems(Update, sync_screen.before(bevy_hui::HuiSystems::Build));
        wait_for(&mut app, "> Reprendre");
        for _ in 0..200 {
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let file = dir.join("ui/pause_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace("{hint}", "version B {hint}");
        std::fs::write(&file, edited).unwrap();
        let (_, took) = wait_for(&mut app, "version B");
        eprintln!("hui : watcher, modification sur disque visible après {took:?}");
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
        assert!(
            shown.contains(&"> pistolet (en main)  8 | 40".to_string()),
            "{shown:?}"
        );
        assert!(shown.contains(&"1 / 2".to_string()), "{shown:?}");

        let file = dir.join("ui/inventory_screen.html");
        let edited = std::fs::read_to_string(&file)
            .unwrap()
            .replace(">Armes<", ">Armes équipées<");
        std::fs::write(&file, edited).unwrap();
        let start = real_elapsed(&app);
        app.world()
            .resource::<AssetServer>()
            .reload("ui/inventory_screen.html");
        wait_for(&mut app, "Armes équipées");
        let (updates, _) = wait_for(&mut app, "Blank  ×2");
        eprintln!(
            "hui : inventaire rechargé, lignes recréées en {updates} updates, {:?}",
            real_elapsed(&app) - start
        );
        let row_file = dir.join(ROW_TEMPLATE);
        let edited = std::fs::read_to_string(&row_file)
            .unwrap()
            .replace("{label}", "ligne {label}");
        std::fs::write(&row_file, edited).unwrap();
        app.world().resource::<AssetServer>().reload(ROW_TEMPLATE);
        wait_for(&mut app, "ligne   Blank  ×2");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// La vue de l'inventaire change pendant que l'écran est ouvert (la charge de l'actif
    /// retombe) : les lignes sont recréées, sans commande sur une entité détruite, et chaque
    /// update garde un affichage complet.
    #[test]
    fn inventaire_changement_de_vue_sans_commande_orpheline() {
        use super::super::model::{ActiveLine, WeaponLine};
        let dir = copy_templates("hui-change");
        let mut input = InventoryInput {
            weapons: vec![WeaponLine {
                name: "pistolet".into(),
                mag: Some(8),
                reserve: 40,
                active: true,
            }],
            active: Some(ActiveLine {
                name: "Fiole".into(),
                charge: 2,
                required: 2,
            }),
            passives: vec!["Amulette".into()],
            consumables: vec![("Blank".into(), 2)],
        };
        let mut app = app_on(&dir, inventory_view(&input, &ScreensState::default()));
        wait_for(&mut app, "Blank  ×2");
        for step in 0..6 {
            input.active.as_mut().unwrap().charge = step % 2;
            app.world_mut().resource_mut::<Screens>().view =
                Some(inventory_view(&input, &ScreensState::default()));
            app.update();
            app.update();
            app.update();
            let shown = texts(&mut app);
            eprintln!("changement {step} : {} textes", shown.len());
            assert!(
                shown.iter().any(|t| t.contains("Blank  ×2")),
                "lignes absentes après le changement {step} : {shown:?}"
            );
            assert!(
                shown.contains(&format!("{} / 2", step % 2)),
                "charge absente après le changement {step} : {shown:?}"
            );
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn proprietes_de_la_jauge() {
        let view = inventory_view(&InventoryInput::default(), &ScreensState::default());
        let props = view_properties(&view);
        assert_eq!(props["charge_width"], "0%");
    }
}
