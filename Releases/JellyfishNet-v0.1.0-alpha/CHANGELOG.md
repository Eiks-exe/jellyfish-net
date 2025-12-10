
# Changelog
All notable changes to **JellyfishNet** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [Unreleased]
### Added
- Planned support for cross‑keyboard layout hotkeys
- Planned configuration file for user‑defined hotkeys
- Planned Linux/macOS support
- Planned custom tray icon with bundled `.ico`

---

## [0.1.0-alpha] - 2025-12-10
### Added
- Initial alpha release of JellyfishNet
- Ability to mark and jump between frequently used windows
- Default hotkeys:
  - **Alt + ~ (QWERTY) / Alt + ² (AZERTY)** → mark window
  - **Alt + A** → cycle between windows
- Minimal tray icon integration (default system icon)
- Lightweight Rust implementation inspired by Harpoon plugin

### Known Limitations
- Hotkeys are hardcoded and vary by keyboard layout
- Tray icon uses default system icon (no custom `.ico` yet)
- Only Windows is supported
