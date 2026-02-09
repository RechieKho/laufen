use std::net;
use std::time;

pub use renet::ClientId;
pub use renet::ConnectionConfig;
pub use renet::DefaultChannel;
pub use renet::ServerEvent;
pub use renet_netcode::NetcodeTransportError;
pub use renet_netcode::ServerAuthentication;
pub use renet_netcode::ServerConfig;

pub struct ServerContext {
    server: renet::RenetServer,
    transport: renet_netcode::NetcodeServerTransport,
}

impl ServerContext {
    pub fn new(p_address: &net::SocketAddr, p_config: ConnectionConfig) -> Self {
        let server = renet::RenetServer::new(p_config);

        // Setup transport layer using renet_netcode
        let socket: net::UdpSocket = net::UdpSocket::bind(p_address).unwrap();
        let server_config = ServerConfig {
            current_time: time::SystemTime::now()
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap(),
            max_clients: 64,
            protocol_id: 0,
            public_addresses: vec![*p_address],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = renet_netcode::NetcodeServerTransport::new(server_config, socket).unwrap();

        Self { server, transport }
    }

    pub fn update(&mut self, p_delta: time::Duration) -> anyhow::Result<(), NetcodeTransportError> {
        // Receive new messages and update clients
        self.server.update(p_delta);
        self.transport.update(p_delta, &mut self.server)?;

        // Check for client connections/disconnections
        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("Client {client_id} connected");
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("Client {client_id} disconnected: {reason}");
                }
            }
        }

        // Receive message from channel
        for client_id in self.server.clients_id() {
            // The enum DefaultChannel describe the channels used by the default configuration
            while let Some(message) = self
                .server
                .receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                // Handle received message
                println!("Server received: {:?}", message);
            }
        }

        // Send a text message for all clients
        self.server
            .broadcast_message(DefaultChannel::ReliableOrdered, "server message");

        let client_id: ClientId = 0;
        // Send a text message for all clients except for Client 0
        self.server.broadcast_message_except(
            client_id,
            DefaultChannel::ReliableOrdered,
            "server message",
        );

        // Send message to only one client
        self.server
            .send_message(client_id, DefaultChannel::ReliableOrdered, "server message");

        // Send packets to clients using the transport layer
        self.transport.send_packets(&mut self.server);

        Ok(())
    }
}
