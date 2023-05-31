use lobsterchat::component::Component;
use picolimbo_proto::{nbt::Blob, Encodeable, Identifier, Protocol, Varint};
use uuid::Uuid;

use crate::{build_packets, byte_enum, varint_enum};

use super::login::PlayerProperty;

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

// We can't really use automatic packet building because of different protocol versions
build_packets! { Play:
    // Clientbound
    packet PluginMessageOut(out 0x17) {
        channel: Identifier,
        data: Vec<u8> as unprefixed
    };

    packet PlayLogin(out 0x28) {
        eid: i32,
        is_hardcore: bool,
        gamemode: Gamemode,
        prev_gamemode: Gamemode,
        dimensions: Vec<Identifier> as prefixed(Varint),
        registry_codec: Blob,
        spawn_dimension: Identifier,
        dimension_name: Identifier,
        hashed_seed: i64,
        max_players: i32 as varint,
        view_distance: i32 as varint,
        simulation_distance: i32 as varint,
        reduced_debug_info: bool,
        enable_respawn_screen: bool,
        is_debug: bool,
        is_flat: bool,
        has_death_pos: bool // ALWAYS FALSE
    };

    packet SetHeldItem(out 0x4D) {
        slot: i8
    };

    packet UpdateRecipes(out 0x6D) {
        num_recipes: i32 as varint, // none for now
    };

    packet UpdateTags(out 0x6E) {
        num_tags: i32 as varint, // none for now
    };

    packet EntityEvent(out 0x1C) {
        eid: i32,
        status: EntityStatusPlayer
    };

    packet SendCommands(out 0x10) {
        num_nodes: i32 as varint, // none for now
    };

    packet SyncPlayerPosition(out 0x3C) {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: i8, // dont touch
        tp_id: i32 as varint
    };

    packet PlayerPosition(out 0x14) {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool
    };

    packet PlayerPositionRotation(out 0x15) {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool
    };

    packet PlayerInfo(out 0x3A) {
        actions_bitmask: i8,
        actions: Vec<PlayerAction> as prefixed(Varint)
    };

    packet SetCenterChunk(out 0x4E) {
        c_x: i16 as varint,
        c_y: i16 as varint,
    };

    packet KeepAliveClientbound(out 0x23) {
        ka_id: i64 as varint,
    };

    packet PlayDisconnect(out 0x1A) {
        reason: Component
    };

    // Serverbound
    packet ClientInformation(in 0x08) {
        locale: String,
        view_distance: i8,
        chat_mode: ChatMode,
        chat_colored: bool,
        displayed_skin_parts: u8,
        main_hand: MainHand,
        enable_text_filtering: bool,
        allow_server_listing: bool
    };

    packet PluginMessageIn(in 0x0D) {
        channel: Identifier,
        data: Vec<u8> as unprefixed
    };

    packet ConfirmTeleportation(in 0x00) {
        tp_id: i32 as varint
    };

    packet KeepAliveServerbound(in 0x12) {
        ka_id: i64
    };
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PlayerAction {
    AddPlayer(PlayerActionAddPlayer),
    UpdateLatency(PlayerActionUpdateLatency),
}

impl Encodeable for PlayerAction {
    fn encode(
        &self,
        out: &mut picolimbo_proto::BytesMut,
        ver: Protocol,
    ) -> picolimbo_proto::Result<()> {
        match self {
            PlayerAction::AddPlayer(add) => add.encode(out, ver),
            PlayerAction::UpdateLatency(upd) => upd.encode(out, ver),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encodeable)]
pub struct PlayerActionAddPlayer {
    pub player_id: Uuid,
    pub name: String,
    #[prefixed(Varint)]
    pub props: Vec<PlayerProperty>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Encodeable)]
pub struct PlayerActionUpdateLatency {
    pub player_id: Uuid,
    #[varint]
    pub ping: i32,
}
