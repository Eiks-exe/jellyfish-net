use crate::HWND;
#[derive(Clone, Debug)]
pub struct Jfnindow {
    pub title: String,
    pub handle: HWND,
    pub _pid: u32,
    pub _tid: u32,
}
