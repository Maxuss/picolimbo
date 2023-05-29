pub mod client;
pub mod proto;
pub mod server;

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

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
    /// IP to which bind this limbo. Can also be specified in the config
    #[arg(short, long, default_value = "127.0.0.1:24431")]
    ip: SocketAddr,
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
    if args.config_path.exists() {
        // TODO: parse config
    }

    setup_server(args.ip).await
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use lobsterchat::component::{Component, NamedColor};
    use picolimbo_proto::{BytesMut, Decodeable, Encodeable, Identifier, Result};
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
        original.encode(&mut buf)?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestDerived::decode(&mut reader)?;
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
        original.encode(&mut buf)?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestTuple::decode(&mut reader)?;
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
        original.encode(&mut buf)?;

        let mut reader = Cursor::new(&buf[..]);
        let decoded = TestDerived::decode(&mut reader)?;
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
