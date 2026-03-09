pub mod channel;

use super::consumer;
use crate::adapter::net;
use crate::adapter::shared;
use crate::app::biosphere;
use crate::app::geosphere;
use crate::app::geosphere::morton::Morton;
use shipyard::IntoIter;

pub type SharedProvider = shared::Shared<Provider>;

type ChunkSubscription = rapidhash::RapidHashMap<net::server::ClientId, Vec<glam::IVec3>>;
type ReadyClientSet = rapidhash::RapidHashSet<net::server::ClientId>;

pub struct Provider {
    server_context: net::server::ServerContext,
    #[allow(unused)]
    geosphere: geosphere::active_geosphere::ActiveGeosphere,
    biosphere: biosphere::Biosphere,
    chunk_subscription: ChunkSubscription,
    client_active_chunk_range_radius: u32,
    ready_client_set: ReadyClientSet,
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
    pub client_active_chunk_range_radius: u32,
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
            client_active_chunk_range_radius: 5,
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
            ..Default::default()
        };

        Provider {
            server_context: server_context_builder
                .build(p_parameters.server_context_builder_parameters),
            geosphere: self.geosphere_builder.build(),
            biosphere: Default::default(),
            chunk_subscription: Default::default(),
            client_active_chunk_range_radius: self.client_active_chunk_range_radius,
            ready_client_set: Default::default(),
        }
    }
}

impl Provider {
    fn handle_client_subscription(&mut self) {
        self.biosphere.entities().run(
            |p_players: shipyard::View<biosphere::player::Player>,
             p_spatials: shipyard::View<biosphere::spatial::Spatial>| {
                for (player, spatial) in (&p_players, &p_spatials).iter() {
                    if !self.ready_client_set.contains(player.client_id()) {
                        continue;
                    }

                    let slot_position = glam::IVec3::new(
                        spatial.position.x as i32,
                        spatial.position.y as i32,
                        spatial.position.z as i32,
                    );
                    let (chunk_position, _) =
                        geosphere::chunk::ChunkMorton::compute_coordinate(slot_position);

                    let chunk_point_cluster = geosphere::point_cluster::PointCluster {
                        min: chunk_position - self.client_active_chunk_range_radius as i32,
                        max: chunk_position + self.client_active_chunk_range_radius as i32,
                    };
                    let player_chunk_subscription = self
                        .chunk_subscription
                        .entry(*player.client_id())
                        .or_default();

                    // TODO: It is too much, it causes the server update to lag for too long, causing
                    // disconnection. Got to find a way to separate this.

                    //{
                    //    let chunk_point_set = chunk_point_cluster
                    //        .into_iter()
                    //        .collect::<rapidhash::RapidHashSet<glam::IVec3>>();
                    //    let obsolete_chunk_points = player_chunk_subscription
                    //        .iter()
                    //        .copied()
                    //        .filter(|p_chunk_position| !chunk_point_set.contains(p_chunk_position));
                    //    let obsolete_chunk_point_set = obsolete_chunk_points
                    //        .clone()
                    //        .collect::<rapidhash::RapidHashSet<glam::IVec3>>(
                    //    );
                    //    for obsolete_point in obsolete_chunk_points {
                    //        self.server_context.send_serializable(
                    //            *player.client_id(),
                    //            channel::Channel::ChunkRemoval,
                    //            &channel::ChunkRemovalMessage {
                    //                key: geosphere::chunk::ChunkKey::from(obsolete_point),
                    //            },
                    //        );
                    //    }
                    //    player_chunk_subscription.retain(|p_chunk_point| {
                    //        obsolete_chunk_point_set.contains(p_chunk_point)
                    //    });
                    //}

                    {
                        for new_point in chunk_point_cluster.into_iter() {
                            if player_chunk_subscription.contains(&new_point) {
                                continue;
                            }

                            let key = geosphere::chunk::ChunkKey::from(new_point);
                            let chunk = self.geosphere.chunk(&key).clone();
                            self.server_context.send_serializable(
                                *player.client_id(),
                                channel::Channel::ChunkInsertion,
                                &channel::ChunkInsertionMessage { chunk, key },
                            );
                            player_chunk_subscription.push(new_point);
                        }
                    }
                }
            },
        )
    }
}

impl super::pollable::Pollable for Provider {
    fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.server_context
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        while let Some(event) = self.server_context.get_event() {
            match event {
                net::server::ServerEvent::ClientConnected { client_id } => {
                    self.biosphere.entities_mut().add_entity((
                        biosphere::player::Player::new(client_id),
                        biosphere::spatial::Spatial::default(),
                    ));

                    self.server_context.send_large_serializable(
                        client_id,
                        channel::Channel::PrepareMessage,
                        &channel::PrepareMessage {
                            cube_registry: self.geosphere.cube_registry().clone(),
                        },
                    );
                    log::info!("Client {} joined.", client_id);
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

        for client_id in self.server_context.get_client_ids() {
            while let Some(_message) = self
                .server_context
                .receive_deserializable_from::<consumer::channel::ReadyMessage, _>(
                    client_id,
                    consumer::channel::Channel::Ready,
                )
            {
                self.ready_client_set.insert(client_id);
                log::info!("Client {} ready.", client_id);
            }
        }

        self.handle_client_subscription();

        Ok(())
    }
}
