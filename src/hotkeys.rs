use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use tray_icon::menu::MenuEvent;

pub trait HotKeyListener {
    fn register(&self, id: i32, modifiers: u32, key: u32) -> Result<(), Box<dyn Error>>;
    fn listen(&self);
    fn add_action(&self, id: i32, callback: Box<dyn FnMut() + Send + 'static>) -> Result<(), &str>;
}

pub struct HotKeyAction {
    pub id: i32,
    pub callback: Box<dyn FnMut() + Send + 'static>,
}

pub struct WindowsHotKeyListener {
    pub actions: Arc<Mutex<Vec<HotKeyAction>>>,
}

impl Default for WindowsHotKeyListener {
    fn default() -> Self {
        Self {
            actions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl HotKeyListener for WindowsHotKeyListener {
    fn register(&self, id: i32, modifier: u32, key: u32) -> Result<(), Box<dyn Error>> {
        use windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
        use windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS;
        use windows::Win32::Foundation::HWND;

        unsafe { RegisterHotKey(HWND(0), id, HOT_KEY_MODIFIERS(modifier), key)? };
        Ok(())
     }

    fn listen(&self) {
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
        };
        use windows::Win32::Foundation::HWND;

        unsafe {
            let mut msg = MSG::default();

            while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id() == "1" {
                        println!("quitting JellyfishNet...");
                        std::process::exit(0);
                    } else if event.id() == "2" {
                        crate::show_management_window();
                    }
                }
                if msg.message == WM_HOTKEY {
                    let id = msg.wParam.0 as i32;
                    self.trigger_action(id);
                }

                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn add_action(
        &self,
        id: i32,
        callback: Box<dyn FnMut() + Send + 'static>,
    ) -> Result<(), &str> {
        let mut guard = self.actions.lock().unwrap();

        if guard.iter().any(|a| a.id == id) {
            return Err("action already exists");
        }

        guard.push(HotKeyAction { id, callback });
        Ok(())
    }
}

impl WindowsHotKeyListener {
    fn trigger_action(&self, id: i32) {
        let mut guard = self.actions.lock().unwrap();

        if let Some(action) = guard.iter_mut().find(|a| a.id == id) {
            (action.callback)();
        }
    }
}
