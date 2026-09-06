use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE,
    },
};

/// Restores `hwnd` if minimized, then forces it to the foreground using the AttachThreadInput dance --
/// needed because a bare SetForegroundWindow is routinely refused by Windows' foreground-lock heuristic
/// when the calling thread isn't already the foreground thread (exactly our situation: we're reacting to
/// a global hotkey/hook, not to input the target's own thread just received). Attaches to *both* the
/// current foreground window's thread (the one Windows actually grants the foreground-change permission
/// to -- the more load-bearing of the two) and the target's own thread, since either one alone was found
/// insufficient in testing. Returns whether the OS actually granted the foreground switch.
pub fn focus_window(hwnd: HWND) -> bool {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        let current_tid = GetCurrentThreadId();

        let foreground = GetForegroundWindow();
        let foreground_tid = if foreground.0 != 0 {
            GetWindowThreadProcessId(foreground, None)
        } else {
            0
        };
        let attached_fg = foreground_tid != 0
            && foreground_tid != current_tid
            && AttachThreadInput(current_tid, foreground_tid, true).as_bool();

        let target_tid = GetWindowThreadProcessId(hwnd, None);
        let attached_target = target_tid != current_tid
            && target_tid != foreground_tid
            && AttachThreadInput(current_tid, target_tid, true).as_bool();

        let _ = BringWindowToTop(hwnd);
        let ok = SetForegroundWindow(hwnd).as_bool();

        if attached_target {
            let _ = AttachThreadInput(current_tid, target_tid, false);
        }
        if attached_fg {
            let _ = AttachThreadInput(current_tid, foreground_tid, false);
        }
        if !ok {
            eprintln!("[!] focus_window: SetForegroundWindow refused for {:?}", hwnd);
        }
        ok
    }
}

pub fn get_window_title(hwnd: HWND) -> String {
    unsafe {
            let len = GetWindowTextLengthW(hwnd);
                if len <= 0 {
                    return String::new();
                }      

                let mut buf = vec![0u16; (len + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut buf);
                if copied <= 0 {
                    return String::new();
                }

                String::from_utf16_lossy(&buf[..copied as usize])
    }
}
