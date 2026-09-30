//! Validation sémantique du contenu déjà chargé par [`crate::registry::Registry::build`] :
//! références cassées, valeurs hors plage. Chaque erreur nomme le fichier, l'id concerné
//! et le champ en cause (carte T1.5, `docs/taches.md`).
//!
//! Les erreurs de *chargement* (RON cassé, id dupliqué, kind de dossier inconnu) sont
//! rapportées par `Registry::build` lui-même : elles empêchent de construire l'entrée sur
//! laquelle porteraient les règles ci-dessous. `alacod lint` (et tout appelant) concatène
//! les deux listes.

use bevy_fixed::fixed_math::Fixed;

use crate::manifest::GameManifest;
use crate::registry::{self, Registry};

/// Catégorie d'erreur de lint, pour les tests (correspondance sans dépendre du texte du
/// message) et un futur regroupement en CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintErrorKind {
    /// Fichier illisible ou RON invalide (inclut un littéral nu dans un champ `Fixed`,
    /// voir `value::FixedField`).
    Parse,
    /// Même id défini deux fois pour le même kind.
    DuplicateId,
    /// Un champ référence un id qui n'existe dans aucune entrée chargée.
    BrokenReference,
    /// Une valeur ne respecte pas la plage attendue pour son champ (santé > 0, vitesse >=
    /// 0, cadence > 0...).
    OutOfRange,
    /// `content_folders[].kind` n'est pas un des kinds que `content` sait charger.
    UnknownKind,
}

/// Une erreur de lint : `file` et `message` forment le texte affiché par `alacod lint`
/// (stderr), `kind` sert aux appelants qui veulent regrouper ou filtrer sans dépendre du
/// texte.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{file}: {message}")]
pub struct LintError {
    pub kind: LintErrorKind,
    pub file: String,
    pub message: String,
}

/// Valide les références croisées et les plages de valeurs du contenu déjà chargé dans
/// `registry`. Ne relit rien sur disque : opère uniquement sur les entrées déjà parsées.
pub fn run(registry: &Registry, manifest: &GameManifest) -> Vec<LintError> {
    let mut errors = Vec::new();

    lint_characters(registry, &mut errors);
    lint_weapons(registry, &mut errors);
    lint_melee_weapons(registry, &mut errors);
    lint_waves(registry, &mut errors);
    lint_entry_point(registry, manifest, &mut errors);

    errors
}

fn lint_characters(registry: &Registry, errors: &mut Vec<LintError>) {
    for character in registry.characters.values() {
        let file = character.file.display().to_string();

        // Référence : starting_weapons -> WeaponId (T1.5 : les joueurs ne reçoivent plus
        // tout `weapons.ron`, seulement les armes déclarées ici).
        for weapon_id in &character.starting_weapons {
            if !registry.weapons.contains_key(weapon_id) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "personnage « {} » : champ starting_weapons : arme inconnue « {} »",
                        character.id, weapon_id
                    ),
                });
            }
        }

        // Référence : starting_skin -> une clé de skins.
        if !character.skins.contains(&character.starting_skin) {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ starting_skin = « {} » absent de skins ({:?})",
                    character.id, character.starting_skin, character.skins
                ),
            });
        }

        // Hors plage : santé > 0.
        if character.base_health_max.get() <= Fixed::ZERO {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ base_health.max = {} : doit être > 0",
                    character.id,
                    character.base_health_max.get()
                ),
            });
        }

        // Hors plage : vitesse >= 0.
        if character.max_speed.get() < Fixed::ZERO {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ movement.max_speed = {} : doit être >= 0",
                    character.id,
                    character.max_speed.get()
                ),
            });
        }

        // Hors plage (T1.2, chantier B2) : chaque stat surchargée par `stats:` doit être >= 0.
        for (stat_id, value) in &character.stats {
            if value.get() < Fixed::ZERO {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "personnage « {} » : champ stats.{:?} = {} : doit être >= 0",
                        character.id,
                        stat_id,
                        value.get()
                    ),
                });
            }
        }
    }
}

fn lint_weapons(registry: &Registry, errors: &mut Vec<LintError>) {
    for weapon in registry.weapons.values() {
        for (mode, rate) in &weapon.firing_rates {
            if rate.get() <= Fixed::ZERO {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : mode « {} » : champ firing_rate = {} : doit être > 0",
                        weapon.id,
                        mode,
                        rate.get()
                    ),
                });
            }
        }
        lint_weapon_test(&weapon.id, &weapon.file, weapon.test.as_ref(), errors);
    }
}

/// Valide le gabarit de scénario généré d'une arme (T2.10, `WeaponEntry::test`/
/// `MeleeWeaponEntry::test`) : `frames > 0`, `min_hits <= max_hits` si `max_hits` est
/// présent. Partagé par `lint_weapons`/`lint_melee_weapons` (`id` affiché avec son type par
/// l'appelant, comme les autres messages de ce fichier).
fn lint_weapon_test(
    id: &dyn std::fmt::Display,
    file: &std::path::Path,
    test: Option<&registry::WeaponTestRange>,
    errors: &mut Vec<LintError>,
) {
    let Some(test) = test else {
        return;
    };
    if test.frames == 0 {
        errors.push(LintError {
            kind: LintErrorKind::OutOfRange,
            file: file.display().to_string(),
            message: format!("arme « {id} » : champ test.frames = 0 : doit être > 0"),
        });
    }
    if let Some(max_hits) = test.max_hits {
        if test.min_hits > max_hits {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.display().to_string(),
                message: format!(
                    "arme « {id} » : champ test.min_hits = {} > test.max_hits = {}",
                    test.min_hits, max_hits
                ),
            });
        }
    }
}

fn lint_melee_weapons(registry: &Registry, errors: &mut Vec<LintError>) {
    for weapon in registry.melee_weapons.values() {
        if weapon.damage.get() < Fixed::ZERO {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: weapon.file.display().to_string(),
                message: format!(
                    "arme de corps à corps « {} » : champ damage = {} : doit être >= 0",
                    weapon.id,
                    weapon.damage.get()
                ),
            });
        }
        lint_weapon_test(&weapon.id, &weapon.file, weapon.test.as_ref(), errors);
    }
}

fn lint_waves(registry: &Registry, errors: &mut Vec<LintError>) {
    for wave in registry.waves.values() {
        for enemy in &wave.enemy_refs {
            let known = registry
                .characters
                .contains_key(&registry::CharacterId::from(enemy.as_str()));
            if !known {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: wave.file.display().to_string(),
                    message: format!(
                        "vagues « {} » : champ enemy_probabilities référence un personnage inconnu « {} »",
                        wave.id, enemy
                    ),
                });
            }
        }
    }
}

fn lint_entry_point(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let start_map_id = registry::map_id_from_path(&manifest.entry.start_map);
    if !registry.maps.contains_key(&start_map_id) {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: format!(
                "champ entry.start_map = « {} » : aucune carte chargée avec cet id (« {} »)",
                manifest.entry.start_map, start_map_id
            ),
        });
    }
}
