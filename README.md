# Alacod

[![PR](https://github.com/estran-studio/alacod/actions/workflows/pr.yaml/badge.svg)](https://github.com/estran-studio/alacod/actions/workflows/pr.yaml)
[![Build & Deploy](https://github.com/estran-studio/alacod/actions/workflows/main.yaml/badge.svg)](https://github.com/estran-studio/alacod/actions/workflows/main.yaml)

Alacod is a modular Bevy engine for top-down 2D roguelikes and shooters with P2P rollback networking.

* [Documentation and Demo](https://alacod.bascanada.org)

## Quick Start

Use the Rust nightly toolchain selected by `rust-toolchain.toml` and the native system
prerequisites listed in [.github/workflows/pr.yaml](.github/workflows/pr.yaml).
`make zombies`, `make throne`, `make gungeon`, and `make isaac` launch the current games locally;
`make testbed` opens the mechanics sandbox.

Use `make test_tools` for fast validation-tool checks without building the engine.
`make check` runs format checks, deterministic-pattern and rollback-registration guards,
headless tests across the workspace, the scenario suite, and content lint. The full suite can
be lengthy; `make test_scenarios SCENARIO=idle` runs one scenario.

To play gungeon (solo, two local heroes, two players online, keys): [docs/jouer.md](docs/jouer.md).

[Current engine status and milestone exit criteria](docs/etat-engine.md) ·
[Engine plan](docs/plan-engine.md) · [Task journal](docs/taches.md)

P2P sessions exchange state checksums every 10 frames by default. Unit tests exercise delayed
inputs, rollback, and injected checksum divergence with two in-memory peers. Real WebRTC,
multi-machine and degraded-network validation remain separate checks.

For performance review, use `make perf_review PERF_BASELINE=/path/to/reference.json` after
measuring the same benchmark scenarios on the same idle machine. Historical scenario budgets
are regression floors; they do not establish the production rollback frame budget.

## Crates

| Module | Responsibility |
|---|---|
| [sim_core](crates/sim_core) | Shared simulation contracts and frame events |
| [bevy_fixed](crates/bevy_fixed), [utils](crates/utils) | Fixed-point math, RNG, rollback registration and traces |
| [combat](crates/combat), [stats](crates/stats) | Weapons, projectiles, collision, damage and modifiers |
| [effects](crates/effects), [behaviors](crates/behaviors), [items](crates/items) | Composable effects, AI and inventory |
| [world](crates/world), [map](crates/map), [map_ldtk](crates/map_ldtk) | Terrain, navigation, generation and LDtk adapters |
| [run](crates/run), [meta](crates/meta) | Run state, economy and profile contracts |
| [content](crates/content) | Game manifests, typed registry and content lint |
| [game](crates/game), [animation](crates/animation) | Runtime integration, networking and presentation |
| [scenario](crates/scenario), [bots](crates/bots), [telemetry](crates/telemetry) | Replays, integration tests, simulation campaigns and diagnostics |

These are module responsibilities, not a guarantee of ECS-only dependencies; remaining
simulation/presentation coupling is tracked in [the current status](docs/etat-engine.md).

## Major Bevy dependencies

* [bevy_common_assets](https://github.com/NiklasEi/bevy_common_assets)
* [bevy_ecs_ldtk](https://github.com/estran-studio/bevy_ecs_ldtk/tree/transform_ldtk_project)
* [bevy_ggrs](https://github.com/gschup/bevy_ggrs)
* [matchbox](https://github.com/johanhelsing/matchbox)
* [bevy_kira_audio](https://github.com/NiklasEi/bevy_kira_audio)
* [leafwing-input-manager](https://github.com/Leafwing-Studios/leafwing-input-manager)
* [bevy_light_2d](https://github.com/jgayfer/bevy_light_2d)