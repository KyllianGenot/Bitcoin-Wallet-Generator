use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use crate::wallet::Wallet;
use crate::bip39::{Mnemonic, Seed};
use crate::bip32::ExtendedPrivKey;
use serde_json::{json, Value};
use chrono::Utc;
use qrcode::QrCode;
use std::fs;
use std::env;

// Lancement du serveur
pub async fn start_server() {
    // Configuration des routes
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/generate_wallets", get(generate_wallets_form).post(generate_wallets))
        .route("/extended_priv_key", get(extended_priv_key_form).post(generate_extended_priv_key))
        .route("/derive_child_key", get(derive_child_key_form).post(derive_child_key))
        .route("/qr_code", get(qr_code_form).post(generate_qr_code_web))
        .route("/save_all_wallets", post(save_all_wallets))
        .route("/save_all_qr_codes", post(save_all_qr_codes))
        .route("/save_extended_priv_keys", post(save_extended_priv_keys))
        .route("/save_child_keys", post(save_child_keys));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✅ Server running at http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Page principale
async fn landing_page() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Bitcoin Wallet Generator</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg text-center">
            <h1 class="text-3xl font-bold mb-6">Welcome to the Bitcoin Wallet Generator</h1>
            <div class="space-y-4">
                <a href="/generate_wallets" class="block bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate Wallets</a>
                <a href="/extended_priv_key" class="block bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate Extended Private Key</a>
                <a href="/derive_child_key" class="block bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Derive Child Key</a>
                <a href="/qr_code" class="block bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate QR Code</a>
            </div>
        </div>
    </body>
    </html>
    "#)
}

// Formulaire pour générer des wallets
async fn generate_wallets_form() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Generate Wallets</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg">
            <h1 class="text-3xl font-bold mb-6">Generate Wallets</h1>
            <form method="post" action="/generate_wallets" class="space-y-4">
                <div>
                    <label for="count" class="block text-sm font-medium text-gray-700">Number of wallets:</label>
                    <input type="number" id="count" name="count" min="1" max="10" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <button type="submit" class="w-full bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate</button>
            </form>
        </div>
    </body>
    </html>
    "#)
}

// Route pour gérer la génération de wallets
#[derive(Deserialize)]
struct WalletRequest {
    count: usize,
}

async fn generate_wallets(Form(input): Form<WalletRequest>) -> impl IntoResponse {
    let mut wallets = Vec::new();
    for _ in 0..input.count {
        let mnemonic = Mnemonic::generate(128);
        let seed = Seed::new(&mnemonic.to_string(), "");
        let wallet = Wallet::from_seed(seed.as_bytes()).unwrap();
        let wallet_data = json!({
            "Mnemonic": mnemonic.to_string(),
            "Address": wallet.get_address(),
            "PublicKey": wallet.get_public_key(),
            "PrivateKey": wallet.get_private_key(),
            "GeneratedAt": Utc::now().to_rfc3339(),
        });

        wallets.push(wallet_data);
    }

    let wallets_json = serde_json::to_string(&wallets).unwrap();

    Html(format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Generated Wallets</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
        <script>
            function saveAllWallets() {{
                fetch('/save_all_wallets', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/x-www-form-urlencoded',
                    }},
                    body: new URLSearchParams({{
                        wallets: JSON.stringify({}),
                    }}),
                }})
                .then(response => response.text())
                .then(message => {{
                    alert(message);
                }})
                .catch(error => {{
                    alert('Failed to save wallets: ' + error);
                }});
            }}

            function saveAllQrCodes() {{
                fetch('/save_all_qr_codes', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/x-www-form-urlencoded',
                    }},
                    body: new URLSearchParams({{
                        wallets: JSON.stringify({}),
                    }}),
                }})
                .then(response => response.text())
                .then(message => {{
                    alert(message);
                }})
                .catch(error => {{
                    alert('Failed to save QR codes: ' + error);
                }});
            }}
        </script>
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg">
            <h1 class="text-3xl font-bold mb-6">Generated Wallets</h1>
            {}
            <div class="flex space-x-4 mt-6">
                <button onclick="saveAllWallets()" class="bg-green-500 text-white py-2 px-4 rounded hover:bg-green-600">Save All Wallets</button>
                <button onclick="saveAllQrCodes()" class="bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Save All QR Codes</button>
                <a href="/" class="bg-gray-500 text-white py-2 px-4 rounded hover:bg-gray-600">Back to Home</a>
            </div>
        </div>
    </body>
    </html>
    "#,
        wallets_json,
        wallets_json,
        wallets.iter().map(|wallet| {
            format!(
                r#"
                <div class='mb-4 p-4 bg-gray-50 rounded-lg'>
                    <p class='text-sm text-gray-600'>Mnemonic: <span class='font-medium text-gray-800'>{}</span></p>
                    <p class='text-sm text-gray-600'>Address: <span class='font-medium text-gray-800'>{}</span></p>
                    <p class='text-sm text-gray-600'>Public Key: <span class='font-medium text-gray-800'>{}</span></p>
                    <p class='text-sm text-gray-600'>Private Key: <span class='font-medium text-gray-800'>{}</span></p>
                </div>
                "#,
                wallet["Mnemonic"],
                wallet["Address"],
                wallet["PublicKey"],
                wallet["PrivateKey"]
            )
        }).collect::<Vec<_>>().join("")
    ))
}

// Route pour sauvegarder tous les wallets
#[derive(Deserialize)]
struct SaveAllWalletsRequest {
    wallets: String,
}

async fn save_all_wallets(Form(input): Form<SaveAllWalletsRequest>) -> impl IntoResponse {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join("data/wallets/wallets.json");

    let wallets_data: Vec<Value> = match serde_json::from_str(&input.wallets) {
        Ok(data) => data,
        Err(_) => return "Failed to parse wallets data.".to_string(),
    };

    let mut existing_wallets: Vec<Value> = match fs::read_to_string(&file_path) {
        Ok(file) => serde_json::from_str(&file).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };
    existing_wallets.extend(wallets_data);

    match fs::write(&file_path, serde_json::to_string_pretty(&existing_wallets).unwrap()) {
        Ok(_) => "All wallets saved successfully!".to_string(),
        Err(_) => "Failed to save wallets.".to_string(),
    }
}

// Route pour sauvegarder tous les QR codes
#[derive(Deserialize)]
struct SaveAllQrCodesRequest {
    wallets: String,
}

async fn save_all_qr_codes(Form(input): Form<SaveAllQrCodesRequest>) -> impl IntoResponse {
    let wallets_data: Vec<Value> = match serde_json::from_str(&input.wallets) {
        Ok(data) => data,
        Err(_) => return "Failed to parse wallets data.".to_string(),
    };

    for wallet in wallets_data.iter() {
        let address = wallet["Address"].as_str().unwrap();
        let file_name = format!("data/qr_codes/{}.svg", address);
        let file_path = env::current_dir().unwrap().join(&file_name);
        match generate_qr_code(address, &file_path.to_string_lossy()) {
            Ok(_) => (),
            Err(_) => return format!("Failed to generate QR code for wallet {}", address).to_string(),
        }
    }
    "All QR codes saved successfully!".to_string()
}

// Formulaire pour générer une clé privée étendue
async fn extended_priv_key_form() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Generate Extended Private Key</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg">
            <h1 class="text-3xl font-bold mb-6">Generate Extended Private Key</h1>
            <form method="post" action="/extended_priv_key" class="space-y-4">
                <div>
                    <label for="seed" class="block text-sm font-medium text-gray-700">Seed (hex):</label>
                    <input type="text" id="seed" name="seed" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <button type="submit" class="w-full bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate</button>
            </form>
        </div>
    </body>
    </html>
    "#)
}

// Route pour gérer la génération d'une clé privée étendue
#[derive(Deserialize)]
struct ExtendedPrivKeyRequest {
    seed: String,
}

async fn generate_extended_priv_key(Form(input): Form<ExtendedPrivKeyRequest>) -> impl IntoResponse {
    match hex::decode(&input.seed) {
        Ok(seed) => match ExtendedPrivKey::new(&seed) {
            Ok(key) => {
                let ext_key_json = json!({
                    "PrivateKey": hex::encode(key.private_key),
                    "ChainCode": hex::encode(key.chain_code),
                    "GeneratedAt": Utc::now().to_rfc3339(),
                });

                Html(format!(
                    r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Extended Private Key</title>
                    <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
                    <script>
                        function saveExtendedPrivKeys() {{
                            fetch('/save_extended_priv_keys', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/x-www-form-urlencoded',
                                }},
                                body: new URLSearchParams({{
                                    ext_keys: JSON.stringify([{}]),
                                }}),
                            }})
                            .then(response => response.text())
                            .then(message => {{
                                alert(message);
                            }})
                            .catch(error => {{
                                alert('Failed to save extended private key: ' + error);
                            }});
                        }}
                    </script>
                </head>
                <body class="bg-gray-100 flex items-center justify-center h-screen">
                    <div class="bg-white p-8 rounded-lg shadow-lg">
                        <h1 class="text-3xl font-bold mb-6">Extended Private Key</h1>
                        <div class="space-y-4">
                            <p class="text-sm text-gray-600">Private Key: <span class="font-medium text-gray-800">{}</span></p>
                            <p class="text-sm text-gray-600">Chain Code: <span class="font-medium text-gray-800">{}</span></p>
                        </div>
                        <div class="flex space-x-4 mt-6">
                            <button onclick="saveExtendedPrivKeys()" class="bg-green-500 text-white py-2 px-4 rounded hover:bg-green-600">Save Extended Private Key</button>
                            <a href="/" class="bg-gray-500 text-white py-2 px-4 rounded hover:bg-gray-600">Back to Home</a>
                        </div>
                    </div>
                </body>
                </html>
                "#,
                    serde_json::to_string(&ext_key_json).unwrap(),
                    hex::encode(key.private_key),
                    hex::encode(key.chain_code)
                ))
            }
            Err(err) => Html(format!(
                r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Failed to generate extended private key: <span class="font-medium text-gray-800">{}</span></p>
                    <a href="/extended_priv_key" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#,
                err
            )),
        },
        Err(_) => Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Invalid seed format. Please provide a valid hex-encoded seed.</p>
                    <a href="/extended_priv_key" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#
        )),
    }
}

// Route pour sauvegarder toutes les clés privées étendues
#[derive(Deserialize)]
struct SaveExtendedPrivKeysRequest {
    ext_keys: String,
}

async fn save_extended_priv_keys(Form(input): Form<SaveExtendedPrivKeysRequest>) -> impl IntoResponse {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join("data/extended_keys/extended_keys.json");

    let ext_keys_data: Vec<Value> = match serde_json::from_str(&input.ext_keys) {
        Ok(data) => data,
        Err(_) => return "Failed to parse extended private keys data.".to_string(),
    };

    let mut existing_ext_keys: Vec<Value> = match fs::read_to_string(&file_path) {
        Ok(file) => serde_json::from_str(&file).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };
    existing_ext_keys.extend(ext_keys_data);

    match fs::write(&file_path, serde_json::to_string_pretty(&existing_ext_keys).unwrap()) {
        Ok(_) => "Extended private key saved successfully!".to_string(),
        Err(_) => "Failed to save extended private key.".to_string(),
    }
}

// Formulaire pour dériver une clé enfant
async fn derive_child_key_form() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Derive Child Key</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg">
            <h1 class="text-3xl font-bold mb-6">Derive Child Key</h1>
            <form method="post" action="/derive_child_key" class="space-y-4">
                <div>
                    <label for="private_key" class="block text-sm font-medium text-gray-700">Parent Private Key (hex):</label>
                    <input type="text" id="private_key" name="private_key" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <div>
                    <label for="chain_code" class="block text-sm font-medium text-gray-700">Chain Code (hex):</label>
                    <input type="text" id="chain_code" name="chain_code" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <div>
                    <label for="index" class="block text-sm font-medium text-gray-700">Index:</label>
                    <input type="number" id="index" name="index" min="0" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <button type="submit" class="w-full bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Derive</button>
            </form>
        </div>
    </body>
    </html>
    "#)
}

// Route pour gérer la dérivation d'une clé enfant
#[derive(Deserialize)]
struct ChildKeyRequest {
    private_key: String,
    chain_code: String,
    index: u32,
}

async fn derive_child_key(Form(input): Form<ChildKeyRequest>) -> impl IntoResponse {
    // Conversion de la clé privée parent et du chain code
    let parent_private_key = match hex::decode(&input.private_key) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            key
        }
        _ => {
            return Html(format!(
                r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Invalid private key format. Must be a 64-character hex string.</p>
                    <a href="/derive_child_key" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#
            ));
        }
    };

    let chain_code = match hex::decode(&input.chain_code) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut code = [0u8; 32];
            code.copy_from_slice(&bytes);
            code
        }
        _ => {
            return Html(format!(
                r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Invalid chain code format. Must be a 64-character hex string.</p>
                    <a href="/derive_child_key" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#
            ));
        }
    };

    // Création de la clé parent
    let parent_ext_key = ExtendedPrivKey {
        private_key: parent_private_key,
        chain_code,
    };

    // Dérivation de la clé enfant
    match parent_ext_key.derive_child_key(input.index) {
        Ok(child_key) => {
            let child_key_json = json!({
                "PrivateKey": hex::encode(child_key.private_key),
                "ChainCode": hex::encode(child_key.chain_code),
                "Index": input.index,
                "DerivationPath": format!("m/44'/0'/0'/0/{}", input.index),
                "GeneratedAt": Utc::now().to_rfc3339(),
            });

            Html(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Derived Child Key</title>
                    <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
                    <script>
                        function saveChildKeys() {{
                            fetch('/save_child_keys', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/x-www-form-urlencoded',
                                }},
                                body: new URLSearchParams({{
                                    child_keys: JSON.stringify([{}]),
                                }}),
                            }})
                            .then(response => response.text())
                            .then(message => {{
                                alert(message);
                            }})
                            .catch(error => {{
                                alert('Failed to save child key: ' + error);
                            }});
                        }}
                    </script>
                </head>
                <body class="bg-gray-100 flex items-center justify-center h-screen">
                    <div class="bg-white p-8 rounded-lg shadow-lg">
                        <h1 class="text-3xl font-bold mb-6">Child Key Derived Successfully</h1>
                        <div class="space-y-4">
                            <p class="text-sm text-gray-600"><strong>Index:</strong> <span class="font-medium text-gray-800">{}</span></p>
                            <p class="text-sm text-gray-600"><strong>Private Key:</strong> <span class="font-medium text-gray-800">{}</span></p>
                            <p class="text-sm text-gray-600"><strong>Chain Code:</strong> <span class="font-medium text-gray-800">{}</span></p>
                            <p class="text-sm text-gray-600"><strong>Derivation Path:</strong> <span class="font-medium text-gray-800">m/44'/0'/0'/0/{}</span></p>
                        </div>
                        <div class="flex space-x-4 mt-6">
                            <button onclick="saveChildKeys()" class="bg-green-500 text-white py-2 px-4 rounded hover:bg-green-600">Save Child Key</button>
                            <a href="/" class="bg-gray-500 text-white py-2 px-4 rounded hover:bg-gray-600">Back to Home</a>
                        </div>
                    </div>
                </body>
                </html>
                "#,
                serde_json::to_string(&child_key_json).unwrap(),
                input.index,
                hex::encode(child_key.private_key),
                hex::encode(child_key.chain_code),
                input.index
            ))
        }
        Err(err) => Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Failed to derive child key: <span class="font-medium text-gray-800">{}</span></p>
                    <a href="/derive_child_key" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#,
            err
        )),
    }
}

// Route pour sauvegarder toutes les clés enfants
#[derive(Deserialize)]
struct SaveChildKeysRequest {
    child_keys: String,
}

async fn save_child_keys(Form(input): Form<SaveChildKeysRequest>) -> impl IntoResponse {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join("data/child_keys/child_keys.json");

    let child_keys_data: Vec<Value> = match serde_json::from_str(&input.child_keys) {
        Ok(data) => data,
        Err(_) => return "Failed to parse child keys data.".to_string(),
    };

    let mut existing_child_keys: Vec<Value> = match fs::read_to_string(&file_path) {
        Ok(file) => serde_json::from_str(&file).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };
    existing_child_keys.extend(child_keys_data);

    match fs::write(&file_path, serde_json::to_string_pretty(&existing_child_keys).unwrap()) {
        Ok(_) => "Child key saved successfully!".to_string(),
        Err(_) => "Failed to save child key.".to_string(),
    }
}

// Formulaire pour générer un QR code
async fn qr_code_form() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Generate QR Code</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
    </head>
    <body class="bg-gray-100 flex items-center justify-center h-screen">
        <div class="bg-white p-8 rounded-lg shadow-lg">
            <h1 class="text-3xl font-bold mb-6">Generate QR Code</h1>
            <form method="post" action="/qr_code" class="space-y-4">
                <div>
                    <label for="address" class="block text-sm font-medium text-gray-700">Wallet Address:</label>
                    <input type="text" id="address" name="address" required class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500">
                </div>
                <button type="submit" class="w-full bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Generate</button>
            </form>
        </div>
    </body>
    </html>
    "#)
}

// Route pour gérer la génération de QR codes
#[derive(Deserialize)]
struct QRCodeRequest {
    address: String,
}

async fn generate_qr_code_web(Form(input): Form<QRCodeRequest>) -> impl IntoResponse {
    let file_name = format!("data/qr_codes/{}.svg", input.address);
    match generate_qr_code(&input.address, &file_name) {
        Ok(_) => Html(format!(
            r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>QR Code</title>
            <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
        </head>
        <body class="bg-gray-100 flex items-center justify-center h-screen">
            <div class="bg-white p-8 rounded-lg shadow-lg">
                <h1 class="text-3xl font-bold mb-6">QR Code Generated</h1>
                <p class="text-sm text-gray-600">QR Code saved as <code class="bg-gray-100 p-1 rounded">{}</code>.</p>
                <a href="/" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Back to Home</a>
            </div>
        </body>
        </html>
        "#,
            file_name
        )),
        Err(err) => Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Error</title>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
            </head>
            <body class="bg-gray-100 flex items-center justify-center h-screen">
                <div class="bg-white p-8 rounded-lg shadow-lg">
                    <h1 class="text-3xl font-bold mb-6 text-red-600">Error</h1>
                    <p class="text-sm text-gray-600">Failed to generate QR code: <span class="font-medium text-gray-800">{}</span></p>
                    <a href="/qr_code" class="block mt-6 text-center bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600">Try Again</a>
                </div>
            </body>
            </html>
            "#,
            err
        )),
    }
}

// Fonction pour générer un QR code
fn generate_qr_code(data: &str, file_name: &str) -> Result<(), String> {
    let code = QrCode::new(data).map_err(|e| e.to_string())?;
    let image = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();
    fs::write(file_name, image).map_err(|e| e.to_string())?;
    Ok(())
}