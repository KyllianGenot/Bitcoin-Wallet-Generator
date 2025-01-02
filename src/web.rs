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

// Launch the server
pub async fn start_server() {
    // Configure routes
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/generate_wallets", get(generate_wallets_form).post(generate_wallets))
        .route("/extended_priv_key", get(extended_priv_key_form).post(generate_extended_priv_key))
        .route("/derive_child_key", get(derive_child_key_form).post(derive_child_key))
        .route("/qr_code", get(qr_code_form).post(generate_qr_code_web))
        .route("/save_all_wallets", post(save_all_wallets))
        .route("/save_all_qr_codes", post(save_all_qr_codes))
        .route("/save_extended_priv_keys", post(save_extended_priv_keys))
        .route("/save_child_keys", post(save_child_keys))
        .fallback(handle_404); // Catch-all route for 404 errors

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✅ Server running at http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Reusable HTML template function
fn html_template(title: &str, content: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{}</title>
        <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
        <link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@400;500;700&display=swap" rel="stylesheet">
        <style>
            body {{
                background: linear-gradient(135deg, #1e1e2f, #2d2d44);
                color: #fff;
                font-family: 'Quicksand', sans-serif;
            }}
            .neumorphic {{
                background: linear-gradient(145deg, #2d2d44, #1e1e2f);
                border-radius: 20px;
                box-shadow:  10px 10px 20px #1a1a28, 
                            -10px -10px 20px #24243c;
            }}
            .btn {{
                background: linear-gradient(135deg, #6d28d9, #8b5cf6);
                border: none;
                color: white;
                padding: 12px 24px;
                border-radius: 12px;
                cursor: pointer;
                transition: all 0.3s ease;
            }}
            .btn:hover {{
                transform: translateY(-2px);
                box-shadow: 0 4px 15px rgba(109, 40, 217, 0.4);
            }}
            .navbar {{
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                padding: 16px;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                z-index: 1000;
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}
            .navbar a {{
                color: white;
                margin: 0 12px;
                text-decoration: none;
                font-weight: 500;
                transition: all 0.3s ease;
            }}
            .navbar a:not(.navbar-logo):hover {{
                color: #8b5cf6;
                transform: translateY(-2px);
            }}
            .navbar-logo {{
                font-size: 1.5rem;
                font-weight: bold;
                color: white;
                text-decoration: none;
            }}
            .fade-in {{
                animation: fadeIn 1.5s ease-in-out;
            }}
            @keyframes fadeIn {{
                from {{ opacity: 0; }}
                to {{ opacity: 1; }}
            }}
            .slide-up {{
                animation: slideUp 1s ease-in-out;
            }}
            @keyframes slideUp {{
                from {{ transform: translateY(20px); opacity: 0; }}
                to {{ transform: translateY(0); opacity: 1; }}
            }}
            /* Custom Scrollbar */
            ::-webkit-scrollbar {{
                width: 10px;
            }}
            ::-webkit-scrollbar-track {{
                background: #2d2d44;
                border-radius: 10px;
            }}
            ::-webkit-scrollbar-thumb {{
                background: #6d28d9;
                border-radius: 10px;
            }}
            ::-webkit-scrollbar-thumb:hover {{
                background: #8b5cf6;
            }}
            .scrollable-wallets {{
                max-height: 50vh;
                overflow-y: auto;
                padding-right: 10px;
            }}
            .error-container {{
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                text-align: center;
                gap: 1rem;
            }}
        </style>
    </head>
    <body>
        <div class="navbar">
            <a href="/" class="navbar-logo">Bitcoin Wallet Generator</a>
            <div>
                <a href="/">Home</a>
                <a href="/generate_wallets">Generate Wallets</a>
                <a href="/extended_priv_key">Extended Private Key</a>
                <a href="/derive_child_key">Derive Child Key</a>
                <a href="/qr_code">QR Code</a>
            </div>
        </div>
        {}
    </body>
    </html>
    "#,
        title, content
    )
}

// Main landing page
async fn landing_page() -> impl IntoResponse {
    Html(html_template(
        "Bitcoin Wallet Generator",
        r#"
        <div class="flex items-center justify-center h-screen px-4">
            <div class="neumorphic p-8 max-w-3xl text-center fade-in">
                <h1 class="text-5xl font-bold mb-6 slide-up">Welcome to the Bitcoin Wallet Generator</h1>
                <p class="text-lg text-gray-300 mb-8 slide-up">
                    This tool allows you to securely generate Bitcoin wallets, derive extended private keys, 
                    create child keys, and generate QR codes for wallet addresses. All operations are performed 
                    locally, ensuring your data remains private and secure.
                </p>
                <div class="space-y-6 slide-up">
                    <p class="text-gray-300">Explore the features using the navigation bar above.</p>
                </div>
            </div>
        </div>
        "#,
    ))
}

// 404 Error Page
async fn handle_404() -> impl IntoResponse {
    Html(html_template(
        "404 Not Found",
        r#"
        <div class="flex items-center justify-center h-screen">
            <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">404 Not Found</h1>
                <p class="text-sm text-gray-300 slide-up">The page you are looking for does not exist.</p>
                <a href="/" class="btn w-full mt-6 slide-up">Back to Home</a>
            </div>
        </div>
        "#,
    ))
}

// Form to generate wallets
async fn generate_wallets_form() -> impl IntoResponse {
    Html(html_template(
        "Generate Wallets",
        r#"
        <div class="flex items-center justify-center h-screen">
            <div class="neumorphic p-8 max-w-md w-full fade-in">
                <h1 class="text-3xl font-bold mb-6 slide-up">Generate Wallets</h1>
                <form method="post" action="/generate_wallets" class="space-y-4">
                    <div>
                        <label for="count" class="block text-sm font-medium text-gray-300">Number of wallets:</label>
                        <input type="number" id="count" name="count" min="1" max="100" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <button type="submit" class="btn w-full">Generate</button>
                </form>
            </div>
        </div>
        "#,
    ))
}

// Route to handle wallet generation
#[derive(Deserialize)]
struct WalletRequest {
    count: usize,
}

async fn generate_wallets(Form(input): Form<WalletRequest>) -> impl IntoResponse {
    // Ensure the count is within the allowed range
    let count = input.count.min(100).max(1);

    let mut wallets = Vec::new();
    for _ in 0..count {
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

    Html(html_template(
        "Generated Wallets",
        &format!(
            r#"
            <div class="flex items-center justify-center h-screen">
                <div class="neumorphic p-8 max-w-3xl w-full fade-in">
                    <h1 class="text-3xl font-bold mb-6 slide-up">Generated Wallets</h1>
                    <div class="scrollable-wallets">
                        {}
                    </div>
                    <div class="flex space-x-4 mt-6">
                        <button onclick="saveAllWallets()" class="btn">Save All Wallets</button>
                        <button onclick="saveAllQrCodes()" class="btn">Save All QR Codes</button>
                        <a href="/" class="btn bg-gray-500 hover:bg-gray-600">Back to Home</a>
                    </div>
                </div>
            </div>
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
            "#,
            wallets.iter().map(|wallet| {
                format!(
                    r#"
                    <div class='mb-4 p-4 bg-gray-700 rounded-lg'>
                        <p class='text-sm text-gray-300'>Mnemonic: <span class='font-medium text-white'>{}</span></p>
                        <p class='text-sm text-gray-300'>Address: <span class='font-medium text-white'>{}</span></p>
                        <p class='text-sm text-gray-300'>Public Key: <span class='font-medium text-white'>{}</span></p>
                        <p class='text-sm text-gray-300'>Private Key: <span class='font-medium text-white'>{}</span></p>
                    </div>
                    "#,
                    wallet["Mnemonic"],
                    wallet["Address"],
                    wallet["PublicKey"],
                    wallet["PrivateKey"]
                )
            }).collect::<Vec<_>>().join(""),
            wallets_json,
            wallets_json
        ),
    ))
}

// Route to save all wallets
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

// Route to save all QR codes
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

// Form to generate extended private key
async fn extended_priv_key_form() -> impl IntoResponse {
    Html(html_template(
        "Generate Extended Private Key",
        r#"
        <div class="flex items-center justify-center h-screen">
            <div class="neumorphic p-8 max-w-md w-full fade-in">
                <h1 class="text-3xl font-bold mb-6 slide-up">Generate Extended Private Key</h1>
                <form method="post" action="/extended_priv_key" class="space-y-4">
                    <div>
                        <label for="seed" class="block text-sm font-medium text-gray-300">Seed (hex):</label>
                        <input type="text" id="seed" name="seed" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <button type="submit" class="btn w-full">Generate</button>
                </form>
            </div>
        </div>
        "#,
    ))
}

// Route to handle extended private key generation
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

                Html(html_template(
                    "Extended Private Key",
                    &format!(
                        r#"
                        <div class="flex items-center justify-center h-screen">
                            <div class="neumorphic p-8 max-w-3xl w-full fade-in">
                                <h1 class="text-3xl font-bold mb-6 slide-up">Extended Private Key</h1>
                                <div class="mb-4 p-4 bg-gray-700 rounded-lg">
                                    <p class="text-sm text-gray-300">Private Key: <span class="font-medium text-white">{}</span></p>
                                    <p class="text-sm text-gray-300">Chain Code: <span class="font-medium text-white">{}</span></p>
                                </div>
                                <div class="flex space-x-4 mt-6">
                                    <button onclick="saveExtendedPrivKeys()" class="btn">Save Extended Private Key</button>
                                    <a href="/" class="btn bg-gray-500 hover:bg-gray-600">Back to Home</a>
                                </div>
                            </div>
                        </div>
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
                        "#,
                        hex::encode(key.private_key),
                        hex::encode(key.chain_code),
                        serde_json::to_string(&ext_key_json).unwrap()
                    ),
                ))
            }
            Err(err) => Html(html_template(
                "Error",
                &format!(
                    r#"
                    <div class="flex items-center justify-center h-screen">
                        <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                            <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                            <p class="text-sm text-gray-300 slide-up">Failed to generate extended private key: <span class="font-medium text-white">{}</span></p>
                            <a href="/extended_priv_key" class="btn w-full mt-6 slide-up">Try Again</a>
                        </div>
                    </div>
                    "#,
                    err
                ),
            )),
        },
        Err(_) => Html(html_template(
            "Error",
            r#"
            <div class="flex items-center justify-center h-screen">
                <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                    <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                    <p class="text-sm text-gray-300 slide-up">Invalid seed format. Please provide a valid hex-encoded seed.</p>
                    <a href="/extended_priv_key" class="btn w-full mt-6 slide-up">Try Again</a>
                </div>
            </div>
            "#,
        )),
    }
}

// Route to save extended private keys
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

// Form to derive child key
async fn derive_child_key_form() -> impl IntoResponse {
    Html(html_template(
        "Derive Child Key",
        r#"
        <div class="flex items-center justify-center h-screen">
            <div class="neumorphic p-8 max-w-md w-full fade-in">
                <h1 class="text-3xl font-bold mb-6 slide-up">Derive Child Key</h1>
                <form method="post" action="/derive_child_key" class="space-y-4">
                    <div>
                        <label for="private_key" class="block text-sm font-medium text-gray-300">Parent Private Key (hex):</label>
                        <input type="text" id="private_key" name="private_key" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <div>
                        <label for="chain_code" class="block text-sm font-medium text-gray-300">Chain Code (hex):</label>
                        <input type="text" id="chain_code" name="chain_code" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <div>
                        <label for="index" class="block text-sm font-medium text-gray-300">Index:</label>
                        <input type="number" id="index" name="index" min="0" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <button type="submit" class="btn w-full">Derive</button>
                </form>
            </div>
        </div>
        "#,
    ))
}

// Route to handle child key derivation
#[derive(Deserialize)]
struct ChildKeyRequest {
    private_key: String,
    chain_code: String,
    index: u32,
}

async fn derive_child_key(Form(input): Form<ChildKeyRequest>) -> impl IntoResponse {
    // Convert parent private key and chain code
    let parent_private_key = match hex::decode(&input.private_key) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            key
        }
        _ => {
            return Html(html_template(
                "Error",
                r#"
                <div class="flex items-center justify-center h-screen">
                    <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                        <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                        <p class="text-sm text-gray-300 slide-up">Invalid private key format. Must be a 64-character hex string.</p>
                        <a href="/derive_child_key" class="btn w-full mt-6 slide-up">Try Again</a>
                    </div>
                </div>
                "#,
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
            return Html(html_template(
                "Error",
                r#"
                <div class="flex items-center justify-center h-screen">
                    <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                        <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                        <p class="text-sm text-gray-300 slide-up">Invalid chain code format. Must be a 64-character hex string.</p>
                        <a href="/derive_child_key" class="btn w-full mt-6 slide-up">Try Again</a>
                    </div>
                </div>
                "#,
            ));
        }
    };

    // Create parent key
    let parent_ext_key = ExtendedPrivKey {
        private_key: parent_private_key,
        chain_code,
    };

    // Derive child key
    match parent_ext_key.derive_child_key(input.index) {
        Ok(child_key) => {
            let child_key_json = json!({
                "PrivateKey": hex::encode(child_key.private_key),
                "ChainCode": hex::encode(child_key.chain_code),
                "Index": input.index,
                "DerivationPath": format!("m/44'/0'/0'/0/{}", input.index),
                "GeneratedAt": Utc::now().to_rfc3339(),
            });

            Html(html_template(
                "Derived Child Key",
                &format!(
                    r#"
                    <div class="flex items-center justify-center h-screen">
                        <div class="neumorphic p-8 max-w-3xl w-full fade-in">
                            <h1 class="text-3xl font-bold mb-6 slide-up">Child Key Derived Successfully</h1>
                            <div class="mb-4 p-4 bg-gray-700 rounded-lg">
                                <p class="text-sm text-gray-300"><strong>Index:</strong> <span class="font-medium text-white">{}</span></p>
                                <p class="text-sm text-gray-300"><strong>Private Key:</strong> <span class="font-medium text-white">{}</span></p>
                                <p class="text-sm text-gray-300"><strong>Chain Code:</strong> <span class="font-medium text-white">{}</span></p>
                                <p class="text-sm text-gray-300"><strong>Derivation Path:</strong> <span class="font-medium text-white">m/44'/0'/0'/0/{}</span></p>
                            </div>
                            <div class="flex space-x-4 mt-6">
                                <button onclick="saveChildKeys()" class="btn">Save Child Key</button>
                                <a href="/" class="btn bg-gray-500 hover:bg-gray-600">Back to Home</a>
                            </div>
                        </div>
                    </div>
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
                    "#,
                    input.index,
                    hex::encode(child_key.private_key),
                    hex::encode(child_key.chain_code),
                    input.index,
                    serde_json::to_string(&child_key_json).unwrap()
                ),
            ))
        }
        Err(err) => Html(html_template(
            "Error",
            &format!(
                r#"
                <div class="flex items-center justify-center h-screen">
                    <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                        <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                        <p class="text-sm text-gray-300 slide-up">Failed to derive child key: <span class="font-medium text-white">{}</span></p>
                        <a href="/derive_child_key" class="btn w-full mt-6 slide-up">Try Again</a>
                    </div>
                </div>
                "#,
                err
            ),
        )),
    }
}

// Route to save child keys
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

// Form to generate QR code
async fn qr_code_form() -> impl IntoResponse {
    Html(html_template(
        "Generate QR Code",
        r#"
        <div class="flex items-center justify-center h-screen">
            <div class="neumorphic p-8 max-w-md w-full fade-in">
                <h1 class="text-3xl font-bold mb-6 slide-up">Generate QR Code</h1>
                <form method="post" action="/qr_code" class="space-y-4">
                    <div>
                        <label for="address" class="block text-sm font-medium text-gray-300">Wallet Address:</label>
                        <input type="text" id="address" name="address" required class="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-purple-500 focus:border-purple-500 text-white">
                    </div>
                    <button type="submit" class="btn w-full">Generate</button>
                </form>
            </div>
        </div>
        "#,
    ))
}

// Route to handle QR code generation
#[derive(Deserialize)]
struct QRCodeRequest {
    address: String,
}

async fn generate_qr_code_web(Form(input): Form<QRCodeRequest>) -> impl IntoResponse {
    let file_name = format!("data/qr_codes/{}.svg", input.address);
    match generate_qr_code(&input.address, &file_name) {
        Ok(_) => Html(html_template(
            "QR Code",
            &format!(
                r#"
                <div class="flex items-center justify-center h-screen">
                    <div class="neumorphic p-8 max-w-md w-full fade-in">
                        <h1 class="text-3xl font-bold mb-6 slide-up">QR Code Generated</h1>
                        <p class="text-sm text-gray-300 slide-up">QR Code saved as <code class="bg-gray-700 p-1 rounded">{}</code>.</p>
                        <a href="/" class="btn w-full mt-6 slide-up">Back to Home</a>
                    </div>
                </div>
                "#,
                file_name
            ),
        )),
        Err(err) => Html(html_template(
            "Error",
            &format!(
                r#"
                <div class="flex items-center justify-center h-screen">
                    <div class="neumorphic p-8 max-w-md w-full fade-in error-container">
                        <h1 class="text-3xl font-bold mb-6 text-red-600 slide-up">Error</h1>
                        <p class="text-sm text-gray-300 slide-up">Failed to generate QR code: <span class="font-medium text-white">{}</span></p>
                        <a href="/qr_code" class="btn w-full mt-6 slide-up">Try Again</a>
                    </div>
                </div>
                "#,
                err
            ),
        )),
    }
}

// Function to generate QR code
fn generate_qr_code(data: &str, file_name: &str) -> Result<(), String> {
    let code = QrCode::new(data).map_err(|e| e.to_string())?;
    let image = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();
    fs::write(file_name, image).map_err(|e| e.to_string())?;
    Ok(())
}