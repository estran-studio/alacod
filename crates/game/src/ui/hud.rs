use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
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

use crate::camera::CameraFollowOverride;
use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::enemy::Enemy;
use crate::character::health::Health;
use crate::character::player::{LocalPlayer, Player};
use crate::collider::Collider;
use crate::core::{AppState, SIM_FPS};
use crate::economy::{EconomyConfig, PerkMachine, PerksConfig};
use crate::global_asset::GlobalAsset;
use crate::interaction::{point_to_collider_surface_distance_sq, Interactable, InteractionType};
use crate::powerups::PowerUpsConfig;
use crate::waves::state::WaveState;
use crate::weapons::{WeaponInventory, WeaponModesState, WeaponPickup, WeaponState};

/// HUD Root marker component
#[derive(Component)]
pub struct HudRoot;

/// Widget marker components for updates
#[derive(Component)]
pub struct HudBarWidget {
    pub source: String,
    pub full_width: f32,
}

#[derive(Component)]
pub struct HudTextWidget {
    pub source: String,
    /// Texte fixe devant la valeur (« Vague »), gardé ici : le texte affiché est recomposé.
    pub prefix: String,
    /// Fond derrière le texte (T2.12, `HudWidget::background`), affiché seulement quand le
    /// texte n'est pas vide.
    pub background: Option<Color>,
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

/// Les sources que le HUD sait lire ; une autre dans le RON déclenche un `warn!` au chargement.
/// T2.12 : `perks`, `downed`, `powerups`, `prompt` (voir [`update_hud_v1_values`]).
const SOURCES: &[&str] = &[
    "health", "wave", "ammo", "weapon", "enemies", "players", "currency", "perks", "downed",
    "powerups", "prompt",
];

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
}

/// Icône d'un id de contenu (T2.12, perks) : pas de sprite, un carré de couleur et une
/// courte étiquette (l'initiale). Un id absent de [`HudConfig::icons`] prend un carré gris
/// et la première lettre de son id en majuscule (voir [`icon_for`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudIcon {
    pub color: String,
    pub label: String,
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
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<HudConfig>::new(&["ron"]))
            .init_resource::<HudConfigHandle>()
            .add_systems(OnEnter(AppState::InGame), load_hud_config)
            .add_systems(
                Update,
                spawn_hud_when_ready.run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, update_hud_values.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                update_hud_v1_values.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_hud_config_changes.run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_hud);
    }
}

/// Resource to hold the HUD config asset handle
#[derive(Resource, Default)]
struct HudConfigHandle(Option<Handle<HudConfig>>);

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
                spawn_hud_widgets(&mut commands, entity, config, font);
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
fn hud_text_font(font: &Option<Handle<Font>>, size: f32) -> TextFont {
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
                | HudWidgetKind::Icons { source } => source,
            };
            if !SOURCES.contains(&source.as_str()) {
                warn!("hud.ron : source inconnue « {source} » (connues : {SOURCES:?})");
            }
            let node = anchor_to_node(widget.anchor, widget.offset, size);
            let font_size = widget.font_size.unwrap_or(16.0);
            let color = parse_color(widget.color.as_deref().unwrap_or("#ffffff"));

            match &widget.kind {
                HudWidgetKind::Bar { source } => {
                    parent.spawn((
                        node,
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        HudBarWidget {
                            source: source.clone(),
                            full_width: size.0,
                        },
                    ));
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
                        },
                    ));
                }
            }
        }
    });
}

/// Joueur dont le HUD affiche l'état (T2.12) : celui que la caméra suit de force
/// (`CameraFollowOverride`, `play_scenario --follow <handle>`), sinon le joueur local de plus
/// petit handle. Avant T2.12 : le premier `LocalPlayer` rendu par la query, arbitraire quand
/// plusieurs joueurs sont locaux (scénarios, local multi-joueurs).
#[derive(SystemParam)]
struct HudPlayer<'w, 's> {
    players: Query<'w, 's, (Entity, &'static Player, Has<LocalPlayer>)>,
    follow_override: Option<Res<'w, CameraFollowOverride>>,
}

impl HudPlayer<'_, '_> {
    fn entity(&self) -> Option<Entity> {
        if let Some(handle) = self.follow_override.as_deref().map(|o| o.0) {
            if let Some((entity, _, _)) = self.players.iter().find(|(_, p, _)| p.handle == handle) {
                return Some(entity);
            }
        }
        self.players
            .iter()
            .filter(|(_, _, local)| *local)
            .min_by_key(|(_, player, _)| player.handle)
            .map(|(entity, _, _)| entity)
    }
}

/// Update HUD values from game state
fn update_hud_values(
    hud_player: HudPlayer,
    healths: Query<&Health, With<Player>>,
    // T2.3, chantier C5 v1 : solde de monnaie du joueur du HUD.
    currency_query: Query<&run::currency::Currency>,
    all_players: Query<(), With<Player>>,
    enemies: Query<(), With<Enemy>>,
    wave_state: Res<WaveState>,
    inventories: Query<(&WeaponInventory, &combat::inventory::AmmoReserves)>,
    weapons_query: Query<(&WeaponState, &WeaponModesState)>,
    mut bar_widgets: Query<
        (&HudBarWidget, &mut Node, &mut BackgroundColor),
        Without<HudTextWidget>,
    >,
    mut text_widgets: Query<
        (&HudTextWidget, &mut Text, &mut BackgroundColor),
        Without<HudBarWidget>,
    >,
) {
    let player = hud_player.entity();

    // Gather game state
    let health_info = player
        .and_then(|e| healths.get(e).ok())
        .map(|health| (health.current, health.max));

    let enemy_count = enemies.iter().count();
    let player_count = all_players.iter().count();
    let wave_num = wave_state.current_wave;
    let currency = player.and_then(|e| currency_query.get(e).ok()).map(|c| c.0);

    // L'arme active vient de `WeaponInventory` (rollback), comme dans `weapons/ui.rs`. La
    // réserve (T2.2, `combat::inventory::AmmoReserves`) remplace l'ancien `mag_quantity`
    // par arme pour l'affichage « chargeur / réserve ».
    let weapon_info =
        player
            .and_then(|e| inventories.get(e).ok())
            .and_then(|(inventory, reserves)| {
                let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
                let name = weapon.config.name.clone();
                let reserve = reserves.get(&weapon.config.ammo_type);
                let mode = weapons_query
                    .get(*entity)
                    .ok()
                    .and_then(|(state, modes)| modes.modes.get(&state.active_mode).cloned());
                Some((name, mode, reserve))
            });

    // Update bar widgets
    for (bar_widget, mut node, mut bg_color) in bar_widgets.iter_mut() {
        match bar_widget.source.as_str() {
            "health" => {
                if let Some((current, max)) = health_info {
                    let ratio = if max > fixed_math::FIXED_ZERO {
                        (current.to_num::<f32>()) / (max.to_num::<f32>())
                    } else {
                        0.0
                    };
                    let ratio = ratio.clamp(0.0, 1.0);

                    // Update bar width based on ratio
                    node.width = Val::Px(bar_widget.full_width * ratio);

                    // Color gradient: red to green
                    let r = (1.0 - ratio).max(0.0);
                    let g = ratio.max(0.0);
                    *bg_color = BackgroundColor(Color::srgba(r, g, 0.0, 0.8));
                }
            }
            _ => {
                // Silently ignore unknown sources
            }
        }
    }

    let values = HudPlayerValues {
        health: health_info.map(|(current, max)| (current.to_num::<i32>(), max.to_num::<i32>())),
        currency,
        weapon: weapon_info.map(|(name, mode, reserve)| (name, mode.map(|m| m.mag_ammo), reserve)),
    };

    // Update text widgets
    for (text_widget, mut text, mut background) in text_widgets.iter_mut() {
        let prefix_text = text_widget.prefix.clone();
        let new_text = match text_widget.source.as_str() {
            "wave" => format!("{}{}", prefix_text, wave_num),
            "enemies" => format!("{}{}", prefix_text, enemy_count),
            "players" => format!("{}{}", prefix_text, player_count),
            source => match player_source_text(source, &prefix_text, &values) {
                Some(text) => text,
                // Sources T2.12 (`update_hud_v1_values`) et sources inconnues : pas touchées ici.
                None => continue,
            },
        };
        set_text(&mut text, &mut background, text_widget, new_text);
    }
}

/// Valeurs du joueur du HUD lues par [`update_hud_values`] ; `None` partout quand ce
/// joueur n'existe plus (mort : son entité est détruite) ou n'a pas encore d'arme.
#[derive(Debug, Default, Clone, PartialEq)]
struct HudPlayerValues {
    /// `(actuelle, max)`.
    health: Option<(i32, i32)>,
    currency: Option<u32>,
    /// `(nom, munitions du chargeur du mode actif, réserve du type de munition)`.
    weapon: Option<(String, Option<u32>, u32)>,
}

/// Texte d'une source du joueur (`health`, `ammo`, `weapon`, `currency`) ; `None` pour une
/// autre source. D22 : sans valeur (joueur mort, pas d'arme), **rien**, pas même le préfixe :
/// avant, le HUD d'un joueur mort affichait « $ » sans montant et « ? | ? » (comparaison de
/// T3.2, f1120 d'`idle`). Même règle que les sources T2.12 (`perks`, `downed`…).
fn player_source_text(source: &str, prefix: &str, values: &HudPlayerValues) -> Option<String> {
    let value = match source {
        "health" => values
            .health
            .map(|(current, max)| format!("{current}/{max}")),
        "ammo" => values
            .weapon
            .as_ref()
            .and_then(|(_, mag, reserve)| mag.map(|mag| format!("{mag} | {reserve}"))),
        "weapon" => values.weapon.as_ref().map(|(name, _, _)| name.clone()),
        "currency" => values.currency.map(|amount| amount.to_string()),
        _ => return None,
    };
    Some(value.map_or_else(String::new, |value| format!("{prefix}{value}")))
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
            Option<&'static CharacterConfigHandles>,
        ),
        With<Player>,
    >,
    /// Joueurs à terre en cours de réanimation, pour « Réanimation 40 % » côté réanimateur.
    revivals: Query<'w, 's, (&'static Reviving, &'static CharacterConfigHandles), With<Downed>>,
    character_configs: Res<'w, Assets<CharacterConfig>>,
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
        ),
        With<Rollback>,
    >,
    global_assets: Option<Res<'w, GlobalAsset>>,
    perks_configs: Res<'w, Assets<PerksConfig>>,
    economy_configs: Res<'w, Assets<EconomyConfig>>,
    powerups_configs: Res<'w, Assets<PowerUpsConfig>>,
}

fn update_hud_v1_values(
    mut commands: Commands,
    frame: Res<FrameCount>,
    hud_player: HudPlayer,
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
        .map(|(net_id, _, _, downed, reviving, _, _, handles)| {
            let revive_frames = |handles: Option<&CharacterConfigHandles>| {
                handles
                    .and_then(|h| state.character_configs.get(&h.config))
                    .map_or(180, |c| c.revive_frames)
            };
            match downed {
                Some(downed) => downed_text(
                    frame,
                    Some(downed.bleedout_at_frame),
                    reviving.map(|r| (r.progress_frames, revive_frames(handles))),
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

    let powerup_config = world
        .global_assets
        .as_deref()
        .and_then(|g| g.powerups_config.as_ref())
        .and_then(|h| world.powerups_configs.get(h));
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
        .and_then(|(_, transform, perks_owned, _, _, _, inventory, _)| {
            closest_interactable(&world, transform.translation)
                .map(|target| prompt_for(&world, config, target, perks_owned, inventory))
        })
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
        let ids: &[String] = match widget.source.as_str() {
            "perks" => &perks,
            _ => &[],
        };
        if widget.shown == ids {
            continue;
        }
        widget.shown = ids.to_vec();
        let size = widget.icon_size;
        let font = widget.font.clone();
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|row| {
                for id in ids {
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
) -> String {
    let Some((_, _, interactable, _, door, window, pickup, perk_machine)) = world
        .interactables
        .iter()
        .find(|(net_id, ..)| **net_id == target)
    else {
        return String::new();
    };
    let global = world.global_assets.as_deref();
    let prompt = match interactable.interaction_type {
        InteractionType::Door => Prompt::Door {
            cost: door.map_or(0, |d| d.config.cost),
        },
        InteractionType::Window => Prompt::Window {
            damaged: window.is_some_and(|w| w.current < w.max),
        },
        InteractionType::Revive => Prompt::Revive,
        InteractionType::Weapon => {
            let Some(pickup) = pickup else {
                return String::new();
            };
            let owned = inventory.is_some_and(|inv| {
                inv.weapons
                    .iter()
                    .any(|(_, w)| w.config.name == pickup.weapon_id)
            });
            let refill_ratio = global
                .and_then(|g| g.economy_config.as_ref())
                .and_then(|h| world.economy_configs.get(h));
            Prompt::Weapon {
                name: config
                    .names
                    .get(&pickup.weapon_id)
                    .cloned()
                    .unwrap_or_else(|| pickup.weapon_id.clone()),
                price: pickup.price,
                refill_price: pickup
                    .price
                    .map(|p| refill_ratio.map_or(p, |economy| economy.refill_price(p))),
                owned,
            }
        }
        InteractionType::Perk => {
            let Some(machine) = perk_machine else {
                return String::new();
            };
            let def = global
                .and_then(|g| g.perks_config.as_ref())
                .and_then(|h| world.perks_configs.get(h))
                .and_then(|c| c.0.get(&machine.perk_id));
            Prompt::Perk {
                name: def.map_or_else(|| machine.perk_id.clone(), |d| d.name.clone()),
                price: def.map(|d| d.price),
                owned: perks_owned.is_some_and(|p| p.0.contains(&machine.perk_id)),
            }
        }
    };
    prompt.text()
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
    /// prix `refill_price` (`EconomyConfig::refill_price`).
    Weapon {
        name: String,
        price: Option<u32>,
        refill_price: Option<u32>,
        owned: bool,
    },
    /// `owned` : déjà acheté, l'interaction ne fait rien.
    Perk {
        name: String,
        price: Option<u32>,
        owned: bool,
    },
}

impl Prompt {
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
            Prompt::Perk {
                name, owned: true, ..
            } => format!("{name} — possédé"),
            Prompt::Perk {
                name,
                price: Some(price),
                ..
            } => format!("{name} — ${price}"),
            Prompt::Perk { name, .. } => name.clone(),
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
    let color = text_background(widget.background, &new_text);
    if background.0 != color {
        background.0 = color;
    }
    if text.0 != new_text {
        text.0 = new_text;
    }
}

fn text_background(background: Option<Color>, text: &str) -> Color {
    match background {
        Some(color) if !text.is_empty() => color,
        _ => Color::NONE,
    }
}

/// `#rrggbb` ou `#rrggbbaa` (T2.12 : alpha, pour les fonds).
fn parse_color(hex: &str) -> Color {
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

    #[test]
    fn icon_falls_back_to_grey_initial() {
        let mut config = HudConfig {
            widgets: Vec::new(),
            icons: BTreeMap::new(),
            names: BTreeMap::new(),
            font: None,
        };
        config.icons.insert(
            "juggernog".into(),
            HudIcon {
                color: "#c0392b".into(),
                label: "J".into(),
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
                | HudWidgetKind::Icons { source } => source,
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
}
