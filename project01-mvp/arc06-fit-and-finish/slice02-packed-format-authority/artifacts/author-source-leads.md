# Author source leads — 2026-10-03

Research leads, not accepted encoding decisions. Source main remains a800950; slice01's unresolved scope is unchanged.

Pinned [NMSCoordinates source](https://github.com/Kevin0M16/NMSCoordinates/tree/b8f765b03082cf5e130a538896db6d67481f9bae) at commit `b8f765b03082cf5e130a538896db6d67481f9bae`. Read complete `Coordinates.cs` PersistentBases (101–109) and HexToAll (242–343), GetGalacticCoordHex (198–241), and `CoordCalculations.cs` VoxelToHex (194–240), GalacticToPortal (241–277), PortalToHex (278–297). The base caller explicitly reads PersistentPlayerBases.GalacticAddress and invokes HexToAll. That decoder separates a galaxy component and formats location/portal outputs. This is a field-applicable lead; discovery DD.UA applicability remains unproven. Check short values, prefix handling and decimal planet formatting before using it as an oracle.

- [NMSCoordinates/Coords/Coordinates.cs](https://github.com/Kevin0M16/NMSCoordinates/blob/b8f765b03082cf5e130a538896db6d67481f9bae/NMSCoordinates/Coords/Coordinates.cs): blob `9336a2b1d660f9d9d716e939525941bf37e8dd3c`, SHA256 `7bb2e8a0ca407947770e01e37f306f92b4321630b1b762aa129ee5e65d48fdf3`.
- [NMSCoordinates/Coords/CoordCalculations.cs](https://github.com/Kevin0M16/NMSCoordinates/blob/b8f765b03082cf5e130a538896db6d67481f9bae/NMSCoordinates/Coords/CoordCalculations.cs): blob `1f3ff3d17a26c836525bf3bdcb7b50959e793294`, SHA256 `388f61f63097247eb4ce6e803cdd413456e3e5c51558b363a420d1d627bf94b4`.

Existing libNom.io at 652aabb0147888776a088909cdb13bccbd3d9588 has a Steam test archive: 119 members, 51 binary-save members, no JSON. This inventory is not decoded semantic evidence. No private decoded candidate was located in the bounded `/private/tmp/*nms*` query; previously examined project02 snapshots are not supplied here.

Acquisition: browser directory requests for Coords/LocationData returned cache misses; public GitHub API tree/commit/blob reads succeeded. Reference bytes remain temporary. Exclude Glyphsmith (binary/README-only inventory) and the inspected nmslib convert.go (portal/signal conversions) as packed-save authority. No downloaded application was executed.

Author probe-route inspection: existing nms-save publicly exports decompress_save, KeyMapping::bundled and deobfuscate_json. Preserve serde_json::Value rather than a typed SaveRoot round trip. No archive payload was decoded during assignment preparation. Initial lookups for parse.rs/deobfuscate.rs and cc-iteration01.md failed; actual entrypoints are lib.rs/mapping.rs and the preserved cc-prompt-iteration01.md.
