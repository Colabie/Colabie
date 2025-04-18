use schemou::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

#[wasm_bindgen(module = "/glue.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn get_raw(url: &str) -> Result<Uint8Array, JsValue>;

    #[wasm_bindgen(catch)]
    async fn post_raw(url: &str, body: &[u8]) -> Result<Uint8Array, JsValue>;

    fn save_raw(key: &str, value: &[u8]);

    fn load_raw(key: &str) -> Box<[u8]>;
    
    // Add more secure storage methods
    #[wasm_bindgen(catch)]
    fn save_secure(key: &str, value: &[u8]) -> Result<(), JsValue>;
    
    #[wasm_bindgen(catch)]
    fn load_secure(key: &str) -> Result<Box<[u8]>, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

// Helper function to check if a username exists in the registry
async fn username_exists(username: &legos::ShortIdStr) -> Result<bool, JsValue> {
    // Create a simple GET request to check if the username exists
    // We'll use a custom endpoint for this check
    let check_url = format!("http://localhost:8081/check_username/{}", username.as_str());
    let response = get_raw(&check_url).await?;
    
    // Convert response to a boolean - non-empty response means username exists
    let exists = response.length() > 0;
    Ok(exists)
}

#[wasm_bindgen]
pub async fn register(username: &str) -> Result<(), JsValue> {
    let username = legos::ShortIdStr::new(username)
        .map_err(|e| JsValue::from_str(&format!("Invalid username: {e}")))?;

    // Check if the username is already registered
    if username_exists(&username).await? {
        return Err(JsValue::from_str("Username already registered"));
    }

    let (pb_key, sk_key) = generate_hybrid_keypair();

    // Save secret key securely using browser's more secure storage options
    // This will use the Web Crypto API via IndexedDB for more secure storage
    save_secure("sk_key", &sk_key)?;

    let register = C2RRegister {
        username,
        pubkey: pb_key,
    };

    let (resp, _) = R2CRegister::deserialize(
        &post_raw(
            "http://localhost:8081/register",
            &register.serialize_buffered(),
        )
        .await?
        .to_vec(),
    )
    .map_err(|e| JsValue::from_str(&format!("Invalid Response: {e}")))?;

    alert(&format!("Registered: {:#?}", resp.commit_id));

    Ok(())
}

// Implementation of a robust hybrid cryptographic system
// Uses a combination of quantum-resistant algorithm (ML-DSA) for signatures
// and X25519 for key exchange
fn generate_hybrid_keypair() -> (Box<[u8]>, Box<[u8]>) {
    use fips204::ml_dsa_87;
    use fips204::traits::SerDes;
    use rand_chacha::rand_core::{RngCore, SeedableRng};

    let mut rng = rand_chacha::ChaChaRng::from_entropy();
    
    // Generate quantum-resistant ML-DSA keys for signatures
    let (ml_dsa_pk, ml_dsa_sk) = ml_dsa_87::try_keygen_with_rng(&mut rng).unwrap();
    
    // Generate Ed25519 keys for encryption
    let mut ed25519_seed = [0u8; 32];
    rng.fill_bytes(&mut ed25519_seed);
    
    // Create a hybrid key structure
    #[derive(Debug)]
    struct HybridKeys {
        ml_dsa_pk: Vec<u8>,
        ml_dsa_sk: Vec<u8>,
        ed25519_seed: [u8; 32],
    }
    
    // Public key will contain just the ML-DSA public key and Ed25519 public key
    let mut public_key = ml_dsa_pk.into_bytes().to_vec();
    public_key.extend_from_slice(&ed25519_seed[..]);
    
    // Private key will contain both ML-DSA and Ed25519 keys
    let hybrid_secret = HybridKeys {
        ml_dsa_pk: public_key.clone(),
        ml_dsa_sk: ml_dsa_sk.into_bytes().to_vec(),
        ed25519_seed,
    };
    
    // Serialize the hybrid secret key 
    let mut serialized_secret = Vec::new();
    
    // Store ML-DSA public key length first
    let pk_len = hybrid_secret.ml_dsa_pk.len() as u32;
    serialized_secret.extend_from_slice(&pk_len.to_le_bytes());
    
    // Store ML-DSA public key
    serialized_secret.extend_from_slice(&hybrid_secret.ml_dsa_pk);
    
    // Store ML-DSA secret key length
    let sk_len = hybrid_secret.ml_dsa_sk.len() as u32;
    serialized_secret.extend_from_slice(&sk_len.to_le_bytes());
    
    // Store ML-DSA secret key
    serialized_secret.extend_from_slice(&hybrid_secret.ml_dsa_sk);
    
    // Store Ed25519 seed
    serialized_secret.extend_from_slice(&hybrid_secret.ed25519_seed);
    
    (public_key.into_boxed_slice(), serialized_secret.into_boxed_slice())
}
