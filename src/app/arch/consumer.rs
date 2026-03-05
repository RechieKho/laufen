use super::relay;
use crate::adapter::net;
use crate::app::geosphere::cube;
use crate::app::geosphere::spectator;

pub type SharedClientContext = std::sync::Arc<std::sync::Mutex<net::client::ClientContext>>;

impl From<net::client::ClientContext> for SharedClientContext {
    fn from(p_value: net::client::ClientContext) -> Self {
        std::sync::Arc::new(std::sync::Mutex::new(p_value))
    }
}

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
            is_cube_registry_requested: false,
            spectator: spectator::Spectator::default(),
            shared_client_context,
        }
    }
}

pub struct Consumer {
    is_cube_registry_requested: bool,
    #[allow(unused)]
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

    pub fn cube_registry(&mut self) -> Option<cube::CubeRegistry> {
        if !self.shared_client_context.lock().unwrap().is_connected() {
            return Default::default();
        }

        if !self.is_cube_registry_requested {
            let input_message = Vec::<u8>::default();
            let input_message =
                postcard::to_extend(&relay::CubeRegistryInputMessage(), input_message).unwrap();
            self.shared_client_context
                .lock()
                .unwrap()
                .send(relay::RelayChannel::CubeRegistry, input_message);
            self.is_cube_registry_requested = true;
            return Default::default();
        }

        if let Some(message) = self
            .shared_client_context
            .lock()
            .unwrap()
            .receive(relay::RelayChannel::CubeRegistry)
        {
            let message =
                postcard::from_bytes::<relay::CubeRegistryOutputMessage>(&message).unwrap();
            self.is_cube_registry_requested = false;
            return Some(message.cube_registry);
        }

        Default::default()
    }
}
