pub mod channel;

use super::provider;
use crate::adapter::net;
use crate::adapter::renderer;
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
    pub poll_interval_duration: std::time::Duration,
    pub blocking_poll_interval_duration: std::time::Duration,
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
            spectator: Default::default(),
            preparation_notice: Default::default(),
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
            shared_spectator: poller.spectator.clone(),
            shared_preparation_notice: poller.preparation_notice.clone(),
            blocking_poll_abort_flag,
            poll_join_handle,
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
    shared_spectator: spectator::SharedSpectator,
    shared_preparation_notice: shared::Shared<Option<ConsumerPreparationNotice>>,

    blocking_poll_abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    poll_join_handle: tokio::task::JoinHandle<()>,
}

impl Consumer {
    pub fn spectate(
        &mut self,
        p_abort_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
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
}

#[derive(Clone)]
struct ConsumerPoller {
    pub client_context: net::client::SharedClientContext,
    pub passive_geosphere: geosphere::passive_geosphere::SharedPassiveGeosphere,
    pub preparation_notice: shared::Shared<Option<ConsumerPreparationNotice>>,
    pub spectator: spectator::SharedSpectator,
}

impl ConsumerPoller {
    pub fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.client_context
            .lock()
            .unwrap()
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        {
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
                self.passive_geosphere.lock().unwrap().cube_registry = message.cube_registry;
                client_context
                    .send_serializable(channel::Channel::Ready, &channel::ReadyMessage {});
            }
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
            self.passive_geosphere
                .lock()
                .unwrap()
                .chunk_map
                .remove(&message.key);
        }

        Ok(())
    }
}
