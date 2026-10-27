//! Bac à sable de l'engine (`docs/taches/plan-bac-a-sable.md`, `docs/conventions.md` §87) : les données de
//! session que partagent la ligne de commande, `map_ldtk::game::sandbox` (qui pose le profil) et le remote
//! (`alacod/sandbox_profile`). Aucune simulation ici.
//!
//! `--sandbox [profil]` (`args::BaseArgsPlugin`) pose [`SandboxRequest`] ; `LdtkLocalGamePlugin` voit la
//! ressource, valide le profil ([`resolve`]) puis pose [`SandboxSession`], que le remote change pour charger
//! un autre profil.

use bevy::prelude::*;
use content::{manifest::GameManifest, registry::Registry, sandbox::SandboxProfile};

/// Profil demandé par `--sandbox [profil]` (`content::sandbox::DEFAULT_PROFILE` sans argument). Posée par
/// `GameArgsPlugin` seulement si le drapeau est là ; son absence = partie ordinaire.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SandboxRequest(pub String);

/// Profil actif d'une partie en bac à sable : celui que `OnEnter(GameLoading)` pose (donc que `restart`
/// rejoue). `alacod/sandbox_profile` le change puis relance.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SandboxSession {
    pub profile: String,
}

/// Graine de carte du bac à sable quand `--seed` n'en impose pas : celle de `Scenario::map_seed` par défaut,
/// avec laquelle SB-1 a mesuré les placements des profils (joueur 0 en (424, 696), un ennemi en (624, 696) est
/// 200 unités à sa droite). Une arène assemblée par graine (zombies : `exemples/test_map.ldtk`) serait sinon
/// tirée au hasard par le jeu et les placements tomberaient dans un mur.
pub const ARENA_SEED: i32 = 123456;

/// Un profil validé : son contenu et l'arène où il se joue.
pub struct Resolved<'a> {
    pub profile: &'a SandboxProfile,
    /// Carte de l'arène (`content::sandbox::SandboxProfile::arena`, D2).
    pub arena: &'a str,
}

/// Le profil `id` et son arène, ou une erreur qui dit quoi faire : jeu sans kind `Sandbox`, profil inconnu
/// (les profils disponibles sont listés), jeu sans arène.
pub fn resolve<'a>(
    registry: &'a Registry,
    manifest: &'a GameManifest,
    id: &str,
) -> Result<Resolved<'a>, String> {
    let game = &manifest.name;
    if registry.sandbox_profiles.is_empty() {
        return Err(format!(
            "bac à sable : le jeu « {game} » n'a aucun profil. Déclarer le dossier `Sandbox` dans `content_folders` de game.ron \
             et créer `sandbox/{}.ron` (`alacod new-content Sandbox {}`)",
            content::sandbox::DEFAULT_PROFILE,
            content::sandbox::DEFAULT_PROFILE
        ));
    }
    let Some(entry) = registry.sandbox_profiles.get(id) else {
        let known: Vec<&str> = registry
            .sandbox_profiles
            .keys()
            .map(String::as_str)
            .collect();
        return Err(format!(
            "bac à sable : profil « {id} » inconnu du jeu « {game} » (profils : {})",
            known.join(", ")
        ));
    };
    let arena = entry.profile.arena(manifest).ok_or_else(|| {
        format!(
            "bac à sable : aucune arène pour le profil « {id} » du jeu « {game} » : poser `map` dans le profil, \
             `sandbox: (arena: \"…\")` dans game.ron, ou `generate_template`"
        )
    })?;
    Ok(Resolved {
        profile: &entry.profile,
        arena,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::manifest::{EntryPoint, SandboxEntry};
    use content::registry::SandboxProfileEntry;

    fn manifest(arena: Option<&str>) -> GameManifest {
        GameManifest {
            name: "demo".to_string(),
            content_folders: vec![],
            entry: EntryPoint {
                start_map: "m.ldtk".to_string(),
                default_seed: 1,
                mode: None,
                floors: None,
                hub: None,
                progression: None,
                clocks: None,
                difficulty: None,
                player: None,
                map: Default::default(),
            },
            generate_template: None,
            mechanics: vec![],
            plugin: Default::default(),
            sandbox: arena.map(|arena| SandboxEntry {
                arena: arena.to_string(),
            }),
        }
    }

    fn registry(ids: &[&str]) -> Registry {
        let mut registry = Registry::default();
        for id in ids {
            registry.sandbox_profiles.insert(
                id.to_string(),
                SandboxProfileEntry {
                    id: id.to_string(),
                    file: format!("sandbox/{id}.ron").into(),
                    profile: SandboxProfile::default(),
                },
            );
        }
        registry
    }

    fn error(registry: &Registry, manifest: &GameManifest, id: &str) -> String {
        match resolve(registry, manifest, id) {
            Ok(_) => panic!("« {id} » aurait dû être refusé"),
            Err(error) => error,
        }
    }

    #[test]
    fn la_graine_de_l_arene_est_celle_des_scenarios() {
        let scenario = crate::replay::Scenario::from_ron(
            "Scenario(game: \"x\", map: \"m\", frames: 1, players: [])",
        )
        .expect("scénario minimal");
        assert_eq!(scenario.map_seed, ARENA_SEED);
    }

    #[test]
    fn un_jeu_sans_profil_est_refuse_avec_la_marche_a_suivre() {
        let error = error(&registry(&[]), &manifest(Some("a.ldtk")), "arene");
        assert!(error.contains("n'a aucun profil"), "{error}");
        assert!(
            error.contains("alacod new-content Sandbox arene"),
            "{error}"
        );
    }

    #[test]
    fn un_profil_inconnu_liste_les_profils_du_jeu() {
        let error = error(
            &registry(&["arene", "lourd"]),
            &manifest(Some("a.ldtk")),
            "x",
        );
        assert!(error.contains("« x » inconnu"), "{error}");
        assert!(error.contains("arene, lourd"), "{error}");
    }

    #[test]
    fn une_arene_absente_est_une_erreur_claire() {
        let error = error(&registry(&["arene"]), &manifest(None), "arene");
        assert!(error.contains("aucune arène"), "{error}");
        assert!(error.contains("generate_template"), "{error}");
    }

    #[test]
    fn l_arene_du_profil_gagne_sur_celle_du_manifeste() {
        let mut registry = registry(&["arene", "autre"]);
        registry
            .sandbox_profiles
            .get_mut("autre")
            .unwrap()
            .profile
            .map = Some("b.ldtk".into());
        let manifest = manifest(Some("a.ldtk"));
        assert_eq!(
            resolve(&registry, &manifest, "arene").unwrap().arena,
            "a.ldtk"
        );
        assert_eq!(
            resolve(&registry, &manifest, "autre").unwrap().arena,
            "b.ldtk"
        );
    }
}
