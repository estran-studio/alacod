#!/usr/bin/env bash
# Matrice de mesure de D87 (silences fatals de la session p2p sous perte de paquets) : rejoue une run
# gungeon enregistrée (`scripts/p2p-gungeon-inputs.sh`) à deux clients sous `tc netem` sans root
# (`scripts/p2p-stress.sh`, `STRESS=0`), pour chaque cellule du réseau et chaque réglage de la session, et
# mesure à chaque essai (`scripts/p2p-silences.py`) la durée de chaque silence du pair (journaux GGRS
# horodatés), les déconnexions et la charge de la machine (taux d'occupation du CPU, un échantillon toutes les
# 2 s). Les réglages d'essai sont `ALACOD_DISCONNECT_TIMEOUT_MS` et `ALACOD_P2P_CHANNEL=unreliable`
# (`game::jjrs::p2p`) ; le défaut du jeu est le canal fiable et 2 000 ms.
#
# Usage : scripts/p2p-d87.sh <script-inputs.ron> <frames> <sortie-dossier>
# Variables : CELLS (« delay 60ms 10ms loss 1%|delay 60ms 10ms loss 3%|delay 100ms 10ms loss 3% », séparées par `|`),
#   VARIANTS (« default|timeout5000|unreliable »), REPS (« 3 » : répétitions par cellule et variante, ou
#   « default=3,timeout5000=2,unreliable=2 »), GAME (gungeon), GAME_ARGS (« --seed 10 »), P2P_WAIT_CALM.
# Écrit `<sortie>/summary.tsv` (une ligne par essai) et garde `<sortie>/<essai>/` (journaux, silences, charge).
set -uo pipefail

[ $# -eq 3 ] || { sed -n '/^# Usage/,/^# Écrit/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
SCRIPT=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
FRAMES=$2
mkdir -p "$3"
OUTDIR=$(cd "$3" && pwd)
GAME=${GAME:-gungeon}
GAME_ARGS=${GAME_ARGS:---seed 10}
CELLS=${CELLS:-delay 60ms 10ms loss 1%|delay 60ms 10ms loss 3%|delay 100ms 10ms loss 3%}
VARIANTS=${VARIANTS:-default|timeout5000|unreliable}
REPS=${REPS:-3}
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

reps_for() {
    case "$REPS" in
        *=*) local r; r=$(echo ",$REPS," | sed -n "s/.*,$1=\([0-9]*\),.*/\1/p"); echo "${r:-1}" ;;
        *) echo "$REPS" ;;
    esac
}

# Taux d'occupation du CPU (tous coeurs), « epoch pourcentage », un échantillon toutes les 2 s.
sample_load() {
    python3 - "$1" >/dev/null 2>&1 <<'PY' &
import sys, time
def read():
    f = open("/proc/stat").readline().split()[1:]
    v = list(map(int, f))
    return sum(v), v[3] + v[4]
total, idle = read()
with open(sys.argv[1], "w") as out:
    while True:
        time.sleep(2)
        t, i = read()
        out.write(f"{time.time():.0f} {100.0 * (1 - (i - idle) / max(t - total, 1)):.1f}\n")
        out.flush()
        total, idle = t, i
PY
    echo $!
}

printf 'essai\tnetem\tvariante\tsortie\tdesync\tresimulees\tinterruptions\treprises\tdeconnexions\tsilence_max_ms\tcpu_moy_pct\tcpu_max_pct\n' > "$OUTDIR/summary.tsv"
IFS='|' read -r -a cells <<< "$CELLS"
IFS='|' read -r -a variants <<< "$VARIANTS"
n=0
for cell in "${cells[@]}"; do
    for variant in "${variants[@]}"; do
        reps=$(reps_for "$variant")
        for ((rep = 1; rep <= reps; rep++)); do
            n=$((n + 1))
            label="$n-$(echo "$cell" | tr -c 'a-z0-9\n' '_')-$variant-$rep"
            dir="$OUTDIR/$label"
            mkdir -p "$dir"
            envs=()
            case "$variant" in
                default) ;;
                timeout*) envs+=("ALACOD_DISCONNECT_TIMEOUT_MS=${variant#timeout}") ;;
                unreliable) envs+=("ALACOD_P2P_CHANNEL=unreliable") ;;
                *) echo "variante inconnue : $variant"; exit 2 ;;
            esac
            loadpid=$(sample_load "$dir/load.log")
            echo "== essai $n : netem='$cell' variante=$variant rép=$rep $(date +%T) (charge 1 min $(cut -d' ' -f1 /proc/loadavg))"
            env "${envs[@]}" NETEM="$cell" STRESS=0 MIN_RESIMULATED=0 ALACOD_SKIP_HUB=1 GAME_ARGS="$GAME_ARGS" \
                P2P_WAIT_CALM="${P2P_WAIT_CALM:-}" OUT="$dir" TIMEOUT=900 \
                "$HERE/p2p-stress.sh" "$GAME" "$SCRIPT" "$FRAMES" > "$dir/run.out" 2>&1
            code=$?
            kill "$loadpid" 2>/dev/null
            wait "$loadpid" 2>/dev/null
            python3 "$HERE/p2p-silences.py" --load "$dir/load.log" "$dir/p2p-0.log" "$dir/p2p-1.log" > "$dir/silences.txt" 2>&1
            desync=$(cat "$dir"/p2p-*.log | grep -c "DesyncDetected")
            resim=$(for i in 0 1; do sed -n 's/.*frames resimulées après rollback : \([0-9][0-9]*\).*/\1/p' "$dir/p2p-$i.log" | tail -1; done | paste -sd/ -)
            inter=$(grep '^silences' "$dir/silences.txt" | sed -n 's/.*interruptions=\([0-9]*\).*/\1/p' | paste -sd/ -)
            resumed=$(grep '^silences' "$dir/silences.txt" | sed -n 's/.* reprises=\([0-9]*\).*/\1/p' | paste -sd/ -)
            disc=$(grep '^silences' "$dir/silences.txt" | sed -n 's/.*deconnexions=\([0-9]*\).*/\1/p' | paste -sd/ -)
            smax=$(grep '^silences' "$dir/silences.txt" | sed -n 's/.*max_ms=\([0-9]*\).*/\1/p' | sort -n | tail -1)
            cpu=$(awk '{s+=$2; if ($2>m) m=$2; n++} END{if (n) printf "%.0f\t%.0f", s/n, m; else printf "?\t?"}' "$dir/load.log")
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$n-$rep" "$cell" "$variant" "$code" "$desync" "$resim" "$inter" "$resumed" "$disc" "${smax:-0}" "$cpu" >> "$OUTDIR/summary.tsv"
            tail -1 "$OUTDIR/summary.tsv"
        done
    done
done
echo "p2p-d87 : terminé, voir $OUTDIR/summary.tsv"
