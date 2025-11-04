use crate::{
    errors::VaultError,
    models::{Argon2Params, EncryptionData, PasswordEntry, PasswordList, Vault},
};

use std::fs::{self, File};
use std::io::prelude::*;

impl PasswordEntry {
    pub fn new(name: &str, username: &str, password: &str) -> PasswordEntry {
        PasswordEntry {
            name: name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl Vault {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: 1,
            argon2: Argon2Params::default(),
            encryption: EncryptionData::default(),
        }
    }

    pub fn new_from_file(file_name: String) -> Result<Vault, VaultError> {
        let file_path = format!("{}.vault", file_name);
        let file = File::open(file_path)?;
        let vault: Vault = serde_json::from_reader(file)?;
        Ok(vault)
    }

    pub fn delete(&self, master_password: &[u8]) -> Result<(), VaultError> {
        let vault_key = self.derive_vault_key(&master_password)?;
        self.decrypt_data(&vault_key)?;

        let file_path = format!("{}.vault", self.name.as_str());
        fs::remove_file(&file_path)?;
        Ok(())
    }

    pub fn list(&self, master_password: &[u8]) -> Result<PasswordList, VaultError> {
        let vault_key = self.derive_vault_key(&master_password)?;
        let plane_text = String::from_utf8(self.decrypt_data(&vault_key)?)?;
        let password_list: PasswordList = serde_json::from_str(&plane_text)?;
        Ok(password_list)
    }

    pub fn add_entry(
        &mut self,
        master_password: &[u8],
        password_entry: PasswordEntry,
    ) -> Result<(), VaultError> {
        let mut password_list = self.list(&master_password)?;
        if password_list
            .passwords
            .iter()
            .any(|entry| entry.name == password_entry.name)
        {
            return Err(VaultError::DuplicateEntry(password_entry.name.clone()));
        }
        password_list.passwords.push(password_entry);
        self.encrypt_data(
            &self.derive_vault_key(&master_password)?,
            serde_json::to_string_pretty(&password_list)?.as_bytes(),
        )?;
        self.save_to_file()?;

        Ok(())
    }
    pub fn remove_entry(&mut self, master_password: &[u8], name: &str) -> Result<(), VaultError> {
        let mut password_list = self.list(&master_password)?;
        if let Some(index) = password_list
            .passwords
            .iter()
            .position(|entry| entry.name == name)
        {
            password_list.passwords.remove(index);

            self.encrypt_data(
                &self.derive_vault_key(&master_password)?,
                serde_json::to_string_pretty(&password_list)?.as_bytes(),
            )?;
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn save_to_file(&self) -> Result<(), VaultError> {
        let json = serde_json::to_string_pretty(&self)?;

        let file_name = format!("{}.vault", self.name.as_str());

        let mut file = File::create(file_name)?;

        file.write_all(json.as_ref())?;

        Ok(())
    }
}
