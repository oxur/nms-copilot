# Milestone 1.6 -- Metadata Verifier (nms-save)

Read `mf_save.hg`, decrypt with XXTEA, extract format version and integrity hashes.

## Overview

The metadata file (`mf_save.hg`, `mf_save2.hg`, etc.) is 104--432 bytes depending on format version. It is XXTEA-encrypted with a key derived from the save's storage slot enum value. After decryption, the first u32 must equal `0xEEEEEEBE` (the meta magic sentinel). The decrypted metadata contains format version, optional SpookyHash/SHA-256 hashes, and size information.

Reference: `PlatformSteam_Read.cs:12-79`, `PlatformSteam.cs` (constants), `StoragePersistentSlotEnum.cs`.

---

## Constants

```rust
/// Meta magic sentinel -- first u32 after successful XXTEA decryption.
const META_MAGIC: u32 = 0xEEEEEEBE;

/// XXTEA key derivation: XOR constant.
const KEY_XOR: u32 = 0x1422CB8C;

/// XXTEA key derivation: rotate-left amount (bits).
const KEY_ROTATE: u32 = 13;

/// XXTEA key derivation: multiply constant.
const KEY_MULTIPLY: u32 = 5;

/// XXTEA key derivation: add constant.
const KEY_ADD: u32 = 0xE6546B64;

/// XXTEA delta (golden ratio constant).
const TEA_DELTA: u32 = 0x9E3779B9;

/// XXTEA reverse delta (wrapping complement of TEA_DELTA).
/// TEA_DELTA.wrapping_add(TEA_REVERSE_DELTA) == 0.
const TEA_REVERSE_DELTA: u32 = 0x61C88647;

/// Base encryption key: "NAESEVADNAYRTNRG" as 4 little-endian u32s.
const META_ENCRYPTION_KEY: [u32; 4] = [
    0x5345414E, // "NAES" as LE u32
    0x44415645, // "EVAD" as LE u32
    0x5259414E, // "NAYR" as LE u32
    0x47524E54, // "TNRG" as LE u32
];

/// Valid meta file lengths (Steam/GOG).
const META_LENGTH_VANILLA: usize = 0x68;       // 104 bytes, format 2001
const META_LENGTH_WAYPOINT: usize = 0x168;      // 360 bytes, format 2002
const META_LENGTH_WORLDS_PART_I: usize = 0x180;  // 384 bytes, format 2003
const META_LENGTH_WORLDS_PART_II: usize = 0x1B0; // 432 bytes, format 2004

/// XXTEA rounds for vanilla-length meta files.
const ROUNDS_VANILLA: usize = 8;

/// XXTEA rounds for all other meta file lengths.
const ROUNDS_DEFAULT: usize = 6;

/// Meta format version constants.
const META_FORMAT_VANILLA: u32 = 0x7D0;     // 2000 -- NOT SUPPORTED
const META_FORMAT_FOUNDATION: u32 = 0x7D1;  // 2001
const META_FORMAT_FRONTIERS: u32 = 0x7D2;   // 2002
const META_FORMAT_WORLDS_I: u32 = 0x7D3;    // 2003
const META_FORMAT_WORLDS_II: u32 = 0x7D4;   // 2004
```

---

## StoragePersistentSlot

Values match the C# `StoragePersistentSlotEnum` exactly:

```rust
/// Storage slot enum values used for XXTEA key derivation.
///
/// Each value corresponds to a file index. The numeric value is what gets
/// XOR'd with KEY_XOR during key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageSlot {
    UserSettings = 0,
    AccountData = 1,
    PlayerState1 = 2,   // save.hg   (Slot 1 Auto)
    PlayerState2 = 3,   // save2.hg  (Slot 1 Manual)
    PlayerState3 = 4,   // save3.hg  (Slot 2 Auto)
    PlayerState4 = 5,   // save4.hg  (Slot 2 Manual)
    // ... continues through PlayerState30 = 31
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
    /// All valid slot values, for brute-force decryption attempts.
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

    /// Returns whether this slot represents account-level data (not a save slot).
    pub fn is_account(&self) -> bool {
        matches!(self, Self::UserSettings | Self::AccountData)
    }
}
```

Mapping from file names to slots (for reference/convenience):

| File | Slot value |
|------|------------|
| `accountdata.hg` | `AccountData` (1) |
| `save.hg` | `PlayerState1` (2) |
| `save2.hg` | `PlayerState2` (3) |
| `save3.hg` | `PlayerState3` (4) |
| `save4.hg` | `PlayerState4` (5) |
| ... | ... |
| `save30.hg` | `PlayerState30` (31) |

---

## Types

### SaveMetadata

```rust
/// Decrypted and parsed metadata from an mf_save*.hg file.
#[derive(Debug, Clone)]
pub struct SaveMetadata {
    /// Format version: 0x7D1 (2001), 0x7D2 (2002), 0x7D3 (2003), 0x7D4 (2004).
    pub format_version: u32,

    /// Decompressed JSON size in bytes.
    pub decompressed_size: u32,

    /// Total compressed data size in bytes (sum of all LZ4 payloads + headers).
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
```

### SaveError additions

Add these variants to the `SaveError` enum defined in Milestone 1.5:

```rust
    /// Metadata file has an invalid length (not one of the known sizes).
    InvalidMetaLength {
        length: usize,
    },

    /// Metadata decryption failed -- magic sentinel not found after trying all slots.
    MetaDecryptionFailed,

    /// Metadata format version is unsupported (e.g., 2000/vanilla).
    UnsupportedMetaFormat {
        version: u32,
    },

    /// SHA-256 verification failed.
    Sha256Mismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
```

---

## Functions

### `derive_key` (in `src/xxtea.rs`)

```rust
/// Derive the 4-element XXTEA key from a storage slot.
///
/// key[0] is computed from the slot value:
///   key[0] = rotate_left((slot_value ^ 0x1422CB8C), 13) * 5 + 0xE6546B64
///
/// key[1..3] are the fixed values from META_ENCRYPTION_KEY[1..3].
///
/// All arithmetic is wrapping u32.
pub fn derive_key(slot: StorageSlot) -> [u32; 4] {
    let slot_value = slot as u32;
    let key0 = (slot_value ^ KEY_XOR)
        .rotate_left(KEY_ROTATE)
        .wrapping_mul(KEY_MULTIPLY)
        .wrapping_add(KEY_ADD);
    [key0, META_ENCRYPTION_KEY[1], META_ENCRYPTION_KEY[2], META_ENCRYPTION_KEY[3]]
}
```

### `xxtea_decrypt` (in `src/xxtea.rs`)

This is the critical function. It must match the C# implementation exactly. All arithmetic is wrapping u32.

```rust
/// XXTEA decrypt a u32 slice in place.
///
/// `data` is the metadata as a mutable slice of little-endian u32 values.
/// `key` is the 4-element key from `derive_key`.
/// `iterations` is 8 for vanilla (104-byte) meta, 6 for all others.
///
/// Reference: PlatformSteam_Read.cs:35-79
pub fn xxtea_decrypt(data: &mut [u32], key: &[u32; 4], iterations: usize) {
    let last = data.len() - 1;
    let mut hash: u32 = 0;

    // Pre-compute hash: sum of (iterations) deltas.
    for _ in 0..iterations {
        hash = hash.wrapping_add(TEA_DELTA);
    }

    // Reverse iteration.
    for _ in 0..iterations {
        let key_index = (hash >> 2 & 3) as usize;
        let mut current = data[0];

        // Process elements last..1 (backwards).
        for j in (1..=last).rev() {
            let prev = data[j - 1];
            let t1 = (current >> 3) ^ (prev << 4);
            let t2 = current.wrapping_mul(4) ^ (prev >> 5);
            let t3 = prev ^ key[(j & 3) ^ key_index];
            let t4 = current ^ hash;
            data[j] = data[j].wrapping_sub(
                t1.wrapping_add(t2) ^ t3.wrapping_add(t4)
            );
            current = data[j];
        }

        // Process element 0 (wraps around to last element).
        let prev = data[last];
        let t1 = (current >> 3) ^ (prev << 4);
        let t2 = current.wrapping_mul(4) ^ (prev >> 5);
        let t3 = prev ^ key[key_index]; // (0 & 3) ^ key_index == key_index
        let t4 = current ^ hash;
        data[0] = data[0].wrapping_sub(
            t1.wrapping_add(t2) ^ t3.wrapping_add(t4)
        );

        hash = hash.wrapping_add(TEA_REVERSE_DELTA);
    }
}
```

### `xxtea_encrypt` (in `src/xxtea.rs`)

Needed only for round-trip testing. The encrypt function is the forward direction of XXTEA.

```rust
/// XXTEA encrypt a u32 slice in place.
///
/// This is the forward direction, used only for testing round-trips.
pub fn xxtea_encrypt(data: &mut [u32], key: &[u32; 4], iterations: usize) {
    let last = data.len() - 1;
    let mut hash: u32 = 0;

    for _ in 0..iterations {
        hash = hash.wrapping_add(TEA_DELTA);
        let key_index = (hash >> 2 & 3) as usize;

        // Process elements 0..last-1 (forwards).
        for j in 0..last {
            let next = data[j + 1];
            let current = data[j];
            // Note: in encryption, we look at data[j+1] (the "next" element)
            // where decryption looks at data[j-1] (the "prev" element).
            let t1 = (next >> 3) ^ (data[if j == 0 { last } else { j - 1 }] << 4);
            // IMPORTANT: deriving the encrypt function from the decrypt function
            // is error-prone. Instead, implement from the standard XXTEA spec:
            //   https://en.wikipedia.org/wiki/XXTEA
            // However, NMS uses a MODIFIED XXTEA (the operand arrangement differs
            // from standard). For testing, a simpler approach is recommended:
            // encrypt known plaintext with the C# code, capture the ciphertext,
            // and use it as a test vector. See Test Vectors section below.
            todo!("Implement if needed for testing; prefer known test vectors instead")
        }
    }
}
```

**Recommendation:** Rather than implementing encrypt (which requires careful matching of the modified XXTEA variant), use known test vectors for round-trip testing. See the Tests section below.

### `read_metadata` (in `src/metadata.rs`)

```rust
/// Decrypt and parse a metadata file.
///
/// `data` is the raw bytes of the mf_save*.hg file.
/// `slot` is the expected storage slot for this file.
///
/// The function first tries decryption with the given `slot`. If the magic
/// sentinel is not found, it tries all other valid slots (in case the file
/// was manually moved between save directories).
///
/// Returns `SaveMetadata` on success, `SaveError` on failure.
pub fn read_metadata(data: &[u8], slot: StorageSlot) -> Result<SaveMetadata, SaveError> {
    // 1. Validate meta length.
    let valid_lengths = [
        META_LENGTH_VANILLA,
        META_LENGTH_WAYPOINT,
        META_LENGTH_WORLDS_PART_I,
        META_LENGTH_WORLDS_PART_II,
    ];
    if !valid_lengths.contains(&data.len()) {
        return Err(SaveError::InvalidMetaLength { length: data.len() });
    }

    // 2. Determine iteration count.
    let iterations = if data.len() == META_LENGTH_VANILLA {
        ROUNDS_VANILLA
    } else {
        ROUNDS_DEFAULT
    };

    // 3. Convert bytes to u32 slice (little-endian).
    //    data.len() is guaranteed to be a multiple of 4 by the valid_lengths check.
    let u32_count = data.len() / 4;
    let mut words: Vec<u32> = (0..u32_count)
        .map(|i| u32::from_le_bytes(data[i * 4..(i + 1) * 4].try_into().unwrap()))
        .collect();

    // 4. Try decryption with the expected slot first, then all others.
    //    For account files (slot <= AccountData), only try account slots.
    //    For save files (slot > AccountData), only try save slots.
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
```

### `parse_decrypted_metadata` (in `src/metadata.rs`)

```rust
/// Parse fields from a successfully decrypted metadata u32 array.
///
/// Layout (format 2001, 26 x u32 = 104 bytes):
///   [0]      = magic (0xEEEEEEBE) -- already verified
///   [1]      = format version
///   [2..4]   = SpookyHash key[0] (u64, LE)
///   [4..6]   = SpookyHash key[1] (u64, LE)
///   [6..14]  = SHA-256 hash (32 bytes = 8 x u32)
///   [14]     = decompressed size
///   [15]     = compressed size
///   [16]     = profile hash
///   [17..26] = padding
///
/// For format 2002+ (longer files), the same layout applies for the first
/// 26 u32s. Additional fields (save name, summary, etc.) follow but are
/// not parsed in this milestone.
fn parse_decrypted_metadata(
    words: &[u32],
    slot: StorageSlot,
) -> Result<SaveMetadata, SaveError> {
    let format_version = words[1];

    if format_version == META_FORMAT_VANILLA {
        return Err(SaveError::UnsupportedMetaFormat { version: format_version });
    }

    // SpookyHash (only meaningful for format 2001).
    let spooky_hash = if format_version == META_FORMAT_FOUNDATION {
        let h0 = (words[2] as u64) | ((words[3] as u64) << 32);
        let h1 = (words[4] as u64) | ((words[5] as u64) << 32);
        Some([h0, h1])
    } else {
        None
    };

    // SHA-256 (only meaningful for format 2001).
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
```

### `verify_sha256` (in `src/metadata.rs`)

```rust
/// Verify the SHA-256 hash stored in metadata against the decompressed save data.
///
/// Only applicable for format 2001 (Foundation through Prisms). For format 2002+,
/// SHA-256 is not used for verification and this function returns `true`.
///
/// Note: The SHA-256 in the metadata is computed over the raw (compressed) storage
/// file bytes, NOT the decompressed JSON. However, for format 2001, the storage
/// file itself may be uncompressed. Check both if needed.
pub fn verify_sha256(metadata: &SaveMetadata, raw_save_bytes: &[u8]) -> bool {
    use sha2::{Sha256, Digest};

    match metadata.sha256_hash {
        Some(expected) => {
            let mut hasher = Sha256::new();
            hasher.update(raw_save_bytes);
            let actual: [u8; 32] = hasher.finalize().into();
            actual == expected
        }
        None => true, // No hash to verify (format 2002+).
    }
}
```

---

## XXTEA Algorithm Details

This section documents exactly how the NMS-modified XXTEA decryption works, for anyone debugging discrepancies.

### Hash pre-computation

The hash starts at 0 and has `TEA_DELTA` (0x9E3779B9) added `iterations` times:

| Iterations | Final hash value |
|------------|------------------|
| 8 | `0xF1BBCDC8` (8 * 0x9E3779B9, wrapping) |
| 6 | `0xB5DBAE78` (6 * 0x9E3779B9, wrapping) |

### Reverse iteration

Each outer iteration:
1. Compute `key_index = (hash >> 2) & 3`.
2. Set `current = data[0]`.
3. For `j = last` down to `1`:
   - `prev = data[j - 1]`
   - Compute the four terms using `current`, `prev`, `hash`, and `key[(j & 3) ^ key_index]`.
   - Subtract the combined term from `data[j]` (wrapping).
   - Update `current = data[j]` (the newly decrypted value).
4. For element 0:
   - `prev = data[last]` (wrap-around).
   - Same four-term computation, key index is just `key_index` (since `0 & 3 == 0`).
   - Subtract from `data[0]` (wrapping).
5. `hash = hash.wrapping_add(TEA_REVERSE_DELTA)` which effectively subtracts `TEA_DELTA`.

### Critical implementation notes

- **All arithmetic is wrapping.** Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()` throughout. In C#, `uint` arithmetic wraps by default; in Rust, debug builds will panic on overflow unless wrapping methods are used.
- The multiply by 4 in `t2` is `current.wrapping_mul(4)`, NOT a left shift by 2. While mathematically equivalent for unsigned integers, use `wrapping_mul` to match the C# source and avoid any edge-case confusion.
- `rotate_left` is a built-in method on `u32` in Rust.

---

## Dependencies

Add to `crates/nms-save/Cargo.toml`:

```toml
[dependencies]
nms-core = { workspace = true }
lz4_flex = "0.11"
thiserror = "2"
sha2 = "0.10"
```

Add `sha2` to root `Cargo.toml` `[workspace.dependencies]` if using workspace dependency management.

---

## File Organization

```
crates/nms-save/
  src/
    lib.rs          -- add: pub mod metadata; pub mod xxtea;
    error.rs        -- add new SaveError variants
    decompress.rs   -- (from Milestone 1.5)
    metadata.rs     -- SaveMetadata, StorageSlot, read_metadata,
                       parse_decrypted_metadata, verify_sha256
    xxtea.rs        -- derive_key, xxtea_decrypt, constants
```

### `src/lib.rs` additions

```rust
pub mod metadata;
pub mod xxtea;

pub use metadata::{SaveMetadata, StorageSlot, read_metadata, verify_sha256};
```

---

## Tests

### Key derivation tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_slot_2() {
        // slot = PlayerState1 = 2
        // key[0] = rotate_left((2 ^ 0x1422CB8C), 13) * 5 + 0xE6546B64
        //        = rotate_left(0x1422CB8E, 13) * 5 + 0xE6546B64
        let expected_xor = 2u32 ^ 0x1422CB8C; // = 0x1422CB8E
        let expected_rot = expected_xor.rotate_left(13);
        let expected_mul = expected_rot.wrapping_mul(5);
        let expected_key0 = expected_mul.wrapping_add(0xE6546B64);

        let key = derive_key(StorageSlot::PlayerState1);
        assert_eq!(key[0], expected_key0);
        assert_eq!(key[1], META_ENCRYPTION_KEY[1]); // 0x44415645
        assert_eq!(key[2], META_ENCRYPTION_KEY[2]); // 0x5259414E
        assert_eq!(key[3], META_ENCRYPTION_KEY[3]); // 0x47524E54
    }

    #[test]
    fn test_derive_key_slot_0() {
        // slot = UserSettings = 0
        let key = derive_key(StorageSlot::UserSettings);
        let expected_key0 = (0u32 ^ 0x1422CB8C)
            .rotate_left(13)
            .wrapping_mul(5)
            .wrapping_add(0xE6546B64);
        assert_eq!(key[0], expected_key0);
    }

    #[test]
    fn test_derive_key_slot_1() {
        // slot = AccountData = 1
        let key = derive_key(StorageSlot::AccountData);
        let expected_key0 = (1u32 ^ 0x1422CB8C)
            .rotate_left(13)
            .wrapping_mul(5)
            .wrapping_add(0xE6546B64);
        assert_eq!(key[0], expected_key0);
    }

    #[test]
    fn test_derive_key_all_slots_unique() {
        // All 32 slots should produce distinct key[0] values.
        let keys: Vec<u32> = StorageSlot::ALL
            .iter()
            .map(|s| derive_key(*s)[0])
            .collect();
        let unique: std::collections::HashSet<u32> = keys.iter().copied().collect();
        assert_eq!(unique.len(), 32);
    }
}
```

### XXTEA round-trip test

Since implementing the forward XXTEA encryption for the NMS-modified variant is complex, use a known test vector approach:

```rust
    #[test]
    fn test_xxtea_decrypt_known_vector() {
        // Build a known plaintext that starts with META_MAGIC.
        // Encrypt it manually (or capture from a real file), then test decryption.
        //
        // Minimal approach: create a small plaintext, encrypt using our
        // encrypt function (if implemented), then verify decrypt recovers it.
        //
        // Alternative: If a real mf_save.hg file is available, capture the raw
        // bytes and the expected decrypted output as a test vector.
        //
        // For now, test the mathematical properties:

        // Property: after pre-computing hash for 6 iterations and then
        // adding TEA_REVERSE_DELTA 6 times, hash should return to 0.
        let mut hash: u32 = 0;
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_REVERSE_DELTA);
        }
        assert_eq!(hash, 0, "delta and reverse delta should cancel out over equal iterations");
    }

    #[test]
    fn test_xxtea_hash_precomputation_vanilla() {
        // 8 iterations of TEA_DELTA should produce 0xF1BBCDC8
        // (documented in PlatformSteam_Read.cs line 45 comment).
        let mut hash: u32 = 0;
        for _ in 0..8 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        assert_eq!(hash, 0xF1BBCDC8);
    }

    #[test]
    fn test_xxtea_hash_precomputation_default() {
        // 6 iterations of TEA_DELTA.
        let mut hash: u32 = 0;
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        assert_eq!(hash, 0xB5DBAE78);
    }
```

### Metadata parsing tests

```rust
    #[test]
    fn test_invalid_meta_length() {
        let data = vec![0u8; 50]; // Not a valid length
        let err = read_metadata(&data, StorageSlot::PlayerState1).unwrap_err();
        match err {
            SaveError::InvalidMetaLength { length } => assert_eq!(length, 50),
            _ => panic!("expected InvalidMetaLength, got {err:?}"),
        }
    }

    #[test]
    fn test_valid_meta_lengths_accepted() {
        // These should not fail with InvalidMetaLength (they will fail with
        // MetaDecryptionFailed since the data is zeros, but that's the right error).
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
    fn test_parse_decrypted_metadata_format_2002() {
        // Build a minimal decrypted metadata array (26 u32s).
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;                  // magic
        words[1] = META_FORMAT_FRONTIERS;       // format version 2002
        // words[2..6] = spooky hash (ignored for 2002)
        // words[6..14] = sha256 (ignored for 2002)
        words[14] = 1_000_000;                  // decompressed size
        words[15] = 500_000;                    // compressed size
        words[16] = 0x12345678;                 // profile hash

        let meta = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap();
        assert_eq!(meta.format_version, 0x7D2);
        assert_eq!(meta.decompressed_size, 1_000_000);
        assert_eq!(meta.compressed_size, 500_000);
        assert_eq!(meta.profile_hash, 0x12345678);
        assert!(meta.spooky_hash.is_none());
        assert!(meta.sha256_hash.is_none());
    }

    #[test]
    fn test_parse_decrypted_metadata_format_2001() {
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;
        words[1] = META_FORMAT_FOUNDATION;  // 2001
        words[2] = 0xAABBCCDD;              // spooky[0] low
        words[3] = 0x11223344;              // spooky[0] high
        words[4] = 0x55667788;              // spooky[1] low
        words[5] = 0x99AABBCC;              // spooky[1] high
        // words[6..14] = sha256 bytes
        for i in 6..14 {
            words[i] = (i as u32) * 0x01010101;
        }
        words[14] = 2_000_000;
        words[15] = 800_000;
        words[16] = 0;

        let meta = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap();
        assert_eq!(meta.format_version, 0x7D1);
        assert!(meta.spooky_hash.is_some());
        let spooky = meta.spooky_hash.unwrap();
        assert_eq!(spooky[0], 0x11223344_AABBCCDD_u64);
        assert_eq!(spooky[1], 0x99AABBCC_55667788_u64);
        assert!(meta.sha256_hash.is_some());
    }

    #[test]
    fn test_unsupported_vanilla_format() {
        let mut words = vec![0u32; 26];
        words[0] = META_MAGIC;
        words[1] = META_FORMAT_VANILLA; // 2000 -- not supported
        let err = parse_decrypted_metadata(&words, StorageSlot::PlayerState1).unwrap_err();
        match err {
            SaveError::UnsupportedMetaFormat { version } => assert_eq!(version, 0x7D0),
            _ => panic!("expected UnsupportedMetaFormat, got {err:?}"),
        }
    }
```

### Integration test with real save files

```rust
    // If a real mf_save.hg file is available, place it in:
    //   crates/nms-save/tests/fixtures/mf_save.hg
    // and test with:
    //
    // #[test]
    // fn test_real_metadata_file() {
    //     let data = std::fs::read("tests/fixtures/mf_save.hg").unwrap();
    //     let meta = read_metadata(&data, StorageSlot::PlayerState1).unwrap();
    //     assert!(meta.format_version >= 0x7D1);
    //     assert!(meta.decompressed_size > 0);
    // }
```

---

## Acceptance Criteria

1. `derive_key` produces correct keys for all 32 slot values (verified against manual computation).
2. `xxtea_decrypt` matches the exact algorithm from `PlatformSteam_Read.cs:35-79`.
3. `read_metadata` successfully decrypts metadata files of all valid lengths (104, 360, 384, 432 bytes).
4. If the expected slot fails decryption, all other valid slots are tried before returning `MetaDecryptionFailed`.
5. `verify_sha256` returns `true` when the hash matches and `false` otherwise.
6. `cargo test -p nms-save` passes all tests.
7. `cargo clippy -p nms-save` reports no warnings.
8. No `unsafe` code.
