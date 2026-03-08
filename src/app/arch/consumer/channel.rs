use crate::adapter::net;
use crate::app::arch::channel_config_builder;
use crate::app::arch::channel_config_builder::Message;
use crate::app::arch::channel_config_builder::ReliableOrderedMessage;
use crate::app::arch::channel_config_builder::ReliableUnorderedMessage;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ReadyMessage {}
impl Message for ReadyMessage {
    const MAX_COUNT: usize = 0;
}
impl ReliableUnorderedMessage for ReadyMessage {}
impl ReliableOrderedMessage for ReadyMessage {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Channel {
    Ready,
}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    pub fn channel_id(&self) -> u8 {
        match self {
            Self::Ready => u8::MAX,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![channel_config_builder::channel_config_reliable_ordered::<
            ReadyMessage,
        >(Channel::Ready.into())]
    }
}
