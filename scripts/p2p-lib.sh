#!/usr/bin/env bash
# Fonctions communes aux scripts p2p (p2p-stress.sh, p2p-disconnect.sh, p2p-restart.sh) : binaire
# headless du jeu, serveur de signaling (Docker, serveur déjà en ligne, ou processus local dans un
# espace de noms réseau), réseau dégradé `tc netem` sans root. À sourcer depuis les scripts de
# `scripts/` ; ne rien exécuter à la source.
#
# Variables lues :
#   MATCHBOX_URL   : signaling déjà en ligne (`ws://hôte:port` ; rien n'est démarré, `docker` non
#                    requis). Vide : signaling local (Docker, ou processus dans l'espace de noms).
#   MATCHBOX_PORT  : port hôte du signaling local (3536 par défaut ; un par session, README §1).
#   NETEM          : arguments de `tc qdisc … netem` (ex. « delay 60ms 10ms loss 1% » : le délai
#                    s'applique à chaque sens, donc un RTT de 120 ms). Le script se relance alors
#                    dans un espace de noms réseau et utilisateur (`unshare -Urn`, sans privilège)
#                    où l'interface `lo` est dégradée ; le signaling y tourne aussi (processus
#                    `matchbox_server`, extrait de l'image Docker du compose, ou `MATCHBOX_BIN`).
#   KEEP_SIGNALING=1 : laisse le signaling Docker démarré.
#   P2P_NO_BUILD=1 : ne compile pas (binaire déjà construit).

# Racine du dépôt, binaire du jeu `$BIN`, bibliothèques partagées de développement.
p2p_init() {
    local game=$1
    ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    cd "$ROOT" || exit 1
    local features
    features="$(./scripts/dev-features.sh features)"
    if [ "${P2P_NO_BUILD:-0}" != 1 ]; then
        # shellcheck disable=SC2086
        cargo build -q -p "$game" --profile headless --no-default-features $features || exit 1
    fi
    TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
    BIN="$TARGET/headless/$game"
    [ -x "$BIN" ] || { echo "p2p : binaire introuvable : $BIN"; exit 2; }
    local libpath
    libpath="$(./scripts/dev-features.sh libpath 2>/dev/null || true)"
    export LD_LIBRARY_PATH="${libpath:+$libpath:}${LD_LIBRARY_PATH:-}"
    export DYLD_LIBRARY_PATH="${libpath:+$libpath:}${DYLD_LIBRARY_PATH:-}"
    export MATCHBOX_PORT=${MATCHBOX_PORT:-3536}
}

# Serveur `matchbox_server` : `MATCHBOX_BIN`, sinon extrait de l'image construite par le compose.
p2p_matchbox_bin() {
    if [ -n "${MATCHBOX_BIN:-}" ] && [ -x "$MATCHBOX_BIN" ]; then
        return 0
    fi
    MATCHBOX_BIN="$TARGET/matchbox_server"
    [ -x "$MATCHBOX_BIN" ] && return 0
    command -v docker >/dev/null || { echo "p2p : ni MATCHBOX_BIN ni docker pour extraire matchbox_server"; exit 2; }
    docker compose -f docker-compose.ci.yaml build signaling >/dev/null || exit 1
    local image id
    image="$(docker compose -f docker-compose.ci.yaml config --images signaling | head -1)"
    id="$(docker create "$image")" || exit 1
    docker cp "$id:/usr/local/bin/matchbox_server" "$MATCHBOX_BIN" >/dev/null
    docker rm "$id" >/dev/null
    [ -x "$MATCHBOX_BIN" ] || { echo "p2p : extraction de matchbox_server échouée"; exit 2; }
}

# Avec NETEM : se relance dans un espace de noms réseau où `lo` est dégradé. À appeler après
# `p2p_init` (le build et l'extraction se font avant, avec le réseau normal) :
#   p2p_netns_reexec "$0" "$@"
p2p_netns_reexec() {
    [ -n "${NETEM:-}" ] && [ -z "${P2P_NETNS:-}" ] || return 0
    p2p_matchbox_bin
    export MATCHBOX_BIN
    if [ -n "${MATCHBOX_URL:-}" ]; then
        echo "p2p : NETEM et MATCHBOX_URL sont exclusifs (le réseau dégradé est local)"; exit 2
    fi
    exec unshare -Urn env P2P_NETNS=1 P2P_NO_BUILD=1 bash -c '
        PATH="$PATH:/sbin:/usr/sbin"
        ip link set lo up || exit 2
        # shellcheck disable=SC2086
        tc qdisc add dev lo root netem $NETEM || { echo "p2p : tc netem indisponible (noyau sans sch_netem ?)"; exit 2; }
        echo "réseau dégradé : $(tc qdisc show dev lo | head -1)"
        exec "$@"' bash "$@"
}

# Démarre le signaling ; `$SIGNAL_URL` est l'adresse à passer aux clients.
p2p_signaling_up() {
    SIGNAL_PID=""
    if [ -n "${MATCHBOX_URL:-}" ]; then
        SIGNAL_URL=$MATCHBOX_URL
    elif [ "${P2P_NETNS:-}" = 1 ]; then
        "$MATCHBOX_BIN" "127.0.0.1:$MATCHBOX_PORT" > "${OUT:-/tmp}/signaling.log" 2>&1 &
        SIGNAL_PID=$!
        SIGNAL_URL="ws://127.0.0.1:$MATCHBOX_PORT"
        sleep 1
    else
        docker compose -f docker-compose.ci.yaml up -d signaling >/dev/null || exit 1
        SIGNAL_URL="ws://127.0.0.1:$MATCHBOX_PORT"
        sleep 3
    fi
}

p2p_signaling_down() {
    if [ -n "${SIGNAL_PID:-}" ]; then
        kill "$SIGNAL_PID" 2>/dev/null
    elif [ -z "${MATCHBOX_URL:-}" ] && [ "${KEEP_SIGNALING:-0}" != 1 ] && [ "${P2P_NETNS:-}" != 1 ]; then
        docker compose -f docker-compose.ci.yaml down >/dev/null 2>&1
    fi
    return 0
}

# `--players` d'une session à $1 pairs : `localhost remote … remote`.
p2p_players() {
    local n=$1 list=localhost i
    for ((i = 1; i < n; i++)); do list="$list remote"; done
    echo "$list"
}

# Frames resimulées d'un journal de client (compteur du jeu, `-1` si absent).
p2p_resimulated() {
    local n
    n=$(sed -n 's/.*frames resimulées après rollback : \([0-9][0-9]*\).*/\1/p' "$1" | tail -1)
    echo "${n:--1}"
}
