//! Fenêtre de jugement générique (X-3, verbes D1) : note un instant par rapport à
//! un repère en frames entières (`parfait` / `bien` / `raté`).
//!
//! Règle pure, sans ECS : le repère est une frame (événement d'horloge, début d'une
//! action comme un coup de mêlée, coup qui arrive), l'instant noté une autre frame.
//! Les durées sont en frames entières à 60 Hz (4 frames ≈ 60 ms, 7 frames ≈ 120 ms :
//! l'arrondi est au contenu de choisir ses fenêtres, jamais à ce module).
//! Tout en entiers `u32` (différence saturante), jamais de `f32` (CLAUDE.md, règle 1).

/// Note d'une fenêtre de jugement.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimingGrade {
    /// Dans la fenêtre parfaite (`écart <= perfect_frames`).
    Perfect,
    /// Hors fenêtre parfaite, dans la fenêtre large (`écart <= good_frames`).
    Good,
    /// Hors des deux fenêtres.
    Miss,
}

/// Note `instant` par rapport au repère `reference` : `écart = |instant - reference|`
/// (différence saturante) ; `Perfect` si `écart <= perfect_frames`, sinon `Good` si
/// `écart <= good_frames`, sinon `Miss`. Exige `perfect_frames <= good_frames`
/// (le lint le garantit côté contenu ; ici un `perfect_frames` plus grand que
/// `good_frames` se comporte comme `good_frames`).
pub fn grade_timing(
    instant: u32,
    reference: u32,
    perfect_frames: u32,
    good_frames: u32,
) -> TimingGrade {
    let gap = instant.abs_diff(reference);
    if gap <= perfect_frames.min(good_frames) {
        TimingGrade::Perfect
    } else if gap <= good_frames {
        TimingGrade::Good
    } else {
        TimingGrade::Miss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecart_nul_est_parfait() {
        assert_eq!(grade_timing(60, 60, 4, 7), TimingGrade::Perfect);
    }

    #[test]
    fn bornes_incluses() {
        assert_eq!(grade_timing(64, 60, 4, 7), TimingGrade::Perfect);
        assert_eq!(grade_timing(65, 60, 4, 7), TimingGrade::Good);
        assert_eq!(grade_timing(67, 60, 4, 7), TimingGrade::Good);
        assert_eq!(grade_timing(68, 60, 4, 7), TimingGrade::Miss);
    }

    #[test]
    fn symetrique_avant_apres() {
        assert_eq!(grade_timing(57, 60, 4, 7), TimingGrade::Perfect);
        assert_eq!(grade_timing(53, 60, 4, 7), TimingGrade::Good);
        assert_eq!(grade_timing(52, 60, 4, 7), TimingGrade::Miss);
    }

    #[test]
    fn fenetre_parfaite_nulle() {
        assert_eq!(grade_timing(60, 60, 0, 7), TimingGrade::Perfect);
        assert_eq!(grade_timing(61, 60, 0, 7), TimingGrade::Good);
    }

    #[test]
    fn sature_sans_panique() {
        assert_eq!(grade_timing(u32::MAX, 0, 4, 7), TimingGrade::Miss);
        assert_eq!(grade_timing(0, u32::MAX, 4, 7), TimingGrade::Miss);
    }
}
