# Soteric Desktop

The desktop GUI for Soteric, built with [Tauri](https://tauri.app/) + React + Tailwind CSS + shadcn/ui.

## Overview

Soteric Desktop provides a visual interface for managing file protection profiles, monitoring AI coding tools, and controlling encryption — all backed by the same Rust core library as the CLI.

### Pages

- **Dashboard** — At-a-glance view of active profile, encryption status, AI tool detections, and quick encrypt/decrypt actions.
- **Profiles** — Create, view, activate, deactivate, and delete protection profiles. Add files or glob patterns to existing profiles.
- **Live Monitor** — Real-time process scanning with background monitoring support. Auto-encrypts files when mapped AI tools are detected.
- **Activity Log** — Timestamped record of all actions: scans, activations, encryption events, and errors.
- **Settings** — Manage encryption keys, configure Touch ID (macOS), and set up process-to-profile mappings for automatic protection.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.85+)
- [Node.js](https://nodejs.org/) (20+)
- macOS, Windows, or Linux

## Getting Started

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://tauri.app/) |
| Frontend | React 19, TypeScript |
| Styling | Tailwind CSS v4, [shadcn/ui](https://ui.shadcn.com/) |
| Backend | Rust (shared `soteric` core library) |
| Encryption | AES-256-GCM + Argon2id |
| Biometrics | macOS Keychain + Touch ID |

## Project Structure

```
desktop/
├── src/                    # React frontend
│   ├── pages/              # Page components
│   │   ├── Dashboard.tsx   # Main dashboard
│   │   ├── Profiles.tsx    # Profile management
│   │   ├── Monitor.tsx     # Live AI tool monitor
│   │   ├── ActivityLog.tsx # Event history
│   │   └── Settings.tsx    # Keys, biometrics, mappings
│   ├── components/ui/      # shadcn/ui components
│   ├── App.tsx             # Layout + routing
│   ├── store.ts            # State management + Tauri IPC
│   └── types.ts            # TypeScript interfaces
├── src-tauri/              # Tauri Rust backend
│   ├── src/lib.rs          # Tauri commands (wraps soteric library)
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── package.json
└── vite.config.ts
```

## Tauri Commands

The backend exposes these IPC commands to the frontend:

| Command | Description |
|---------|-------------|
| `list_profiles` | List all profiles with encryption status |
| `create_profile` | Create a new profile |
| `append_profile` | Add files/globs to a profile |
| `delete_profile` | Delete a profile |
| `activate_profile` | Activate + encrypt a profile |
| `deactivate_profile` | Deactivate + decrypt a profile |
| `encrypt_now` | Manually encrypt active profile files |
| `decrypt_now` | Manually decrypt active profile files |
| `scan_processes` | Scan for AI coding tools |
| `set_secret` | Change encryption key |
| `setup_biometric` | Enable Touch ID (macOS) |
| `remove_biometric` | Disable Touch ID |
| `check_biometric` | Check if biometric is configured |
| `list_mappings` | List process-to-profile mappings |
| `set_mapping` | Add a process-to-profile mapping |
| `delete_mapping` | Remove a mapping (auto-decrypts if active) |
| `start_monitor` | Start background monitoring |
| `stop_monitor` | Stop background monitoring |
| `is_monitor_running` | Check monitor status |

## License

[MIT](../LICENSE)
