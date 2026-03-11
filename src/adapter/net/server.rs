use std::hash::Hash;
use std::hash::Hasher;
use std::net;
use std::time;

pub use renet::*;
pub use renet_netcode::*;

use crate::adapter::shared;

pub const LARGE_PARCEL_DATA_MAX_BYTE_COUNT: u64 = 32;
pub type LargeParcelData = tinyvec::ArrayVec<[u8; LARGE_PARCEL_DATA_MAX_BYTE_COUNT as usize]>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LargeParcelMetadataMessage {
    pub key: u64,
    pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LargeParcelBodyMessage {
    pub key: u64,
    pub index: u64,
    pub data: LargeParcelData,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum LargeParcelMessage {
    Metadata(LargeParcelMetadataMessage),
    Body(LargeParcelBodyMessage),
}

#[derive(Debug)]
struct LargeParcelSendInstruction {
    pub key: u64,
    pub index: u64,
    pub target_client_id: Option<ClientId>,
    pub exclude_client: bool,
    pub channel_id: u8,
    pub data: LargeParcelData,
}

pub type SharedServerContext = shared::Shared<ServerContext>;

pub struct ServerContext {
    server: renet::RenetServer,
    transport: renet_netcode::NetcodeServerTransport,
    large_parcel_send_instruction_queue: std::collections::VecDeque<LargeParcelSendInstruction>,
    pub parcel_per_update: u8,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct ServerContextBuilder {
    pub connection_config: ConnectionConfig,
    pub max_client: usize,
    pub overriding_current_time: Option<time::Duration>,
    pub overriding_protocol_id: Option<u64>,
    pub authentication: ServerAuthentication,
    pub parcel_per_update: u8,
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
            parcel_per_update: 16,
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

        ServerContext {
            server,
            transport,
            large_parcel_send_instruction_queue: Default::default(),
            parcel_per_update: self.parcel_per_update,
        }
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

    pub fn receive_deserializable_from<D, C>(
        &mut self,
        p_client_id: ClientId,
        p_channel_id: C,
    ) -> Option<D>
    where
        D: serde::de::DeserializeOwned,
        C: Into<u8>,
    {
        self.receive_from(p_client_id, p_channel_id)
            .map(|p_bytes| postcard::from_bytes::<D>(&p_bytes).unwrap())
    }

    pub fn get_event(&mut self) -> Option<ServerEvent> {
        self.server.get_event()
    }

    pub fn update(&mut self, p_delta: time::Duration) -> anyhow::Result<(), NetcodeTransportError> {
        // Receive new messages and update clients
        self.server.update(p_delta);
        self.transport.update(p_delta, &mut self.server)?;
        self.transport.send_packets(&mut self.server);

        for _ in 0..self.parcel_per_update {
            let parcel = self.large_parcel_send_instruction_queue.pop_back();
            if parcel.is_none() {
                break;
            }
            let parcel = parcel.unwrap();

            let message = LargeParcelMessage::Body(LargeParcelBodyMessage {
                key: parcel.key,
                index: parcel.index,
                data: parcel.data,
            });
            if let Some(client_id) = parcel.target_client_id {
                if parcel.exclude_client {
                    self.send_serializable_all_except(client_id, parcel.channel_id, &message);
                } else {
                    self.send_serializable(client_id, parcel.channel_id, &message);
                }
            } else {
                self.send_serializable_all(parcel.channel_id, &message);
            }
        }

        Ok(())
    }

    fn setup_large_parcel<C, M>(
        &mut self,
        p_target_client_id: Option<ClientId>,
        p_channel_id: C,
        p_message: M,
        p_exclude_client: bool,
    ) where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        let p_message = {
            let uncompressed = p_message.into();
            let mut encoder = snap::raw::Encoder::new();
            encoder
                .compress_vec(uncompressed.iter().as_slice())
                .unwrap()
        };

        let p_channel_id = p_channel_id.into();
        let key = {
            let mut hasher = rapidhash::quality::RapidHasher::default();
            p_message.hash(&mut hasher);
            p_channel_id.hash(&mut hasher);
            p_target_client_id.hash(&mut hasher);
            hasher.finish()
        };

        let full_parcel_count = p_message.len() as u64 / LARGE_PARCEL_DATA_MAX_BYTE_COUNT;
        let remaining_parcel_byte_count = p_message.len() as u64 % LARGE_PARCEL_DATA_MAX_BYTE_COUNT;
        let total_parcel_count = if remaining_parcel_byte_count == 0 {
            full_parcel_count
        } else {
            full_parcel_count + 1
        };

        for i in 0..full_parcel_count {
            let mut data = LargeParcelData::default();
            data.extend_from_slice(
                &p_message[(i * LARGE_PARCEL_DATA_MAX_BYTE_COUNT) as usize
                    ..((i + 1) * LARGE_PARCEL_DATA_MAX_BYTE_COUNT) as usize],
            );
            self.large_parcel_send_instruction_queue
                .push_front(LargeParcelSendInstruction {
                    key,
                    index: i,
                    target_client_id: p_target_client_id,
                    exclude_client: p_exclude_client,
                    channel_id: p_channel_id,
                    data,
                });
        }

        if remaining_parcel_byte_count != 0 {
            let mut data = LargeParcelData::default();
            data.extend_from_slice(
                &p_message[(full_parcel_count * LARGE_PARCEL_DATA_MAX_BYTE_COUNT) as usize
                    ..((full_parcel_count * LARGE_PARCEL_DATA_MAX_BYTE_COUNT)
                        + remaining_parcel_byte_count) as usize],
            );
            self.large_parcel_send_instruction_queue
                .push_front(LargeParcelSendInstruction {
                    key,
                    index: full_parcel_count,
                    target_client_id: p_target_client_id,
                    exclude_client: p_exclude_client,
                    channel_id: p_channel_id,
                    data,
                });
        }

        let metadata = LargeParcelMessage::Metadata(LargeParcelMetadataMessage {
            key,
            count: total_parcel_count,
        });
        if let Some(client_id) = p_target_client_id {
            if p_exclude_client {
                self.send_serializable_all_except(client_id, p_channel_id, &metadata);
            } else {
                self.send_serializable(client_id, p_channel_id, &metadata);
            }
        } else {
            self.send_serializable_all(p_channel_id, &metadata);
        }
    }

    pub fn send_large<C, M>(&mut self, p_client_id: ClientId, p_channel_id: C, p_message: M)
    where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.setup_large_parcel(Some(p_client_id), p_channel_id, p_message, false);
    }

    pub fn send_large_all<C, M>(&mut self, p_channel_id: C, p_message: M)
    where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.setup_large_parcel(None, p_channel_id, p_message, false);
    }

    pub fn send_large_all_except<C, M>(
        &mut self,
        p_client_id: ClientId,
        p_channel_id: C,
        p_message: M,
    ) where
        C: Into<u8>,
        M: Into<Bytes>,
    {
        self.setup_large_parcel(Some(p_client_id), p_channel_id, p_message, true);
    }

    pub fn send_large_serializable<C, S>(
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
        self.send_large(p_client_id, p_channel_id, message);
    }

    pub fn send_large_serializable_all<C, S>(&mut self, p_channel_id: C, p_serializable: &S)
    where
        C: Into<u8>,
        S: serde::Serialize + ?Sized,
    {
        let message = Vec::<u8>::default();
        let message = postcard::to_extend(p_serializable, message).unwrap();
        self.send_large_all(p_channel_id, message);
    }

    pub fn send_large_serializable_all_except<C, S>(
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
        self.send_large_all_except(p_client_id, p_channel_id, message);
    }
}
