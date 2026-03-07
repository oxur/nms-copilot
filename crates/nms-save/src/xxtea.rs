//! XXTEA encryption/decryption for NMS metadata files.
//!
//! Implements the modified XXTEA variant used by No Man's Sky for
//! `mf_save*.hg` metadata encryption.

use crate::metadata::StorageSlot;

/// XXTEA key derivation: XOR constant.
const KEY_XOR: u32 = 0x1422CB8C;

/// XXTEA key derivation: rotate-left amount (bits).
const KEY_ROTATE: u32 = 13;

/// XXTEA key derivation: multiply constant.
const KEY_MULTIPLY: u32 = 5;

/// XXTEA key derivation: add constant.
const KEY_ADD: u32 = 0xE6546B64;

/// XXTEA delta (golden ratio constant).
pub(crate) const TEA_DELTA: u32 = 0x9E3779B9;

/// XXTEA reverse delta (wrapping complement of TEA_DELTA).
pub(crate) const TEA_REVERSE_DELTA: u32 = 0x61C88647;

/// Base encryption key: "NAESEVADNAYRTNRG" as 4 little-endian u32s.
pub(crate) const META_ENCRYPTION_KEY: [u32; 4] = [
    0x5345414E, // "NAES" as LE u32
    0x44415645, // "EVAD" as LE u32
    0x5259414E, // "NAYR" as LE u32
    0x47524E54, // "TNRG" as LE u32
];

/// Derive the 4-element XXTEA key from a storage slot.
///
/// `key[0]` is computed from the slot value:
///   `key[0] = rotate_left((slot_value ^ 0x1422CB8C), 13) * 5 + 0xE6546B64`
///
/// `key[1..3]` are the fixed values from `META_ENCRYPTION_KEY[1..3]`.
pub fn derive_key(slot: StorageSlot) -> [u32; 4] {
    let slot_value = slot as u32;
    let key0 = (slot_value ^ KEY_XOR)
        .rotate_left(KEY_ROTATE)
        .wrapping_mul(KEY_MULTIPLY)
        .wrapping_add(KEY_ADD);
    [
        key0,
        META_ENCRYPTION_KEY[1],
        META_ENCRYPTION_KEY[2],
        META_ENCRYPTION_KEY[3],
    ]
}

/// XXTEA decrypt a u32 slice in place.
///
/// `data` is the metadata as a mutable slice of little-endian u32 values.
/// `key` is the 4-element key from [`derive_key`].
/// `iterations` is 8 for vanilla (104-byte) meta, 6 for all others.
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
            data[j] = data[j].wrapping_sub(t1.wrapping_add(t2) ^ t3.wrapping_add(t4));
            current = data[j];
        }

        // Process element 0 (wraps around to last element).
        let prev = data[last];
        let t1 = (current >> 3) ^ (prev << 4);
        let t2 = current.wrapping_mul(4) ^ (prev >> 5);
        let t3 = prev ^ key[key_index];
        let t4 = current ^ hash;
        data[0] = data[0].wrapping_sub(t1.wrapping_add(t2) ^ t3.wrapping_add(t4));

        hash = hash.wrapping_add(TEA_REVERSE_DELTA);
    }
}

/// XXTEA encrypt a u32 slice in place.
///
/// Forward direction of the NMS-modified XXTEA. Used for testing round-trips.
pub fn xxtea_encrypt(data: &mut [u32], key: &[u32; 4], iterations: usize) {
    let last = data.len() - 1;
    let mut hash: u32 = 0;

    for _ in 0..iterations {
        hash = hash.wrapping_add(TEA_DELTA);
        let key_index = (hash >> 2 & 3) as usize;

        // Process element 0 (wraps around to last element).
        let next = data[1];
        let prev = data[last];
        let t1 = (next >> 3) ^ (prev << 4);
        let t2 = next.wrapping_mul(4) ^ (prev >> 5);
        let t3 = prev ^ key[key_index];
        let t4 = next ^ hash;
        data[0] = data[0].wrapping_add(t1.wrapping_add(t2) ^ t3.wrapping_add(t4));

        // Process elements 1..last (forwards).
        for j in 1..=last {
            let next = data[if j == last { 0 } else { j + 1 }];
            let prev = data[j - 1];
            let t1 = (next >> 3) ^ (prev << 4);
            let t2 = next.wrapping_mul(4) ^ (prev >> 5);
            let t3 = prev ^ key[(j & 3) ^ key_index];
            let t4 = next ^ hash;
            data[j] = data[j].wrapping_add(t1.wrapping_add(t2) ^ t3.wrapping_add(t4));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_slot_2() {
        let expected_key0 = (2u32 ^ KEY_XOR)
            .rotate_left(KEY_ROTATE)
            .wrapping_mul(KEY_MULTIPLY)
            .wrapping_add(KEY_ADD);

        let key = derive_key(StorageSlot::PlayerState1);
        assert_eq!(key[0], expected_key0);
        assert_eq!(key[1], META_ENCRYPTION_KEY[1]);
        assert_eq!(key[2], META_ENCRYPTION_KEY[2]);
        assert_eq!(key[3], META_ENCRYPTION_KEY[3]);
    }

    #[test]
    fn derive_key_slot_0() {
        let key = derive_key(StorageSlot::UserSettings);
        let expected_key0 = (0u32 ^ KEY_XOR)
            .rotate_left(KEY_ROTATE)
            .wrapping_mul(KEY_MULTIPLY)
            .wrapping_add(KEY_ADD);
        assert_eq!(key[0], expected_key0);
    }

    #[test]
    fn derive_key_slot_1() {
        let key = derive_key(StorageSlot::AccountData);
        let expected_key0 = (1u32 ^ KEY_XOR)
            .rotate_left(KEY_ROTATE)
            .wrapping_mul(KEY_MULTIPLY)
            .wrapping_add(KEY_ADD);
        assert_eq!(key[0], expected_key0);
    }

    #[test]
    fn derive_key_all_slots_unique() {
        let keys: Vec<u32> = StorageSlot::ALL.iter().map(|s| derive_key(*s)[0]).collect();
        let unique: std::collections::HashSet<u32> = keys.iter().copied().collect();
        assert_eq!(unique.len(), 32);
    }

    #[test]
    fn hash_precomputation_vanilla() {
        let mut hash: u32 = 0;
        for _ in 0..8 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        assert_eq!(hash, 0xF1BBCDC8);
    }

    #[test]
    fn hash_precomputation_default() {
        let mut hash: u32 = 0;
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        assert_eq!(hash, 0xB54CDA56);
    }

    #[test]
    fn delta_reverse_delta_cancel() {
        let mut hash: u32 = 0;
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_DELTA);
        }
        for _ in 0..6 {
            hash = hash.wrapping_add(TEA_REVERSE_DELTA);
        }
        assert_eq!(hash, 0);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_6_rounds() {
        let original = [0xEEEEEEBEu32, 0x7D2, 42, 100, 200, 300, 0, 0, 0, 0];
        let key = derive_key(StorageSlot::PlayerState1);

        let mut data = original;
        xxtea_encrypt(&mut data, &key, 6);
        // Encrypted data should differ from original
        assert_ne!(data, original);

        xxtea_decrypt(&mut data, &key, 6);
        assert_eq!(data, original);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_8_rounds() {
        let original = [0xEEEEEEBEu32, 0x7D1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let key = derive_key(StorageSlot::PlayerState5);

        let mut data = original;
        xxtea_encrypt(&mut data, &key, 8);
        assert_ne!(data, original);

        xxtea_decrypt(&mut data, &key, 8);
        assert_eq!(data, original);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_all_slots() {
        let original = [0xEEEEEEBEu32, 0x7D3, 99, 88, 77, 66, 55, 44];
        for slot in &StorageSlot::ALL {
            let key = derive_key(*slot);
            let mut data = original;
            xxtea_encrypt(&mut data, &key, 6);
            xxtea_decrypt(&mut data, &key, 6);
            assert_eq!(data, original, "roundtrip failed for slot {slot:?}");
        }
    }
}
