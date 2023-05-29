use std::net::SocketAddr;

use tokio::net::TcpListener;

pub async fn setup_server(ip: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(ip).await?;

    tracing::info!("Limbo server listening on {}", ip);

    while let Ok((mut client_stream, client_ip)) = listener.accept().await {
        tracing::debug!("Receiving connection from {client_ip}");
        // TODO: finish packets
    }

    Ok(())
}
