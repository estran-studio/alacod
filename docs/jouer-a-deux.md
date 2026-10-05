# Jouer à deux (ou plus) sur deux machines

Recette pratique pour une partie de `zombies` entre deux personnes, chacune sur sa machine. Le
réseau est du p2p GGRS sur WebRTC (matchbox) : il faut un **serveur de signaling** joignable par
les deux clients, puis les clients se parlent directement (STUN Google en dur, pas de TURN).

État au 2026-10-04 : recette **vérifiée** avec deux clients headless sur la même machine qui
passent par l'IP Tailscale (`ws://100.64.0.6:3536`, 600 frames, traces identiques, sha
`39654b07…`). **Non vérifié** : deux machines distinctes, clients avec fenêtre, traversée d'un
NAT résidentiel sans Tailscale. `wss://allumette.bascanada.org` (défaut `MATCHBOX_URL` du
Makefile) **ne résout pas en DNS** : ne pas compter dessus.

## 1. Les deux machines

- Rust nightly (`rustup toolchain install nightly`), clone du dépôt `alacod` **sur le même
  commit** chez les deux (le checksum GGRS détecte toute différence de simulation : `Desync
  detected` dans le log).
- Première compilation : `cargo build -p zombies --profile headless` (20 à 40 min sur un portable ;
  le profil `headless` est le profil **optimisé** ; `make zombies` tout nu compile en `dev`,
  optimisé lui aussi depuis movement-feel : `opt-level = 1` pour le workspace, `3` pour les
  dépendances, dette D30 ; avant, ~15 fps). Le binaire lit `games/zombies` par le chemin
  capturé à la compilation : on ne copie pas le binaire d'une machine à l'autre, chacun compile.
- Réseau : les deux sur le même tailnet Tailscale (inviter l'ami), ou le même LAN. Sans ça, il
  faut ouvrir le port TCP 3536 vers la machine du serveur de signaling et donner l'IP publique ;
  la liaison de jeu (UDP via STUN) passe sur la plupart des NAT résidentiels, sans garantie.

## 2. Serveur de signaling (sur la machine Linux de William)

```bash
cd alacod
docker compose -f docker-compose.ci.yaml up -d signaling   # matchbox_server sur 0.0.0.0:3536
tailscale ip -4                                             # ex. 100.64.0.6
# fin de soirée :
docker compose -f docker-compose.ci.yaml down
```

## 3. Lancer le jeu (chaque joueur, à moins d'une minute d'écart)

```bash
APP_VERSION=x cargo run -p zombies --profile headless -- \
  --matchbox ws://100.64.0.6:3536 --lobby soiree \
  --number-player 2 --players localhost remote --name William
```

Même `--lobby` pour tous, `--number-player` = nombre de joueurs, `--players localhost remote`
(un `remote` par joueur distant) et un `--name` différent. La partie démarre quand le lobby est
plein. Un joueur à 0 PV tombe à terre et peut être réanimé par l'autre.

## 4. Si ça ne marche pas

| Symptôme | Cause probable | Quoi faire |
|---|---|---|
| Le jeu attend indéfiniment dans le lobby | signaling injoignable ou lobbies différents | `curl -i http://IP:3536/` depuis la machine de l'ami ; même `--lobby` ; `docker compose ps` |
| Lobby plein mais rien ne bouge, puis sortie | WebRTC bloqué (NAT) | passer par Tailscale, ou même LAN |
| `Desync detected on frame …` dans le log | commits ou contenu différents | `git rev-parse HEAD` et `git status` identiques chez les deux, recompiler |
| Panic à la fermeture de la fenêtre | dette D31 (`Query<&Window>`), sans effet sur la partie | ignorer |
| ~15 fps | binaire `dev` compilé avant movement-feel (D30) | recompiler, ou relancer avec `--profile headless` |

Plan B sans réseau p2p : une seule machine, un seul clavier (`make zombies`), ou en local
`--players localhost` avec des bots (`--bots N` sur `alacod-sim` seulement, pas dans le jeu
fenêtré).
