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
            let entry = registry.sprite_sheets.get(&SpriteSheetId::from(id.as_str()))?;
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
