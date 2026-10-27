//! Plugin Rust d'un jeu (X-1, `docs/conventions.md` §89, `docs/plan-engine.md` §4.8).
//!
//! Un jeu externe porte du code Rust quand sa mécanique ne se découpe pas en verbes d'engine. Ce
//! module est le contrat entre ce code et l'engine :
//!
//! - [`GamePlugin`] : **trois** entrées. [`GamePlugin::extension`] déclare le vocabulaire (kinds de
//!   contenu, `Custom`, règles de lint) avant le chargement du contenu ; [`GamePlugin::build`] pose la
//!   **simulation** (composants et ressources rollback, systèmes de `GgrsSchedule`) ;
//!   [`GamePlugin::assemble`] pose le reste de la **partie jouée** (ressources, systèmes `Update`,
//!   présentation), jamais appelé par le runner de scénarios.
//! - [`install`] : appelé **une fois** par l'entrée du jeu (`entry::run`), avant tout chargement. État
//!   global au processus : le runner de scénarios, les bots et le rechargement à chaud construisent
//!   leurs propres `App` sans se passer le plugin ; tous le voient par ici.
//! - [`CustomVerbs`] : ressource (non rollback, immuable) qui exécute les `Behavior::Custom` et
//!   `Condition::Custom`. Une `Action::Custom` n'a pas de fonction : l'engine pose
//!   `EffectOut::Custom` dans la frame, le plugin le lit avec [`custom_actions`] (événements en
//!   `FrameEvents`, règle du plugin de jeu).
//!
//! **Point d'insertion rollback.** `CoreSimPlugin` appelle `build` de chaque plugin juste **avant** les cinq
//! enregistrements de clôture (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`,
//! `RngStreams`), comme toute registration nouvelle (`CLAUDE.md`). Un clone sans plugin n'a donc aucun type
//! de plus ni de moins : `make schedule_order_diff` reste vide.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedVec2;
use content::plugin::{LintRule, Provided};
use parking_lot::RwLock;
use sim_core::frame_events::FrameEvents;
use sim_core::kinds::{KindDecl, KindRegistry};

pub use crate::character::enemy::ai::rules::BehaviorMotion;
use crate::effects_v3::EffectOut;
pub use behaviors::SelectionContext;
pub use effects::runtime::{Carrier, Trigger};
pub use sim_core::custom::CustomParams;

/// Faits d'un ennemi à un behavior `Custom` retenu, pour son déplacement.
#[derive(Debug, Clone, Copy)]
pub struct CustomMotionView {
    /// Frame courante.
    pub now: u32,
    /// Frame depuis laquelle cette règle est retenue.
    pub since_frame: u32,
    pub position: FixedVec2,
    /// Dernière position connue de la cible.
    pub target: Option<FixedVec2>,
}

/// La règle est-elle applicable cette frame ? Lue dans l'ordre de la liste de behaviors : la première
/// règle applicable gagne (`behaviors::select_with`).
pub type BehaviorApplicable = fn(&CustomParams, &SelectionContext) -> bool;
/// Déplacement imposé par la règle retenue (`None` : le déplacement d'origine de l'ennemi).
pub type BehaviorMotionFn = fn(&CustomParams, &CustomMotionView) -> Option<BehaviorMotion>;
/// Condition `Custom` d'un effet : lit le porteur et le déclencheur.
pub type ConditionFn = fn(&CustomParams, &Carrier, Option<&Trigger>) -> bool;

/// Implémentation d'un `Behavior::Custom`.
#[derive(Clone, Copy)]
pub struct CustomBehaviorImpl {
    pub applicable: BehaviorApplicable,
    pub motion: Option<BehaviorMotionFn>,
}

/// Vocabulaire déclaré par un plugin ([`GamePlugin::extension`]). Chaque nom doit aussi figurer dans
/// `plugin` de `game.ron` (le lint compare, dans les deux sens : `content::plugin`).
#[derive(Default, Clone)]
pub struct Extension {
    kinds: BTreeSet<String>,
    behaviors: BTreeMap<String, CustomBehaviorImpl>,
    conditions: BTreeMap<String, ConditionFn>,
    actions: BTreeSet<String>,
    lint_rules: BTreeMap<String, LintRule>,
}

impl Extension {
    /// Kind de contenu (`content_folders[].kind`) : fichiers RON lus par le plugin dans
    /// `Registry::plugin_content`.
    pub fn content_kind(mut self, kind: &str) -> Self {
        self.kinds.insert(kind.to_string());
        self
    }

    /// `Behavior::Custom { id }`.
    pub fn behavior(mut self, id: &str, implementation: CustomBehaviorImpl) -> Self {
        self.behaviors.insert(id.to_string(), implementation);
        self
    }

    /// `Condition::Custom { id }`.
    pub fn condition(mut self, id: &str, condition: ConditionFn) -> Self {
        self.conditions.insert(id.to_string(), condition);
        self
    }

    /// `Action::Custom { id }` : exécutée par un système du plugin (voir [`custom_actions`]).
    pub fn action(mut self, id: &str) -> Self {
        self.actions.insert(id.to_string());
        self
    }

    /// Règle de lint écrite en Rust (`content::plugin::rule_error` pour ses erreurs).
    pub fn lint_rule(mut self, name: &str, rule: LintRule) -> Self {
        self.lint_rules.insert(name.to_string(), rule);
        self
    }
}

/// Le code Rust d'un jeu. Voir la documentation du module.
pub trait GamePlugin: Send + Sync + 'static {
    /// Vocabulaire déclaré (avant tout chargement de contenu).
    fn extension(&self) -> Extension {
        Extension::default()
    }

    /// Simulation : enregistrements rollback (`RollbackTraceApp`, règles de `CLAUDE.md`), systèmes de
    /// `GgrsSchedule`, `FrameEvents`. Appelé par `CoreSimPlugin` avant les cinq enregistrements de
    /// clôture, dans **toute** application : partie, `--sandbox`, runner de scénarios, bots.
    fn build(&self, _app: &mut App) {}

    /// Partie jouée seulement (jamais le runner de scénarios) : ressources, systèmes `Update`,
    /// présentation. Appelé après `LdtkLocalGamePlugin`.
    fn assemble(&self, _app: &mut App) {}
}

/// Exécution des verbes `Custom` : table immuable posée par `CoreSimPlugin`, vide sans plugin.
#[derive(Resource, Clone, Default)]
pub struct CustomVerbs(Arc<Verbs>);

#[derive(Default)]
struct Verbs {
    behaviors: BTreeMap<String, CustomBehaviorImpl>,
    conditions: BTreeMap<String, ConditionFn>,
}

impl CustomVerbs {
    /// `Behavior::Custom` applicable ? Un id inconnu ne l'est jamais (le lint l'interdit).
    pub fn behavior_applicable(
        &self,
        id: &str,
        params: &CustomParams,
        ctx: &SelectionContext,
    ) -> bool {
        self.0
            .behaviors
            .get(id)
            .is_some_and(|b| (b.applicable)(params, ctx))
    }

    /// Déplacement imposé par un `Behavior::Custom` retenu.
    pub fn behavior_motion(
        &self,
        id: &str,
        params: &CustomParams,
        view: &CustomMotionView,
    ) -> Option<BehaviorMotion> {
        self.0
            .behaviors
            .get(id)?
            .motion
            .and_then(|motion| motion(params, view))
    }

    /// `Condition::Custom` vraie ? Un id inconnu est faux.
    pub fn condition(
        &self,
        id: &str,
        params: &CustomParams,
        carrier: &Carrier,
        trigger: Option<&Trigger>,
    ) -> bool {
        self.0
            .conditions
            .get(id)
            .is_some_and(|condition| condition(params, carrier, trigger))
    }
}

/// Un `Action::Custom` déclenché cette frame.
#[derive(Debug, Clone, Copy)]
pub struct CustomActionCall<'a> {
    /// `GgrsNetId` du porteur de l'effet.
    pub carrier: usize,
    /// `GgrsNetId` de la cible du déclencheur (`OnHit`, `OnKill`, `OnDamageTaken`), s'il y en a une.
    pub target: Option<usize>,
    pub params: &'a CustomParams,
}

/// Les `Action::Custom { id }` de la frame, dans l'ordre d'émission (déterministe : ordre des porteurs
/// puis des effets). À lire dans un système `GgrsSchedule` ordonné **après**
/// `effects_v3::effect_status_system` (dernier système de la chaîne d'effets : il suit
/// `apply_triggered_effects_system` et `apply_event_effects_system`, qui émettent aussi des `Custom`) et avant `RollbackSystemSet::FrameCounter`.
pub fn custom_actions<'a>(
    outbox: &'a FrameEvents<EffectOut>,
    id: &'a str,
) -> impl Iterator<Item = CustomActionCall<'a>> + 'a {
    outbox.iter().filter_map(move |event| match event {
        EffectOut::Custom {
            carrier,
            target,
            id: emitted,
            params,
        } if emitted == id => Some(CustomActionCall {
            carrier: *carrier,
            target: *target,
            params,
        }),
        _ => None,
    })
}

struct Installed {
    plugins: Vec<Arc<dyn GamePlugin>>,
    verbs: CustomVerbs,
    kinds: Extension,
}

static INSTALLED: RwLock<Option<Arc<Installed>>> = RwLock::new(None);

/// Installe les plugins du jeu pour ce processus : vérifie qu'un même nom n'est pas fourni deux fois,
/// installe côté lint (`content::plugin::install`). À appeler **avant** de charger le contenu.
///
/// # Panics
/// Si deux plugins fournissent le même id : deux jeux de verbes qui se marchent dessus.
pub fn install(plugins: Vec<Arc<dyn GamePlugin>>) {
    let mut merged = Extension::default();
    for plugin in &plugins {
        let extension = plugin.extension();
        merge(&mut merged.kinds, extension.kinds, "kind");
        merge(&mut merged.actions, extension.actions, "action");
        merge_map(&mut merged.behaviors, extension.behaviors, "behavior");
        merge_map(&mut merged.conditions, extension.conditions, "condition");
        merge_map(
            &mut merged.lint_rules,
            extension.lint_rules,
            "règle de lint",
        );
    }
    content::plugin::install(Provided {
        kinds: merged.kinds.clone(),
        behaviors: merged.behaviors.keys().cloned().collect(),
        actions: merged.actions.clone(),
        conditions: merged.conditions.keys().cloned().collect(),
        lint_rules: merged.lint_rules.clone(),
    });
    let verbs = CustomVerbs(Arc::new(Verbs {
        behaviors: merged.behaviors.clone(),
        conditions: merged.conditions.clone(),
    }));
    *INSTALLED.write() = Some(Arc::new(Installed {
        plugins,
        verbs,
        kinds: merged,
    }));
}

fn merge(into: &mut BTreeSet<String>, from: BTreeSet<String>, what: &str) {
    for name in from {
        assert!(
            into.insert(name.clone()),
            "deux plugins fournissent le même {what} « {name} »"
        );
    }
}

fn merge_map<V>(into: &mut BTreeMap<String, V>, from: BTreeMap<String, V>, what: &str) {
    for (name, value) in from {
        assert!(
            into.insert(name.clone(), value).is_none(),
            "deux plugins fournissent le même {what} « {name} »"
        );
    }
}

/// Retire l'installation (tests).
pub fn uninstall() {
    *INSTALLED.write() = None;
    content::plugin::uninstall();
}

/// Verbes `Custom` du jeu installé (vide sans plugin).
pub fn custom_verbs() -> CustomVerbs {
    INSTALLED
        .read()
        .as_ref()
        .map(|installed| installed.verbs.clone())
        .unwrap_or_default()
}

/// Appelé par `CoreSimPlugin::build`, juste avant les enregistrements de clôture : déclare les ids dans
/// `KindRegistry` (catégories `custom_behavior`, `custom_action`, `custom_condition`, `content_folder`
/// pour les kinds du plugin) et appelle `GamePlugin::build` de chaque plugin, dans l'ordre d'installation.
pub(crate) fn build_plugins(app: &mut App) {
    let installed = INSTALLED.read().clone();
    app.insert_resource(
        installed
            .as_ref()
            .map(|i| i.verbs.clone())
            .unwrap_or_default(),
    );
    let Some(installed) = installed else {
        return;
    };
    let ext = &installed.kinds;
    app.register_kinds(
        ext.behaviors
            .keys()
            .map(|id| KindDecl::new("custom_behavior", id.as_str())),
    );
    app.register_kinds(
        ext.conditions
            .keys()
            .map(|id| KindDecl::new("custom_condition", id.as_str())),
    );
    app.register_kinds(
        ext.actions
            .iter()
            .map(|id| KindDecl::new("custom_action", id.as_str())),
    );
    app.register_kinds(
        ext.kinds
            .iter()
            .map(|kind| KindDecl::new("plugin_content_folder", kind.as_str())),
    );
    for plugin in &installed.plugins {
        plugin.build(app);
    }
}

/// Appelé par l'assemblage de la partie jouée (`entry`), après `LdtkLocalGamePlugin`.
pub fn assemble_plugins(app: &mut App) {
    let installed = INSTALLED.read().clone();
    for plugin in installed.iter().flat_map(|i| i.plugins.iter()) {
        plugin.assemble(app);
    }
}
