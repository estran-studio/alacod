//! Progression v1 : rads, niveaux et mutations (T1.10, chantier C4 v1, `docs/conventions.md`
//! §27).
//!
//! La progression est **opt-in** : active seulement si le manifeste la nomme
//! (`entry.progression`) ou si un scénario l'impose (`Scenario::progression`,
//! [`ProgressionOverride`]). Sans elle, [`ProgressionTable`] est vide et aucun système ne pose
//! ni ne tire rien (flux `loot` intact, traces inchangées).
//!
//! Active, chaque joueur reçoit (composants rollback, checksum **neutre**) : [`Gauges`] (la
//! jauge `gauge` de la progression), [`Level`], [`Mutations`], et `Effects`/`EffectState` vides
//! s'il n'en a pas. [`progression_system`] tourne dans `DeathManagement` juste après
//! `apply_effects_system` (les `GaugeAdd` de la frame comptent) :
//!
//! 1. chaque mort attribuée à un joueur ajoute `per_kill` à sa jauge ;
//! 2. un choix ouvert ([`MutationChoice`]) se résout : bouton `ChoiceA/B/C` (bits 13–15), ou la
//!    première option à `since_frame + choice_frames` ; la mutation choisie ajoute ses effets,
//!    `Mutations` la note, et `OnLevelUp` se déclenche à la frame suivante ;
//! 3. niveau = seuils `levels` atteints ; chaque niveau gagné ouvre un choix (tirage de
//!    `choices` mutations dans le flux `loot`, joueurs par `GgrsNetId`) ou s'empile
//!    (`pending`) sur le choix déjà ouvert. La simulation ne se met jamais en pause.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use bevy_fixed::rng::RngStreams;
use bevy_ggrs::{PlayerInputs, Rollback};
use combat::actors::{INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C};
use content::manifest::GameManifest;
use content::registry::Registry;
use effects::progression::{draw_options, level_for, resolve_choice, Candidate};
use effects::Effect;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;
use utils::order_mut_iter;

use crate::character::health::Death;
use crate::character::player::jjrs::PeerConfig;
use crate::character::player::Player;
use crate::effects_runtime::{
    kills_by_killer, DeadQuery, EffectState, Effects, Gauges, OthersQuery, PendingTrigger,
};

/// Progression imposée par un scénario (`Scenario::progression`) : id d'un fichier du kind
/// `Progression`, prioritaire sur `entry.progression` du manifeste.
#[derive(Resource, Clone, Debug)]
pub struct ProgressionOverride(pub String);

/// Progression résolue au lancement (hors rollback, identique sur tous les clients).
#[derive(Clone, Debug)]
pub struct ProgressionDef {
    pub id: String,
    pub gauge: String,
    pub per_kill: Fixed,
    pub levels: Vec<Fixed>,
    pub choices: u32,
    pub choice_frames: u32,
    /// Pool tiré au `LevelUp` (ordre des ids).
    pub pool: Vec<Candidate>,
    /// Pool d'armes par niveau (`level`, armes), drop à la mort.
    pub weapon_pool: Vec<(u32, Vec<String>)>,
    pub weapon_drop_chance: Fixed,
}

/// Progression active de la partie et effets de chaque mutation du jeu. Les mutations sont
/// connues même sans progression : un scénario peut en imposer (`PlayerScript::mutations`).
#[derive(Resource, Clone, Debug, Default)]
pub struct ProgressionTable {
    pub active: Option<ProgressionDef>,
    pub mutations: BTreeMap<String, Vec<Effect>>,
}

/// Niveau du joueur (nombre de seuils de la jauge atteints).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Level(pub u32);

/// Mutations prises, dans l'ordre (une mutation à plusieurs piles apparaît plusieurs fois).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mutations(pub Vec<String>);

/// Choix de mutation ouvert.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MutationChoice {
    pub options: Vec<String>,
    pub since_frame: u32,
    /// Niveaux gagnés pendant ce choix : chacun rouvre un choix après celui-ci.
    pub pending: u32,
    /// Aucun bouton de choix tenu depuis l'ouverture (voir `resolve_choice`).
    pub armed: bool,
}

/// `OnEnter(AppState::GameLoading)` : résout la progression active et les mutations depuis le
/// registre (comme `crate::patterns`).
pub fn resolve_progression_system(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    manifest: Option<Res<GameManifest>>,
    progression_override: Option<Res<ProgressionOverride>>,
) {
    let table = registry
        .map(|registry| {
            let wanted = progression_override
                .map(|o| o.0.clone())
                .or_else(|| manifest.and_then(|m| m.entry.progression.clone()));
            build_progression_table(&registry, wanted.as_deref())
        })
        .unwrap_or_default();
    commands.insert_resource(table);
}

/// Table de progression du registre ; `wanted` : id de la progression active (`None` : aucune).
pub fn build_progression_table(registry: &Registry, wanted: Option<&str>) -> ProgressionTable {
    let mutations: BTreeMap<String, Vec<Effect>> = registry
        .mutations
        .iter()
        .map(|(id, m)| (id.as_str().to_string(), m.effects.clone()))
        .collect();
    let active = wanted.and_then(|wanted| {
        let Some(entry) = registry.progression.values().find(|p| p.id.as_str() == wanted) else {
            warn!("progression « {wanted} » absente du registre (voir `alacod lint`)");
            return None;
        };
        let pool = registry
            .mutations
            .values()
            .filter(|m| entry.mutations.is_empty() || entry.mutations.contains(&m.id.to_string()))
            .map(|m| Candidate {
                id: m.id.to_string(),
                weight: m.weight,
                max_stacks: m.max_stacks,
            })
            .collect();
        Some(ProgressionDef {
            id: entry.id.to_string(),
            gauge: entry.gauge.clone(),
            per_kill: entry.per_kill,
            levels: entry.levels.clone(),
            choices: entry.choices,
            choice_frames: entry.choice_frames,
            pool,
            weapon_pool: entry
                .weapon_pool
                .iter()
                .map(|w| (w.level, w.weapons.clone()))
                .collect(),
            weapon_drop_chance: entry.weapon_drop_chance,
        })
    });
    ProgressionTable { active, mutations }
}

/// Jauge vide de la progression (bornée à `[0, Fixed::MAX]`, sans seuil : le niveau est lu
/// par `level_for`).
pub fn new_gauge() -> sim_core::gauge::Gauge {
    sim_core::gauge::Gauge {
        value: Fixed::ZERO,
        min: Fixed::ZERO,
        max: Fixed::MAX,
        floor: Fixed::ZERO,
        thresholds: Vec::new(),
        shared: None,
    }
}

fn progression_active(table: Option<Res<ProgressionTable>>) -> bool {
    table.is_some_and(|t| t.active.is_some())
}

/// Pose les composants de progression sur les joueurs qui ne les ont pas encore.
pub fn init_progression_players(
    mut commands: Commands,
    table: Res<ProgressionTable>,
    players: Query<Entity, (With<Player>, With<Rollback>, Without<Level>)>,
) {
    let Some(def) = &table.active else {
        return;
    };
    for entity in players.iter() {
        let mut gauges = Gauges::default();
        gauges.0.insert(def.gauge.clone(), new_gauge());
        commands
            .entity(entity)
            .insert((gauges, Level(0)))
            .insert_if_new((
                Mutations::default(),
                Effects::default(),
                EffectState::default(),
            ));
    }
}

type ProgressionPlayers<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Entity,
        &'static Player,
        &'static mut Gauges,
        &'static mut Level,
        &'static mut Mutations,
        &'static mut Effects,
        &'static mut EffectState,
        Option<&'static mut MutationChoice>,
        Has<Death>,
    ),
    With<Rollback>,
>;

/// Voir la doc du module.
#[allow(clippy::too_many_arguments)]
pub fn progression_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<ProgressionTable>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    mut rng: ResMut<RngStreams>,
    mut players: ProgressionPlayers,
    dead: DeadQuery,
    others: OthersQuery,
) {
    let Some(def) = &table.active else {
        return;
    };
    let frame = frame.frame;
    let kills = kills_by_killer(&dead, &others);

    for (
        net_id,
        entity,
        player,
        mut gauges,
        mut level,
        mut mutations,
        mut effects,
        mut state,
        choice,
        dead_now,
    ) in order_mut_iter!(players)
    {
        if dead_now {
            continue;
        }
        // 1. Rads des kills de la frame
        let killed = kills.get(&net_id.0).map_or(0, Vec::len);
        for _ in 0..killed {
            gauges.add(&def.gauge, def.per_kill);
        }

        // 2. Choix ouvert
        let mut choice = choice.map(|c| c.clone());
        if let Some(open) = choice.as_mut() {
            let buttons = inputs[player.handle].0.buttons;
            let pressed = [INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C]
                .iter()
                .position(|bit| buttons & bit != 0);
            let picked = resolve_choice(
                pressed,
                open.armed,
                open.options.len(),
                open.since_frame,
                def.choice_frames,
                frame,
            );
            if pressed.is_none() {
                open.armed = true;
            }
            if let Some(index) = picked {
                let id = open.options[index].clone();
                info!(
                    "ggrs{{f={} mutation player={} id={} options={:?}}}",
                    frame, player.handle, id, open.options
                );
                mutations.0.push(id.clone());
                effects
                    .0
                    .extend(table.mutations.get(&id).cloned().unwrap_or_default());
                state.pending.push(PendingTrigger::LevelUp);
                if open.pending > 0 {
                    // Niveau suivant en attente : nouveau tirage (mutation prise comptée)
                    let remaining = open.pending;
                    choice = open_choice(def, &mutations, &mut state, &mut rng, frame, remaining);
                } else {
                    choice = None;
                }
            }
        }

        // 3. Niveaux gagnés
        let value = gauges.0.get(&def.gauge).map_or(Fixed::ZERO, |g| g.value);
        let reached = level_for(&def.levels, value);
        if reached > level.0 {
            let gained = reached - level.0;
            info!(
                "ggrs{{f={} levelup player={} level={} gauge={}}}",
                frame, player.handle, reached, value
            );
            level.0 = reached;
            match choice.as_mut() {
                Some(open) => open.pending += gained,
                None => choice = open_choice(def, &mutations, &mut state, &mut rng, frame, gained),
            }
        }

        // Écriture du choix
        match choice {
            Some(open) => {
                commands.entity(entity).insert(open);
            }
            None => {
                commands.entity(entity).remove::<MutationChoice>();
            }
        }
    }
}

/// Ouvre un choix pour `levels` niveaux à résoudre (le premier tout de suite, les autres en
/// `pending`). Pool épuisé : pas de choix, `OnLevelUp` se déclenche pour chaque niveau.
fn open_choice(
    def: &ProgressionDef,
    mutations: &Mutations,
    state: &mut EffectState,
    rng: &mut RngStreams,
    frame: u32,
    levels: u32,
) -> Option<MutationChoice> {
    let options = draw_options(&def.pool, &mutations.0, def.choices, |n| {
        rng.get_mut("loot").next_u32_range(0, n)
    });
    if options.is_empty() {
        for _ in 0..levels {
            state.pending.push(PendingTrigger::LevelUp);
        }
        return None;
    }
    Some(MutationChoice {
        options,
        since_frame: frame,
        pending: levels - 1,
        armed: false,
    })
}

/// Drop d'arme à la mort (T1.10, décision 8) : chaque ennemi mort dans la frame (ordre
/// `GgrsNetId`, comme les power-ups) tire `weapon_drop_chance` dans le flux `loot` ; gagné, une
/// arme est tirée parmi celles du `weapon_pool` dont `level ≤` niveau **max** des joueurs, et
/// posée à terre (`spawn_weapon_pickup`, chargeur plein, gratuite). Rien sans progression ou à
/// chance 0 (aucun tirage).
#[allow(clippy::too_many_arguments)]
pub fn weapon_drop_on_death_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<ProgressionTable>,
    mut rng: ResMut<RngStreams>,
    global_assets: Option<Res<crate::global_asset::GlobalAsset>>,
    weapons_asset: Res<Assets<crate::weapons::WeaponsConfig>>,
    mut id_factory: ResMut<utils::net_id::GgrsNetIdFactory>,
    levels: Query<&Level, With<Player>>,
    dying: Query<
        (
            &GgrsNetId,
            &sim_core::team::Team,
            &bevy_fixed::fixed_math::FixedTransform3D,
        ),
        (With<Rollback>, Added<Death>),
    >,
) {
    let Some(def) = &table.active else {
        return;
    };
    if def.weapon_drop_chance <= Fixed::ZERO || def.weapon_pool.is_empty() {
        return;
    }
    let Some(weapons) = global_assets
        .as_ref()
        .and_then(|g| weapons_asset.get(&g.weapons))
    else {
        return;
    };
    let max_level = levels.iter().map(|l| l.0).max().unwrap_or(0);
    let candidates = effects::progression::weapon_candidates(&def.weapon_pool, max_level);
    for (net_id, team, transform) in utils::order_iter!(dying) {
        if *team != sim_core::team::Team::Enemies {
            continue;
        }
        if rng.get_mut("loot").next_fixed() >= def.weapon_drop_chance || candidates.is_empty() {
            continue;
        }
        let pick = rng.get_mut("loot").next_u32_range(0, candidates.len() as u32) as usize;
        let id = candidates[pick];
        let Some(asset) = weapons.0.get(id) else {
            warn!("weapon_pool : arme « {id} » inconnue (voir `alacod lint`)");
            continue;
        };
        info!(
            "ggrs{{f={} weapon_drop from={} weapon={}}}",
            frame.frame, net_id, id
        );
        let mag_ammo = combat::weapons::default_mode_capacity(asset);
        combat::weapons::spawn_weapon_pickup(
            &mut commands,
            asset.clone().into(),
            mag_ammo,
            transform.translation,
            None,
            &mut id_factory,
        );
    }
}

/// Enregistre les composants (rollback, checksum neutre) et place les systèmes.
pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        use utils::rollback::RollbackTraceApp;
        app.init_resource::<ProgressionTable>()
            .rollback_and_trace_neutral::<Level>()
            .rollback_and_trace_neutral::<Mutations>()
            .rollback_and_trace_neutral::<MutationChoice>()
            .add_systems(
                bevy_ggrs::GgrsSchedule,
                (
                    init_progression_players,
                    progression_system,
                    weapon_drop_on_death_system,
                )
                    .chain()
                    .run_if(progression_active)
                    .after(crate::effects_runtime::apply_effects_system)
                    // Flux `loot` : drops de power-ups d'abord, puis tirages des mutations
                    .after(crate::powerups::loot_drop_on_death_system)
                    .before(crate::character::health::rollback_apply_bleedout)
                    .in_set(sim_core::system_set::RollbackSystemSet::DeathManagement),
            );
    }
}
