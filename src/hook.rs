use crate::get_window_title;
use crate::GLOBAL_MANAGER;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{EVENT_OBJECT_DESTROY, WINEVENT_OUTOFCONTEXT},
    },
};

pub extern "system" fn win_event_callback(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    let title = get_window_title(hwnd);
    let excluded_classes = ["Shell_TrayWnd", "Progman", "ActiveMovie Window"];
    if excluded_classes.contains(&title.as_str()) {
        return;
    }

    if title.is_empty() {
        return;
    }

    if event == EVENT_OBJECT_DESTROY {
        let manager = GLOBAL_MANAGER.get().unwrap();
        let guard = manager.lock().unwrap();
        let window_exist = guard.w_info.lock().unwrap().contains_key(&hwnd.0);
        if window_exist {
            guard.remove(hwnd.0);
        }
    }
}

pub fn start_hook() {
    unsafe {
        SetWinEventHook(
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_DESTROY,
            None,
            Some(win_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
    };
}
