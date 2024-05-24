use windows::Win32::System::ProcessStatus::GetModuleFileNameExA;
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_QUERY_INFORMATION,
    PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use windows::Win32::Foundation::{
    GetLastError, ERROR_APP_HANG, HANDLE, HWND, LPARAM, LUID, WAIT_FAILED, WAIT_TIMEOUT, WPARAM,
};

use windows::Win32::UI::WindowsAndMessaging::{SendNotifyMessageA, WM_CLOSE, WM_DESTROY, WM_QUIT};

use crate::logf;
use crate::window::Window;

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
    valid: bool, // true if xterminate has exited or terminated the process
}

impl Process {
    /// Opens the process with the specified PID and returns a [`Process`].
    ///
    /// # Panics
    ///
    /// This function panics if the internal call to [`OpenProcess()`] returns a [`HANDLE`] of value `0`.
    #[must_use]
    pub fn open(pid: u32) -> Self {
        let handle = unsafe {
            OpenProcess(
                PROCESS_TERMINATE
                    | PROCESS_SYNCHRONIZE
                    | PROCESS_QUERY_INFORMATION
                    | PROCESS_TERMINATE,
                false,
                pid,
            )
        }
        .unwrap_or_else(|_| {
            panic!("failed to open target process ({}) [{}]", pid, unsafe {
                GetLastError().unwrap_err()
            })
        });

        // Retreive privileges required to access
        //  - PROCESS_QUERY_INFORMATION for GetModuleFileNameExA
        //  - PROCESS_TERMINATE for TerminateProcess

        Self {
            id: pid,
            handle: handle.0,
            valid: true,
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
                "Trying to close process' {} windows via method '{}'",
                self,
                method
            );

            let result = Window::windows()
                .into_iter()
                .filter(|window| {
                    // Ensure window belongs to the target process
                    // and that the window is a top level window
                    // (i.e. its root parent is itself) and that
                    // it isn't a disabled window.
                    window.process().id() == self.id() && window.is_root() && !window.is_disabled()
                })
                .collect::<Vec<Window>>();

            assert!(
                !result.is_empty(),
                "could not find any windows associated with target process"
            );

            for window in result {
                logf!("Sending '{}' to window {}", method, window);

                assert!(
                    SendNotifyMessageA(HWND(window.handle()), method.to_wm(), WPARAM(0), LPARAM(0))
                        .is_ok(),
                    "failed to send message to window {}: SendNotifyMessageA() returned false [{}]",
                    window,
                    GetLastError().unwrap_err()
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

            // Process exited successfully and no longer exists
            self.valid = false;

            true
        }
    }

    /// Terminates the `self` process.
    ///
    /// # Panics
    ///
    /// This method panics if the internal call to [`TerminateProcess()`] returns ´false´.
    pub fn terminate(&mut self) {
        logf!("Terminating process {}", self);

        if !self.valid {
            logf!(
                "ERROR: unable to terminate process [{}] as it is no longer valid",
                self.id()
            );

            return;
        }

        let success = unsafe { TerminateProcess(HANDLE(self.handle), ERROR_APP_HANG.0).is_ok() };

        assert!(success, "failed to terminate process [{}]", unsafe {
            GetLastError().unwrap_err()
        });

        // Process terminated successfully and is no longer valid
        self.valid = false;
    }

    /// Returns the abnsolute path to the process executable.
    ///
    /// # Panics
    /// This method panics if the call to retrieve the process name results
    /// in an empty buffer of utf-8 characters.
    #[must_use]
    pub fn path(&self) -> String {
        if !self.valid {
            return "invalid process handle".to_owned();
        }

        let mut buffer = [0u8; 256];
        let exe_path = PSTR::from_raw(buffer.as_mut_ptr());
        let mut process_name_length = 256;
        unsafe {
            QueryFullProcessImageNameA(
                HANDLE(self.handle()),
                PROCESS_NAME_FORMAT(0),
                exe_path,
                &mut process_name_length,
            )
            .expect("failed to get process path");
        }

        assert!(
            process_name_length > 0,
            "failed to get path for process ({}) [{}]",
            self.id(),
            unsafe { GetLastError().unwrap_err() }
        );

        std::str::from_utf8(&buffer[..process_name_length as usize])
            .unwrap()
            .to_owned()
    }

    /// Returns the name of the process executable (including its extension).
    ///
    /// # Panics
    ///
    /// This method panics if the call to retrieve the process name results
    /// in an empty buffer of utf-8 characters.
    #[must_use]
    pub fn name(&self) -> String {
        if !self.valid {
            return "invalid process handle".to_owned();
        }

        // Strips the absolute path and keeps only the executable filename
        let path = self.path();

        let last = path
            .rfind('\\')
            .or_else(|| path.rfind('/').or(Some(path.len())))
            .unwrap();

        path[last + 1..path.len()].to_owned()
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

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("name", &self.name())
            .field("id", &self.id)
            .field("handle", &format_args!("0x{0:08X}", self.handle))
            .field("valid", &self.valid)
            .finish()
    }
}

impl std::fmt::Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (pid: {})", self.name(), self.id())
    }
}
