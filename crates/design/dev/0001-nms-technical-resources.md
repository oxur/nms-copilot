# NMS Technical Resources

## Save File Tools

- **goatfungus NMSSaveEditor** — Java .jar, exports save to JSON
  <https://github.com/goatfungus/NMSSaveEditor>
  - No source available, .class files only
  - Produces JSON with non-standard `\x` escapes

- **NomNom** — Most complete save editor (.NET/C#)
  <https://github.com/zencq/NomNom>
  - Uses libNOM.io, libNOM.map, libNOM.collect libraries
  - <https://github.com/zencq/libNOM.io> (read/write saves, all platforms)
  - <https://github.com/zencq/libNOM.map> (obfuscate/deobfuscate save JSON)

- **nmssavetool** — Lighter save tool
  <https://github.com/matthew-humphrey/nmssavetool>

## Game Data / Modding

- **MBINCompiler / libMBIN** — Decompile game .MBIN data files
  <https://github.com/monkeyman192/MBINCompiler>
  - Contains all game struct/enum definitions as C# classes
  - **GcBiomeType.cs** — biome enum (verified working)
    `libMBIN/Source/NMS/GameComponents/GcBiomeType.cs`
  - **GcBiomeSubType.cs** — biome subtype enum
    `libMBIN/Source/NMS/GameComponents/GcBiomeSubType.cs`
  - Development branch has latest game version structs

## Portal / Coordinate Tools

- **NMS Portals Decoder** — Online coordinate converter
  <https://nmsportals.github.io/>

- **NMS Glyphs Decoder** — Another converter
  <https://glyphs.had.sh/>
  - Docs: <https://glyphs.had.sh/docs/glyphs>

- **Portal Repository** — Catalog of portal addresses
  <https://portalrepository.com/>

## Wikis

- **NMS Wiki (Miraheze)** — Active wiki
  <https://nomanssky.miraheze.org/wiki/>
  - Portal address: <https://nomanssky.miraheze.org/wiki/Portal_address>
  - Biome: <https://nms.miraheze.org/wiki/Biome>

- **NMS Wiki (Fandom)** — Older wiki, still has good data
  <https://nomanssky.fandom.com/wiki/>
  - Galactic Coordinates: <https://nomanssky.fandom.com/wiki/Galactic_Coordinates>
  - Biome: <https://nomanssky.fandom.com/wiki/Biome>

## Community Resources

- **NMS Resources** — Planet/biome reference
  <https://www.nomansskyresources.com/planets-and-moons>

- **vectorcmdr NMSSaveEditor fork**
  <https://github.com/vectorcmdr/NMSSaveEditor>

## Nexus Mods

- **Planet Type Converters** (Exosolar & Babs)
  <https://www.nexusmods.com/nomanssky/mods/1270>
