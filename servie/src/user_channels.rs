use schemou::legos::ShortIdStr;

use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc, RwLock},
    time::timeout,
};

pub struct SelfChannel {
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
            username,
            channel: rx,
            channels,
        }
    }

    pub async fn hear(&mut self) -> ChannelMsgWithSender {
        self.channel
            .recv()
            .await
            .expect("unreachable: a sender should always be present in the users_channels map")
    }

    pub async fn listen(&mut self, to: &ShortIdStr) -> Option<ChannelMsg> {
        loop {
            // FIXME: The timer resets even if the message is not from the expected user
            match timeout(Duration::from_secs(10), self.channel.recv()).await {
                Ok(Some(msg)) if &msg.from == to => {
                    return Some(msg.message);
                }
                Ok(Some(msg)) => {
                    if let Some(other) = self.channels.get(&msg.from).await {
                        let _ = other.try_tell(&self.username, ChannelMsg::UserBusy);
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
        tokio::runtime::Handle::current().block_on(self.channels.remove(&self.username));
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

#[derive(Clone)]
pub struct UserChannels(Arc<RwLock<HashMap<ShortIdStr, mpsc::Sender<ChannelMsgWithSender>>>>);

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
