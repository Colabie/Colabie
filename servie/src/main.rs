mod mirror;

use mirror::Mirror;
use schemou::{C2SAck, C2SAuthRes, S2CAuthReq, S2CAuthResult, Serde};

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

    tracing::info!("loading .env");
    dotenvy::dotenv().expect("Failed to load .env file");

    let appstate = AppState {
        mirror: Mirror::open_or_create()
            .await
            .expect("Could not connect to the DB"),
    };

    let router = Router::new()
        .route("/connect", any(connect))
        .with_state(appstate);

    let address = "0.0.0.0:8082";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on: http://{}\n", address);
    axum::serve(listener, router).await.unwrap();
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

    // TODO: Use commit id from the clientie as a hint that registrie might need to be refetched
    // Issue URL: https://github.com/Colabie/Colabie/issues/61
    // labels: enhancement, good first issue, discussion
    let _record = mirror
        .lookup_record(username.clone())
        .await
        // TODO: Ban IPs in case of failed login
        // Issue URL: https://github.com/Colabie/Colabie/issues/60
        // labels: enhancement, discussion
        .ok_or("Invalid username")?;

    let mut rng = ChaCha20Rng::from_os_rng();
    let auth_req = S2CAuthReq {
        random: rng.random(),
    };
    socket.send(auth_req.serialize_buffered().into()).await?;

    // TODO: Verify the User and signed random
    // labels: enhancement
    // Issue URL: https://github.com/Colabie/Colabie/issues/54
    // coupled with #4
    let C2SAuthRes { signed_random: _ } = recv(&mut socket).await?;
    socket
        .send(S2CAuthResult::Success.serialize_buffered().into())
        .await?;

    Ok(())
}

async fn recv<T: Serde>(socket: &mut WebSocket) -> Result<T, Box<dyn Error + Send + Sync>> {
    let msg = socket.recv().await.ok_or("Closed")??;
    Ok(T::deserialize(&msg.into_data()).map(|x| x.0)?)
}
