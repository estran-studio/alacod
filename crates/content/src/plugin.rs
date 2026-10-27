//! Ce que le **plugin Rust** d'un jeu apporte au lint (X-1, `docs/conventions.md` §91).
//!
//! Deux sources, deux rôles :
//! - les **noms** (kinds de contenu, ids `Custom`, noms de règles) sont déclarés **en données** dans
//!   `game.ron` (`plugin`, [`crate::manifest::PluginDecl`]) : `alacod lint`, un binaire sans le code du
//!   jeu, les connaît et refuse un `Custom` non déclaré ;
//! - les **règles de lint** sont du Rust ([`LintRule`]) : seul le binaire du jeu les a. Il les installe
//!   avec [`install`] avant de charger le contenu ; [`crate::lint::run`] les exécute alors (partie,
//!   `--sandbox`, runner de scénarios, `<jeu> lint`) et vérifie que ce que le binaire fournit est
//!   exactement ce que `game.ron` déclare. Sans installation (`alacod lint`), elles sont listées en
//!   avertissement ([`warnings`]) : ni exécutées, ni oubliées en silence.
//!
//! État global au processus (un jeu par binaire) : `load_and_lint` est appelé depuis une dizaine
//! d'endroits (jeu, rechargement à chaud, runner, générateur) qui n'ont pas à se passer le plugin.

use std::collections::{BTreeMap, BTreeSet};

use parking_lot::RwLock;

use crate::lint::{LintError, LintErrorKind};
use crate::manifest::{GameManifest, PluginDecl};
use crate::reach::{LintWarning, LintWarningKind};
use crate::registry::{Registry, KNOWN_KIND_NAMES};

/// Une règle de lint écrite en Rust par le plugin : elle lit le registre entier (ses kinds, les
/// `Custom` de tout le contenu) et pousse ses erreurs (`LintErrorKind::PluginRule`, [`rule_error`]).
pub type LintRule = fn(&Registry, &GameManifest, &mut Vec<LintError>);

/// Le vocabulaire que le code du plugin fournit, à comparer à `GameManifest::plugin`.
#[derive(Clone, Default)]
pub struct Provided {
    pub kinds: BTreeSet<String>,
    pub behaviors: BTreeSet<String>,
    pub actions: BTreeSet<String>,
    pub conditions: BTreeSet<String>,
    pub lint_rules: BTreeMap<String, LintRule>,
}

static PROVIDED: RwLock<Option<Provided>> = RwLock::new(None);

/// Installe ce que le plugin fournit (appelé une fois, avant tout chargement de contenu).
pub fn install(provided: Provided) {
    *PROVIDED.write() = Some(provided);
}

/// Retire l'installation (tests).
pub fn uninstall() {
    *PROVIDED.write() = None;
}

/// Un plugin est-il installé dans ce processus ?
pub fn is_installed() -> bool {
    PROVIDED.read().is_some()
}

/// Erreur d'une règle de plugin : `file` situe le fichier ou le kind en cause.
pub fn rule_error(file: impl Into<String>, message: impl Into<String>) -> LintError {
    LintError {
        kind: LintErrorKind::PluginRule,
        file: file.into(),
        message: message.into(),
    }
}

/// Erreurs de **déclaration** du manifeste, indépendantes du code du plugin (donc aussi pour
/// `alacod lint`) : un kind du plugin qui porte le nom d'un kind de l'engine, un nom en double.
pub(crate) fn lint_declaration(manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let decl = &manifest.plugin;
    for kind in &decl.kinds {
        if KNOWN_KIND_NAMES.contains(&kind.as_str()) {
            errors.push(LintError {
                kind: LintErrorKind::PluginMismatch,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!("plugin.kinds : « {kind} » est déjà un kind de l'engine"),
            });
        }
    }
    for (space, names) in spaces(decl) {
        let mut seen = BTreeSet::new();
        for name in names {
            if name.is_empty() || !seen.insert(name) {
                errors.push(LintError {
                    kind: LintErrorKind::PluginMismatch,
                    file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                    message: format!("plugin.{space} : nom vide ou en double (« {name} »)"),
                });
            }
        }
    }
}

fn spaces(decl: &PluginDecl) -> [(&'static str, &Vec<String>); 5] {
    [
        ("kinds", &decl.kinds),
        ("behaviors", &decl.behaviors),
        ("actions", &decl.actions),
        ("conditions", &decl.conditions),
        ("lint_rules", &decl.lint_rules),
    ]
}

/// Compare le vocabulaire fourni au vocabulaire déclaré (dans les deux sens), puis exécute les règles
/// de lint du plugin. Rien sans plugin installé.
pub(crate) fn lint(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let guard = PROVIDED.read();
    let Some(provided) = guard.as_ref() else {
        return;
    };
    let decl = &manifest.plugin;
    let provided_names: [(&str, BTreeSet<&str>); 5] = [
        ("kinds", provided.kinds.iter().map(String::as_str).collect()),
        (
            "behaviors",
            provided.behaviors.iter().map(String::as_str).collect(),
        ),
        (
            "actions",
            provided.actions.iter().map(String::as_str).collect(),
        ),
        (
            "conditions",
            provided.conditions.iter().map(String::as_str).collect(),
        ),
        (
            "lint_rules",
            provided.lint_rules.keys().map(String::as_str).collect(),
        ),
    ];
    for ((space, declared), (_, provided)) in spaces(decl).into_iter().zip(provided_names) {
        let declared: BTreeSet<&str> = declared.iter().map(String::as_str).collect();
        for name in declared.difference(&provided) {
            errors.push(LintError {
                kind: LintErrorKind::PluginMismatch,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!("plugin.{space} : « {name} » est déclaré mais le plugin du jeu ne le fournit pas"),
            });
        }
        for name in provided.difference(&declared) {
            errors.push(LintError {
                kind: LintErrorKind::PluginMismatch,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!("plugin.{space} : « {name} » est fourni par le plugin du jeu mais absent de game.ron (le déclarer : `alacod lint` doit le connaître)"),
            });
        }
    }
    // Ordre des noms (`BTreeMap`) : sorties reproductibles.
    for (name, rule) in &provided.lint_rules {
        if decl.lint_rules.contains(name) {
            rule(registry, manifest, errors);
        }
    }
}

/// Règles déclarées que ce processus n'exécute pas (aucun plugin installé : `alacod lint`).
pub fn warnings(manifest: &GameManifest) -> Vec<LintWarning> {
    if is_installed() || manifest.plugin.lint_rules.is_empty() {
        return Vec::new();
    }
    vec![LintWarning {
        kind: LintWarningKind::PluginRulesNotRun,
        file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
        message: format!(
            "règles de lint du plugin non exécutées par ce binaire ({}) : le jeu les exécute à chaque chargement du contenu (`<jeu> lint`)",
            manifest.plugin.lint_rules.join(", ")
        ),
    }]
}
