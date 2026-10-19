#!/usr/bin/env node
// Pair fantôme du signaling matchbox (essais D109/D116, `p2p-connect-timeout.sh GHOST=1`) : il rejoint la salle
// et, à chaque pair qui arrive, lui envoie un candidat ICE (premier signal d'un pair inconnu : le client ouvre sa
// négociation côté « accepte » et attend une offre qui ne vient jamais ; une offre invalide ferait paniquer le socket
// matchbox). Le serveur l'annonce donc à chaque socket neuf du client (« premier Signal reçu de lui » :
// `jjrs::signal_watch`) sans qu'il se connecte jamais : un pair resté visible à chaque tentative. Un processus suspendu (SIGSTOP) ne peut pas le faire : après
// une nouvelle tentative, le signaling n'annonce plus un pair déjà là que par l'offre que celui-ci envoie.
//
// Usage : node scripts/p2p-ghost-peer.mjs ws://127.0.0.1:3536/<salle>   (s'arrête sur SIGTERM)
const url = process.argv[2];
if (!url) {
    console.error("usage : p2p-ghost-peer.mjs ws://hôte:port/salle");
    process.exit(2);
}
const ws = new WebSocket(url);
let me = null;
ws.addEventListener("message", (message) => {
    const event = JSON.parse(String(message.data));
    if (event.IdAssigned) {
        me = event.IdAssigned;
        console.log(`ghost ${me} dans la salle`);
    } else if (event.NewPeer) {
        console.log(`ghost : candidat ICE sans suite pour ${event.NewPeer}`);
        ws.send(JSON.stringify({ Signal: { receiver: event.NewPeer, data: { IceCandidate: "{}" } } }));
    } else if (event.PeerLeft) {
        console.log(`ghost : ${event.PeerLeft} parti`);
    }
});
ws.addEventListener("close", () => process.exit(0));
ws.addEventListener("error", (error) => {
    console.error("ghost : erreur websocket", error.message ?? error);
    process.exit(1);
});
// Garde la connexion ouverte (le serveur n'en demande pas plus qu'un KeepAlive périodique).
setInterval(() => ws.readyState === 1 && ws.send(JSON.stringify("KeepAlive")), 5000);
