use crate::adapter::net;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryMessage();

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GeosphereMessage {
    pub position: glam::IVec3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Channel {
    Geosphere,
    CubeRegistry,
}

impl From<Channel> for u8 {
    fn from(p_value: Channel) -> Self {
        p_value.channel_id()
    }
}

impl Channel {
    const RESEND_TIME: std::time::Duration = std::time::Duration::from_millis(300);
    const MAX_GEOSPHERE_MESSAGE_COUNT: usize = 512;
    // TODO: Please make all channel size to be more than just one struct.

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
                max_memory_usage_bytes: std::mem::size_of::<GeosphereMessage>()
                    * Self::MAX_GEOSPHERE_MESSAGE_COUNT,
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
