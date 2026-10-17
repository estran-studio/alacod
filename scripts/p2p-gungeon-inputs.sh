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
# HUB=1 : l'enregistrement est celui d'une run lancée par la PORTE DU HUB en ligne (D85) : `alacod-sim` joue
# le hub puis la run (`ALACOD_SIM_HUB=1`), avec la graine de run `restart_seed(graine, 1)`
# (`ALACOD_RUN_SEED_ONLINE_GAME=1`) et le fichier reçoit les
# pistes du hub (le joueur 0 va à la porte, f170) ; le client se lance SANS `ALACOD_SKIP_HUB`.
# ALLOW_SHORT=1 : accepte une partie plus courte que <frames> (fin de run : victoire ou défaite,
# pour un restart après la fin de run). Sortie : 0 si le fichier est écrit ET qu'au moins un étage est franchi dans la partie
# enregistrée ; affiche la frame du passage d'étage.
set -euo pipefail

[ $# -eq 4 ] || { sed -n '/^# Usage/,/^# Sortie/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
PLAYERS=$1
SEED=$2
FRAMES=$3
case "$4" in /*) OUT_FILE=$4 ;; *) OUT_FILE=$PWD/$4 ;; esac

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
${HUB:+env ALACOD_RUN_SEED_ONLINE_GAME=1 ALACOD_SIM_HUB=1} "$TARGET/headless/alacod-sim" --game gungeon --bots "$PLAYERS" --profiles "$profiles" \
    --floors etages_deux --until-floor 2 --seeds "$SEED..$SEED" --shopping \
    --max-frames "$FRAMES" --save-scenario "$TMP" --json "$TMP/sim.json" 2> "$TMP/sim.log" || {
    tail -5 "$TMP/sim.log"; echo "p2p-gungeon-inputs : alacod-sim a échoué"; exit 1; }
[ -f "$TMP/seed_$SEED.ron" ] || { echo "p2p-gungeon-inputs : enregistrement absent"; exit 1; }
# Délai d'input : le synctest d'`alacod-sim` applique l'input décidé à la frame f à la frame f+5
# (`ALACOD_INPUT_DELAY` : 5 par défaut), alors que les clients p2p des preuves tournent à 0 (une entrée
# distante tardive y est une prédiction fausse, donc un rollback) : sans décalage, la run rejouée en
# ligne n'est pas celle qu'ont jouée les bots (constaté : boss mort 110 frames plus tard, jamais de victoire).
# INPUT_SHIFT (5) : décalage ajouté aux segments d'inputs des joueurs ; 0 pour des clients à délai 5.
python3 - "$TMP/seed_$SEED.ron" "$OUT_FILE" "${INPUT_SHIFT:-5}" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
shift = int(sys.argv[3])
text = re.sub(r"Segment\(from: (\d+), to: (\d+),",
              lambda m: f"Segment(from: {int(m[1]) + shift}, to: {int(m[2]) + shift},", text)
open(sys.argv[2], "w").write(text)
PY
if [ -n "${HUB:-}" ]; then
    # Pistes du hub (même parcours que `p2p-stress-inputs.py --hub-door`) ajoutées avant la dernière `)`.
    python3 - "$OUT_FILE" "$(dirname "${BASH_SOURCE[0]}")" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("stress_inputs", sys.argv[2] + "/p2p-stress-inputs.py")
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
text = open(sys.argv[1]).read().rstrip()
assert text.endswith(")")
open(sys.argv[1], "w").write(text[:-1] + module.HUB_DOOR + ")\n")
PY
fi
python3 - "$TMP/sim.json" "$SEED" "$FRAMES" "$OUT_FILE" <<'EOF'
import json, os, sys
result = json.load(open(sys.argv[1]))[0]
frames = result["floor_frames"]
print(f"p2p-gungeon-inputs : graine {sys.argv[2]}, {result['frames']} frames enregistrées "
      f"(demandées {sys.argv[3]}), passages d'étage {frames}, morts {result['deaths']}, "
      f"issue {result.get('run_end')} -> {sys.argv[4]}")
if not frames:
    sys.exit("p2p-gungeon-inputs : aucun étage franchi : choisir une autre graine ou plus de frames")
if result["frames"] < int(sys.argv[3]) and os.environ.get("ALLOW_SHORT") != "1":
    sys.exit("p2p-gungeon-inputs : la partie s'est arrêtée avant les frames demandées")
EOF
