use windows::Win32::Foundation::{ POINT, HWND };
use windows::Win32::UI::WindowsAndMessaging::{ WindowFromPoint, GetWindowThreadProcessId };

use crate::process::Process;

pub struct Window {
    handle: isize
}

impl Window {
    /// Returns the [Window] located at a specific `x` and `y` screen coordinate
    /// or `None` if no window is located at the specified coordinate.
    pub fn from_point(x: i32, y: i32) -> Option<Self> {
        let hwnd = unsafe { 
            WindowFromPoint(POINT { x: x, y: y }) 
        };

        if hwnd.0 == 0 {
            return None;
        }

        Some(Self {
            handle: hwnd.0
        })
    }

    /// Returns the handle of this [Window]
    pub fn handle(&self) -> isize {
        self.handle
    }

    /// Retrieves and returns the [Process] associated with this [Window].
    pub fn process(&self) -> Process {
        let mut pid = u32::default();

        unsafe {
            GetWindowThreadProcessId(HWND { 0: self.handle }, &mut pid) 
        }; 

        Process::open(pid)
    }
}