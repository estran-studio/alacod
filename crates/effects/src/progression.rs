//! Progression v1 (T1.10, chantier C4 v1, `docs/conventions.md` §27) : règles pures du niveau,
//! du tirage des mutations proposées, du choix et du pool d'armes. Le système
//! (`game::progression`) fournit le tirage (flux RNG `loot`) et l'état des joueurs.

use bevy_fixed::fixed_math::Fixed;

/// Niveau = nombre de seuils `levels` (croissants) atteints par `value`.
pub fn level_for(levels: &[Fixed], value: Fixed) -> u32 {
    levels
        .iter()
        .take_while(|threshold| value >= **threshold)
        .count() as u32
}

/// Une mutation candidate du pool (ordre stable : celui du registre, par id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub id: String,
    pub weight: u32,
    pub max_stacks: u32,
}

/// Tire jusqu'à `choices` mutations **distinctes** du pool, pondérées par `weight`, en sautant
/// celles que le joueur porte déjà `max_stacks` fois (`owned`). `next(n)` rend un entier dans
/// `[0, n)` (flux `loot`) ; appelé une fois par option tirée. Moins d'options si le pool
/// s'épuise.
pub fn draw_options(
    pool: &[Candidate],
    owned: &[String],
    choices: u32,
    mut next: impl FnMut(u32) -> u32,
) -> Vec<String> {
    let mut remaining: Vec<&Candidate> = pool
        .iter()
        .filter(|c| c.weight > 0)
        .filter(|c| (owned.iter().filter(|o| **o == c.id).count() as u32) < c.max_stacks)
        .collect();
    let mut options = Vec::new();
    while (options.len() as u32) < choices && !remaining.is_empty() {
        let total: u32 = remaining.iter().map(|c| c.weight).sum();
        let pick = next(total);
        let mut cumulative = 0;
        let index = remaining
            .iter()
            .position(|c| {
                cumulative += c.weight;
                pick < cumulative
            })
            .unwrap_or(remaining.len() - 1);
        options.push(remaining.remove(index).id.clone());
    }
    options
}

/// Option prise cette frame, ou `None` (choix encore ouvert). `pressed` : premier bouton de
/// choix tenu (0 = A, 1 = B, 2 = C) ; il ne compte que si le choix est `armed` (aucun bouton
/// de choix tenu depuis l'ouverture : un bouton gardé enfoncé ne choisit pas deux niveaux de
/// suite) et désigne une option existante. Sans choix, la première option est prise à
/// `since_frame + choice_frames`.
pub fn resolve_choice(
    pressed: Option<usize>,
    armed: bool,
    options: usize,
    since_frame: u32,
    choice_frames: u32,
    frame: u32,
) -> Option<usize> {
    if options == 0 {
        return None;
    }
    if let Some(index) = pressed {
        if armed && index < options {
            return Some(index);
        }
    }
    (frame >= since_frame.saturating_add(choice_frames)).then_some(0)
}

/// Armes du pool débloquées au niveau `max_level` (`level ≤ max_level`), dans l'ordre du pool.
pub fn weapon_candidates<'a>(pool: &'a [(u32, Vec<String>)], max_level: u32) -> Vec<&'a str> {
    pool.iter()
        .filter(|(level, _)| *level <= max_level)
        .flat_map(|(_, weapons)| weapons.iter().map(String::as_str))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(v: f32) -> Fixed {
        Fixed::from_num(v)
    }

    fn pool() -> Vec<Candidate> {
        ["coriace", "tireur", "vampire"]
            .iter()
            .map(|id| Candidate {
                id: id.to_string(),
                weight: 1,
                max_stacks: 1,
            })
            .collect()
    }

    /// Générateur congruentiel de test (même graine ⇒ même suite).
    fn lcg(seed: u32) -> impl FnMut(u32) -> u32 {
        let mut state = seed;
        move |n| {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (state >> 8) % n
        }
    }

    #[test]
    fn niveau_par_seuils() {
        let levels = [fx(2.0), fx(4.0)];
        assert_eq!(level_for(&levels, fx(0.0)), 0);
        assert_eq!(level_for(&levels, fx(1.9)), 0);
        assert_eq!(level_for(&levels, fx(2.0)), 1);
        assert_eq!(level_for(&levels, fx(3.0)), 1);
        assert_eq!(level_for(&levels, fx(9.0)), 2);
    }

    #[test]
    fn tirage_deterministe_et_distinct() {
        let a = draw_options(&pool(), &[], 3, lcg(7));
        let b = draw_options(&pool(), &[], 3, lcg(7));
        assert_eq!(a, b, "même graine ⇒ mêmes options");
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(sorted, ["coriace", "tireur", "vampire"], "distinctes");
        // Le tirage suit le générateur
        assert_eq!(
            draw_options(&pool(), &[], 1, |_| 0),
            ["coriace"],
            "0 ⇒ la première du pool"
        );
        assert_eq!(draw_options(&pool(), &[], 1, |n| n - 1), ["vampire"]);
    }

    #[test]
    fn tirage_pondere_et_max_stacks() {
        let mut weighted = pool();
        weighted[1].weight = 8; // tireur : 8 sur 10
        assert_eq!(draw_options(&weighted, &[], 1, |_| 1), ["tireur"]);
        assert_eq!(draw_options(&weighted, &[], 1, |_| 8), ["tireur"]);
        assert_eq!(draw_options(&weighted, &[], 1, |_| 9), ["vampire"]);
        // tireur déjà porté (max_stacks 1) : exclu ; coriace à 2 piles autorisées : gardée
        weighted[0].max_stacks = 2;
        let owned = ["tireur".to_string(), "coriace".to_string()];
        let options = draw_options(&weighted, &owned, 3, lcg(1));
        assert_eq!(options.len(), 2);
        assert!(!options.contains(&"tireur".to_string()));
        // Pool épuisé : aucune option
        let owned = ["coriace", "coriace", "tireur", "vampire"].map(String::from);
        assert!(draw_options(&weighted, &owned, 3, lcg(1)).is_empty());
    }

    #[test]
    fn choix_par_bouton_et_par_expiration() {
        // Bouton B, choix armé
        assert_eq!(resolve_choice(Some(1), true, 3, 100, 600, 130), Some(1));
        // Bouton tenu depuis l'ouverture (non armé) : ignoré
        assert_eq!(resolve_choice(Some(1), false, 3, 100, 600, 130), None);
        // Bouton C sans troisième option : ignoré
        assert_eq!(resolve_choice(Some(2), true, 2, 100, 600, 130), None);
        // Expiration : première option à since + choice_frames
        assert_eq!(resolve_choice(None, true, 3, 100, 600, 699), None);
        assert_eq!(resolve_choice(None, true, 3, 100, 600, 700), Some(0));
        assert_eq!(resolve_choice(None, true, 0, 100, 600, 700), None);
    }

    #[test]
    fn pool_d_armes_filtre_par_niveau() {
        let pool = vec![
            (0, vec!["pistol".to_string()]),
            (2, vec!["shotgun".to_string(), "machine_gun".to_string()]),
        ];
        assert_eq!(weapon_candidates(&pool, 0), ["pistol"]);
        assert_eq!(weapon_candidates(&pool, 1), ["pistol"]);
        assert_eq!(
            weapon_candidates(&pool, 2),
            ["pistol", "shotgun", "machine_gun"]
        );
    }
}
