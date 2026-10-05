pub mod ui;

use bevy::{
    log::{tracing::span, Level},
    prelude::*,
};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::damage::{resolve_damage, Defenses};
use combat::downed::{downed_modifier_source, Downed, Reviving};
use ggrs::PlayerHandle;
use run::{Run, RunEnd, RunStep};
use serde::{Deserialize, Serialize};
use sim_core::damage::DamageEvent;
use sim_core::modifier::{ModifierOp, Modifiers};
use sim_core::players::PlayersCount;
use sim_core::stats::StatId;
use sim_core::team::Team;
use stats::StatReader;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::player::Player;
use crate::economy::PointsCredit;
use crate::frame_events::FrameEvents;
use crate::interaction::{Interactable, InteractionType};

#[derive(Component, Reflect, Debug, Clone, Hash, Serialize, Deserialize)]
pub enum HitBy {
    Entity(GgrsNetId),
    Player(PlayerHandle),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthConfig {
    /// F5 (chantier m0-v11) : littéral (chaîne `Fixed`, inchangé) ou expression évaluée
    /// une fois au lancement (voir `crate::balance` et `docs/conventions.md` §18).
    pub max: content::expr::NumOrExpr,
    #[serde(default)]
    pub regen_rate: Option<fixed_math::Fixed>, // Health per second
    #[serde(default)]
    pub regen_delay_frames: Option<u32>, // Frames to wait after taking damage before regen starts
}

#[derive(Component, Clone, Debug, Hash, Serialize, Default, Deserialize)]
pub struct HealthRegen {
    pub last_damage_frame: u32,
    pub regen_rate: fixed_math::Fixed,
    pub regen_delay_frames: u32,
}

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct Death {
    pub last_hit_by: Option<Vec<HitBy>>,
}

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct DamageAccumulator {
    pub total_damage: fixed_math::Fixed,
    pub hit_count: u32,
    pub last_hit_by: Option<Vec<HitBy>>,
}

/// Compte les coups reçus par une entité (T2.9, testbed : la cible `target`). Posé à la
/// création par `character::create::create_character` quand `CharacterConfig::counts_hits`
/// est vrai (aucun personnage zombie/joueur existant ne le déclare) ; incrémenté ici par
/// [`rollback_resolve_damage_events`] pour chaque `DamageEvent` résolu en un dégât réel
/// (`combat::damage::resolve_damage` renvoie `Some`), qu'il porte ce composant ou non — seule
/// une entité qui le porte voit son compteur avancer. Rollback + trace (composant `Hash`,
/// voir `BaseCharacterGamePlugin`), pour rester lisible par une attente `EntityHits` de
/// scénario (`game::replay::Expectation::EntityHits`).
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct HitCount(pub u32);

impl fmt::Display for HitBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HitBy::Entity(net_id) => write!(f, "NetId({})", net_id.0),
            HitBy::Player(player_handle) => write!(f, "Player({})", player_handle),
        }
    }
}

impl fmt::Display for Death {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.last_hit_by {
            Some(hits) if !hits.is_empty() => {
                for (i, hit_by) in hits.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", hit_by)?;
                }
                Ok(())
            }
            Some(_) | None => write!(f, "Died (cause unknown or no direct hit)"),
        }
    }
}

/// Santé pleine (`current == max`) pour une valeur max **déjà résolue** (F5, chantier
/// m0-v11 : `ResolvedBalance::health_max_by_character` ; remplace `From<HealthConfig>`,
/// qui ne peut plus exister puisque `HealthConfig::max` peut être une expression non
/// évaluée ici). `Health` vient de la crate `combat` : fonction libre plutôt qu'un
/// `impl Health` (règle de l'orphelin).
pub fn new_health(max: fixed_math::Fixed) -> Health {
    Health {
        current: max,
        max,
        invulnerable_until_frame: None,
    }
}

/// Lit `FrameEvents<DamageEvent>` (émis par six émetteurs : collision de balles, collision de
/// mêlée, attaque d'ennemi, collision de projectile composable (B5), brûlure des statuts (T1.3),
/// charge d'ennemi (T1.4) ; `docs/conventions.md` §8) dans l'ordre d'émission et applique
/// `combat::damage::resolve_damage` (équipe, tir ami, tags, résistances, immunités,
/// invulnérabilité — `Health.invulnerable_until_frame` enfin honoré) ; accumule le
/// résultat dans `DamageAccumulator` (`HitBy`/`last_hit_by` pour l'attribution des kills,
/// comme avant T1.1). Seul point d'écriture de `DamageAccumulator` : les émetteurs
/// n'y touchent plus directement (T1.1, chantier B1).
///
/// `RollbackSystemSet::CollisionDamage`, après les émetteurs (`Weapon`, `Projectiles`, et
/// les traducteurs `enemy_attack_damage_translate_system`/`charge_damage_translate_system`),
/// avant `DeathManagement`.
pub fn rollback_resolve_damage_events(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<DamageEvent>>,
    mut commands: Commands,
    // T2.3, chantier C5 v1 : points au coup au but (voir `crate::economy`, doc du module,
    // section « Attribution des points »).
    mut points_credits: ResMut<FrameEvents<PointsCredit>>,
    net_id_query: Query<(&GgrsNetId, Entity), With<Rollback>>,
    player_handle_query: Query<(&GgrsNetId, &Player)>,
    mut target_query: Query<
        (
            &GgrsNetId,
            &Team,
            Option<&Defenses>,
            Option<&Health>,
            Option<&mut DamageAccumulator>,
            Has<Downed>,
            Option<&mut HitCount>,
        ),
        With<Rollback>,
    >,
    // T1.9 : multiplicateur de difficulté des dégâts infligés par les ennemis (1 sans
    // difficulté activée : aucun calcul, voir `crate::clock::scale`).
    clock: Res<run::Clock>,
) {
    if events.is_empty() {
        return;
    }

    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "resolve_damage");
    let _enter = system_span.enter();

    // GgrsNetId -> Entity, déterministe (BTreeMap) : construit une fois pour retrouver la
    // cible de chaque événement (identifiée par net_id, jamais par Entity — CLAUDE.md règle 3).
    let mut entity_by_net_id: BTreeMap<usize, Entity> = BTreeMap::new();
    for (net_id, entity) in order_iter!(net_id_query) {
        entity_by_net_id.insert(net_id.0, entity);
    }
    let mut player_handle_by_net_id: BTreeMap<usize, PlayerHandle> = BTreeMap::new();
    for (net_id, player) in order_iter!(player_handle_query) {
        player_handle_by_net_id.insert(net_id.0, player.handle);
    }

    let default_defenses = Defenses::default();

    for event in events.iter() {
        let Some(&entity) = entity_by_net_id.get(&event.target.0) else {
            continue; // cible déjà disparue (rollback, mort le même frame par un autre coup)
        };
        let Ok((
            target_net_id,
            target_team,
            opt_defenses,
            opt_health,
            opt_accumulator,
            downed,
            opt_hit_count,
        )) = target_query.get_mut(entity)
        else {
            continue;
        };
        // Pas de santé : rien à blesser (ex. un mur touché par erreur).
        if opt_health.is_none() {
            continue;
        }
        // À terre (T1.3) : `Health` n'est plus touchée tant qu'un joueur est `Downed` — voir
        // la doc de `combat::downed::Downed`. Ignoré ici plutôt que dans `resolve_damage`
        // (fonction pure, sans accès ECS) : ce résolveur est déjà le seul point qui lit à la
        // fois l'événement et l'état de la cible.
        if downed {
            continue;
        }
        let invulnerable = opt_health
            .and_then(|h| h.invulnerable_until_frame)
            .is_some_and(|until| event.frame <= until);
        let defenses = opt_defenses.unwrap_or(&default_defenses);

        let Some(amount) = resolve_damage(
            event.source_team,
            *target_team,
            event.friendly_fire,
            &event.tags,
            defenses,
            event.kind.clone(),
            // T1.9 : un seul point pour tous les dégâts d'ennemis (griffes, coups directs,
            // charges, projectiles d'émetteurs) : ceux dont le tireur est de l'équipe Enemies.
            if event.source_team == Team::Enemies {
                crate::clock::scale(event.amount, clock.difficulty)
            } else {
                event.amount
            },
            invulnerable,
        ) else {
            continue;
        };

        // T2.9 (testbed) : compte le coup pour toute entité qui porte `HitCount`, quel que
        // soit le montant final (le coup a bien été résolu en dégât réel ici, après équipe,
        // tir ami, immunités et résistances).
        if let Some(mut hit_count) = opt_hit_count {
            hit_count.0 += 1;
        }

        let mut last_hit_by = Vec::with_capacity(2);
        if let Some(&handle) = player_handle_by_net_id.get(&event.source.0) {
            last_hit_by.push(HitBy::Player(handle));
            // T2.3, chantier C5 v1 : coup au but porté par un joueur sur une cible qui n'en
            // est pas un (pas de points à se tirer dessus entre joueurs, même sous tir ami).
            if *target_team != Team::Players {
                points_credits.send(PointsCredit::Hit { handle });
            }
        }
        last_hit_by.push(HitBy::Entity(event.source.clone()));

        if let Some(mut accumulator) = opt_accumulator {
            accumulator.total_damage = accumulator.total_damage.saturating_add(amount);
            accumulator.hit_count += 1;
            accumulator.last_hit_by = Some(last_hit_by);
        } else {
            commands.entity(entity).insert(DamageAccumulator {
                hit_count: 1,
                total_damage: amount,
                last_hit_by: Some(last_hit_by),
            });
        }

        info!(
            "{} <- {} dmg from {} (kind {:?}, tags {:?})",
            target_net_id, amount, event.source, event.kind, event.tags
        );
    }
}

/// Handle par défaut de `bleedout_frames`/`downed_speed_mult` quand la config n'a pas pu
/// être lue (`CharacterConfigHandles` absent ou asset pas encore chargé — ne devrait pas
/// arriver en pratique, garde défensive). Mêmes valeurs que `CharacterConfig::default_*`
/// (`character/config.rs`), dupliquées ici en constantes simples pour ne pas dépendre d'une
/// instance de `CharacterConfig` déjà construite.
const FALLBACK_BLEEDOUT_FRAMES: u32 = 1800;

pub fn rollback_apply_accumulated_damage(
    frame: Res<FrameCount>,
    mut commands: Commands,
    players_count: Res<PlayersCount>,
    character_configs: Res<Assets<CharacterConfig>>,
    // T2.3, chantier C5 v1 : points au kill (voir `crate::economy`, doc du module, section
    // « Attribution des points »).
    mut points_credits: ResMut<FrameEvents<PointsCredit>>,
    // Lecture seule, indépendante de la query mutable ci-dessous (aucun composant en
    // commun : `Player`/`Downed` vs `Health`/`DamageAccumulator`/`Modifiers`/
    // `CharacterConfigHandles`) : sert à savoir, pour un joueur qui tombe à 0 PV cette
    // frame, si un AUTRE joueur est encore debout (ni mort, ni déjà à terre).
    standing_players: Query<(&Player, Has<Downed>)>,
    mut query: Query<
        (
            &GgrsNetId,
            Entity,
            &DamageAccumulator,
            &mut Health,
            Option<&mut HealthRegen>,
            Option<&Player>,
            Option<&CharacterConfigHandles>,
            Option<&mut Modifiers>,
        ),
        With<Rollback>,
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "apply_damage");
    let _enter = system_span.enter();

    // À terre (T1.3, chantier B6) : handles des joueurs encore debout, lus une fois avant la
    // boucle. Un joueur qui tombe à terre CETTE frame ne porte pas encore `Downed` au moment
    // de cette lecture (il est ajouté plus bas, après coup, par `commands`) : il compte donc
    // comme « debout » pour lui-même, ce qui est correct — la règle porte sur les AUTRES
    // joueurs (`other != player.handle` plus bas), jamais sur lui-même.
    let standing_handles: BTreeSet<usize> = standing_players
        .iter()
        .filter(|(_, downed)| !downed)
        .map(|(player, _)| player.handle)
        .collect();

    for (
        g_id,
        entity,
        accumulator,
        mut health,
        opt_regen,
        opt_player,
        opt_config_handles,
        opt_modifiers,
    ) in order_mut_iter!(query)
    {
        if accumulator.total_damage > fixed_math::FIXED_ZERO {
            health.current = health.current.saturating_sub(accumulator.total_damage);

            info!(
                "{} receive {} dmg health is {}",
                g_id, accumulator.total_damage, health.current
            );

            // Update last damage frame for regen
            if let Some(mut regen) = opt_regen {
                regen.last_damage_frame = frame.frame;
            }

            commands.entity(entity).remove::<DamageAccumulator>();

            if health.current <= fixed_math::FIXED_ZERO {
                // À terre (T1.3) plutôt que mort : seulement un joueur, et seulement s'il
                // reste au moins un AUTRE joueur debout pour le réanimer (`PlayersCount > 1`
                // et un autre handle que le sien dans `standing_handles`) ; sinon (seul, ou
                // tous les autres déjà à terre/morts) il meurt comme avant T1.3. Voir la doc
                // de `combat::downed::Downed`.
                let other_standing = opt_player
                    .is_some_and(|player| standing_handles.iter().any(|&h| h != player.handle));

                if players_count.0 > 1 && other_standing {
                    let player = opt_player.expect("other_standing implique un Player");
                    let config = opt_config_handles.and_then(|h| character_configs.get(&h.config));
                    let bleedout_frames =
                        config.map_or(FALLBACK_BLEEDOUT_FRAMES, |c| c.bleedout_frames);
                    let downed_speed_mult =
                        config.map_or_else(|| fixed_math::new(0.3), |c| c.downed_speed_mult);

                    // Gelée à exactement 0 (pas laissée à la valeur négative du
                    // `saturating_sub` ci-dessus) : l'invariant `sante_bornee`
                    // (`crates/scenario/src/invariants.rs`) exige `0 <= current <= max` à
                    // CHAQUE frame simulée, y compris pour un joueur à terre dont `Health`
                    // « n'est plus touchée » ensuite (elle ne doit donc jamais y entrer
                    // négative).
                    health.current = fixed_math::FIXED_ZERO;

                    commands.entity(entity).insert((
                        Downed {
                            since_frame: frame.frame,
                            bleedout_at_frame: frame.frame + bleedout_frames,
                        },
                        Interactable {
                            interaction_range: combat::downed::revive_range(),
                            interaction_type: InteractionType::Revive,
                        },
                    ));

                    if let Some(mut modifiers) = opt_modifiers {
                        modifiers.push_from(
                            downed_modifier_source(),
                            StatId::MoveSpeed,
                            ModifierOp::Mul,
                            downed_speed_mult,
                            None,
                        );
                    }

                    info!(
                        "{} downed (player {}, bleedout at frame {})",
                        g_id,
                        player.handle,
                        frame.frame + bleedout_frames
                    );
                } else {
                    // T2.3, chantier C5 v1 : kill (entité qui meurt directement, jamais un
                    // joueur — `opt_player.is_some()` implique `other_standing` géré plus
                    // haut) attribué au dernier joueur à avoir touché, comme CoD (le tireur
                    // du coup fatal, pas qui a le plus tapé dedans).
                    if opt_player.is_none() {
                        if let Some(handle) = accumulator.last_hit_by.as_ref().and_then(|hits| {
                            hits.iter().find_map(|hit_by| match hit_by {
                                HitBy::Player(handle) => Some(*handle),
                                HitBy::Entity(_) => None,
                            })
                        }) {
                            points_credits.send(PointsCredit::Kill { handle });
                        }
                    }
                    commands.entity(entity).insert(Death {
                        last_hit_by: accumulator.last_hit_by.clone(),
                    });
                }
            }
        }
    }
}

/// À terre (T1.3) : joueur non réanimé à temps (`Downed::bleedout_at_frame` atteint) → meurt
/// de saignement (`Death`). `last_hit_by: None` : la cause du coup fatal d'origine n'est pas
/// gardée sur `Downed` (voir sa doc) — cohérent avec `Death::last_hit_by` déjà optionnel pour
/// une cause inconnue (voir son `Display`). Retire aussi `Reviving`, `Interactable` et le
/// modificateur de vitesse « downed » (T1.2, `Modifiers::remove_by_source`) : cohérent avec
/// une réanimation complète, même si l'entité est de toute façon détruite juste après
/// (`rollback_apply_death`, plus tard dans `DeathManagement`).
///
/// `RollbackSystemSet::DeathManagement`, après `rollback_apply_accumulated_damage` (un
/// joueur ne peut pas tomber à terre et saigner à mort la même frame : `bleedout_frames`
/// est borné `> 0` par `content::lint`, donc `bleedout_at_frame > since_frame` toujours),
/// avant `rollback_check_defeat` et `rollback_apply_death`.
pub fn rollback_apply_bleedout(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut query: Query<(&GgrsNetId, Entity, &Downed, Option<&mut Modifiers>), With<Rollback>>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "bleedout");
    let _enter = system_span.enter();

    for (g_id, entity, downed, opt_modifiers) in order_mut_iter!(query) {
        if frame.frame < downed.bleedout_at_frame {
            continue;
        }

        info!(
            "{} bled out (downed since frame {})",
            g_id, downed.since_frame
        );

        if let Some(mut modifiers) = opt_modifiers {
            modifiers.remove_by_source(&downed_modifier_source());
        }

        commands
            .entity(entity)
            .remove::<Downed>()
            .remove::<Reviving>()
            .remove::<Interactable>()
            .insert(Death { last_hit_by: None });
    }
}

/// À terre (T1.3) : dès que tous les joueurs actuellement en jeu sont à terre ou sur le
/// point de mourir cette même frame (`Death` déjà posé, entité pas encore détruite — voir
/// `RollbackSystemSet::DeathManagement`), fige `Run.step` en `Ended { outcome: Defeat, .. }`
/// une bonne fois pour toutes (`if !run.is_playing() { return; }`, T2.4 : ne réécrit jamais
/// une issue déjà posée — ni par une défaite déjà constatée, ni par la victoire du mode
/// (`crate::run_state::check_run_victory_system`, plus tard dans la frame), ni par un
/// abandon vers le lobby).
///
/// Aucun joueur en jeu (`Player` déjà tous détruits, ex. un run solo où le seul joueur est
/// mort il y a plusieurs frames) : pas de défaite déclenchée ici (`any` reste faux) — elle a
/// déjà été posée la frame où ce dernier joueur est mort.
///
/// Doit tourner après les systèmes qui posent `Downed`/`Death` cette frame
/// (`rollback_apply_accumulated_damage`, `rollback_apply_bleedout`) et avant
/// `rollback_apply_death` (qui détruirait les entités `Death`, les sortant de la query
/// `Player` — même contrainte que `waves::systems::wave_enemy_death_tracking_system`, voir
/// sa doc).
pub fn rollback_check_defeat(
    frame: Res<FrameCount>,
    mut run: ResMut<Run>,
    players: Query<(Has<Death>, Has<Downed>), With<Player>>,
) {
    if !run.is_playing() {
        return;
    }

    let statuses: Vec<(bool, bool)> = players.iter().collect();
    let all_down_or_dead =
        !statuses.is_empty() && statuses.iter().all(|(dead, downed)| *dead || *downed);

    if all_down_or_dead {
        run.step = RunStep::Ended {
            at_frame: frame.frame,
            outcome: RunEnd::Defeat,
        };
        info!("f{} defeat: all players downed or dead", frame.frame);
    }
}

pub fn rollback_apply_death(
    frame: Res<FrameCount>,
    mut commands: Commands,
    query: Query<(&GgrsNetId, Entity, &Death), With<Rollback>>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "apply_death");
    let _enter = system_span.enter();

    for (id, entity, death_info) in order_iter!(query) {
        info!("{} entity killed by {}", id, death_info);

        // Despawn différé : ressuscitable en cas de rollback (voir `RollbackDespawnPlugin`)
        use bevy_ggrs::RollbackDespawnCommandExtension;
        commands.entity(entity).despawn_rollback();
    }
}

// SYSTEM: HEALTH REGENERATION
pub fn rollback_health_regeneration(
    frame: Res<FrameCount>,
    // À terre (T1.3) : `Health` n'est plus touchée, y compris par la regen (sinon un
    // joueur à terre remonterait passivement vers `max` au lieu de saigner) — voir la doc
    // de `combat::downed::Downed`.
    mut query: Query<(&GgrsNetId, &mut Health, &HealthRegen), (With<Rollback>, Without<Downed>)>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "health_regen");
    let _enter = system_span.enter();

    for (g_id, mut health, regen) in order_mut_iter!(query) {
        // Check if enough time has passed since last damage
        let frames_since_damage = frame.frame.saturating_sub(regen.last_damage_frame);

        if frames_since_damage >= regen.regen_delay_frames && health.current < health.max {
            let health_before = health.current;
            // Regenerate health (60 frames per second)
            let regen_per_frame = regen.regen_rate / fixed_math::new(60.0);
            health.current = (health.current + regen_per_frame).min(health.max);

            // Log every 60 frames (once per second) or when reaching max health
            if frame.frame % 60 == 0 || health.current >= health.max {
                info!(
                    "{} regen {} -> {} (+{}/s, {}f since dmg)",
                    g_id, health_before, health.current, regen.regen_rate, frames_since_damage
                );
            }
        }
    }
}

/// Stats branchées (T1.2, chantier B2) : `Health.max` suit la stat `MaxHealth` (base +
/// modificateurs actifs à la frame courante) et `HealthRegen.regen_rate` suit `HealthRegen`.
/// `current` est borné à `max` s'il le dépasse (perte de stat, fin d'un buff) ; jamais
/// relevé automatiquement quand `max` augmente (un soin/regen explicite s'en charge).
///
/// N'écrit rien si l'entité n'a pas la stat correspondante (`StatReader::try_get`, pas
/// `get` avec un défaut) : reprendre la valeur déjà en place comme base de `resolve()` la
/// ferait dériver à chaque frame sous un modificateur multiplicatif (`Mul`/`Pct`), au lieu
/// de la laisser simplement inchangée.
///
/// `RollbackSystemSet::Status`, après `stats::expire_modifiers_system` (voir sa doc : les
/// deux touchent `Modifiers`/l'état qui en dérive, l'`ambiguity_detection: Error` du
/// `GgrsSchedule` exige un ordre explicite entre eux).
pub fn sync_health_from_stats(
    stats: StatReader,
    mut query: Query<(&GgrsNetId, Entity, &mut Health, Option<&mut HealthRegen>), With<Rollback>>,
) {
    for (_net_id, entity, mut health, regen) in order_mut_iter!(query) {
        if let Some(max) = stats.try_get(entity, &StatId::MaxHealth) {
            health.max = max;
            if health.current > max {
                health.current = max;
            }
        }
        if let Some(mut regen) = regen {
            if let Some(rate) = stats.try_get(entity, &StatId::HealthRegen) {
                regen.regen_rate = rate;
            }
        }
    }
}

pub use combat::actors::Health;
