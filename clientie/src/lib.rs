pub mod servie_conn;
pub mod ws;

use crate::servie_conn::ServieConn;
use schemou::{legos::ShortIdStr, C2RRegister, R2CRegister, Sirius};

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
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn confirm(s: &str) -> bool;

    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

#[wasm_bindgen]
pub async fn register(username: &str) -> Result<(), JsValue> {
    // TODO: Check if the username is already registered
    // This is not trivial, needs discussion if we could hit registrie for read calls
    // labels: help wanted, discussion
    // Issue URL: https://github.com/Colabie/Colabie/issues/6

    let username = ShortIdStr::new(username)
        .map_err(|e| JsValue::from_str(&format!("Invalid username: {e}")))?;

    let (pb_key, sk_key) = generate_keypair();

    // TODO: Save secret key securely in a file instead
    // labels: enhancement, discussion
    // Issue URL: https://github.com/Colabie/Colabie/issues/5
    save_raw("sk_key", &sk_key);
    save_raw("username", username.as_bytes());

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

#[wasm_bindgen]
pub async fn login() -> Result<ServieConn, JsValue> {
    let username = load_raw("username");
    let username = str::from_utf8(&username)
        .map_err(|e| JsValue::from_str(&format!("Unreachable: Corrupted username {e}")))?;

    // TODO: Save secret key securely in a file instead
    // labels: enhancement, discussion
    // Issue URL: https://github.com/Colabie/Colabie/issues/5
    let sk_key = load_raw("sk_key");

    ServieConn::new("ws://localhost:8082/connect", username, &sk_key).await
}

// TODO: Use more robust hybrid cryptographic methods instead
// labels: enhancement
// Issue URL: https://github.com/Colabie/Colabie/issues/4
fn generate_keypair() -> (Box<[u8]>, Box<[u8]>) {
    use fips204::ml_dsa_87;
    use fips204::traits::SerDes;
    use rand_chacha::rand_core::SeedableRng;

    let mut rng = rand_chacha::ChaChaRng::from_entropy();
    let (pb_key, sk_key) = ml_dsa_87::try_keygen_with_rng(&mut rng).unwrap();
    (pb_key.into_bytes().into(), sk_key.into_bytes().into())
}
