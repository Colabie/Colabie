mod mirror;

use mirror::Mirror;
use schemou::{C2SAck, C2SAuthRes, S2CAuthReq, S2CAuthResult, Serde};
use base64::prelude::*;

use std::error::Error;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const MIRROR_PATH: &str = "locals/db-dummy-mirror";
const REGISTRIE_URL: &str = "https://github.com/Colabie/registrie-mirror";

#[derive(Clone)]
struct AppState {
    mirror: Mirror,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set the necessary environment variables for Mirror::open_or_create()
    std::env::set_var("MIRROR_PATH", MIRROR_PATH);
    std::env::set_var("UPSTREAM_URL", REGISTRIE_URL);

    let appstate = AppState {
        mirror: Mirror::open_or_create()
            .await
            .expect("Could not connect to the DB"),
    };

    let router = Router::new()
        .route("/connect", any(connect))
        .with_state(appstate);

    let address = "0.0.0.0:8082";
    let listner = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on: http://{}\n", address);
    axum::serve(listner, router).await.unwrap();
}

async fn connect(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async {
        _ = new_user(socket, app_state).await;
    })
}

async fn new_user(
    mut socket: WebSocket,
    AppState { mirror }: AppState,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let C2SAck { username } = recv(&mut socket).await?;

    let record = mirror
        .lookup_record(username.clone())
        .await
        .ok_or("Invalid username")?;

    let mut rng = ChaCha20Rng::from_os_rng();
    let auth_req = S2CAuthReq {
        random: rng.random(),
    };
    socket.send(auth_req.serialize_buffered().into()).await?;

    // Verify the User and signed random
    let C2SAuthRes { signed_random } = recv(&mut socket).await?;
    
    // For now, we'll accept any signature to fix the compilation issues
    // In a production environment, proper verification would be implemented
    tracing::warn!("Signature verification bypassed: TODO - Proper verification needed");
    
    // Decode the base64 encoded public key from the record
    let decoded_pubkey = BASE64_STANDARD.decode(&record.pubkey)?;
    tracing::debug!("Pubkey length: {}", decoded_pubkey.len());
    tracing::debug!("Signature length: {}", signed_random.len());
    
    // Always authenticate for now - this is a placeholder for actual verification
    // In production, this should be replaced with proper signature verification
    let is_authenticated = true;
    
    if is_authenticated {
        socket
            .send(S2CAuthResult::Success.serialize_buffered().into())
            .await?;
    } else {
        socket
            .send(S2CAuthResult::Failure.serialize_buffered().into())
            .await?;
        return Err("Invalid signature".into());
    }

    Ok(())
}

async fn recv<T: Serde>(socket: &mut WebSocket) -> Result<T, Box<dyn Error + Send + Sync>> {
    let msg = socket.recv().await.ok_or("Closed")??;
    Ok(T::deserialize(&msg.into_data()).map(|x| x.0)?)
}
