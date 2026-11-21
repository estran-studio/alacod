#!/bin/bash
# Journaux ggrs{…} d'un scénario, frames resimulées dédoublonnées. Exige un crochet de journal TEMPORAIRE en tête de
# `fn scenarios()` (crates/scenario/tests/scenarios.rs), retiré avant commit :
#   if let Ok(f) = std::env::var("X6E_LOG") { let _ = tracing_subscriber::fmt().with_env_filter(
#       tracing_subscriber::EnvFilter::new(f)).without_time().with_ansi(false).with_writer(std::io::stderr).try_init(); }
# usage : x6e-log.sh <scénario> [motif egrep]
X6E_LOG=${X6E_LOG:-info} "$(dirname "$0")/x6e-run.sh" $1 2>&1 | grep -oE "ggrs\{f=[0-9]+ [^}]*\}|\([a-z_]+-[0-9]+\) spawn at.*|chef[a-z_]*: .*" | awk '!seen[$0]++' | grep -E "${2:-.}"
