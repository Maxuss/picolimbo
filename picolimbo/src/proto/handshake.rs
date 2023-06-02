use lobsterchat::component::Component;
use picolimbo_proto::{Decodeable, Encodeable, Protocol, Varint};
use serde::Serialize;
use uuid::Uuid;

use crate::{build_packets, varint_enum};

varint_enum!(in HsNextState {
    Status = 0x01,
    Login = 0x02
});

#[derive(Debug, Clone, PartialEq)]
pub enum Handshake {
    HandshakeInitial(HandshakeInitial),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),
    PingRequest(PingRequest),
    PingResponse(PingResponse),
}

build_packets! { Handshake:
    packet HandshakeInitial(in 0x00) {
        protocol_version: i32 as varint,
        server_address: String,
        server_port: u16,
        next_state: HsNextState
    }
}

build_packets! { Status:
    packet StatusRequest(in 0x00) {
        // Empty packet
    };

    packet StatusResponse(out 0x00) {
        status: ServerStatus as json
    };

    packet PingRequest(in 0x01) {
        payload: i64
    };

    packet PingResponse(out 0x01) {
        payload: i64
    };
}

impl Encodeable for Status {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: picolimbo_proto::Protocol,
    ) -> picolimbo_proto::Result<()> {
        match self {
            Status::StatusResponse(resp) => {
                Varint(0x00).encode(out, ver)?;
                resp.encode(out, ver)
            }
            Status::PingResponse(resp) => {
                Varint(0x01).encode(out, ver)?;
                resp.encode(out, ver)
            }
            _ => Ok(()),
        }
    }

    fn predict_size(&self) -> usize {
        1 + match self {
            Status::StatusResponse(res) => res.predict_size(),
            Status::PingResponse(res) => res.predict_size(),
            _ => 0,
        }
    }
}

impl Decodeable for Status {
    fn decode(
        read: &mut std::io::Cursor<&[u8]>,
        ver: picolimbo_proto::Protocol,
    ) -> picolimbo_proto::Result<Self>
    where
        Self: Sized,
    {
        let id = Varint::decode(read, ver)?;
        match id.0 {
            0x00 => StatusRequest::decode(read, ver).map(Self::StatusRequest),
            0x01 => PingRequest::decode(read, ver).map(Self::PingRequest),
            other => Err(picolimbo_proto::ProtoError::InvalidPacket(other)),
        }
    }
}

impl Encodeable for Handshake {
    fn encode(
        &self,
        _out: &mut picolimbo_proto::BytesMut,
        _ver: picolimbo_proto::Protocol,
    ) -> picolimbo_proto::Result<()> {
        // Only inbound packets here
        Ok(())
    }
}

impl Decodeable for Handshake {
    fn decode(
        read: &mut std::io::Cursor<&[u8]>,
        ver: picolimbo_proto::Protocol,
    ) -> picolimbo_proto::Result<Self>
    where
        Self: Sized,
    {
        let id = Varint::decode(read, ver)?;
        match id.0 {
            0x00 => HandshakeInitial::decode(read, ver).map(Self::HandshakeInitial),
            other => Err(picolimbo_proto::ProtoError::InvalidPacket(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerStatus {
    pub version: ServerVersion,
    pub players: ServerPlayers,
    pub description: Component,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            version: ServerVersion {
                name: "latest".to_owned(),
                protocol: Protocol::latest() as i32,
            },
            players: ServerPlayers {
                max: 1,
                online: 1,
                sample: Default::default(),
            },
            description: Default::default(),
            favicon: Default::default(),
            enforces_secure_chat: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<ServerPlayerSingle>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerPlayerSingle {
    pub name: &'static str,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: i32,
}
