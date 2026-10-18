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
use sim_core::stats::StatId;

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
    /// T1.10 : contrat déclaré mais pas encore exécuté (déclencheur, condition ou action
    /// d'effet « v2 ») : refusé plutôt qu'ignoré en silence.
    Unsupported,
    /// M2-T15 : modificateur sur une stat qu'aucun système ne lit
    /// (`sim_core::stats::stat_reader`).
    UnreadStat,
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
/// exception : les assets référencés (sons, feuilles de sprites, police de l'écran de
/// mutation) et la lisibilité du gabarit partagé des listes HTML, sous
/// `registry.game_dir/assets/` (ou dans l'inventaire embarqué du navigateur).
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
    lint_hud_sources(registry, &mut errors);
    lint_html_screens(registry, &mut errors);
    lint_sprite_sheets(registry, &mut errors);
    lint_floors(registry, &mut errors);
    lint_caves(registry, &mut errors);
    lint_floor_grammars(registry, &mut errors);
    lint_key_sources(registry, &mut errors);
    lint_surfaces(registry, &mut errors);
    lint_props(registry, &mut errors);
    lint_patterns(registry, &mut errors);
    lint_progression(registry, &mut errors);
    lint_mutations(registry, &mut errors);
    lint_statuses(registry, &mut errors);
    lint_map_characters(registry, &mut errors);
    lint_room_kinds(registry, &mut errors);
    lint_arenas(registry, &mut errors);
    lint_items(registry, &mut errors);
    lint_map_layers(registry, &mut errors);
    lint_synergies(registry, &mut errors);
    lint_loot(registry, &mut errors);
    lint_unlocks(registry, &mut errors);
    lint_curses(registry, &mut errors);
    lint_achievements(registry, &mut errors);
    lint_familiars(registry, &mut errors);
    lint_hub_services(registry, &mut errors);
    lint_entry_progression(registry, manifest, &mut errors);
    lint_forced_variants(registry, &mut errors);
    lint_character_tests(registry, &mut errors);
    lint_item_tests(registry, &mut errors);
    lint_room_tests(registry, &mut errors);
    lint_clocks(registry, manifest, &mut errors);
    lint_difficulty(registry, manifest, &mut errors);
    lint_entry_point(registry, manifest, &mut errors);
    lint_generate_template(registry, manifest, &mut errors);
    lint_stat_readers(registry, &mut errors);

    errors
}

/// T1.10 (`docs/conventions.md` §27) : un effet v1. `at` situe l'effet dans le message.
pub(crate) fn lint_effect(
    registry: &Registry,
    at: &str,
    effect: &effects::Effect,
    allow_scale: bool,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    use effects::runtime::{condition_supported, trigger_supported};
    if !trigger_supported(&effect.on) {
        push(
            LintErrorKind::Unsupported,
            format!(
                "{at} : déclencheur {:?} : pas encore exécuté (v2)",
                effect.on
            ),
        );
    }
    // M3-T2 (§67) : filtre `item` de `OnPickup`
    if let Some(item) = &effect.item {
        if effect.on != effects::On::OnPickup {
            push(
                LintErrorKind::Unsupported,
                format!(
                    "{at} : item « {item} » : filtre réservé à OnPickup (sans effet avec {:?})",
                    effect.on
                ),
            );
        }
        if !registry.items.contains_key(item.as_str()) {
            push(
                LintErrorKind::BrokenReference,
                format!("{at} : OnPickup : objet « {item} » inconnu"),
            );
        }
    }
    // M2-T3 : limites d'exécution
    if effect.cooldown == Some(0) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : cooldown = 0 : doit être > 0 (ou absent)"),
        );
    }
    if effect.charges == Some(0) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : charges = 0 : doit être > 0 (ou absent)"),
        );
    }
    if effect.recharge.is_some() && effect.charges.is_none() {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : recharge sans charges : sans effet (déclarer `charges`)"),
        );
    }
    if effect.on == effects::On::Tick(0) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : Tick(0) : doit être > 0"),
        );
    }
    if matches!(&effect.on, effects::On::OnEvent(name) if name.is_empty()) {
        push(
            LintErrorKind::OutOfRange,
            format!("{at} : OnEvent : nom vide"),
        );
    }
    // Jauges : seule celle de la progression existe en v1
    let gauge_known = |id: &str| registry.progression.values().any(|p| p.gauge == id);
    if let effects::On::OnGauge(id, _) = &effect.on {
        if !gauge_known(id) {
            push(
                LintErrorKind::BrokenReference,
                format!("{at} : OnGauge : jauge « {id} » inconnue (pas la jauge de `progression`)"),
            );
        }
    }
    for condition in &effect.r#if {
        if !condition_supported(condition) {
            push(
                LintErrorKind::Unsupported,
                format!("{at} : condition {condition:?} : pas encore exécutée (v2)"),
            );
        }
    }
    for outer in &effect.r#do {
        // M3-T0b : `Scaled` n'est qu'une enveloppe ; l'action intérieure est validée comme une
        // action nue.
        let mut action = outer;
        let mut per_copy = false;
        while let effects::Action::Scaled {
            action: inner,
            scale,
        } = action
        {
            if !allow_scale {
                push(
                    LintErrorKind::Unsupported,
                    format!("{at} : scale {scale:?} : uniquement dans un effet d'objet"),
                );
            }
            per_copy |= scale.is_some();
            action = inner;
        }
        // M3-T2 (§67) : `PerCopy` ne multiplie que les actions linéaires (`effects::actions::is_linear`) ;
        // sinon l'exécution la sauterait sans rien dire.
        if per_copy && !effects::actions::is_linear(action) {
            push(
                LintErrorKind::Unsupported,
                format!("{at} : Scaled(PerCopy) : action {action:?} non linéaire (Heal, GainShield, GainSoul, Modifier/TimedModifier Add ou Pct, ApplyStatus, ApplyStatusSelf, Explode seulement)"),
            );
        }
        match action {
            effects::Action::Heal(amount) if *amount <= Fixed::ZERO => push(
                LintErrorKind::OutOfRange,
                format!("{at} : Heal = {amount} : doit être > 0"),
            ),
            effects::Action::GainShield(0) => push(
                LintErrorKind::OutOfRange,
                format!("{at} : GainShield(0) : doit être > 0"),
            ),
            effects::Action::GainSoul(amount) if *amount <= Fixed::ZERO => push(
                LintErrorKind::OutOfRange,
                format!("{at} : GainSoul = {amount} : doit être > 0"),
            ),
            // M3-T0c (§62) : contrat de l'explosion ; M3-T4 (§69) l'exécute (bombe à mèche d'une frame
            // au point du porteur)
            effects::Action::Explode { radius, damage, .. } => {
                if *radius <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : Explode : rayon = {radius} : doit être > 0"),
                    );
                }
                if *damage < Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : Explode : dégâts = {damage} : doivent être >= 0"),
                    );
                }
            }
            effects::Action::GaugeAdd(id, _) if !gauge_known(id) => push(
                LintErrorKind::BrokenReference,
                format!(
                    "{at} : GaugeAdd : jauge « {id} » inconnue (pas la jauge de `progression`)"
                ),
            ),
            effects::Action::SpawnPattern { pattern, weapon } => {
                if !registry
                    .patterns
                    .contains_key(&registry::PatternId::from(pattern.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("{at} : SpawnPattern : pattern « {pattern} » inconnu"),
                    );
                }
                if !registry
                    .weapons
                    .contains_key(&registry::WeaponId::from(weapon.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("{at} : SpawnPattern : arme « {weapon} » inconnue"),
                    );
                }
            }
            // M3-T2 (§67) : statut posé sur la cible du déclencheur (`OnHit`,
            // `OnDamageTaken`) ou sur le porteur (`ApplyStatusSelf`)
            effects::Action::ApplyStatus { status, stacks }
            | effects::Action::ApplyStatusSelf { status, stacks } => {
                let on_self = matches!(action, effects::Action::ApplyStatusSelf { .. });
                if !on_self
                    && !matches!(
                        effect.on,
                        effects::On::OnHit | effects::On::OnDamageTaken
                    )
                {
                    push(
                        LintErrorKind::Unsupported,
                        format!("{at} : ApplyStatus : pas de cible hors OnHit et OnDamageTaken (OnKill : la cible est déjà morte ; ApplyStatusSelf pose sur le porteur)"),
                    );
                }
                if !registry
                    .statuses
                    .contains_key(&registry::StatusId::from(status.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("{at} : ApplyStatus : statut « {status} » inconnu"),
                    );
                }
                if *stacks == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : ApplyStatus « {status} » : stacks = 0 : doit être > 0"),
                    );
                }
            }
            // M3-T2 : événement nommé, lu par les `OnEvent` du même porteur
            effects::Action::Emit(name) if name.is_empty() => push(
                LintErrorKind::OutOfRange,
                format!("{at} : Emit : nom vide"),
            ),
            effects::Action::RefillAmmo
            | effects::Action::RefillAmmoOf(_)
            | effects::Action::RepairAllWindows
            | effects::Action::KillAllWaveEnemies
            | effects::Action::DestroyTerrain { .. }
            // M3-T1 : chance d'`on_hit` de projectile seulement
            | effects::Action::Chance { .. } => push(
                LintErrorKind::Unsupported,
                format!("{at} : action {action:?} : pas exécutée dans un effet (v2)"),
            ),
            // M2-T7 : actions de frise de boss
            effects::Action::SpawnEnemies { .. } | effects::Action::Telegraph(_) => push(
                LintErrorKind::Unsupported,
                format!("{at} : action {action:?} : réservée à la frise d'un boss (§45)"),
            ),
            // M3-T0d : division d'un champion à sa mort (`variants`, §63)
            effects::Action::SpawnCharacters { .. } => push(
                LintErrorKind::Unsupported,
                format!("{at} : action {action:?} : réservée à la mort d'un champion (§63)"),
            ),
            _ => {}
        }
    }
}

/// M2-T2 (§43) : règles du dash. Un personnage sans `dash_charges` ni `dash` garde les réglages
/// v1 : les mêmes bornes ne le concernent que s'il déclare des charges ou un dash.
fn lint_dash(character: &registry::CharacterEntry, errors: &mut Vec<LintError>) {
    let dash = &character.dash;
    if !dash.declared {
        return;
    }
    let file = character.file.display().to_string();
    let mut refuse = |message: String| {
        errors.push(LintError {
            kind: LintErrorKind::OutOfRange,
            file: file.clone(),
            message: format!("personnage « {} » : {message}", character.id),
        })
    };
    if let Some(charges) = dash.charges {
        if charges == 0 {
            refuse("dash_charges = 0 : doit être > 0".into());
        }
        if dash.recharge_frames == 0 {
            refuse("dash_recharge_frames = 0 avec des charges : doit être > 0".into());
        }
    }
    if dash.duration_frames == 0 && !dash.in_place {
        refuse("dash : durée 0 hors InPlace".into());
    }
    if dash.iframes > dash.duration_frames.max(1) {
        refuse(format!(
            "dash : i-frames ({}) > durée ({})",
            dash.iframes, dash.duration_frames
        ));
    }
}

fn lint_characters(registry: &Registry, errors: &mut Vec<LintError>) {
    for character in registry.characters.values() {
        let file = character.file.display().to_string();

        // T1.10 : effets v1
        for (index, effect) in character.effects.iter().enumerate() {
            let at = format!("personnage « {} » : effects[{index}]", character.id);
            lint_effect(registry, &at, effect, false, &mut |kind, message| {
                errors.push(LintError {
                    kind,
                    file: file.clone(),
                    message,
                })
            });
        }

        // M2-T2 : dash (charges, recharge, durée, i-frames).
        lint_dash(character, errors);
        // T1.4 : règles de comportement (`ai.behaviors`) et ciblage.
        lint_behaviors(registry, character, errors);
        // M2-T8 : formations.
        lint_formation(character, errors);
        // M2-T0c : boss à phases.
        lint_boss(registry, character, errors);
        // T1.5 : variantes et élites.
        lint_variants(registry, character, errors);
        // M3-T0c : bombe.
        if let Some(bomb) = &character.bomb {
            for message in world::bomb_problems(bomb) {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: character.file.display().to_string(),
                    message,
                });
            }
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

        // M2-T1 : consommables de départ -> objets consommables du kind `Item`.
        for (id, count) in &character.starting_consumables {
            let item = registry
                .items
                .get(id)
                .filter(|item| item.def.kind == items::ItemKind::Consumable);
            let Some(item) = item else {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "personnage « {} » : champ starting_consumables : « {id} » n'est pas un objet consommable (kind Item)",
                        character.id
                    ),
                });
                continue;
            };
            // M3-T3 (§68) : le plafond du compteur vaut aussi au départ.
            if item.def.max_stack.is_some_and(|cap| *count > cap) {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "personnage « {} » : champ starting_consumables : {count} « {id} » dépasse max_stack = {}",
                        character.id,
                        item.def.max_stack.unwrap_or(0)
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

        // D78 : le personnage joueur (`player`) porte le skin "1" pour le handle 0 et "2" pour
        // tous les autres (`game::character::player::create`) : sans "2", le second joueur en
        // ligne n'a pas de skin.
        if character.id.as_str() == "player" {
            for skin in ["1", "2"] {
                if !character.skins.contains(skin) {
                    errors.push(LintError {
                        kind: LintErrorKind::BrokenReference,
                        file: file.clone(),
                        message: format!(
                            "personnage « player » : skin « {skin} » absent de skins ({:?}) : le joueur de handle {} n'aurait pas de skin",
                            character.skins,
                            if skin == "1" { "0" } else { "1 et plus" }
                        ),
                    });
                }
            }
        }
        if character.hit_iframes == Some(0) {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!(
                    "personnage « {} » : hit_iframes = 0 : doit être > 0 (ou absent)",
                    character.id
                ),
            });
        }

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
    for weapon in registry.weapons.values() {
        // T2.8 : `Custom` est la porte d'un type de munition propre au jeu ; un nom vide (ou
        // fait d'espaces) ne désigne rien et partagerait la réserve de toute autre arme
        // `Custom("")` par accident.
        // M3-T0a (§60) : munitions infinies. Une arme dont un mode consomme des munitions doit
        // déclarer `ammo_type` (comme avant M3-T0a, où son absence était une erreur RON) ; une
        // arme dont **tous** les modes sont `Infinite` n'a pas de réserve à consommer et le refuse.
        let consumes = weapon.firing_rates.is_empty()
            || weapon
                .firing_rates
                .keys()
                .any(|mode| !weapon.infinite_modes.contains(mode));
        match (&weapon.ammo_type, consumes) {
            (None, true) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: weapon.file.display().to_string(),
                message: format!(
                    "arme « {} » : champ ammo_type absent : obligatoire pour une arme dont un mode consomme des munitions (Mag, Magless)",
                    weapon.id
                ),
            }),
            (Some(ammo), false) => errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: weapon.file.display().to_string(),
                message: format!(
                    "arme « {} » : ammo_type = {ammo:?} avec des munitions Infinite : pas de réserve à consommer, retirer ammo_type",
                    weapon.id
                ),
            }),
            _ => {}
        }
        if let Some(AmmoType::Custom(name)) = &weapon.ammo_type {
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
            if !registry.asset_exists(&sound.path) {
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
        // Assets (lot A) : `audio_config` ne peut viser qu'un mode de tir de l'arme (sinon
        // le son ne jouerait jamais). Une erreur par mode, même s'il a deux sons.
        let audio_modes: std::collections::BTreeSet<&str> = weapon
            .sounds
            .iter()
            .map(|sound| sound.mode.as_str())
            .collect();
        for mode in audio_modes {
            if !weapon.firing_rates.contains_key(mode) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : audio_config.modes.{mode} : mode de tir inconnu",
                        weapon.id
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
        // D51 : `spread` (radians, pleine largeur : angle dans [−spread/2, spread/2]) entre 0
        // et π ; au-delà, une balle partirait vers l'arrière.
        for (mode, spread) in &weapon.spreads {
            let spread = spread.get();
            if spread < Fixed::ZERO || spread > bevy_fixed::fixed_math::FIXED_PI {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: weapon.file.display().to_string(),
                    message: format!(
                        "arme « {} » : mode « {} » : champ spread = {} : doit être entre 0 et π (radians)",
                        weapon.id, mode, spread
                    ),
                });
            }
        }
        lint_weapon_test(&weapon.id, &weapon.file, weapon.test.as_ref(), errors);
        lint_weapon_projectiles(weapon, &registry.patterns, errors);
        lint_weapon_statuses(registry, weapon, errors);
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
    // M3-T1 : cycles de `Split` dans la table (un enfant qui finit par se diviser en lui-même).
    let split_children = |id: &str| -> Vec<&str> {
        weapon
            .projectiles
            .get(id)
            .map(|def| {
                def.modifiers
                    .iter()
                    .filter_map(|modifier| match modifier {
                        registry::ProjectileModifierEntry::Split {
                            child: Some(child), ..
                        } => Some(child.as_str()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    for start in weapon.projectiles.keys() {
        let mut seen = BTreeSet::new();
        let mut stack = vec![start.as_str()];
        while let Some(id) = stack.pop() {
            for next in split_children(id) {
                if next == start {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("projectiles « {start} » : Split se divise en lui-même (cycle)"),
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
        | PatternEntry::Scatter { projectile, .. }
        | PatternEntry::Fan { projectile, .. }
        | PatternEntry::Spiral { projectile, .. }
        | PatternEntry::Burst { projectile, .. } => out.push(projectile),
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
            // M3-T0a (§60), exécutés par M3-T1 (§66) : formes et plages.
            registry::ProjectileModifierEntry::Spectral => {}
            registry::ProjectileModifierEntry::Split {
                count,
                spread,
                child,
            } => {
                lint_split_orbit(at, Some((*count, spread.get())), None, push);
                if let Some(child) = child {
                    if !table.contains_key(child) {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : Split : child « {child} » absent de la table projectiles de l'arme"),
                        );
                    }
                }
            }
            registry::ProjectileModifierEntry::Orbit {
                radius,
                frames_per_turn,
            } => lint_split_orbit(at, None, Some((radius.get(), *frames_per_turn)), push),
            _ => {}
        }
    }
    if spec
        .modifiers
        .iter()
        .any(|m| matches!(m, registry::ProjectileModifierEntry::Orbit { .. }))
        && spec.modifiers.iter().any(|m| {
            matches!(
                m,
                registry::ProjectileModifierEntry::Homing(_)
                    | registry::ProjectileModifierEntry::Gravity(_)
            )
        })
    {
        push(
            LintErrorKind::Unsupported,
            format!(
                "{at} : Orbit avec Homing ou Gravity : l'orbite impose la vitesse du projectile"
            ),
        );
    }
    for action in &spec.on_hit {
        lint_on_hit_action(at, action, push);
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

/// `ApplyStatus` d'une action d'`on_hit`, `Chance` déballée (M3-T1) : `(statut, piles)`.
fn on_hit_status(action: &effects::Action) -> Option<(&str, u32)> {
    match action {
        effects::Action::ApplyStatus { status, stacks } => Some((status.as_str(), *stacks)),
        effects::Action::Chance { then, .. } => on_hit_status(then),
        _ => None,
    }
}

/// Une action d'`on_hit` d'un projectile (arme ou objet, M3-T1) : `DestroyTerrain` (rayon > 0),
/// les actions à modificateur et `ApplyStatus` (posées sur la cible), `Chance` (base dans
/// `[0, 1]`, qui enveloppe une de ces trois dernières, pas `DestroyTerrain` ni une autre
/// `Chance`). Le reste est refusé.
fn lint_on_hit_action(
    at: &str,
    action: &effects::Action,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    match action {
        // T1.6 : `DestroyTerrain` creuse au point d'impact (mur de caverne touché)
        effects::Action::DestroyTerrain { radius } => {
            if *radius <= Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : on_hit DestroyTerrain : radius = {radius} : doit être > 0"),
                );
            }
        }
        effects::Action::TimedModifier { .. }
        | effects::Action::CurrencyMultiplier { .. }
        // T1.3 : statut posé sur la cible (référence vérifiée par `lint_weapon_statuses`)
        | effects::Action::ApplyStatus { .. } => {}
        effects::Action::Chance { base, then, .. } => {
            if *base < Fixed::ZERO || *base > Fixed::ONE {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : on_hit Chance : base = {base} : doit être dans [0, 1]"),
                );
            }
            if matches!(
                **then,
                effects::Action::Chance { .. } | effects::Action::DestroyTerrain { .. }
            ) {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : on_hit Chance : then {then:?} : une action à modificateur ou ApplyStatus seulement"),
                );
            } else {
                lint_on_hit_action(at, then, push);
            }
        }
        other => push(
            LintErrorKind::OutOfRange,
            format!(
                "{at} : on_hit {other:?} : seules les actions à modificateur (TimedModifier, CurrencyMultiplier), ApplyStatus, DestroyTerrain et Chance s'appliquent à la cible"
            ),
        ),
    }
}

/// M3-T0a : plages de `Split` (`count > 0`, `spread >= 0`) et d'`Orbit` (`radius > 0`,
/// `frames_per_turn > 0`).
fn lint_split_orbit(
    at: &str,
    split: Option<(u32, Fixed)>,
    orbit: Option<(Fixed, u32)>,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    if let Some((count, spread)) = split {
        if count == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : Split : count = 0 : doit être > 0"),
            );
        }
        if spread < Fixed::ZERO {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : Split : spread = {spread} : doit être >= 0"),
            );
        }
    }
    if let Some((radius, frames_per_turn)) = orbit {
        if radius <= Fixed::ZERO {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : Orbit : radius = {radius} : doit être > 0"),
            );
        }
        if frames_per_turn == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : Orbit : frames_per_turn = 0 : doit être > 0"),
            );
        }
    }
}

/// M3-T0a (§60), lu au tir par M3-T1 (§66) : `projectile:` d'un objet ou d'une synergie
/// (`effects::projectile::ProjectileMods`). Mêmes règles de forme que la liste d'une arme (chaque
/// modificateur au plus une fois **dans cette liste**, l'empilement vaut entre sources ;
/// `Size > 0`, `0 < Homing <= 1`, plages de `Split`/`Orbit`, `Orbit` sans `Homing` ni `Gravity`).
/// Un objet n'a pas de table `projectiles` (celle de l'arme qui tire) : `Split` sans `child`
/// (l'enfant par défaut du porteur) et `on_expire` sans `Spawn` seulement. `on_hit` : mêmes
/// actions qu'une arme, statut connu.
fn lint_carried_projectile(
    registry: &Registry,
    at: &str,
    mods: &effects::projectile::ProjectileMods,
    push: &mut impl FnMut(LintErrorKind, String),
) {
    use effects::projectile::ProjectileModifier as M;
    let mut seen = BTreeSet::new();
    for modifier in &mods.modifiers {
        if !seen.insert(modifier.name()) {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : modificateur {} répété", modifier.name()),
            );
        }
        match modifier {
            M::Size(size) if *size <= Fixed::ZERO => push(
                LintErrorKind::OutOfRange,
                format!("{at} : Size({size}) : doit être > 0"),
            ),
            M::Homing(force)
                if *force <= Fixed::ZERO || *force > bevy_fixed::fixed_math::FIXED_ONE =>
            {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : Homing({force}) : doit être dans ]0, 1]"),
                )
            }
            M::Split {
                count,
                spread,
                child,
            } => {
                lint_split_orbit(at, Some((*count, *spread)), None, push);
                if let Some(child) = child {
                    push(
                        LintErrorKind::Unsupported,
                        format!("{at} : Split : child « {child} » : un objet n'a pas de table projectiles (celle de l'arme) ; Split sans child (copie du tir du porteur) seulement"),
                    );
                }
            }
            M::Orbit {
                radius,
                frames_per_turn,
            } => lint_split_orbit(at, None, Some((*radius, *frames_per_turn)), push),
            _ => {}
        }
    }
    if mods.modifiers.iter().any(|m| matches!(m, M::Orbit { .. }))
        && mods
            .modifiers
            .iter()
            .any(|m| matches!(m, M::Homing(_) | M::Gravity(_)))
    {
        push(
            LintErrorKind::Unsupported,
            format!(
                "{at} : Orbit avec Homing ou Gravity : l'orbite impose la vitesse du projectile"
            ),
        );
    }
    for action in &mods.on_hit {
        lint_on_hit_action(at, action, push);
        if let Some((status, stacks)) = on_hit_status(action) {
            if !registry
                .statuses
                .contains_key(&registry::StatusId::from(status.to_string()))
            {
                push(
                    LintErrorKind::BrokenReference,
                    format!("{at} : ApplyStatus : statut « {status} » inconnu"),
                );
            }
            if stacks == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : ApplyStatus « {status} » : stacks = 0 : doit être > 0"),
                );
            }
        }
    }
    for action in &mods.on_expire {
        match action {
            effects::projectile::ExpireAction::Spawn(_) => push(
                LintErrorKind::Unsupported,
                format!("{at} : on_expire Spawn : un objet n'a pas de table projectiles (celle de l'arme) ; DestroyTerrain seulement"),
            ),
            effects::projectile::ExpireAction::DestroyTerrain { radius } => {
                if *radius <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : on_expire DestroyTerrain : radius = {radius} : doit être > 0"),
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
        PatternEntry::Fan {
            count,
            spread,
            speed,
            projectile,
        } => {
            if speed.get() < Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : on_expire : Fan speed < 0"),
                );
            }
            (*count, Some(spread.get()), None, projectile)
        }
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
        // T1.2 : aléatoire (flux `patterns`), M2-T1 : multi-salves, réservés aux émetteurs.
        PatternEntry::Scatter { .. } | PatternEntry::Spiral { .. } | PatternEntry::Burst { .. } => {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : on_expire : pattern {pattern:?} (aléatoire ou multi-salves) : réservé aux émetteurs, seuls Aimed, Spread, Fan, Ring et Sequence sont joués à la fin d'un projectile"),
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
    for sheet in registry.sprite_sheets.values() {
        let file = sheet.file.display().to_string();
        let mut check = |field: String, path: &str| {
            if !registry.asset_exists(path) {
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

/// `test:` d'un personnage (T1.13, `docs/conventions.md` §28) : `frames > 0`, au moins un
/// gabarit actif (`still`/`moving`/`boss`), attentes explicites seulement pour un gabarit actif.
/// M2-T17 (§54) : `boss:` seulement sur un personnage qui a `boss:`, scène connue.
fn lint_character_tests(registry: &Registry, errors: &mut Vec<LintError>) {
    for character in registry.characters.values() {
        let Some(test) = &character.test else {
            continue;
        };
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: character.file.display().to_string(),
                message: format!("personnage « {} » : {message}", character.id),
            });
        };
        if test.frames == 0 {
            push(
                LintErrorKind::OutOfRange,
                "champ test.frames = 0 : doit être > 0".to_string(),
            );
        }
        if !test.still && !test.moving && test.boss.is_none() && test.champions.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                "test : aucun gabarit actif (still et moving faux)".to_string(),
            );
        }
        if !test.still && test.expect_still > 0 {
            push(
                LintErrorKind::OutOfRange,
                "test.expect_still sans gabarit still actif".to_string(),
            );
        }
        if !test.moving && test.expect_moving > 0 {
            push(
                LintErrorKind::OutOfRange,
                "test.expect_moving sans gabarit moving actif".to_string(),
            );
        }
        // M3-T13 (§78) : gabarit ChampionForced, une entrée par variante de la table.
        let mut seen = BTreeSet::new();
        for champion in &test.champions {
            if !seen.insert(champion.variant.as_str()) {
                push(
                    LintErrorKind::DuplicateId,
                    format!("test.champions : variante « {} » répétée", champion.variant),
                );
            }
            let known = character.variants.as_ref().is_some_and(|variants| {
                variants
                    .table
                    .iter()
                    .any(|(name, _)| *name == champion.variant)
            });
            if !known {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "test.champions : variante « {} » absente de `variants`",
                        champion.variant
                    ),
                );
            }
            if champion.kill {
                let translatable = character.variants.as_ref().is_some_and(|variants| {
                    variants
                        .table
                        .iter()
                        .filter(|(name, _)| *name == champion.variant)
                        .any(|(_, def)| {
                            def.loot.is_some()
                                || def.on_death.iter().any(|action| {
                                    matches!(
                                        action,
                                        effects::Action::Explode { .. }
                                            | effects::Action::SpawnCharacters { .. }
                                            | effects::Action::SpawnPattern { .. }
                                    )
                                })
                        })
                });
                if known && !translatable {
                    push(
                        LintErrorKind::Unsupported,
                        format!(
                            "test.champions : variante « {} » : kill: true sans effet de mort traduisible (Explode, SpawnCharacters, SpawnPattern, loot)",
                            champion.variant
                        ),
                    );
                }
            }
            if let Some(weapon) = &champion.weapon {
                if !champion.kill {
                    push(
                        LintErrorKind::OutOfRange,
                        "test.champions : `weapon` sans `kill: true`".to_string(),
                    );
                }
                if !registry
                    .weapons
                    .contains_key(&registry::WeaponId::from(weapon.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("test.champions : arme « {weapon} » inconnue"),
                    );
                }
            }
        }
        if let Some(fight) = &test.boss {
            if character.boss.is_none() {
                push(
                    LintErrorKind::OutOfRange,
                    "test.boss (gabarit BossFight) sur un personnage sans boss:".to_string(),
                );
            }
            if fight.floors.is_some() && fight.map.is_some() {
                push(
                    LintErrorKind::OutOfRange,
                    "test.boss : floors et map sont exclusifs".to_string(),
                );
            }
            if let Err(message) =
                scene_room_kinds(registry, fight.floors.as_deref(), fight.map.as_deref())
            {
                push(
                    LintErrorKind::BrokenReference,
                    format!("test.boss : {message}"),
                );
            }
        }
    }
}

/// Types de salle (`room_kind` des niveaux) des cartes d'une scène de scénario généré (M2-T17) :
/// une séquence `Floors` (ses cartes, ou les gabarits de ses grammaires `floor:<id>`), une carte
/// seule, ou ni l'une ni l'autre (la carte de départ du manifeste n'est pas vérifiée ici : la
/// scène vide rend un ensemble vide). `Err` : référence inconnue.
fn scene_room_kinds(
    registry: &Registry,
    floors: Option<&str>,
    map: Option<&str>,
) -> Result<BTreeSet<String>, String> {
    let kinds_of = |path: &str| -> Result<BTreeSet<String>, String> {
        let id = registry::map_id_from_path(path);
        let entry = registry.maps.get(&id).ok_or_else(|| {
            format!("carte « {path} » inconnue (aucune carte chargée avec l'id « {id} »)")
        })?;
        Ok(entry
            .room_kinds
            .iter()
            .map(|(_, kind)| kind.clone())
            .collect())
    };
    let mut kinds = BTreeSet::new();
    if let Some(map) = map {
        kinds.extend(kinds_of(map)?);
    }
    if let Some(floors) = floors {
        let entry = registry
            .floors
            .get(&registry::FloorsConfigId::from(floors.to_string()))
            .ok_or_else(|| format!("séquence Floors « {floors} » inconnue"))?;
        for level in &entry.levels {
            if let Some(grammar) = level.strip_prefix("floor:") {
                let grammar = registry
                    .floor_grammars
                    .get(&registry::FloorGrammarId::from(grammar.to_string()))
                    .ok_or_else(|| {
                        format!("séquence « {floors} » : grammaire d'étage « {grammar} » inconnue")
                    })?;
                kinds.extend(kinds_of(&grammar.templates)?);
            } else if !level.contains(':') {
                // Une carte LDtk absente du registre n'a pas de type de salle : elle est signalée
                // par `lint_floors`.
                if let Ok(found) = kinds_of(level) {
                    kinds.extend(found);
                }
            }
        }
    }
    Ok(kinds)
}

/// `test:` d'un objet (M2-T17, §54) : `frames > 0`, pas sur un consommable (il n'entre pas dans
/// l'inventaire comme un passif ou un actif : `HasItem` ne le verrait pas).
fn lint_item_tests(registry: &Registry, errors: &mut Vec<LintError>) {
    for item in registry.items.values() {
        let Some(test) = &item.test else {
            continue;
        };
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: item.file.display().to_string(),
                message: format!("objet « {} » : {message}", item.id),
            });
        };
        if test.frames == 0 {
            push(
                LintErrorKind::OutOfRange,
                "champ test.frames = 0 : doit être > 0".to_string(),
            );
        }
        if item.def.kind == items::ItemKind::Consumable {
            push(
                LintErrorKind::Unsupported,
                "test (gabarit ItemHeld) sur un consommable : seuls un passif et un actif sont tenus"
                    .to_string(),
            );
        }
        // M3-T13 (§78) : gabarits TearsOnTarget et FamiliarHeld.
        let weapon_known = |weapon: &str| {
            registry
                .weapons
                .contains_key(&registry::WeaponId::from(weapon.to_string()))
        };
        if let Some(tears) = &test.tears {
            if tears.frames == Some(0) {
                push(
                    LintErrorKind::OutOfRange,
                    "test.tears : frames = 0 : doit être > 0".to_string(),
                );
            }
            if let Some(weapon) = tears.weapon.as_deref().filter(|w| !weapon_known(w)) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("test.tears : arme « {weapon} » inconnue"),
                );
            }
            for (index, (target, _, _)) in tears.targets.iter().enumerate() {
                match registry
                    .characters
                    .get(&registry::CharacterId::from(target.clone()))
                {
                    None => push(
                        LintErrorKind::BrokenReference,
                        format!("test.tears : personnage cible « {target} » (targets[{index}]) inconnu"),
                    ),
                    Some(character) if !character.counts_hits => push(
                        LintErrorKind::Unsupported,
                        format!("test.tears : la cible « {target} » ne compte pas ses coups (`counts_hits: true`)"),
                    ),
                    Some(_) => {}
                }
            }
            // Le témoin (même tir sans l'objet) est la preuve : chaque seuil doit dépasser ce que
            // le témoin tolère, et chaque attente propre à l'objet doit avoir son contraire.
            if tears.hits.is_empty() && tears.expect == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    "test.tears : ni `hits` ni `expect` : rien ne prouve l'effet de l'objet"
                        .to_string(),
                );
            }
            if (tears.expect == 0) != (tears.control == 0) {
                push(
                    LintErrorKind::OutOfRange,
                    "test.tears : `expect` et `control` vont ensemble (la preuve propre à l'objet et son contraire sans lui)"
                        .to_string(),
                );
            }
            for hits in &tears.hits {
                if hits.with_item == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        "test.tears.hits : with_item = 0 : doit être > 0".to_string(),
                    );
                }
                if hits.with_item <= hits.without_item {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "test.tears.hits : with_item ({}) doit dépasser without_item ({}) : le témoin sans objet passerait le seuil",
                            hits.with_item, hits.without_item
                        ),
                    );
                }
                if hits.target > tears.targets.len() {
                    push(
                        LintErrorKind::BrokenReference,
                        format!(
                            "test.tears.hits : cible {} absente (0 = la cible de l'arène, puis {} cible(s) posée(s))",
                            hits.target,
                            tears.targets.len()
                        ),
                    );
                }
            }
        }
        if let Some(familiar) = &test.familiar {
            if familiar.frames == Some(0) {
                push(
                    LintErrorKind::OutOfRange,
                    "test.familiar : frames = 0 : doit être > 0".to_string(),
                );
            }
            if item.def.familiar.is_none() {
                push(
                    LintErrorKind::Unsupported,
                    "test.familiar (gabarit FamiliarHeld) sur un objet sans `familiar:`"
                        .to_string(),
                );
            }
            if let Some(weapon) = familiar.weapon.as_deref().filter(|w| !weapon_known(w)) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("test.familiar : arme « {weapon} » inconnue"),
                );
            }
        }
    }
}

/// `test:` d'un type de salle (M2-T17, §54) : `frames > 0`, exactement un de `floors`/`map`, scène
/// connue, et au moins un gabarit LDtk de ce type dans cette scène.
fn lint_room_tests(registry: &Registry, errors: &mut Vec<LintError>) {
    for room in registry.rooms.values() {
        let Some(test) = &room.test else {
            continue;
        };
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: room.file.display().to_string(),
                message: format!("type de salle « {} » : {message}", room.id),
            });
        };
        if test.frames == 0 {
            push(
                LintErrorKind::OutOfRange,
                "champ test.frames = 0 : doit être > 0".to_string(),
            );
        }
        if test.floors.is_some() == test.map.is_some() {
            push(
                LintErrorKind::OutOfRange,
                "test : exactement un de floors et map".to_string(),
            );
            continue;
        }
        match scene_room_kinds(registry, test.floors.as_deref(), test.map.as_deref()) {
            Err(message) => push(LintErrorKind::BrokenReference, format!("test : {message}")),
            Ok(kinds) if !kinds.contains(&room.id) => push(
                LintErrorKind::BrokenReference,
                format!(
                    "test (gabarit RoomClear) : aucun gabarit LDtk de type « {} » dans la scène du test",
                    room.id
                ),
            ),
            Ok(_) => {}
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
        // M2-T6 (§50) : `shells` lâchés et prix des présentoirs.
        if let Some((min, max)) = economy.shells_drop {
            if min > max {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "économie « {} » : shells_drop = ({min}, {max}) : min > max",
                        economy.id
                    ),
                });
            }
        }
        for (rarity, price) in &economy.shop_prices {
            if *price == 0 {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "économie « {} » : shop_prices[{rarity:?}] = 0 : un prix doit être > 0",
                        economy.id
                    ),
                });
            }
        }
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
        // D40 : `RefillAmmoOf(munition)` vers une munition qu'aucune arme ne déclare.
        let mut unknown_ammo = Vec::new();
        for (index, action) in powerup.actions.iter().enumerate() {
            match action {
                effects::Action::RefillAmmoOf(ammo) => {
                    if !registry
                        .weapons
                        .values()
                        .any(|weapon| weapon.ammo_type.as_ref() == Some(ammo)) {
                        unknown_ammo.push(format!(
                            "power-up « {} » : actions[{index}] (RefillAmmoOf) : munition {ammo:?} déclarée par aucune arme",
                            powerup.id
                        ));
                    }
                }
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
                // T1.10 : actions propres aux effets du porteur
                effects::Action::Modifier { .. }
                | effects::Action::Heal(_)
                | effects::Action::GainShield(_)
                | effects::Action::GainSoul(_)
                | effects::Action::Explode { .. }
                | effects::Action::SpawnPattern { .. }
                | effects::Action::GaugeAdd(..) => push(format!(
                    "power-up « {} » : actions[{index}] : action d'effet (T1.10), pas de power-up",
                    powerup.id
                )),
                // T1.3 : un statut se pose sur un personnage touché (projectile)
                effects::Action::ApplyStatus { .. } | effects::Action::Chance { .. } => push(format!(
                    "power-up « {} » : actions[{index}] ({}) : réservée aux projectiles (on_hit)",
                    powerup.id,
                    if matches!(action, effects::Action::Chance { .. }) {
                        "Chance"
                    } else {
                        "ApplyStatus"
                    }
                )),
                // M2-T7 : actions de frise de boss
                effects::Action::SpawnEnemies { .. } | effects::Action::Telegraph(_) => {
                    push(format!(
                        "power-up « {} » : actions[{index}] : réservée à la frise d'un boss (§45)",
                        powerup.id
                    ))
                }
                // M3-T0d : mort d'un champion (§63)
                effects::Action::SpawnCharacters { .. } => push(format!(
                    "power-up « {} » : actions[{index}] : réservée à la mort d'un champion (§63)",
                    powerup.id
                )),
                // M3-T0b : `scale` est réservé aux effets d'objet
                effects::Action::Scaled { .. } => push(format!(
                    "power-up « {} » : actions[{index}] : Scaled réservée aux effets d'objet (§61)",
                    powerup.id
                )),
                _ => {}
            }
        }
        for message in unknown_ammo {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: file.clone(),
                message,
            });
        }
    }
}

/// Réglages de présentation : aucun état de simulation n'est modifié (T3.4). T1.17 :
/// valeurs de `FeedbackSettings::problems` (flash, secousse, télégraphe, surcharges
/// `by_kind`/`by_weapon`) et armes de `by_weapon` connues du jeu (à distance ou de corps à
/// corps).
/// Assets (lot A) : chaque clé de `volumes` existe dans `sounds` (sa plage est vérifiée par
/// `problems`).
fn lint_feedback(registry: &Registry, errors: &mut Vec<LintError>) {
    for feedback in &registry.feedback {
        let file = feedback.file.display().to_string();
        let settings = &feedback.settings;
        for (key, path) in &settings.sounds {
            if !registry.asset_exists(path) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "feedback : champ sounds.{key} = « {path} » : fichier absent de assets/"
                    ),
                });
            }
        }
        for problem in settings.problems() {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!("feedback : champ {} = {}", problem.field, problem.message),
            });
        }
        for weapon in settings.by_weapon.keys() {
            let known = registry
                .weapons
                .contains_key(&registry::WeaponId::from(weapon.as_str()))
                || registry
                    .melee_weapons
                    .contains_key(&registry::MeleeWeaponId::from(weapon.as_str()));
            if !known {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!("feedback : by_weapon « {weapon} » : arme inconnue du jeu"),
                });
            }
        }
        for key in settings.volumes.keys() {
            if !settings.sounds.contains_key(key) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!("feedback : volumes.{key} : clé absente de sounds"),
                });
            }
        }
    }
}

/// T1.18 (§32) : chaque widget de `ui/hud.ron` lit une source de la liste fermée
/// [`crate::ui::HUD_SOURCES`].
fn lint_hud_sources(registry: &Registry, errors: &mut Vec<LintError>) {
    for (rel, sources) in &registry.hud_sources {
        for source in sources {
            if !crate::ui::HUD_SOURCES.contains(&source.as_str()) {
                errors.push(LintError {
                    kind: LintErrorKind::UnknownKind,
                    file: rel.display().to_string(),
                    message: format!(
                        "hud : source inconnue « {source} » (connues : {})",
                        crate::ui::HUD_SOURCES.join(", ")
                    ),
                });
            }
        }
    }
    // Légende : chaque sprite cité existe dans la table des feuilles (si le jeu en déclare une).
    if registry.sprite_sheets.is_empty() {
        return;
    }
    for (rel, sprites) in &registry.hud_legend_sprites {
        for sprite in sprites {
            if !registry
                .sprite_sheets
                .contains_key(&registry::SpriteSheetId::from(sprite.as_str()))
            {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: rel.display().to_string(),
                    message: format!("hud : légende : sprite « {sprite} » absent de sprites.ron"),
                });
            }
        }
    }
}

/// Les écrans pause/inventaire instancient leurs listes depuis ce gabarit chargé par
/// le rendu, pas depuis une référence HTML. Les autres gabarits HTML sont vérifiés
/// à leur découverte par `Registry::build`, sans dépendance sur `bevy_hui`.
fn lint_html_screens(registry: &Registry, errors: &mut Vec<LintError>) {
    let Some(screen) = registry.ui_files.iter().find(|path| {
        path.ends_with("ui/pause_screen.html") || path.ends_with("ui/inventory_screen.html")
    }) else {
        return;
    };
    let files = match registry.embedded_files {
        Some(files) => crate::files::ContentFiles::embedded(files),
        None => crate::files::ContentFiles::native(GameManifest::assets_dir(&registry.game_dir)),
    };
    let row = std::path::Path::new("ui/hui_row.html");
    if let Err(error) = files.read(row) {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: screen.display().to_string(),
            message: format!(
                "écran HTML : gabarit de liste référencé « {} » absent ou illisible : {error}",
                row.display()
            ),
        });
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
        | PatternEntry::Scatter { count, spread, .. }
        | PatternEntry::Fan { count, spread, .. } => {
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
        PatternEntry::Spiral {
            count,
            shots,
            every,
            speed,
            ..
        } => {
            if *count == 0 || *shots == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : count et shots doivent être > 0"),
                );
            }
            if *every == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : every = 0 : doit être > 0"),
                );
            }
            if speed.get() < Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : speed < 0"),
                );
            }
        }
        PatternEntry::Burst {
            count,
            per_burst,
            interval,
            spread,
            ..
        } => {
            if *count == 0 || *per_burst == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : count et per_burst doivent être > 0"),
                );
            }
            if *interval == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : interval = 0 : doit être > 0"),
                );
            }
            if spread.get() < Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{pattern:?} : spread < 0"),
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

/// T1.9 : horloges (kind `Clock`, `docs/conventions.md` §23) : ids d'événements uniques sur
/// toutes les horloges (ils partagent `Clock::fired`), échéances croissantes dans un fichier,
/// `repeat > 0` ; `entry.clocks` ne cite que des horloges connues.
fn lint_clocks(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for clock in registry.clocks.values() {
        let file = clock.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message: format!("horloge « {} » : {message}", clock.id),
            });
        };
        let mut previous = 0u32;
        for event in &clock.events {
            if let Some(other) = seen.insert(event.id.as_str(), clock.id.as_str()) {
                push(
                    LintErrorKind::DuplicateId,
                    format!(
                        "événement « {} » en double (déjà dans « {other} »)",
                        event.id
                    ),
                );
            }
            let at = event.at.frames();
            if at < previous {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "événement « {} » : échéance {at} frames avant la précédente ({previous}) : les échéances doivent être croissantes",
                        event.id
                    ),
                );
            }
            previous = previous.max(at);
            if event.repeat.is_some_and(|repeat| repeat.frames() == 0) {
                push(
                    LintErrorKind::OutOfRange,
                    format!("événement « {} » : repeat = 0 : doit être > 0", event.id),
                );
            }
        }
    }
    for id in manifest.entry.clocks.iter().flatten() {
        if !registry
            .clocks
            .contains_key(&registry::ClockId::from(id.clone()))
        {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!("entry.clocks : horloge inconnue « {id} » (kind Clock)"),
            });
        }
    }
}

/// T1.9 : difficulté (kind `Difficulty`) : l'expression ne cite que les identifiants de
/// `content::expr::DIFFICULTY_IDENTIFIERS` et vaut > 0 pour `floor ∈ {0, 3}`,
/// `minutes ∈ {0, 10}`, `players ∈ {1, 4}` ; `entry.difficulty` exige le kind.
fn lint_difficulty(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    if manifest.entry.difficulty == Some(true) && registry.difficulty.is_none() {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: "entry.difficulty : aucun fichier de kind Difficulty".to_string(),
        });
    }
    let Some(entry) = &registry.difficulty else {
        return;
    };
    let file = entry.file.display().to_string();
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: file.clone(),
            message: format!("difficulté : {message}"),
        });
    };
    let expr = match crate::expr::Expr::parse(&entry.value) {
        Ok(expr) => expr,
        Err(e) => {
            push(
                LintErrorKind::Parse,
                format!("expression « {} » invalide : {e:?}", entry.value),
            );
            return;
        }
    };
    for floor in [0, 3] {
        for minutes in [0, 10] {
            for players in [1, 4] {
                let ctx = crate::expr::difficulty_context(players, floor, minutes * 3600, 0);
                match expr.try_eval_num(&ctx) {
                    Err(e) => {
                        push(
                            LintErrorKind::OutOfRange,
                            format!(
                                "« {} » : {e:?} (identifiants admis : {})",
                                entry.value,
                                crate::expr::DIFFICULTY_IDENTIFIERS.join(", ")
                            ),
                        );
                        return;
                    }
                    Ok(value) if value <= Fixed::ZERO => {
                        push(
                            LintErrorKind::OutOfRange,
                            format!(
                                "« {} » vaut {value} pour floor = {floor}, minutes = {minutes}, players = {players} : doit être > 0",
                                entry.value
                            ),
                        );
                        return;
                    }
                    Ok(_) => {}
                }
            }
        }
    }
}

/// M3-T7 : le pattern (après résolution des `Named`) joué par l'émetteur d'une mort est-il infini
/// (`Ring` avec `every > 0` en tête) ? `None` : chaîne `Named` inconnue, cyclique ou plus profonde que
/// [`MAX_NAMED_DEPTH`].
fn death_pattern_infinite(registry: &Registry, pattern: &registry::PatternEntry) -> Option<bool> {
    let mut current = pattern;
    for _ in 0..=MAX_NAMED_DEPTH {
        match current {
            registry::PatternEntry::Named(name) => {
                current = &registry
                    .patterns
                    .get(&registry::PatternId::from(name.clone()))?
                    .pattern;
            }
            registry::PatternEntry::Ring { every, .. } => return Some(*every > 0),
            _ => return Some(false),
        }
    }
    None
}

/// T1.5 : `variants` d'un personnage (`docs/conventions.md` §25) : `chance` dans `[0, 1]`,
/// `weight > 0`, nom en double, `skin` clé de `skins`, `MoveSpeed` refusé sur un personnage IA
/// (sa vitesse est `EnemyMoveSpeed`). Une stat inconnue échoue déjà au chargement RON.
fn lint_variants(
    registry: &Registry,
    character: &registry::CharacterEntry,
    errors: &mut Vec<LintError>,
) {
    let Some(variants) = &character.variants else {
        return;
    };
    let file = character.file.display().to_string();
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: file.clone(),
            message: format!("personnage « {} » : variants : {message}", character.id),
        });
    };
    // M3-T0d : un nombre doit être dans [0, 1] ; une expression n'utilise que `players` et
    // `floor` et reste dans [0, 1] pour 1 et 4 joueurs aux étages 0 et 5.
    match &variants.chance {
        NumOrExpr::Expression(expr) => {
            for id in expr.expr().identifiers() {
                if id != "players" && id != "floor" {
                    push(
                        LintErrorKind::BrokenReference,
                        format!(
                            "chance : identifiant « {id} » inconnu — seuls « players » et « floor » existent"
                        ),
                    );
                }
            }
            for (players, floor) in [(1u32, 0u32), (4, 0), (1, 5), (4, 5)] {
                let ctx = std::collections::BTreeMap::from([
                    ("players".to_string(), Fixed::from_num(players)),
                    ("floor".to_string(), Fixed::from_num(floor)),
                ]);
                match variants.chance.resolve_with(&ctx) {
                    Ok(value) if value < Fixed::ZERO || value > Fixed::from_num(1) => push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "chance : expression « {expr} » = {value} ({players} joueur(s), étage {floor}) : doit être dans [0, 1]"
                        ),
                    ),
                    Ok(_) => {}
                    Err(e) => push(
                        LintErrorKind::OutOfRange,
                        format!("chance : expression « {expr} » : {e} ({players} joueur(s), étage {floor})"),
                    ),
                }
            }
        }
        literal => {
            let chance = literal.resolve(1).unwrap_or(Fixed::ZERO);
            if chance < Fixed::ZERO || chance > Fixed::from_num(1) {
                push(
                    LintErrorKind::OutOfRange,
                    format!("chance = {chance} : doit être dans [0, 1]"),
                );
            }
        }
    }
    let mut seen = BTreeSet::new();
    for (name, variant) in &variants.table {
        if !seen.insert(name.as_str()) {
            push(
                LintErrorKind::DuplicateId,
                format!("variante « {name} » en double"),
            );
        }
        if variant.weight == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!("variante « {name} » : weight = 0 : doit être > 0 (jamais tirée)"),
            );
        }
        if let Some(skin) = &variant.skin {
            if !character.skins.contains(skin) {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "variante « {name} » : skin « {skin} » absent de skins ({:?})",
                        character.skins
                    ),
                );
            }
        }
        // M3-T0d (§63) : champs de champion.
        for (index, action) in variant.on_death.iter().enumerate() {
            let at = format!("variante « {name} » : on_death[{index}]");
            match action {
                effects::Action::Explode { radius, damage, .. } => {
                    if *radius <= Fixed::ZERO {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("{at} : Explode : rayon = {radius} : doit être > 0"),
                        );
                    }
                    if *damage < Fixed::ZERO {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("{at} : Explode : dégâts = {damage} : doivent être >= 0"),
                        );
                    }
                }
                effects::Action::SpawnCharacters { id, count } => {
                    if !registry
                        .characters
                        .contains_key(&registry::CharacterId::from(id.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnCharacters : personnage « {id} » inconnu"),
                        );
                    }
                    if *count == 0 {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("{at} : SpawnCharacters « {id} » : count = 0 : doit être > 0"),
                        );
                    }
                }
                effects::Action::SpawnPattern { pattern, weapon } => {
                    match registry
                        .patterns
                        .get(&registry::PatternId::from(pattern.clone()))
                    {
                        None => push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnPattern : pattern « {pattern} » inconnu"),
                        ),
                        // M3-T7 (§72) : l'émetteur d'une mort n'a pas de personnage pour l'arrêter : un
                        // pattern infini (`Ring` avec `every > 0`) tirerait pour toujours. La chaîne
                        // `Named` est suivie comme à l'exécution (`resolve_pattern`, profondeur
                        // bornée) ; une chaîne cyclique ou trop profonde est refusée aussi.
                        Some(file) => match death_pattern_infinite(registry, &file.pattern) {
                            Some(true) => push(
                                LintErrorKind::Unsupported,
                                format!("{at} : SpawnPattern : pattern « {pattern} » infini (Ring avec every > 0, directement ou par `Named`) : refusé à la mort, utiliser every: 0"),
                            ),
                            None => push(
                                LintErrorKind::Unsupported,
                                format!("{at} : SpawnPattern : pattern « {pattern} » : chaîne `Named` cyclique, inconnue ou trop profonde"),
                            ),
                            Some(false) => {}
                        },
                    }
                    if !registry
                        .weapons
                        .contains_key(&registry::WeaponId::from(weapon.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnPattern : arme « {weapon} » inconnue"),
                        );
                    }
                }
                other => push(
                    LintErrorKind::Unsupported,
                    format!(
                        "{at} : action {other:?} refusée à la mort (Explode, SpawnCharacters et SpawnPattern seulement)"
                    ),
                ),
            }
        }
        if let Some(rules) = &variant.behaviors {
            if rules.is_empty() {
                push(
                    LintErrorKind::OutOfRange,
                    format!("variante « {name} » : behaviors vide (omettre pour garder ceux du personnage)"),
                );
            }
            lint_rules(registry, rules, &mut |kind, message| {
                push(kind, format!("variante « {name} » : {message}"))
            });
        }
        if let Some(table) = &variant.loot {
            if !registry.loot_tables.contains_key(table) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("variante « {name} » : loot : pool « {table} » inconnu"),
                );
            }
        }
        if let Some(tint) = &variant.tint {
            let valid = tint.len() == 7
                && tint.starts_with('#')
                && tint[1..].chars().all(|c| c.is_ascii_hexdigit());
            if !valid {
                push(
                    LintErrorKind::OutOfRange,
                    format!("variante « {name} » : tint « {tint} » : attendu #rrggbb"),
                );
            }
        }
        if character.has_ai
            && variant
                .modifiers
                .iter()
                .any(|modifier| modifier.stat == StatId::MoveSpeed)
        {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "variante « {name} » : MoveSpeed sur un personnage IA : sa vitesse est EnemyMoveSpeed"
                ),
            );
        }
    }
}

/// T1.5 : un `CharacterSpawn` LDtk qui impose une variante (`variant`) désigne un personnage
/// connu qui déclare cette variante.
fn lint_forced_variants(registry: &Registry, errors: &mut Vec<LintError>) {
    for map in registry.maps.values() {
        for (character_id, variant) in &map.forced_variants {
            let character = registry
                .characters
                .get(&registry::CharacterId::from(character_id.clone()));
            let known = character
                .and_then(|c| c.variants.as_ref())
                .is_some_and(|v| v.table.iter().any(|(name, _)| name == variant));
            if !known {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: map.file.display().to_string(),
                    message: format!(
                        "CharacterSpawn « {character_id} » : variant « {variant} » inconnue (personnage inconnu ou sans cette variante)"
                    ),
                });
            }
        }
    }
}

/// Profils de `Chase` connus (même liste que `game::character::enemy::ai::state::CHASE_PROFILES`).
const CHASE_PROFILES: &[&str] = &["Ground", "Flying", "Phasing", "GroundBreaker"];

/// T1.4 : `ai.behaviors` et `ai.targeting` d'un personnage (`docs/conventions.md` §22) :
/// liste non vide ; `Shoot` (arme et pattern connus, projectiles du pattern dans la table de
/// l'arme, `cooldown_frames > 0`, `range > 0` — les règles `ranged` de T1.2) ; `Melee(arme)`
/// connue ; `Chase` profil connu ; `KeepDistance` `min < max` ; `Charge` télégraphe > 0 ; tag
/// d'`ignore` porté par au moins un personnage du jeu.
fn lint_behaviors(
    registry: &Registry,
    character: &registry::CharacterEntry,
    errors: &mut Vec<LintError>,
) {
    use registry::BehaviorEntry;
    let file = character.file.display().to_string();
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: file.clone(),
            message: format!("personnage « {} » : ai.{message}", character.id),
        });
    };
    if !character.ignore_tags.is_empty() {
        let known: BTreeSet<&str> = registry
            .characters
            .values()
            .flat_map(|c| c.tags.iter().map(String::as_str))
            .collect();
        for tag in &character.ignore_tags {
            if !known.contains(tag.as_str()) {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "targeting : tag ignoré inconnu « {tag} » (porté par aucun personnage)"
                    ),
                );
            }
        }
    }
    let Some(rules) = &character.behaviors else {
        return;
    };
    if rules.is_empty() {
        push(
            LintErrorKind::OutOfRange,
            "behaviors : liste vide (absente = liste par défaut)".to_string(),
        );
    }
    lint_rules(registry, rules, &mut push);
}

/// M2-T0c (`docs/conventions.md` §37) : un boss a au moins une phase ; seule la dernière peut
/// ne pas avoir de fin (`until`), toutes les autres en ont une ; `HealthBelow` dans `]0, 1]`,
/// `AfterFrames` > 0 ; une phase non vide (règles, `on_enter` ou frise) ; frise : `repeat` > 0 et
/// `at` < `repeat`, événements non vides ; règles et actions comme dans un personnage (références,
/// plages ; un `SpawnPattern` désigne un pattern et une arme connus).
fn lint_boss(
    registry: &Registry,
    character: &registry::CharacterEntry,
    errors: &mut Vec<LintError>,
) {
    let Some(boss) = &character.boss else {
        return;
    };
    let file = character.file.display().to_string();
    let last = boss.phases.len().saturating_sub(1);
    let mut push = |kind: LintErrorKind, message: String| {
        errors.push(LintError {
            kind,
            file: file.clone(),
            message: format!("personnage « {} » : boss.{message}", character.id),
        });
    };
    if boss.phases.is_empty() {
        push(
            LintErrorKind::OutOfRange,
            "phases : liste vide (au moins une phase)".to_string(),
        );
    }
    for (index, phase) in boss.phases.iter().enumerate() {
        let at = format!("phases[{index}]");
        match (&phase.until, index == last) {
            (None, false) => push(
                LintErrorKind::OutOfRange,
                format!("{at} : until absent sur une phase qui n'est pas la dernière"),
            ),
            (Some(registry::PhaseEndEntry::HealthBelow(ratio)), _) => {
                let ratio = ratio.get();
                if ratio <= Fixed::ZERO || ratio > Fixed::ONE {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : HealthBelow = {ratio} : doit être dans ]0, 1]"),
                    );
                }
            }
            (Some(registry::PhaseEndEntry::AfterFrames(0)), _) => push(
                LintErrorKind::OutOfRange,
                format!("{at} : AfterFrames = 0 : doit être > 0"),
            ),
            _ => {}
        }
        if phase.behaviors.is_empty() && phase.on_enter.is_empty() && phase.timeline.is_none() {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : phase vide (ni behaviors, ni on_enter, ni timeline)"),
            );
        }
        lint_rules(registry, &phase.behaviors, &mut |kind, message| {
            push(kind, format!("{at}.{message}"))
        });
        let mut actions: Vec<&effects::Action> = phase.on_enter.iter().collect();
        // M2-T7 : un `Telegraph` précède un `SpawnPattern` de la même liste.
        let mut lists: Vec<&[effects::Action]> = vec![&phase.on_enter];
        if let Some(timeline) = &phase.timeline {
            lists.extend(timeline.events.iter().map(|event| event.r#do.as_slice()));
        }
        for list in lists {
            for (k, action) in list.iter().enumerate() {
                if matches!(action, effects::Action::Telegraph(_))
                    && !list[k + 1..]
                        .iter()
                        .any(|next| matches!(next, effects::Action::SpawnPattern { .. }))
                {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : Telegraph sans SpawnPattern après lui dans la même liste"),
                    );
                }
            }
        }
        if let Some(timeline) = &phase.timeline {
            if timeline.events.is_empty() {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : timeline sans événement"),
                );
            }
            if timeline.repeat == Some(0) {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : timeline.repeat = 0 : doit être > 0"),
                );
            }
            for event in &timeline.events {
                if timeline
                    .repeat
                    .is_some_and(|period| period > 0 && event.at >= period)
                {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "{at} : événement at = {} >= repeat : il ne se déclencherait jamais",
                            event.at
                        ),
                    );
                }
                if event.r#do.is_empty() {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : événement at = {} sans action", event.at),
                    );
                }
                actions.extend(event.r#do.iter());
            }
        }
        for action in actions {
            match action {
                effects::Action::SpawnPattern { pattern, weapon } => {
                    if !registry
                        .patterns
                        .contains_key(&registry::PatternId::from(pattern.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnPattern : pattern « {pattern} » inconnu"),
                        );
                    }
                    if !registry
                        .weapons
                        .contains_key(&registry::WeaponId::from(weapon.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnPattern : arme « {weapon} » inconnue"),
                        );
                    }
                }
                effects::Action::TimedModifier { frames: 0, .. } => push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : TimedModifier frames = 0 : doit être > 0"),
                ),
                effects::Action::Heal(amount) if *amount <= Fixed::ZERO => push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : Heal = {amount} : doit être > 0"),
                ),
                effects::Action::GainShield(0) => push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : GainShield(0) : doit être > 0"),
                ),
                effects::Action::GainSoul(amount) if *amount <= Fixed::ZERO => push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : GainSoul = {amount} : doit être > 0"),
                ),
                // M2-T7 : renforts et télégraphe (§45)
                effects::Action::SpawnEnemies { id, count } => {
                    if !registry
                        .characters
                        .contains_key(&registry::CharacterId::from(id.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("{at} : SpawnEnemies : personnage « {id} » inconnu"),
                        );
                    }
                    if *count == 0 {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("{at} : SpawnEnemies « {id} » : count = 0 : doit être > 0"),
                        );
                    }
                }
                effects::Action::Telegraph(0) => push(
                    LintErrorKind::OutOfRange,
                    format!("{at} : Telegraph(0) : doit être > 0"),
                ),
                effects::Action::RefillAmmo
                | effects::Action::RefillAmmoOf(_)
                | effects::Action::RepairAllWindows
                | effects::Action::KillAllWaveEnemies
                | effects::Action::DestroyTerrain { .. }
                | effects::Action::ApplyStatus { .. }
                | effects::Action::Chance { .. }
                | effects::Action::GaugeAdd(..)
                | effects::Action::CurrencyMultiplier { .. }
                // M3-T0d : mort d'un champion (§63), pas une frise de boss
                | effects::Action::SpawnCharacters { .. } => push(
                    LintErrorKind::Unsupported,
                    format!("{at} : action {action:?} : pas exécutée par un boss (effets v2)"),
                ),
                // M3-T0b : `scale` est réservé aux effets d'objet
                effects::Action::Scaled { .. } => push(
                    LintErrorKind::Unsupported,
                    format!("{at} : Scaled : réservée aux effets d'objet (§61)"),
                ),
                _ => {}
            }
        }
    }
}

/// Règles d'une liste de behaviors (personnage ou phase de boss) : références et plages.
fn lint_rules(
    registry: &Registry,
    rules: &[registry::BehaviorEntry],
    push: &mut impl FnMut(LintErrorKind, String),
) {
    use registry::BehaviorEntry;
    for rule in rules {
        match rule {
            BehaviorEntry::Shoot {
                weapon,
                pattern,
                range,
                cooldown_frames,
            } => {
                let weapon_entry = registry
                    .weapons
                    .get(&registry::WeaponId::from(weapon.clone()));
                if weapon_entry.is_none() {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("behaviors : Shoot : arme inconnue « {weapon} »"),
                    );
                }
                match registry
                    .patterns
                    .get(&registry::PatternId::from(pattern.clone()))
                {
                    None => push(
                        LintErrorKind::BrokenReference,
                        format!("behaviors : Shoot : pattern inconnu « {pattern} » (kind Pattern)"),
                    ),
                    Some(entry) => {
                        if let Some(weapon_entry) = weapon_entry {
                            let mut projectiles = Vec::new();
                            pattern_projectiles(
                                &entry.pattern,
                                &registry.patterns,
                                0,
                                &mut projectiles,
                            );
                            for projectile in projectiles {
                                if !weapon_entry.projectiles.contains_key(projectile) {
                                    push(
                                        LintErrorKind::BrokenReference,
                                        format!(
                                            "behaviors : Shoot : pattern « {pattern} » : projectile « {projectile} » absent de la table projectiles de l'arme « {weapon} »"
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
                if *cooldown_frames == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        "behaviors : Shoot : cooldown_frames = 0 : doit être > 0".to_string(),
                    );
                }
                if range.get() <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "behaviors : Shoot : range = {} : doit être > 0",
                            range.get()
                        ),
                    );
                }
            }
            BehaviorEntry::Melee(weapon) => {
                if !registry
                    .melee_weapons
                    .contains_key(&registry::MeleeWeaponId::from(weapon.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("behaviors : Melee : arme de corps à corps inconnue « {weapon} »"),
                    );
                }
            }
            BehaviorEntry::Chase { profile } => {
                if !CHASE_PROFILES.contains(&profile.as_str()) {
                    push(
                        LintErrorKind::BrokenReference,
                        format!(
                            "behaviors : Chase : profil inconnu « {profile} » (connus : {})",
                            CHASE_PROFILES.join(", ")
                        ),
                    );
                }
            }
            BehaviorEntry::KeepDistance { min, max } => {
                if min.get() >= max.get() {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "behaviors : KeepDistance : min = {} >= max = {} : la bande doit être non vide",
                            min.get(),
                            max.get()
                        ),
                    );
                }
            }
            BehaviorEntry::Charge { telegraph } => {
                if *telegraph == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        "behaviors : Charge : telegraph = 0 : doit être > 0".to_string(),
                    );
                }
            }
            BehaviorEntry::Strafe | BehaviorEntry::Flee | BehaviorEntry::Wander => {}
            // Rayon, ouverture, période : voir `lint_formation` (qui voit aussi le reste de la
            // liste et l'escouade).
            BehaviorEntry::Formation { .. } => {}
            // M3-T0d (§63) : comportements de familier ; la présence d'un objet porteur est
            // vérifiée par `lint_familiars`.
            BehaviorEntry::Follow { gap } => {
                if gap.get() <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("behaviors : Follow : gap = {} : doit être > 0", gap.get()),
                    );
                }
            }
            BehaviorEntry::Orbit {
                radius,
                frames_per_turn,
            } => {
                if radius.get() <= Fixed::ZERO {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "behaviors : Orbit : radius = {} : doit être > 0",
                            radius.get()
                        ),
                    );
                }
                if *frames_per_turn == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        "behaviors : Orbit : frames_per_turn = 0 : doit être > 0".to_string(),
                    );
                }
            }
            BehaviorEntry::MirrorShot { every } => {
                if *every == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        "behaviors : MirrorShot : every = 0 : doit être > 0".to_string(),
                    );
                }
            }
            BehaviorEntry::BlockBullets => {}
        }
    }
}

/// M3-T0d (`docs/conventions.md` §63) : familiers. Un objet qui porte `familiar` cite un
/// personnage connu, qui a au moins un behavior de familier, et `per_copy > 0` ; réciproquement,
/// un behavior de familier n'est valide que sur un personnage cité par un objet.
fn lint_familiars(registry: &Registry, errors: &mut Vec<LintError>) {
    use std::collections::BTreeSet;
    let is_familiar = |rule: &registry::BehaviorEntry| {
        matches!(
            rule,
            registry::BehaviorEntry::Follow { .. }
                | registry::BehaviorEntry::Orbit { .. }
                | registry::BehaviorEntry::MirrorShot { .. }
                | registry::BehaviorEntry::BlockBullets
        )
    };
    let mut cited: BTreeSet<&str> = BTreeSet::new();
    for item in registry.items.values() {
        let Some(familiar) = &item.def.familiar else {
            continue;
        };
        let file = item.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message: format!("objet « {} » : familiar : {message}", item.id),
            })
        };
        cited.insert(familiar.character.as_str());
        if familiar.per_copy == 0 {
            push(
                LintErrorKind::OutOfRange,
                "per_copy = 0 : doit être > 0".to_string(),
            );
        }
        match registry
            .characters
            .get(&registry::CharacterId::from(familiar.character.clone()))
        {
            None => push(
                LintErrorKind::BrokenReference,
                format!("personnage « {} » inconnu", familiar.character),
            ),
            Some(character) => {
                if !character
                    .behaviors
                    .as_deref()
                    .is_some_and(|rules| rules.iter().any(is_familiar))
                {
                    push(
                        LintErrorKind::Unsupported,
                        format!(
                            "personnage « {} » sans behavior de familier (Follow, Orbit, MirrorShot, BlockBullets)",
                            familiar.character
                        ),
                    );
                }
            }
        }
    }
    for character in registry.characters.values() {
        let has_familiar_rule = character
            .behaviors
            .as_deref()
            .is_some_and(|rules| rules.iter().any(is_familiar));
        if has_familiar_rule && !cited.contains(character.id.as_str()) {
            errors.push(LintError {
                kind: LintErrorKind::Unsupported,
                file: character.file.display().to_string(),
                message: format!(
                    "personnage « {} » : behavior de familier sans porteur (aucun objet ne le cite par `familiar`)",
                    character.id
                ),
            });
        }
    }
}

/// M2-T8 (§49) : une formation a un rayon > 0, une ouverture dans ]0, 2π], une période > 0
/// (hors `volley`), une escouade (`ai.squad`) et un moyen d'agir (`Shoot` ou `Chase` dans la
/// liste) ; sinon elle ne fait rien.
fn lint_formation(character: &registry::CharacterEntry, errors: &mut Vec<LintError>) {
    use registry::{BehaviorEntry, FormationShapeEntry};
    let Some(rules) = &character.behaviors else {
        return;
    };
    let file = character.file.display().to_string();
    let mut push = |message: String| {
        errors.push(LintError {
            kind: LintErrorKind::OutOfRange,
            file: file.clone(),
            message: format!("personnage « {} » : ai.behaviors : {message}", character.id),
        })
    };
    let acts = rules.iter().any(|rule| {
        matches!(
            rule,
            BehaviorEntry::Shoot { .. } | BehaviorEntry::Chase { .. }
        )
    });
    for rule in rules {
        let BehaviorEntry::Formation {
            shape: FormationShapeEntry::Arc { radius, spread },
            period,
            volley,
        } = rule
        else {
            continue;
        };
        if radius.get() <= Fixed::ZERO {
            push(format!(
                "Formation : radius = {} : doit être > 0",
                radius.get()
            ));
        }
        let tau = bevy_fixed::fixed_math::FIXED_TAU;
        if spread.get() <= Fixed::ZERO || spread.get() > tau {
            push(format!(
                "Formation : spread = {} : doit être dans ]0, 2π]",
                spread.get()
            ));
        }
        if *period == 0 && !*volley {
            push("Formation : period = 0 : doit être > 0 (ou volley: true)".to_string());
        }
        if !acts {
            push(
                "Formation : ni Shoot ni Chase dans la liste : la formation ne ferait rien"
                    .to_string(),
            );
        }
        if character.squad.is_none() {
            push("Formation : sans escouade (ai.squad)".to_string());
        }
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
        for id in floors.consumables_per_floor.keys() {
            let known = registry
                .items
                .get(id)
                .is_some_and(|item| item.def.kind == items::ItemKind::Consumable);
            if !known {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "séquence de niveaux « {} » : champ consumables_per_floor : « {id} » n'est pas un objet consommable (kind Item)",
                        floors.id
                    ),
                });
            }
        }
        for level in &floors.levels {
            if let Some(cave) = registry::cave_designation(level) {
                // D36 : une caverne sans ennemi ouvre son portail aussitôt (et en boucle si elle
                // est le dernier niveau) ; `transit: true` déclare ce passage voulu.
                if let Some(entry) = registry.caves.get(&cave) {
                    let c = &entry.config;
                    if !c.transit && (c.characters.is_empty() || c.enemy_spawns == 0) {
                        errors.push(LintError {
                            kind: LintErrorKind::OutOfRange,
                            file: file.clone(),
                            message: format!(
                                "séquence de niveaux « {} » : champ levels : « {level} » : caverne sans ennemi (characters vide ou enemy_spawns = 0) : son portail s'ouvre aussitôt ; transit: true dans {} si c'est voulu",
                                floors.id,
                                entry.file.display()
                            ),
                        });
                    }
                }
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
            if let Some(grammar) = registry::floor_designation(level) {
                if !registry.floor_grammars.contains_key(&grammar) {
                    errors.push(LintError {
                        kind: LintErrorKind::BrokenReference,
                        file: file.clone(),
                        message: format!(
                            "séquence de niveaux « {} » : champ levels : « {level} » : aucune grammaire d'étage chargée avec cet id (« {grammar} »)",
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

/// M2-T10 (`docs/conventions.md` §40) : une grammaire d'étage est cohérente (bornes, exigences)
/// et satisfiable avec ses gabarits : la carte `templates` est chargée et chaque type nécessaire
/// (départ, boss, remplissage, requis) a au moins un niveau de ce `room_kind`.
fn lint_floor_grammars(registry: &Registry, errors: &mut Vec<LintError>) {
    for entry in registry.floor_grammars.values() {
        let file = entry.file.display().to_string();
        for problem in entry.grammar.problems() {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!("grammaire d'étage « {} » : {problem}", entry.id),
            });
        }
        // M3-T12 (§77) : la salle secrète d'une grille n'est murée que par `door_lock: Secret` de son
        // type de salle ; sans lui, elle serait posée mais ouverte, jamais « secrète ».
        if let world::FloorLayout::Grid { secret, .. } = entry.grammar.layout() {
            if entry.grammar.required.contains_key(&secret) {
                if let Some(room) = registry.rooms.get(&secret) {
                    if room.def.door_lock != Some(world::DoorLock::Secret) {
                        errors.push(LintError {
                            kind: LintErrorKind::OutOfRange,
                            file: file.clone(),
                            message: format!(
                                "grammaire d'étage « {} » : layout Grid : la salle secrète « {secret} » (required) a un type de salle sans `door_lock: Secret` : elle ne serait pas murée",
                                entry.id
                            ),
                        });
                    }
                }
            }
        }
        let map_id = registry::map_id_from_path(&entry.templates);
        let Some(map) = registry.maps.get(&map_id) else {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file,
                message: format!(
                    "grammaire d'étage « {} » : champ templates : « {} » : aucune carte chargée avec cet id (« {map_id} »)",
                    entry.id, entry.templates
                ),
            });
            continue;
        };
        // M3-T0c (§62), M3-T5 (§70) : disposition en grille, chaque gabarit d'un type nécessaire
        // couvre un rectangle de cases de salle (dimensions multiples de `cell`, Q8) et a, sur
        // chaque côté de chaque case, une ouverture centrée sur le segment.
        if let world::FloorLayout::Grid { cell, .. } = entry.grammar.layout() {
            let needed = entry.grammar.kinds_needed();
            let mut reference: Option<(&str, usize)> = None;
            for level in &map.levels {
                let Some(kind) = level.room_kind.as_deref().filter(|k| needed.contains(k)) else {
                    continue;
                };
                if !registry::size_fits_cell(level.size_cells, cell) {
                    errors.push(LintError {
                        kind: LintErrorKind::OutOfRange,
                        file: file.clone(),
                        message: format!(
                            "grammaire d'étage « {} » : gabarit « {} » (type « {kind} ») fait {} × {} cases : doit être un multiple (au moins 1 × 1) de la case de salle {} × {} de la grille",
                            entry.id, level.id, level.size_cells.0, level.size_cells.1, cell.0, cell.1
                        ),
                    });
                    continue;
                }
                let faults = level.connection.segment_faults(level.size_cells, cell);
                if !faults.is_empty() {
                    let faults: Vec<String> = faults.iter().map(ToString::to_string).collect();
                    errors.push(LintError {
                        kind: LintErrorKind::OutOfRange,
                        file: file.clone(),
                        message: format!(
                            "grammaire d'étage « {} » : gabarit « {} » (type « {kind} », {} × {} cases, case de salle {} × {}) sans ouverture centrée sur chaque segment de chaque côté ({})",
                            entry.id,
                            level.id,
                            level.size_cells.0,
                            level.size_cells.1,
                            cell.0,
                            cell.1,
                            faults.join(" ; ")
                        ),
                    });
                    continue;
                }
                // largeur des ouvertures : le planificateur n'accepte une porte que si les deux
                // ouvertures ont la même largeur, donc elle est unique pour toute la grammaire
                let mut widths = level.connection.opening_widths(level.size_cells, cell);
                widths.dedup();
                for width in widths {
                    match &reference {
                        None => reference = Some((level.id.as_str(), width)),
                        Some((first, first_width)) if *first_width != width => {
                            errors.push(LintError {
                                kind: LintErrorKind::OutOfRange,
                                file: file.clone(),
                                message: format!(
                                    "grammaire d'étage « {} » : gabarit « {} » (type « {kind} ») a une ouverture de {width} case(s) mais gabarit « {first} » une de {first_width} case(s) : toutes les ouvertures des gabarits d'une grille ont la même largeur (une porte ne relie que deux ouvertures de même largeur)",
                                    entry.id, level.id
                                ),
                            });
                            break;
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        for kind in entry.grammar.kinds_needed() {
            if !map.room_kinds.iter().any(|(_, k)| k == kind) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "grammaire d'étage « {} » : type de salle « {kind} » sans gabarit dans {} (room_kind)",
                        entry.id, entry.templates
                    ),
                });
            }
        }
    }
}

/// Consommable qui ouvre une porte `door_lock: Key(n)` (`game::loot::KEY_ITEM`, que `content` ne
/// peut pas importer).
const KEY_ITEM: &str = "key";

/// M3-T5 (§70), trésor derrière une clé obtenable : un type de salle nécessaire à une grammaire
/// d'étage dont la porte est verrouillée par `Key(n)` exige une source d'au moins `n` clés à
/// **chaque** étage où le verrou s'applique. Pour chaque séquence `Floors` qui nomme la grammaire
/// à l'indice `i ≥ lock_from_floor` (en dessous, la porte est ordinaire), `i` est couvert si
/// `consumables_per_floor["key"] ≥ n` et `i ≥ 1` (remontés au changement d'étage : l'étage 0 ne
/// les reçoit jamais), ou si un personnage a `starting_consumables["key"] ≥ n`, ou si un pool de
/// butin peut donner la clé. Un message par séquence et par type, qui nomme les indices non
/// couverts. Une grammaire qu'aucune séquence ne nomme n'est pas vérifiée.
fn lint_key_sources(registry: &Registry, errors: &mut Vec<LintError>) {
    let pool_gives_key = crate::reach::loot_pool_items(registry).contains(KEY_ITEM);
    let most_starting_keys = registry
        .characters
        .values()
        .filter_map(|c| c.starting_consumables.get(KEY_ITEM))
        .max()
        .copied()
        .unwrap_or(0);
    for entry in registry.floor_grammars.values() {
        for kind in entry.grammar.kinds_needed() {
            let Some(room) = registry.rooms.get(kind) else {
                continue;
            };
            let Some(world::DoorLock::Key(needed)) = room.def.door_lock else {
                continue;
            };
            if needed == 0 {
                continue; // `Key(0)` : déjà refusé par `lint_room_kinds`
            }
            if pool_gives_key || most_starting_keys >= needed {
                continue;
            }
            for floors in registry.floors.values() {
                let per_floor = floors
                    .consumables_per_floor
                    .get(KEY_ITEM)
                    .copied()
                    .unwrap_or(0)
                    >= needed;
                let uncovered: Vec<String> = floors
                    .levels
                    .iter()
                    .enumerate()
                    .filter(|(_, level)| {
                        registry::floor_designation(level).as_ref() == Some(&entry.id)
                    })
                    .map(|(index, _)| index)
                    .filter(|index| *index as u32 >= room.def.lock_from_floor)
                    .filter(|index| !(per_floor && *index >= 1))
                    .map(|index| index.to_string())
                    .collect();
                if uncovered.is_empty() {
                    continue;
                }
                errors.push(LintError {
                    kind: LintErrorKind::Unsupported,
                    file: entry.file.display().to_string(),
                    message: format!(
                        "grammaire d'étage « {} » : type de salle « {kind} » : trésor fermé à clé sans source de clé : door_lock Key({needed}) (lock_from_floor = {}), séquence Floors « {} » : étage(s) d'indice {} sans clé (consumables_per_floor « {KEY_ITEM} » ≥ {needed} : {}, et il ne sert qu'à partir de l'indice 1 ; aucun pool de butin ne donne « {KEY_ITEM} » et aucun personnage n'a starting_consumables « {KEY_ITEM} » ≥ {needed})",
                        entry.id,
                        room.def.lock_from_floor,
                        floors.id,
                        uncovered.join(", "),
                        if per_floor { "présent" } else { "absent" }
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
        if !registry.asset_exists(&cave.template) {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: file.clone(),
                message: format!(
                    "caverne « {} » : gabarit LDtk « {} » introuvable",
                    cave.id, cave.template
                ),
            });
        }
        // D48 : une caverne peuplée doit offrir des points d'apparition atteints par le champ de
        // flux de son gabarit (`nav_large`, dérivé du plus grand corps de `characters`) ; vérifié
        // sur les graines de contrôle 1 à 5 (4 points de joueurs, `CAVE_PLAYER_SPAWNS` de
        // `map_ldtk`).
        if c.enemy_spawns > 0 && !c.characters.is_empty() && c.width >= 16 && c.height >= 16 {
            for seed in 1..=CAVE_CONTROL_SEEDS {
                let grid = world::generate(seed, c);
                let points = world::points_of_interest(
                    &grid,
                    4,
                    c.enemy_spawns,
                    c.spawn_clearance,
                    c.nav_large,
                );
                if points.zombie_spawns.is_empty() {
                    errors.push(LintError {
                        kind: LintErrorKind::OutOfRange,
                        file: file.clone(),
                        message: format!(
                            "caverne « {} » : graine de contrôle {seed} : aucun point d'apparition d'ennemi atteint par le champ de flux du gabarit {} (plus grand corps de characters) : caverne trop fermée",
                            cave.id,
                            if c.nav_large { "grand" } else { "petit" }
                        ),
                    });
                    break;
                }
            }
        }
    }
}

/// D48 : graines de contrôle du lint des cavernes (points d'apparition atteignables).
const CAVE_CONTROL_SEEDS: u64 = 5;

/// T1.7 : `intgrid_value` unique et > 0, facteurs > 0, tags non vides.
fn lint_surfaces(registry: &Registry, errors: &mut Vec<LintError>) {
    let mut values: BTreeMap<u8, &registry::SurfaceName> = BTreeMap::new();
    for surface in registry.surfaces.values() {
        let file = surface.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message: format!("surface « {} » : {message}", surface.id),
            })
        };
        if surface.intgrid_value == 0 {
            push(
                LintErrorKind::OutOfRange,
                "intgrid_value = 0 : doit être > 0 (0 = case sans surface)".into(),
            );
        } else if let Some(other) = values.insert(surface.intgrid_value, &surface.id) {
            push(
                LintErrorKind::DuplicateId,
                format!(
                    "intgrid_value = {} : déjà prise par la surface « {other} »",
                    surface.intgrid_value
                ),
            );
        }
        for (name, factor) in [
            ("move_speed", surface.move_speed),
            ("acceleration", surface.acceleration),
        ] {
            if factor <= Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("{name} = {factor} : doit être > 0"),
                );
            }
        }
        if surface.tags.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                "tags vide : au moins un tag".into(),
            );
        }
        // M3-T0c (§62) : piques.
        if let Some(hazard) = &surface.hazard {
            for message in hazard.problems() {
                push(LintErrorKind::OutOfRange, message);
            }
        }
        // M2-T11 (§46) : une fosse inflige des dégâts.
        if let Some(pit) = &surface.pit {
            if pit.fall_damage <= Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!("fall_damage = {} : doit être > 0", pit.fall_damage),
                );
            }
        }
    }
}

/// M2-T11 (`docs/conventions.md` §46) : objets de décor. Table : au moins une balle ; baril :
/// PV, rayon et dégâts > 0.
fn lint_props(registry: &Registry, errors: &mut Vec<LintError>) {
    for prop in registry.props.values() {
        let file = prop.file.display().to_string();
        let mut push = |message: String| {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!("objet de décor « {} » : {message}", prop.id),
            })
        };
        match &prop.def {
            world::PropDef::Table(table) => {
                if table.bullets == 0 {
                    push("bullets = 0 : une table encaisse au moins une balle".into());
                }
                if table.thickness <= Fixed::ZERO {
                    push(format!("thickness = {} : doit être > 0", table.thickness));
                }
                if table.range <= Fixed::ZERO {
                    push(format!("range = {} : doit être > 0", table.range));
                }
            }
            world::PropDef::Barrel(barrel) => {
                for (name, value) in [
                    ("health", barrel.health),
                    ("radius", barrel.radius),
                    ("damage", barrel.damage),
                ] {
                    if value <= Fixed::ZERO {
                        push(format!("{name} = {value} : doit être > 0"));
                    }
                }
            }
        }
    }
}

/// `generate_template` (T1.13, `docs/conventions.md` §28) : carte connue, cible connue qui
/// compte les coups (`counts_hits: true`).
fn lint_generate_template(
    registry: &Registry,
    manifest: &GameManifest,
    errors: &mut Vec<LintError>,
) {
    let Some(template) = &manifest.generate_template else {
        return;
    };
    let mut push = |message: String| {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message,
        });
    };
    let map_id = registry::map_id_from_path(&template.map);
    if !registry.maps.contains_key(&map_id) {
        push(format!(
            "champ generate_template.map = « {} » : aucune carte chargée avec cet id (« {map_id} »)",
            template.map
        ));
    }
    match registry
        .characters
        .values()
        .find(|character| character.id.as_str() == template.target)
    {
        None => push(format!(
            "champ generate_template.target = « {} » : personnage inconnu",
            template.target
        )),
        Some(character) if !character.counts_hits => push(format!(
            "champ generate_template.target = « {} » : le personnage doit déclarer counts_hits: true",
            template.target
        )),
        Some(_) => {}
    }
}

fn lint_entry_point(registry: &Registry, manifest: &GameManifest, errors: &mut Vec<LintError>) {
    let start_map_id = registry::map_id_from_path(&manifest.entry.start_map);
    let start_is_known = match (
        registry::cave_designation(&manifest.entry.start_map),
        registry::floor_designation(&manifest.entry.start_map),
    ) {
        (Some(cave), _) => registry.caves.contains_key(&cave),
        (None, Some(floor)) => registry.floor_grammars.contains_key(&floor),
        (None, None) => registry.maps.contains_key(&start_map_id),
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
    // M2-T13 : le hub désigne une carte chargée et offre au moins une porte d'entrée en run.
    if let Some(hub) = &manifest.entry.hub {
        let hub_map = registry::map_id_from_path(&hub.map);
        if !registry.maps.contains_key(&hub_map) {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!(
                    "champ entry.hub.map = « {} » : aucune carte chargée avec cet id (« {hub_map} »)",
                    hub.map
                ),
            });
        }
        if !registry
            .hub_services
            .values()
            .any(|service| service.def.action == meta::HubAction::EnterRun)
        {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message:
                    "champ entry.hub : aucun service d'action EnterRun (dossier « HubService »)"
                        .to_string(),
            });
        }
    }
    // D59 : `entry.floors` désigne la séquence par défaut du mode `Floors`.
    if let Some(floors) = &manifest.entry.floors {
        if manifest.entry.mode != Some(crate::manifest::EntryMode::Floors) {
            errors.push(LintError {
                kind: LintErrorKind::Unsupported,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!("champ entry.floors = « {floors} » : exige entry.mode = Floors"),
            });
        }
        if !registry
            .floors
            .contains_key(&registry::FloorsConfigId::from(floors.clone()))
        {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
                message: format!(
                    "champ entry.floors = « {floors} » : aucune séquence de ce nom dans le dossier « Floors »"
                ),
            });
        }
    }
}

/// M2-T13 (`docs/conventions.md` §57) : services du hub. Une condition `Unlock` désigne un
/// déblocage du jeu ou un jalon du profil ; une statistique à 0 ne conditionne rien.
fn lint_hub_services(registry: &Registry, errors: &mut Vec<LintError>) {
    for service in registry.hub_services.values() {
        let file = service.file.display().to_string();
        let id = &service.id;
        for need in &service.def.needs {
            match need {
                meta::HubCondition::Unlock(unlock) => {
                    if !registry.unlocks.contains_key(unlock)
                        && !meta::MILESTONES.contains(&unlock.as_str())
                    {
                        errors.push(LintError {
                            kind: LintErrorKind::BrokenReference,
                            file: file.clone(),
                            message: format!(
                                "service du hub « {id} » : condition Unlock(« {unlock} ») inconnue (ni déblocage du jeu, ni jalon)"
                            ),
                        });
                    }
                }
                meta::HubCondition::Stat(name, min) => {
                    if *min == 0 {
                        errors.push(LintError {
                            kind: LintErrorKind::OutOfRange,
                            file: file.clone(),
                            message: format!(
                                "service du hub « {id} » : condition Stat(« {name} », 0) (toujours vraie)"
                            ),
                        });
                    }
                }
            }
        }
    }
}

/// T1.10 (`docs/conventions.md` §27) : `progression.ron`.
fn lint_progression(registry: &Registry, errors: &mut Vec<LintError>) {
    for progression in registry.progression.values() {
        let file = progression.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message: format!("progression « {} » : {message}", progression.id),
            })
        };
        if progression.gauge.is_empty() {
            push(LintErrorKind::OutOfRange, "gauge vide".into());
        }
        if progression.per_kill <= Fixed::ZERO {
            push(
                LintErrorKind::OutOfRange,
                format!("per_kill = {} : doit être > 0", progression.per_kill),
            );
        }
        if progression.levels.is_empty() {
            push(LintErrorKind::OutOfRange, "levels vide".into());
        }
        let mut previous = Fixed::ZERO;
        for (index, level) in progression.levels.iter().enumerate() {
            if *level <= previous {
                push(
                    LintErrorKind::OutOfRange,
                    format!("levels[{index}] = {level} : seuils strictement croissants et > 0"),
                );
            }
            previous = *level;
        }
        if progression.choice_frames == 0 {
            push(
                LintErrorKind::OutOfRange,
                "choice_frames = 0 : doit être > 0".into(),
            );
        }
        for id in &progression.mutations {
            if !registry
                .mutations
                .contains_key(&registry::MutationId::from(id.clone()))
            {
                push(
                    LintErrorKind::BrokenReference,
                    format!("mutations : mutation « {id} » inconnue"),
                );
            }
        }
        let pool = if progression.mutations.is_empty() {
            registry.mutations.len()
        } else {
            progression.mutations.len()
        };
        if progression.choices == 0 || progression.choices as usize > pool {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "choices = {} : doit être entre 1 et la taille du pool ({pool})",
                    progression.choices
                ),
            );
        }
        if progression.weapon_drop_chance < Fixed::ZERO
            || progression.weapon_drop_chance > Fixed::ONE
        {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "weapon_drop_chance = {} : doit être dans [0, 1]",
                    progression.weapon_drop_chance
                ),
            );
        }
        for (index, entry) in progression.weapon_pool.iter().enumerate() {
            for weapon in &entry.weapons {
                if !registry
                    .weapons
                    .contains_key(&registry::WeaponId::from(weapon.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("weapon_pool[{index}] : arme « {weapon} » inconnue"),
                    );
                }
            }
        }
    }
}

/// T1.10 (`docs/conventions.md` §27) : `mutations/<id>.ron`.
fn lint_mutations(registry: &Registry, errors: &mut Vec<LintError>) {
    for mutation in registry.mutations.values() {
        let file = mutation.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let at = format!("mutation « {} »", mutation.id);
        if mutation.weight == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : weight = 0 : doit être > 0"),
            );
        }
        if mutation.max_stacks == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!("{at} : max_stacks = 0 : doit être > 0"),
            );
        }
        if mutation.effects.is_empty() {
            push(LintErrorKind::OutOfRange, format!("{at} : effects vide"));
        }
        for (index, effect) in mutation.effects.iter().enumerate() {
            lint_effect(
                registry,
                &format!("{at} : effects[{index}]"),
                effect,
                false,
                &mut push,
            );
        }
    }
}

/// T1.3 (`docs/conventions.md` §19) : `ApplyStatus` des `on_hit` d'une arme (modes et table
/// `projectiles`) : statut connu, `stacks > 0`.
fn lint_weapon_statuses(
    registry: &Registry,
    weapon: &registry::WeaponEntry,
    errors: &mut Vec<LintError>,
) {
    let specs = weapon
        .mode_projectiles
        .iter()
        .map(|(mode, spec)| (format!("mode « {mode} »"), spec.on_hit.clone()))
        .chain(
            weapon
                .projectiles
                .iter()
                .map(|(id, def)| (format!("projectiles « {id} »"), def.on_hit.clone())),
        );
    for (at, on_hit) in specs {
        for action in on_hit {
            let Some((status, stacks)) = on_hit_status(&action).map(|(s, n)| (s.to_string(), n))
            else {
                continue;
            };
            let file = weapon.file.display().to_string();
            if !registry
                .statuses
                .contains_key(&registry::StatusId::from(status.clone()))
            {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "arme « {} » : {at} : ApplyStatus : statut « {status} » inconnu",
                        weapon.id
                    ),
                });
            }
            if stacks == 0 {
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file,
                    message: format!(
                        "arme « {} » : {at} : ApplyStatus « {status} » : stacks = 0 : doit être > 0",
                        weapon.id
                    ),
                });
            }
        }
    }
}

/// T1.3 (`docs/conventions.md` §19) : `statuses/<id>.ron`.
fn lint_statuses(registry: &Registry, errors: &mut Vec<LintError>) {
    use registry::StatusKindEntry;
    for status in registry.statuses.values() {
        let file = status.file.display().to_string();
        let mut push = |message: String| {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: file.clone(),
                message: format!("statut « {} » : {message}", status.id),
            })
        };
        if status.frames == 0 {
            push("frames = 0 : doit être > 0".into());
        }
        match status.kind {
            StatusKindEntry::Burn | StatusKindEntry::Poison => {
                let name = if status.kind == StatusKindEntry::Burn {
                    "Burn"
                } else {
                    "Poison"
                };
                if status.damage <= Fixed::ZERO {
                    push(format!(
                        "damage = {} : doit être > 0 ({name})",
                        status.damage
                    ));
                }
                if status.period == 0 {
                    push(format!("period = 0 : doit être > 0 ({name})"));
                }
            }
            StatusKindEntry::Slow => {
                if status.factor <= Fixed::ZERO || status.factor > Fixed::ONE {
                    push(format!(
                        "factor = {} : doit être dans ]0, 1] (Slow)",
                        status.factor
                    ));
                }
            }
            StatusKindEntry::Stun
            | StatusKindEntry::Freeze
            | StatusKindEntry::Fear
            | StatusKindEntry::Charm => {}
        }
    }
}

/// M2-T0b (`docs/conventions.md` §36) : objets. Charge d'un actif > 0, `pickup_range` > 0,
/// modificateur sans valeur nulle inutile ignoré ; effets : `OnUse` d'un objet **actif** avec
/// `TimedModifier`/`Modifier` (T0b) ; tout autre déclencheur suit les règles d'un effet de
/// personnage (`lint_effect`, M2-T3) ; un actif sans effet ni modificateur ne fait rien.
fn lint_items(registry: &Registry, errors: &mut Vec<LintError>) {
    for item in registry.items.values() {
        let file = item.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let id = &item.id;
        if let Some(mods) = &item.def.projectile {
            lint_carried_projectile(
                registry,
                &format!("objet « {id} » : projectile"),
                mods,
                &mut push,
            );
        }
        if let items::ItemKind::Active(charge) = item.def.kind {
            if charge.required() == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("objet « {id} » : charge {charge:?} : doit être > 0"),
                );
            }
        }
        if item.def.pickup_range <= Fixed::ZERO {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "objet « {id} » : pickup_range = {} : doit être > 0",
                    item.def.pickup_range
                ),
            );
        }
        if let Some(blank) = item.def.blank {
            if item.def.kind != items::ItemKind::Consumable {
                push(
                    LintErrorKind::Unsupported,
                    format!("objet « {id} » : `blank` n'a de sens que pour un consommable"),
                );
            }
            if blank.radius <= Fixed::ZERO {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "objet « {id} » : blank.radius = {} : doit être > 0",
                        blank.radius
                    ),
                );
            }
        }
        let active = matches!(item.def.kind, items::ItemKind::Active(_));
        let consumable = item.def.kind == items::ItemKind::Consumable;
        if let Some(max_stack) = item.def.max_stack {
            if max_stack == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("objet « {id} » : max_stack = 0 : doit être > 0 (ou absent)"),
                );
            }
            if !consumable {
                push(
                    LintErrorKind::Unsupported,
                    format!("objet « {id} » : max_stack n'a de sens que pour un consommable"),
                );
            }
        }
        // M3-T3 (§68) : `refuse_when_full` exige un consommable qui soigne (sinon il ne refuse jamais rien).
        if item.def.refuse_when_full
            && !(consumable
                && item.def.effects.iter().any(|effect| {
                    effect
                        .r#do
                        .iter()
                        .any(|action| matches!(action, effects::Action::Heal(_)))
                }))
        {
            push(
                LintErrorKind::Unsupported,
                format!(
                    "objet « {id} » : refuse_when_full exige un consommable avec un effet Heal"
                ),
            );
        }
        if consumable && !item.def.modifiers.is_empty() {
            push(
                LintErrorKind::Unsupported,
                format!("objet « {id} » : un consommable n'a pas de modificateurs (compteur, ou effets OnUse instantanés)"),
            );
        }
        // M2-T4 : limites et coût d'un actif
        if item.def.uses_per_floor == Some(0) {
            push(
                LintErrorKind::OutOfRange,
                format!("objet « {id} » : uses_per_floor = 0 : doit être > 0 (ou absent)"),
            );
        }
        if item.def.uses_per_floor.is_some() && !active {
            push(
                LintErrorKind::Unsupported,
                format!("objet « {id} » : `uses_per_floor` n'a de sens que pour un actif"),
            );
        }
        if let Some((cost_item, count)) = &item.def.cost {
            if !active {
                push(
                    LintErrorKind::Unsupported,
                    format!("objet « {id} » : `cost` n'a de sens que pour un actif"),
                );
            }
            if *count == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "objet « {id} » : cost = ({cost_item:?}, 0) : la quantité doit être > 0"
                    ),
                );
            }
            match registry.items.get(cost_item) {
                None => push(
                    LintErrorKind::BrokenReference,
                    format!("objet « {id} » : cost : objet « {cost_item} » inconnu"),
                ),
                Some(target) if target.def.kind != items::ItemKind::Consumable => push(
                    LintErrorKind::Unsupported,
                    format!("objet « {id} » : cost : « {cost_item} » n'est pas un consommable"),
                ),
                Some(_) => {}
            }
        }
        for (index, effect) in item.def.effects.iter().enumerate() {
            let at = format!("objet « {id} » : effects[{index}]");
            if consumable && effect.on != effects::On::OnUse {
                push(
                    LintErrorKind::Unsupported,
                    format!(
                        "{at} : un consommable n'a que des effets OnUse (appliqués au ramassage)"
                    ),
                );
                continue;
            }
            if effect.on != effects::On::OnUse {
                // M2-T3 : effet porté par le porteur tant que l'objet est tenu (même règles
                // qu'un effet de personnage)
                lint_effect(registry, &at, effect, true, &mut push);
                continue;
            }
            if !active && !consumable {
                push(
                    LintErrorKind::Unsupported,
                    format!("{at} : OnUse n'a de sens que pour un objet actif ou un consommable"),
                );
                continue;
            }
            for action in &effect.r#do {
                // M3-T3 (§68) : `Scaled` est refusé à `OnUse` : `items::apply_instant_effects` n'exécute pas
                // l'enveloppe, l'effet serait accepté puis sans effet (la valeur de `PerCopy` d'un consommable
                // consommé au ramassage n'est pas décidée).
                if matches!(action, effects::Action::Scaled { .. }) {
                    push(
                        LintErrorKind::Unsupported,
                        format!("{at} : Scaled : pas exécuté par OnUse d'un objet (consommable ou actif)"),
                    );
                    continue;
                }
                match action {
                    // M2-T4 : soin instantané d'un actif ou d'un consommable
                    effects::Action::Heal(amount) if *amount <= Fixed::ZERO => push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : Heal = {amount} : doit être > 0"),
                    ),
                    effects::Action::Heal(_) => {}
                    // D61 : bouclier instantané
                    effects::Action::GainShield(0) => push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : GainShield(0) : doit être > 0"),
                    ),
                    effects::Action::GainShield(_) => {}
                    // M3-T3 (§68) : cœurs d'âme instantanés
                    effects::Action::GainSoul(amount) if *amount <= Fixed::ZERO => push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : GainSoul = {amount} : doit être > 0"),
                    ),
                    effects::Action::GainSoul(_) => {}
                    effects::Action::TimedModifier { frames: 0, .. } => push(
                        LintErrorKind::OutOfRange,
                        format!("{at} : TimedModifier frames = 0 : doit être > 0"),
                    ),
                    effects::Action::TimedModifier { .. } | effects::Action::Modifier { .. } => {}
                    other => push(
                        LintErrorKind::Unsupported,
                        format!("{at} : action {other:?} : pas exécutée par OnUse d'un objet"),
                    ),
                }
            }
        }
    }
}

/// M2-T4 (`docs/conventions.md` §44) : synergies. Au moins deux objets distincts et connus,
/// pas deux synergies pour le même ensemble, pas d'effet `OnUse`. Suivi Q10 : une **transformation**
/// (`requires_tag`) peut n'avoir aucun `requires` ; `count` > 0 et atteignable (assez d'objets non
/// consommables portent le tag, sinon la transformation ne s'active jamais), tag connu, `tint` en
/// `#rrggbb`.
fn lint_synergies(registry: &Registry, errors: &mut Vec<LintError>) {
    type Key<'a> = (BTreeSet<&'a str>, Option<(&'a str, u32)>);
    let mut seen: BTreeMap<Key, &str> = BTreeMap::new();
    for synergy in registry.synergies.values() {
        let file = synergy.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let id = &synergy.id;
        if let Some(mods) = &synergy.def.projectile {
            lint_carried_projectile(
                registry,
                &format!("synergie « {id} » : projectile"),
                mods,
                &mut push,
            );
        }
        let set: BTreeSet<&str> = synergy.def.requires.iter().map(String::as_str).collect();
        if let Some(requirement) = &synergy.def.requires_tag {
            // Objets non consommables qui portent le tag : ce que `Inventory::held_with_tag` peut compter.
            let carriers = registry
                .items
                .values()
                .filter(|item| {
                    item.def.kind != items::ItemKind::Consumable
                        && item.def.tags.iter().any(|tag| *tag == requirement.tag)
                })
                .count() as u32;
            if requirement.count == 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!("synergie « {id} » : requires_tag : count = 0 : doit être > 0"),
                );
            }
            if carriers == 0 {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "synergie « {id} » : requires_tag : tag « {} » inconnu (aucun objet ne le porte)",
                        requirement.tag
                    ),
                );
            } else if requirement.count > carriers {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "synergie « {id} » : requires_tag : count = {} : seuls {carriers} objet(s) portent le tag « {} » (jamais active)",
                        requirement.count, requirement.tag
                    ),
                );
            }
        }
        if let Some(tint) = &synergy.def.tint {
            let valid = tint.len() == 7
                && tint.starts_with('#')
                && tint[1..].chars().all(|c| c.is_ascii_hexdigit());
            if !valid {
                push(
                    LintErrorKind::OutOfRange,
                    format!("synergie « {id} » : tint = {tint:?} : attendu #rrggbb"),
                );
            }
        }
        if set.len() < 2 && synergy.def.requires_tag.is_none() {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "synergie « {id} » : requires = {:?} : au moins deux objets distincts",
                    synergy.def.requires
                ),
            );
        }
        for required in &synergy.def.requires {
            match registry.items.get(required) {
                None => push(
                    LintErrorKind::BrokenReference,
                    format!("synergie « {id} » : objet « {required} » inconnu"),
                ),
                Some(item) if item.def.kind == items::ItemKind::Consumable => push(
                    LintErrorKind::Unsupported,
                    format!("synergie « {id} » : « {required} » est un consommable (passifs et actif seulement)"),
                ),
                Some(_) => {}
            }
        }
        let key: Key = (
            set,
            synergy
                .def
                .requires_tag
                .as_ref()
                .map(|requirement| (requirement.tag.as_str(), requirement.count)),
        );
        if let Some(first) = seen.get(&key) {
            push(
                LintErrorKind::DuplicateId,
                format!("synergie « {id} » : même ensemble d'objets que « {first} »"),
            );
        } else {
            seen.insert(key, id);
        }
        for (index, effect) in synergy.def.effects.iter().enumerate() {
            let at = format!("synergie « {id} » : effects[{index}]");
            if effect.on == effects::On::OnUse {
                push(
                    LintErrorKind::Unsupported,
                    format!("{at} : OnUse n'a de sens que pour un objet actif"),
                );
                continue;
            }
            lint_effect(registry, &at, effect, false, &mut push);
        }
    }
}

/// M2-T12 (`docs/conventions.md` §51) : déblocages. Coût ≤ 0, prérequis inconnu ou cyclique,
/// objet ou pool inconnus, poids ≤ 0 d'une entrée ajoutée.
fn lint_unlocks(registry: &Registry, errors: &mut Vec<LintError>) {
    for unlock in registry.unlocks.values() {
        let file = unlock.file.display().to_string();
        let id = &unlock.id;
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        if let Some(cost) = unlock.def.cost.filter(|cost| *cost <= 0) {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "déblocage « {id} » : cost = {cost} (doit être > 0, ou absent pour gratuit)"
                ),
            );
        }
        for required in &unlock.def.requires {
            if !registry.unlocks.contains_key(required) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("déblocage « {id} » : prérequis « {required} » inconnu"),
                );
            }
        }
        if unlock.def.grants.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!("déblocage « {id} » : grants vide (il n'accorde rien)"),
            );
        }
        for (index, grant) in unlock.def.grants.iter().enumerate() {
            match grant {
                items::UnlockGrant::Item(item) => {
                    if !registry.items.contains_key(item) {
                        push(
                            LintErrorKind::BrokenReference,
                            format!(
                                "déblocage « {id} » : grants[{index}] : objet « {item} » inconnu"
                            ),
                        );
                    }
                }
                items::UnlockGrant::Loot {
                    table,
                    pick,
                    weight,
                } => {
                    if !registry.loot_tables.contains_key(table) {
                        push(
                            LintErrorKind::BrokenReference,
                            format!(
                                "déblocage « {id} » : grants[{index}] : pool « {table} » inconnu"
                            ),
                        );
                    }
                    if *weight <= 0 {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("déblocage « {id} » : grants[{index}] : poids {weight} (doit être > 0)"),
                        );
                    }
                    if *weight > items::MAX_LOOT_WEIGHT {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("déblocage « {id} » : grants[{index}] : poids {weight} (au plus {}, le tirage joue un billet par unité de poids)", items::MAX_LOOT_WEIGHT),
                        );
                    }
                    if matches!(pick, items::LootPick::Currency(0)) {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("déblocage « {id} » : grants[{index}] : Currency(0) : le montant doit être > 0"),
                        );
                    }
                    if matches!(pick, items::LootPick::Currency(_))
                        && registry.rooms.values().any(|room| {
                            room.def.shop.as_ref().is_some_and(|shop| {
                                shop.table == *table || shop.floor_tables.contains(table)
                            })
                        })
                    {
                        push(
                            LintErrorKind::Unsupported,
                            format!("déblocage « {id} » : grants[{index}] : Currency ajoutée à « {table} », pool de boutique"),
                        );
                    }
                    if let items::LootPick::Item(item) = pick {
                        if !registry.items.contains_key(item) {
                            push(
                                LintErrorKind::BrokenReference,
                                format!("déblocage « {id} » : grants[{index}] : objet « {item} » inconnu"),
                            );
                        }
                    }
                }
            }
        }
    }
    let table = items::UnlockTable(
        registry
            .unlocks
            .values()
            .map(|unlock| (unlock.id.clone(), unlock.def.clone()))
            .collect(),
    );
    if let Some(id) = table.requires_cycle() {
        let file = registry
            .unlocks
            .get(&id)
            .map(|unlock| unlock.file.display().to_string())
            .unwrap_or_default();
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file,
            message: format!("déblocage « {id} » : cycle de prérequis"),
        });
    }
}

/// M3-T0e (`docs/conventions.md` §64) : curses de run. Poids 0, aucun effet, `FloorRooms` ≤ 0 ou
/// modificateur `Mul` ≤ 0, puis la chance et le pool du champ `curses` de chaque séquence `Floors`.
fn lint_curses(registry: &Registry, errors: &mut Vec<LintError>) {
    let zero = bevy_fixed::fixed_math::Fixed::ZERO;
    let one = bevy_fixed::fixed_math::Fixed::ONE;
    for curse in registry.curses.values() {
        let file = curse.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let id = &curse.id;
        if curse.def.weight == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "curse « {id} » : weight = 0 (doit être > 0 : elle ne serait jamais tirée)"
                ),
            );
        }
        if curse.def.effects.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!("curse « {id} » : effects vide (elle ne fait rien)"),
            );
        }
        for (index, effect) in curse.def.effects.iter().enumerate() {
            match effect {
                run::CurseEffect::FloorRooms(mult) if *mult <= zero => push(
                    LintErrorKind::OutOfRange,
                    format!("curse « {id} » : effects[{index}] (FloorRooms) : mult = {mult} (doit être > 0)"),
                ),
                run::CurseEffect::Modifier(stat, sim_core::modifier::ModifierOp::Mul, value)
                    if *value <= zero =>
                {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("curse « {id} » : effects[{index}] (Modifier {stat:?}, op Mul) : value = {value} (doit être > 0)"),
                    )
                }
                _ => {}
            }
        }
    }
    for floors in registry.floors.values() {
        let Some(curses) = &floors.curses else {
            continue;
        };
        let file = floors.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        if curses.chance < zero || curses.chance > one {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "séquence de niveaux « {} » : curses.chance = {} (doit être dans [0, 1])",
                    floors.id, curses.chance
                ),
            );
        }
        if curses.pool.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "séquence de niveaux « {} » : curses.pool vide (aucune curse ne peut tomber ; retirer `curses`)",
                    floors.id
                ),
            );
        }
        for id in &curses.pool {
            if !registry.curses.contains_key(id) {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "séquence de niveaux « {} » : curses.pool : curse « {id} » inconnue",
                        floors.id
                    ),
                );
            }
        }
    }
}

/// M3-T0e (`docs/conventions.md` §64) : accomplissements. Sans condition ni déblocage, déblocage,
/// objet ou boss inconnus.
fn lint_achievements(registry: &Registry, errors: &mut Vec<LintError>) {
    for achievement in registry.achievements.values() {
        let file = achievement.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let id = &achievement.id;
        if achievement.def.when.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!("accomplissement « {id} » : when vide (il ne serait jamais accompli)"),
            );
        }
        if achievement.def.unlocks.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!("accomplissement « {id} » : unlocks vide (il ne débloque rien)"),
            );
        }
        for unlock in &achievement.def.unlocks {
            if !registry.unlocks.contains_key(unlock) {
                push(
                    LintErrorKind::BrokenReference,
                    format!("accomplissement « {id} » : unlocks : déblocage « {unlock} » inconnu"),
                );
            }
        }
        for (index, condition) in achievement.def.when.iter().enumerate() {
            match condition {
                meta::AchievementCondition::ItemHeld(item)
                    if !registry.items.contains_key(item) =>
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("accomplissement « {id} » : when[{index}] (ItemHeld) : objet « {item} » inconnu"),
                    )
                }
                meta::AchievementCondition::BossDefeated(boss) => {
                    let is_boss = registry
                        .characters
                        .get(&registry::CharacterId::from(boss.clone()))
                        .is_some_and(|character| character.boss.is_some());
                    if !is_boss {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("accomplissement « {id} » : when[{index}] (BossDefeated) : « {boss} » n'est pas un personnage à `boss:`"),
                        )
                    }
                }
                _ => {}
            }
        }
    }
}

/// M2-T5 (`docs/conventions.md` §48) : pools de butin, coffres des types de salle et pools par
/// étage. Pool vide, poids ≤ 0, objet ou rareté sans objet, chances hors de 0..=1, pool ou clé
/// inconnus.
fn lint_loot(registry: &Registry, errors: &mut Vec<LintError>) {
    let zero = bevy_fixed::fixed_math::Fixed::ZERO;
    let one = bevy_fixed::fixed_math::Fixed::ONE;
    for table in registry.loot_tables.values() {
        let file = table.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        let id = &table.id;
        if table.def.entries.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                format!("pool de butin « {id} » : entries vide (au moins une entrée)"),
            );
        }
        // M3-T8 (§73) : un pool à épuisement est tiré à la génération, sans ouvreur : l'exclusion des
        // objets tenus (`exclude_held`) dépendrait du moment et de l'inventaire.
        if table.def.exhaust && table.def.exclude_held {
            push(
                LintErrorKind::Unsupported,
                format!("pool de butin « {id} » : exhaust et exclude_held sont incompatibles (le tirage d'un pool qui s'épuise est fait à la génération de l'étage, sans ouvreur)"),
            );
        }
        for (index, entry) in table.def.entries.iter().enumerate() {
            if entry.weight <= 0 {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "pool de butin « {id} » : entries[{index}] : poids {} (doit être > 0)",
                        entry.weight
                    ),
                );
            }
            if entry.weight > items::MAX_LOOT_WEIGHT {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "pool de butin « {id} » : entries[{index}] : poids {} (au plus {}, le tirage joue un billet par unité de poids)",
                        entry.weight,
                        items::MAX_LOOT_WEIGHT
                    ),
                );
            }
            match &entry.pick {
                items::LootPick::Item(item) => {
                    if !registry.items.contains_key(item) {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("pool de butin « {id} » : entries[{index}] : objet « {item} » inconnu"),
                        );
                    }
                }
                items::LootPick::Weapon(weapon, _) => {
                    if !registry
                        .weapons
                        .contains_key(&crate::registry::WeaponId::from(weapon.clone()))
                    {
                        push(
                            LintErrorKind::BrokenReference,
                            format!("pool de butin « {id} » : entries[{index}] : arme « {weapon} » inconnue"),
                        );
                    }
                }
                items::LootPick::Rarity(rarity) => {
                    if !registry
                        .items
                        .values()
                        .any(|item| item.def.rarity == *rarity)
                    {
                        push(
                            LintErrorKind::OutOfRange,
                            format!("pool de butin « {id} » : entries[{index}] : aucun objet de rareté {rarity:?}"),
                        );
                    }
                }
                items::LootPick::Currency(0) => push(
                    LintErrorKind::OutOfRange,
                    format!("pool de butin « {id} » : entries[{index}] : Currency(0) : le montant doit être > 0"),
                ),
                items::LootPick::Currency(_) => {}
            }
        }
        // M3-T12 (§77) : l'épuisement ne retient que des objets et des armes (`PoolState::record`
        // ignore `Currency`) ; un pool qui n'a que des tas de sous ne s'épuiserait jamais.
        if table.def.exhaust
            && !table.def.entries.is_empty()
            && table
                .def
                .entries
                .iter()
                .all(|entry| matches!(entry.pick, items::LootPick::Currency(_)))
        {
            push(
                LintErrorKind::OutOfRange,
                format!("pool de butin « {id} » : exhaust: true mais toutes les entrées sont des Currency (jamais « vues » par l'épuisement)"),
            );
        }
        if let Some(reward) = &table.def.reward {
            for (name, value) in [
                ("chance", reward.chance),
                ("chest_chance", reward.chest_chance),
            ] {
                if value < zero || value > one {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("pool de butin « {id} » : reward.{name} = {value} (0 à 1)"),
                    );
                }
            }
        }
    }
    let has_key = registry
        .items
        .get("key")
        .is_some_and(|item| item.def.kind == items::ItemKind::Consumable);
    for room in registry.rooms.values() {
        let Some(chest) = &room.def.chest else {
            continue;
        };
        let file = room.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        if chest.locked && !has_key {
            push(
                LintErrorKind::BrokenReference,
                format!(
                    "type de salle « {} » : coffre verrouillé mais aucun objet consommable « key » dans le jeu",
                    room.id
                ),
            );
        }
        if let Some(table) = &chest.table {
            if !registry.loot_tables.contains_key(table) {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "type de salle « {} » : coffre : pool « {table} » inconnu",
                        room.id
                    ),
                );
            }
        }
    }
    for room in registry.rooms.values() {
        let Some(shop) = &room.def.shop else {
            continue;
        };
        let file = room.file.display().to_string();
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: file.clone(),
                message,
            })
        };
        if shop.shelves == 0 {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "type de salle « {} » : boutique sans présentoir (shelves = 0)",
                    room.id
                ),
            );
        }
        if shop.weapon_shelves > shop.shelves {
            push(
                LintErrorKind::OutOfRange,
                format!(
                    "type de salle « {} » : boutique : weapon_shelves = {} > shelves = {}",
                    room.id, shop.weapon_shelves, shop.shelves
                ),
            );
        }
        let mut shop_tables = vec![shop.table.as_str()];
        shop_tables.extend(shop.floor_tables.iter().map(String::as_str));
        for name in shop_tables.iter().skip(1) {
            if !registry.loot_tables.contains_key(*name) {
                push(
                    LintErrorKind::BrokenReference,
                    format!(
                        "type de salle « {} » : boutique : floor_tables : pool « {name} » inconnu",
                        room.id
                    ),
                );
            }
        }
        // M3-T3 (§68) : un tas de sous n'est pas une marchandise (une boutique le vendrait contre
        // des sous) ; la boutique n'y pioche pas.
        for name in &shop_tables {
            let has_currency = registry.loot_tables.get(*name).is_some_and(|table| {
                table
                    .def
                    .entries
                    .iter()
                    .any(|entry| matches!(entry.pick, items::LootPick::Currency(_)))
            });
            if has_currency {
                push(
                    LintErrorKind::Unsupported,
                    format!(
                        "type de salle « {} » : boutique : le pool « {name} » a une entrée Currency (un présentoir ne vend pas de sous)",
                        room.id
                    ),
                );
            }
        }
        if shop.weapon_shelves > 0 {
            let used: Vec<&str> = if shop.floor_tables.is_empty() {
                vec![shop.table.as_str()]
            } else {
                shop.floor_tables.iter().map(String::as_str).collect()
            };
            for name in used {
                let has_weapon = registry.loot_tables.get(name).is_some_and(|table| {
                    table
                        .def
                        .entries
                        .iter()
                        .any(|entry| matches!(entry.pick, items::LootPick::Weapon(..)))
                });
                if registry.loot_tables.contains_key(name) && !has_weapon {
                    push(
                        LintErrorKind::OutOfRange,
                        format!(
                            "type de salle « {} » : boutique : weapon_shelves > 0 mais le pool « {name} » n'a aucune arme",
                            room.id
                        ),
                    );
                }
            }
        }
        // M3-T8 (§73) : prix propre des présentoirs (vie maximale, consommable).
        if let Some(cost) = &shop.cost {
            let player_max = registry
                .characters
                .values()
                .find(|character| character.id.as_str() == "player")
                .and_then(|character| character.base_health_max.literal())
                .unwrap_or(bevy_fixed::fixed_math::Fixed::MAX);
            if let items::Price::Consumable(id, _) = cost {
                let is_consumable = registry
                    .items
                    .get(id)
                    .is_some_and(|item| item.def.kind == items::ItemKind::Consumable);
                if !is_consumable {
                    push(
                        LintErrorKind::BrokenReference,
                        format!(
                            "type de salle « {} » : boutique : cost : « {id} » n'est pas un consommable du jeu",
                            room.id
                        ),
                    );
                }
            }
            for problem in cost.problems(|_| true, player_max) {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "type de salle « {} » : boutique : cost : {problem}",
                        room.id
                    ),
                );
            }
        }
        match registry.loot_tables.get(&shop.table) {
            None => push(
                LintErrorKind::BrokenReference,
                format!(
                    "type de salle « {} » : boutique : pool « {} » inconnu",
                    room.id, shop.table
                ),
            ),
            Some(table) if table.def.entries.is_empty() => push(
                LintErrorKind::OutOfRange,
                format!(
                    "type de salle « {} » : boutique : le pool « {} » est vide",
                    room.id, shop.table
                ),
            ),
            Some(_) => {}
        }
    }
    // M3-T8 (§73) : butin du boss d'un type de salle : le pool existe (et n'est pas vide : le lint des
    // pools le dit déjà).
    for room in registry.rooms.values() {
        let Some(drop) = &room.def.boss_drop else {
            continue;
        };
        if !registry.loot_tables.contains_key(&drop.table) {
            errors.push(LintError {
                kind: LintErrorKind::BrokenReference,
                file: room.file.display().to_string(),
                message: format!(
                    "type de salle « {} » : boss_drop : pool « {} » inconnu",
                    room.id, drop.table
                ),
            });
        }
    }
    for floors in registry.floors.values() {
        for table in &floors.loot {
            if !registry.loot_tables.contains_key(table) {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: floors.file.display().to_string(),
                    message: format!(
                        "séquence de niveaux « {} » : champ loot : pool « {table} » inconnu",
                        floors.id
                    ),
                });
            }
        }
    }
}

/// M2-E1 (`docs/conventions.md` §35) : le champ de niveau `room_kind` d'un gabarit de salle
/// (`RoomTemplate`) désigne un type du dossier `Room` ; un type inconnu est une erreur.
fn lint_room_kinds(registry: &Registry, errors: &mut Vec<LintError>) {
    // M3-T0c (§62) : verrou de porte (`Key(0)` n'ouvrirait jamais rien).
    for room in registry.rooms.values() {
        if let Some(message) = room.def.door_lock.and_then(|lock| lock.problem()) {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: room.file.display().to_string(),
                message: format!("type de salle « {} » : {message}", room.id),
            });
        }
        // M3-T5 (§70) : `lock_from_floor` ne retarde que le verrou d'une porte.
        if room.def.lock_from_floor > 0 && room.def.door_lock.is_none() {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: room.file.display().to_string(),
                message: format!(
                    "type de salle « {} » : lock_from_floor sans door_lock (lock_from_floor = {}, aucun verrou à retarder)",
                    room.id, room.def.lock_from_floor
                ),
            });
        }
        // Le mur secret ne lit pas l'étage (`is_secret`) : un retard serait ignoré, donc refusé.
        if room.def.lock_from_floor > 0 && room.def.door_lock == Some(world::DoorLock::Secret) {
            errors.push(LintError {
                kind: LintErrorKind::OutOfRange,
                file: room.file.display().to_string(),
                message: format!(
                    "type de salle « {} » : lock_from_floor sur door_lock Secret (le mur secret est actif dès le premier étage ; lock_from_floor = {})",
                    room.id, room.def.lock_from_floor
                ),
            });
        }
    }
    for map in registry.maps.values() {
        for (level, kind) in &map.room_kinds {
            if !registry.rooms.contains_key(kind) {
                let known: Vec<_> = registry.rooms.keys().map(String::as_str).collect();
                errors.push(LintError {
                    kind: LintErrorKind::UnknownKind,
                    file: map.file.display().to_string(),
                    message: format!(
                        "RoomTemplate « {level} » : room_kind « {kind} » inconnu (types de salle chargés : {})",
                        if known.is_empty() {
                            "aucun, dossier de kind Room absent".to_string()
                        } else {
                            known.join(", ")
                        }
                    ),
                });
            }
        }
    }
}

/// M2-T7 (§45) : une arène a une horloge > 0, au moins une vague, des vagues datées dans
/// l'horloge (`at < clock`), dans l'ordre, et des ennemis connus en nombre > 0.
fn lint_arenas(registry: &Registry, errors: &mut Vec<LintError>) {
    for room in registry.rooms.values() {
        let Some(arena) = &room.def.arena else {
            continue;
        };
        let mut push = |kind: LintErrorKind, message: String| {
            errors.push(LintError {
                kind,
                file: room.file.display().to_string(),
                message: format!("salle « {} » : arène : {message}", room.id),
            })
        };
        if arena.clock == 0 {
            push(
                LintErrorKind::OutOfRange,
                "clock = 0 : doit être > 0".into(),
            );
        }
        if arena.waves.is_empty() {
            push(
                LintErrorKind::OutOfRange,
                "aucune vague (au moins une)".into(),
            );
        }
        let mut previous: Option<u32> = None;
        for (index, wave) in arena.waves.iter().enumerate() {
            if arena.clock > 0 && wave.at >= arena.clock {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "waves[{index}] : at = {} >= clock ({}) : la vague n'arriverait jamais",
                        wave.at, arena.clock
                    ),
                );
            }
            if previous.is_some_and(|before| wave.at < before) {
                push(
                    LintErrorKind::OutOfRange,
                    format!(
                        "waves[{index}] : at = {} avant la vague précédente",
                        wave.at
                    ),
                );
            }
            previous = Some(wave.at);
            if wave.enemies.is_empty() {
                push(
                    LintErrorKind::OutOfRange,
                    format!("waves[{index}] : aucun ennemi"),
                );
            }
            for (id, count) in &wave.enemies {
                if !registry
                    .characters
                    .contains_key(&registry::CharacterId::from(id.clone()))
                {
                    push(
                        LintErrorKind::BrokenReference,
                        format!("waves[{index}] : personnage « {id} » inconnu"),
                    );
                }
                if *count == 0 {
                    push(
                        LintErrorKind::OutOfRange,
                        format!("waves[{index}] : « {id} » : count = 0 : doit être > 0"),
                    );
                }
            }
        }
    }
}

/// T1.12 : chaque `CharacterSpawn` d'une carte désigne un personnage connu (champ `character`
/// non vide). Avant, seuls ceux qui imposent une variante étaient vérifiés
/// ([`lint_forced_variants`]).
fn lint_map_characters(registry: &Registry, errors: &mut Vec<LintError>) {
    for map in registry.maps.values() {
        for character in &map.spawned_characters {
            if !registry
                .characters
                .contains_key(&registry::CharacterId::from(character.clone()))
            {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: map.file.display().to_string(),
                    message: format!(
                        "CharacterSpawn : personnage inconnu « {character} » (aucun personnage chargé avec cet id)"
                    ),
                });
            }
        }
    }
}

/// T1.12 (audit) : `entry.progression` (T1.10) désigne une progression chargée. Le champ
/// `progression` d'un scénario n'est pas du contenu : le runner l'ignore avec un avertissement
/// s'il est inconnu (§27).
fn lint_entry_progression(
    registry: &Registry,
    manifest: &GameManifest,
    errors: &mut Vec<LintError>,
) {
    let Some(id) = &manifest.entry.progression else {
        return;
    };
    if !registry
        .progression
        .contains_key(&registry::ProgressionId::from(id.clone()))
    {
        errors.push(LintError {
            kind: LintErrorKind::BrokenReference,
            file: crate::manifest::MANIFEST_FILE_NAME.to_string(),
            message: format!(
                "entry.progression : progression inconnue « {id} » (kind Progression)"
            ),
        });
    }
}

/// M2-T15 (`docs/conventions.md` §53) : un modificateur (ou une base de stat) sur une stat que
/// `sim_core::stats::stat_reader` ne connaît pas ne ferait rien : erreur.
fn lint_stat_readers(registry: &Registry, errors: &mut Vec<LintError>) {
    for (file, at, stat) in crate::reach::stat_uses(registry) {
        if sim_core::stats::stat_reader(&stat).is_none() {
            errors.push(LintError {
                kind: LintErrorKind::UnreadStat,
                file,
                message: format!("{at} : stat {stat:?} : aucun système ne la lit"),
            });
        }
    }
}

/// M3-T12 (`docs/conventions.md` §77) : couches `Rocks` et `Surfaces` des niveaux d'une carte. Une
/// valeur de rocher autre que 1 (rocher) ou 2 (rocher à butin) et une valeur de surface sans fichier
/// `surfaces/*.ron` sont ignorées au chargement : le gabarit ne ferait pas ce que son auteur a peint.
fn lint_map_layers(registry: &Registry, errors: &mut Vec<LintError>) {
    let declared: BTreeSet<u8> = registry
        .surfaces
        .values()
        .map(|surface| surface.intgrid_value)
        .collect();
    for map in registry.maps.values() {
        let file = map.file.display().to_string();
        for level in &map.levels {
            if !level.rock_other_values.is_empty() {
                let values: Vec<String> = level
                    .rock_other_values
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                errors.push(LintError {
                    kind: LintErrorKind::OutOfRange,
                    file: file.clone(),
                    message: format!(
                        "niveau « {} » : couche Rocks : valeur(s) {} : seules 1 (rocher) et 2 (rocher à butin) existent",
                        level.id,
                        values.join(", ")
                    ),
                });
            }
            let unknown: Vec<String> = level
                .surface_values
                .iter()
                .filter(|value| !declared.contains(value))
                .map(ToString::to_string)
                .collect();
            if !unknown.is_empty() {
                errors.push(LintError {
                    kind: LintErrorKind::BrokenReference,
                    file: file.clone(),
                    message: format!(
                        "niveau « {} » : couche Surfaces : valeur(s) {} sans surface (intgrid_value d'un fichier surfaces/*.ron)",
                        level.id,
                        unknown.join(", ")
                    ),
                });
            }
        }
    }
}
