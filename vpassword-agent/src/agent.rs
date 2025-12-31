use std::error::Error;
use std::path::PathBuf;
use tokio::time::Instant;
use zeroize::Zeroizing;

use crate::AgentState;

impl AgentState {
    pub fn new() -> Self {
        AgentState {
            vault_key: None,
            vault_path: None,
            last_activity: None,
        }
    }

    pub fn unlock_vault(&mut self, path: PathBuf, key: [u8; 32]) -> Result<(), Box<dyn Error>> {
        self.vault_path = Some(path);
        self.vault_key = Some(Zeroizing::new(key.to_vec()));
        self.last_activity = Some(Instant::now());
        Ok(())
    }

    pub fn lock_vault(&mut self) -> Result<(), Box<dyn Error>> {
        if self.vault_key.is_some() {
            self.vault_key = None;
        }
        if self.vault_path.is_some() {
            self.vault_path = None;
        }
        Ok(())
    }
}
