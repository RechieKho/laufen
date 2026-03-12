pub mod channel;

use super::consumer;
use crate::adapter::net;
use crate::adapter::shared;
use crate::app::biosphere;
use crate::app::geosphere;
use crate::app::geosphere::morton::Morton;
use itertools::Itertools;
use shipyard::IntoIter;

pub type SharedProvider = shared::Shared<Provider>;

type ChunkSubscription = rapidhash::RapidHashMap<net::server::ClientId, Vec<glam::IVec3>>;
type SharedChunkSubscription = shared::Shared<ChunkSubscription>;
type ReadyClientSet = rapidhash::RapidHashSet<net::server::ClientId>;
type SharedReadyClientSet = shared::Shared<ReadyClientSet>;

pub struct Provider {
    poll_join_handle: tokio::task::JoinHandle<()>,
    abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
    pub poll_interval_duration: std::time::Duration,
    pub blocking_poll_interval_duration: std::time::Duration,
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
            poll_interval_duration: std::time::Duration::from_millis(16),
            blocking_poll_interval_duration: std::time::Duration::from_millis(16),
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

        let abort_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let poller = ProviderPoller {
            server_context: shared::share(
                server_context_builder.build(p_parameters.server_context_builder_parameters),
            ),
            active_geosphere: shared::share(self.geosphere_builder.build()),
            active_biosphere: Default::default(),
            chunk_subscription: Default::default(),
            client_active_chunk_range_radius: std::sync::Arc::new(
                std::sync::atomic::AtomicU32::new(self.client_active_chunk_range_radius),
            ),
            ready_client_set: Default::default(),
            chunk_insertion_queue: Default::default(),
            chunk_removal_queue: Default::default(),
            abort_flag: abort_flag.clone(),
        };

        let poll_join_handle = {
            let mut poller = poller.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(self.poll_interval_duration);

                loop {
                    poller.poll(self.poll_interval_duration).unwrap();
                    interval.tick().await;
                }
            })
        };

        {
            let abort_flag = abort_flag.clone();
            let mut poller = poller.clone();
            tokio::task::spawn_blocking(move || loop {
                if abort_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                poller
                    .blocking_poll(self.blocking_poll_interval_duration)
                    .unwrap();
                std::thread::sleep(self.blocking_poll_interval_duration);
            })
        };

        Provider {
            poll_join_handle,
            abort_flag,
        }
    }
}

impl Provider {
    pub fn shut_down(&self) {
        self.poll_join_handle.abort();
        self.abort_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

struct SendInstruction<M: serde::Serialize> {
    pub client_id: net::server::ClientId,
    pub message: M,
}
type SharedSendInstructionQueue<M> = shared::Shared<std::collections::VecDeque<SendInstruction<M>>>;

#[derive(Clone)]
struct ProviderPoller {
    pub server_context: net::server::SharedServerContext,
    pub active_geosphere: geosphere::active_geosphere::SharedActiveGeosphere,
    pub active_biosphere: biosphere::active_biosphere::SharedActiveBiosphere,
    pub chunk_subscription: SharedChunkSubscription,
    pub ready_client_set: SharedReadyClientSet,
    pub client_active_chunk_range_radius: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub chunk_insertion_queue: SharedSendInstructionQueue<channel::ChunkInsertionMessage>,
    pub chunk_removal_queue: SharedSendInstructionQueue<channel::ChunkRemovalMessage>,
    pub abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn compute_active_chunk_point_cluster(
    p_slot_position: glam::IVec3,
    p_active_chunk_range_radius: std::sync::Arc<std::sync::atomic::AtomicU32>,
) -> geosphere::point_cluster::PointCluster {
    let (chunk_position, _) = geosphere::chunk::ChunkMorton::compute_coordinate(p_slot_position);

    geosphere::point_cluster::PointCluster {
        min: chunk_position
            - p_active_chunk_range_radius.load(std::sync::atomic::Ordering::Relaxed) as i32,
        max: chunk_position
            + p_active_chunk_range_radius.load(std::sync::atomic::Ordering::Relaxed) as i32,
    }
}

impl ProviderPoller {
    const MAX_QUEUE_SEND_PER_POLL: u16 = 32;

    pub fn blocking_poll(&mut self, _p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.active_biosphere.lock().unwrap().entities().run(
            |p_players: shipyard::View<biosphere::player::Player>,
             p_spatials: shipyard::View<biosphere::spatial::Spatial>| {
                for (player, spatial) in (&p_players, &p_spatials).iter() {
                    if !self
                        .ready_client_set
                        .lock()
                        .unwrap()
                        .contains(player.client_id())
                    {
                        continue;
                    }

                    let slot_position = glam::IVec3::new(
                        spatial.position.x as i32,
                        spatial.position.y as i32,
                        spatial.position.z as i32,
                    );
                    let active_chunk_point_cluster = compute_active_chunk_point_cluster(
                        slot_position,
                        self.client_active_chunk_range_radius.clone(),
                    );

                    {
                        let mut chunk_subscription = self.chunk_subscription.lock().unwrap();
                        let player_chunk_subscription =
                            chunk_subscription.entry(*player.client_id()).or_default();
                        let chunk_point_set = active_chunk_point_cluster
                            .into_iter()
                            .collect::<rapidhash::RapidHashSet<glam::IVec3>>();
                        let obsolete_chunk_points = player_chunk_subscription
                            .iter()
                            .copied()
                            .filter(|p_chunk_position| !chunk_point_set.contains(p_chunk_position));
                        let obsolete_chunk_point_set = obsolete_chunk_points
                            .clone()
                            .collect::<rapidhash::RapidHashSet<glam::IVec3>>(
                        );
                        for obsolete_point in obsolete_chunk_points {
                            self.chunk_removal_queue
                                .lock()
                                .unwrap()
                                .push_back(SendInstruction {
                                    client_id: *player.client_id(),
                                    message: channel::ChunkRemovalMessage {
                                        key: geosphere::chunk::ChunkKey::from(obsolete_point),
                                    },
                                });
                        }
                        player_chunk_subscription.retain(|p_chunk_point| {
                            !obsolete_chunk_point_set.contains(p_chunk_point)
                        });
                    }

                    {
                        let mut chunk_subscription = self.chunk_subscription.lock().unwrap();
                        let player_chunk_subscription =
                            chunk_subscription.entry(*player.client_id()).or_default();
                        for new_point in active_chunk_point_cluster.into_iter() {
                            if player_chunk_subscription.contains(&new_point) {
                                continue;
                            }
                            let key = geosphere::chunk::ChunkKey::from(new_point);
                            let chunk = self.active_geosphere.lock().unwrap().chunk(&key).clone();
                            self.chunk_insertion_queue
                                .lock()
                                .unwrap()
                                .push_back(SendInstruction {
                                    client_id: *player.client_id(),
                                    message: channel::ChunkInsertionMessage { chunk, key },
                                });
                            player_chunk_subscription.push(new_point);
                        }
                    }
                }
            },
        );

        Ok(())
    }

    pub fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.server_context
            .lock()
            .unwrap()
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        if let Ok(mut server_context) = self.server_context.try_lock() {
            while let Some(event) = server_context.get_event() {
                match event {
                net::server::ServerEvent::ClientConnected { client_id } => {
                    log::info!("Client {} joined.", client_id);
                    let player  = biosphere::player::Player::new(client_id);
                    let spatial = biosphere::spatial::Spatial::default();

                    self.active_biosphere.lock().unwrap().entities_mut().add_entity(( player.clone(), spatial.clone()));

                    {
                        let server_context = self.server_context.clone();
                        let active_chunk_range_radius = self.client_active_chunk_range_radius.clone();
                        let chunk_subscription = self.chunk_subscription.clone();
                        let active_geosphere = self.active_geosphere.clone();
                        let abort_flag = self.abort_flag.clone();
                        tokio::task::spawn_blocking(move || {
                            let initial_slot_position = glam::IVec3::new(
                                spatial.position.x as i32,
                                spatial.position.y as i32,
                                spatial.position.z as i32,
                            );

                            let mut chunk_insertions = Vec::<channel::ChunkInsertionMessage>::default();

                            for new_point in compute_active_chunk_point_cluster(initial_slot_position, active_chunk_range_radius) {
                                if abort_flag.load(std::sync::atomic::Ordering::Relaxed) {
                                    return;
                                }

                                let key = geosphere::chunk::ChunkKey::from(new_point);
                                let chunk = active_geosphere.lock().unwrap().chunk(&key).clone();
                                chunk_insertions.push(channel::ChunkInsertionMessage{
                                    key,
                                    chunk
                                });
                            }

                            {
                                let mut chunk_subscription = chunk_subscription.lock().unwrap();
                                let player_chunk_subscription =
                                    chunk_subscription.entry(client_id).or_default();
                                chunk_insertions.iter().for_each(|p_message| player_chunk_subscription.push(*p_message.key));
                            }

                            server_context.lock().unwrap().send_large_serializable(
                                client_id,
                                channel::Channel::PrepareMessage,
                                &channel::PrepareMessage {
                                    cube_registry: active_geosphere.lock().unwrap().cube_registry().clone(),
                                    chunk_insertions,
                                    self_insertion: channel::PlayerInsertionMessage { player, spatial }
                                },
                            );

                            log::info!("Client {} preparation sent.", client_id);
                        });
                    }

                }

                net::server::ServerEvent::ClientDisconnected {
                    client_id,
                    reason: _,
                } => self.active_biosphere.lock().unwrap().entities_mut().run(
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
        }

        if let Ok(mut server_context) = self.server_context.try_lock() {
            for client_id in server_context.get_client_ids() {
                while let Some(_message) = server_context
                    .receive_deserializable_from::<consumer::channel::ReadyMessage, _>(
                        client_id,
                        consumer::channel::Channel::Ready,
                    )
                {
                    self.ready_client_set.lock().unwrap().insert(client_id);
                    log::info!("Client {} ready.", client_id);
                }
            }
        }

        let mut queue_send_count = 0u16;

        while let Ok(mut queue) = self.chunk_insertion_queue.try_lock() {
            if queue_send_count > Self::MAX_QUEUE_SEND_PER_POLL {
                break;
            }
            if let Some(instruction) = queue.pop_front() {
                self.server_context.lock().unwrap().send_serializable(
                    instruction.client_id,
                    channel::Channel::ChunkInsertion,
                    &instruction.message,
                );
                queue_send_count += 1;
            } else {
                break;
            }
        }

        while let Ok(mut queue) = self.chunk_removal_queue.try_lock() {
            if queue_send_count > Self::MAX_QUEUE_SEND_PER_POLL {
                break;
            }
            if let Some(instruction) = queue.pop_front() {
                self.server_context.lock().unwrap().send_serializable(
                    instruction.client_id,
                    channel::Channel::ChunkRemoval,
                    &instruction.message,
                );
                queue_send_count += 1;
            } else {
                break;
            }
        }

        {
            let client_ids = {
                let ready_client_set = self.ready_client_set.lock().unwrap();
                self.server_context
                    .lock()
                    .unwrap()
                    .get_client_ids()
                    .into_iter()
                    .filter(|p_client_id| ready_client_set.contains(p_client_id))
                    .collect_vec()
            };
            self.active_biosphere.lock().unwrap().entities().run(
                |p_players: shipyard::View<biosphere::player::Player>,
                 p_spatials: shipyard::View<biosphere::spatial::Spatial>| {
                    for (player, spatial) in (&p_players, &p_spatials).iter() {
                        for client_id in client_ids.iter() {
                            self.server_context.lock().unwrap().send_serializable(
                                *client_id,
                                channel::Channel::PlayerUpdate,
                                &channel::PlayerUpdateMessage {
                                    player: player.clone(),
                                    spatial: spatial.clone(),
                                },
                            );
                        }
                    }
                },
            );
        }

        {
            let mut player_inputs = std::collections::BTreeMap::<
                net::server::ClientId,
                consumer::channel::PlayerInputMessage,
            >::default();
            if let Ok(mut server_context) = self.server_context.try_lock() {
                for client_id in server_context.get_client_ids() {
                    while let Some(message) = server_context
                        .receive_deserializable_from::<consumer::channel::PlayerInputMessage, _>(
                            client_id,
                            consumer::channel::Channel::PlayerInput,
                        )
                    {
                        player_inputs.insert(client_id, message);
                    }
                }
            }
            if !player_inputs.is_empty() {
                self.active_biosphere.lock().unwrap().entities_mut().run(|p_players: shipyard::View<biosphere::player::Player>, mut p_spatials: shipyard::ViewMut<biosphere::spatial::Spatial>| {
                    for (player, spatial) in (&p_players, &mut p_spatials).iter() {
                        if let Some(input) = player_inputs.remove(player.client_id()) {
                            if let Some(normalized) = input.direction.try_normalize() {
                                spatial.direction = normalized;
                                spatial.position += normalized * 1f32;
                            }
                        }
                    }
                });
            }
        }

        Ok(())
    }
}
