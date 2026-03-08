use crate::adapter::net;
use crate::app::arch::channel_config_builder;
use crate::app::arch::channel_config_builder::Message;
use crate::app::arch::channel_config_builder::ReliableUnorderedMessage;
use crate::app::geosphere;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PrepareMessage {
    pub cube_registry: geosphere::cube::CubeRegistry,
}

impl Message for PrepareMessage {}
impl ReliableUnorderedMessage for PrepareMessage {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChunkInsertionMessage {
    pub key: geosphere::chunk::ChunkKey,
    pub chunk: geosphere::chunk::Chunk,
}

impl Message for ChunkInsertionMessage {
    const MAX_COUNT: usize = 512;
}
impl ReliableUnorderedMessage for ChunkInsertionMessage {}

pub enum Channel {
    ChunkInsertion,
    PrepareMessage,
}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    pub fn channel_id(&self) -> u8 {
        match self {
            Self::ChunkInsertion => 0,
            Self::PrepareMessage => 1,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![
            channel_config_builder::channel_config_reliable_unordered::<ChunkInsertionMessage>(
                Self::ChunkInsertion.into(),
            ),
            channel_config_builder::channel_config_reliable_unordered::<PrepareMessage>(
                Self::PrepareMessage.into(),
            ),
        ]
    }
}
