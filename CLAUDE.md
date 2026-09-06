# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

JellyfishNet is a Windows-only, Rust-based lightweight window manager inspired by ThePrimeagen's Harpoon plugin for Vim: mark ("catch") windows and cycle between them with a global hotkey instead of alt-tabbing through everything. Has a small egui-based GUI (a cycle overlay and a management window) alongside the tray icon. Work in progress.

## Commands

- Build: `cargo build`
- Run (must run on Windows — uses Win32 APIs): `cargo run`
- Release build: `cargo build --release`
- Check without building: `cargo check`
- There are no tests in this repo currently.

## Architecture

Two threads: a dedicated Win32 input thread (hotkey/hook registration, unchanged in spirit from before the GUI existed) and the main thread, which `eframe::run_native` owns for the lifetime of the process. They share state through a handful of process-wide statics in `main.rs` (`GLOBAL_MANAGER`, `EGUI_CTX`, `MGMT_VISIBLE`, `PREVIEW_SESSION`) rather than message-passing.

- **`main.rs`** — entry point. Sets up the tray icon (menu: "Open" id 2, "Quit" id 1), initializes `GLOBAL_MANAGER: OnceCell<Arc<Mutex<WindowManager>>>`, then spawns the input thread (`start_hook()`, `keyboard_hook::start_keyboard_hook()`, registers Alt+B as a `RegisterHotKey` action, and blocks in `listener.listen()`) before calling `eframe::run_native` on the main thread with `gui::JellyfishApp`. `jfn_catch` (the Alt+B handler) skips the foreground window when its owning pid is `std::process::id()`, so the app never catches its own GUI windows.
- **`manager.rs`** (`WindowManager`) — the core data structure: a doubly-linked list of tracked windows implemented over `HashMap<isize, Jfnindow>` (handle → window info) plus a parallel `HashMap<isize, Link>` (handle → prev/next handle) with separate `head`/`tail`/`current` pointers. Handles (`HWND.0`, an `isize`) are used as map keys instead of storing pointers/references directly, which sidesteps Rust borrow-checker issues with the linked list. All fields are individually `Arc<Mutex<..>>`-wrapped and locked independently — when touching more than one field, lock ordering matters (`w_info` → `links` → `head` → `tail` → `current`); the existing methods are the pattern to follow. `add()` appends to the tail (or removes-then-no-ops if the window is already tracked, i.e. "catch" toggles). `cycle()` (mutating, foreground-aware) is the original instant-cycle logic, now unused directly but kept as the reference `peek_next()` mirrors; `peek_next()`/`peek_after()` are its read-only/pure counterparts used to drive the hold-to-preview overlay without mutating `current` on every keystroke, `get_current()`/`set_current()` read/write it directly, `snapshot_ordered()` returns an ordered `Vec` for rendering, and `move_up()`/`move_down()` reorder the list (`move_down` just delegates to `move_up` on the successor — same pointer swap, other node's perspective).
- **`hook.rs`** — installs a `SetWinEventHook` for `EVENT_OBJECT_DESTROY` so windows are automatically removed from the manager when they're closed, keeping the linked list from accumulating dead handles.
- **`hotkeys.rs`** — `HotKeyListener` trait + `WindowsHotKeyListener` impl wrapping `RegisterHotKey`/`GetMessageW`/`WM_HOTKEY`. Actions are registered by integer id via `add_action`, decoupling hotkey id from behavior; `listen()` runs the Win32 message loop and also drains tray `MenuEvent`s (id "1" Quit, id "2" Open) on each pass. Only Alt+B is registered here now — see `keyboard_hook.rs`.
- **`keyboard_hook.rs`** — a `WH_KEYBOARD_LL` hook implementing Alt+A's hold-to-preview: holding Alt and tapping A advances a preview through the list (`PreviewSession`, in `PREVIEW_SESSION`) without touching anything else; releasing Alt commits it (`focus_window` + `set_current`). The actual work is deferred via `std::thread::spawn` rather than run inline in the hook callback — low-level hook procedures must return quickly, and the `AttachThreadInput`/`SetForegroundWindow`-adjacent work in `focus_window` was observed to race with itself when run synchronously there.
- **`gui.rs`** (`JellyfishApp`) — the `eframe::App`. Its root viewport (configured in `main.rs`'s `NativeOptions`) is a 1x1, off-screen, non-activatable window that's never shown to the user but is deliberately kept genuinely *visible* (not `with_visible(false)`) and never itself hidden/minimized. This isn't cosmetic: painting an actually-invisible/minimized window on Windows goes through a different eframe code path (a workaround for a separate bug, https://github.com/emilk/egui/issues/5229) that doesn't set up the context `show_viewport_immediate` needs, and calling it from there panics ("egui backend is implemented incorrectly - the user callback was never called"). Both real UI surfaces — the management window (list + Jump/Up/Down/Remove per row) and the cycle overlay (icon+title rows, current preview highlighted) — are child immediate viewports created only on the frames they should exist; closing/minimizing the management window just stops it being called (`MGMT_VISIBLE = false`) rather than routing through `ViewportCommand::Visible`/`Minimized`, which have open Windows-specific bugs. The overlay has a 150ms debounce (a quick tap-and-release shouldn't flash it) implemented via `request_repaint_after`, not a one-shot `request_repaint()` — the latter fires immediately, before the debounce has elapsed, and nothing would ever look again once it passes.
- **`icon.rs`** — `get_window_icon_rgba`: `WM_GETICON` then the class icon, converted to RGBA via `GetIconInfo`/`GetDIBits` for use as an egui texture. Icon handles from both sources are borrowed (never destroyed); the two bitmaps `GetIconInfo` returns are the caller's to clean up.
- **`window.rs`** — `Jfnindow`, the plain struct tracked per window (title, `HWND`, pid, tid).
- **`utils.rs`** — small Win32 helpers: `get_window_title`, and `focus_window` (restore-if-minimized + `AttachThreadInput` dance, attaching to *both* the current foreground thread and the target's own thread — needed because a bare `SetForegroundWindow` is routinely refused by Windows' foreground-lock heuristic for a background/hotkey-driven caller).

### Key conventions

- Window identity throughout the manager is the raw `HWND.0` (`isize`), not `HWND` itself, since `HWND` isn't hashable/`Send`-friendly for this use.
- Unsafe Win32 calls interacting with *other* processes'/the system's windows are localized at call sites (no wrapper/abstraction layer over `windows`-rs); follow that pattern rather than introducing a Win32 abstraction. This doesn't extend to the app's own UI, where egui/eframe is used idiomatically.
- `excluded_classes` in `hook.rs` filters out shell/desktop windows from event handling — extend this list if new false-positive window classes turn up.
- Renderer is `eframe::Renderer::Glow` (not the default wgpu) purely to keep the dependency tree and build times down — no functional reason, safe to revisit.
