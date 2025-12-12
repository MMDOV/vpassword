use std::path::PathBuf;
use zeroize::Zeroizing;

pub struct AgentState {
    pub vault_key: Option<Zeroizing<Vec<u8>>>,
    pub vault_path: Option<PathBuf>,
}
