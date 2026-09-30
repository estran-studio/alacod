/// Donnée portée par une entité LDtk `SodaLocation` (T2.3, chantier C5 v1) à travers le
/// pipeline de génération procédurale, à l'identique de `WeaponLocationConfig`/
/// `CharacterSpawnConfig` (voir leur doc) : `perk` est une valeur d'auteur, jamais recalculée.
#[derive(Debug, Clone, Default)]
pub struct SodaLocationConfig {
    /// `PerkId` du registre (`economy/perks.ron`).
    pub perk: String,
}
