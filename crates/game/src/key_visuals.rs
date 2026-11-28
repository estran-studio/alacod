//! Visuel d'une clé au sol (C-8, `docs/conventions.md` § 134) : la **décision** (hors rendu, dans tout jeu) et la
//! mesure d'affichage ; le dessin est dans `world_sprites::attach_key_visuals` (présentation).
//!
//! - **Donnée** : une entrée de la table `SpriteSheet` du jeu (`sprites/sprites.ron`) nommée `key/<id de clé>`, à défaut `key`
//!   (comme `weapon/<id>` puis `weapon`), avec le champ optionnel `display_scale` (défaut 1) : facteur d'affichage
//!   **par-dessus** l'échelle de la feuille, présentation seule.
//! - **Repli** : sans entrée, la clé garde un marqueur par défaut (rectangle doré cerclé de sombre) : aucune clé n'est invisible.
//! - **Décision** : [`decide_key_visuals`] (`PreUpdate`, après `RunGgrsSystems`, comme `attach_bullet_visuals`) pose
//!   [`KeyVisual`] sur toute `KeyPickup` qui n'en a pas (`Without<KeyVisual>` : une clé recréée par un rollback le reçoit de
//!   nouveau). Le composant n'est pas rollback, jamais haché ; l'invariant `cle_visible` du runner exige qu'une clé vivante en ait un.
//!
//! Rien ici ne lit ni n'écrit la simulation au-delà de la clé de contenu de `KeyPickup`.

use bevy::prelude::*;
use content::registry::{Registry, SpriteSheetId};
use items::KeyPickup;

/// Ce que la présentation dessine pour une clé au sol (non rollback, hors checksum et snapshot).
#[derive(Component, Clone, Debug, PartialEq)]
pub struct KeyVisual {
    /// Id de la table `SpriteSheet` retenu (`key/<id>` ou `key`) ; `None` : marqueur par défaut.
    pub sheet: Option<String>,
    /// Facteur d'affichage (`display_scale` de l'entrée, 1 sans entrée ni champ).
    pub scale: f32,
}

/// Côté (largeur, hauteur) du marqueur par défaut, en pixels monde avant l'échelle.
pub const FALLBACK_SIZE: Vec2 = Vec2::new(12.0, 6.0);

/// Ids de la table `SpriteSheet` cherchés pour la clé `key`, du plus précis au plus général.
pub fn key_candidates(key: &str) -> [String; 2] {
    [format!("key/{key}"), "key".to_string()]
}

/// Taille affichée d'un visuel : taille de base × facteur.
pub fn key_display_size(base: Vec2, scale: f32) -> Vec2 {
    base * scale
}

/// Visuel de la clé `key` d'après la table `SpriteSheet` du registre (sans registre : marqueur par défaut).
pub fn resolve_key_visual(registry: Option<&Registry>, key: &str) -> KeyVisual {
    let found = registry.and_then(|registry| {
        key_candidates(key).into_iter().find_map(|id| {
            let entry = registry
                .sprite_sheets
                .get(&SpriteSheetId::from(id.as_str()))?;
            Some((id, entry.display_scale))
        })
    });
    match found {
        Some((id, scale)) => KeyVisual {
            sheet: Some(id),
            scale: scale.unwrap_or(1.0),
        },
        None => KeyVisual {
            sheet: None,
            scale: 1.0,
        },
    }
}

/// Pose [`KeyVisual`] sur toute clé au sol qui n'en a pas. Tourne dans tout jeu (headless compris), `PreUpdate` après la
/// simulation : le visuel existe avant tout système d'`Update`, comme celui d'une balle.
pub fn decide_key_visuals(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    keys: Query<(Entity, &KeyPickup), Without<KeyVisual>>,
) {
    for (entity, pickup) in &keys {
        commands
            .entity(entity)
            .try_insert(resolve_key_visual(registry.as_deref(), &pickup.key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn the_size_is_the_base_size_times_the_factor() {
        assert_eq!(
            key_display_size(Vec2::new(19.0, 7.0), 2.0),
            Vec2::new(38.0, 14.0)
        );
        assert_eq!(key_display_size(FALLBACK_SIZE, 1.0), FALLBACK_SIZE);
    }

    #[test]
    fn candidates_go_from_the_specific_id_to_the_general_one() {
        assert_eq!(
            key_candidates("cle_a"),
            ["key/cle_a".to_string(), "key".to_string()]
        );
    }

    #[test]
    fn a_key_without_registry_gets_the_default_marker() {
        let visual = resolve_key_visual(None, "cle_a");
        assert_eq!(
            visual,
            KeyVisual {
                sheet: None,
                scale: 1.0
            }
        );
    }

    /// Une clé vivante reçoit un visuel, une seule fois, et un visuel déjà posé n'est pas remplacé.
    #[test]
    fn every_live_key_gets_a_visual_and_keeps_it() {
        let mut world = World::new();
        let fresh = world
            .spawn(KeyPickup {
                key: "cle_a".to_string(),
            })
            .id();
        let decided = world
            .spawn((
                KeyPickup {
                    key: "cle_b".to_string(),
                },
                KeyVisual {
                    sheet: Some("key".to_string()),
                    scale: 3.0,
                },
            ))
            .id();
        world.run_system_once(decide_key_visuals).unwrap();
        assert_eq!(world.get::<KeyVisual>(fresh).unwrap().sheet, None);
        assert_eq!(world.get::<KeyVisual>(decided).unwrap().scale, 3.0);
    }

    /// Le testbed : `cle_image` a `key/cle_image` à ×2, `cle_test` n'a rien (marqueur par défaut), une clé inconnue aussi.
    #[test]
    fn the_testbed_data_decides_the_image_the_factor_and_the_fallback() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/testbed");
        let (registry, _, _) = content::load_and_lint(&dir).unwrap();
        assert_eq!(
            resolve_key_visual(Some(&registry), "cle_image"),
            KeyVisual {
                sheet: Some("key/cle_image".to_string()),
                scale: 2.0
            }
        );
        for key in ["cle_test", "cle_inconnue"] {
            assert_eq!(
                resolve_key_visual(Some(&registry), key),
                KeyVisual {
                    sheet: None,
                    scale: 1.0
                },
                "{key}"
            );
        }
    }
}
