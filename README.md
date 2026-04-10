# Soteric

Soteric is a Rust CLI tool that protects sensitive files from AI coding assistants (like GitHub Copilot or Claude) by automatically encrypting them when these tools are detected running on your system.

## Current Model

Soteric is intentionally profile-based rather than repo-wide. The idea is to blacklist only a few sensitive files instead of locking down an entire project.

Each profile stores:
- a profile name
- a root directory
- a small list of canonical file paths
- lightweight metadata about how the profile was created

The CLI also tracks one active profile. Right now, scanning and profile management are the working features. Automatic encryption and decryption are placeholders.

Current Implementation:
- Automatic encryption/decryption of protected files when profiles are activated or deactivated
- Mapping specific processes (like AI coding tools) to profiles for automatic activation
- Background monitoring to detect and respond to running AI tools

## Encryption

The encryption module handles secure file encryption and decryption using industry-standard cryptography:

- **Encryption**: Files are encrypted with AES-256-GCM (Authenticated Encryption with Associated Data) for confidentiality and integrity. A random salt and nonce are generated for each file to ensure unique encryption.
- **Key Derivation**: User-provided keys are strengthened using Argon2 (a memory-hard function) to resist brute-force attacks.
- **Decryption**: Reverses the process, verifying data integrity during decryption. Invalid keys or corrupted files are rejected.

This ensures protected files remain unreadable to AI tools while maintaining strong security practices.

## Biometric Authentication (macOS)

On macOS, Soteric supports Touch ID authentication to secure your encryption key:

- **Secure Storage**: The encryption secret is stored in the system Keychain, a secure OS-level credential store.
- **Biometric Unlock**: When you run Soteric, it first attempts to retrieve the secret from Keychain using Touch ID. If biometric auth isn't set up, it falls back to reading from `secret.txt`.
- **Setup and Management**: Use `setup-biometric` to enable Touch ID protection and `remove-biometric` to disable it.

This eliminates the need to store your encryption key in plaintext while providing convenient, biometric-secured access to your sensitive files.

## Commands

Create a profile from explicit files:

```bash
soteric add-profile secrets \
  --file ./secret.txt \
  --file ./temp/codex.txt
```

Create a profile from globs:

```bash
soteric add-profile hidden-files --glob './.*'
```

Append additional files or globs to an existing profile:

```bash
soteric append-profile hidden-files --file /tmp/codex.txt
soteric append-profile hidden-files --glob 'temp/*.txt'
```

Create and activate a profile in one step:

```bash
soteric add-profile hidden-files --glob './.*' --activate
```

List configured profiles:

```bash
soteric list-profiles
```

Show one profile:

```bash
soteric show-profile hidden-files
```

Activate the profile you want to use:

```bash
soteric activate hidden-files
```

Deactivate a specific profile:

```bash
soteric deactivate hidden-files
```

Delete a profile:

```bash
soteric delete-profile hidden-files --yes
```

Scan running processes for supported AI coding tools:

```bash
soteric scan
```

Show the active profile and current detections together:

```bash
soteric status
```

Set the secret for file encryption and decryption:

```bash
soteric set-secret my-secret
```

Define a mapping from a process to a profile:

```bash
soteric set-mapping --process cursor --profile hidden-files
```

Delete a process-to-profile mapping:

```bash
soteric delete-mapping cursor
```

List all process-to-profile mappings:

```bash
soteric list-mappings
```

Set up biometric (Touch ID) authentication for the encryption key (macOS only):

```bash
soteric setup-biometric
```

Remove biometric authentication (macOS only):

```bash
soteric remove-biometric
```

Start the background process that monitors for AI coding tools and activates profiles accordingly:

```bash
soteric run
```

## Scan Behavior

`scan` inspects running processes and reports matches for known AI coding-tool binaries. The current matcher includes names such as:

- `codex`
- `claude`
- `claude-code`
- `opencode`
- `openhands`
- `cursor`
- `copilot`
- `windsurf`
- `antigravity`

## Profile Notes

- `--file` can be passed multiple times.
- `--glob` can be passed multiple times.
- In a Git repository, relative `--file` and `--glob` inputs are resolved from the repo root.
- Outside a Git repository, relative paths are resolved from the current working directory.
- Only files are included in a profile. Directory matches are ignored.
- Paths are canonicalized before they are stored.
- If all files in a profile share the same parent directory, that directory becomes the profile root. Otherwise, the workspace root is used.

## Development

Build:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

See [TESTING.md](TESTING.md) for detailed testing documentation and how to write new tests.

Run lints:

```bash
cargo clippy --all-targets --all-features
```

Format:

```bash
cargo fmt
```

The runtime profile store lives at `.soteric/profiles.json` in the repository root when Soteric is run inside a Git repository. It should be treated as local state rather than committed project data.
