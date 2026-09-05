# Starter Gear

New-character inventory is configured with versioned RON files under the directory selected by `[game.starter-gear]`.
Definitions are loaded recursively, sorted for deterministic diagnostics, validated against `Media.pk2`, and compiled once
at startup. Invalid definitions abort startup. The shipped `base.ron`, `chinese.ron`, and `european.ron` files enumerate
all supported selectors as empty templates ready for item grants.

```toml
[game.starter-gear]
directory = "configs/starter-gear"
```

## Sets and matching

Each file contains any number of uniquely named sets:

```ron
(
    version: 1,
    sets: [
        (
            name: "everyone",
            applies_to: All,
            grants: [
                (code: "ITEM_ETC_HP_POTION_01", amount: 20),
            ],
        ),
        (
            name: "european-characters",
            applies_to: Race(European),
            grants: [],
        ),
        (
            name: "robe-characters",
            applies_to: Clothing(Robe),
            grants: [],
        ),
        (
            name: "crossbow-ammunition",
            applies_to: Weapon(Crossbow),
            grants: [
                (
                    code: "ITEM_ETC_AMMO_BOLT_01",
                    amount: 250,
                    placement: Equip(SecondaryWeapon),
                ),
            ],
        ),
    ],
)
```

Item codes in examples must exist in the server's own `Media.pk2` version.

A character receives matching sets additively in this order:

1. `All`
2. `Race(Chinese)` or `Race(European)`
3. their selected `Clothing` family
4. their selected `Weapon` type

Matching bag stacks of the same item are combined and split at the catalogue maximum stack size. Set names are unique
across all files; filename order never provides override behavior.

Supported clothing values are `Garment`, `Protector`, `Armor`, `Robe`, `LightArmor`, and `HeavyArmor`. Supported weapon
values are `Sword`, `Blade`, `Spear`, `Glavie`, `Bow`, `OneHandSword`, `TwoHandSword`, `Axe`, `WarlockStaff`, `Staff`,
`Crossbow`, `Dagger`, `Harp`, and `ClericRod`.

## Grants

A grant has these fields:

- `code`: required Media item code.
- `amount`: defaults to `1`; must fit one catalogue stack. Amounts from matching sets may combine into several stacks.
- `upgrade`: defaults to `0` and is only valid for equipment.
- `placement`: defaults to `Bag`. `Equip(slot)` places the item in a named equipment slot.

Equipment slots are `HeadArmor`, `ShoulderArmor`, `WristArmor`, `ChestArmor`, `LegArmor`, `FootArmor`, `Weapon`,
`SecondaryWeapon`, `Earring`, `Necklace`, `LeftRing`, and `RightRing`.

The client-selected starter pieces continue to occupy slots 1, 4, 5, and 6. Configured grants cannot replace those
pieces. The compiler also rejects incompatible equipment slots, race-incompatible equipment, duplicate occupied slots,
weapon-incompatible shields or ammunition, items above level 1, values that persistence cannot represent, oversized
stacks, inactive or unknown items, and any matching loadout that exceeds the 45-slot starter inventory.

## Character creation

Character Selection validates the selected character and equipment against `Media.pk2`, derives the race, clothing
family, and weapon type, resolves matching starter sets, and persists all items in the same transaction as the new
Character. Client-selected pieces must be active, ordinary, droppable level-1 equipment of the expected race, part, and
weapon family. Starter gear is not granted during World Entry, so logging in cannot grant it again.
