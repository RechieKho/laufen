use crate::app::geosphere;
use crate::adapter::net;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryMessage {
    pub cube_registry: geosphere::cube::CubeRegistry,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GeosphereMessage {
    pub slot: geosphere::slot::Slot,
    pub cube: Option<geosphere::cube::Cube>,
}

pub enum Channel {
    Geosphere,
    CubeRegistry
}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    const RESEND_TIME: std::time::Duration = std::time::Duration::from_millis(300);
    const MAX_GEOSPHERE_MESSAGE_COUNT: usize = 512;

    pub fn channel_id(&self) -> u8 {
        match self {
            Self::Geosphere => 0,
            Self::CubeRegistry => 1,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![
            net::server::ChannelConfig {
                channel_id: 0,
                max_memory_usage_bytes: std::mem::size_of::<GeosphereMessage>() * Self::MAX_GEOSPHERE_MESSAGE_COUNT,
                send_type: net::server::SendType::ReliableOrdered {
                    resend_time: Self::RESEND_TIME,
                },
            },
            net::server::ChannelConfig {
                channel_id: 1,
                max_memory_usage_bytes: std::mem::size_of::<CubeRegistryMessage>(),
                send_type: net::server::SendType::ReliableOrdered {
                    resend_time: Self::RESEND_TIME,
                },
            },
        ]
    }
}
