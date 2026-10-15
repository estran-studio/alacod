//! Coût du rollback par frame : temps de sauvegarde (`SaveWorld` : checksum puis clone des
//! snapshots), de restauration (`LoadWorld`) et de simulation (`AdvanceWorld`), et inventaire
//! des types rollback (nombre de porteurs × taille en mémoire, estimation sans le tas).
//!
//! C'est la mesure qui manquait pour juger le ressenti en ligne : à 60 Hz, un tick dispose de
//! 16,7 ms, et un paquet en retard coûte jusqu'à `max_prediction_window` restaurations et
//! resimulations d'affilée. Les métriques de scénario (`crates/scenario`) publient ces valeurs ;
//! le jeu les écrit dans son journal à la sortie (`ALACOD_SNAPSHOT_PROFILE=1`).
//!
//! Mesure seulement : rien ici n'entre dans la simulation ni dans le checksum (les chronos
//! vivent hors de `GgrsSchedule`, dans des systèmes aux bornes des schedules bevy_ggrs).

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_ggrs::{
    AdvanceWorld, AdvanceWorldSystems, LoadWorld, LoadWorldSystems, Rollback, SaveWorld,
    SaveWorldSystems,
};

/// Compteur d'un poste : nombre de passages, total et maximum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhaseStats {
    pub count: u32,
    pub total: Duration,
    pub max: Duration,
}

impl PhaseStats {
    fn record(&mut self, elapsed: Duration) {
        self.count += 1;
        self.total += elapsed;
        self.max = self.max.max(elapsed);
    }

    /// Moyenne en millisecondes (0 sans passage).
    pub fn avg_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total.as_secs_f64() * 1000.0 / f64::from(self.count)
        }
    }

    pub fn max_ms(&self) -> f64 {
        self.max.as_secs_f64() * 1000.0
    }
}

/// Un type rollback dans l'inventaire : porteurs à l'échantillon et octets estimés
/// (`porteurs × size_of`, le tas des `Vec`/`BTreeMap` n'est pas compté).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSample {
    pub name: &'static str,
    pub count: usize,
    pub bytes: usize,
}

/// Résultat du profil, lu par le runner de scénarios et le journal de sortie du jeu.
#[derive(Resource, Debug, Clone, Default)]
pub struct SnapshotProfile {
    /// `SaveWorld` entier (checksum + clone).
    pub save: PhaseStats,
    /// Partie checksum de `SaveWorld` (hash de chaque type sous checksum).
    pub checksum: PhaseStats,
    /// `LoadWorld` : restauration d'une frame (rollback).
    pub load: PhaseStats,
    /// `AdvanceWorld` : une frame de simulation (`GgrsSchedule`).
    pub advance: PhaseStats,
    /// Octets estimés d'un snapshot à l'échantillon le plus gros, et son inventaire trié
    /// (octets décroissants, puis nom).
    pub snapshot_bytes_max: usize,
    pub inventory_at_max: Vec<TypeSample>,
    started: Option<(Phase, Instant)>,
}

impl SnapshotProfile {
    /// Résumé sur une ligne pour les journaux.
    pub fn summary(&self) -> String {
        format!(
            "save {:.3}/{:.3} ms (checksum {:.3}) | load {:.3}/{:.3} ms ×{} | advance {:.3}/{:.3} ms | snapshot ~{} KiB",
            self.save.avg_ms(),
            self.save.max_ms(),
            self.checksum.avg_ms(),
            self.load.avg_ms(),
            self.load.max_ms(),
            self.load.count,
            self.advance.avg_ms(),
            self.advance.max_ms(),
            self.snapshot_bytes_max / 1024
        )
    }

    /// Les `n` types les plus lourds à l'échantillon le plus gros.
    pub fn heaviest(&self, n: usize) -> impl Iterator<Item = &TypeSample> {
        self.inventory_at_max.iter().take(n)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Save,
    Load,
    Advance,
}

/// Un type enregistré en rollback (`RollbackTraceApp`) : nom, taille en mémoire, compteur de
/// porteurs (`&mut World` : les `QueryState` se construisent à l'échantillon ; pour une
/// ressource, 0 ou 1).
pub type Counter = fn(&mut World) -> usize;

#[derive(Resource, Default)]
pub struct SnapshotInventory {
    pub types: Vec<(&'static str, usize, Counter)>,
}

impl SnapshotInventory {
    pub fn register_component<C: Component>(&mut self) {
        self.types.push((
            std::any::type_name::<C>(),
            std::mem::size_of::<C>(),
            |world| {
                world
                    .query_filtered::<(), (With<C>, With<Rollback>)>()
                    .iter(world)
                    .count()
            },
        ));
    }

    pub fn register_resource<R: Resource>(&mut self) {
        self.types.push((
            std::any::type_name::<R>(),
            std::mem::size_of::<R>(),
            |world| usize::from(world.contains_resource::<R>()),
        ));
    }

    /// Inventaire courant, trié par octets décroissants puis nom.
    pub fn sample(&self, world: &mut World) -> Vec<TypeSample> {
        let mut samples: Vec<TypeSample> = self
            .types
            .iter()
            .map(|&(name, size, counter)| {
                let count = counter(world);
                TypeSample {
                    name,
                    count,
                    bytes: count * size,
                }
            })
            .collect();
        samples.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.name.cmp(b.name)));
        samples
    }
}

/// Sauvegardes entre deux échantillons de l'inventaire (compter ~90 types n'est pas gratuit).
const INVENTORY_EVERY: u32 = 30;

/// À ajouter une seule fois par `App` (plugin unique : un second ajout panique dans bevy ;
/// vérifier `is_plugin_added` quand deux chemins peuvent l'installer).
pub struct SnapshotProfilePlugin;

impl Plugin for SnapshotProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnapshotProfile>()
            .init_resource::<SnapshotInventory>()
            .add_systems(
                SaveWorld,
                (
                    start(Phase::Save).before(SaveWorldSystems::Checksum),
                    checksum_done
                        .after(SaveWorldSystems::Checksum)
                        .before(SaveWorldSystems::Snapshot),
                    save_done.after(SaveWorldSystems::Snapshot),
                ),
            )
            .add_systems(
                LoadWorld,
                (
                    start(Phase::Load).before(LoadWorldSystems::EntityResurrect),
                    end(Phase::Load).after(LoadWorldSystems::Mapping),
                ),
            )
            .add_systems(
                AdvanceWorld,
                (
                    start(Phase::Advance).before(AdvanceWorldSystems::First),
                    end(Phase::Advance).after(AdvanceWorldSystems::Last),
                ),
            );
    }
}

fn start(phase: Phase) -> impl Fn(ResMut<SnapshotProfile>) {
    move |mut profile| profile.started = Some((phase, Instant::now()))
}

fn end(phase: Phase) -> impl Fn(ResMut<SnapshotProfile>) {
    move |mut profile| finish(&mut profile, phase)
}

fn finish(profile: &mut SnapshotProfile, phase: Phase) {
    let Some((started_phase, at)) = profile.started.take() else {
        return;
    };
    debug_assert_eq!(started_phase, phase);
    let elapsed = at.elapsed();
    match phase {
        Phase::Save => profile.save.record(elapsed),
        Phase::Load => profile.load.record(elapsed),
        Phase::Advance => profile.advance.record(elapsed),
    }
}

/// Fin du set `Checksum` : le temps écoulé depuis le début de `SaveWorld` est le coût du hash.
fn checksum_done(mut profile: ResMut<SnapshotProfile>) {
    if let Some((Phase::Save, at)) = profile.started {
        let elapsed = at.elapsed();
        profile.checksum.record(elapsed);
    }
}

/// Fin de `SaveWorld` ; toutes les `INVENTORY_EVERY` sauvegardes, échantillonne l'inventaire.
fn save_done(world: &mut World) {
    finish(&mut world.resource_mut::<SnapshotProfile>(), Phase::Save);
    let count = world.resource::<SnapshotProfile>().save.count;
    if count % INVENTORY_EVERY != 1 {
        return;
    }
    let inventory = world
        .remove_resource::<SnapshotInventory>()
        .unwrap_or_default();
    let samples = inventory.sample(world);
    world.insert_resource(inventory);
    let total: usize = samples.iter().map(|s| s.bytes).sum();
    let mut profile = world.resource_mut::<SnapshotProfile>();
    if total > profile.snapshot_bytes_max || profile.inventory_at_max.is_empty() {
        profile.snapshot_bytes_max = total;
        profile.inventory_at_max = samples;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_stats_average_and_max() {
        let mut stats = PhaseStats::default();
        assert_eq!(stats.avg_ms(), 0.0);
        stats.record(Duration::from_millis(2));
        stats.record(Duration::from_millis(4));
        assert_eq!(stats.count, 2);
        assert!((stats.avg_ms() - 3.0).abs() < 1e-9);
        assert!((stats.max_ms() - 4.0).abs() < 1e-9);
    }

    #[derive(Component)]
    struct Big([u8; 64]);
    #[derive(Resource)]
    struct Small(u8);

    #[test]
    fn inventory_counts_rollback_carriers_only_and_sorts_by_bytes() {
        let mut world = World::new();
        // Le hook `on_add` de `Rollback` enregistre l'entité dans `RollbackOrdered`.
        world.init_resource::<bevy_ggrs::RollbackOrdered>();
        world.spawn((Big([0; 64]), Rollback));
        world.spawn((Big([0; 64]), Rollback));
        world.spawn(Big([0; 64])); // sans Rollback : hors snapshot
        world.insert_resource(Small(0));
        let mut inventory = SnapshotInventory::default();
        inventory.register_resource::<Small>();
        inventory.register_component::<Big>();
        let samples = inventory.sample(&mut world);
        assert_eq!(samples[0].count, 2);
        assert_eq!(samples[0].bytes, 128);
        assert_eq!(samples[1].count, 1);
        assert_eq!(samples[1].bytes, 1);
    }
}
