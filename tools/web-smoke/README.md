# Recette multijoueur du site

Le test pilote deux invités isolés : invitation, admission avec la même version,
readiness, partie forcée via TURN, quelques entrées clavier, retour au salon,
deuxième session puis fermeture du salon. Il vérifie les candidats sélectionnés
et échoue sur les erreurs JavaScript, requêtes HTTP en échec ou logs de désync.
Les captures de gameplay sont écrites dans `/tmp`. Le test utilise un renderer
logiciel : ce n’est pas une mesure de performance sur les ordinateurs des amis.

```sh
cd tools/web-smoke
npm ci --ignore-scripts
SITE_ORIGIN=http://127.0.0.1:4174 \
ALLUMETTE_ORIGIN=https://allumette.estran.studio \
CHROMIUM_EXECUTABLE=/usr/bin/chromium npm run test:online
SMOKE_GAME=throne npm run test:online
```

Les variables d’environnement doivent être conservées pour chaque exécution.
`CHROMIUM_NO_SANDBOX=1` active explicitement ce mode dans un environnement de
recette sans sandbox utilisable. L’origine du site doit être autorisée par le
CORS Allumette. Le catalogue de cette preview doit désigner les builds testés
avec `online: true`; ne pas modifier le catalogue public avant la recette bêta.

Pour tester le transport indépendamment du jeu et du signaling, utiliser
`tools/webrtc-smoke` dans le dépôt Allumette. Deux contextes sur un même hôte
ne remplacent pas la validation depuis deux machines/réseaux différents.

Le solo se vérifie avec `npm run test:solo` et les mêmes variables d’origine et
de Chromium : lancement des deux clones, entrées clavier et R, absence d’erreurs
JavaScript/HTTP et aucune requête Allumette. Le lancement online sur Pages est
testé en retrouvant l’iframe par sa relation au document parent : Pages redirige
`/loader.html` vers `/loader` en préservant les paramètres.
