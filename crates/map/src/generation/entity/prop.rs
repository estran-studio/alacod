/// Donnée portée par une entité LDtk `Table` ou `Barrel` (M2-T11, `docs/conventions.md` §46) à
/// travers le pipeline de génération (comme `WeaponLocationConfig` : valeur d'auteur, jamais
/// recalculée). `entity` est l'identifiant LDtk (`Table` ou `Barrel`), `prop` l'id de l'objet de
/// décor (`props/<id>.ron`), vide pour le défaut (`table` / `barrel`).
#[derive(Debug, Clone)]
pub struct PropConfig {
    pub entity: String,
    pub prop: String,
}
