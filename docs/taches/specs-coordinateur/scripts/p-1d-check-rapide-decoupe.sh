#!/usr/bin/env bash
# Contrôle rapide équivalent (décision du coordinateur alacod, 20 nov 2026 : `make check_rapide` d'un bloc monte à ~47 Go de cible) :
# make check_rapide sans test_crates, puis test_crates découpé (un binaire d'intégration à la fois, supprimé après). À lancer depuis le worktree.
# LOGS : dossier des journaux (défaut : ./target/logs) ; CARGO_TARGET_DIR : cible (défaut : ./target).
ICI=$(cd "$(dirname "$0")" && pwd)
cd "$(git rev-parse --show-toplevel)" || exit 1
TGT=${CARGO_TARGET_DIR:-target}; LOGS=${LOGS:-target/logs}; mkdir -p "$LOGS"
S=$(git rev-parse --short HEAD); L=$LOGS/check-rapide-decoupe-$S.log
echo "# contrôle rapide équivalent (test_crates découpé, décision du coordinateur), engine $S, $(date -u), df $(df -h . | tail -1)" > $L
T0=$(date +%s)
echo "## make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_animation" >> $L
env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_animation >> $L 2>&1
c=$?; echo "make (sans test_crates) : code $c, $(( $(date +%s) - T0 )) s" >> $L
rm -rf "$TGT"/headless/build/scenario; scripts/target-sweep.sh "$TGT" >> $L 2>&1
[ $c -ne 0 ] && { echo "ÉCHEC avant test_crates" >> $L; exit $c; }
T1=$(date +%s)
bash "$ICI/p-1d-test-crates-decoupe.sh" $L; c=$?
echo "test_crates découpé : code $c, $(( $(date +%s) - T1 )) s ; total $(( $(date +%s) - T0 )) s" >> $L
scripts/target-sweep.sh "$TGT" >> $L 2>&1; rm -rf "$TGT"/headless/build/scenario
echo "fin $(date -u), df $(df -h . | tail -1)" >> $L
tail -8 $L
