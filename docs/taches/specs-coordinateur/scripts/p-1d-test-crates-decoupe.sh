#!/usr/bin/env bash
# P-1d : test_crates découpé (feu vert du coordinateur, 2026-11-20) : même commande que la cible du Makefile, d'abord
# --lib --bins --doc pour tout le workspace, puis chaque test d'intégration seul (liste de cargo metadata), binaire supprimé
# juste après. Garde : / sous 12 Go → arrêt. Usage : p-1d-test-crates-decoupe.sh <journal>   (lancé depuis le worktree)
set -u
L=$1
export APP_VERSION=v0.0.0 CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1
unset CI
FEAT=$(./scripts/dev-features.sh features)
BASE=(cargo test -q --workspace --profile headless --no-default-features --no-fail-fast $FEAT)
SUM=/tmp/p1d-tc-summary.txt; : > $SUM
free_gb() { df -Pk . | awk 'NR==2 {print int($4/1048576)}'; }  # portable (Linux, macOS)
guard() { # lance "$@" en arrière-plan, l'arrête si le disque passe sous 12 Go ; rend son code
  "$@" & local P=$!
  while kill -0 $P 2>/dev/null; do
    if [ "$(free_gb)" -lt 12 ]; then echo "GARDE DISQUE $(free_gb) Go : arrêt" | tee -a $L; pkill -P $P; kill $P; return 99; fi
    sleep 10
  done
  wait $P
}
count() { # résumé passed/failed d'une sortie
  python3 -c "
import re,sys
p=f=0
for l in open(sys.argv[1]):
    m=re.search(r'test result: \w+\. (\d+) passed; (\d+) failed',l)
    if m: p+=int(m.group(1)); f+=int(m.group(2))
print(p,f)" "$1"
}
for part in "--lib --bins" "--doc"; do
  echo "## test_crates découpé : $part, $(date -u)" >> $L
  guard "${BASE[@]}" $part -- --exact --skip scenarios < /dev/null > /tmp/p1d-tc-out.txt 2>&1; c=$?
  grep -vE "^\s*(warning|-->|\||[0-9]+ \||= )|^\s+\^" /tmp/p1d-tc-out.txt >> $L
  read p f < <(count /tmp/p1d-tc-out.txt); echo "workspace $part code=$c passed=$p failed=$f" >> $SUM
  [ $c -eq 99 ] && exit 99
done
[ "${P1D_LIB_ONLY:-0}" = 1 ] && { echo "## Résumé (lib, bins, doc)" >> $L; cat $SUM >> $L; exit 0; }
cargo metadata --format-version 1 --no-deps | python3 -c "
import json,sys; d=json.load(sys.stdin)
for p in d['packages']:
    for t in p['targets']:
        if 'test' in t['kind']: print(p['name'], t['name'])" > /tmp/p1d-tc-list.txt
echo "## $(wc -l < /tmp/p1d-tc-list.txt) tests d'intégration (cargo metadata)" >> $L
while read pkg name; do
  [ "$(date -u +%H%M)" -ge 0330 ] && [ "$(date -u +%H%M)" -lt 0500 ] && { echo "03:30 UTC : arrêt" | tee -a $L; exit 98; }
  echo "## -p $pkg --test $name, $(date -u +%T), df $(free_gb) Go" >> $L
  guard "${BASE[@]}" --test "$name" -- --exact --skip scenarios < /dev/null > /tmp/p1d-tc-out.txt 2>&1; c=$?
  exe=$("${BASE[@]}" --test "$name" --no-run --message-format=json < /dev/null 2>/dev/null \
        | python3 -c "import json,sys
for l in sys.stdin:
    try: m=json.loads(l)
    except: continue
    if m.get('reason')=='compiler-artifact' and m.get('executable') and 'test' in m['target']['kind']: print(m['executable'])" | tail -1)
  grep -vE "^\s*(warning|-->|\||[0-9]+ \||= )|^\s+\^" /tmp/p1d-tc-out.txt >> $L
  read p f < <(count /tmp/p1d-tc-out.txt); echo "$pkg/$name code=$c passed=$p failed=$f" >> $SUM
  [ -n "$exe" ] && rm -f "$exe" "${exe}.d"
  [ $c -eq 99 ] && exit 99
done < /tmp/p1d-tc-list.txt
echo "## Résumé test_crates découpé (par binaire)" >> $L; cat $SUM >> $L
echo "binaires joués : $(($(wc -l < $SUM) - 1)) / $(wc -l < /tmp/p1d-tc-list.txt) tests d'intégration + lib/bins/doc ; failed : $(awk '{split($4,a,"=");s+=a[2]} END{print s}' $SUM), codes non nuls : $(grep -vc ' code=0 ' $SUM)" >> $L
