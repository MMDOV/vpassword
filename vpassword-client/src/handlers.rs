use crate::cli::Commands;
use passwords::PasswordGenerator;
use tokio::net::UnixStream;
use vpassword_core::models::{PasswordEntry, Request, Vault};

pub async fn handle_command(command: Commands, stream: UnixStream) {
    match command {
        Commands::Init { vault_path } => {
            let mut vault = Vault::new(&vault_path);
            let master_password = rpassword::prompt_password("Your master password: ").unwrap();
            let vault_key = vault
                .derive_vault_key(master_password.as_ref())
                .expect("Error trying to encrypt master key");
            vault
                .encrypt_data(&vault_key, b"")
                .expect("Error encrypting data");
            vault.save_to_file().expect("Error saving to file")
        }
        Commands::Open { vault_path } => {
            // NOTE: check if vault exists
            // check if a vault is already open
            // check if that vault is this vault
            // if so prompt user with appropriate message
            // if not prompt for master password
            // send open request to agent
            // send appropriate message based on if open or not
            let master_password = rpassword::prompt_password("Your master password: ").unwrap();
            send_request_to_agent(
                stream,
                Request::UnlockVault {
                    vault_path,
                    master_password: master_password.into_bytes(),
                },
            )
            .await;
        }
        Commands::Close => send_request_to_agent(stream, Request::LockVault).await,
        Commands::Generate { name, username } => {
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
            let user_password = pg.generate_one().expect("Error generating password");
            let password_entry = PasswordEntry::new(&name, &username, &user_password);
            send_request_to_agent(
                stream,
                Request::AddEntry {
                    entry: password_entry,
                },
            )
            .await;
        }
        Commands::Show { name } => send_request_to_agent(stream, Request::GetEntry { name }).await,
        Commands::List => send_request_to_agent(stream, Request::ListEntries).await,

        Commands::Remove { name } => {
            send_request_to_agent(stream, Request::RemoveEntry { name }).await
        }

        Commands::Add { name, username } => {
            let user_password = rpassword::prompt_password("Your password: ").unwrap();
            let password_entry = PasswordEntry::new(&name, &username, &user_password);
            send_request_to_agent(
                stream,
                Request::AddEntry {
                    entry: password_entry,
                },
            )
            .await;
        }
    }
}

async fn send_request_to_agent(stream: UnixStream, request: Request) {
    let json = serde_json::to_string(&request).unwrap();
    loop {
        stream.writable().await.unwrap();

        match stream.try_write(json.as_bytes()) {
            Ok(_) => {
                println!("sent");
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => {
                println!("err");
            }
        }
    }
}

//fn handle_vault(command: VaultCommands) {
//    match command {
//        VaultCommands::Create {
//            vault_name,
//            master_pass,
//        } => {
//            let mut vault = Vault::new(&vault_name);
//            let vault_key = vault
//                .derive_vault_key(master_pass.as_ref())
//                .expect("Error trying to encrypt master key");
//            vault
//                .encrypt_data(&vault_key, b"")
//                .expect("Error encrypting data");
//            vault.save_to_file().expect("Error saving to file")
//        }
//        VaultCommands::Remove {
//            vault_name,
//            master_pass,
//        } => {
//            let vault = Vault::new_from_file(vault_name).expect("Errcr trying to load the vault");
//            vault
//                .delete(master_pass.as_ref())
//                .expect("Error removing vault");
//        }
//        VaultCommands::List {
//            vault_name,
//            master_pass,
//        } => {
//            let vault = Vault::new_from_file(vault_name).expect("Error trying to load the vault");
//            let list = serde_json::to_string_pretty(
//                &vault
//                    .list(master_pass.as_ref())
//                    .expect("Error trying to list vault"),
//            )
//            .expect("Error");
//            println!("{}", &list);
//        }
//    }
//}
//
//fn handle_password(command: PasswordCommands) {
//    match command {
//        PasswordCommands::Add {
//            vault_name,
//            master_pass,
//            name,
//            username,
//            password,
//        } => {
//            let mut vault =
//                Vault::new_from_file(vault_name).expect("Error trying to load the vault");
//
//            let user_password = if password.to_lowercase() == "generate" {
//                let pg = PasswordGenerator {
//                    length: 15,
//                    numbers: true,
//                    lowercase_letters: true,
//                    uppercase_letters: true,
//                    symbols: true,
//                    spaces: false,
//                    exclude_similar_characters: false,
//                    strict: true,
//                };
//                pg.generate_one().expect("Error generating password")
//            } else {
//                password
//            };
//
//            let password_entry = PasswordEntry::new(&name, &username, &user_password);
//            println!("{:#?}", password_entry);
//
//            vault
//                .add_entry(master_pass.as_ref(), password_entry)
//                .expect("Failed to add entry");
//        }
//        PasswordCommands::Remove {
//            vault_name,
//            master_pass,
//            name,
//        } => {
//            let mut vault =
//                Vault::new_from_file(vault_name).expect("Error trying to load the vault");
//            vault
//                .remove_entry(master_pass.as_ref(), &name)
//                .expect("Failed to remove entry");
//        }
//        PasswordCommands::Generate {} => {
//            let pg = PasswordGenerator {
//                length: 15,
//                numbers: true,
//                lowercase_letters: true,
//                uppercase_letters: true,
//                symbols: true,
//                spaces: false,
//                exclude_similar_characters: false,
//                strict: true,
//            };
//            let generated_pass = pg.generate_one().expect("Error generating password");
//            println!("{}", generated_pass);
//        }
//    }
//}
