//! LZ4 block decompression for NMS save files.

use crate::error::SaveError;

/// LZ4 block magic number (little-endian).
const BLOCK_MAGIC: u32 = 0xFEEDA1E5;

/// Size of a single block header in bytes.
const BLOCK_HEADER_SIZE: usize = 0x10;

/// Maximum decompressed size per block.
const MAX_CHUNK_SIZE: usize = 0x80000;

/// Detected format of a save file's raw bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    /// Standard NMS format (2002+): sequential LZ4 blocks with 16-byte headers.
    Lz4Compressed,
    /// Uncompressed JSON (first byte is 0x7B, i.e. `{`).
    PlaintextJson,
}

/// Parsed header for a single LZ4 block.
#[derive(Debug, Clone, Copy)]
struct BlockHeader {
    compressed_size: u32,
    decompressed_size: u32,
}

/// Classify raw bytes using only their first byte.
///
/// A first byte of `{` selects [`SaveFormat::PlaintextJson`]; every other
/// value, including empty input and JSON with leading whitespace, selects
/// [`SaveFormat::Lz4Compressed`]. The function does not validate JSON or
/// inspect an LZ4 header.
pub fn detect_format(data: &[u8]) -> SaveFormat {
    if !data.is_empty() && data[0] == 0x7B {
        SaveFormat::PlaintextJson
    } else {
        SaveFormat::Lz4Compressed
    }
}

/// Decompress NMS LZ4 blocks or pass through first-byte-classified JSON bytes.
///
/// When the first byte is `{`, this returns an exact copy of the input without
/// checking UTF-8 or JSON syntax. Otherwise it parses consecutive 16-byte LZ4
/// headers with little-endian magic `0xFEEDA1E5`, validates each block, and
/// concatenates decompressed payloads; the fourth reserved header word is
/// ignored. An empty slice is classified as compressed input and produces an
/// empty output.
///
/// # Examples
///
/// ```
/// use nms_save::{SaveError, SaveFormat, decompress_save, detect_format};
/// assert_eq!(decompress_save(b"{}")?, b"{}");
/// assert_eq!(decompress_save(&[])?, b"");
/// assert_eq!(detect_format(b" {}"), SaveFormat::Lz4Compressed);
/// assert!(matches!(decompress_save(b" {}"), Err(SaveError::UnexpectedEof { offset: 0, expected: 16 })));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`SaveError::UnexpectedEof`] for an incomplete header or payload,
/// [`SaveError::InvalidMagic`] for a wrong block magic,
/// [`SaveError::ChunkTooLarge`] for a declared block above 0x80000 bytes, or
/// [`SaveError::DecompressionFailed`] when the LZ4 decoder rejects a payload.
///
/// # Panics
///
/// On 32-bit targets, adding a declared compressed size to the payload offset
/// uses unchecked `usize` arithmetic. A 16-byte header declaring
/// `u32::MAX` compressed bytes can overflow that end offset and panic while
/// slicing the input.
pub fn decompress_save(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    match detect_format(data) {
        SaveFormat::PlaintextJson => Ok(data.to_vec()),
        SaveFormat::Lz4Compressed => decompress_blocks(data),
    }
}

/// Read a file and apply [`decompress_save`] to its bytes.
///
/// This reads the file but does not parse the returned bytes as UTF-8 or JSON.
///
/// # Examples
///
/// ```no_run
/// # fn read(path: &std::path::Path) -> Result<(), nms_save::SaveError> {
/// let bytes = nms_save::decompress_save_file(path)?;
/// println!("{} decompressed bytes", bytes.len());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`SaveError::Io`] when reading fails, or the decompression error
/// returned by [`decompress_save`].
///
/// # Panics
///
/// Propagates the conditional 32-bit malformed-offset panic documented on
/// [`decompress_save`].
pub fn decompress_save_file(path: &std::path::Path) -> Result<Vec<u8>, SaveError> {
    let data = std::fs::read(path)?;
    decompress_save(&data)
}

/// Parse a 16-byte block header at the given byte offset.
fn parse_block_header(data: &[u8], offset: usize) -> Result<BlockHeader, SaveError> {
    if offset + BLOCK_HEADER_SIZE > data.len() {
        return Err(SaveError::UnexpectedEof {
            offset,
            expected: BLOCK_HEADER_SIZE,
        });
    }

    let magic = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    if magic != BLOCK_MAGIC {
        return Err(SaveError::InvalidMagic {
            offset,
            found: magic,
        });
    }

    let compressed_size = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
    let decompressed_size = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());

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

/// Read and decompress all LZ4 blocks from raw save file bytes.
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

        let decompressed =
            lz4_flex::block::decompress(compressed, header.decompressed_size as usize).map_err(
                |e| SaveError::DecompressionFailed {
                    offset: offset - BLOCK_HEADER_SIZE,
                    message: e.to_string(),
                },
            )?;

        output.extend_from_slice(&decompressed);
        offset = payload_end;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a valid NMS-format LZ4 file from raw JSON bytes.
    fn create_test_save(json: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        for chunk in json.chunks(MAX_CHUNK_SIZE) {
            let compressed = lz4_flex::block::compress(chunk);
            let compressed_size = compressed.len() as u32;
            let decompressed_size = chunk.len() as u32;

            output.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
            output.extend_from_slice(&compressed_size.to_le_bytes());
            output.extend_from_slice(&decompressed_size.to_le_bytes());
            output.extend_from_slice(&0u32.to_le_bytes());
            output.extend_from_slice(&compressed);
        }
        output
    }

    #[test]
    fn detect_plaintext_json() {
        let data = br#"{"Version": 6726}"#;
        assert_eq!(detect_format(data), SaveFormat::PlaintextJson);
    }

    #[test]
    fn detect_lz4_compressed() {
        let mut data = vec![0xE5, 0xA1, 0xED, 0xFE];
        data.extend_from_slice(&[0; 12]);
        assert_eq!(detect_format(&data), SaveFormat::Lz4Compressed);
    }

    #[test]
    fn detect_empty_input() {
        assert_eq!(detect_format(&[]), SaveFormat::Lz4Compressed);
    }

    #[test]
    fn parse_block_header_valid() {
        let mut header = Vec::new();
        header.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        header.extend_from_slice(&100u32.to_le_bytes());
        header.extend_from_slice(&200u32.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());

        let bh = parse_block_header(&header, 0).unwrap();
        assert_eq!(bh.compressed_size, 100);
        assert_eq!(bh.decompressed_size, 200);
    }

    #[test]
    fn parse_block_header_invalid_magic() {
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
    fn parse_block_header_truncated() {
        let data = [0u8; 8];
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
    fn parse_block_header_chunk_too_large() {
        let mut header = Vec::new();
        header.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        header.extend_from_slice(&100u32.to_le_bytes());
        header.extend_from_slice(&(MAX_CHUNK_SIZE as u32 + 1).to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());

        let err = parse_block_header(&header, 0).unwrap_err();
        match err {
            SaveError::ChunkTooLarge { offset, declared } => {
                assert_eq!(offset, 0);
                assert_eq!(declared, MAX_CHUNK_SIZE as u32 + 1);
            }
            _ => panic!("expected ChunkTooLarge, got {err:?}"),
        }
    }

    #[test]
    fn roundtrip_single_block() {
        let json = br#"{"Version": 6726, "Platform": "PC"}"#;
        let save = create_test_save(json);
        let result = decompress_save(&save).unwrap();
        assert_eq!(&result, json);
    }

    #[test]
    fn roundtrip_multiple_blocks() {
        let big_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(MAX_CHUNK_SIZE + 1000));
        let save = create_test_save(big_json.as_bytes());
        let result = decompress_save(&save).unwrap();
        assert_eq!(result, big_json.as_bytes());
    }

    #[test]
    fn plaintext_passthrough() {
        let json = br#"{"Version": 6726}"#;
        let result = decompress_save(json).unwrap();
        assert_eq!(&result, json);
    }

    #[test]
    fn truncated_payload() {
        let mut data = Vec::new();
        data.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        data.extend_from_slice(&1000u32.to_le_bytes());
        data.extend_from_slice(&2000u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 10]);

        let err = decompress_save(&data).unwrap_err();
        assert!(matches!(err, SaveError::UnexpectedEof { .. }));
    }

    #[test]
    fn invalid_magic_at_second_block() {
        let json = b"hello";
        let compressed = lz4_flex::block::compress(json);
        let mut data = Vec::new();

        // Valid first block
        data.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
        data.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        data.extend_from_slice(&(json.len() as u32).to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&compressed);

        // Invalid second block
        let bad_magic: u32 = 0xBAD;
        data.extend_from_slice(&bad_magic.to_le_bytes());
        data.extend_from_slice(&[0u8; 12]);

        let err = decompress_save(&data).unwrap_err();
        match err {
            SaveError::InvalidMagic { offset, .. } => {
                assert_eq!(offset, BLOCK_HEADER_SIZE + compressed.len());
            }
            _ => panic!("expected InvalidMagic, got {err:?}"),
        }
    }

    #[test]
    fn decompress_save_file_not_found() {
        let err = decompress_save_file(std::path::Path::new("/nonexistent/save.hg")).unwrap_err();
        assert!(matches!(err, SaveError::Io(_)));
    }
}
