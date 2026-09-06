mod hook;
mod manager;
mod window;
mod hotkeys;
mod utils;
mod icon;
mod keyboard_hook;
mod gui;
//use hook::start_hook;

use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;
use tray_icon::{TrayIconBuilder, Icon, menu::{Menu, MenuItemBuilder}};
use windows::Win32::{Foundation::HINSTANCE, UI::{Input::KeyboardAndMouse::MOD_ALT, WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, IDI_APPLICATION, LoadIconW}}};

use crate::{hook::start_hook, hotkeys::HotKeyListener, manager::WindowManager, utils::get_window_title, window::Jfnindow};


pub static GLOBAL_MANAGER: OnceCell<Arc<Mutex<WindowManager>>> = OnceCell::new();
/// Cloned from the eframe CreationContext at startup; lets the Win32 input thread wake the GUI thread
/// (via request_repaint()) after mutating shared state such as PREVIEW_SESSION.
pub static EGUI_CTX: OnceCell<eframe::egui::Context> = OnceCell::new();
/// Whether the management window (a child viewport, see gui.rs) is supposed to exist right now.
pub static MGMT_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// The active hold-to-preview session, if any. Written by keyboard_hook.rs (on the Win32 input thread),
/// read by gui.rs (on the eframe thread) to drive the overlay.
pub static PREVIEW_SESSION: OnceCell<Arc<Mutex<Option<keyboard_hook::PreviewSession>>>> = OnceCell::new();

fn jfn_catch(manager: &WindowManager) {
    let focused_window_handle = unsafe {GetForegroundWindow()};
    let mut pid = 0u32;
    let tid = unsafe {
        GetWindowThreadProcessId(focused_window_handle, Some(&mut pid))
    };
    if pid == std::process::id() {
        // Never catch this process's own GUI windows (management window / overlay).
        return;
    }
    let focused_window_title= get_window_title(focused_window_handle);

    let new_window = Jfnindow {
        handle: focused_window_handle,
        title: focused_window_title,
        _pid: pid,
        _tid: tid,
    };
    manager.add(new_window);
    manager.enum_windows();
}

fn init_manager() {
    let manager = Arc::new(Mutex::new(WindowManager::new()));
    GLOBAL_MANAGER.set(manager.clone()).expect("could not set manager");
}

/// Shows the management window. Safe to call from any thread (the tray menu handler calls this from the
/// Win32 input thread) -- it just flips a flag gui.rs checks each frame and wakes the eframe thread.
pub fn show_management_window() {
    MGMT_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.request_repaint();
    }
}

fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Warn).init();
    println!("start");
    let hicon = unsafe {
        LoadIconW(HINSTANCE::default(), IDI_APPLICATION).unwrap()
    };
    let icon = Icon::from_handle(hicon.0);

    let open_item = MenuItemBuilder::new()
        .id(2.into())
        .text("Open")
        .enabled(true)
        .build();
    let quit_item = MenuItemBuilder::new()
        .id(1.into())
        .text("Quit")
        .enabled(true)
        .build();
    let menu = Menu::new();
    let _append_open_item = menu.append(&open_item);
    let _append_quit_item = menu.append(&quit_item);
    let _systray_menu = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("JellyfishNet is running...")
        .with_icon(icon)
        .build()
        .unwrap();


    init_manager();
    PREVIEW_SESSION.set(Arc::new(Mutex::new(None))).expect("could not set preview session");

    // Dedicated Win32 message-pump thread: everything that needs a continuously-pumped GetMessageW loop
    // on the thread that registered it (the destroy hook, the low-level keyboard hook, and the Alt+B
    // "catch" hotkey) stays together here -- unchanged in spirit from before this feature, just moved
    // off the main thread so eframe can own that one instead.
    std::thread::spawn(|| {
        start_hook();
        keyboard_hook::start_keyboard_hook();

        let listener = hotkeys::WindowsHotKeyListener{
            actions: Arc::new(Mutex::new(Vec::new()))
        } ;
        listener.add_action(1, {
            Box::new(
                move || {
                    if let Some(mgr) = GLOBAL_MANAGER.get() {
                        let manager = mgr.lock().unwrap();
                        jfn_catch(&manager);
                    }
                }
            )
        }).expect("could not add action");

        // Alt+A is no longer a RegisterHotKey action -- keyboard_hook.rs's low-level hook now owns the
        // whole Alt+A gesture (hold-to-preview, committed on Alt-up) so the same physical keystroke is
        // never handled by two mechanisms at once.
        listener.register(1, MOD_ALT.0, 66).expect("could not register the hotkey...");
        println!("listening for hotkeys");
        listener.listen();
    });

    let native_options = eframe::NativeOptions {
        // The root viewport itself is never shown to the user: 1x1, off-screen, undecorated. It hosts
        // gui.rs's App::ui(), which creates the real management window and overlay as child viewports.
        // Deliberately kept genuinely visible (not with_visible(false)) -- see the doc comment on
        // gui::JellyfishApp for why an invisible root breaks show_viewport_immediate on Windows.
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("JellyfishNet")
            .with_inner_size([1.0, 1.0])
            .with_position([-32000.0, -32000.0])
            .with_decorations(false)
            .with_taskbar(false)
            .with_resizable(false)
            // Never activatable: without this, once any child viewport (overlay or management
            // window) has been created and later destroyed, eframe/winit reliably hands activation
            // back to its parent -- the root -- which races with and can undo focus_window's own
            // SetForegroundWindow when a hold-to-preview commit happens around the same time.
            .with_active(false),
        // glow (OpenGL) rather than the default wgpu: no functional need, just a much smaller dependency
        // tree and faster builds, in keeping with this app being meant to stay lightweight.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "JellyfishNet",
        native_options,
        Box::new(|cc| {
            let _ = EGUI_CTX.set(cc.egui_ctx.clone());
            Ok(Box::new(gui::JellyfishApp::default()))
        }),
    ).expect("eframe failed to run");
}
