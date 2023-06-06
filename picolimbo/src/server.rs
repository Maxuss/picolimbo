use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use tokio::net::TcpListener;

use crate::{client::ClientStream, config::LimboConfig, handle::handle_client};

#[derive(Debug, Clone)]
pub struct LimboServer(Arc<LimboServerInner>);

impl LimboServer {
    pub fn try_add_player(&self) -> bool {
        let players = self.0.player_count.current_players.load(Ordering::SeqCst);
        if players + 1 > self.0.player_count.max_players {
            false
        } else {
            self.0
                .player_count
                .current_players
                .fetch_add(1, Ordering::SeqCst);
            true
        }
    }

    pub fn online_players(&self) -> u32 {
        self.0.player_count.current_players.load(Ordering::SeqCst)
    }

    pub fn remove_player(&self) {
        self.0
            .player_count
            .current_players
            .fetch_sub(1, Ordering::SeqCst);
    }

    pub fn config(&self) -> &LimboConfig {
        &self.0.config
    }
}

#[derive(Debug)]
pub struct LimboServerInner {
    player_count: PlayerCount,
    config: LimboConfig,
}

#[derive(Debug)]
pub struct PlayerCount {
    current_players: AtomicU32,
    max_players: u32,
}

pub async fn setup_server(cfg: LimboConfig) -> anyhow::Result<()> {
    let listener = TcpListener::bind(cfg.address).await?;

    tracing::info!("Limbo server listening on {}", cfg.address);

    let server = LimboServer(Arc::new(LimboServerInner {
        player_count: PlayerCount {
            current_players: AtomicU32::new(0),
            max_players: cfg.max_players,
        },
        config: cfg,
    }));

    while let Ok((client_stream, addr)) = listener.accept().await {
        let client = ClientStream::new(client_stream);
        let server_c = server.clone();
        tokio::task::spawn(async move {
            // all errors are already handled in underlying methods
            let _ = handle_client(client, addr, server_c).await;
        });
    }

    Ok(())
}
