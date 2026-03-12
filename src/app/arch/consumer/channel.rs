use crate::adapter::net;
use crate::adapter::net::channel_config_builder;
use crate::adapter::net::channel_config_builder::Message;
use crate::adapter::net::channel_config_builder::ReliableOrderedMessage;
use crate::adapter::net::channel_config_builder::ReliableUnorderedMessage;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ReadyMessage {}
impl Message for ReadyMessage {}
impl ReliableUnorderedMessage for ReadyMessage {}
impl ReliableOrderedMessage for ReadyMessage {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PlayerInputMessage {
    pub direction: glam::Vec3,
}
impl Message for PlayerInputMessage {
    const MAX_COUNT: usize = 1024;
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Channel {
    Ready,
    PlayerInput,
}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    pub fn channel_id(&self) -> u8 {
        match self {
            Self::Ready => 0,
            Self::PlayerInput => 1,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![
            channel_config_builder::channel_config_reliable_ordered::<ReadyMessage>(
                Channel::Ready.into(),
            ),
            channel_config_builder::channel_config_unreliable::<PlayerInputMessage>(
                Channel::PlayerInput.into(),
            ),
        ]
    }
}
