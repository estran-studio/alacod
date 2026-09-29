//! Extension unique pour enregistrer un état rollback : rollback bevy_ggrs, checksum GGRS
//! (ce que le synctest et la détection de desync p2p comparent) et trace d'état, en un
//! seul appel. Les appels directs à `rollback_component_*` / `rollback_resource_*` sont à
//! remplacer par [`RollbackTraceApp`] (plan §9.6, tâche K0).
//!
//! Vit dans `utils` (et non `game`) parce que `animation` a besoin de l'extension sans
//! dépendre de `game` (qui dépend déjà d'`animation` : une dépendance dans l'autre sens
//! créerait un cycle). `utils` a déjà `bevy` et `bevy_ggrs` en dépendances et ne dépend
//! d'aucun crate de gameplay ; `game` réexporte ce module (`game::rollback`) pour que les
//! sites existants n'aient pas à changer leur `use crate::rollback::RollbackTraceApp;`.

use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy_ggrs::RollbackApp;
use std::any::type_name;

/// Ressource pour tracker les types enregistrés avec tracing.
/// Utile pour vérifier la couverture plus tard (voir [`StateTracers`] et le test de
/// couverture dans `game::rollback`).
#[derive(Resource, Default)]
pub struct TracedTypes(pub Vec<&'static str>);

/// Fonction de trace d'un composant : lit le composant `Debug` sur une entité si présent.
pub type ComponentTracer = fn(&World, Entity) -> Option<String>;
/// Fonction de trace d'une ressource : lit la ressource `Debug` si présente.
pub type ResourceTracer = fn(&World) -> Option<String>;

/// Tracers génériques enregistrés par [`RollbackTraceApp`], dans l'ordre d'enregistrement
/// des types. Utilisée par `state_trace::record_state` (mode `ALACOD_STATE_TRACE_FULL`)
/// pour construire un dump détaillé sans connaître la liste des types à l'avance.
#[derive(Resource, Default)]
pub struct StateTracers {
    pub components: Vec<(&'static str, ComponentTracer)>,
    pub resources: Vec<(&'static str, ResourceTracer)>,
}

/// Extension sur [`App`] pour enregistrer composants et ressources en rollback
/// tout en activant le checksum GGRS et en enregistrant le type.
pub trait RollbackTraceApp {
    /// Enregistre un composant en rollback avec clone, checksum et trace.
    fn rollback_and_trace<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable>
            + Clone
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static;

    /// Variante pour les types qui n'implémentent pas `Hash` : utilise `Debug`.
    fn rollback_and_trace_debug<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Enregistre une ressource en rollback avec clone, checksum et trace.
    fn rollback_and_trace_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Clone
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static;

    /// Variante pour les ressources qui n'implémentent pas `Hash` : utilise `Debug`.
    fn rollback_and_trace_debug_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Enregistre une ressource `Copy` en rollback avec checksum et trace.
    fn rollback_and_trace_copy_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Copy
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static;

    /// Enregistre une ressource `Copy` en rollback et en trace, **sans** l'ajouter au
    /// checksum GGRS. Réservé aux ressources de présentation qui se sont glissées dans
    /// l'état rollback (ex. `PointerWorldPosition`, position du curseur) : leur valeur
    /// diffère légitimement d'un client à l'autre, et les inclure au checksum
    /// déclencherait de faux désyncs p2p / faux mismatches synctest.
    fn rollback_and_trace_copy_resource_no_checksum<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable> + Copy + std::fmt::Debug + Send + Sync + 'static;
}

impl RollbackTraceApp for App {
    fn rollback_and_trace<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable>
            + Clone
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static,
    {
        register_traced_component::<C>(self);
        self.rollback_component_with_clone::<C>()
            .checksum_component_with_hash::<C>()
    }

    fn rollback_and_trace_debug<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static,
    {
        register_traced_component::<C>(self);
        self.rollback_component_with_clone::<C>()
            .checksum_component(hash_debug::<C>)
    }

    fn rollback_and_trace_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Clone
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static,
    {
        register_traced_resource::<R>(self);
        self.rollback_resource_with_clone::<R>()
            .checksum_resource_with_hash::<R>()
    }

    fn rollback_and_trace_debug_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static,
    {
        register_traced_resource::<R>(self);
        self.rollback_resource_with_clone::<R>()
            .checksum_resource(hash_debug::<R>)
    }

    fn rollback_and_trace_copy_resource<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Copy
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static,
    {
        register_traced_resource::<R>(self);
        self.rollback_resource_with_copy::<R>()
            .checksum_resource_with_hash::<R>()
    }

    fn rollback_and_trace_copy_resource_no_checksum<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable> + Copy + std::fmt::Debug + Send + Sync + 'static,
    {
        register_traced_resource::<R>(self);
        self.rollback_resource_with_copy::<R>()
    }
}

/// Enregistre un type de trace dans `TracedTypes`, initialisant les ressources de trace
/// au besoin (premier type enregistré).
fn ensure_trace_resources(app: &mut App) {
    if !app.world().contains_resource::<TracedTypes>() {
        app.init_resource::<TracedTypes>();
    }
    if !app.world().contains_resource::<StateTracers>() {
        app.init_resource::<StateTracers>();
    }
}

/// Enregistre un composant : nom dans `TracedTypes`, tracer `Debug` dans `StateTracers`.
fn register_traced_component<C: Component + std::fmt::Debug>(app: &mut App) {
    ensure_trace_resources(app);
    let name = type_name::<C>();
    let tracer: ComponentTracer = |world, entity| {
        world
            .get_entity(entity)
            .ok()?
            .get::<C>()
            .map(|c| format!("{c:?}"))
    };
    let state = app.world_mut();
    state.resource_mut::<TracedTypes>().0.push(name);
    state
        .resource_mut::<StateTracers>()
        .components
        .push((name, tracer));
}

/// Enregistre une ressource : nom dans `TracedTypes`, tracer `Debug` dans `StateTracers`.
fn register_traced_resource<R: Resource + std::fmt::Debug>(app: &mut App) {
    ensure_trace_resources(app);
    let name = type_name::<R>();
    let tracer: ResourceTracer = |world| world.get_resource::<R>().map(|r| format!("{r:?}"));
    let state = app.world_mut();
    state.resource_mut::<TracedTypes>().0.push(name);
    state
        .resource_mut::<StateTracers>()
        .resources
        .push((name, tracer));
}

/// Fonction de hashage pour les types qui n'implémentent pas `Hash` :
/// formate le `Debug` et le hache en FNV-1a 64 bits.
pub fn hash_debug<T: std::fmt::Debug>(value: &T) -> u64 {
    let debug_str = format!("{:?}", value);
    fnv1a(debug_str.as_bytes())
}

/// Hash FNV-1a 64 bits : stable entre les runs et les machines.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
