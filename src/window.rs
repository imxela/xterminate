use windows::Win32::Foundation::{HWND, POINT};

use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, WindowFromPoint,
};

use crate::process::Process;

pub struct Window {
    handle: isize,
}

impl Window {
    /// Returns the [Window] located at a specific `x` and `y` screen coordinate
    /// or `None` if no window is located at the specified coordinate.
    #[must_use]
    pub fn from_point(x: i32, y: i32) -> Option<Self> {
        let hwnd = unsafe { WindowFromPoint(POINT { x, y }) };

        if hwnd.0 == 0 {
            return None;
        }

        Some(Self { handle: hwnd.0 })
    }

    /// Returns the currently focused [Window] or `None` if there is no
    /// valid window in the foreground.
    #[must_use]
    pub fn from_foreground() -> Option<Self> {
        let hwnd = unsafe { GetForegroundWindow() };

        if hwnd.0 == 0 {
            return None;
        }

        Some(Self { handle: hwnd.0 })
    }

    /// Returns the handle of this [Window]
    #[must_use]
    pub fn handle(&self) -> isize {
        self.handle
    }

    /// Retrieves and returns the [Process] associated with this [Window].
    #[must_use]
    pub fn process(&self) -> Process {
        let mut pid = u32::default();

        unsafe { GetWindowThreadProcessId(HWND(self.handle), Some(&mut pid)) };

        Process::open(pid)
    }
}
