use crate::cli::Commands;
use crate::config::{get_config_path, load_config, save_config};
use crate::models::AppConfig;
use passwords::PasswordGenerator;
use std::{fs::read_dir, process::Command};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::Duration,
};
use vpassword_core::models::{PasswordEntry, PasswordList, Request, Response, Vault};

pub async fn handle_command(command: Commands) {
    match command {
        Commands::Init { vault_path } => {
            handle_init(vault_path);
        }
        _ => {
            let stream: UnixStream = match UnixStream::connect("/tmp/vault.sock").await {
                Ok(s) => s,
                Err(_) => {
                    println!("Agent not found. Attempting to start it..");

                    match Command::new("vpassword-agent").spawn() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error trying to start agent: {e}");
                            std::process::exit(1);
                        }
                    };
                    let mut attempts = 0;

                    loop {
                        if attempts >= 5 {
                            eprintln!("Error trying to start agent");
                            std::process::exit(1);
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        let stream = match UnixStream::connect("/tmp/vault.sock").await {
                            Ok(s) => s,
                            Err(_) => {
                                attempts += 1;
                                continue;
                            }
                        };
                        break stream;
                    }
                }
            };
            println!("Coneected to agent!");

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
        .encrypt_data(
            &vault_key,
            serde_json::to_string(&PasswordList::default())
                .unwrap()
                .as_bytes(),
        )
        .expect("Error encrypting data");
    vault.save_to_file().expect("Error saving to file");
    println!("Vault initialized at {:?}", vault_path);
}

pub async fn handle_agent_command(command: Commands, stream: UnixStream) {
    match command {
        Commands::Init { vault_path: _ } => {}
        Commands::Open { vault_path } => {
            let path = match vault_path {
                Some(vault_path) => {
                    if vault_path.components().count() > 1 {
                        vault_path
                    } else {
                        let mut config_path = get_config_path();
                        config_path.push("vaults");
                        config_path.push(vault_path);
                        config_path
                    }
                }
                None => {
                    let config = load_config().expect("Error loading config");
                    match config.last_opened {
                        Some(path) => path,
                        None => {
                            let mut config_path = get_config_path();
                            config_path.push("vaults");
                            let vaults = read_dir(config_path)
                                .expect("Error reading vaults directory")
                                .filter_map(|entry| entry.ok());
                            let vaults_vector: Vec<_> = vaults.collect();
                            let vault_count = vaults_vector.len();
                            if vault_count > 1 {
                                println!(" Theres more then one vault.\nWhich one do you mean?");
                                let mut index = 1;
                                for vault in &vaults_vector {
                                    println!("{index} - {}", vault.file_name().to_str().unwrap());
                                    index += 1
                                }
                                println!("Pick from (1-{index})");
                                let mut input = String::new();
                                std::io::stdin()
                                    .read_line(&mut input)
                                    .expect("Failed to read line");
                                let vault_index = input.trim().parse::<usize>().unwrap() - 1;
                                vaults_vector[vault_index].path()
                            } else if vault_count == 1 {
                                vaults_vector[0].path()
                            } else {
                                println!("There are no vaults. Create one with `vpassword init`");
                                std::process::exit(1);
                            }
                        }
                    }
                }
            };
            save_config(&AppConfig {
                last_opened: Some(path.clone()),
            })
            .expect("Error saving config");
            let master_password = rpassword::prompt_password("Your master password: ").unwrap();
            let response = send_request_to_agent(
                stream,
                Request::UnlockVault {
                    vault_path: path,
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
