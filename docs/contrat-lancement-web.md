# Intégration navigateur Alacod / Allumette

2026-10-09. Découpage retenu : Allumette versionne ses contrats génériques dans
`docs/browser-sessions-v1.md` de son dépôt; Alacod implémente un adaptateur client.

## Solo livré dans cette branche

Catalogue éditorial : `website/src/lib/game/catalog.ts`. Disponibilité réelle :
`website/static/releases.json`, généré après build des clones. Un clone non disponible
ne propose pas de bouton de lancement. Pas de JWT, salon ou requête ICE en solo.

`/play?id=zombies|throne` vérifie le release et le build.json. Un clic lance `/loader.html`
avec les seuls paramètres publics jeu/build. Le chargeur importe le module versionné,
configure un joueur local et relaie les erreurs/initialisation au parent en postMessage.
Le parent vérifie origine et source de l'iframe. Les assets utilisent la racine absolue
`/builds/<buildId>/<game>/assets`; les définitions RON/LDtk nécessaires au registre sont
embarquées par le build script, les images/sons restent HTTP. Les deux sources passent le
même parser/lint. Le hot reload de contenu reste une capacité native.

## Multijoueur : prochaine intégration

Le SDK Allumette doit fournir un objet de session versionné. L'adaptateur Alacod traduit :

- Jeu/build/compatibility_id vers un build du catalogue vérifié.
- URL WSS **exacte** et ticket court vers le socket; jamais de suffixe de restart sur le token.
- ICE vers une ressource consommée par le socket, en tenant compte de son API mono-config.
- Identité locale et mapping identité/PeerId vers les configs GGRS, sans associer les noms
  par indice d'un HashSet ou d'une liste distante.
- Fin de partie vers fermeture de session/retour au salon; nouvelle partie = nouvelle session.

Les credentials restent en mémoire ou dans un objet transmis par une API/postMessage de
même origine validée, sans URL de navigation. Ne pas ouvrir un salon avec deux builds distincts.
Les valeurs du moteur (seed, inputs, simulation) restent à la charge d'Alacod.

Le client engine verrouille Matchbox `0.15.0` d'un fork, le serveur Allumette utilise le
protocole `0.13.0`. Vérifier les formats et l'interop effective avec des peers WebRTC; aucun
alignement de version présumé depuis des noms de crates proches.

## Preuves attendues

Tests de parité des registres natifs/embarqués sur les deux clones; typecheck/build du site;
chargement navigateur et assets sans 404; solo sans Allumette; défaite/relance.
Pour online : isolation serveur multijeu, compatibilité/refus, handles cohérents, duo sur
réseaux différents, TURN relay forcé, EOF SSE et deuxième session. Les boutons online restent
désactivés jusqu'à ce que le contrat et cette recette soient réalisés.
