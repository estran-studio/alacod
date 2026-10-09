# VPS Allumette

Déployé le 2026-10-09 sur le VPS accessible par `ssh alumette`, IP publique
`158.69.192.54`. La stack est indépendante de l’engine; aucun jeu n’est installé
sur ce serveur. Le site et les builds sont publiés sur Cloudflare; voir [Cloudflare](../cloudflare.md).

## Configuration installée

- Debian 13, Docker et Compose; fichiers dans `/opt/allumette`.
  Node.js et Chromium ont été installés pour la recette; les navigateurs et le
  serveur HTTP temporaire ont été arrêtés, les builds de test retirés du VPS.
- Allumette : image `allumette:browser-a25e859b41ef`, API interne sur 3536.
  Archive source SHA-256 :
  `a25e859b41ef0ab772400e3dec7fbe7faaf0ed961df02a4bae145e05683a3b8a`.
- Caddy 2.10.2 : HTTPS/WSS sur `allumette.estran.studio`, certificat émis et
  renouvellement automatique. Les volumes Caddy persistent entre redémarrages.
- coturn 4.6.3 : `turn.estran.studio`, écoute/relais liés à l’IP publique,
  authentification par credentials temporaires provenant d’Allumette.
- Deux secrets distincts générés sur le VPS dans `.env`, propriété root, mode 0600.
  Allumette et coturn partagent uniquement le secret TURN.
- CORS limité à `https://alacod.estran.studio`, `https://alacod.pages.dev` et aux deux origines locales de
  recette `http://127.0.0.1:4173` et `http://127.0.0.1:4174`. Retirer ces origines
  locales après la recette; ajouter explicitement toute origine de preview nécessaire.
- Rotation des logs Docker : trois fichiers de 10 MiB par service. Pas d’access logs
  Caddy; logs Allumette au niveau warn pour éviter les tickets dans les URLs.
- Unités `allumette-stack.service` et `allumette-firewall.service` activées au boot;
  conteneurs avec restart `unless-stopped`.

Les deux noms DNS pointent vers le VPS en **DNS-only**. Le pare-feu hôte laisse
entrer SSH TCP 22, HTTP/HTTPS TCP 80/443, TURN UDP/TCP 3478, relais UDP
49160–49260 et les flux nécessaires à DHCP/ICMP. L’API 3536 n’est pas publiée.
La table nftables dédiée ne supprime pas les règles Docker.

WebRTC chiffre les data channels. Cette configuration ne propose pas TURN/TLS
sur 443; les réseaux bloquant 3478 nécessiteront une extension.

## Vérifications et limite actuelle

Vérifiés : services actifs, résolution DNS publique, HTTPS de confiance, `/health`
200, restrictions CORS, authentification invitée, STUN UDP et allocations TURN
avec les credentials temporaires. Les 40 tests Rust Allumette et les quatre tests
SDK passent; le site passe les vérifications Svelte et le build de production.

**Le transport TURN est validé sur Linux.** Deux Chromium 154 isolés sur le VPS
échangent des données dans les deux sens, avec candidats sélectionnés `relay`
des deux côtés : configuration complète, UDP seul et TCP seul passent. Le test
reproductible est dans le dépôt Allumette : `tools/webrtc-smoke`.

Le Mac de recette échoue aussi sur un test direct sans engine. Sa route vers le
VPS passe par un tunnel `utun4`; des requêtes STUN/TURN expirent. L’environnement
réseau local doit être comparé hors VPN avant d’attribuer la cause au tunnel.
Zombies et Throne passent aussi le parcours du site avec TURN forcé et deux
sessions successives, sans erreur JavaScript/HTTP ou désync observée, après
correction du panic GGRS sur canal fermé côté navigateur. La preuve Linux
utilise un seul hôte : la recette sur deux machines/réseaux reste nécessaire
avant la bêta. Le site est publié sur `https://alacod.pages.dev`; voir [Cloudflare](../cloudflare.md).

## Exploitation

```sh
ssh alumette
sudo systemctl status allumette-stack allumette-firewall
sudo docker compose -f /opt/allumette/compose.yaml ps
sudo docker compose -f /opt/allumette/compose.yaml logs --tail=100 allumette
sudo docker compose -f /opt/allumette/compose.yaml config --quiet
```

Pour une installation neuve : copier les fichiers de ce dossier dans
`/opt/allumette`, construire une image Allumette à tag immuable, renseigner
`.env` à partir de `.env.example`, puis installer les unités et le pare-feu.
Ne pas afficher `.env` ni la configuration Compose développée contenant les secrets.

Sauvegarder les secrets et les volumes Caddy hors du VPS. Les salons sont en RAM;
un redémarrage d’Allumette les efface. Avant une mise à jour, conserver l’image
courante : changer `ALLUMETTE_IMAGE` puis exécuter `docker compose up -d` permet
une promotion ou un retour à cette image. Aucun rollback d’une version précédente
n’a encore été exercé. Surveiller trafic TURN, RAM, disque et certificats.
