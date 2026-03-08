pub mod channel;

use super::consumer;
use crate::adapter::net;
use crate::adapter::shared;
use crate::app::biosphere;
use crate::app::geosphere;
use crate::app::geosphere::Geosphere;
use shipyard::IntoIter;

pub type SharedProvider = shared::Shared<Provider>;

pub struct Provider {
    server_context: net::server::ServerContext,
    #[allow(unused)]
    geosphere: geosphere::active_geosphere::ActiveGeosphere,
    biosphere: biosphere::Biosphere,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct ProviderBuilder {
    pub server_max_client: usize,
    pub server_overriding_current_time: Option<std::time::Duration>,
    pub server_overriding_protocol_id: Option<u64>,
    pub server_authentication: net::server::ServerAuthentication,
    pub connection_available_bytes_per_tick: u64,
    pub geosphere_builder: geosphere::active_geosphere::ActiveGeosphereBuilder,
}

pub struct ProviderBuilderParameters {
    pub server_context_builder_parameters: net::server::ServerContextBuilderParameters,
}

impl ProviderBuilder {
    pub fn try_with_sample() -> anyhow::Result<Self> {
        Ok(Self {
            server_max_client: 60,
            server_overriding_current_time: None,
            server_overriding_protocol_id: None,
            server_authentication: net::server::ServerAuthentication::Unsecure,
            geosphere_builder:
                geosphere::active_geosphere::ActiveGeosphereBuilder::try_with_sample()?,
            connection_available_bytes_per_tick: 60_000,
        })
    }

    pub fn build(self, p_parameters: ProviderBuilderParameters) -> Provider {
        let server_context_builder = net::server::ServerContextBuilder {
            max_client: self.server_max_client,
            overriding_current_time: self.server_overriding_current_time,
            overriding_protocol_id: self.server_overriding_protocol_id,
            authentication: self.server_authentication,
            connection_config: net::server::ConnectionConfig {
                available_bytes_per_tick: self.connection_available_bytes_per_tick,
                server_channels_config: channel::Channel::channels_config(),
                client_channels_config: consumer::channel::Channel::channels_config(),
            },
        };

        Provider {
            server_context: server_context_builder
                .build(p_parameters.server_context_builder_parameters),
            geosphere: self.geosphere_builder.build(),
            biosphere: biosphere::Biosphere::default(),
        }
    }
}

impl super::Pollable for Provider {
    fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.server_context
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        while let Some(event) = self.server_context.get_event() {
            match event {
                net::server::ServerEvent::ClientConnected { client_id } => {
                    self.biosphere
                        .entities_mut()
                        .add_entity(biosphere::player::Player::new(client_id));

                    // TODO: I think it wouldn't be too nice only give a chunk of the world only.
                    // This should be dynamically given based on the client's position.
                    // Moreover, it should keep a list of cluster that the player is registered to,
                    // So to only send to players the updates it need for the modified chunk.
                    let cluster = geosphere::point_cluster::PointCluster {
                        min: glam::IVec3::new(-5, -5, -5),
                        max: glam::IVec3::new(5, 5, 5),
                    };

                    for point in cluster {
                        let key = geosphere::chunk::ChunkKey::from(point);
                        let chunk = self.geosphere.chunk(&key).clone();
                        self.server_context.send_serializable(
                            client_id,
                            channel::Channel::ChunkInsertion,
                            &channel::ChunkInsertionMessage { chunk, key },
                        );
                    }
                }

                net::server::ServerEvent::ClientDisconnected {
                    client_id,
                    reason: _,
                } => self.biosphere.entities_mut().run(
                    |mut p_all_storages: shipyard::AllStoragesViewMut,
                     p_players: shipyard::View<biosphere::player::Player>| {
                        for (id, player) in p_players.iter().with_id() {
                            if *player.client_id() == client_id {
                                p_all_storages.delete_entity(id);
                            }
                        }
                    },
                ),
            }
        }

        for id in self.server_context.get_client_ids() {
            while let Some(message) = self
                .server_context
                .receive_from(id, channel::Channel::Geosphere)
            {
                let message =
                    postcard::from_bytes::<consumer::channel::GeosphereMessage>(&message).unwrap();

                let (slot, cube) = self.geosphere.slot_cube(message.position);

                self.server_context.send_serializable(
                    id,
                    channel::Channel::Geosphere,
                    &channel::GeosphereMessage { slot: *slot, cube },
                );
            }

            if self
                .server_context
                .receive_from(id, channel::Channel::CubeRegistry)
                .is_some()
            {
                self.server_context.send_serializable(
                    id,
                    channel::Channel::Geosphere,
                    &channel::CubeRegistryMessage {
                        cube_registry: self.geosphere.cube_registry().clone(),
                    },
                );
            }
        }

        Ok(())
    }
}
