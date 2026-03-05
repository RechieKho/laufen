use std::time::Duration;

use crate::{adapter::net, app::geosphere};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryInputMessage();

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryOutputMessage {
    pub cube_registry: geosphere::cube::CubeRegistry,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GeosphereInputMessage {
    pub position: glam::IVec3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GeosphereOutputMessage {
    pub slot: geosphere::slot::Slot,
    pub cube: Option<geosphere::cube::Cube>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum RelayChannel {
    Geosphere,
    CubeRegistry,
}

impl From<RelayChannel> for u8 {
    fn from(p_value: RelayChannel) -> Self {
        p_value.channel_id()
    }
}

impl RelayChannel {
    const RESEND_TIME: Duration = Duration::from_millis(300);

    pub fn channel_id(&self) -> u8 {
        match self {
            RelayChannel::Geosphere => 0,
            RelayChannel::CubeRegistry => 1,
        }
    }

    pub fn channels_config() -> Vec<net::server::ChannelConfig> {
        vec![
            net::server::ChannelConfig {
                channel_id: 0,
                max_memory_usage_bytes: std::cmp::max(
                    std::mem::size_of::<GeosphereInputMessage>(),
                    std::mem::size_of::<GeosphereOutputMessage>(),
                ),
                send_type: net::server::SendType::ReliableUnordered {
                    resend_time: Self::RESEND_TIME,
                },
            },
            net::server::ChannelConfig {
                channel_id: 1,
                max_memory_usage_bytes: std::cmp::max(
                    std::mem::size_of::<CubeRegistryInputMessage>(),
                    std::mem::size_of::<CubeRegistryOutputMessage>(),
                ),
                send_type: net::server::SendType::ReliableOrdered {
                    resend_time: Self::RESEND_TIME,
                },
            },
        ]
    }

    pub fn connection_config() -> net::server::ConnectionConfig {
        net::server::ConnectionConfig {
            available_bytes_per_tick: 60_000,
            server_channels_config: Self::channels_config(),
            client_channels_config: Self::channels_config(),
        }
    }
}
