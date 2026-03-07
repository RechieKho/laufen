pub mod channel;

use super::consumer;
use crate::adapter::net;
use crate::adapter::shared;
use crate::app::biosphere;
use crate::app::geosphere;
use shipyard::IntoIter;

pub type SharedProvider = shared::Shared<Provider>;

pub struct Provider {
    server_context: net::server::ServerContext,
    #[allow(unused)]
    geosphere: geosphere::Geosphere,
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
    pub geosphere_builder: geosphere::GeosphereBuilder,
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
            geosphere_builder: geosphere::GeosphereBuilder::try_with_sample()?,
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
                let message = postcard::from_bytes::<channel::GeosphereMessage>(&message).unwrap();

                let (slot, cube) = self.geosphere.slot_cube(message.position);

                self.server_context.send_serializable(
                    id,
                    consumer::channel::Channel::Geosphere,
                    &consumer::channel::GeosphereMessage { slot: *slot, cube },
                );
            }

            if self
                .server_context
                .receive_from(id, channel::Channel::CubeRegistry)
                .is_some()
            {
                self.server_context.send_serializable(
                    id,
                    consumer::channel::Channel::Geosphere,
                    &consumer::channel::CubeRegistryMessage {
                        cube_registry: self.geosphere.cube_registry().clone(),
                    },
                );
            }
        }

        Ok(())
    }
}
