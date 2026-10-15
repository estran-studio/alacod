# Deuxième run humaine enregistrée — lancement 03

Lancée le 2026-10-11 sur `c19447de`, `m1-revue-humaine`, graine 123456.
Archive récupérée le 2026-10-13 : original intact, 39026 frames, 2025 segments
avec commandes, métadonnées throne/Floors/run/etage/difficulté présentes.
Le précédent lancement 02 était sans commandes ; celui-ci contient du combat.
L'enregistrement comporte aussi une longue période sans balles après f3593.
Sa durée totale ne représente donc pas une durée de combat actif.

`performance.csv.gz` conserve les mesures natives complètes. Pour les 2466
mises à jour avec au moins une balle vivante : p95 17,2943 ms, p99 17,6967 ms,
2 mises à jour >33,3 ms, au plus 22 balles vivantes. Cadence de l'application,
pas timestamps GPU ni preuve de charge à 500 balles ; le filtre sur les balles
ne couvre pas nécessairement tous les combats. Tous les samples restent dans
le fichier pour examiner les pics. `summary.json` contient aussi les stats globales.

Les constats de William sur ce second jeu, l'étage atteint, le résultat final
et le restart restent à recueillir/extraire. Le boss avait déjà été rencontré
et tué dans la première run. Aucun nouveau verdict humain n'est inventé à
partir du seul enregistrement ou de la mesure de cadence.
