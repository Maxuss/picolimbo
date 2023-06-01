use lobsterchat::component::Component;
use picolimbo_proto::{Decodeable, Encodeable, Protocol};
use picolimbo_proto::{Identifier, Varint};
use uuid::Uuid;

use crate::build_packets;

#[derive(Debug, Clone, PartialEq)]
pub enum Login {
    LoginStart(LoginStart),
    LoginPluginResponse(LoginPluginResponse),
    LoginPluginRequest(LoginPluginRequest),
    LoginDisconnect(LoginDisconnect),
    LoginSuccess(LoginSuccess),
}

build_packets! { Login:
    packet LoginStart(in 0x00) {
        username: String
    };

    packet LoginPluginResponse(in 0x02) {
        message_id: i32 as varint,
        successful: bool,
        data: Vec<u8> as unprefixed
    };

    packet LoginDisconnect(out 0x00) {
        reason: Component
    };

    packet LoginPluginRequest(out 0x04) {
        message_id: i32 as varint,
        channel: Identifier,
        data: Vec<u8> as unprefixed
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginSuccess {
    pub username: String,
    pub uuid: Uuid,
}

impl Encodeable for LoginSuccess {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: picolimbo_proto::Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.uuid.encode(out, ver)?;
        self.username.encode(out, ver)?;
        if ver >= Protocol::V1_19 {
            Varint(0).encode(out, ver)?; // no profile properties
        }
        Ok(())
    }

    fn predict_size(&self) -> usize {
        self.username.len() + 8
    }
}

impl Encodeable for Login {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        match self {
            Login::LoginPluginRequest(p) => {
                Varint(0x04).encode(out, ver)?;
                p.encode(out, ver)
            }
            Login::LoginDisconnect(p) => {
                Varint(0x00).encode(out, ver)?;
                p.encode(out, ver)
            }
            Login::LoginSuccess(p) => {
                Varint(0x02).encode(out, ver)?;
                p.encode(out, ver)
            }
            _ => Ok(()),
        }
    }

    fn predict_size(&self) -> usize {
        1 + match self {
            Login::LoginPluginRequest(p) => p.predict_size(),
            Login::LoginDisconnect(p) => p.predict_size(),
            Login::LoginSuccess(p) => p.predict_size(),
            _ => 0,
        }
    }
}

impl Decodeable for Login {
    fn decode(read: &mut std::io::Cursor<&[u8]>, ver: Protocol) -> picolimbo_proto::Result<Self>
    where
        Self: Sized,
    {
        let id = Varint::decode(read, ver)?;
        match id.0 {
            0x00 => LoginStart::decode(read, ver).map(Login::LoginStart),
            0x02 => LoginPluginResponse::decode(read, ver).map(Login::LoginPluginResponse),
            other => Err(picolimbo_proto::ProtoError::InvalidPacket(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encodeable)]
pub struct PlayerProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}
