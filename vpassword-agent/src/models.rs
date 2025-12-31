use std::path::PathBuf;
use tokio::time::Instant;
use zeroize::Zeroizing;

pub struct AgentState {
    pub vault_key: Option<Zeroizing<Vec<u8>>>,
    pub vault_path: Option<PathBuf>,
    pub last_activity: Option<Instant>,
}
