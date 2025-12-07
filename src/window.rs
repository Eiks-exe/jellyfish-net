use crate::HWND;
#[derive(Clone)]
pub struct Jfnindow {
    pub title: String,
    pub handle: HWND,
    pub _pid: u32,
    pub _tid:u32,
}


