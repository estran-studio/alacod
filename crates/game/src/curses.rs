//! Curses d'étage en jeu (M3-T9, `docs/conventions.md` §74) : le tirage à l'entrée de chaque
//! étage et l'exécution des cinq effets de Q7. Les contrats et règles pures sont dans
//! `run::curse` (§64).
//!
//! # Tirage
//!
//! [`curse_system`] (`GgrsSchedule`, `RollbackSystemSet::FrameCounter`, donc **après** le passage
//! d'étage du set `Run`) compare l'étage courant (`FloorState::index`) au dernier étage traité
//! (`RunCurses::entered`, rollback) : à l'entrée dans un étage, y compris le premier, il tire la
//! curse par [`run::draw_curse`] (RNG local de graine `curse_seed(graine, étage)`, jamais
//! `RngStreams`) et l'écrit dans `RunCurses`. `FloorEntered` n'est émis que quand les horloges ou
//! la difficulté sont activées et jamais pour le premier étage : le tirage détecte donc lui-même
//! l'entrée, à la même frame. Sans champ `curses` dans la séquence ([`CurseTable::config`] vide),
//! le système ne fait rien : aucune ressource ni aucun composant ne change.
//!
//! # Effets
//!
//! - `Modifier` : posé sur chaque joueur, source `curse:<id>`, retiré à l'entrée de l'étage
//!   suivant ([`sync_curse_modifiers`]) ; un joueur qui n'a pas ses modificateurs (apparu plus tard)
//!   les reçoit à la frame suivante.
//! - `FloorRooms` : lu **à la génération de l'étage** (`map_ldtk::loader::setup_generated_map`,
//!   [`floor_rooms_for_slot`]), car toutes les cartes de la séquence sont assemblées au chargement
//!   de la partie, avant la première frame. Le tirage ne dépendant que de la graine **de carte**
//!   ([`curse_map_seed`], pas `RunSeed`) et de l'étage, le planificateur retrouve exactement la
//!   curse que `curse_system` écrira à l'entrée, y compris après un restart en ligne ou une
//!   entrée du hub (où `RunSeed` est dérivée).
//! - `HideMinimap`, `HideHealth`, `HideItemNames` : présentation, dérivée de `RunCurses` à chaque
//!   `Update` ([`CurseHides`], lue par `ui::hud_model` et `ui::hud`) ; jamais d'un événement.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use bevy_ggrs::GgrsSchedule;
use content::registry::{FloorsConfigId, Registry};
use map::generation::config::MapGenerationConfig;
use run::{
    curse::is_curse_source, draw_curse, floor_rooms_multiplier, CurseConfig, CurseDef, CurseEffect,
    FloorState, RunCurses, RunMode,
};
use sim_core::modifier::{Modifier, Modifiers};
use utils::{frame::FrameCount, net_id::GgrsNetId, order_mut_iter};

use crate::character::player::Player;
use crate::system_set::RollbackSystemSet;

/// Contenu des curses de la partie, hors rollback : posé au chargement (`map_ldtk::game::floors::
/// compute_floor_plan`), vide hors du mode `Floors` et pour une séquence sans champ `curses`.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct CurseTable {
    /// Champ `curses` de la séquence `Floors` jouée ; `None` : aucune curse, rien ne tourne.
    pub config: Option<CurseConfig>,
    /// Définitions du pool (`curses/<id>.ron`), par id.
    pub defs: BTreeMap<String, CurseDef>,
}

impl CurseTable {
    /// La table du mode joué : champ `curses` de sa séquence `Floors` et les curses du registre
    /// (toutes : le lint garantit que le pool y est). Vide hors `Floors`.
    pub fn for_mode(mode: &RunMode, registry: Option<&Registry>) -> Self {
        let (RunMode::Floors { config }, Some(registry)) = (mode, registry) else {
            return Self::default();
        };
        let Some(curses) = registry
            .floors
            .get(&FloorsConfigId::from(config.clone()))
            .and_then(|entry| entry.curses.clone())
        else {
            return Self::default();
        };
        Self {
            config: Some(curses),
            defs: registry
                .curses
                .iter()
                .map(|(id, entry)| (id.clone(), entry.def.clone()))
                .collect(),
        }
    }

    /// La curse de l'étage `floor` d'une partie de graine de carte `seed` : le tirage pur de
    /// `run::draw_curse`, `None` sans champ `curses`. Même résultat qu'à l'entrée dans l'étage.
    pub fn drawn(&self, seed: u32, floor: u32) -> Option<(String, &CurseDef)> {
        let id = draw_curse(self.config.as_ref()?, &self.defs, seed, floor)?;
        let def = self.defs.get(&id)?;
        Some((id, def))
    }

    /// Définition de la curse `id`.
    pub fn def(&self, id: &str) -> Option<&CurseDef> {
        self.defs.get(id)
    }
}

/// Multiplicateur de salles (`FloorRooms`) à appliquer à la grammaire du `floor`-ième étage d'une
/// partie de graine de carte `seed` ([`curse_map_seed`]) ; `None` sans curse ou sans effet
/// `FloorRooms`. Appelé par le chargeur de cartes pour l'étage de chaque emplacement de la séquence.
pub fn floor_rooms_for_slot(table: &CurseTable, seed: u32, floor: u32) -> Option<Fixed> {
    let (_, def) = table.drawn(seed, floor)?;
    floor_rooms_multiplier(&def.effects)
}

/// Ce que la curse de l'étage courant cache au joueur (présentation seule).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CurseHides {
    /// `HideMinimap` : la minicarte n'affiche aucune salle.
    pub minimap: bool,
    /// `HideHealth` : ni la barre ni le texte de santé.
    pub health: bool,
    /// `HideItemNames` : le nom des objets et armes à ramasser ou acheter devient `???`.
    pub item_names: bool,
}

impl CurseHides {
    /// Ce que cachent les effets d'une curse.
    pub fn of(effects: &[CurseEffect]) -> Self {
        let has = |wanted: &CurseEffect| effects.contains(wanted);
        Self {
            minimap: has(&CurseEffect::HideMinimap),
            health: has(&CurseEffect::HideHealth),
            item_names: has(&CurseEffect::HideItemNames),
        }
    }
}

/// Curse de l'étage `floor` d'après l'état de la partie : son id et sa définition.
pub fn current_curse<'a>(
    curses: &'a RunCurses,
    table: &'a CurseTable,
    floor: u32,
) -> Option<(&'a str, &'a CurseDef)> {
    let id = curses.get(floor)?;
    Some((id, table.def(id)?))
}

/// Nombre de modificateurs de stat qu'une curse pose.
fn modifier_count(def: &CurseDef) -> usize {
    def.effects
        .iter()
        .filter(|effect| matches!(effect, CurseEffect::Modifier(..)))
        .count()
}

/// Aligne les modificateurs de curse de `modifiers` sur ceux de la curse `current` (`(id,
/// définition)`, `None` : aucune) : retire tous ceux de source `curse:*` puis pose ceux de la
/// curse, dans l'ordre de ses effets. Les autres modificateurs ne bougent pas.
pub fn sync_curse_modifiers(modifiers: &mut Modifiers, current: Option<(&str, &CurseDef)>) {
    modifiers
        .0
        .retain(|modifier| !is_curse_source(&modifier.source));
    if let Some((id, def)) = current {
        modifiers.0.extend(
            def.effects
                .iter()
                .filter_map(|effect| effect.to_modifier(id)),
        );
    }
}

/// Nombre de modificateurs de curse portés (sans allocation).
fn curse_modifiers_held(modifiers: &Modifiers) -> usize {
    modifiers
        .iter()
        .filter(|modifier: &&Modifier| is_curse_source(&modifier.source))
        .count()
}

/// Graine des tirages de curses : la graine de **carte** (`MapGenerationConfig::seed`, jamais
/// modifiée en cours de partie), la même expression pour la génération des étages (`FloorRooms`,
/// `map_ldtk::loader::with_curse_floor_rooms`) et pour `curse_system`. `RunSeed` ne convient pas : la
/// session la remplace par `restart_seed` après un « Rejouer » en ligne ou une entrée du hub (§33), alors
/// que les cartes gardent la graine de carte ; deux graines feraient annoncer « Labyrinth » sur un étage
/// de taille ordinaire. Sans configuration de carte : 0.
pub fn curse_map_seed(config: Option<&MapGenerationConfig>) -> u32 {
    config.map_or(0, |config| config.seed as u32)
}

/// Tire la curse de l'étage à son entrée et pose ses modificateurs sur les joueurs
/// (`RollbackSystemSet::FrameCounter`, voir le module). Sans `CurseTable::config`, ne fait rien.
pub fn curse_system(
    frame: Res<FrameCount>,
    table: Res<CurseTable>,
    floor: Option<Res<FloorState>>,
    map_config: Option<Res<MapGenerationConfig>>,
    mut curses: ResMut<RunCurses>,
    mut players: Query<(&GgrsNetId, &mut Modifiers), With<Player>>,
) {
    if table.config.is_none() {
        return;
    }
    let floor_index = floor.map_or(0, |floor| floor.index);
    let entered_now = curses.entered != Some(floor_index);
    if entered_now {
        let drawn = table
            .drawn(curse_map_seed(map_config.as_deref()), floor_index)
            .map(|(id, _)| id);
        info!(
            "ggrs{{f={} curse floor={} id={}}}",
            frame.frame,
            floor_index,
            drawn.as_deref().unwrap_or("none")
        );
        curses.enter(floor_index, drawn);
    }
    let current = current_curse(&curses, &table, floor_index);
    let expected = current.map_or(0, |(_, def)| modifier_count(def));
    for (_, mut modifiers) in order_mut_iter!(players) {
        // À l'entrée dans un étage : retire ceux de l'étage quitté. Ensuite : seulement si un
        // joueur n'a pas le compte attendu (apparu après l'entrée). Rien d'écrit sinon.
        if entered_now || curse_modifiers_held(&modifiers) != expected {
            sync_curse_modifiers(&mut modifiers, current);
        }
    }
}

/// Enregistre le tirage des curses (`GgrsSchedule`) ; `RunCurses` l'est déjà (`run::RunPlugin`).
pub struct CursesPlugin;

impl Plugin for CursesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurseTable>().add_systems(
            GgrsSchedule,
            curse_system
                .before(crate::frame::increase_frame_system)
                .in_set(RollbackSystemSet::FrameCounter),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::modifier::{ModifierOp, ModifierSource};
    use sim_core::stats::StatId;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn maze() -> CurseDef {
        CurseDef {
            weight: 1,
            effects: vec![CurseEffect::Modifier(
                StatId::MoveSpeed,
                ModifierOp::Mul,
                fx(0.85),
            )],
        }
    }

    fn other_modifier() -> Modifier {
        Modifier {
            stat: StatId::Damage,
            op: ModifierOp::Add,
            value: fx(1.0),
            source: ModifierSource::Named("item:bottes".into()),
            until: None,
        }
    }

    #[test]
    fn hides_follow_the_effects() {
        assert_eq!(CurseHides::of(&[]), CurseHides::default());
        assert_eq!(
            CurseHides::of(&[CurseEffect::HideMinimap]),
            CurseHides {
                minimap: true,
                ..default()
            }
        );
        assert_eq!(
            CurseHides::of(&[CurseEffect::HideHealth, CurseEffect::HideItemNames]),
            CurseHides {
                health: true,
                item_names: true,
                ..default()
            }
        );
        // Un modificateur ou un agrandissement ne cache rien.
        assert_eq!(
            CurseHides::of(&maze().effects),
            CurseHides::default(),
            "Modifier ne cache rien"
        );
        assert_eq!(
            CurseHides::of(&[CurseEffect::FloorRooms(fx(2.0))]),
            CurseHides::default()
        );
    }

    #[test]
    fn sync_poses_the_curse_modifiers_and_replaces_the_previous_ones() {
        let mut modifiers = Modifiers(vec![other_modifier()]);
        sync_curse_modifiers(&mut modifiers, Some(("maze", &maze())));
        assert_eq!(modifiers.0.len(), 2);
        assert_eq!(
            modifiers.0[1].source,
            ModifierSource::Named("curse:maze".into())
        );
        // Resynchroniser avec la même curse ne duplique rien.
        sync_curse_modifiers(&mut modifiers, Some(("maze", &maze())));
        assert_eq!(curse_modifiers_held(&modifiers), 1);
        // Autre curse : l'ancienne disparaît ; aucune : tout disparaît, le reste est intact.
        let slow = CurseDef {
            weight: 1,
            effects: vec![CurseEffect::Modifier(
                StatId::Luck,
                ModifierOp::Add,
                fx(-1.0),
            )],
        };
        sync_curse_modifiers(&mut modifiers, Some(("lost", &slow)));
        assert_eq!(curse_modifiers_held(&modifiers), 1);
        assert!(modifiers.iter().all(|m| m.stat != StatId::MoveSpeed));
        sync_curse_modifiers(&mut modifiers, None);
        assert_eq!(modifiers.0, vec![other_modifier()]);
    }

    #[test]
    fn drawn_matches_the_pure_rule_and_floor_rooms_reads_the_same_draw() {
        let mut defs = BTreeMap::new();
        defs.insert(
            "labyrinth".to_string(),
            CurseDef {
                weight: 1,
                effects: vec![CurseEffect::FloorRooms(fx(2.0))],
            },
        );
        defs.insert("maze".to_string(), maze());
        let table = CurseTable {
            config: Some(CurseConfig {
                chance: fx(0.5),
                pool: vec!["labyrinth".into(), "maze".into()],
            }),
            defs,
        };
        let mut doubled = 0;
        for floor in 0..200 {
            let by_table = table.drawn(7, floor).map(|(id, _)| id);
            let by_rule = draw_curse(table.config.as_ref().unwrap(), &table.defs, 7, floor);
            assert_eq!(by_table, by_rule);
            // Le multiplicateur de la grammaire vient du même tirage que l'entrée d'étage.
            let rooms = floor_rooms_for_slot(&table, 7, floor);
            assert_eq!(rooms.is_some(), by_table.as_deref() == Some("labyrinth"));
            doubled += usize::from(rooms == Some(fx(2.0)));
        }
        assert!(doubled > 20, "{doubled} étages agrandis sur 200");
        // Sans champ `curses` : jamais rien.
        assert_eq!(floor_rooms_for_slot(&CurseTable::default(), 7, 0), None);
        assert_eq!(CurseTable::default().drawn(7, 0), None);
    }

    #[test]
    fn current_curse_ignores_a_curse_missing_from_the_table() {
        let mut curses = RunCurses::default();
        curses.set(0, "maze");
        let mut table = CurseTable::default();
        assert_eq!(current_curse(&curses, &table, 0), None);
        table.defs.insert("maze".into(), maze());
        assert_eq!(
            current_curse(&curses, &table, 0).map(|(id, _)| id),
            Some("maze")
        );
        assert_eq!(current_curse(&curses, &table, 1), None);
    }

    /// Revue de M3-T9 : après un « Rejouer » en ligne ou une entrée du hub, `RunSeed` est dérivée
    /// (`restart_seed`) alors que les cartes gardent la graine de carte. Le tirage de l'entrée d'étage
    /// doit suivre la graine de carte, celle qu'a lue la génération (`FloorRooms`), jamais `RunSeed`.
    #[test]
    fn the_draw_follows_the_map_seed_not_the_derived_run_seed() {
        let mut defs = BTreeMap::new();
        defs.insert("a".to_string(), maze());
        defs.insert("b".to_string(), maze());
        let table = CurseTable {
            config: Some(CurseConfig {
                chance: Fixed::ONE,
                pool: vec!["a".into(), "b".into()],
            }),
            defs,
        };
        let by = |seed: u32| table.drawn(seed, 0).map(|(id, _)| id);
        // Deux graines qui tirent des curses différentes à l'étage 0.
        let map_seed = 1u32;
        let run_seed = (2..200u32)
            .find(|seed| by(*seed) != by(map_seed))
            .expect("une graine qui tire autrement");
        let mut app = App::new();
        app.init_resource::<FrameCount>()
            .init_resource::<RunCurses>()
            .insert_resource(table.clone())
            .insert_resource(MapGenerationConfig {
                seed: map_seed as i32,
                ..default()
            })
            .insert_resource(bevy_fixed::rng::RunSeed(run_seed))
            .add_systems(Update, curse_system);
        app.update();
        let curses = app.world().resource::<RunCurses>();
        assert_eq!(curses.get(0).map(str::to_string), by(map_seed));
        assert_ne!(curses.get(0).map(str::to_string), by(run_seed));
        assert_eq!(curse_map_seed(None), 0);
    }
}
