use lobsterchat::component::Component;
use picolimbo_proto::Encodeable;
use picolimbo_proto::{Identifier, Varint};
use uuid::Uuid;

use crate::build_packets;

build_packets! { Login:
    packet LoginStart(in 0x00) {
        username: String,
        uuid: Option<Uuid>
    };

    packet LoginPluginResponse(in 0x02) {
        message_id: i32 as varint,
        successful: bool,
        data: Vec<u8> as unprefixed
    };

    packet LoginDisconnect(out 0x00) {
        reason: Component
    };

    packet LoginSuccess(out 0x02) {
        player_uuid: Uuid,
        player_username: String,
        profile_properties: Vec<PlayerProperty> as prefixed(Varint)
    };

    packet LoginPluginRequest(out 0x04) {
        message_id: i32 as varint,
        channel: Identifier,
        data: Vec<u8> as unprefixed
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encodeable)]
pub struct PlayerProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}
