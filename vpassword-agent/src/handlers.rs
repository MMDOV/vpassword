use std::sync::Arc;
use tokio::sync::Mutex;

use vpassword_core::models::{Request, RequestType, Vault};

use crate::AgentState;

pub async fn handle_request(request: Request, state: Arc<Mutex<AgentState>>) {
    match request.request_type {
        RequestType::VaultOpen => {
            let vault_key = Vault::new_from_file(request.vault_name)
                .expect("Error trying to load the vault")
                .unlock_and_get_key(request.master_password.as_ref())
                .unwrap();
        }
        RequestType::VaultList => {}
        RequestType::VaultRemove => {}
        RequestType::PasswordAdd => {}
        RequestType::PasswordRemove => {}
    }
}
