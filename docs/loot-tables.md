# Custom Loot Tables

Loot is defined entirely in versioned `.ron` files shipped with the server or provided by as a custom configuration.
Still, the `Media.pk2` file from an install provides the specifically available items.

## Overview

- Definitions live in `.ron` files inside the directory configured under
  `[game.loot]` (`directory`). Relative paths resolve against the server process working directory, consistent with
  other configuration values.
- All files below that directory are loaded **recursively** and sorted lexically, but file order does not control
  precedence.
- Files are read once at startup and compiled into immutable runtime tables. There is no hot reloading; invalid
  definitions abort startup with source-aware error messages collected from all files.
- Every definition must validate against the Media catalogue: item codes must exist, monsters in assignments must be
  known, and gold/equipment generators must be able to produce output for every applicable monster level.

## Configuration

```toml
[game.loot]
directory = "configs/loot"
rate = 1.0
```

- `rate` globally multiplies the expected successful pool selections. It must be finite and non-negative. Values above
  `1.0` produce multiple selections per attempt instead of saturating at 100%
  (see [probability semantics](#probability-semantics)).

## File structure

Every file has a versioned root and may contribute any subset of definitions:

```ron
(
    version: 1,
    default_profile: None,
    pools: [],
    profiles: [],
    level_assignments: [],
    monster_assignments: [],
    rarity_modifiers: [],
    party_modifier: None,
)
```

Definitions use vectors of named records rather than maps, which lets the loader detect duplicates reliably:

- **Pools** and **profiles** are referred to by name. Names must be unique across all files; duplicates are an error
  reporting both locations.
- Exactly **one** default profile must be declared across the whole directory.
- A monster may have at most **one exact assignment**.
- **Level assignment** ranges are inclusive and may not overlap.
- At most one modifier may be declared per rarity kind, and at most one **party modifier** overall.

## Probability semantics

A profile contains rolls. Each roll names a pool and has a positive integer `attempts` count and a `chance` in the range
`0..=1`. Rolls are independent, so gold, consumables, and equipment can drop together.

For each configured attempt:

```text
expected selections = chance × global rate × chance modifiers
```

The expected value is split into `floor(expected)` guaranteed selections plus one extra selection made with probability
equal to the fractional remainder. This results in rates above `1.0` being actually useful. The rarity
`attempts_multiplier` (an integer) multiplies the attempt count before this calculation.

A successful selection picks exactly one entry of the pool using its integer weight; weights do not need to sum to any
fixed value.

An internal safety limit bounds generation at **128 drops per death**. During compilation the worst-case output of every
plan (attempts × modifiers × chances) is checked against this limit; configurations able to exceed it are rejected.

## Assignment resolution

For each dead monster:

1. The base profile comes from the matching inclusive level band (`level_assignments`), or the default profile if no
   band matches.
2. An exact monster assignment modifies this:
    - `Add`: run the base profile, then the assigned profile.
    - `Replace`: run only the assigned profile.
3. Runtime modifiers are applied on top: first the rarity-kind modifier of the actual spawned variant
   (champion/giant/etc.), then the party modifier if the spawn carries the party bit.

Multipliers multiply and upgrade bonuses add.

## Generators

### `Gold`

Samples an amount from the level-appropriate inclusive range of `levelgold.txt`, applies the gold amount multipliers,
and picks the existing small/medium/large gold item based on the final amount. Missing or invalid ranges for applicable
levels will throw an error on startup.

### `Item`

Drops an exact stackable/expendable item by its code name:

```ron
generator: Item((
    code: "ITEM_ETC_HP_POTION_01",
    amount: Uniform(min: 1, max: 3),
))
```

Item codes resolve exclusively through data provided by `Media.pk2`. Supported records are ordinary stackable items
(potions, arrows/bolts, scrolls, etc.). Gold currency, pet/COS items, trade goods, avatars, inactive records, and
non-droppable records are rejected. Amount distributions must stay within the item's maximum stack size.

### `Equipment`

Selects generated equipment from a precompiled candidate set:

```ron
generator: Equipment((
    required_level: AroundMonster(min: -3, max: 2),
    origins: [Chinese, European],
    kinds: [Weapon, Shield, Clothing, Jewelry],
    item_rarities: [General],
    upgrade: Weighted([
        (weight: 90, value: 0),
        (weight: 9, value: 1),
        (weight: 1, value: 2),
    ]),
))
```

Candidates are filtered by required level relative to the killed monster's level (inclusive window), origin, broad kind,
and item rarity. This only includes equipment which can drop normally in the first place. The upgrade level comes from
the weighted distribution, plus any upgrade bonus from active modifiers. Variance is unset and no random blue/magic
options are generated in version 1. There must be at least one match for every monster level that can execute the
selector.

## Distributions

Amounts and upgrade levels use one of three distributions:

```ron
Fixed(5)
Uniform(min: 1, max: 3)
Weighted([(weight: 90, value: 0), (weight: 10, value: 1)])
```

All ranges are inclusive; weights must be positive.

## Example

A fuller example demonstrating consumables, equipment, level bands, a monster-specific extra profile, and modifiers:

```ron
(
    version: 1,
    default_profile: Some("world-default"),

    pools: [
        (
            name: "gold",
            entries: [(weight: 1, generator: Gold)],
        ),
        (
            name: "basic-consumables",
            entries: [
                (
                    weight: 3,
                    generator: Item((
                        code: "ITEM_ETC_HP_POTION_01",
                        amount: Uniform(min: 1, max: 3),
                    )),
                ),
                (
                    weight: 2,
                    generator: Item((
                        code: "ITEM_ETC_MP_POTION_01",
                        amount: Uniform(min: 1, max: 2),
                    )),
                ),
            ],
        ),
        (
            name: "level-equipment",
            entries: [
                (
                    weight: 1,
                    generator: Equipment((
                        required_level: AroundMonster(min: -3, max: 2),
                        origins: [Chinese, European],
                        kinds: [Weapon, Shield, Clothing, Jewelry],
                        item_rarities: [General],
                        upgrade: Weighted([
                            (weight: 90, value: 0),
                            (weight: 9, value: 1),
                            (weight: 1, value: 2),
                        ]),
                    )),
                ),
            ],
        ),
    ],

    profiles: [
        (
            name: "world-default",
            rolls: [
                (pool: "gold", attempts: 1, chance: 1.0),
                (pool: "basic-consumables", attempts: 1, chance: 0.08),
                (pool: "level-equipment", attempts: 1, chance: 0.015),
            ],
        ),
        (
            name: "early-game",
            // Level assignments replace the *default* profile for a band.
            rolls: [
                (pool: "gold", attempts: 1, chance: 1.0),
                (pool: "basic-consumables", attempts: 1, chance: 0.15),
            ],
        ),
        (
            // Executed in addition to whatever base profile applies.
            name: "tiger-woman-extra",
            rolls: [
                (pool: "level-equipment", attempts: 2, chance: 0.5),
            ],
        ),
    ],

    // Replace the default base profile for matching (inclusive) level bands.
    // Ranges may not overlap.
    level_assignments: [
        (levels: (min: 1, max: 20), profile: "early-game"),
    ],

    // Each monster may have at most one exact assignment.
    monster_assignments: [
        (
            monster: "MOB_CH_TIGERWOMAN",
            mode: Add,
            profile: "tiger-woman-extra",
        ),
    ],

    rarity_modifiers: [
        (
            rarity: Champion,
            attempts_multiplier: 2,
            chance_multiplier: 1.0,
            gold_amount_multiplier: 1.5,
            stack_amount_multiplier: 1.5,
            equipment_upgrade_bonus: Uniform(min: 0, max: 1),
        ),
        (
            rarity: Giant,
            attempts_multiplier: 3,
            chance_multiplier: 1.0,
            gold_amount_multiplier: 2.0,
            stack_amount_multiplier: 2.0,
            equipment_upgrade_bonus: Uniform(min: 0, max: 2),
        ),
    ],

    party_modifier: Some((
        attempts_multiplier: 1,
        chance_multiplier: 1.5,
        gold_amount_multiplier: 1.5,
        stack_amount_multiplier: 1.5,
        equipment_upgrade_bonus: Fixed(0),
    )),
)
```

## Shipped defaults

The server ships with `configs/loot/base.ron`, a conservative definition giving every ordinary monster death one
guaranteed gold drop drawn from `levelgold.txt`. Everything else is intentionally left to operators. Note that item code
names depend on your `Media.pk2` version; verify codes against your own client before enabling additional pools.
