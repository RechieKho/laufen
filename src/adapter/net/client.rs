use super::server;
use std::net;
use std::time;

pub struct ClientContext {
    client: renet::RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
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

        ClientContext { client, transport }
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
}
