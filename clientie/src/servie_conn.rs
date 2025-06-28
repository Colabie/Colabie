use crate::{alert, confirm, log, ws::WebSocket};
use schemou::{
    legos::ShortIdStr, C2SAck, C2SAuthRes, C2SConnectToUserResult, ConnectToUser, S2CAuthReq,
    S2CAuthResult, S2CConnectToUserResult,
};

use futures::{channel::mpsc, select, FutureExt, SinkExt, StreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub enum ClientEvent {
    ConnectToUser(ConnectToUser),
}

#[wasm_bindgen]
pub struct ServieConn {
    tx: mpsc::Sender<ClientEvent>,
}

#[wasm_bindgen]
impl ServieConn {
    #[wasm_bindgen(constructor)]
    pub async fn new(url: &str, username: &str, _sk_key: &[u8]) -> Result<ServieConn, JsValue> {
        let username = ShortIdStr::new(username)
            .map_err(|e| JsValue::from_str(&format!("Invalid username: {e}")))?;

        let mut ws = WebSocket::new(url).await?;
        ws.send_se(C2SAck { username })?;

        let S2CAuthReq { random } = ws.recv_de().await?;
        // TODO: Verify the User and signed random
        // labels: enhancement
        // Issue URL: https://github.com/Colabie/Colabie/issues/54
        // coupled with #4
        ws.send_se(C2SAuthRes {
            signed_random: random,
        })?;

        let auth_result = ws.recv_de().await?;
        let S2CAuthResult::Success = auth_result else {
            return Err(JsValue::from_str("Authentication failed"));
        };

        let (tx, mut rx) = mpsc::channel(1);

        spawn_local(async move {
            async {
                loop {
                    select! {
                        server_msg = ws.recv_de::<ConnectToUser>().fuse() => {
                            let server_msg = server_msg?;
                            let connect = confirm(&format!("User {} wants to connect to you", *server_msg.username));
                            if connect {
                                ws.send_se(C2SConnectToUserResult::Accept)?;
                            } else {
                                ws.send_se(C2SConnectToUserResult::Reject)?;
                            }
                        }

                        client_ev = rx.next() => {
                            let client_ev = client_ev.expect("Client event sender was dropped");

                            match client_ev {
                                ClientEvent::ConnectToUser(connect) => {
                                    ws.send_se(connect)?;

                                    let result = ws.recv_de::<S2CConnectToUserResult>().await?;
                                    match result {
                                        S2CConnectToUserResult::Accept => {
                                            alert("User accepted your connection request");
                                        }
                                        S2CConnectToUserResult::Reject => {
                                            alert("User rejected your connection request");
                                        }
                                        S2CConnectToUserResult::UserBusy => {
                                            alert("User is busy");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                #[allow(unreachable_code)]
                // For type inference
                Ok::<(), JsValue>(())
            }
            .await
            .expect("Error");
        });

        log("abcde");

        Ok(ServieConn { tx })
    }

    #[wasm_bindgen(js_name = "connectToUser")]
    pub async fn connect_to_user(&mut self, username: &str) -> Result<(), JsValue> {
        let username = ShortIdStr::new(username)
            .map_err(|e| JsValue::from_str(&format!("Invalid username: {e}")))?;

        self.tx
            .send(ClientEvent::ConnectToUser(ConnectToUser { username }))
            .await
            .expect("Unreachable: Client event receiver was dropped");
        // self.ws.send_se(ConnectToUser { username })?;

        Ok(())
    }
}
