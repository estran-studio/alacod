# SDK Allumette navigateur

`bascanada-allumette-browser-0.1.0.tgz` provient de `packages/browser-client` dans
le dépôt indépendant Allumette. Le site consomme ce paquet versionné via `file:`;
aucun accès au registre npm ni chemin absolu vers le dépôt Allumette n'est requis.

Pour régénérer après un changement du SDK : depuis Allumette,
`npm pack ./packages/browser-client --pack-destination <alacod>/website/vendor --ignore-scripts`,
puis depuis le site `npm install ./vendor/bascanada-allumette-browser-0.1.0.tgz --ignore-scripts`.
Augmenter la version pour une nouvelle livraison. Ne pas modifier silencieusement
un paquet déjà publié. Cette première version locale n'est pas publiée sur npm.
