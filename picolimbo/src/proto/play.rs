use std::{borrow::Cow, mem::size_of};

use lobsterchat::component::Component;
use picolimbo_proto::{ArrayPrefix, Decodeable, Encodeable, Identifier, Protocol, Varint};
use uuid::Uuid;

use crate::{byte_enum, dim::DIMENSION_MANAGER, varint_enum};

byte_enum!(out Gamemode {
    Undefined = -0x01,
    Survival = 0x00,
    Creative = 0x01,
    Adventure = 0x02,
    Spectator = 0x03
});

varint_enum!(in ChatMode {
    Enabled = 0x00,
    CommandsOnly = 0x01,
    Hidden = 0x02
});

varint_enum!(in MainHand {
    Left = 0x00,
    Right = 0x01
});

byte_enum!(out EntityStatusPlayer {
    OpPerm0 = 24
});

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ChatMessagePosition {
    Chat = 0,
    System = 1,
    ActionBar = 2,
}

pub trait PacketMapping {
    fn id_for_proto(protocol: Protocol) -> i32;
}

macro_rules! mapped_packets {
    ($(
        $(out $out_packet_name:ident)? $(in $in_packet_name:ident)? {
            $(
                $field_name:ident: $field_type:ty
            ),* $(,)?
            ;
            mapping {
                $(
                map($pkt_id:literal, $proto_version_from:ident, $proto_version_to:ident)
                ),* $(,)?
            }
        }
    );* $(;)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Play {
            $(
                $(
                $out_packet_name($out_packet_name),
                )?
                $(
                $in_packet_name($in_packet_name),
                )?
            )*
            None
        }

        impl Encodeable for Play {
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut, ver: Protocol) -> picolimbo_proto::Result<()> {
                match self {
                    $(
                        $(
                        Self::$out_packet_name(pkt) => {
                            Varint::from(<$out_packet_name>::id_for_proto(ver)).encode(buf, ver)?;
                            pkt.encode(buf, ver)
                        }
                    )   ?
                    )*
                    _ => { Ok(()) }
                }
            }
        }

        impl Decodeable for Play {
            fn decode(read: &mut std::io::Cursor<&[u8]>, ver: Protocol) -> picolimbo_proto::Result<Self> {
                let id = Varint::decode(read, ver)?.0; // only packet we need is keepalive
                if(KeepAliveServerbound::id_for_proto(ver) == id) {
                    Ok(Self::KeepAliveServerbound(KeepAliveServerbound::decode(read, ver)?))
                } else {
                    Ok(Self::None)
                }
            }
        }

        $(
            #[derive(Debug, Clone, PartialEq)]
            pub struct $($out_packet_name)? $($in_packet_name)? {
                $(
                    pub $field_name: $field_type,
                )*
            }

            impl PacketMapping for $($out_packet_name)? $($in_packet_name)? {
                fn id_for_proto(protocol: Protocol) -> i32 {
                    match protocol {
                        $(
                            _pid if (Protocol::$proto_version_from..=Protocol::$proto_version_to).contains(&_pid) => $pkt_id,
                        )*
                        _ => -1
                    }
                }
            }

            impl crate::proto::IntoPacket for $($out_packet_name)? $($in_packet_name)? {
                fn into_packet(self) -> crate::proto::Packet {
                    crate::proto::Packet::Play(Play::$($out_packet_name)?$($in_packet_name)?(self))
                }
            }
        )*
    };
}

mapped_packets! {
    in KeepAliveServerbound {
        ka_id: i64
        ;
        mapping {
            map(0x00, V1_7_2, V1_8),
            map(0x0B, V1_9, V1_11_1),
            map(0x0C, V1_12, V1_12),
            map(0x0B, V1_12_1, V1_12_2),
            map(0x0E, V1_13, V1_13_2),
            map(0x0F, V1_14, V1_15_2),
            map(0x10, V1_16, V1_16_4),
            map(0x0F, V1_17, V1_18_2),
            map(0x11, V1_19, V1_19),
            map(0x12, V1_19_1, V1_19_1),
            map(0x11, V1_19_3, V1_19_3),
            map(0x12, V1_19_4, V1_19_4)
        }
    };

    out SendCommands {
        ;
        mapping {
            map(0x11, V1_13, V1_14_4),
            map(0x12, V1_15, V1_15_2),
            map(0x11, V1_16, V1_16_1),
            map(0x10, V1_16_2, V1_16_4),
            map(0x12, V1_17, V1_18_2),
            map(0x0F, V1_19, V1_19_1),
            map(0x0E, V1_19_3, V1_19_3),
            map(0x10, V1_19_4, V1_19_4)
        }
    };

    out PlayLogin {
        eid: i32,
        is_hardcore: bool,
        gamemode: Gamemode,
        prev_gamemode: Gamemode,
        dimensions: Vec<String>,
        spawn_dimension: Identifier,
        dimension_name: Identifier,
        hashed_seed: i64,
        max_players: i32,
        view_distance: i32,
        simulation_distance: i32,
        reduced_debug_info: bool,
        enable_respawn_screen: bool,
        is_debug: bool,
        is_flat: bool,
        has_death_pos: bool
        ;
        mapping {
            map(0x01, V1_7_2, V1_8),
            map(0x23, V1_9, V1_12_2),
            map(0x25, V1_13, V1_14_4),
            map(0x26, V1_15, V1_15_2),
            map(0x25, V1_16, V1_16_1),
            map(0x24, V1_16_2, V1_16_4),
            map(0x26, V1_17, V1_18_2),
            map(0x23, V1_19, V1_19),
            map(0x25, V1_19_1, V1_19_1),
            map(0x24, V1_19_3, V1_19_3),
            map(0x28, V1_19_4, V1_19_4)
        }
    };

    out PluginMessageOut {
        channel: String,
        data: String
        ;
        mapping {
            map(0x3F, V1_7_2, V1_8),
            map(0x18, V1_8, V1_12_2),
            map(0x19, V1_13, V1_13_2),
            map(0x18, V1_14, V1_14_4),
            map(0x19, V1_15, V1_15_2),
            map(0x18, V1_16, V1_16_1),
            map(0x17, V1_16_2, V1_16_4),
            map(0x18, V1_17, V1_18_2),
            map(0x15, V1_19, V1_19),
            map(0x16, V1_19_1, V1_19_1),
            map(0x15, V1_19_3, V1_19_3),
            map(0x17, V1_19_4, V1_19_4)
        }
    };

    out PlayerAbilities {
        flags: u8,
        flying_speed: f32,
        fov_mod: f32
        ;
        mapping {
            map(0x39, V1_7_2, V1_8),
            map(0x2B, V1_9, V1_12),
            map(0x2C, V1_12_1, V1_12_2),
            map(0x2E, V1_13, V1_13_2),
            map(0x31, V1_14, V1_14_4),
            map(0x32, V1_15, V1_15_2),
            map(0x31, V1_16, V1_16_1),
            map(0x30, V1_16_2, V1_16_4),
            map(0x32, V1_17, V1_18_2),
            map(0x2F, V1_19, V1_19),
            map(0x31, V1_19_1, V1_19_1),
            map(0x30, V1_19_3, V1_19_3),
            map(0x34, V1_19_4, V1_19_4)
        }
    };

    out PlayerPositionRotation {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool
        ;
        mapping {
            map(0x08, V1_7_2, V1_8),
            map(0x2E, V1_9, V1_12),
            map(0x2F, V1_12_1, V1_12_2),
            map(0x32, V1_13, V1_13_2),
            map(0x35, V1_14, V1_14_4),
            map(0x36, V1_15, V1_15_2),
            map(0x35, V1_16, V1_16_1),
            map(0x34, V1_16_2, V1_16_4),
            map(0x38, V1_17, V1_18_2),
            map(0x36, V1_19, V1_19),
            map(0x39, V1_19_1, V1_19_1),
            map(0x38, V1_19_3, V1_19_3),
            map(0x3C, V1_19_4, V1_19_4)
        }
    };

    out KeepAliveClientbound {
        ka_id: i64
        ;
        mapping {
            map(0x00, V1_7_2, V1_8),
            map(0x1F, V1_9, V1_12_2),
            map(0x21, V1_13, V1_13_2),
            map(0x20, V1_14, V1_14_4),
            map(0x21, V1_15, V1_15_2),
            map(0x20, V1_16, V1_16_1),
            map(0x1F, V1_16_2, V1_16_4),
            map(0x21, V1_17, V1_18_2),
            map(0x1E, V1_19, V1_19),
            map(0x20, V1_19_1, V1_19_1),
            map(0x1F, V1_19_3, V1_19_3),
            map(0x23, V1_19_4, V1_19_4)
        }
    };

    out ChatMessage {
        message: Component,
        position: ChatMessagePosition,
        sender: Uuid

        ;
        mapping {
            map(0x02, V1_7_2, V1_8),
            map(0x0F, V1_9, V1_12_2),
            map(0x0E, V1_13, V1_14_4),
            map(0x0F, V1_15, V1_15_2),
            map(0x0E, V1_16, V1_16_4),
            map(0x0F, V1_17, V1_18_2),
            map(0x5F, V1_19, V1_19),
            map(0x62, V1_19_1, V1_19_1),
            map(0x60, V1_19_3, V1_19_3),
            map(0x64, V1_19_4, V1_19_4)
        }
    };

    out PlayerInfo {
        username: String,
        gamemode: i32,
        uuid: Uuid
        ;
        mapping {
            map(0x38, V1_7_2, V1_8),
            map(0x2D, V1_9, V1_12),
            map(0x2E, V1_12_1, V1_12_2),
            map(0x30, V1_13, V1_13_2),
            map(0x33, V1_14, V1_14_4),
            map(0x34, V1_15, V1_15_2),
            map(0x33, V1_16, V1_16_1),
            map(0x32, V1_16_2, V1_16_4),
            map(0x36, V1_17, V1_18_2),
            map(0x34, V1_19, V1_19),
            map(0x37, V1_19_1, V1_19_1),
            map(0x36, V1_19_3, V1_19_3),
            map(0x3A, V1_19_4, V1_19_4)
        }
    };

    out SpawnPosition {
        x: i32,
        y: i32,
        z: i32,
        rotation: f32
        ;
        mapping {
            map(0x4C, V1_19_3, V1_19_3),
            map(0x50, V1_19_4, V1_19_4)
        }
    };

    out DisconnectPlay {
        reason: Component
        ;
        mapping {
            map(0x40, V1_7_2, V1_8),
            map(0x1A, V1_8, V1_12_2),
            map(0x1B, V1_12_2, V1_14),
            map(0x1A, V1_14, V1_14_4),
            map(0x1B, V1_14_4, V1_19_4)
        }
    }
}

impl Decodeable for KeepAliveServerbound {
    fn decode(read: &mut std::io::Cursor<&[u8]>, ver: Protocol) -> picolimbo_proto::Result<Self>
    where
        Self: Sized,
    {
        let ka_id = if ver >= Protocol::V1_12_2 {
            i64::decode(read, ver)?
        } else if ver >= Protocol::V1_8 {
            Varint::decode(read, ver)?.0 as i64
        } else {
            i32::decode(read, ver)? as i64
        };
        Ok(Self { ka_id })
    }
}

impl Encodeable for KeepAliveClientbound {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        if ver >= Protocol::V1_12_2 {
            self.ka_id.encode(out, ver)
        } else if ver >= Protocol::V1_8 {
            Varint::from(self.ka_id).encode(out, ver)
        } else {
            (self.ka_id as i32).encode(out, ver)
        }
    }

    fn predict_size(&self) -> usize {
        size_of::<i64>()
    }
}

impl Encodeable for SendCommands {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        Varint(1).encode(out, ver)?;
        0u8.encode(out, ver)?;
        Varint(0).encode(out, ver)?;
        Varint(1).encode(out, ver)
    }

    fn predict_size(&self) -> usize {
        3
    }
}

impl Encodeable for PlayLogin {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.eid.encode(out, ver)?;

        match ver {
            v if (Protocol::V1_7_2..=Protocol::V1_7_6).contains(&v) => {
                self.gamemode.encode(out, ver)?; // gamemode

                DIMENSION_MANAGER
                    .default_dim_1_16()
                    .unwrap()
                    .id
                    .encode(out, ver)?;

                0u8.encode(out, ver)?; // difficulty
                (self.max_players as u8).encode(out, ver)?; // max players
                "flat".encode(out, ver) // world type
            }
            v if (Protocol::V1_8..=Protocol::V1_9).contains(&v) => {
                self.gamemode.encode(out, ver)?; // gamemode

                DIMENSION_MANAGER
                    .default_dim_1_16()
                    .unwrap()
                    .id
                    .encode(out, ver)?;

                0u8.encode(out, ver)?; // difficulty
                (self.max_players as u8).encode(out, ver)?; // max players
                "flat".encode(out, ver)?; // world type
                self.reduced_debug_info.encode(out, ver) // reduced debug info
            }
            v if (Protocol::V1_9_1..=Protocol::V1_13_2).contains(&v) => {
                self.gamemode.encode(out, ver)?; // gamemode

                (DIMENSION_MANAGER.default_dim_1_16().unwrap().id as i32).encode(out, ver)?;

                0u8.encode(out, ver)?; // difficulty
                (self.max_players as u8).encode(out, ver)?; // max players
                "flat".encode(out, ver)?;
                // Varint(2).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)
            }
            v if (Protocol::V1_14..=Protocol::V1_14_4).contains(&v) => {
                self.gamemode.encode(out, ver)?;
                (DIMENSION_MANAGER.default_dim_1_16().unwrap().id as i32).encode(out, ver)?;
                (self.max_players as u8).encode(out, ver)?;
                "flat".encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)
            }
            v if (Protocol::V1_15..=Protocol::V1_15_2).contains(&v) => {
                self.gamemode.encode(out, ver)?;

                (DIMENSION_MANAGER.default_dim_1_16().unwrap().id as i32).encode(out, ver)?;

                self.hashed_seed.encode(out, ver)?;
                (self.max_players as u8).encode(out, ver)?;
                "flat".encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)?;
                self.enable_respawn_screen.encode(out, ver)
            }
            v if (Protocol::V1_16..=Protocol::V1_16_1).contains(&v) => {
                self.gamemode.encode(out, ver)?;
                self.prev_gamemode.encode(out, ver)?;
                Varint(1).encode(out, ver)?; // dimensions
                self.spawn_dimension.encode(out, ver)?;

                DIMENSION_MANAGER.codec_legacy.encode(out, ver)?;
                DIMENSION_MANAGER
                    .default_dim_1_16()
                    .unwrap()
                    .name
                    .encode(out, ver)?;

                self.spawn_dimension.encode(out, ver)?;
                self.dimension_name.encode(out, ver)?;
                self.hashed_seed.encode(out, ver)?;
                Varint(self.max_players).encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)?;
                self.enable_respawn_screen.encode(out, ver)?;
                self.is_debug.encode(out, ver)?;
                self.is_flat.encode(out, ver)
            }
            v if (Protocol::V1_16_2..=Protocol::V1_17_1).contains(&v) => {
                self.is_hardcore.encode(out, ver)?;
                self.gamemode.encode(out, ver)?;
                self.prev_gamemode.encode(out, ver)?;
                Varint(1).encode(out, ver)?; // dimensions
                self.spawn_dimension.encode(out, ver)?;

                DIMENSION_MANAGER.codec_1_16.encode(out, ver)?;
                DIMENSION_MANAGER
                    .default_dim_1_16()
                    .unwrap()
                    .data
                    .encode(out, ver)?;

                self.dimension_name.encode(out, ver)?;
                self.hashed_seed.encode(out, ver)?;
                Varint(self.max_players).encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)?;
                self.enable_respawn_screen.encode(out, ver)?;
                self.is_debug.encode(out, ver)?;
                self.is_flat.encode(out, ver)
            }
            v if (Protocol::V1_18..=Protocol::V1_18_2).contains(&v) => {
                self.is_hardcore.encode(out, ver)?;
                self.gamemode.encode(out, ver)?;
                Varint(1).encode(out, ver)?; // dimensions
                self.spawn_dimension.encode(out, ver)?;

                if ver == Protocol::V1_18_2 {
                    DIMENSION_MANAGER.codec_1_18_2.encode(out, ver)?;
                    DIMENSION_MANAGER
                        .default_dim_1_18_2()
                        .unwrap()
                        .data
                        .encode(out, ver)?;
                } else {
                    DIMENSION_MANAGER.codec_1_16.encode(out, ver)?;
                    DIMENSION_MANAGER
                        .default_dim_1_16()
                        .unwrap()
                        .data
                        .encode(out, ver)?;
                }

                self.dimension_name.encode(out, ver)?;
                self.hashed_seed.encode(out, ver)?;
                Varint(self.max_players).encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                Varint(self.simulation_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)?;
                self.enable_respawn_screen.encode(out, ver)?;
                self.is_debug.encode(out, ver)?;
                self.is_flat.encode(out, ver)
            }
            v if v >= Protocol::V1_19 => {
                self.is_hardcore.encode(out, ver)?;
                (self.gamemode as u8).encode(out, ver)?;
                (self.prev_gamemode as u8).encode(out, ver)?;
                Varint(1).encode(out, ver)?;
                self.dimension_name.encode(out, ver)?;
                if v >= Protocol::V1_19_1 {
                    if v >= Protocol::V1_19_4 {
                        DIMENSION_MANAGER.codec_1_19_4.encode(out, ver)?;
                    } else {
                        DIMENSION_MANAGER.codec_1_19_1.encode(out, ver)?;
                    }
                } else {
                    DIMENSION_MANAGER.codec_1_19.encode(out, ver)?;
                }
                self.spawn_dimension.encode(out, ver)?;
                self.dimension_name.encode(out, ver)?;
                self.hashed_seed.encode(out, ver)?;
                Varint(self.max_players).encode(out, ver)?;
                Varint(self.view_distance).encode(out, ver)?;
                Varint(self.simulation_distance).encode(out, ver)?;
                self.reduced_debug_info.encode(out, ver)?;
                self.enable_respawn_screen.encode(out, ver)?;
                self.is_debug.encode(out, ver)?;
                self.is_flat.encode(out, ver)?;
                self.has_death_pos.encode(out, ver)
            }
            _ => Ok(()),
        }
    }
}

impl Encodeable for PluginMessageOut {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.channel.encode(out, ver)?;
        if ver < Protocol::V1_8 {
            u16::array(Cow::Borrowed(self.data.as_bytes())).encode(out, ver)
        } else {
            self.data.encode(out, ver)
        }
    }
}

impl Encodeable for PlayerAbilities {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.flags.encode(out, ver)?;
        self.flying_speed.encode(out, ver)?;
        self.fov_mod.encode(out, ver)
    }
}

impl Encodeable for PlayerPositionRotation {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.x.encode(out, ver)?;
        (self.y + if ver < Protocol::V1_8 { 1.62 } else { 0. }).encode(out, ver)?;
        self.z.encode(out, ver)?;
        self.yaw.encode(out, ver)?;
        self.pitch.encode(out, ver)?;

        if ver >= Protocol::V1_8 {
            0x08u8.encode(out, ver)?;
        } else {
            true.encode(out, ver)?;
        }

        if ver >= Protocol::V1_9 {
            Varint(1).encode(out, ver)?;
        }

        if ver >= Protocol::V1_17 && ver <= Protocol::V1_19_3 {
            false.encode(out, ver)?;
        };

        Ok(())
    }
}

impl Encodeable for ChatMessage {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.message.encode(out, ver)?;
        if ver >= Protocol::V1_19_1 {
            (self.position == ChatMessagePosition::ActionBar).encode(out, ver)?;
        } else if ver >= Protocol::V1_19 {
            Varint(self.position as i32).encode(out, ver)?;
        } else if ver >= Protocol::V1_8 {
            (self.position as u8).encode(out, ver)?;
        }

        if ver >= Protocol::V1_16 && ver < Protocol::V1_19 {
            self.sender.encode(out, ver)?;
        }
        Ok(())
    }
}

impl Encodeable for PlayerInfo {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        if ver < Protocol::V1_8 {
            self.username.encode(out, ver)?;
            true.encode(out, ver)?; // is online
            0i16.encode(out, ver)
        } else if ver >= Protocol::V1_19_3 {
            0b101100u8.encode(out, ver)?; // actions bitmask
            Varint(1).encode(out, ver)?; // actions count

            self.uuid.encode(out, ver)?;
            self.username.encode(out, ver)?;
            0.encode(out, ver)?;

            true.encode(out, ver)?; // update listend
            Varint(self.gamemode).encode(out, ver) // gamemode
        } else {
            Varint(0).encode(out, ver)?;
            Varint(1).encode(out, ver)?;
            self.uuid.encode(out, ver)?;
            self.username.encode(out, ver)?;
            Varint(0).encode(out, ver)?;
            self.gamemode.encode(out, ver)?;
            Varint(60).encode(out, ver)?;
            false.encode(out, ver)?;

            if ver >= Protocol::V1_19 {
                false.encode(out, ver)?;
            }

            Ok(())
        }
    }
}

impl Encodeable for SpawnPosition {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        (((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF))
            .encode(out, ver)?;
        self.rotation.encode(out, ver)
    }
}

impl Encodeable for DisconnectPlay {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        self.reason.encode(out, ver)
    }
}
