use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::{client::ClientStream, handle::do_initial_handle};

pub async fn setup_server(ip: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(ip).await?;

    tracing::info!("Limbo server listening on {}", ip);

    while let Ok((client_stream, _client_ip)) = listener.accept().await {
        let client = ClientStream::new(client_stream);
        if let Err(err) = do_initial_handle(client).await {
            tracing::warn!("Failed to handle client: {err}");
        }
    }

    Ok(())
}
