use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{UnixListener, UnixStream},
    time::Instant,
};
use zeroize::Zeroizing;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AgentState::empty()));
    let listener = UnixListener::bind("/tmp/vault.sock").unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    println!("new client!");
                    handle_client(stream).await.unwrap();
                });
            }
            Err(e) => eprintln!("accept failed: {e}"),
        }
    }
}

async fn handle_client(mut stream: UnixStream) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let message = String::from_utf8_lossy(&buf[..n]);
    stream.write_all(b"Message received").await?;

    Ok(())
}

struct AgentState {
    master_key: Option<Zeroizing<Vec<u8>>>,
    expires_at: Option<Instant>,
}

impl AgentState {
    fn empty() -> Self {
        AgentState {
            master_key: None,
            expires_at: None,
        }
    }
}
