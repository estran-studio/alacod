# Revue humaine M1 — William

Date : 2026-10-10 (America/St_Johns). Base : `origin/main`, `f53cd1ae`.
Branche : `m1-revue-humaine`.

Première run humaine terminée ; correctifs autorisés le 2026-10-10, deuxième run et décisions de sortie à recueillir.
Version revue : movement-feel, m1-bots-apres-movement-feel, D55 et vague 0 de M2 inclus.
D30 réglée ; Espace = objet actif, Q = blank. Les vidéos du digest précèdent movement-feel.

## Parties

À renseigner pour chaque run : jeu, mode, graine, fichier d'enregistrement, frames utiles,
étage atteint, boss rencontré, restart essayé. Deux runs throne requis, dont un jusqu'à roi_rat.
Vérification des armes zombies également requise.

- 2026-10-10 — Run throne solo 01 lancée sur `ced52082`, graine initiale `123456` (manifeste), enregistrement configuré `/tmp/alacod-m1-ced52082-solo-01.ron` (écrit à la fermeture), log `/tmp/alacod-m1-ced52082-solo-01.log`. Étage atteint, boss, restart et constats à recueillir ; le lancement ne vaut pas validation humaine.
- 2026-10-10 — Run solo 01 fermée et enregistrée : 11882 frames. Logs : étage 2 à f1477, étage 3 à f3108, portail du troisième étage ouvert à f6550 et boucle suivante à f6755 ; deuxième passage du boss, roi_rat-1068 tué à f9604. Run jusqu'au boss réalisée ; deuxième run humaine et restart encore à faire.

- 2026-10-10 — Run throne solo 02 de validation lancée sur le code `12a35292`, graine `123456`, `make throne ARGS="--profile headless"`, enregistrement `/tmp/alacod-m1-12a35292-solo-02.ron`, métriques `/tmp/alacod-m1-12a35292-solo-02.csv`, log `/tmp/alacod-m1-12a35292-solo-02.log`. Aucun constat ou résultat humain encore recueilli pour cette seconde partie.

## Constats de William

Une ligne par constat : date | point 1–8 | où (run, graine, frame) | quoi | gravité
(bloquant / à corriger / idée) | preuve ou reproduction.

Points à couvrir : 1. armes throne et zombies après D51 ; 2. troisième étage ; 3. roi_rat ;
4. mutations ; 5. HUD ; 6. feedback ; 7. cavernes ; 8. restart et en ligne si possible.

- 2026-10-10 | points 3/7, présentation des ennemis | throne, avant les runs, base f53cd1ae (graine/frame sans objet) | William : « Premier probleme avant que je sais sur throne, on a pas assets pour les enemies je crois » ; absence confirmée : assets/assets.yaml documente des ennemis sans sprite, sprites/sprites.ron ne déclare aucun ennemi, rat et roi_rat ont des skins aux layers vides | bloquant | preuve statique ; correction explicitement demandée par William, commit séparé et validation des scénarios/traces requis.
- 2026-10-10 | suivi du constat de visibilité | correctif ced52082 | onze ennemis et tous leurs skins raccordés à des sprites zombie provisoires animés et à une ombre ; captures rat/boss visibles, lint vert, 39 scénarios générés et 9 manuscrits verts ; solo/duo/quad identiques hors ActiveLayers | bloquant initial corrigé techniquement, confirmation visuelle de William encore attendue | preuve : m1-f53cd1ae-visibilite.md ; la distinction des silhouettes reste à juger.
- 2026-10-10 | point 7, cavernes/navigation | run solo 01, ced52082, graine 123456, étage/frame à préciser | William : « Des enemies restes coincé dans les murs au lieu de venir vers nous » | à corriger (provisoire, gravité à confirmer par William) | enregistrement et logs conservés ; collider bloqué et sprite chevauchant la roche à distinguer ; D55 ne suffit pas à écarter ce constat.
- 2026-10-10 | point 4, écran de mutation | run solo 01, graine 123456 ; première montée de niveau f694, choix résolu f1052 | William : « On recoit les augments pendant qu'on est en plein combat » | à corriger (provisoire, gravité à confirmer par William) | progression.rs documente le choix à LevelUp sans pause de simulation ; décision attendue sur choix entre étages ou pause pendant le choix.
- 2026-10-10 | points 6/7, fluidité | run solo 01, graine 123456, moments précis à identifier | William : « Beaucoup beaucoup de frame drop quand il a bcp de bullets » | à corriger (provisoire, gravité à confirmer par William) | ressenti humain établi ; nombre de balles et temps de frame non encore mesurés ; distinguer simulation, rendu et feedback/hit stop avant d'attribuer une cause.
- 2026-10-10 | enregistrement/reproduction, constat technique de l'agent | run solo 01, fichier original de 11882 frames | RecordedSettings::default conserve game="zombies" et omet floors/clocks/difficulty/mode/progression dans l'enregistrement natif throne | à corriger (proposition technique à soumettre à William) | original intact ; copie temporaire de diagnostic à préparer avec métadonnées du manifeste throne ; défaut corrigé ensuite avec autorisation du 2026-10-10 (57de6173).
- 2026-10-10 | reproduction du point 4 | copie temporaire de diagnostic, f720 | capture montrant les trois cartes de mutation alors qu'un ennemi est vivant et que le combat continue | à corriger (gravité humaine encore à confirmer) | m1-f53cd1ae-run-01/mutation-f720.png ; première sélection à f1053 dans le rejeu contre f1052 dans le log natif, aucune preuve d'identité complète du rejeu au natif.
- 2026-10-10 | diagnostic du point 7 | copie temporaire, étage 2, pillard 220, f2325, position (581.99936,64.0485) | sprite débordant de 6 px dans la roche ; collider 20×20 décalé de -6 en y juste au bord des murs, sans pénétration démontrée à cette frame ; vitesse (19.42,-67.25) et Flee retenu dans les logs natifs après f1902 | à corriger (gravité humaine encore à confirmer) | sonde pillard-220-f2325.log/json ; piste de blocage au coin pendant la fuite, distincte de la poursuite corrigée par D55 ; à confronter à l'endroit vu par William.

## Décisions de William

Transcription mot pour mot avec date : D28, D29, D34, D40b (reliquat D40 dans dettes.md),
D50, rééquilibrage du troisième étage, D54, relecture du plan M2 (vague 0 mergée, T10 en cours).
- 2026-10-10 — Visibilité des ennemis : « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies ». Les runs humains sont suspendus jusqu'au correctif de visibilité ; les vérifications automatiques du correctif restent nécessaires.
- 2026-10-10 — Après correctif, appréciation générale de William : « Ca marche assez bien generalement mais problemes ». Revue reprise ; trois problèmes consignés ci-dessus, sans décision de gravité définitive à cette étape de la revue.

- 2026-10-10 — Plan des quatre correctifs accepté : « Parfait revalue la taches et si toute est bon tu peux ty lancer ». Cet accord inclut le report des choix entre étages ; il ne vaut pas acceptation de M1.

Décisions D28/D29/D34/D40b/D50/D54, difficulté du troisième étage et plan M2 encore à recueillir.
Verdict de sortie M1 à recueillir après la revue.

## Suivi des correctifs autorisés — 2026-10-10

- 2026-10-10 | points 6/7, fluidité | rejeu diagnostique de la run solo 01, f3200–5600, au plus 28 balles vivantes | flow field optimisé ; cadence applicative p99 98,15 → 17,63 ms, aucune mise à jour >33,3 ms dans le passage optimisé ; simulation moyenne 9,31 → 1,27 ms avec passage du synctest local 2 à 0, gain du moteur isolé également à distance 2 | à corriger initial, validé techniquement, ressenti humain à confirmer | mesures avant/après et 204 scénarios/traces identiques dans m1-f53cd1ae-performance/README.md ; pas une mesure GPU ni une charge de 500 balles.
- 2026-10-10 | point 7, navigation | pillard 220 à f2325, rejeu à timing historique | recul limité aux voisins franchissables et recentrage sûr ; sprite hors des deux murs à (575.96277,64.99998) | à corriger initial, cas reproduit corrigé, endroit vu par William à confirmer | faa5120c ; 22 scénarios concernés, 14 traces modifiées avec preuves, quatre tests de recul et régression de la run.
- 2026-10-10 | point 8, reproduction | session native et restart | nom du jeu, mode, étages, horloges, difficulté, progression et carte/graine initiales capturés ; restart archive la partie précédente dans un fichier distinct | à corriger technique résolu | 57de6173 ; aller-retour frame par frame et test d'archivage verts.
- 2026-10-10 | point 4, mutations | correctif 12a35292 | choix entre étages, niveaux cumulés conservés, attente de tous les joueurs et protection pendant la lecture ; duo avec trois niveaux gagnés d'un coup validé | à corriger initial, corrigé techniquement, lecture des cartes à confirmer | m1-f53cd1ae-mutations/README.md ; sept traces mises à jour avec preuve du premier écart à f303 ; autres jeux/modes gardent le choix immédiat.
- 2026-10-10 | point 8, restart | smoke natif Mac rendu | deux fichiers distincts après restart, 92 puis 60 frames d'inputs, bons réglages throne et graine ; traces GGRS exportées | à corriger technique résolu ; test humain du bouton encore manquant | m1-f53cd1ae-recording/native-smoke*.ron et trace-g1/g2.
