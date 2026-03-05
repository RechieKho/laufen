use shipyard::IntoIter;

use super::relay;
use crate::adapter::net;
use crate::app::biosphere;
use crate::app::geosphere;

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
        })
    }

    pub fn build(self, p_parameters: ProviderBuilderParameters) -> Provider {
        let server_context_builder = net::server::ServerContextBuilder {
            max_client: self.server_max_client,
            overriding_current_time: self.server_overriding_current_time,
            overriding_protocol_id: self.server_overriding_protocol_id,
            authentication: self.server_authentication,
            connection_config: relay::RelayChannel::connection_config(),
        };

        Provider {
            server_context: server_context_builder
                .build(p_parameters.server_context_builder_parameters),
            geosphere: self.geosphere_builder.build(),
            biosphere: biosphere::Biosphere::default(),
        }
    }
}

impl Provider {
    pub fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
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
                .receive_from(id, relay::RelayChannel::Geosphere)
            {
                let message =
                    postcard::from_bytes::<relay::GeosphereInputMessage>(&message).unwrap();

                let (slot, cube) = self.geosphere.slot_cube(message.position);

                let output_message = Vec::<u8>::default();
                let output_message = postcard::to_extend(
                    &relay::GeosphereOutputMessage { slot: *slot, cube },
                    output_message,
                )
                .unwrap();

                self.server_context
                    .send(id, relay::RelayChannel::Geosphere, output_message);
            }

            if self
                .server_context
                .receive_from(id, relay::RelayChannel::CubeRegistry)
                .is_some()
            {
                let output_message = Vec::<u8>::default();
                let output_message = postcard::to_extend(
                    &relay::CubeRegistryOutputMessage {
                        cube_registry: self.geosphere.cube_registry().clone(),
                    },
                    output_message,
                )
                .unwrap();

                self.server_context
                    .send(id, relay::RelayChannel::CubeRegistry, output_message);
            }
        }

        Ok(())
    }
}
