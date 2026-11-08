//! X-6a1 (`docs/conventions.md` §117) : une porte de carte **verrouillée** bloque comme une porte fermée, champ de flux compris.
//!
//! Le champ de flux (`FlowFieldCache::wall_cells`) range les cases d'une porte tant qu'elle a un collider : une porte à serrure garde
//! le sien, donc ses cases restent murées pour tous les ennemis, avec ou sans clé en jeu ; ouverte (clé tenue par l'équipe), elle
//! rend ses cases. Un scénario ne voit pas la différence (le collider physique bloque de toute façon) : ce test lit le cache lui-même.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use game::character::enemy::ai::navigation::{get_collider_cells, FlowFieldCache, GridPos};
use game::collider::Collider;
use map::game::entity::map::door::DoorComponent;
use scenario::Scenario;
use utils::frame::FrameCount;

fn load(ron: &str) -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/{ron}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} : {e}"));
    let mut scenario = Scenario::from_ron(&text).unwrap_or_else(|e| panic!("{ron} : {e}"));
    scenario.expect.clear();
    scenario
}

/// Ce que le cache de navigation voit des portes, frame après frame.
#[derive(Default)]
struct Observed {
    /// Cases des portes vues avec un collider (cumul).
    door_cells: BTreeSet<GridPos>,
    /// Frames où une porte fermée avait au moins une case hors de `wall_cells` (après le premier calcul du champ).
    closed_but_free: Vec<u32>,
    /// Première frame où une porte est ouverte.
    opened_at: Option<u32>,
    /// Frames où une case de porte (vue fermée plus tôt) restait dans `wall_cells` alors que **toutes** les portes sont ouvertes.
    open_but_blocked: Vec<u32>,
}

/// Joue `ron` en lisant, à la fin de chaque update, les cases de portes et `FlowFieldCache::wall_cells`.
fn observe(ron: &str) -> Observed {
    let scenario = load(ron);
    let seen = Arc::new(Mutex::new(Observed::default()));
    let sink = seen.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            Last,
            move |frame: Res<FrameCount>,
                  cache: Res<FlowFieldCache>,
                  doors: Query<(&FixedTransform3D, Option<&Collider>), With<DoorComponent>>| {
                let mut seen = sink.lock().unwrap();
                // Le champ n'existe qu'une fois construit (premiers ticks de `GgrsSchedule`).
                if cache.level_info.is_none() || frame.frame < 2 {
                    return;
                }
                let mut closed = 0;
                for (transform, collider) in &doors {
                    match collider {
                        Some(collider) => {
                            closed += 1;
                            let cells = get_collider_cells(transform.translation.truncate(), collider);
                            if cells.iter().any(|cell| !cache.wall_cells.contains(cell)) {
                                seen.closed_but_free.push(frame.frame);
                            }
                            seen.door_cells.extend(cells);
                        }
                        None => {
                            seen.opened_at.get_or_insert(frame.frame);
                        }
                    }
                }
                // Marge de 2 frames : le champ est reconstruit au changement des cases bloquées.
                let settled = seen.opened_at.is_some_and(|at| frame.frame > at + 2);
                if closed == 0
                    && settled
                    && seen.door_cells.iter().any(|cell| cache.wall_cells.contains(cell))
                {
                    seen.open_but_blocked.push(frame.frame);
                }
            },
        );
    });
    Arc::try_unwrap(seen)
        .ok()
        .expect("sonde seule")
        .into_inner()
        .unwrap()
}

#[test]
fn une_porte_verrouillee_reste_murees_dans_le_champ_de_flux() {
    let seen = observe("door_state_locked_no_key");
    assert!(
        !seen.door_cells.is_empty(),
        "aucune case de porte vue : la sonde ne lit rien"
    );
    assert!(seen.opened_at.is_none(), "la porte s'est ouverte sans clé");
    assert!(
        seen.closed_but_free.is_empty(),
        "cases de la porte verrouillée libres dans le champ de flux aux frames {:?}",
        seen.closed_but_free
    );
}

#[test]
fn la_porte_ouverte_par_la_cle_rend_ses_cases() {
    let seen = observe("key_carrier_coop");
    assert!(
        !seen.door_cells.is_empty(),
        "aucune case de porte vue : la sonde ne lit rien"
    );
    let opened = seen
        .opened_at
        .expect("la porte s'ouvre avec la clé d'équipe");
    assert!(
        seen.closed_but_free.is_empty(),
        "cases de la porte fermée libres dans le champ de flux aux frames {:?}",
        seen.closed_but_free
    );
    assert!(
        seen.open_but_blocked.is_empty(),
        "cases de la porte ouverte (frame {opened}) encore murées aux frames {:?}",
        seen.open_but_blocked
    );
}
