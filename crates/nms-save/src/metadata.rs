//! Metadata file (`mf_save*.hg`) decryption and parsing.

use crate::error::SaveError;
use crate::xxtea::{derive_key, xxtea_decrypt};

/// Meta magic sentinel -- first u32 after successful XXTEA decryption.
const META_MAGIC: u32 = 0xEEEEEEBE;

/// Valid meta file lengths.
const META_LENGTH_VANILLA: usize = 0x68; // 104 bytes, format 2001
const META_LENGTH_WAYPOINT: usize = 0x168; // 360 bytes, format 2002
const META_LENGTH_WORLDS_PART_I: usize = 0x180; // 384 bytes, format 2003
const META_LENGTH_WORLDS_PART_II: usize = 0x1B0; // 432 bytes, format 2004

/// XXTEA rounds for vanilla-length meta files.
const ROUNDS_VANILLA: usize = 8;

/// XXTEA rounds for all other meta file lengths.
const ROUNDS_DEFAULT: usize = 6;

/// Meta format version constants.
const META_FORMAT_VANILLA: u32 = 0x7D0; // 2000 -- NOT SUPPORTED
const META_FORMAT_FOUNDATION: u32 = 0x7D1; // 2001

/// All valid metadata file lengths.
const VALID_META_LENGTHS: [usize; 4] = [
    META_LENGTH_VANILLA,
    META_LENGTH_WAYPOINT,
    META_LENGTH_WORLDS_PART_I,
    META_LENGTH_WORLDS_PART_II,
];

// ---------------------------------------------------------------------------
// StorageSlot
// ---------------------------------------------------------------------------

/// Storage slot enum values used for XXTEA key derivation.
///
/// Each value corresponds to a file index. The numeric value is XOR'd
/// with a constant during key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageSlot {
    UserSettings = 0,
    AccountData = 1,
    PlayerState1 = 2,
    PlayerState2 = 3,
    PlayerState3 = 4,
    PlayerState4 = 5,
    PlayerState5 = 6,
    PlayerState6 = 7,
    PlayerState7 = 8,
    PlayerState8 = 9,
    PlayerState9 = 10,
    PlayerState10 = 11,
    PlayerState11 = 12,
    PlayerState12 = 13,
    PlayerState13 = 14,
    PlayerState14 = 15,
    PlayerState15 = 16,
    PlayerState16 = 17,
    PlayerState17 = 18,
    PlayerState18 = 19,
    PlayerState19 = 20,
    PlayerState20 = 21,
    PlayerState21 = 22,
    PlayerState22 = 23,
    PlayerState23 = 24,
    PlayerState24 = 25,
    PlayerState25 = 26,
    PlayerState26 = 27,
    PlayerState27 = 28,
    PlayerState28 = 29,
    PlayerState29 = 30,
    PlayerState30 = 31,
}

impl StorageSlot {
    /// All valid slot values.
    pub const ALL: [StorageSlot; 32] = [
        Self::UserSettings,
        Self::AccountData,
        Self::PlayerState1,
        Self::PlayerState2,
        Self::PlayerState3,
        Self::PlayerState4,
        Self::PlayerState5,
        Self::PlayerState6,
        Self::PlayerState7,
        Self::PlayerState8,
        Self::PlayerState9,
        Self::PlayerState10,
        Self::PlayerState11,
        Self::PlayerState12,
        Self::PlayerState13,
        Self::PlayerState14,
        Self::PlayerState15,
        Self::PlayerState16,
        Self::PlayerState17,
        Self::PlayerState18,
        Self::PlayerState19,
        Self::PlayerState20,
        Self::PlayerState21,
        Self::PlayerState22,
        Self::PlayerState23,
        Self::PlayerState24,
        Self::PlayerState25,
        Self::PlayerState26,
        Self::PlayerState27,
        Self::PlayerState28,
        Self::PlayerState29,
        Self::PlayerState30,
    ];

    /// Whether this slot represents account-level data (not a save slot).
    pub fn is_account(&self) -> bool {
        matches!(self, Self::UserSettings | Self::AccountData)
    }
}

// ---------------------------------------------------------------------------
// SaveMetadata
// ---------------------------------------------------------------------------

/// Decrypted and parsed metadata from an `mf_save*.hg` file.
#[derive(Debug, Clone)]
pub struct SaveMetadata {
    /// Format version: 0x7D1 (2001), 0x7D2 (2002), 0x7D3 (2003), 0x7D4 (2004).
    pub format_version: u32,
    /// Decompressed JSON size in bytes.
    pub decompressed_size: u32,
    /// Total compressed data size in bytes.
    pub compressed_size: u32,
    /// Profile hash (0 if none).
    pub profile_hash: u32,
    /// SpookyHash V2 keys (format 2001 only, None for 2002+).
    pub spooky_hash: Option<[u64; 2]>,
    /// SHA-256 of the raw storage file (format 2001 only, None for 2002+).
    pub sha256_hash: Option<[u8; 32]>,
    /// The storage slot that successfully decrypted this metadata.
    pub decrypted_with_slot: StorageSlot,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decrypt and parse a metadata file.
///
/// `data` is the raw bytes of the `mf_save*.hg` file.
/// `slot` is the expected storage slot for this file.
///
/// The function first tries decryption with the given `slot`. If the magic
/// sentinel is not found, it tries all other valid slots (in case the file
/// was manually moved between save directories).
pub fn read_metadata(data: &[u8], slot: StorageSlot) -> Result<SaveMetadata, SaveError> {
    if !VALID_META_LENGTHS.contains(&data.len()) {
        return Err(SaveError::InvalidMetaLength { length: data.len() });
    }

    let iterations = if data.len() == META_LENGTH_VANILLA {
        ROUNDS_VANILLA
    } else {
        ROUNDS_DEFAULT
    };

    let u32_count = data.len() / 4;
    let words: Vec<u32> = (0..u32_count)
        .map(|i| u32::from_le_bytes(data[i * 4..(i + 1) * 4].try_into().unwrap()))
        .collect();

    // Try expected slot first, then all others of the same kind.
    let is_account = slot.is_account();
    let slots_to_try: Vec<StorageSlot> = std::iter::once(slot)
        .chain(
            StorageSlot::ALL
                .iter()
                .copied()
                .filter(|&s| s != slot && s.is_account() == is_account),
        )
        .collect();

    for try_slot in &slots_to_try {
        let mut attempt = words.clone();
        let key = derive_key(*try_slot);
        xxtea_decrypt(&mut attempt, &key, iterations);

        if attempt[0] == META_MAGIC {
            return parse_decrypted_metadata(&attempt, *try_slot);
        }
    }

    Err(SaveError::MetaDecryptionFailed)
}

/// Verify the SHA-256 hash stored in metadata against the raw save file bytes.
///
/// Only applicable for format 2001 (Foundation through Prisms). For format 2002+,
/// SHA-256 is not used and this function returns `true`.
pub fn verify_sha256(metadata: &SaveMetadata, raw_save_bytes: &[u8]) -> bool {
    use sha2::{Digest, Sha256};

    match metadata.sha256_hash {
        Some(expected) => {
            let mut hasher = Sha256::new();
            hasher.update(raw_save_bytes);
            let actual: [u8; 32] = hasher.finalize().into();
            actual == expected
        }
        None => true,
    }
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

/// Parse fields from a successfully decrypted metadata u32 array.
fn parse_decrypted_metadata(words: &[u32], slot: StorageSlot) -> Result<SaveMetadata, SaveError> {
    let format_version = words[1];

    if format_version == META_FORMAT_VANILLA {
        return Err(SaveError::UnsupportedMetaFormat {
            version: format_version,
        });
    }

    let spooky_hash = if format_version == META_FORMAT_FOUNDATION {
        let h0 = (words[2] as u64) | ((words[3] as u64) << 32);
        let h1 = (words[4] as u64) | ((words[5] as u64) << 32);
        Some([h0, h1])
    } else {
        None
    };

    let sha256_hash = if format_version == META_FORMAT_FOUNDATION {
        let mut hash = [0u8; 32];
        for i in 0..8 {
            hash[i * 4..(i + 1) * 4].copy_from_slice(&words[6 + i].to_le_bytes());
        }
        Some(hash)
    } else {
        None
    };

    let decompressed_size = words[14];
    let compressed_size = words[15];
    let profile_hash = words[16];

    Ok(SaveMetadata {
        format_version,
        decompressed_size,
        compressed_size,
        profile_hash,
        spooky_hash,
        sha256_hash,
        decrypted_with_slot: slot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xxtea::{META_ENCRYPTION_KEY, xxtea_encrypt};

    #[test]
    fn invalid_meta_length() {
        let data = vec![0u8; 50];
        let err = read_metadata(&data, StorageSlot::PlayerState1).unwrap_err();
        match err {
            SaveError::InvalidMetaLength { length } => assert_eq!(length, 50),
            _ => panic!("expected InvalidMetaLength, got {err:?}"),
        }
    }

    #[test]
    fn valid_meta_lengths_reach_decryption() {
        for &len in &[0x68, 0x168, 0x180, 0x1B0] {
            let data = vec![0u8; len];
            let err = read_metadata(&data, StorageSlot::PlayerState1).unwrap_err();
            assert!(
                matches!(err, SaveError::MetaDecryptionFailed),
                "length {len:#x} should reach decryption stage, got {err:?}"
            );
        }
    }

    #[test]
    fn parse_decrypted_metadata_format_2002() {
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;
        words[1] = 0x7D2; // Frontiers
        words[14] = 1_000_000;
        words[15] = 500_000;
        words[16] = 0x12345678;

        let meta = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap();
        assert_eq!(meta.format_version, 0x7D2);
        assert_eq!(meta.decompressed_size, 1_000_000);
        assert_eq!(meta.compressed_size, 500_000);
        assert_eq!(meta.profile_hash, 0x12345678);
        assert!(meta.spooky_hash.is_none());
        assert!(meta.sha256_hash.is_none());
    }

    #[test]
    fn parse_decrypted_metadata_format_2001() {
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;
        words[1] = META_FORMAT_FOUNDATION;
        words[2] = 0xAABBCCDD;
        words[3] = 0x11223344;
        words[4] = 0x55667788;
        words[5] = 0x99AABBCC;
        for i in 6..14 {
            words[i] = (i as u32) * 0x01010101;
        }
        words[14] = 2_000_000;
        words[15] = 800_000;
        words[16] = 0;

        let meta = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap();
        assert_eq!(meta.format_version, 0x7D1);
        let spooky = meta.spooky_hash.unwrap();
        assert_eq!(spooky[0], 0x11223344_AABBCCDD_u64);
        assert_eq!(spooky[1], 0x99AABBCC_55667788_u64);
        assert!(meta.sha256_hash.is_some());
    }

    #[test]
    fn unsupported_vanilla_format() {
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;
        words[1] = META_FORMAT_VANILLA;
        let err = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap_err();
        match err {
            SaveError::UnsupportedMetaFormat { version } => assert_eq!(version, 0x7D0),
            _ => panic!("expected UnsupportedMetaFormat, got {err:?}"),
        }
    }

    #[test]
    fn storage_slot_is_account() {
        assert!(StorageSlot::UserSettings.is_account());
        assert!(StorageSlot::AccountData.is_account());
        assert!(!StorageSlot::PlayerState1.is_account());
        assert!(!StorageSlot::PlayerState30.is_account());
    }

    #[test]
    fn storage_slot_all_has_32_entries() {
        assert_eq!(StorageSlot::ALL.len(), 32);
    }

    #[test]
    fn read_metadata_with_synthetic_encrypted_data() {
        // Build plaintext metadata, encrypt it, then verify read_metadata decrypts it.
        let slot = StorageSlot::PlayerState1;
        let iterations = ROUNDS_DEFAULT;

        // Build a 360-byte (META_LENGTH_WAYPOINT) metadata: 90 u32s
        let u32_count = META_LENGTH_WAYPOINT / 4;
        let mut words = vec![0u32; u32_count];
        words[0] = META_MAGIC;
        words[1] = 0x7D2; // format 2002
        words[14] = 5_000_000;
        words[15] = 2_000_000;
        words[16] = 0xDEADBEEF;

        // Encrypt
        let key = derive_key(slot);
        let mut encrypted = words.clone();
        xxtea_encrypt(&mut encrypted, &key, iterations);

        // Convert to bytes (little-endian)
        let data: Vec<u8> = encrypted.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(data.len(), META_LENGTH_WAYPOINT);

        // Decrypt and parse
        let meta = read_metadata(&data, slot).unwrap();
        assert_eq!(meta.format_version, 0x7D2);
        assert_eq!(meta.decompressed_size, 5_000_000);
        assert_eq!(meta.compressed_size, 2_000_000);
        assert_eq!(meta.profile_hash, 0xDEADBEEF);
        assert_eq!(meta.decrypted_with_slot, slot);
    }

    #[test]
    fn read_metadata_tries_other_slots() {
        // Encrypt with PlayerState5, but pass PlayerState1 as expected slot.
        let actual_slot = StorageSlot::PlayerState5;
        let wrong_slot = StorageSlot::PlayerState1;
        let iterations = ROUNDS_DEFAULT;

        let u32_count = META_LENGTH_WAYPOINT / 4;
        let mut words = vec![0u32; u32_count];
        words[0] = META_MAGIC;
        words[1] = 0x7D3;
        words[14] = 100;
        words[15] = 50;

        let key = derive_key(actual_slot);
        let mut encrypted = words.clone();
        xxtea_encrypt(&mut encrypted, &key, iterations);

        let data: Vec<u8> = encrypted.iter().flat_map(|w| w.to_le_bytes()).collect();

        // Should succeed by trying all slots
        let meta = read_metadata(&data, wrong_slot).unwrap();
        assert_eq!(meta.decrypted_with_slot, actual_slot);
    }

    #[test]
    fn read_metadata_vanilla_length_uses_8_rounds() {
        let slot = StorageSlot::PlayerState1;
        let iterations = ROUNDS_VANILLA;

        let u32_count = META_LENGTH_VANILLA / 4;
        let mut words = vec![0u32; u32_count];
        words[0] = META_MAGIC;
        words[1] = META_FORMAT_FOUNDATION; // 2001
        words[14] = 999;
        words[15] = 500;

        let key = derive_key(slot);
        let mut encrypted = words.clone();
        xxtea_encrypt(&mut encrypted, &key, iterations);

        let data: Vec<u8> = encrypted.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(data.len(), META_LENGTH_VANILLA);

        let meta = read_metadata(&data, slot).unwrap();
        assert_eq!(meta.format_version, 0x7D1);
        assert_eq!(meta.decompressed_size, 999);
    }

    #[test]
    fn verify_sha256_no_hash_returns_true() {
        let meta = SaveMetadata {
            format_version: 0x7D2,
            decompressed_size: 0,
            compressed_size: 0,
            profile_hash: 0,
            spooky_hash: None,
            sha256_hash: None,
            decrypted_with_slot: StorageSlot::PlayerState1,
        };
        assert!(verify_sha256(&meta, b"anything"));
    }

    #[test]
    fn verify_sha256_correct_hash() {
        use sha2::{Digest, Sha256};

        let data = b"test data for hashing";
        let hash: [u8; 32] = Sha256::digest(data).into();

        let meta = SaveMetadata {
            format_version: 0x7D1,
            decompressed_size: 0,
            compressed_size: 0,
            profile_hash: 0,
            spooky_hash: None,
            sha256_hash: Some(hash),
            decrypted_with_slot: StorageSlot::PlayerState1,
        };
        assert!(verify_sha256(&meta, data));
    }

    #[test]
    fn verify_sha256_wrong_hash() {
        let meta = SaveMetadata {
            format_version: 0x7D1,
            decompressed_size: 0,
            compressed_size: 0,
            profile_hash: 0,
            spooky_hash: None,
            sha256_hash: Some([0xFF; 32]),
            decrypted_with_slot: StorageSlot::PlayerState1,
        };
        assert!(!verify_sha256(&meta, b"test data"));
    }

    #[test]
    fn meta_encryption_key_values() {
        // Verify the key matches "NAESEVADNAYRTNRG"
        assert_eq!(META_ENCRYPTION_KEY[0], 0x5345414E);
        assert_eq!(META_ENCRYPTION_KEY[1], 0x44415645);
        assert_eq!(META_ENCRYPTION_KEY[2], 0x5259414E);
        assert_eq!(META_ENCRYPTION_KEY[3], 0x47524E54);
    }
}
