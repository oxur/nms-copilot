# Exploring galaxies in the REPL

The REPL keeps the atlas loaded while you search, plan routes, and play. Start it with the save you want to follow:

```bash
nms-copilot --save /path/to/save.hg
```

Commands below are entered inside the REPL. Base names are examples; substitute names from `list bases`.

## Galaxy view and reference position

The selected galaxy is the view you are exploring. Your physical location comes from the save. A distance query also needs a reference position, which follows the player until you choose a base.

```text
galaxy
galaxy Eissentam
galaxy "Hilbert Dimension"
galaxy 0
```

`galaxy` alone lists the selected and discovered galaxies. Selection accepts indices 0–255 and galaxy names, including quoted names containing spaces. Selecting a galaxy does not move the player or invent discoveries there.

A default `find` or `route` needs the player's galaxy to match the selected view. If they differ, choose a base in the galaxy you want to search:

```text
find --biome Lush --from "Camp-9" --nearest 5
set position "Camp-9"
find --biome Lush --nearest 5
```

`--from` affects that query. `set position` retains the base address as the session reference. Both use the base's own galaxy, even if you later select another view. To follow your physical location again:

```text
reset position
```

A physical change of galaxy in a subsequent save selects the destination galaxy. Movement within the same galaxy preserves an independently selected view. A base reference stays anchored until reset.

## Session defaults and status

```text
set biome Lush
set warp-range 2500
find --nearest 5
route --biome Swamp --round-trip
status
reset biome
reset warp-range
reset all
```

Explicit query options take precedence over session defaults. `status` shows your reference position and settings; `info` describes the loaded model and physical player location. `list bases` and `list systems` browse the whole model, and model counts span galaxies; those totals are not counts of only the selected view.

Routes use nearest-neighbor or 2-opt heuristics within one galaxy. Warp-range guidance is best-effort; review legs that exceed your ship's range.

## Recalling a result

```text
find --biome Frozen --nearest 3
last
```

`last` recalls the rendered response from the last successful `find`, `show`, `stats`, or `route` in this session. Successful empty results count too. Errors and other commands leave the stored response unchanged.

The response is a snapshot: later save updates do not refresh it, and it is not retained after the session ends. Run the query again for current data. This is separate from command history, which is stored in `~/.nms-copilot/history.txt` and can be recalled with the history keys.

## Opening the map

Enter `map` for the full-screen galaxy, region, and local views. Use the map's help overlay for controls; `q` returns to the REPL.

The map captures a snapshot when opened. Background save ingestion, cache writes, and MCP queries continue while it is open; reopen the map to see newer atlas data. The map projects voxel X/Z rather than displaying all three spatial dimensions.

`find`, `show`, `stats`, `route`, `convert`, and `list` are shared with the CLI. `saves`, `export`, `import`, and shell-script `completions` are currently CLI-only. See [data interchange](data-interchange.md) for exports/imports and [live atlas behavior](live-atlas.md) for changes received during play.
