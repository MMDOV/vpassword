use std::sync::Arc;
use tokio::{net::UnixListener, sync::Mutex};
mod agent;
mod handlers;
mod models;
use handlers::handle_client;
use models::AgentState;

// TODO: we need to be sending stuff back (results, errors, etc,)
// FIX: clean up of sock file after exiting
#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AgentState::empty()));
    let listener = UnixListener::bind("/tmp/vault.sock").unwrap();

    loop {
        let clone_for_task = Arc::clone(&state);
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    println!("new client!");
                    handle_client(stream, clone_for_task).await.unwrap();
                });
            }
            Err(e) => eprintln!("accept failed: {e}"),
        }
    }
}
