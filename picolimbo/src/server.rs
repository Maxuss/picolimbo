use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::{client::ClientStream, handle::do_initial_handle};

pub async fn setup_server(ip: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(ip).await?;

    tracing::info!("Limbo server listening on {}", ip);

    while let Ok((client_stream, _client_ip)) = listener.accept().await {
        let client = ClientStream::new(client_stream);
        if (do_initial_handle(client).await).is_err() {
            // Client disconnected
            continue;
        }
    }

    Ok(())
}
