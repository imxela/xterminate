use std::ops::BitAnd;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT};

use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetAncestor, GetClassNameA, GetForegroundWindow, GetWindowLongPtrA,
    GetWindowThreadProcessId, WindowFromPoint, GA_ROOT, GWL_STYLE, WS_DISABLED,
};

use crate::process::Process;

pub struct Window {
    handle: isize,
}

impl Window {
    /// Turns a window handle ([HWND]) into a [Window] structure.
    #[must_use]
    pub fn from_handle(handle: isize) -> Self {
        Self { handle }
    }

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

    /// Returns this [Window]'s class name.
    ///
    /// # Panics
    ///
    /// This function will panic if [`GetClassNameA()`] returns 0.
    #[must_use]
    pub fn class_name(&self) -> String {
        // Class name max size is 256 including nul.
        let mut window_class_name = [0u8; 255];

        let class_name_len = unsafe { GetClassNameA(HWND(self.handle()), &mut window_class_name) };

        assert!(class_name_len != 0, "failed to get window class name");

        String::from_utf8_lossy(
            &window_class_name[0_usize..usize::try_from(class_name_len).unwrap()],
        )
        .to_string()
    }

    /// Returns true if this [Window] is a top-level window, i.e. its top-most
    /// (root) ancestor is itself.
    #[must_use]
    pub fn is_root(&self) -> bool {
        unsafe { self.handle() == GetAncestor(HWND(self.handle()), GA_ROOT).0 }
    }

    /// Returns true if this [Window] is disabled, i.e. it has the
    /// [`WS_DISABLED`] window-style.
    ///
    /// # Panics
    ///
    /// Will panic if the Win32 API's [`WS_DISABLED`] flag is greater than the
    /// max length of [`isize`].
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        unsafe {
            GetWindowLongPtrA(HWND(self.handle()), GWL_STYLE)
                .bitand(isize::try_from(WS_DISABLED.0).unwrap())
                == isize::try_from(WS_DISABLED.0).unwrap()
        }
    }

    /// Returns a [Vec] of all top-level [Window]s.
    #[must_use]
    pub fn windows() -> Vec<Self> {
        let mut result: Vec<isize> = Vec::new();

        unsafe {
            EnumWindows(
                Some(Self::enum_windows_cb),
                LPARAM(std::ptr::addr_of_mut!(result) as isize),
            );
        }

        result.iter().map(|hwnd| Self::from_handle(*hwnd)).collect()
    }

    unsafe extern "system" fn enum_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out_result: *mut Vec<isize> = lparam.0 as *mut Vec<isize>;

        out_result.as_mut().unwrap().push(hwnd.0);

        BOOL(i32::from(true))
    }
}
