# m1-v4-feedback-v1 — feedback v1 : hit stop, secousse, flash, télégraphe, chiffres (T1.17, voie V4, I2)

Lire d'abord `docs/taches/README.md` (agent **local**, `origin/main` mergé). Tâche moyenne (3 j),
**présentation seule** : aucune trace ne change. Prérequis : T1.2 (`Emitter.telegraph`), T1.4
(`Charge` et sa phase de télégraphe), T2.13 (`feedback.rs`, `ui/feedback.ron`, conventions §9).

## Décisions (fixées après proposition de b1)

1. **Hit stop** : ne fige jamais la simulation (rollback, p2p) ; gèle l'animation et la caméra `n`
   frames de **rendu** ; la simulation continue.
2. **`feedback.ron` étendu** : `hit_stop_frames`, secousse (amplitude, frames) et flash (couleur,
   frames) par genre de dégâts (`DamageKind`) et par arme ; `damage_numbers: bool`. Lint des champs.
3. **Télégraphe au sol** : cercle dérivé de `Emitter.telegraph` (frames restantes) et de la phase de
   télégraphe de `Charge` (cible figée), rayon issu de la portée ; lecture seule.
4. **Bug à corriger d'abord** : le flash actuel cherche un `Sprite` sur l'entité racine alors que les
   calques sont des enfants : parcourir les enfants (même parcours que la teinte des statuts T1.3).
5. **Preuve sans écran** : ressource de présentation `FeedbackLog` (genre, frame, position), remplie
   aussi en headless, relue par un moment clé `feedback` de `scenario::events` ; attente optionnelle
   `Event("feedback")`. Un scénario prouve « télégraphe à f…, hit stop à f… » sans écran. Captures hors
   écran d'`enemy_charge` (cercle) et d'un tir de `grenade` (secousse, chiffres) si le GPU le permet.
6. **Hors périmètre** : sons (D32), particules, écran de mort.

## Règles

Deux compilations au plus ; purge + point d'état ; `docs/conventions.md` : **uniquement** §31 « Feedback
v1 » (+ une ligne au §9 Feedback) ; traces : `make test_scenarios` vert sans bless (le `FeedbackLog`
est hors simulation et hors trace).

## Livrer

Rapport `docs/taches/rapports/m1-v4-feedback-v1.md` ; `git push -u origin m1-v4-feedback-v1` ;
`SendMessage` à `orch` : `LIVRÉ m1-v4-feedback-v1 <sha> : <une ligne>`. Ne merge pas.
