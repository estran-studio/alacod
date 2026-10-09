# Préparation VPS Allumette

Configuration de départ, **non déployée et non validée sur un VPS**. La stack est autonome;
aucun build de jeu ou moteur n'est installé sur ce serveur. Le contrat multijeu est dans
le dépôt Allumette : `docs/browser-sessions-v1.md`.

## Préparation

1. Linux avec Docker Compose, IP publique stable. Confirmer région/budget et hostname TURN.
2. Construire/livrer une image Allumette depuis son dépôt et fixer ALLUMETTE_IMAGE à un tag
   immuable/digest validé. Vérifier les versions d'images Caddy/coturn avant promotion.
3. Copier `.env.example` vers `.env` (non versionné), renseigner IP/image et deux secrets
   forts différents. Allumette et coturn partagent uniquement TURN_SECRET.
4. DNS : `allumette.estran.studio` vers le VPS, `turn.estran.studio` DNS-only vers la même IP.
   Site `alacod.estran.studio` vers Cloudflare Pages, via son association de domaine.
5. Pare-feu : HTTPS TCP 80/443, TURN UDP/TCP 3478 et relais UDP 49160–49260; SSH restreint.
   Le port API interne 3536 n'est pas publié sur l'hôte. Pas de load balancing multi-instance.
6. Valider l'image et les fichiers avec `docker compose config --quiet`, puis lancer lorsque
   le VPS et les secrets sont en place. Tester `/health`, HTTP auth, SSE et WebSocket.

Le signaling est chiffré en WSS; WebRTC chiffre ses data channels même avec TURN sur 3478.
Cette première config ne propose pas TURN/TLS 443 : les réseaux qui bloquent ces ports
nécessitent une extension (TLS, certificats et routage/second IP). Ne pas les annoncer validés.

Le code actuel d'Allumette a encore un CORS permissif et des JWT utilisateurs dans les chemins
WSS; les restrictions d'origine et tickets de session du contrat restent à implémenter avant
ouverture publique. Le proxy n'écrit pas d'access logs et le serveur démarre au niveau warn.

## Exploitation et recette

- Sauvegarder hors VPS les secrets, le compose et les volumes Caddy nécessaires. Les salons
  restent en RAM; le redémarrage d'Allumette efface leur présence, retour au salon côté client.
- Figer une image précédente pour revenir en arrière. Surveiller trafic TURN, connexions,
  erreurs, RAM, disque et renouvellement des certificats. Rotation des logs Docker à configurer.
- Une allocation TURN authentifiée doit échouer sans credentials et fonctionner avec ceux
  d'`/ice-servers`. Vérifier que l'IP publique est annoncée et qu'un duo forcé en `relay` joue.
- Avant une soirée : vérifier certificats/DNS, santé, expiration invitation/JWT, SSE reconnect,
  deux parties successives et suppression de salon. Noter la version des clients et du serveur.
