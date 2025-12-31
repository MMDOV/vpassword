use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
    time::Instant,
};

use vpassword_core::models::{Request, Response, Vault};

use crate::AgentState;

// TODO: expiration time
// TODO: better handling of vault state
async fn handle_request(request: Request, state: Arc<Mutex<AgentState>>) -> Response {
    let mut guard = state.lock().await;
    if guard.last_activity.is_some() {
        let time_since_last_activity = guard.last_activity.unwrap().elapsed().as_secs();
        if time_since_last_activity >= 300 {
            return match guard.lock_vault() {
                Ok(_) => {
                    Response::Error("Session timed out. You need to run open again.".to_string())
                }
                Err(e) => Response::Error(e.to_string()),
            };
        } else {
            guard.last_activity = Some(Instant::now())
        }
    }

    match request {
        Request::UnlockVault {
            vault_path,
            master_password,
        } => {
            if guard.vault_key.is_some() {
                return Response::Error("a vault is already open".to_string());
            }
            let vault_key = match Vault::new_from_file(&vault_path) {
                Ok(v) => match v.unlock_and_get_key(master_password.as_ref()) {
                    Ok(key) => key,
                    Err(e) => return Response::Error(e.to_string()),
                },
                Err(e) => return Response::Error(e.to_string()),
            };
            return match guard.unlock_vault(vault_path, vault_key) {
                Ok(_) => {
                    println!("sending back unlcok");
                    Response::Ok
                }
                Err(e) => Response::Error(e.to_string()),
            };
        }
        Request::LockVault => {
            return match guard.lock_vault() {
                Ok(_) => Response::Ok,
                Err(e) => Response::Error(e.to_string()),
            };
        }
        Request::ListEntries => {
            if guard.vault_key.is_some() {
                let vault = match Vault::new_from_file(guard.vault_path.as_ref().unwrap()) {
                    Ok(vault) => vault,
                    Err(e) => return Response::Error(e.to_string()),
                };
                match vault.list(guard.vault_key.as_ref().unwrap()) {
                    Ok(list) => Response::PasswordList { list },
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("No vault is open".to_string())
            }
        }
        Request::GetEntry { name } => {
            if guard.vault_key.is_none() {
                return Response::Error("No vault is open".to_string());
            }
            let mut vault = match Vault::new_from_file(guard.vault_path.as_ref().unwrap()) {
                Ok(vault) => vault,
                Err(e) => return Response::Error(e.to_string()),
            };
            match vault.get_entry(guard.vault_key.as_ref().unwrap(), name.as_ref()) {
                Ok(entry) => Response::PasswordEntry { entry },
                Err(e) => Response::Error(e.to_string()),
            }
        }
        Request::AddEntry { entry } => {
            if guard.vault_key.is_none() {
                return Response::Error("No vault is open".to_string());
            }
            let mut vault = match Vault::new_from_file(guard.vault_path.as_ref().unwrap()) {
                Ok(vault) => vault,
                Err(e) => return Response::Error(e.to_string()),
            };
            match vault.add_entry(guard.vault_key.as_ref().unwrap(), entry) {
                Ok(_) => Response::Ok,
                Err(e) => Response::Error(e.to_string()),
            }
        }
        Request::RemoveEntry { name } => {
            if guard.vault_key.is_none() {
                return Response::Error("No vault is open".to_string());
            }
            let mut vault = match Vault::new_from_file(guard.vault_path.as_ref().unwrap()) {
                Ok(vault) => vault,
                Err(e) => return Response::Error(e.to_string()),
            };
            match vault.remove_entry(guard.vault_key.as_ref().unwrap(), name.as_ref()) {
                Ok(_) => Response::Ok,
                Err(e) => Response::Error(e.to_string()),
            }
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
    let response = handle_request(request, state).await;
    let response_bytes = serde_json::to_vec(&response)?;
    stream.write_all(&response_bytes).await?;

    Ok(())
}
