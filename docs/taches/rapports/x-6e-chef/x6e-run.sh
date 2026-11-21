#!/bin/bash
# Joue des scénarios avec le binaire de test déjà construit (make test_scenarios SCENARIO=a,b fait la même chose en recompilant).
# usage : CARGO_TARGET_DIR=<cible> x6e-run.sh scen1,scen2   (depuis la racine du dépôt)
B=$(ls -t "$CARGO_TARGET_DIR"/headless/build/scenario/*/out/scenarios-* "$CARGO_TARGET_DIR"/headless/deps/scenarios-* 2>/dev/null | head -1)
cd crates/scenario && ALACOD_MUTE=1 ALACOD_SCENARIO=$1 "$B" scenarios --nocapture 2>&1
