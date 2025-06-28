pub mod legos;

mod axum;

#[cfg(feature = "axum")]
pub use axum::Schemou;
pub use sirius::Sirius;
pub use sirius::SiriusError;

const AUTH_SIZE: usize = 2048;

#[derive(Sirius, Debug)]
pub struct C2RRegister {
    pub username: legos::ShortIdStr,
    // TODO: All schemou types should be Hardened and have explicit invarients, no generic de-serialization
    // labels: enhancement, help wanted
    // Issue URL: https://github.com/Colabie/Colabie/issues/21
    pub pubkey: Box<[u8]>,
}

#[derive(Sirius, Debug)]
pub struct R2CRegister {
    pub commit_id: Box<[u8]>,
}

#[derive(Sirius, Debug)]
pub struct C2SAck {
    pub username: legos::ShortIdStr,
}

#[derive(Sirius, Debug)]
pub struct S2CAuthReq {
    pub random: [u8; AUTH_SIZE],
}

#[derive(Sirius, Debug)]
pub struct C2SAuthRes {
    pub signed_random: [u8; AUTH_SIZE],
}

#[derive(Sirius, Debug)]
pub enum S2CAuthResult {
    Success,
    Failure,
}

#[derive(Sirius, Debug)]
pub struct ConnectToUser {
    // TODO: Send WebRTC offer
    // Issue URL: https://github.com/Colabie/Colabie/issues/73
    pub username: legos::ShortIdStr,
}

#[derive(Sirius, Debug)]
pub enum S2CConnectToUserResult {
    UserBusy,
    Reject,
    Accept,
}

#[derive(Sirius, Debug)]
pub enum C2SConnectToUserResult {
    Reject,
    Accept,
}
