//! Demandes de visuel posées par la simulation ou la carte sur une entité (données pures).
//!
//! `map_ldtk` les pose à la génération d'un étage ; la présentation les lit (`legibility`,
//! `world_sprites`). Aucune dépendance de rendu : ce fichier est de la simulation.

use bevy::prelude::*;

/// Texte d'étiquette posé par un système de la simulation ou de la carte sur une entité qui n'a pas
/// d'autre source de nom (ex. le portail) : `WorldLabel("Portail")`.
#[derive(Component, Clone, Debug)]
pub struct WorldLabel(pub String);

/// Demande explicite d'un sprite du monde par id (`sprites.ron`) : posé sur une entité de la carte
/// qui n'a pas de composant de jeu reconnaissable (le portail). L'image propre de l'entité (carré
/// de couleur de repli) est alors rendue transparente.
#[derive(Component, Clone, Debug)]
pub struct SpriteRequest(pub String);
