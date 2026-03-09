pub mod channel;

use super::provider;
use crate::adapter::net;
use crate::adapter::shared;
use crate::app::geosphere;
use crate::app::geosphere::morton::Morton;
use crate::app::geosphere::spectator;
use crate::app::viewport;
use crate::app::viewport::quad;

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct UnsecureConsumerBuilder {
    pub client_overriding_current_time: Option<std::time::Duration>,
    pub client_overriding_protocol_id: Option<u64>,
    pub client_user_data: Option<[u8; renet_netcode::NETCODE_USER_DATA_BYTES]>,
    pub connection_available_bytes_per_tick: u64,
}

impl Default for UnsecureConsumerBuilder {
    fn default() -> Self {
        Self {
            client_overriding_current_time: None,
            client_overriding_protocol_id: None,
            client_user_data: None,
            connection_available_bytes_per_tick: 60_000,
        }
    }
}

pub struct UnsecureConsumerBuilderParameters {
    pub client_context_builder_parameters: net::client::UnsecureClientContextBuilderParameters,
}

impl UnsecureConsumerBuilder {
    pub fn build(self, p_parameters: UnsecureConsumerBuilderParameters) -> Consumer {
        let client_context_builder = net::client::UnsecureClientContextBuilder {
            overriding_current_time: self.client_overriding_current_time,
            overriding_protocol_id: self.client_overriding_protocol_id,
            user_data: self.client_user_data,
            connection_config: net::server::ConnectionConfig {
                available_bytes_per_tick: self.connection_available_bytes_per_tick,
                server_channels_config: provider::channel::Channel::channels_config(),
                client_channels_config: channel::Channel::channels_config(),
            },
        };

        let shared_client_context = net::client::SharedClientContext::from(
            client_context_builder.build(p_parameters.client_context_builder_parameters),
        );

        // TODO: Need to fetch from provider.
        //let geosphere = geosphere::passive_geosphere::SharedPassiveGeosphere::default();
        //geosphere.lock().unwrap().cube_registry =
        //    geosphere::cube::CubeRegistryBuilder::try_with_sample()
        //        .unwrap()
        //        .build();

        Consumer {
            shared_client_context,
            shared_geosphere: Default::default(),
            spectator: spectator::Spectator::default(),
        }
    }
}

pub type SharedConsumer = shared::Shared<Consumer>;

pub struct Consumer {
    shared_client_context: net::client::SharedClientContext,
    shared_geosphere: geosphere::passive_geosphere::SharedPassiveGeosphere,
    spectator: spectator::Spectator,
}

impl super::pollable::Pollable for Consumer {
    fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.shared_client_context
            .lock()
            .unwrap()
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        {
            let mut client_context = self.shared_client_context.lock().unwrap();
            while let Some(message) = client_context
                .receive_large_deserializable::<provider::channel::PrepareMessage, _>(
                    provider::channel::Channel::PrepareMessage,
                )
            {
                self.shared_geosphere.lock().unwrap().cube_registry = message.cube_registry;
                client_context
                    .send_serializable(channel::Channel::Ready, &channel::ReadyMessage {});
            }
        }

        while let Some(message) = self
            .shared_client_context
            .lock()
            .unwrap()
            .receive_deserializable::<provider::channel::ChunkInsertionMessage, _>(
            provider::channel::Channel::ChunkInsertion,
        ) {
            let chunk_position = *message.key;
            let chunk_slot_point_cluster =
                geosphere::chunk::ChunkMorton::compute_cluster(chunk_position);
            self.shared_geosphere
                .lock()
                .unwrap()
                .chunk_map
                .insert(message.key, message.chunk);
            self.spectator
                .purge_slot_point_cluster(chunk_slot_point_cluster);
        }

        Ok(())
    }
}

impl Consumer {
    pub fn spectate(
        &mut self,
        p_abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        self.spectator.spectate_geosphere(
            self.shared_geosphere.clone(),
            p_abort_flag,
            p_camera_properties,
        )
    }

    pub fn disconnect(&mut self) {
        self.shared_client_context.lock().unwrap().disconnect();
    }
}
