# M3-T2 : effets v3

Branche `m3-t2-effets-v3`. Détail des règles : `docs/conventions.md` §67.

## Livré
`OnHit` (via `HitLanded`, émis par le résolveur de dégâts), `OnEvent`/`Emit` (même frame, 8 tours),
`ApplyStatus` sur la cible et `ApplyStatusSelf`, `Scaled { PerCopy }` exécuté, `TargetInRange`, limites
par exemplaire (`ItemDef::stack`, Q16 : indépendantes sur demande, partagées par défaut). Module
`game::effects_v3`. Lint : `ApplyStatus` hors cibles, statut inconnu, `stacks = 0`, noms vides.

## Questions
Q10 (transformations) : non traité ici. Q16 : défaut `Shared`, `Independent` en option par objet.

## Frames à regarder
`effect_on_hit` : f10 tir, poison à f~20-45. `effect_per_copy` : même tir, deux piles.

## Vérification
Voir le journal ; traces existantes inchangées, deux traces neuves.
