use std::time::Instant;

use windows::Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, SetWindowsHookExW, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

const VK_A: u32 = 0x41;
const VK_MENU: u32 = 0x12;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;

/// A hold-to-preview cycle session: which window is currently highlighted, and when the hold started
/// (used by the overlay to debounce a quick tap-and-release so it doesn't flash for nothing).
#[derive(Clone, Copy, Debug)]
pub struct PreviewSession {
    pub preview_handle: Option<isize>,
    pub started_at: Instant,
}

/// Installs the WH_KEYBOARD_LL hook. Must be called from the same thread that pumps messages via
/// GetMessageW (matches the existing win-event hook's requirement in `hook.rs`) -- the hook callback is
/// only actually invoked while that thread keeps its message loop running.
pub fn start_keyboard_hook() {
    unsafe {
        let _ = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), HINSTANCE::default(), 0);
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let msg = wparam.0 as u32;
        let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        let alt_down = (kbd.flags.0 & LLKHF_ALTDOWN.0) != 0;

        // Alt+A (including OS auto-repeat while held): advance the preview and swallow the key --
        // this is the same "fully consumed" behavior Alt+A already had via RegisterHotKey.
        if is_keydown && kbd.vkCode == VK_A && alt_down {
            // Deferred to a spawned thread rather than run inline: WH_KEYBOARD_LL procedures must
            // return quickly, and doing AttachThreadInput/SetForegroundWindow-adjacent work (in
            // commit_preview, reached the same way, via utils::focus_window) synchronously here was
            // observed to delay this event's delivery system-wide long enough to race with and
            // sometimes undo the very foreground/restore change it had just made.
            std::thread::spawn(advance_preview);
            return LRESULT(1);
        }

        // Alt released while a preview session is active: commit it. Always passed through
        // (never swallowed) so Alt's normal modifier behavior elsewhere is never disturbed.
        if is_keyup && matches!(kbd.vkCode, VK_MENU | VK_LMENU | VK_RMENU) {
            std::thread::spawn(commit_preview);
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// Advances the preview by one step: seeds from `WindowManager::peek_next()` (honoring the same
/// foreground-resync semantics as an instant tap) on the first 'A', then walks the list locally via
/// `peek_after` on every repeat -- the real foreground never changes mid-hold, so re-checking it each
/// tap would be pointless and, in the common case of the real foreground being untracked, would wrongly
/// never advance at all (see `WindowManager::cycle`'s "untracked foreground" branch).
fn advance_preview() {
    let Some(mgr) = crate::GLOBAL_MANAGER.get() else {
        return;
    };
    let Some(session_lock) = crate::PREVIEW_SESSION.get() else {
        return;
    };

    let manager = mgr.lock().unwrap();
    let mut session_guard = session_lock.lock().unwrap();

    let next = match &*session_guard {
        Some(s) => manager.peek_after(s.preview_handle, 1),
        None => manager.peek_next(),
    };
    let Some(next) = next else {
        return; // nothing tracked at all -- nothing to preview
    };

    let started_at = session_guard.as_ref().map_or_else(Instant::now, |s| s.started_at);
    *session_guard = Some(PreviewSession {
        preview_handle: Some(next),
        started_at,
    });
    drop(session_guard);
    drop(manager);

    if let Some(ctx) = crate::EGUI_CTX.get() {
        ctx.request_repaint();
    }
}

/// Commits whatever is currently previewed (if a session is active) by focusing it, writes it into
/// `WindowManager::current`, and clears the session. A no-op if no session was active (e.g. a bare
/// Alt press/release elsewhere with no A in between).
fn commit_preview() {
    let Some(session_lock) = crate::PREVIEW_SESSION.get() else {
        return;
    };
    let session = session_lock.lock().unwrap().take();
    let Some(session) = session else {
        return;
    };

    if let Some(handle) = session.preview_handle {
        crate::utils::focus_window(windows::Win32::Foundation::HWND(handle));
        if let Some(mgr) = crate::GLOBAL_MANAGER.get() {
            mgr.lock().unwrap().set_current(Some(handle));
        }
    }
    if let Some(ctx) = crate::EGUI_CTX.get() {
        ctx.request_repaint();
    }
}
