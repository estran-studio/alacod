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

/// Note un écart **rationnel** de `gap_num / gap_den` frames (signé : avance ou retard) contre des
/// fenêtres en millisecondes (P-1, `docs/conventions.md` §110 : un repère de tempo tombe entre deux
/// frames, une fenêtre de 60 ms fait 3,6 frames). `|écart| × 1000 / 60 <= perfect_ms` ⇔
/// `|gap_num| × 50 <= perfect_ms × 3 × gap_den`, en entiers 128 bits : aucun arrondi, aucune
/// panique. Un `gap_den` nul est traité comme 1.
pub fn grade_gap_ms(gap_num: i64, gap_den: u32, perfect_ms: u32, good_ms: u32) -> TimingGrade {
    let gap = i128::from(gap_num).unsigned_abs() * 50;
    let den = u128::from(gap_den.max(1)) * 3;
    let within = |ms: u32| gap <= u128::from(ms) * den;
    if within(perfect_ms.min(good_ms)) {
        TimingGrade::Perfect
    } else if within(good_ms) {
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

    /// P-1 : 60 ms = 3,6 frames, 120 ms = 7,2 frames ; écarts en 129es de frame (tempo 129).
    #[test]
    fn fenetres_en_millisecondes_sur_un_ecart_rationnel() {
        // 3,6 frames pile : parfait (borne incluse).
        assert_eq!(grade_gap_ms(36, 10, 60, 120), TimingGrade::Perfect);
        assert_eq!(grade_gap_ms(-37, 10, 60, 120), TimingGrade::Good);
        assert_eq!(grade_gap_ms(72, 10, 60, 120), TimingGrade::Good);
        assert_eq!(grade_gap_ms(73, 10, 60, 120), TimingGrade::Miss);
        // 12/129 de frame : parfait.
        assert_eq!(grade_gap_ms(12, 129, 60, 120), TimingGrade::Perfect);
        assert_eq!(grade_gap_ms(i64::MIN, 1, 60, 120), TimingGrade::Miss);
        assert_eq!(grade_gap_ms(0, 0, 0, 0), TimingGrade::Perfect);
    }
}
