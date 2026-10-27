#!/bin/sh
# Piste de laboratoire de la musique calée sur une horloge (X-3, docs/conventions.md §90) :
# deux battements de 2/3 s (40 frames à 60 Hz), un aigu (880 Hz) puis un grave (440 Hz), 4/3 s au
# total (80 frames) : la boucle de l'horloge `pouls` de `games/testbed` (`repeat: Frames(40)`,
# `loop_events: 2`). Dépend de ffmpeg (libvorbis). Usage : scripts/generer-son-metronome.sh [sortie.ogg]
set -eu
OUT="${1:-games/testbed/assets/sounds/metronome.ogg}"
ffmpeg -y -hide_banner -loglevel error -f lavfi \
  -i "aevalsrc='0.5*sin(2*PI*880*t)*exp(-40*t)*lt(t,0.33) + 0.3*sin(2*PI*440*mod(t,0.666667))*exp(-40*mod(t,0.666667))*gte(t,0.666667)':s=44100:d=1.333333" \
  -c:a libvorbis -q:a 3 "$OUT"
