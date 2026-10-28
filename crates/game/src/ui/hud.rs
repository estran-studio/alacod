use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::downed::{Downed, Reviving};
use map::game::entity::map::door::DoorComponent;
use map::game::entity::map::window::WindowHealth;
use run::perks::Perks;
use serde::{Deserialize, Serialize};
use sim_core::modifier::{ModifierSource, Modifiers};
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;

use crate::character::config::{CharacterConfigKey, CharacterConfigs};
use crate::character::enemy::Enemy;
use crate::character::player::Player;
use crate::collider::Collider;
use crate::economy::PerkMachine;
use crate::interaction::{point_to_collider_surface_distance_sq, Interactable, InteractionType};
use crate::powerups::PowerUpsTable;
use crate::session::{AppState, SIM_FPS};
use crate::ui::hud_model::{
    player_source_text, HeartKind, HeartSlot, HudMapRoom, HudPlayer, HudSnapshot,
};
use crate::waves::state::WaveState;
use crate::weapons::{WeaponInventory, WeaponPickup};

/// HUD Root marker component
#[derive(Component)]
pub struct HudRoot;

/// Widget marker components for updates
#[derive(Component)]
pub struct HudBarWidget {
    pub source: String,
    pub full_width: f32,
    /// Couleur du RON (`rads`) ; `health` garde son dégradé rouge → vert.
    pub color: Color,
}

/// Sans actif (ou définition de charge), cacher également le fond de sa jauge.
#[derive(Component)]
struct HudActiveChargeTrack;

#[derive(Component)]
pub struct HudTextWidget {
    pub source: String,
    /// Texte fixe devant la valeur (« Vague »), gardé ici : le texte affiché est recomposé.
    pub prefix: String,
    /// Fond derrière le texte (T2.12, `HudWidget::background`), affiché seulement quand le
    /// texte n'est pas vide.
    pub background: Option<Color>,
    pub compact: bool,
}

/// Rangée d'icônes (T2.12, source `perks`) : un carré de couleur par entrée, avec son
/// initiale. `shown` garde les ids affichés : les enfants ne sont reconstruits que si la
/// liste change.
#[derive(Component)]
pub struct HudIconsWidget {
    pub source: String,
    /// Côté d'une icône, en pixels (hauteur du widget).
    pub icon_size: f32,
    pub shown: Vec<String>,
    /// Police des étiquettes (`HudConfig::font`), `None` : police par défaut de Bevy.
    pub font: Option<Handle<Font>>,
}

/// Rangée d'un compteur à icône : masquée tant que sa source n'a pas de valeur (D22, pas même
/// l'icône).
#[derive(Component)]
struct HudCounterRow {
    source: String,
}

/// Rangée de cœurs (source `hearts`) : `shown` garde les cœurs dessinés, les enfants ne sont
/// reconstruits que s'ils changent.
#[derive(Component)]
struct HudHeartsWidget {
    #[allow(dead_code)]
    source: String,
    icon_size: f32,
    shown: Option<Vec<HeartSlot>>,
}

/// Minicarte spatiale dérivée du snapshot ; reconstruite seulement si ses salles changent.
#[derive(Component)]
struct HudMapWidget {
    source: String,
    shown: Vec<HudMapRoom>,
    font: Option<Handle<Font>>,
    font_size: f32,
    /// Icône par type de salle (`HudConfig::room_icons`).
    room_icons: BTreeMap<String, HudIcon>,
}

/// Les sources que le HUD sait lire (liste fermée, lint `UnknownKind`) : une autre dans le RON
/// déclenche aussi un `warn!` au chargement. T2.12 : `perks`, `downed`, `powerups`, `prompt` ;
/// T1.18 (§32) : `rads`, `level`, `ammo_by_type`, `statuses`, `floor`.
const SOURCES: &[&str] = content::ui::HUD_SOURCES;

/// Position anchor for HUD widgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    /// Centre de l'écran (T2.12, indicateur « à terre ») : l'offset décale depuis le centre.
    Center,
}

/// HUD widget kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HudWidgetKind {
    Bar {
        source: String,
    },
    Text {
        source: String,
        prefix: Option<String>,
    },
    /// Rangée d'icônes (T2.12) : un carré `size.1` × `size.1` par entrée de la source
    /// (`perks`), couleur et lettre lues dans [`HudConfig::icons`].
    Icons {
        source: String,
    },
    /// Rectangles des salles de l'étage, positions stables et états issus de `RoomStates`.
    Map {
        source: String,
    },
    /// Légende des silhouettes et couleurs (touche F7, source `legend`) : lignes lues dans
    /// [`HudConfig::legend`]. Fermée par défaut ; `size` est celle du panneau.
    Legend {
        source: String,
    },
    /// Rangée de cœurs (M3-T16, §81, source `hearts`) : un carré `size.1` par cœur, rouge puis
    /// d'âme, rempli à moitié pour un demi-cœur ; passe à la ligne à la largeur `size.0`.
    Hearts {
        source: String,
    },
    /// Compteur à icône (M3-T16, `coins`, `keys`, `bombs`) : le carré `icons[source]` suivi du
    /// nombre, de hauteur `size.1`.
    Counter {
        source: String,
    },
}

/// A single HUD widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudWidget {
    pub kind: HudWidgetKind,
    pub anchor: HudAnchor,
    pub offset: (f32, f32),
    pub size: Option<(f32, f32)>,
    pub color: Option<String>,
    pub font_size: Option<f32>,
    /// T2.12 : fond derrière un `Text` (`#rrggbb` ou `#rrggbbaa`), masqué quand le texte
    /// est vide. `None` : pas de fond.
    #[serde(default)]
    pub background: Option<String>,
    /// Short presentation labels and bounded temporary-effect lists.
    #[serde(default)]
    pub compact: bool,
}

/// Icône d'un id de contenu (T2.12, perks) : un carré de couleur et une courte étiquette
/// (l'initiale), ou, depuis S1-8, l'`image` d'un pack (chemin relatif à `assets/`) : l'image remplace
/// le carré et l'étiquette. Un id absent de [`HudConfig::icons`] prend un carré gris et la première
/// lettre de son id en majuscule (voir [`icon_for`]). Une image n'est jamais posée par le `hud.ron`
/// public d'un jeu dont le pack n'est pas redistribuable : elle vient de la surcharge locale
/// (`ui/hud.local.ron`, [`HudOverlay`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudIcon {
    pub color: String,
    pub label: String,
    #[serde(default)]
    pub image: Option<String>,
}

/// Groupe de la légende (`HudConfig::legend`) : un titre et des lignes, rangés dans la colonne
/// `column` du panneau (0, 1, 2…).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendGroup {
    pub title: String,
    #[serde(default)]
    pub column: usize,
    pub entries: Vec<LegendEntry>,
}

/// Une ligne de la légende : l'image `sprite` (id de `sprites.ron`, case `Idle`) éventuellement
/// teintée (`tint`), ou une pastille de couleur (`color`), et son texte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendEntry {
    pub label: String,
    #[serde(default)]
    pub sprite: Option<String>,
    /// Animation de l'entrée à montrer (`Open`, `Locked`…) ; `Idle` par défaut.
    #[serde(default)]
    pub anim: Option<String>,
    #[serde(default)]
    pub tint: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Dessin des murs d'une couche IntGrid (`HudConfig::wall_art`) : présentation seule, dérivée des cases
/// de la couche à chaque apparition d'un niveau (auto-tuile : `map_ldtk::wall_art`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallArt {
    /// Couche IntGrid dont les cases non nulles sont des murs (`Walls`).
    pub layer: String,
    /// Feuille de tuiles (relative à `assets/`), tuiles carrées de `tile` pixels sur `columns` colonnes.
    pub atlas: String,
    #[serde(default = "WallArt::default_tile")]
    pub tile: u32,
    #[serde(default = "WallArt::default_columns")]
    pub columns: u32,
    /// Profondeur locale au niveau (le fond du niveau est à 0, la couche `Walls` à 1).
    #[serde(default = "WallArt::default_z")]
    pub z: f32,
    /// Couches de métadonnées que la carte ne doit pas dessiner (`LevelConnection` : ouvertures des gabarits,
    /// que le rendu IntGrid colorié de LDtk peint en noir).
    #[serde(default)]
    pub hide_layers: Vec<String>,
    /// Cases d'autres couches IntGrid dessinées chacune par une tuile fixe de la feuille (m3-salles-murs-isaac : les piques,
    /// couche `Surfaces` valeur 5, tuile 36). La couche d'origine se cache par `hide_layers`.
    #[serde(default)]
    pub cells: Vec<CellArt>,
    /// Sol : une tuile de la feuille posée sous chaque case qui n'est pas un mur (S1-8, zombies). Absent :
    /// le sol reste celui de la carte (couleur de fond du niveau ou couches de tuiles de LDtk).
    #[serde(default)]
    pub floor: Option<FloorArt>,
}

/// Remplissage de sol de [`WallArt`] : une tuile par case non murée, choisie parmi `tiles` (indices de la
/// feuille de `WallArt`) par une fonction de la position de la case, donc la même sur tous les clients et
/// à chaque rechargement du niveau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorArt {
    pub tiles: Vec<u32>,
    /// Profondeur locale au niveau : au-dessus du fond (0) et sous tout ce qui se pose au sol, halos des
    /// ramassables compris (leur `z` est celui de leur porteur + 0,1).
    #[serde(default = "FloorArt::default_z")]
    pub z: f32,
}

impl FloorArt {
    fn default_z() -> f32 {
        0.05
    }
}

/// Une case d'IntGrid dessinée par une tuile de la feuille de `WallArt` (`WallArt::cells`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellArt {
    /// Couche IntGrid lue.
    pub layer: String,
    /// Valeur d'IntGrid dessinée.
    pub value: i32,
    /// Indice de la tuile dans la feuille (ligne par ligne).
    pub tile: u32,
    /// Profondeur locale au niveau (sous les personnages et les objets du monde, au-dessus du fond).
    #[serde(default = "CellArt::default_z")]
    pub z: f32,
}

impl CellArt {
    fn default_z() -> f32 {
        0.5
    }
}

impl WallArt {
    fn default_tile() -> u32 {
        16
    }
    fn default_columns() -> u32 {
        8
    }
    fn default_z() -> f32 {
        1.5
    }
}

/// HUD configuration asset
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct HudConfig {
    pub widgets: Vec<HudWidget>,
    /// T2.12 : icône par id de perk (`economy/perks.ron`), pour `Icons(source: "perks")` et
    /// l'étiquette de `Text(source: "perks")`.
    #[serde(default)]
    pub icons: BTreeMap<String, HudIcon>,
    /// T2.12 : nom affiché par id d'arme (`weapons.ron`), pour la source `prompt`
    /// (« Acheter fusil à pompe — $1000 »). Absent : l'id lui-même.
    #[serde(default)]
    pub names: BTreeMap<String, String>,
    /// T2.12 : police de tous les textes du HUD, relative à `assets/`. La police par défaut
    /// de Bevy n'a ni lettres accentuées (« À TERRE », « possédé ») ni tiret long : les jeux
    /// déclarent `fonts/FiraMono-Medium.ttf`. `None` : police par défaut.
    #[serde(default)]
    pub font: Option<String>,
    /// Logical viewport that fits this game's HUD and modal screens. Shrinks on
    /// smaller windows; omitted by existing games to preserve their UI scale.
    #[serde(default)]
    pub reference_size: Option<(f32, f32)>,
    /// Couleur des joueurs par handle (`#rrggbb`), appliquée à leurs calques (modulo le nombre de
    /// couleurs). Absent : les personnages gardent leur teinte.
    #[serde(default)]
    pub player_colors: Vec<String>,
    /// Légende des silhouettes et couleurs (source `legend`, touche F7).
    #[serde(default)]
    pub legend: Vec<LegendGroup>,
    /// Cache les repères d'apparition de la carte (tuiles d'éditeur) : `true` dans un jeu dont les
    /// ennemis ont leurs propres sprites.
    #[serde(default)]
    pub hide_markers: bool,
    /// M3-T16 : icône par type de salle (`tresor`, `boutique`, `boss`, `secrete`, `diable`…) sur la
    /// minicarte : l'étiquette est écrite dans la salle, la couleur en teinte la bordure.
    #[serde(default)]
    pub room_icons: BTreeMap<String, HudIcon>,
    /// Dessin des murs de la carte (`map_ldtk::wall_art`, `docs/conventions.md` §83) : absent, les murs ne
    /// sont pas dessinés (autres jeux inchangés).
    #[serde(default)]
    pub wall_art: Option<WallArt>,
}

/// Surcharge locale du HUD (S1-8) : `ui/hud.local.ron`, à côté de `ui/hud.ron`, **gitignorée** comme
/// `sprites.local.ron` (pack d'images non redistribuable ; `docs/conventions.md` §2). Partielle : seuls les
/// champs qu'elle porte remplacent ceux du `hud.ron` public (`wall_art` en bloc, `icons` id par id, un id
/// du fichier public absent de la surcharge est gardé). Sans fichier (CI, autres postes, amis, wasm) le HUD
/// public est lu tel quel : le repli est le `hud.ron` commité.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HudOverlay {
    #[serde(default)]
    pub wall_art: Option<WallArt>,
    #[serde(default)]
    pub icons: BTreeMap<String, HudIcon>,
}

impl HudConfig {
    /// Pose la surcharge locale sur la configuration publique.
    pub fn apply_overlay(&mut self, overlay: HudOverlay) {
        if overlay.wall_art.is_some() {
            self.wall_art = overlay.wall_art;
        }
        self.icons.extend(overlay.icons);
    }
}

/// Options RON du HUD : celles de `RonAssetPlugin` (`Some` implicite), pour que les fichiers lus ne changent pas.
fn hud_ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

#[derive(Debug, thiserror::Error)]
pub enum HudLoadError {
    #[error("lecture du hud : {0}")]
    Io(#[from] std::io::Error),
    #[error("RON du hud : {0}")]
    Ron(#[from] ron::error::SpannedError),
}

/// Chargeur de `ui/hud.ron` : comme `RonAssetPlugin<HudConfig>`, plus la surcharge locale
/// `ui/hud.local.ron` quand elle existe (natif seulement : le web ne cherche pas un fichier absent du
/// site, il lirait un 404). L'absence du fichier est silencieuse ; sa présence l'enregistre comme
/// dépendance (rechargement à chaud).
#[derive(TypePath, Default)]
pub struct HudConfigLoader;

impl bevy::asset::AssetLoader for HudConfigLoader {
    type Asset = HudConfig;
    type Settings = ();
    type Error = HudLoadError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &(),
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<HudConfig, HudLoadError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let mut config: HudConfig = hud_ron_options().from_bytes(&bytes)?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let local = load_context.path().path().with_extension("local.ron");
            if let Ok(local_bytes) = load_context.read_asset_bytes(local).await {
                match hud_ron_options().from_bytes::<HudOverlay>(&local_bytes) {
                    Ok(overlay) => config.apply_overlay(overlay),
                    Err(error) => warn!("ui/hud.local.ron ignoré : {error}"),
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = load_context;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HudConfig>()
            .register_asset_loader(HudConfigLoader)
            .init_resource::<HudConfigHandle>()
            .add_systems(OnEnter(AppState::InGame), load_hud_config)
            .add_systems(
                Update,
                spawn_hud_when_ready.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                update_hud_values
                    .after(crate::ui::hud_model::update_hud_snapshot)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    update_hud_v1_values,
                    update_hud_map,
                    update_hud_hearts,
                    update_hud_counters,
                    update_hud_legend,
                )
                    .after(crate::ui::hud_model::HudModelSet)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_hud_config_changes.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                PostUpdate,
                sync_hud_presentation
                    .after(bevy::camera::CameraUpdateSystems)
                    .before(UiSystems::Layout)
                    .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_hud);
    }
}

/// Resource to hold the HUD config asset handle
#[derive(Resource, Default)]
pub struct HudConfigHandle(Option<Handle<HudConfig>>);

impl HudConfigHandle {
    /// La configuration du HUD, une fois chargée.
    pub fn get<'a>(&self, configs: &'a Assets<HudConfig>) -> Option<&'a HudConfig> {
        self.0.as_ref().and_then(|handle| configs.get(handle))
    }
}

/// Derived presentation only: reserve the screen for mutation/end modals and fit
/// the complete UI into the viewport, including screenshot render targets.
fn sync_hud_presentation(
    view: Res<crate::ui::mutation_screen::MutationScreenView>,
    run: Option<Res<run::Run>>,
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    cameras: Query<&Camera, With<crate::camera::GameCamera>>,
    mut scale: ResMut<UiScale>,
    mut roots: Query<&mut Visibility, With<HudRoot>>,
) {
    let config = config_handle.0.as_ref().and_then(|h| configs.get(h));
    // Opt-in with the compact layout; legacy games keep their HUD during modals.
    let Some(reference) = config.and_then(|config| config.reference_size) else {
        return;
    };
    if let Ok(camera) = cameras.single() {
        if let Some(size) = camera.logical_viewport_size() {
            scale.0 = hud_scale(size, reference);
        }
    }
    let modal = view.open
        || run
            .as_deref()
            .is_some_and(|r| matches!(r.step, run::RunStep::Ended { .. }));
    for mut visibility in &mut roots {
        *visibility = if modal {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn hud_scale(viewport: Vec2, reference: (f32, f32)) -> f32 {
    if reference.0 <= 0.0 || reference.1 <= 0.0 || viewport.min_element() <= 0.0 {
        return 1.0;
    }
    (viewport.x / reference.0)
        .min(viewport.y / reference.1)
        .min(1.0)
}

/// Load the HUD config handle
fn load_hud_config(asset_server: Res<AssetServer>, mut config_handle: ResMut<HudConfigHandle>) {
    config_handle.0 = Some(asset_server.load("ui/hud.ron"));
}

/// Spawn the HUD UI tree when config is ready
fn spawn_hud_when_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    q_hud_root: Query<Entity, With<HudRoot>>,
) {
    if q_hud_root.is_empty() {
        if let Some(handle) = &config_handle.0 {
            if let Some(config) = configs.get(handle) {
                let entity = commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        HudRoot,
                    ))
                    .id();
                let font = config.font.as_ref().map(|path| asset_server.load(path));
                spawn_hud_widgets(&mut commands, &asset_server, entity, config, font);
            }
        }
    }
}

/// Convert anchor enum to bevy UI layout properties
fn anchor_to_node(anchor: HudAnchor, offset: (f32, f32), size: (f32, f32)) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(size.0),
        height: Val::Px(size.1),
        ..default()
    };

    match anchor {
        HudAnchor::TopLeft => {
            node.left = Val::Px(offset.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::TopCenter => {
            node.left = Val::Percent(50.0);
            node.margin.left = Val::Px(-size.0 / 2.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::TopRight => {
            node.right = Val::Px(offset.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::BottomLeft => {
            node.left = Val::Px(offset.0);
            node.bottom = Val::Px(offset.1);
        }
        HudAnchor::BottomCenter => {
            node.left = Val::Percent(50.0);
            node.margin.left = Val::Px(-size.0 / 2.0);
            node.bottom = Val::Px(offset.1);
        }
        HudAnchor::BottomRight => {
            node.right = Val::Px(offset.0);
            node.bottom = Val::Px(offset.1);
        }
        HudAnchor::Center => {
            node.left = Val::Percent(50.0);
            node.top = Val::Percent(50.0);
            node.margin.left = Val::Px(offset.0 - size.0 / 2.0);
            node.margin.top = Val::Px(offset.1 - size.1 / 2.0);
        }
    }

    node
}

/// Alignement du texte dans sa boîte (T2.12) : centré pour les ancres centrées, à droite
/// pour les ancres de droite (les lignes multiples, ex. `powerups`, restent collées au
/// bord), à gauche sinon.
fn anchor_justify(anchor: HudAnchor) -> Justify {
    match anchor {
        HudAnchor::TopCenter | HudAnchor::BottomCenter | HudAnchor::Center => Justify::Center,
        HudAnchor::TopRight | HudAnchor::BottomRight => Justify::Right,
        HudAnchor::TopLeft | HudAnchor::BottomLeft => Justify::Left,
    }
}

/// Police d'un texte du HUD : celle de `hud.ron` si déclarée, sinon celle de Bevy.
pub(crate) fn hud_text_font(font: &Option<Handle<Font>>, size: f32) -> TextFont {
    match font {
        Some(font) => TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        None => TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
    }
}

fn spawn_hud_widgets(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity: Entity,
    config: &HudConfig,
    font: Option<Handle<Font>>,
) {
    commands.entity(entity).with_children(|parent| {
        for widget in &config.widgets {
            let size = widget.size.unwrap_or((160.0, 24.0));
            let source = match &widget.kind {
                HudWidgetKind::Bar { source }
                | HudWidgetKind::Text { source, .. }
                | HudWidgetKind::Icons { source }
                | HudWidgetKind::Map { source }
                | HudWidgetKind::Legend { source }
                | HudWidgetKind::Hearts { source }
                | HudWidgetKind::Counter { source } => source,
            };
            if !SOURCES.contains(&source.as_str()) {
                warn!("hud.ron : source inconnue « {source} » (connues : {SOURCES:?})");
            }
            let node = anchor_to_node(widget.anchor, widget.offset, size);
            let font_size = widget.font_size.unwrap_or(16.0);
            let color = parse_color(widget.color.as_deref().unwrap_or("#ffffff"));

            match &widget.kind {
                HudWidgetKind::Bar { source } => {
                    let mut track =
                        parent.spawn((node, BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.85))));
                    if source == "active_charge" {
                        track.insert((HudActiveChargeTrack, Visibility::Hidden));
                    }
                    track.with_children(|track| {
                        track.spawn((
                            Node {
                                width: Val::Px(size.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(color),
                            HudBarWidget {
                                source: source.clone(),
                                full_width: size.0,
                                color,
                            },
                        ));
                    });
                }
                HudWidgetKind::Icons { source } => {
                    let mut node = node;
                    // Rangée qui s'étend vers la droite à partir de l'ancre : la largeur
                    // du RON ne borne pas le nombre d'icônes.
                    node.width = Val::Auto;
                    node.flex_direction = FlexDirection::Row;
                    node.column_gap = Val::Px(4.0);
                    parent.spawn((
                        node,
                        HudIconsWidget {
                            source: source.clone(),
                            icon_size: size.1,
                            shown: Vec::new(),
                            font: font.clone(),
                        },
                    ));
                }
                HudWidgetKind::Hearts { source } => {
                    // Rangée qui passe à la ligne à la largeur du RON (jusqu'à douze cœurs et plus).
                    let mut node = node;
                    node.height = Val::Auto;
                    node.flex_direction = FlexDirection::Row;
                    node.flex_wrap = FlexWrap::Wrap;
                    node.column_gap = Val::Px(3.0);
                    node.row_gap = Val::Px(3.0);
                    parent.spawn((
                        node,
                        HudHeartsWidget {
                            source: source.clone(),
                            icon_size: size.1,
                            shown: None,
                        },
                    ));
                }
                HudWidgetKind::Counter { source } => {
                    let (icon_color, label) = config
                        .icons
                        .get(source)
                        .map_or((Color::srgb(0.5, 0.5, 0.5), String::new()), |icon| {
                            (parse_color(&icon.color), icon.label.clone())
                        });
                    let icon_image = config
                        .icons
                        .get(source)
                        .and_then(|icon| icon.image.as_deref())
                        .map(|path| load_icon(asset_server, path));
                    let mut node = node;
                    node.flex_direction = FlexDirection::Row;
                    node.align_items = AlignItems::Center;
                    node.column_gap = Val::Px(6.0);
                    parent
                        .spawn((
                            node,
                            Visibility::Hidden,
                            HudCounterRow {
                                source: source.clone(),
                            },
                        ))
                        .with_children(|row| {
                            if let Some(image) = icon_image {
                                // Image du pack (surcharge locale) : remplace le carré et l'étiquette.
                                row.spawn((
                                    Node {
                                        width: Val::Px(size.1),
                                        height: Val::Px(size.1),
                                        ..default()
                                    },
                                    ImageNode::new(image),
                                ));
                            } else {
                                row.spawn((
                                    Node {
                                        width: Val::Px(size.1),
                                        height: Val::Px(size.1),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(icon_color),
                                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                                ))
                                .with_children(|icon| {
                                    icon.spawn((
                                        Text::new(label),
                                        hud_text_font(&font, size.1 * 0.6),
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                            row.spawn((
                                Text::new(""),
                                hud_text_font(&font, font_size),
                                TextColor(color),
                                BackgroundColor(Color::NONE),
                                HudTextWidget {
                                    source: source.clone(),
                                    prefix: String::new(),
                                    background: None,
                                    compact: false,
                                },
                            ));
                        });
                }
                HudWidgetKind::Map { source } => {
                    parent.spawn((
                        node,
                        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.85)),
                        HudMapWidget {
                            source: source.clone(),
                            shown: Vec::new(),
                            font: font.clone(),
                            font_size,
                            room_icons: config.room_icons.clone(),
                        },
                    ));
                }
                HudWidgetKind::Legend { source } => {
                    // Panneau de la légende : fermé jusqu'à F7, rempli à l'ouverture (`update_hud_legend`).
                    let mut node = node;
                    node.height = Val::Auto;
                    node.flex_direction = FlexDirection::Row;
                    node.column_gap = Val::Px(14.0);
                    node.padding = UiRect::all(Val::Px(8.0));
                    let background = widget.background.as_deref().map(parse_color);
                    parent.spawn((
                        node,
                        BackgroundColor(background.unwrap_or(Color::srgba(0.02, 0.03, 0.05, 0.9))),
                        Visibility::Hidden,
                        HudLegendWidget {
                            source: source.clone(),
                            built: false,
                            font: font.clone(),
                            font_size,
                        },
                    ));
                }
                HudWidgetKind::Text { source, prefix } => {
                    let text = prefix.clone().unwrap_or_default();
                    let background = widget.background.as_deref().map(parse_color);
                    let mut node = node;
                    if background.is_some() {
                        // Le fond suit le nombre de lignes (`powerups` : une par power-up) ;
                        // la largeur reste celle du RON (centrage des ancres centrées).
                        node.height = Val::Auto;
                        node.padding = UiRect::axes(Val::Px(8.0), Val::Px(4.0));
                    }
                    parent.spawn((
                        node,
                        BackgroundColor(text_background(background, &text)),
                        Text::new(text.clone()),
                        hud_text_font(&font, font_size),
                        TextColor(color),
                        TextLayout::justify(anchor_justify(widget.anchor)),
                        HudTextWidget {
                            source: source.clone(),
                            prefix: text,
                            background,
                            compact: widget.compact,
                        },
                    ));
                }
            }
        }
    });
}

/// Met à jour le HUD : barres et textes du joueur depuis [`HudSnapshot`] (T1.18, calculé
/// aussi en headless), `wave`/`enemies`/`players` depuis l'ECS.
fn update_hud_values(
    snapshot: Res<HudSnapshot>,
    hud_player: HudPlayer,
    inventories: Query<&WeaponInventory>,
    all_players: Query<(), With<Player>>,
    enemies: Query<(), With<Enemy>>,
    wave_state: Res<WaveState>,
    mut bar_widgets: Query<
        (&HudBarWidget, &mut Node, &mut BackgroundColor),
        Without<HudTextWidget>,
    >,
    mut text_widgets: Query<
        (&HudTextWidget, &mut Text, &mut BackgroundColor),
        Without<HudBarWidget>,
    >,
    mut active_tracks: Query<&mut Visibility, With<HudActiveChargeTrack>>,
) {
    let enemy_count = enemies.iter().count();
    let player_count = all_players.iter().count();
    let wave_num = wave_state.current_wave;
    let reloading = hud_player
        .entity()
        .and_then(|entity| inventories.get(entity).ok())
        .is_some_and(|inventory| inventory.is_reloading());

    for mut visibility in &mut active_tracks {
        *visibility = if snapshot.bars.contains_key("active_charge") {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (bar_widget, mut node, mut bg_color) in bar_widgets.iter_mut() {
        let Some(ratio) = snapshot.bars.get(bar_widget.source.as_str()).copied() else {
            // Sans valeur (joueur mort, progression inactive) : barre vide.
            node.width = Val::Px(0.0);
            continue;
        };
        node.width = Val::Px(bar_widget.full_width * ratio);
        *bg_color = match bar_widget.source.as_str() {
            // Dégradé rouge → vert
            "health" => BackgroundColor(Color::srgba(1.0 - ratio, ratio, 0.0, 0.8)),
            _ => BackgroundColor(bar_widget.color),
        };
    }

    for (text_widget, mut text, mut background) in text_widgets.iter_mut() {
        let prefix_text = text_widget.prefix.clone();
        let mut new_text = match text_widget.source.as_str() {
            "wave" => format!("{}{}", prefix_text, wave_num),
            "enemies" => format!("{}{}", prefix_text, enemy_count),
            "players" => format!("{}{}", prefix_text, player_count),
            source => match player_source_text(source, &prefix_text, &snapshot.values) {
                Some(text) => text,
                // Sources T2.12 (`update_hud_v1_values`) et sources inconnues : pas touchées ici.
                None => continue,
            },
        };
        if text_widget.compact
            && text_widget.source == "weapon"
            && reloading
            && !new_text.is_empty()
        {
            new_text.push_str(" · recharge");
        }
        set_text(&mut text, &mut background, text_widget, new_text);
    }
}

// ---------------------------------------------------------------------------------------
// Sources T2.12 : perks, à terre, power-ups actifs, prompt d'achat
// ---------------------------------------------------------------------------------------
//
// Tout se dérive de l'état à chaque `Update` (jamais dans `GgrsSchedule`, aucun état
// rollback) : l'affichage reste juste après un rollback. Le texte est composé par des
// fonctions pures (testées en bas du fichier) ; les systèmes ne font que lire l'ECS.

/// Ce que le joueur du HUD peut lire de lui-même (perks, à terre, power-ups, armes).
#[derive(SystemParam)]
struct HudPlayerState<'w, 's> {
    players: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static fixed_math::FixedTransform3D,
            Option<&'static Perks>,
            Option<&'static Downed>,
            Option<&'static Reviving>,
            Option<&'static Modifiers>,
            Option<&'static WeaponInventory>,
            Option<&'static CharacterConfigKey>,
            Option<&'static items::Inventory>,
        ),
        With<Player>,
    >,
    /// Joueurs à terre en cours de réanimation, pour « Réanimation 40 % » côté réanimateur.
    revivals: Query<'w, 's, (&'static Reviving, &'static CharacterConfigKey), With<Downed>>,
    character_configs: Res<'w, CharacterConfigs>,
}

/// Interactables (même filtre que `interaction::interaction_detection_system`) et ce qu'il
/// faut pour en écrire le prompt.
#[derive(SystemParam)]
struct HudInteractables<'w, 's> {
    interactables: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static fixed_math::FixedTransform3D,
            &'static Interactable,
            Option<&'static Collider>,
            Option<&'static DoorComponent>,
            Option<&'static WindowHealth>,
            Option<&'static WeaponPickup>,
            Option<&'static PerkMachine>,
            Option<&'static items::Chest>,
            Option<&'static items::Shelf>,
            Option<&'static items::ItemPickup>,
        ),
        With<Rollback>,
    >,
    item_table: Option<Res<'w, items::ItemTable>>,
    // F5 (chantier m0-v11) : prix et perks résolus une fois au lancement (`crate::balance`).
    balance: Res<'w, crate::balance::ResolvedBalance>,
    powerups: Res<'w, PowerUpsTable>,
}

fn update_hud_v1_values(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    frame: Res<FrameCount>,
    hud_player: HudPlayer,
    snapshot: Res<HudSnapshot>,
    state: HudPlayerState,
    world: HudInteractables,
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    run: Option<Res<run::Run>>,
    mut text_widgets: Query<(&HudTextWidget, &mut Text, &mut BackgroundColor)>,
    mut icon_widgets: Query<(Entity, &mut HudIconsWidget)>,
) {
    let empty = HudConfig {
        widgets: Vec::new(),
        icons: BTreeMap::new(),
        names: BTreeMap::new(),
        font: None,
        reference_size: None,
        player_colors: Vec::new(),
        legend: Vec::new(),
        hide_markers: false,
        room_icons: BTreeMap::new(),
        wall_art: None,
    };
    let config = config_handle
        .0
        .as_ref()
        .and_then(|h| configs.get(h))
        .unwrap_or(&empty);
    // Run terminée : l'écran de fin (`ui::game_over`) prend le centre ; « À TERRE » d'un
    // joueur qui saigne encore, un prompt ou un power-up n'ont plus de sens par-dessus.
    let ended = run
        .as_deref()
        .is_some_and(|run| matches!(run.step, run::RunStep::Ended { .. }));
    let player = hud_player
        .entity()
        .filter(|_| !ended)
        .and_then(|e| state.players.get(e).ok());
    let frame = frame.frame;

    let perks: Vec<String> = player
        .and_then(|p| p.2)
        .map(|perks| perks.0.iter().cloned().collect())
        .unwrap_or_default();

    let downed = player
        .map(|(net_id, _, _, downed, reviving, _, _, config_key, _)| {
            let revive_frames = |config_key: Option<&CharacterConfigKey>| {
                config_key
                    .and_then(|key| state.character_configs.get(&key.0))
                    .map_or(180, |c| c.revive_frames)
            };
            match downed {
                Some(downed) => downed_text(
                    frame,
                    Some(downed.bleedout_at_frame),
                    reviving.map(|r| (r.progress_frames, revive_frames(config_key))),
                ),
                // Réanimateur : la progression du joueur qu'il relève.
                None => state
                    .revivals
                    .iter()
                    .filter(|(r, _)| r.by == *net_id)
                    .map(|(r, h)| (r.progress_frames, revive_frames(Some(h))))
                    .max()
                    .map_or(String::new(), |progress| {
                        downed_text(frame, None, Some(progress))
                    }),
            }
        })
        .unwrap_or_default();

    let powerup_config = world.powerups.0.as_ref();
    let powerups = player
        .and_then(|p| p.5)
        .map(|modifiers| {
            let active = active_powerups(modifiers, frame);
            powerups_text(&active, frame, |id| {
                powerup_config
                    .and_then(|c| c.powerups.get(id))
                    .map(|def| def.name.clone())
            })
        })
        .unwrap_or_default();

    let prompt = player
        .filter(|p| p.3.is_none())
        .and_then(
            |(_, transform, perks_owned, _, _, _, inventory, _, items)| {
                closest_interactable(&world, transform.translation).map(|target| {
                    prompt_for(
                        &world,
                        config,
                        target,
                        perks_owned,
                        inventory,
                        items,
                        snapshot.values.hides.item_names,
                    )
                })
            },
        )
        .unwrap_or_default();

    for (widget, mut text, mut background) in text_widgets.iter_mut() {
        let value = match widget.source.as_str() {
            "perks" => perks
                .iter()
                .map(|id| icon_for(config, id).1)
                .collect::<Vec<_>>()
                .join(" "),
            "downed" => downed.clone(),
            "powerups" => powerups.clone(),
            "prompt" => prompt.clone(),
            _ => continue,
        };
        // Valeur vide : rien d'affiché, pas même le préfixe (« [H] » sans prompt).
        let new_text = if value.is_empty() {
            String::new()
        } else {
            format!("{}{}", widget.prefix, value)
        };
        set_text(&mut text, &mut background, widget, new_text);
    }

    for (entity, mut widget) in icon_widgets.iter_mut() {
        let is_active_icon = widget.source == "active_item";
        let ids = perks
            .iter()
            .take(if widget.source == "perks" {
                perks.len()
            } else {
                0
            })
            .map(String::as_str)
            .chain(
                snapshot
                    .values
                    .passives
                    .iter()
                    .take(if widget.source == "passives" {
                        snapshot.values.passives.len()
                    } else {
                        0
                    })
                    .map(|(id, _)| id.as_str()),
            )
            // M3-T16 : l'icône de l'objet actif.
            .chain(
                snapshot
                    .values
                    .active_item
                    .iter()
                    .filter(|_| is_active_icon)
                    .map(|item| item.id.as_str()),
            );
        if ids.clone().eq(widget.shown.iter().map(String::as_str)) {
            continue;
        }
        widget.shown = ids.clone().map(str::to_string).collect();
        let size = widget.icon_size;
        let font = widget.font.clone();
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|row| {
                for id in ids {
                    if let Some(path) = config.icons.get(id).and_then(|icon| icon.image.as_deref())
                    {
                        row.spawn((
                            Node {
                                width: Val::Px(size),
                                height: Val::Px(size),
                                ..default()
                            },
                            ImageNode::new(load_icon(&asset_server, path)),
                        ));
                        continue;
                    }
                    let (color, label) = icon_for(config, id);
                    row.spawn((
                        Node {
                            width: Val::Px(size),
                            height: Val::Px(size),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ))
                    .with_children(|icon| {
                        icon.spawn((
                            Text::new(label),
                            hud_text_font(&font, size * 0.6),
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
    }
}

/// Panneau de la légende (F7) ; `built` : ses lignes sont posées pour l'ouverture courante.
#[derive(Component)]
struct HudLegendWidget {
    #[allow(dead_code)]
    source: String,
    built: bool,
    font: Option<Handle<Font>>,
    font_size: f32,
}

/// Légende (F7) : ouvre ou ferme le panneau ; à l'ouverture, une colonne par `column` des groupes de
/// `hud.ron`, chaque ligne montrant l'image du sprite (case `Idle`, teintée) ou une pastille de
/// couleur, puis son texte. Reconstruite à chaque ouverture : le jeu courant fait foi.
#[allow(clippy::too_many_arguments)]
fn update_hud_legend(
    mut commands: Commands,
    open: Res<crate::legibility::LegendOpen>,
    handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    table: crate::world_sprites::SpriteTable,
    items: Option<Res<items::ItemTable>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut widgets: Query<(
        Entity,
        &mut HudLegendWidget,
        &mut Visibility,
        Option<&Children>,
    )>,
) {
    let Some(config) = handle.get(&configs) else {
        return;
    };
    for (entity, mut widget, mut visibility, children) in &mut widgets {
        let wanted = if open.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        if !open.0 {
            widget.built = false;
            continue;
        }
        if widget.built {
            continue;
        }
        widget.built = true;
        for child in children.into_iter().flat_map(|c| c.iter()) {
            commands.entity(child).despawn();
        }
        let columns = config
            .legend
            .iter()
            .map(|g| g.column + 1)
            .max()
            .unwrap_or(0);
        let text_font = hud_text_font(&widget.font, widget.font_size.max(10.0));
        let title_font = hud_text_font(&widget.font, widget.font_size.max(10.0) + 1.0);
        commands.entity(entity).with_children(|panel| {
            for column in 0..columns {
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|col| {
                        for group in config.legend.iter().filter(|g| g.column == column) {
                            col.spawn((
                                Text::new(group.title.clone()),
                                title_font.clone(),
                                TextColor(Color::srgb(1.0, 0.85, 0.3)),
                                Node {
                                    margin: UiRect::top(Val::Px(6.0)),
                                    ..default()
                                },
                            ));
                            for entry in &group.entries {
                                spawn_legend_row(
                                    col,
                                    entry,
                                    &text_font,
                                    &table,
                                    items.as_deref(),
                                    &asset_server,
                                    &mut layouts,
                                );
                            }
                        }
                    });
            }
        });
    }
}

fn spawn_legend_row(
    col: &mut ChildSpawnerCommands,
    entry: &LegendEntry,
    text_font: &TextFont,
    table: &crate::world_sprites::SpriteTable,
    items: Option<&items::ItemTable>,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) {
    col.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        height: Val::Px(22.0),
        ..default()
    })
    .with_children(|row| {
        let tint = entry
            .tint
            .as_deref()
            .map(parse_color)
            .unwrap_or(Color::WHITE);
        let image = entry.sprite.as_deref().and_then(|id| {
            crate::legibility::legend_frame(
                table,
                id,
                entry.anim.as_deref().unwrap_or("Idle"),
                items,
            )
        });
        if let Some((sheet, frame)) = image {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(sheet.tile_size.0, sheet.tile_size.1),
                sheet.columns,
                sheet.rows,
                None,
                None,
            );
            let mut node = ImageNode::new(asset_server.load(&sheet.path)).with_color(tint);
            node.texture_atlas = Some(TextureAtlas {
                layout: layouts.add(layout),
                index: frame,
            });
            row.spawn((
                node,
                Node {
                    width: Val::Px(22.0),
                    height: Val::Px(22.0),
                    ..default()
                },
            ));
        } else if let Some(color) = entry.color.as_deref() {
            row.spawn((
                Node {
                    width: Val::Px(22.0),
                    height: Val::Px(14.0),
                    ..default()
                },
                BackgroundColor(parse_color(color)),
            ));
        } else {
            row.spawn(Node {
                width: Val::Px(22.0),
                ..default()
            });
        }
        row.spawn((
            Text::new(entry.label.clone()),
            text_font.clone(),
            TextColor(Color::WHITE),
        ));
    });
}

fn update_hud_counters(
    snapshot: Res<HudSnapshot>,
    mut rows: Query<(&HudCounterRow, &mut Visibility)>,
) {
    for (row, mut visibility) in &mut rows {
        let shown = snapshot
            .texts
            .get(row.source.as_str())
            .is_some_and(|text| !text.is_empty());
        let next = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

/// Couleurs des cœurs : `icons.heart`, `icons.soul_heart`, `icons.heart_empty` du RON, sinon
/// ces valeurs par défaut.
fn heart_colors(config: &HudConfig) -> (Color, Color, Color) {
    let color = |id: &str, default: &str| {
        parse_color(
            config
                .icons
                .get(id)
                .map_or(default, |icon| icon.color.as_str()),
        )
    };
    (
        color("heart", "#d63031"),
        color("soul_heart", "#74b9ff"),
        color("heart_empty", "#2d1b1b"),
    )
}

/// Image d'une icône (chemin relatif à `assets/`).
fn load_icon(asset_server: &AssetServer, path: &str) -> Handle<Image> {
    asset_server.load(path.to_owned())
}

/// Image d'un cœur (surcharge locale d'un pack) : `heart` / `heart_half` / `soul_heart` /
/// `soul_heart_half` / `heart_empty` de `icons`, selon le type et le remplissage (en demi-cœurs).
/// `None` : un des ids n'a pas d'image, le cœur garde ses carrés de couleur.
fn heart_image(config: &HudConfig, kind: HeartKind, fill: u8) -> Option<&str> {
    let id = match (kind, fill) {
        (_, 0) => "heart_empty",
        (HeartKind::Red, 1) => "heart_half",
        (HeartKind::Soul, 1) => "soul_heart_half",
        (HeartKind::Red, _) => "heart",
        (HeartKind::Soul, _) => "soul_heart",
    };
    config.icons.get(id)?.image.as_deref()
}

/// Cœurs de la source `hearts`, dérivés du snapshot : reconstruits seulement quand ils changent.
fn update_hud_hearts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    snapshot: Res<HudSnapshot>,
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    mut widgets: Query<(Entity, &mut HudHeartsWidget)>,
) {
    let slots = snapshot.values.hearts.map(|hearts| hearts.slots());
    let default_config;
    let config = match config_handle.get(&configs) {
        Some(config) => config,
        None => {
            default_config = HudConfig {
                widgets: Vec::new(),
                icons: BTreeMap::new(),
                names: BTreeMap::new(),
                font: None,
                reference_size: None,
                player_colors: Vec::new(),
                legend: Vec::new(),
                hide_markers: false,
                room_icons: BTreeMap::new(),
                wall_art: None,
            };
            &default_config
        }
    };
    let (red, soul, empty) = heart_colors(config);
    for (entity, mut widget) in &mut widgets {
        if widget.shown == slots {
            continue;
        }
        widget.shown = slots.clone();
        let size = widget.icon_size;
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|row| {
                for slot in slots.iter().flatten() {
                    if let Some(path) = heart_image(config, slot.kind, slot.fill) {
                        row.spawn((
                            Node {
                                width: Val::Px(size),
                                height: Val::Px(size),
                                ..default()
                            },
                            ImageNode::new(load_icon(&asset_server, path)),
                        ));
                        continue;
                    }
                    let fill = match slot.kind {
                        HeartKind::Red => red,
                        HeartKind::Soul => soul,
                    };
                    row.spawn((
                        Node {
                            width: Val::Px(size),
                            height: Val::Px(size),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(empty),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    ))
                    .with_children(|heart| {
                        // Un demi-cœur remplit la moitié gauche.
                        if slot.fill > 0 {
                            heart.spawn((
                                Node {
                                    width: Val::Percent(f32::from(slot.fill) * 50.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(fill),
                            ));
                        }
                    });
                }
            });
    }
}

fn update_hud_map(
    mut commands: Commands,
    snapshot: Res<HudSnapshot>,
    mut widgets: Query<(Entity, &mut HudMapWidget)>,
) {
    for (entity, mut widget) in &mut widgets {
        let rooms = if widget.source == "minimap" {
            snapshot.values.minimap.as_slice()
        } else {
            &[]
        };
        if widget.shown == rooms {
            continue;
        }
        widget.shown = rooms.to_vec();
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|map| {
                for room in rooms {
                    let [x, y, width, height] = room.rect;
                    let fill = match room.state {
                        world::RoomState::Locked => Color::srgb(0.65, 0.12, 0.12),
                        world::RoomState::Cleared => Color::srgb(0.12, 0.42, 0.28),
                        _ if room.visited => Color::srgb(0.27, 0.36, 0.48),
                        _ => Color::srgb(0.08, 0.10, 0.14),
                    };
                    let icon = widget.room_icons.get(&room.kind);
                    let border = if room.current {
                        Color::srgb(1.0, 0.85, 0.2)
                    } else if let Some(icon) = icon {
                        parse_color(&icon.color)
                    } else if room.boss {
                        Color::srgb(0.8, 0.45, 1.0)
                    } else {
                        Color::srgb(0.4, 0.45, 0.5)
                    };
                    map.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(x * 100.0),
                            top: Val::Percent(y * 100.0),
                            width: Val::Percent(width * 100.0),
                            height: Val::Percent(height * 100.0),
                            border: UiRect::all(Val::Px(if room.current { 3.0 } else { 1.0 })),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(fill),
                        BorderColor::all(border),
                    ))
                    .with_children(|tile| {
                        let locked = room.state == world::RoomState::Locked;
                        let label = match (icon, room.boss, locked) {
                            (Some(icon), _, true) => format!("{} ×", icon.label),
                            (Some(icon), _, false) => icon.label.clone(),
                            (None, true, true) => "B ×".to_string(),
                            (None, true, false) => "B".to_string(),
                            (None, false, true) => "×".to_string(),
                            _ => String::new(),
                        };
                        if !label.is_empty() {
                            tile.spawn((
                                Text::new(label),
                                hud_text_font(&widget.font, widget.font_size),
                                TextColor(Color::WHITE),
                            ));
                        }
                    });
                }
            });
    }
}

/// Interactable le plus proche à portée, avec la **même règle** que
/// `interaction::interaction_detection_system` (distance à la surface du collider s'il y en
/// a un, ordre par `GgrsNetId`, le premier gagne à égalité) : le prompt annonce exactement
/// ce que l'appui sur Interaction déclencherait.
fn closest_interactable(
    world: &HudInteractables,
    position: fixed_math::FixedVec3,
) -> Option<GgrsNetId> {
    let mut candidates: Vec<_> = world.interactables.iter().collect();
    candidates.sort_by_key(|(net_id, ..)| net_id.0);
    let mut closest: Option<(fixed_math::FixedWide, GgrsNetId)> = None;
    for (net_id, transform, interactable, collider, ..) in candidates {
        let distance_sq = match collider {
            Some(collider) => {
                point_to_collider_surface_distance_sq(position, transform.translation, collider)
            }
            None => (transform.translation - position).length_squared(),
        };
        let range = fixed_math::FixedWide::from_num(interactable.interaction_range.to_num::<i64>());
        if distance_sq > range.saturating_mul(range) {
            continue;
        }
        if closest.as_ref().is_none_or(|(best, _)| distance_sq < *best) {
            closest = Some((distance_sq, net_id.clone()));
        }
    }
    closest.map(|(_, net_id)| net_id)
}

fn prompt_for(
    world: &HudInteractables,
    config: &HudConfig,
    target: GgrsNetId,
    perks_owned: Option<&Perks>,
    inventory: Option<&WeaponInventory>,
    items: Option<&items::Inventory>,
    hide_names: bool,
) -> String {
    let Some((_, _, interactable, _, door, window, pickup, perk_machine, chest, shelf, item)) =
        world
            .interactables
            .iter()
            .find(|(net_id, ..)| **net_id == target)
    else {
        return String::new();
    };
    let prompt = match interactable.interaction_type {
        InteractionType::Door => Prompt::Door {
            cost: door.map_or(0, |d| d.config.cost),
        },
        InteractionType::Window => Prompt::Window {
            damaged: window.is_some_and(|w| w.current < w.max),
        },
        InteractionType::Revive => Prompt::Revive,
        // M3-T16 : « Ramasser Reliquaire » ; le nom devient « ??? » sous `HideItemNames` (dette T9 :
        // l'objet au sol n'avait pas de prompt, donc rien à cacher).
        InteractionType::Item => {
            let Some(item) = item else {
                return String::new();
            };
            Prompt::Item {
                name: world
                    .item_table
                    .as_ref()
                    .and_then(|table| table.0.get(&item.item_id))
                    .map_or_else(|| item.item_id.clone(), |def| def.name.clone()),
            }
        }
        InteractionType::Chest => {
            let Some(chest) = chest else {
                return String::new();
            };
            Prompt::Chest {
                locked: chest.locked && !chest.opened,
                has_key: items.is_some_and(|i| i.consumable(crate::loot::KEY_ITEM) > 0),
            }
        }
        // M2-T11 : pas de prompt de table avant le HUD de M2 (V4).
        InteractionType::Table => return String::new(),
        // M2-T6 : « Acheter Trèfle — 30 » (nom de l'objet, prix en douilles).
        InteractionType::Shop => {
            let Some(shelf) = shelf.filter(|shelf| !shelf.sold) else {
                return String::new();
            };
            Prompt::Shelf {
                name: world
                    .item_table
                    .as_ref()
                    .and_then(|table| table.0.get(&shelf.item_id))
                    .map_or_else(|| shelf.item_id.clone(), |def| def.name.clone()),
                price: shelf_price_label(shelf, world.item_table.as_deref()),
            }
        }
        // M2-T13 : pas de prompt du hub avant le HUD de M2 (V4).
        InteractionType::Hub => return String::new(),
        // M3-T5 : porte à clé (la clé est consommée à l'ouverture, `key_door_system`).
        InteractionType::KeyDoor => Prompt::KeyDoor {
            keys: items.map_or(0, |i| i.consumable(crate::loot::KEY_ITEM)),
        },
        InteractionType::Weapon => {
            let Some(pickup) = pickup else {
                return String::new();
            };
            let owned = inventory.is_some_and(|inv| {
                inv.weapons
                    .iter()
                    .any(|(_, w)| w.config.name == pickup.weapon_id)
            });
            Prompt::Weapon {
                name: config
                    .names
                    .get(&pickup.weapon_id)
                    .cloned()
                    .unwrap_or_else(|| pickup.weapon_id.clone()),
                price: pickup.price,
                refill_price: pickup.price.map(|p| world.balance.economy.refill_price(p)),
                owned,
            }
        }
        InteractionType::Perk => {
            let Some(machine) = perk_machine else {
                return String::new();
            };
            // F5 (chantier m0-v11) : perk résolu une fois au lancement (`crate::balance`).
            let def = world.balance.perks.get(&machine.perk_id);
            Prompt::Perk {
                name: def.map_or_else(|| machine.perk_id.clone(), |d| d.name.clone()),
                price: def.map(|d| d.price),
                owned: perks_owned.is_some_and(|p| p.0.contains(&machine.perk_id)),
            }
        }
    };
    let prompt = if hide_names {
        prompt.hide_name()
    } else {
        prompt
    };
    prompt.text()
}

/// Prix d'un présentoir pour son prompt (M3-T8, §73) : les douilles (`price`), ou le coût propre du
/// présentoir — « 1 cœur » pour la vie maximale (`items::heart_label`), « 3 Bombe » pour un
/// consommable (nom de l'objet).
fn shelf_price_label(shelf: &items::Shelf, item_table: Option<&items::ItemTable>) -> String {
    match &shelf.cost {
        None => shelf.price.to_string(),
        Some(items::Price::Currency(amount)) => amount.to_string(),
        Some(items::Price::MaxHealth(amount)) => items::heart_label(*amount),
        Some(items::Price::Consumable(id, amount)) => {
            let name = item_table
                .and_then(|table| table.0.get(id))
                .map_or(id.as_str(), |def| def.name.as_str());
            format!("{amount} {name}")
        }
    }
}

/// Ce que le prompt annonce, indépendamment de l'ECS (testé en bas du fichier).
#[derive(Debug, Clone, PartialEq)]
enum Prompt {
    /// `cost <= 0` : porte gratuite.
    Door {
        cost: i32,
    },
    /// Une fenêtre intacte n'a rien à réparer : pas de prompt.
    Window {
        damaged: bool,
    },
    Revive,
    /// `price: None` : arme au sol (gratuite). `owned` : l'achat recharge les munitions au
    /// prix `refill_price` (`crate::balance::ResolvedEconomyConfig::refill_price`).
    Weapon {
        name: String,
        price: Option<u32>,
        refill_price: Option<u32>,
        owned: bool,
    },
    /// M2-T5 : coffre ; `locked` sans clé en main : refus (« Fermé — 1 clé »).
    Chest {
        locked: bool,
        has_key: bool,
    },
    /// M2-T6 : présentoir de boutique.
    Shelf {
        name: String,
        /// Prix tel qu'écrit : des douilles (« 30 ») ou, pour un prix de vie, des cœurs
        /// (« 1 cœur », M3-T8 §73).
        price: String,
    },
    /// `owned` : déjà acheté, l'interaction ne fait rien.
    Perk {
        name: String,
        price: Option<u32>,
        owned: bool,
    },
    /// M3-T5 : porte d'une salle à clé ; `keys` : clés du joueur (M3-T16).
    KeyDoor {
        keys: u32,
    },
    /// M3-T16 : passif ou actif au sol.
    Item {
        name: String,
    },
}

impl Prompt {
    /// M3-T9 (curse `HideItemNames`, §74) : le nom de l'objet ou de l'arme à ramasser ou acheter
    /// devient `???` ; le prix et le reste du texte restent.
    fn hide_name(self) -> Self {
        const HIDDEN: &str = crate::ui::hud_model::HIDDEN_ITEM_NAME;
        match self {
            Prompt::Weapon {
                price,
                refill_price,
                owned,
                ..
            } => Prompt::Weapon {
                name: HIDDEN.to_string(),
                price,
                refill_price,
                owned,
            },
            Prompt::Shelf { price, .. } => Prompt::Shelf {
                name: HIDDEN.to_string(),
                price,
            },
            Prompt::Item { .. } => Prompt::Item {
                name: HIDDEN.to_string(),
            },
            other => other,
        }
    }

    fn text(&self) -> String {
        match self {
            Prompt::Door { cost } if *cost > 0 => format!("Ouvrir — ${cost}"),
            Prompt::Door { .. } => "Ouvrir".to_string(),
            Prompt::Window { damaged: true } => "Réparer".to_string(),
            Prompt::Window { damaged: false } => String::new(),
            Prompt::Revive => "Réanimer".to_string(),
            Prompt::Weapon {
                name,
                price: Some(_),
                refill_price: Some(refill),
                owned: true,
            } => format!("Munitions {name} — ${refill}"),
            Prompt::Weapon {
                name,
                price: Some(price),
                ..
            } => format!("Acheter {name} — ${price}"),
            Prompt::Weapon { name, .. } => format!("Ramasser {name}"),
            Prompt::Shelf { name, price } => format!("Acheter {name} — {price}"),
            Prompt::Chest { locked: false, .. } => "Ouvrir le coffre".to_string(),
            Prompt::Chest { has_key: false, .. } => "Fermé — 1 clé".to_string(),
            Prompt::Chest { .. } => "Ouvrir — 1 clé".to_string(),
            Prompt::Perk {
                name, owned: true, ..
            } => format!("{name} — possédé"),
            Prompt::Perk {
                name,
                price: Some(price),
                ..
            } => format!("{name} — ${price}"),
            Prompt::Perk { name, .. } => name.clone(),
            Prompt::KeyDoor { keys: 0 } => "Fermé — clé (0)".to_string(),
            Prompt::KeyDoor { keys } => format!("Ouvrir — clé ({keys})"),
            Prompt::Item { name } => format!("Ramasser {name}"),
        }
    }
}

/// Secondes affichées pour `frames` restantes, arrondies au-dessus (« 1 s » jusqu'à la
/// dernière frame, jamais « 0 s » tant que l'état dure).
fn seconds_left(frames: u32) -> u32 {
    frames.div_ceil(SIM_FPS as u32)
}

/// Texte de la source `downed` : « À TERRE 12 s » pour un joueur à terre
/// (`bleedout_at_frame`), suivi de « Réanimation 40 % » si une réanimation est en cours
/// (`(progress_frames, revive_frames)`) ; la seconde ligne seule pour un réanimateur.
fn downed_text(frame: u32, bleedout_at_frame: Option<u32>, revive: Option<(u32, u32)>) -> String {
    let mut lines = Vec::new();
    if let Some(bleedout_at) = bleedout_at_frame {
        lines.push(format!(
            "À TERRE {} s",
            seconds_left(bleedout_at.saturating_sub(frame))
        ));
    }
    if let Some((progress, total)) = revive {
        let percent = (progress.saturating_mul(100) / total.max(1)).min(100);
        lines.push(format!("Réanimation {percent} %"));
    }
    lines.join("\n")
}

/// Power-ups actifs d'un joueur : id → frame d'expiration la plus lointaine parmi ses
/// modificateurs `ModifierSource::Named("powerup:<id>")` encore actifs (`frame <= until`,
/// voir `sim_core::modifier::Modifier::is_expired`). Un power-up à plusieurs actions pose
/// plusieurs modificateurs : une seule ligne.
fn active_powerups(modifiers: &Modifiers, frame: u32) -> BTreeMap<String, u32> {
    let mut active = BTreeMap::new();
    for modifier in &modifiers.0 {
        let (ModifierSource::Named(source), Some(until)) = (&modifier.source, modifier.until)
        else {
            continue;
        };
        let Some(id) = source.strip_prefix("powerup:") else {
            continue;
        };
        if until < frame {
            continue;
        }
        let entry = active.entry(id.to_string()).or_insert(until);
        *entry = (*entry).max(until);
    }
    active
}

/// Texte de la source `powerups` : une ligne par power-up actif, « Insta-Kill 24 s », dans
/// l'ordre des ids. `name` donne le nom affiché (`items/powerups.ron`), l'id sinon.
fn powerups_text(
    active: &BTreeMap<String, u32>,
    frame: u32,
    name: impl Fn(&str) -> Option<String>,
) -> String {
    active
        .iter()
        .map(|(id, until)| {
            let left = until.saturating_sub(frame).saturating_add(1);
            format!(
                "{} {} s",
                name(id).unwrap_or_else(|| id.clone()),
                seconds_left(left)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Couleur et étiquette de l'icône d'un id : `hud.ron` (`icons`), sinon gris et initiale
/// en majuscule.
fn icon_for(config: &HudConfig, id: &str) -> (Color, String) {
    match config.icons.get(id) {
        Some(icon) => (parse_color(&icon.color), icon.label.clone()),
        None => (
            Color::srgb(0.4, 0.4, 0.4),
            id.chars()
                .next()
                .map(|c| c.to_uppercase().collect())
                .unwrap_or_default(),
        ),
    }
}

/// Rechargement à chaud : quand `hud.ron` est modifié (feature `native`, file watcher de bevy),
/// l'arbre est détruit ; `spawn_hud_when_ready` le reconstruit à la frame suivante.
fn handle_hud_config_changes(
    mut events: MessageReader<AssetEvent<HudConfig>>,
    config_handle: Res<HudConfigHandle>,
    q_hud_root: Query<Entity, With<HudRoot>>,
    mut commands: Commands,
) {
    let Some(handle) = &config_handle.0 else {
        return;
    };
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            if *id == handle.id() {
                for entity in q_hud_root.iter() {
                    commands.entity(entity).despawn();
                }
                info!("hud.ron rechargé");
            }
        }
    }
}

fn despawn_hud(mut commands: Commands, q_hud_root: Query<Entity, With<HudRoot>>) {
    for entity in q_hud_root.iter() {
        commands.entity(entity).despawn();
    }
}

/// Écrit le texte d'un widget s'il a changé, et montre son fond seulement s'il n'est pas vide.
fn set_text(
    text: &mut Text,
    background: &mut BackgroundColor,
    widget: &HudTextWidget,
    new_text: String,
) {
    let new_text = if widget.compact {
        compact_hud_text(&widget.source, &new_text)
    } else {
        new_text
    };
    let color = text_background(widget.background, &new_text);
    if background.0 != color {
        background.0 = color;
    }
    if text.0 != new_text {
        text.0 = new_text;
    }
}

fn compact_hud_text(source: &str, text: &str) -> String {
    match source {
        "weapon" => {
            let label = text.replace('_', " ");
            let mut chars = label.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        }
        "ammo" => text.replace(" | ", " / "),
        "statuses" | "powerups" => {
            let lines: Vec<_> = text.lines().collect();
            let mut visible: Vec<_> = lines
                .iter()
                .take(2)
                .map(|line| line.replace('_', " "))
                .collect();
            if lines.len() > 2 {
                visible.push(format!("+{} effets", lines.len() - 2));
            }
            visible.join("\n")
        }
        _ => text.to_string(),
    }
}

fn text_background(background: Option<Color>, text: &str) -> Color {
    match background {
        Some(color) if !text.is_empty() => color,
        _ => Color::NONE,
    }
}

/// `#rrggbb` ou `#rrggbbaa` (T2.12 : alpha, pour les fonds).
pub(crate) fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    match (hex.len(), u32::from_str_radix(hex, 16)) {
        (6, Ok(val)) => {
            let r = ((val >> 16) & 0xFF) as f32 / 255.0;
            let g = ((val >> 8) & 0xFF) as f32 / 255.0;
            let b = (val & 0xFF) as f32 / 255.0;
            Color::srgb(r, g, b)
        }
        (8, Ok(val)) => {
            let r = ((val >> 24) & 0xFF) as f32 / 255.0;
            let g = ((val >> 16) & 0xFF) as f32 / 255.0;
            let b = ((val >> 8) & 0xFF) as f32 / 255.0;
            let a = (val & 0xFF) as f32 / 255.0;
            Color::srgba(r, g, b, a)
        }
        _ => Color::WHITE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temporary_effects_keep_durations_and_count_hidden_entries() {
        assert_eq!(
            compact_hud_text(
                "statuses",
                "brulure ×2 · 3 s\npoison · 8 s\ngel · 2 s\nlent · 5 s"
            ),
            "brulure ×2 · 3 s\npoison · 8 s\n+2 effets"
        );
        assert_eq!(compact_hud_text("powerups", ""), "");
        assert_eq!(compact_hud_text("ammo", "4 | 912"), "4 / 912");
        assert_eq!(
            compact_hud_text("weapon", "lance_grenades"),
            "Lance grenades"
        );
    }

    #[test]
    fn compact_layout_fits_small_viewports_and_preserves_large_ones() {
        assert_eq!(hud_scale(Vec2::new(960.0, 540.0), (800.0, 540.0)), 1.0);
        let scale = hud_scale(Vec2::new(640.0, 360.0), (800.0, 540.0));
        assert!(684.0 * scale < 640.0, "all three mutation cards must fit");
        assert!(
            340.0 * scale < 360.0,
            "title/cards/hint must fit vertically"
        );
        assert_eq!(hud_scale(Vec2::ZERO, (800.0, 540.0)), 1.0);
    }
    use crate::ui::hud_model::HudPlayerValues;
    use bevy_fixed::fixed_math::new as fx;
    use sim_core::modifier::{Modifier, ModifierOp};
    use sim_core::stats::StatId;

    #[test]
    fn downed_text_counts_seconds_up_and_shows_revive_progress() {
        // 12 s pile, puis 11 s et une frame : arrondi au-dessus.
        assert_eq!(downed_text(100, Some(820), None), "À TERRE 12 s");
        assert_eq!(downed_text(100, Some(819), None), "À TERRE 12 s");
        assert_eq!(downed_text(100, Some(101), None), "À TERRE 1 s");
        assert_eq!(
            downed_text(100, Some(820), Some((72, 180))),
            "À TERRE 12 s\nRéanimation 40 %"
        );
        assert_eq!(downed_text(100, None, Some((90, 180))), "Réanimation 50 %");
        assert_eq!(downed_text(100, None, None), "");
    }

    fn powerup_modifier(id: &str, until: Option<u32>) -> Modifier {
        Modifier {
            stat: StatId::Damage,
            op: ModifierOp::Set,
            value: fx(100.0),
            source: ModifierSource::Named(format!("powerup:{id}")),
            until,
        }
    }

    #[test]
    fn active_powerups_keeps_live_powerup_modifiers_only() {
        let mut perk = powerup_modifier("x", None);
        perk.source = ModifierSource::Named("perk:juggernog".into());
        let modifiers = Modifiers(vec![
            powerup_modifier("insta_kill", Some(1810)),
            powerup_modifier("insta_kill", Some(1800)),
            powerup_modifier("double_points", Some(99)),
            powerup_modifier("permanent", None),
            perk,
        ]);
        let active = active_powerups(&modifiers, 100);
        assert_eq!(active.len(), 1);
        assert_eq!(active.get("insta_kill"), Some(&1810));
        // Actif jusqu'à `until` compris.
        assert_eq!(active_powerups(&modifiers, 1810).len(), 1);
        assert!(active_powerups(&modifiers, 1811).is_empty());
    }

    #[test]
    fn powerups_text_one_line_per_powerup_with_name() {
        let mut active = BTreeMap::new();
        active.insert("insta_kill".to_string(), 1810);
        active.insert("double_points".to_string(), 1300);
        let text = powerups_text(&active, 370, |id| {
            (id == "insta_kill").then(|| "Insta-Kill".to_string())
        });
        // 1810 - 370 + 1 = 1441 frames -> 25 s ; 931 frames -> 16 s.
        assert_eq!(text, "double_points 16 s\nInsta-Kill 25 s");
        assert_eq!(powerups_text(&BTreeMap::new(), 0, |_| None), "");
    }

    #[test]
    fn prompt_texts() {
        assert_eq!(Prompt::Door { cost: 750 }.text(), "Ouvrir — $750");
        assert_eq!(Prompt::Door { cost: 0 }.text(), "Ouvrir");
        assert_eq!(Prompt::Window { damaged: true }.text(), "Réparer");
        assert_eq!(Prompt::Window { damaged: false }.text(), "");
        assert_eq!(Prompt::Revive.text(), "Réanimer");
        assert_eq!(Prompt::KeyDoor { keys: 2 }.text(), "Ouvrir — clé (2)");
        assert_eq!(Prompt::KeyDoor { keys: 0 }.text(), "Fermé — clé (0)");
        assert_eq!(
            Prompt::Item {
                name: "Reliquaire".into()
            }
            .text(),
            "Ramasser Reliquaire"
        );
        let weapon = |price, owned| Prompt::Weapon {
            name: "fusil à pompe".into(),
            price,
            refill_price: price.map(|p: u32| p / 2),
            owned,
        };
        assert_eq!(
            weapon(Some(1000), false).text(),
            "Acheter fusil à pompe — $1000"
        );
        assert_eq!(
            weapon(Some(1000), true).text(),
            "Munitions fusil à pompe — $500"
        );
        assert_eq!(weapon(None, false).text(), "Ramasser fusil à pompe");
        let perk = |owned| Prompt::Perk {
            name: "Juggernog".into(),
            price: Some(2500),
            owned,
        };
        assert_eq!(perk(false).text(), "Juggernog — $2500");
        assert_eq!(perk(true).text(), "Juggernog — possédé");
        // M2-T5 : coffres.
        let chest = |locked, has_key| Prompt::Chest { locked, has_key };
        assert_eq!(
            Prompt::Shelf {
                name: "Trèfle".into(),
                price: "30".into()
            }
            .text(),
            "Acheter Trèfle — 30"
        );
        // M3-T8 : prix de vie du marché du diable, en cœurs (2 PV = 1 cœur).
        let devil = |points: i32| items::Shelf {
            key: "k".into(),
            item_id: "x".into(),
            price: 0,
            sold: false,
            retry_after: 0,
            cost: Some(items::Price::MaxHealth(
                bevy_fixed::fixed_math::Fixed::from_num(points),
            )),
        };
        let label = |points| shelf_price_label(&devil(points), None);
        assert_eq!(label(2), "1 cœur");
        assert_eq!(label(4), "2 cœurs");
        assert_eq!(label(1), "½ cœur");
        assert_eq!(
            Prompt::Shelf {
                name: "Lentille".into(),
                price: label(2),
            }
            .text(),
            "Acheter Lentille — 1 cœur"
        );
        assert_eq!(chest(false, false).text(), "Ouvrir le coffre");
        assert_eq!(chest(true, false).text(), "Fermé — 1 clé");
        assert_eq!(chest(true, true).text(), "Ouvrir — 1 clé");
    }

    #[test]
    fn blind_curse_hides_the_name_of_pickups_and_keeps_the_rest() {
        let weapon = Prompt::Weapon {
            name: "fusil à pompe".into(),
            price: Some(1000),
            refill_price: Some(500),
            owned: false,
        };
        assert_eq!(weapon.clone().hide_name().text(), "Acheter ??? — $1000");
        let owned = Prompt::Weapon {
            name: "fusil à pompe".into(),
            price: Some(1000),
            refill_price: Some(500),
            owned: true,
        };
        assert_eq!(owned.hide_name().text(), "Munitions ??? — $500");
        let free = Prompt::Weapon {
            name: "fusil à pompe".into(),
            price: None,
            refill_price: None,
            owned: false,
        };
        assert_eq!(free.hide_name().text(), "Ramasser ???");
        let shelf = Prompt::Shelf {
            name: "Trèfle".into(),
            price: "30".into(),
        };
        assert_eq!(shelf.hide_name().text(), "Acheter ??? — 30");
        // M3-T16 : l'objet au sol a un prompt, donc un nom à cacher.
        let item = Prompt::Item {
            name: "Reliquaire".into(),
        };
        assert_eq!(item.hide_name().text(), "Ramasser ???");
        // Une porte, un coffre ou une machine à perk n'ont pas de nom d'objet : inchangés.
        for prompt in [
            Prompt::Door { cost: 750 },
            Prompt::Chest {
                locked: true,
                has_key: true,
            },
            Prompt::Perk {
                name: "Juggernog".into(),
                price: Some(2500),
                owned: false,
            },
        ] {
            assert_eq!(prompt.clone().hide_name().text(), prompt.text());
        }
    }

    #[test]
    fn sources_du_joueur_vides_quand_il_est_mort() {
        // D22 : joueur mort (entité détruite) : aucune valeur, rien d'affiché, pas même le
        // préfixe (« $ », « Vie : »).
        let dead = HudPlayerValues::default();
        for source in ["health", "ammo", "weapon", "currency"] {
            assert_eq!(
                player_source_text(source, "$", &dead),
                Some(String::new()),
                "{source}"
            );
        }
        let alive = HudPlayerValues {
            health: Some((80, 100)),
            currency: Some(1500),
            weapon: Some(("pistol".to_string(), Some(6), 48)),
            ..Default::default()
        };
        assert_eq!(
            player_source_text("currency", "$", &alive).as_deref(),
            Some("$1500")
        );
        assert_eq!(
            player_source_text("ammo", "", &alive).as_deref(),
            Some("6 | 48")
        );
        assert_eq!(
            player_source_text("health", "", &alive).as_deref(),
            Some("80/100")
        );
        assert_eq!(
            player_source_text("weapon", "", &alive).as_deref(),
            Some("pistol")
        );
        // Arme sans mode actif lisible : rien (avant : « ? | ? »).
        let no_mode = HudPlayerValues {
            weapon: Some(("pistol".to_string(), None, 48)),
            ..alive.clone()
        };
        assert_eq!(
            player_source_text("ammo", "", &no_mode),
            Some(String::new())
        );
        // Source hors joueur : pas traitée ici.
        assert_eq!(player_source_text("wave", "", &alive), None);
    }

    #[test]
    fn parse_color_accepts_alpha() {
        assert_eq!(parse_color("#ff0000"), Color::srgb(1.0, 0.0, 0.0));
        assert_eq!(
            parse_color("#00000080"),
            Color::srgba(0.0, 0.0, 0.0, 128.0 / 255.0)
        );
        assert_eq!(parse_color("pas une couleur"), Color::WHITE);
    }

    /// La surcharge locale remplace `wall_art` en bloc, ajoute ou remplace les icônes par id et garde les autres.
    #[test]
    fn the_local_overlay_replaces_wall_art_and_merges_icons_by_id() {
        let base = r##"(
            widgets: [],
            icons: {
                "juggernog": (color: "#c0392b", label: "J"),
                "speed_cola": (color: "#27ae60", label: "S"),
            },
        )"##;
        let overlay = r##"(
            wall_art: Some((layer: "Walls", atlas: "sprites/postapo/atlas_murs.png", floor: Some((tiles: [36, 37])))),
            icons: { "juggernog": (color: "#c0392b", label: "J", image: Some("sprites/postapo/hud/juggernog.png")) },
        )"##;
        let mut config: HudConfig = hud_ron_options().from_str(base).expect("hud public");
        assert!(config.wall_art.is_none());
        config.apply_overlay(hud_ron_options().from_str(overlay).expect("surcharge"));
        let art = config.wall_art.as_ref().expect("wall_art posé");
        assert_eq!((art.tile, art.columns), (16, 8));
        let floor = art.floor.as_ref().expect("sol");
        assert_eq!((floor.tiles.as_slice(), floor.z), (&[36, 37][..], 0.05));
        assert_eq!(
            config.icons["juggernog"].image.as_deref(),
            Some("sprites/postapo/hud/juggernog.png")
        );
        // Un id absent de la surcharge garde son carré de couleur.
        assert!(config.icons["speed_cola"].image.is_none());
        // Une surcharge vide ne change rien.
        config.apply_overlay(HudOverlay::default());
        assert!(config.wall_art.is_some() && config.icons.len() == 2);
    }

    /// Un cœur n'a d'image que si son type et son remplissage en ont une : sinon les carrés de couleur.
    #[test]
    fn a_heart_image_needs_every_state_it_draws() {
        let mut config: HudConfig = hud_ron_options()
            .from_str("(widgets: [])")
            .expect("hud vide");
        let icon = |path: &str| HudIcon {
            color: "#ffffff".into(),
            label: String::new(),
            image: Some(path.into()),
        };
        config.icons.insert("heart".into(), icon("h.png"));
        config.icons.insert("heart_half".into(), icon("hh.png"));
        assert_eq!(heart_image(&config, HeartKind::Red, 2), Some("h.png"));
        assert_eq!(heart_image(&config, HeartKind::Red, 1), Some("hh.png"));
        assert_eq!(heart_image(&config, HeartKind::Red, 0), None);
        assert_eq!(heart_image(&config, HeartKind::Soul, 2), None);
    }

    #[test]
    fn icon_falls_back_to_grey_initial() {
        let mut config = HudConfig {
            widgets: Vec::new(),
            icons: BTreeMap::new(),
            names: BTreeMap::new(),
            font: None,
            reference_size: None,
            player_colors: Vec::new(),
            legend: Vec::new(),
            hide_markers: false,
            room_icons: BTreeMap::new(),
            wall_art: None,
        };
        config.icons.insert(
            "juggernog".into(),
            HudIcon {
                color: "#c0392b".into(),
                label: "J".into(),
                image: None,
            },
        );
        assert_eq!(icon_for(&config, "juggernog").1, "J");
        assert_eq!(icon_for(&config, "speed_cola").1, "S");
    }

    #[test]
    fn zombies_hud_ron_parses_with_new_sources() {
        let text = include_str!("../../../../games/zombies/assets/ui/hud.ron");
        let config: HudConfig = ron::from_str(text).expect("hud.ron zombies");
        for widget in &config.widgets {
            let source = match &widget.kind {
                HudWidgetKind::Bar { source }
                | HudWidgetKind::Text { source, .. }
                | HudWidgetKind::Icons { source }
                | HudWidgetKind::Map { source }
                | HudWidgetKind::Legend { source }
                | HudWidgetKind::Hearts { source }
                | HudWidgetKind::Counter { source } => source,
            };
            assert!(
                SOURCES.contains(&source.as_str()),
                "source inconnue {source}"
            );
        }
        for source in ["perks", "downed", "powerups", "prompt"] {
            assert!(
                config.widgets.iter().any(|w| matches!(
                    &w.kind,
                    HudWidgetKind::Text { source: s, .. } | HudWidgetKind::Icons { source: s }
                        if s == source
                )),
                "widget {source} absent de hud.ron"
            );
        }
    }

    #[test]
    fn gungeon_hud_ron_partage_le_schema_et_affiche_les_objets_et_la_carte() {
        let text = include_str!("../../../../games/gungeon/assets/ui/hud.ron");
        let config: HudConfig = ron::from_str(text).expect("HUD gungeon");
        let schema: content::ui::HudFileSchema = ron::from_str(text).expect("schéma HUD");
        for widget in &schema.widgets {
            assert!(SOURCES.contains(&widget.kind.source()));
        }
        for source in [
            "blanks",
            "keys",
            "shells",
            "active_item",
            "active_charge",
            "passives",
            "minimap",
        ] {
            assert!(
                schema
                    .widgets
                    .iter()
                    .any(|widget| widget.kind.source() == source),
                "{source}"
            );
        }
        assert!(config.widgets.iter().any(|widget| matches!(
            &widget.kind, HudWidgetKind::Map { source } if source == "minimap"
        )));
        assert!(config.widgets.iter().any(|widget| matches!(
            &widget.kind, HudWidgetKind::Icons { source } if source == "passives"
        )));
        assert!(config.widgets.iter().any(|widget| matches!(
            &widget.kind, HudWidgetKind::Bar { source } if source == "active_charge"
        )));
        assert_eq!(icon_for(&config, "amulette").1, "A");
        assert_eq!(icon_for(&config, "bottes").1, "B");
    }

    #[test]
    fn isaac_hud_ron_montre_cœurs_compteurs_actif_bandeau_et_icones_de_salle() {
        let text = include_str!("../../../../games/isaac/assets/ui/hud.ron");
        let config: HudConfig = ron::from_str(text).expect("HUD isaac");
        for source in ["hearts", "coins", "keys", "bombs", "item_banner", "minimap"] {
            assert!(
                config.widgets.iter().any(|widget| match &widget.kind {
                    HudWidgetKind::Hearts { source: s }
                    | HudWidgetKind::Counter { source: s }
                    | HudWidgetKind::Text { source: s, .. }
                    | HudWidgetKind::Map { source: s } => s == source,
                    _ => false,
                }),
                "widget {source} absent de hud.ron"
            );
        }
        assert!(config.widgets.iter().any(|widget| matches!(
            &widget.kind, HudWidgetKind::Icons { source } if source == "active_item"
        )));
        for kind in ["tresor", "boutique", "boss", "secrete", "diable"] {
            assert!(config.room_icons.contains_key(kind), "{kind}");
        }
        // Chaque compteur a son icône, chaque cœur sa couleur.
        for id in [
            "coins",
            "keys",
            "bombs",
            "heart",
            "soul_heart",
            "heart_empty",
        ] {
            assert!(config.icons.contains_key(id), "{id}");
        }
        let schema: content::ui::HudFileSchema = ron::from_str(text).expect("schéma");
        for widget in &schema.widgets {
            assert!(SOURCES.contains(&widget.kind.source()));
        }
    }
}
