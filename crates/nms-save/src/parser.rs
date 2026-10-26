//! Object-safe parsing for supported on-disk No Man's Sky save bytes.

use crate::error::SaveError;
use crate::validated::{ValidatedSave, parse_validated_save_bytes};

/// Parses supported raw save bytes into the checked typed save subset.
///
/// The input may be plaintext JSON beginning with `{` or standard NMS LZ4
/// blocks. The existing F2P first-key heuristic and bundled mapping are applied
/// to decompressed bytes. Unknown source fields remain outside [`ValidatedSave`].
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use nms_save::{NmsSaveParser, SaveParser};
/// let raw = std::fs::read("save.hg")?;
/// let parser: &dyn SaveParser = &NmsSaveParser;
/// let save = parser.parse(&raw)?;
/// println!("version {}", save.root().version);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns the existing [`SaveError`] variants for malformed LZ4, invalid JSON
/// or typed metadata, and invalid modeled source addresses.
///
/// # Panics
///
/// Parsing inherits the documented 32-bit compressed-offset and invalid
/// bundled-mapping panic limits from the existing checked file pipeline.
pub trait SaveParser {
    /// Parses one supported on-disk save representation from memory.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] when decompression, JSON decoding, typed
    /// deserialization, or modeled address validation fails.
    ///
    /// # Panics
    ///
    /// May panic on the conditional malformed compressed-size offset on
    /// 32-bit targets or if bundled mapping data is invalid.
    fn parse(&self, input: &[u8]) -> Result<ValidatedSave, SaveError>;
}

/// Stateless parser for the supported NMS on-disk save representations.
///
/// This implementation delegates to the same checked byte pipeline used by
/// [`crate::parse_validated_save_file`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NmsSaveParser;

impl SaveParser for NmsSaveParser {
    fn parse(&self, input: &[u8]) -> Result<ValidatedSave, SaveError> {
        parse_validated_save_bytes(input)
    }
}
