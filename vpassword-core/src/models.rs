use std::path::PathBuf;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Ok,
    Error(String),
    PasswordEntry { entry: PasswordEntry },
    PasswordList { list: PasswordList },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    UnlockVault {
        vault_path: PathBuf,
        master_password: Vec<u8>,
    },
    LockVault,

    ListEntries,
    GetEntry {
        name: String,
    },
    AddEntry {
        entry: PasswordEntry,
    },
    RemoveEntry {
        name: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct PasswordEntry {
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PasswordList {
    pub passwords: Vec<PasswordEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argon2Params {
    pub salt: String,
    pub mem_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EncryptionData {
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vault {
    pub name: String,
    pub path: PathBuf,
    pub version: u8,
    pub argon2: Argon2Params,
    pub encryption: EncryptionData,
}

impl Default for Argon2Params {
    fn default() -> Self {
        let mut salt = [0u8; 32];
        OsRng.try_fill_bytes(&mut salt).unwrap();
        Self {
            salt: STANDARD.encode(salt),
            mem_cost: 64 * 1024,
            time_cost: 3,
            parallelism: 1,
        }
    }
}
