use std::{io::Error, string::FromUtf8Error};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Serde JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("Argon2 error: {0}")]
    Argon2(String),

    #[error("AEAD encryption/decryption error")]
    Aead,

    #[error("duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("no such entry: {0}")]
    NoSuchEntry(String),
}

impl From<argon2::Error> for VaultError {
    fn from(e: argon2::Error) -> Self {
        VaultError::Argon2(e.to_string())
    }
}

impl From<aes_gcm::aead::Error> for VaultError {
    fn from(_: aes_gcm::aead::Error) -> Self {
        VaultError::Aead
    }
}
