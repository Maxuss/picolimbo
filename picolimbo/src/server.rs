use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::{client::ClientStream, handle::do_initial_handle};

pub async fn setup_server(ip: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(ip).await?;

    tracing::info!("Limbo server listening on {}", ip);

    while let Ok((client_stream, addr)) = listener.accept().await {
        let client = ClientStream::new(client_stream);
        tokio::task::spawn(async move {
            let _ = do_initial_handle(client, addr).await;
        });
    }

    Ok(())
}
