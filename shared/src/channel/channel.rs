use crate::channel::receivers::ordered_reliable::OrderedReliableReceiver;
use crate::channel::receivers::sequenced_reliable::SequencedReliableReceiver;
use crate::channel::receivers::sequenced_unreliable::SequencedUnreliableReceiver;
use crate::channel::receivers::unordered_reliable::UnorderedReliableReceiver;
use crate::channel::receivers::unordered_unreliable::UnorderedUnreliableReceiver;
use crate::channel::receivers::ChannelReceiver;
use crate::channel::senders::reliable::ReliableSender;
use crate::channel::senders::unreliable::{SequencedUnreliableSender, UnorderedUnreliableSender};
use crate::channel::senders::ChannelSender;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// A Channel is an abstraction for a way to send messages over the network
/// You can define the direction, ordering, reliability of the channel
pub struct ChannelContainer {
    pub setting: ChannelSettings,
    pub(crate) receiver: ChannelReceiver,
    pub(crate) sender: ChannelSender,
}

pub trait Channel: 'static {
    fn get_builder(settings: ChannelSettings) -> Box<dyn ChannelBuilder>;
}

pub trait ChannelBuilder {
    fn build(&self) -> ChannelContainer;
}

/// Data from the channel that will be serialized in the header of the packet
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct ChannelHeader {
    pub(crate) kind: ChannelKind,
    // TODO: add fragmentation data
}

impl ChannelContainer {
    pub fn new(settings: ChannelSettings) -> Self {
        let receiver: ChannelReceiver;
        let sender: ChannelSender;
        let settings_clone = settings.clone();
        match settings.mode {
            ChannelMode::UnorderedUnreliable => {
                receiver = UnorderedUnreliableReceiver::new().into();
                sender = UnorderedUnreliableSender::new().into();
            }
            ChannelMode::SequencedUnreliable => {
                receiver = SequencedUnreliableReceiver::new().into();
                sender = SequencedUnreliableSender::new().into();
            }
            ChannelMode::UnorderedReliable(reliable_settings) => {
                receiver = UnorderedReliableReceiver::new().into();
                sender = ReliableSender::new(reliable_settings).into();
            }
            ChannelMode::SequencedReliable(reliable_settings) => {
                receiver = SequencedReliableReceiver::new().into();
                sender = ReliableSender::new(reliable_settings).into();
            }
            ChannelMode::OrderedReliable(reliable_settings) => {
                receiver = OrderedReliableReceiver::new().into();
                sender = ReliableSender::new(reliable_settings).into();
            }
        }
        Self {
            setting: settings_clone,
            receiver,
            sender,
        }
    }
}

/// Type of the channel
// TODO: update the serialization
// #[derive(Serialize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct ChannelKind(u16);

impl ChannelKind {
    pub const fn new(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Clone, Default)]
pub struct ChannelSettings {
    // TODO: split into Ordering and Reliability? Or not because we might to add new modes like TickBuffered
    pub mode: ChannelMode,
    pub direction: ChannelDirection,
}

pub enum ChannelOrdering {
    /// Messages will arrive in the order that they were sent
    Ordered,
    /// Messages will arrive in any order
    Unordered,
    /// Only the newest messages are accepted; older messages are discarded
    Sequenced,
}

#[derive(Clone, Debug, PartialEq, Default)]
/// ChannelMode specifies how packets are sent and received
/// See more information: http://www.jenkinssoftware.com/raknet/manual/reliabilitytypes.html
pub enum ChannelMode {
    #[default]
    /// Packets may arrive out-of-order, or not at all
    UnorderedUnreliable,
    /// Same as unordered unreliable, but only the newest packet is ever accepted, older packets
    /// are ignored
    SequencedUnreliable,
    /// Packets may arrive out-of-order, but we make sure (with retries, acks) that the packet
    /// will arrive
    UnorderedReliable(ReliableSettings),
    /// Same as unordered reliable, but the packets are sequenced (only the newest packet is accepted)
    SequencedReliable(ReliableSettings),
    /// Packets will arrive in the correct order at the destination
    OrderedReliable(ReliableSettings),
}

#[derive(Clone, PartialEq, Default)]
pub enum ChannelDirection {
    ClientToServer,
    ServerToClient,
    #[default]
    Bidirectional,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ReliableSettings {
    /// Duration to wait before resending a packet if it has not been acked
    pub rtt_resend_factor: f32,
}

impl ReliableSettings {
    pub const fn default() -> Self {
        Self {
            rtt_resend_factor: 1.5,
        }
    }
}