use crate::adapter::net;
use crate::adapter::net::channel_config_builder;
use crate::adapter::net::channel_config_builder::Message;
use crate::adapter::net::channel_config_builder::ReliableOrderedMessage;
use crate::adapter::net::channel_config_builder::ReliableUnorderedMessage;
use crate::app::geosphere;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PrepareMessage {
    pub cube_registry: geosphere::cube::CubeRegistry,
    pub chunk_insertions: Vec<ChunkInsertionMessage>,
}

impl Message for PrepareMessage {
    const MAX_COUNT: usize = 512;
}
impl ReliableUnorderedMessage for PrepareMessage {}
impl ReliableOrderedMessage for PrepareMessage {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChunkInsertionMessage {
    pub key: geosphere::chunk::ChunkKey,
    pub chunk: geosphere::chunk::Chunk,
}

impl Message for ChunkInsertionMessage {
    const MAX_COUNT: usize = 512;
}
impl ReliableUnorderedMessage for ChunkInsertionMessage {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChunkRemovalMessage {
    pub key: geosphere::chunk::ChunkKey,
}

impl Message for ChunkRemovalMessage {
    const MAX_COUNT: usize = 512;
}
impl ReliableUnorderedMessage for ChunkRemovalMessage {}

pub enum Channel {
    ChunkInsertion,
    PrepareMessage,
    ChunkRemoval,
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
            Self::ChunkRemoval => 2,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![
            channel_config_builder::channel_config_reliable_unordered::<ChunkInsertionMessage>(
                Self::ChunkInsertion.into(),
            ),
            channel_config_builder::channel_config_large::<PrepareMessage>(
                Self::PrepareMessage.into(),
            ),
            channel_config_builder::channel_config_reliable_unordered::<ChunkRemovalMessage>(
                Self::ChunkRemoval.into(),
            ),
        ]
    }
}
