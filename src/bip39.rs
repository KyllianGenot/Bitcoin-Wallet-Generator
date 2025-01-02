use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use hmac::Hmac;
use pbkdf2::pbkdf2;

pub struct Mnemonic {
    phrase: String,
}

impl Mnemonic {
    pub fn generate(bits: usize) -> Self {
        if bits % 32 != 0 || bits < 128 || bits > 256 {
            panic!("Entropy must be a multiple of 32 and between 128 and 256 bits.");
        }

        let mut rng = rand::thread_rng();
        let entropy: Vec<u8> = (0..bits / 8).map(|_| rng.gen()).collect();

        let checksum = Self::calculate_checksum(&entropy, bits);

        let mut binary = String::new();
        for byte in &entropy {
            binary.push_str(&format!("{:08b}", byte));
        }
        binary.push_str(&checksum);

        let chunks: Vec<&str> = binary.as_bytes()
            .chunks(11)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();

        let wordlist = Self::load_wordlist("src/english.txt");
        let words: Vec<String> = chunks.iter().map(|chunk| {
            let index = usize::from_str_radix(chunk, 2).unwrap();
            wordlist[index].clone()
        }).collect();

        Mnemonic {
            phrase: words.join(" "),
        }
    }

    pub fn to_string(&self) -> String {
        self.phrase.clone()
    }

    fn load_wordlist(filepath: &str) -> Vec<String> {
        let content = fs::read_to_string(filepath).expect("Failed to load wordlist.");
        content.lines().map(String::from).collect()
    }

    fn calculate_checksum(entropy: &[u8], bits: usize) -> String {
        let hash = Sha256::digest(entropy);
        let checksum_bits = bits / 32;
        let checksum_binary = format!("{:08b}", hash[0]);
        checksum_binary[..checksum_bits].to_string()
    }
}

pub struct Seed {
    data: Vec<u8>,
}

impl Seed {
    pub fn new(mnemonic: &str, passphrase: &str) -> Self {
        let salt = format!("mnemonic{}", passphrase);
        let mut seed = vec![0u8; 64];
        pbkdf2::<Hmac<Sha512>>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed);
        Seed { data: seed }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}