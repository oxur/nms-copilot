//! Format adapters for No Man's Sky save exports.
//!
//! Handles non-standard save formats, starting with the goatfungus JSON export:
//! state-machine character walker that fixes `\xNN` → `\u00NN` and invalid
//! escapes inside strings, producing valid JSON for serde.

pub mod nomnom;
