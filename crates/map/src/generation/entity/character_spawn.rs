/// Donnée portée par une entité LDtk `CharacterSpawn` (T2.9, testbed) à travers le pipeline
/// de génération procédurale (`crates/map/src/generation`) : contrairement aux portes
/// (`DoorConfig`, recalculé à partir de la profondeur de la salle) ou aux fenêtres
/// (`WindowConfig`, vide), `character`/`team` sont des valeurs d'auteur qui doivent
/// traverser la génération telles quelles (aucune règle ne les recalcule).
#[derive(Debug, Clone)]
pub struct CharacterSpawnConfig {
    /// `CharacterId` du registre (`games/<jeu>/assets/characters/*.ron`).
    pub character: String,
    /// `players`/`enemies`/`allies`/`neutral` ; `None` si le champ est absent ou vide dans
    /// l'éditeur LDtk (voir `crate::game::entity::map::character_spawn::CharacterSpawnComponent`).
    pub team: Option<String>,
    /// T1.5 : variante imposée (`variant`), `None` si absente ou vide — traverse la
    /// génération telle quelle, comme `team`.
    pub variant: Option<String>,
}
