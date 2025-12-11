# NOT WORKING AT THE MOMENT AT ALL IM REFACTORING THE WHOLE THING

# VPassword CLI

A simple password manager command-line application written in Rust.

## Project Overview

This project began as a personal passion after my Steam password was compromised. Through research, I discovered that browser-based password generators are not as secure as they seem. Wanting a safer solution and an opportunity to improve my Rust skills, I set out to build a secure, local-only password manager. The goal is to generate and store passwords securely on disk, with strong encryption and no reliance on cloud services. I’ve enjoyed learning about security best practices and what truly makes a password manager trustworthy.

## Features

- Create and remove password vaults
- List existing vaults
- Add, remove, and generate password entries
- Secure vault access with a master password
- Vault data is always encrypted at rest
- Password generation functionality

## Security Notice

All vault data is encrypted before being saved to disk and only decrypted in memory during use. This ensures your passwords are protected even if the storage files are accessed directly. The master password is required to access and manage vault contents.

## Installation

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)

### Steps

1. Clone the repository:
   ```sh
   git clone https://github.com/mmdov/vpassword.git
   cd vpassword
   ```
2. Build the project:
   ```sh
   cargo build --release
   ```

The executable will be located at `target/release/vpassword`.

## Usage

Run the CLI with the following commands:

### Vault Commands

- **Create a vault**
  ```sh
  vpassword vault create <vault_name> <master_pass>
  ```

- **Remove a vault**
  ```sh
  vpassword vault remove <vault_name> <master_pass>
  ```

- **List vaults**
  ```sh
  vpassword vault list <vault_name> <master_pass>
  ```

### Password Commands

- **Add a password entry**
  ```sh
  vpassword password add <vault_name> <master_pass> <name> <username> <password>
  ```
  - Use `"generate"` as `<password>` to auto-generate a password.

- **Remove a password entry**
  ```sh
  vpassword password remove <vault_name> <master_pass> <name>
  ```

- **Generate a password**
  ```sh
  vpassword password generate
  ```

## Example

```sh
vpassword vault create myvault mymasterpassword
vpassword password add myvault mymasterpassword github johndoe generate
vpassword password list myvault mymasterpassword
```

## Dependencies

- [clap](https://crates.io/crates/clap) - Command-line argument parsing
- [aes-gcm](https://crates.io/crates/aes-gcm) - AES-GCM encryption
- [argon2](https://crates.io/crates/argon2) - Password hashing
- [serde](https://crates.io/crates/serde) - Serialization/deserialization
- [base64](https://crates.io/crates/base64) - Base64 encoding/decoding
- [rand](https://crates.io/crates/rand) - Random number generation

## Contributing

Contributions are welcome! Please open issues or submit pull requests on GitHub.

## Contact/Support

For questions or issues, please use the [GitHub Issues](https://github.com/mmdov/vpassword/issues) page.

## License

MIT

## TODO

- Password agent to keep master password saved for some time
- Safer/better way to ask for master password (e.g. not via CLI args)
- Add password strength meter
- Support for password categories/tags
- Import/export vaults
- Interactive TUI mode

---
