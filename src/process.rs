use windows::core::{PCWSTR, PSTR};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
};

use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameA, TerminateProcess,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    PROCESS_VM_READ,
};

use windows::Win32::Foundation::{GetLastError, ERROR_APP_HANG, HANDLE, LUID};

use crate::logf;

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
        let mut token_handle = HANDLE(0);

        unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES,
                &mut token_handle,
            )
            .expect("failed to open process token");

            // Can't find any privilege specifically for terminating so
            // I'll just use SE_DEBUG_NAME since that one gives all of them.

            let mut luid = LUID::default();
            LookupPrivilegeValueW(PCWSTR(std::ptr::null()), SE_DEBUG_NAME, &mut luid)
                .expect("failed to lookup privilege value");

            let mut token_privileges = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                ..TOKEN_PRIVILEGES::default()
            };

            token_privileges.Privileges[0].Luid = luid;
            token_privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

            AdjustTokenPrivileges(
                token_handle,
                false,
                Some(&token_privileges),
                u32::try_from(std::mem::size_of::<TOKEN_PRIVILEGES>()).unwrap(),
                None,
                None,
            )
            .expect("failed to adjust token privileges");
        }

        let handle = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE
                    | PROCESS_VM_READ
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

        Self {
            id: pid,
            handle: handle.0,
            valid: true,
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
