use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use hmac::Hmac;
use pbkdf2::pbkdf2;

// Mnemonic structure for generating and storing mnemonic phrases
pub struct Mnemonic {
    phrase: String, // Stores the mnemonic phrase
}

impl Mnemonic {
    // Generates a mnemonic phrase from random entropy
    pub fn generate(bits: usize) -> Self {
        if bits % 32 != 0 || bits < 128 || bits > 256 {
            panic!("Entropy must be a multiple of 32 and between 128 and 256 bits.");
        }

        // Generate random entropy
        let mut rng = rand::thread_rng();
        let entropy: Vec<u8> = (0..bits / 8).map(|_| rng.gen()).collect();

        // Calculate checksum for the entropy
        let checksum = Self::calculate_checksum(&entropy, bits);

        // Convert entropy and checksum to binary string
        let mut binary = String::new();
        for byte in &entropy {
            binary.push_str(&format!("{:08b}", byte));
        }
        binary.push_str(&checksum);

        // Split binary string into 11-bit chunks
        let chunks: Vec<&str> = binary.as_bytes()
            .chunks(11)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();

        // Load wordlist and map chunks to words
        let wordlist = Self::load_wordlist("src/english.txt");
        let words: Vec<String> = chunks.iter().map(|chunk| {
            let index = usize::from_str_radix(chunk, 2).unwrap();
            wordlist[index].clone()
        }).collect();

        Mnemonic {
            phrase: words.join(" "), // Join words to form mnemonic phrase
        }
    }

    // Returns the mnemonic phrase as a string
    pub fn to_string(&self) -> String {
        self.phrase.clone()
    }

    // Loads the wordlist from a file
    fn load_wordlist(filepath: &str) -> Vec<String> {
        let content = fs::read_to_string(filepath).expect("Failed to load wordlist.");
        content.lines().map(String::from).collect()
    }

    // Calculates the checksum for the entropy
    fn calculate_checksum(entropy: &[u8], bits: usize) -> String {
        let hash = Sha256::digest(entropy); // SHA-256 hash of entropy
        let checksum_bits = bits / 32; // Number of checksum bits
        let checksum_binary = format!("{:08b}", hash[0]); // First byte of hash as binary
        checksum_binary[..checksum_bits].to_string() // Truncate to required bits
    }
}

// Seed structure for deriving a seed from a mnemonic phrase
pub struct Seed {
    data: Vec<u8>, // Stores the seed bytes
}

impl Seed {
    // Derives a seed from a mnemonic phrase and passphrase using PBKDF2
    pub fn new(mnemonic: &str, passphrase: &str) -> Self {
        let salt = format!("mnemonic{}", passphrase); // Salt for PBKDF2
        let mut seed = vec![0u8; 64]; // 64-byte seed
        pbkdf2::<Hmac<Sha512>>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed); // PBKDF2 with HMAC-SHA512
        Seed { data: seed }
    }

    // Returns the seed as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}