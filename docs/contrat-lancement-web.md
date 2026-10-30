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
désactivés jusqu'à ce que le contrat et cette recette soient réalisés. (État au 2026-10-30 : la prod
`v0.0.2-4630291a` publie `online: true` sans cette recette ; elle est jouée le 2026-10-31, section
« Service en production ».)

## Intégration réalisée après le commit de base

Commits de reprise : Alacod `6af4e60`, Allumette `e7e9f06`, tous deux sur `main`
(poussés depuis : Allumette déployé le 2026-10-10 en `23524d8`, Alacod publié en `v0.0.2-4630291a`).

- Allumette expose `/browser/v1/lobbies`, invitations, join, ready, start et session.
  Les salons sont privés, capacity 2 ou 4, compatibility_id opaque et membres figés
  pendant une session. Le site ne propose initialement que deux joueurs.
- Les invitations expirent après 15 minutes; une nouvelle invitation révoque la
  précédente. Les données en RAM sont perdues au redémarrage.
- Chaque start crée une session distincte; connexion bornée à 120 s et session à
  30 minutes. Les tickets de signaling expirent après 60 s et portent une audience
  spécifique : ils ne remplacent pas les JWT d’identité des endpoints HTTP.
- `/session` donne les participants et leur PeerId lorsqu’il est attribué. Le jeu
  attend ce mapping avant d’associer les handles GGRS aux identités.
- SDK indépendant `@bascanada/allumette-browser` 0.1.0 dans Allumette, consommé
  comme archive versionnée dans `website/vendor`. Aucune publication npm effectuée.
- Auth invité Ed25519 via WebCrypto, sans wallet ni persistance de credentials.
  L’identité est temporaire; reload = nouvel invité. Les identités existantes du
  SDK historique ne sont pas migrées vers ce client.
- Le site utilise un polling borné à la durée de présence de la page online, avec
  AbortController. L’ancien SDK conserve le SSE; ce nouveau client ne le requiert pas.
- Invitations dans le fragment du lien, retiré après lecture; JWT et tickets ne
  passent que par HTTP authentifié ou postMessage de même origine/source vérifiées.
- Matchbox accepte une seule configuration ICE. L’iframe adapte son constructeur
  RTCPeerConnection pour fournir toute la liste STUN/TURN avec credentials, sans
  modifier le fork Matchbox. Cet adaptateur doit être revérifié à chaque migration
  du binding WebRTC; il est isolé du site hôte et absent du mode solo.
- Le retour/restart d’une partie navigateur Allumette repasse par le salon pour un
  nouveau ticket; aucun suffixe n’est ajouté au JWT.

Recette actuelle : tests serveur, SDK, typecheck/build et parcours navigateur.
La preuve de connexion WebRTC et de simulation, puis TURN forcé sur réseaux distincts,
reste le critère pour autoriser `online: true` dans un catalogue publié. Les builds
`preview` locaux peuvent être activés pour cette recette; le catalogue commité demeure vide.

## Preuve WebRTC du 2026-10-09

Transport TURN authentifié validé en UDP et TCP depuis deux Chromium 154 sur
Debian 13, avec échange bidirectionnel et candidats `relay` sélectionnés.
Les deux clones `rtc-fix-20261009` passent le parcours réel du site : invités,
invitation, readiness, deux sessions successives, retours et fermeture du salon;
aucune erreur JavaScript/HTTP ou désync observée. Capture et commandes dans
`docs/captures/multijoueur` et `tools/web-smoke`. Le signaling en production
interagit donc effectivement avec le client Matchbox, pas seulement avec un fixture.

Le canal GGRS navigateur est protégé contre la fermeture du socket Matchbox :
la fermeture demande un retour au salon plutôt qu’un envoi sur un canal fermé
qui provoquerait un panic. Cette garde repose sur le runtime WASM monothread actuel.

Cette preuve utilise deux contextes sur le même hôte Linux. Le Mac de recette
passe par un tunnel et échoue même sans engine; la comparaison hors VPN et
la preuve sur deux réseaux restent ouvertes. Le catalogue online publié reste
soumis à cette recette; seuls les builds locaux de test sont activés.

## Service en production (S1-3, 2026-10-30)

Rapport : `docs/taches/rapports/polissage-reseau-amis.md` (§ 1 état des lieux du 28 oct, sondes rejouées le 30).

- **URL** : API et signaling `https://allumette.estran.studio` / `wss://allumette.estran.studio` (Caddy, Let's Encrypt),
  STUN/TURN `turn.estran.studio:3478` UDP et TCP (coturn 4.6.3, identifiants temporaires servis par `/ice-servers` ;
  TURNS 5349 fermé). Site : `https://alacod.estran.studio` (Cloudflare Pages) ; CORS : ce domaine et `alacod.pages.dev`,
  **pas** `localhost` (un site lancé en local doit parler à un allumette local).
- **Opérateur** : William. Dépôt `estran-studio/allumette` (gelé), image `ghcr.io/estran-studio/allumette-server:<sha>`,
  hôte alumette (`/opt/allumette/compose.yaml` et `.env`, hors dépôt). Version en service : `23524d8` (2026-10-10),
  protocole Matchbox 0.13.0, filaire identique au client 0.15 du fork.
- **Redéployer** : un push sur `main` d'allumette (workflow « Allumette CI and Deploy »), ou
  `gh run rerun 38068410753 -R estran-studio/allumette`. **Redémarrer** : sur alumette, `cd /opt/allumette && sudo docker
  compose restart allumette`. Salons et sessions vivent en mémoire : jamais pendant une partie. Journaux :
  `sudo docker compose logs --since 1h` dans `/opt/allumette`.
- **Durées** : invitation 15 min ; connexion 120 s après « Lancer » ; **session 30 min** (`SESSION_TTL`, décision de
  William du 2026-10-29 : gardé) ; ticket de signaling 60 s. À l'expiration de la session, le serveur ferme le socket
  (tâche de 5 s) : le canal du jeu se ferme, `BrowserChannel` demande le retour au salon et la page, qui voit la session
  effacée à son prochain sondage (1 s), retire le jeu ; la partie en cours est perdue, sans écran figé.
- **Ticket de 60 s** : le site le reçoit au « Lancer », le jeu le lit au démarrage du wasm et ouvre le socket après le
  chargement des assets (`OnEnter(LobbyOnline)`). Un premier chargement lent (≈ 11 Mo de wasm compressé, ≈ 8 Mo
  d'assets) peut le périmer : socket refusé, retour au salon (correctif du 2026-10-30) ou, sur le build de prod, écran
  d'attente jusqu'à l'effacement de la session (120 s). Correctif du site préparé (rapport, § 2.4), appliqué si la
  recette le montre.
- **Recette** : protocole en dix lignes dans `docs/jouer.md` (« En ligne depuis le site »).
- **Natif** : `--allumette` parle l'ancienne API `/lobbies`, ne garde que la première entrée ICE (le STUN, donc jamais
  TURN : D165) ; hors de la sortie (site, Chrome).
