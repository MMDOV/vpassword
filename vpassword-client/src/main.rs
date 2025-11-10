mod cli;
mod handlers;
use std::{io, process::Command};
use tokio::net::UnixStream;

// NOTE: take a master password and create a vault with that password
// that vault is a file inside that file theres our salt, nonce and the ciphered text
// all the new passwords are being added to that vault
// TODO: rethink the cli to make it more practical
// FIX: add error handling everyting is panicking right now
#[tokio::main]
async fn main() {
    Command::new("vpassword-agent").status().unwrap();
    let stream = UnixStream::connect("/tmp/vault.sock").await.unwrap();
    loop {
        stream.writable().await.unwrap();

        match stream.try_write(b"hello world") {
            Ok(_) => {
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => {
                println!("err");
            }
        }
    }
    let commands = cli::parse_cli();
    handlers::handle_command(commands);
}
