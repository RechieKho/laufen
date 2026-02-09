use super::server;
use std::net;
use std::time;

pub struct ClientContext {
    client: renet::RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
}

impl ClientContext {
    pub fn new(
        p_socket_address: &net::SocketAddr,
        p_server_address: &net::SocketAddr,
        p_config: server::ConnectionConfig,
    ) -> Self {
        let client = renet::RenetClient::new(p_config);

        // Setup transport layer using renet_netcode
        let socket = net::UdpSocket::bind(p_socket_address).unwrap();
        let current_time = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap();
        let authentication = renet_netcode::ClientAuthentication::Unsecure {
            server_addr: *p_server_address,
            client_id: 0,
            user_data: None,
            protocol_id: server::get_default_protocol_id(),
        };

        let transport =
            renet_netcode::NetcodeClientTransport::new(current_time, authentication, socket)
                .unwrap();

        Self { client, transport }
    }

    pub fn update(
        &mut self,
        p_delta: time::Duration,
    ) -> anyhow::Result<(), server::NetcodeTransportError> {
        // Receive new messages and update client
        self.client.update(p_delta);
        self.transport.update(p_delta, &mut self.client).unwrap();

        if self.client.is_connected() {
            // Receive message from server
            while let Some(message) = self
                .client
                .receive_message(server::DefaultChannel::ReliableOrdered)
            {
                println!("Client Received: {:?}", message);
                // Handle received message
            }

            // Send message
            self.client
                .send_message(server::DefaultChannel::ReliableOrdered, "client text");
        }

        // Send packets to server using the transport layer
        self.transport.send_packets(&mut self.client)?;

        Ok(())
    }
}
