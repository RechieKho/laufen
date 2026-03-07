use std::hash::Hash;
use std::hash::Hasher;
use std::net;
use std::time;

pub use renet::*;
pub use renet_netcode::*;

pub struct ServerContext {
    server: renet::RenetServer,
    transport: renet_netcode::NetcodeServerTransport,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct ServerContextBuilder {
    pub connection_config: ConnectionConfig,
    pub max_client: usize,
    pub overriding_current_time: Option<time::Duration>,
    pub overriding_protocol_id: Option<u64>,
    pub authentication: ServerAuthentication,
}

pub struct ServerContextBuilderParameters {
    pub address: net::SocketAddr,
}

pub fn get_default_protocol_id() -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    let current_version = env!("CARGO_PKG_VERSION");
    current_version.hash(&mut hasher);
    hasher.finish()
}

impl Default for ServerContextBuilder {
    fn default() -> Self {
        Self {
            connection_config: ConnectionConfig::default(),
            max_client: 64,
            overriding_current_time: None,
            overriding_protocol_id: None,
            authentication: ServerAuthentication::Unsecure,
        }
    }
}

impl ServerContextBuilder {
    pub fn build(self, p_parameters: ServerContextBuilderParameters) -> ServerContext {
        let server = renet::RenetServer::new(self.connection_config);

        // Setup transport layer using renet_netcode
        let socket: net::UdpSocket = net::UdpSocket::bind(p_parameters.address).unwrap();
        let server_config = ServerConfig {
            current_time: self.overriding_current_time.unwrap_or(
                time::SystemTime::now()
                    .duration_since(time::SystemTime::UNIX_EPOCH)
                    .unwrap(),
            ),
            max_clients: self.max_client,
            protocol_id: self
                .overriding_protocol_id
                .unwrap_or(get_default_protocol_id()),
            public_addresses: vec![p_parameters.address],
            authentication: self.authentication,
        };

        let transport = renet_netcode::NetcodeServerTransport::new(server_config, socket).unwrap();

        ServerContext { server, transport }
    }
}

impl ServerContext {
    pub fn send<C, M>(&mut self, p_client_id: ClientId, p_channel_id: C, p_message: M)
    where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.server
            .send_message(p_client_id, p_channel_id, p_message)
    }

    pub fn send_serializable<C, S>(
        &mut self,
        p_client_id: ClientId,
        p_channel_id: C,
        p_serializable: &S,
    ) where
        C: Into<u8>,
        S: serde::Serialize + ?Sized,
    {
        let message = Vec::<u8>::default();
        let message = postcard::to_extend(p_serializable, message).unwrap();
        self.send(p_client_id, p_channel_id, message);
    }

    pub fn send_all<C, M>(&mut self, p_channel_id: C, p_message: M)
    where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.server.broadcast_message(p_channel_id, p_message)
    }

    pub fn send_serializable_all<C, S>(&mut self, p_channel_id: C, p_serializable: &S)
    where
        C: Into<u8>,
        S: serde::Serialize + ?Sized,
    {
        let message = Vec::<u8>::default();
        let message = postcard::to_extend(p_serializable, message).unwrap();
        self.send_all(p_channel_id, message)
    }

    pub fn send_all_except<C, M>(
        &mut self,
        p_except_client_id: ClientId,
        p_channel_id: C,
        p_message: M,
    ) where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.server
            .broadcast_message_except(p_except_client_id, p_channel_id, p_message)
    }

    pub fn send_serializable_all_except<C, S>(
        &mut self,
        p_except_client_id: ClientId,
        p_channel_id: C,
        p_serializable: &S,
    ) where
        C: Into<u8>,
        S: serde::Serialize + ?Sized,
    {
        let message = Vec::<u8>::default();
        let message = postcard::to_extend(p_serializable, message).unwrap();
        self.server
            .broadcast_message_except(p_except_client_id, p_channel_id, message)
    }

    pub fn get_client_ids(&self) -> Vec<ClientId> {
        self.server.clients_id()
    }

    pub fn receive_from<C>(&mut self, p_client_id: ClientId, p_channel_id: C) -> Option<Bytes>
    where
        C: Into<u8>,
    {
        self.server.receive_message(p_client_id, p_channel_id)
    }

    pub fn get_event(&mut self) -> Option<ServerEvent> {
        self.server.get_event()
    }

    pub fn update(&mut self, p_delta: time::Duration) -> anyhow::Result<(), NetcodeTransportError> {
        // Receive new messages and update clients
        self.server.update(p_delta);
        self.transport.update(p_delta, &mut self.server)?;
        self.transport.send_packets(&mut self.server);
        Ok(())
    }
}
