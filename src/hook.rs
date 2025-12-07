
use windows::Win32::{
    Foundation::{BOOL, HWND, RECT},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook},
        WindowsAndMessaging::{
            EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, GA_ROOT, GetWindowTextLengthW, GetWindowTextW, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, GetAncestor, GetWindowRect,IsWindow
        },
    },
};
extern "system" fn win_event_callback(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
){
    unsafe {
        fn get_window_title(hwnd: HWND) -> String {
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

        if _id_object != OBJID_WINDOW.0 {
            return;
        }

        if GetAncestor(hwnd, GA_ROOT) != hwnd {
            return;
        }

        if IsWindow(hwnd) == BOOL(0) {
            return;
        }

        let mut rect = windows::Win32::Foundation::RECT::default();
        let result = {GetWindowRect(hwnd, &mut rect as *mut RECT)};
        match result {
            Ok(()) => {
                if rect.left == rect.right || rect.top == rect.bottom {
                    return ;
                }
            }
            Err(e) => {
                eprintln!("GetWindowRect failed: {:?}", e); 
            }
        }
        let title = get_window_title(hwnd);
        let excluded_classes = ["Shell_TrayWnd", "Progman", "ActiveMovie Window"];
        if excluded_classes.contains(&title.as_str()) {
            return;
        }

        if title.is_empty() {
            return; 
        }

        match event {
            EVENT_OBJECT_CREATE =>{
                println!("new window opened: {}, title: {}", event, title);  
            }

            EVENT_OBJECT_DESTROY =>{
                println!("new window destroyed: {}, title: {}", event, title); 
            }
            
            _=> {}
        }
    }
}

pub fn start_hook() {
     
    unsafe {
        SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_DESTROY,
            None,
            Some(win_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
    };
}

