#!/usr/bin/env bash
# Exécute un script p2p (`scripts/p2p-*.sh`, `NETEM` compris) dans un conteneur Linux depuis un Mac
# (OrbStack ou Docker Desktop) : macOS n'a ni `tc netem` ni `unshare`. Le dépôt (fichiers suivis et nouveaux, état de
# l'arbre de travail) est copié dans un volume, compilé en `--profile headless` (volume `…-target`, réutilisé d'un
# appel à l'autre) puis le script est lancé avec `NET_ADMIN` : `tc` dégrade `lo` du conteneur, le signaling
# (`matchbox_server`) y tourne aussi, comme dans l'espace de noms de `p2p-lib.sh` (`P2P_NETNS=1`).
#
# Usage : [NETEM="delay 60ms 10ms loss 1%"] scripts/p2p-linux-box.sh scripts/p2p-connect-starts.sh gungeon
# Variables : BOX (nom du conteneur et, avec un tiret, préfixe des volumes : `alacod-box`), BOX_VOL_PREFIX, BOX_CPUS (4),
#   MATCHBOX_BIN_LOCAL (binaire Linux `matchbox_server` de même architecture ; sinon extrait de l'image du
#   compose `docker-compose.ci.yaml`, `cargo install` : plusieurs minutes la première fois),
#   BOX_KEEP_BIN=<nom> (copie sous `/t/bin/<nom>` le binaire du jeu après la compilation, pour le comparer ensuite
#   avec `BIN_OVERRIDE`), OUT (sorties du script : dossier du conteneur copié dans `$ROOT/target/box-out`).
#   Aucune écriture dans `target/` du dépôt hors `target/box-out`.
set -euo pipefail
export PATH="$HOME/.orbstack/bin:$PATH"
command -v docker >/dev/null || { echo "p2p-linux-box : docker introuvable (OrbStack : ~/.orbstack/bin)"; exit 2; }
[ $# -ge 1 ] || { sed -n '/^# Usage/,/^# Variables/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BOX=${BOX:-alacod-box}
V=${BOX_VOL_PREFIX:-$BOX-}   # préfixe des volumes (BOX_VOL_PREFIX=b0 : volumes b0src, b0target, b0cargo)
BOX_CPUS=${BOX_CPUS:-4}
IMAGE=${BOX_IMAGE:-rust:1-bookworm}
for v in src target cargo; do docker volume create "${V}${v}" >/dev/null; done

# 1. Arbre de travail (fichiers existants seulement : une suppression non commitée est omise).
echo "p2p-linux-box : copie de l'arbre de travail"
git ls-files -z --cached --others --exclude-standard | while IFS= read -r -d '' f; do
    [ -e "$f" ] && printf '%s\0' "$f"
done | tar --null -T - -cf - | docker run --rm -i -v "${V}src:/src" debian:bookworm-slim \
    sh -c 'find /src -mindepth 1 -maxdepth 1 -exec rm -rf {} + && tar -x -C /src'

# 2. matchbox_server Linux dans le volume cible.
if [ -n "${MATCHBOX_BIN_LOCAL:-}" ]; then
    docker run --rm -i -v "${V}target:/t" debian:bookworm-slim sh -c 'cat > /t/matchbox_server && chmod +x /t/matchbox_server' < "$MATCHBOX_BIN_LOCAL"
elif ! docker run --rm -v "${V}target:/t" debian:bookworm-slim test -x /t/matchbox_server; then
    echo "p2p-linux-box : extraction de matchbox_server depuis l'image du compose"
    docker compose -f docker-compose.ci.yaml build signaling >/dev/null
    image="$(docker compose -f docker-compose.ci.yaml config --images signaling | head -1)"
    id="$(docker create "$image")"
    docker cp "$id:/usr/local/bin/matchbox_server" "$ROOT/target/box-matchbox_server" >/dev/null
    docker rm "$id" >/dev/null
    docker run --rm -i -v "${V}target:/t" debian:bookworm-slim sh -c 'cat > /t/matchbox_server && chmod +x /t/matchbox_server' < "$ROOT/target/box-matchbox_server"
fi

# 3. Compilation et exécution (les deux dans le même conteneur : dépendances système installées une fois).
GAME_ARG=${2:-gungeon}
mkdir -p "$ROOT/target/box-out"
docker rm -f "$BOX-run" >/dev/null 2>&1 || true
set +e
docker run --name "$BOX-run" --cpus "$BOX_CPUS" --cap-add NET_ADMIN \
    -e CI=1 -e CARGO_TARGET_DIR=/t -e P2P_NETNS=1 -e MATCHBOX_BIN=/t/matchbox_server \
    -e NETEM="${NETEM:-}" -e OUT=/out -e KEEP="${BOX_KEEP_BIN:-}" -e GAME_ARG="$GAME_ARG" \
    -e STARTS="${STARTS:-}" -e WAIT_S="${WAIT_S:-}" -e BIN_OVERRIDE="${BIN_OVERRIDE:-}" \
    -e PLAY_FRAMES="${PLAY_FRAMES:-}" -e PLAY_WAIT_S="${PLAY_WAIT_S:-}" \
    -e ALACOD_CONNECT_TIMEOUT_SECS="${ALACOD_CONNECT_TIMEOUT_SECS:-}" \
    -v "${V}src:/src" -v "${V}target:/t" -v "${V}cargo:/usr/local/cargo/registry" -v "${V}cargo:/usr/local/cargo/git" \
    -w /src "$IMAGE" bash -c '
set -e
apt-get update -qq >/dev/null
apt-get install -y -qq pkg-config libudev-dev libasound2-dev libwayland-dev libxkbcommon-dev libx11-dev iproute2 python3 procps gawk >/dev/null
for v in STARTS WAIT_S BIN_OVERRIDE PLAY_FRAMES PLAY_WAIT_S ALACOD_CONNECT_TIMEOUT_SECS NETEM; do [ -n "${!v}" ] || unset "$v"; done
cargo build -q -p "$GAME_ARG" --profile headless --no-default-features 2>&1 | grep -E "^error" -A12 || true
if [ -n "${KEEP:-}" ]; then mkdir -p /t/bin && cp "/t/headless/$GAME_ARG" "/t/bin/$KEEP"; fi
if [ -n "${NETEM:-}" ]; then tc qdisc replace dev lo root netem $NETEM && tc qdisc show dev lo | head -1; fi
export P2P_NO_BUILD=1
mkdir -p /out
"$@"' bash "$@"
status=$?
set -e
docker cp "$BOX-run:/out/." "$ROOT/target/box-out/" >/dev/null 2>&1 || true
docker rm "$BOX-run" >/dev/null 2>&1 || true
echo "p2p-linux-box : sorties dans $ROOT/target/box-out"
exit "$status"
