# Project 02 — Inventory and Bases

> What do I own, where is it, and what is happening at my bases?

Project 01 delivered the galactic atlas: systems, planets, routes, and the three interfaces that share one live model. Project 02 turns the same pipeline toward the player's holdings. The save file carries every inventory grid, every owned ship and vehicle, and every object placed at every base, and almost none of it is surfaced today.

Two arcs, each with its own plan document:

| Arc | Doc | Question it answers |
|-----|-----|---------------------|
| arc01-inventory | [0044](arc01-inventory/0044-inventory-model-and-queries.md) | Do I have this item, how much, and where is it? |
| arc02-farm-and-industry | [0045](arc02-farm-and-industry/0045-crops-and-supply-depots.md) | Are my crops ready, and how full are my supply depots? |

Both arcs were scoped against a real save on 2026-09-12 and 2026-09-13. Every field and constant quoted in the plans was read from that save unless marked as inferred or unverified.

## Shared groundwork

Both arcs need pieces that do not exist yet. Build them once, in this order:

1. **Item name table.** Inventory slots, refiner buffers, and crop yields are all keyed by the game's internal item IDs (`^ASTEROID2` is Gold). The save contains no display names. A bundled ID-to-name table in `nms-core`, with the raw ID as fallback, unblocks every listing in both arcs. Sources and licensing are discussed in 0044.
2. **Typed base objects.** `PersistentPlayerBases[].Objects[]` is currently ignored beyond the base name and address. Arc 02 is built entirely on it, and arc 01 needs it to say which base a storage container is reachable from.
3. **Model and cache growth.** The `GalaxyModel` and the rkyv `CacheData` both need new sections for holdings and base objects. The cache is invalidated by save mtime, so no format versioning exists yet; adding one is a small prerequisite so a cache written by an older binary is rebuilt rather than misread.

## Status

Both plan documents are first drafts. Neither has been reviewed or started.
