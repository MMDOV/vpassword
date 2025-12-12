use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::UnixStream, sync::Mutex};
use zeroize::Zeroizing;

use vpassword_core::models::{Request, Vault};

use crate::AgentState;

// FIX: needs to send data back to client instead of printing and panicing
async fn handle_request(request: Request, state: Arc<Mutex<AgentState>>) {
    match request {
        Request::UnlockVault {
            vault_path,
            master_password,
        } => {
            let mut guard = state.lock().await;
            if guard.vault_key.is_some() {
                panic!("another vault is open close that first!")
            }
            let vault_key = Vault::new_from_file(&vault_path)
                .expect("Error trying to load the vault")
                .unlock_and_get_key(master_password.as_ref())
                .expect("wrong pass");
            guard.vault_key = Some(Zeroizing::new(vault_key.to_vec()));
            guard.vault_path = Some(vault_path);
            println!("{}", guard.vault_key.is_some());
        }
        Request::LockVault => {
            let mut guard = state.lock().await;
            if guard.vault_key.is_some() {
                guard.clear_key();
            } else {
                panic!("no vault is open")
            }
        }
        Request::ListEntries => {
            let guard = state.lock().await;
            if guard.vault_key.is_some() {
                let vault = Vault::new_from_file(&guard.vault_path.as_ref().unwrap())
                    .expect("Error trying to load the vault");
                let list = serde_json::to_string_pretty(
                    &vault
                        .list(guard.vault_key.as_ref().unwrap())
                        .expect("Error trying to list vault"),
                )
                .expect("Error");
                println!("{}", &list);
            }
        }
        Request::GetEntry { name } => {
            let guard = state.lock().await;
            if guard.vault_key.is_none() {
                panic!("No vault is open");
            }
            let mut vault = Vault::new_from_file(&guard.vault_path.as_ref().unwrap())
                .expect("Error trying to load the vault");
            let entry = vault
                .get_entry(guard.vault_key.as_ref().unwrap(), name.as_ref())
                .expect("failed to add entry");
            println!("{entry:?}")
        }
        Request::AddEntry { entry } => {
            let guard = state.lock().await;
            if guard.vault_key.is_none() {
                panic!("No vault is open");
            }
            let mut vault = Vault::new_from_file(&guard.vault_path.as_ref().unwrap())
                .expect("Error trying to load the vault");
            vault
                .add_entry(guard.vault_key.as_ref().unwrap(), entry)
                .expect("failed to add entry");
        }
        Request::RemoveEntry { name } => {
            let guard = state.lock().await;
            if guard.vault_key.is_none() {
                panic!("No vault is open");
            }
            let mut vault = Vault::new_from_file(&guard.vault_path.as_ref().unwrap())
                .expect("Error trying to load the vault");
            vault
                .remove_entry(guard.vault_key.as_ref().unwrap(), name.as_ref())
                .expect("failed to add entry");
        }
    }
}

pub async fn handle_client(
    mut stream: UnixStream,
    state: Arc<Mutex<AgentState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let message = String::from_utf8_lossy(&buf[..n]);
    let request: Request = serde_json::from_str(&message)?;
    handle_request(request, state).await;

    Ok(())
}
