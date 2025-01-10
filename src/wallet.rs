use secp256k1::{Secp256k1, SecretKey, PublicKey};
use bitcoin_hashes::{sha256d, Hash, hash160};
use bitcoin::util::base58;
use crate::bip32::ExtendedPrivKey;

// Wallet structure to store private key, public key, and address
pub struct Wallet {
    private_key: String, // Hex-encoded private key
    public_key: String,  // Hex-encoded public key
    address: String,     // Base58-encoded Bitcoin address
}

impl Wallet {
    // Creates a wallet from a seed
    pub fn from_seed(seed: &[u8]) -> Result<Self, &'static str> {
        let secp = Secp256k1::new(); // Create a new secp256k1 context

        // Generate the master extended private key from the seed
        let master_key = ExtendedPrivKey::new(seed)?;

        // Derive the secret key from the master private key
        let secret_key = SecretKey::from_slice(&master_key.private_key).map_err(|_| "Invalid private key")?;

        // Derive the public key from the secret key
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        // Generate the Bitcoin address from the public key
        let address = Self::generate_address(&public_key);

        Ok(Wallet {
            private_key: hex::encode(&master_key.private_key), // Encode private key as hex
            public_key: hex::encode(&public_key.serialize()),  // Encode public key as hex
            address,                                           // Store the generated address
        })
    }

    // Generates a Bitcoin address from a public key
    fn generate_address(public_key: &PublicKey) -> String {
        // Hash the public key using RIPEMD-160(SHA-256)
        let pubkey_hash = hash160::Hash::hash(&public_key.serialize());

        // Create the payload with version byte (0x00 for mainnet) and the public key hash
        let mut payload = vec![0x00];
        payload.extend_from_slice(&pubkey_hash[..]);

        // Calculate the checksum using SHA-256(SHA-256(payload))
        let checksum = &sha256d::Hash::hash(&payload)[..4];

        // Append the checksum to the payload
        payload.extend_from_slice(checksum);

        // Encode the payload in Base58 to get the final address
        base58::encode_slice(&payload)
    }

    // Returns the wallet's Bitcoin address
    pub fn get_address(&self) -> &str {
        &self.address
    }

    // Returns the wallet's private key (hex-encoded)
    pub fn get_private_key(&self) -> &str {
        &self.private_key
    }

    // Returns the wallet's public key (hex-encoded)
    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests wallet generation with an invalid seed
    #[test]
    fn test_invalid_seed() {
        let seed = [0u8; 10]; // Seed is too short
        let result = Wallet::from_seed(&seed);
        assert!(result.is_err()); // Expect an error
    }

    // Tests wallet generation with a valid seed
    #[test]
    fn test_wallet_generation() {
        let seed = [0u8; 64]; // Valid seed length
        let wallet = Wallet::from_seed(&seed).expect("Wallet generation failed");

        // Ensure private key, public key, and address are generated
        assert!(!wallet.get_private_key().is_empty());
        assert!(!wallet.get_public_key().is_empty());
        assert!(!wallet.get_address().is_empty());
    }
}