# m4-t17-feedback-v3 — feedback v3 : larmes lisibles, familiers, champions, explosions (M4, V4, I2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §31 et §55
(feedback v1/v2), §66, §69, §71, §72. Branche `m4-t17-feedback-v3` depuis `origin/main`, **après le merge de
M4-T1, M4-T4, M4-T6 et M4-T7**. Estimation : 2 j. Section de conventions réservée : **§82**.

## À faire

1. Larmes : couleur et taille selon les modificateurs résolus (guidage, spectrale, statut), lues sur le
   composant `Projectile` en présentation.
2. Familiers et orbitaux : sprites provisoires (placeholders licenciés, `assets.yaml`), orientation.
3. Champions : teinte `tint` de la variante.
4. Bombe : mèche (clignotement selon les frames restantes), explosion (anneau, secousse), rocher cassé,
   mur secret qui s'ouvre.
5. Preuve par `FeedbackLog` et attentes `Event`, comme M2-T19 ; vidéos des scénarios `tears_*`,
   `familiar_*`, `bomb_*`, `champion_*` au rapport.

## Règles

Présentation seule : aucune trace ne bouge.

## Vérification

README §4 ; `make videos SCENARIO=…` ; rapport `docs/taches/rapports/m4-t17-feedback-v3.md`, conventions §82.

## Session cloud

Tâche prévue pour une session Claude Code cloud (captures en rendu logiciel). Ignorer les consignes `orca` du
README (§1). Pousser la branche régulièrement ; commit final `LIVRÉ m4-t17-feedback-v3` avec le rapport.
