use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use crate::window::Jfnindow;

#[derive(Copy, Clone, Debug)]
pub struct Link {
    pub prev: Option<isize>,
    pub next: Option<isize>,
}

#[derive(Clone)]
pub struct WindowManager {
    // TID → Window
    pub w_info: Arc<Mutex<HashMap<isize, Jfnindow>>>,
    // TID → linked list pointers
    pub links: Arc<Mutex<HashMap<isize, Link>>>,
    // First and last item in linked list
    pub head: Arc<Mutex<Option<isize>>>,
    pub tail: Arc<Mutex<Option<isize>>>,
    pub current: Arc<Mutex<Option<isize>>>
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            w_info: Arc::new(Mutex::new(HashMap::new())),
            links: Arc::new(Mutex::new(HashMap::new())),
            head: Arc::new(Mutex::new(None)),
            tail: Arc::new(Mutex::new(None)),
            current: Arc::new(Mutex::new(Option::None)) 
        }
    }

    /// Add a new window at the end of the linked list
    pub fn add(&self, window: Jfnindow) {
        let handle = window.handle.0; 

        let mut info = self.w_info.lock().unwrap();
        let mut links = self.links.lock().unwrap();
        let mut head = self.head.lock().unwrap();
        let mut tail = self.tail.lock().unwrap();

        if info.contains_key(&handle) {
            drop(links);
            drop(head);
            drop(tail);
            drop(info);

            // call remove() safely
            self.remove(handle);
            return;
        }

        info.insert(handle, window);

        match *tail {
            None => {
                // First element
                *head = Some(handle);
                *tail = Some(handle);
                links.insert(
                    handle,
                    Link {
                        prev: None,
                        next: None,
                    },
                );
            }
            Some(old_tail) => {
                // Patch old tail
                links
                    .entry(old_tail)
                    .and_modify(|l| l.next = Some(handle));

                // Insert new tail
                links.insert(
                    handle,
                    Link {
                        prev: Some(old_tail),
                        next: None,
                    },
                );

                *tail = Some(handle);
            }
        }

        println!("[+] Added window handle {}", handle);
    }

    /// Remove window by TID
    pub fn remove(&self, handle: isize) {
        let mut info = self.w_info.lock().unwrap();
        let mut links = self.links.lock().unwrap();
        let mut head = self.head.lock().unwrap();
        let mut tail = self.tail.lock().unwrap();

        if let Some(link) = links.remove(&handle) {
            // Fix prev neighbor
            if let Some(prev_handle) = link.prev {
                if let Some(prev_link) = links.get_mut(&prev_handle) {
                    prev_link.next = link.next;
                }
            } else {
                // Was head
                *head = link.next;
            }

            // Fix next neighbor
            if let Some(next_handle) = link.next {
                if let Some(next_link) = links.get_mut(&next_handle) {
                    next_link.prev = link.prev;
                }
            } else {
                // Was tail
                *tail = link.prev;
            }
        }

        info.remove(&handle);

        println!("[-] Removed window handle {}", handle);
    }

    /// Print windows in linked-list order
    pub fn enum_windows(&self) {
        let info = self.w_info.lock().unwrap();
        let links = self.links.lock().unwrap();
        let mut cur = *self.head.lock().unwrap();

        while let Some(handle) = cur {
            if let Some(w) = info.get(&handle) {
                println!("{}", w.title);
            }
            cur = links.get(&handle).and_then(|l| l.next);
        }
    }
    
    pub fn cycle(&self) -> Option<isize>{
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let fg = unsafe {GetForegroundWindow()};
        let fg_handle = fg.0 as isize;
        let mut current = self.current.lock().unwrap();
        let links = self.links.lock().unwrap();
        if let Some(cur) = *current {
            if fg_handle == cur {
            // Foreground is our current → advance
            if let Some(next) = links.get(&cur).and_then(|l| l.next) {
                *current = Some(next);
                return Some(next);
            }
        }else if links.contains_key(&fg_handle) {
                *current = Some(fg_handle);
                return Some(fg_handle); 
            } else {
                return Some(cur);
            }
        }
        *current = self.get_head();
        self.get_head() 
    }
    
    
    pub fn _get_prev(&self, handle: isize) -> Option<isize> {
        self.links.lock().unwrap().get(&handle).and_then(|l| l.prev)
    }

    pub fn _get_next(&self, handle: isize) -> Option<isize> {
        self.links.lock().unwrap().get(&handle).and_then(|l| l.next)
    }

    pub fn get_head(&self) -> Option<isize> {
        *self.head.lock().unwrap()
    }

    pub fn _get_tail(&self) -> Option<isize> {
        *self.tail.lock().unwrap()
    }
}
