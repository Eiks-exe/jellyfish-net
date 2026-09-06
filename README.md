# JellyfishNet (Work in progress)

**JellyfishNet** is a lightweight, Windows-only window manager inspired by ThePrimeagen's [Harpoon](https://github.com/ThePrimeagen/harpoon) plugin for Vim. Just like Harpoon lets you hook into files and jump between them instantly, JellyfishNet brings that same philosophy to your desktop: mark ("catch") your most important windows and cycle between them with a global hotkey instead of alt-tabbing through everything.

It runs quietly in the system tray, with a small [egui](https://github.com/emilk/egui)-based GUI for previewing and managing the windows you've caught.

## Features

- **Catch a window** — `Alt+B` toggles the foreground window in and out of your tracked list.
- **Hold-to-preview cycling** — hold `Alt` and tap `A` to step through your caught windows one at a time; a small overlay shows the list with the current preview highlighted. Release `Alt` to commit and focus that window.
- **Management window** — opened from the tray icon, lists every caught window (with icon and title) and lets you jump to, reorder (move up/down), or remove any of them.
- **System tray icon** — right-click for `Open` (management window) and `Quit`.
- Automatically forgets a window once it's closed, so the list never accumulates dead handles.

## Requirements

- Windows (uses Win32 APIs directly — this will not build or run on other platforms).
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain).

## Building & running

```sh
# Build
cargo build

# Run (must be run on Windows)
cargo run

# Release build
cargo build --release

# Type-check without building
cargo check
```

There are no automated tests in this repo currently.

## Usage

1. Run the app — it starts minimized to the system tray.
2. Focus a window you want to track and press `Alt+B` to catch it. Press `Alt+B` again on the same window to release it.
3. Hold `Alt` and tap `A` to preview-cycle through your caught windows; keep tapping `A` while holding `Alt` to advance further. Release `Alt` to jump to whichever window is highlighted.
4. Right-click the tray icon and choose `Open` to bring up the management window, where you can jump to, reorder, or remove caught windows directly.
5. Choose `Quit` from the tray icon to exit.

## Status

This project is a work in progress — expect rough edges and missing features.
