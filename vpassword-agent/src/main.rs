use tokio::net::{UnixListener, UnixStream};

#[tokio::main]
async fn main() {
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
    println!("Client said: {}", message);

    // Echo back
    stream.write_all(b"Message received").await?;

    Ok(())
}
