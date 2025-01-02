use hmac::{Hmac, Mac};
use sha2::Sha512;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use num_bigint::BigUint;
use num_traits::Num;
use std::convert::TryInto;

pub struct ExtendedPrivKey {
    pub private_key: [u8; 32],
    pub chain_code: [u8; 32],
}

impl ExtendedPrivKey {
    pub fn new(seed: &[u8]) -> Result<Self, &'static str> {
        if seed.len() < 16 || seed.len() > 64 {
            return Err("Seed length must be between 16 and 64 bytes");
        }

        let mut hmac = Hmac::<Sha512>::new_from_slice(b"Bitcoin seed")
            .map_err(|_| "HMAC initialization failed")?;
        hmac.update(seed);
        let result = hmac.finalize().into_bytes();
        let (private_key, chain_code) = result.split_at(32);

        let private_key: [u8; 32] = private_key
            .try_into()
            .map_err(|_| "Invalid private key length")?;
        let chain_code: [u8; 32] = chain_code
            .try_into()
            .map_err(|_| "Invalid chain code length")?;

        if !Self::is_valid_private_key(&private_key) {
            return Err("Invalid private key: out of range");
        }

        Ok(ExtendedPrivKey {
            private_key,
            chain_code,
        })
    }

    fn is_valid_private_key(key: &[u8; 32]) -> bool {
        SecretKey::from_slice(key).is_ok()
    }

    pub fn derive_child_key(&self, index: u32) -> Result<Self, &'static str> {
        let mut data = Vec::new();

        if index >= 0x80000000 {
            data.push(0);
            data.extend_from_slice(&self.private_key);
        } else {
            let secp = Secp256k1::new();
            let secret_key = SecretKey::from_slice(&self.private_key)
                .map_err(|_| "Invalid private key")?;
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            data.extend_from_slice(&public_key.serialize());
        }

        data.extend_from_slice(&index.to_be_bytes());

        let mut hmac = Hmac::<Sha512>::new_from_slice(&self.chain_code)
            .map_err(|_| "HMAC initialization failed")?;
        hmac.update(&data);
        let result = hmac.finalize().into_bytes();
        let (child_key, child_chain_code) = result.split_at(32);

        let derived_key = add_scalars(
            &self.private_key,
            &child_key.try_into().map_err(|_| "Invalid child key length")?,
        )?;

        Ok(ExtendedPrivKey {
            private_key: derived_key,
            chain_code: child_chain_code.try_into()
                .map_err(|_| "Invalid chain code length")?,
        })
    }
}

fn add_scalars(a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], &'static str> {
    const GROUP_ORDER_HEX: &str = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141";

    let a_int = BigUint::from_bytes_be(a);
    let b_int = BigUint::from_bytes_be(b);

    let n = BigUint::from_str_radix(GROUP_ORDER_HEX, 16)
        .map_err(|_| "Failed to parse group order")?;

    let sum = (a_int + b_int) % &n;

    let sum_bytes = sum.to_bytes_be();
    if sum_bytes.len() > 32 {
        return Err("Scalar addition overflow");
    }

    let mut sum_padded = [0u8; 32];
    sum_padded[32 - sum_bytes.len()..].copy_from_slice(&sum_bytes);
    Ok(sum_padded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_with_valid_seed() {
        let seed = [0u8; 64];
        let key = ExtendedPrivKey::new(&seed).expect("Failed to create key");
        assert_eq!(key.private_key.len(), 32);
        assert_eq!(key.chain_code.len(), 32);
    }

    #[test]
    fn test_key_generation_with_invalid_seed_length() {
        let seed = [0u8; 10];
        let result = ExtendedPrivKey::new(&seed);
        assert!(result.is_err());
    }

    #[test]
    fn test_child_key_derivation() {
        let seed = [0u8; 64];
        let parent_key = ExtendedPrivKey::new(&seed).expect("Failed to create key");
        let child_key = parent_key.derive_child_key(0).expect("Failed to derive child key");

        assert_ne!(parent_key.private_key, child_key.private_key);
        assert_ne!(parent_key.chain_code, child_key.chain_code);
    }

    #[test]
    fn test_hardened_key_derivation() {
        let seed = [0u8; 64];
        let parent_key = ExtendedPrivKey::new(&seed).expect("Failed to create key");
        let child_key = parent_key.derive_child_key(0x80000000)
            .expect("Failed to derive hardened child key");

        assert_ne!(parent_key.private_key, child_key.private_key);
        assert_ne!(parent_key.chain_code, child_key.chain_code);
    }

    #[test]
    fn test_add_scalars() {
        let a = [
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8,
            0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
        ];
        let b = [
            0x20, 0x1F, 0x1E, 0x1D, 0x1C, 0x1B, 0x1A, 0x19,
            0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11,
            0x10, 0xF, 0xE, 0xD, 0xC, 0xB, 0xA, 0x9,
            0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1,
        ];

        let expected = [
            0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21,
            0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21,
            0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21,
            0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21, 0x21,
        ];

        let result = add_scalars(&a, &b).expect("Failed to add scalars");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_scalars_overflow() {
        let a = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
            0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
        ];
        let b = [0u8; 32];

        let expected = [0u8; 32];

        let result = add_scalars(&a, &b).expect("Failed to add scalars");
        assert_eq!(result, expected);
    }
}
