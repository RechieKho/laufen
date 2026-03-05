use super::relay;
use crate::adapter::net;
use crate::app::geosphere::cube;
use crate::app::geosphere::spectator;
use crate::app::viewport;
use crate::app::viewport::quad;

pub type SharedClientContext = std::sync::Arc<std::sync::Mutex<net::client::ClientContext>>;

impl From<net::client::ClientContext> for SharedClientContext {
    fn from(p_value: net::client::ClientContext) -> Self {
        std::sync::Arc::new(std::sync::Mutex::new(p_value))
    }
}

pub struct ProviderResponse {
    shared_client_context: SharedClientContext,
    channel: relay::RelayChannel,
}

impl ProviderResponse {
    pub fn new(
        p_shared_client_context: SharedClientContext,
        p_channel: relay::RelayChannel,
    ) -> Self {
        Self {
            shared_client_context: p_shared_client_context,
            channel: p_channel,
        }
    }
}

impl std::future::Future for ProviderResponse {
    type Output = net::server::Bytes;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        p_cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(message) = self
            .shared_client_context
            .lock()
            .unwrap()
            .receive(self.channel.clone())
        {
            return std::task::Poll::Ready(message);
        }

        p_cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}

#[derive(partially::Partial, Default)]
#[partially(derive(Default))]
pub struct UnsecureConsumerBuilder {
    pub client_overriding_current_time: Option<std::time::Duration>,
    pub client_overriding_protocol_id: Option<u64>,
    pub client_user_data: Option<[u8; renet_netcode::NETCODE_USER_DATA_BYTES]>,
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
            connection_config: relay::RelayChannel::connection_config(),
        };

        let shared_client_context = SharedClientContext::from(
            client_context_builder.build(p_parameters.client_context_builder_parameters),
        );

        Consumer {
            spectator: spectator::Spectator::default(),
            shared_client_context,
        }
    }
}

pub struct Consumer {
    spectator: spectator::Spectator,
    shared_client_context: SharedClientContext,
}

impl Consumer {
    pub fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()> {
        self.shared_client_context
            .lock()
            .unwrap()
            .update(p_delta)
            .map_err(anyhow::Error::msg)?;

        Ok(())
    }

    pub fn slot_cube_proxy(&mut self) -> impl spectator::SlotCubeProxy + Clone {
        {
            let shared_client_context = self.shared_client_context.clone();

            async move |p_position: glam::IVec3| {
                let input_message = Vec::<u8>::default();
                let input_message = postcard::to_extend(
                    &relay::GeosphereInputMessage {
                        position: p_position,
                    },
                    input_message,
                )
                .unwrap();
                shared_client_context
                    .lock()
                    .unwrap()
                    .send(relay::RelayChannel::Geosphere, input_message);

                let message = ProviderResponse::new(
                    shared_client_context.clone(),
                    relay::RelayChannel::Geosphere,
                )
                .await;

                let output_message =
                    postcard::from_bytes::<relay::GeosphereOutputMessage>(&message).unwrap();

                (output_message.slot, output_message.cube)
            }
        }
    }

    pub async fn cube_registry(&mut self) -> Option<cube::CubeRegistry> {
        if !self.shared_client_context.lock().unwrap().is_connected() {
            return Default::default();
        }

        let input_message = Vec::<u8>::default();
        let input_message =
            postcard::to_extend(&relay::CubeRegistryInputMessage(), input_message).unwrap();
        self.shared_client_context
            .lock()
            .unwrap()
            .send(relay::RelayChannel::CubeRegistry, input_message);

        let message = ProviderResponse::new(
            self.shared_client_context.clone(),
            relay::RelayChannel::CubeRegistry,
        )
        .await;

        Some(
            postcard::from_bytes::<relay::CubeRegistryOutputMessage>(&message)
                .unwrap()
                .cube_registry,
        )
    }

    pub fn spectate(
        &mut self,
        p_camera_properties: &viewport::ViewportCameraProperties,
    ) -> Vec<quad::QuadInstance> {
        let proxy = self.slot_cube_proxy();
        self.spectator.spectate(proxy, p_camera_properties)
    }
}
