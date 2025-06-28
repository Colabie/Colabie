use std::cell::RefCell;

use schemou::Sirius;

use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{js_sys, MessageEvent};

pub struct WebSocket {
    ws: web_sys::WebSocket,
    rx: mpsc::Receiver<MessageEvent>,
}

impl WebSocket {
    pub async fn new(url: &str) -> Result<Self, JsValue> {
        let ws = web_sys::WebSocket::new(url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (onopen_notify, onopen_done) = oneshot::channel();
        let onopen_notify = RefCell::new(Some(onopen_notify));

        let onopen_fn = Closure::wrap(Box::new(move || {
            if let Some(onopen_notify) = onopen_notify.borrow_mut().take() {
                let _ = onopen_notify.send(());
            }
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(onopen_fn.into_js_value().unchecked_ref()));

        let (mut tx, rx) = mpsc::channel(1);

        let onmessage_fn = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            _ = tx.try_send(event);
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        ws.set_onmessage(Some(onmessage_fn.into_js_value().unchecked_ref()));

        let onclose_fn = {
            let ws = ws.clone();
            Closure::wrap(Box::new(move |_e| {
                // drop the sender in onmessage handler to stop receiving messages
                ws.set_onmessage(None);
            }) as Box<dyn FnMut(web_sys::Event)>)
        };
        ws.set_onclose(Some(onclose_fn.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onclose_fn.as_ref().unchecked_ref()));
        onclose_fn.forget();

        onopen_done
            .await
            .expect("unreachable: WebSocket onopen should always resolve");

        Ok(Self { ws, rx })
    }

    pub fn send_se<T: Sirius>(&mut self, data: T) -> Result<(), JsValue> {
        let serialized = data.serialize_buffered();
        self.send(&serialized)
    }

    pub async fn recv_de<T: Sirius>(&mut self) -> Result<T, JsValue> {
        let msg = self.recv().await?;

        let array_buffer: js_sys::ArrayBuffer = msg
            .data()
            .dyn_into()
            .map_err(|_| JsValue::from_str("MessageEvent data is not an ArrayBuffer"))?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);

        let deserialized_t = T::deserialize(&uint8_array.to_vec())
            .map(|(t, _)| t)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)));

        deserialized_t
    }

    fn send(&self, data: &[u8]) -> Result<(), JsValue> {
        self.ws.send_with_u8_array(data)
    }

    async fn recv(&mut self) -> Result<web_sys::MessageEvent, JsValue> {
        self.rx
            .next()
            .await
            .ok_or_else(|| JsValue::from_str("WebSocket closed"))
    }
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        _ = self.ws.close();
    }
}
