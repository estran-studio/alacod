# polissage-reseau-amis — rapport (S1-3)

Fiche : `docs/taches/polissage-reseau-amis.md`. Branche : `polissage-reseau-amis`.

## 1. État réel d'Allumette (préparation, 28–29 oct)

Préparation de S1-3 faite le 2026-10-28 depuis le VPS `ovh-inner`, **en lecture seule** : aucun changement d'alacod ni
d'allumette, aucun redéploiement, aucun secret écrit (jetons, identifiants TURN et défis masqués dans les transcripts).
Dépôt allumette cloné en lecture dans `/home/debian/orca/workspaces/allumette-lecture` (`23524d8`, = `origin/main`).

**Verdict en une ligne : le service tourne et il est bon pour S1-3.** Site, API, signaling et TURN répondent, le
protocole Matchbox est le même au fil près. Rien n'est à redéployer avant J3. Quatre écueils pour la recette :
(a) une session navigateur est **coupée au bout de 30 min** (`SESSION_TTL`) ; (b) un client natif n'utilise **jamais**
TURN ; (c) le ticket de signaling vit **60 s** et il est délivré avant le téléchargement du wasm, ce qui est risqué sur
une connexion lente ; (d) **pas d'accès au serveur** pour lire ses journaux si la recette échoue.

| # | Point | Issue |
|---|---|---|
| 1 | Service vu de l'extérieur | **vérifié** |
| 2 | Version Matchbox serveur/client | **vérifié** (lecture des sources) ; interop de bout en bout : prouvée le 2026-10-09, à rejouer à J3 |
| 3 | Déploiement | **vérifié par le dépôt et les journaux de CI** ; état du conteneur : **bloqué par l'accès SSH** |
| 4 | TURN | **vérifié** (allocation authentifiée UDP et TCP) |
| 5 | Secrets (noms) | **vérifié par le dépôt** ; contenu de `/opt/allumette` : **bloqué par l'accès SSH et Docker** |
| 6 | Flux site → jeu | **vérifié** (lecture) |
| 7 | Plan J3–J5 | écrit ci-dessous |

### Accès manquants (signalés le 28 oct, carte `--unread` et `status` au run)

1. **SSH depuis `ovh-inner` vers alumette** (`debian@vps-eda50ee1.vps.ovh.ca`) : `Permission denied (publickey,password)`.
   Option la plus simple pour William : depuis le Mac, ajouter la clé publique d'ovh-inner
   (`/home/debian/.ssh/id_ed25519.pub`, commentaire `orca-orch`, `SHA256:d4L/72uVlZQCYRHK9sWy/qSH1eAOxM94Yf9BcEUMYXQ`)
   à `~debian/.ssh/authorized_keys` sur alumette.
2. **Docker sur alumette** : `debian` n'a pas le socket. Option : `sudo usermod -aG docker debian`, ou une règle sudoers
   limitée à `docker ps`, `docker logs` et `docker inspect`.

**Ces accès ne bloquent pas S1-3** : le service tourne et le redéploiement passe par la CI. Ils ne servent qu'à lire les
journaux d'allumette et de coturn si la recette à deux réseaux échoue. Sans eux, ce diagnostic revient à William, depuis le Mac.

### 1.1 Service vu de l'extérieur — vérifié

DNS et TLS :

```
$ getent ahosts allumette.estran.studio ; host turn.estran.studio
158.69.192.54   STREAM allumette.estran.studio          (turn.estran.studio : même IP)
$ openssl s_client -servername allumette.estran.studio -connect allumette.estran.studio:443 | openssl x509 -noout -issuer -dates
issuer=C=US, O=Let's Encrypt, CN=YE2      subject=CN=allumette.estran.studio
notBefore=Oct  9 19:38:47 2026 GMT        notAfter=Jan  7 19:38:46 2027 GMT     (Caddy, renouvellement automatique)
```

Ports de `158.69.192.54` : 80 (308 vers https), 443, 3478 et 22 ouverts ; 3536 (allumette nu) et 5349 (TURNS) fermés.

Routes du contrat (`website/vendor/…allumette-browser-0.1.0.tgz`, `index.js` ; serveur `src/lib.rs:220`, `src/browser.rs`).
Les appels **sans jeton** :

```
GET  /health                                   200 (corps vide)
GET  /                                         400 « Connection header did not include 'upgrade' » (via: 1.1 Caddy)
GET  /lobbies                                  200 []            (API historique, liste publique)
GET  /ice-servers                              401 {"error":"Invalid token"}
GET  /browser/v1/lobbies                       401 {"error":"Invalid token"}     (POST : idem)
POST /browser/v1/lobbies/<uuid>/invitations|join|ready|start   401 {"error":"Invalid token"}
GET  /browser/v1/lobbies/<uuid>/session        401 {"error":"Invalid token"}     (id non UUID : 400 « UUID parsing failed »)
WS   /<jeton invalide> et WS /                 401 Unauthorized
POST /auth/challenge                           200 {"challenge":"<32 car.>"}
```

Puis **avec un invité éphémère** (clé Ed25519 jetable, défi signé, comme `loginGuest` du SDK ; script en annexe A) :

```
login 200 token: <JWT masqué>
ice-servers 200 [{"urls": ["stun:turn.estran.studio:3478"]},
                 {"urls": ["turn:turn.estran.studio:3478?transport=udp", "turn:turn.estran.studio:3478?transport=tcp"],
                  "username": "<masqué>", "credential": "<masqué>"}]
browser/v1/lobbies 200 []        lobbies 200 []     (aucun salon ouvert le 28 oct)
```

Le CORS (`ALLOWED_ORIGINS` explicite) accepte `https://alacod.estran.studio` et `https://alacod.pages.dev` (preflight 200,
`authorization,content-type`, `GET,POST,DELETE,OPTIONS`). Il **refuse `http://localhost:5173`** : un site lancé en local
(`npm run dev`) ne peut pas parler à l'allumette publique. Pour tester le site en local, il faut un allumette local.

Je n'ai pas créé de salon ni de session sur le serveur public : les points d'entrée et le refus sans jeton suffisent à
l'état des lieux, et un vrai parcours relève de la recette J3–J4 (point 7).

Le site public est servi par Cloudflare Pages : `https://alacod.estran.studio/releases.json` annonce
`v0.0.2-4630291a` (build du 2026-10-18, `engineCommit 4630291aaffd`), `online: true` pour zombies et throne. Le
`releases.json` du dépôt est **en retard** sur la prod (`rtc-fix-20261009`). Le `loader.js` déployé est identique à celui
du dépôt (md5 `aa5bc59c…`).

### 1.2 Version Matchbox — vérifié (sources) ; interop à rejouer

- Serveur : `allumette/Cargo.toml:16-17` `matchbox_signaling`/`matchbox_protocol = "0.13"` ; `Cargo.lock` : **0.13.0**
  (crates.io). Image déployée : `ghcr.io/estran-studio/allumette-server:23524d87…` (point 3), construite `--locked` sur ce lock.
- Client alacod : fork `AdamWhitehurst/matchbox` rev `593f900c` (`crates/game/Cargo.toml:14,17`) ; `Cargo.lock` :
  `matchbox_socket`/`matchbox_protocol` **0.15.0**.
- **Protocole filaire identique** : j'ai comparé `matchbox_protocol/src/lib.rs` de 0.13.0 (crates.io) et du fork 0.15. Seuls
  deux commentaires diffèrent (`IdAssigned`/`NewPeer` : « Includes the PeerId… »). Types, variantes et sérialisation JSON
  sont les mêmes. Le serveur relaie le `data` des `Signal` sans le lire (`src/topology.rs:140-146`) : l'offre et la réponse
  WebRTC du client 0.15 passent telles quelles.
- **Prouvé le 2026-10-09** (`docs/contrat-lancement-web.md`, « Preuve WebRTC du 2026-10-09 ») : deux Chromium sur le même
  hôte, TURN UDP et TCP avec candidats `relay`, deux sessions successives, `rtc-fix-20261009`.
- **Plus prouvé** : le build en ligne `v0.0.2-4630291a` (18 oct) n'a jamais joué en ligne ; deux réseaux distincts jamais
  essayés ; le client natif `--allumette` contre l'API `/browser/v1` jamais essayé (il parle l'API historique `/lobbies`).

### 1.3 Déploiement — vérifié par le dépôt et la CI ; serveur bloqué par l'accès

- **Image** : `Dockerfile` (rust 1.90 bookworm, `cargo build --release --locked`, binaire `allumette_server`, port
  3536 dans le conteneur).
- **Workflow** `.github/workflows/main.yaml` (« Allumette CI and Deploy »), lancé à chaque push sur `main` :
  - `make test` et `make format` ;
  - build et push de `ghcr.io/estran-studio/allumette-server:<sha>` (jeton `github.token`) ;
  - job `deploy` : `appleboy/ssh-action` vers `secrets.VPS_HOST`/`VPS_USER`/`VPS_SSH_KEY`, puis
    `sudo … cd /opt/allumette`, réécriture de `ALLUMETTE_IMAGE=` dans `.env`, `docker compose pull allumette`,
    `docker compose up -d --no-deps allumette`, `curl --fail https://allumette.estran.studio/health`.
  - `release.yaml` (tags `v*`) publie aussi l'image, mais ne déploie pas.
- **Version qui tourne** : la dernière exécution réussie est le run `38068410753`, commit `23524d8` du 2026-10-10. Rien n'a
  été poussé sur `main` depuis (`pushedAt 2026-10-10T16:37:18Z`). Extrait de son journal :

  ```
  out: NAME                    IMAGE                                                                             STATUS
  out: allumette-allumette-1   ghcr.io/estran-studio/allumette-server:23524d87f9015336f2b336dbb7172198b4535c73   Up Less than a second   3536/tcp
  ✅ Successfully executed commands to all host.
  ```

  Je n'ai pas pu confirmer que ce conteneur tourne encore à l'identique (`docker ps` : accès). `/health` répond 200, et le
  CORS comme l'`/ice-servers` correspondent au code de `23524d8`.
- **Mise en place sur l'hôte** : `/opt/allumette/compose.yaml` et `/opt/allumette/.env`, **absents du dépôt** (créés à
  la main). Caddy est en frontal TLS (`via: 1.1 Caddy`). Coturn 4.6.3 répond sur 3478 (point 4). On ne sait pas si Caddy
  et coturn sont dans le même compose : il faut l'accès pour le lire.
- **Opérateur** : William (`wquintal`, auteur des commits et acteur GitHub des déploiements). Le dépôt est **gelé**.
- **Redémarrer**, sur alumette, avec sudo : `cd /opt/allumette && sudo docker compose restart allumette`.
- **Redéployer** : un push sur `main` d'allumette, ou relancer le dernier run :
  `gh run rerun 38068410753 -R estran-studio/allumette`. Ces deux gestes reviennent à William (dépôt gelé, aucun
  redéploiement dans cette tâche).
- **Les salons vivent en mémoire** (`ServerState::default()`) : un redémarrage ou un redéploiement efface les salons et
  sessions en cours. Il ne faut donc rien redéployer pendant une session de recette.

### 1.4 TURN — vérifié

- **Configuré.** `TURN_SECRET` et `TURN_URLS` sont posés sur l'instance : `/ice-servers` renvoie une entrée TURN avec des
  identifiants temporaires (`src/lib.rs:505`). Si l'une des deux variables manque, le serveur ne renvoie que du STUN.
- **Joignable** depuis ovh-inner (script en annexe A) :

  ```
  STUN UDP réponse type=0x0101 de 158.69.192.54   XOR-MAPPED-ADDRESS = 148.113.207.199:38816   SOFTWARE = Coturn-4.6.3 'Gorst'
  [udp] Allocate sans auth -> 0x0113 erreur 401
  [udp] Allocate authentifié -> SUCCÈS, relais 158.69.192.54:49194, lifetime 600 s, realm turn.estran.studio
  [udp] Refresh lifetime=0 (libération) -> 0x0104
  [tcp] Allocate authentifié -> SUCCÈS, relais 158.69.192.54:49174, lifetime 600 s
  ```

  Les identifiants servis par `/ice-servers` ouvrent donc un relais en UDP et en TCP. TURNS (5349) n'est pas ouvert. Un
  réseau qui ne laisse sortir que le 443 (pare-feu d'entreprise) n'aurait pas de relais : ce n'est pas le cas d'un
  partage de connexion de téléphone.
- **Côté alacod** :
  - **Navigateur : TURN utilisé.** `website/static/loader.js:67-83` remplace `RTCPeerConnection` dans l'iframe et lui
    passe la liste ICE complète du serveur, STUN et TURN avec leurs identifiants. Le jeu, lui, garde le STUN Google par
    défaut (`p2p_socket.rs:313-320`), mais cette valeur est écrasée par le loader.
    `ice_transport_policy: 'relay'` est lu par le loader, mais **le serveur ne l'envoie jamais** : on ne peut pas forcer
    TURN sans changer de code. Pour voir quel candidat est retenu, il faut ouvrir `chrome://webrtc-internals`.
  - **Natif : TURN jamais utilisé.** `p2p_socket.rs:295-307` ne garde que la **première** entrée de `/ice-servers`, et
    c'est le STUN. Hors périmètre de la sortie (§ 0.4 : site, Chrome). Candidat à la dette **D164** si b2 le confirme.

### 1.5 Secrets — noms et emplacements seulement

| Secret | Rôle | Emplacement attendu |
|---|---|---|
| `JWT_SECRET` | signature HS256 des JWT d'identité **et** des tickets de signaling | `/opt/allumette/.env` (lu par `dotenvy`/compose). **Le code se rabat en silence sur une clé de développement si la variable manque** (`src/lib.rs:65`) : à vérifier une fois l'accès obtenu (par la présence de la variable, jamais par sa valeur). |
| `TURN_SECRET` | secret partagé des identifiants temporaires TURN | `/opt/allumette/.env`, et le **même** secret dans la config coturn (`static-auth-secret`, fichier inconnu sans accès) |
| `TURN_URLS`, `STUN_URL`, `ALLOWED_ORIGINS` | configuration, pas des secrets | `/opt/allumette/.env` |
| `ALLUMETTE_IMAGE` | tag de l'image, réécrit par le déploiement | `/opt/allumette/.env` |
| `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY` | déploiement par SSH | secrets GitHub Actions de `estran-studio/allumette` (listés le 28 oct, valeurs illisibles) |
| `github.token` | push et pull GHCR pendant le déploiement | automatique (Actions) |
| `ACCESS_TOKEN`, `NPM_TOKEN` | `release.yaml`, `trigger-doc.yaml` ; `publish-npm.yaml` | cités par les workflows, absents de la liste du dépôt : sans doute des secrets d'organisation. Inutiles pour redéployer le serveur. |
| TLS | certificats Let's Encrypt | gérés par Caddy (son volume de données), aucun secret à fournir |
| Site : `ACCESS_TOKEN`, `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` | `alacod/.github/workflows/release.yaml` (tag `v*`, runner auto-hébergé `alacod-builder`, Cloudflare Pages `alacod`) | secrets Actions d'`estran-studio/alacod` (ou de l'organisation) |

**Pour redéployer allumette, William n'a rien à fournir** : les trois secrets `VPS_*` sont posés depuis le 2026-10-10.
Pour publier un nouveau build du jeu sur le site, il faut un tag `v*` sur alacod, donc un geste de William.

### 1.6 Flux site → jeu (résumé pour S1-3)

1. `/online` (`website/src/routes/online/+page.svelte`) crée un `AllumetteBrowserClient(settings.allumetteServerUrl)`,
   par défaut `https://allumette.estran.studio` (`settingsStore.ts:14`), puis `loginGuest(pseudo)` : clé Ed25519 en
   mémoire, défi, JWT ; rien n'est persistant.
2. L'hôte appelle `createLobby(game, "${game}:${buildId}", 2)` puis `createInvitation`. Le lien
   `/online?lobby=<id>&game=…&build=…#invite=<jeton>` vit 15 min (`INVITE_TTL`).
3. L'invité ouvre le lien et appelle `joinLobby(id, jeton, "${game}:${buildId}")`, où le `buildId` vient du
   `releases.json` **de son site**. Si les builds diffèrent, le serveur répond `incompatible_build`.
4. Chacun passe `setReady`, puis le propriétaire appelle `startSession`. Le serveur fige les participants, crée
   `session_id` et ouvre une session de **30 min** (`SESSION_TTL = 1800`), avec 120 s pour se connecter.
5. Les deux pages sondent `refresh()` chaque seconde et appellent `getSessionContext`. Cette réponse donne
   `participants`, `local_player_id`, un **ticket de signaling de 60 s** (`signaling_url = wss://…/<ticket>`) et
   `ice_servers`.
6. La page crée l'iframe `/loader.html?game&build&mode=online`. Le loader demande la session par `postMessage`
   (même origine, `source: 'alacod-loader'`/`'alacod-host'`) et valide le contrat : `schema_version` 1, jeu, build,
   capacité 2 ou 4, `expires_at` non dépassé.
7. Le loader remplace `RTCPeerConnection` pour imposer les ICE du serveur (point 4), puis écrit sur `#bevy-canvas` les
   attributs `data-session` (URL de signaling), `data-participants`, `data-players` (`is_local`), `data-number-player` et
   `data-matchbox`.
8. Le jeu lit ces attributs dans `jjrs/browser.rs` (`session()`, `participants()`) et ouvre le socket Matchbox sur
   `signaling_url` (`p2p_socket.rs:313-319`). Le serveur n'admet que les membres de la session, une connexion par joueur
   (`already_connected`, `session_expired`, `src/browser.rs:155-176`).
9. Le loader renvoie `connected` ou `error` (WebRTC `failed`) à la page. Le retour au salon passe par
   `ReturnToHostLobby` → `alacod_return_to_lobby()` → `return-lobby`. La page appelle alors `endSession` et peut relancer
   une partie, avec une nouvelle session.
10. `BrowserChannel` (`browser.rs`) coupe les envois GGRS et renvoie au salon dès que le canal Matchbox se ferme, au lieu
    de paniquer.

Conséquences pour la recette :

- **Une partie de plus de 30 min est coupée** quand la session expire (`prune`, `src/browser.rs:122` ; fermeture des
  connexions). Il faut le vérifier à J3, et rester sous 30 min par partie (retour au salon puis nouvelle partie).
  Allonger `SESSION_TTL` demande une décision de William, car allumette est gelé.
- **Le ticket de 60 s est un risque sur une connexion lente.** La page ne lit le contexte de session **qu'une fois**
  (`+page.svelte:89-96`) ; ensuite, `refresh()` ne renvoie que les participants. Le loader demande la session
  (`loader.js:56`) **avant** de télécharger et d'initialiser le wasm (`loader.js:102-103`). Le ticket doit donc encore être
  valable quand le jeu ouvre son WebSocket, après le téléchargement, le démarrage et le chargement des assets. Si cela
  prend plus de 60 s (sur le partage de connexion d'un téléphone, au premier chargement sans cache), on s'attend à un
  refus du signaling. À mesurer à J4 : chronométrer de « Lancer » à « Connexion établie » sur la machine B, cache vidé.
  Correctif côté alacod, à J3 si la mesure l'exige : redemander un contexte frais (`getSessionContext`) à chaque
  `session-request`, et faire demander la session au loader **après** le chargement du wasm. Ce correctif est dans le
  périmètre de b2 (`website/`).
- **D116 (arrivée tardive)** ne peut pas arriver sur ce chemin : le serveur n'admet que les participants figés au `start`.
  Le point ouvert de D116 ne concerne plus que les chemins `--matchbox` (nu) et `--allumette` natif.

### 1.7 Plan de J3–J5 pour b2

**J3, vendredi 30 (b2, VPS, sans navigateur)**

1. Rejouer les sondes de l'annexe A (5 min). Si un point a bougé, s'arrêter et poser la question.
2. **D116** (fiche, point 4) : borner l'attente de `wait_for_players` quand des pairs en trop restent connectés, avec un test
   unitaire (`p2p_tests.rs`). Écrire dans le rapport que le chemin navigateur est protégé par le serveur (point 6).
3. **Session de 30 min** : établir par la lecture ce que voit le jeu quand le signaling ferme en pleine partie (le canal
   WebRTC reste-t-il ouvert ? `BrowserChannel` renvoie-t-il au salon ?). Le dire dans le rapport et dans le protocole de J4.
   Proposer à William `SESSION_TTL` à 2 h, ou garder 30 min avec un retour au salon entre deux parties.
4. **`online` dans `releases.json`** (décision 4, défaut : `false` jusqu'à la preuve) : aligner le fichier du dépôt sur la
   prod (`v0.0.2-4630291a`), ou sur le build de S1-3 si William tague. La prod annonce `online: true` sans preuve.
5. **Build de la recette** : décider avec le coordinateur si J4 se joue sur la prod actuelle (`v0.0.2-4630291a`, aucun
   déploiement) ou sur un nouveau tag (William tague ; `release.yaml` publie sur Cloudflare Pages). Défaut : la prod
   actuelle, puisque S1-3 prouve le réseau, pas le polish.
6. **Ticket de 60 s** (point 6) : vérifier à la lecture si le WebSocket s'ouvre avant ou après le chargement des assets.
   Préparer le correctif `website/` (contexte frais à chaque `session-request`), à appliquer si J4 le montre.
7. Écrire le protocole de J4 (ci-dessous) dans `docs/jouer.md`. Corriger `docs/jouer.md:103` (le serveur public marche,
   mais seulement depuis le site) et la section « Préparation » de `docs/contrat-lancement-web.md`.
8. Optionnel, dette **D164** : client natif sans TURN (`p2p_socket.rs:295-307`, première entrée seulement), si b2 le confirme.

**J4, samedi 31 (b2 et b0, avec William : second réseau)**

Ce que William fournit : deux machines avec Chrome, l'une sur le réseau de la maison, l'autre sur le **partage de
connexion d'un téléphone** (ou chez un ami), une manette si possible, et 45 min. Protocole en dix lignes :

1. Machine A (maison) : ouvrir `https://alacod.estran.studio/online`, choisir un pseudo, créer un salon **zombies**,
   copier le lien d'invitation.
2. Machine B (téléphone) : vider le cache, ouvrir le lien (moins de 15 min), rejoindre. Chronométrer de « Lancer » à
   « Connexion établie » (ticket de 60 s).
3. Les deux cliquent « Prêt » ; A lance. Ouvrir `chrome://webrtc-internals` dans un autre onglet sur A.
4. Noter le type du candidat retenu (`host`, `srflx` ou `relay`) ; derrière le CGNAT d'un téléphone, `relay` est attendu.
5. Jouer 5 min : déplacement, tir, un mort. Noter tout écran « DESYNC », gel ou retour au salon imprévu.
6. Fin de partie, puis « Lobby » : vérifier le retour au salon des deux côtés (`ReturnToHostLobby`).
7. Relancer une seconde partie dans le même salon.
8. Recommencer les étapes 1 à 7 avec **throne**.
9. Refus d'un build incompatible : B ouvre l'invitation depuis `https://alacod.pages.dev`, ou depuis une prévisualisation
   Cloudflare dont le `buildId` diffère. Attendu : « Cette invitation utilise une autre version du jeu. »
10. Envoyer à b2 : les heures, les captures de `webrtc-internals`, et la console (F12) de chaque machine si une erreur apparaît.

**J5, dimanche 1er** : terminer S1-3 (rapport, journal `LIVRÉ`, `make check`), puis jouer **30 min de zombies à 2 avec
William**. À cause du `SESSION_TTL`, ce sont deux parties de moins de 30 min, ou une partie unique si le TTL a été allongé.
Si la recette de J4 échoue, il faut les journaux d'allumette et de coturn : soit l'accès SSH et Docker ci-dessus, soit
William lance `sudo docker compose logs --since 1h` dans `/opt/allumette` et transmet le résultat.

### Annexe A — sondes (relançables, sans secret écrit)

Les scripts du 28 oct sont dans le scratchpad du worker, qui n'est pas commité. Les recréer à l'identique :

- Sans jeton : `curl -sS -X <méthode> -w ' [%{http_code}]' https://allumette.estran.studio<route>` pour chaque route du
  point 1.
- Invité et `/ice-servers` : en Python avec `cryptography`. Générer une clé `Ed25519PrivateKey`, appeler
  `POST /auth/challenge`, puis `POST /auth/login` avec `{username, challenge, public_key_b64, signature_b64}`
  (base64 standard, signature des octets UTF-8 du défi), puis `GET /ice-servers` avec `Authorization: Bearer`.
  **Ne jamais afficher** `token`, `username` ni `credential`.
- TURN : envoyer `Allocate` (0x0003, `REQUESTED-TRANSPORT` 17) sans authentification ; la réponse 401 donne `REALM` et
  `NONCE`. Renvoyer `Allocate` avec `USERNAME`, `REALM`, `NONCE` et `MESSAGE-INTEGRITY` (HMAC-SHA1, clé
  `MD5(user:realm:pass)`), puis `Refresh` avec `LIFETIME=0` pour libérer le relais. Le faire en UDP, puis en TCP
  (trame STUN sans préfixe, RFC 5766).

## 2. Vendredi 30 (J3, VPS, sans navigateur)

Branche `polissage-reseau-amis` rebasée sur `origin/polissage-integration` (`a9c41166`, S1-4 compris). Aucune trace ne bouge :
le code touché est le salon en ligne (horloge murale, hors `GgrsSchedule`).

| # | Point | Issue | Preuve |
|---|---|---|---|
| 1 | Sondes de l'annexe A | **fait, rien n'a bougé** | § 2.1 |
| 2 | D116 | **fait, fermée** (attente d'admission bornée, test) | § 2.2, `p2p_socket_tests.rs` |
| 3 | Fin de session à 30 min | **établi par lecture : retour au salon, pas d'écran figé** ; un écran figé trouvé **avant** la partie, corrigé | § 2.3 |
| 4 | Ticket de 60 s | **établi par lecture : le socket s'ouvre après le chargement des assets** ; correctif préparé sur une branche à part | § 2.4 |
| 5 | Protocole de samedi, `docs/jouer.md:103`, contrat | **fait** | § 2.5 |

### 2.1 Sondes rejouées (2026-10-30, 02 h 52 UTC) — identiques au 28

Scripts de l'annexe A recréés (scratchpad, non commités). Résultats, secrets masqués :

```
DNS allumette/turn.estran.studio 158.69.192.54 ; certificat Let's Encrypt YE2, 9 oct → 7 janv. 2027
/health 200 · / 400 (upgrade) · /lobbies 200 [] · /ice-servers, /browser/v1/… 401 Invalid token · id non UUID 400 · /auth/challenge 200
ports 80 443 3478 22 ouverts ; 3536 5349 fermés ; CORS alacod.estran.studio 200, localhost:5173 sans Access-Control-Allow-Origin
login invité 200 ; ice-servers 200 [stun:turn…:3478, turn:…?transport=udp|tcp + identifiants] ; browser/v1/lobbies 200 [] ; lobbies 200 []
STUN udp 0x0101 SOFTWARE=Coturn-4.6.3 'Gorst'
[udp] Allocate sans auth 401 → authentifié SUCCÈS relais 158.69.192.54:49233, 600 s → Refresh 0 → 0x0104
[tcp] Allocate sans auth 401 → authentifié SUCCÈS relais 158.69.192.54:49236, 600 s → Refresh 0 → 0x0104
prod releases.json : v0.0.2-4630291a, online: true (zombies, throne) ; loader.js de prod md5 aa5bc59c… (inchangé)
allumette : dernier déploiement run 38068410753 (23524d8, 2026-10-10), pushed_at 2026-10-10T16:37:18Z
```

Seul écart : le `loader.js` **du dépôt** a changé depuis le 28 (S1-5, compteur FPS `?perf=1`, `c5dab98f`) ; celui de la prod
non. Sans effet sur la recette (prod).

### 2.2 D116 — attente d'admission bornée, fermée

Ce qui restait non borné (lecture de `wait_for_players`, `p2p_socket.rs`) : quand la salle a assez de joueurs, l'admission
attend 3 s (`ADMISSION_SETTLE_SECS`) après l'apparition **la plus récente** d'un pair vu par le signaling ou d'un excédent
connecté. Cette fenêtre repart à chaque apparition : un pair en trop qui clignote (fantôme qui se reconnecte, client relancé
en boucle, excédent qui part et revient) plus vite que toutes les 3 s retenait l'admission **sans fin**. Les autres attentes
étaient déjà bornées : salle pleine (le local ignoré) 15 s puis salon ; pair manquant 15 s par essai ; session non
synchronisée 30 s.

Correctif : `ADMISSION_MAX_SECS` = 10 s, borne **cumulée** comptée depuis que la salle a assez de joueurs connectés (remise à
zéro quand il en manque un ou à chaque socket neuf), dans une fonction pure `admission_waits(youngest_pending, excess_for,
enough_for)` ; journal unique `ggrs{f=0 lobby_admission_capped peers=… max=… waited_secs=…}` quand la borne coupe. Passé 10 s,
les admis du moment démarrent ; deux clients en désaccord sur l'ensemble finissent comme une arrivée tardive (abandon de la
session, 30 s, jamais un desync).

Test `admission_wait_is_bounded_when_extra_peers_keep_appearing` (`jjrs/p2p_socket_tests.rs`) : cas limites de la fenêtre et
de la borne, puis un fantôme qui réapparaît toutes les 2 s, une décision toutes les 100 ms pendant 60 s : **sans la borne,
aucune admission ; avec, admission à 10,0 s.**

```
$ env -u CI cargo test -q -p game --profile headless --no-default-features --lib -- p2p_socket
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 339 filtered out
```

Chemins : `--matchbox` et `--allumette` natif. Le chemin navigateur est protégé par le serveur : Allumette fige les participants
au `start`, n'admet sur le signaling que ces membres et une connexion par joueur (`already_connected`, `session_expired`,
`allumette/src/browser.rs:155-176`) ; l'excédent et l'arrivée tardive n'y existent pas. Le point « arrivée tardive » de D116
(abandon borné, natif seulement) n'est pas reconduit en dette : aucun chemin produit ne le rencontre.

### 2.3 Fin de session à 30 min — lecture du code

Chaîne lue (allumette `23524d8`, fork matchbox `593f900c`, alacod) :

1. **Serveur** : une tâche toutes les **5 s** (`allumette/src/lib.rs:103-107`) appelle `BrowserState::prune`
   (`src/browser.rs:118`) : une session expirée (`expires_at`, 30 min) ou non connectée à temps (`connect_until`, 120 s) est
   effacée (`lobby.reset()`), ses connexions passent dans `closing` et reçoivent un `Close` WebSocket.
2. **Client matchbox (wasm)** : le flux du WebSocket se termine (`wasm.rs:84-87`, `None` → `StreamExhausted`) ; la boucle de
   signaling sort, `run_socket` aussi (`socket.rs:768-773`, `StreamExhausted` → `Ok`) et emporte la boucle de messages : ses
   récepteurs des canaux sortants sont détruits, donc `WebRtcChannel::is_closed()` (`socket.rs:309`, `tx.is_closed()`)
   devient vrai **même si la connexion WebRTC entre pairs tenait encore**.
3. **Jeu, en partie** : `BrowserChannel::closed()` (`jjrs/browser.rs:57-66`, présent dans `4630291a`) cesse d'envoyer et de
   recevoir et appelle **une fois** `alacod_return_to_lobby()` → `loader.js:28` poste `return-lobby` → la page
   (`+page.svelte`, `returnToLobby`) oublie la session (l'iframe disparaît) et rafraîchit le salon.
4. **Page, filet indépendant** : `refresh()` sonde le salon chaque seconde ; une session effacée par le serveur
   (`!lobby.session_id`) remet `session = null`, ce qui retire l'iframe même sans message du jeu.

Verdict : à 30 min, les deux joueurs reviennent au salon en ≤ 5 s + 1 s, la partie en cours est perdue, **pas d'écran figé**.
`endSession` peut alors répondre une erreur (session déjà effacée) : le salon affiche un message, sans autre effet. Le
protocole de samedi garde des parties de moins de 30 min (décision du 29).

**Écran figé trouvé avant la partie, corrigé.** Dans le salon du jeu (`AppState::LobbyOnline`), un socket refusé ou fermé
(ticket périmé, `connect_until` dépassé) rendait `try_update_peers()` en erreur à chaque frame : `wait_for_players` journalisait
`socket dropped` et revenait, **sans fin** ; l'écran « En attente des joueurs » restait jusqu'à ce que le serveur efface la
session (≤ 120 s), puis la page retirait le jeu. Correctif (`p2p_socket.rs`, `wait_for_players`) : sur le web, retour
immédiat au salon du site (`alacod_return_to_lobby`), journal unique `ggrs{f=0 lobby_socket_lost}` au lieu d'un `warn!` par
frame ; en natif, l'écran reste (chemins d'essai `--matchbox`, inchangés). Pas dans la prod de samedi (aucun tag).

### 2.4 Ticket de 60 s — lecture du code, correctif préparé

- Le ticket naît au premier `getSessionContext` de la page (≤ 1 s après « Lancer », `+page.svelte`, `refresh`) ; chaque appel en
  frappe un neuf (`exp = now + 60`, borné par la fin de session : `allumette/src/browser.rs:521-526`).
- La page le passe tel quel au chargeur à son `session-request`, **avant** le téléchargement du wasm (`loader.js`, `launch`).
- Le jeu le lit **au démarrage** du wasm (`args/mod.rs:227`, ressource `BrowserSession`) et n'ouvre le socket qu'à
  `OnEnter(AppState::LobbyOnline)` (`core.rs:130`), atteint **après le chargement des assets** (`global_asset.rs:324`).
- Donc le ticket doit survivre au téléchargement du wasm, à sa compilation, au démarrage et au chargement des assets.
  Tailles de la prod (`v0.0.2-4630291a/zombies`) : `wasm_bg.wasm` 43,8 Mo, **10,7 Mo en brotli** (Chrome) ; assets ≈ 8,5 Mo
  (dossier source). Au premier chargement, ≈ 19 Mo : ≈ 15 s à 10 Mbit/s, ≈ 50 s à 3 Mbit/s, plus la compilation. Risque réel
  seulement sur une liaison lente ; la mesure du point 2 du protocole tranche.
- Sur la prod, un ticket périmé donne : socket refusé → écran d'attente jusqu'à `connect_until` (120 s) → la page revient
  au salon ; relancer (cache chaud) suffit. Avec le correctif du § 2.3 : retour immédiat.

Correctif préparé, **non appliqué** (branche `polissage-reseau-amis-ticket`, commit à part, poussée) :

- page : chaque `session-request` reçoit un contexte frais (`getSessionContext`), l'ancien en repli ;
- chargeur : `data-session` reposé toutes les 30 s avec un ticket frais, jusqu'à `connected` ;
- jeu (wasm) : `start_matchbox_socket` relit `data-session` à l'ouverture du socket (la ressource du démarrage en repli).

Les trois sont nécessaires : un build de jeu ancien ignore le rafraîchissement (il a lu l'attribut au démarrage).
Vérifié : `svelte-check` 0 erreur, `node --check` du chargeur ; `cargo check` wasm du jeu (§ 2.6). Non vérifié dans un
navigateur (D94). À appliquer pour le **prochain tag** si samedi montre un chronomètre au-delà de 60 s ou un refus.

### 2.5 Protocole de samedi et documentation

- `docs/jouer.md` : section « En ligne depuis le site (Allumette, deux réseaux) » (protocole en dix lignes, repris du § 1.7,
  durée de session, conduite à tenir si le ticket expire) ; le point 3 de « En ligne à deux » (ancienne ligne 103 : « le
  serveur public ne marche pas ») dit maintenant qu'il marche depuis le site seulement et pourquoi la ligne de commande est
  refusée ; textes du salon cités en français (S1-4).
- `docs/contrat-lancement-web.md` : section « Service en production » (URL, opérateur, version, redéployer, redémarrer,
  journaux, durées, fin de session, ticket, natif) ; passages périmés corrigés (« sans push », boutons online).
- Dette nouvelle **D165** (client natif sans TURN) ; D164 laissée à S1-4 (perks) ; D166 prise le soir même (§ 2.7).

### 2.6 Vérifications de J3

| Commande | Résultat |
|---|---|
| `cargo test -p game --lib -- p2p_socket` (headless) | 4 passés |
| `cargo check -p game --target wasm32-unknown-unknown` (branche et branche ticket) | 0 erreur |
| `npm ci && npm run check` (website, branche ticket) | `svelte-check` 0 erreur, 0 avertissement |
| `node --check` du chargeur (branche ticket) | OK |
| `make check` (2026-10-30, tête de J3 `f1429d74`) | `✓ All checks passed` |
| `make test_scenarios` sans `BLESS` | `test result: ok. 5 passed; 0 failed; 12 ignored` (7 395 s), aucune trace modifiée (`git status` propre) |

`releases.json` du dépôt : déjà `online: false` (`rtc-fix-20261009`) ; il passera à `v0.0.2-4630291a` avec `online: true` pour
les seuls clones recettés samedi (§ 3).

### 2.7 Vérification dans Chrome sur le Mac (ven 30, 18 h 30–18 h 51 UTC) — ticket, retour au salon

Accord du coordinateur (18 h 22 UTC). Banc local, rien en production : allumette `23524d8` cloné dans `/tmp` (seul écart :
`SESSION_TTL` lu dans `S13_SESSION_TTL` pour raccourcir la session, défaut inchangé ; jamais poussé), `127.0.0.1:3637`, sans TURN
(STUN Google par défaut) ; site `npm run dev` du worktree ; zombies en wasm `dev` (112 Mo, `scripts/build-web-games.mjs`) :
**avant** = `da5377be` (prod + § 2.3), **après** = `2a5573b1` (correctif du ticket). Deux Chrome sans tête séparés (deux profils
`/tmp`, deux origines `127.0.0.1` / `localhost` : deux invités), `--mute-audio`, `AudioContext.resume` neutralisé, aucun clic
dans le jeu. B « cache vidé et liaison lente » : cache coupé et débit limité à 10 Mbit/s par CDP. Journal relevé par un crochet
de console et de `WebSocket` (tickets masqués). Builds, profils et `releases.json` remis en état après.

| Essai | Build | Condition | Résultat |
|---|---|---|---|
| R1 | avant | session 240 s | connexion à +42 s, `Synchronized`, 60 images/s ; à +240,7 s les deux iframes retirées à 4 ms d'écart, salon « 2 / 2 · En attente » |
| R2 | avant | B à 10 Mbit/s, session 300 s | B chargé à +93 s, ticket expiré à +60 s → socket refusé 3 fois (code 1006, 3 s d'écart) → `lobby_socket_lost` → `return-lobby` à +100 s ; A (socket fermé par le serveur, 1005) idem au même instant ; **pas d'écran figé** (en prod : attente jusqu'à `connect_until`, 120 s) |
| R3 | après | B à 10 Mbit/s, session 300 s | ticket reposé toutes les 30 s (`session-request` à +30, +60, +90 s) ; B chargé à +93 s, socket **ouvert** (ticket de +90 s) ; connexion à +134 s, `Synchronized` ; serveur arrêté en partie → `ws close 1006` → `return-lobby` en 7 ms sur les deux |
| R4 | après | session 25 s | session expirée avant la connexion WebRTC → les deux au salon à +25 s |
| R5 | après | session 90 s | `getStats` chaque 1,5 s : A crée sa connexion à +2 s, B ne la reçoit qu'à +41 s ; connectés à +42,3 s, paire `host`/UDP des deux côtés |
| R6 | après | session 62 s | en partie (vague 1, `f=886` identique sur A et B, joueur 77/100) ; à +62 s `ws close 1005` → `return-lobby` en 7 ms sur les deux, salon « 2 / 2 · En attente » |

```
R3 B 18:42:08.083 jeu<-page session ticket_exp=18:43:08
R3 B 18:43:38.130 jeu<-page session ticket_exp=18:44:38
R3 B 18:43:40.797 jeu ws open-request ws://127.0.0.1:3637/<ticket>   18:43:40.845 jeu ws open
R2 B 18:39:51.689 jeu WARN game::jjrs::p2p_socket: ggrs{f=0 lobby_socket_lost} socket de signaling fermé avant la partie
R6 A 18:50:52.310 jeu ws close code=1005   18:50:52.317 page<-jeu return-lobby
```

Verdict : **correctif du ticket vérifié** (un chargement de 93 s passe, il échoue sans lui) ; **retour au salon vérifié** à
l'expiration en partie (R6), avant la connexion (R4), à l'arrêt du serveur (R3) et sur ticket périmé (R2, correction du § 2.3) ;
aucun `DESYNC` dans les journaux. Le correctif reste non appliqué (décision du 29) ; la mesure de samedi tranche.

Trouvé en passant, **D166** : la connexion WebRTC prend 40–41 s après l'ouverture du socket dans les essais mesurés (R3, R5,
R6 ; paire `host`/UDP, R5). Le fork matchbox attend la fin de la collecte ICE avant l'offre (`wasm.rs:156`, `:292`) et ce
Mac sort par `utun5` (VPN) : STUN UDP de Google et de `turn.estran.studio:3478` sans réponse, TCP 3478 joignable. Cause
déduite, non isolée. Conséquence pour samedi : VPN coupé sur les deux machines (`docs/jouer.md`, protocole) ; un chronomètre
du point 2 vers 40 s signale ce cas, pas le ticket.

Non vérifié ici : deux réseaux, TURN (`relay`), throne, la prod.

## 3. Samedi 31 (J4) — recette à deux réseaux avec William

(à remplir)
