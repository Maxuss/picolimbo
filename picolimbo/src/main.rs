pub mod client;
pub mod config;
pub mod dim;
pub mod handle;
pub mod player;
pub mod proto;
pub mod server;

use std::path::PathBuf;

use clap::Parser;

use config::{load_config, save_default_config};
use server::setup_server;

use tracing_subscriber::{
    filter::filter_fn, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to the config file
    #[arg(short, long, default_value = "limbo.conf")]
    config_path: PathBuf,
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(true)
                .with_filter(filter_fn(|f| {
                    f.module_path().unwrap_or_default().starts_with("picolimbo")
                        && *f.level() <= tracing::Level::DEBUG
                })),
        )
        .init();

    let args = Args::parse();
    if !args.config_path.exists() {
        save_default_config(args.config_path.clone())?;
        tracing::info!(
            "Created default config file at {}",
            args.config_path.display()
        );
    }
    let config = load_config(args.config_path.clone())?;
    tracing::info!("Loaded config from {}", args.config_path.display());

    setup_server(config).await
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use lobsterchat::component::NamedColor;
    use picolimbo_proto::{BytesMut, Decodeable, Encodeable, Identifier, Protocol, Result};
    use uuid::Uuid;

    #[derive(Encodeable, Decodeable, Debug, PartialEq)]
    struct TestDerived {
        #[varint]
        some_number: i16,
        some_string: String,
        some_id: Identifier,
        some_uuid: Uuid,
    }

    #[derive(Encodeable, Decodeable, Debug, PartialEq)]
    struct TestTuple(#[varint] i16, String, Identifier, Uuid);

    #[test]
    fn test_derived_preserve() -> Result<()> {
        let mut buf = BytesMut::new();
        let original = TestDerived {
            some_number: 0x1234,
            some_string: String::from("Hello, world!"),
            some_id: Identifier::from("minecraft:stone"),
            some_uuid: Uuid::new_v4(),
        };
        original.encode(&mut buf, Protocol::latest())?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestDerived::decode(&mut reader, Protocol::latest())?;
        assert_eq!(original, decoded);
        Ok(())
    }

    #[test]
    fn test_tuple_derived_preserve() -> Result<()> {
        let mut buf = BytesMut::new();
        let original = TestTuple(
            0x1234,
            String::from("Hello, world!"),
            Identifier::from("minecraft:stone"),
            Uuid::new_v4(),
        );
        original.encode(&mut buf, Protocol::latest())?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestTuple::decode(&mut reader, Protocol::latest())?;
        assert_eq!(original, decoded);
        Ok(())
    }

    #[test]
    fn test_tuple_struct_interop() -> Result<()> {
        let mut buf = BytesMut::new();
        let uid = Uuid::new_v4();
        let original = TestTuple(
            0x1234,
            String::from("Hello, world!"),
            Identifier::from("minecraft:stone"),
            uid.clone(),
        );
        original.encode(&mut buf, Protocol::latest())?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestDerived::decode(&mut reader, Protocol::latest())?;
        assert_eq!(
            TestDerived {
                some_number: 0x1234,
                some_string: String::from("Hello, world!"),
                some_id: Identifier::from("minecraft:stone"),
                some_uuid: uid
            },
            decoded
        );

        Ok(())
    }
}
