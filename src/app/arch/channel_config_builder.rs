use crate::adapter::net;

pub trait Message {
    const MAX_COUNT: usize = 10;
}

pub trait ReliableUnorderedMessage: Message {
    const RESEND_TIME: std::time::Duration = std::time::Duration::from_millis(500);
}

pub trait ReliableOrderedMessage: ReliableUnorderedMessage {}

struct ChannelConfigBuilder<M>
where
    M: Message,
{
    pub channel: u8,
    message_phantom: std::marker::PhantomData<M>,
}

impl<M> ChannelConfigBuilder<M>
where
    M: Message,
{
    pub fn new(p_channel: u8) -> Self {
        Self {
            channel: p_channel,
            message_phantom: Default::default(),
        }
    }

    pub fn build_unreliable(self) -> net::server::ChannelConfig {
        net::server::ChannelConfig {
            channel_id: self.channel,
            max_memory_usage_bytes: std::mem::size_of::<M>() * M::MAX_COUNT,
            send_type: net::server::SendType::Unreliable,
        }
    }
}

impl<M> ChannelConfigBuilder<M>
where
    M: ReliableUnorderedMessage,
{
    pub fn build_reliable_unordered(self) -> net::server::ChannelConfig {
        net::server::ChannelConfig {
            channel_id: self.channel,
            max_memory_usage_bytes: std::mem::size_of::<M>() * M::MAX_COUNT,
            send_type: net::server::SendType::ReliableUnordered {
                resend_time: M::RESEND_TIME,
            },
        }
    }
}

impl<M> ChannelConfigBuilder<M>
where
    M: ReliableOrderedMessage,
{
    pub fn build_reliable_ordered(self) -> net::server::ChannelConfig {
        net::server::ChannelConfig {
            channel_id: self.channel,
            max_memory_usage_bytes: std::mem::size_of::<M>() * M::MAX_COUNT,
            send_type: net::server::SendType::ReliableOrdered {
                resend_time: M::RESEND_TIME,
            },
        }
    }

    pub fn build_large(self) -> net::server::ChannelConfig {
        net::server::ChannelConfig {
            channel_id: self.channel,
            max_memory_usage_bytes: std::mem::size_of::<net::server::LargeParcelMessage>()
                * M::MAX_COUNT,
            send_type: net::server::SendType::ReliableOrdered {
                resend_time: M::RESEND_TIME,
            },
        }
    }
}

pub fn channel_config_unreliable<M: Message>(p_channel_id: u8) -> net::server::ChannelConfig {
    ChannelConfigBuilder::<M>::new(p_channel_id).build_unreliable()
}

pub fn channel_config_reliable_unordered<M: ReliableUnorderedMessage>(
    p_channel_id: u8,
) -> net::server::ChannelConfig {
    ChannelConfigBuilder::<M>::new(p_channel_id).build_reliable_unordered()
}

pub fn channel_config_reliable_ordered<M: ReliableOrderedMessage>(
    p_channel_id: u8,
) -> net::server::ChannelConfig {
    ChannelConfigBuilder::<M>::new(p_channel_id).build_reliable_ordered()
}

pub fn channel_config_large<M: ReliableOrderedMessage>(
    p_channel_id: u8,
) -> net::server::ChannelConfig {
    ChannelConfigBuilder::<M>::new(p_channel_id).build_large()
}
