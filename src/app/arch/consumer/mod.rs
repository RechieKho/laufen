pub mod camera_control;
pub mod channel;

use shipyard::IntoIter;

use super::provider;
use crate::adapter::net;
use crate::adapter::renderer;
use crate::adapter::shared;
use crate::app::biosphere;
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
    pub poll_interval_duration: std::time::Duration,
    pub blocking_poll_interval_duration: std::time::Duration,
    pub initial_camera_direction_rotation: glam::Vec2,
    pub initial_camera_sensitivity: f32,
}

impl Default for UnsecureConsumerBuilder {
    fn default() -> Self {
        Self {
            client_overriding_current_time: None,
            client_overriding_protocol_id: None,
            client_user_data: None,
            connection_available_bytes_per_tick: 60_000,
            poll_interval_duration: std::time::Duration::from_millis(16),
            blocking_poll_interval_duration: std::time::Duration::from_millis(16),
            initial_camera_direction_rotation: Default::default(),
            initial_camera_sensitivity: 7f32,
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

        let poller = ConsumerPoller {
            client_context: net::client::SharedClientContext::from(
                client_context_builder.build(p_parameters.client_context_builder_parameters),
            ),
            passive_geosphere: Default::default(),
            passive_biosphere: Default::default(),
            spectator: Default::default(),
            preparation_notice: Default::default(),
            ready: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            self_entity_id: Default::default(),
        };

        let poll_join_handle = {
            let mut poller = poller.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(self.poll_interval_duration);
                let mut last_timestamp = std::time::Instant::now();

                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                interval.tick().await;

                loop {
                    let current_timestamp = std::time::Instant::now();
                    let delta = current_timestamp.duration_since(last_timestamp);
                    last_timestamp = current_timestamp;
                    poller.poll(delta).unwrap();
                    interval.tick().await;
                }
            })
        };

        let blocking_poll_abort_flag =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        //{
        //    let blocking_poll_abort_flag = blocking_poll_abort_flag.clone();
        //    let mut poller = poller.clone();
        //    tokio::task::spawn_blocking(move || loop {
        //        if blocking_poll_abort_flag.load(std::sync::atomic::Ordering::Relaxed) {
        //            break;
        //        }
        //        poller
        //            .blocking_poll(self.blocking_poll_interval_duration)
        //            .unwrap();
        //        std::thread::sleep(self.blocking_poll_interval_duration);
        //    })
        //};

        Consumer {
            shared_client_context: poller.client_context.clone(),
            shared_passive_geosphere: poller.passive_geosphere.clone(),
            shared_passive_biosphere: poller.passive_biosphere.clone(),
            shared_spectator: poller.spectator.clone(),
            shared_preparation_notice: poller.preparation_notice.clone(),
            shared_self_entity_id: poller.self_entity_id.clone(),
            blocking_poll_abort_flag,
            poll_join_handle,
            ready: poller.ready.clone(),
            camera_direction_rotation: self.initial_camera_direction_rotation,
            camera_sensitivity: self.initial_camera_sensitivity,
        }
    }
}

pub struct ConsumerPreparationNotice {
    pub geosphere_texture_images: Vec<renderer::texture::TextureImage>,
}

pub type SharedConsumer = shared::Shared<Consumer>;

pub struct Consumer {
    shared_client_context: net::client::SharedClientContext,
    shared_passive_geosphere: geosphere::passive_geosphere::SharedPassiveGeosphere,
    shared_passive_biosphere: biosphere::passive_biosphere::SharedPassiveBiosphere,
    shared_spectator: spectator::SharedSpectator,
    shared_preparation_notice: shared::Shared<Option<ConsumerPreparationNotice>>,
    shared_self_entity_id: shared::Shared<Option<shipyard::EntityId>>,

    blocking_poll_abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    poll_join_handle: tokio::task::JoinHandle<()>,

    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    camera_direction_rotation: glam::Vec2,
    pub camera_sensitivity: f32,
}

pub struct CameraSpatial {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
}

impl Default for CameraSpatial {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            direction: glam::Vec3::NEG_Z,
        }
    }
}

impl Consumer {
    pub fn spectate(
        &mut self,
        p_abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        if !self.ready.load(std::sync::atomic::Ordering::Relaxed) {
            return Default::default();
        }

        self.shared_spectator.lock().unwrap().spectate_geosphere(
            self.shared_passive_geosphere.clone(),
            p_abort_flag,
            p_camera_properties,
        )
    }

    pub fn shut_down(&self) {
        self.poll_join_handle.abort();
        self.blocking_poll_abort_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.shared_client_context.lock().unwrap().disconnect();
    }

    pub fn take_preparation_notice(&mut self) -> Option<ConsumerPreparationNotice> {
        self.shared_preparation_notice.lock().unwrap().take()
    }

    pub fn poll_handle_input(
        &mut self,
        p_input: &winit_input_helper::WinitInputHelper,
        p_current_resolution: (u32, u32),
    ) {
        if !self.ready.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let (width, height) = p_current_resolution;

        let (yaw, pitch) = p_input.mouse_diff();
        self.camera_direction_rotation = glam::Vec2::new(
            (self.camera_direction_rotation.x - pitch / height as f32 * self.camera_sensitivity)
                .clamp(
                    -std::f32::consts::PI / 2f32 + 0.001,
                    std::f32::consts::PI / 2f32 - 0.001,
                ),
            (self.camera_direction_rotation.y - yaw / width as f32 * self.camera_sensitivity)
                % (std::f32::consts::TAU),
        );

        if let Ok(mut client_context) = self.shared_client_context.try_lock() {
            let x = if p_input.key_held(winit::keyboard::KeyCode::KeyD) {
                1f32
            } else {
                0f32
            } - if p_input.key_held(winit::keyboard::KeyCode::KeyA) {
                1f32
            } else {
                0f32
            };
            let y = if p_input.key_held(winit::keyboard::KeyCode::KeyE) {
                1f32
            } else {
                0f32
            } - if p_input.key_held(winit::keyboard::KeyCode::KeyQ) {
                1f32
            } else {
                0f32
            };
            let z = if p_input.key_held(winit::keyboard::KeyCode::KeyS) {
                1f32
            } else {
                0f32
            } - if p_input.key_held(winit::keyboard::KeyCode::KeyW) {
                1f32
            } else {
                0f32
            };

            let input_direction = glam::Vec3::new(x, y, z);
            let rotation = glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                self.camera_direction_rotation.y,
                self.camera_direction_rotation.x,
                0f32,
            );
            client_context.send_serializable(
                channel::Channel::PlayerInput,
                &channel::PlayerInputMessage {
                    direction: rotation * input_direction,
                },
            );
        }
    }

    pub fn camera_spatial(&self) -> CameraSpatial {
        if !self.ready.load(std::sync::atomic::Ordering::Relaxed) {
            return Default::default();
        }

        let entity_id = self.shared_self_entity_id.lock().unwrap().unwrap();
        let biosphere = self.shared_passive_biosphere.lock().unwrap();
        let spatial = biosphere
            .entities()
            .get::<&biosphere::spatial::Spatial>(entity_id)
            .unwrap();
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::YXZ,
            self.camera_direction_rotation.y,
            self.camera_direction_rotation.x,
            0f32,
        );
        CameraSpatial {
            origin: spatial.position,
            direction: rotation * glam::Vec3::NEG_Z,
        }
    }
}

#[derive(Clone)]
struct ConsumerPoller {
    pub client_context: net::client::SharedClientContext,
    pub passive_geosphere: geosphere::passive_geosphere::SharedPassiveGeosphere,
    pub passive_biosphere: biosphere::passive_biosphere::SharedPassiveBiosphere,
    pub preparation_notice: shared::Shared<Option<ConsumerPreparationNotice>>,
    pub spectator: spectator::SharedSpectator,
    pub ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub self_entity_id: shared::Shared<Option<shipyard::EntityId>>,
}

impl ConsumerPoller {
    pub fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.client_context
            .lock()
            .unwrap()
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        if !self.ready.load(std::sync::atomic::Ordering::Relaxed) {
            let mut client_context = self.client_context.lock().unwrap();
            while let Some(message) = client_context
                .receive_large_deserializable::<provider::channel::PrepareMessage, _>(
                    provider::channel::Channel::PrepareMessage,
                )
            {
                *self.preparation_notice.lock().unwrap() = Some(ConsumerPreparationNotice {
                    geosphere_texture_images: message
                        .cube_registry
                        .geosphere_serializable_texture_image()
                        .iter()
                        .map(|p_image| p_image.clone().to_texture_image().unwrap())
                        .collect::<Vec<renderer::texture::TextureImage>>(),
                });
                {
                    let mut geosphere = self.passive_geosphere.lock().unwrap();
                    let mut spectator = self.spectator.lock().unwrap();
                    geosphere.cube_registry = message.cube_registry;
                    for insertion in message.chunk_insertions {
                        let chunk_position = *insertion.key;
                        let chunk_slot_point_cluster =
                            geosphere::chunk::ChunkMorton::compute_cluster(chunk_position);
                        geosphere.chunk_map.insert(insertion.key, insertion.chunk);
                        spectator.purge_slot_point_cluster(chunk_slot_point_cluster);
                    }
                }
                let entity_id = self
                    .passive_biosphere
                    .lock()
                    .unwrap()
                    .entities_mut()
                    .add_entity((
                        message.self_insertion.player,
                        message.self_insertion.spatial,
                    ));

                client_context
                    .send_serializable(channel::Channel::Ready, &channel::ReadyMessage {});
                *self.self_entity_id.lock().unwrap() = Some(entity_id);
                self.ready.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            return Ok(());
        }

        while let Some(message) = self
            .client_context
            .lock()
            .unwrap()
            .receive_deserializable::<provider::channel::ChunkInsertionMessage, _>(
            provider::channel::Channel::ChunkInsertion,
        ) {
            let chunk_position = *message.key;
            let chunk_slot_point_cluster =
                geosphere::chunk::ChunkMorton::compute_cluster(chunk_position);
            self.passive_geosphere
                .lock()
                .unwrap()
                .chunk_map
                .insert(message.key, message.chunk);
            self.spectator
                .lock()
                .unwrap()
                .purge_slot_point_cluster(chunk_slot_point_cluster);
        }

        while let Some(message) = self
            .client_context
            .lock()
            .unwrap()
            .receive_deserializable::<provider::channel::ChunkRemovalMessage, _>(
            provider::channel::Channel::ChunkRemoval,
        ) {
            let chunk_position = *message.key;
            let chunk_slot_point_cluster =
                geosphere::chunk::ChunkMorton::compute_cluster(chunk_position);
            self.passive_geosphere
                .lock()
                .unwrap()
                .chunk_map
                .remove(&message.key);
            self.spectator
                .lock()
                .unwrap()
                .purge_slot_point_cluster(chunk_slot_point_cluster);
        }

        {
            let mut player_updates = std::collections::BTreeMap::<
                biosphere::player::Player,
                biosphere::spatial::Spatial,
            >::default();
            while let Some(message) = self
                .client_context
                .lock()
                .unwrap()
                .receive_deserializable::<provider::channel::PlayerUpdateMessage, _>(
                provider::channel::Channel::PlayerUpdate,
            ) {
                player_updates.insert(message.player, message.spatial);
            }

            self.passive_biosphere.lock().unwrap().entities_mut().run(
                |p_players: shipyard::View<biosphere::player::Player>,
                 mut p_spatials: shipyard::ViewMut<biosphere::spatial::Spatial>| {
                    for (player, spatial) in (&p_players, &mut p_spatials).iter() {
                        if let Some(updated_spatial) = player_updates.remove(player) {
                            *spatial = updated_spatial;
                        }
                    }
                },
            )
        }

        Ok(())
    }
}
