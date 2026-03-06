# Milestone 1.5 -- LZ4 Block Decompressor (nms-save)

Read `save.hg` files and decompress LZ4 block-compressed data.

## Overview

NMS save files (format 2002+) are sequences of LZ4 blocks. Each block has a 16-byte header followed by a compressed payload. The decompressed blocks are concatenated to produce a single UTF-8 JSON document.

Reference: `Platform_Read.cs:228-240` (block parsing), `LZ4.cs:32-45` (decompression wrapper).

---

## Block Header Structure

Each block begins with a 16-byte header, all fields little-endian:

```
Offset  Size  Field               Value / Constraint
0x00    4     magic               0xFEEDA1E5  (bytes: [0xE5, 0xA1, 0xED, 0xFE])
0x04    4     compressed_size     Size of the LZ4 payload following the header
0x08    4     decompressed_size   Size of the output after decompression
0x0C    4     padding             0x00000000 (always zero)
```

Constants:

| Name | Value | Notes |
|------|-------|-------|
| `BLOCK_MAGIC` | `0xFEEDA1E5` | Validated per block |
| `BLOCK_HEADER_SIZE` | `0x10` (16) | Fixed 16-byte header |
| `MAX_CHUNK_SIZE` | `0x80000` (524,288) | Maximum `decompressed_size` per block |

---

## Types

### SaveFormat

```rust
/// Detected format of a save file's raw bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    /// Standard NMS format (2002+): sequential LZ4 blocks with 16-byte headers.
    Lz4Compressed,
    /// Uncompressed JSON (first two bytes are 0x7B 0x22, i.e. `{"`).
    PlaintextJson,
}
```

### BlockHeader

```rust
/// Parsed header for a single LZ4 block.
#[derive(Debug, Clone, Copy)]
struct BlockHeader {
    /// Size of the compressed LZ4 payload (bytes).
    compressed_size: u32,
    /// Expected size after decompression (bytes). Must be <= MAX_CHUNK_SIZE.
    decompressed_size: u32,
}
```

### SaveError

Place in `src/error.rs`:

```rust
use std::fmt;

#[derive(Debug)]
pub enum SaveError {
    /// Block header magic mismatch.
    InvalidMagic {
        /// Byte offset in the file where the bad magic was found.
        offset: usize,
        /// The four bytes actually read (as little-endian u32).
        found: u32,
    },
    /// LZ4 decompression failed for a block.
    DecompressionFailed {
        /// Byte offset of the block header.
        offset: usize,
        /// Underlying error message from lz4_flex.
        message: String,
    },
    /// File ended before the expected number of bytes could be read.
    UnexpectedEof {
        /// Byte offset where reading started.
        offset: usize,
        /// Number of bytes we attempted to read.
        expected: usize,
    },
    /// A block's decompressed_size exceeds MAX_CHUNK_SIZE.
    ChunkTooLarge {
        offset: usize,
        declared: u32,
    },
    /// Wrapper for std::io::Error (file I/O).
    Io(std::io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic { offset, found } => {
                write!(f, "invalid block magic at offset {offset:#x}: expected 0xFEEDA1E5, found {found:#010x}")
            }
            Self::DecompressionFailed { offset, message } => {
                write!(f, "LZ4 decompression failed at offset {offset:#x}: {message}")
            }
            Self::UnexpectedEof { offset, expected } => {
                write!(f, "unexpected EOF at offset {offset:#x}: needed {expected} more bytes")
            }
            Self::ChunkTooLarge { offset, declared } => {
                write!(f, "block at offset {offset:#x} declares decompressed size {declared}, exceeds max {MAX_CHUNK_SIZE}")
            }
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
```

Note: Use `thiserror = "2"` if preferred, replacing the manual `Display`/`Error` impls with derive macros:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("invalid block magic at offset {offset:#x}: expected 0xFEEDA1E5, found {found:#010x}")]
    InvalidMagic { offset: usize, found: u32 },

    #[error("LZ4 decompression failed at offset {offset:#x}: {message}")]
    DecompressionFailed { offset: usize, message: String },

    #[error("unexpected EOF at offset {offset:#x}: needed {expected} more bytes")]
    UnexpectedEof { offset: usize, expected: usize },

    #[error("block at offset {offset:#x} declares decompressed size {declared}, exceeds max 0x80000")]
    ChunkTooLarge { offset: usize, declared: u32 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## Public Functions

### `detect_format`

```rust
/// Detect whether raw save file bytes are LZ4-compressed or plaintext JSON.
///
/// Checks the first 4 bytes:
/// - If first two bytes are 0x7B 0x22 (ASCII `{"`), returns `PlaintextJson`.
/// - If first four bytes are 0xFEEDA1E5 (little-endian), returns `Lz4Compressed`.
/// - Otherwise returns `Lz4Compressed` as default (will fail at decompression with InvalidMagic).
pub fn detect_format(data: &[u8]) -> SaveFormat {
    if data.len() >= 2 && data[0] == 0x7B && data[1] == 0x22 {
        SaveFormat::PlaintextJson
    } else {
        SaveFormat::Lz4Compressed
    }
}
```

### `decompress_save`

```rust
/// Decompress an entire NMS save file.
///
/// - If the data is plaintext JSON (starts with `{"`), returns a clone of the input.
/// - If the data is LZ4 compressed, parses all blocks and returns concatenated decompressed bytes.
///
/// The returned bytes are UTF-8 JSON.
pub fn decompress_save(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    match detect_format(data) {
        SaveFormat::PlaintextJson => Ok(data.to_vec()),
        SaveFormat::Lz4Compressed => decompress_blocks(data),
    }
}
```

### `decompress_save_file`

```rust
/// Convenience function: read a file from disk and decompress it.
pub fn decompress_save_file(path: &std::path::Path) -> Result<Vec<u8>, SaveError> {
    let data = std::fs::read(path)?;
    decompress_save(&data)
}
```

---

## Internal Functions

### `parse_block_header`

```rust
/// Parse a 16-byte block header at the given byte offset.
///
/// Returns a `BlockHeader` on success.
/// Errors:
/// - `UnexpectedEof` if fewer than 16 bytes remain at `offset`.
/// - `InvalidMagic` if the first 4 bytes are not 0xFEEDA1E5.
/// - `ChunkTooLarge` if `decompressed_size > MAX_CHUNK_SIZE`.
fn parse_block_header(data: &[u8], offset: usize) -> Result<BlockHeader, SaveError> {
    if offset + BLOCK_HEADER_SIZE > data.len() {
        return Err(SaveError::UnexpectedEof {
            offset,
            expected: BLOCK_HEADER_SIZE,
        });
    }

    let magic = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    if magic != BLOCK_MAGIC {
        return Err(SaveError::InvalidMagic { offset, found: magic });
    }

    let compressed_size = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
    let decompressed_size = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
    // padding at offset+12..offset+16 is ignored

    if decompressed_size > MAX_CHUNK_SIZE as u32 {
        return Err(SaveError::ChunkTooLarge {
            offset,
            declared: decompressed_size,
        });
    }

    Ok(BlockHeader {
        compressed_size,
        decompressed_size,
    })
}
```

### `decompress_blocks`

```rust
/// Read and decompress all LZ4 blocks from raw save file bytes.
///
/// Iterates from the beginning of `data`, reading 16-byte headers followed by
/// `compressed_size` bytes of LZ4 payload. Each payload is decompressed into
/// `decompressed_size` bytes. All chunks are concatenated into the output.
fn decompress_blocks(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    let mut output = Vec::new();
    let mut offset: usize = 0;

    while offset < data.len() {
        let header = parse_block_header(data, offset)?;
        offset += BLOCK_HEADER_SIZE;

        let payload_end = offset + header.compressed_size as usize;
        if payload_end > data.len() {
            return Err(SaveError::UnexpectedEof {
                offset,
                expected: header.compressed_size as usize,
            });
        }

        let compressed = &data[offset..payload_end];

        // lz4_flex::block::decompress requires knowing the uncompressed size.
        let decompressed = lz4_flex::block::decompress(compressed, header.decompressed_size as usize)
            .map_err(|e| SaveError::DecompressionFailed {
                offset: offset - BLOCK_HEADER_SIZE,
                message: e.to_string(),
            })?;

        output.extend_from_slice(&decompressed);
        offset = payload_end;
    }

    Ok(output)
}
```

---

## Algorithm Summary

1. Read first bytes of the file.
2. If bytes `[0] == 0x7B` and `[1] == 0x22` (ASCII `{"`), treat as plaintext JSON -- return data as-is.
3. Otherwise, enter block loop starting at offset 0.
4. At each iteration:
   a. Parse 16-byte header. Validate magic == `0xFEEDA1E5`.
   b. Read `compressed_size` bytes of LZ4 payload.
   c. Decompress into a buffer of `decompressed_size` bytes using `lz4_flex::block::decompress`.
   d. Append decompressed bytes to output.
   e. Advance offset by `BLOCK_HEADER_SIZE + compressed_size`.
5. When offset reaches end of data, return concatenated output.
6. The output is a complete UTF-8 JSON document.

---

## Dependencies

Add to `crates/nms-save/Cargo.toml`:

```toml
[dependencies]
nms-core = { workspace = true }
lz4_flex = "0.11"
thiserror = "2"

[dev-dependencies]
# No additional dev dependencies needed for this milestone.
```

Also add `lz4_flex` and `thiserror` to the workspace `[workspace.dependencies]` section in the root `Cargo.toml` if the project uses workspace-level dependency management.

Note: Use the `lz4_flex` **block** API (`lz4_flex::block::decompress` and `lz4_flex::block::compress`), NOT the frame API. NMS uses raw LZ4 block compression, not LZ4 frames.

---

## File Organization

```
crates/nms-save/
  Cargo.toml
  src/
    lib.rs          -- re-exports: pub mod decompress; pub mod error;
    error.rs        -- SaveError enum
    decompress.rs   -- constants, SaveFormat, BlockHeader, detect_format,
                       decompress_save, decompress_save_file, parse_block_header,
                       decompress_blocks
```

### `src/lib.rs` additions

```rust
pub mod decompress;
pub mod error;

pub use decompress::{decompress_save, decompress_save_file, detect_format, SaveFormat};
pub use error::SaveError;
```

### Constants in `src/decompress.rs`

```rust
use crate::error::SaveError;

/// LZ4 block magic number (little-endian).
const BLOCK_MAGIC: u32 = 0xFEEDA1E5;

/// Size of a single block header in bytes.
const BLOCK_HEADER_SIZE: usize = 0x10;

/// Maximum decompressed size per block.
const MAX_CHUNK_SIZE: usize = 0x80000;
```

---

## Tests

Place tests in `crates/nms-save/src/decompress.rs` as a `#[cfg(test)] mod tests` block, or in `crates/nms-save/tests/decompress.rs` as an integration test.

### Test helper: build a synthetic NMS save file

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Build a valid NMS-format LZ4 file from raw JSON bytes.
    /// Splits the input into MAX_CHUNK_SIZE chunks, compresses each,
    /// and wraps each in a 16-byte NMS block header.
    fn create_test_save(json: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        for chunk in json.chunks(MAX_CHUNK_SIZE) {
            let compressed = lz4_flex::block::compress(chunk);
            let compressed_size = compressed.len() as u32;
            let decompressed_size = chunk.len() as u32;

            // Write header
            output.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
            output.extend_from_slice(&compressed_size.to_le_bytes());
            output.extend_from_slice(&decompressed_size.to_le_bytes());
            output.extend_from_slice(&0u32.to_le_bytes()); // padding

            // Write payload
            output.extend_from_slice(&compressed);
        }
        output
    }
```

### Required test cases

```rust
    #[test]
    fn test_detect_plaintext_json() {
        let data = br#"{"Version": 6726}"#;
        assert_eq!(detect_format(data), SaveFormat::PlaintextJson);
    }

    #[test]
    fn test_detect_lz4_compressed() {
        let mut data = vec![0xE5, 0xA1, 0xED, 0xFE]; // magic LE bytes
        data.extend_from_slice(&[0; 12]); // rest of header (will fail at decompression, but detection succeeds)
        assert_eq!(detect_format(&data), SaveFormat::Lz4Compressed);
    }

    #[test]
    fn test_detect_empty_input() {
        // Empty data defaults to Lz4Compressed (will fail gracefully during decompress).
        assert_eq!(detect_format(&[]), SaveFormat::Lz4Compressed);
    }

    #[test]
    fn test_parse_block_header_valid() {
        let mut header = Vec::new();
        header.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        header.extend_from_slice(&100u32.to_le_bytes());   // compressed_size
        header.extend_from_slice(&200u32.to_le_bytes());   // decompressed_size
        header.extend_from_slice(&0u32.to_le_bytes());     // padding

        let bh = parse_block_header(&header, 0).unwrap();
        assert_eq!(bh.compressed_size, 100);
        assert_eq!(bh.decompressed_size, 200);
    }

    #[test]
    fn test_parse_block_header_invalid_magic() {
        let mut header = Vec::new();
        header.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());
        header.extend_from_slice(&[0; 12]);

        let err = parse_block_header(&header, 0).unwrap_err();
        match err {
            SaveError::InvalidMagic { offset, found } => {
                assert_eq!(offset, 0);
                assert_eq!(found, 0xDEADBEEF);
            }
            _ => panic!("expected InvalidMagic, got {err:?}"),
        }
    }

    #[test]
    fn test_parse_block_header_truncated() {
        let data = [0u8; 8]; // only 8 bytes, need 16
        let err = parse_block_header(&data, 0).unwrap_err();
        match err {
            SaveError::UnexpectedEof { offset, expected } => {
                assert_eq!(offset, 0);
                assert_eq!(expected, BLOCK_HEADER_SIZE);
            }
            _ => panic!("expected UnexpectedEof, got {err:?}"),
        }
    }

    #[test]
    fn test_roundtrip_single_block() {
        let json = br#"{"Version": 6726, "Platform": "PC"}"#;
        let save = create_test_save(json);
        let result = decompress_save(&save).unwrap();
        assert_eq!(&result, json);
    }

    #[test]
    fn test_roundtrip_multiple_blocks() {
        // Create data larger than MAX_CHUNK_SIZE to force multiple blocks.
        let big_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(MAX_CHUNK_SIZE + 1000));
        let save = create_test_save(big_json.as_bytes());
        let result = decompress_save(&save).unwrap();
        assert_eq!(result, big_json.as_bytes());
    }

    #[test]
    fn test_plaintext_passthrough() {
        let json = br#"{"Version": 6726}"#;
        let result = decompress_save(json).unwrap();
        assert_eq!(&result, json);
    }

    #[test]
    fn test_truncated_payload() {
        // Build a header claiming 1000 compressed bytes, but only provide 10.
        let mut data = Vec::new();
        data.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        data.extend_from_slice(&1000u32.to_le_bytes());  // compressed_size = 1000
        data.extend_from_slice(&2000u32.to_le_bytes());  // decompressed_size
        data.extend_from_slice(&0u32.to_le_bytes());     // padding
        data.extend_from_slice(&[0u8; 10]);              // only 10 bytes of payload

        let err = decompress_save(&data).unwrap_err();
        match err {
            SaveError::UnexpectedEof { .. } => {}
            _ => panic!("expected UnexpectedEof, got {err:?}"),
        }
    }

    #[test]
    fn test_invalid_magic_at_second_block() {
        let json = b"hello";
        let compressed = lz4_flex::block::compress(json);
        let mut data = Vec::new();

        // Valid first block
        data.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        data.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        data.extend_from_slice(&(json.len() as u32).to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&compressed);

        // Invalid second block header
        data.extend_from_slice(&0xBAD_MAGIC.to_le_bytes());
        data.extend_from_slice(&[0u8; 12]);

        let err = decompress_save(&data).unwrap_err();
        match err {
            SaveError::InvalidMagic { offset, .. } => {
                assert_eq!(offset, BLOCK_HEADER_SIZE + compressed.len());
            }
            _ => panic!("expected InvalidMagic, got {err:?}"),
        }
    }
}
```

### Note on test fixtures

The file `workbench/save.hg.json` (if present) is already-decompressed JSON, not a raw `.hg` file. It cannot be used directly to test decompression. Instead, use the `create_test_save` helper above to construct synthetic LZ4 files with known content.

If a real `save.hg` file becomes available for integration testing, add it as a binary fixture in `crates/nms-save/tests/fixtures/` and test with `decompress_save_file`.

---

## Acceptance Criteria

1. `decompress_save` correctly decompresses a synthetic multi-block LZ4 file back to original JSON.
2. `detect_format` correctly distinguishes plaintext from LZ4 by inspecting the first bytes.
3. All error variants are exercised: `InvalidMagic`, `UnexpectedEof`, `DecompressionFailed`, `ChunkTooLarge`.
4. `cargo test -p nms-save` passes all tests.
5. `cargo clippy -p nms-save` reports no warnings.
6. No `unsafe` code.
