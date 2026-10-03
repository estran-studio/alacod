//! Économie de run (T2.3, chantier C5 v1, `docs/plan-engine.md` §5 C5) : contenu RON
//! (`EconomyConfig`, `PerksConfig`) et systèmes qui *produisent* de la monnaie (points de
//! kill/coup/réparation, [`award_points_system`]) — [`run::currency::Currency`]/
//! [`run::perks::Perks`] eux-mêmes, et l'API saturante qui les manipule, vivent dans le
//! crate minimal `run` (voir sa doc). Les achats (portes, armes murales, perks) *dépensent*
//! directement dans `interaction.rs`, aux côtés des autres gestionnaires d'interaction.
//!
//! # Attribution des points (décision)
//!
//! Chaque source de points détecte l'occasion **au moment où l'information est disponible**
//! (pas en `RollbackSystemSet::Run`, trop tard : une entité tuée est déjà despawn par
//! `rollback_apply_death`, `DeathManagement`) et pousse un [`PointsCredit`] dans
//! `FrameEvents<PointsCredit>` :
//! - **Kill** : `character::health::rollback_apply_accumulated_damage`
//!   (`RollbackSystemSet::DeathManagement`), au moment exact où `Death` est posée sur une
//!   entité qui n'est pas un joueur, si `DamageAccumulator::last_hit_by` contient
//!   `HitBy::Player(handle)` (dernier coup porté par ce joueur — CoD attribue le kill au tireur
//!   du coup fatal, pas à qui a le plus tapé dedans).
//! - **Hit** : `character::health::rollback_resolve_damage_events`
//!   (`RollbackSystemSet::CollisionDamage`), à chaque `DamageEvent` résolu en dégât réel
//!   (`combat::damage::resolve_damage` renvoie `Some`) dont la source est un joueur et la
//!   cible n'est pas un joueur (pas de points pour se tirer dessus entre joueurs, même sous
//!   tir ami `Cursed`/`Always`).
//! - **Repair** : `interaction::handle_window_repair` (`RollbackSystemSet::Interaction`), à
//!   chaque réparation qui avance réellement la santé de la fenêtre.
//! - **Nuke** (D17) : `powerups::apply_powerup_actions_system` (`RollbackSystemSet::Effects`),
//!   au ramassage d'un power-up portant `effects::Action::KillAllWaveEnemies` : un crédit
//!   par joueur vivant, pas par ennemi tué (CoD : 400 points à chaque joueur, quel que soit
//!   le nombre de zombies). Les ennemis tués par le nuke ne rapportent pas de points de kill
//!   (`Death { last_hit_by: None }`).
//!
//! [`award_points_system`] (`RollbackSystemSet::Run`, en fin de frame) est le **seul**
//! point qui appelle `Currency::earn`/émet `CurrencyEvent` pour ces quatre sources : il vide
//! `FrameEvents<PointsCredit>` (encore valide à ce point de la frame, vidée seulement au
//! `FrameStart` suivant) et résout `economy.ron` une fois pour tous les crédits de la frame.
//! Centraliser la lecture de configuration et l'émission d'événements ici, plutôt que dans
//! chacun des trois émetteurs, évite de dupliquer la résolution de `EconomyConfig` trois fois
//! par frame et garde un seul endroit qui décide de la forme exacte de `CurrencyEvent::reason`.

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_fixed::fixed_math::{self, Fixed};
use bevy_ggrs::{GgrsSchedule, Rollback};
use effects::CURRENCY_MULTIPLIER_STAT;
use serde::{Deserialize, Serialize};
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::modifier::ModifierSource;
use sim_core::stats::StatId;
use stats::StatReader;
use std::collections::BTreeMap;
use utils::{net_id::GgrsNetId, order_mut_iter};

use crate::character::player::Player;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use crate::waves::WaveState;
use run::currency::{Currency, CurrencyEvent};

/// `ModifierSource` d'un perk acheté (T2.3) : `perk:<id>`, permanent (`until: None`). Un seul
/// achat par perk et par joueur (`run::perks::Perks`, vérifié par
/// `interaction::handle_perk_purchase_interaction` avant de poser le modificateur) : pas
/// besoin de retirer cette source en cours de partie aujourd'hui (contrairement à un statut
/// temporaire, voir `combat::downed::downed_modifier_source`).
pub fn perk_modifier_source(perk_id: &str) -> ModifierSource {
    ModifierSource::Named(format!("perk:{perk_id}"))
}

/// Configuration d'économie (`games/<jeu>/assets/economy/economy.ron`, kind de contenu
/// `Economy`, T2.3). Défauts repris de `content::registry` (voir sa doc) : un fichier qui ne
/// déclare qu'un sous-ensemble des champs se comporte pareil ici et au lint.
///
/// F5 (chantier m0-v11) : les champs numériques sont des [`NumOrExpr`] — littéral
/// (inchangé) ou expression évaluée une fois au lancement (voir `crate::balance`).
///
/// `starting_currency` n'est **pas** un champ de ce type : la monnaie de départ est par
/// personnage (`character::config::CharacterConfig::starting_currency`), pas globale au jeu —
/// voir sa doc pour la justification (un testbed pourrait vouloir un personnage riche et un
/// autre pauvre, par exemple).
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    #[serde(default = "default_kill_points")]
    pub kill_points: content::expr::NumOrExpr,
    #[serde(default = "default_hit_points")]
    pub hit_points: content::expr::NumOrExpr,
    #[serde(default = "default_repair_points")]
    pub repair_points: content::expr::NumOrExpr,
    /// Points crédités à chaque joueur vivant au ramassage d'un nuke (D17, CoD : 400),
    /// une fois par joueur et non par ennemi tué.
    #[serde(default = "default_nuke_points")]
    pub nuke_points: content::expr::NumOrExpr,
    /// Plafond de points de réparation gagnés par joueur et par vague (CoD : les points de
    /// réparation de fenêtre sont limités par round). `None` (défaut) : pas de plafond.
    #[serde(default)]
    pub repair_points_cap_per_wave: Option<content::expr::NumOrExpr>,
    /// Ratio du prix d'achat facturé pour recharger la réserve d'une arme murale déjà
    /// possédée (`interaction::handle_weapon_pickup_interaction`) : `refill_price =
    /// round(price × refill_price_ratio)`.
    #[serde(default = "default_refill_price_ratio")]
    pub refill_price_ratio: content::expr::NumOrExpr,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            kill_points: default_kill_points(),
            hit_points: default_hit_points(),
            repair_points: default_repair_points(),
            nuke_points: default_nuke_points(),
            repair_points_cap_per_wave: None,
            refill_price_ratio: default_refill_price_ratio(),
        }
    }
}

fn default_kill_points() -> content::expr::NumOrExpr {
    content::expr::NumOrExpr::Integer(60)
}

fn default_hit_points() -> content::expr::NumOrExpr {
    content::expr::NumOrExpr::Integer(10)
}

fn default_repair_points() -> content::expr::NumOrExpr {
    content::expr::NumOrExpr::Integer(10)
}

fn default_nuke_points() -> content::expr::NumOrExpr {
    content::expr::NumOrExpr::Integer(400)
}

fn default_refill_price_ratio() -> content::expr::NumOrExpr {
    content::expr::NumOrExpr::Literal(fixed_math::new(0.5))
}

/// Un perk de `games/<jeu>/assets/economy/perks.ron` (kind de contenu `Perk`, T2.3) :
/// `{ "juggernog": (name: "Juggernog", price: 2500, modifiers: [(stat: MaxHealth, op: Mul,
/// value: "2.0")]) }`. `modifiers` posés permanents (`until: None`) via
/// `sim_core::modifier::Modifiers::push_from` à l'achat, source [`perk_modifier_source`].
///
/// F5 (chantier m0-v11) : `price` accepte un littéral ou une expression (voir
/// `crate::balance::ResolvedPerkDef` pour la valeur résolue) ; `modifiers[].value` reste
/// un `Fixed` littéral (hors périmètre F5).
#[derive(Debug, Clone, Deserialize)]
pub struct PerkDef {
    pub name: String,
    pub price: content::expr::NumOrExpr,
    #[serde(default)]
    pub modifiers: Vec<PerkModifierDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PerkModifierDef {
    pub stat: StatId,
    pub op: sim_core::modifier::ModifierOp,
    pub value: Fixed,
}

#[derive(Asset, TypePath, Debug, Clone, Default, Deserialize)]
pub struct PerksConfig(pub BTreeMap<String, PerkDef>);

/// Machine à perks (T2.3) : posée par
/// `map_ldtk::game::local::spawn_soda_locations_when_map_loaded` sur une entité rollback aux
/// côtés d'`Interactable { interaction_type: Perk }`. `perk_id` est relu au chargement de la
/// map (pas de champ `price` : le prix vit dans `perks.ron`, une seule source de vérité,
/// contrairement à `WeaponPickup::price` qui doit voyager avec l'entité murale parce que
/// l'arme elle-même n'est identifiée que par son id, sans schéma RON propre côté carte).
#[derive(Component, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct PerkMachine {
    pub perk_id: String,
}

/// Occasion de gagner des points, détectée au plus près de l'information (voir la doc du
/// module) et résolue par [`award_points_system`] en fin de frame. `Nuke` (D17) : un crédit
/// par joueur vivant au ramassage, émis par `powerups::apply_powerup_actions_system`.
#[derive(Debug, Clone, Copy, Hash)]
pub enum PointsCredit {
    Kill { handle: usize },
    Hit { handle: usize },
    Repair { handle: usize },
    Nuke { handle: usize },
}

/// Points de réparation déjà gagnés par joueur pour la vague en cours (T2.3,
/// `EconomyConfig::repair_points_cap_per_wave`). Remise à zéro pour tout le monde au
/// changement de vague (`wave` ne correspond plus à `WaveState::current_wave`) : un plafond
/// « par vague » repart à zéro à chaque nouvelle vague, comme dans CoD.
#[derive(Resource, Debug, Clone, Default, Hash, Serialize, Deserialize)]
pub struct RepairPointsTracking {
    wave: u32,
    /// `BTreeMap` (pas `HashMap`, CLAUDE.md règle 5) : handle -> points de réparation déjà
    /// crédités cette vague.
    earned: BTreeMap<usize, u32>,
}

impl RepairPointsTracking {
    /// Vrai si `points` peuvent encore être crédités à `handle` pour `current_wave` sous
    /// `cap` (`None` = pas de plafond, toujours vrai). Incrémente le compteur seulement
    /// quand elle renvoie vrai (un refus ne consomme rien, une réparation refusée peut
    /// gagner des points la vague suivante si le joueur retente).
    fn try_earn(
        &mut self,
        handle: usize,
        current_wave: u32,
        points: u32,
        cap: Option<u32>,
    ) -> bool {
        if self.wave != current_wave {
            self.wave = current_wave;
            self.earned.clear();
        }
        let Some(cap) = cap else {
            return true;
        };
        let entry = self.earned.entry(handle).or_insert(0);
        if *entry >= cap {
            false
        } else {
            *entry += points;
            true
        }
    }
}

/// Seul point d'écriture de `Currency::earn`/d'émission de `CurrencyEvent` pour les points de
/// kill/coup/réparation/nuke (voir la doc du module) ; `RollbackSystemSet::Run`, après tout ce qui
/// peut produire un [`PointsCredit`] cette frame (`CollisionDamage`, `DeathManagement`,
/// `Interaction` — tous avant `Run` dans `RollbackSystemSet::ORDER`).
///
/// Double Points (T2.5, chantier C1 v0, `effects::Action::CurrencyMultiplier`) : chaque
/// montant crédité est multiplié par la stat résolue [`CURRENCY_MULTIPLIER_STAT`] du joueur
/// crédité (`stats::StatReader`, base neutre `1.0` — voir la doc du module `game::powerups`
/// et `sim_core::modifier::resolve`, aucun modificateur actif ne multiplie même pas par 1).
/// Arrondi au plus proche (`Fixed::round`, comme `EconomyConfig::refill_price`). Les points
/// du nuke (D17) passent par le même chemin : sous Double Points, un nuke rapporte 800
/// (comme dans CoD).
#[allow(clippy::too_many_arguments)]
pub fn award_points_system(
    credits: Res<FrameEvents<PointsCredit>>,
    balance: Res<crate::balance::ResolvedBalance>,
    wave_state: Res<WaveState>,
    mut repair_tracking: ResMut<RepairPointsTracking>,
    stats: StatReader,
    mut players: Query<(&GgrsNetId, Entity, &Player, &mut Currency), With<Rollback>>,
    mut currency_events: ResMut<FrameEvents<CurrencyEvent>>,
) {
    if credits.is_empty() {
        return;
    }

    // F5 (chantier m0-v11) : config résolue une fois au lancement (voir `crate::balance`),
    // plus jamais lue depuis l'asset — valeurs identiques pour un contenu littéral.
    let config = &balance.economy;

    for credit in credits.iter() {
        let (target_handle, amount, reason) = match *credit {
            PointsCredit::Kill { handle } => (handle, config.kill_points, "kill"),
            PointsCredit::Hit { handle } => (handle, config.hit_points, "hit"),
            PointsCredit::Repair { handle } => {
                if !repair_tracking.try_earn(
                    handle,
                    wave_state.current_wave,
                    config.repair_points,
                    config.repair_points_cap_per_wave,
                ) {
                    continue;
                }
                (handle, config.repair_points, "repair")
            }
            PointsCredit::Nuke { handle } => (handle, config.nuke_points, "nuke"),
        };
        if amount == 0 {
            continue;
        }
        for (_net_id, entity, player, mut currency) in order_mut_iter!(players) {
            if player.handle == target_handle {
                let multiplier = stats.get(
                    entity,
                    &StatId::Custom(CURRENCY_MULTIPLIER_STAT.to_string()),
                    fixed_math::FIXED_ONE,
                );
                let boosted = if multiplier == fixed_math::FIXED_ONE {
                    amount
                } else {
                    Fixed::from_num(amount)
                        .saturating_mul(multiplier)
                        .round()
                        .to_num::<u32>()
                };
                currency.earn(boosted);
                currency_events.send(CurrencyEvent {
                    handle: target_handle,
                    delta: boosted as i64,
                    reason: reason.to_string(),
                });
                break;
            }
        }
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<EconomyConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<PerksConfig>::new(&["ron"]));

        app.add_frame_events::<PointsCredit>();

        app.init_resource::<RepairPointsTracking>();
        app.rollback_and_trace_resource::<RepairPointsTracking>();
        app.rollback_and_trace::<PerkMachine>();

        app.add_systems(
            GgrsSchedule,
            award_points_system.in_set(RollbackSystemSet::Run),
        );
    }
}
