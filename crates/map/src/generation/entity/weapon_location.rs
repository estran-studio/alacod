/// Donnée portée par une entité LDtk `WeaponLocation` (T2.3, chantier C5 v1) à travers le
/// pipeline de génération procédurale (`crates/map/src/generation`), à l'identique de
/// `CharacterSpawnConfig` (voir sa doc) : `weapon`/`price` sont des valeurs d'auteur
/// (posées dans l'éditeur LDtk) qui doivent traverser la génération telles quelles, aucune
/// règle ne les recalcule (contrairement aux portes, dont le `cost` est recalculé depuis la
/// profondeur de la salle).
#[derive(Debug, Clone, Default)]
pub struct WeaponLocationConfig {
    /// `WeaponId` du registre (`weapons.ron`).
    pub weapon: String,
    /// Prix d'achat, en points.
    pub price: u32,
}
