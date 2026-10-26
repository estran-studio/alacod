//! Directions dessinées d'une planche de sprites (X-4a, `docs/conventions.md` §89).
//!
//! **Présentation seulement** : rien ici n'est un composant, rien n'est rollback, la simulation ne
//! lit jamais ce module. [`FacingDirection`] (rollback) reste la seule orientation de la
//! simulation ; une planche déclare combien de directions elle dessine et dans quel ordre, et
//! [`SheetDirections::choose`] dit quelle **ligne** afficher et s'il faut la retourner. Le module
//! vit dans `sim_core` (à côté de `FacingDirection`) parce que le lint de contenu (`content`, sans
//! crate de rendu) lit la même déclaration que `animation`.
//!
//! # Déclaration (champ `directions` d'une `SpriteSheetConfig`, absent = comportement d'avant)
//!
//! ```ron
//! directions: Some((
//!     rows: [E, NE, N, NW, W, SW, S, SE], // direction dessinée par ligne, dans l'ordre des lignes
//!     mirror: false,                      // vrai : le côté non dessiné est la ligne retournée
//! )),
//! ```
//!
//! | Planche | `rows` | `mirror` |
//! |---|---|---|
//! | une orientation, retournée pour la gauche (clones, **défaut**) | `[E]` | `true` |
//! | quatre directions | `[E, N, W, S]` | `false` |
//! | cinq directions, l'autre côté en miroir | `[S, SE, E, NE, N]` | `true` |
//! | huit directions, ordre `angle / 45°` | `[E, NE, N, NW, W, SW, S, SE]` | `false` |
//!
//! # Règle de choix
//!
//! Chaque ligne dessinée est un candidat à sa direction ; avec `mirror`, la **réflexion
//! horizontale** de chaque ligne (`E↔W`, `NE↔NW`, `SE↔SW`, `N` et `S` à elles-mêmes) en est un
//! autre, à retourner. On prend la plus petite distance angulaire à la direction demandée ; à
//! égalité, la ligne non retournée, puis la ligne d'indice le plus petit. Pour `[E]` + `mirror`,
//! le résultat est exactement celui d'avant X-4a (`FacingDirection::should_flip_x`).

use serde::{Deserialize, Serialize};

use crate::animation_state::FacingDirection;

/// Une des huit directions, nommée par le compas (`E` = vers la droite, `N` = vers le haut de
/// l'écran). L'indice est `angle / 45°`, comme `FacingDirection` : c'est l'ordre des lignes d'une
/// planche à huit directions.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Compass {
    E,
    NE,
    N,
    NW,
    W,
    SW,
    S,
    SE,
}

impl Compass {
    /// Les huit directions dans l'ordre des angles croissants (`angle / 45°`).
    pub const EIGHT: [Compass; 8] = [
        Compass::E,
        Compass::NE,
        Compass::N,
        Compass::NW,
        Compass::W,
        Compass::SW,
        Compass::S,
        Compass::SE,
    ];

    /// `angle / 45°`, de 0 (`E`) à 7 (`SE`).
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_facing(facing: FacingDirection) -> Self {
        match facing {
            FacingDirection::Right => Compass::E,
            FacingDirection::UpRight => Compass::NE,
            FacingDirection::Up => Compass::N,
            FacingDirection::UpLeft => Compass::NW,
            FacingDirection::Left => Compass::W,
            FacingDirection::DownLeft => Compass::SW,
            FacingDirection::Down => Compass::S,
            FacingDirection::DownRight => Compass::SE,
        }
    }

    /// Réflexion par rapport à l'axe vertical : `E↔W`, `NE↔NW`, `SE↔SW`, `N` et `S` inchangées.
    pub fn mirrored(self) -> Self {
        Compass::EIGHT[((12 - self.index()) % 8) as usize]
    }

    /// Distance angulaire en pas de 45° (0 à 4).
    pub fn distance(self, other: Compass) -> u8 {
        let d = self.index().abs_diff(other.index());
        d.min(8 - d)
    }
}

/// Ligne à afficher pour une direction, et retournement horizontal de l'image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowChoice {
    pub row: usize,
    pub flip_x: bool,
}

/// Directions dessinées d'une planche : la direction de chaque ligne, dans l'ordre des lignes.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetDirections {
    /// `rows[i]` : direction dessinée à la ligne `i` du bloc de directions.
    pub rows: Vec<Compass>,
    /// Le côté non dessiné s'obtient en retournant la ligne de la direction réfléchie.
    #[serde(default)]
    pub mirror: bool,
}

impl SheetDirections {
    /// Une orientation tournée vers la droite, retournée pour la gauche : le comportement de
    /// toute planche sans déclaration.
    pub fn single_mirrored() -> Self {
        Self {
            rows: vec![Compass::E],
            mirror: true,
        }
    }

    /// Huit lignes dans l'ordre `E, NE, N, NW, W, SW, S, SE`, sans miroir.
    pub fn eight() -> Self {
        Self {
            rows: Compass::EIGHT.to_vec(),
            mirror: false,
        }
    }

    /// Ligne et retournement pour `facing`. Sans ligne (déclaration refusée par le lint) : ligne 0.
    pub fn choose(&self, facing: FacingDirection) -> RowChoice {
        let wanted = Compass::from_facing(facing);
        let mut best: Option<((u8, bool, usize), RowChoice)> = None;
        for (row, drawn) in self.rows.iter().enumerate() {
            let plain = (wanted.distance(*drawn), false, row);
            if best.as_ref().is_none_or(|(k, _)| plain < *k) {
                best = Some((plain, RowChoice { row, flip_x: false }));
            }
            if self.mirror {
                let flipped = (wanted.distance(drawn.mirrored()), true, row);
                if best.as_ref().is_none_or(|(k, _)| flipped < *k) {
                    best = Some((flipped, RowChoice { row, flip_x: true }));
                }
            }
        }
        best.map(|(_, c)| c).unwrap_or(RowChoice {
            row: 0,
            flip_x: false,
        })
    }

    /// Incohérences avec une planche de `sheet_rows` lignes (lint) : déclaration vide, direction
    /// répétée, plus de directions que de lignes, lignes qui ne sont pas un multiple du nombre de
    /// directions (la planche est faite de blocs de directions, un par animation).
    pub fn problems(&self, sheet_rows: Option<u32>) -> Vec<String> {
        let mut out = Vec::new();
        if self.rows.is_empty() {
            out.push("`directions.rows` est vide : au moins une direction".to_string());
            return out;
        }
        for (i, d) in self.rows.iter().enumerate() {
            if self.rows[..i].contains(d) {
                out.push(format!(
                    "`directions.rows` : direction {d:?} répétée (ligne {i})"
                ));
            }
        }
        if let Some(rows) = sheet_rows {
            let n = self.rows.len() as u32;
            if rows < n {
                out.push(format!(
                    "la planche a {rows} ligne(s) pour {n} direction(s) déclarée(s) : `rows` de la planche doit être un multiple de {n}"
                ));
            } else if rows % n != 0 {
                out.push(format!(
                    "la planche a {rows} ligne(s), pas un multiple des {n} direction(s) déclarée(s)"
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use FacingDirection::*;

    const ALL: [FacingDirection; 8] = [Right, UpRight, Up, UpLeft, Left, DownLeft, Down, DownRight];

    fn pick(d: &SheetDirections, f: FacingDirection) -> (usize, bool) {
        let c = d.choose(f);
        (c.row, c.flip_x)
    }

    #[test]
    fn single_mirrored_is_the_pre_x4a_behaviour() {
        let d = SheetDirections::single_mirrored();
        for f in ALL {
            assert_eq!(pick(&d, f), (0, f.should_flip_x()), "{f:?}");
        }
    }

    #[test]
    fn eight_rows_pick_their_own_row_in_angle_order() {
        let d = SheetDirections::eight();
        for (i, f) in ALL.into_iter().enumerate() {
            assert_eq!(pick(&d, f), (i, false), "{f:?}");
        }
    }

    #[test]
    fn four_directions_snap_diagonals_to_the_first_row_in_order() {
        // Lignes E, N, W, S ; une diagonale est à 45° de deux lignes : la plus petite ligne gagne.
        let d = SheetDirections {
            rows: vec![Compass::E, Compass::N, Compass::W, Compass::S],
            mirror: false,
        };
        assert_eq!(pick(&d, Right), (0, false));
        assert_eq!(pick(&d, Up), (1, false));
        assert_eq!(pick(&d, Left), (2, false));
        assert_eq!(pick(&d, Down), (3, false));
        assert_eq!(pick(&d, UpRight), (0, false)); // E (0) avant N (1)
        assert_eq!(pick(&d, UpLeft), (1, false)); // N (1) avant W (2)
        assert_eq!(pick(&d, DownLeft), (2, false)); // W (2) avant S (3)
        assert_eq!(pick(&d, DownRight), (0, false)); // E (0) avant S (3)
    }

    #[test]
    fn four_directions_never_flip_without_mirror() {
        let d = SheetDirections {
            rows: vec![Compass::S, Compass::E, Compass::N, Compass::W],
            mirror: false,
        };
        for f in ALL {
            assert!(!d.choose(f).flip_x, "{f:?}");
        }
        // L'ordre des lignes est celui de la déclaration, pas celui du compas.
        assert_eq!(pick(&d, Down), (0, false));
        assert_eq!(pick(&d, Left), (3, false));
    }

    #[test]
    fn five_directions_mirror_the_other_side() {
        // S, SE, E, NE, N dessinées ; SW, W, NW viennent du miroir de SE, E, NE.
        let d = SheetDirections {
            rows: vec![Compass::S, Compass::SE, Compass::E, Compass::NE, Compass::N],
            mirror: true,
        };
        assert_eq!(pick(&d, Down), (0, false));
        assert_eq!(pick(&d, DownRight), (1, false));
        assert_eq!(pick(&d, Right), (2, false));
        assert_eq!(pick(&d, UpRight), (3, false));
        assert_eq!(pick(&d, Up), (4, false));
        assert_eq!(pick(&d, DownLeft), (1, true));
        assert_eq!(pick(&d, Left), (2, true));
        assert_eq!(pick(&d, UpLeft), (3, true));
    }

    #[test]
    fn mirror_tie_prefers_the_unflipped_row() {
        // `N` est sa propre réflexion : jamais retournée quand elle est dessinée.
        let d = SheetDirections {
            rows: vec![Compass::N, Compass::E],
            mirror: true,
        };
        assert_eq!(pick(&d, Up), (0, false));
    }

    #[test]
    fn mirrored_is_an_involution_on_the_compass() {
        for c in Compass::EIGHT {
            assert_eq!(c.mirrored().mirrored(), c);
        }
        assert_eq!(Compass::E.mirrored(), Compass::W);
        assert_eq!(Compass::NE.mirrored(), Compass::NW);
        assert_eq!(Compass::SE.mirrored(), Compass::SW);
        assert_eq!(Compass::N.mirrored(), Compass::N);
        assert_eq!(Compass::S.mirrored(), Compass::S);
    }

    #[test]
    fn compass_index_matches_facing_angle_over_45_degrees() {
        for (i, f) in ALL.into_iter().enumerate() {
            assert_eq!(Compass::from_facing(f).index() as usize, i);
            assert_eq!(Compass::EIGHT[i], Compass::from_facing(f));
        }
    }

    #[test]
    fn problems_cover_empty_duplicates_and_row_counts() {
        assert!(SheetDirections::eight().problems(Some(8)).is_empty());
        assert!(SheetDirections::eight().problems(Some(16)).is_empty());
        assert!(SheetDirections::eight().problems(None).is_empty());
        assert_eq!(SheetDirections::eight().problems(Some(5)).len(), 1);
        assert_eq!(SheetDirections::eight().problems(Some(12)).len(), 1);
        let dup = SheetDirections {
            rows: vec![Compass::E, Compass::E],
            mirror: false,
        };
        assert_eq!(dup.problems(Some(2)).len(), 1);
        let empty = SheetDirections {
            rows: vec![],
            mirror: true,
        };
        assert_eq!(empty.problems(Some(1)).len(), 1);
    }

    #[test]
    fn ron_declaration_round_trips_and_refuses_unknown_names() {
        let ok: SheetDirections = ron::from_str("(rows: [E, NE, N, NW, W, SW, S, SE])").unwrap();
        assert_eq!(ok, SheetDirections::eight());
        let mirrored: SheetDirections = ron::from_str("(rows: [E], mirror: true)").unwrap();
        assert_eq!(mirrored, SheetDirections::single_mirrored());
        assert!(ron::from_str::<SheetDirections>("(rows: [East])").is_err());
        assert!(ron::from_str::<SheetDirections>("(rows: [E], mirrored: true)").is_err());
    }
}
