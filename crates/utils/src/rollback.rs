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
use bevy_ggrs::{
    ChecksumFlag, ChecksumPart, RollbackApp, RollbackId, RollbackOrdered, SaveWorld,
    SaveWorldSystems,
};
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

    /// Enregistre un composant en rollback avec clone et trace, **sans** l'ajouter au
    /// checksum GGRS. Réservé à un composant nouveau dont aucune entité existante ne doit
    /// changer le checksum comparé par le synctest/desync (T2.9 : `HitCount`, posé
    /// uniquement sur du contenu nouveau — même simplement *enregistrer* un type au
    /// checksum GGRS déplace le checksum agrégé de toutes les entités existantes, y compris
    /// quand aucune n'en porte). Mêmes garanties de rollback qu'avec checksum : seule la
    /// comparaison entre clients change.
    fn rollback_and_trace_no_checksum<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Comme [`Self::rollback_and_trace`], mais un type que **aucune** entité ne porte
    /// contribue **`0`** au checksum GGRS (élément neutre du XOR de `bevy_ggrs::ChecksumPlugin`) ;
    /// dès qu'une entité le porte, sa contribution est exactement celle de
    /// `checksum_component_with_hash` (même hachage par entité, même ordre `RollbackOrdered`).
    ///
    /// Pourquoi (T1.2, vérifié sur `idle`) : `bevy_ggrs::ComponentChecksumPlugin` fait
    /// contribuer à un type sans porteur une `ChecksumPart` constante `K = hash(0u64)`,
    /// **identique pour tous les types vides** ; les parts sont combinées par XOR. Ajouter un
    /// type vide déplace donc toutes les traces existantes (un type vide de plus : trace
    /// différente dès la ligne 1 ; deux de plus : ils s'annulent, trace identique). Réservé à
    /// un composant nouveau, absent du contenu existant (T1.2 : `combat::emitter::Emitter`) :
    /// l'enregistrer ne change aucune trace, quelle que soit la parité des autres types vides.
    fn rollback_and_trace_neutral<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable>
            + Clone
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static;

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

    /// Comme [`Self::rollback_and_trace_resource`], mais la valeur par défaut (`R::default()`)
    /// contribue **`0`** au checksum GGRS (élément neutre du XOR de `bevy_ggrs::ChecksumPlugin`) ;
    /// toute autre valeur contribue son hash habituel. Réservé à l'état d'un mode ou d'un
    /// contenu nouveau qui reste à sa valeur par défaut partout ailleurs (T1.8 : `FloorState`
    /// du mode `Floors`) : l'enregistrer ne déplace pas le checksum des parties qui ne
    /// l'utilisent pas (traces inchangées), alors qu'une ressource ordinaire ajoute toujours
    /// une part non nulle. La ressource doit exister (`init_resource`) dès que `SaveWorld`
    /// tourne. Une valeur non défaut dont le hash vaudrait exactement `0` (probabilité 2⁻⁶⁴)
    /// passerait inaperçue d'un desync : risque accepté.
    fn rollback_and_trace_resource_neutral<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Clone
            + Default
            + PartialEq
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

    fn rollback_and_trace_no_checksum<C>(&mut self) -> &mut Self
    where
        C: Component<Mutability = Mutable> + Clone + std::fmt::Debug + Send + Sync + 'static,
    {
        register_traced_component::<C>(self);
        self.rollback_component_with_clone::<C>()
    }

    fn rollback_and_trace_neutral<C>(&mut self) -> &mut Self
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
        self.rollback_component_with_clone::<C>();
        self.add_systems(
            SaveWorld,
            neutral_component_checksum::<C>.in_set(SaveWorldSystems::Checksum),
        )
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

    fn rollback_and_trace_resource_neutral<R>(&mut self) -> &mut Self
    where
        R: Resource<Mutability = Mutable>
            + Clone
            + Default
            + PartialEq
            + std::hash::Hash
            + std::fmt::Debug
            + Send
            + Sync
            + 'static,
    {
        register_traced_resource::<R>(self);
        self.rollback_resource_with_clone::<R>()
            .checksum_resource(hash_neutral_default::<R>)
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

/// Checksum d'un composant à checksum neutre (voir
/// [`RollbackTraceApp::rollback_and_trace_neutral`]) : copie de
/// `bevy_ggrs::ComponentChecksumPlugin` (même hasher, même XOR des entités ordonnées par
/// `RollbackOrdered`, même repli final), sauf qu'aucun porteur donne `ChecksumPart(0)`.
#[allow(clippy::type_complexity)]
fn neutral_component_checksum<C: Component + std::hash::Hash>(
    mut commands: Commands,
    rollback_ordered: Res<RollbackOrdered>,
    components: Query<(&RollbackId, &C), (With<RollbackId>, Without<ChecksumFlag<C>>)>,
    mut checksum: Query<&mut ChecksumPart, (Without<RollbackId>, With<ChecksumFlag<C>>)>,
) {
    use std::hash::{Hash, Hasher};
    let hasher = bevy_ggrs::checksum_hasher();
    let mut result = 0u64;
    let mut carried = false;
    for (&rollback, component) in components.iter() {
        carried = true;
        let mut entity_hasher = hasher;
        rollback_ordered.order(rollback).hash(&mut entity_hasher);
        let mut component_hasher = bevy_ggrs::checksum_hasher();
        component.hash(&mut component_hasher);
        component_hasher.finish().hash(&mut entity_hasher);
        result ^= entity_hasher.finish();
    }
    let part = if carried {
        let mut hasher = hasher;
        result.hash(&mut hasher);
        ChecksumPart(hasher.finish() as u128)
    } else {
        ChecksumPart(0)
    };
    if let Ok(mut current) = checksum.single_mut() {
        *current = part;
    } else {
        commands.spawn((part, ChecksumFlag::<C>::default()));
    }
}

/// Hash d'une ressource à checksum neutre (voir
/// [`RollbackTraceApp::rollback_and_trace_resource_neutral`]) : `0` pour `R::default()`,
/// sinon le même hasher que `bevy_ggrs` pour une ressource `Hash`.
pub fn hash_neutral_default<R: Default + PartialEq + std::hash::Hash>(value: &R) -> u64 {
    use std::hash::Hasher;
    if *value == R::default() {
        return 0;
    }
    let mut hasher = bevy_ggrs::checksum_hasher();
    value.hash(&mut hasher);
    hasher.finish()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, PartialEq, Hash)]
    struct Etat {
        index: u32,
    }

    #[test]
    fn checksum_neutre_nul_a_la_valeur_par_defaut_seulement() {
        assert_eq!(hash_neutral_default(&Etat::default()), 0);
        assert_ne!(hash_neutral_default(&Etat { index: 1 }), 0);
        assert_ne!(
            hash_neutral_default(&Etat { index: 1 }),
            hash_neutral_default(&Etat { index: 2 })
        );
    }
}
