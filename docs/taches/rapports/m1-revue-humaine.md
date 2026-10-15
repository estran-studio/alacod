# M1 — revue humaine de throne

2026-10-10, William, MacBook. Branche `m1-revue-humaine`, base main `f53cd1ae`.
Movement-feel, bots après movement-feel, D55 et vague 0 M2 présents ; D30 déjà réglée.
Espace = objet actif, Q = blank. M2 est déjà lancée : vague 0 mergée, T10 en cours.

## Verdict de sortie M1

**Non à ce stade : revue humaine incomplète et constats encore ouverts.** Trois
enregistrements solo avec commandes sont archivés (11882, 39026 puis 5567 frames,
graine 123456). La première comporte deux passages jusqu'à `roi_rat`, la seconde
finit au troisième étage, la troisième nettoie cet étage, boss compris.
Le 2026-10-14 William valide le HUD et signale encore des ennemis immobiles ainsi
que des cercles rouges disproportionnés. Restart et décisions restent à consigner.
Les validations automatiques des correctifs ne remplacent pas ce dernier critère.
Ce qui empêche de prononcer la sortie de M1 :

- Ennemis immobiles encore signalés ; un cas identifié comme tourelle, les autres à localiser.
- Télégraphes boss/tourelle confondus avec une immense zone de dégâts ; lisibilité à corriger.
- Confirmation du ressenti après correctifs, restart humain et avis des huit points encore incomplets.
- Décisions D28/D29/D34/D40b/D50/D54, difficulté du troisième étage et relecture du plan M2 encore absentes.

## Constats triés

| Gravité | Où | Constat de William | Traitement et preuve |
|---|---|---|---|
| Bloquant | Avant de jouer, présentation des ennemis | « Premier probleme avant que je sais sur throne, on a pas assets pour les enemies je crois » ; « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies » | Onze ennemis visibles avec sprites provisoires attribués, commit `ced52082`. Scénarios verts, traces identiques hors ActiveLayers. La première run a ensuite pu être jouée ; distinction visuelle des silhouettes à juger. |
| À corriger, gravité humaine à confirmer | Run solo 01, passages denses | « Beaucoup beaucoup de frame drop quand il a bcp de bullets » | Coût dominant mesuré dans la géométrie du flow field ; optimisation pure `41d3859d`, 204 scénarios verts et traces inchangées. Comparaison détaillée ci-dessous ; ressenti à confirmer. |
| À corriger, gravité humaine à confirmer | Run solo 01, cavernes ; candidat pillard 220, étage 2, f2325 | « Des enemies restes coincé dans les murs au lieu de venir vers nous » | Fuite/recul utilisant des voisins franchissables et centre sûr du corps (`faa5120c`). Régression de la run verte ; sprite sorti du coin. 22 scénarios concernés, 14 traces modifiées avec preuve. L'emplacement vu par William reste à confirmer. |
| À corriger, gravité humaine à confirmer | Run solo 01, écran de mutation ; capture de diagnostic f720 | « On recoit les augments pendant qu'on est en plein combat » | Choix conservés en attente pendant le combat, présentés après nettoyage ; portail attend tous les joueurs, projectiles hostiles supprimés et dégâts résiduels neutralisés pendant l'interlude. Politique BetweenFloors réservée à throne en mode Floors. Validation et traces dans le dossier mutations. |
| Corrigé et validé humainement | 2026-10-14, HUD et textes | « Pour les HUD, il a bcp de overlap entre les elements » ; puis « HUD parfait. » | Correctif `3ae7d2e5`, revue visuelle acceptée le 2026-10-14 ; ne vaut pas validation des autres points. |
| À corriger, gravité proposée | 2026-10-14, troisième run ; cas diagnostiqué à l'étage 3, f5100 | « toujours zombie qui sont coicé sans bouger et qui ne regarde pas. » | Ennemi 461 confirmé comme tourelle stationnaire, qui tire ; orientation figée et silhouette zombie trompeuse. Signalement global ouvert, endroit des autres cas encore demandé. |
| À corriger, lisibilité, gravité proposée | 2026-10-14, troisième run, boss f4290 et tourelle f5160 | « le boss fait des immenses cercle rouge qui font toute la map,  es-ce que c'est normal ? Ca represente quoi ? » | Annonce de salve couronne (20 frames, huit projectiles), dessinée avec un rayon de portée de 400. Aucune explosion sur toute cette surface. Preuve dans la troisième run ; aucune modification de gameplay effectuée pour ce retour. |
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
| 3. Boss roi_rat | Boss rencontré et tué dans les runs 1 et 3 ; cercles rouges disproportionnés signalés dans la run 3, lisibilité ouverte. |
| 4. Écran de mutation | Constat précis recueilli, timing corrigé ; lecture des cartes/texte à confirmer. |
| 5. HUD | Validé par William le 2026-10-14 : « HUD parfait. ». |
| 6. Feedback | Constat de fluidité recueilli ; hit stop, secousse, flash/chiffres à juger séparément. |
| 7. Cavernes | Ennemis immobiles encore signalés dans la run 3 ; une tourelle identifiée, signalement global ouvert. Variété, roche et surfaces à juger. |
| 8. Restart / en ligne | Restart humain encore manquant ; réseau non essayé, selon disponibilité d'un partenaire. |

## Décisions datées

- 2026-10-10 — Visibilité : « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies ».
- 2026-10-10 — Plan des quatre correctifs, mutations entre étages incluses : « Parfait revalue la taches et si toute est bon tu peux ty lancer ».
- 2026-10-14 — HUD : « HUD parfait. ».

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
| Ennemis immobiles / orientation | Nouveau signalement humain malgré le correctif de recul ; dernier ennemi de run 3 identifié comme tourelle stationnaire, orientation figée et silhouette trompeuse | ai/pathing.rs, tourelle.ron, preuve run-03 | Garder ouvert ; distinguer blocage des mobiles et lisibilité des stationnaires, localiser les autres cas avant correction. |
| Télégraphe de salve | Immenses cercles boss/tourelle interprétés comme danger sur toute la carte, alors que couronne tire huit projectiles | feedback.rs, couronne.ron, preuve run-03 | Ouvrir une dette de lisibilité ; proposer une annonce près de l'émetteur et des directions de tir, sans modifier dégâts/portée. |
| HUD chevauchements / densité | Correctif 3ae7d2e5 accepté par William : « HUD parfait. », le 2026-10-14 | hud.ron, preuves hud et run-03 | Clore ce constat ; conserver les autres décisions ouvertes. |
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

## Mise à jour du 2026-10-13

Le lancement du 2026-10-11 est maintenant archivé comme
[deuxième run avec commandes](../../../tests/review-notes/m1-f53cd1ae-run-02/README.md),
39026 frames, 2025 segments d'inputs. Le lancement précédent 02 était sans commandes.
Sur 2466 mises à jour natives avec balles présentes : p95 17,29 ms, p99 17,70 ms,
deux mises à jour >33,3 ms, au plus 22 balles. Le filtre ne représente pas tous
les combats ; les samples complets sont conservés. Confirmation du ressenti,
constats des huit points, restart et décisions de William encore manquants.
Résultat extrait le 2026-10-14 : étage 3 à f2062, défaite observée à f3393 (résumé 3392 frames).

## HUD : demande et correctifs du 2026-10-14

William : « Pour les HUD, il a bcp de overlap entre les elements » ;
« il a bcp de pollisage possible aussi, il a bcp trop de texte et information aussi.
J'aimerais qu'on travailles les 2 points, ». Gravité proposée : à corriger.
Cet accord autorise le correctif de disposition et de densité d'information.

Rejeu actuel : inspecteur moteur ouvert, Weapon/Ammo sur la vie/rads, frame/FPS
sur les munitions, réserves répétées. Correctif de présentation `3ae7d2e5` : diagnostics
optionnels, cinq textes permanents et deux barres, effets contextuels, HUD masqué
pour les modals, cartes abrégées et panneau de fin avec durée lisible. Capture
640×360 et vue à quatre joueurs examinées ; événements de la partie humaine
identiques avant/après, scénarios concernés verts, aucune trace modifiée.

[Analyse, captures et vidéo](../../../tests/review-notes/m1-f53cd1ae-hud/README.md).
Confirmation visuelle reçue le 2026-10-14 : « HUD parfait. ».
Proposition à orch : clore le constat HUD (chevauchements/densité).
Les décisions de dettes et le verdict de sortie M1 restent ouverts.

## Troisième run et nouveaux constats — 2026-10-14

[Enregistrement, captures et diagnostic](../../../tests/review-notes/m1-f53cd1ae-run-03/README.md),
code `beaa7b9f`, graine 123456, 5567 frames. Étage 3 à f2296, portail ouvert après
nettoyage à f5381. La sonde existante `nav_probe` identifie le dernier ennemi comme
tourelle ; elle vise et tire, mais ne s'oriente pas vers le joueur. Les cercles
rouges sont reproduits visuellement et reliés au dessin des télégraphes de tir.
Les deux tests diagnostiques existants passent ; aucun code/contenu/trace modifié.
Mesures natives avec balles : p95 17,1618 ms, p99 17,5455 ms, zéro mise à jour
>33,3 ms sur 4359 samples, au plus 19 balles ; pas de nouveau verdict humain de fluidité.

Précision humaine du 2026-10-14, mot pour mot : « 2. ah ok je voais pas vraiment
les projectils arrivers apres , c'est quoi animation ? les balles viennent du ciel ou  ??? ».
Constat à corriger proposé, points 3/6 : les huit projectiles partent de l'ennemi
dans le plan de jeu, rendus comme de petits carrés noirs de 3,5 unités sans animation
dédiée, de la même couleur que les balles standard du joueur. La capture f4320 de
la troisième run montre la salve. Ajouter à la dette de lisibilité proposée :
différencier les tirs hostiles et rendre leur départ visible, en cohérence avec
l'annonce de salve. Aucun correctif de présentation appliqué à ce stade.
