use crate::adapter::net;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Channel {}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    // TODO: Please make all channel size to be more than just one struct.

    pub fn channel_id(&self) -> u8 {
        0
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![]
    }
}
