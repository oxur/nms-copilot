# Slice01 map result

## Outcome

The rebuilt `nms-copilot` binary at source `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc` completed the bounded native PTY run with exit 0. The session entered the map and emitted the visible `Euclid—Galaxy`, `Region`, and `Local` views; the raw bytes also contain the `Galaxy Map` help panel and its `Zoom Levels` text. The controlled inputs were map entry, Right, Down, Enter twice, `?`, `x`, resize, Escape three times, ordinary REPL `help`, and `exit`. The saved PTY stream is the evidence for the exact output and control sequences.

The fixture was copied to `/private/tmp/nms-copilot-slice01-home-final/fixture/multi_system_save.json`. The isolated HOME used a config with cache and watcher disabled, and the command also passed `--no-cache`; the requested `127.0.0.1:0` listener was assigned port `49243`. The owned PTY began at 100 columns × 32 rows and was resized to 90 × 28 with `TIOCSWINSZ` plus `SIGWINCH`. Reedline's cursor-position requests were answered with the xterm-compatible `ESC[1;1R` reply.

The preserved screen text gives these visible cursor statuses: initial Galaxy `X=-1948 Z=-2248`, Region `X=-1906 Z=-2107` after Right/Down and zoom, Local `X=-1903 Z=-2102`, and Region again `X=-1906 Z=-2107` after zooming out. The stream then contains the leave-alternate-screen sequence, the next ordinary REPL help output, and `Goodbye!` after `exit`.

Termios snapshots were read from the PTY slave. Before child start, `c_iflag=0x2b02`, `c_oflag=0x3`, `c_cflag=0x4b00`, and `c_lflag=0x5cb`. While Reedline was active before and after the map, the flags were `0x801`, `0x2`, `0x4b00`, and `0x41`. At the read taken on the output chunk containing the map's alternate-screen leave sequence, they had returned to the pre-process values (`c_lflag=0x5cb`); after Reedline resumed they returned to its active values, and after normal process exit they again matched the pre-process values. This gives a direct observation of map cleanup before Reedline resumes its own mode. The PTY stream contains alternate-screen enter/leave (`ESC[?1049h` / `ESC[?1049l`) and cursor hide/show sequences.

One earlier startup attempt did not answer Reedline's `ESC[6n` cursor-position query and exited before the prompt. A second attempt answered it and reached the map, but preceded the binary identity refresh and is not used as source-qualified evidence. The retained refreshed-source attempt followed `cargo build --locked -p nms-copilot --bin nms-copilot` (exit 0) and used the same CPR handling. In its JSON metadata, `map_seen` remains `false` because the temporary capture code never assigned that bookkeeping variable; this does not describe the product result. The exact transcript bytes contain `Euclid—Galaxy`, and the capture continued past its map-entry wait, which would have failed otherwise.

No behavioral defect appeared in these exercised paths, so no source change was made. This is one native interaction witness, not exact-cell rendering proof or a general guarantee for other terminals, resize sizes, error paths, process aborts, or OS failures. Slice48's accepted injected guard/input evidence remains the proof for its already-covered failure cases; it was not rerun. The known `main.rs` modal dispatch holds a model read guard while `run_map` runs; this observation did not exercise live model writes or resolve that separate integration question.

## Reproduction and retained evidence

From `.worktrees/workspace-coverage-contract` at the source commit above:

```text
cargo build --locked -p nms-copilot --bin nms-copilot
./target/debug/nms-copilot --save /private/tmp/nms-copilot-slice01-home-final/fixture/multi_system_save.json --no-cache --http 127.0.0.1:0
```

The second command was run under Python's `pty.openpty`, with the isolated HOME/config above. The transient runner was not retained as a tool. `artifacts/pty-transcript.json` preserves the exact bytes read from all three attempts as base64, with attempt descriptions, the refreshed-source command, return code, CPR count, termios snapshots, and input setup; decoding `attempt_03_refreshed_source.bytes_base64` reproduces the successful PTY byte stream. Attempt01 is retained as the cursor-query failure; attempt02 is retained as an unqualified pre-build observation.

The transcript contains 14,103 bytes for the refreshed-source run and nine CPR queries; SHA-256 of those decoded PTY bytes is `a4cc472cebab147f42aa73c8f4ca9b9c39191694476804288b1fc2d1c13dbb47`. `run_map` source behavior is documented at `crates/nms-copilot/src/map/mod.rs:1-211`; input meanings are at `crates/nms-copilot/src/map/input.rs:1-114`; REPL dispatch is at `crates/nms-copilot/src/main.rs:267-273`. The input contract says arrows move, Enter/`+` zoom in, Esc/`-` zoom out and exit only at the outer level, and help consumes the next key to dismiss. The observed sequence used those ordinary actions.

Accepted Slice48 proof reused: `project01-mvp/arc06-fit-and-finish/slice48-map-terminal-guard/cdc-verification.md`, “Six-row source and oracle disposition” and “Bubble-up to Arc06” (lines 30-47). That record covers the injected guard cleanup/error and input contracts while explicitly limiting the older evidence's native-terminal claims.
