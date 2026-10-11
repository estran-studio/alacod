# M1 — revue humaine de throne

2026-10-10, William, MacBook. Branche `m1-revue-humaine`, base main `f53cd1ae`.
Movement-feel, bots après movement-feel, D55 et vague 0 M2 présents ; D30 déjà réglée.
Espace = objet actif, Q = blank. M2 est déjà lancée : vague 0 mergée, T10 en cours.

## Verdict de sortie M1

**Non à ce stade : revue humaine incomplète.** Une run solo humaine est archivée
(11882 frames, graine 123456), avec deux passages jusqu'à `roi_rat` ; une deuxième
run humaine, le restart et les jugements/décisions ci-dessous restent à recueillir.
Les validations automatiques des correctifs ne remplacent pas ce dernier critère.
Ce qui empêche de prononcer la sortie de M1 :

- Deuxième run humaine encore sans constats et sans résultat consigné.
- Confirmation du ressenti après correctifs, restart humain et avis des huit points encore incomplets.
- Décisions D28/D29/D34/D40b/D50/D54, difficulté du troisième étage et relecture du plan M2 encore absentes.

## Constats triés

| Gravité | Où | Constat de William | Traitement et preuve |
|---|---|---|---|
| Bloquant | Avant de jouer, présentation des ennemis | « Premier probleme avant que je sais sur throne, on a pas assets pour les enemies je crois » ; « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies » | Onze ennemis visibles avec sprites provisoires attribués, commit `ced52082`. Scénarios verts, traces identiques hors ActiveLayers. La première run a ensuite pu être jouée ; distinction visuelle des silhouettes à juger. |
| À corriger, gravité humaine à confirmer | Run solo 01, passages denses | « Beaucoup beaucoup de frame drop quand il a bcp de bullets » | Coût dominant mesuré dans la géométrie du flow field ; optimisation pure `41d3859d`, 204 scénarios verts et traces inchangées. Comparaison détaillée ci-dessous ; ressenti à confirmer. |
| À corriger, gravité humaine à confirmer | Run solo 01, cavernes ; candidat pillard 220, étage 2, f2325 | « Des enemies restes coincé dans les murs au lieu de venir vers nous » | Fuite/recul utilisant des voisins franchissables et centre sûr du corps (`faa5120c`). Régression de la run verte ; sprite sorti du coin. 22 scénarios concernés, 14 traces modifiées avec preuve. L'emplacement vu par William reste à confirmer. |
| À corriger, gravité humaine à confirmer | Run solo 01, écran de mutation ; capture de diagnostic f720 | « On recoit les augments pendant qu'on est en plein combat » | Choix conservés en attente pendant le combat, présentés après nettoyage ; portail attend tous les joueurs, projectiles hostiles supprimés et dégâts résiduels neutralisés pendant l'interlude. Politique BetweenFloors réservée à throne en mode Floors. Validation et traces dans le dossier mutations. |
| À corriger, constat technique de l'agent | Enregistrement natif/restart | La run throne enregistrait game=zombies et omettait les réglages de run | Réglages effectifs capturés, fichiers de restart séparés (`57de6173`). Aller-retour de 400 frames exact et test d'archivage verts. Original humain conservé intact. |

Appréciation générale, mot pour mot : « Ca marche assez bien generalement mais problemes ».
Aucune gravité définitive n'a été donnée pour les trois constats de gameplay.

## Performance mesurée

Mac Apple M4 Max, 14 cœurs, 36 Gio, macOS 26.7.1 ; même séquence diagnostique,
profil headless avec rendu, 800×600 logiques, aucune compilation/capture PNG pendant
les mesures. Passage f3200–5600, au plus 28 balles vivantes.

| Configuration | Cadence p95 / p99 | Mises à jour >33,3 ms | Simulation moyenne |
|---|---|---|---|
| Avant, logs info, synctest 2 | 53,79 / 98,15 ms | 208/1860 | 9,31 ms |
| Avant, logs warn, synctest 0 | 21,08 / 22,02 ms | 3/2400 | 2,42 ms |
| Après, logs warn, synctest 0 | 17,27 / 17,63 ms | 0/2400 | 1,27 ms |
| Après, logs warn, synctest 2 | 17,33 / 17,60 ms | 0/2401 | 3,06 ms |

Le gain moteur est isolé du changement de distance de vérification. Le client normal
simule une fois ; les scénarios gardent le synctest 2. Les traces natives activent aussi
2 pour conserver les snapshots (`50257691`). Le p95 strict proposé à 16,7 ms n'est pas
atteint exactement. Ces mesures portent sur la cadence applicative, sans timestamps
GPU/images présentées ni comparaison release ; elles ne valident pas 500 projectiles M2.
Le rejeu humain utilise une copie diagnostique aux métadonnées réparées : l'identité
complète avec l'ancien natif n'est pas affirmée.

Preuves : [performance](../../../tests/review-notes/m1-f53cd1ae-performance/README.md),
[navigation](../../../tests/review-notes/m1-f53cd1ae-navigation/README.md),
[mutations](../../../tests/review-notes/m1-f53cd1ae-mutations/README.md),
[enregistreur](../../../tests/review-notes/m1-f53cd1ae-recording/README.md),
[run originale](../../../tests/review-notes/m1-f53cd1ae-run-01/README.md),
[carnet](../../../tests/review-notes/m1-f53cd1ae.md).

## Couverture des huit points

| Point | État de la revue humaine |
|---|---|
| 1. Armes throne/zombies après D51 | Avis comparatif encore manquant ; zombies compilé pour la session. |
| 2. Difficulté du troisième étage | Étage joué ; jugement et décision de rééquilibrage encore manquants. |
| 3. Boss roi_rat | Boss rencontré et tué deux fois dans la première run ; lisibilité et télégraphes à juger. |
| 4. Écran de mutation | Constat précis recueilli, timing corrigé ; lecture des cartes/texte à confirmer. |
| 5. HUD | Avis encore manquant. |
| 6. Feedback | Constat de fluidité recueilli ; hit stop, secousse, flash/chiffres à juger séparément. |
| 7. Cavernes | Constat de navigation recueilli ; variété, roche et surfaces à juger. |
| 8. Restart / en ligne | Restart humain encore manquant ; réseau non essayé, selon disponibilité d'un partenaire. |

## Décisions datées

- 2026-10-10 — Visibilité : « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies ».
- 2026-10-10 — Plan des quatre correctifs, mutations entre étages incluses : « Parfait revalue la taches et si toute est bon tu peux ty lancer ».

D28 (dégât direct), D29 (multiplicateurs zombies), D34 (CORDIC), D40b (grunt sans
attaque, reliquat D40), D50 (munitions garanties), rééquilibrage du troisième étage,
D54 (tir ami sur le tireur) : **aucune décision de William recueillie**. Ne pas les fermer.
Relecture humaine de [m2-plan-brouillon.md](../m2-plan-brouillon.md) encore attendue,
en tenant compte du dash à i-frames et des contrats/input M2 déjà mergés.

## Lignes de dettes proposées à orch

Ces propositions n'ont pas été intégrées à `dettes.md` ; les identifiants sont à attribuer.

| Proposition | Description | Où | Suite proposée |
|---|---|---|---|
| Visibilité throne | Ennemis désormais visibles avec des sprites zombies provisoires attribués ; silhouettes distinctes encore à juger | assets throne, preuve m1-f53cd1ae-visibilite.md | Clore le blocage de visibilité après confirmation humaine ; conserver la question des silhouettes si William la retient. |
| Fluidité native | Flow field optimisé et vérification locale séparée ; forte amélioration sur le passage humain diagnostiqué | performance.rs, navigation.rs, preuves performance | Confirmer le ressenti dans une seconde run ; mesure GPU/charge 500 balles reste dans le travail M2. |
| Fuite dans les coins | Recul corrigé, pillard 220 sorti de la roche ; peut rester au repos dans une cellule sûre | ai/retreat.rs, preuves navigation | Clore le cas reproduit après confirmation ; D55 ne prouvait pas cette fuite. |
| Mutations en combat | Choix entre étages, tous les niveaux conservés, transition attend tous les joueurs | progression.rs, floors.rs, preuves mutations | Clore après validation humaine de l'interlude et des cartes. |
| Enregistrements natifs | Métadonnées effectives et restart séparé corrigés, aller-retour exact | recording.rs, preuve recording | Correctif technique validé ; en p2p, capture des seuls inputs locaux reste une limite. |
| D28 / D29 / D34 / D40b / D50 / D54 | Décisions absentes ; pas de correction/rebalance autorisée au titre de ces dettes | lignes existantes | Garder ouvertes jusqu'aux décisions mot pour mot de William. |

## Validation finale du lot

Code du lot terminé à `12a35292`, en commits séparés sur `m1-revue-humaine`.
109 tests unitaires game et 50 tests de contrats/attentes de scénario passent ;
les 57 scénarios concernés passent ensuite avec `ALACOD_BLESS=0`, traces identiques
aux références corrigées. Les deux builds Mac avec `--features native` sont verts.
[Résultats détaillés](../../../tests/review-notes/m1-f53cd1ae-validation.json).
Aucun changement aux armes, profils de bots, dégâts ennemis, munitions ou difficulté
configurée du troisième étage.

Deuxième solo de validation lancé sur ce code, graine 123456, via `make throne
ARGS="--profile headless"` ; `ALACOD_RECORD=/tmp/alacod-m1-12a35292-solo-02.ron`,
`ALACOD_PERF_CSV=/tmp/alacod-m1-12a35292-solo-02.csv`, `RUST_LOG=warn`.
Ce lancement ne vaut pas une deuxième run humaine terminée. Les résultats et
les décisions seront ajoutés après le retour de William.
