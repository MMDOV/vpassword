use crate::AgentState;

impl AgentState {
    pub fn empty() -> Self {
        AgentState {
            vault_key: None,
            vault_path: None,
        }
    }

    pub fn clear_key(&mut self) {
        self.vault_key = None;
    }
}
