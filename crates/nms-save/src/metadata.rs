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

/// Storage-file indices used to derive XXTEA keys for metadata files.
///
/// The enum has 32 values: `UserSettings` is 0, `AccountData` is 1, and
/// `PlayerState1` through `PlayerState30` are 2 through 31. These are file
/// indices, not the 15 user-visible paired save slots. Metadata fallback tries
/// the requested index first, then only other indices of the same account-
/// versus-player kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageSlot {
    /// Account-level metadata file index 0.
    UserSettings = 0,
    /// Account-level metadata file index 1.
    AccountData = 1,
    /// Player-state metadata file index 2.
    PlayerState1 = 2,
    /// Player-state metadata file index 3.
    PlayerState2 = 3,
    /// Player-state metadata file index 4.
    PlayerState3 = 4,
    /// Player-state metadata file index 5.
    PlayerState4 = 5,
    /// Player-state metadata file index 6.
    PlayerState5 = 6,
    /// Player-state metadata file index 7.
    PlayerState6 = 7,
    /// Player-state metadata file index 8.
    PlayerState7 = 8,
    /// Player-state metadata file index 9.
    PlayerState8 = 9,
    /// Player-state metadata file index 10.
    PlayerState9 = 10,
    /// Player-state metadata file index 11.
    PlayerState10 = 11,
    /// Player-state metadata file index 12.
    PlayerState11 = 12,
    /// Player-state metadata file index 13.
    PlayerState12 = 13,
    /// Player-state metadata file index 14.
    PlayerState13 = 14,
    /// Player-state metadata file index 15.
    PlayerState14 = 15,
    /// Player-state metadata file index 16.
    PlayerState15 = 16,
    /// Player-state metadata file index 17.
    PlayerState16 = 17,
    /// Player-state metadata file index 18.
    PlayerState17 = 18,
    /// Player-state metadata file index 19.
    PlayerState18 = 19,
    /// Player-state metadata file index 20.
    PlayerState19 = 20,
    /// Player-state metadata file index 21.
    PlayerState20 = 21,
    /// Player-state metadata file index 22.
    PlayerState21 = 22,
    /// Player-state metadata file index 23.
    PlayerState22 = 23,
    /// Player-state metadata file index 24.
    PlayerState23 = 24,
    /// Player-state metadata file index 25.
    PlayerState24 = 25,
    /// Player-state metadata file index 26.
    PlayerState25 = 26,
    /// Player-state metadata file index 27.
    PlayerState26 = 27,
    /// Player-state metadata file index 28.
    PlayerState27 = 28,
    /// Player-state metadata file index 29.
    PlayerState28 = 29,
    /// Player-state metadata file index 30.
    PlayerState29 = 30,
    /// Player-state metadata file index 31.
    PlayerState30 = 31,
}

impl StorageSlot {
    /// All 32 supported storage-file indices in numeric order.
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

    /// Whether this index is one of the two account-level files.
    ///
    /// `UserSettings` and `AccountData` return `true`; all 30 player-state
    /// indices return `false`. This classification controls same-kind fallback
    /// during [`read_metadata`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_save::StorageSlot;
    /// assert!(StorageSlot::UserSettings.is_account());
    /// assert!(!StorageSlot::PlayerState1.is_account());
    /// ```
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
    /// Decrypted format value; only `0x7D0` is rejected by the parser.
    pub format_version: u32,
    /// Decompressed JSON size in bytes.
    pub decompressed_size: u32,
    /// Total compressed data size in bytes.
    pub compressed_size: u32,
    /// Profile hash (0 if none).
    pub profile_hash: u32,
    /// SpookyHash V2 key pair, populated for `0x7D1` and absent otherwise.
    pub spooky_hash: Option<[u64; 2]>,
    /// SHA-256 of raw storage bytes for `0x7D1`, absent for other accepted values.
    pub sha256_hash: Option<[u8; 32]>,
    /// The storage slot that successfully decrypted this metadata.
    pub decrypted_with_slot: StorageSlot,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decrypt and parse a metadata file.
///
/// `data` is the complete raw content of an `mf_save*.hg` file. Accepted
/// lengths are 104, 360, 384, and 432 bytes. `slot` is the expected
/// storage-file index. The expected index is tried first, followed by other
/// indices of the same account-versus-player kind.
///
/// A 104-byte file uses eight XXTEA rounds; the other accepted sizes use six.
/// The first decrypted value with the magic sentinel is parsed immediately.
/// Only metadata version `0x7D0` is rejected; this parser does not enforce a
/// closed version range.
///
/// # Examples
///
/// A 50-byte input is rejected before any decryption attempt:
///
/// ```
/// use nms_save::{SaveError, StorageSlot, read_metadata};
/// let error = read_metadata(&[0; 50], StorageSlot::PlayerState1)
///     .err()
///     .ok_or_else(|| std::io::Error::other("expected an invalid-length error"))?;
/// assert!(matches!(error, SaveError::InvalidMetaLength { length: 50 }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// This synthetic 360-byte record is encrypted for `PlayerState5`. The
/// requested `PlayerState1` index is tried first, then same-kind player files.
/// It illustrates the fallback result; it is not an authentic game metadata file.
///
/// ```
/// use nms_save::{StorageSlot, read_metadata};
/// use nms_save::xxtea::{derive_key, xxtea_encrypt};
/// let mut words = vec![0_u32; 90];
/// words[0] = 0xEEEEEEBE;
/// words[1] = 0x7D2;
/// words[14] = 1000;
/// words[15] = 500;
/// words[16] = 0x12345678;
/// let key = derive_key(StorageSlot::PlayerState5);
/// xxtea_encrypt(&mut words, &key, 6);
/// let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
/// let metadata = read_metadata(&bytes, StorageSlot::PlayerState1)?;
/// assert_eq!(metadata.decrypted_with_slot, StorageSlot::PlayerState5);
/// assert_eq!(metadata.format_version, 0x7D2);
/// assert_eq!(metadata.decompressed_size, 1000);
/// assert_eq!(metadata.compressed_size, 500);
/// assert_eq!(metadata.profile_hash, 0x12345678);
/// assert!(metadata.spooky_hash.is_none());
/// assert!(metadata.sha256_hash.is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`SaveError::InvalidMetaLength`] for an unsupported length,
/// [`SaveError::MetaDecryptionFailed`] when no same-kind key yields the
/// sentinel, or the parse error for a decrypted unsupported version.
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

/// Compare a stored SHA-256 digest with supplied raw save-file bytes.
///
/// When metadata contains a digest (the 2001 format), this returns whether the
/// computed digest matches it. When no digest is stored, it returns `true`;
/// that means there was no digest to compare, not that the bytes were
/// independently verified. This function does not read a file or return an
/// error.
///
/// # Examples
///
/// ```
/// let mut metadata = nms_save::SaveMetadata {
///     format_version: 0x7D2,
///     decompressed_size: 0,
///     compressed_size: 0,
///     profile_hash: 0,
///     spooky_hash: None,
///     sha256_hash: None,
///     decrypted_with_slot: nms_save::StorageSlot::PlayerState1,
/// };
/// assert!(nms_save::verify_sha256(&metadata, b"no stored digest"));
/// metadata.sha256_hash = Some([0; 32]);
/// assert!(!nms_save::verify_sha256(&metadata, b"abc"));
/// ```
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
        for (i, word) in words.iter_mut().enumerate().take(14).skip(6) {
            *word = (i as u32) * 0x01010101;
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
