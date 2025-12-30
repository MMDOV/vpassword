mod cli;
mod handlers;

// NOTE: take a master password and create a vault with that password
// that vault is a file inside that file theres our salt, nonce and the ciphered text
// all the new passwords are being added to that vault
// TODO: rethink the cli to make it more practical
// FIX: add error handling everyting is panicking right now
#[tokio::main]
async fn main() {
    let commands = cli::parse_cli();
    handlers::handle_command(commands).await;
}
