//! Règles d'équipe : qui peut viser qui, indépendamment du montant du dégât (voir
//! [`crate::damage::resolve_damage`] pour le montant). `Team` (le type, contrat partagé)
//! vient de `sim_core::team` (T0.2) ; ce module ajoute les règles (T1.1, chantier B1).

use sim_core::damage::FriendlyFire;
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;

/// Tag qui active la politique de tir ami [`FriendlyFire::Cursed`] : la source doit le
/// porter (dans `DamageEvent::tags`, qui inclut les tags du personnage) pour toucher un
/// allié.
pub const CURSED_TAG: &str = "cursed";

/// Bord d'une équipe pour le tir ami : `Players` et `Allies` comptent comme une seule
/// équipe (les alliés suivent les joueurs) ; `Enemies` et `Neutral` sont chacune leur
/// propre bord.
fn side_of(team: Team) -> u8 {
    match team {
        Team::Players | Team::Allies => 0,
        Team::Enemies => 1,
        Team::Neutral => 2,
    }
}

/// Deux équipes sont-elles du même bord (voir [`side_of`]) ?
fn same_side(a: Team, b: Team) -> bool {
    side_of(a) == side_of(b)
}

/// Est-ce que `source_team` peut viser `target_team`, du point de vue de l'équipe et de la
/// politique de tir ami de l'arme — indépendamment des résistances, immunités et de
/// l'invulnérabilité (voir [`crate::damage::resolve_damage`] pour le montant final).
///
/// `Team::Neutral` bloque toujours le tir (elle intercepte le coup, voir la doc de
/// `resolve_damage`) mais ne sera jamais blessée : cette fonction renvoie `true` pour
/// qu'une balle/mêlée la traite comme une cible valide (elle s'y arrête), et
/// `resolve_damage` renvoie ensuite `None` (aucun dégât) pour cette même cible.
pub fn team_allows_hit(
    source_team: Team,
    target_team: Team,
    weapon_policy: FriendlyFire,
    source_tags: &Tags,
) -> bool {
    if target_team == Team::Neutral {
        return true;
    }
    if same_side(source_team, target_team) {
        match weapon_policy {
            FriendlyFire::Never => false,
            FriendlyFire::Always => true,
            FriendlyFire::Cursed => source_tags.has(&Tag::new(CURSED_TAG)),
        }
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(names: &[&str]) -> Tags {
        Tags::parse(names.iter().copied())
    }

    #[test]
    fn cross_team_always_allowed_regardless_of_policy() {
        for policy in [
            FriendlyFire::Never,
            FriendlyFire::Always,
            FriendlyFire::Cursed,
        ] {
            assert!(team_allows_hit(
                Team::Players,
                Team::Enemies,
                policy,
                &Tags::default()
            ));
            assert!(team_allows_hit(
                Team::Enemies,
                Team::Players,
                policy,
                &Tags::default()
            ));
        }
    }

    #[test]
    fn allies_and_players_are_the_same_side() {
        assert!(!team_allows_hit(
            Team::Players,
            Team::Allies,
            FriendlyFire::Never,
            &Tags::default()
        ));
        assert!(!team_allows_hit(
            Team::Allies,
            Team::Players,
            FriendlyFire::Never,
            &Tags::default()
        ));
    }

    #[test]
    fn same_side_never_blocks() {
        assert!(!team_allows_hit(
            Team::Players,
            Team::Players,
            FriendlyFire::Never,
            &tags(&["cursed"])
        ));
    }

    #[test]
    fn same_side_always_allows() {
        assert!(team_allows_hit(
            Team::Players,
            Team::Players,
            FriendlyFire::Always,
            &Tags::default()
        ));
    }

    #[test]
    fn same_side_cursed_requires_tag() {
        assert!(!team_allows_hit(
            Team::Players,
            Team::Players,
            FriendlyFire::Cursed,
            &Tags::default()
        ));
        assert!(team_allows_hit(
            Team::Players,
            Team::Players,
            FriendlyFire::Cursed,
            &tags(&["cursed"])
        ));
    }

    #[test]
    fn neutral_target_always_counts_as_a_valid_hit_for_blocking() {
        // Le montant (toujours nul contre Neutral) est de la responsabilité de
        // `resolve_damage`, pas de cette fonction.
        for policy in [
            FriendlyFire::Never,
            FriendlyFire::Always,
            FriendlyFire::Cursed,
        ] {
            assert!(team_allows_hit(
                Team::Players,
                Team::Neutral,
                policy,
                &Tags::default()
            ));
            assert!(team_allows_hit(
                Team::Enemies,
                Team::Neutral,
                policy,
                &Tags::default()
            ));
        }
    }

    #[test]
    fn enemies_are_their_own_side() {
        assert!(!team_allows_hit(
            Team::Enemies,
            Team::Enemies,
            FriendlyFire::Never,
            &Tags::default()
        ));
    }
}
