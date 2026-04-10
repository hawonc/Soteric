# Soteric

Soteric is a small Rust CLI for protecting a narrow set of files from AI coding tools.

Today, the implemented pieces are:
- profile creation from explicit files and globs
- profile activation and deletion
- process scanning for known AI coding tools such as Codex and Claude

The intended next step is cryptographic encryption of the files associated with the relevant profile when one of those tools is detected. That encryption workflow is not implemented yet.

## Current Model

Soteric is intentionally profile-based rather than repo-wide. The idea is to blacklist only a few sensitive files instead of locking down an entire project.

Each profile stores:
- a profile name
- a root directory
- a small list of canonical file paths
- lightweight metadata about how the profile was created

The CLI also tracks one active profile. Right now, scanning and profile management are the working features. Automatic encryption and decryption are placeholders.

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

Define the secret for file encryption and decryption:

```bash
soteric secret *****
```

Current placeholders:

```bash
soteric encrypt-now
soteric decrypt-now
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

At the moment, scanning only reports detections. It does not yet trigger encryption or map a detected process to a stored profile automatically.

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

Run lints:

```bash
cargo clippy --all-targets --all-features
```

Format:

```bash
cargo fmt
```

The runtime profile store lives at `.soteric/profiles.json` in the repository root when Soteric is run inside a Git repository. It should be treated as local state rather than committed project data.

## Desktop GUI

Soteric includes a desktop application built with [Tauri](https://tauri.app/) + React + Tailwind CSS + shadcn/ui.

- **Dashboard** — Active profile, encryption status, AI tool detections, quick encrypt/decrypt actions.
- **Profiles** — Create, activate, deactivate, and delete profiles. Add files or globs.
- **Live Monitor** — Real-time process scanning with background monitoring. Auto-encrypts when mapped AI tools are detected.
- **Activity Log** — Timestamped history of all actions and events.
- **Settings** — Encryption key management, Touch ID setup, process-to-profile mappings.

```bash
cd desktop
npm install
npm run tauri dev
```

See [`desktop/README.md`](desktop/README.md) for details.
