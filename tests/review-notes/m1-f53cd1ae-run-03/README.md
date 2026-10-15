# Troisième run humaine — test du HUD, lancement 04

2026-10-14, code `beaa7b9f`, graine `123456`, solo throne/Floors,
5567 frames. `original.ron` est une copie intacte de
`/tmp/alacod-m1-beaa7b9f-solo-04.ron` ; SHA-256 dans `summary.json`.
William valide le HUD mot pour mot : « HUD parfait. ».

Le rejeu entre à l'étage 2 à f698 puis à l'étage 3 à f2296. Le portail du
troisième étage s'ouvre à f5381 : cet étage, boss compris, a été nettoyé.
Choix de mutations à f561, f2136 et f5473, après ouverture des portails.
Pas de défaite ni de restart dans cet enregistrement. Ces événements ne
remplacent pas les décisions humaines sur difficulté, mutations et sortie M1.

## Ennemis immobiles : signalement toujours ouvert

William : « toujours zombie qui sont coicé sans bouger et qui ne regarde pas. ».
Gravité proposée : à corriger ; endroit précis encore demandé à William.

`nav_probe` confirme qu'à f5100 le dernier ennemi, 461 à (952,504), est une
**tourelle**, immobile par configuration. Le joueur est à 120 unités, visible,
une cible existe et le point de guidage est libre. Le rejeu émet ses télégraphes
de tir à f4726, f4867, f5008, f5149 et f5290. Ce cas ne démontre pas un blocage
de navigation. Le sprite provisoire zombie ne communique pas le rôle de tourelle.

L'orientation dans `ai/pathing.rs` dépend du déplacement ; les ennemis
stationnaires quittent le système avant cette mise à jour. Cela explique
l'orientation figée de cette tourelle. Le constat global de William reste
ouvert : cette identification ne prouve pas que tous les ennemis vus étaient
des tourelles.

Le diagnostic `nav_stats` compte 19 ennemis, 16360 frames-ennemi et aucun
chevauchement de sprite au-delà de sa tolérance de 3 px (maximum mesuré 1,4 px).
Il classe pourtant la tourelle comme « bloquée » via l'ancien état `Chasing` :
ce compteur n'est pas une preuve du comportement effectivement sélectionné.
L'autre candidat n'est immobile que deux frames. Logs et sonde conservés.

## Cercles rouges : annonce de tir disproportionnée

William : « le boss fait des immenses cercle rouge qui font toute la map,
 es-ce que c'est normal ? Ca represente quoi ? ».
Gravité proposée : à corriger, lisibilité du feedback.

Les captures [boss f4290](telegraphe-f4290.png) et
[tourelle f5160](telegraphe-f5160.png) reproduisent les grands cercles.
`couronne.ron` annonce pendant 20 frames (environ 0,33 s), puis émet huit
projectiles en cercle. Boss et tourelle utilisent ce pattern.
`feedback::telegraph_shape` utilise la portée maximale des projectiles de
l'arsenal, 400 unités, pour dessiner le cercle extérieur et un cercle intérieur
qui grandit. Il n'applique aucun dégât de zone à toute cette surface.
La charge du boss a une annonce distincte de rayon 40 pendant 30 frames.

Proposition de présentation : annoncer la salve près de l'émetteur et indiquer
les directions des tirs ; garder une zone de danger pour une attaque qui
touche effectivement cette zone. Aucun changement de code/contenu/trace effectué
pour ces nouveaux constats.

Précision de William, 2026-10-14 : « 2. ah ok je voais pas vraiment les projectils
arrivers apres , c'est quoi animation ? les balles viennent du ciel ou  ??? ».
La [salve à f4320](salve-f4320.png) montre les petits carrés noirs autour du boss.
`emitter_system` les crée à la position de l'émetteur ; `Ring` répartit les huit
directions sur 360 degrés, avec une vitesse de 60 unités/s. `spawn_bullet` dessine
un carré noir de 3,5 unités pour ce projectile standard, sans animation dédiée,
comme les balles standard du joueur. Ils voyagent dans le plan de jeu, sans chute
du ciel. La visibilité des tirs est donc à traiter avec celle du télégraphe.

## Preuves et reproduction

- `replay-events.json` : événements extraits avec le renderer existant.
- `probe-461-5100.json` et `.log` : sonde existante, test vert.
- `nav-stats.log` : diagnostic existant, test vert ; fixture temporaire retirée.
- `performance.csv.gz` : mesures natives complètes ; percentiles dans `summary.json`.
  Cadence applicative, pas mesure GPU ; aucun nouveau verdict humain de fluidité.
- Captures tous les 30 frames : analyse visuelle, sans mesure de performance.

Reproduction de la sonde depuis la racine, avec le runner de scénario compilé :

```sh
ALACOD_PROBE=461:5100 \
ALACOD_PROBE_FILE=tests/review-notes/m1-f53cd1ae-run-03/original.ron \
target/headless/build/scenario/c0a99deedfd607f6/out/scenarios-c0a99deedfd607f6 \
  --exact nav_probe --ignored --nocapture
```

Le chemin du binaire dépend du build ; le test source est `nav_probe` dans
`crates/scenario/tests/scenarios.rs`.
