# polissage-feel — game feel, lot commun du moteur (S1-6)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 8), inventaire §9,
`crates/game/src/character/movement.rs` (l. 7, 27-90, 131-205), `core_sim.rs:193-198` (recopie `FixedTransform3D` →
`Transform`), `feedback.rs`, `feedback_log.rs`, `games/*/assets/ui/feedback.ron`, `camera.ron`,
`docs/digests/revue-m0.md:31-33`, `docs/conventions.md` §15, §31, §55. Branche `polissage-feel` depuis `origin/main`.
Estimation : 2 j. Clones : le moteur pour les quatre ; **réglage par donnée sur zombies**. Machine : Mac (vidéo).

## Contexte

Mouvement accéléré/freiné en fixed-point, dash, knockback, flash, secousse, hit-stop, chiffres de dégâts existent ; William
a validé course et dash de zombies (2026-10-09). **Mêmes réglages dans les quatre jeux** (flash 4, secousse 8/12, hit-stop
2) ; **aucune** donnée `movement`/`dash` dans les configs de zombies ; isaac sans dash ; **aucun** flash de bouche, recul
de tir, traînée ni douille ; le rendu recopie l'état fixe **sans interpolation**.

## À faire

1. **Audit chiffré d'abord** (2 h) : par clone, les valeurs effectives de mouvement et de feedback (accélération,
   décélération, dash, knockback, caméra) dans un tableau ; vidéo de référence de 60 s « avant ».
2. **Interpolation d'affichage** : entre deux états fixes, en `PostUpdate`, sur `Transform` seulement (jamais dans
   `GgrsSchedule`, jamais dans un hash) ; coupée au téléport, à la transition d'étage et au rollback ; mesurer sur un écran
   à 60 Hz et à une fréquence supérieure si disponible.
3. **Réglages par clone en données** (`feedback.ron`, `camera.ron`, `movement` des configs) : zombies d'abord, l'écart avec
   les autres jeux motivé ; changer `movement`/dash d'un personnage **bouge les traces** : preuve §5 et `BLESS`
   justifiés, sinon rester en présentation.
4. **Flash de bouche, recul visuel du tir, effet de mort** : présentation dérivée de `FeedbackLog`/état (`Update`),
   déclarés en données (par arme dans `by_weapon`), pas de mécanique ni de recul qui change la trajectoire.
5. Vidéo « après » ; William juge course, dash et tir.

## Acceptation

- Vidéo avant/après de zombies, note de William.
- L'interpolation est invisible à la simulation : `make test_scenarios` vert sans `BLESS` pour tout ce qui est présentation ;
  `schedule_order_diff` vide.
- Valeurs par clone publiées dans le rapport `docs/taches/rapports/polissage-feel.md` ; lot 2 (traînées, douilles) renvoyé en P8.
