use schemou::*;
use servie::*;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    mirror: Mirror,
    user_channels: UserChannels,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
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
        user_channels: UserChannels::new(),
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
        _ = handle_ws(socket, app_state).await;
    })
}

async fn handle_ws(
    mut socket: WebSocket,
    AppState {
        mirror,
        user_channels,
    }: AppState,
) -> anyhow::Result<()> {
    let C2SAck { username } = socket.recv_de().await?;

    if user_channels.is_online(&username).await {
        return Err(anyhow::anyhow!("User {:?} is already online", *username));
    }

    // TODO: Use commit id from the clientie as a hint that registrie might need to be refetched
    // Issue URL: https://github.com/Colabie/Colabie/issues/61
    // labels: enhancement, good first issue, discussion
    let _record = mirror
        .lookup_record(username.clone())
        .await
        // TODO: Ban IPs in case of failed login
        // Issue URL: https://github.com/Colabie/Colabie/issues/60
        // labels: enhancement, discussion
        .ok_or_else(|| anyhow::anyhow!("Invalid username"))?;

    let mut rng = ChaCha20Rng::from_os_rng();
    let auth_req = S2CAuthReq {
        random: rng.random(),
    };
    socket.send_se(auth_req).await?;

    // TODO: Verify the User and signed random
    // labels: enhancement
    // Issue URL: https://github.com/Colabie/Colabie/issues/54
    // coupled with #4
    let C2SAuthRes { signed_random: _ } = socket.recv_de().await?;

    let user_span = tracing::debug_span!("user", username = *username);
    async move {
        socket.send_se(S2CAuthResult::Success).await?;
        tracing::debug!("User connected");

        let mut self_channel = SelfChannel::new(username.clone(), user_channels.clone()).await;
        loop {
            tokio::select! {
                ws_recv = socket.recv_de() => {
                    let ConnectToUser { username: other_username } = ws_recv?;
                    // TODO: Ban IPs in case of invalid username
                    // Issue URL: https://github.com/Colabie/Colabie/issues/74
                    // labels: enhancement, discussion
                    let Some(other) = user_channels.get(&other_username).await else {
                        socket.send_se(S2CConnectToUserResult::UserBusy).await?;
                        continue;
                    };

                    // try_tell on the first interaction, but wait for next times
                    let Ok(_) = other.try_tell(&username, ChannelMsg::ConnectToUser) else {
                        socket.send_se(S2CConnectToUserResult::UserBusy).await?;
                        continue;
                    };

                    match self_channel.listen(&other_username).await {
                        Some(ChannelMsg::ConnectToUserReject) => {
                            socket.send_se(S2CConnectToUserResult::Reject).await?;
                            continue;
                        }

                        Some(ChannelMsg::UserBusy) | None => {
                            socket.send_se(S2CConnectToUserResult::UserBusy).await?;
                            continue;
                        }

                        // Implicitly accept if the other user also tries to connect at the same time
                        Some(ChannelMsg::ConnectToUserAccept | ChannelMsg::ConnectToUser) => {
                            socket.send_se(S2CConnectToUserResult::Accept).await?;
                        }
                    }
                }

                ChannelMsgWithSender { from, message } = self_channel.hear() => {
                    match message {
                        ChannelMsg::ConnectToUser => {
                            socket.send_se(ConnectToUser { username: from.clone() }).await?;
                            let connect = socket.recv_de::<C2SConnectToUserResult>().await?;

                            let Some(other) = user_channels
                                .get(&from)
                                .await else {
                                    continue;
                                };

                            match connect {
                                C2SConnectToUserResult::Reject => {
                                    _ = other.try_tell(
                                        &username,
                                        ChannelMsg::ConnectToUserReject,
                                    );
                                }
                                C2SConnectToUserResult::Accept => {
                                    let Ok(_) = other.tell(&username, ChannelMsg::ConnectToUserAccept).await else {
                                        continue;
                                    };
                                }
                            }
                        }

                        _ => unreachable!("Inappropriate message from self channel"),
                    }
                }
            }
        }
    }
    .instrument(user_span)
    .await
}
