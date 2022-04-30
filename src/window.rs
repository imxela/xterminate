use windows::Win32::Foundation::{ POINT, HWND };
use windows::Win32::UI::WindowsAndMessaging::{ WindowFromPoint, GetWindowThreadProcessId };

use crate::process::Process;

pub struct Window {
    handle: isize
}

impl Window {
    pub fn from_point(x: i32, y: i32) -> Self {
        let hwnd = unsafe { 
            WindowFromPoint(POINT { x: x, y: y }) 
        };

        if hwnd.0 == 0 {
            // Todo: Handle error
        }

        Self {
            handle: hwnd.0
        }
    }

    pub fn handle(&self) -> isize {
        self.handle
    }

    pub fn process(&self) -> Process {
        let mut pid = u32::default();

        unsafe { 
            GetWindowThreadProcessId(HWND { 0: self.handle }, &mut pid) 
        }; 

        Process::open(pid)
    }
}