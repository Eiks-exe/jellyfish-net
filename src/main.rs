mod hook;
mod manager;
mod window;  
mod hotkeys;
mod utils; 
//use hook::start_hook;

use std::{sync::{Arc, Mutex}};
use once_cell::sync::OnceCell;
use windows::Win32::{Foundation::HWND, UI::{Input::KeyboardAndMouse::{MOD_ALT}, WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow}}};

use crate::{hook::start_hook, hotkeys::HotKeyListener, manager::WindowManager, utils::get_window_title, window::Jfnindow};


pub static GLOBAL_MANAGER: OnceCell<Arc<Mutex<WindowManager>>> = OnceCell::new(); 

fn jfn_catch(manager: &WindowManager) {
    let focused_window_handle = unsafe {GetForegroundWindow()};
    let focused_window_title= get_window_title(focused_window_handle);
    let mut pid = 0u32;
    let tid = unsafe {
        GetWindowThreadProcessId(focused_window_handle, Some(&mut pid))
    };

    let new_window = Jfnindow {
        handle: focused_window_handle,
        title: focused_window_title,
        _pid: pid,
        _tid: tid, 
    };
    manager.add(new_window);
    manager.enum_windows();
}

fn cycle(manager: &WindowManager) {
    if let Some(next_handle) = manager.cycle() {
        let guard = manager.w_info.lock().unwrap();
        if let Some(win) = guard.get(&next_handle)  {
            println!("-> {}", win.title);
            unsafe {SetForegroundWindow(win.handle)};
        } 
    } else {
        eprintln!("no next window to cycle to")
    }
}
fn init_manager() {
    let manager = Arc::new(Mutex::new(WindowManager::new())); 
    GLOBAL_MANAGER.set(manager.clone()).expect("could not set manager");     
}

fn main() { 
    println!("start");
    init_manager();
    start_hook();

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

    listener.add_action(2, {
        Box::new(
            move || {
                if let Some(mgr) = GLOBAL_MANAGER.get() {
                    let manager = mgr.lock().unwrap();
                    cycle(&manager);
                }
                
            }
        )
    }).expect("could not add action");

    listener.register(1, MOD_ALT.0 , 222).expect("could not register the hotkey...");
    listener.register(2, MOD_ALT.0, 65).expect("could not register the hotkey..."); 
    println!("listening for hotkeys"); 
    listener.listen();
}

