# VPassword

**A secure, local-first password manager written in Rust.**

## Project Overview

This project began as a personal mission after my Steam password was compromised. I realized that while many browser-based solutions exist, I wanted to understand *exactly* how my secrets were being handled.

**VPassword** is the result: a password manager that prioritizes security and transparency. Unlike simple CLI tools, VPassword uses a **Client-Daemon architecture**. The sensitive cryptographic keys are held in a background agent process—never on disk and never passed as command-line arguments—ensuring your master password stays secure even if your shell history is logged.

## Key Features

* **Agent-Based Security:** A background daemon (`vpassword-agent`) holds decrypted keys in memory, allowing you to use the CLI without re-typing your password for every command.
* **Auto-Locking:** The agent monitors activity and automatically locks the vault (wipes keys from memory) after a period of inactivity (default: 5 minutes).
* **Strong Encryption:**
* **Argon2id** for key derivation (resisting GPU cracking attacks).
* **AES-256-GCM** for authenticated vault encryption.


* **Secure Memory Handling:** Uses the `zeroize` crate to ensure sensitive data is actively wiped from RAM when no longer needed.
* **Unix Socket Communication:** Secure, permission-locked IPC between the client and agent.

## Architecture

The project is structured as a Rust Workspace with three components:

1. **`vpassword-core`**: The shared library containing the cryptographic logic (`Argon2`, `AES-GCM`) and data models.
2. **`vpassword-agent`**: A background service that listens on a Unix socket (`/tmp/vault.sock`). It manages the vault state and enforces security timeouts.
3. **`vpassword-client`**: The `vpassword` CLI tool. It sends commands to the agent and handles user interaction. *It automatically spawns the agent if it isn't running.*

## Installation

### Prerequisites

* [Rust and Cargo](https://www.rust-lang.org/tools/install) (2021/2024 Edition)

### Build & Install

Since this is a workspace, you need to install the agent and the client binaries.

```sh
# Clone the repository
git clone https://github.com/mmdov/vpassword.git
cd vpassword

# Install the Agent
cargo install --path vpassword-agent

# Install the Client (binary name is 'vpassword')
cargo install --path vpassword-client

```

Ensure `~/.cargo/bin` is in your system `PATH`.

## Usage

You interact with the system entirely through the `vpassword` command. The Agent is managed automatically.

### 1. Initialize a Vault

Creates a new encrypted vault file. You will be prompted securely for a master password.

```sh
vpassword init ./my_vault.dat

```

### 2. Open the Vault

Unlocks the vault and starts the session.

```sh
vpassword open ./my_vault.dat
# Enter master password when prompted

```

### 3. Manage Passwords

Once the vault is open, you don't need to provide the path or password again until the session times out.

* **Add a new entry:**
```sh
vpassword add github johndoe
# You will be prompted for the password

```


* **Generate a strong password & add it:**
```sh
vpassword generate google my_email

```


* **List all entries:**
```sh
vpassword list

```


* **Show a specific password:**
```sh
vpassword show github

```



### 4. Lock the Vault

Manually closes the session and wipes keys from the agent's memory.

```sh
vpassword close

```

## Security Details

* **No CLI Arguments:** Passwords are never typed as command-line arguments, preventing leakage into shell history (`.bash_history`, etc.).
* **Memory Hygiene:** Secrets are wrapped in `Zeroizing<T>` types to prevent compiler optimizations from leaving copies in RAM.
* **Socket Permissions:** The IPC socket is created with `600` permissions (read/write only by the owner), preventing other users on the system from snooping on the connection.

## Roadmap

* [ ] **Clipboard Support:** Automatically copy passwords to clipboard on retrieval (with auto-clear).
* [ ] **Default Vault Path:** Support `~/.vpassword/` so paths don't need to be typed explicitly.
* [ ] **Git Sync:** Encrypted cloud backup via private GitHub repositories.
* [ ] **Import/Export:** Support for standard CSV formats.

## License

MIT

---
