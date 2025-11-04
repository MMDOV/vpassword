use crate::{errors::VaultError, models::Vault};
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit},
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose::STANDARD};
impl Vault {
    pub fn derive_vault_key(&self, master_password: &[u8]) -> Result<[u8; 32], VaultError> {
        let argon2_params = &self.argon2;
        let params = Params::new(
            argon2_params.mem_cost,
            argon2_params.time_cost,
            argon2_params.parallelism,
            Some(32),
        )?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut output_key = [0u8; 32];
        argon2.hash_password_into(
            master_password,
            &STANDARD.decode(&argon2_params.salt)?,
            &mut output_key,
        )?;

        Ok(output_key)
    }

    pub fn encrypt_data(&mut self, vault_key: &[u8], plaintext: &[u8]) -> Result<(), VaultError> {
        let cipher = Aes256Gcm::new(vault_key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

        let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;

        self.encryption.nonce = STANDARD.encode(nonce);
        self.encryption.ciphertext = STANDARD.encode(ciphertext);
        Ok(())
    }

    pub fn decrypt_data(&self, vault_key: &[u8]) -> Result<Vec<u8>, VaultError> {
        let cipher = Aes256Gcm::new(vault_key.into());
        let nonce = STANDARD.decode(&self.encryption.nonce)?;
        let ciphertext = STANDARD.decode(&self.encryption.ciphertext)?;
        let decrypted_text = cipher.decrypt(nonce.as_slice().into(), ciphertext.as_ref())?;

        Ok(decrypted_text)
    }
}
