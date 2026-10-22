# Slice25 shell completions — CC proposed handback

The candidate adds integration checks for generated Bash, Zsh, and Fish completion scopes, CLI help/error behavior, available-shell registration and candidates, and shell activation instructions. The final bounded correction accounts for Clap's automatic Fish `help` child and asserts empty stdout from both syntax-check children. Production code, dependencies, and existing tests remain unchanged.

The exact two-path commit is `03b6536636f00a74a52f2d783a9ef80a671d191e`, with the required consecutive coauthor trailers. Captured format, lint, focused CLI, whole CLI, binary build, private shell proof, full workspace tests, workspace build, workspace doctests, strict CLI docs, and final baseline diff checks all exited 0. Full test output contains 1,070 unit/integration passes and 31 doctest passes; the whole CLI test run contains 163 passes. Capture JSON and raw channels are retained at `/private/tmp/nms-slice25-cc-attempts.json`; previous failed attempts remain in that record.

The private controller generated all three scripts, checked Bash and Zsh syntax and registration, and exercised seven exact Bash candidate vectors. Fish was not executed in a Fish shell; its generated output and scope predicates were inspected by the integration test. This is the stated runtime boundary, not a claim of all-shell execution.

SC-01 through SC-05 are proposed complete by CC and remain open for CDC's independent verification. Original M7.2 and higher-level obligations remain with their existing owners. The ignored return directory contains the append-only execution history, this validation record, and this report; the raw capture file and private controller remain in `/private/tmp`.
