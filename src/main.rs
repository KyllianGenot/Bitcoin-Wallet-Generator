mod wallet;
mod bip39;
mod bip32;
mod web;

use std::io::{self, Write};
use std::thread;
use std::fmt;
use wallet::Wallet;
use bip39::{Mnemonic, Seed};
use bip32::ExtendedPrivKey;
use hex;
use serde_json::{json, Value};
use chrono::Utc;
use qrcode::QrCode;
use web::start_server;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create necessary directories if they don't exist
    create_directories().expect("Failed to create directories");

    loop {
        println!("\n=============================");
        println!("   Bitcoin Wallet Generator  ");
        println!("=============================");
        println!("1. Generate wallets");
        println!("2. Generate extended private key");
        println!("3. Derive child key");
        println!("4. Generate QR code for a wallet address");
        println!("5. Use Bitcoin Wallet Generator on web interface");
        println!("6. Exit");
        println!("=============================");

        print!("Please select an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        let choice: usize = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("\n❌ Invalid input. Please enter a valid number.");
                continue;
            }
        };

        match choice {
            1 => generate_wallets(),
            2 => generate_extended_priv_key(),
            3 => derive_child_key(),
            4 => generate_qr_code_for_address(),
            5 => {
                println!("\nStarting web interface...");
                rt.block_on(start_server());
            }
            6 => {
                println!("\n✅ Exiting... Thank you for using Bitcoin Wallet Generator!");
                break;
            }
            _ => println!("\n❌ Invalid option. Please select a valid number."),
        }
    }
}

fn create_directories() -> Result<(), std::io::Error> {
    let directories = ["data/wallets", "data/extended_keys", "data/child_keys", "data/qr_codes"];
    for dir in directories.iter() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

fn generate_wallets() {
    print!("\n🔢 How many wallets do you want to generate? ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let count: usize = match input.trim().parse() {
        Ok(num) if num > 0 => num,
        _ => {
            println!("\n❌ Invalid number. Please enter a positive integer.");
            return;
        }
    };

    let mut handles = Vec::new();

    for i in 0..count {
        let handle = thread::spawn(move || {
            let mnemonic = Mnemonic::generate(128);
            let seed = Seed::new(&mnemonic.to_string(), "");
            let wallet = Wallet::from_seed(seed.as_bytes()).unwrap();

            (
                i,
                mnemonic.to_string(),
                wallet.get_address().to_string(),
                wallet.get_public_key().to_string(),
                wallet.get_private_key().to_string(),
            )
        });
        handles.push(handle);
    }

    let mut wallets: Vec<Value> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();

    for handle in handles {
        let (index, mnemonic, address, public_key, private_key) = handle.join().expect("Thread panicked");
        println!("\n🚀 Wallet #{}:", index + 1);
        println!("  Mnemonic     : {}", mnemonic);
        println!("  Address      : {}", address);
        println!("  Public Key   : {}", public_key);
        println!("  Private Key  : {}", private_key);

        let wallet_json = json!({
            "Mnemonic": mnemonic,
            "Address": address,
            "PublicKey": public_key,
            "PrivateKey": private_key,
            "GeneratedAt": Utc::now().to_rfc3339(),
        });

        wallets.push(wallet_json);
        addresses.push(address); // Stocker les adresses pour une sélection ultérieure
    }

    // Une fois les portefeuilles générés, demander pour quels QR codes générer
    println!("\n📷 Which wallets would you like to generate QR codes for?");
    println!("   Enter 'all' for all wallets, 'none' for none, or a comma-separated list of indexes (e.g., 1,3,5): ");

    let mut qr_choice = String::new();
    io::stdin().read_line(&mut qr_choice).expect("Failed to read input");
    let qr_choice = qr_choice.trim().to_lowercase();

    match qr_choice.as_str() {
        "all" => {
            for (i, address) in addresses.iter().enumerate() {
                let file_name = format!("data/qr_codes/{}.svg", address);
                if let Err(err) = generate_qr_code(address, &file_name) {
                    println!("\n❌ Failed to generate QR code for Wallet #{}: {}", i + 1, err);
                }
            }
        }
        "none" => {
            println!("\n📝 No QR codes will be generated.");
        }
        _ => {
            // Gérer une liste d'index spécifiés
            let indexes: Vec<usize> = qr_choice
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();

            for &index in &indexes {
                if index > 0 && index <= addresses.len() {
                    let file_name = format!("data/qr_codes/{}.svg", addresses[index - 1]);
                    if let Err(err) = generate_qr_code(&addresses[index - 1], &file_name) {
                        println!("\n❌ Failed to generate QR code for Wallet #{}: {}", index, err);
                    }
                } else {
                    println!("\n❌ Invalid index: {}. Skipping...", index);
                }
            }
        }
    }

    print!("\n💾 Do you want to save these wallets to a file? (y/n): ");
    io::stdout().flush().unwrap();

    let mut save_choice = String::new();
    io::stdin().read_line(&mut save_choice).expect("Failed to read input");

    if save_choice.trim().eq_ignore_ascii_case("y") {
        if let Err(err) = save_wallets_to_file(&wallets) {
            println!("\n❌ Failed to save wallets: {}", err);
        } else {
            println!("\n✅ Wallets saved successfully!");
        }
    } else {
        println!("\n📝 Wallets were not saved.");
    }
}

fn save_wallets_to_file(wallets: &[Value]) -> Result<(), std::io::Error> {
    let file_path = "data/wallets/wallets.json";

    let mut existing_wallets: Vec<Value> = if let Ok(file) = std::fs::read_to_string(file_path) {
        serde_json::from_str(&file).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };
    existing_wallets.extend_from_slice(wallets);

    let json_data = serde_json::to_string_pretty(&existing_wallets)?;
    std::fs::write(file_path, json_data)?;

    Ok(())
}

fn save_to_file(data: &Value, file_name: &str) -> Result<(), std::io::Error> {
    let mut existing_data: Vec<Value> = if let Ok(file_content) = std::fs::read_to_string(file_name) {
        match serde_json::from_str(&file_content) {
            Ok(parsed) => parsed,
            Err(_) => {
                println!("\n⚠️ Le fichier JSON existant est corrompu. Il sera réinitialisé.");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    existing_data.push(data.clone());

    let json_data = serde_json::to_string_pretty(&existing_data)?;
    std::fs::write(file_name, json_data)?;

    Ok(())
}

fn generate_extended_priv_key() {
    print!("\n🔑 Enter a seed (hex-encoded): ");
    io::stdout().flush().unwrap();

    let mut seed_hex = String::new();
    io::stdin().read_line(&mut seed_hex).expect("Failed to read input");
    let seed = match hex::decode(seed_hex.trim()) {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("\n❌ Invalid seed format. Please enter a valid hex string.");
            return;
        }
    };

    match ExtendedPrivKey::new(&seed) {
        Ok(ext_key) => {
            println!("\n✅ Extended Private Key generated:");
            println!("  🔒 Private Key: {}", hex::encode(ext_key.private_key));
            println!("  🔗 Chain Code: {}", hex::encode(ext_key.chain_code));

            let ext_key_json = json!({
                "PrivateKey": hex::encode(ext_key.private_key),
                "GeneratedAt": Utc::now().to_rfc3339(),
                "ChainCode": hex::encode(ext_key.chain_code),
            });

            print!("\n💾 Do you want to save this extended private key to a file? (y/n): ");
            io::stdout().flush().unwrap();

            let mut save_choice = String::new();
            io::stdin().read_line(&mut save_choice).expect("Failed to read input");

            if save_choice.trim().eq_ignore_ascii_case("y") {
                if let Err(err) = save_to_file(&ext_key_json, "data/extended_keys/extended_keys.json") {
                    println!("\n❌ Failed to save extended private key: {}", err);
                } else {
                    println!("\n✅ Extended private key saved successfully!");
                }
            }
        }
        Err(e) => println!("\n❌ Error: {}", e),
    }
}

fn derive_child_key() {
    print!("\n🔑 Enter a parent private key (hex-encoded, 64 characters): ");
    io::stdout().flush().unwrap();

    let mut parent_key_hex = String::new();
    io::stdin().read_line(&mut parent_key_hex).expect("Failed to read input");
    let parent_key = match hex::decode(parent_key_hex.trim()) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes);
            array
        }
        _ => {
            println!("\n❌ Invalid private key. Must be a 64-character hex string.");
            return;
        }
    };

    print!("\n🔗 Enter a chain code (hex-encoded, 64 characters): ");
    io::stdout().flush().unwrap();

    let mut chain_code_hex = String::new();
    io::stdin().read_line(&mut chain_code_hex).expect("Failed to read input");
    let chain_code = match hex::decode(chain_code_hex.trim()) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes);
            array
        }
        _ => {
            println!("\n❌ Invalid chain code. Must be a 64-character hex string.");
            return;
        }
    };

    print!("\n🔢 Enter an index for the child key (e.g., 0, 1, ...): ");
    io::stdout().flush().unwrap();

    let mut index_input = String::new();
    io::stdin().read_line(&mut index_input).expect("Failed to read input");
    let index: u32 = match index_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("\n❌ Invalid index. Please enter a valid number.");
            return;
        }
    };

    let parent_ext_key = ExtendedPrivKey {
        private_key: parent_key,
        chain_code,
    };

    match parent_ext_key.derive_child_key(index) {
        Ok(child_key) => {
            println!("\n✅ Child Key derived:");
            println!("  🔒 Private Key: {}", hex::encode(child_key.private_key));
            println!("  🔗 Chain Code: {}", hex::encode(child_key.chain_code));

            let child_key_json = json!({
                "PrivateKey": hex::encode(child_key.private_key),
                "ChainCode": hex::encode(child_key.chain_code),
                "Index": index,
                "DerivationPath": format!("m/44'/0'/0'/0/{}", index),
                "GeneratedAt": Utc::now().to_rfc3339(),
            });

            print!("\n💾 Do you want to save this child key to a file? (y/n): ");
            io::stdout().flush().unwrap();

            let mut save_choice = String::new();
            io::stdin().read_line(&mut save_choice).expect("Failed to read input");

            if save_choice.trim().eq_ignore_ascii_case("y") {
                if let Err(err) = save_to_file(&child_key_json, "data/child_keys/child_keys.json") {
                    println!("\n❌ Failed to save child key: {}", err);
                } else {
                    println!("\n✅ Child key saved successfully!");
                }
            }
        }
        Err(e) => println!("\n❌ Error: {}", e),
    }
}

#[derive(Debug)]
enum MyError {
    QrError(qrcode::types::QrError),
    IoError(std::io::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::QrError(e) => write!(f, "QR Error: {}", e),
            MyError::IoError(e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl std::error::Error for MyError {}

impl From<qrcode::types::QrError> for MyError {
    fn from(err: qrcode::types::QrError) -> MyError {
        MyError::QrError(err)
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::IoError(err)
    }
}

fn generate_qr_code(data: &str, file_name: &str) -> Result<(), MyError> {
    let code = QrCode::new(data)?;

    let image = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();

    std::fs::write(file_name, image)?;

    println!("✅ QR Code saved as {}", file_name);

    Ok(())
}

fn generate_qr_code_for_address() {
    print!("\n🔑 Enter the wallet address to generate the QR code: ");
    io::stdout().flush().unwrap();

    let mut wallet_address = String::new();
    io::stdin().read_line(&mut wallet_address).expect("Failed to read input");
    let wallet_address = wallet_address.trim();

    if wallet_address.is_empty() {
        println!("\n❌ Invalid input. The wallet address cannot be empty.");
        return;
    }

    let file_name = format!("data/qr_codes/{}.svg", wallet_address);

    if let Err(err) = generate_qr_code(wallet_address, &file_name) {
        println!("\n❌ Failed to generate QR code: {}", err);
    } else {
        println!("\n✅ QR code generated successfully: {}", file_name);
    }
}