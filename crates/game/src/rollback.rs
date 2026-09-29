//! Réexport de l'extension rollback : l'implémentation vit dans `utils` (voir sa doc)
//! parce qu'`animation` en a besoin sans dépendre de `game`. Réexporté ici pour que les
//! sites de `game` continuent d'écrire `use crate::rollback::RollbackTraceApp;`.

pub use utils::rollback::*;
