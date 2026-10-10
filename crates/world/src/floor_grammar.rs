//! Grammaire d'étage (M2-T10, `docs/conventions.md` §40) : les exigences d'un étage assemblé à
//! partir de gabarits typés (`room_kind`). Donnée pure (RON) ; l'algorithme est dans
//! `map::generation::floor`, le lint dans `content::lint`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

fn default_start() -> String {
    "depart".to_string()
}
fn default_boss() -> String {
    "boss".to_string()
}
fn default_filler() -> String {
    "combat".to_string()
}
fn default_attempts() -> u32 {
    64
}

/// Exigences d'un étage (RON : fichier du kind de contenu `FloorGrammar`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorGrammar {
    /// Nombre de salles de l'étage, bornes incluses (départ, boss et salles requises comprises).
    pub rooms: (usize, usize),
    /// Salles requises par type, **boss compris** (`"boss": 1, "boutique": 1, "recompense": 1`) ;
    /// le boss est toujours exactement une salle.
    #[serde(default)]
    pub required: BTreeMap<String, usize>,
    /// Type de la salle de départ (gabarit `spawn: true`).
    #[serde(default = "default_start")]
    pub start: String,
    /// Type du boss (une seule connexion, à distance maximale du départ).
    #[serde(default = "default_boss")]
    pub boss: String,
    /// Type des salles de remplissage.
    #[serde(default = "default_filler")]
    pub filler: String,
    /// Distance minimale (en salles) du départ au boss.
    #[serde(default)]
    pub min_boss_distance: usize,
    /// Tentatives au plus avant d'échouer.
    #[serde(default = "default_attempts")]
    pub max_attempts: u32,
}

impl FloorGrammar {
    /// Types de salle dont il faut au moins un gabarit : départ, boss, remplissage, requis.
    pub fn kinds_needed(&self) -> Vec<&str> {
        let mut kinds = vec![self.start.as_str(), self.boss.as_str()];
        if self.rooms.1 > 2 + self.others() || self.min_boss_distance > 1 {
            kinds.push(self.filler.as_str());
        }
        kinds.extend(self.required.keys().map(String::as_str));
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }

    /// Salles requises hors départ et boss.
    pub fn others(&self) -> usize {
        self.required
            .iter()
            .filter(|(kind, _)| **kind != self.boss && **kind != self.start)
            .map(|(_, n)| *n)
            .sum()
    }

    /// Incohérences internes de la grammaire (indépendantes des gabarits) ; vide si elle est
    /// satisfiable sur le papier.
    pub fn problems(&self) -> Vec<String> {
        let (min, max) = self.rooms;
        let mut out = Vec::new();
        if min > max {
            out.push(format!("rooms = ({min}, {max}) : minimum supérieur au maximum"));
        }
        if min < 2 {
            out.push(format!("rooms = ({min}, {max}) : au moins 2 salles (départ et boss)"));
        }
        if self.max_attempts == 0 {
            out.push("max_attempts = 0 : au moins une tentative".to_string());
        }
        if let Some((kind, 0)) = self.required.iter().find(|(_, n)| **n == 0) {
            out.push(format!("required : « {kind} » demandé 0 fois (retirer l'entrée)"));
        }
        if self.required.contains_key(&self.start) {
            out.push(format!(
                "required : « {} » est le type du départ, déjà posé une fois",
                self.start
            ));
        }
        if self.required.get(&self.boss).is_some_and(|n| *n > 1) {
            out.push(format!("required : un seul boss (« {} »)", self.boss));
        }
        let fixed = 2 + self.others();
        if fixed > max {
            out.push(format!(
                "rooms = ({min}, {max}) : départ, boss et salles requises font {fixed} salles"
            ));
        }
        // Le boss est à distance maximale : il faut au moins `min_boss_distance + 1` salles.
        if self.min_boss_distance + 1 > max {
            out.push(format!(
                "min_boss_distance = {} : impossible avec au plus {max} salles",
                self.min_boss_distance
            ));
        }
        out
    }
}
