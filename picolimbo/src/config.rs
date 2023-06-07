use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::SocketAddr,
    path::PathBuf,
};

use anyhow::bail;
use lobsterchat::{component::Component, lobster};
use picolimbo_proto::Protocol;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
struct ConfigContainer {
    limbo: LimboConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimboConfig {
    pub address: SocketAddr,
    #[serde(default)]
    #[serde(rename = "default protocol")]
    pub default_protocol_version: Protocol,
    #[serde(rename = "max players")]
    pub max_players: u32,
    #[serde(default)]
    #[serde(rename = "server full message")]
    #[serde(deserialize_with = "deserialize_opt_component")]
    pub full_message: Option<Component>,
    #[serde(deserialize_with = "deserialize_component")]
    pub motd: Component,
    #[serde(rename = "brand")]
    pub server_brand: String,
    #[serde(rename = "dimension")]
    pub dimension: String,

    #[serde(default)]
    #[serde(rename = "on join")]
    pub on_join_actions: Vec<LimboJoinAction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LimboJoinAction {
    SendMessage {
        #[serde(rename = "send message")]
        #[serde(deserialize_with = "deserialize_component")]
        send_message: Component,
    },
    SendTitle {
        #[serde(rename = "send title")]
        send_title: TitleData,
    },
    SendBossbar {
        #[serde(rename = "send bossbar")]
        send_bossbar: BossbarData,
    },
    SendPluginMessage {
        #[serde(rename = "send plugin message")]
        send_plugin_message: PluginMessageData,
    },
    SendActionBar {
        #[serde(rename = "send action bar")]
        #[serde(deserialize_with = "deserialize_component")]
        send_action_bar: Component,
    },
    MapForVersions {
        #[serde(rename = "match version")]
        match_version: HashMap<Protocol, LimboJoinAction>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct TitleData {
    #[serde(default)]
    #[serde(rename = "fade in")]
    pub fade_in: Option<i32>,
    #[serde(default)]
    pub stay: Option<i32>,
    #[serde(default)]
    #[serde(rename = "fade out")]
    pub fade_out: Option<i32>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_opt_component")]
    pub title: Option<Component>,
    #[serde(deserialize_with = "deserialize_opt_component")]
    pub subtitle: Option<Component>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BossbarData {
    #[serde(deserialize_with = "deserialize_component")]
    pub title: Component,
    #[serde(default)]
    pub progress: f32,
    pub color: BossbarColor,
    pub notches: BossbarNotches,
    #[serde(default)]
    #[serde(rename = "darkens sky")]
    pub darkens_sky: Option<bool>,
    #[serde(default)]
    #[serde(rename = "is dragon bar")]
    pub is_dragon_bar: Option<bool>,
    #[serde(default)]
    #[serde(rename = "create fog")]
    pub create_fog: Option<bool>,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum BossbarColor {
    Pink = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum BossbarNotches {
    None = 0,
    Six = 1,
    Ten = 2,
    Twelve = 3,
    Twenty = 4,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginMessageData {
    pub channel: String,
    pub message: String,
}

fn deserialize_component<'de, D: Deserializer<'de>, E: serde::de::Error>(
    de: D,
) -> std::result::Result<Component, E> {
    <String>::deserialize(de)
        .map_err(serde::de::Error::custom)
        .map(lobster)
}

fn deserialize_opt_component<'de, D: Deserializer<'de>, E: serde::de::Error>(
    de: D,
) -> std::result::Result<Option<Component>, E> {
    <Option<String>>::deserialize(de)
        .map_err(serde::de::Error::custom)
        .map(|it| it.map(lobster))
}

pub fn load_config(path: PathBuf) -> anyhow::Result<LimboConfig> {
    if !path.exists() {
        bail!("Config file does not exist!");
    }
    let mut cfg_file = File::open(path)?;
    let mut buf = String::with_capacity(cfg_file.metadata()?.len() as usize);
    cfg_file.read_to_string(&mut buf)?;
    hocon::de::from_str::<ConfigContainer>(&buf)
        .map(|it| it.limbo)
        .map_err(anyhow::Error::from)
}

pub fn save_default_config(path: PathBuf) -> anyhow::Result<()> {
    let default_config = include_str!("../res/limbo.conf");
    File::create(path)?.write_all(default_config.as_bytes())?;
    Ok(())
}
