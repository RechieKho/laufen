use shipyard::IntoIter;

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
    pub server_context_builder: net::server::ServerContextBuilder,
    pub geosphere_builder: geosphere::GeosphereBuilder,
}

pub struct ProviderBuilderParameters {
    pub server_context_builder_parameters: net::server::ServerContextBuilderParameters,
}

impl ProviderBuilder {
    pub fn build(self, p_parameters: ProviderBuilderParameters) -> Provider {
        Provider {
            server_context: self
                .server_context_builder
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

        // TODO: Handle Relay requests and responses.

        Ok(())
    }
}
