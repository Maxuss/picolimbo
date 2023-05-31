pub mod handshake;
pub mod login;
pub mod play;

use picolimbo_proto::{Encodeable, Protocol, Varint};

use self::{
    handshake::{Handshake, Status},
    login::Login,
    play::Play,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Handshake(Handshake),
    Status(Status),
    Login(Login),
    Play(Play),
}

impl Encodeable for Packet {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        match self {
            Packet::Handshake(hs) => {
                let mut hs_buf = picolimbo_proto::BytesMut::with_capacity(hs.predict_size());
                hs.encode(&mut hs_buf, ver)?;
                Varint(hs_buf.len() as i32).encode(out, ver)?;
                out.extend_from_slice(&hs_buf);
            }
            Packet::Status(st) => {
                let mut hs_buf = picolimbo_proto::BytesMut::with_capacity(st.predict_size());
                st.encode(&mut hs_buf, ver)?;
                Varint(hs_buf.len() as i32).encode(out, ver)?;
                out.extend_from_slice(&hs_buf);
            }
            Packet::Login(lg) => {
                let mut hs_buf = picolimbo_proto::BytesMut::with_capacity(lg.predict_size());
                lg.encode(&mut hs_buf, ver)?;
                Varint(hs_buf.len() as i32).encode(out, ver)?;
                out.extend_from_slice(&hs_buf);
            }
            Packet::Play(lg) => {
                let mut hs_buf = picolimbo_proto::BytesMut::with_capacity(lg.predict_size());
                lg.encode(&mut hs_buf, ver)?;
                Varint(hs_buf.len() as i32).encode(out, ver)?;
                out.extend_from_slice(&hs_buf);
            }
        }
        Ok(())
    }

    fn predict_size(&self) -> usize {
        let child_size = match self {
            Packet::Handshake(hs) => hs.predict_size(),
            Packet::Status(st) => st.predict_size(),
            Packet::Login(lg) => lg.predict_size(),
            Packet::Play(pl) => pl.predict_size(),
        };
        Varint::size_of(child_size as i32) + child_size
    }
}

impl IntoPacket for Packet {
    fn into_packet(self) -> Packet {
        self
    }
}

pub trait IntoPacket {
    fn into_packet(self) -> Packet;
}

#[macro_export]
macro_rules! build_packets {
    ($parent:ident: $(
        packet $name:ident ($(out $enc_id:literal)? $(in $dec_id:literal)?) {
            $(
                $field_name:ident: $field_type:ty $(as $attr:meta)?
            ),* $(,)?
        }
    );* $(;)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum $parent {
            $(
                $name($name),
            )*
            None
        }

        impl $crate::proto::IntoPacket for $parent {
            fn into_packet(self) -> $crate::proto::Packet {
                $crate::proto::Packet::$parent(self)
            }
        }

        impl picolimbo_proto::Encodeable for $parent {
            #[allow(unused)]
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<()> {
                match self {
                    $(
                        Self::$name(pkt) => {
                            $(
                                picolimbo_proto::Varint($enc_id).encode(buf, ver)?;
                                pkt.encode(buf, ver)?;
                            )?
                        }
                    )*
                    Self::None => { /* noop */ }
                };
                Ok(())
            }

            #[allow(unused)]
            fn predict_size(&self) -> usize {
                1 + match self {
                    $(
                        Self::$name(pkt) => $crate::__pkt_struct_predict_size!($($enc_id)? pkt),
                    )*
                    Self::None => 0
                }
            }
        }

        impl picolimbo_proto::Decodeable for $parent {
            fn decode(read: &mut std::io::Cursor<&[u8]>, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<Self> {
                let packet_id = picolimbo_proto::Varint::decode(read, ver)?.0;
                match packet_id {
                    $(
                        $(
                            $dec_id => <$name>::decode(read, ver).map(Self::$name),
                        )?
                    )*
                    _ => Ok(Self::None)
                }
            }
        }

        $(
            $crate::__pkt_struct_build_derive!(
                $(__enc $enc_id)? $(__dec $dec_id)? then $name:
                    $(
                        $field_name: $field_type $(as $attr)?
                    ),*
            );

            impl $crate::proto::IntoPacket for $name {
                fn into_packet(self) -> $crate::proto::Packet {
                    $crate::proto::Packet::$parent($parent::$name(self))
                }
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pkt_struct_predict_size {
    ($pkt:ident) => {
        0
    };
    ($enc_id_marker:literal $pkt:ident) => {
        $pkt.predict_size()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pkt_struct_build_derive {
    (then $name:ident: $($field_name:ident: $field_type:ty $(as $attr:meta)?),*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(
                $(#[$attr])?
                $field_name: $field_type,
            )*
        }
    };
    (__enc $enc_id:literal __dec $dec_id:literal then $name:ident: $($field_name:ident: $field_type:ty $(as $attr:meta)?),*) => {
        #[derive(Debug, Clone, PartialEq, picolimbo_proto::Encodeable, picolimbo_proto::Decodeable)]
        pub struct $name {
            $(
                $(#[$attr])?
                pub $field_name: $field_type,
            )*
        }
    };
    (__enc $enc_id:literal then $name:ident: $($field_name:ident: $field_type:ty $(as $attr:meta)?),*) => {
        #[derive(Debug, Clone, PartialEq, picolimbo_proto::Encodeable)]
        pub struct $name {
            $(
                $(#[$attr])?
                pub $field_name: $field_type,
            )*
        }
    };
    (__dec $dec_id:literal then $name:ident: $($field_name:ident: $field_type:ty $(as $attr:meta)?),*) => {
        #[derive(Debug, Clone, PartialEq, picolimbo_proto::Decodeable)]
        pub struct $name {
            $(
                $(#[$attr])?
                pub $field_name: $field_type,
            )*
        }
    }
}

#[macro_export]
macro_rules! byte_enum {
    (out $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::byte_enum!(__only_impl_out $name $($variant $idx),*);
    };
    (in $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::byte_enum!(__only_impl_in $name $($variant $idx),*);
    };
    (all $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::byte_enum!(__only_impl_in $name $($variant $idx),*);
        $crate::byte_enum!(__only_impl_out $name $($variant $idx),*);
    };
    (__only_impl_out $name:ident $($variant:ident $idx:literal),*) => {
        impl picolimbo_proto::Encodeable for $name {
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<()> {
                match self {
                    $(
                        Self::$variant => ($idx as i8).encode(buf, ver)
                    ),*
                }
            }

            fn predict_size(&self) -> usize {
                1
            }
        }
    };
    (__only_impl_in $name:ident $($variant:ident $idx:literal),*) => {
        impl picolimbo_proto::Decodeable for $name {
            fn decode(read: &mut std::io::Cursor<&[u8]>, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<Self> {
                let idx = i8::decode(read, ver)?;
                match idx {
                    $(
                        $idx => Ok(Self::$variant),
                    )*
                    _ => Err(picolimbo_proto::ProtoError::EnumError(idx))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! varint_enum {
    (out $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::varint_enum!(__only_impl_out $name $($variant $idx),*);
    };
    (in $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::varint_enum!(__only_impl_in $name $($variant $idx),*);
    };
    (all $name:ident {
        $(
            $variant:ident = $idx:literal
        ),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $(
                $variant
            ),*
        }

        $crate::varint_enum!(__only_impl_in $name $($variant $idx),*);
        $crate::varint_enum!(__only_impl_out $name $($variant $idx),*);
    };
    (__only_impl_out $name:ident $($variant:ident $idx:literal),*) => {
        impl picolimbo_proto::Encodeable for $name {
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<()> {
                match self {
                    $(
                        Self::$variant => Varint($idx).encode(buf, ver)
                    ),*
                }
            }

            fn predict_size(&self) -> usize {
                Varint::size_of($idx)
            }
        }
    };
    (__only_impl_in $name:ident $($variant:ident $idx:literal),*) => {
        impl picolimbo_proto::Decodeable for $name {
            fn decode(read: &mut std::io::Cursor<&[u8]>, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<Self> {
                let idx = picolimbo_proto::Varint::decode(read, ver)?.0;
                match idx {
                    $(
                        $idx => Ok(Self::$variant),
                    )*
                    _ => Err(picolimbo_proto::ProtoError::EnumError(idx))
                }
            }
        }
    }
}
