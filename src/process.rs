use windows::Win32::System::Threading::{ OpenProcess, TerminateProcess, PROCESS_TERMINATE };
use windows::Win32::Foundation::{ HANDLE, ERROR_APP_HANG, GetLastError };

pub struct Process {
    id: u32,
    handle: isize
}

impl Process {
    /// Opens the process with the specified PID and returns a [Process].
    /// 
    /// ## Panics
    /// 
    /// This function panics if the internal call to `OpenProcess()` returns a [HANDLE] of value `0`.
    pub fn open(pid : u32) -> Self {
        let handle = unsafe {
            OpenProcess(PROCESS_TERMINATE, false, pid)
        }.expect(format!("failed to open target process ({}) (system error {})", pid, unsafe { GetLastError().0 }).as_str());

        Self {
            id: pid,
            handle: handle.0
        }
    }

    /// Terminates the `self` process.
    /// 
    /// ## Panics
    /// 
    /// This method panics if the internal call to `TerminateProcess()` returns ´false´.
    pub fn terminate(&self) {
        let success = unsafe { 
            TerminateProcess(HANDLE { 0: self.handle }, ERROR_APP_HANG.0).as_bool()
        };

        if !success {
            panic!("{}", format!("failed to terminate target process ({}) (system error {})", self.id(), unsafe { GetLastError().0 }));
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle(&self) -> isize {
        self.handle
    }

}