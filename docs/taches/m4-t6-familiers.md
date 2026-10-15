# m4-t6-familiers — familiers et orbitaux (M4, V1c, C3)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (numérotation des entités, despawn différé),
`docs/taches.md` §8.1, `docs/conventions.md` §20, §22, §36, §41, §49, §63 (contrats de M4-T0d), §66 (larmes
v3) et les rapports de M4-T0d et M4-T1. Branche `m4-t6-familiers` depuis `origin/main`, **après le merge de
M4-T0d et M4-T1**, et la réponse de William à **Q13** (larmes du familier). Estimation : 4 j. Section de
conventions réservée : **§71**.

## À faire

1. **Création et destruction** : ramasser un objet `familiar` crée `per_copy` familiers par exemplaire
   (`FamiliarOf { owner, rank }`, `GgrsNetId` dans l'ordre des ramassages de la frame) ; perdre l'objet
   (actif remplacé) les détruit par `despawn_rollback()`. Un porteur à terre ou mort : ses familiers
   restent immobiles (proposition ; le dire au rapport).
2. **Behaviors** `Follow`, `Orbit`, `MirrorShot`, `BlockBullets` exécutés, dans un set fixe après le
   mouvement des joueurs et avant les projectiles ; orbites par `fixed_math` (aucun `f32`).
3. **Tir** : `MirrorShot` tire avec l'arme du familier ou les larmes composées du porteur selon Q13.
4. **Coop** : quatre joueurs à deux familiers chacun ; aucun familier ne bloque le déplacement d'un joueur.
5. **Scénarios** (testbed) : `familiar_follow`, `familiar_mirror_shot` (`HitsAtLeast` sur une cible),
   `orbital_blocks_bullet` (`BulletCount` hostile baisse au contact), `familiar_two_copies`,
   `familiar_lost_with_active`, `familiar_quad` (synctest à 4).

## Règles

Familiers en équipe `Players` (ou `Allies`, selon T0d), immunisés, jamais ciblés par l'IA ennemie (règle
actuelle `With<Player>`). Traces existantes intactes (aucun objet existant n'a `familiar`).

## Hors périmètre

Familiers qui ramassent ou qui ont une IA de cible (D5) ; visuels (T17) ; contenu (T11b).

## Vérification

README §4 ; mesure de `advance_ms_avg` sur `familiar_quad` au rapport ; rapport
`docs/taches/rapports/m4-t6-familiers.md`, conventions §71.

## Atteignabilité

T11b met au moins deux familiers et un orbital dans le pool trésor de `isaac` ; graine nommée.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et
rejouer la suite juste avant le commit `LIVRÉ m4-t6-familiers`, qui contient le rapport.
