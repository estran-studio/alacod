//! Grammaire d'étage (M2-T10, `docs/conventions.md` §40) : les exigences d'un étage assemblé à
//! partir de gabarits typés (`room_kind`). Donnée pure (RON) ; l'algorithme est dans
//! `map::generation::floor`, le lint dans `content::lint`.

use std::collections::BTreeMap;

use bevy_fixed::fixed_math::Fixed;
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// M3-T0c (§62) : disposition de l'étage. Absent (tout le contenu existant) : `Graph`, le
    /// planificateur en arbre d'aujourd'hui.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<FloorLayout>,
}

/// Disposition d'un étage assemblé (M3-T0c, `docs/conventions.md` §62 ; planificateur : M3-T5, §70).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FloorLayout {
    /// Arbre de gabarits par connexions alignées (M2-T10).
    #[default]
    Graph,
    /// Cases de salle sur une grille (M3-T5, §70) : `cell` = taille d'une case en cases de 16 px
    /// (Q8 : 15 × 9), `max` = étendue maximale de la grille en cases. Un gabarit couvre un
    /// rectangle de cases (1×1, 2×1, 1×2, 2×2…, Q8 : grandes salles dès T5) dont chaque côté de
    /// chaque case a une ouverture centrée.
    Grid {
        cell: (u32, u32),
        max: (u32, u32),
        /// Arêtes supplémentaires entre deux salles voisines (boucles), au plus ce nombre ; 0 :
        /// un arbre, comme Isaac.
        #[serde(default)]
        loops: u32,
        /// Type de la salle secrète (case vide voisine d'au moins deux salles, ouvertures
        /// `door_lock: Secret`), posée seulement si `required` la demande.
        #[serde(default = "default_secret")]
        secret: String,
        /// Salle accrochée au boss (le marché du diable), tirée avec une chance ; absente : aucune.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        post_boss: Option<PostBoss>,
    },
}

/// Salle post-boss d'une disposition en grille (M3-T5) : une case accrochée au boss, tirée avec la
/// probabilité `chance` par un RNG local de la graine de l'emplacement. Quand elle est tirée, elle
/// compte dans les bornes `rooms` (le planificateur réserve sa place).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostBoss {
    /// Type de la salle (gabarit `room_kind`), `diable` par défaut.
    #[serde(default = "default_post_boss")]
    pub kind: String,
    /// Probabilité d'apparition, `Fixed` en chaîne dans [0, 1].
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "[0, 1]")))]
    pub chance: Fixed,
}

fn default_secret() -> String {
    "secrete".to_string()
}
fn default_post_boss() -> String {
    "diable".to_string()
}

impl FloorGrammar {
    /// Disposition effective (`Graph` si absente).
    pub fn layout(&self) -> FloorLayout {
        self.layout.clone().unwrap_or_default()
    }
}

impl FloorGrammar {
    /// La grammaire dont le nombre de salles est multiplié par `mult` (curse `FloorRooms`, M3-T9,
    /// `docs/conventions.md` §74) : les deux bornes de `rooms`, arrondies à l'entier le plus proche
    /// (arithmétique `Fixed`, jamais de flottant : le résultat est celui de tous les clients).
    /// Le résultat reste **satisfiable** : au moins le départ, le boss et les salles requises (et
    /// `min_boss_distance + 1`), au plus la capacité de la grille (`max.0 × max.1`) : une curse ne
    /// peut pas rendre un étage impossible à planifier. `mult` = 1 rend la grammaire inchangée.
    pub fn scaled_rooms(&self, mult: Fixed) -> FloorGrammar {
        let scale = |n: usize| -> usize {
            let scaled = Fixed::saturating_from_num(n).saturating_mul(mult);
            // arrondi au plus proche (`+ 1/2` puis troncature), borné à zéro
            let rounded = (i64::from(scaled.to_bits()) + (1 << 15)) >> 16;
            rounded.max(0) as usize
        };
        let floor = (2 + self.others()).max(self.min_boss_distance + 1);
        let mut low = scale(self.rooms.0).max(floor);
        let mut high = scale(self.rooms.1).max(low);
        if let FloorLayout::Grid { max, .. } = self.layout() {
            let capacity = (max.0 as usize).saturating_mul(max.1 as usize);
            high = high.min(capacity.max(floor));
            low = low.min(high);
        }
        FloorGrammar {
            rooms: (low, high),
            ..self.clone()
        }
    }
}

impl FloorGrammar {
    /// Types de salle dont il faut au moins un gabarit : départ, boss, remplissage, requis et, en
    /// grille, la salle post-boss quand elle a une chance d'apparaître.
    pub fn kinds_needed(&self) -> Vec<&str> {
        let mut kinds = vec![self.start.as_str(), self.boss.as_str()];
        if self.rooms.1 > 2 + self.others() || self.min_boss_distance > 1 {
            kinds.push(self.filler.as_str());
        }
        kinds.extend(self.required.keys().map(String::as_str));
        if let Some(post) = self.post_boss() {
            if post.chance > Fixed::ZERO {
                kinds.push(post.kind.as_str());
            }
        }
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }

    /// Salle post-boss de la disposition en grille, s'il y en a une.
    pub fn post_boss(&self) -> Option<&PostBoss> {
        match self.layout.as_ref()? {
            FloorLayout::Grid { post_boss, .. } => post_boss.as_ref(),
            FloorLayout::Graph => None,
        }
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
            out.push(format!(
                "rooms = ({min}, {max}) : minimum supérieur au maximum"
            ));
        }
        if min < 2 {
            out.push(format!(
                "rooms = ({min}, {max}) : au moins 2 salles (départ et boss)"
            ));
        }
        if self.max_attempts == 0 {
            out.push("max_attempts = 0 : au moins une tentative".to_string());
        }
        if let Some((kind, 0)) = self.required.iter().find(|(_, n)| **n == 0) {
            out.push(format!(
                "required : « {kind} » demandé 0 fois (retirer l'entrée)"
            ));
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
        if let FloorLayout::Grid {
            cell,
            max,
            secret,
            post_boss,
            ..
        } = self.layout()
        {
            if cell.0 == 0 || cell.1 == 0 {
                out.push(format!("layout Grid : cell = {cell:?} : doit être > 0"));
            }
            if max.0 == 0 || max.1 == 0 {
                out.push(format!("layout Grid : max = {max:?} : doit être > 0"));
            }
            let capacity = (max.0 as usize).saturating_mul(max.1 as usize);
            if capacity < min {
                out.push(format!(
                    "layout Grid : max = {max:?} ne contient pas les {min} salles minimum"
                ));
            }
            if [&self.start, &self.boss, &self.filler].contains(&&secret) {
                out.push(format!(
                    "layout Grid : secret « {secret} » est aussi le type du départ, du boss ou du remplissage"
                ));
            }
            if self.required.contains_key(&self.filler) {
                out.push(format!(
                    "required : « {} » est le type de remplissage : une salle requise de ce type se confond avec lui",
                    self.filler
                ));
            }
            if self.required.get(&secret).is_some_and(|n| *n > 1) {
                out.push(format!(
                    "required : une seule salle secrète par étage (« {secret} »)"
                ));
            }
            if let Some(post) = post_boss {
                if post.chance < Fixed::ZERO || post.chance > Fixed::ONE {
                    out.push(format!(
                        "layout Grid : post_boss.chance = {} : hors de [0, 1]",
                        post.chance
                    ));
                }
                if post.kind == secret
                    || [&self.start, &self.boss, &self.filler].contains(&&post.kind)
                    || self.required.contains_key(&post.kind)
                {
                    out.push(format!(
                        "layout Grid : post_boss.kind « {} » est déjà un autre type de la grammaire",
                        post.kind
                    ));
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(secret: &str, post: Option<(&str, f32)>) -> FloorGrammar {
        FloorGrammar {
            rooms: (8, 12),
            required: [("boss", 1), ("tresor", 1), ("secrete", 1)]
                .map(|(k, n)| (k.to_string(), n))
                .into(),
            start: "depart".into(),
            boss: "boss".into(),
            filler: "combat".into(),
            min_boss_distance: 3,
            max_attempts: 64,
            layout: Some(FloorLayout::Grid {
                cell: (15, 9),
                max: (7, 7),
                loops: 1,
                secret: secret.into(),
                post_boss: post.map(|(kind, chance)| PostBoss {
                    kind: kind.into(),
                    chance: Fixed::from_num(chance),
                }),
            }),
        }
    }

    #[test]
    fn scaled_rooms_multiplie_les_bornes_et_reste_satisfiable() {
        let g = grid("secrete", Some(("diable", 0.33)));
        let double = g.scaled_rooms(Fixed::from_num(2));
        assert_eq!(double.rooms, (16, 24));
        assert_eq!(double.problems(), Vec::<String>::new());
        // le reste de la grammaire est intact
        assert_eq!(
            FloorGrammar {
                rooms: g.rooms,
                ..double.clone()
            },
            g
        );
        // ×1 : inchangée, au bit près
        assert_eq!(g.scaled_rooms(Fixed::ONE), g);
        // arrondi au plus proche : 8 × 1,5 = 12, 12 × 1,5 = 18, 8 × 1,06 = 8,48 → 8, 12 × 1,06 → 13
        assert_eq!(g.scaled_rooms(Fixed::from_num(1.5)).rooms, (12, 18));
        assert_eq!(g.scaled_rooms(Fixed::from_num(1.06)).rooms, (8, 13));
        // une réduction ne descend pas sous le départ, le boss et les requises (4 ici) ni sous la
        // distance minimale du boss (3 + 1)
        let tiny = g.scaled_rooms(Fixed::from_num(0.1));
        assert_eq!(tiny.rooms, (4, 4));
        assert_eq!(tiny.problems(), Vec::<String>::new());
        // un multiplicateur énorme est borné par la capacité de la grille (7 × 7)
        let huge = g.scaled_rooms(Fixed::from_num(100));
        assert_eq!(huge.rooms, (49, 49));
    }

    #[test]
    fn une_grille_valide_n_a_aucun_probleme_et_le_post_boss_compte_dans_les_types_necessaires() {
        let g = grid("secrete", Some(("diable", 0.33)));
        assert_eq!(g.problems(), Vec::<String>::new());
        assert!(g.kinds_needed().contains(&"diable"));
        // chance nulle : le type n'est jamais tiré, aucun gabarit n'est exigé
        assert!(!grid("secrete", Some(("diable", 0.0)))
            .kinds_needed()
            .contains(&"diable"));
        assert_eq!(g.post_boss().map(|p| p.kind.as_str()), Some("diable"));
    }

    #[test]
    fn les_types_de_la_grille_ne_se_confondent_pas() {
        let has = |g: &FloorGrammar, needle: &str| g.problems().iter().any(|p| p.contains(needle));
        assert!(has(
            &grid("secrete", Some(("diable", 1.5))),
            "hors de [0, 1]"
        ));
        assert!(has(&grid("secrete", Some(("boss", 0.5))), "post_boss.kind"));
        assert!(has(
            &grid("secrete", Some(("tresor", 0.5))),
            "post_boss.kind"
        ));
        assert!(has(&grid("combat", None), "secret « combat »"));
        let mut twice = grid("secrete", None);
        twice.required.insert("secrete".into(), 2);
        assert!(has(&twice, "une seule salle secrète"));
        let mut filler = grid("secrete", None);
        filler.required.insert("combat".into(), 1);
        assert!(has(&filler, "type de remplissage"));
        // en `Graph`, aucune de ces règles ne s'applique
        let mut graph = grid("secrete", None);
        graph.layout = None;
        graph.required.insert("combat".into(), 1);
        assert!(!has(&graph, "type de remplissage"));
    }
}
