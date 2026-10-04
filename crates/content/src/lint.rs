//! Validation sémantique du contenu déjà chargé par [`crate::registry::Registry::build`] :
//! références cassées, valeurs hors plage. Chaque erreur nomme le fichier, l'id concerné
//! et le champ en cause (carte T1.5, `docs/taches.md`).
//!
//! Les erreurs de *chargement* (RON cassé, id dupliqué, kind de dossier inconnu) sont
//! rapportées par `Registry::build` lui-même : elles empêchent de construire l'entrée sur
//! laquelle porteraient les règles ci-dessous. `alacod lint` (et tout appelant) concatène
//! les deux listes.

use bevy_fixed::fixed_math::Fixed;
use sim_core::ammo::AmmoType;
use sim_core::modifier::ModifierOp;
use std::collections::{BTreeMap, BTreeSet};

use crate::expr::NumOrExpr;
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
/// `registry`. Ne relit aucun fichier de contenu : opère sur les entrées déjà parsées. Seule
/// exception : l'**existence** des sons référencés par une arme (`audio_config`) ou
/// par le feedback (`ui/feedback.ron`), et des fichiers d'une feuille de sprites (D3 :
/// animation, feuilles, images), vérifiée sous `registry.game_dir/assets/`.
pub fn run(registry: &Registry, manifest: &GameManifest) -> Vec<LintError> {
    let mut errors = Vec::new();

    lint_characters(registry, &mut errors);
    lint_weapons(registry, &mut errors);
    lint_melee_weapons(registry, &mut errors);
    lint_waves(registry, &mut errors);
    lint_economy(registry, &mut errors);
    lint_perks(registry, &mut errors);
    lint_powerups(registry, &mut errors);
    lint_feedback(registry, &mut errors);
    lint_sprite_sheets(registry, &mut errors);
    lint_floors(registry, &mut errors);
    lint_caves(registry, &mut errors);
    lint_patterns(registry, &mut errors);
    lint_entry_point(registry, manifest, &mut errors);

    errors
}

fn lint_characters(registry: &Registry, errors: &mut Vec<LintError>) {
    for character in registry.characters.values() {
        let file = character.file.display().to_string();

        // T1.2 : tir à distance (`ai.ranged`).
        if let Some(ranged) = &character.ranged {
            lint_ranged(registry, character, ranged, errors);
        }

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

        if character.starting_weapons.len() > character.weapon_slots as usize {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : starting_weapons contient {} armes : dépasse weapon_slots = {}",
                    character.id, character.starting_weapons.len(), character.weapon_slots
                ),
            });
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

        // Hors plage : santé > 0 (littéral seulement ; une expression est validée par
        // `lint_num_or_expr`).
        if let Some(max) = character.base_health_max.literal() {
            if max <= Fixed::ZERO {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "personnage « {} » : champ base_health.max = {max} : doit être > 0",
                        character.id
                    ),
                });
            }
        }
        lint_num_or_expr(
            &character.base_health_max,
            &format!("personnage « {} », champ base_health.max", character.id),
            &file,
            false,
            errors,
        );

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

        // Hors plage (T1.3, chantier B6, « À terre ») : le minuteur de saignement et le
        // temps de réanimation doivent avancer (0 frame boucherait le personnage dans un
        // état transitoire, ou le réanimerait instantanément).
        if character.bleedout_frames == 0 {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ bleedout_frames = 0 : doit être > 0",
                    character.id
                ),
            });
        }
        if character.revive_frames == 0 {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ revive_frames = 0 : doit être > 0",
                    character.id
                ),
            });
        }
        // `downed_speed_mult` multiplie `MoveSpeed` (`ModifierOp::Mul`) : 0 fige le
        // personnage à terre, au-delà de 1 il irait plus vite à terre que debout.
        let downed_speed_mult = character.downed_speed_mult.get();
        if downed_speed_mult <= Fixed::ZERO || downed_speed_mult > Fixed::from_num(1.0) {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ downed_speed_mult = {} : doit être dans ]0, 1]",
                    character.id, downed_speed_mult
                ),
            });
        }

        // Hors plage (T2.2, chantier B7) : zéro emplacement bloquerait tout ramassage
        // d'arme (voir `interaction::handle_weapon_pickup_interaction`).
        if character.weapon_slots == 0 {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : champ weapon_slots = 0 : doit être > 0",
                    character.id
                ),
            });
        }
    }
}

fn lint_weapons(registry: &Registry, errors: &mut Vec<LintError>) {
    let assets_dir = GameManifest::assets_dir(&registry.game_dir);
    for weapon in registry.weapons.values() {
        // T2.8 : `Custom` est la porte d'un type de munition propre au jeu ; un nom vide (ou
        // fait d'espaces) ne désigne rien et partagerait la réserve de toute autre arme
        // `Custom("")` par accident.
        if let AmmoType::Custom(name) = &weapon.ammo_type {
            if name.trim().is_empty() {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : champ ammo_type = Custom({name:?}) : le nom d'un type de munition personnalisé ne doit pas être vide",
                        weapon.id
                    ),
                });
            }
        }
        // T2.8 : un son d'arme doit exister sous `assets/` (référence vers un fichier, pas
        // vers un id de contenu).
        for sound in &weapon.sounds {
            if !assets_dir.join(&sound.path).is_file() {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : champ {} = « {} » : fichier absent de assets/",
                        weapon.id, sound.field, sound.path
                    ),
                });
            }
        }
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
        lint_weapon_projectiles(weapon, &registry.patterns, errors);
        // D3 : `sprite_config.name` désigne une entrée de la table `SpriteSheet`. Vérifié
        // seulement si le jeu en déclare une : sans table, aucun sprite n'est chargé (les
        // fixtures de lint n'en ont pas).
        if let Some(sprite) = &weapon.sprite {
            if !registry.sprite_sheets.is_empty()
                && !registry
                    .sprite_sheets
                    .contains_key(&registry::SpriteSheetId::from(sprite.as_str()))
            {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : champ sprite_config.name = « {sprite} » : feuille de sprites inconnue (kind SpriteSheet)",
                        weapon.id
                    ),
                });
            }
        }
    }
}

/// T1.1 (B5 v1) : projectiles composables d'une arme (`projectile:` de chaque mode, table
/// `projectiles`). Règles : chaque modificateur au plus une fois, `Size > 0`,
/// `0 < Homing <= 1` ; `on_hit` limité aux actions à modificateur (`TimedModifier`,
/// `CurrencyMultiplier`) ; patterns de `on_expire` instantanés (`Telegraph`/`Wait`
/// refusés), `count > 0`, `spread >= 0`, `speed >= 0`, projectile référencé présent dans la
/// table ; définitions de la table : `damage >= 0`, `speed >= 0`, `range > 0`, aucun cycle
/// de `on_expire`.
fn lint_weapon_projectiles(
    weapon: &registry::WeaponEntry,
    patterns: &Patterns,
    errors: &mut Vec<LintError>,
) {
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: weapon.file.display().to_string(),
            message: format!("arme « {} » : {message}", weapon.id),
        });
    };
    for (mode, spec) in &weapon.mode_projectiles {
        lint_projectile_spec(
            &format!("mode « {mode} » : projectile"),
            spec,
            &weapon.projectiles,
            patterns,
            &mut push,
        );
    }
    for (id, def) in &weapon.projectiles {
        let at = format!("projectiles « {id} »");
        for (field, value, strict) in [
            ("damage", def.damage, false),
            ("speed", def.speed, false),
            ("range", def.range, true),
        ] {
            let value = value.get();
            if value < Fixed::ZERO || (strict && value == Fixed::ZERO) {
                let rule = if strict { "> 0" } else { ">= 0" };
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : champ {field} = {value} : doit être {rule}"),
                );
            }
        }
        lint_projectile_spec(&at, &def.spec(), &weapon.projectiles, patterns, &mut push);
    }
    // Cycles de `on_expire` dans la table (un projectile qui finit par se refaire naître).
    for start in weapon.projectiles.keys() {
        let mut seen = BTreeSet::new();
        let mut stack = vec![start.as_str()];
        while let Some(id) = stack.pop() {
            let Some(def) = weapon.projectiles.get(id) else {
                continue;
            };
            for next in expire_references(&def.on_expire, patterns) {
                if next == start {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("projectiles « {start} » : on_expire se refait naître (cycle)"),
                    );
                    stack.clear();
                    break;
                }
                if seen.insert(next) {
                    stack.push(next);
                }
            }
        }
    }
}

/// T1.2 : patterns nommés du jeu (kind `Pattern`), pour résoudre `Named`.
type Patterns = BTreeMap<registry::PatternId, registry::PatternFileEntry>;

/// Garde-fou de résolution des `Named` imbriqués (le même que
/// `combat::projectile::MAX_NAMED_DEPTH`) ; un cycle est rapporté par [`lint_patterns`].
const MAX_NAMED_DEPTH: u8 = 8;

/// Ids de projectiles nommés par un pattern, `Named` suivis dans `patterns` (un nom
/// inconnu ne donne rien : rapporté ailleurs).
fn pattern_projectiles<'a>(
    pattern: &'a registry::PatternEntry,
    patterns: &'a Patterns,
    depth: u8,
    out: &mut Vec<&'a str>,
) {
    use registry::PatternEntry;
    match pattern {
        PatternEntry::Aimed { projectile, .. }
        | PatternEntry::Spread { projectile, .. }
        | PatternEntry::Ring { projectile, .. }
        | PatternEntry::Scatter { projectile, .. } => out.push(projectile),
        PatternEntry::Sequence(children) => children
            .iter()
            .for_each(|child| pattern_projectiles(child, patterns, depth, out)),
        PatternEntry::Named(name) => {
            if depth < MAX_NAMED_DEPTH {
                if let Some(entry) = patterns.get(&registry::PatternId::from(name.clone())) {
                    pattern_projectiles(&entry.pattern, patterns, depth + 1, out);
                }
            }
        }
        PatternEntry::Telegraph(_) | PatternEntry::Wait(_) => {}
    }
}

/// Ids de projectiles nommés par les patterns d'une liste `on_expire`.
fn expire_references<'a>(
    on_expire: &'a [registry::ExpireActionEntry],
    patterns: &'a Patterns,
) -> Vec<&'a str> {
    let mut out = Vec::new();
    for action in on_expire {
        if let registry::ExpireActionEntry::Spawn(pattern) = action {
            pattern_projectiles(pattern, patterns, 0, &mut out);
        }
    }
    out
}

fn lint_projectile_spec(
    at: &str,
    spec: &registry::ProjectileSpecEntry,
    table: &BTreeMap<String, registry::ProjectileDefEntry>,
    patterns: &Patterns,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    let mut seen = BTreeSet::new();
    for modifier in &spec.modifiers {
        if !seen.insert(modifier.name()) {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : modificateur {} répété", modifier.name()),
            );
        }
        match modifier {
            registry::ProjectileModifierEntry::Size(size) if size.get() <= Fixed::ZERO => push(
                LintErrorKind::OutOfRange,
                format!("{at} : Size({}) : doit être > 0", size.get()),
            ),
            registry::ProjectileModifierEntry::Homing(force)
                if force.get() <= Fixed::ZERO
                    || force.get() > bevy_fixed::fixed_math::FIXED_ONE =>
            {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : Homing({}) : doit être dans ]0, 1]", force.get()),
                )
            }
            _ => {}
        }
    }
    for action in &spec.on_hit {
        // T1.6 : `DestroyTerrain` creuse au point d'impact (mur de caverne touché)
        if let effects::Action::DestroyTerrain { radius } = action {
            if *radius <= Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : on_hit DestroyTerrain : radius = {radius} : doit être > 0"),
                );
            }
            continue;
        }
        if !matches!(
            action,
            effects::Action::TimedModifier { .. } | effects::Action::CurrencyMultiplier { .. }
        ) {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "{at} : on_hit {action:?} : seules les actions à modificateur (TimedModifier, CurrencyMultiplier) s'appliquent à la cible"
                ),
            );
        }
    }
    for action in &spec.on_expire {
        match action {
            registry::ExpireActionEntry::Spawn(pattern) => {
                lint_expire_pattern(at, pattern, table, patterns, 0, push)
            }
            // T1.6 : `on_expire: [DestroyTerrain(radius: "40")]`
            registry::ExpireActionEntry::DestroyTerrain { radius } => {
                if radius.get() <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "{at} : on_expire DestroyTerrain : radius = {} : doit être > 0",
                            radius.get()
                        ),
                    );
                }
            }
        }
    }
}

fn lint_expire_pattern(
    at: &str,
    pattern: &registry::PatternEntry,
    table: &BTreeMap<String, registry::ProjectileDefEntry>,
    patterns: &Patterns,
    depth: u8,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    use registry::PatternEntry;
    let (count, spread, speed, projectile) = match pattern {
        PatternEntry::Aimed {
            count,
            spread,
            projectile,
        }
        | PatternEntry::Spread {
            count,
            spread,
            projectile,
        } => (*count, Some(spread.get()), None, projectile),
        PatternEntry::Ring {
            count,
            speed,
            projectile,
            ..
        } => (*count, None, Some(speed.get()), projectile),
        PatternEntry::Sequence(children) => {
            for child in children {
                lint_expire_pattern(at, child, table, patterns, depth, push);
            }
            return;
        }
        PatternEntry::Telegraph(_) | PatternEntry::Wait(_) => {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : on_expire : pattern temporel {pattern:?} : seuls Aimed, Spread, Ring et Sequence sont joués à la fin d'un projectile"),
            );
            return;
        }
        // T1.2 : aléatoire (flux `patterns`), réservé aux émetteurs.
        PatternEntry::Scatter { .. } => {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : on_expire : pattern aléatoire Scatter : réservé aux émetteurs, seuls Aimed, Spread, Ring et Sequence sont joués à la fin d'un projectile"),
            );
            return;
        }
        // T1.2 : pattern nommé, linté comme s'il était écrit ici (cycle : `lint_patterns`).
        PatternEntry::Named(name) => {
            match patterns.get(&registry::PatternId::from(name.clone())) {
                None => push(
                    LintErrorKind::BrokenReference,
                    format!("{at} : on_expire : pattern nommé « {name} » inconnu (kind Pattern)"),
                ),
                Some(entry) if depth < MAX_NAMED_DEPTH => lint_expire_pattern(
                    &format!("{at} : pattern « {name} »"),
                    &entry.pattern,
                    table,
                    patterns,
                    depth + 1,
                    push,
                ),
                Some(_) => {}
            }
            return;
        }
    };
    if count == 0 {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : on_expire : count = 0 : doit être > 0"),
        );
    }
    if spread.is_some_and(|spread| spread < Fixed::ZERO) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : on_expire : spread < 0"),
        );
    }
    if speed.is_some_and(|speed| speed < Fixed::ZERO) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : on_expire : speed < 0"),
        );
    }
    if !table.contains_key(projectile) {
        push(
            LintErrorKind::BrokenReference,
            format!("{at} : on_expire : projectile « {projectile} » absent de la table projectiles de l'arme"),
        );
    }
}

/// D3 : chaque fichier d'une entrée `SpriteSheet` (animation, feuille de chaque calque,
/// image de chaque feuille) doit exister sous `assets/`.
fn lint_sprite_sheets(registry: &Registry, errors: &mut Vec<LintError>) {
    let assets_dir = GameManifest::assets_dir(&registry.game_dir);
    for sheet in registry.sprite_sheets.values() {
        let file = sheet.file.display().to_string();
        let mut check = |field: String, path: &str| {
            if !assets_dir.join(path).is_file() {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "feuille de sprites « {} » : champ {field} = « {path} » : fichier absent de assets/",
                        sheet.id
                    ),
                });
            }
        };
        check("animation".to_string(), &sheet.animation);
        for (layer, path) in &sheet.layers {
            check(format!("layers.{layer}"), path);
        }
        for (layer, image) in &sheet.images {
            check(format!("layers.{layer} (path de la feuille)"), image);
        }
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

/// T2.3, chantier C5 v1 : prix > 0 (un perk gratuit n'a pas de sens — pas de cas d'usage
/// documenté par la tâche). « stat connue » n'a pas besoin d'une règle ici : un `StatId`
/// inconnu dans `modifiers[].stat` échoue déjà au chargement RON
/// (`registry::PerkModifierSchema`, voir sa doc), rapporté comme `LintErrorKind::Parse` par
/// `Registry::build` avant même d'atteindre ce lint.
///
/// T2.8 : au moins un modificateur (un perk sans effet se paie pour rien), et `value > 0`
/// pour un `op: Mul` (0 annulerait la stat, une valeur négative l'inverserait). Les autres
/// opérations n'ont pas de plage interdite (`Add`/`Pct` négatifs = malus voulu, `Set`
/// arbitraire). Un id écrit deux fois dans `perks.ron` est rapporté au chargement
/// (`DuplicateId`, voir `registry::KeyedEntries`).
fn lint_perks(registry: &Registry, errors: &mut Vec<LintError>) {
    for perk in registry.perks.values() {
        let file = perk.file.display().to_string();
        // Littéral seulement ; une expression est validée par `lint_num_or_expr`.
        if let Some(price) = perk.price.literal() {
            if price <= Fixed::ZERO {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "perk « {} » : champ price = {price} : doit être > 0",
                        perk.id
                    ),
                });
            }
        }
        lint_num_or_expr(
            &perk.price,
            &format!("perk « {} », champ price", perk.id),
            &file,
            true,
            errors,
        );
        if perk.modifiers.is_empty() {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "perk « {} » : champ modifiers vide : doit contenir au moins un modificateur",
                    perk.id
                ),
            });
        }
        for (index, modifier) in perk.modifiers.iter().enumerate() {
            let value = modifier.value.get();
            if modifier.op == ModifierOp::Mul && value <= Fixed::ZERO {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "perk « {} » : modifiers[{index}] (stat {:?}, op Mul) : champ value = {value} : doit être > 0",
                        perk.id, modifier.stat
                    ),
                });
            }
        }
    }
}

/// T2.8 (dette T2.3) : `refill_price_ratio` dans `[0, 1]` : au-delà de 1, recharger une
/// arme murale déjà possédée coûterait plus cher que l'acheter ; négatif, le prix
/// (`EconomyConfig::refill_price`, arrondi en `u32`) n'aurait pas de sens. F5 (chantier
/// m0-v11) : la règle de plage s'applique aux littéraux, les expressions sont validées par
/// `lint_num_or_expr`.
fn lint_economy(registry: &Registry, errors: &mut Vec<LintError>) {
    for economy in registry.economy.values() {
        let file = economy.file.display().to_string();
        if let Some(ratio) = economy.refill_price_ratio.literal() {
            if ratio < Fixed::ZERO || ratio > Fixed::from_num(1.0) {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "économie « {} » : champ refill_price_ratio = {ratio} : doit être dans [0, 1]",
                        economy.id
                    ),
                });
            }
        }
        // F5 : expressions (identifiants et évaluation) pour tous les champs numériques.
        lint_num_or_expr(
            &economy.kill_points,
            &format!("économie « {} », champ kill_points", economy.id),
            &file,
            true,
            errors,
        );
        lint_num_or_expr(
            &economy.hit_points,
            &format!("économie « {} », champ hit_points", economy.id),
            &file,
            true,
            errors,
        );
        lint_num_or_expr(
            &economy.repair_points,
            &format!("économie « {} », champ repair_points", economy.id),
            &file,
            true,
            errors,
        );
        lint_num_or_expr(
            &economy.nuke_points,
            &format!("économie « {} », champ nuke_points", economy.id),
            &file,
            true,
            errors,
        );
        if let Some(cap) = &economy.repair_points_cap_per_wave {
            lint_num_or_expr(
                cap,
                &format!(
                    "économie « {} », champ repair_points_cap_per_wave",
                    economy.id
                ),
                &file,
                true,
                errors,
            );
        }
        lint_num_or_expr(
            &economy.refill_price_ratio,
            &format!("économie « {} », champ refill_price_ratio", economy.id),
            &file,
            false,
            errors,
        );
    }
}

/// F5 (chantier m0-v11) : valide l'expression d'un champ `NumOrExpr` (`Expression`) —
/// no-op pour un littéral, dont les règles de plage existantes s'occupent déjà.
///
/// Deux règles, en miroir du chargement strict de `game::balance` (où une erreur est un
/// échec de chargement, jamais une valeur par défaut silencieuse) :
/// 1. identifiants limités à `players`, la seule variable du contexte
///    (`expr::players_context`, `docs/conventions.md` §18) ;
/// 2. l'expression s'évalue pour 1 et 4 joueurs (division par zéro, résultat négatif sur
///    un champ `u32`, etc.).
fn lint_num_or_expr(
    num: &NumOrExpr,
    what: &str,
    file: &str,
    is_u32: bool,
    errors: &mut Vec<LintError>,
) {
    let NumOrExpr::Expression(expr) = num else {
        return;
    };
    for id in expr.expr().identifiers() {
        if id != "players" {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: file.to_string(),
                message: format!(
                    "{what} : identifiant « {id} » inconnu — seule la variable « players » existe"
                ),
            });
        }
    }
    for players in [1u32, 4] {
        let result = if is_u32 {
            num.resolve_u32(players).map(|_| ())
        } else {
            num.resolve(players).map(|_| ())
        };
        if let Err(e) = result {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.to_string(),
                message: format!(
                    "{what} : expression « {expr} » : {e} (évaluée à {players} joueur(s) ; en jeu, ce serait un échec de chargement)"
                ),
            });
        }
    }
}

/// T2.5, chantier C1 v0 : `drop_chance` dans `[0, 1]` (racine de `items/powerups.ron`) et
/// `weight > 0` pour chaque power-up (un poids nul ne serait jamais tiré : sans intérêt,
/// et exclu pour rester cohérent avec `lint_perks`/`price == 0`). « stat connue » n'a pas
/// besoin d'une règle ici : voir la doc de `registry::PowerUpEntry`/`PowerUpEntrySchema`.
fn lint_powerups(registry: &Registry, errors: &mut Vec<LintError>) {
    if let Some(entry) = &registry.powerup_drop_chance {
        let value = entry.drop_chance.get();
        if value < Fixed::ZERO || value > Fixed::from_num(1.0) {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: entry.file.display().to_string(),
                message: format!("power-ups : champ drop_chance = {value} : doit être dans [0, 1]"),
            });
        }
    }

    for powerup in registry.powerups.values() {
        let file = powerup.file.display().to_string();
        let mut push = |message: String| {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message,
            })
        };
        if powerup.weight == 0 {
            push(format!(
                "power-up « {} » : champ weight = 0 : doit être > 0",
                powerup.id
            ));
        }
        // T2.8 : un power-up qui disparaît à l'apparition, ou qu'on ne peut pas ramasser,
        // n'existe pas pour le joueur.
        if powerup.lifetime_frames == 0 {
            push(format!(
                "power-up « {} » : champ lifetime_frames = 0 : doit être > 0",
                powerup.id
            ));
        }
        let pickup_range = powerup.pickup_range.get();
        if pickup_range <= Fixed::ZERO {
            push(format!(
                "power-up « {} » : champ pickup_range = {pickup_range} : doit être > 0",
                powerup.id
            ));
        }
        // T2.8 : un ramassage sans action n'a aucun effet.
        if powerup.actions.is_empty() {
            push(format!(
                "power-up « {} » : champ actions vide : doit contenir au moins une action",
                powerup.id
            ));
        }
        for (index, action) in powerup.actions.iter().enumerate() {
            match action {
                // `frames: 0` : modificateur posé avec `until = frame de ramassage`, expiré
                // aussitôt (voir `effects::Action::as_modifier`).
                effects::Action::TimedModifier {
                    frames, op, value, ..
                } => {
                    if *frames == 0 {
                        push(format!(
                            "power-up « {} » : actions[{index}] (TimedModifier) : champ frames = 0 : doit être > 0",
                            powerup.id
                        ));
                    }
                    if *op == ModifierOp::Mul && *value <= Fixed::ZERO {
                        push(format!(
                            "power-up « {} » : actions[{index}] (TimedModifier, op Mul) : champ value = {value} : doit être > 0",
                            powerup.id
                        ));
                    }
                }
                effects::Action::CurrencyMultiplier { factor, frames } => {
                    if *frames == 0 {
                        push(format!(
                            "power-up « {} » : actions[{index}] (CurrencyMultiplier) : champ frames = 0 : doit être > 0",
                            powerup.id
                        ));
                    }
                    // `ModifierOp::Mul` sur les points gagnés : 0 les annulerait, une valeur
                    // négative les retirerait.
                    if *factor <= Fixed::ZERO {
                        push(format!(
                            "power-up « {} » : actions[{index}] (CurrencyMultiplier) : champ factor = {factor} : doit être > 0",
                            powerup.id
                        ));
                    }
                }
                // T1.6 : action positionnelle, sans sens pour un power-up (ramassé, pas tiré)
                effects::Action::DestroyTerrain { .. } => push(format!(
                    "power-up « {} » : actions[{index}] (DestroyTerrain) : réservée aux projectiles (on_hit, on_expire)",
                    powerup.id
                )),
                _ => {}
            }
        }
    }
}

/// Réglages de présentation : aucun état de simulation n'est modifié (T3.4).
fn lint_feedback(registry: &Registry, errors: &mut Vec<LintError>) {
    let assets_dir = GameManifest::assets_dir(&registry.game_dir);
    for feedback in &registry.feedback {
        let file = feedback.file.display().to_string();
        for (key, path) in &feedback.sounds {
            if !assets_dir.join(path).is_file() {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "feedback : champ sounds.{key} = « {path} » : fichier absent de assets/"
                    ),
                });
            }
        }
        if feedback.shake_amplitude < 0.0 {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "feedback : champ shake.amplitude = {} : doit être >= 0",
                    feedback.shake_amplitude
                ),
            });
        }
        for (field, frames) in [
            ("hit_flash.frames", feedback.hit_flash_frames),
            ("shake.frames", feedback.shake_frames),
        ] {
            if frames == 0 {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!("feedback : champ {field} = 0 : doit être > 0"),
                });
            }
        }
    }
}

/// T1.8 : une séquence de niveaux (`Floors`) n'est pas vide et chaque niveau désigne une
/// carte chargée (kind `Map`, même résolution par id que `entry.start_map`).
/// T1.2 : patterns nommés (kind `Pattern`) — valeurs (`count > 0`, `spread`/`speed >= 0`),
/// `Named` vers un pattern connu, pas de cycle de `Named`. Les projectiles cités sont
/// vérifiés là où l'arme est connue (`ranged` d'un personnage, `on_expire` d'une arme).
fn lint_patterns(registry: &Registry, errors: &mut Vec<LintError>) {
    for entry in registry.patterns.values() {
        let file = entry.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message: format!("pattern « {} » : {message}", entry.id),
            });
        };
        lint_pattern_values(&entry.pattern, &registry.patterns, &mut push);
        // Cycle : en suivant les `Named` depuis ce pattern, on revient à lui.
        let mut stack = vec![(&entry.pattern, 0u8)];
        let mut cycle = false;
        while let Some((pattern, depth)) = stack.pop() {
            match pattern {
                registry::PatternEntry::Sequence(children) => {
                    stack.extend(children.iter().map(|child| (child, depth)))
                }
                registry::PatternEntry::Named(name) => {
                    if name.as_str() == entry.id.as_str() {
                        cycle = true;
                        break;
                    }
                    if depth < MAX_NAMED_DEPTH {
                        if let Some(next) = registry
                            .patterns
                            .get(&registry::PatternId::from(name.clone()))
                        {
                            stack.push((&next.pattern, depth + 1));
                        }
                    }
                }
                _ => {}
            }
        }
        if cycle {
            push(
                LintErrorKind::OutOfRange,
                "Named se référence lui-même (cycle)".to_string(),
            );
        }
    }
}

/// Valeurs d'un pattern d'émetteur (tous les patterns sont admis, y compris temporels).
fn lint_pattern_values(
    pattern: &registry::PatternEntry,
    patterns: &Patterns,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    use registry::PatternEntry;
    match pattern {
        PatternEntry::Aimed { count, spread, .. }
        | PatternEntry::Spread { count, spread, .. }
        | PatternEntry::Scatter { count, spread, .. } => {
            if *count == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : count = 0 : doit être > 0"),
                );
            }
            if spread.get() < Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : spread < 0"),
                );
            }
        }
        PatternEntry::Ring { count, speed, .. } => {
            if *count == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : count = 0 : doit être > 0"),
                );
            }
            if speed.get() < Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : speed < 0"),
                );
            }
        }
        PatternEntry::Sequence(children) => children
            .iter()
            .for_each(|child| lint_pattern_values(child, patterns, push)),
        PatternEntry::Named(name) => {
            if !patterns.contains_key(&registry::PatternId::from(name.clone())) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("pattern nommé « {name} » inconnu (kind Pattern)"),
                );
            }
        }
        PatternEntry::Telegraph(_) | PatternEntry::Wait(_) => {}
    }
}

/// T1.2 : `ai.ranged` d'un personnage — arme et pattern connus, projectiles du pattern
/// présents dans la table `projectiles` de l'arme, `cooldown_frames > 0`, `range > 0`.
fn lint_ranged(
    registry: &Registry,
    character: &registry::CharacterEntry,
    ranged: &registry::RangedEntry,
    errors: &mut Vec<LintError>,
) {
    let file = character.file.display().to_string();
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: file.clone(),
            message: format!("personnage « {} » : ai.ranged : {message}", character.id),
        });
    };
    let weapon = registry
        .weapons
        .get(&registry::WeaponId::from(ranged.weapon.clone()));
    if weapon.is_none() {
        push(
            LintErrorKind::BrokenReference,
            format!("arme inconnue « {} »", ranged.weapon),
        );
    }
    match registry
        .patterns
        .get(&registry::PatternId::from(ranged.pattern.clone()))
    {
        None => push(
            LintErrorKind::BrokenReference,
            format!("pattern inconnu « {} » (kind Pattern)", ranged.pattern),
        ),
        Some(entry) => {
            if let Some(weapon) = weapon {
                let mut projectiles = Vec::new();
                pattern_projectiles(&entry.pattern, &registry.patterns, 0, &mut projectiles);
                for projectile in projectiles {
                    if !weapon.projectiles.contains_key(projectile) {
                        push(
                            LintErrorKind::BrokenReference,
                            format!(
                                "pattern « {} » : projectile « {projectile} » absent de la table projectiles de l'arme « {} »",
                                ranged.pattern, ranged.weapon
                            ),
                        );
                    }
                }
            }
        }
    }
    if ranged.cooldown_frames == 0 {
        push(
            LintErrorKind::OutOfRange,
            "cooldown_frames = 0 : doit être > 0".to_string(),
        );
    }
    if ranged.range.get() <= Fixed::ZERO {
        push(
            LintErrorKind::OutOfRange,
            format!("range = {} : doit être > 0", ranged.range.get()),
        );
    }
}

fn lint_floors(registry: &Registry, errors: &mut Vec<LintError>) {
    for floors in registry.floors.values() {
        let file = floors.file.display().to_string();
        if floors.levels.is_empty() {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "séquence de niveaux « {} » : champ levels vide (au moins un niveau)",
                    floors.id
                ),
            });
        }
        for level in &floors.levels {
            if let Some(cave) = registry::cave_designation(level) {
                if !registry.caves.contains_key(&cave) {
                    errors.push(LintError {
                        kind: LintErrorKind::BrokenReference,
                        file: file.clone(),
                        message: format!(
                            "séquence de niveaux « {} » : champ levels : « {level} » : aucune caverne chargée avec cet id (« {cave} »)",
                            floors.id
                        ),
                    });
                }
                continue;
            }
            let map_id = registry::map_id_from_path(level);
            if !registry.maps.contains_key(&map_id) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "séquence de niveaux « {} » : champ levels : « {level} » : aucune carte chargée avec cet id (« {map_id} »)",
                        floors.id
                    ),
                });
            }
        }
    }
}

/// T1.6 : plages de l'automate (`docs/conventions.md` §21) et présence du gabarit LDtk.
fn lint_caves(registry: &Registry, errors: &mut Vec<LintError>) {
    for cave in registry.caves.values() {
        let file = cave.file.display().to_string();
        let c = &cave.config;
        let mut out_of_range = |message: String| {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!("caverne « {} » : {message}", cave.id),
            })
        };
        if c.width < 16 || c.height < 16 {
            out_of_range(format!(
                "width × height = {} × {} : au moins 16 × 16 cases",
                c.width, c.height
            ));
        }
        if c.fill_ratio < Fixed::ZERO || c.fill_ratio > Fixed::ONE {
            out_of_range(format!("fill_ratio = {} : hors de [0, 1]", c.fill_ratio));
        }
        if c.min_floor_ratio < Fixed::ZERO || c.min_floor_ratio > Fixed::from_num(0.9) {
            out_of_range(format!(
                "min_floor_ratio = {} : hors de [0, 0.9]",
                c.min_floor_ratio
            ));
        }
        if c.birth > 8 || c.survive > 8 {
            out_of_range(format!(
                "birth = {}, survive = {} : au plus 8 voisins",
                c.birth, c.survive
            ));
        }
        for character in &c.characters {
            if !registry
                .characters
                .contains_key(&registry::CharacterId::from(character.clone()))
            {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "caverne « {} » : champ characters : « {character} » : aucun personnage chargé avec cet id",
                        cave.id
                    ),
                });
            }
        }
        if !GameManifest::assets_dir(&registry.game_dir)
            .join(&cave.template)
            .is_file()
        {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: file.clone(),
                message: format!(
                    "caverne « {} » : gabarit LDtk « {} » introuvable",
                    cave.id, cave.template
                ),
            });
        }
    }
}

fn lint_entry_point(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let start_map_id = registry::map_id_from_path(&manifest.entry.start_map);
    let start_is_known = match registry::cave_designation(&manifest.entry.start_map) {
        Some(cave) => registry.caves.contains_key(&cave),
        None => registry.maps.contains_key(&start_map_id),
    };
    if !start_is_known {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: format!(
                "champ entry.start_map = « {} » : aucune carte chargée avec cet id (« {} »)",
                manifest.entry.start_map, start_map_id
            ),
        });
    }
    // T2.4, chantier F1 : `entry.mode: Waves` explicite exige un dossier `Wave` (sinon le
    // mode `Waves` du run n'aurait aucune config à charger). Pas de règle symétrique pour
    // `Sandbox` : déclarer des vagues sans les utiliser n'est pas une erreur (contenu de
    // test, migration progressive).
    if manifest.entry.mode == Some(crate::manifest::EntryMode::Waves) && registry.waves.is_empty() {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: "champ entry.mode = Waves : aucun dossier de contenu « Wave » déclaré"
                .to_string(),
        });
    }
    // T1.8 : même règle pour `Floors` (la séquence jouée vient d'un dossier `Floors`).
    if manifest.entry.mode == Some(crate::manifest::EntryMode::Floors) && registry.floors.is_empty()
    {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: "champ entry.mode = Floors : aucun dossier de contenu « Floors » déclaré"
                .to_string(),
        });
    }
}
