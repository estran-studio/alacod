# polissage-sons-evenements — événements sonores manquants dans le moteur (S1-7)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axes 5 et 10), inventaire §5 et §11,
`crates/game/src/feedback.rs:203-212,907-950,1172-1351`, `feedback_log.rs`, `crates/content/src/feedback.rs` (`sounds`,
`volumes`, lint), `games/*/assets/ui/feedback.ron`, `crates/game/src/audio/`, `docs/taches/plan-assets.md`,
`docs/taches/m0-revue-suite-contenu.md:47-58` (S6, S7). Branche `polissage-sons-evenements` depuis `origin/main`.
Estimation : 1,5 j. Clones : le moteur (les quatre). Machine : VPS pour le code, Mac pour l'écoute et le déblocage audio.

## Contexte

Les clés `sounds` couvrent tir (trois familles), recharge, hit, melee, hurt, death, dash, door (et blank pour gungeon et
isaac). **Aucun** son de vague/salle/étage, boss, achat, ramassage, explosion/bombe ni menu ; une seule clé `death` ; volume
par clé en dB, **pas de volume global** ; déblocage de l'audio au premier clic dans le navigateur **non prouvé**.

## À faire

1. Ajouter les clés d'événement manquantes (vague début/fin, salle nettoyée, étage, boss (apparition, phase, mort), achat,
   ramassage, explosion, bombe, menu naviguer/confirmer/retour, mort par famille via une clé par `kind` avec repli sur
   `death`) au lint de `content::feedback` et aux `FeedbackKind` ; **optionnelles** : un jeu qui n'en déclare aucune garde
   son rendu d'avant.
2. Tous dérivés de l'état ou du `FeedbackLog` en `Update` (jamais un événement de la simulation, `CLAUDE.md`).
3. **Volume global** (curseur dans la pause, mémorisé hors rollback) en plus des `volumes` par clé.
4. **Audio navigateur** : l'audio démarre au premier clic ou appui (`AudioContext` repris) ; vérifier sur Chrome et Safari
   si disponible.
5. Tableau « événement × clone » (inventaire §11 en entrée) : chaque case dit « son », « repli » ou « vide » ; les sons
   propres à un clone sont des tâches de clone (S1-9, S2-2, S2-5).

## Acceptation

- Lint vert, tests unitaires des nouvelles clés ; aucun son joué depuis `GgrsSchedule`.
- `make test_scenarios` sans `BLESS` (journal `feedback … sound` : si une trace de journal bouge, c'est une preuve à dire).
- Son débloqué au premier clic dans Chrome (note). Rapport `docs/taches/rapports/polissage-sons-evenements.md`.
