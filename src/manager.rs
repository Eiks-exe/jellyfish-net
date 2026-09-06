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

#[derive(Clone, Debug)]
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

    /// Remove window by handle
    pub fn remove(&self, handle: isize) {
        let mut info = self.w_info.lock().unwrap();
        let mut links = self.links.lock().unwrap();
        let mut head = self.head.lock().unwrap();
        let mut tail = self.tail.lock().unwrap();
        let mut current = self.current.lock().unwrap();

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
        if let Some(cur) = *current{
            if handle == cur {
                *current = None; 
            }
        }
        info.remove(&handle);

        println!("[-] Removed window handle {}", handle);
    }

    /// Ordered snapshot of tracked windows, head -> tail. Lock order: w_info -> links -> head.
    pub fn snapshot_ordered(&self) -> Vec<(isize, Jfnindow)> {
        let info = self.w_info.lock().unwrap();
        let links = self.links.lock().unwrap();
        let mut cur = *self.head.lock().unwrap();
        let mut result = Vec::new();

        while let Some(handle) = cur {
            if let Some(w) = info.get(&handle) {
                result.push((handle, w.clone()));
            }
            cur = links.get(&handle).and_then(|l| l.next);
        }
        result
    }

    /// Print windows in linked-list order
    pub fn enum_windows(&self) {
        for (_, w) in self.snapshot_ordered() {
            println!("{}", w.title);
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

    pub fn get_current(&self) -> Option<isize> {
        *self.current.lock().unwrap()
    }

    /// Writes `current` directly, bypassing cycle()'s foreground-based branching. Used by the
    /// hold-to-preview commit path once the user has picked a target and it's already been focused --
    /// there's no foreground check left to do at that point. Lock order: current only.
    pub fn set_current(&self, handle: Option<isize>) {
        *self.current.lock().unwrap() = handle;
    }

    /// Read-only mirror of cycle()'s decision logic: the handle cycle() would move `current` to right
    /// now, without actually mutating `current`. Used to seed a hold-to-preview session so its first
    /// step honors the same foreground-resync semantics as an instant tap; subsequent steps within the
    /// same hold should use `peek_after` instead, since the real foreground doesn't change until commit.
    /// Lock order: current -> links (matches cycle()).
    pub fn peek_next(&self) -> Option<isize> {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let fg = unsafe { GetForegroundWindow() };
        let fg_handle = fg.0 as isize;
        let current = *self.current.lock().unwrap();
        let links = self.links.lock().unwrap();

        if let Some(cur) = current {
            if fg_handle == cur {
                if let Some(next) = links.get(&cur).and_then(|l| l.next) {
                    return Some(next);
                }
                // cur is the tail -> fall through and wrap to head, same as cycle().
            } else if links.contains_key(&fg_handle) {
                return Some(fg_handle);
            } else {
                return Some(cur);
            }
        }
        drop(links);
        self.get_head()
    }

    /// The handle `steps` positions after `from` in the linked list, wrapping tail -> head.
    /// If `from` is `None` (or untracked), counts from `head`. Pure: does not touch `current`.
    /// Lock order: links -> head.
    pub fn peek_after(&self, from: Option<isize>, steps: usize) -> Option<isize> {
        let links = self.links.lock().unwrap();
        let head = *self.head.lock().unwrap();

        let mut cur = from.or(head)?;
        for _ in 0..steps {
            cur = links.get(&cur).and_then(|l| l.next).or(head)?;
        }
        Some(cur)
    }

    /// Swap two adjacent nodes where `a` immediately precedes `b`. Lock order: links -> head -> tail.
    fn swap_adjacent(&self, a: isize, b: isize) {
        let mut links = self.links.lock().unwrap();
        let mut head = self.head.lock().unwrap();
        let mut tail = self.tail.lock().unwrap();

        let p = links.get(&a).and_then(|l| l.prev);
        let n = links.get(&b).and_then(|l| l.next);

        links.insert(b, Link { prev: p, next: Some(a) });
        links.insert(a, Link { prev: Some(b), next: n });

        match p {
            Some(p_handle) => {
                if let Some(l) = links.get_mut(&p_handle) {
                    l.next = Some(b);
                }
            }
            None => *head = Some(b),
        }
        match n {
            Some(n_handle) => {
                if let Some(l) = links.get_mut(&n_handle) {
                    l.prev = Some(a);
                }
            }
            None => *tail = Some(a),
        }
    }

    /// Move `handle` one position toward the head, swapping it with its previous neighbor.
    /// Returns false (no-op) if untracked or already head.
    pub fn move_up(&self, handle: isize) -> bool {
        let prev = self.links.lock().unwrap().get(&handle).and_then(|l| l.prev);
        match prev {
            Some(prev) => {
                self.swap_adjacent(prev, handle);
                true
            }
            None => false,
        }
    }

    /// Move `handle` one position toward the tail. Equivalent to moving its successor up (the same
    /// pointer swap, just named from the other node's perspective), so it just delegates to move_up.
    /// Returns false (no-op) if untracked or already tail.
    pub fn move_down(&self, handle: isize) -> bool {
        let next = self.links.lock().unwrap().get(&handle).and_then(|l| l.next);
        match next {
            Some(next) => self.move_up(next),
            None => false,
        }
    }
}
