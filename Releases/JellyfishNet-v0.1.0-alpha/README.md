# JellyfishNet (Work in progress)
**JellyfishNet** is a lightweight window manager inspired by ThePrimeagen’s Harpoon plugin for Vim. Just like Harpoon lets you hook into files and jump between them instantly, JellyfishNet brings that same philosophy to your desktop: mark your most important windows and cycle between them with precision and speed.
___
## ✨ Features
- Mark and jump between frequently used windows instantly
- Minimalist design with Rust performance
- Inspired by Harpoon’s workflow philosophy
- Lightweight and fast

___
## 📦 Installation
(wait for the...)
Releases (that are) coming soons... 
---
(or you can build it yourself:)
```bash
# Clone the repository
git clone https://github.com/Eiks-exe/jellyfish-net.git
cd jellyfish-net

# Build 
cargo build --release

# Run
./target/release/jellyfish-net
```
### 🔑 Default Hotkeys

By default, JellyfishNet registers the following hotkeys:

- **Alt + ~ (QWERTY) / Alt + ² (AZERTY)** → Action #1 (mark window)
- **Alt + A** → Action #2 (cycle between windows)

> Note: The second hotkey depends on your keyboard layout.  
> - On US QWERTY keyboards, key code `222` maps to `~` / `'`.  
> - On French AZERTY keyboards, the same code maps to `²`.  
> Future versions will allow user‑configurable hotkeys.


## 🚧 Status
JellyfishNet is currently in **alpha**.  
Features are limited, hotkeys are hardcoded, and only Windows is supported.  
Feedback and contributions are welcomeu!

## ⚠️ Known Limitations
- Hotkeys are not yet user‑configurable
- Tray icon is basic (custom icon coming soon)
- Only tested on Windows
- Keyboard layout differences (QWERTY vs AZERTY) may affect hotkey behavior

## 📍 Roadmap
- [ ] Add tray icon with custom PNG
- [ ] Cross‑layout hotkey support
- [ ] Config file for user‑defined hotkeys
- [ ] Linux/macOS support
