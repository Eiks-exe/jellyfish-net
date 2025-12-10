use windows::Win32::{
    Foundation::HWND,
    UI::{
        WindowsAndMessaging::{
            GetWindowTextLengthW, GetWindowTextW
        },
    },
};
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
