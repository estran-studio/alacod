//! Statuts (B3, T1.3, `docs/conventions.md` §19) : `Burn`, `Slow`, `Stun`, `Freeze`.
//!
//! Un statut du contenu (kind `Status`, `statuses/<id>.ron`) est un [`StatusSpec`] : son genre
//! ([`StatusDef`]) et ses paramètres. Les définitions du jeu forment la [`StatusLibrary`]
//! (hors rollback, résolue au lancement comme la `PatternLibrary`). Une entité qui porte des
//! statuts a [`Statuses`] (rollback, checksum **neutre** : aucun personnage n'en porte sans
//! statut posé, les traces existantes ne bougent pas).
//!
//! Règles (pures, testées ici) :
//! - `Burn` : un tick de `damage` toutes les `period` frames depuis la pose ; réapplication =
//!   durée restante + durée de base, plafonnée à 2 × la base ; `stacks` = applications
//!   actives, au plus 2.
//! - `Slow` : vitesse × `factor` (modificateurs posés par l'application) ; réapplication
//!   rafraîchit la durée.
//! - `Stun` : ni déplacement ni tir ; réapplication rafraîchit.
//! - `Freeze` : comme `Stun`, plus vitesse et recul remis à zéro ; réapplication rafraîchit.
//!
//! Un statut expire à la frame `expires_at_frame` (retiré par [`Statuses::tick`]).
use std::collections::BTreeMap;

use bevy::prelude::{Component, Resource};
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::team::Team;
use utils::net_id::GgrsNetId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum StatusDef {
    Burn,
    Slow,
    Stun,
    Freeze,
}

/// Plafond des piles de `Burn` (et de sa durée : 2 × la base).
pub const BURN_MAX_STACKS: u32 = 2;

/// Définition d'un statut du contenu (`statuses/<id>.ron`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StatusSpec {
    pub kind: StatusDef,
    /// Durée de base, en frames.
    pub frames: u32,
    /// `Burn` : dégât par tick.
    pub damage: Fixed,
    /// `Burn` : frames entre deux ticks.
    pub period: u32,
    /// `Slow` : facteur de vitesse (`]0, 1]`).
    pub factor: Fixed,
}

/// Statuts du jeu par id (hors rollback, identique sur tous les clients).
#[derive(Resource, Clone, Debug, Default)]
pub struct StatusLibrary {
    pub statuses: BTreeMap<String, StatusSpec>,
}

/// Une entrée posée. Les trois premiers champs sont le contrat de T1.0a ; les autres
/// (T1.3) ont été ajoutés après.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct StatusEntry {
    pub status: StatusDef,
    pub stacks: u32,
    pub expires_at_frame: u32,
    /// Id du statut du contenu (`statuses/<id>.ron`).
    pub id: String,
    /// Source de la dernière application (dégâts de `Burn` crédités à elle) et son équipe
    /// (relevée à la pose : la source peut avoir disparu au moment du tick).
    pub source: Option<GgrsNetId>,
    pub source_team: Option<Team>,
    /// `Burn` : frame du prochain tick.
    pub next_tick_frame: u32,
}

/// Statuts portés, dans l'ordre de pose, sans état caché.
#[derive(Component, Default, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Statuses(pub Vec<StatusEntry>);

/// Un tick de `Burn` dû cette frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnTick {
    pub id: String,
    pub source: Option<GgrsNetId>,
    pub source_team: Option<Team>,
    pub damage: Fixed,
}

impl Statuses {
    /// Pose (ou réapplique) le statut `id` à la frame `frame` (voir les règles du module).
    pub fn apply(
        &mut self,
        id: &str,
        spec: &StatusSpec,
        source: Option<(GgrsNetId, Option<Team>)>,
        frame: u32,
        stacks: u32,
    ) {
        let stacks = stacks.max(1);
        let (source, source_team) = match source {
            Some((id, team)) => (Some(id), team),
            None => (None, None),
        };
        if let Some(entry) = self.0.iter_mut().find(|e| e.id == id) {
            entry.source = source;
            entry.source_team = source_team;
            match spec.kind {
                StatusDef::Burn => {
                    let remaining = entry.expires_at_frame.saturating_sub(frame);
                    let extended = remaining
                        .saturating_add(spec.frames.saturating_mul(stacks))
                        .min(spec.frames.saturating_mul(BURN_MAX_STACKS));
                    entry.expires_at_frame = frame.saturating_add(extended);
                    entry.stacks = entry.stacks.saturating_add(stacks).min(BURN_MAX_STACKS);
                }
                StatusDef::Slow | StatusDef::Stun | StatusDef::Freeze => {
                    entry.expires_at_frame = frame.saturating_add(spec.frames);
                }
            }
            return;
        }
        let (duration, stacks) = match spec.kind {
            StatusDef::Burn => {
                let stacks = stacks.min(BURN_MAX_STACKS);
                (spec.frames.saturating_mul(stacks), stacks)
            }
            _ => (spec.frames, 1),
        };
        self.0.push(StatusEntry {
            status: spec.kind,
            stacks,
            expires_at_frame: frame.saturating_add(duration),
            id: id.to_string(),
            source,
            source_team,
            next_tick_frame: frame.saturating_add(spec.period.max(1)),
        });
    }

    /// Ticks de `Burn` dus à `frame` (dans l'ordre de pose), puis retrait des statuts
    /// expirés (`expires_at_frame <= frame`). Un tick tombant à l'expiration compte.
    pub fn tick(&mut self, frame: u32, library: &StatusLibrary) -> Vec<BurnTick> {
        let mut ticks = Vec::new();
        for entry in &mut self.0 {
            if entry.status != StatusDef::Burn {
                continue;
            }
            let Some(spec) = library.statuses.get(&entry.id) else {
                continue;
            };
            if frame >= entry.next_tick_frame && entry.next_tick_frame <= entry.expires_at_frame {
                ticks.push(BurnTick {
                    id: entry.id.clone(),
                    source: entry.source.clone(),
                    source_team: entry.source_team,
                    damage: spec.damage,
                });
                entry.next_tick_frame = entry.next_tick_frame.saturating_add(spec.period.max(1));
            }
        }
        self.0.retain(|entry| entry.expires_at_frame > frame);
        ticks
    }

    pub fn has(&self, id: &str) -> bool {
        self.0.iter().any(|e| e.id == id)
    }

    /// Piles du statut `id` (0 s'il n'est pas porté).
    pub fn stacks(&self, id: &str) -> u32 {
        self.0.iter().find(|e| e.id == id).map_or(0, |e| e.stacks)
    }

    /// `Stun` ou `Freeze` : ni déplacement ni tir.
    pub fn incapacitated(&self) -> bool {
        self.0
            .iter()
            .any(|e| matches!(e.status, StatusDef::Stun | StatusDef::Freeze))
    }

    /// `Freeze` : vitesse et recul remis à zéro.
    pub fn frozen(&self) -> bool {
        self.0.iter().any(|e| e.status == StatusDef::Freeze)
    }
}

/// `Stun`/`Freeze` sur une entité qui porte peut-être des statuts.
pub fn incapacitated(statuses: Option<&Statuses>) -> bool {
    statuses.is_some_and(Statuses::incapacitated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn spec(kind: StatusDef) -> StatusSpec {
        StatusSpec {
            kind,
            frames: 120,
            damage: fx(4.0),
            period: 30,
            factor: fx(0.5),
        }
    }

    fn library() -> StatusLibrary {
        let mut library = StatusLibrary::default();
        library.statuses.insert("brulure".into(), spec(StatusDef::Burn));
        library.statuses.insert("lenteur".into(), spec(StatusDef::Slow));
        library.statuses.insert("etourdi".into(), spec(StatusDef::Stun));
        library.statuses.insert("gel".into(), spec(StatusDef::Freeze));
        library
    }

    fn net(id: usize) -> GgrsNetId {
        GgrsNetId(id as _, "t".into())
    }

    #[test]
    fn status_def_ron_round_trip() {
        let values = vec![
            StatusDef::Burn,
            StatusDef::Slow,
            StatusDef::Stun,
            StatusDef::Freeze,
        ];
        assert_eq!(
            ron::from_str::<Vec<StatusDef>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }

    #[test]
    fn burn_tique_puis_expire() {
        let lib = library();
        let mut s = Statuses::default();
        s.apply("brulure", &lib.statuses["brulure"], Some((net(7), None)), 100, 1);
        assert_eq!(s.0[0].expires_at_frame, 220);
        let mut ticks = Vec::new();
        for frame in 101..=220 {
            for tick in s.tick(frame, &lib) {
                ticks.push((frame, tick));
            }
        }
        let frames: Vec<u32> = ticks.iter().map(|(f, _)| *f).collect();
        assert_eq!(frames, [130, 160, 190, 220], "un tick toutes les 30 frames, 4 en 120");
        assert_eq!(ticks[0].1.source, Some(net(7)), "crédité à la source");
        assert_eq!(ticks[0].1.damage, fx(4.0));
        assert!(!s.has("brulure"), "expiré à f220");
    }

    #[test]
    fn burn_empile_la_duree_au_plafond() {
        let lib = library();
        let mut s = Statuses::default();
        s.apply("brulure", &lib.statuses["brulure"], Some((net(1), None)), 100, 1);
        // À f160 : reste 60, + 120 = 180 (sous le plafond 240)
        s.apply("brulure", &lib.statuses["brulure"], Some((net(2), None)), 160, 1);
        assert_eq!(s.0[0].expires_at_frame, 340);
        assert_eq!(s.stacks("brulure"), 2);
        assert_eq!(s.0[0].source, Some(net(2)), "dernière source");
        // Troisième pose à f170 : 170 + 120 = 290 → plafond 240, piles plafonnées à 2
        s.apply("brulure", &lib.statuses["brulure"], Some((net(2), None)), 170, 1);
        assert_eq!(s.0[0].expires_at_frame, 410);
        assert_eq!(s.stacks("brulure"), 2);
        assert_eq!(s.0.len(), 1);
    }

    #[test]
    fn slow_stun_freeze_rafraichissent() {
        let lib = library();
        for id in ["lenteur", "etourdi", "gel"] {
            let mut s = Statuses::default();
            s.apply(id, &lib.statuses[id], None, 100, 1);
            s.apply(id, &lib.statuses[id], None, 150, 3);
            assert_eq!(s.0[0].expires_at_frame, 270, "{id} : rafraîchi depuis f150");
            assert_eq!(s.stacks(id), 1, "{id} : une pile");
            assert!(s.tick(269, &lib).is_empty());
            assert!(s.has(id));
            s.tick(270, &lib);
            assert!(!s.has(id), "{id} : expiré");
        }
    }

    #[test]
    fn incapacite_et_gel() {
        let lib = library();
        let mut s = Statuses::default();
        s.apply("lenteur", &lib.statuses["lenteur"], None, 0, 1);
        assert!(!s.incapacitated() && !s.frozen());
        s.apply("etourdi", &lib.statuses["etourdi"], None, 0, 1);
        assert!(s.incapacitated() && !s.frozen());
        let mut g = Statuses::default();
        g.apply("gel", &lib.statuses["gel"], None, 0, 1);
        assert!(g.incapacitated() && g.frozen());
        assert!(!incapacitated(None));
    }
}
