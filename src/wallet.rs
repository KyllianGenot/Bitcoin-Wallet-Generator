use secp256k1::{Secp256k1, SecretKey, PublicKey};
use bitcoin_hashes::{sha256d, Hash, hash160};
use bitcoin::util::base58;
use crate::bip32::ExtendedPrivKey;

pub struct Wallet {
    private_key: String,
    public_key: String,
    address: String,
}

impl Wallet {
    pub fn from_seed(seed: &[u8]) -> Result<Self, &'static str> {
        let secp = Secp256k1::new();

        let master_key = ExtendedPrivKey::new(seed)?;
        
        let secret_key = SecretKey::from_slice(&master_key.private_key).map_err(|_| "Invalid private key")?;

        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let address = Self::generate_address(&public_key);

        Ok(Wallet {
            private_key: hex::encode(&master_key.private_key),
            public_key: hex::encode(&public_key.serialize()),
            address,
        })
    }

    fn generate_address(public_key: &PublicKey) -> String {
        let pubkey_hash = hash160::Hash::hash(&public_key.serialize());
        let mut payload = vec![0x00];
        payload.extend_from_slice(&pubkey_hash[..]);
        let checksum = &sha256d::Hash::hash(&payload)[..4];
        payload.extend_from_slice(checksum);
        base58::encode_slice(&payload)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_private_key(&self) -> &str {
        &self.private_key
    }

    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_seed() {
        let seed = [0u8; 10];
        let result = Wallet::from_seed(&seed);
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_generation() {
        let seed = [0u8; 64];
        let wallet = Wallet::from_seed(&seed).expect("Wallet generation failed");

        assert!(!wallet.get_private_key().is_empty());
        assert!(!wallet.get_public_key().is_empty());
        assert!(!wallet.get_address().is_empty());
    }
}
