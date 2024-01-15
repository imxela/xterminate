use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use windows::Win32::Foundation::{
    GetLastError, BOOL, ERROR_APP_HANG, HANDLE, HWND, LPARAM, WAIT_FAILED, WAIT_TIMEOUT, WPARAM,
};

use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetWindowThreadProcessId, SendNotifyMessageA, WM_CLOSE,
};

use crate::logf;

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

    /// Attempts to exit the process gracefully by sending a `WM_CLOSE`
    /// message to all associated windows. Returns true if the process
    /// exits or, false if it fails or if the timeout is exceeded.
    ///
    /// # Panics
    ///
    /// Panics if this process has an unexpectedly large process ID (PID)
    /// too large to fit inside a [`u32`].
    pub fn try_exit(&mut self, timeout_ms: u32) -> bool {
        unsafe {
            logf!(
                "Trying to close process' (pid: {}) windows gracefully",
                self.id
            );

            EnumWindows(
                Some(Self::enumerate_windows_cb),
                LPARAM(isize::try_from(self.id()).expect("PID is unexpectedly large")),
            );

            let result = WaitForSingleObject(HANDLE(self.handle()), timeout_ms);

            if result == WAIT_TIMEOUT {
                logf!("Timed out!");
                return false;
            } else if result == WAIT_FAILED {
                logf!("WaitForSingleObject failed, try_exit() will return false");
                // panic!("failed to wait for process exit: WaitForSingleObject returned WAIT_FAILED (os error {})", GetLastError().0);
                return false;
            }

            true
        }
    }

    unsafe extern "system" fn enumerate_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut wnd_process_id = 0;

        GetWindowThreadProcessId(hwnd, Some(&mut wnd_process_id));

        if wnd_process_id == u32::try_from(lparam.0).expect("PID is unexpectedly large") {
            EnumChildWindows(hwnd, Some(Self::enumerate_child_windows_cb), LPARAM(0));

            logf!("Sending WM_CLOSE to window (hwnd: {})", hwnd.0);

            assert!(
                SendNotifyMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).as_bool(),
                "failed to send WM_CLOSE message to window: SendNotifyMessageA() returned false (os error {})",
                GetLastError().0
            );
        }

        // No matching process was found
        BOOL(i32::from(true))
    }

    unsafe extern "system" fn enumerate_child_windows_cb(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        logf!("Sending WM_CLOSE to child window (hwnd: {})", hwnd.0);

        assert!(
            SendNotifyMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).as_bool(),
            "failed to send WM_CLOSE message to window: SendNotifyMessageA() returned false (os error {})",
            GetLastError().0
        );

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
