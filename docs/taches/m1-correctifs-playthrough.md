# Correctifs proposés après la première run humaine M1

2026-10-10 — William, run solo `ced52082`, graine 123456, 11882 frames.
Sources : [carnet de revue](../../tests/review-notes/m1-f53cd1ae.md),
[preuves de la run](../../tests/review-notes/m1-f53cd1ae-run-01/README.md).

Plan accepté par William le 2026-10-10 : « Parfait revalue la taches et si toute est bon tu peux ty lancer ».
Cet accord autorise les correctifs ci-dessous, dont les choix de mutations entre étages.
Il ne remplace pas les décisions de gravité ni le verdict humain de sortie M1.
Commits distincts avec scénarios verts et preuve de traces si une trace bouge ; aucun changement à dettes.md.

## 1. Performance : mesure et correction en premier

Les budgets headless mesurent la simulation, pas la fluidité du rendu sur le MacBook.
Le profil `headless` utilisé pour jouer conserve le rendu ; il optimise le code workspace
à `opt-level=1`. D30 est réglée, mais cela ne prouve pas 60 images/s en jeu.

Pistes observées dans le code, à mesurer sans les présenter comme causes établies :

- Le local passe par `Session::SyncTest`. `ALACOD_CHECK_DISTANCE` vaut 2 par défaut :
  deux frames précédentes sont resimulées à chaque nouvelle frame pour vérifier les
  checksums. La première run montre des événements répétés trois fois dans les logs.
- Les apparitions/collisions de balles et dégâts ont de nombreux logs `info!`.
- Les collisions utilisent déjà une grille spatiale ; il faut profiler son coût,
  les candidats, allocations/tris et mises à jour plutôt que proposer une seconde grille
  sans mesure. Examiner aussi le rendu, les sons de tirs et les effets de dégâts/hit stop.

Procédure sur le même MacBook, même résolution, même séquence d'inputs, au calme :

1. Identifier les passages à forte densité de balles dans la run et compter les balles
   vivantes. Valider les réglages du rejeu ; le défaut de métadonnées de l'enregistreur
   est documenté au point 4. La copie temporaire sert au diagnostic, son identité à
   la session native n'est pas encore prouvée.
2. Instrumenter les temps réels de frames rendues, temps CPU de simulation/présentation
   et GPU si accessible ; enregistrer p50/p95/p99, nombre de frames >33 ms et densité
   de balles. Mesurer sans capture PNG ni compilation concurrente. Le FPS fixe de
   l'overlay d'une capture et les horodatages de logs ne valident pas cette cible.
3. Comparer logging courant / `RUST_LOG=warn`, puis vérification locale 2 / 0
   (`ALACOD_CHECK_DISTANCE`) comme expériences distinctes. La mesure à 0 isole le coût
   du vérificateur ; les validations synctest à 2/4 joueurs conservent leur filet.
   Mesurer aussi l'effet du profil release pour situer la marge liée à l'optimisation.
4. Profiler le passage qui ralentit, corriger le coût dominant identifié, puis reprendre
   exactement les mêmes mesures. Ne pas attribuer un gain au moteur si seul le mode
   de vérification ou le profil de compilation a changé.
5. Extraire un scénario de charge représentatif et une mesure de rendu reproductible,
   en complément de `bench_bullets`/`bench_horde`. Une charge de 500 projectiles prévue
   pour M2 complète ce contrôle ; elle ne remplace pas le cas humain de throne.

Cible proposée : 60 images/s dans les combats ordinaires et dans le passage humain
signalé ; p95 du temps de frame <=16,7 ms, p99 <=33,3 ms, sans séquences prolongées
de chutes. Rapporter les résultats avant/après et la configuration exacte. Validation :
traces identiques pour une optimisation pure, scénarios concernés verts, budgets de
simulation conservés, confirmation de William sur le passage dense.

## 2. Ennemis coincés : traiter le cas de fuite au coin

Cas candidat du diagnostic : pillard 220, étage 2, f2325 du rejeu, position
(581.99936,64.0485), 10 PV. Les logs natifs montrent Flee après f1902.
Son sprite déborde de 6 px dans la roche ; le collider est juste au bord de deux murs,
sans pénétration démontrée par cette sonde. Les tourelles stationnaires ne constituent
pas une preuve de blocage. Demander l'endroit vu par William pour confirmer le cas.

Créer un scénario ciblé « ennemi blessé fuit vers un coin » ; vérifier une sortie
déterministe du coin, la collision du corps et l'écart visuel du sprite. Corriger la
navigation de fuite si le blocage est reproduit, et l'alignement/marge visuelle si requis.
Ce cas dépasse la poursuite dans les chicanes traitée par D55. Vérifier les autres règles
de recul, les tailles de corps et les scénarios du clone ; preuve de traces obligatoire
pour tout changement de mouvement. Critère : aucune immobilisation persistante due au
coin dans le cas reproduit, et sprites lisibles près de la roche.

## 3. Mutations : choix entre les étages, accepté le 2026-10-10

Le choix s'ouvre actuellement à LevelUp, sans pause ; capture f720 avec un ennemi vivant.
Recommandation : conserver l'acquisition des niveaux/rads pendant le combat et présenter
les choix en attente à un moment sûr entre les étages, avant le prochain combat.
L’accord du 2026-10-10 retient cette recommandation, également pour le duo/en ligne.

Critères : aucun choix masquant un combat actif pour la solution entre étages ; aucun
niveau/choix perdu si plusieurs sont gagnés ; progression du duo cohérente et transition
possible après les choix. Scénarios de mutation et de progression concernés verts,
preuves de traces pour le changement de timing.

## 4. Enregistreur : rendre les preuves fiables

L'enregistrement natif throne utilise `RecordedSettings::default` avec game="zombies"
et sans floors/clocks/difficulty/mode/progression. Récupérer le jeu et les réglages réels
au démarrage de la session, y compris après restart. Tester un aller-retour d'une vraie
configuration throne : mêmes réglages, inputs et événements/traces rejoués. Examiner le
décalage observé de sélection de mutation (f1052 natif / f1053 rejeu) sans supposer qu'il
est uniforme. Le correctif ne doit pas altérer la partie jouée.

## Reprise de la revue humaine

Après le lot performance, refaire le passage dense avec William ; après les deux
correctifs de jeu, deuxième run humaine incluant restart et les points restants de la
fiche. En ligne si possible. Recueillir les gravités et décisions mot pour mot, puis
écrire le rapport de sortie M1 avec les lignes de dettes proposées à orch.
