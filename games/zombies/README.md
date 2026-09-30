# Zombies Game Content

## Overview

The `zombies` game is a Call of Duty Zombies-style shooter implemented with the alacod engine. This directory contains all game content (maps, characters, weapons, economy, UI) loaded at runtime via the manifest (`game.ron`).

## Asset Structure

All assets are relative to `games/zombies/assets/`.

### Directories

- **`ZombieShooter/Sprites/`** - Character sprites and configs
  - `Character/` - Player and character definitions
    - `player_config.ron` - Player character configuration
    - `weapons.ron` - Ranged weapon definitions
    - `player_sheet.ron` - Player sprite sheet config
    - `player_animation.ron` - Player animation definitions
  - `Zombie/` - Zombie and enemy definitions
    - `zombie_config.ron` - Basic zombie configuration
    - `zombie_hard_config.ron` - Tougher zombie variant
    - `zombie_full_config.ron` - Full-featured zombie
    - `*.ron` / `*.png` - Zombie sprite sheets and animations

- **`weapons/melee/`** - Melee weapon definitions
  - `melee_weapons.ron` - Melee weapon configurations (knife, club, etc.)

- **`waves/`** - Wave configuration
  - `wave_config.ron` - Zombie spawning rules, scaling, difficulty

- **`economy/`** - In-game economy definitions
  - `economy.ron` - Kill/hit/repair point values
  - `perks.ron` - Perk machines and their stat modifiers

- **`exemples/`** - Map definitions
  - `test_map.ldtk` - Main playable map with 3 rooms
  - `test_map_shop.ldtk` - Shop reference map (used in scenarios)
  - `atlas/` - Tileset and map graphics

- **`ui/`** - UI configurations
  - `hud.ron` - Head-up display layout and widgets
  - `camera.ron` - Camera behavior settings

- **`items/`** - Item definitions (TODO: T2.5)
  - `powerups.ron` - Power-up drop table (format defined in T2.5)

## How to Add Content

### Add a Weapon

1. **Define the weapon** in `weapons/melee/melee_weapons.ron` or create a new ranged weapon in `ZombieShooter/Sprites/Character/weapons.ron`:
   ```ron
   "club": (
       config: (
           name: "Club",
           damage: "12.0",
           range: "30.0",
           attack_pattern: SingleStrike,
           attack_duration_frames: 10,
           cooldown_frames: 30,
           knockback_force: "5.0",
           stamina_cost: "20.0",
       ),
       sprite_config: (
           name: "club",
           index: 0,
           weapon_offset: (-10.0, 0.0),
       ),
   ),
   ```

2. **Add to a character's starting weapons** in `ZombieShooter/Sprites/Character/player_config.ron`:
   ```ron
   starting_weapons: ["pistol", "club"],
   ```

3. **Place on a map** (optional): Add a `WeaponLocation` entity in LDtk:
   - Set `weapon: "club"`
   - Set `price: 750` (or appropriate value)

4. **Test**: Run `cargo run -p content --bin alacod --profile headless -- lint games/zombies` to validate the reference.

### Add a Perk

1. **Define the perk** in `economy/perks.ron`:
   ```ron
   "who_dares_wins": (
       name: "Who Dares Wins",
       price: 1500,
       modifiers: [
           (stat: FireRate, op: Mul, value: "1.2"),
       ],
   ),
   ```

2. **Place on a map**: Add a `SodaLocation` entity in LDtk:
   - Set `perk: "who_dares_wins"`

3. **Test**: Run lint and scenarios.

### Add a Wall Weapon or Perk Machine to the Map

Use the provided Python script to update `exemples/test_map.ldtk`:

```bash
python3 << 'EOF'
import json
import uuid

# Load map
with open('games/zombies/assets/exemples/test_map.ldtk', 'r') as f:
    test_map = json.load(f)

# Find level and Entities layer, add entity with proper UUID and position
# See script in T2.6 implementation for full details
EOF
```

**Key details**:
- Positions must be on a 16px grid boundary (aligned to LDtk cell positions)
- Place wall weapons and machines against interior walls, not on player spawns, doors, or windows
- Keep `iid` (instance ID) unique - generate a UUID4 for each new entity
- Weapon prices follow CoD: cheap entry weapons (500), medium (1000), high (1500+)

## Game Manifest (`game.ron`)

The manifest declares all content directories and entry point:

```ron
(
    name: "zombies",
    content_folders: [
        (path: "ZombieShooter/Sprites/Character/player_config.ron", kind: "Character"),
        (path: "ZombieShooter/Sprites/Zombie/zombie_config.ron", kind: "Character"),
        // ... more content folders ...
        (path: "economy/economy.ron", kind: "Economy"),
        (path: "economy/perks.ron", kind: "Perk"),
    ],
    entry: (
        start_map: "exemples/test_map.ldtk",
        default_seed: 123456,
    ),
)
```

Run `cargo run -p content --bin alacod --profile headless -- lint games/zombies` to validate.

## Validation

### Lint

Checks all references, types, and values:

```bash
cargo run -p content --bin alacod --profile headless -- lint games/zombies
```

**What it validates**:
- All referenced weapons, perks, maps exist
- No duplicate IDs
- All `Fixed`-point numbers are strings (`"100.0"` not `100.0`)
- Stat values are in valid ranges
- LDtk entities have required fields

### Scenarios

Test content with scripted gameplay:

```bash
make test_scenarios SCENARIO=shop_tour
```

### Play

Run the game locally:

```bash
cargo run -p zombies --profile headless
```

## Known Limitations & Future Work

1. **Sprite organization** (A5): Sprites are currently stored under `ZombieShooter/Sprites/` and named in code (`crates/game/src/global_asset.rs`). Future work will move to a data-driven asset registry.

2. **Power-ups** (T2.5): `items/powerups.ron` format is being defined. Wall weapons and perks work; pickups on the ground are next.

3. **Melee weapons**: Club and other melee weapons exist but may need additional wall placement configurations.

## References

- **Conventions**: `docs/conventions.md` - LDtk layers, entity types, RON formats
- **Engine**: `CLAUDE.md` - Determinism rules, deferred spawning, rollback mechanics
- **Plan**: `docs/plan-engine.md` - Overall engine architecture and clones
