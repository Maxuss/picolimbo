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
    use lobsterchat::component::{Colored, Component, NamedColor};
    use picolimbo_proto::{BytesMut, Encodeable, Identifier, Result};

    #[derive(Encodeable)]
    struct TestDerived {
        #[varint]
        some_number: i16,
        some_string: &'static str,
        some_id: Identifier,
        some_chat: Component,
    }

    #[test]
    fn test_derived_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let value = TestDerived {
            some_number: 12345,
            some_string: "Hello, world!",
            some_id: "minecraft:diamond".into(),
            some_chat: Component::text("Some cool text")
                .color(NamedColor::Aqua)
                .bold(true),
        };
        value.encode(&mut buf)?;
        // dont worry, these numbers were generated
        assert_eq!(
            &[
                185, 96, 13, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 17, 109,
                105, 110, 101, 99, 114, 97, 102, 116, 58, 100, 105, 97, 109, 111, 110, 100, 52,
                123, 34, 98, 111, 108, 100, 34, 58, 116, 114, 117, 101, 44, 34, 99, 111, 108, 111,
                114, 34, 58, 34, 97, 113, 117, 97, 34, 44, 34, 116, 101, 120, 116, 34, 58, 34, 83,
                111, 109, 101, 32, 99, 111, 111, 108, 32, 116, 101, 120, 116, 34, 125
            ],
            &buf[..]
        );
        Ok(())
    }
}
