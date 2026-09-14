# Inventory model and queries

Answer "do I have X, how much, and where?" from the save file, across every container the player owns.

**Depends on:** nothing in code. Needs the item name table described below before any listing is readable.

---

## Context

The atlas knows where the player has been but nothing about what they carry. The save file records every inventory grid the game has: exosuit, freighter, the ten storage containers, every owned ship, exocraft, and multi-tool, plus refiner buffers and a handful of minor containers. Each grid lists its occupied slots with the item ID, amount, and stack maximum. Nothing in the tool reads any of it.

The motivating question was "do I have gold and silver, and where?" The answer from the save is: Gold (`^ASTEROID2`) 11,297 total, in Storage 1 (9,999 and 927) and the exosuit (371); Silver (`^ASTEROID1`) 2,959 in Storage 1 and the freighter; Platinum (`^ASTEROID3`) 2,994 in Storage 1. That report is the target output of this arc.

---

## What the save contains (verified)

All paths are under `BaseContext.PlayerStateData` (or the expedition context; the active one applies).

### Grids

| Key | Grid | Occupied in sample | Notes |
|-----|------|--------------------|-------|
| `Inventory` | 10x12 | 30 | exosuit general |
| `Inventory_Cargo` | 7x5 | 0 | exosuit cargo (high-capacity) |
| `Inventory_TechOnly` | 10x6 | 42 | exosuit technology |
| `FreighterInventory` (+`_Cargo`, `_TechOnly`) | 7x5 | 24 | class A |
| `Chest1Inventory` .. `Chest10Inventory` | 10x6 each | 5 to 49 | the ten storage containers |
| `ShipOwnership[i].Inventory` (+`_Cargo`, `_TechOnly`) | 7x5 | 6 on the primary ship | 12 entries, 8 real ships, 4 empty stubs with no `Resource.Filename` |
| `VehicleOwnership[i].Inventory` | | 0 | 7 exocraft; `Location` holds the base address where parked |
| `Multitools[i].Store` | | 17 to 18 | technology only |
| `WeaponInventory` | 10x6 | 18 | the equipped multi-tool |
| `RefinerBufferData[i].InventoryContainer` | | | 38 refiners; 2 held Chromatic Metal (180 and 49) |
| `ChestMagicInventory`, `ChestMagic2Inventory` | 10x6 | 11, 0 | meaning unknown, see open questions |
| `CookingIngredientsInventory`, `FishBaitBoxInventory`, `FoodUnitInventory`, `RocketLockerInventory`, `GraveInventory` | small | 0 to 1 | minor containers |

Currency sits beside them: `Units` (351,503,622 in the sample), `Nanites` (45,087), `Specials` (quicksilver, 240).

Every grid object has `Width`, `Height`, `Class.InventoryClass` (C/B/A/S), `Slots`, `SpecialSlots`, `ValidSlotIndices`, `StackSizeGroup`, and `Name` (empty in the sample). `PrimaryShip` is the index of the active ship in `ShipOwnership`.

### Slots

`Slots` lists occupied cells only. An empty cell is simply absent, so free space is the unlocked grid minus the occupied entries. Each slot:

```json
{"Id": "^ASTEROID2", "Amount": 9999, "MaxAmount": 9999, "Index": {"X": 2, "Y": 3},
 "Type": {"InventoryType": "Substance"}, "DamageFactor": 0.0, "FullyInstalled": true, "AddedAutomatically": false}
```

`InventoryType` is one of `Substance`, `Product`, or `Technology`. Technology slots carry `Amount` as charge or condition, not a count, and are excluded from holdings totals.

### Where a container physically is

- **Storage containers** are one inventory per number, reachable from every base that has the matching object placed. `^CONTAINER0` in a planet base and `^FRE_ROOM_STORE0` on the freighter both open `Chest1Inventory`. The sample has `^CONTAINER0` at two planet bases and `^FRE_ROOM_STORE0` through `9` on the freighter. So "where" for storage is a list of access points, derived from base objects.
- **Exocraft** carry the address of the base they are parked at in `Location` (four at the farm base, one at another system, two at zero).
- **Ships** carry `Location` and `Position` too, but every ship in the sample reads zero. The non-primary ships are presumably aboard the freighter. Until the field is understood, ships are labelled "with you" (primary) or "on the freighter".
- **Ship type** comes from `Resource.Filename`: the path segment before the file name is `FIGHTERS`, `DROPSHIPS` (hauler), `SCIENTIFIC` (explorer), `SAILSHIP` (solar), `S-CLASS` (exotic), `BIGGS` (living ship). Ships in the sample are unnamed, so the type is the label.

### Item IDs and names

The save uses internal IDs and carries no display names. The sample save holds 267 distinct IDs across all containers. Known correspondences that matter for the first tests:

| ID | Name |
|----|------|
| `^ASTEROID1` | Silver |
| `^ASTEROID2` | Gold |
| `^ASTEROID3` | Platinum |
| `^STELLAR2` | Chromatic Metal |
| `^LAND1` | Ferrite Dust |
| `^CATALYST1` | Sodium |
| `^OXYGEN` | Oxygen |
| `^GAS1`, `^GAS2`, `^GAS3` | Sulphurine, Radon, Nitrogen |

`KnownProducts` (835 entries) and `KnownTech` (137) are ID lists too and could seed a "known recipes" view later, but they are out of scope here.

---

## Architecture

```
save (PlayerStateData) ──► nms-save::model::inventory ──► nms-core::holdings ──► GalaxyModel.holdings
                                                                                        │
                                       nms-query::inventory (have / inventory / items) ◄┘
                                                        │
                                   CLI `nms have` · REPL `have` · MCP `have_item`
```

- **nms-core** gains the domain types and the item name table. No serde of save shapes here.
- **nms-save** gains serde structs for the grids and owners, plus `to_holdings()`.
- **nms-graph** stores `Holdings` on the model so all three interfaces and the cache see one copy.
- **nms-query** gains `inventory.rs` with the three queries and display functions.
- **nms-cache** stores holdings alongside systems, and gains a format version.

### Core types (`nms-core/src/holdings.rs`)

```rust
pub struct ItemId(pub String);            // "^ASTEROID2", stored with the caret

pub enum ItemKind { Substance, Product, Technology }

pub struct ItemStack {
    pub id: ItemId,
    pub kind: ItemKind,
    pub amount: u32,
    pub max: u32,
    pub slot: (u8, u8),
}

pub enum ContainerKind {
    Exosuit, ExosuitCargo, ExosuitTech,
    Freighter, FreighterCargo, FreighterTech,
    Storage(u8),                          // 1..=10
    Ship { index: u8, primary: bool, ship_type: ShipType },
    ShipTech { index: u8 },
    Exocraft { index: u8 },
    MultiTool { index: u8 },
    Refiner { index: u8 },
    Other(String),                        // cooking, bait, grave, ...
}

pub struct Container {
    pub kind: ContainerKind,
    pub class: InventoryClass,
    pub width: u8,
    pub height: u8,
    pub unlocked_slots: u16,              // from ValidSlotIndices when present, else width*height
    pub stacks: Vec<ItemStack>,
}

pub struct Holdings {
    pub containers: Vec<Container>,
    pub units: i64,
    pub nanites: i64,
    pub quicksilver: i64,
    pub ships: Vec<ShipSummary>,          // type, class, tech count, location
    pub exocraft: Vec<VehicleSummary>,    // type, parked-at address
}
```

Technology stacks are kept (they drive `list ships` tech counts and a later "installed tech" view) but every totalling query filters them out.

### Item name table (`nms-core/src/items.rs`)

A bundled JSON file, `crates/nms-core/data/items.json`, parsed once into a `OnceLock<HashMap<ItemId, ItemInfo>>`:

```json
{"ASTEROID2": {"name": "Gold", "symbol": "Au", "kind": "Substance"}}
```

Lookup is by ID with the caret stripped. `ItemInfo::display(&ItemId)` returns the name or, when unknown, the raw ID without the caret. Searches accept either a display-name substring (case-insensitive) or an ID substring, so `have gold` and `have asteroid2` both work.

**Sources.** A public refiner-recipe gist keys roughly 90 substances by internal ID with name and symbol, and the wiki's Item Id List covers products and technology. A one-off script under `scripts/` merges them into `items.json`; the script and its inputs are committed so the table is reproducible when the game adds items. Item names are facts about the game and are not a licensing concern; the script's inputs are cited in a comment.

**Coverage target.** Every ID present in the sample save (267) resolves to a name, verified by a test that loads the sample fixture and asserts no fallback IDs appear in `list items`.

### Save structs (`nms-save/src/model.rs`)

```rust
#[serde(rename_all = "PascalCase")]
pub struct Inventory { width, height, class: InventoryClass, #[serde(default)] slots: Vec<InventorySlot>, #[serde(default)] valid_slot_indices: Vec<SlotIndex>, ... }

pub struct InventorySlot { id: String, amount: i64, max_amount: i64, index: SlotIndex, #[serde(rename = "Type")] slot_type: SlotType }

pub struct ShipOwnership { name, resource: ResourceRef, inventory, inventory_cargo, inventory_tech_only, location: PackedGalacticAddress, ... }
pub struct VehicleOwnership { ... same shape ... }
pub struct Multitool { name, store: Inventory, ... }
```

`PlayerStateData` gains the grid fields, `chest1_inventory` .. `chest10_inventory` (a small macro or an array built in `to_holdings()`), `ship_ownership`, `vehicle_ownership`, `multitools`, `primary_ship`, and `refiner_buffer_data`. All default when absent so older or partial saves still load. Amounts are `i64` in serde and clamped to `u32` on conversion; the game has written negative units before.

### Queries (`nms-query/src/inventory.rs`)

```rust
pub struct HaveQuery { pub pattern: String, pub kind: Option<ItemKind> }
pub struct HaveResult { pub id: ItemId, pub name: String, pub total: u64, pub locations: Vec<HaveLocation> }
pub struct HaveLocation { pub container: ContainerKind, pub label: String, pub amount: u32, pub max: u32, pub access: Vec<String> }

pub struct InventoryQuery { pub container: Option<ContainerFilter>, pub free_only: bool }
pub struct ListItemsQuery { pub kind: Option<ItemKind>, pub min_amount: u32 }
```

`HaveLocation.access` is where the container can be opened: base names for storage containers, the parked base for exocraft, "with you" or "on the freighter" for ships. It is derived once when the model is built, from base objects, and stored on the container.

A pattern that matches several items (`have gas`) returns one `HaveResult` per item, sorted by total.

### Commands

CLI and REPL share the wording. MCP tools mirror the three queries and return JSON with the same fields.

```
nms have gold                        # total, then per-location rows
nms have gas --type substance        # several matches, one block each
nms inventory                        # every container: occupied/unlocked, class
nms inventory storage 3              # one container's contents as a grid listing
nms inventory --free                 # free slots per container, most free first
nms list items                       # everything, sorted by amount
nms list items --type product --min 100
nms list ships                       # index, type, class, tech count, location, primary marker
nms list exocraft                    # index, type, parked at
```

Sample `have gold` output, from the sample save:

```
 HOLDINGS: Gold (ASTEROID2)            Total: 11,297
 Location     Amount   Stack   Reachable from
 Storage 1     9,999   9,999   Radioactive Base, Smira Colony, Freighter
 Storage 1       927   9,999   Radioactive Base, Smira Colony, Freighter
 Exosuit         371   9,999   with you
```

Two stacks in the same container stay as two rows: they are two slots, and the stack headroom column is per slot.

### Live updates

Out of scope for the first cut. The watcher's `SaveDelta` has no inventory section; adding one (stacks that changed amount, stacks that appeared or vanished) is a follow-up once the model side is stable. Until then a save change rebuilds holdings along with everything else on the next reload.

### Cache

`CacheData` gains a `holdings` section and a `format_version: u32` constant checked on read; a mismatch is treated as a stale cache and rebuilt from the save. This is the first change that makes a cache written by an older binary unusable, so the version field lands with it.

---

## Files to create or modify

| File | Change |
|------|--------|
| `crates/nms-core/src/holdings.rs` | new: `Holdings`, `Container`, `ItemStack`, `ContainerKind`, `ShipType` |
| `crates/nms-core/src/items.rs` | new: name table loader and display helper |
| `crates/nms-core/data/items.json` | new: generated ID-to-name table |
| `scripts/gen-items.py` | new: merges the public sources into `items.json` |
| `crates/nms-save/src/model.rs` | grids, slots, ship and vehicle ownership, multi-tools, refiner buffers |
| `crates/nms-save/src/convert.rs` | `PlayerStateData::to_holdings()` |
| `crates/nms-graph/src/model.rs` | `holdings` field; access points derived from base objects |
| `crates/nms-cache/src/data.rs`, `serialize.rs` | holdings in `CacheData`; format version |
| `crates/nms-query/src/inventory.rs` | new: the three queries |
| `crates/nms-query/src/display.rs` | `format_have`, `format_inventory`, `format_items` |
| `crates/nms-cli/src/have.rs`, `inventory.rs`, `list.rs` | commands |
| `crates/nms-copilot/src/commands.rs`, `dispatch.rs`, `completer.rs` | REPL commands and completion of item names |
| `crates/nms-copilot/src/mcp/tools.rs` | `have_item`, `inventory_summary`, `list_ships` |
| `data/test/multi_system_save.json` | inventories and ownership added to the fixture |

---

## Key design decisions

- **Holdings live on the model, not in a side channel.** One copy, one cache, one reload path, same as systems. The REPL's live watcher then needs no special case.
- **Technology is stored but never totalled.** A technology slot's `Amount` is charge, and counting it as inventory would produce nonsense totals.
- **Unknown IDs are shown, not hidden.** The raw ID is the fallback so a missing name is visible and reportable rather than silently dropped.
- **Storage "location" means access points.** The game treats a numbered container as one inventory; reporting each placement as a separate location would double-count.
- **Search matches names and IDs.** Players know "Gold"; tests and power users know `ASTEROID2`.
- **Ship location stays conservative.** Report only what the save proves until the `Location` field is understood.

---

## Testing strategy

- Serde: a `PlayerStateData` fixture with every grid key present and another with none, asserting defaults.
- Conversion: occupied-only slots produce correct free counts; `ValidSlotIndices` limits unlocked slots when present.
- Name table: every ID in the sample fixture resolves; a made-up ID falls back to itself.
- Queries: `have` totals across containers, splits stacks, and filters technology; pattern matching by name and by ID; `inventory --free` ordering.
- Display: table contents for each formatter under the plain theme.
- Integration: CLI `nms have gold` against the fixture asserts the total and each location row.
- Cache: a cache with an older format version is rebuilt.

---

## Verification

Against the sample save, after the change:

- `nms have gold` reports 11,297 with the three rows above; `nms have silver` reports 2,959; `nms have platinum` reports 2,994.
- `nms list items` shows no raw IDs.
- `nms list ships` shows 8 ships, index 3 marked primary, class S, type exotic.
- `nms inventory --free` lists Storage 2 as the emptiest container (5 of 60 occupied).

---

## Open questions

1. What do non-zero ship `Location` values mean, and does zero mean "aboard the freighter"? Needs a save where a ship was left on a planet.
2. What are `ChestMagicInventory` and `ChestMagic2Inventory`? Eleven items sit in the first. Candidates: the Space Anomaly item vault, the settlement storage, or a legacy container.
3. Does `ValidSlotIndices` list unlocked cells, and is it always populated? The sample's exosuit grid is 10x12 but the game caps unlocked slots lower.
4. `SpecialSlots` marks supercharged slots. Worth surfacing in a later "installed tech" view, not here.
5. The `Inventory_Cargo` grids are all empty in the sample. Confirm they are the high-capacity slots introduced with the inventory rework and label them accordingly.
