//! Services du hub (M2-T13, `docs/conventions.md` §57) : contrats sans dépendance au rendu.
//!
//! Un service (`hub/<id>.ron`, kind `HubService`) est une action du hub ([`HubAction`]) ouverte
//! par des conditions de profil ([`HubCondition`]). Les conditions se lisent sur le [`Profile`]
//! du joueur **hors simulation** (menu, système `Update`) : le hub n'en tire jamais de règle de
//! simulation, les déblocages entrent dans une run par `items::RunUnlocks` (§51).

use serde::{Deserialize, Serialize};

use crate::Profile;

/// Ce qu'un joueur doit avoir pour qu'un service s'ouvre pour lui.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HubCondition {
    /// Le profil possède ce déblocage (`Unlock` du jeu ou jalon comme `first_run`).
    Unlock(String),
    /// Statistique de profil minimale `(nom, valeur)`.
    Stat(String, u64),
}

/// Ce que le service fait quand on l'utilise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HubAction {
    /// Ouvre l'écran du marchand de déblocages.
    Merchant,
    /// Lance la run (porte d'entrée).
    EnterRun,
}

/// Un service du hub, `hub/<id>.ron` (kind `HubService`) ; id = nom de fichier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HubServiceDef {
    #[serde(default)]
    pub name: String,
    /// Toutes les conditions doivent être remplies ; vide : toujours ouvert.
    #[serde(default)]
    pub needs: Vec<HubCondition>,
    pub action: HubAction,
}

impl HubCondition {
    pub fn holds(&self, profile: &Profile) -> bool {
        match self {
            Self::Unlock(id) => profile.has_unlock(id),
            Self::Stat(name, min) => profile.stats.get(name).copied().unwrap_or(0) >= *min,
        }
    }
}

impl HubServiceDef {
    /// Le service est ouvert pour ce profil.
    pub fn is_open(&self, profile: &Profile) -> bool {
        self.needs.iter().all(|need| need.holds(profile))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(needs: Vec<HubCondition>) -> HubServiceDef {
        HubServiceDef {
            name: String::new(),
            needs,
            action: HubAction::Merchant,
        }
    }

    #[test]
    fn sans_condition_le_service_est_ouvert() {
        assert!(def(vec![]).is_open(&Profile::new("p")));
    }

    #[test]
    fn toutes_les_conditions_comptent() {
        let mut profile = Profile::new("p");
        let service = def(vec![
            HubCondition::Unlock("first_run".into()),
            HubCondition::Stat("runs".into(), 2),
        ]);
        assert!(!service.is_open(&profile));
        profile.unlocks.insert("first_run".into());
        assert!(!service.is_open(&profile));
        profile.stats.insert("runs".into(), 2);
        assert!(service.is_open(&profile));
    }

    #[test]
    fn lecture_ron() {
        let def: HubServiceDef =
            ron::from_str(r#"(name: "Marchand", needs: [Unlock("first_run")], action: Merchant)"#)
                .unwrap();
        assert_eq!(def.action, HubAction::Merchant);
        assert_eq!(def.needs, vec![HubCondition::Unlock("first_run".into())]);
    }
}
