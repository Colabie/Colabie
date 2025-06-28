pub mod mirror;

pub use mirror::Mirror;

use schemou::legos::ShortIdStr;
use schemou::Sirius;

use std::{collections::HashMap, error::Error, fmt, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use tokio::{
    sync::{mpsc, RwLock},
    time::{timeout_at, Instant},
};

#[derive(Debug, thiserror::Error)]
pub enum ServieError {
    #[error("Websocket connection closed")]
    SocketClosed,

    #[error("Axum error: {0}")]
    AxumError(#[from] axum::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] schemou::SiriusError),

    #[error("User doesn't comply with protocol: {0}")]
    NonCompliance(&'static str),
}

pub type Result<T, E = ServieError> = std::result::Result<T, E>;

#[allow(async_fn_in_trait)]
pub trait SerdeSocket {
    async fn recv_de<T: Sirius + fmt::Debug>(&mut self) -> Result<T>;
    async fn send_se<T: Sirius + fmt::Debug>(&mut self, data: T) -> Result<()>;
}

impl SerdeSocket for WebSocket {
    async fn recv_de<T: Sirius + fmt::Debug>(&mut self) -> Result<T> {
        loop {
            let msg = self
                .recv()
                .await
                .ok_or_else(|| ServieError::SocketClosed)??;

            let data = match msg {
                Message::Binary(msg) => {
                    tracing::trace!("Received message: {:?}", msg);
                    msg
                }
                Message::Text(_) => {
                    tracing::trace!("Received a text message, expected binary data, weird");
                    return Err(ServieError::NonCompliance(
                        "Received a text message, expected binary data",
                    ));
                }
                Message::Close(_) => {
                    tracing::trace!("Received a close message");
                    return Err(ServieError::SocketClosed);
                }
                Message::Ping(_) => {
                    tracing::trace!("Received a ping message");
                    continue;
                }
                Message::Pong(_) => {
                    tracing::trace!("Received a pong message");
                    continue;
                }
            };

            let (deserialized, _) = T::deserialize(&data)?;
            tracing::trace!("Deserialized message: {:?}", deserialized);
            return Ok(deserialized);
        }
    }

    async fn send_se<T: Sirius + fmt::Debug>(&mut self, data: T) -> Result<()> {
        tracing::trace!("Sending message: {:?}", data);

        let serialized = data.serialize_buffered();
        self.send(serialized.into()).await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct SelfChannel {
    i: Option<SelfChannelInner>,
}

struct SelfChannelInner {
    username: ShortIdStr,
    channel: mpsc::Receiver<ChannelMsgWithSender>,
    channels: UserChannels,
}

pub struct ChannelMsgWithSender {
    pub from: ShortIdStr,
    pub message: ChannelMsg,
}

#[derive(Debug)]
pub enum ChannelMsg {
    ConnectToUser,

    UserBusy,
    ConnectToUserReject,
    ConnectToUserAccept,
}

impl SelfChannel {
    pub async fn new(username: ShortIdStr, channels: UserChannels) -> Self {
        let (tx, rx) = mpsc::channel(1);
        channels.add(username.clone(), tx).await;

        Self {
            i: Some(SelfChannelInner {
                username,
                channel: rx,
                channels,
            }),
        }
    }

    pub async fn hear(&mut self) -> ChannelMsgWithSender {
        let this = self.i.as_mut().expect("SelfChannel is dropped");

        this.channel
            .recv()
            .await
            .expect("unreachable: a sender should always be present in the users_channels map")
    }

    pub async fn listen(&mut self, to: &ShortIdStr) -> Option<ChannelMsg> {
        let this = self.i.as_mut().expect("SelfChannel is dropped");

        let assume_ignored = Instant::now() + Duration::from_secs(10);
        loop {
            match timeout_at(assume_ignored, this.channel.recv()).await {
                Ok(Some(msg)) if &msg.from == to => {
                    return Some(msg.message);
                }
                Ok(Some(msg)) => {
                    if let Some(other) = this.channels.get(&msg.from).await {
                        let _ = other.try_tell(&this.username, ChannelMsg::UserBusy);
                    }
                    continue;
                }
                Ok(None) => {
                    unreachable!("a sender should always be present in the users_channels map")
                }
                Err(_) => return None,
            }
        }
    }
}

impl Drop for SelfChannel {
    fn drop(&mut self) {
        if let Some(this) = self.i.take() {
            tracing::debug!("dropping user");
            tokio::spawn(async move { this.channels.remove(&this.username).await });
        }
    }
}

pub struct UserChannel(mpsc::Sender<ChannelMsgWithSender>);

impl UserChannel {
    pub async fn tell(&self, from: &ShortIdStr, value: ChannelMsg) -> Result<(), impl Error> {
        self.0
            .send(ChannelMsgWithSender {
                from: from.clone(),
                message: value,
            })
            .await
    }

    pub fn try_tell(&self, from: &ShortIdStr, value: ChannelMsg) -> Result<(), impl Error> {
        self.0.try_send(ChannelMsgWithSender {
            from: from.clone(),
            message: value,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UserChannels(Arc<RwLock<HashMap<ShortIdStr, mpsc::Sender<ChannelMsgWithSender>>>>);

impl Default for UserChannels {
    fn default() -> Self {
        Self::new()
    }
}

impl UserChannels {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub async fn get(&self, username: &ShortIdStr) -> Option<UserChannel> {
        self.0.read().await.get(username).cloned().map(UserChannel)
    }

    pub async fn is_online(&self, username: &ShortIdStr) -> bool {
        self.0.read().await.contains_key(username)
    }

    async fn add(&self, username: ShortIdStr, channel: mpsc::Sender<ChannelMsgWithSender>) {
        self.0.write().await.insert(username, channel);
    }

    async fn remove(&self, username: &ShortIdStr) {
        self.0.write().await.remove(username);
    }
}
