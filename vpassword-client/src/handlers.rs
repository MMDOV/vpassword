use crate::cli::{Commands, PasswordCommands, VaultCommands};
use passwords::PasswordGenerator;
use vpassword_core::models::{PasswordEntry, Vault};

pub fn handle_command(command: Commands) {
    match command {
        Commands::Vault { command } => handle_vault(command),
        Commands::Password { command } => handle_password(command),
    }
}

fn handle_vault(command: VaultCommands) {
    match command {
        VaultCommands::Create {
            vault_name,
            master_pass,
        } => {
            let mut vault = Vault::new(&vault_name);
            let vault_key = vault
                .derive_vault_key(master_pass.as_ref())
                .expect("Error trying to encrypt master key");
            vault
                .encrypt_data(&vault_key, b"")
                .expect("Error encrypting data");
            vault.save_to_file().expect("Error saving to file")
        }
        VaultCommands::Remove {
            vault_name,
            master_pass,
        } => {
            let vault = Vault::new_from_file(vault_name).expect("Errcr trying to load the vault");
            vault
                .delete(master_pass.as_ref())
                .expect("Error removing vault");
        }
        VaultCommands::List {
            vault_name,
            master_pass,
        } => {
            let vault = Vault::new_from_file(vault_name).expect("Error trying to load the vault");
            let list = serde_json::to_string_pretty(
                &vault
                    .list(master_pass.as_ref())
                    .expect("Error trying to list vault"),
            )
            .expect("Error");
            println!("{}", &list);
        }
    }
}

fn handle_password(command: PasswordCommands) {
    match command {
        PasswordCommands::Add {
            vault_name,
            master_pass,
            name,
            username,
            password,
        } => {
            let mut vault =
                Vault::new_from_file(vault_name).expect("Error trying to load the vault");

            let user_password = if password.to_lowercase() == "generate" {
                let pg = PasswordGenerator {
                    length: 15,
                    numbers: true,
                    lowercase_letters: true,
                    uppercase_letters: true,
                    symbols: true,
                    spaces: false,
                    exclude_similar_characters: false,
                    strict: true,
                };
                pg.generate_one().expect("Error generating password")
            } else {
                password
            };

            let password_entry = PasswordEntry::new(&name, &username, &user_password);
            println!("{:#?}", password_entry);

            vault
                .add_entry(master_pass.as_ref(), password_entry)
                .expect("Failed to add entry");
        }
        PasswordCommands::Remove {
            vault_name,
            master_pass,
            name,
        } => {
            let mut vault =
                Vault::new_from_file(vault_name).expect("Error trying to load the vault");
            vault
                .remove_entry(master_pass.as_ref(), &name)
                .expect("Failed to remove entry");
        }
        PasswordCommands::Generate {} => {
            let pg = PasswordGenerator {
                length: 15,
                numbers: true,
                lowercase_letters: true,
                uppercase_letters: true,
                symbols: true,
                spaces: false,
                exclude_similar_characters: false,
                strict: true,
            };
            let generated_pass = pg.generate_one().expect("Error generating password");
            println!("{}", generated_pass);
        }
    }
}
