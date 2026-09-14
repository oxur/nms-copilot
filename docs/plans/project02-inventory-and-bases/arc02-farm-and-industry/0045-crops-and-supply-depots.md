# Crops and supply depots

Report crop readiness and supply-depot fill levels for every base, from the base objects stored in the save.

**Depends on:** typed base objects in `nms-save` (shared groundwork). The item name table from arc01 is used for crop yields and, later, depot contents, but the first cut works without it.

---

## Context

A base in the save is a list of placed objects. Each object records what it is, where it sits relative to the base, a timestamp, and a 64-bit `UserData` field. For crops, extractors, supply depots, and batteries, `UserData` holds the object's simulation state at the moment it was last saved. That state is enough to answer "is anything ready to harvest?" exactly, and "how full are the depots?" as of the last visit.

The tool currently reads a base's name, type, and address and discards its objects. This arc parses them.

---

## What the save contains (verified)

### Base objects

Under `PlayerStateData.PersistentPlayerBases[i]`:

| Field | Meaning |
|-------|---------|
| `Name`, `BaseType.PersistentBaseTypes` | as today; types seen: `HomePlanetBase`, `FreighterBase`, `PlayerShipBase` |
| `GalacticAddress` | save-layout universe address |
| `Position`, `Forward` | base origin and orientation in world space |
| `LastUpdateTimestamp` | Unix seconds |
| `Objects[]` | placed objects |

Each object:

```json
{"ObjectID": "^SNOWPLANT", "Position": [5.68, 3.86, -4.03], "Up": [..], "At": [..],
 "Timestamp": 1789279852, "UserData": 15461882265600}
```

`Position` is base-relative. `Timestamp` is Unix seconds. `UserData` is a `u64`; for every object type below, the meaningful value is the high 32 bits (`UserData >> 32`) and the low 32 bits are zero, with the exceptions noted in open questions.

The sample save's farm base ("Farm, Nitrogen & Paraffinium") has 239 objects: 109 crops, 16 supply depots, 13 extractors, 5 batteries, 10 generators, 2 bio-generators, plus structure.

### Crops: `UserData >> 32` is seconds remaining

| ObjectID | Crop | Growth time | Observed values |
|----------|------|-------------|-----------------|
| `^LUSHPLANT` | Star Bulb | 4 h (14,400 s) | 14,400 on all 16 |
| `^RADIOPLANT` | Gamma Root | 4 h | 14,400 on all 32 |
| `^TOXICPLANT` | Fungal Mould | 4 h | 14,400 on all 10 |
| `^POOPPLANT` | Coprite | 4 h | 14,400 on all 3 |
| `^SNOWPLANT` | Frost Crystal | 1 h (3,600 s) | 3,600 on all 16 |
| `^BARRENPLANT` | Cactus Flesh | 16 h (57,600 s) | 34,065 on 13; 21,073 to 21,076 on 3 |
| `^SCORCHEDPLANT` | Solanium | 16 h | 34,073 on 7; 21,039 to 21,052 on 9 |

The growth times are the published ones. The 4 h and 1 h crops all read exactly their full growth time, meaning they were harvested at the last visit and restarted. The 16 h crops read below 57,600 in two clusters, meaning two planting batches part-way through. That pattern only fits "seconds remaining at `Timestamp`". So:

```
remaining_now = (UserData >> 32) - (now - Timestamp)      // seconds, may be negative
ready         = remaining_now <= 0
progress      = clamp(1 - remaining_now / growth_time, 0, 1)
```

`remaining_now` needs no constants. `progress` needs the growth-time table, which also gives the yield per harvest for a "harvest value" column later.

Crops not present in the sample, to be added to the table with IDs confirmed from a save that has them: Mordite (8 h), Gutrot Flower, NipNip Buds, Echinocactus. Freighter planter rooms (`^FRE_ROOM_PLANT1`) encode something else (see open questions).

### Industry: `UserData >> 32` is stored amount times 1,440

| ObjectID | Object | Observed | Reading |
|----------|--------|----------|---------|
| `^U_SILO_S` | Supply depot | 1,440,000; 1,330,620; 1,322,643; 607,863; 520,402 | 1,000; 924; 918; 422; 361 units |
| `^U_EXTRACTOR_S` | Mineral extractor | 360,000 on all 3 | 250 units, buffer full |
| `^U_GASEXTRACTOR` | Gas extractor | 360,000 on all 10 | 250 units, buffer full |
| `^U_BATTERY_S` | Battery | 45,000 on 4, 0 on 1 | charge in power units, 45,000 is full |
| `^U_BIOGENERATOR` | Bio-fuel reactor | 87,267 and 0 | unknown |
| `^U_GENERATOR_S` | Generator | 0 on all 10 | no state |

The 1,440 factor is inferred from two independent constants: a supply depot holds 1,000 units and reads 1,440,000 when full, and an extractor's buffer holds 250 units and reads 360,000 when full. Depots on one pipe network share a value, which matches the game treating a network as one pool: the 16 depots fall into five groups of identical values. **Treat the factor as inferred until verified in-game** (verification section).

```
depot_units    = (UserData >> 32) / 1440           // 0..=1000 per depot, network-shared
depot_fill     = depot_units / 1000
extractor_buf  = (UserData >> 32) / 1440           // 0..=250
battery_charge = UserData >> 32                     // 0..=45,000
```

### Snapshot semantics

Every object at the farm carries the same `Timestamp` (1789279852), later than the base's `LastUpdateTimestamp`. The game appears to rewrite object timestamps when it saves while the base is loaded. So `UserData` is the state at the last save taken at that base:

- **Crops are exact.** Their timers run on real time, so subtracting elapsed time is correct whether or not the player is present.
- **Depots are a lower bound.** Extractors only produce while the game simulates the base, and the hotspot class that sets the rate is not in the save. The tool reports "N units as of <time>, <elapsed> ago" and does not extrapolate.

### What is not in the object

Nothing on a depot or extractor says which resource it holds. Refiner contents are stored separately in `RefinerBufferData` (38 entries; two held Chromatic Metal in the sample), and there are eleven `StoredInteractions` tables keyed by world-space position that index into per-object state such as `MaintenanceInteractions`. Linking an object to those records means transforming its base-relative position into world space with the base's `Position`, `Forward`, and `Up`, then matching. That is a research task, not part of the first cut.

---

## Architecture

```
PersistentPlayerBases[].Objects[] ──► nms-save::model::BaseObject ──► nms-core::base::{Crop, Depot, Extractor, Battery}
                                                                                 │
                                    nms-query::farm (crops) · nms-query::industry (depots) ◄┘
                                                        │
                        CLI `nms crops` / `nms depots` · REPL · MCP `farm_status` / `industry_status`
```

Time-dependent logic takes `now` as a parameter everywhere so tests are deterministic.

### Core types (`nms-core/src/base.rs`)

```rust
pub enum CropKind { StarBulb, GammaRoot, FungalMould, Coprite, FrostCrystal, CactusFlesh, Solanium, Mordite, /* ... */ }

impl CropKind {
    pub fn from_object_id(id: &str) -> Option<Self>;
    pub fn growth_secs(self) -> u32;
    pub fn yield_item(self) -> ItemId;       // ^PLANT_SNOW etc.
    pub fn display_name(self) -> &'static str;
}

pub struct Crop { pub kind: CropKind, pub remaining_at_snapshot: i64, pub snapshot: DateTime<Utc> }
impl Crop {
    pub fn remaining(&self, now: DateTime<Utc>) -> Duration;   // saturating at zero
    pub fn ready(&self, now: DateTime<Utc>) -> bool;
    pub fn progress(&self, now: DateTime<Utc>) -> f64;
}

pub struct Depot { pub units: u32, pub capacity: u32, pub snapshot: DateTime<Utc>, pub network: u32 }
pub struct Extractor { pub kind: ExtractorKind, pub buffer: u32, pub capacity: u32, pub snapshot: DateTime<Utc> }
pub struct Battery { pub charge: u32, pub capacity: u32 }

pub struct BaseObjects { pub crops: Vec<Crop>, pub depots: Vec<Depot>, pub extractors: Vec<Extractor>, pub batteries: Vec<Battery>, pub other: usize }
```

`Depot.network` is a group index assigned to depots that share a value at the same base, so the display can say "4 depots, 1,000 units each" rather than list sixteen rows.

`PlayerBase` (already in `nms-core`) gains an `objects: BaseObjects` field, populated by the save conversion.

### Save structs (`nms-save/src/model.rs`)

```rust
#[serde(rename_all = "PascalCase")]
pub struct BaseObject {
    #[serde(rename = "ObjectID")] pub object_id: String,
    #[serde(default)] pub position: [f32; 3],
    #[serde(default)] pub timestamp: u64,
    #[serde(default)] pub user_data: u64,
}
```

`PersistentPlayerBase` gains `#[serde(default)] pub objects: Vec<BaseObject>` plus `last_update_timestamp`. Decoding into `BaseObjects` lives in `nms-core` so it is testable without serde.

### Queries and display

```rust
pub struct CropsQuery { pub base: Option<String>, pub now: DateTime<Utc> }
pub struct CropsRow { pub base: String, pub crop: CropKind, pub count: usize, pub ready: usize, pub next_ready: Option<Duration>, pub progress: f64 }

pub struct DepotsQuery { pub base: Option<String> }
pub struct DepotsRow { pub base: String, pub network: u32, pub depots: usize, pub units: u32, pub capacity: u32, pub snapshot_age: Duration }
```

Sample output against the sample save, at 16:00 on the day of the snapshot:

```
 CROPS: Farm, Nitrogen & Paraffinium                      as of 12:10 today
 Crop           Count   Ready   Next ready   Progress
 Gamma Root        32      32   now          100%
 Star Bulb         16      16   now          100%
 Fungal Mould      10      10   now          100%
 Frost Crystal     16      16   now          100%
 Coprite            3       3   now          100%
 Cactus Flesh      16       0   in 2h 01m (3 plants)   87%
 Solanium          16       0   in 2h 00m (9 plants)   87%
```

`Next ready` and `Progress` describe the nearest batch when a crop was planted in several batches; the remaining batches are listed on request with `--batches`.

```
 DEPOTS: Farm, Nitrogen & Paraffinium                     as of 12:10 today (3h 50m ago)
 Network   Depots   Stored   Capacity   Fill
 1              4    4,000      4,000   100%
 2              2    1,848      2,000    92%
 3              2    1,836      2,000    92%
 4              4    1,688      4,000    42%
 5              4    1,444      4,000    36%
 Extractors: 13, all buffers full (250/250)     Batteries: 4 full, 1 empty
```

### Commands

```
nms crops                      # every base with crops, grouped by base
nms crops "Farm"               # one base
nms crops --ready              # only crops ready now
nms depots                     # every base with depots or extractors
nms depots "Farm"
nms bases                      # one line per base: crops ready / total, depot fill, batteries
```

REPL: the same words. On model load and on each live refresh, the REPL prints a one-line notice when any crop batch is ready ("32 Gamma Root ready at Farm, Nitrogen & Paraffinium"). MCP: `farm_status` and `industry_status` returning the row structs as JSON, with `now` supplied by the server.

### Later steps (not in the first cut)

1. **Resource identity** for depots and extractors via the interaction tables, so a network reads "Nitrogen, 1,848 / 2,000".
2. **Rate learning.** When two consecutive saves at the same base show a network's value rising, record units per hour per network in the session and show a projected fill.
3. **Refiners.** `RefinerBufferData` already holds contents; link buffers to `^BUILD_REFINER*` objects and show what is sitting in them.
4. **Freighter farms.** `^FRE_ROOM_PLANT1` rooms carry a different encoding.

---

## Files to create or modify

| File | Change |
|------|--------|
| `crates/nms-core/src/base.rs` | new: crop table, decoders, `BaseObjects`; `PlayerBase.objects` |
| `crates/nms-save/src/model.rs` | `BaseObject`; `objects` and `last_update_timestamp` on bases |
| `crates/nms-save/src/convert.rs` | decode objects in `to_core_base()` |
| `crates/nms-graph/src/model.rs` | nothing structural; bases already carry `PlayerBase` |
| `crates/nms-cache/src/data.rs`, `serialize.rs` | base objects in the cached base record |
| `crates/nms-query/src/farm.rs`, `industry.rs` | new: queries |
| `crates/nms-query/src/display.rs` | `format_crops`, `format_depots`, `format_bases` |
| `crates/nms-cli/src/crops.rs`, `depots.rs`, `bases.rs` | commands |
| `crates/nms-copilot/src/commands.rs`, `dispatch.rs`, `watch.rs` | REPL commands; ready notice on load and refresh |
| `crates/nms-copilot/src/mcp/tools.rs` | `farm_status`, `industry_status` |
| `data/test/multi_system_save.json` | a base with crops, depots, extractors, batteries |

---

## Key design decisions

- **Report snapshots honestly.** Every depot row carries the snapshot time and its age. No extrapolation until the rate is measured, and then only labelled as an estimate.
- **Crops compute from the timer, not the growth table.** The table is only for progress percentages and yields, so a crop with an unknown ID still shows "ready in 2h 10m", just without a percentage.
- **Group depots by network.** Sixteen identical rows would hide the structure the player actually built.
- **`now` is a parameter.** Nothing in `nms-core` or `nms-query` reads the clock; the front ends pass it in, and tests pin it.
- **Unknown object state stays raw.** Bio-generator and freighter planter values are exposed in a debug listing (`nms bases --raw`) so future decoding has data to work from, but they are not interpreted.

---

## Testing strategy

- Decoders pinned to the values above: 15,461,882,265,600 decodes to a Frost Crystal with 3,600 s remaining; 6,184,752,906,240,000 to a depot at 1,000 units; 1,546,188,226,560,000 to a full extractor buffer; 193,273,528,320,000 to a full battery.
- Crop timing: with `now` equal to the snapshot, a 4 h crop at 14,400 shows 0% and 4 h remaining; at snapshot plus 5 h it is ready; at snapshot plus 2 h, 50%.
- Network grouping: depots with equal values at one base share a network index; equal values at different bases do not.
- Serde: bases without `Objects` still load; unknown `ObjectID`s count as `other`.
- Display and CLI integration against the fixture base, with a fixed `now`.

---

## Verification

In-game checks against the sample save, to confirm the inferred parts:

1. Open a supply depot in the farm's network 2 and compare its displayed fill to 924 units. If it matches, the 1,440 factor and the 1,000 capacity are confirmed.
2. Harvest one Cactus Flesh and save. The object's `UserData >> 32` should return to 57,600 and its `Timestamp` to the save time.
3. Leave the base for an hour, return, save, and compare a network's change against the extractor count to get a first rate reading.

---

## Open questions

1. What do the low 32 bits of `UserData` mean? Zero on crops and industry, 51 or 83 on freighter rooms, 256 on one bio-generator.
2. What does a bio-generator's 87,267 represent? Possibly fuel remaining in seconds.
3. Is a depot's capacity always 1,000, or do older or upgraded depots differ? The sample has only one depot type (`^U_SILO_S`).
4. Which `StoredInteractions` table maps to which per-object record, and does `Value` index into `MaintenanceInteractions` directly? Needed for resource identity.
5. Do crops in a bio-dome and hydroponic trays (`^PLANTTUBE`) use the same encoding? The sample has one tray with a 2022 timestamp and zero `UserData`, which suggests an empty tray.
