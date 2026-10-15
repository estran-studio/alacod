# m3-t10-profil-accomplissements — accomplissements qui débloquent des objets (M3, V1e, G1 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §38
(profil), §51 (déblocages de M2-T12), §64 (contrat `Achievement` de M3-T0e), et les rapports de M2-T12 et
M3-T0e. Branche `m3-t10-profil-accomplissements` depuis `origin/main`, **après le merge de M2-T12 et
M3-T0e**. Estimation : 3 j. Section de conventions réservée : **§75**.

## À faire

1. **Évaluation hors simulation** en fin de run (système de `game::profile`, jamais dans `GgrsSchedule`) :
   chaque `Achievement` atteint ajoute ses `unlocks` au profil de chaque joueur local concerné ; migration de
   version du profil si un champ s'ajoute (`PROFILE_VERSION`).
2. **Entrée dans la run suivante** : par `RunUnlocks` de M2-T12, sans autre chemin (en ligne : la règle de
   M2-T12, profil de l'hôte ou intersection).
3. **Contenu `isaac`** : trois accomplissements (vaincre le boss de l'étage 1, finir la run, tenir une
   transformation si Q10) qui débloquent chacun un objet retiré du pool de base.
4. **Scénarios** : `achievement_unlocks` (deux runs enchaînées : l'objet apparaît dans le pool à graine
   égale), `achievement_not_met`, test de crate de migration du profil.

## Règles

Profil lu et écrit hors simulation ; scénarios sans écriture sauf demande. Pas de trace qui bouge hors
`isaac`.

## Hors périmètre

Hub (Isaac n'en a pas ; M2-T13 reste à gungeon), écran des accomplissements (dette de présentation au
rapport si besoin).

## Vérification

README §4 ; p2p à deux profils différents rejoué par l'orchestrateur ; rapport
`docs/taches/rapports/m3-t10-profil-accomplissements.md`, conventions §75.

## Atteignabilité

Une partie ordinaire qui bat le boss de l'étage 1 écrit l'accomplissement ; la suivante voit l'objet dans le
pool (scénario enchaîné, pas une graine).

## Session cloud

Tâche prévue pour une session Claude Code cloud, **après le merge de M2-T12**. Ignorer les consignes `orca`
du README (§1). Pas de Docker : le dire au rapport. Pousser la branche régulièrement. Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ m3-t10-profil-accomplissements`, qui contient le rapport.
