use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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
    let socket_path = "/tmp/vault.sock";

    if Path::new(socket_path).exists() {
        let _ = fs::remove_file(socket_path);
    }

    let state = Arc::new(Mutex::new(AgentState::new()));
    let listener = UnixListener::bind(socket_path).unwrap();
    let metadata = fs::metadata(socket_path).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(socket_path, permissions).unwrap();

    loop {
        let clone_for_task = Arc::clone(&state);
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    handle_client(stream, clone_for_task).await.unwrap();
                });
            }
            Err(e) => eprintln!("accept failed: {e}"),
        }
    }
}
