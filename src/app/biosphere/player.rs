use crate::adapter::net;

#[derive(
    shipyard::Component,
    getset::Getters,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
)]
pub struct Player {
    #[getset(get = "pub")]
    client_id: net::server::ClientId,
}

impl Player {
    pub fn new(p_client_id: u64) -> Self {
        Self {
            client_id: p_client_id,
        }
    }
}
