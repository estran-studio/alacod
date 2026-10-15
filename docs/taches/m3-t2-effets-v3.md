# m3-t2-effets-v3 — effets v3 à l'échelle : OnHit, OnEvent, exemplaires, transformations (M3, V1b, C1 v3)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §27, §42,
§44, §61 (contrats de M3-T0b) et le rapport `docs/taches/rapports/m3-t0b-contrats-objets-sante.md`, le code
`crates/effects/src/runtime.rs`, `crates/game/src/effects_runtime.rs`. Branche `m3-t2-effets-v3` depuis
`origin/main`, **après le merge de M3-T0b**. Estimation : 4 j. Section de conventions réservée : **§67**.

## Contexte

Les déclencheurs `OnHit` et `OnEvent`, la condition `TargetInRange`, l'action `ApplyStatus` dans un effet
sont des contrats refusés par le lint. `OnPickup` se déclenche sur n'importe quel ramassage. Deux exemplaires
d'un objet partagent leurs limites. `EffectsPlugin` ne déclare pas `OnFloorStart` ni `OnBlank` dans ses
kinds (constat de lecture).

## À faire

1. **`OnHit`** (le porteur touche un personnage : `DamageEvent` dont il est la source, après résolution),
   **`OnEvent(nom)`** (événements nommés émis par d'autres effets : action `Emit(nom)` en fin d'enum, même
   frame, ordre des porteurs par `GgrsNetId`), **`TargetInRange`** (cible du déclencheur à portée du porteur).
2. **`ApplyStatus` dans un effet** (sur la cible du déclencheur ; sur le porteur si `to: Self`).
3. **`OnPickup(Some(item))`** : filtre optionnel, champ hors hash quand absent.
4. **Exemplaires** : brancher la clé `item:<id>#<k>:<index>` de T0b dans `listed_effects` et `Scale::PerCopy`.
   Ce point **change** des parties gungeon où un doublon d'`amulette` ou de `gilet` est ramassé : preuve §5
   contre `main`, cause dans le commit, et réponse de William à **Q16** avant le bless (sinon garder le
   partage par défaut et une option par objet `stack: Independent`).
5. **Transformations** (seulement si **Q10** les retient) : extension de `SynergyDef` par
   `requires_tag: Some((tag, n))` (au moins `n` objets distincts portant le tag), état dérivé comme les
   synergies (§44), aucun composant en plus.
6. **Kinds** : déclarer tous les déclencheurs exécutés.
7. **Échelle** : scénario `effects_hundred_items` (un joueur qui tient 100 objets d'essai à effets `Tick`,
   `OnHit`, `OnKill`) ; mesurer `advance_ms_avg` avant/après, chiffre au rapport ; si `listed_effects` coûte,
   précalculer la liste par porteur hors rollback (donnée dérivée réécrite à chaque frame, règle de
   `CLAUDE.md`).
8. **Scénarios** : `effect_on_hit`, `effect_on_event`, `effect_target_in_range`, `effect_apply_status`,
   `effect_pickup_filtered`, `effect_per_copy`, `effect_scale_per_copy`, `transformation_three` (si Q10).

## Règles

Traces : seule la partie 4 peut en bouger, avec preuve. Ordre d'exécution documenté au §67 (porteurs par
`GgrsNetId`, effets du personnage, objets par ordre de ramassage et rang d'exemplaire, synergies,
transformations). Pas de `Message`, `FrameEvents` seulement.

## Hors périmètre

Contenu `isaac` (T11b), HUD de transformation (T16).

## Vérification

README §4 ; rapport `docs/taches/rapports/m3-t2-effets-v3.md`, conventions §67.

## Atteignabilité

Les objets à effets de T11b dans les pools trésor, boutique et boss de `isaac` ; transformation : graine
`alacod-sim` nommée par T11b.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et
rejouer la suite juste avant le commit `LIVRÉ m3-t2-effets-v3`, qui contient le rapport.
