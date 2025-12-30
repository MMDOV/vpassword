use crate::cli::Commands;
use passwords::PasswordGenerator;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use vpassword_core::models::{PasswordEntry, Request, Response, Vault};

pub async fn handle_command(command: Commands) {
    match command {
        Commands::Init { vault_path } => {
            handle_init(vault_path);
        }
        _ => {
            let stream = match UnixStream::connect("/tmp/vault.sock").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Could not connect to agent: {e}");
                    eprintln!("Make sure 'vault-agent' is running.");
                    std::process::exit(1);
                }
            };

            handle_agent_command(command, stream).await;
        }
    }
}
pub fn handle_init(vault_path: std::path::PathBuf) {
    let mut vault = Vault::new(&vault_path);
    let master_password = rpassword::prompt_password("Your master password: ").unwrap();
    let vault_key = vault
        .derive_vault_key(master_password.as_ref())
        .expect("Error trying to encrypt master key");
    vault
        .encrypt_data(&vault_key, b"")
        .expect("Error encrypting data");
    vault.save_to_file().expect("Error saving to file");
    println!("Vault initialized at {:?}", vault_path);
}

pub async fn handle_agent_command(command: Commands, stream: UnixStream) {
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
            let response = send_request_to_agent(
                stream,
                Request::UnlockVault {
                    vault_path,
                    master_password: master_password.into_bytes(),
                },
            )
            .await;
            match response {
                Response::Ok => println!("Vault is Opened!"),
                Response::Error(e) => println!("Problem Openning Vault: {e}"),
                _ => eprintln!("Unexpected response type."),
            };
        }
        Commands::Close => match send_request_to_agent(stream, Request::LockVault).await {
            Response::Ok => println!("Vault sucessfully closed!"),
            Response::Error(e) => println!("Problem closing Vault: {e}"),
            _ => eprintln!("Unexpected response type."),
        },
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
            match send_request_to_agent(
                stream,
                Request::AddEntry {
                    entry: password_entry,
                },
            )
            .await
            {
                Response::Ok => println!(
                    "Entry {name} with user: {username} and password: {user_password} added to vault!"
                ),
                Response::Error(e) => println!("Error trying to add entry: {e}"),
                _ => eprintln!("Unexpected response type."),
            }
        }
        Commands::Show { name } => {
            match send_request_to_agent(stream, Request::GetEntry { name }).await {
                Response::PasswordEntry { entry } => {
                    println!(
                        "Entry found:\nName: {}\nUsername: {}\nPassword: {}",
                        entry.name, entry.username, entry.password
                    );
                }
                Response::Error(e) => eprintln!("Error: {}", e),
                _ => eprintln!("Unexpected response type."),
            }
        }
        Commands::List => match send_request_to_agent(stream, Request::ListEntries).await {
            Response::PasswordList { list } => {
                for entry in list.passwords {
                    println!(
                        "Name: {}\nUsername: {}\nPassword: {}",
                        entry.name, entry.username, entry.password
                    );
                }
            }
            Response::Error(e) => eprintln!("Error: {}", e),
            _ => eprintln!("Unexpected response type."),
        },

        Commands::Remove { name } => {
            match send_request_to_agent(stream, Request::RemoveEntry { name }).await {
                Response::Ok => println!("Sucessfully Removed Entry."),
                Response::Error(e) => eprintln!("Error: {}", e),
                _ => eprintln!("Unexpected response type."),
            }
        }

        Commands::Add { name, username } => {
            let user_password = rpassword::prompt_password("Your password: ").unwrap();
            let password_entry = PasswordEntry::new(&name, &username, &user_password);
            let response = send_request_to_agent(
                stream,
                Request::AddEntry {
                    entry: password_entry,
                },
            )
            .await;
            match response {
                Response::Ok => println!(
                    "Entry {name} with user: {username} and password: {user_password} added to vault!"
                ),
                Response::Error(e) => println!("Error trying to add entry: {e}"),
                _ => eprintln!("Unexpected response type."),
            }
        }
    }
}

async fn send_request_to_agent(mut stream: UnixStream, request: Request) -> Response {
    let json_bytes = serde_json::to_vec(&request).expect("Serialization failed");
    stream
        .write_all(&json_bytes)
        .await
        .expect("Failed to write to socket");

    let mut buf = vec![0u8; 4096]; // A reasonable buffer size
    let n = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from socket");

    if n == 0 {
        panic!("Agent closed connection unexpectedly");
    }

    serde_json::from_slice(&buf[..n]).expect("Failed to parse response")
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
