use schemou::{legos::ShortIdStr, *};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;
use web_sys::{js_sys, ErrorEvent, MessageEvent, WebSocket};

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
pub async fn connect_to_servie() -> Result<(), JsValue> {
    let username = ShortIdStr::new(
        std::str::from_utf8(&load_raw("username"))
            .map_err(|e| JsValue::from_str(&format!("Unreachable: Corrupted username {e}")))?,
    )
    .map_err(|e| JsValue::from_str(&format!("Invalid username: {e}")))?;

    let ws = WebSocket::new("ws://localhost:8082/connect")
        .map_err(|e| JsValue::from_str(&format!("WebSocket error: {:?}", e)))?;

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let cloned_ws = ws.clone();
    let onopen = Closure::wrap(Box::new(move || {
        log("Connected to servie");
    }) as Box<dyn FnMut()>);
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            log(&format!("message event, received arraybuffer: {:?}", abuf));
            let array = js_sys::Uint8Array::new(&abuf);
            let len = array.byte_length() as usize;
            log(&format!("Arraybuffer received {} bytes: {:?}", len, array.to_vec()));
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log(&format!("Received: {}", txt));
        } else {
            log(&format!("message event, received Unknown: {:?}", e.data()));
        }
    }) as Box<dyn FnMut(_)>);
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
        log(&format!("WebSocket error: {:?}", e));
    }) as Box<dyn FnMut(_)>);
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    Ok(())
}

// TODO: Use more robust hybrid cryptographic methods instead
// labels: enhancement
fn generate_keypair() -> (Box<[u8]>, Box<[u8]>) {
    use fips204::ml_dsa_87;
    use fips204::traits::SerDes;
    use rand_chacha::rand_core::SeedableRng;

    let mut rng = rand_chacha::ChaChaRng::from_entropy();
    let (pb_key, sk_key) = ml_dsa_87::try_keygen_with_rng(&mut rng).unwrap();
    (pb_key.into_bytes().into(), sk_key.into_bytes().into())
}
