# Rendered CLI captures

This directory contains complete raw output from real `nms` commands and
monochrome PNGs rendered from those exact text files. The statistics command
uses the committed synthetic fixture; the help command reports the CLI's own
usage text. The images are not desktop screenshots, game screenshots, or
evidence of a particular terminal theme. Review the actual `.txt` and `.png`
files together with their capture records; this page does not stand in for
those outputs.

| Capture | Raw text | Rendered image |
|---|---|---|
| CLI help | [`nms-help.txt`](nms-help.txt) | [`nms-help.png`](nms-help.png) |
| Synthetic-save statistics | [`nms-stats.txt`](nms-stats.txt) | [`nms-stats.png`](nms-stats.png) |

The help capture is produced from the built CLI with `--help`. The statistics
capture uses the committed `data/test/save56_multi_system.json` fixture and an
explicit `--save` path, so it does not inspect a machine's game-save folders.
The corresponding `.txt` files preserve complete stdout byte-for-byte,
including its final newline and any terminal styling codes. The renderer adds
a separate command caption; only for image presentation it omits the documented
SGR styles `ESC[30m`, `ESC[37m`, `ESC[39m`, `ESC[40m`, and `ESC[49m`. Raw text
remains unchanged. These are monochrome rendered captures, not terminal or game
screenshots and not evidence of a particular terminal theme.

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

The renderer reads UTF-8, removes only those five complete styling tokens for
the image, and fails closed on every other escape or control character except preserved LF. It
retains every other character, whitespace, and line feed, and sizes the image
to the complete caption and output. It does not launch the CLI or synthesize
missing rows. Regeneration provenance records the actual
source revision, working-tree identity, executable and fixture hashes, raw
capture identities, text and image hashes, font path, and renderer hash.

## Recorded generation

The CLI was built offline from source revision
`be4145b631405ff19c3e403457ecaa581afe805f`; its captured build ID is
`81e93b4717d84a77a452156194b8fecb` (exit 0). The executable SHA-256 was
`985e5688c6cbde138fac3899baed5688e1ba23cc023356d41a22211d57bd4574`; the
synthetic fixture SHA-256 was
`66fdbda21fb47cb5459d210613a08e14abfb1505d00ee849b20241cae75019a1`. The
build and both CLI captures observed working identity
`932bd10dc488378e3970a51773cd68c75eb3e6682191c21d53189733ff414b9b`.

| Output | Actual command and capture | Raw file | Rendered PNG |
|---|---|---|---|
| Help | `nms --help`; capture `eeae09e86c65466eadf4774ad6e4e087`; exit 0, stdout 887 bytes, stderr 0 | `nms-help.txt`, SHA-256 `2bba2f1030814740c5063af89546e4b214b0cbfd77332829cb7215da737f00bb` | render `a73c3519c8324b5b9f6ee645d6a4696f`; `nms-help.png`, 65,711 bytes, SHA-256 `83a6528fc9e621c9273452753547090d7ba85051d626839d8185bc8cbbfce2f2` |
| Fixture statistics | `nms stats --save data/test/save56_multi_system.json --biomes`; capture `e974a83b290146298e30fce60ada9b08`; exit 0, stdout 2,029 bytes, stderr 0 | `nms-stats.txt`, SHA-256 `d6ae97e45f6007dffd6a467bc074b86717d2502cb3fb7e9b9f5beca01519db53` | render `aac49ae5bca14f16a8c939d9c12e851c`; `nms-stats.png`, 38,823 bytes, SHA-256 `19acabe53dd1adaf8aba9ee4d1fe0709508393a774a0a244ad7d52760a00ebba` |

Both raw files are the exact captured stdout bytes, including final newlines.
Help has no ESC or styling controls and retains its LF line feeds. Stats contains 288 SGR sequences: 76
`ESC[40m`, 76 `ESC[49m`, 68 `ESC[39m`, 60 `ESC[37m`, and 8 `ESC[30m`.
Removing exactly those sequences for image presentation yields 589 UTF-8 bytes,
25 LF characters, SHA-256
`aef4bcdb07328f749142888d8f0b3774812d33d6d4d077262c1d815dd76bc0ad`. The renderer SHA-256 is
`2b7a92d12c5844a4d4c1c8850b0761052a403caa2b7ceb815c23a98a6b3b6baf`. Both renders used the explicit font `/System/Library/Fonts/Menlo.ttc`. CDC measured that font after generation during final provenance review: 2,156,036 bytes, SHA-256
`dc256e0b39c2a6fec947129d421fef41b8b429f58f9b6e5d1b148c87f775c1f6`; this was not a pre-render measurement. The captured `py_compile` check is `2547bde3d3ed4dbebf9e7e2f8a433eff` (exit 0;
stdout and stderr empty). The help render ran at working identity
`93a5810e0348e6c575adc294dfa66cfee0a195a4794f2b999c1112daa9ae30b7` before
creating its PNG; the stats render ran at
`dafd9d78d705d0178708e87a0b75ae5b68174f2c862ee6c6308a6411e62421df` before
creating its PNG. Complete raw gate records are retained in the local capture
record; the render commands and explicit captions are listed above.
