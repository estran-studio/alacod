# Alacod

[![PR](https://github.com/bascanada/alacod/actions/workflows/pr.yaml/badge.svg)](https://github.com/bascanada/alacod/actions/workflows/pr.yaml)
[![Build & Deploy](https://github.com/bascanada/alacod/actions/workflows/main.yaml/badge.svg)](https://github.com/bascanada/alacod/actions/workflows/main.yaml)

Alacod is a engine (set of bevy plugins) to build a 2D rogue like shooter with p2p networking using rollback.

* [Documentation and Demo](https://alacod.bascanada.org)

## Quick Start

To run the fast CI checks locally, use `make check` which runs format verification, unit tests, scenario tests, and forbidden pattern detection.

## Crates

* [animations](./crates/animation/Cargo.toml) for animation that support rollback
* [bevy_fixed](./crates/bevy_fixed/Cargo.toml) for fixed size math calculation in bevy
* [map](./crates/map/Cargo.toml) for rogue like map generation
* [map_ldtk](./crates/map_ldtk/Cargo.toml) LDTK implementation of the map generation
* [utils](./crates/utils//Cargo.toml) Utilis functionnality used in all crates



## Major Bevy dependencies

* [bevy_common_assets](https://github.com/NiklasEi/bevy_common_assets)
* [bevy_ecs_ldtk](https://github.com/bascanada/bevy_ecs_ldtk/tree/transform_ldtk_project)
* [bevy_ggrs](https://github.com/gschup/bevy_ggrs)
* [matchbox](https://github.com/johanhelsing/matchbox)
* [bevy_kira_audio](https://github.com/NiklasEi/bevy_kira_audio)
* [leafwing-input-manager](https://github.com/Leafwing-Studios/leafwing-input-manager)
* [bevy_light_2d](https://github.com/jgayfer/bevy_light_2d)