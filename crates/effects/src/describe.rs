//! Description en français d'un effet (T1.16, `docs/conventions.md` §30) : le texte des cartes
//! de l'écran de mutation, dérivé des effets eux-mêmes (aucun champ de contenu en plus). Pure,
//! testée ; une phrase par effet : « <déclencheur>[, <conditions>] : <actions>. ».

use bevy_fixed::fixed_math::Fixed;
use sim_core::modifier::ModifierOp;
use sim_core::stats::StatId;

use crate::{Action, Condition, Effect, GaugeThreshold, On};

/// Nombre en français : virgule décimale, zéros inutiles retirés (`1.5` → `1,5`, `25` → `25`).
pub fn number(value: Fixed) -> String {
    let text = format!("{:.2}", value.to_num::<f64>());
    let text = text.trim_end_matches('0').trim_end_matches('.');
    text.replace('.', ",")
}

/// Durée en secondes d'un nombre de frames (60 par seconde).
fn seconds(frames: u32) -> String {
    number(Fixed::from_num(frames) / Fixed::from_num(60))
}

fn stat_name(stat: &StatId) -> String {
    match stat {
        StatId::MaxHealth => "vie max".into(),
        StatId::HealthRegen => "régénération".into(),
        StatId::MoveSpeed => "vitesse".into(),
        StatId::Acceleration => "accélération".into(),
        StatId::SprintMultiplier => "sprint".into(),
        StatId::Damage => "dégâts".into(),
        StatId::FireRate => "cadence de tir".into(),
        StatId::ReloadSpeed => "vitesse de rechargement".into(),
        StatId::Range => "portée".into(),
        StatId::Armor => "armure".into(),
        StatId::Luck => "chance".into(),
        StatId::Custom(name) => name.clone(),
        other => format!("{other:?}"),
    }
}

fn modifier(stat: &StatId, op: ModifierOp, value: Fixed) -> String {
    let stat = stat_name(stat);
    match op {
        ModifierOp::Add if value >= Fixed::ZERO => format!("+{} {stat}", number(value)),
        ModifierOp::Add => format!("{} {stat}", number(value)),
        ModifierOp::Pct => {
            let pct = number(value * Fixed::from_num(100));
            if value >= Fixed::ZERO {
                format!("+{pct} % de {stat}")
            } else {
                format!("{pct} % de {stat}")
            }
        }
        ModifierOp::Mul => format!("{stat} × {}", number(value)),
        ModifierOp::Set => format!("{stat} à {}", number(value)),
    }
}

/// Le déclencheur, en début de phrase.
pub fn describe_trigger(on: &On) -> String {
    match on {
        On::OnKill => "À chaque ennemi tué".into(),
        On::OnDamageTaken => "Quand vous êtes touché".into(),
        On::OnLevelUp => "À chaque niveau".into(),
        On::Tick(60) => "Chaque seconde".into(),
        On::Tick(frames) => format!("Toutes les {} s", seconds(*frames)),
        On::OnGauge(id, GaugeThreshold::Above(x)) => {
            format!("Quand la jauge {id} dépasse {}", number(*x))
        }
        On::OnHit => "À chaque coup porté".into(),
        On::OnDodge => "À chaque esquive".into(),
        On::OnReload => "À chaque rechargement".into(),
        On::OnRoomClear => "À chaque salle vidée".into(),
        On::OnPickup => "À chaque ramassage".into(),
        On::OnUse => "À l'utilisation".into(),
        On::OnEvent(id) => format!("À l'événement {id}"),
    }
}

/// Une condition, à la suite du déclencheur.
pub fn describe_condition(condition: &Condition) -> String {
    match condition {
        Condition::HpBelow(x) => {
            format!("sous {} % de vie", number(*x * Fixed::from_num(100)))
        }
        Condition::HasTag(tag) => format!("si vous êtes {tag}"),
        Condition::TargetTag(tag) => format!("si la cible est {tag}"),
        Condition::NotHitFor(frames) => {
            format!("sans avoir été touché depuis {} s", seconds(*frames))
        }
        Condition::Carrying(item) => format!("en portant {item}"),
        Condition::SquadSize(n) => format!("à {n} joueurs ou plus"),
        Condition::TargetInRange(range) => format!("cible à moins de {}", number(*range)),
    }
}

/// Une action, après les deux-points.
pub fn describe_action(action: &Action) -> String {
    match action {
        Action::Heal(amount) => format!("regagnez {} points de vie", number(*amount)),
        Action::Modifier { stat, op, value } => modifier(stat, *op, *value),
        Action::TimedModifier {
            stat,
            op,
            value,
            frames,
        } => format!(
            "{} pendant {} s",
            modifier(stat, *op, *value),
            seconds(*frames)
        ),
        Action::SpawnPattern { pattern, .. } => format!("une salve part de vous ({pattern})"),
        Action::GaugeAdd(id, amount) => format!("+{} {id}", number(*amount)),
        Action::ApplyStatus { status, stacks } if *stacks > 1 => {
            format!("pose {status} (×{stacks})")
        }
        Action::ApplyStatus { status, .. } => format!("pose {status}"),
        Action::CurrencyMultiplier { factor, frames } => {
            format!(
                "points × {} pendant {} s",
                number(*factor),
                seconds(*frames)
            )
        }
        Action::RefillAmmo => "munitions rechargées".into(),
        Action::RepairAllWindows => "fenêtres réparées".into(),
        Action::KillAllWaveEnemies => "tous les ennemis de la vague tués".into(),
        Action::DestroyTerrain { radius } => format!("creuse la roche ({})", number(*radius)),
    }
}

/// Une phrase par effet : « <déclencheur>[, <conditions>] : <actions>. ».
pub fn describe_effect(effect: &Effect) -> String {
    let mut text = describe_trigger(&effect.on);
    for condition in &effect.r#if {
        text.push_str(", ");
        text.push_str(&describe_condition(condition));
    }
    let actions: Vec<String> = effect.r#do.iter().map(describe_action).collect();
    format!("{text} : {}.", actions.join(", "))
}

/// Les effets d'une mutation, une phrase chacun, séparés par une espace.
pub fn describe_effects(effects: &[Effect]) -> String {
    effects
        .iter()
        .map(describe_effect)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    #[test]
    fn nombres_a_la_francaise() {
        assert_eq!(number(fx(25.0)), "25");
        assert_eq!(number(fx(1.5)), "1,5");
        assert_eq!(number(fx(0.25)), "0,25");
        assert_eq!(seconds(180), "3");
        assert_eq!(seconds(90), "1,5");
    }

    #[test]
    fn mutations_du_throne() {
        let coriace = Effect {
            on: On::OnLevelUp,
            r#if: vec![],
            r#do: vec![
                Action::Modifier {
                    stat: StatId::MaxHealth,
                    op: ModifierOp::Add,
                    value: fx(25.0),
                },
                Action::Heal(fx(25.0)),
            ],
        };
        assert_eq!(
            describe_effect(&coriace),
            "À chaque niveau : +25 vie max, regagnez 25 points de vie."
        );
        let adrenaline = Effect {
            on: On::OnDamageTaken,
            r#if: vec![Condition::HpBelow(fx(0.4))],
            r#do: vec![Action::TimedModifier {
                stat: StatId::MoveSpeed,
                op: ModifierOp::Mul,
                value: fx(1.4),
                frames: 180,
            }],
        };
        assert_eq!(
            describe_effect(&adrenaline),
            "Quand vous êtes touché, sous 40 % de vie : vitesse × 1,4 pendant 3 s."
        );
        let sang_froid = Effect {
            on: On::Tick(60),
            r#if: vec![Condition::NotHitFor(300)],
            r#do: vec![Action::Heal(fx(3.0))],
        };
        assert_eq!(
            describe_effect(&sang_froid),
            "Chaque seconde, sans avoir été touché depuis 5 s : regagnez 3 points de vie."
        );
        let irradie = Effect {
            on: On::OnKill,
            r#if: vec![],
            r#do: vec![Action::GaugeAdd("rads".into(), fx(1.0))],
        };
        assert_eq!(describe_effect(&irradie), "À chaque ennemi tué : +1 rads.");
    }
}
