#!/usr/bin/env bash
# Lance une commande avec une carte son virtuelle à l'heure (M-7b, docs/conventions.md § 124) : chaque flux de sortie
# ALSA du processus (kira, harmonium) devient <dossier>/flux-<pid>-<n>.wav + .log (underruns, marge du tampon, écart
# maximal entre deux écritures), sans carte son, sans PulseAudio, sans rien changer au poste : la configuration ALSA ne
# vaut que pour ce processus (ALSA_CONFIG_PATH), jamais un ~/.asoundrc.
#
# Usage : scripts/audio-capture.sh <dossier> <commande> [arguments...]
# Exige gcc et les en-têtes ALSA (libasound2-dev) : le plugin (tools/alsa-pacer/pacer.c) est compilé dans <dossier>.
# Analyse : scripts/audio-gaps.py <musique.wav> --effects <effets.wav>.
set -euo pipefail

if [ $# -lt 2 ]; then
  sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
fi
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
plugin="$out/libasound_module_pcm_pacer.so"
cc -shared -fPIC -DPIC -O2 -Wall -o "$plugin" "$ROOT/tools/alsa-pacer/pacer.c" -lasound
cat >"$out/asound.conf" <<EOF
pcm_type.pacer { lib "$plugin" }
pcm.!default { type pacer prefix "$out/flux" }
EOF
ALSA_CONFIG_PATH="$out/asound.conf" "$@"
