//! Hub (M2-T13, `docs/conventions.md` §57) : la salle où l'on revient entre les runs.
//!
//! - **Stations** : une entité LDtk `HubStation` (champ `service`) devient une [`HubStation`]
//!   rollback (neutre) munie d'un `Interactable { Hub }` ; son service vient de `hub/<id>.ron`
//!   ([`HubServices`]). Une station dont le service est fermé pour le profil de la session n'est
//!   pas posée (`map_ldtk::game::local`).
//! - **Décisions dans la simulation** : [`hub_station_system`] écrit [`HubState`] (ressource
//!   rollback neutre) quand un joueur utilise une station : `EnterRun` pose [`HubExit`],
//!   `Merchant` enregistre une [`HubVisit`]. Rien d'autre n'est écrit : l'achat et le lancement de
//!   la run sont hors simulation et ne lisent `HubState` qu'une fois sa frame **confirmée**
//!   (`jjrs::hub`).

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use meta::{HubAction, HubServiceDef};
use sim_core::frame_events::FrameEvents;
use sim_core::interaction::InteractionType;
use std::collections::BTreeMap;
use utils::frame::FrameCount;

use crate::interaction::InteractionEvent;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;

/// Station du hub (entité rollback, neutre) : le service qu'elle offre (`hub/<id>.ron`).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HubStation {
    pub service: String,
}

/// Services du jeu par identifiant (contenu, hors rollback ; vide hors jeu avec dossier
/// `HubService`).
#[derive(Resource, Clone, Debug, Default)]
pub struct HubServices(pub BTreeMap<String, HubServiceDef>);

/// Sortie du hub décidée par un joueur : la run démarre quand cette frame est confirmée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HubExit {
    pub frame: u32,
    pub handle: usize,
}

/// Dernier service utilisé par un joueur (le marchand s'ouvre pour lui).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HubVisit {
    pub service: String,
    pub frame: u32,
}

/// État du hub dans la simulation : rollback **neutre** (défaut = vide = contribution 0, les
/// traces des jeux sans hub ne bougent pas).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HubState {
    pub exit: Option<HubExit>,
    pub visits: BTreeMap<usize, HubVisit>,
}

/// Une utilisation de station : le premier joueur (ordre `GgrsNetId`, déjà celui des
/// événements) l'emporte pour la sortie ; un joueur qui utilise le marchand l'ouvre pour lui.
pub fn hub_station_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    services: Res<HubServices>,
    stations: Query<&HubStation>,
    players: Query<&crate::character::player::Player>,
    mut state: ResMut<HubState>,
) {
    for event in events.iter() {
        if event.interaction_type != InteractionType::Hub {
            continue;
        }
        let (Ok(station), Ok(player)) = (
            stations.get(event.interactable),
            players.get(event.interactor),
        ) else {
            continue;
        };
        let Some(def) = services.0.get(&station.service) else {
            continue;
        };
        match def.action {
            HubAction::EnterRun => {
                if state.exit.is_none() {
                    state.exit = Some(HubExit {
                        frame: frame.frame,
                        handle: player.handle,
                    });
                }
            }
            HubAction::Merchant => {
                state.visits.insert(
                    player.handle,
                    HubVisit {
                        service: station.service.clone(),
                        frame: frame.frame,
                    },
                );
            }
        }
    }
}

/// Où en est la session : au hub ou dans la run (hors rollback, survit aux relances). Posée au
/// lancement du jeu quand le manifeste déclare `entry.hub` (`in_hub: true`), ou par un scénario
/// (`hub: true`). Absente : pas de hub, comportement d'avant M2-T13.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct HubSession {
    /// Carte du hub (`entry.hub.map`).
    pub hub_map: String,
    /// Carte de la run (`entry.start_map`).
    pub run_map: String,
    pub in_hub: bool,
}

impl HubSession {
    /// Carte à charger pour la phase courante.
    pub fn current_map(&self) -> &str {
        if self.in_hub {
            &self.hub_map
        } else {
            &self.run_map
        }
    }
}

/// Un scénario qui prouve la sortie sans la suivre (`HubSetup::stay`) : la run n'est pas lancée.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct HubStay;

/// Mode de run de la session : au hub, `Sandbox` (aucune condition de fin, pas de niveaux) ;
/// sinon la résolution habituelle de `run_state`.
pub fn resolve_session_mode(
    manifest: Option<&content::manifest::GameManifest>,
    registry: Option<&content::registry::Registry>,
    floors_override: Option<&str>,
    hub: Option<&HubSession>,
) -> run::RunMode {
    if hub.is_some_and(|hub| hub.in_hub) {
        return run::RunMode::Sandbox;
    }
    crate::run_state::resolve_run_mode_with_floors(manifest, registry, floors_override)
}

/// Lance la run quand la sortie décidée dans la simulation est **confirmée** (`Update`) : un
/// état prédit puis annulé par un rollback ne déclenche rien, et les pairs qui confirment la
/// même frame prennent la même décision au même moment.
pub fn hub_exit_trigger(
    state: Res<HubState>,
    session: Option<Res<HubSession>>,
    confirmed: Option<Res<bevy_ggrs::ConfirmedFrameCount>>,
    request: Option<Res<crate::run_state::RunRequest>>,
    stay: Option<Res<HubStay>>,
    mut commands: Commands,
) {
    let (Some(session), Some(confirmed), None, None) = (session, confirmed, request, stay) else {
        return;
    };
    let Some(exit) = state.exit else {
        return;
    };
    if session.in_hub && i64::from(confirmed.0) >= i64::from(exit.frame) {
        info!(
            "hub : sortie du joueur {} confirmée (frame {}), lancement de la run",
            exit.handle, exit.frame
        );
        commands.insert_resource(crate::run_state::RunRequest::EnterRun);
    }
}

pub struct HubPlugin;

impl Plugin for HubPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HubServices>()
            .init_resource::<HubState>()
            .rollback_and_trace_resource_neutral::<HubState>()
            .rollback_and_trace_neutral::<HubStation>();
        app.add_systems(
            Update,
            hub_exit_trigger.run_if(in_state(crate::core::AppState::InGame)),
        );
        app.add_systems(
            GgrsSchedule,
            hub_station_system
                .after(crate::shop::shelf_interaction_system)
                .before(RollbackSystemSet::Movement)
                .in_set(RollbackSystemSet::Interaction),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_etat_par_defaut_est_vide() {
        let state = HubState::default();
        assert!(state.exit.is_none() && state.visits.is_empty());
    }
}
