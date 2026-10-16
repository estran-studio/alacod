#!/usr/bin/env bash
# Enregistre les inputs de N bots `explorateur` sur une run gungeon (graine, étages assemblés
# `etages_deux`, hub ignoré) pour les rejouer en p2p réel (`scripts/p2p-stress.sh`, `ALACOD_INPUT=
# script:`), m2-p2p-run-complete. Les bots ne sont pas dans le binaire du jeu : leurs décisions
# sont figées en inputs par `alacod-sim --save-scenario` (l'enregistrement exact de ce qui part
# à GGRS), puis rejouées par chaque client. Elles ne valent donc que pour la même graine, les
# mêmes réglages et le même contenu : tout changement de simulation les rend caduques, et
# `p2p-stress.sh` (EXPECT_FLOORS) le dit (aucun étage franchi) au lieu de l'avaler.
#
# Usage : scripts/p2p-gungeon-inputs.sh <joueurs> <graine> <frames> <sortie.ron>
# Le client p2p doit être lancé avec `ALACOD_SKIP_HUB=1` et `--seed <graine>` : la graine de run
# d'une partie en ligne passée par le hub est dérivée (§33), celle d'`alacod-sim` non.
# Sortie : 0 si le fichier est écrit ET qu'au moins un étage est franchi dans la partie
# enregistrée ; affiche la frame du passage d'étage.
set -euo pipefail

[ $# -eq 4 ] || { sed -n '/^# Usage/,/^# Sortie/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
PLAYERS=$1
SEED=$2
FRAMES=$3
OUT_FILE=$4

cd "$(dirname "${BASH_SOURCE[0]}")/.."
DEV_FEATURES="$(./scripts/dev-features.sh features)"
# shellcheck disable=SC2086
cargo build -q -p scenario --bin alacod-sim --profile headless --no-default-features $DEV_FEATURES
TARGET="${CARGO_TARGET_DIR:-$PWD/target}"
libpath="$(./scripts/dev-features.sh libpath 2>/dev/null || true)"
export LD_LIBRARY_PATH="${libpath:+$libpath:}${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="${libpath:+$libpath:}${DYLD_LIBRARY_PATH:-}"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
profiles=$(printf 'explorateur,%.0s' $(seq "$PLAYERS"))
profiles=${profiles%,}
"$TARGET/headless/alacod-sim" --game gungeon --bots "$PLAYERS" --profiles "$profiles" \
    --floors etages_deux --until-floor 2 --seeds "$SEED..$SEED" --shopping \
    --max-frames "$FRAMES" --save-scenario "$TMP" --json "$TMP/sim.json" 2> "$TMP/sim.log" || {
    tail -5 "$TMP/sim.log"; echo "p2p-gungeon-inputs : alacod-sim a échoué"; exit 1; }
[ -f "$TMP/seed_$SEED.ron" ] || { echo "p2p-gungeon-inputs : enregistrement absent"; exit 1; }
cp "$TMP/seed_$SEED.ron" "$OUT_FILE"
python3 - "$TMP/sim.json" "$SEED" "$FRAMES" "$OUT_FILE" <<'EOF'
import json, sys
result = json.load(open(sys.argv[1]))[0]
frames = result["floor_frames"]
print(f"p2p-gungeon-inputs : graine {sys.argv[2]}, {result['frames']} frames enregistrées "
      f"(demandées {sys.argv[3]}), passages d'étage {frames}, morts {result['deaths']}, "
      f"issue {result.get('run_end')} -> {sys.argv[4]}")
if not frames:
    sys.exit("p2p-gungeon-inputs : aucun étage franchi : choisir une autre graine ou plus de frames")
if result["frames"] < int(sys.argv[3]):
    sys.exit("p2p-gungeon-inputs : la partie s'est arrêtée avant les frames demandées")
EOF
