use std::ops::BitAnd;

use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use windows::Win32::Foundation::{
    GetLastError, BOOL, ERROR_APP_HANG, HANDLE, HWND, LPARAM, WAIT_FAILED, WAIT_TIMEOUT, WPARAM,
};

use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetAncestor, GetClassNameA, GetWindowLongPtrA, GetWindowThreadProcessId,
    SendNotifyMessageA, GA_ROOT, GWL_STYLE, WM_CLOSE, WM_DESTROY, WM_QUIT, WS_DISABLED,
};

use crate::logf;

/// Used to tell [`Process::try_exit()`] which exit-method to try on a process.
pub enum ExitMethod {
    /// Sends [`WM_CLOSE`]
    Close,

    /// Sends [`WM_DESTROY`]
    Destroy,

    /// Sends [`WM_QUIT`]
    Quit,
}

impl std::fmt::Display for ExitMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitMethod::Close => write!(f, "Close"),
            ExitMethod::Destroy => write!(f, "Destroy"),
            ExitMethod::Quit => write!(f, "Quit"),
        }
    }
}

impl ExitMethod {
    /// Returns the Win32 window-message associated with this [`ExitMethod`].
    #[must_use]
    pub fn to_wm(&self) -> u32 {
        match self {
            ExitMethod::Close => WM_CLOSE,
            ExitMethod::Destroy => WM_DESTROY,
            ExitMethod::Quit => WM_QUIT,
        }
    }
}

pub struct Process {
    id: u32,
    handle: isize,
}

impl Process {
    /// Opens the process with the specified PID and returns a [`Process`].
    ///
    /// # Panics
    ///
    /// This function panics if the internal call to [`OpenProcess()`] returns a [`HANDLE`] of value `0`.
    #[must_use]
    pub fn open(pid: u32) -> Self {
        logf!("Opening process '{}'", pid);
        let handle = unsafe { OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, pid) }
            .unwrap_or_else(|_| {
                panic!(
                    "failed to open target process ({}) (system error {})",
                    pid,
                    unsafe { GetLastError().0 }
                )
            });

        Self {
            id: pid,
            handle: handle.0,
        }
    }

    /// Attempts to exit the process gracefully using the method specified.
    /// If the specified timeout time has passed and the process has not
    /// exited or if it the exit process encountered an error, this function
    /// will return false. If the process exits successfully, it returns true.
    ///
    /// # Panics
    ///
    /// Panics if no window could be found or if a window-message could
    /// not be sent to the target process' top-level window(s), i.e. the
    /// call to [`SendNotifyMessageA()`] fails.
    pub fn try_exit(&mut self, method: &ExitMethod, timeout_ms: u32) -> bool {
        unsafe {
            logf!(
                "Trying to close process' (pid: {}) windows via method '{}'",
                self.id,
                method
            );

            let result = Self::enumerate_window_handles()
                .into_iter()
                .filter(|hwnd| {
                    let mut wnd_proc_id = 0;
                    GetWindowThreadProcessId(HWND(*hwnd), Some(&mut wnd_proc_id));

                    // Ensure window belongs to the target process
                    // and that the window is a top level window
                    // (i.e. its root parent is itself) and that
                    // it isn't a disabled window.
                    wnd_proc_id == self.id()
                        && *hwnd == GetAncestor(HWND(*hwnd), GA_ROOT).0
                        && GetWindowLongPtrA(HWND(*hwnd), GWL_STYLE)
                            .bitand(isize::try_from(WS_DISABLED.0).unwrap())
                            != isize::try_from(WS_DISABLED.0).unwrap()
                })
                .collect::<Vec<isize>>();

            assert!(
                !result.is_empty(),
                "could not find any windows associated with the target process"
            );

            for hwnd in result {
                let mut hwnd_classname = [0u8; 256];

                let hwnd_classname_len = GetClassNameA(HWND(hwnd), &mut hwnd_classname);
                assert!(hwnd_classname_len != 0, "failed to get window class name");

                logf!(
                    "Sending '{}' to window (hwnd: {} [{:08X}]) (class name: {})",
                    method,
                    hwnd,
                    hwnd,
                    String::from_utf8_lossy(
                        &hwnd_classname[0_usize..usize::try_from(hwnd_classname_len).unwrap()]
                    )
                );

                assert!(
                    SendNotifyMessageA(HWND(hwnd), method.to_wm(), WPARAM(0), LPARAM(0)).as_bool(),
                    "failed to send message to window: SendNotifyMessageA() returned false (os error {})",
                    GetLastError().0
                );
            }

            let result = WaitForSingleObject(HANDLE(self.handle()), timeout_ms);

            if result == WAIT_TIMEOUT {
                logf!("Timed out!");
                return false;
            } else if result == WAIT_FAILED {
                logf!("WaitForSingleObject failed, try_exit() will return false");
                return false;
            }

            true
        }
    }

    /// Enumerates all top-level windows and returns a list of their [`HWND`]'s as a [`Vec<isize>`].
    #[must_use]
    fn enumerate_window_handles() -> Vec<isize> {
        let mut result: Vec<isize> = Vec::new();

        unsafe {
            EnumWindows(
                Some(Self::enum_windows_cb),
                LPARAM(std::ptr::addr_of_mut!(result) as isize),
            );
        }

        result
    }

    unsafe extern "system" fn enum_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out_result: *mut Vec<isize> = lparam.0 as *mut Vec<isize>;

        out_result.as_mut().unwrap().push(hwnd.0);

        BOOL(i32::from(true))
    }

    /// Terminates the `self` process.
    ///
    /// # Panics
    ///
    /// This method panics if the internal call to [`TerminateProcess()`] returns ´false´.
    pub fn terminate(&self) {
        logf!("Terminating process (pid: {})", self.id);

        let success = unsafe { TerminateProcess(HANDLE(self.handle), ERROR_APP_HANG.0).as_bool() };

        assert!(
            success,
            "failed to terminate target process ({}) (system error {})",
            self.id(),
            unsafe { GetLastError().0 }
        );
    }

    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn handle(&self) -> isize {
        self.handle
    }
}
