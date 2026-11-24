# C-3 — Un décor solide bloque les corps

- **Existait** : prop (`prop: true`, § 112) sur `enemy_layer` : séparation douce (ennemis), ralentissement (joueur), ignoré du champ de flux ; `cover` (§ 118) pour les balles seules.
- **Manquait** : un champ de donnée rendant le collider d'un prop dur pour les corps, et sa preuve (aucun test ne posait un corps contre un prop).
- **Ajouté** : `solid: true` (§ 132), composant statique `SolidBody`, lecture par `move_characters`, `move_enemies`, `rebuild_blocked_cells` ; lint ; 6 scénarios `decor_solide_*`.

## Preuve
(à remplir : rouge, vert, traces inchangées, contrôle rapide, capture)
