# Contributing

Thanks for helping improve NMS Copilot. The project is a Rust 2024 workspace
with a minimum supported Rust version of 1.91. See [AGENTS.md](AGENTS.md) for
repository conventions and the crate layout in the [README](README.md).

## Build and validate

From the repository root, the normal local sequence is:

```bash
make format
make lint
make test
```

For a focused change, run the relevant crate or test target as well as the full
required checks for the change. For example:

```bash
cargo test -p nms-save
cargo test -p nms-cli --test export_integration
cargo test -p nms-copilot --doc
```

To build the workspace and preview its generated API documentation, run:

```bash
cargo build --workspace --all-features
cargo doc --workspace --lib --all-features --no-deps
cargo doc -p nms-cli --bin nms --document-private-items --all-features --no-deps --target-dir target/cli-docs
```

Library documentation starts at `target/doc/nms_copilot/index.html`; the
umbrella crate has its own page at `target/doc/nms/index.html`. The CLI's
internal documentation starts at `target/cli-docs/doc/nms/index.html`.
Keeping the CLI output in its own target directory prevents it from sharing
the umbrella crate's `nms` documentation namespace.

Name new tests `test_<function>_<scenario>_<expectation>` so their purpose is
clear in test output.

The workspace's `make coverage` target covers the `nms-copilot` library with
default features disabled; it is not a workspace-wide coverage measurement.
The project has a 95% coverage target, not a claim that this target has already
been measured or reached. Review the Makefile before using `make check` or
`make check-all`: those convenience targets can provision tools. Dependency
installation, publication, and release commands are separate from ordinary
validation and should only be used when a task explicitly calls for them.

## Fixtures and save data

Tests use committed synthetic fixtures such as
[`data/test/save56_multi_system.json`](data/test/save56_multi_system.json).
The fixture contains constructed systems, planets, and bases for repeatable
tests; it is not a player's save. Other committed legacy fixtures exercise
compatibility with historical input formats and should remain unchanged unless
a task specifically owns them.

Never add a personal save, account data, or an automatically discovered
machine-local save to a test or documentation change. Use an explicit
`--save PATH` argument when demonstrating or testing a command. NMS Copilot is
read-only: it reads the selected save and does not edit it.

## Documentation and examples

Keep command examples aligned with the actual CLI and REPL interfaces. Label
illustrative transcripts as examples and distinguish them from output captured
by running a command. Documentation that uses the synthetic fixture should pass
its path explicitly and should not depend on a developer's local save directory.

Rendered CLI examples and their reproduction notes live in
[`docs/captures/README.md`](docs/captures/README.md). Regenerating those images
uses the optional Python Pillow package and an explicitly supplied font file;
neither is an application dependency. Do not add generated captures unless
their raw command output and provenance are retained with them.

## Workspace map

- `crates/nms-core` contains shared types and address handling.
- `crates/nms-save` parses save data.
- `crates/nms-compat` contains compatibility helpers for JSON inputs.
- `crates/nms-graph` builds the galaxy model and spatial indexes.
- `crates/nms-query` formats and runs shared queries.
- `crates/nms-watch` observes save-file changes and publishes updates.
- `crates/nms-cache` stores the serialized galaxy model.
- `crates/nms-cli` implements the one-shot `nms` command.
- `crates/nms-copilot` implements the interactive REPL and MCP server.
- `crates/nms` is the workspace umbrella metadata/documentation crate; it does
  not re-export the other crates or provide an executable.

Follow the existing APIs and tests when changing a crate. Keep user data
private, retain meaningful negative controls, and report which checks were
actually run when proposing a change.
