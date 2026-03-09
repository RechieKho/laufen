use super::server;
use crate::adapter::shared;
use std::net;
use std::time;

pub type LargeParcelGroup = Vec<Option<server::LargeParcelData>>;
pub type SharedClientContext = shared::Shared<ClientContext>;
pub type LargeParcelGroupRegistry = rapidhash::RapidHashMap<u64, LargeParcelGroup>;

impl From<ClientContext> for SharedClientContext {
    fn from(p_value: ClientContext) -> Self {
        std::sync::Arc::new(std::sync::Mutex::new(p_value))
    }
}

pub struct ClientContext {
    client: renet::RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
    large_parcel_group_registry: LargeParcelGroupRegistry,
}

#[derive(partially::Partial, Default)]
#[partially(derive(Default))]
pub struct UnsecureClientContextBuilder {
    pub connection_config: server::ConnectionConfig,
    pub overriding_current_time: Option<time::Duration>,
    pub overriding_protocol_id: Option<u64>,
    pub user_data: Option<[u8; renet_netcode::NETCODE_USER_DATA_BYTES]>,
}

pub struct UnsecureClientContextBuilderParameters {
    pub client_address: net::SocketAddr,
    pub server_address: net::SocketAddr,
    pub client_id: u64,
}

impl UnsecureClientContextBuilder {
    pub fn build(self, p_parameters: UnsecureClientContextBuilderParameters) -> ClientContext {
        let client = renet::RenetClient::new(self.connection_config);

        // Setup transport layer using renet_netcode
        let socket = net::UdpSocket::bind(p_parameters.client_address).unwrap();
        let current_time = self.overriding_current_time.unwrap_or(
            time::SystemTime::now()
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap(),
        );
        let authentication = renet_netcode::ClientAuthentication::Unsecure {
            server_addr: p_parameters.server_address,
            client_id: p_parameters.client_id,
            user_data: self.user_data,
            protocol_id: self
                .overriding_protocol_id
                .unwrap_or(server::get_default_protocol_id()),
        };

        let transport =
            renet_netcode::NetcodeClientTransport::new(current_time, authentication, socket)
                .unwrap();

        ClientContext {
            client,
            transport,
            large_parcel_group_registry: Default::default(),
        }
    }
}

impl ClientContext {
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn is_connecting(&self) -> bool {
        self.client.is_connecting()
    }

    pub fn is_disconnected(&self) -> bool {
        self.client.is_disconnected()
    }

    pub fn disconnect(&mut self) {
        self.client.disconnect();
    }

    pub fn receive<C>(&mut self, p_channel_id: C) -> Option<server::Bytes>
    where
        C: Into<u8>,
    {
        self.client.receive_message(p_channel_id)
    }

    pub fn receive_deserializable<D, C>(&mut self, p_channel_id: C) -> Option<D>
    where
        D: serde::de::DeserializeOwned,
        C: Into<u8>,
    {
        self.receive(p_channel_id)
            .map(|p_bytes| postcard::from_bytes::<D>(&p_bytes).unwrap())
    }

    pub fn send<C, M>(&mut self, p_channel_id: C, p_message: M)
    where
        C: Into<u8>,
        M: Into<server::Bytes>,
    {
        self.client.send_message(p_channel_id, p_message);
    }

    pub fn send_serializable<C, S>(&mut self, p_channel_id: C, p_serializable: &S)
    where
        C: Into<u8>,
        S: serde::Serialize + ?Sized,
    {
        let message = Vec::<u8>::default();
        let message = postcard::to_extend(p_serializable, message).unwrap();
        self.send(p_channel_id, message);
    }

    pub fn update(
        &mut self,
        p_delta: time::Duration,
    ) -> anyhow::Result<(), server::NetcodeTransportError> {
        self.client.update(p_delta);
        self.transport.update(p_delta, &mut self.client)?;
        if self.client.is_connected() {
            self.transport.send_packets(&mut self.client)?;
        }
        Ok(())
    }

    pub fn receive_large<C>(&mut self, p_channel_id: C) -> Option<server::Bytes>
    where
        C: Into<u8>,
    {
        let p_channel_id = p_channel_id.into();
        while let Some(message) =
            self.receive_deserializable::<server::LargeParcelMessage, _>(p_channel_id)
        {
            match message {
                server::LargeParcelMessage::Metadata(metadata) => {
                    let parcel_group = vec![None; metadata.count as usize] as LargeParcelGroup;
                    assert!(self
                        .large_parcel_group_registry
                        .insert(metadata.key, parcel_group)
                        .is_none());
                }
                server::LargeParcelMessage::Body(body) => {
                    *self
                        .large_parcel_group_registry
                        .get_mut(&body.key)
                        .unwrap()
                        .get_mut(body.index as usize)
                        .unwrap() = Some(body.data);
                }
            };
        }

        let mut selected_key = None as Option<u64>;

        for (key, value) in self.large_parcel_group_registry.iter() {
            if value.iter().all(|p_parcel| p_parcel.is_some()) {
                selected_key = Some(*key);
                break;
            }
        }

        if let Some(selected_key) = selected_key {
            let completed_parcel_group = self
                .large_parcel_group_registry
                .remove(&selected_key)
                .unwrap();
            let mut message = Vec::<u8>::default();
            for parcel in completed_parcel_group {
                let parcel = parcel.unwrap();
                message.extend_from_slice(parcel.as_slice());
            }
            return Some(server::Bytes::from(message));
        }

        None
    }

    pub fn receive_large_deserializable<D, C>(&mut self, p_channel_id: C) -> Option<D>
    where
        D: serde::de::DeserializeOwned,
        C: Into<u8>,
    {
        self.receive_large(p_channel_id)
            .map(|p_bytes| postcard::from_bytes::<D>(&p_bytes).unwrap())
    }
}

pub struct ServerResponse {
    shared_client_context: SharedClientContext,
    channel: u8,
}

impl ServerResponse {
    pub fn new<C>(p_shared_client_context: SharedClientContext, p_channel: C) -> Self
    where
        C: Into<u8>,
    {
        Self {
            shared_client_context: p_shared_client_context,
            channel: p_channel.into(),
        }
    }
}

impl std::future::Future for ServerResponse {
    type Output = server::Bytes;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        p_cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(message) = self
            .shared_client_context
            .lock()
            .unwrap()
            .receive(self.channel)
        {
            return std::task::Poll::Ready(message);
        }

        if self.shared_client_context.lock().unwrap().is_disconnected() {
            return std::task::Poll::Ready(Self::Output::default());
        }

        p_cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}
