# Rendered CLI captures

This directory is for plain-text output from real `nms` commands and PNGs
rendered from those exact text files. The statistics command uses the committed
synthetic fixture; the help command reports the CLI's own usage text. The images
are not desktop screenshots, game screenshots, or evidence of a particular
terminal theme. The `.txt` and `.png` files are added only after the
corresponding captured commands and renderings
have been reviewed; this page does not stand in for those outputs.

| Capture | Raw text | Rendered image |
|---|---|---|
| CLI help | [`nms-help.txt`](nms-help.txt) | [`nms-help.png`](nms-help.png) |
| Synthetic-save statistics | [`nms-stats.txt`](nms-stats.txt) | [`nms-stats.png`](nms-stats.png) |

The help capture is produced from the built CLI with `--help`. The statistics
capture uses the committed `data/test/save56_multi_system.json` fixture and an
explicit `--save` path, so it does not inspect a machine's game-save folders.
The corresponding `.txt` files preserve complete stdout, including its final
newline. The renderer adds a separate command caption and does not alter the
captured text.

To reproduce the captures from the repository root, build the CLI and run the
commands while redirecting stdout. Keep stderr separate and check that each
command exits successfully before treating its output as a capture:

```bash
cargo build -p nms-cli --bin nms
target/debug/nms --help > docs/captures/nms-help.txt
target/debug/nms stats --save data/test/save56_multi_system.json --biomes > docs/captures/nms-stats.txt
```

Render each text file with [`render-cli-capture.py`](../render-cli-capture.py)
and the same explicit font used for the documented image. Pillow is optional
tooling for this documentation task, not a runtime or Cargo dependency. Supply
an installed font path that supports the captured
characters; for example, on macOS:

```bash
python3 docs/render-cli-capture.py docs/captures/nms-help.txt docs/captures/nms-help.png \
  --command 'nms --help' --font /System/Library/Fonts/Menlo.ttc
python3 docs/render-cli-capture.py docs/captures/nms-stats.txt docs/captures/nms-stats.png \
  --command 'nms stats --save data/test/save56_multi_system.json --biomes' \
  --font /System/Library/Fonts/Menlo.ttc
```

The renderer reads UTF-8, rejects ANSI escape sequences, retains every line,
and sizes the image to the complete caption and output. It does not launch the
CLI or synthesize missing rows. Regeneration provenance records the actual
source revision, working-tree identity, executable and fixture hashes, raw
capture identities, text and image hashes, font path, and renderer hash.
